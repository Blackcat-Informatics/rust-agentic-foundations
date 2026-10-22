// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `change complete` through the built binary and the real `git` and `cargo`:
//! the refusal that lists every failure and writes nothing (exit 1), the
//! completion that rewrites the generated blocks and removes the record
//! (exit 0), what stops it looking (exit 2), `-q`, the JSON objects, and the
//! whole path from `change new` to a completed change.

mod support;

use raf_workplan::exec::Programs;
use raf_workplan::lifecycle::Lifecycle;
use serde_json::{Value, json};
use support::{BLOCKS, Repo, array, capability, change, changed, member_names, strings};

const RECORD: &str = "work/changes/shape.md";
const CAP: &str = "docs/capabilities/fixture.md";

const PASSES: &str = "[demo::tests::passes]";
const GOOD: &str = "[demo::example_crate::good]";

/// An item of the record that has its outcome.
const DONE: &str = "items:\n  - claim: \"fixture is shaped\"\n    outcome: implemented";

/// A checkout of a workspace with the capability `fixture` in `state`, bound
/// as given, and the change `shape` that lists it with `items`. The contract
/// of the capability is edited after the fork, so its section is touched.
fn shaped(tag: &str, state: &str, tests: &str, examples: &str, items: &str) -> Repo {
    let repo = Repo::real(tag);
    repo.put(
        CAP,
        &capability(
            "fixture",
            state,
            ["[demo::add]", tests, examples],
            "[]",
            &[("Verification", BLOCKS)],
        ),
    );
    repo.put(
        RECORD,
        &change("shape", "[cap:fixture]", &["cap:fixture#contract"]).replace("items: []", items),
    );
    repo.sync();
    repo.checkout();
    repo.edit(
        CAP,
        "Text for Contract of fixture.",
        "Fixture states its contract here.",
    );
    repo.sync();
    // The contract as it now reads is assessed.
    repo.assess("shape");
    repo
}

/// A delivered capability and a resolved record: `complete` completes.
fn ready(tag: &str) -> Repo {
    shaped(tag, "delivered", PASSES, GOOD, DONE)
}

/// The text the library gives for a refusal of the same repository.
fn library(repo: &Repo, slug: &str) -> String {
    Lifecycle::open(Programs::new(repo.root()))
        .expect("the lifecycle opens")
        .complete(slug)
        .expect("the gate gives an answer")
        .to_string()
}

#[test]
fn a_capability_left_implementing_is_refused_and_nothing_is_written() {
    let repo = shaped("complete-implementing", "implementing", PASSES, GOOD, DONE);
    let before = repo.worktree();
    let run = repo.run(&["change", "complete", "shape"]).expect(1);
    assert_eq!(
        run.lines(),
        [
            "refused: 1 finding once the record is removed",
            "once the record is removed, docs/capabilities/fixture.md: `state`: state implementing requires an open change record that lists cap:fixture [no-open-record]; list the capability in the `changes` of an open change record, or move the state to delivered",
        ]
    );
    assert_eq!(run.stderr, "");
    assert_eq!(repo.worktree(), before, "a refusal wrote");
    assert!(repo.root().join(RECORD).is_file());
    // The text is the library's.
    assert_eq!(run.stdout, library(&repo, "shape"));
    assert_eq!(repo.both(&["change", "complete", "shape"]), run);

    // The neighbour: the author sets the state and it completes.
    repo.edit(CAP, "state: implementing", "state: delivered");
    repo.sync();
    repo.run(&["change", "complete", "shape"]).expect(0);
}

#[test]
fn a_completed_change_prints_what_was_removed_and_rewritten_and_leaves_a_clean_check() {
    let repo = ready("complete-done");
    repo.run(&["check"]).expect(1);
    let before = repo.worktree();
    let run = repo.run(&["change", "complete", "shape"]).expect(0);
    assert_eq!(
        run.stdout,
        "completed: removed work/changes/shape.md\nrewrote docs/capabilities/fixture.md\n"
    );
    assert_eq!(run.stderr, "");
    // The record is gone, the blocks it staled are rewritten, and nothing else
    // moved.
    assert!(!repo.root().join(RECORD).exists());
    assert_eq!(changed(&before, &repo.worktree()), [CAP, RECORD]);
    assert!(!repo.read(CAP).contains("work/changes/shape.md"));
    repo.both(&["check"]).expect(0);
    repo.git(&["add", "-A"]);
    // The capability and the removed record, beside the assessment the
    // fixture recorded before the run, which `complete` leaves as it is.
    let mut status: Vec<String> = repo
        .git(&["status", "--short"])
        .lines()
        .map(str::to_string)
        .collect();
    status.sort();
    assert_eq!(
        status,
        [
            "A  work/assessments/fixture-contract.md".to_string(),
            format!("D  {RECORD}"),
            format!("M  {CAP}"),
        ]
    );

    // A second run finds no record.
    let again = repo.run(&["change", "complete", "shape"]).expect(2);
    assert!(
        again.stderr.contains("no change record `shape`"),
        "{again:#?}"
    );
}

#[test]
fn both_shapes_complete_the_same_way() {
    let repo = ready("complete-shapes");
    let run = repo
        .both_writing(&["change", "complete", "shape"])
        .expect(0);
    assert!(
        run.stdout
            .starts_with("completed: removed work/changes/shape.md\n")
    );
}

#[test]
fn every_failure_is_listed_and_each_condition_is_named_in_json() {
    // An item with no outcome is an obligation (condition 1) and a failure of
    // condition 2; a capability left implementing is reported once the record
    // is removed. The bound test fails too, but a static failure spends no
    // process on `cargo`, so it is not run and not reported.
    let repo = shaped(
        "complete-many",
        "implementing",
        "[demo::tests::fails]",
        GOOD,
        "items:\n  - claim: \"fixture is shaped\"",
    );
    let before = repo.worktree();
    let run = repo.run(&["change", "complete", "shape"]).expect(1);
    assert_eq!(
        run.lines()[0],
        "refused: 1 obligation, 1 item without outcome, 1 finding once the record is removed"
    );
    assert!(run.stdout.contains("item-without-outcome"));
    assert!(!run.stdout.contains("demo::tests::fails"));
    assert!(run.stdout.contains("[no-open-record]"));
    assert_eq!(repo.worktree(), before);

    let json = repo
        .run(&["--format", "json", "change", "complete", "shape"])
        .expect(1);
    let objects = array(&json);
    assert_eq!(objects[0]["kind"], "refused");
    assert_eq!(objects[0]["failures"], 3);
    assert_eq!(
        objects[0]["message"],
        "refused: 1 obligation, 1 item without outcome, 1 finding once the record is removed"
    );
    let names = member_names(&json);
    assert_eq!(names[0], strings(&["kind", "failures", "message"]));
    let refusals: Vec<&Value> = objects[1..].iter().collect();
    for (object, names) in refusals.iter().zip(&names[1..]) {
        assert_eq!(object["kind"], "refusal");
        assert_eq!(
            *names,
            strings(&[
                "kind",
                "condition",
                "number",
                "source",
                "message",
                "repeated"
            ])
        );
    }
    let summary: Vec<(String, Option<u64>, String, bool)> = refusals
        .iter()
        .map(|object| {
            (
                object["condition"].as_str().expect("condition").to_string(),
                object["number"].as_u64(),
                object["source"].as_str().expect("source").to_string(),
                object["repeated"].as_bool().expect("repeated"),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            (
                "obligations".to_string(),
                Some(1),
                "item-without-outcome".to_string(),
                false
            ),
            (
                "outcomes".to_string(),
                Some(2),
                "item-without-outcome".to_string(),
                true
            ),
            (
                "after-removal".to_string(),
                None,
                "no-open-record".to_string(),
                false
            ),
        ]
    );
    // The lines the text form prints are those that no other line repeats.
    let printed: Vec<&str> = refusals
        .iter()
        .filter(|object| object["repeated"] == false)
        .map(|object| object["message"].as_str().expect("message"))
        .collect();
    assert_eq!(&run.lines()[1..], printed.as_slice());
}

#[test]
fn a_failing_example_stands_under_conditions_one_and_five() {
    let repo = shaped(
        "complete-example",
        "delivered",
        PASSES,
        "[demo::example_crate::exiting]",
        DONE,
    );
    let run = repo.run(&["change", "complete", "shape"]).expect(1);
    assert_eq!(
        run.lines()[0],
        "refused: 1 obligation, 1 bound example failing"
    );
    assert!(
        run.lines()[1].starts_with("cap:fixture  examples  example-failing  "),
        "{run:#?}"
    );
    assert_eq!(run.lines().len(), 2, "{run:#?}");
    let json = repo
        .run(&["--format", "json", "change", "complete", "shape"])
        .expect(1);
    let sources: Vec<(Value, Value)> = array(&json)[1..]
        .iter()
        .map(|object| (object["number"].clone(), object["source"].clone()))
        .collect();
    assert_eq!(
        sources,
        [
            (json!(1), json!("example-failing")),
            (json!(5), json!("bound-example"))
        ]
    );
    assert!(repo.root().join(RECORD).is_file());
    // Once the example builds and runs the change completes.
    repo.put("examples/exiting.rs", support::GOOD_EXAMPLE);
    repo.run(&["change", "complete", "shape"]).expect(0);
}

#[test]
fn a_failing_bound_test_is_condition_five_and_the_record_stays() {
    let repo = shaped(
        "complete-test",
        "delivered",
        "[demo::tests::fails]",
        GOOD,
        DONE,
    );
    let before = repo.worktree();
    let run = repo.run(&["change", "complete", "shape"]).expect(1);
    assert_eq!(run.lines()[0], "refused: 1 bound test failing");
    assert!(
        run.lines()[1].starts_with("fail cap:fixture tests `demo::tests::fails`: "),
        "{run:#?}"
    );
    assert_eq!(run.lines().len(), 2);
    assert_eq!(repo.worktree(), before);
    let json = repo
        .run(&["--format", "json", "change", "complete", "shape"])
        .expect(1);
    let refusal = &array(&json)[1];
    assert_eq!(refusal["condition"], "bound-runs");
    assert_eq!(refusal["number"], 5);
    assert_eq!(refusal["source"], "bound-test");
    // The neighbour: the test that passes.
    repo.edit(CAP, "[demo::tests::fails]", PASSES);
    repo.sync();
    repo.run(&["change", "complete", "shape"]).expect(0);
}

#[test]
fn a_stale_generated_block_is_refused_until_the_sync_is_run() {
    let repo = shaped("complete-stale", "implementing", PASSES, GOOD, DONE);
    // The catalogue lists the capability under the state it is in.
    repo.edit(CAP, "state: implementing", "state: delivered");
    let run = repo.run(&["change", "complete", "shape"]).expect(1);
    assert!(
        run.lines()[0].starts_with("refused: 1 obligation, 1 stale generated block"),
        "{run:#?}"
    );
    assert!(run.stdout.contains("generated-stale"), "{run:#?}");
    assert!(repo.root().join(RECORD).is_file());
    repo.run(&["docs", "sync"]).expect(0);
    repo.run(&["change", "complete", "shape"]).expect(0);
}

#[test]
fn what_stops_it_looking_is_exit_two_and_the_record_stays() {
    let repo = ready("complete-two");
    let before = repo.worktree();

    let unknown = repo.both(&["change", "complete", "nowhere"]).expect(2);
    assert_eq!(unknown.stdout, "");
    assert!(
        unknown.stderr.contains("no change record `nowhere`"),
        "{unknown:#?}"
    );

    // An outcome that is not one: the record does not load.
    repo.edit(RECORD, "outcome: implemented", "outcome: bogus");
    let bad = repo.both(&["change", "complete", "shape"]).expect(2);
    assert_eq!(bad.stdout, "");
    assert!(bad.stderr.contains("did not load"), "{bad:#?}");
    repo.edit(RECORD, "outcome: bogus", "outcome: implemented");
    assert_eq!(repo.worktree(), before);

    // No default branch to compare with.
    repo.git(&["symbolic-ref", "--delete", "refs/remotes/origin/HEAD"]);
    let headless = repo.both(&["change", "complete", "shape"]).expect(2);
    assert_eq!(headless.stdout, "");
    assert!(
        headless
            .stderr
            .contains("git remote set-head origin --auto"),
        "{headless:#?}"
    );
    assert!(repo.root().join(RECORD).is_file());
    assert_eq!(repo.worktree(), before);
}

#[test]
fn quiet_hides_a_completion_and_keeps_a_refusal() {
    let refused = shaped("complete-quiet-refused", "implementing", PASSES, GOOD, DONE);
    let loud = refused.run(&["change", "complete", "shape"]).expect(1);
    let quiet = refused
        .both(&["-q", "change", "complete", "shape"])
        .expect(1);
    assert_eq!(quiet.stdout, loud.stdout);

    let done = ready("complete-quiet-done");
    let run = done
        .run(&["--quiet", "change", "complete", "shape"])
        .expect(0);
    assert_eq!((run.stdout.as_str(), run.stderr.as_str()), ("", ""));
    assert!(!done.root().join(RECORD).exists());
}

#[test]
fn json_for_a_completion_is_a_completed_object_and_a_rewrote_object_for_each_file() {
    let repo = ready("complete-json");
    let run = repo
        .run(&["--format", "json", "change", "complete", "shape"])
        .expect(0);
    assert_eq!(
        array(&run),
        [
            json!({"kind": "completed", "path": RECORD}),
            json!({"kind": "rewrote", "path": CAP}),
        ]
    );
    assert_eq!(
        member_names(&run),
        [strings(&["kind", "path"]), strings(&["kind", "path"])]
    );
}

/// The contract settled and not assessed owes its assessment alone; the
/// assessor then records it, through the library.
fn assess_the_contract(repo: &Repo) {
    let unassessed = repo.run(&["change", "reconcile", "shape"]).expect(1);
    assert_eq!(
        unassessed.lines(),
        [
            "cap:fixture  contract  assessment-missing  an assessment of the section recorded with assess (`cap:fixture#contract` has no assessment `assessment:fixture-contract`)"
        ],
        "{unassessed:#?}"
    );
    assert_eq!(
        repo.assess("shape"),
        ["work/assessments/fixture-contract.md"]
    );
}

#[test]
fn from_new_to_a_completed_change_through_the_binary() {
    let repo = Repo::real("complete-flow");
    repo.put(
        CAP,
        &capability(
            "fixture",
            "implementing",
            ["[demo::add]", PASSES, GOOD],
            "[]",
            &[("Verification", BLOCKS)],
        ),
    );
    repo.sync();
    repo.checkout();

    // Open the change; the output says what to do, and a record is one file.
    let before = repo.worktree();
    let created = repo
        .run(&[
            "change",
            "new",
            "shape",
            "--capability",
            "cap:fixture",
            "--copyright",
            "2026 Example Org <dev@example.org>",
            "--licence",
            "CC-BY-4.0",
        ])
        .expect(0);
    assert!(
        created
            .stdout
            .starts_with("created work/changes/shape.md\n")
    );
    assert!(created.stdout.contains("no change of state is needed"));
    assert_eq!(changed(&before, &repo.worktree()), [RECORD]);
    assert!(repo.read(RECORD).starts_with(&format!(
        "---\n# {}2026 Example Org <dev@example.org>\n# {}CC-BY-4.0\ntype: Change\n",
        concat!("SPDX-", "FileCopyrightText: "),
        concat!("SPDX-", "License-Identifier: "),
    )));
    // The output names two steps and nothing else is done here: stage the
    // record, then sync. Each step is read from the printed text.
    let steps: Vec<&str> = created
        .stdout
        .lines()
        .skip_while(|line| !line.starts_with("The record makes generated blocks stale."))
        .skip(1)
        .collect();
    assert_eq!(
        steps,
        ["  git add work/changes/shape.md", "  docs sync"],
        "{created:#?}"
    );
    repo.run(&["check"]).expect(1);
    let staged: Vec<&str> = steps[0].trim().split(' ').skip(1).collect();
    repo.git(&staged);
    let words: Vec<&str> = steps[1].trim().split(' ').collect();
    repo.run(&words).expect(0);
    repo.run(&["check"]).expect(0);

    // Edit: the proposed text goes into the contract as a PROPOSED block.
    repo.edit(
        CAP,
        "Text for Contract of fixture.\n",
        "Text for Contract of fixture.\n\n> [!PROPOSED change:shape]\n> Fixture does it.\n",
    );
    repo.run(&["check"]).expect(0);
    let owed = repo.run(&["change", "reconcile", "shape"]).expect(1);
    assert_eq!(owed.lines().len(), 2, "{owed:#?}");
    assert!(owed.stdout.contains("  unresolved-proposal  "), "{owed:#?}");
    assert!(owed.stdout.contains("  assessment-missing  "), "{owed:#?}");
    let refused = repo.run(&["change", "complete", "shape"]).expect(1);
    assert!(refused.stdout.starts_with("refused: "), "{refused:#?}");

    // The block becomes the contract, and once the contract is assessed
    // nothing is owed.
    repo.edit(
        CAP,
        "\n> [!PROPOSED change:shape]\n> Fixture does it.\n",
        "Fixture does it.\n",
    );
    assess_the_contract(&repo);
    let empty = repo.both(&["change", "reconcile", "shape"]).expect(0);
    assert_eq!(empty.stdout, "");

    // The author sets the state; the catalogue then needs a sync, and the gate
    // says so until it is run.
    repo.edit(CAP, "state: implementing", "state: delivered");
    let stale = repo.run(&["change", "complete", "shape"]).expect(1);
    assert!(stale.stdout.contains("generated-stale"), "{stale:#?}");
    repo.run(&["docs", "sync"]).expect(0);
    repo.both(&["change", "reconcile", "shape"]).expect(0);
    repo.run(&["verify", "shape"]).expect(0);

    let before = repo.worktree();
    let done = repo.run(&["change", "complete", "shape"]).expect(0);
    assert_eq!(
        done.stdout,
        "completed: removed work/changes/shape.md\nrewrote docs/capabilities/fixture.md\n"
    );
    assert!(!repo.root().join(RECORD).exists());
    assert_eq!(changed(&before, &repo.worktree()), [CAP, RECORD]);
    let check = repo.run(&["check"]).expect(0);
    assert_eq!(check.stdout, "");
}

/// `--strict` judges the repository once the record is removed under strict
/// mode: the delivered capability of the fixture holds one sentence in each
/// section, so the completion is refused with the findings of strict mode and
/// nothing is written, while `change complete` without the flag completes the
/// same change. `--strict change reconcile` owes what it owes without the flag,
/// since strict mode adds no obligation the fixture owes.
#[test]
fn strict_refuses_a_completion_that_the_default_gate_accepts() {
    let repo = ready("complete-strict");
    let owed = repo.run(&["change", "reconcile", "shape"]);
    assert_eq!(
        repo.run(&["--strict", "change", "reconcile", "shape"]),
        owed
    );
    let before = repo.worktree();
    let run = repo
        .run(&["--strict", "change", "complete", "shape"])
        .expect(1);
    assert!(
        run.stdout.starts_with("refused: ") && run.stdout.contains("[strict-section-thin]"),
        "{run:#?}"
    );
    assert_eq!(repo.worktree(), before, "a refusal writes nothing");
    let run = repo.run(&["change", "complete", "shape"]).expect(0);
    assert!(run.stdout.starts_with("completed: "), "{run:#?}");
}
