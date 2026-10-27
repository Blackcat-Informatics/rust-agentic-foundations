// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The work of the line report, counted and never timed, the exact
//! comparison of a percent ceiling, the paths that may name what a macro
//! generates, and a walk that fails.
//!
//! Each unit is swept once over its lines and looks up its own paths and the
//! modules above them in the index of the exceptions, which is built once, so
//! the work grows with the units plus the exceptions and not with their
//! product.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use raf_okf::{CeilingLimit, Number};
use raf_source::{LoadOptions, MemoryTree, Namespace, Node, Workspace};

use super::{compute, lex_runs, within};
use crate::Workplan;
use crate::coverage::render;
use crate::strict::tests::fixture;
use crate::work::{Work, measure, measure_kind};

/// The functions of a module of the crate `big`.
const FUNCTIONS: usize = 10;
/// The modules of one unit of units.
const MODULES_PER_UNIT: usize = 10;
/// The exceptions of one unit of exceptions.
const EXCEPTIONS_PER_UNIT: usize = 20;
/// What one unit of the report, or one exception, may cost at most: a sweep,
/// one mark for its one line and a lookup for each segment of its path and
/// each module above it.
const PER_ENTRY: usize = 8;

/// The work of the report of a crate of `modules` modules of ten private
/// functions beside a bound `run`, under `exceptions` exceptions of items
/// that are no units, with every kind of work held to `ceiling`; and the
/// units the report swept.
fn work(modules: usize, exceptions: usize, ceiling: usize) -> (usize, usize) {
    let mut lib = String::from("pub fn run() {}\n");
    for m in 0..modules {
        let _ = writeln!(lib, "mod m{m} {{");
        for f in 0..FUNCTIONS {
            let _ = writeln!(lib, "    fn f{f}() {{}}");
        }
        let _ = writeln!(lib, "}}");
    }
    let source = Workspace::load(
        &MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname = \"big\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            ("src/lib.rs", lib.as_str()),
        ]),
        &["."],
    )
    .expect("a workspace");
    let mut record =
        String::from("---\ntype: Coverage\nid: coverage:default\nscope: []\nexclusions: []\n");
    if exceptions > 0 {
        record.push_str("exceptions:\n");
        for e in 0..exceptions {
            let _ = write!(
                record,
                "  - item: big::m0::f0::x{e}\n    reason: \"Nothing of it.\"\n"
            );
        }
    }
    record.push_str("---\n");
    let capability = "---\ntype: Capability\nid: cap:big\ntitle: Big\ndescription: Big.\n\
         state: implementing\nbindings:\n  code: [big::run]\n  tests: []\n  examples: []\n\
         depends: []\n---\n\n# Big\n\n## Purpose\n\nText.\n\n## Contract\n\nText.\n\n## Usage\n\n\
         Text.\n\n## Examples\n\nText.\n\n## Constraints\n\nText.\n\n## Explanation\n\nText.\n\n\
         ## Verification\n\nText.\n";
    let docs = MemoryTree::from([
        ("docs/capabilities/big.md", capability),
        ("work/coverage/default.md", record.as_str()),
    ]);
    let plan = Workplan::load(&docs, &source);
    let (report, tally) = measure(ceiling, || plan.lines_report());
    assert!(report.errors().is_empty(), "{:?}", report.errors());
    assert_eq!(report.orphans().len(), modules * FUNCTIONS);
    (tally.total(), tally.get(Work::LineUnits))
}

/// The units of `big` with `modules` modules: its functions and `run`.
const fn units_of(modules: usize) -> usize {
    modules * FUNCTIONS + 1
}

#[test]
fn the_work_grows_with_the_units_and_no_faster_at_fixed_exceptions() {
    let exceptions = 3 * EXCEPTIONS_PER_UNIT;
    let mut total = [0; 3];
    for units in 1..=3 {
        let (work, swept) = work(units * MODULES_PER_UNIT, exceptions, usize::MAX);
        assert_eq!(swept, units_of(units * MODULES_PER_UNIT));
        total[units - 1] = work;
    }
    let [one, two, three] = total;
    assert!(one > 0);
    assert!(three + one <= 2 * two, "{one}, {two}, {three}");
}

#[test]
fn the_work_grows_with_the_exceptions_and_no_faster_at_fixed_units() {
    let modules = 2 * MODULES_PER_UNIT;
    let mut total = [0; 3];
    for units in 1..=3 {
        total[units - 1] = work(modules, units * EXCEPTIONS_PER_UNIT, usize::MAX).0;
    }
    let [one, two, three] = total;
    assert!(one > 0);
    assert!(three + one <= 2 * two, "{one}, {two}, {three}");
}

/// Every kind of work is held to a ceiling linear in the units plus the
/// exceptions, below their product, so a report that tests each exception
/// against each unit passes the ceiling and stops at once.
#[test]
fn every_kind_of_work_is_held_to_a_ceiling_linear_in_the_units_and_the_exceptions() {
    let modules = 3 * MODULES_PER_UNIT;
    let exceptions = 3 * EXCEPTIONS_PER_UNIT;
    let units = units_of(modules);
    let ceiling = PER_ENTRY * (units + exceptions);
    assert!(ceiling < units * exceptions);
    let (_, swept) = work(modules, exceptions, ceiling);
    assert_eq!(swept, units);
}

#[test]
fn the_default_check_sweeps_each_counted_line_unit() {
    let tree = |files: Vec<(&'static str, String)>| -> MemoryTree {
        files
            .iter()
            .map(|(path, text)| (*path, text.as_str()))
            .collect()
    };
    let source = Workspace::load(&tree(fixture::source_files()), &["."]).expect("the source");
    let docs = tree(fixture::documents());
    let plan = Workplan::load(&docs, &source);
    let lines = |found: Vec<crate::Finding>| -> Vec<crate::Finding> {
        found
            .into_iter()
            .filter(|finding| finding.kind == "coverage-lines-over-ceiling")
            .collect()
    };
    let (found, tally) = measure_kind(Work::LineUnits, usize::MAX, || plan.check());
    // The function and its documentation test, and the test.
    assert_eq!(tally.get(Work::LineUnits), 2);
    assert_eq!(lines(found), []);
}

#[test]
fn a_percent_is_compared_as_whole_numbers_without_division_or_overflow() {
    let percent = |text: &str| CeilingLimit::Percent(Number::parse(text).expect("a number"));
    // 1 of 3 is 33.33...%.
    assert!(within(1, 3, &percent("33.334")));
    assert!(!within(1, 3, &percent("33.333")));
    assert!(within(1, 3, &percent("33.3340000")));
    assert!(within(0, 0, &percent("0")));
    assert!(!within(1, 0, &percent("100")));
    // A fraction too long for any machine word compares exactly.
    let long = format!("33.{}4", "3".repeat(80));
    assert!(within(1, 3, &percent(&long)));
    let short = format!("33.{}", "3".repeat(80));
    assert!(!within(1, 3, &percent(&short)));
    assert!(within(usize::MAX, usize::MAX, &percent("100")));
    assert!(within(1, 8, &percent("12.5")));
    assert!(!within(2, 8, &percent("12.5")));
    assert!(within(0, 3, &percent("-0.000")));
    assert!(!within(1, 3, &percent("-0.000")));
    let lines = |text: &str| CeilingLimit::Lines(Number::parse(text).expect("a number"));
    assert!(within(3, 9, &lines("3")));
    assert!(!within(4, 9, &lines("3")));
    assert!(within(4, 9, &lines("99999999999999999999999999")));
}

#[test]
fn a_path_that_may_name_what_a_macro_generates_is_a_run_and_a_method_a_field_or_a_declared_name_is_not()
 {
    let wanted: BTreeSet<String> = ["made".to_string()].into();
    let texts = |text: &str| -> Vec<String> {
        lex_runs(text, &wanted)
            .expect("lexes")
            .into_iter()
            .map(|run| run.text)
            .collect()
    };
    assert_eq!(
        texts("fn f() { made(); a::made(); ::b::made; crate::made(); }"),
        ["made", "a::made", "::b::made", "crate::made"]
    );
    assert_eq!(
        texts("fn f() { x.made(); let made = 1; fn made() {} }"),
        [""; 0]
    );
    assert_eq!(texts("S { made: 1 }; fn g(made: u8) {}"), [""; 0]);
    assert_eq!(texts("fn f() { let made = || {}; made(); }"), ["made"]);
    assert_eq!(texts("made::other(); other::made"), ["other::made"]);
    // Offsets are those of the text, past a byte order mark.
    let text = "\u{feff}fn f() { made() }";
    let runs = lex_runs(text, &wanted).expect("lexes");
    assert_eq!(runs[0].start, text.find("made").expect("made"));
}

/// Two workspaces: `demo` with one function, and `wide` with three.
fn demo_and_wide() -> (Workspace, Workspace) {
    let small = Workspace::load(
        &MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            ("src/lib.rs", "pub fn run() {}\n"),
        ]),
        &["."],
    )
    .expect("a workspace");
    let large = Workspace::load(
        &MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname = \"wide\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            (
                "src/lib.rs",
                "pub fn a() {}\npub fn b() {}\npub fn c() {}\n",
            ),
        ]),
        &["."],
    )
    .expect("a workspace");
    (small, large)
}

/// The table and the JSON of the report of a failed walk, whose message is
/// the one the walk gives.
#[test]
fn a_walk_that_fails_retains_a_single_typed_error_and_the_error_table() {
    auxiliary_query_refusals();
    let (small, large) = demo_and_wide();
    let foreign = large
        .resolve_in("wide::c", Namespace::Value)
        .expect("the item")
        .sites[0];
    let docs = MemoryTree::from([(
        "work/coverage/default.md",
        "---\ntype: Coverage\nid: coverage:default\nscope: []\nexclusions: []\n---\n",
    )]);
    let plan = Workplan::load(&docs, &small);
    let report = compute(&plan, &[Node::Item(foreign)]);
    let message = report.errors()[0].clone();
    assert_eq!(
        render::lines_table(&report),
        format!(
            "| Crate | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |\n\
             | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n\n\
             | Crate | Module | Counted | Orphan |\n| --- | --- | --- | --- |\n\n\
             | Crate | Item | Kind | File | Orphan lines |\n| --- | --- | --- | --- | --- |\n\n\
             | Error |\n| --- |\n| {message} |\n"
        )
    );
    assert_eq!(report.errors(), &[message]);
    assert_eq!(report.crates(), []);
    assert_eq!(report.orphans(), []);
}

fn auxiliary_query_refusals() {
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        ),
        (
            "src/lib.rs",
            "#[macro_export] macro_rules! make { () => {} }\n\
             mod carrier { pub use crate::make; pub fn exported() {} }\n\
             mod child { use crate::carrier::*; make!(); } pub fn good() {}",
        ),
    ]);
    for limited in [false, true] {
        let mut options = LoadOptions::default();
        if limited {
            options.max_query_work = 0;
        }
        let source = Workspace::load_with(&tree, &["."], &options).expect("workspace");
        let missing = super::missing_run(&source, "demo", "self::missing");
        let present = super::missing_run(&source, "demo::child", "self::exported");
        assert_eq!(missing, Ok(true));
        let mut units =
            super::collect(&source, &BTreeSet::from(["demo"])).expect("actual source units");
        let mut roots = super::Roots {
            nodes: BTreeSet::new(),
            opened: BTreeSet::from(["demo::child".into()]),
            whole: BTreeSet::new(),
        };
        let covered = super::cover(&source, &mut units, &mut roots, &[]);
        if limited {
            let error = present.expect_err("glob query refused");
            assert!(
                error.contains("from `demo::child`") && error.contains("budget"),
                "{error}"
            );
            let error = covered.expect_err("macro lookup refuses before ancestor fallback");
            assert!(
                error.contains("macro lookup")
                    && error.contains("make")
                    && error.contains("budget"),
                "{error}"
            );
        } else {
            assert_eq!(present, Ok(false));
            covered.expect("macro resolves with the normal allowance");
        }
    }
}

#[test]
fn a_walk_that_fails_is_the_one_error_of_the_report_and_one_finding_under_strict() {
    let (small, large) = demo_and_wide();
    // An item of a larger workspace is no item of this one.
    let foreign = large
        .resolve_in("wide::c", Namespace::Value)
        .expect("the item")
        .sites[0];
    let docs = MemoryTree::from([(
        "work/coverage/default.md",
        "---\ntype: Coverage\nid: coverage:default\nscope: []\nexclusions: []\n---\n",
    )]);
    let plan = Workplan::load(&docs, &small);
    let report = compute(&plan, &[Node::Item(foreign)]);
    assert_eq!(report.errors().len(), 1, "{report:#?}");
    assert!(
        report.errors()[0].starts_with("the walk of the bound items failed:"),
        "{:?}",
        report.errors()
    );
    assert!(report.crates().is_empty() && !report.is_complete());
    let loaded = plan.coverage().expect("the record");
    let found = super::ceiling_findings(loaded, &report);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].package, None);
    assert_eq!(found[0].finding.kind, "coverage-lines-over-ceiling");
    assert!(
        found[0].finding.message.contains("cannot be counted"),
        "{}",
        found[0].finding.message
    );
    // The same plan without the foreign root counts its lines.
    assert_eq!(compute(&plan, &[]).errors(), [""; 0]);
}
