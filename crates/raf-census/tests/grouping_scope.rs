// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The grouping scope over one two-package workspace that holds a copy of
//! every test-only form, copies under a named directory, and copies in
//! integration-test and example targets.

use std::collections::BTreeSet;

use raf_census::{Census, CensusOptions, GroupingScope};
#[path = "support/scope_fixture.rs"]
mod fixture;

use fixture::tree;

/// The pinned default-scope baseline and index of [`tree`]. The complete scope
/// adds no assertion to either, so these bytes hold with or without a scope API.
const OFF: &str = include_str!("fixtures/reports/scope-off.ttl");

fn census(scope: GroupingScope) -> Census {
    let mut options = CensusOptions::default();
    options.scope = scope;
    Census::load(&tree(), &["."], &options).expect("the workspace loads")
}

/// The member symbols of every group, each group in member order.
fn groups(census: &Census) -> BTreeSet<Vec<String>> {
    census
        .groups()
        .iter()
        .map(|group| group.member_symbols().map(str::to_owned).collect())
        .collect()
}

fn has_unit(census: &Census, symbol: &str) -> bool {
    census.units().iter().any(|unit| unit.symbol == symbol)
}

const HELPER: &[&str] = &["one::tests::helper", "two::tests::helper"];
const ALL: &[&str] = &["one::all_form", "two::all_form"];
const ANY: &[&str] = &["one::any_form", "two::any_form"];
const MIXED: &[&str] = &["one::mixed_form", "two::mixed_form"];
const TEST_FNS: &[&str] = &[
    "one::test_crate::a::check",
    "one::test_crate::b::check",
    "one::test_fn",
    "two::test_fn",
];
const INTEGRATION_TEST_FNS: &[&str] = &["one::test_crate::a::check", "one::test_crate::b::check"];
const COPIES: &[&str] = &[
    "one::generated::copy",
    "two::copy_again",
    "two::emitted::copy",
];
const COPIES_ELSEWHERE: &[&str] = &["two::copy_again", "two::emitted::copy"];
const MADE: &[&str] = &[
    "one::test_crate::a::generated::made",
    "one::test_crate::b::made_b",
];
const SHARED: &[&str] = &[
    "one::test_crate::a::shared_a",
    "one::test_crate::b::shared_b",
];
const EXAMPLES: &[&str] = &[
    "one::example_crate::x::t::example_x",
    "one::example_crate::y::t::example_y",
];

fn expected(sets: &[&[&str]]) -> BTreeSet<Vec<String>> {
    sets.iter()
        .map(|members| members.iter().map(|&member| member.to_owned()).collect())
        .collect()
}

/// With no exclusion the census groups every copy, and its reports are byte for
/// byte the pinned reports of this workspace.
#[test]
fn the_complete_scope_groups_every_copy_and_keeps_its_reports_unchanged() {
    let census = census(GroupingScope::default());
    assert_eq!(
        groups(&census),
        expected(&[
            HELPER, ALL, ANY, MIXED, TEST_FNS, COPIES, MADE, SHARED, EXAMPLES
        ])
    );
    let reports = format!(
        "baseline\n{}index\n{}",
        census.baseline().to_turtle().unwrap(),
        census.index(&[]).unwrap().to_turtle().unwrap()
    );
    assert_eq!(reports, OFF);
    let explicit = self::census(GroupingScope::default().exclude_test_items(false));
    assert_eq!(
        census.index(&[]).unwrap().to_turtle().unwrap(),
        explicit.index(&[]).unwrap().to_turtle().unwrap()
    );
}

/// Excluding test items removes every test-only form a library reaches,
/// `cfg(test)`, `cfg(all(unix, test))`, `cfg(any(test, test))` and
/// `#[test]`, and keeps `cfg(any(test, unix))`. Integration-test and example
/// targets keep their copies, `#[test]` functions included, and the excluded
/// items stay units for detectors, jobs and the index.
#[test]
fn excluding_test_items_drops_library_test_code_and_keeps_test_targets() {
    let census = census(GroupingScope::default().exclude_test_items(true));
    assert_eq!(
        groups(&census),
        expected(&[MIXED, INTEGRATION_TEST_FNS, COPIES, MADE, SHARED, EXAMPLES])
    );
    for symbol in ["one::tests::helper", "two::all_form", "one::test_fn"] {
        assert!(has_unit(&census, symbol), "{symbol} is still a unit");
    }
    let index = census.index(&[]).unwrap().to_turtle().unwrap();
    assert!(index.contains("excludeTestItems"), "{index}");
    assert!(!index.contains("excludeDir"), "{index}");
}

/// Excluding a directory removes the library copy under it and keeps the
/// copies elsewhere, so the group shrinks to them; a file under a directory
/// of that name that an integration test reaches keeps its group.
#[test]
fn excluding_a_directory_drops_only_library_files_under_it() {
    let scope = GroupingScope::default().exclude_dir("generated").unwrap();
    let census = census(scope);
    assert_eq!(
        groups(&census),
        expected(&[
            HELPER,
            ALL,
            ANY,
            MIXED,
            TEST_FNS,
            COPIES_ELSEWHERE,
            MADE,
            SHARED,
            EXAMPLES
        ])
    );
    assert!(has_unit(&census, "one::generated::copy"));
    let baseline = census.baseline();
    assert_eq!(
        baseline.scope.excluded_dirs().collect::<Vec<_>>(),
        ["generated"]
    );
    let text = baseline.to_turtle().unwrap();
    assert_eq!(raf_census::Baseline::from_turtle(&text).unwrap(), baseline);
}
