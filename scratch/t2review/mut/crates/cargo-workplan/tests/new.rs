// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `change new` through the built binary: the record it writes and the one file
//! it writes, the text it prints (the library's), the SPDX lines, the exit 2
//! of every refusal, `-q`, and the printed template placed in a capability of
//! each state.

mod support;

use std::fs;
use std::path::PathBuf;

use support::{Repo, Run, array, changed, member_names, strings};

const RECORD: &str = "work/changes/fresh-change.md";

/// What `change new fresh-change --capability cap:alpha --capability cap:beta`
/// prints for the roadmap repository: `alpha` is delivered and `beta`
/// implementing.
const PRINTED: &str = "\
created work/changes/fresh-change.md
changes: cap:alpha, cap:beta
affects: cap:alpha#contract, cap:beta#contract

Place this block in a section named in `affects`, in the document of a listed capability, and replace its second line with the proposed text:

> [!PROPOSED change:fresh-change]
> State the proposed text here.

State of the listed capabilities:
  cap:alpha  delivered  set `state: implementing` in the same commit as the block, since in this state a PROPOSED block is a finding of `check` (`state-delivered-proposal`) and so is the open record itself (`state-delivered-open-record`)
  cap:beta  implementing  no change of state is needed
";

/// The record that command writes.
const WRITTEN: &str = "\
---
type: Change
id: change:fresh-change
title: Name this change
description: Say in one line what this change does to the capabilities it lists.
changes: [cap:alpha, cap:beta]
affects:
  - cap:alpha#contract
  - cap:beta#contract
contract_effect: changes
items: []
---

## Summary

Describe what this change does and why, in a short paragraph.
";

const NEW: [&str; 7] = [
    "change",
    "new",
    "fresh-change",
    "--capability",
    "cap:alpha",
    "--capability",
    "cap:beta",
];

fn with(extra: &[&str]) -> Vec<String> {
    NEW.iter().chain(extra).map(ToString::to_string).collect()
}

fn run_new(repo: &Repo, extra: &[&str]) -> Run {
    let args = with(extra);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    repo.run(&args)
}

#[test]
fn new_prints_the_library_text_writes_the_record_and_nothing_else_in_both_shapes() {
    let repo = Repo::synced("new-one", false);
    let before = repo.snapshot();
    let args = with(&[]);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let run = repo.both_writing(&args).expect(0);
    assert_eq!(run.stdout, PRINTED);
    assert_eq!(run.stderr, "");
    assert_eq!(changed(&before, &repo.snapshot()), [RECORD]);
    assert_eq!(repo.read(RECORD), WRITTEN);

    // The record loads: the commands that read it find it.
    let impact = repo.both(&["change", "impact", "fresh-change"]).expect(0);
    assert!(
        impact.stdout.starts_with("change:fresh-change\n"),
        "{impact:#?}"
    );
}

#[test]
fn a_second_run_for_the_same_slug_is_exit_two_and_leaves_the_record_as_it_is() {
    let repo = Repo::synced("new-twice", false);
    run_new(&repo, &[]).expect(0);
    let before = repo.snapshot();
    let run = run_new(&repo, &[]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr
            .contains("work/changes/fresh-change.md already exists"),
        "{run:#?}"
    );
    assert_eq!(repo.snapshot(), before);
}

#[test]
fn every_refusal_of_the_library_is_exit_two_with_the_tree_untouched() {
    let repo = Repo::synced("new-refused", false);
    let before = repo.snapshot();
    for (args, said) in [
        // A record that exists.
        (
            vec!["change", "new", "grow", "--capability", "cap:beta"],
            "already exists",
        ),
        // A capability that is not one.
        (
            vec!["change", "new", "fresh", "--capability", "cap:nowhere"],
            "`cap:nowhere` is not a capability",
        ),
        (
            vec!["change", "new", "fresh", "--capability", "beta"],
            "capability ids read `cap:<name>`",
        ),
        // An `affects` entry that is refused.
        (
            vec![
                "change",
                "new",
                "fresh",
                "--capability",
                "cap:beta",
                "--affects",
                "cap:beta#nowhere",
            ],
            "`nowhere` is not a section",
        ),
        (
            vec![
                "change",
                "new",
                "fresh",
                "--capability",
                "cap:beta",
                "--affects",
                "cap:alpha#contract",
            ],
            "the capability is not one the change lists",
        ),
        (
            vec![
                "change",
                "new",
                "fresh",
                "--capability",
                "cap:beta",
                "--affects",
                "contract",
            ],
            "must read `cap:<name>#<section>`",
        ),
    ] {
        let run = repo.run(&args).expect(2);
        assert_eq!(run.stdout, "", "{args:?}");
        assert!(run.stderr.contains(said), "{args:?}: {run:#?}");
        assert_eq!(repo.snapshot(), before, "{args:?} wrote");
    }
    // The neighbour of each: the same shapes with a value that is allowed.
    run_new(&repo, &["--affects", "cap:beta#usage"]).expect(0);
}

#[test]
fn a_capability_that_is_missing_or_a_bad_option_is_a_usage_error_before_anything_is_read() {
    let repo = Repo::synced("new-usage", false);
    let before = repo.snapshot();
    for args in [
        // No capability.
        vec!["change", "new", "fresh"],
        // A slug that is not one.
        vec!["change", "new", "Fresh", "--capability", "cap:beta"],
        vec!["change", "new", "-rf", "--capability", "cap:beta"],
        // No slug.
        vec!["change", "new", "--capability", "cap:beta"],
        // A value for no option.
        vec!["change", "new", "fresh", "--capability"],
        vec![
            "change",
            "new",
            "fresh",
            "--capability",
            "cap:beta",
            "--bogus",
        ],
        // One SPDX line without the other.
        vec![
            "change",
            "new",
            "fresh",
            "--capability",
            "cap:beta",
            "--copyright",
            "2026 X",
        ],
        vec![
            "change",
            "new",
            "fresh",
            "--capability",
            "cap:beta",
            "--licence",
            "CC-BY-4.0",
        ],
    ] {
        let run = repo.both(&args).expect(2);
        assert_eq!(run.stdout, "", "{args:?}");
        assert!(!run.stderr.is_empty(), "{args:?}");
        assert_eq!(repo.snapshot(), before, "{args:?} wrote");
    }
}

#[test]
fn affects_is_repeatable_and_the_record_holds_the_entries_given_in_order() {
    let repo = Repo::synced("new-affects", false);
    let run = run_new(
        &repo,
        &[
            "--affects",
            "cap:beta#usage",
            "--affects",
            "cap:alpha#purpose",
            "--affects",
            "cap:beta#usage",
        ],
    )
    .expect(0);
    assert!(
        run.stdout
            .contains("affects: cap:beta#usage, cap:alpha#purpose\n"),
        "{run:#?}"
    );
    assert!(
        repo.read(RECORD)
            .contains("affects:\n  - cap:beta#usage\n  - cap:alpha#purpose\ncontract_effect"),
        "{}",
        repo.read(RECORD)
    );
    // `impact` then reports those sections, sorted by capability.
    let impact = repo.run(&["change", "impact", "fresh-change"]).expect(0);
    assert!(
        impact
            .stdout
            .contains("sections:\n  cap:alpha#purpose\n  cap:beta#usage\n"),
        "{impact:#?}"
    );
}

#[test]
fn a_capability_given_twice_is_listed_once() {
    let repo = Repo::synced("new-twice-listed", false);
    repo.run(&[
        "change",
        "new",
        "fresh-change",
        "--capability",
        "cap:beta",
        "--capability",
        "cap:beta",
    ])
    .expect(0);
    assert!(repo.read(RECORD).contains("changes: [cap:beta]\n"));
}

/// The first two lines of the frontmatter comment of the change records of
/// this repository, which `make reuse` accepts: the copyright and the licence.
fn repository_header() -> (String, String, [String; 2]) {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/changes");
    let mut paths: Vec<PathBuf> = fs::read_dir(&directory)
        .unwrap_or_else(|e| panic!("cannot list {}: {e}", directory.display()))
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    paths.sort();
    let first = paths.first().expect("the repository has a change record");
    let text = fs::read_to_string(first).expect("a record");
    let lines: Vec<&str> = text.lines().take(3).collect();
    assert_eq!(lines[0], "---");
    let copyright = lines[1]
        .strip_prefix(concat!("# SPDX-", "FileCopyrightText: "))
        .expect("a copyright line");
    let licence = lines[2]
        .strip_prefix(concat!("# SPDX-", "License-Identifier: "))
        .expect("a licence line");
    (
        copyright.to_string(),
        licence.to_string(),
        [lines[1].to_string(), lines[2].to_string()],
    )
}

#[test]
fn the_spdx_lines_are_the_two_comment_lines_of_the_records_of_this_repository() {
    let (copyright, licence, header) = repository_header();
    let repo = Repo::synced("new-spdx", false);
    let before = repo.snapshot();
    let run = run_new(&repo, &["--copyright", &copyright, "--licence", &licence]).expect(0);
    assert_eq!(run.stdout, PRINTED);
    assert_eq!(changed(&before, &repo.snapshot()), [RECORD]);

    // The same two lines, directly after the opening fence, as in the records
    // `make reuse` accepts.
    let record = repo.read(RECORD);
    let lines: Vec<&str> = record.lines().collect();
    assert_eq!(lines[0], "---");
    assert_eq!(lines[1], header[0]);
    assert_eq!(lines[2], header[1]);
    // The rest is the record without them.
    let rest: Vec<&str> = lines[3..].to_vec();
    let plain: Vec<&str> = WRITTEN.lines().skip(1).collect();
    assert_eq!(rest, plain);
    // And it loads.
    repo.run(&["change", "impact", "fresh-change"]).expect(0);

    // Without the options no line is written: the record opens as before.
    let bare = Repo::synced("new-no-spdx", false);
    run_new(&bare, &[]).expect(0);
    assert!(!bare.read(RECORD).contains("SPDX"));
}

#[test]
fn spdx_text_that_is_not_one_line_is_exit_two_and_writes_nothing() {
    let repo = Repo::synced("new-spdx-bad", false);
    let before = repo.snapshot();
    for (copyright, licence, said) in [
        ("two\nlines", "CC-BY-4.0", "the copyright"),
        ("2026 X", "CC-BY-4.0\rmore", "the licence"),
        ("   ", "CC-BY-4.0", "the copyright"),
    ] {
        let run = run_new(&repo, &["--copyright", copyright, "--licence", licence]).expect(2);
        assert_eq!(run.stdout, "");
        assert!(run.stderr.contains(said), "{run:#?}");
        assert_eq!(repo.snapshot(), before);
    }
}

#[test]
fn quiet_writes_the_record_and_prints_nothing() {
    let repo = Repo::synced("new-quiet", false);
    let run = repo
        .run(&[
            "-q",
            "change",
            "new",
            "fresh-change",
            "--capability",
            "cap:beta",
        ])
        .expect(0);
    assert_eq!(run.stdout, "");
    assert_eq!(run.stderr, "");
    assert!(repo.read(RECORD).contains("changes: [cap:beta]"));
    // A refusal is still said under `-q`.
    let refused = repo
        .run(&[
            "--quiet",
            "change",
            "new",
            "fresh-change",
            "--capability",
            "cap:beta",
        ])
        .expect(2);
    assert!(refused.stderr.contains("already exists"), "{refused:#?}");
}

#[test]
fn json_is_one_new_change_object_with_its_fields() {
    let repo = Repo::synced("new-json", false);
    let args = with(&["--affects", "cap:beta#usage"]);
    let mut json = vec!["--format", "json"];
    json.extend(args.iter().map(String::as_str));
    let run = repo.run(&json).expect(0);
    let objects = array(&run);
    assert_eq!(objects.len(), 1, "{run:#?}");
    assert_eq!(
        member_names(&run),
        [strings(&[
            "kind",
            "slug",
            "path",
            "capabilities",
            "affects",
            "template",
            "guidance",
            "sync_needed"
        ])]
    );
    let object = &objects[0];
    assert_eq!(object["kind"], "new-change");
    assert_eq!(object["slug"], "fresh-change");
    assert_eq!(object["path"], RECORD);
    assert_eq!(
        object["capabilities"],
        serde_json::json!(["cap:alpha", "cap:beta"])
    );
    assert_eq!(object["affects"], serde_json::json!(["cap:beta#usage"]));
    assert_eq!(
        object["template"],
        "> [!PROPOSED change:fresh-change]\n> State the proposed text here.\n"
    );
    assert_eq!(
        object["guidance"],
        serde_json::json!([
            {"advice": "set-implementing", "capability": "cap:alpha", "state": "delivered"},
            {"advice": "keep", "capability": "cap:beta", "state": "implementing"},
        ])
    );
    assert_eq!(object["sync_needed"], false);
}

#[test]
fn the_note_about_the_generated_blocks_is_there_when_the_record_makes_them_stale() {
    // A capability with the generated relationship table lists its open
    // changes, so a record that lists it makes the table stale.
    let repo = Repo::synced("new-sync-needed", false);
    repo.edit(
        "docs/capabilities/beta.md",
        "Text for Verification of beta.\n",
        &format!("Text for Verification of beta.\n{}", support::BLOCKS),
    );
    repo.sync();
    repo.run(&["check"]).expect(0);
    let run = repo
        .run(&["change", "new", "fresh-change", "--capability", "cap:beta"])
        .expect(0);
    assert!(
        run.stdout.ends_with(
            "\nThe record makes generated blocks stale. Stage the record first, since a link to a file git does not track is a finding of `check`, then bring the blocks up to date:\n  git add work/changes/fresh-change.md\n  docs sync\n"
        ),
        "{run:#?}"
    );
    repo.run(&["check"]).expect(1);
    let sync = repo.run(&["docs", "sync"]).expect(0);
    assert_eq!(sync.stdout, "rewrote docs/capabilities/beta.md\n");
    repo.run(&["check"]).expect(0);

    // The target file was already stale: the note is still there.
    let stale = Repo::synced("new-sync-needed-stale", false);
    stale.edit(
        "docs/capabilities/beta.md",
        "Text for Verification of beta.\n",
        &format!("Text for Verification of beta.\n{}", support::BLOCKS),
    );
    stale.run(&["check"]).expect(1);
    let run = stale
        .run(&["change", "new", "fresh-change", "--capability", "cap:beta"])
        .expect(0);
    assert!(
        run.stdout
            .ends_with("  git add work/changes/fresh-change.md\n  docs sync\n"),
        "{run:#?}"
    );
}

/// The text of the template a run printed: the marker line and the line after.
fn printed_template(run: &Run) -> String {
    let lines: Vec<&str> = run.stdout.lines().collect();
    let at = lines
        .iter()
        .position(|line| line.starts_with("> [!PROPOSED change:"))
        .expect("the printed template");
    format!("{}\n{}\n", lines[at], lines[at + 1])
}

#[test]
fn the_printed_template_in_a_capability_of_each_state_gives_a_clean_check_and_one_unresolved_proposal()
 {
    for (capability, state) in [
        ("alpha", "delivered"),
        ("beta", "implementing"),
        ("gamma", "designed"),
        ("delta", "considered"),
    ] {
        let repo = Repo::synced("new-template", false);
        // `reconcile` compares with the default branch of a checkout.
        repo.checkout();
        let id = format!("cap:{capability}");
        let run = repo
            .run(&["change", "new", "fresh-change", "--capability", &id])
            .expect(0);
        assert!(
            run.stdout.contains(&format!("  {id}  {state}  ")),
            "{capability}: {run:#?}"
        );

        // Follow the guidance: the block, and the state when it says so.
        let path = format!("docs/capabilities/{capability}.md");
        let anchor = format!("Text for Contract of {capability}.\n");
        repo.edit(
            &path,
            &anchor,
            &format!("{anchor}\n{}", printed_template(&run)),
        );
        let says_implementing = run.stdout.contains("set `state: implementing`");
        assert_eq!(
            says_implementing,
            matches!(state, "delivered" | "designed"),
            "{capability}"
        );
        if says_implementing {
            repo.edit(&path, &format!("state: {state}\n"), "state: implementing\n");
        }

        // The generated blocks are brought up to date, as the output says.
        repo.run(&["docs", "sync"]).expect(0);
        let check = repo.both(&["check"]).expect(0);
        assert_eq!(check.stdout, "", "{capability}");

        let owed = repo
            .both(&["change", "reconcile", "fresh-change"])
            .expect(1);
        let proposals: Vec<&str> = owed
            .lines()
            .into_iter()
            .filter(|line| line.contains("  unresolved-proposal  "))
            .collect();
        assert_eq!(proposals.len(), 1, "{capability}: {owed:#?}");
        assert!(
            proposals[0].starts_with(&format!("{id}  contract  unresolved-proposal  line ")),
            "{capability}: {}",
            proposals[0]
        );
    }
}

#[test]
fn the_template_left_in_a_state_that_refuses_it_is_the_finding_the_guidance_names() {
    // The guidance for a delivered capability names both findings; leaving the
    // state as it is gives them.
    let repo = Repo::synced("new-template-wrong", false);
    let run = repo
        .run(&["change", "new", "fresh-change", "--capability", "cap:alpha"])
        .expect(0);
    repo.edit(
        "docs/capabilities/alpha.md",
        "Text for Contract of alpha.\n",
        &format!("Text for Contract of alpha.\n\n{}", printed_template(&run)),
    );
    let check = repo.run(&["check"]).expect(1);
    assert!(
        check.stdout.contains("[state-delivered-proposal]"),
        "{check:#?}"
    );
    assert!(
        check.stdout.contains("[state-delivered-open-record]"),
        "{check:#?}"
    );
}

#[test]
fn new_works_from_a_subdirectory_and_by_root() {
    let repo = Repo::synced("new-root", false);
    let run = support::run_in(
        &repo.root().join("src"),
        support::Shape::Cargo,
        &["change", "new", "from-below", "--capability", "cap:beta"],
    )
    .expect(0);
    assert!(
        run.stdout
            .starts_with("created work/changes/from-below.md\n")
    );
    let root = repo.root().to_string_lossy().into_owned();
    let rooted = support::run_in(
        &support::elsewhere(),
        support::Shape::Direct,
        &[
            "--root",
            &root,
            "change",
            "new",
            "by-root",
            "--capability",
            "cap:beta",
        ],
    )
    .expect(0);
    assert!(
        rooted
            .stdout
            .starts_with("created work/changes/by-root.md\n")
    );
    assert!(repo.root().join("work/changes/by-root.md").is_file());
}

#[test]
fn the_printed_steps_end_in_a_clean_check_and_skipping_the_stage_step_does_not() {
    let make = |tag: &str| {
        let repo = Repo::synced(tag, false);
        repo.edit(
            "docs/capabilities/beta.md",
            "Text for Verification of beta.\n",
            &format!("Text for Verification of beta.\n{}", support::BLOCKS),
        );
        repo.sync();
        repo.checkout();
        repo.run(&["check"]).expect(0);
        repo
    };

    // Only the printed steps, in order: stage the record, then sync.
    let repo = make("new-steps");
    let run = repo
        .run(&["change", "new", "fresh-change", "--capability", "cap:beta"])
        .expect(0);
    let steps: Vec<&str> = run
        .stdout
        .lines()
        .skip_while(|line| !line.starts_with("The record makes generated blocks stale."))
        .skip(1)
        .collect();
    assert_eq!(
        steps,
        ["  git add work/changes/fresh-change.md", "  docs sync"]
    );
    let staged: Vec<&str> = steps[0].trim().split(' ').skip(1).collect();
    repo.git(&staged);
    let words: Vec<&str> = steps[1].trim().split(' ').collect();
    repo.run(&words).expect(0);
    repo.run(&["check"]).expect(0);

    // The neighbour: the sync alone leaves the table linking to a file that git
    // does not track.
    let repo = make("new-steps-unstaged");
    repo.run(&["change", "new", "fresh-change", "--capability", "cap:beta"])
        .expect(0);
    repo.run(&["docs", "sync"]).expect(0);
    let check = repo.run(&["check"]).expect(1);
    assert!(check.stdout.contains("[link-broken]"), "{check:#?}");
}
