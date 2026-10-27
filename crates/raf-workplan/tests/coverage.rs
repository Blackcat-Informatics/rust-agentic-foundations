// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Two-way coverage: the public items and the tests no capability binds, the
//! exclusions of the coverage record, what `check` enforces for the crates of
//! its `scope`, and the two forms of the report.

mod support;

use std::fmt::Write as _;
use std::path::Path;

use raf_okf::Location;
use raf_source::{LoadOptions, MemoryTree, Workspace};
use raf_workplan::coverage::{CoverageReport, Gap, GapKind, SurfaceError, SurfaceFailure, render};
use raf_workplan::{Finding, Gate, Workplan};
use support::cases::coverage_with;
use support::changes::{capability, record};
use support::{COVERAGE, Repo, without_sync_findings};

const CAP: &str = "docs/capabilities/omega.md";
const SHAPE: &str = "work/changes/shape.md";

/// The library of the package `cov-demo` (crate `cov_demo`): items bound and
/// not, a private item, a `pub(crate)` item, a `pub` item of a private module
/// that a re-export exports and one it does not, a public module with a type
/// and a nested module, a `doc(hidden)` item, and a unit test.
const LIB: &str = "\
pub fn bound() {}
pub fn open() {}
fn private() {}
pub(crate) fn crate_only() {}
mod inner;
pub use inner::Shown;
pub mod tools {
    pub struct Tool;
    impl Tool {
        pub fn method(&self) {}
    }
    pub mod deep {
        pub fn nested() {}
    }
}
#[doc(hidden)]
pub fn hidden() {}
#[cfg(test)]
mod tests {
    #[test]
    fn unit_case() {}
}
";

const INNER: &str = "\
pub struct Shown;
impl Shown {
    pub fn go(&self) {}
}
pub fn not_reexported() {}
";

/// The package `other`, outside every scope here, with one public item and
/// one test.
const OTHER: &str =
    "pub fn elsewhere() {}\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn other_case() {}\n}\n";

/// The source files: `cov-demo` and `other` under `crates/`.
fn files() -> Vec<(String, String)> {
    vec![
        (
            "crates/cov-demo/Cargo.toml".to_string(),
            "[package]\nname = \"cov-demo\"\nversion = \"0.1.0\"\n".to_string(),
        ),
        ("crates/cov-demo/src/lib.rs".to_string(), LIB.to_string()),
        (
            "crates/cov-demo/src/inner.rs".to_string(),
            INNER.to_string(),
        ),
        (
            "crates/cov-demo/tests/it.rs".to_string(),
            "#[test]\nfn bound_test() {}\n#[test]\nfn open_test() {}\n".to_string(),
        ),
        (
            "crates/other/Cargo.toml".to_string(),
            "[package]\nname = \"other\"\nversion = \"0.1.0\"\n".to_string(),
        ),
        ("crates/other/src/lib.rs".to_string(), OTHER.to_string()),
    ]
}

fn load(files: &[(String, String)]) -> Workspace {
    let tree: MemoryTree = files
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();
    Workspace::load(&tree, &["crates"]).expect("the workspace loads")
}

/// The documents: `omega` implementing with the bindings given, the change
/// that lists it, and the coverage record (none when `coverage` is `None`).
fn docs(code: &str, tests: &str, coverage: Option<String>) -> Repo {
    let mut repo = Repo::new();
    repo.insert(
        CAP.to_string(),
        capability("omega", "implementing", [code, tests, "[]"], &[]),
    );
    repo.insert(
        SHAPE.to_string(),
        record(
            "shape",
            "[cap:omega]",
            &["cap:omega#contract"],
            "changes",
            "",
        ),
    );
    if let Some(text) = coverage {
        repo.insert(COVERAGE.to_string(), text);
    }
    repo
}

fn with_plan<R>(repo: &Repo, source: &Workspace, f: impl FnOnce(&Workplan<'_>) -> R) -> R {
    let tree: MemoryTree = repo.iter().collect();
    f(&Workplan::load(&tree, source))
}

fn report_of(repo: &Repo, source: &Workspace) -> CoverageReport {
    let tree: MemoryTree = repo.iter().collect();
    Workplan::load(&tree, source).coverage_report()
}

fn findings_of(repo: &Repo, source: &Workspace) -> Vec<Finding> {
    with_plan(repo, source, |plan| without_sync_findings(plan.check()))
}

/// Keep the public-item/test assertions separate only after proving the exact
/// retained default-zero line refusals. These fixtures deliberately leave helper
/// lines unowned; neither scope nor item exclusions waive the line policy.
fn item_findings_after_line_refusals(
    repo: &Repo,
    source: &Workspace,
    expected: &[(&str, usize, usize)],
) -> Vec<Finding> {
    with_plan(repo, source, |plan| {
        let lines = plan.lines_report();
        assert!(lines.errors().is_empty(), "{:?}", lines.errors());
        let rows: Vec<_> = lines
            .crates()
            .iter()
            .map(|row| {
                assert_eq!((row.excepted(), row.debt()), (0, 0));
                assert_eq!(row.ceilings(), []);
                (row.package(), row.shipping_orphan(), row.test_orphan())
            })
            .collect();
        assert_eq!(rows, expected);
        let (refusals, other): (Vec<_>, Vec<_>) = without_sync_findings(plan.check())
            .into_iter()
            .partition(|finding| finding.kind == "coverage-lines-over-ceiling");
        let expected_refusals: Vec<_> = expected
            .iter()
            .flat_map(|&(package, shipping, test)| {
                [("shipping", shipping), ("test", test)]
                    .into_iter()
                    .filter(|(_, orphan)| *orphan > 0)
                    .map(move |(class, orphan)| (package, shipping, test, class, orphan))
            })
            .collect();
        assert_eq!(refusals.len(), expected_refusals.len(), "{refusals:#?}");
        for (finding, (package, shipping, test, class, orphan)) in
            refusals.iter().zip(expected_refusals)
        {
            assert_eq!(finding.file, COVERAGE);
            assert_eq!(finding.location, Location::Key("ceilings".to_string()));
            let total = shipping + test;
            let plural = if total == 1 { "" } else { "s" };
            assert!(
                finding.message.starts_with(&format!(
                    "`{package}` has {total} orphan line{plural} ({shipping} shipping, {test} test)"
                )),
                "{finding:#?}"
            );
            assert!(
                finding
                    .message
                    .contains(&format!("{class} orphan lines, {orphan} of ")),
                "{finding:#?}"
            );
            assert!(
                finding.message.contains("default of 0 lines"),
                "{finding:#?}"
            );
        }
        other
    })
}

const ALL_UNOWNED_LINES: &[(&str, usize, usize)] = &[("cov-demo", 16, 6), ("other", 1, 2)];
const HELPER_UNOWNED_LINES: &[(&str, usize, usize)] = &[("cov-demo", 5, 2), ("other", 1, 2)];

fn paths(report: &CoverageReport, kind: GapKind) -> Vec<&str> {
    report.gaps(kind).iter().map(Gap::path).collect()
}

fn of_demo(report: &CoverageReport) -> CoverageReport {
    report.package("cov-demo").expect("the package is reported")
}

const SURFACE: [&str; 9] = [
    "cov_demo::Shown",
    "cov_demo::Shown::go",
    "cov_demo::bound",
    "cov_demo::open",
    "cov_demo::tools",
    "cov_demo::tools::Tool",
    "cov_demo::tools::Tool::method",
    "cov_demo::tools::deep",
    "cov_demo::tools::deep::nested",
];

#[test]
fn the_code_report_lists_every_public_item_and_no_private_or_hidden_one() {
    let source = load(&files());
    let report = of_demo(&report_of(&docs("[]", "[]", None), &source));
    assert_eq!(paths(&report, GapKind::PublicItem), SURFACE);
    for absent in [
        "cov_demo::private",
        "cov_demo::crate_only",
        "cov_demo::inner::not_reexported",
        "cov_demo::hidden",
    ] {
        assert!(
            report
                .code()
                .iter()
                .all(|gap| gap.item() != absent && gap.path() != absent),
            "{absent} is not public surface"
        );
    }
    // The re-export through the private module is public, under its exported
    // path, and names its definition.
    let shown = &report.code()[0];
    assert_eq!(
        (shown.kind(), shown.package(), shown.item()),
        (GapKind::PublicItem, "cov-demo", "cov_demo::inner::Shown")
    );
    assert_eq!(
        (shown.file(), shown.line()),
        ("crates/cov-demo/src/inner.rs", 1)
    );
    assert_eq!(report.packages()[0].items(), SURFACE.len());
    assert_eq!(report.packages()[0].covered_items(), 0);
}

#[test]
fn trait_members_alias_order_and_foreign_reexports_keep_their_defining_owners() {
    let source = load(&[
        (
            "crates/cov-demo/Cargo.toml".to_string(),
            "[package]\nname=\"cov-demo\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[dependencies]\nother = { path = \"../other\" }\n".to_string(),
        ),
        (
            "crates/cov-demo/src/lib.rs".to_string(),
            "mod inner {\n    pub trait Api {\n        type Associated;\n        const VALUE: u8;\n        fn call(&self);\n    }\n    pub struct Shared;\n}\npub use inner::{Api, Shared as Zebra, Shared as Alpha};\npub use other::elsewhere as Foreign;\npub use std::fmt::Display as Display;\n".to_string(),
        ),
        (
            "crates/other/Cargo.toml".to_string(),
            "[package]\nname=\"other\"\nversion=\"0.1.0\"\nedition=\"2021\"\n".to_string(),
        ),
        ("crates/other/src/lib.rs".to_string(), "pub fn elsewhere() {}\n".to_string()),
    ]);
    let uncovered = report_of(&docs("[]", "[]", None), &source);
    let rows: Vec<_> = uncovered
        .code()
        .iter()
        .map(|gap| {
            assert_eq!(gap.kind(), GapKind::PublicItem, "{gap:?}");
            (gap.path(), gap.item(), gap.line())
        })
        .collect();
    assert_eq!(
        rows,
        [
            ("cov_demo::Alpha", "cov_demo::inner::Shared", 7),
            ("cov_demo::Api", "cov_demo::inner::Api", 2),
            (
                "cov_demo::Api::Associated",
                "cov_demo::inner::Api::Associated",
                3
            ),
            ("cov_demo::Api::VALUE", "cov_demo::inner::Api::VALUE", 4),
            ("cov_demo::Api::call", "cov_demo::inner::Api::call", 5),
            ("other::elsewhere", "other::elsewhere", 1),
        ]
    );
    assert_eq!(
        uncovered.package("cov-demo").unwrap().packages()[0].items(),
        5
    );
    let foreign = uncovered.code().last().unwrap();
    assert_eq!(foreign.package(), "other");
    assert_eq!(foreign.file(), "crates/other/src/lib.rs");
    let explained = report_of(
        &docs(
            "[cov_demo::Zebra, cov_demo::inner::Api, other::elsewhere]",
            "[]",
            None,
        ),
        &source,
    );
    assert_eq!(explained.code(), []);
    assert!(explained.is_complete());
    assert_eq!(
        explained.package("cov-demo").unwrap().packages()[0].covered_items(),
        5
    );
    assert_eq!(
        explained.package("other").unwrap().packages()[0].covered_items(),
        1
    );
}

#[test]
fn binary_public_items_follow_visibility_rather_than_their_names() {
    let mut files = files();
    files.push((
        "crates/cov-demo/src/main.rs".to_string(),
        "pub fn private_looking() {}\nfn public_looking() {}\npub(crate) fn crate_only() {}\n#[doc(hidden)]\npub fn hidden() {}\nfn main() { public_looking(); private_looking(); }\n".to_string(),
    ));
    let source = load(&files);
    let repo = docs("[]", "[]", Some(coverage_with("[cov-demo]", &[])));
    let report = of_demo(&report_of(&repo, &source));
    let binary: Vec<_> = report
        .code()
        .iter()
        .filter(|gap| gap.file().ends_with("src/main.rs"))
        .collect();
    assert_eq!(binary.len(), 1);
    assert_eq!(
        binary[0].path(),
        "cov_demo::bin_crate::cov_demo::private_looking"
    );
    assert_eq!(
        binary[0].item(),
        "cov_demo::bin_crate::cov_demo::private_looking"
    );
    assert_eq!(binary[0].line(), 1);
    let explained = docs(
        "[cov_demo::bin_crate::cov_demo::private_looking]",
        "[]",
        Some(coverage_with("[cov-demo]", &[])),
    );
    let fixed = of_demo(&report_of(&explained, &source));
    assert!(
        fixed
            .code()
            .iter()
            .all(|gap| !gap.file().ends_with("src/main.rs"))
    );
    assert_eq!(
        fixed.packages()[0].covered_items(),
        report.packages()[0].covered_items() + 1
    );
}

#[test]
fn a_bound_item_is_covered_and_its_unbound_neighbour_is_a_gap() {
    let source = load(&files());
    let report = of_demo(&report_of(&docs("[cov_demo::bound]", "[]", None), &source));
    let gaps = paths(&report, GapKind::PublicItem);
    assert!(!gaps.contains(&"cov_demo::bound"), "{gaps:?}");
    assert!(gaps.contains(&"cov_demo::open"), "{gaps:?}");
    assert_eq!(report.packages()[0].covered_items(), 1);
}

#[test]
fn binding_an_item_covers_what_is_below_it_and_nothing_beside_it() {
    let source = load(&files());
    // A module covers every item below it.
    let report = of_demo(&report_of(&docs("[cov_demo::tools]", "[]", None), &source));
    assert_eq!(
        paths(&report, GapKind::PublicItem),
        [
            "cov_demo::Shown",
            "cov_demo::Shown::go",
            "cov_demo::bound",
            "cov_demo::open"
        ]
    );
    // A type covers its methods, not its sibling module.
    let report = of_demo(&report_of(
        &docs("[cov_demo::tools::Tool]", "[]", None),
        &source,
    ));
    let gaps = paths(&report, GapKind::PublicItem);
    assert!(!gaps.contains(&"cov_demo::tools::Tool::method"), "{gaps:?}");
    assert!(gaps.contains(&"cov_demo::tools::deep::nested"), "{gaps:?}");
    assert!(gaps.contains(&"cov_demo::tools"), "{gaps:?}");
    // A crate root covers nothing below it.
    let report = of_demo(&report_of(&docs("[cov_demo]", "[]", None), &source));
    assert_eq!(paths(&report, GapKind::PublicItem), SURFACE);
}

#[test]
fn an_item_bound_through_a_re_export_or_its_definition_is_covered_under_its_exported_path() {
    let source = load(&files());
    for code in ["[cov_demo::Shown]", "[cov_demo::inner::Shown]"] {
        let report = of_demo(&report_of(&docs(code, "[]", None), &source));
        let gaps = paths(&report, GapKind::PublicItem);
        assert!(
            !gaps.contains(&"cov_demo::Shown") && !gaps.contains(&"cov_demo::Shown::go"),
            "{code}: {gaps:?}"
        );
        assert!(gaps.contains(&"cov_demo::bound"), "{code}: {gaps:?}");
    }
}

#[test]
fn a_bound_test_is_covered_and_every_unbound_test_unit_tests_included_is_a_gap() {
    let source = load(&files());
    let report = report_of(
        &docs("[]", "[cov_demo::test_crate::it::bound_test]", None),
        &source,
    );
    assert_eq!(
        paths(&of_demo(&report), GapKind::Test),
        [
            "cov_demo::test_crate::it::open_test",
            "cov_demo::tests::unit_case"
        ]
    );
    let demo = of_demo(&report);
    let unit = &demo.tests()[1];
    assert_eq!(unit.kind(), GapKind::Test);
    assert_eq!(
        (unit.file(), unit.line()),
        ("crates/cov-demo/src/lib.rs", 21)
    );
    assert_eq!(
        report.package("cov-demo").expect("demo").packages()[0].covered_tests(),
        1
    );
    // The other package's unit test is in the report too.
    assert_eq!(
        paths(&report.package("other").expect("other"), GapKind::Test),
        ["other::tests::other_case"]
    );
}

#[test]
fn a_unit_test_module_is_excluded_by_its_prefix_with_the_reason_shown() {
    let source = load(&files());
    let coverage = coverage_with("[]", &[("cov_demo::tests::*", "Unit tests of internals.")]);
    let report = of_demo(&report_of(&docs("[]", "[]", Some(coverage)), &source));
    assert_eq!(
        paths(&report, GapKind::Test),
        [
            "cov_demo::test_crate::it::bound_test",
            "cov_demo::test_crate::it::open_test"
        ]
    );
    let [excluded] = report.excluded() else {
        panic!("{:?}", report.excluded());
    };
    assert_eq!(excluded.gap().path(), "cov_demo::tests::unit_case");
    assert_eq!(excluded.exclusion(), "cov_demo::tests::*");
    assert_eq!(excluded.reason(), "Unit tests of internals.");
}

#[test]
fn an_item_is_excluded_by_its_exported_or_its_defining_path() {
    let source = load(&files());
    for item in [
        "cov_demo::Shown",
        "cov_demo::inner::Shown",
        "cov_demo::inner::*",
    ] {
        let coverage = coverage_with("[cov-demo]", &[(item, "Shown to the fixtures only.")]);
        let report = of_demo(&report_of(&docs("[]", "[]", Some(coverage)), &source));
        let excluded: Vec<&str> = report.excluded().iter().map(|e| e.gap().path()).collect();
        let want: &[&str] = if item.ends_with("::*") {
            &["cov_demo::Shown", "cov_demo::Shown::go"]
        } else {
            &["cov_demo::Shown"]
        };
        assert_eq!(excluded, want, "{item}");
        assert!(!paths(&report, GapKind::PublicItem).contains(&"cov_demo::Shown"));
    }
}

#[test]
fn the_report_of_check_and_its_findings_follow_the_scope() {
    let source = load(&files());
    let coverage = coverage_with(
        "[cov-demo]",
        &[("cov_demo::tests::*", "Unit tests of internals.")],
    );
    let repo = docs(
        "[cov_demo::tools, cov_demo::Shown]",
        "[cov_demo::test_crate::it::bound_test]",
        Some(coverage),
    );
    let findings =
        item_findings_after_line_refusals(&repo, &source, &[("cov-demo", 7, 4), ("other", 1, 2)]);
    let found: Vec<(&str, &str, &Location)> = findings
        .iter()
        .map(|f| (f.kind, f.file.as_str(), &f.location))
        .collect();
    let lib = "crates/cov-demo/src/lib.rs";
    assert_eq!(
        found,
        [
            ("coverage-item-unbound", lib, &Location::Line(1)),
            ("coverage-item-unbound", lib, &Location::Line(2)),
            (
                "coverage-test-unbound",
                "crates/cov-demo/tests/it.rs",
                &Location::Line(4)
            ),
        ],
        "{findings:#?}"
    );
    assert!(
        findings[0].message.contains("`cov_demo::bound`"),
        "{findings:#?}"
    );
    assert!(findings[0].message.contains(COVERAGE), "{findings:#?}");
    // Outside scope, item/test gaps remain reports; line refusals still apply.
    let report = report_of(&repo, &source);
    let other = report.package("other").expect("other");
    assert!(!other.packages()[0].in_scope());
    assert_eq!(paths(&other, GapKind::PublicItem), ["other::elsewhere"]);
    assert_eq!(paths(&other, GapKind::Test), ["other::tests::other_case"]);
    assert!(findings.iter().all(|f| !f.file.starts_with("crates/other")));
    assert!(report.package("cov-demo").expect("demo").packages()[0].in_scope());
    // Every item/test gap is bound or excluded; helper line refusals remain.
    let coverage = coverage_with(
        "[cov-demo]",
        &[("cov_demo::tests::*", "Unit tests of internals.")],
    );
    let repo = docs(
        "[cov_demo::tools, cov_demo::Shown, cov_demo::bound, cov_demo::open]",
        "[cov_demo::test_crate::it::bound_test, cov_demo::test_crate::it::open_test]",
        Some(coverage),
    );
    assert_eq!(
        item_findings_after_line_refusals(&repo, &source, HELPER_UNOWNED_LINES),
        []
    );
    assert!(
        report_of(&repo, &source)
            .package("cov-demo")
            .expect("demo")
            .is_complete()
    );
}

#[test]
fn an_empty_scope_leaves_item_gaps_unenforced_but_retains_line_refusals() {
    let source = load(&files());
    let repo = docs("[]", "[]", Some(coverage_with("[]", &[])));
    assert_eq!(
        item_findings_after_line_refusals(&repo, &source, ALL_UNOWNED_LINES),
        []
    );
    let report = report_of(&repo, &source);
    assert_eq!(paths(&of_demo(&report), GapKind::PublicItem), SURFACE);
    assert!(report.packages().iter().all(|p| !p.in_scope()));
    assert!(!report.is_complete());
}

#[test]
fn without_a_record_check_has_exactly_one_finding_that_says_to_add_it() {
    let source = load(&files());
    let findings = findings_of(&docs("[]", "[]", None), &source);
    let [finding] = findings.as_slice() else {
        panic!("{findings:#?}");
    };
    assert_eq!(
        (finding.kind, finding.file.as_str(), &finding.location),
        ("coverage-record-missing", COVERAGE, &Location::Document)
    );
    assert!(finding.fix.contains("`scope: []`"), "{finding:#?}");
    // A record that is there and does not load is explained by its own
    // findings, not reported as missing.
    let broken = coverage_with("[]", &[("cov_demo::tests::*", "")]);
    let kinds: Vec<&str> = findings_of(&docs("[]", "[]", Some(broken)), &source)
        .iter()
        .map(|f| f.kind)
        .collect();
    assert_eq!(kinds, ["okf-empty-reason"]);
    // A repository without capability documents owes no record.
    let tree = MemoryTree::from([("README.md", "# Readme\n")]);
    assert_eq!(Workplan::load(&tree, &source).check(), []);
}

#[test]
fn an_exclusion_without_a_reason_or_that_matches_nothing_fails_naming_the_file() {
    let source = load(&files());
    let check = |exclusions: &[(&str, &str)]| {
        findings_of(
            &docs("[]", "[]", Some(coverage_with("[]", exclusions))),
            &source,
        )
    };
    let empty = check(&[("cov_demo::tests::*", "")]);
    assert_eq!(
        empty
            .iter()
            .map(|f| (f.kind, f.file.as_str()))
            .collect::<Vec<_>>(),
        [("okf-empty-reason", COVERAGE)]
    );
    for nothing in [
        "cov_demo::ghost",
        "cov_demo::ghost::*",
        "cov_demo::inner::not_reexported",
        "cov_demo::hidden",
        "ghost::thing",
    ] {
        let found = item_findings_after_line_refusals(
            &docs(
                "[]",
                "[]",
                Some(coverage_with("[]", &[(nothing, "Not there.")])),
            ),
            &source,
            ALL_UNOWNED_LINES,
        );
        let [finding] = found.as_slice() else {
            panic!("{nothing}: {found:#?}");
        };
        assert_eq!(
            (finding.kind, finding.file.as_str(), &finding.location),
            (
                "coverage-exclusion-unmatched",
                COVERAGE,
                &Location::Key("exclusions".to_string())
            ),
            "{nothing}"
        );
        assert!(finding.message.contains(nothing), "{finding:#?}");
    }
    // The neighbours: an exclusion with a reason that matches an item, a
    // test, or the unit tests of a package outside the scope.
    for matching in [
        "cov_demo::open",
        "cov_demo::test_crate::it::*",
        "other::tests::*",
    ] {
        assert_eq!(
            item_findings_after_line_refusals(
                &docs(
                    "[]",
                    "[]",
                    Some(coverage_with("[]", &[(matching, "A reason.")]))
                ),
                &source,
                ALL_UNOWNED_LINES,
            ),
            [],
            "{matching}"
        );
    }
}

#[test]
fn a_scope_entry_is_a_package_name_and_a_crate_name_says_which_package_it_means() {
    let source = load(&files());
    let findings = item_findings_after_line_refusals(
        &docs("[]", "[]", Some(coverage_with("[cov_demo]", &[]))),
        &source,
        ALL_UNOWNED_LINES,
    );
    let [finding] = findings.as_slice() else {
        panic!("{findings:#?}");
    };
    assert_eq!(finding.kind, "scope-crate-missing");
    assert!(
        finding
            .message
            .contains("`cov_demo` is the crate name of the package `cov-demo`"),
        "{finding:#?}"
    );
    // The package name is in scope.
    let report = report_of(
        &docs("[]", "[]", Some(coverage_with("[cov-demo]", &[]))),
        &source,
    );
    assert!(of_demo(&report).packages()[0].in_scope());
}

/// A crate whose surface does not list: two `pub` globs bring in one name.
fn ambiguous_files() -> Vec<(String, String)> {
    files()
        .into_iter()
        .map(|(path, text)| {
            if path == "crates/other/src/lib.rs" {
                (
                    path,
                    "pub mod left {\n    pub fn dup() {}\n}\npub mod right {\n    pub fn dup() {}\n}\npub use left::*;\npub use right::*;\n"
                        .to_string(),
                )
            } else {
                (path, text)
            }
        })
        .collect()
}

#[test]
fn a_surface_that_cannot_be_listed_is_an_error_of_the_report_and_a_finding_in_scope() {
    let source = load(&ambiguous_files());
    let repo = |scope: &str| {
        docs(
            "[cov_demo::tools, cov_demo::Shown, cov_demo::bound, cov_demo::open]",
            "[cov_demo::test_crate::it::bound_test, cov_demo::test_crate::it::open_test]",
            Some(coverage_with(
                scope,
                &[("cov_demo::tests::*", "Unit tests of internals.")],
            )),
        )
    };
    let report = report_of(&repo("[cov-demo, other]"), &source);
    let [error] = report.errors() else {
        panic!("{:?}", report.errors());
    };
    assert_eq!((error.package(), error.target()), ("other", "other"));
    assert_eq!(error.failure(), &SurfaceFailure::Unresolved);
    assert!(error.message().contains("`other::left::dup`"), "{error}");
    assert!(error.message().contains("`other::right::dup`"), "{error}");
    assert_eq!(error.root(), "crates/other/src/lib.rs");
    // Its items are neither covered nor gaps, and the error is not dropped.
    assert_eq!(report.package("other").expect("other").code(), []);
    assert!(!report.is_complete());
    let findings = item_findings_after_line_refusals(
        &repo("[cov-demo, other]"),
        &source,
        &[("cov-demo", 5, 2), ("other", 2, 0)],
    );
    let [finding] = findings.as_slice() else {
        panic!("{findings:#?}");
    };
    assert_eq!(
        (finding.kind, finding.file.as_str(), &finding.location),
        (
            "coverage-surface-unreadable",
            COVERAGE,
            &Location::Key("scope".to_string())
        )
    );
    assert!(finding.message.contains("`other`"), "{finding:#?}");
    // Out of scope it is in the report and not in check.
    assert_eq!(report_of(&repo("[cov-demo]"), &source).errors().len(), 1);
    assert_eq!(
        item_findings_after_line_refusals(
            &repo("[cov-demo]"),
            &source,
            &[("cov-demo", 5, 2), ("other", 2, 0)]
        ),
        []
    );
}

#[test]
fn cyclic_imports_and_empty_public_globs_are_report_errors_beside_listable_neighbors() {
    for (refused, accepted, diagnostic) in [
        (
            "pub mod a { pub use crate::b::X; }\npub mod b { pub use crate::a::X; }\n",
            "pub mod a { pub use crate::b::X; }\npub mod b { pub struct X; }\n",
            "X",
        ),
        (
            "pub mod m { pub use self::missing::*; }\n",
            "pub mod m { use self::missing::*; pub fn f() {} }\n",
            "missing",
        ),
    ] {
        for (body, fails) in [(refused, true), (accepted, false)] {
            let source = load(&[
                (
                    "crates/probe/Cargo.toml".to_string(),
                    "[package]\nname = \"probe\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"
                        .to_string(),
                ),
                ("crates/probe/src/lib.rs".to_string(), body.to_string()),
            ]);
            let repo = docs("[]", "[]", Some(coverage_with("[probe]", &[])));
            let report = report_of(&repo, &source);
            assert_eq!(report.errors().len(), usize::from(fails));
            let package = report.package("probe").expect("the actual source package");
            if fails {
                let error = &report.errors()[0];
                assert_eq!(error.failure(), &SurfaceFailure::Unresolved);
                assert_eq!((error.package(), error.target()), ("probe", "probe"));
                assert_eq!(error.root(), "crates/probe/src/lib.rs");
                assert!(error.message().contains(diagnostic), "{error}");
                assert!(!package.is_complete());
                assert_eq!(package.code(), []);
                assert_eq!(package.packages()[0].items(), 0);
            } else {
                assert!(!package.code().is_empty(), "a listable unbound neighbor");
                assert!(package.packages()[0].items() > 0);
            }
        }
    }
}

#[test]
fn carried_name_search_budget_refusal_is_not_an_empty_covered_surface() {
    let mut lib = "mod z {}\npub use c0::*;\n".to_string();
    for i in 0..16 {
        writeln!(lib, "mod c{i} {{ pub use crate::c{}::*; }}", i + 1).unwrap();
    }
    lib.push_str("mod c16 { pub use crate::m::*; pub use crate::a::*; pub use crate::b::*; }\n");
    let (mut carried, mut bound) = (String::new(), String::new());
    for i in 0..8 {
        writeln!(carried, "pub use std::cell::Cell as C{i};").unwrap();
        writeln!(bound, "fn C{i}() {{}}").unwrap();
    }
    writeln!(lib, "mod m {{ {carried} }}\nmod a {{ {bound} pub use crate::z::*; }}\nmod b {{ {bound} pub use crate::z::*; }}").unwrap();
    let tree = MemoryTree::from([
        (
            "probe/Cargo.toml",
            "[package]\nname=\"probe\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
        ),
        ("probe/src/lib.rs", lib.as_str()),
    ]);
    let roomy = Workspace::load(&tree, &["probe"]).expect("the small source loads");
    let spent = roomy.usage().item_bytes;
    let mut options = LoadOptions::default();
    // This admits the retained surface but refuses the actual carried-name
    // search. It is a bounded refusal fixture, not a minimum-budget claim.
    options.max_item_bytes = spent + 52 * 1024;
    let tight = Workspace::load_with(&tree, &["probe"], &options)
        .expect("the load fits before the surface search");
    let repo = docs("[]", "[]", Some(coverage_with("[probe]", &[])));
    let report = report_of(&repo, &tight);
    let [error] = report.errors() else {
        panic!("{:?}", report.errors());
    };
    let SurfaceFailure::CarryBudget { path } = error.failure() else {
        panic!("the carried-name search must retain its budget refusal: {error}");
    };
    assert!(
        (0..8).any(|name| path == &format!("probe::C{name}")),
        "{error}"
    );
    assert_eq!((error.package(), error.target()), ("probe", "probe"));
    assert_eq!(error.root(), "probe/src/lib.rs");
    assert!(error.message().contains(path), "{error}");
    let name = path
        .strip_prefix("probe::")
        .expect("the reader's public path");
    let declaration = format!("pub use std::cell::Cell as {name};");
    let line = lib
        .lines()
        .position(|line| line.contains(&declaration))
        .expect("the refused declaration exists")
        + 1;
    assert!(
        error
            .message()
            .contains(&format!("probe/src/lib.rs:{line}")),
        "{error}"
    );
    let package = report.package("probe").expect("the actual package");
    assert_eq!(package.code(), []);
    assert_eq!(package.packages()[0].items(), 0);
    assert!(!package.is_complete());

    let neighbor = report_of(&repo, &roomy);
    assert_eq!(neighbor.errors(), []);
    assert_eq!(neighbor.package("probe").unwrap().code(), []);
    assert!(
        neighbor.is_complete(),
        "external reexports are not local items"
    );
}

#[test]
fn a_crate_visible_item_sharing_an_external_carried_name_is_not_a_public_item() {
    for (visibility, public) in [("pub(crate)", false), ("pub", true)] {
        let lib = format!(
            "mod local {{ {visibility} struct Cell; }}\nmod external {{ pub use std::cell::Cell; }}\npub use local::*;\npub use external::*;\n"
        );
        let source = load(&[
            (
                "crates/probe/Cargo.toml".to_string(),
                "[package]\nname=\"probe\"\nversion=\"0.1.0\"\nedition=\"2021\"\n".to_string(),
            ),
            ("crates/probe/src/lib.rs".to_string(), lib),
        ]);
        let repo = docs("[]", "[]", Some(coverage_with("[probe]", &[])));
        if !public {
            let target = source.target_ids().next().expect("the probe library");
            let listed = source.exported_items(target).expect("the source surface");
            assert_eq!(listed.items, []);
            let [external] = listed.reexports.as_slice() else {
                panic!("{:?}", listed.reexports);
            };
            assert_eq!(external.path, "probe::Cell");
            assert_eq!(external.written, "std::cell::Cell");
            assert_eq!(external.item, None);
        }
        let report = report_of(&repo, &source);
        assert_eq!(report.errors(), []);
        assert_eq!(report.code().len(), usize::from(public));
        assert_eq!(report.is_complete(), !public);
        if public {
            let gap = &report.code()[0];
            assert_eq!(gap.kind(), GapKind::PublicItem);
            assert_eq!(
                (gap.path(), gap.item()),
                ("probe::Cell", "probe::local::Cell")
            );
            assert_eq!((gap.file(), gap.line()), ("crates/probe/src/lib.rs", 1));
        }
    }
}

#[test]
fn a_surface_over_the_item_budget_is_a_budget_error() {
    let files = files();
    let tree: MemoryTree = files
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();
    let spent = load(&files).usage().item_bytes;
    let mut options = LoadOptions::default();
    options.max_item_bytes = spent + 1;
    let source = Workspace::load_with(&tree, &["crates"], &options).expect("the load fits");
    let report = report_of(
        &docs("[]", "[]", Some(coverage_with("[cov-demo]", &[]))),
        &source,
    );
    let failures: Vec<&SurfaceFailure> =
        report.errors().iter().map(SurfaceError::failure).collect();
    assert_eq!(failures, [&SurfaceFailure::Budget, &SurfaceFailure::Budget]);
    let kinds: Vec<&str> = item_findings_after_line_refusals(
        &docs("[]", "[]", Some(coverage_with("[cov-demo]", &[]))),
        &source,
        ALL_UNOWNED_LINES,
    )
    .iter()
    .map(|f| f.kind)
    .collect();
    // The surface is unreadable; the tests, which need no listing, are judged.
    assert_eq!(
        kinds,
        [
            "coverage-surface-unreadable",
            "coverage-test-unbound",
            "coverage-test-unbound",
            "coverage-test-unbound"
        ]
    );
}

#[test]
fn the_report_and_its_two_forms_are_the_same_on_every_run() {
    let repo = docs(
        "[cov_demo::bound]",
        "[]",
        Some(coverage_with(
            "[cov-demo]",
            &[("cov_demo::tests::*", "Unit tests of internals.")],
        )),
    );
    let first = report_of(&repo, &load(&files()));
    let second = report_of(&repo, &load(&files()));
    assert_eq!(first, second);
    for kind in [GapKind::PublicItem, GapKind::Test] {
        assert_eq!(render::table(&first, kind), render::table(&second, kind));
        for format in [raf_rdf::Format::Turtle, raf_rdf::Format::NTriples] {
            assert_eq!(
                raf_workplan::report::coverage(&first, kind, format).unwrap(),
                raf_workplan::report::coverage(&second, kind, format).unwrap()
            );
        }
    }
    assert_eq!(first.to_string(), second.to_string());
}

#[test]
fn the_table_and_rdf_preserve_the_same_entries_and_escape_their_carriers() {
    let source = load(&files());
    let coverage = coverage_with(
        "[cov-demo]",
        &[(
            "cov_demo::tests::*",
            "Internals | <!-- not a comment --> [x]",
        )],
    );
    let repo = docs(
        "[cov_demo::tools, cov_demo::Shown, cov_demo::bound]",
        "[cov_demo::test_crate::it::bound_test]",
        Some(coverage),
    );
    let report = of_demo(&report_of(&repo, &source));
    assert_eq!(
        render::table(&report, GapKind::PublicItem),
        "| Gap | Crate | Path | Location |\n| --- | --- | --- | --- |\n\
         | public item | cov-demo | `cov_demo::open` | crates/cov-demo/src/lib.rs:2 |\n"
    );
    assert_eq!(
        render::table(&report, GapKind::Test),
        "| Gap | Crate | Path | Location |\n| --- | --- | --- | --- |\n\
         | test | cov-demo | `cov_demo::test_crate::it::open_test` | crates/cov-demo/tests/it.rs:4 |\n\
         \n| Excluded | Crate | Path | Location | Exclusion | Reason |\n| --- | --- | --- | --- | --- | --- |\n\
         | test | cov-demo | `cov_demo::tests::unit_case` | crates/cov-demo/src/lib.rs:21 | `cov_demo::tests::*` | Internals \\| &lt;!-- not a comment --> \\[x\\] |\n"
    );
    for kind in [GapKind::PublicItem, GapKind::Test] {
        let graph = raf_workplan::report::coverage_graph(&report, kind).unwrap();
        for format in [raf_rdf::Format::Turtle, raf_rdf::Format::NTriples] {
            let text = graph.to_text(format).unwrap();
            assert_eq!(
                raf_rdf::Graph::from_text(&text, format)
                    .unwrap()
                    .to_text(format)
                    .unwrap(),
                text
            );
        }
        if kind == GapKind::Test {
            assert!(graph.assertions().any(|t| matches!(&t.predicate, raf_rdf::TermValue::Iri(p) if p.ends_with("#reason")) && matches!(t.object,raf_rdf::TermValue::Literal { lexical_form,.. } if lexical_form=="Internals | <!-- not a comment --> [x]")));
        }
    }
    // A surface error is a row of its own table and an object of its own kind.
    let source = load(&ambiguous_files());
    let report = report_of(&docs("[]", "[]", None), &source);
    let other = report.package("other").expect("other");
    assert!(
        render::table(&other, GapKind::PublicItem).contains("\n| Crate | Target | Error |\n"),
        "{}",
        render::table(&other, GapKind::PublicItem)
    );
    assert_eq!(other.errors().len(), 1);
    assert_eq!(other.errors()[0].package(), "other");
    assert_eq!(other.errors()[0].root(), "crates/other/src/lib.rs");
    let graph = raf_workplan::report::coverage_graph(&other, GapKind::PublicItem).unwrap();
    assert!(graph.assertions().any(
        |t| matches!(&t.predicate, raf_rdf::TermValue::Iri(p) if p.ends_with("#failure"))
            && t.object == raf_rdf::TermValue::simple_literal("unresolved")
    ));
    let empty =
        raf_workplan::report::coverage_graph(&CoverageReport::default(), GapKind::Test).unwrap();
    assert!(
        !empty
            .assertions()
            .any(|t| matches!(&t.predicate, raf_rdf::TermValue::Iri(p) if p.ends_with("#result")))
    );
}

#[test]
fn a_crate_with_thousands_of_items_and_tests_is_judged() {
    const MODULES: usize = 60;
    const ITEMS: usize = 50;
    let mut lib = String::new();
    let mut tests = String::new();
    for m in 0..MODULES {
        let _ = writeln!(lib, "pub mod m{m} {{");
        for i in 0..ITEMS {
            let _ = writeln!(lib, "    pub fn f{i}() {{}}");
        }
        let _ = writeln!(lib, "}}");
        for i in 0..20 {
            let _ = writeln!(tests, "#[test]\nfn t{m}_{i}() {{}}");
        }
    }
    let files = vec![
        (
            "crates/big/Cargo.toml".to_string(),
            "[package]\nname = \"big\"\nversion = \"0.1.0\"\n".to_string(),
        ),
        ("crates/big/src/lib.rs".to_string(), lib),
        ("crates/big/tests/it.rs".to_string(), tests),
    ];
    let source = load(&files);
    // Half the modules are bound; the tests of the other half are excluded
    // by name, one exclusion each.
    let code: Vec<String> = (0..MODULES / 2).map(|m| format!("big::m{m}")).collect();
    let exclusions: Vec<(String, String)> = (MODULES / 2..MODULES)
        .flat_map(|m| {
            (0..20).map(move |i| {
                (
                    format!("big::test_crate::it::t{m}_{i}"),
                    "Load.".to_string(),
                )
            })
        })
        .collect();
    let rows: Vec<(&str, &str)> = exclusions
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let repo = docs(
        &format!("[{}]", code.join(", ")),
        "[]",
        Some(coverage_with("[big]", &rows)),
    );
    let report = report_of(&repo, &source);
    let findings = item_findings_after_line_refusals(
        &repo,
        &source,
        &[("big", MODULES / 2 * ITEMS, MODULES * 20 * 2)],
    );
    let big = report.package("big").expect("big");
    assert_eq!(big.packages()[0].items(), MODULES * (ITEMS + 1));
    assert_eq!(big.packages()[0].covered_items(), MODULES / 2 * (ITEMS + 1));
    assert_eq!(big.code().len(), MODULES / 2 * (ITEMS + 1));
    assert_eq!(big.tests().len(), MODULES / 2 * 20);
    assert_eq!(big.excluded().len(), MODULES / 2 * 20);
    assert_eq!(findings.len(), MODULES / 2 * (ITEMS + 1) + MODULES / 2 * 20);
}

#[test]
fn this_repository_enforces_all_packages_with_zero_coverage_debt() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let opened = Gate::workspace(&root).open().expect("the repository opens");
    let plan = opened.workplan();
    let record = plan
        .coverage()
        .expect("the repository has a coverage record");
    let packages = [
        "cargo-census",
        "cargo-workplan",
        "raf-census",
        "raf-okf",
        "raf-rdf",
        "raf-source",
        "raf-source-fuzz",
        "raf-workplan",
        "rust-agentic-foundations",
    ];
    assert_eq!(record.value().scope, packages);
    assert_eq!(record.value().exclusions.as_slice(), []);
    assert_eq!(record.value().exceptions.as_slice(), []);
    assert_eq!(record.value().ceilings.as_slice(), []);
    let report = plan.coverage_report();
    for package in packages {
        let covered = report.package(package).expect("a package of the workspace");
        assert!(covered.packages()[0].in_scope());
        assert!(covered.is_complete(), "{package}:\n{covered}");
    }
    let lines = plan.lines_report();
    assert!(lines.errors().is_empty(), "{:?}", lines.errors());
    assert!(lines.is_complete(), "{:?}", lines.orphans());
    assert_eq!(lines.crates().len(), packages.len());
    for row in lines.crates() {
        assert_eq!(
            (row.orphan(), row.excepted(), row.debt()),
            (0, 0, 0),
            "{}",
            row.package()
        );
        assert_eq!(row.covered(), row.counted());
    }
    assert!(
        plan.check()
            .iter()
            .all(|f| !f.kind.starts_with("coverage-")),
        "{:#?}",
        plan.check()
    );
}

/// The package `made` (crate `made`): a type a macro generates in a private
/// module, re-exported at the root, a re-export of a crate outside the
/// workspace, two modules whose names share a prefix, and a documented
/// function.
fn made_files() -> Vec<(String, String)> {
    vec![
        (
            "crates/made/Cargo.toml".to_string(),
            "[package]\nname = \"made\"\nversion = \"0.1.0\"\n".to_string(),
        ),
        (
            "crates/made/src/lib.rs".to_string(),
            "mod gen {\n    macro_rules! make {\n        () => {\n            pub struct Made;\n        };\n    }\n    make!();\n}\n\
             pub use gen::Made;\npub use serde::Serialize;\n\
             pub mod body {\n    pub fn f() {}\n}\npub mod body_x {\n    pub fn f() {}\n}\n\
             /// Documented.\npub fn shown() {}\n"
                .to_string(),
        ),
    ]
}

#[test]
fn a_re_export_the_index_cannot_follow_is_reported_and_enforced_and_its_module_covers_it() {
    let source = load(&made_files());
    let scoped = |code: &str, exclusions: &[(&str, &str)]| {
        docs(code, "[]", Some(coverage_with("[made]", exclusions)))
    };
    let all = "[made::body, made::body_x, made::shown]";
    let report = report_of(&scoped(all, &[]), &source);
    let [gap] = report.code() else {
        panic!("{:#?}", report.code());
    };
    assert_eq!(
        (gap.kind(), gap.path(), gap.item(), gap.file(), gap.line()),
        (
            GapKind::Reexport,
            "made::Made",
            "made::gen::Made",
            "crates/made/src/lib.rs",
            9
        )
    );
    assert_eq!(gap.kind().name(), "unresolved re-export");
    // The re-export of a crate outside the workspace is not counted.
    assert!(
        report
            .code()
            .iter()
            .all(|g| !g.path().contains("Serialize"))
    );
    let findings = item_findings_after_line_refusals(&scoped(all, &[]), &source, &[("made", 6, 0)]);
    let [finding] = findings.as_slice() else {
        panic!("{findings:#?}");
    };
    assert_eq!(
        (finding.kind, finding.file.as_str(), &finding.location),
        (
            "coverage-reexport-unbound",
            "crates/made/src/lib.rs",
            &Location::Line(9)
        )
    );
    assert!(
        finding.message.contains("`made::gen::Made`"),
        "{finding:#?}"
    );
    // The module the written path names covers it, and so does an exclusion
    // of its exported path or of that module.
    let bound = "[made::body, made::body_x, made::shown, made::gen]";
    assert_eq!(
        item_findings_after_line_refusals(&scoped(bound, &[]), &source, &[("made", 0, 0)]),
        []
    );
    for item in ["made::Made", "made::gen::*"] {
        let repo = scoped(all, &[(item, "Generated for the fixtures.")]);
        assert_eq!(
            item_findings_after_line_refusals(&repo, &source, &[("made", 6, 0)]),
            [],
            "{item}"
        );
        let report = report_of(&repo, &source);
        assert_eq!(report.excluded().len(), 1, "{item}");
        assert_eq!(report.excluded()[0].gap().kind(), GapKind::Reexport);
    }
    // The table names the kind.
    assert!(
        render::table(&report, GapKind::Reexport)
            .contains("| unresolved re-export | made | `made::Made` | crates/made/src/lib.rs:9 |"),
        "{}",
        render::table(&report, GapKind::Reexport)
    );
}

#[test]
fn a_prefix_matches_below_its_module_and_never_a_sibling_that_shares_its_text() {
    let source = load(&made_files());
    let repo = docs(
        "[made::gen, made::shown]",
        "[]",
        Some(coverage_with(
            "[made]",
            &[("made::body::*", "Internal helpers.")],
        )),
    );
    let report = report_of(&repo, &source);
    let excluded: Vec<&str> = report.excluded().iter().map(|e| e.gap().path()).collect();
    assert_eq!(excluded, ["made::body::f"]);
    assert_eq!(
        paths(&report, GapKind::PublicItem),
        ["made::body", "made::body_x", "made::body_x::f"]
    );
}

#[test]
fn an_item_bound_only_in_examples_is_a_gap() {
    let source = load(&made_files());
    let mut repo = docs(
        "[made::gen, made::body, made::body_x]",
        "[]",
        Some(coverage_with("[made]", &[])),
    );
    let text = repo.get_mut(CAP).expect("the capability");
    *text = text.replacen("  examples: []", "  examples: [made::shown]", 1);
    let report = report_of(&repo, &source);
    assert_eq!(paths(&report, GapKind::PublicItem), ["made::shown"]);
    // Bound in `code`, it is covered.
    let repo = docs(
        "[made::gen, made::body, made::body_x, made::shown]",
        "[]",
        Some(coverage_with("[made]", &[])),
    );
    assert_eq!(report_of(&repo, &source).code(), []);
}

#[test]
fn the_first_matching_exclusion_gives_the_reason() {
    let source = load(&made_files());
    let repo = docs(
        "[made::gen, made::body_x, made::shown]",
        "[]",
        Some(coverage_with(
            "[made]",
            &[
                ("made::body::*", "The first reason."),
                ("made::body::f", "The second reason."),
            ],
        )),
    );
    let report = report_of(&repo, &source);
    let reasons: Vec<(&str, &str)> = report
        .excluded()
        .iter()
        .map(|e| (e.gap().path(), e.reason()))
        .collect();
    assert_eq!(reasons, [("made::body::f", "The first reason.")]);
    // Both match, so neither is unmatched.
    assert_eq!(
        item_findings_after_line_refusals(&repo, &source, &[("made", 1, 0)])
            .iter()
            .map(|f| f.kind)
            .collect::<Vec<_>>(),
        ["coverage-item-unbound"]
    );
}

#[test]
fn a_capability_that_does_not_load_gives_its_own_findings_and_no_cascade_of_gaps() {
    let source = load(&files());
    let coverage = || {
        coverage_with(
            "[cov-demo]",
            &[("cov_demo::tests::*", "Unit tests of internals.")],
        )
    };
    // `omega` binds part of the crate and `zeta` the rest.
    let mut clean = docs(
        "[cov_demo::tools, cov_demo::Shown]",
        "[cov_demo::test_crate::it::bound_test]",
        Some(coverage()),
    );
    let zeta = "docs/capabilities/zeta.md";
    clean.insert(
        zeta.to_string(),
        capability(
            "zeta",
            "implementing",
            [
                "[cov_demo::bound, cov_demo::open]",
                "[cov_demo::test_crate::it::open_test]",
                "[]",
            ],
            &[],
        ),
    );
    let is_gap = |f: &Finding| f.kind.starts_with("coverage-") && f.kind.ends_with("-unbound");
    assert!(
        findings_of(&clean, &source).iter().all(|f| !is_gap(f)),
        "{:#?}",
        findings_of(&clean, &source)
    );
    // `zeta` does not load: its own finding, and not one gap for what it binds
    // or for what nothing binds.
    let mut broken = clean.clone();
    let text = broken[zeta].replacen("state: implementing", "state: unknown", 1);
    broken.insert(zeta.to_string(), text);
    let findings = findings_of(&broken, &source);
    assert!(findings.iter().any(|f| f.file == zeta), "{findings:#?}");
    assert!(findings.iter().all(|f| !is_gap(f)), "{findings:#?}");
    // An exclusion that matches nothing is still reported meanwhile.
    let mut unmatched = broken.clone();
    unmatched.insert(
        COVERAGE.to_string(),
        coverage_with(
            "[cov-demo]",
            &[
                ("cov_demo::tests::*", "Unit tests of internals."),
                ("cov_demo::gone", "It was removed."),
            ],
        ),
    );
    let findings = findings_of(&unmatched, &source);
    assert!(
        findings
            .iter()
            .any(|f| f.kind == "coverage-exclusion-unmatched"),
        "{findings:#?}"
    );
    // The neighbours: a change record that does not load binds nothing, so the
    // gaps of a capability that is missing its bindings stay; and once `zeta`
    // loads with one binding dropped, that item is a gap again.
    let mut change = docs(
        "[cov_demo::tools, cov_demo::Shown]",
        "[cov_demo::test_crate::it::bound_test]",
        Some(coverage()),
    );
    let text = change[SHAPE].replacen("contract_effect: changes", "contract_effect: maybe", 1);
    change.insert(SHAPE.to_string(), text);
    let findings = findings_of(&change, &source);
    assert!(findings.iter().any(|f| f.file == SHAPE), "{findings:#?}");
    assert!(
        findings
            .iter()
            .any(|f| f.kind == "coverage-item-unbound" && f.message.contains("`cov_demo::open`")),
        "{findings:#?}"
    );
    let mut dropped = clean;
    let text = dropped[zeta].replacen("[cov_demo::bound, cov_demo::open]", "[cov_demo::bound]", 1);
    dropped.insert(zeta.to_string(), text);
    let gaps: Vec<String> = findings_of(&dropped, &source)
        .into_iter()
        .filter(|f| is_gap(f))
        .map(|f| f.message)
        .collect();
    assert_eq!(gaps.len(), 1, "{gaps:#?}");
    assert!(gaps[0].contains("`cov_demo::open`"), "{gaps:#?}");
}

#[test]
fn an_unresolved_bare_reexport_keeps_its_edition_specific_module_origin() {
    for (edition, written, named) in [
        ("2015", "gen::Made", "made::gen::Made"),
        ("2015", "::gen::Made", "made::gen::Made"),
        ("2021", "gen::Made", "made::api::gen::Made"),
    ] {
        let source = load(&[
            (
                "crates/made/Cargo.toml".to_string(),
                format!("[package]\nname=\"made\"\nversion=\"0.1.0\"\nedition=\"{edition}\"\n"),
            ),
            (
                "crates/made/src/lib.rs".to_string(),
                format!(
                    "mod gen {{ macro_rules! make {{ () => {{ pub struct Made; }}; }} make!(); }}\nmod api {{\n    mod gen {{ macro_rules! make {{ () => {{ pub struct Made; }}; }} make!(); }}\n    pub use {written};\n}}\npub use api::*;\n"
                ),
            ),
        ]);
        let report = report_of(&docs("[]", "[]", None), &source);
        let [gap] = report.code() else {
            panic!("{:?}", report.code());
        };
        assert_eq!(gap.kind(), GapKind::Reexport);
        assert_eq!(gap.path(), "made::Made");
        assert_eq!(gap.item(), named, "edition {edition}: {written}");
        assert_eq!((gap.file(), gap.line()), ("crates/made/src/lib.rs", 4));
    }
}

#[test]
fn a_re_export_written_through_super_names_the_path_super_reaches() {
    let made = |lib: &str, api: Option<&str>| {
        let mut files = vec![
            (
                "crates/made/Cargo.toml".to_string(),
                "[package]\nname = \"made\"\nversion = \"0.1.0\"\n".to_string(),
            ),
            (
                "crates/made/src/lib.rs".to_string(),
                format!(
                    "mod gen {{\n    macro_rules! make {{\n        () => {{\n            pub struct Made;\n        }};\n    }}\n    make!();\n}}\n{lib}/// Documented.\npub fn shown() {{}}\n"
                ),
            ),
        ];
        if let Some(api) = api {
            files.push(("crates/made/src/api.rs".to_string(), api.to_string()));
        }
        load(&files)
    };
    // In a module file, one `super::` is one module up from the module that
    // declares the re-export: the module the path reaches covers it, and the
    // module that declares it, which the path leaves, does not.
    let scoped = |code: &str| docs(code, "[]", Some(coverage_with("[made]", &[])));
    let file = made(
        "mod api;\npub use api::*;\n",
        Some("pub use super::gen::Made;\n"),
    );
    let report = report_of(&scoped("[made::shown]"), &file);
    let [gap] = report.code() else {
        panic!("{:#?}", report.code());
    };
    assert_eq!(
        (gap.kind(), gap.path(), gap.item(), gap.file()),
        (
            GapKind::Reexport,
            "made::Made",
            "made::gen::Made",
            "crates/made/src/api.rs"
        )
    );
    assert_eq!(
        item_findings_after_line_refusals(
            &scoped("[made::shown, made::gen]"),
            &file,
            &[("made", 0, 0)]
        ),
        []
    );
    let findings = findings_of(&scoped("[made::shown, made::api]"), &file);
    assert!(
        findings
            .iter()
            .any(|f| f.kind == "coverage-reexport-unbound"
                && f.message.contains("`made::gen::Made`")),
        "{findings:#?}"
    );
    // In an inline module two levels below the root, two `super::` reach the
    // root, and a glob of an inline module is read from the module that holds
    // it, so it is counted as a re-export of the workspace.
    let inline = made(
        "pub mod api {\n    pub mod deep {\n        pub use super::super::gen::Made;\n    }\n}\nmod outer {\n    mod deep {\n        pub use super::super::gen::Made;\n    }\n    pub use self::deep::*;\n}\npub use outer::*;\n",
        None,
    );
    let report = report_of(&scoped("[made::shown, made::api]"), &inline);
    let gaps: Vec<(&str, &str)> = report.code().iter().map(|g| (g.path(), g.item())).collect();
    assert_eq!(
        gaps,
        [("made::Made", "made::gen::Made")],
        "{:#?}",
        report.code()
    );
    assert_eq!(
        item_findings_after_line_refusals(
            &scoped("[made::shown, made::api, made::gen]"),
            &inline,
            &[("made", 0, 0)]
        ),
        []
    );
    let report = report_of(&scoped("[made::shown, made::outer]"), &inline);
    let gaps: Vec<(&str, &str)> = report.code().iter().map(|g| (g.path(), g.item())).collect();
    assert_eq!(
        gaps,
        [
            ("made::Made", "made::gen::Made"),
            ("made::api", "made::api"),
            ("made::api::deep", "made::api::deep"),
            ("made::api::deep::Made", "made::gen::Made"),
        ],
        "{:#?}",
        report.code()
    );
    // The neighbour: `self::` in a module file is read from that module.
    let own = made(
        "mod api;\npub use api::*;\n",
        Some(
            "mod gen {\n    macro_rules! make {\n        () => {\n            pub struct Other;\n        };\n    }\n    make!();\n}\npub use self::gen::Other;\n",
        ),
    );
    let report = report_of(&scoped("[made::shown, made::gen]"), &own);
    let [gap] = report.code() else {
        panic!("{:#?}", report.code());
    };
    assert_eq!(gap.item(), "made::api::gen::Other");
    assert_eq!(
        findings_of(&scoped("[made::shown, made::gen, made::api::gen]"), &own),
        []
    );
}
