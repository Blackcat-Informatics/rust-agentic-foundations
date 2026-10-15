// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `change reconcile` through the built binary: one line for each obligation
//! in the shape of the contract, the exit 1 of a list that is not empty and the
//! exit 0 of one that is, the exit 2 of everything that stops it looking, `-q`,
//! the JSON objects, and that it writes nothing in the repository.

mod support;

use std::path::Path;

use raf_workplan::exec::Programs;
use raf_workplan::lifecycle::Lifecycle;
use serde_json::json;
use support::{Repo, Run, array, change, member_names, strings};

/// The list `change reconcile grow` prints for the roadmap repository on a
/// branch that has not edited a document.
const GROW: &str = "\
cap:beta  contract  section-untouched  a diff of that section against the merge base, or an edit that records why none is needed
cap:beta  verification  binding-missing  a binding that resolves through the source index (bindings.examples is empty)
cap:beta  verification  binding-missing  a binding that resolves through the source index (bindings.tests is empty)
cap:delta  contract  unresolved-proposal  line 26: the block is replaced by plain contract text, or removed because the claim was dropped
cap:delta  contract  section-untouched  a diff of that section against the merge base, or an edit that records why none is needed
cap:gamma  contract  section-untouched  a diff of that section against the merge base, or an edit that records why none is needed
";

/// The roadmap repository as a git checkout on a branch of its own.
fn checkout(tag: &str) -> Repo {
    let repo = Repo::synced(tag, false);
    repo.checkout();
    repo
}

/// The same repository with a change `calm` that lists the delivered `alpha`,
/// declares its contract unaffected, and owes nothing.
fn calm(tag: &str) -> Repo {
    let repo = Repo::synced(tag, false);
    repo.put(
        "work/changes/calm.md",
        &change("calm", "[cap:alpha]", &["cap:alpha#contract"])
            .replace("contract_effect: changes", "contract_effect: none"),
    );
    repo.sync();
    repo.checkout();
    repo
}

/// The text the library gives for the same repository.
fn library(root: &Path, slug: &str) -> String {
    Lifecycle::open(Programs::new(root))
        .expect("the lifecycle opens")
        .reconcile(slug)
        .expect("the change reconciles")
        .to_string()
}

#[test]
fn a_list_with_obligations_is_printed_one_line_each_and_exits_one() {
    let repo = checkout("reconcile-lines");
    let run = repo.both(&["change", "reconcile", "grow"]).expect(1);
    assert_eq!(run.stdout, GROW);
    assert_eq!(run.stderr, "");
    // The line is the library's: the binary decides nothing.
    assert_eq!(run.stdout, library(repo.root(), "grow"));
    // Deterministic: the same run twice, and from elsewhere by `--root`.
    assert_eq!(repo.run(&["change", "reconcile", "grow"]), run);
    assert_eq!(repo.rooted(&["change", "reconcile", "grow"]), run);
}

#[test]
fn an_empty_list_prints_nothing_and_exits_zero() {
    let repo = calm("reconcile-empty");
    let run = repo.both(&["change", "reconcile", "calm"]).expect(0);
    assert_eq!(run.stdout, "");
    assert_eq!(run.stderr, "");
    assert_eq!(library(repo.root(), "calm"), "");
    let json = repo
        .both(&["--format", "json", "change", "reconcile", "calm"])
        .expect(0);
    assert_eq!(json.stdout, "[]\n");
}

#[test]
fn fixing_an_obligation_removes_its_line_and_only_its_line() {
    let repo = checkout("reconcile-fixed");
    let before: Vec<String> = GROW.lines().map(ToString::to_string).collect();
    // Editing the contract of gamma touches the section.
    repo.edit(
        "docs/capabilities/gamma.md",
        "Text for Contract of gamma.",
        "Gamma states its contract here.",
    );
    repo.sync();
    let run = repo.both(&["change", "reconcile", "grow"]).expect(1);
    let after: Vec<&str> = run.lines();
    let expected: Vec<&String> = before
        .iter()
        .filter(|line| !line.starts_with("cap:gamma  contract  section-untouched"))
        .collect();
    assert_eq!(after, expected);

    // Replacing the PROPOSED block of delta by plain text resolves it, and the
    // edit touches the section.
    repo.edit(
        "docs/capabilities/delta.md",
        "> [!PROPOSED change:grow]\n> Delta reads streams as they arrive.\n>\n> It never buffers a whole document.\n",
        "Delta reads streams as they arrive.\n",
    );
    repo.sync();
    let run = repo.both(&["change", "reconcile", "grow"]).expect(1);
    assert!(!run.stdout.contains("unresolved-proposal"), "{run:#?}");
    assert!(!run.stdout.contains("cap:delta"), "{run:#?}");

    // An edit that is committed on the branch counts the same: the merge base
    // is the fork.
    repo.commit("edit");
    let committed = repo.both(&["change", "reconcile", "grow"]).expect(1);
    assert_eq!(committed, run);
}

#[test]
fn nothing_is_written_and_no_cargo_is_started_for_a_fixture_without_bound_examples() {
    let repo = checkout("reconcile-readonly");
    let before = repo.worktree();
    let root = repo.root().to_string_lossy().into_owned();
    // A cargo that does not exist: a run that started it would be exit 2, and
    // `--root` means discovery does not need it either.
    let mut command = support::command(
        &support::elsewhere(),
        support::Shape::Direct,
        &["--root", &root, "change", "reconcile", "grow"],
    );
    command.env("CARGO", repo.root().join("no-such-cargo"));
    let run = support::finish(&mut command).expect(1);
    assert_eq!(run.stdout, GROW);
    assert_eq!(repo.worktree(), before, "reconcile wrote");
    repo.git(&["diff", "--quiet"]);
}

#[test]
fn a_bound_example_that_fails_is_an_obligation_and_nothing_is_written_to_the_tree() {
    let repo = Repo::real("reconcile-example");
    repo.put(
        "docs/capabilities/fixture.md",
        &support::capability(
            "fixture",
            "implementing",
            [
                "[demo::add]",
                "[demo::tests::passes]",
                "[demo::example_crate::exiting]",
            ],
            "[]",
            &[("Verification", support::BLOCKS)],
        ),
    );
    repo.put(
        "work/changes/shape.md",
        &change("shape", "[cap:fixture]", &["cap:fixture#contract"]),
    );
    repo.sync();
    repo.checkout();
    repo.edit(
        "docs/capabilities/fixture.md",
        "Text for Contract of fixture.",
        "Fixture states its contract here.",
    );
    repo.sync();
    let before = repo.worktree();
    let run = repo.both(&["change", "reconcile", "shape"]).expect(1);
    assert_eq!(run.lines().len(), 1, "{run:#?}");
    assert!(
        run.stdout
            .starts_with("cap:fixture  examples  example-failing  "),
        "{run:#?}"
    );
    assert!(
        run.stdout
            .contains("`demo::example_crate::exiting` fails to build or run (exit code 3)"),
        "{run:#?}"
    );
    assert_eq!(repo.worktree(), before, "reconcile wrote into the tree");

    // Once the example builds and runs the line is gone.
    repo.put("examples/exiting.rs", support::GOOD_EXAMPLE);
    let run = repo.both(&["change", "reconcile", "shape"]).expect(0);
    assert_eq!(run.stdout, "");
}

#[test]
fn everything_that_stops_it_looking_is_exit_two() {
    // A slug with no record.
    let repo = checkout("reconcile-two");
    let run = repo.both(&["change", "reconcile", "nowhere"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.contains("no change record `nowhere`"),
        "{run:#?}"
    );

    // A record that does not load: an outcome that is not one.
    let bad = checkout("reconcile-bad-outcome");
    bad.edit(
        "work/changes/grow.md",
        "items: []",
        "items:\n  - claim: \"x\"\n    outcome: bogus",
    );
    let run = bad.both(&["change", "reconcile", "grow"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("did not load"), "{run:#?}");

    // A repository with no default branch.
    let headless = checkout("reconcile-no-head");
    headless.git(&["symbolic-ref", "--delete", "refs/remotes/origin/HEAD"]);
    let run = headless.both(&["change", "reconcile", "grow"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.contains("git remote set-head origin --auto"),
        "{run:#?}"
    );

    // A directory that is not in a git work tree.
    let plain = Repo::synced("reconcile-no-git", false);
    let run = plain.both(&["change", "reconcile", "grow"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.starts_with("cargo-workplan: "), "{run:#?}");
}

#[test]
fn quiet_keeps_the_obligations_and_hides_nothing_that_is_a_finding() {
    let repo = checkout("reconcile-quiet");
    let loud = repo.run(&["change", "reconcile", "grow"]).expect(1);
    let quiet = repo.both(&["-q", "change", "reconcile", "grow"]).expect(1);
    assert_eq!(quiet.stdout, loud.stdout);
    // The neighbour: an empty list stays empty and exits zero.
    let calm = calm("reconcile-quiet-empty");
    let run = calm
        .both(&["--quiet", "change", "reconcile", "calm"])
        .expect(0);
    assert_eq!(run.stdout, "");
    // A failure is said under `-q`.
    let failed = repo
        .run(&["-q", "change", "reconcile", "nowhere"])
        .expect(2);
    assert!(failed.stderr.contains("no change record"), "{failed:#?}");
}

#[test]
fn json_is_one_obligation_object_for_each_obligation() {
    let repo = checkout("reconcile-json");
    let run = repo
        .both(&["--format", "json", "change", "reconcile", "grow"])
        .expect(1);
    let objects = array(&run);
    assert_eq!(objects.len(), 6, "{run:#?}");
    for names in member_names(&run) {
        assert_eq!(
            names,
            strings(&[
                "kind",
                "obligation",
                "subject",
                "section",
                "path",
                "line",
                "evidence"
            ])
        );
    }
    let text: Run = repo.run(&["change", "reconcile", "grow"]);
    for (object, line) in objects.iter().zip(text.lines()) {
        assert_eq!(object["kind"], "obligation");
        let fields = [
            object["subject"].as_str().expect("subject"),
            object["section"].as_str().expect("section"),
            object["obligation"].as_str().expect("obligation"),
        ];
        assert!(line.starts_with(&fields.join("  ")), "{line} vs {fields:?}");
        assert!(
            line.ends_with(object["evidence"].as_str().expect("evidence")),
            "{line}"
        );
    }
    let proposal = objects
        .iter()
        .find(|object| object["obligation"] == "unresolved-proposal")
        .expect("the proposal");
    assert_eq!(
        *proposal,
        json!({
            "kind": "obligation",
            "obligation": "unresolved-proposal",
            "subject": "cap:delta",
            "section": "contract",
            "path": "docs/capabilities/delta.md",
            "line": 26,
            "evidence": "the block is replaced by plain contract text, or removed because the claim was dropped",
        })
    );
    let untouched = objects
        .iter()
        .find(|object| object["obligation"] == "section-untouched")
        .expect("an untouched section");
    assert_eq!(untouched["line"], serde_json::Value::Null);

    // Under `-q` the findings stay: the same array.
    let quiet = repo
        .run(&["-q", "--format", "json", "change", "reconcile", "grow"])
        .expect(1);
    assert_eq!(quiet.stdout, run.stdout);
}
