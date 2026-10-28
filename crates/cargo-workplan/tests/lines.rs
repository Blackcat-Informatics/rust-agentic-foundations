// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `coverage lines` through the built binary: the reverse coverage of every
//! package as three tables or RDF, exit 0 when no orphan line is printed,
//! 1 when one is and 2 for a package that is not one, `coverage --reverse` the
//! same bytes, two runs the same bytes, and nothing written.

mod support;

use raf_workplan::Gate;
use raf_workplan::coverage::render;
use support::{Repo, rdf::Rdf, strict};

/// The report of the strict-clean repository: its documented function and its
/// test are bound, so every counted line is covered.
const CLEAN: &str = "\
| Crate | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| demo | 8 | 8 | 0 | 0 | 0 | 0 | 0 lines | 0 |

| Crate | Module | Counted | Orphan |
| --- | --- | --- | --- |

| Crate | Item | Kind | File | Orphan lines |
| --- | --- | --- | --- | --- |
";

/// The same repository with a private helper that nothing calls.
const ORPHANED: &str = "\
| Crate | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| demo | 9 | 8 | 0 | 1 | 1 | 0 | 0 lines | 0 |

| Crate | Module | Counted | Orphan |
| --- | --- | --- | --- |
| demo | `demo` | 7 | 1 |

| Crate | Item | Kind | File | Orphan lines |
| --- | --- | --- | --- | --- |
| demo | `demo::helper` | fn | src/lib.rs | 8-8 |
";

/// The table the library renders for the repository at `repo`.
fn library_table(repo: &Repo) -> String {
    let opened = Gate::workspace(repo.root())
        .programs(raf_workplan::exec::Programs::new(repo.root()).discover_git())
        .open()
        .expect("a repository");
    render::lines_table(&opened.workplan().lines_report())
}

#[test]
fn a_repository_whose_every_counted_line_is_covered_prints_the_report_and_exits_zero() {
    let repo = Repo::strict("lines-clean", &strict::capability(), "");
    let before = repo.snapshot();
    let run = repo.both(&["coverage", "lines"]).expect(0);
    assert_eq!(run.stdout, CLEAN, "{run:#?}");
    assert_eq!(run.stderr, "");
    assert_eq!(run.stdout, library_table(&repo));
    let json = repo
        .both(&["--format", "turtle", "coverage", "lines"])
        .expect(0);
    let rdf = Rdf::of(&json);
    let rows = rdf.rows("CrateLines");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].required("package"), "demo");
    assert_eq!(rows[0].number("counted"), 8);
    assert_eq!(rows[0].number("covered"), 8);
    assert_eq!(rows[0].number("orphan"), 0);
    assert!(rdf.rows("UnitLines").is_empty());
    // Two runs, the reverse spelling and the package named: the same bytes.
    assert_eq!(repo.both(&["coverage", "lines"]).expect(0), run);
    assert_eq!(repo.both(&["coverage", "--reverse"]).expect(0), run);
    assert_eq!(
        repo.both(&["coverage", "lines", "--crate", "demo"])
            .expect(0),
        run
    );
    // A report with no orphan line is an outcome: `-q` hides it.
    assert_eq!(repo.both(&["-q", "coverage", "lines"]).expect(0).stdout, "");
    assert_eq!(repo.snapshot(), before, "the report wrote");
}

#[test]
fn an_orphan_line_is_a_finding_and_exits_one_in_text_and_rdf_alike_and_reverse_prints_the_same() {
    let repo = Repo::strict("lines-orphan", &strict::capability(), "");
    repo.edit(
        strict::LIB,
        "pub fn run() {}\n",
        "pub fn run() {}\n\nfn helper() {}\n",
    );
    let before = repo.snapshot();
    let text = repo.both(&["coverage", "lines"]).expect(1);
    assert_eq!(text.stdout, ORPHANED, "{text:#?}");
    let reverse = repo.both(&["coverage", "--reverse"]).expect(1);
    assert_eq!(reverse, text);
    // A finding: `-q` keeps it whole.
    assert_eq!(repo.both(&["-q", "coverage", "lines"]).expect(1), text);
    let json = repo
        .both(&["--format", "turtle", "coverage", "lines"])
        .expect(1);
    let rdf = Rdf::of(&json);
    let package = rdf.rows("CrateLines")[0];
    assert_eq!(package.number("orphan"), 1);
    assert_eq!(rdf.rows("ModuleLines")[0].number("counted"), 7);
    let unit = rdf.rows("UnitLines")[0];
    assert_eq!(unit.required("item"), "demo::helper");
    assert_eq!(unit.required("unit"), "fn");
    let range = unit.children("range")[0];
    assert_eq!((range.number("first"), range.number("last")), (8, 8));
    assert_eq!(
        repo.both(&["--format", "turtle", "coverage", "--reverse"])
            .expect(1),
        json
    );
    // The default report and library report enforce the same ceiling.
    let plain = repo.both(&["coverage", "lines"]).expect(1);
    assert!(plain.stdout.contains("| demo | 9 | 8 |"), "{plain:#?}");
    assert_eq!(plain.stdout, library_table(&repo));
    assert_eq!(repo.snapshot(), before, "the report wrote");
}

#[test]
fn a_package_that_is_not_one_exits_two_and_says_why() {
    let repo = Repo::strict("lines-unknown", &strict::capability(), "");
    for args in [
        &["coverage", "lines", "--crate", "ghost"][..],
        &["coverage", "--reverse", "--crate", "ghost"],
    ] {
        let run = repo.both(args).expect(2);
        assert_eq!(run.stdout, "", "{run:#?}");
        assert!(
            run.stderr
                .contains("no package of the workspace is named `ghost`; the packages are demo"),
            "{run:#?}"
        );
    }
}

/// The strict-clean repository with a private helper nothing calls, under the
/// coverage record `record`.
fn with_helper(tag: &str, record: &str) -> Repo {
    let repo = Repo::strict(tag, &strict::capability(), record);
    repo.edit(
        strict::LIB,
        "pub fn run() {}\n",
        "pub fn run() {}\n\nfn helper() {}\n",
    );
    repo
}

/// The `ceilings` lines of a record with one row for `demo`.
fn row(limit: &str) -> String {
    format!("ceilings:\n  - crate: demo\n{limit}")
}

#[test]
fn a_ceiling_row_and_its_class_decide_the_exit_of_the_default_check_on_the_binary() {
    // One shipping orphan line (the helper) and no test orphan line.
    let over = [
        // A shipping row below the count, and an `all` row below it.
        "    lines: 0\n    of: shipping\n",
        "    lines: 0\n",
        // A test row leaves the shipping lines at the default of 0.
        "    lines: 5\n    of: test\n",
    ];
    for (index, limit) in over.iter().enumerate() {
        let repo = with_helper(&format!("lines-over-{index}"), &row(limit));
        let run = repo.both(&["check"]).expect(1);
        assert!(
            run.stdout.contains("[coverage-lines-over-ceiling]"),
            "{limit}: {run:#?}"
        );
        assert!(run.stdout.contains("has 1 orphan line "), "{run:#?}");
        let json = repo.both(&["--format", "turtle", "check"]).expect(1);
        assert!(
            Rdf::of(&json)
                .rows("Finding")
                .iter()
                .any(|f| f.required("kind") == "coverage-lines-over-ceiling")
        );
    }
    // The raised rows pass: the class row at the count, the `all` row at the
    // count and a percent row above the share.
    for (index, limit) in [
        "    lines: 1\n    of: shipping\n",
        "    lines: 1\n    of: all\n",
        "    percent: 50\n",
    ]
    .iter()
    .enumerate()
    {
        let repo = with_helper(&format!("lines-within-{index}"), &row(limit));
        let run = repo.both(&["check"]).expect(0);
        assert!(
            !run.stdout.contains("coverage-lines-over-ceiling"),
            "{run:#?}"
        );
    }
    // The same default policy also enforces the zero ceiling here.
    let repo = with_helper(
        "lines-default",
        "ceilings:\n  - crate: demo\n    lines: 0\n",
    );
    repo.both(&["check"]).expect(1);
}

#[test]
fn an_exception_that_matches_nothing_fails_check_under_default_and_its_matched_neighbour_passes() {
    let matched = "exceptions:\n  - item: demo::helper\n    reason: \"Kept for a caller.\"\n";
    let repo = with_helper("lines-exception-matched", matched);
    repo.both(&["check"]).expect(0);
    let unmatched = format!("{matched}  - item: demo::ghost\n    reason: \"Nothing is there.\"\n");
    let repo = with_helper("lines-exception-unmatched", &unmatched);
    {
        let run = repo.both(&["check"]).expect(1);
        assert!(
            run.stdout.contains("[coverage-exception-unmatched]")
                && run.stdout.contains("`demo::ghost`"),
            "{run:#?}"
        );
        assert!(!run.stdout.contains("`demo::helper`"), "{run:#?}");
    }
}

const EXCEPTED_TABLE: &str = "\
| Crate | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| demo | 9 | 8 | 1 | 0 | 0 | 0 | 0 lines | 0 |

| Crate | Module | Counted | Orphan |
| --- | --- | --- | --- |

| Crate | Item | Kind | File | Orphan lines |
| --- | --- | --- | --- | --- |

| Crate | Item | Kind | File | Excepted lines | Exception | Reason |
| --- | --- | --- | --- | --- | --- | --- |
| demo | `demo::helper` | fn | src/lib.rs | 8-8 | `demo::helper` | Kept for a caller. |
";

#[test]
fn an_exception_is_the_excepted_table_and_a_lines_excepted_object_and_the_report_exits_zero() {
    let matched = "exceptions:\n  - item: demo::helper\n    reason: \"Kept for a caller.\"\n";
    let repo = with_helper("lines-excepted", matched);
    let text = repo.both(&["coverage", "lines"]).expect(0);
    assert_eq!(text.stdout, EXCEPTED_TABLE, "{text:#?}");
    assert_eq!(repo.both(&["coverage", "--reverse"]).expect(0), text);
    let json = repo
        .both(&["--format", "turtle", "coverage", "lines"])
        .expect(0);
    let rdf = Rdf::of(&json);
    assert_eq!(rdf.rows("CrateLines")[0].number("excepted"), 1);
    let row = rdf.rows("ExceptedLines")[0];
    assert_eq!(row.required("exception"), "demo::helper");
    assert_eq!(row.required("reason"), "Kept for a caller.");
    let range = row.children("range")[0];
    assert_eq!((range.number("first"), range.number("last")), (8, 8));
}
