// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Two-way coverage: the public items and the tests no capability binds, the
//! exclusions of the coverage record, what `check` enforces for the crates of
//! its `scope`, and the two forms of the report.

mod support;

use std::fmt::Write as _;
use std::path::Path;
use std::time::{Duration, Instant};

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
    let findings = findings_of(&repo, &source);
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
    // The package outside the scope has gaps in the report and none in check.
    let report = report_of(&repo, &source);
    let other = report.package("other").expect("other");
    assert!(!other.packages()[0].in_scope());
    assert_eq!(paths(&other, GapKind::PublicItem), ["other::elsewhere"]);
    assert_eq!(paths(&other, GapKind::Test), ["other::tests::other_case"]);
    assert!(findings.iter().all(|f| !f.file.starts_with("crates/other")));
    assert!(report.package("cov-demo").expect("demo").packages()[0].in_scope());
    // The neighbour: every gap bound or excluded, and check is clean.
    let coverage = coverage_with(
        "[cov-demo]",
        &[("cov_demo::tests::*", "Unit tests of internals.")],
    );
    let repo = docs(
        "[cov_demo::tools, cov_demo::Shown, cov_demo::bound, cov_demo::open]",
        "[cov_demo::test_crate::it::bound_test, cov_demo::test_crate::it::open_test]",
        Some(coverage),
    );
    assert_eq!(findings_of(&repo, &source), []);
    assert!(
        report_of(&repo, &source)
            .package("cov-demo")
            .expect("demo")
            .is_complete()
    );
}

#[test]
fn an_empty_scope_enforces_nothing_and_the_report_still_names_every_gap() {
    let source = load(&files());
    let repo = docs("[]", "[]", Some(coverage_with("[]", &[])));
    assert_eq!(findings_of(&repo, &source), []);
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
        let found = check(&[(nothing, "Not there.")]);
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
        assert_eq!(check(&[(matching, "A reason.")]), [], "{matching}");
    }
}

#[test]
fn a_scope_entry_is_a_package_name_and_a_crate_name_says_which_package_it_means() {
    let source = load(&files());
    let findings = findings_of(
        &docs("[]", "[]", Some(coverage_with("[cov_demo]", &[]))),
        &source,
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
    let findings = findings_of(&repo("[cov-demo, other]"), &source);
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
    assert_eq!(findings_of(&repo("[cov-demo]"), &source), []);
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
    let kinds: Vec<&str> = findings_of(
        &docs("[]", "[]", Some(coverage_with("[cov-demo]", &[]))),
        &source,
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
        assert_eq!(render::json(&first, kind), render::json(&second, kind));
    }
    assert_eq!(first.to_string(), second.to_string());
}

#[test]
fn the_table_and_the_json_carry_the_same_entries_escaped() {
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
    assert_eq!(
        render::json(&report, GapKind::PublicItem),
        "[\n{\"kind\":\"coverage-gap\",\"gap\":\"public item\",\"crate\":\"cov-demo\",\"in_scope\":true,\
         \"path\":\"cov_demo::open\",\"item\":\"cov_demo::open\",\"file\":\"crates/cov-demo/src/lib.rs\",\"line\":2}\n]\n"
    );
    assert_eq!(
        render::json(&report, GapKind::Test),
        "[\n{\"kind\":\"coverage-gap\",\"gap\":\"test\",\"crate\":\"cov-demo\",\"in_scope\":true,\
         \"path\":\"cov_demo::test_crate::it::open_test\",\"item\":\"cov_demo::test_crate::it::open_test\",\
         \"file\":\"crates/cov-demo/tests/it.rs\",\"line\":4},\n\
         {\"kind\":\"coverage-excluded\",\"gap\":\"test\",\"crate\":\"cov-demo\",\"in_scope\":true,\
         \"path\":\"cov_demo::tests::unit_case\",\"item\":\"cov_demo::tests::unit_case\",\
         \"file\":\"crates/cov-demo/src/lib.rs\",\"line\":21,\"exclusion\":\"cov_demo::tests::*\",\
         \"reason\":\"Internals | <!-- not a comment --> [x]\"}\n]\n"
    );
    // A surface error is a row of its own table and an object of its own kind.
    let source = load(&ambiguous_files());
    let report = report_of(&docs("[]", "[]", None), &source);
    let other = report.package("other").expect("other");
    assert!(
        render::table(&other, GapKind::PublicItem).contains("\n| Crate | Target | Error |\n"),
        "{}",
        render::table(&other, GapKind::PublicItem)
    );
    let json = render::json(&other, GapKind::PublicItem);
    assert!(
        json.contains("{\"kind\":\"coverage-error\",\"crate\":\"other\",\"in_scope\":false,\"target\":\"other\",\"root\":\"crates/other/src/lib.rs\",\"failure\":\"unresolved\",\"reexport\":null,"),
        "{json}"
    );
    assert_eq!(
        render::json(&CoverageReport::default(), GapKind::Test),
        "[]\n"
    );
}

#[test]
fn a_crate_with_thousands_of_items_and_tests_is_judged_in_bounded_time() {
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
    let started = Instant::now();
    let report = report_of(&repo, &source);
    let findings = findings_of(&repo, &source);
    let elapsed = started.elapsed();
    let big = report.package("big").expect("big");
    assert_eq!(big.packages()[0].items(), MODULES * (ITEMS + 1));
    assert_eq!(big.packages()[0].covered_items(), MODULES / 2 * (ITEMS + 1));
    assert_eq!(big.code().len(), MODULES / 2 * (ITEMS + 1));
    assert_eq!(big.tests().len(), MODULES / 2 * 20);
    assert_eq!(big.excluded().len(), MODULES / 2 * 20);
    assert_eq!(findings.len(), MODULES / 2 * (ITEMS + 1) + MODULES / 2 * 20);
    assert!(elapsed < Duration::from_secs(60), "{elapsed:?}");
}

#[test]
fn this_repository_enforces_the_workplan_crates_and_they_are_covered() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let opened = Gate::workspace(&root).open().expect("the repository opens");
    let plan = opened.workplan();
    let record = plan
        .coverage()
        .expect("the repository has a coverage record");
    assert_eq!(record.value().scope, ["raf-workplan", "cargo-workplan"]);
    assert!(
        record
            .value()
            .exclusions
            .iter()
            .all(|e| !e.reason.trim().is_empty())
    );
    let report = plan.coverage_report();
    for package in ["raf-workplan", "cargo-workplan"] {
        let covered = report.package(package).expect("a package of the workspace");
        assert!(covered.packages()[0].in_scope());
        assert!(covered.is_complete(), "{package}:\n{covered}");
    }
    // The crates outside the scope are reported, and their gaps are named.
    for package in ["raf-okf", "raf-source"] {
        let reported = report.package(package).expect("a package of the workspace");
        assert!(!reported.packages()[0].in_scope());
        assert_ne!(
            reported.code().len(),
            0,
            "{package} has public items no capability binds"
        );
        assert_ne!(
            reported.tests().len(),
            0,
            "{package} has tests no capability binds"
        );
        assert!(reported.code().iter().all(|gap| gap.package() == package));
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
    let findings = findings_of(&scoped(all, &[]), &source);
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
    assert_eq!(findings_of(&scoped(bound, &[]), &source), []);
    for item in ["made::Made", "made::gen::*"] {
        let repo = scoped(all, &[(item, "Generated for the fixtures.")]);
        assert_eq!(findings_of(&repo, &source), [], "{item}");
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
        findings_of(&repo, &source)
            .iter()
            .map(|f| f.kind)
            .collect::<Vec<_>>(),
        ["coverage-item-unbound"]
    );
}
