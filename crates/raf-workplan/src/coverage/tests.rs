// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The work of judging coverage, counted, and the index of the exclusions held
//! to the reading of an exclusion it stands for.
//!
//! Every public item and every test of a crate is judged once. The exclusions
//! of the record are put into an index once, and judging an entry looks up each
//! of its paths and each module above them, so the work grows with the entries
//! plus the exclusions. The index finds every exclusion that matches an entry,
//! not only the first: each is marked, so an exclusion that matches nothing is
//! reported, and the first in the order of the record gives the reason. The
//! reading the index stands for is the one stated here as `oracle`: each
//! exclusion of the record tested against each path of the entry.

use std::fmt::Write as _;

use raf_okf::{Exclusion, ExclusionItem};
use raf_source::{MemoryTree, Workspace};

use super::ExclusionIndex;
use crate::Workplan;
use crate::finding::kind;
use crate::work::{Work, measure};

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
/// The paths an entry of `big` is matched by: its exported path and its
/// defining path, which are the same there.
const PATHS_PER_ENTRY: usize = 2;
/// The most segments of a path of an entry of `big`: `big::test_crate::it::t0_0`.
const DEEPEST: usize = 4;
/// The most lookups judging one entry of `big` makes: one for each segment of
/// each of its paths.
const LOOKUPS_PER_ENTRY: usize = PATHS_PER_ENTRY * DEEPEST;

/// What judging counted, and what the report said.
struct Judged {
    entries: usize,
    indexed: usize,
    lookups: usize,
    marks: usize,
    total: usize,
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

/// The paths of the entries of `big` with `modules` modules: each module, each
/// of its functions, and each test.
fn entry_paths(modules: usize) -> Vec<String> {
    let mut paths = Vec::new();
    for m in 0..modules {
        paths.push(format!("big::m{m}"));
        paths.extend((0..FUNCTIONS).map(|i| format!("big::m{m}::f{i}")));
    }
    paths.extend(test_names(TEST_MODULES * TESTS_PER_MODULE));
    paths
}

/// The entries of `big` with `modules` modules.
const fn entries_of(modules: usize) -> usize {
    modules * ENTRIES_PER_MODULE + TEST_MODULES * TESTS_PER_MODULE
}

/// A ceiling linear in the entries plus the exclusions: what an index may
/// count of any kind of work, and less than what testing each exclusion
/// against each entry counts once both are more than a few.
const fn linear(entries: usize, exclusions: usize) -> usize {
    LOOKUPS_PER_ENTRY * (entries + exclusions)
}

/// The sections every capability of these fixtures has.
fn sections() -> String {
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
    sections
}

/// The documents: a capability that binds `code`, and a coverage record of
/// `package` with an exclusion of each of `exclusions`.
fn documents(package: &str, code: &[String], exclusions: &[String]) -> MemoryTree {
    let capability = format!(
        "---\ntype: Capability\nid: cap:omega\ntitle: Omega\ndescription: Omega.\n\
         state: implementing\nbindings:\n  code: [{}]\n  tests: []\n  examples: []\n\
         depends: []\n---\n\n# Omega\n\n{}",
        code.join(", "),
        sections()
    );
    let mut coverage =
        format!("---\ntype: Coverage\nid: coverage:default\nscope: [{package}]\nexclusions:");
    if exclusions.is_empty() {
        coverage.push_str(" []");
    }
    for name in exclusions {
        let _ = write!(
            coverage,
            "\n  - item: \"{name}\"\n    reason: \"For {name}.\""
        );
    }
    coverage.push_str("\n---\n");
    MemoryTree::from([
        ("docs/capabilities/omega.md", capability.as_str()),
        ("work/coverage/default.md", coverage.as_str()),
    ])
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
    let bound: Vec<String> = (0..modules / 2).map(|m| format!("big::m{m}")).collect();
    let names = test_names(exclusions);
    // Every exclusion asked for is in the record.
    assert_eq!(names.len(), exclusions);
    let docs = documents("big", &bound, &names);
    let plan = Workplan::load(&docs, &source);

    let (report, tally) = measure(ceiling, || plan.coverage_report());
    let big = report.package("big").expect("big");
    let counts = &big.packages()[0];
    // The count is of what the report says as well as of what is ticked.
    assert_eq!(counts.items(), modules * ENTRIES_PER_MODULE);
    assert_eq!(counts.tests(), TEST_MODULES * TESTS_PER_MODULE);
    assert_eq!(counts.covered_items(), modules / 2 * ENTRIES_PER_MODULE);
    Judged {
        entries: tally.get(Work::EntriesJudged),
        indexed: tally.get(Work::ExclusionsIndexed),
        lookups: tally.get(Work::IndexLookups),
        marks: tally.get(Work::ExclusionMarks),
        total: tally.total(),
        reported: big.code().len() + big.tests().len(),
        excluded: big.excluded().len(),
    }
}

/// Each exclusion is put into the index once, each entry looks up each of its
/// paths and each module above them, and that is the same for ten times the
/// exclusions: the lookups do not depend on how many exclusions there are.
#[test]
fn the_exclusions_are_indexed_once_and_each_entry_looks_up_its_paths_and_their_modules() {
    let modules = 4;
    let entries = entries_of(modules);
    let lookups: usize = entry_paths(modules)
        .iter()
        .map(|path| PATHS_PER_ENTRY * path.split("::").count())
        .sum();
    let mut seen = Vec::new();
    for exclusions in [12, 120] {
        let work = judged(modules, exclusions, linear(entries, exclusions));
        assert_eq!(work.entries, entries, "{exclusions}");
        assert_eq!(work.indexed, exclusions, "{exclusions}");
        assert_eq!(work.lookups, lookups, "{exclusions}");
        // Each excluded test is found once by each of its two paths.
        assert_eq!(work.marks, PATHS_PER_ENTRY * exclusions, "{exclusions}");
        // Judging reports what it judged: the unbound half of the items and
        // the tests that no exclusion takes, and the excluded tests.
        assert_eq!(work.excluded, exclusions, "{exclusions}");
        assert_eq!(
            work.reported,
            modules / 2 * ENTRIES_PER_MODULE + TEST_MODULES * TESTS_PER_MODULE - exclusions,
            "{exclusions}"
        );
        seen.push(work.lookups);
    }
    assert_eq!(seen[0], seen[1]);
}

/// With no exclusion nothing is indexed, looked up or marked, and the entries
/// are still each judged once.
#[test]
fn without_exclusions_the_entries_are_judged_and_nothing_is_looked_up() {
    let modules = 4;
    let entries = entries_of(modules);
    let work = judged(modules, 0, entries);
    assert_eq!(work.entries, entries);
    assert_eq!(work.indexed, 0);
    assert_eq!(work.lookups, 0);
    assert_eq!(work.marks, 0);
    assert_eq!(work.total, entries);
}

/// With the exclusions fixed, the work grows with the entries and no faster:
/// one, two and three units of modules.
#[test]
fn the_work_grows_with_the_entries_and_no_faster() {
    let exclusions = 3 * EXCLUSIONS_PER_UNIT;
    let mut total = [0; 3];
    let mut entries = [0; 3];
    for units in 1..=3 {
        let work = judged(units * MODULES_PER_UNIT, exclusions, usize::MAX);
        total[units - 1] = work.total;
        entries[units - 1] = work.entries;
    }
    let [one, two, three] = total;
    assert!(one > 0);
    assert!(three + one <= 2 * two, "{one}, {two}, {three}");
    assert_eq!(
        entries,
        [1, 2, 3].map(|units| entries_of(units * MODULES_PER_UNIT))
    );
}

/// With the entries fixed, the work grows with the exclusions and no faster:
/// one, two and three units of them.
#[test]
fn the_work_grows_with_the_exclusions_and_no_faster() {
    let modules = 2 * MODULES_PER_UNIT;
    let mut total = [0; 3];
    let mut indexed = [0; 3];
    for units in 1..=3 {
        let work = judged(modules, units * EXCLUSIONS_PER_UNIT, usize::MAX);
        total[units - 1] = work.total;
        indexed[units - 1] = work.indexed;
    }
    let [one, two, three] = total;
    assert!(one > 0);
    assert!(three + one <= 2 * two, "{one}, {two}, {three}");
    assert_eq!(indexed, [1, 2, 3].map(|units| units * EXCLUSIONS_PER_UNIT));
}

/// With the entries and the exclusions grown together, one, two and three
/// units of each, the work grows with their sum and no faster: work that grows
/// with their product grows faster than that and fails.
#[test]
fn the_work_grows_with_the_entries_and_the_exclusions_together_and_no_faster() {
    let mut total = [0; 3];
    let mut counted = [(0, 0); 3];
    for units in 1..=3 {
        let work = judged(
            units * MODULES_PER_UNIT,
            units * EXCLUSIONS_PER_UNIT,
            usize::MAX,
        );
        total[units - 1] = work.total;
        counted[units - 1] = (work.entries, work.indexed);
    }
    let [one, two, three] = total;
    assert!(one > 0);
    assert!(three + one <= 2 * two, "{one}, {two}, {three}");
    assert_eq!(
        counted,
        [1, 2, 3].map(|units| (
            entries_of(units * MODULES_PER_UNIT),
            units * EXCLUSIONS_PER_UNIT
        ))
    );
}

/// Every kind of work is held to a ceiling linear in the entries plus the
/// exclusions, so judging that tests each exclusion against each entry passes
/// the ceiling and stops at once.
#[test]
fn every_kind_of_work_is_held_to_a_ceiling_linear_in_the_entries_and_the_exclusions() {
    let modules = 3 * MODULES_PER_UNIT;
    let exclusions = 3 * EXCLUSIONS_PER_UNIT;
    let entries = entries_of(modules);
    // The ceiling is below the product of the two.
    assert!(linear(entries, exclusions) < entries * exclusions);
    let work = judged(modules, exclusions, linear(entries, exclusions));
    assert_eq!(work.entries, entries);
    assert_eq!(work.excluded, exclusions);
}

/// An exclusion whose every entry another exclusion matches first is marked
/// all the same, whether the first is a prefix or an item, so `check` reports
/// no such exclusion as matching nothing; the first in the record gives the
/// reason, and an exclusion that matches nothing is the one finding of its kind.
#[test]
fn an_exclusion_that_another_matches_first_is_marked_and_only_one_that_matches_nothing_is_reported()
{
    let source = Workspace::load(
        &MemoryTree::from([
            (
                "crates/small/Cargo.toml",
                "[package]\nname = \"small\"\nversion = \"0.1.0\"\n",
            ),
            (
                "crates/small/src/lib.rs",
                "pub mod alone {\n    pub fn only() {}\n}\npub mod pair {\n    pub fn first() {}\n    pub fn second() {}\n}\n",
            ),
        ]),
        &["crates"],
    )
    .expect("a workspace");
    let exclusions = [
        // A prefix that is the first match of `first` and `second`.
        "small::pair::*",
        // An item matched only by `first`, which the prefix matches first.
        "small::pair::first",
        // An item that is the first match of `only`.
        "small::alone::only",
        // A prefix matched only by `only`, which the item matches first.
        "small::alone::*",
        // Nothing.
        "small::ghost",
    ]
    .map(String::from);
    let docs = documents("small", &[], &exclusions);
    let plan = Workplan::load(&docs, &source);
    let unmatched: Vec<String> = plan
        .check()
        .into_iter()
        .filter(|finding| finding.kind == kind::COVERAGE_EXCLUSION_UNMATCHED)
        .map(|finding| finding.message)
        .collect();
    let [message] = unmatched.as_slice() else {
        panic!("{unmatched:#?}");
    };
    assert!(message.contains("`small::ghost`"), "{message}");
    let report = plan.coverage_report();
    let reasons: Vec<(&str, &str)> = report
        .excluded()
        .iter()
        .map(|excluded| (excluded.gap().path(), excluded.reason()))
        .collect();
    assert_eq!(
        reasons,
        [
            ("small::alone::only", "For small::alone::only."),
            ("small::pair::first", "For small::pair::*."),
            ("small::pair::second", "For small::pair::*."),
        ]
    );
}

/// Whether `exclusion` matches an entry with these paths, read off the
/// exclusion: an item path equal to one of them, or a prefix one of them is
/// below.
fn oracle_matches(exclusion: &Exclusion, paths: &[&str]) -> bool {
    match &exclusion.item {
        ExclusionItem::Item(item) => paths.iter().any(|path| path == item),
        ExclusionItem::Prefix(prefix) => paths.iter().any(|path| {
            path.strip_prefix(prefix.as_str())
                .is_some_and(|rest| rest.starts_with("::"))
        }),
    }
}

/// The position of the first exclusion that matches the paths, each
/// exclusion tested against them in the order of the record, marking every
/// one that does.
fn oracle(exclusions: &[Exclusion], matched: &mut [bool], paths: &[&str]) -> Option<usize> {
    let mut first = None;
    for (position, (exclusion, hit)) in exclusions.iter().zip(matched.iter_mut()).enumerate() {
        if oracle_matches(exclusion, paths) {
            *hit = true;
            first.get_or_insert(position);
        }
    }
    first
}

/// The position in `exclusions` of the exclusion the index gives.
fn position_of(exclusions: &[Exclusion], found: Option<&Exclusion>) -> Option<usize> {
    found.map(|found| {
        exclusions
            .iter()
            .position(|exclusion| std::ptr::eq(exclusion, found))
            .expect("the index gives an exclusion of the record")
    })
}

/// Each path of one to four segments of `a`, `b` and `bc`, so that `a::b` and
/// `a::bc` share their text, and every path has a sibling, a parent and a
/// child.
fn alphabet_paths() -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    let mut last = vec![String::new()];
    for _ in 0..4 {
        last = last
            .iter()
            .flat_map(|head| {
                ["a", "b", "bc"].map(|segment| {
                    if head.is_empty() {
                        segment.to_string()
                    } else {
                        format!("{head}::{segment}")
                    }
                })
            })
            .collect();
        paths.extend(last.iter().cloned());
    }
    paths
}

/// Every exclusion of the alphabet: an item and a prefix of each path, so a
/// prefix equal to a whole path, a prefix longer than a path, prefixes shared
/// by several paths and an exclusion that is a prefix of another are all here.
fn alphabet_exclusions() -> Vec<Exclusion> {
    alphabet_paths()
        .into_iter()
        .flat_map(|path| {
            [
                Exclusion::new(ExclusionItem::Item(path.clone()), "Item."),
                Exclusion::new(ExclusionItem::Prefix(path), "Prefix."),
            ]
        })
        .collect()
}

/// For each entry of `entries`, alone and one after another, the index of
/// `exclusions` gives the first match of the oracle and marks what it marks.
fn agrees(exclusions: &[Exclusion], entries: &[Vec<&str>]) {
    let mut alone = ExclusionIndex::new(exclusions);
    let mut together = ExclusionIndex::new(exclusions);
    let mut marked = vec![false; exclusions.len()];
    for paths in entries {
        alone.matched.fill(false);
        let mut expected = vec![false; exclusions.len()];
        let first = oracle(exclusions, &mut expected, paths);
        assert_eq!(
            position_of(exclusions, alone.exclusion_for(paths)),
            first,
            "{paths:?}"
        );
        assert_eq!(alone.matched, expected, "{paths:?}");
        assert_eq!(
            position_of(exclusions, together.exclusion_for(paths)),
            oracle(exclusions, &mut marked, paths),
            "{paths:?}"
        );
    }
    assert_eq!(together.matched, marked);
}

/// The index finds what testing each exclusion against each path finds, on
/// every exclusion of the alphabet against every entry of one or two of its
/// paths: the same first match and the same exclusions marked, in the order of
/// the record and in the reverse order, alone and in one list, and the same on
/// an index built again.
#[test]
fn the_index_finds_the_first_match_and_marks_what_testing_every_exclusion_finds() {
    let paths = alphabet_paths();
    let mut entries: Vec<Vec<&str>> = paths.iter().map(|path| vec![path.as_str()]).collect();
    for one in &paths {
        for two in &paths {
            entries.push(vec![one.as_str(), two.as_str()]);
        }
    }
    let exclusions = alphabet_exclusions();
    for exclusion in &exclusions {
        agrees(std::slice::from_ref(exclusion), &entries[..paths.len()]);
    }
    let mut reversed = exclusions.clone();
    reversed.reverse();
    for list in [&exclusions, &reversed] {
        // Twice: each index is built with its own hash state.
        agrees(list, &entries);
        agrees(list, &entries);
    }
}

/// Two equal exclusions, which a record that loads never holds, are both
/// kept and both marked, and the first of them is the match.
#[test]
fn equal_exclusions_on_a_hand_built_list_are_both_marked_and_the_first_is_the_match() {
    let exclusions = [
        Exclusion::new(ExclusionItem::Prefix("a".to_string()), "First prefix."),
        Exclusion::new(ExclusionItem::Item("a::b".to_string()), "First item."),
        Exclusion::new(ExclusionItem::Prefix("a".to_string()), "Second prefix."),
        Exclusion::new(ExclusionItem::Item("a::b".to_string()), "Second item."),
    ];
    let mut index = ExclusionIndex::new(&exclusions);
    let found = index.exclusion_for(&["a::b"]);
    assert_eq!(position_of(&exclusions, found), Some(0));
    assert_eq!(index.matched, [true; 4]);
    agrees(
        &exclusions,
        &[vec!["a::b"], vec!["a"], vec!["a::bc", "a::b"]],
    );
}
