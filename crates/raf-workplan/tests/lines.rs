// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Reverse coverage: the orphan lines of `Workplan::lines_report`, the
//! exceptions and ceilings of the coverage record, and the findings of strict
//! mode for them, each fixture beside the neighbour that passes.

use std::fmt::Write as _;

use raf_okf::Location;
use raf_source::{MemoryTree, Workspace};
use raf_workplan::coverage::lines::{CrateLines, LinesReport, UnitLines};
use raf_workplan::coverage::render;
use raf_workplan::{Finding, Strictness, Workplan};

/// The manifest of the package `name`.
fn manifest(name: &str) -> String {
    format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
}

/// The source workspace of `files`, the manifest of `demo` at the root
/// unless `files` holds one.
fn source(files: &[(&str, &str)]) -> Workspace {
    let mut all: Vec<(String, String)> = files
        .iter()
        .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
        .collect();
    if !all.iter().any(|(path, _)| path == "Cargo.toml") {
        all.push(("Cargo.toml".to_string(), manifest("demo")));
    }
    let tree: MemoryTree = all
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect();
    Workspace::load(&tree, &["."]).unwrap_or_else(|error| panic!("the workspace loads: {error}"))
}

/// A capability that binds `code` and `tests`, and the coverage record with
/// `scope: []`, no exclusion and `record` after them.
fn documents(code: &[&str], tests: &[&str], record: &str) -> MemoryTree {
    let mut sections = String::new();
    for name in [
        "Purpose",
        "Contract",
        "Usage",
        "Examples",
        "Constraints",
        "Explanation",
        "Verification",
    ] {
        let _ = write!(sections, "## {name}\n\nText for {name}.\n\n");
    }
    let capability = format!(
        "---\ntype: Capability\nid: cap:lines\ntitle: Lines\ndescription: Lines.\n\
         state: implementing\nbindings:\n  code: [{}]\n  tests: [{}]\n  examples: []\n\
         depends: []\n---\n\n# Lines\n\n{sections}",
        code.join(", "),
        tests.join(", ")
    );
    let coverage = format!(
        "---\ntype: Coverage\nid: coverage:default\nscope: []\nexclusions: []\n{record}---\n"
    );
    MemoryTree::from([
        ("docs/capabilities/lines.md", capability.as_str()),
        ("work/coverage/default.md", coverage.as_str()),
    ])
}

/// The report of `source` under the capability that binds `code` and `tests`
/// and the record with `record`.
fn lines_of(source: &Workspace, code: &[&str], tests: &[&str], record: &str) -> LinesReport {
    let docs = documents(code, tests, record);
    let plan = Workplan::load(&docs, source);
    let report = plan.lines_report();
    assert!(report.errors().is_empty(), "{:?}", report.errors());
    report
}

/// Each orphan unit as its item and ranges.
fn orphans(report: &LinesReport) -> Vec<(String, String)> {
    report
        .orphans()
        .iter()
        .map(|unit| (unit.item().to_string(), unit.ranges_text()))
        .collect()
}

/// The pairs as owned strings.
fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items
        .iter()
        .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
        .collect()
}

/// The findings of `check` under `strictness` of the kinds of the lines.
fn line_findings(
    source: &Workspace,
    code: &[&str],
    record: &str,
    strictness: Strictness,
) -> Vec<Finding> {
    let docs = documents(code, &[], record);
    Workplan::load(&docs, source)
        .with_strictness(strictness)
        .check()
        .into_iter()
        .filter(|finding| {
            matches!(
                finding.kind,
                "coverage-lines-over-ceiling"
                    | "coverage-exception-unmatched"
                    | "scope-crate-missing"
            )
        })
        .collect()
}

const BOUND_LIB: &str = "\
pub fn run() {
    helper();
}

fn helper() {}

pub struct Frame;

impl Frame {
    pub fn new() -> Self {
        Frame
    }
}
";

#[test]
fn a_crate_whose_public_surface_is_bound_and_whose_helpers_are_reached_has_no_orphan_line() {
    let ws = source(&[("src/lib.rs", BOUND_LIB)]);
    let report = lines_of(&ws, &["demo::run", "demo::Frame"], &[], "");
    let row = &report.crates()[0];
    assert_eq!(row.package(), "demo");
    // Every line of an item or an impl block counts; the blank lines do not.
    assert_eq!(row.counted(), 10);
    assert_eq!(row.covered(), 10);
    assert_eq!(row.orphan(), 0);
    assert_eq!(report.orphans(), []);
    assert_eq!(report.modules(), []);
    assert!(report.is_complete());
}

const HELPER_LIB: &str = "\
pub fn run() {}

/// A helper nothing calls.
#[inline]
fn helper() -> u8 {
    1
}
";

#[test]
fn an_unreferenced_private_helper_is_exactly_its_span_and_the_same_helper_called_by_a_bound_function_is_not()
 {
    let ws = source(&[("src/lib.rs", HELPER_LIB)]);
    let report = lines_of(&ws, &["demo::run"], &[], "");
    // Its doc comment and attribute are lines of its span.
    assert_eq!(orphans(&report), pairs(&[("demo::helper", "3-7")]));
    let unit = &report.orphans()[0];
    assert_eq!(
        (unit.kind(), unit.file(), unit.module(), unit.lines()),
        ("fn", "src/lib.rs", "demo", 5)
    );
    assert_eq!(report.crates()[0].shipping_orphan(), 5);
    assert_eq!(report.crates()[0].test_orphan(), 0);
    let module = &report.modules()[0];
    assert_eq!(
        (
            module.package(),
            module.module(),
            module.counted(),
            module.orphan()
        ),
        ("demo", "demo", 6, 5)
    );

    let called = HELPER_LIB.replace("pub fn run() {}", "pub fn run() {\n    helper();\n}");
    let ws = source(&[("src/lib.rs", &called)]);
    assert_eq!(report_of(&ws, &["demo::run"]).orphans(), []);
}

fn report_of(ws: &Workspace, code: &[&str]) -> LinesReport {
    lines_of(ws, code, &[], "")
}

const TYPE_LIB: &str = "\
pub struct Frame {
    pub size: u8,
}

impl Frame {
    pub fn new() -> Self {
        Self { size: 0 }
    }

    pub fn grow(&mut self) {
        self.size += 1;
    }
}

impl Clone for Frame {
    fn clone(&self) -> Self {
        Self { size: self.size }
    }
}
";

#[test]
fn binding_a_type_covers_its_impl_blocks_and_their_methods_and_binding_a_method_covers_nothing_beside_it()
 {
    let ws = source(&[("src/lib.rs", TYPE_LIB)]);
    assert_eq!(report_of(&ws, &["demo::Frame"]).orphans(), []);
    let report = report_of(&ws, &["demo::Frame::new"]);
    assert_eq!(
        orphans(&report),
        pairs(&[
            ("demo::Frame", "1-3"),
            ("demo::<impl Frame>", "5-5, 9-9, 13-13"),
            ("demo::Frame::grow", "10-12"),
            ("demo::<impl Clone for Frame>", "15-15, 19-19"),
            ("demo::Frame::clone", "16-18"),
        ])
    );
    let kinds: Vec<&str> = report.orphans().iter().map(UnitLines::kind).collect();
    assert_eq!(kinds, ["struct", "impl", "fn", "impl", "fn"]);
}

const MACRO_LIB: &str = "\
macro_rules! make {
    ($name:ident) => {
        pub fn $name() {}
    };
}

make!(made);

pub fn run() {}
";

#[test]
fn a_macro_invocation_is_orphan_with_its_range_and_covered_once_a_bound_function_calls_what_it_generates()
 {
    let ws = source(&[("src/lib.rs", MACRO_LIB)]);
    let report = report_of(&ws, &["demo::run"]);
    // The generated function has no lines of its own: they are the
    // invocation's, and the macro is orphan with it.
    assert_eq!(
        orphans(&report),
        pairs(&[("demo::make", "1-5"), ("demo::make!", "7-7")])
    );
    assert_eq!(report.orphans()[1].kind(), "macro");
    let calling = MACRO_LIB.replace("pub fn run() {}", "pub fn run() {\n    made();\n}");
    let ws = source(&[("src/lib.rs", &calling)]);
    let report = report_of(&ws, &["demo::run"]);
    assert!(report.orphans().is_empty(), "{:?}", orphans(&report));
    // The invocation of a bound module is covered with it.
    let module = MACRO_LIB.replace("make!(made);", "pub mod made {\n    make!(made);\n}");
    let ws = source(&[("src/lib.rs", &module)]);
    let report = report_of(&ws, &["demo::run", "demo::made"]);
    assert!(report.orphans().is_empty(), "{:?}", orphans(&report));
}

const TESTED_LIB: &str = "\
pub fn run() {}

#[cfg(test)]
mod tests {
    #[test]
    fn bound() {
        help();
    }

    #[test]
    fn unbound() {
        unhelped();
    }

    fn help() {}

    fn unhelped() {}
}
";

#[test]
fn a_cfg_test_module_with_one_bound_and_one_unbound_test_reports_the_unbound_one_and_its_helper_as_test_lines()
 {
    let ws = source(&[("src/lib.rs", TESTED_LIB)]);
    let report = lines_of(&ws, &["demo::run"], &["demo::tests::bound"], "");
    assert_eq!(
        orphans(&report),
        pairs(&[
            ("demo::tests::unbound", "10-13"),
            ("demo::tests::unhelped", "17-17")
        ])
    );
    let row = &report.crates()[0];
    assert_eq!((row.shipping_orphan(), row.test_orphan()), (0, 5));
    assert_eq!((row.shipping_counted(), row.test_counted()), (1, 10));
    // An exception of the module takes them out with its reason.
    let record =
        "exceptions:\n  - item: \"demo::tests::*\"\n    reason: \"Tests of the fixtures.\"\n";
    let excepting = lines_of(&ws, &["demo::run"], &["demo::tests::bound"], record);
    let report = excepting;
    assert_eq!(report.orphans(), []);
    assert_eq!(report.crates()[0].excepted(), 5);
    let excepted: Vec<(&str, String, &str, &str)> = report
        .excepted()
        .iter()
        .map(|row| {
            (
                row.lines().item(),
                row.lines().ranges_text(),
                row.exception(),
                row.reason(),
            )
        })
        .collect();
    assert_eq!(
        excepted,
        [
            (
                "demo::tests::unbound",
                "10-13".to_string(),
                "demo::tests::*",
                "Tests of the fixtures."
            ),
            (
                "demo::tests::unhelped",
                "17-17".to_string(),
                "demo::tests::*",
                "Tests of the fixtures."
            ),
        ]
    );
}

#[test]
fn a_package_that_no_capability_binds_is_reported_with_its_orphan_helper() {
    let ws = source(&[
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"2\"\n",
        ),
        ("crates/bound/Cargo.toml", &manifest("bound")),
        ("crates/bound/src/lib.rs", "pub fn run() {}\n"),
        ("crates/free/Cargo.toml", &manifest("free")),
        ("crates/free/src/lib.rs", "fn helper() {}\n"),
    ]);
    let report = report_of(&ws, &["bound::run"]);
    let rows: Vec<(&str, usize, usize)> = report
        .crates()
        .iter()
        .map(|row| (row.package(), row.counted(), row.orphan()))
        .collect();
    assert_eq!(rows, [("bound", 1, 0), ("free", 1, 1)]);
    assert_eq!(orphans(&report), pairs(&[("free::helper", "1-1")]));
    let free = report.package("free").expect("the package");
    assert_eq!(free.crates().len(), 1);
    assert_eq!(free.orphans().len(), 1);
    assert!(report.package("ghost").is_none());
}

#[test]
fn crlf_reads_as_lf_and_two_runs_give_the_same_bytes() {
    let ws = source(&[("src/lib.rs", HELPER_LIB)]);
    let crlf = source(&[("src/lib.rs", &HELPER_LIB.replace('\n', "\r\n"))]);
    let lf = report_of(&ws, &["demo::run"]);
    let crlf = report_of(&crlf, &["demo::run"]);
    assert_eq!(lf, crlf);
    assert_eq!(render::lines_table(&lf), render::lines_table(&crlf));
    let again = report_of(&ws, &["demo::run"]);
    assert_eq!(render::lines_table(&lf), render::lines_table(&again));
    assert_eq!(render::lines_json(&lf), render::lines_json(&again));
}

/// The text and the JSON of the helper fixture, with a ceiling of its
/// shipping lines and an exception of nothing at all.
const HELPER_TABLE: &str = "\
| Crate | Enforced | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| demo | yes | 6 | 1 | 0 | 5 | 5 | 0 | shipping 10 lines | 0 |

| Crate | Module | Counted | Orphan |
| --- | --- | --- | --- |
| demo | `demo` | 6 | 5 |

| Crate | Item | Kind | File | Orphan lines |
| --- | --- | --- | --- | --- |
| demo | `demo::helper` | fn | src/lib.rs | 3-7 |
";

const HELPER_JSON: &str = "[
{\"kind\":\"lines-crate\",\"crate\":\"demo\",\"enforced\":true,\"counted\":6,\"covered\":1,\"excepted\":0,\"orphan\":5,\"shipping_orphan\":5,\"test_orphan\":0,\"ceiling\":\"shipping 10 lines\",\"debt\":0},
{\"kind\":\"lines-module\",\"crate\":\"demo\",\"module\":\"demo\",\"counted\":6,\"orphan\":5},
{\"kind\":\"lines-orphan\",\"crate\":\"demo\",\"module\":\"demo\",\"item\":\"demo::helper\",\"unit\":\"fn\",\"file\":\"src/lib.rs\",\"ranges\":[[3,7]]}
]
";

#[test]
fn the_report_renders_as_three_tables_and_as_json_with_the_same_rows() {
    let ws = source(&[("src/lib.rs", HELPER_LIB)]);
    let docs = documents(
        &["demo::run"],
        &[],
        "strictness: strict\nceilings:\n  - crate: demo\n    lines: 10\n    of: shipping\n",
    );
    let report = Workplan::load(&docs, &ws).lines_report();
    assert_eq!(render::lines_table(&report), HELPER_TABLE);
    assert_eq!(render::lines_json(&report), HELPER_JSON);
    assert_eq!(report.to_string(), HELPER_TABLE);
    // Under `default` the ceilings are shown and not enforced.
    let docs = documents(
        &["demo::run"],
        &[],
        "ceilings:\n  - crate: demo\n    lines: 10\n    of: shipping\n",
    );
    let report = Workplan::load(&docs, &ws).lines_report();
    assert!(render::lines_table(&report).contains("| demo | no | 6 |"));
}

#[test]
fn with_strict_mode_off_the_report_is_printed_and_nothing_is_enforced() {
    let ws = source(&[("src/lib.rs", HELPER_LIB)]);
    assert_eq!(
        line_findings(&ws, &["demo::run"], "", Strictness::Default),
        []
    );
    let report = report_of(&ws, &["demo::run"]);
    assert_eq!(report.crates()[0].orphan(), 5);
    assert!(!report.crates()[0].enforced());
    let strict = line_findings(&ws, &["demo::run"], "", Strictness::Strict);
    let kinds: Vec<(&str, &Location)> = strict
        .iter()
        .map(|finding| (finding.kind, &finding.location))
        .collect();
    assert_eq!(
        kinds,
        [(
            "coverage-lines-over-ceiling",
            &Location::Key("ceilings".to_string())
        )]
    );
    assert!(
        strict[0].message.contains(
            "`demo` has 5 orphan lines (5 shipping, 0 test) of 6 counted, and its shipping orphan lines, 5 of 6 counted, are over the default of 0 lines"
        ),
        "{}",
        strict[0].message
    );
    assert!(
        strict[0]
            .message
            .ends_with("the largest orphan items are `demo::helper` (5)"),
        "{}",
        strict[0].message
    );
}

/// The library of the exception fixtures: a helper in a module, a generated
/// table in a file of its own and a support directory, none of them bound.
fn excepted_source() -> Workspace {
    source(&[
        (
            "src/lib.rs",
            "pub fn run() {}\n\npub mod text {\n    pub fn pad() {}\n\n    pub fn trim() {}\n}\n\nmod table;\nmod support;\n",
        ),
        ("src/table.rs", "pub(crate) const ROWS: [u8; 2] = [1, 2];\n"),
        ("src/support/mod.rs", "pub(crate) fn build() {}\n"),
    ])
}

/// The excepted lines of each unit, with the exception that takes them out.
fn excepted(report: &LinesReport) -> Vec<(String, String)> {
    report
        .excepted()
        .iter()
        .map(|row| (row.lines().item().to_string(), row.exception().to_string()))
        .collect()
}

#[test]
fn an_exception_by_item_prefix_file_and_directory_takes_its_lines_out_and_one_beside_it_that_matches_nothing_is_a_finding()
 {
    let ws = excepted_source();
    let cases = [
        (
            "item: demo::text::pad",
            "item: demo::text::ghost",
            vec![("demo::text::pad", "demo::text::pad")],
        ),
        (
            "item: \"demo::text::*\"",
            "item: \"demo::ghost::*\"",
            vec![
                ("demo::text::pad", "demo::text::*"),
                ("demo::text::trim", "demo::text::*"),
            ],
        ),
        (
            "file: src/table.rs",
            "file: src/ghost.rs",
            vec![("demo::table::ROWS", "src/table.rs")],
        ),
        (
            "file: src/support/",
            "file: src/ghost/",
            vec![("demo::support::build", "src/support/")],
        ),
    ];
    for (matching, unmatched, want) in cases {
        let record = format!(
            "exceptions:\n  - {matching}\n    reason: \"Explained elsewhere.\"\n  - {unmatched}\n    reason: \"Nothing is there.\"\n"
        );
        let report = lines_of(&ws, &["demo::run"], &[], &record);
        let want: Vec<(String, String)> = want
            .iter()
            .map(|(item, exception)| ((*item).to_string(), (*exception).to_string()))
            .collect();
        assert_eq!(excepted(&report), want, "{matching}");
        assert!(
            report
                .excepted()
                .iter()
                .all(|row| row.reason() == "Explained elsewhere."),
            "{matching}"
        );
        // The unmatched one is a finding under either strictness, and the
        // matching one is not.
        for strictness in [Strictness::Default, Strictness::Strict] {
            let found: Vec<String> = line_findings(&ws, &["demo::run"], &record, strictness)
                .into_iter()
                .filter(|finding| finding.kind == "coverage-exception-unmatched")
                .map(|finding| finding.message)
                .collect();
            let [message] = found.as_slice() else {
                panic!("{matching}: {found:#?}");
            };
            let written = unmatched
                .split_once(": ")
                .expect("a row")
                .1
                .trim_matches('"');
            assert!(message.contains(&format!("`{written}`")), "{message}");
        }
    }
    // An exception's lines are not orphan lines, and an item exception of a
    // module takes out what is below it.
    let record = "exceptions:\n  - item: demo::text\n    reason: \"Text helpers.\"\n";
    let report = lines_of(&ws, &["demo::run"], &[], record);
    assert_eq!(
        excepted(&report),
        pairs(&[
            ("demo::text::pad", "demo::text"),
            ("demo::text::trim", "demo::text")
        ])
    );
    assert_eq!(
        orphans(&report),
        pairs(&[
            ("demo::support::build", "1-1"),
            ("demo::table::ROWS", "1-1")
        ])
    );
}

/// The library of the ceiling fixtures: three shipping orphan lines and two
/// test orphan lines, beside the bound `run`.
const CEILING_LIB: &str = "\
pub fn run() {}

fn one() {
    run();
}

#[cfg(test)]
mod tests {
    #[test]
    fn two() {}
}
";

/// The kinds and messages of the line findings under strict of `record`.
fn strict_over(record: &str) -> Vec<String> {
    let ws = source(&[("src/lib.rs", CEILING_LIB)]);
    line_findings(&ws, &["demo::run"], record, Strictness::Strict)
        .into_iter()
        .map(|finding| {
            assert_eq!(finding.kind, "coverage-lines-over-ceiling");
            finding.message
        })
        .collect()
}

fn ceilings(rows: &[&str]) -> String {
    let mut text = String::from("ceilings:\n");
    for row in rows.iter().filter(|row| !row.is_empty()) {
        let _ = write!(text, "  - crate: demo\n{row}");
    }
    text
}

#[test]
fn a_ceiling_of_lines_passes_at_its_count_and_fails_one_below_for_each_class() {
    // Three shipping and two test orphan lines.
    let ws = source(&[("src/lib.rs", CEILING_LIB)]);
    let row = lines_of(&ws, &["demo::run"], &[], "").crates()[0].clone();
    assert_eq!((row.shipping_orphan(), row.test_orphan()), (3, 2));
    for (of, count) in [("shipping", 3), ("test", 2), ("all", 5)] {
        // The other class is bounded too, so only this one is judged.
        let others = match of {
            "shipping" => "    lines: 2\n    of: test\n",
            "test" => "    lines: 3\n    of: shipping\n",
            _ => "",
        };
        let at = format!("    lines: {count}\n    of: {of}\n");
        assert!(strict_over(&ceilings(&[&at, others])).is_empty(), "{of}");
        let below = format!("    lines: {}\n    of: {of}\n", count - 1);
        let found = strict_over(&ceilings(&[&below, others]));
        assert_eq!(found.len(), 1, "{of}: {found:#?}");
        let words = if of == "all" {
            "its orphan lines".to_string()
        } else {
            format!("its {of} orphan lines")
        };
        assert!(found[0].contains(&words), "{}", found[0]);
        assert!(
            found[0].contains(&format!(
                "its ceiling of {}{} lines",
                if of == "all" {
                    String::new()
                } else {
                    format!("{of} ")
                },
                count - 1
            )),
            "{}",
            found[0]
        );
    }
}

#[test]
fn a_shipping_row_alone_judges_the_shipping_lines_against_itself_and_holds_the_test_lines_to_zero()
{
    let found = strict_over(&ceilings(&["    lines: 3\n    of: shipping\n"]));
    let [message] = found.as_slice() else {
        panic!("{found:#?}");
    };
    assert!(
        message.contains("its test orphan lines, 2 of 2 counted, are over the default of 0 lines"),
        "{message}"
    );
    // With no row both classes are held to 0, and an `all` row bounds both.
    assert_eq!(strict_over("").len(), 2);
    assert_eq!(
        strict_over(&ceilings(&["    lines: 5\n"])),
        Vec::<String>::new()
    );
}

#[test]
fn a_percent_ceiling_compares_whole_numbers_at_its_boundary_and_an_empty_package_passes_it() {
    // Five orphan lines of six counted: 83.3...%.
    let ws = source(&[("src/lib.rs", CEILING_LIB)]);
    let row = lines_of(&ws, &["demo::run"], &[], "").crates()[0].clone();
    assert_eq!((row.orphan(), row.counted()), (5, 6));
    // 5 * 100 * 10000 <= 833334 * 6 holds, and 5 * 100 * 10000 <= 833333 * 6
    // does not.
    assert_eq!(
        strict_over(&ceilings(&["    percent: 83.3334\n"])),
        Vec::<String>::new()
    );
    assert_eq!(strict_over(&ceilings(&["    percent: 83.3333\n"])).len(), 1);
    assert_eq!(
        strict_over(&ceilings(&["    percent: 100\n"])),
        Vec::<String>::new()
    );
    // A package with no counted line passes a percent ceiling of 0.
    let empty = source(&[
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"2\"\n",
        ),
        ("crates/bound/Cargo.toml", &manifest("bound")),
        ("crates/bound/src/lib.rs", "pub fn run() {}\n"),
        ("crates/empty/Cargo.toml", &manifest("empty")),
        ("crates/empty/src/lib.rs", "//! Nothing here.\n"),
    ]);
    let record = "ceilings:\n  - crate: empty\n    percent: 0\n";
    assert_eq!(
        line_findings(&empty, &["bound::run"], record, Strictness::Strict),
        []
    );
    let report = lines_of(&empty, &["bound::run"], &[], record);
    let row = report.package("empty").expect("the package").crates()[0].clone();
    assert_eq!((row.counted(), row.ceiling_text()), (0, "0%".to_string()));
}

#[test]
fn a_ceiling_row_of_no_package_is_scope_crate_missing_at_ceilings() {
    let ws = source(&[("src/lib.rs", "pub fn run() {}\n")]);
    let record = "ceilings:\n  - crate: ghost\n    lines: 0\n  - crate: demo\n    lines: 0\n";
    for strictness in [Strictness::Default, Strictness::Strict] {
        let found = line_findings(&ws, &["demo::run"], record, strictness);
        let kinds: Vec<(&str, &Location)> = found
            .iter()
            .map(|finding| (finding.kind, &finding.location))
            .collect();
        assert_eq!(
            kinds,
            [(
                "scope-crate-missing",
                &Location::Key("ceilings".to_string())
            )],
            "{found:#?}"
        );
        assert!(found[0].message.contains("`ghost`"), "{}", found[0].message);
    }
}
