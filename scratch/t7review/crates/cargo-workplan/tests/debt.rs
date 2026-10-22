// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `debt --write` through the built binary: exit 0 when the record is written
//! (created, replaced or already equal), 1 when the debt would grow or a
//! finding that no row can hold stands, 2 when the repository is not judged
//! strict or the command is misused, the record the only file written, and the
//! same bytes from both shapes of the command.

mod support;

use support::{Repo, array, changed, member_names, strict, strings};

const DEBT: &str = "work/debt/default.md";

/// The members of the `debt-written` object, in order.
const WRITTEN: [&str; 5] = ["kind", "path", "change", "findings", "lines"];

/// The strict-clean repository with the Examples section made thin, under a
/// coverage record that says strict.
fn thin(tag: &str) -> Repo {
    Repo::strict(tag, &strict::thin("Examples"), "strictness: strict\n")
}

#[test]
fn the_write_creates_the_record_exits_zero_and_the_repository_then_passes() {
    let repo = thin("debt-create");
    let failing = repo.both(&["check"]).expect(1);
    assert!(
        failing.stdout.contains("[strict-section-thin]"),
        "{failing:#?}"
    );
    let before = repo.snapshot();
    let run = repo.both_writing(&["debt", "--write"]).expect(0);
    assert_eq!(
        run.stdout, "created: work/debt/default.md (1 finding(s), 0 item(s) with orphan lines)\n",
        "{run:#?}"
    );
    assert_eq!(run.stderr, "");
    assert_eq!(changed(&before, &repo.snapshot()), [DEBT]);
    assert!(
        repo.read(DEBT).contains("subject: cap:demo#examples"),
        "{}",
        repo.read(DEBT)
    );
    // The repository passes, in both shapes.
    assert_eq!(repo.both(&["check"]).expect(0).stdout, "");
}

#[test]
fn a_second_write_is_unchanged_and_the_bytes_are_the_same() {
    let repo = thin("debt-twice");
    repo.both_writing(&["debt", "--write"]).expect(0);
    let first = repo.snapshot();
    let run = repo.both_writing(&["debt", "--write"]).expect(0);
    assert!(
        run.stdout.starts_with("unchanged: work/debt/default.md"),
        "{run:#?}"
    );
    assert_eq!(repo.snapshot(), first);
}

#[test]
fn the_written_record_in_json_is_one_object_with_its_members_in_order() {
    let repo = thin("debt-json");
    let run = repo
        .both_writing(&["--format", "json", "debt", "--write"])
        .expect(0);
    assert_eq!(
        run.stdout,
        "[\n{\"kind\":\"debt-written\",\"path\":\"work/debt/default.md\",\"change\":\"created\",\"findings\":1,\"lines\":0}\n]\n"
    );
    assert_eq!(member_names(&run), vec![strings(&WRITTEN)]);
    let objects = array(&run);
    assert_eq!(objects[0]["findings"], serde_json::json!(1));
    // An outcome: `-q` hides it.
    let quiet = repo.both(&["-q", "debt", "--write"]).expect(0);
    assert_eq!(quiet.stdout, "");
}

#[test]
fn debt_that_grew_is_exit_one_with_its_findings_and_nothing_is_written() {
    let repo = thin("debt-growth");
    repo.both_writing(&["debt", "--write"]).expect(0);
    repo.edit(
        strict::CAPABILITY,
        &format!("## Explanation\n\n{}\n", strict::PROSE),
        &format!("## Explanation\n\n{}\n", strict::words(19)),
    );
    let before = repo.snapshot();
    let text = repo.both(&["debt", "--write"]).expect(1);
    assert!(
        text.stdout.contains("[strict-section-thin]")
            && text.stdout.contains("section `Explanation`"),
        "{text:#?}"
    );
    assert!(
        text.stderr
            .contains("the debt would grow, so nothing is written"),
        "{text:#?}"
    );
    assert_eq!(repo.snapshot(), before);
    let json = repo
        .both(&["--format", "json", "debt", "--write"])
        .expect(1);
    let objects = array(&json);
    assert_eq!(objects.len(), 1, "{json:#?}");
    assert_eq!(objects[0]["kind"], "strict-section-thin");
    assert_eq!(objects[0]["section"], "Explanation");
    // A finding stays under `-q`.
    assert_eq!(
        repo.both(&["-q", "debt", "--write"]).expect(1).stdout,
        text.stdout
    );
    assert_eq!(repo.snapshot(), before);
}

#[test]
fn paid_debt_fails_the_check_with_debt_resolved_until_the_write_lowers_the_record() {
    let repo = thin("debt-paid");
    repo.both_writing(&["debt", "--write"]).expect(0);
    repo.put(strict::CAPABILITY, &strict::capability());
    repo.sync();
    let check = repo.both(&["check"]).expect(1);
    assert!(
        check.stdout.contains("[debt-resolved]")
            && check.stdout.contains("cargo workplan debt --write"),
        "{check:#?}"
    );
    let run = repo.both_writing(&["debt", "--write"]).expect(0);
    assert!(
        run.stdout
            .starts_with("replaced: work/debt/default.md (0 finding(s)"),
        "{run:#?}"
    );
    assert_eq!(repo.both(&["check"]).expect(0).stdout, "");
}

#[test]
fn a_finding_of_the_default_rules_stops_the_write_with_exit_one() {
    let repo = thin("debt-default-finding");
    repo.edit(
        strict::CAPABILITY,
        "<!-- raf:generated:verification-matrix -->\n",
        "<!-- raf:generated:verification-matrix -->\nstale\n",
    );
    let before = repo.snapshot();
    let run = repo.both(&["debt", "--write"]).expect(1);
    assert!(run.stdout.contains("[generated-stale]"), "{run:#?}");
    assert!(
        run.stderr
            .contains("no debt record can hold stand, so nothing is written"),
        "{run:#?}"
    );
    assert_eq!(repo.snapshot(), before);
}

#[test]
fn a_repository_that_is_not_judged_strict_is_exit_two_and_strict_adopts() {
    let repo = Repo::strict("debt-not-strict", &strict::thin("Examples"), "");
    let before = repo.snapshot();
    let run = repo.both(&["debt", "--write"]).expect(2);
    assert_eq!(run.stdout, "", "{run:#?}");
    assert!(run.stderr.contains("is not judged strict"), "{run:#?}");
    assert_eq!(repo.snapshot(), before);
    let adopted = repo
        .both_writing(&["--strict", "debt", "--write"])
        .expect(0);
    assert!(adopted.stdout.starts_with("created: "), "{adopted:#?}");
}

#[test]
fn orphan_lines_over_the_ceiling_are_recorded_per_item() {
    let repo = Repo::strict("debt-lines", &strict::capability(), "strictness: strict\n");
    repo.edit(
        strict::LIB,
        "pub fn run() {}\n",
        "pub fn run() {}\n\nfn helper() {}\n",
    );
    assert_eq!(repo.both(&["check"]).expect(1).lines().len(), 1);
    let run = repo.both_writing(&["debt", "--write"]).expect(0);
    assert_eq!(
        run.stdout,
        "created: work/debt/default.md (0 finding(s), 1 item(s) with orphan lines)\n"
    );
    assert!(
        repo.read(DEBT)
            .contains("item: demo::helper\n    orphans: 1\n"),
        "{}",
        repo.read(DEBT)
    );
    assert_eq!(repo.both(&["check"]).expect(0).stdout, "");
    // The report names the debt.
    let report = repo.both(&["coverage", "lines"]).expect(1);
    assert!(
        report.stdout.contains("| 0 lines | 1 |"),
        "the debt column: {report:#?}"
    );
    // A second orphan item is growth.
    repo.edit(
        strict::LIB,
        "fn helper() {}\n",
        "fn helper() {}\n\nfn other() {}\n",
    );
    let grown = repo.both(&["debt", "--write"]).expect(1);
    assert!(grown.stdout.contains("`demo::other`"), "{grown:#?}");
}

#[test]
fn the_spdx_lines_are_given_together_and_kept_when_the_record_is_lowered() {
    let repo = thin("debt-spdx");
    let run = repo
        .both_writing(&[
            "debt",
            "--write",
            "--copyright",
            "2026 Example Org <dev@example.org>",
            "--licence",
            "CC-BY-4.0",
        ])
        .expect(0);
    assert!(run.stdout.starts_with("created: "), "{run:#?}");
    assert!(
        repo.read(DEBT).starts_with(
            "---\n# SPDX-FileCopyrightText: 2026 Example Org <dev@example.org>\n# SPDX-License-Identifier: CC-BY-4.0\ntype: Debt\n"
        ),
        "{}",
        repo.read(DEBT)
    );
    repo.put(strict::CAPABILITY, &strict::capability());
    repo.sync();
    repo.both_writing(&["debt", "--write"]).expect(0);
    assert!(
        repo.read(DEBT)
            .starts_with("---\n# SPDX-FileCopyrightText: 2026 Example Org"),
        "{}",
        repo.read(DEBT)
    );
}

#[test]
fn a_command_misused_is_a_usage_error_before_any_work() {
    let repo = thin("debt-usage");
    let before = repo.snapshot();
    for args in [
        &["debt"][..],
        &["debt", "--write", "--copyright", "2026 Example"],
        &["debt", "--write", "--licence", "CC-BY-4.0"],
        &["debt", "--licence", "CC-BY-4.0", "--copyright", "x"],
        &["debt", "--write", "extra"],
    ] {
        let run = repo.both(args).expect(2);
        assert_eq!(run.stdout, "", "{args:?}: {run:#?}");
    }
    assert_eq!(repo.snapshot(), before);
    let bare = Repo::bare("debt-bare");
    let run = bare.rooted(&["debt", "--write"]).expect(2);
    assert_eq!(run.stdout, "");
}
