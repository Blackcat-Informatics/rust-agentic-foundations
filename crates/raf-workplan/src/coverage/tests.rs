// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The work of judging coverage, counted: every public item and every test of
//! a crate is judged once, and each exclusion of the record is tested against
//! each of them, because an exclusion that matches nothing is reported and so
//! every exclusion must be tested against every entry, not only until the
//! first match.

use std::fmt::Write as _;

use raf_source::{MemoryTree, Workspace};

use crate::Workplan;
use crate::work::{Work, measure_kind};

/// The functions of a module of the crate `big`.
const FUNCTIONS: usize = 10;
/// The tests of the crate `big`, `TESTS_PER_MODULE` for each of `TEST_MODULES`.
const TESTS_PER_MODULE: usize = 10;
const TEST_MODULES: usize = 12;
/// The public items of the module count one each, and the module itself one.
const ENTRIES_PER_MODULE: usize = FUNCTIONS + 1;
/// The modules of one unit of entries.
const MODULES_PER_UNIT: usize = 10;
/// The exclusions of one unit of exclusions.
const EXCLUSIONS_PER_UNIT: usize = 20;

/// What judging counted, and what the report said.
struct Judged {
    entries: usize,
    exclusion_tests: usize,
    reported: usize,
    excluded: usize,
}

/// The names of the tests of `big`, the first `count` of them in the order of
/// the exclusions.
fn test_names(count: usize) -> Vec<String> {
    (0..TEST_MODULES)
        .flat_map(|m| (0..TESTS_PER_MODULE).map(move |i| format!("big::test_crate::it::t{m}_{i}")))
        .take(count)
        .collect()
}

/// Judge a crate of `modules` modules of ten public functions and
/// `TEST_MODULES * TESTS_PER_MODULE` tests, half the modules bound by a
/// capability and `exclusions` of the tests excluded by name, one exclusion
/// each, with every kind of work held to `ceiling`; and the work counted.
fn judged(modules: usize, exclusions: usize, ceiling: usize) -> Judged {
    let mut lib = String::new();
    for m in 0..modules {
        let _ = writeln!(lib, "pub mod m{m} {{");
        for i in 0..FUNCTIONS {
            let _ = writeln!(lib, "    pub fn f{i}() {{}}");
        }
        let _ = writeln!(lib, "}}");
    }
    let mut tests = String::new();
    for m in 0..TEST_MODULES {
        for i in 0..TESTS_PER_MODULE {
            let _ = writeln!(tests, "#[test]\nfn t{m}_{i}() {{}}");
        }
    }
    let source = Workspace::load(
        &MemoryTree::from([
            (
                "crates/big/Cargo.toml",
                "[package]\nname = \"big\"\nversion = \"0.1.0\"\n",
            ),
            ("crates/big/src/lib.rs", lib.as_str()),
            ("crates/big/tests/it.rs", tests.as_str()),
        ]),
        &["crates"],
    )
    .expect("a workspace");

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
        let _ = write!(sections, "## {name}\n\nText.\n\n");
    }
    let bound: Vec<String> = (0..modules / 2).map(|m| format!("big::m{m}")).collect();
    let capability = format!(
        "---\ntype: Capability\nid: cap:omega\ntitle: Omega\ndescription: Omega.\n\
         state: implementing\nbindings:\n  code: [{}]\n  tests: []\n  examples: []\n\
         depends: []\n---\n\n# Omega\n\n{sections}",
        bound.join(", ")
    );
    let mut coverage =
        String::from("---\ntype: Coverage\nid: coverage:default\nscope: [big]\nexclusions:");
    let names = test_names(exclusions);
    if names.is_empty() {
        coverage.push_str(" []");
    }
    for name in &names {
        let _ = write!(coverage, "\n  - item: \"{name}\"\n    reason: \"Load.\"");
    }
    coverage.push_str("\n---\n");
    let docs = MemoryTree::from([
        ("docs/capabilities/omega.md", capability.as_str()),
        ("work/coverage/default.md", coverage.as_str()),
    ]);
    let plan = Workplan::load(&docs, &source);

    let (report, tally) = measure_kind(Work::ExclusionTests, ceiling, || plan.coverage_report());
    let big = report.package("big").expect("big");
    let counts = &big.packages()[0];
    // The count is of what the report says as well as of what was ticked.
    assert_eq!(counts.items(), modules * ENTRIES_PER_MODULE);
    assert_eq!(counts.tests(), TEST_MODULES * TESTS_PER_MODULE);
    assert_eq!(counts.covered_items(), modules / 2 * ENTRIES_PER_MODULE);
    Judged {
        entries: tally.get(Work::EntriesJudged),
        exclusion_tests: tally.get(Work::ExclusionTests),
        reported: big.code().len() + big.tests().len(),
        excluded: big.excluded().len(),
    }
}

/// Every entry is judged once, and every exclusion is tested against every
/// entry: the tests are the product of the two, exactly.
#[test]
fn every_exclusion_is_tested_against_every_entry_exactly_once() {
    let modules = 4;
    let exclusions = 30;
    let entries = modules * ENTRIES_PER_MODULE + TEST_MODULES * TESTS_PER_MODULE;
    let work = judged(modules, exclusions, entries * exclusions);
    assert_eq!(work.entries, entries);
    assert_eq!(work.exclusion_tests, entries * exclusions);
    // Judging reports what it judged: the unbound half of the items and the
    // tests that no exclusion takes, and the excluded tests.
    assert_eq!(work.excluded, exclusions);
    assert_eq!(
        work.reported,
        modules / 2 * ENTRIES_PER_MODULE + TEST_MODULES * TESTS_PER_MODULE - exclusions
    );
}

/// With no exclusion nothing is tested, and the entries are still each judged
/// once.
#[test]
fn without_exclusions_the_entries_are_judged_and_nothing_is_tested() {
    let modules = 4;
    let work = judged(modules, 0, 0);
    assert_eq!(
        work.entries,
        modules * ENTRIES_PER_MODULE + TEST_MODULES * TESTS_PER_MODULE
    );
    assert_eq!(work.exclusion_tests, 0);
}

/// With the exclusions fixed, the tests of exclusions grow with the entries:
/// one, two and three units of modules, each entry tested against all of them.
#[test]
fn the_exclusions_tested_grow_with_the_entries_and_no_faster() {
    let exclusions = 3 * EXCLUSIONS_PER_UNIT;
    let mut tested = [0; 3];
    for units in 1..=3 {
        let modules = units * MODULES_PER_UNIT;
        let entries = modules * ENTRIES_PER_MODULE + TEST_MODULES * TESTS_PER_MODULE;
        let work = judged(modules, exclusions, entries * exclusions);
        assert_eq!(work.entries, entries, "{units} units");
        assert_eq!(work.exclusion_tests, entries * exclusions, "{units} units");
        tested[units - 1] = work.exclusion_tests;
    }
    let [one, two, three] = tested;
    assert!(one > 0);
    assert!(three + one <= 2 * two, "{one}, {two}, {three}");
}

/// With the entries fixed, the tests of exclusions grow with the exclusions:
/// one, two and three units of them, each tested against every entry.
#[test]
fn the_exclusions_tested_grow_with_the_exclusions_and_no_faster() {
    let modules = 2 * MODULES_PER_UNIT;
    let entries = modules * ENTRIES_PER_MODULE + TEST_MODULES * TESTS_PER_MODULE;
    let mut tested = [0; 3];
    for units in 1..=3 {
        let exclusions = units * EXCLUSIONS_PER_UNIT;
        let work = judged(modules, exclusions, entries * exclusions);
        assert_eq!(work.entries, entries, "{units} units");
        assert_eq!(work.exclusion_tests, entries * exclusions, "{units} units");
        tested[units - 1] = work.exclusion_tests;
    }
    let [one, two, three] = tested;
    assert!(one > 0);
    assert!(three + one <= 2 * two, "{one}, {two}, {three}");
}
