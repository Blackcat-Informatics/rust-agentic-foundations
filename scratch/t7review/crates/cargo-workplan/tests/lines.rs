// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `coverage lines` through the built binary: the reverse coverage of every
//! package as three tables or as JSON, exit 0 when no orphan line is printed,
//! 1 when one is and 2 for a package that is not one, `coverage --reverse` the
//! same bytes, two runs the same bytes, and nothing written.

mod support;

use raf_workplan::Gate;
use raf_workplan::coverage::render;
use support::{Repo, array, member_names, strict, strings};

/// The members of each kind of object, in order.
const CRATE: [&str; 11] = [
    "kind",
    "crate",
    "enforced",
    "counted",
    "covered",
    "excepted",
    "orphan",
    "shipping_orphan",
    "test_orphan",
    "ceiling",
    "debt",
];
const MODULE: [&str; 5] = ["kind", "crate", "module", "counted", "orphan"];
const ORPHAN: [&str; 7] = ["kind", "crate", "module", "item", "unit", "file", "ranges"];
const EXCEPTED: [&str; 9] = [
    "kind",
    "crate",
    "module",
    "item",
    "unit",
    "file",
    "ranges",
    "exception",
    "reason",
];

/// The report of the strict-clean repository: its documented function and its
/// test are bound, so every counted line is covered.
const CLEAN: &str = "\
| Crate | Enforced | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| demo | no | 8 | 8 | 0 | 0 | 0 | 0 | 0 lines | 0 |

| Crate | Module | Counted | Orphan |
| --- | --- | --- | --- |

| Crate | Item | Kind | File | Orphan lines |
| --- | --- | --- | --- | --- |
";

const CLEAN_JSON: &str = "[
{\"kind\":\"lines-crate\",\"crate\":\"demo\",\"enforced\":false,\"counted\":8,\"covered\":8,\"excepted\":0,\"orphan\":0,\"shipping_orphan\":0,\"test_orphan\":0,\"ceiling\":\"0 lines\",\"debt\":0}
]
";

/// The same repository with a private helper that nothing calls.
const ORPHANED: &str = "\
| Crate | Enforced | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| demo | yes | 9 | 8 | 0 | 1 | 1 | 0 | 0 lines | 0 |

| Crate | Module | Counted | Orphan |
| --- | --- | --- | --- |
| demo | `demo` | 7 | 1 |

| Crate | Item | Kind | File | Orphan lines |
| --- | --- | --- | --- | --- |
| demo | `demo::helper` | fn | src/lib.rs | 8-8 |
";

const ORPHANED_JSON: &str = "[
{\"kind\":\"lines-crate\",\"crate\":\"demo\",\"enforced\":true,\"counted\":9,\"covered\":8,\"excepted\":0,\"orphan\":1,\"shipping_orphan\":1,\"test_orphan\":0,\"ceiling\":\"0 lines\",\"debt\":0},
{\"kind\":\"lines-module\",\"crate\":\"demo\",\"module\":\"demo\",\"counted\":7,\"orphan\":1},
{\"kind\":\"lines-orphan\",\"crate\":\"demo\",\"module\":\"demo\",\"item\":\"demo::helper\",\"unit\":\"fn\",\"file\":\"src/lib.rs\",\"ranges\":[[8,8]]}
]
";

/// The table the library renders for the repository at `repo`.
fn library_table(repo: &Repo) -> String {
    let opened = Gate::workspace(repo.root()).open().expect("a repository");
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
        .both(&["--format", "json", "coverage", "lines"])
        .expect(0);
    assert_eq!(json.stdout, CLEAN_JSON);
    assert_eq!(member_names(&json), vec![strings(&CRATE)]);
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
fn an_orphan_line_is_a_finding_and_exits_one_in_text_and_json_alike_and_reverse_prints_the_same() {
    let repo = Repo::strict("lines-orphan", &strict::capability(), "");
    repo.edit(
        strict::LIB,
        "pub fn run() {}\n",
        "pub fn run() {}\n\nfn helper() {}\n",
    );
    let before = repo.snapshot();
    let text = repo.both(&["--strict", "coverage", "lines"]).expect(1);
    assert_eq!(text.stdout, ORPHANED, "{text:#?}");
    let reverse = repo.both(&["--strict", "coverage", "--reverse"]).expect(1);
    assert_eq!(reverse, text);
    // A finding: `-q` keeps it whole.
    assert_eq!(
        repo.both(&["-q", "--strict", "coverage", "lines"])
            .expect(1),
        text
    );
    let json = repo
        .both(&["--format", "json", "--strict", "coverage", "lines"])
        .expect(1);
    assert_eq!(json.stdout, ORPHANED_JSON);
    assert_eq!(
        member_names(&json),
        vec![strings(&CRATE), strings(&MODULE), strings(&ORPHAN)]
    );
    let objects = array(&json);
    assert_eq!(objects[2]["ranges"], serde_json::json!([[8, 8]]));
    assert_eq!(
        repo.both(&["--format", "json", "--strict", "coverage", "--reverse"])
            .expect(1),
        json
    );
    // Without `--strict` the ceilings are shown as not enforced.
    let plain = repo.both(&["coverage", "lines"]).expect(1);
    assert!(plain.stdout.contains("| demo | no | 9 | 8 |"), "{plain:#?}");
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

/// The `ceilings` and `strictness` lines of a record with one row for `demo`.
fn row(limit: &str) -> String {
    format!("strictness: strict\nceilings:\n  - crate: demo\n{limit}")
}

#[test]
fn a_ceiling_row_and_its_class_decide_the_exit_of_strict_check_on_the_binary() {
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
        let run = repo.both(&["--strict", "check"]).expect(1);
        assert!(
            run.stdout.contains("[coverage-lines-over-ceiling]"),
            "{limit}: {run:#?}"
        );
        assert!(run.stdout.contains("has 1 orphan line "), "{run:#?}");
        let json = repo
            .both(&["--format", "json", "--strict", "check"])
            .expect(1);
        assert!(
            json.stdout
                .contains("\"kind\":\"coverage-lines-over-ceiling\""),
            "{json:#?}"
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
        let run = repo.both(&["--strict", "check"]).expect(0);
        assert!(
            !run.stdout.contains("coverage-lines-over-ceiling"),
            "{run:#?}"
        );
    }
    // Under `default` the same ceiling of 0 is not enforced.
    let repo = with_helper(
        "lines-default",
        "ceilings:\n  - crate: demo\n    lines: 0\n",
    );
    repo.both(&["check"]).expect(0);
    repo.both(&["--strict", "check"]).expect(1);
}

#[test]
fn an_exception_that_matches_nothing_fails_check_under_default_and_its_matched_neighbour_passes() {
    let matched = "exceptions:\n  - item: demo::helper\n    reason: \"Kept for a caller.\"\n";
    let repo = with_helper("lines-exception-matched", matched);
    repo.both(&["check"]).expect(0);
    repo.both(&["--strict", "check"]).expect(0);
    let unmatched = format!("{matched}  - item: demo::ghost\n    reason: \"Nothing is there.\"\n");
    let repo = with_helper("lines-exception-unmatched", &unmatched);
    for args in [&["check"][..], &["--strict", "check"]] {
        let run = repo.both(args).expect(1);
        assert!(
            run.stdout.contains("[coverage-exception-unmatched]")
                && run.stdout.contains("`demo::ghost`"),
            "{run:#?}"
        );
        assert!(!run.stdout.contains("`demo::helper`"), "{run:#?}");
    }
}

const EXCEPTED_TABLE: &str = "\
| Crate | Enforced | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| demo | no | 9 | 8 | 1 | 0 | 0 | 0 | 0 lines | 0 |

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
        .both(&["--format", "json", "coverage", "lines"])
        .expect(0);
    assert_eq!(
        member_names(&json),
        vec![strings(&CRATE), strings(&EXCEPTED)]
    );
    let objects = array(&json);
    assert_eq!(objects[1]["kind"], "lines-excepted");
    assert_eq!(objects[1]["exception"], "demo::helper");
    assert_eq!(objects[1]["reason"], "Kept for a caller.");
    assert_eq!(objects[1]["ranges"], serde_json::json!([[8, 8]]));
}
