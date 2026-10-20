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
