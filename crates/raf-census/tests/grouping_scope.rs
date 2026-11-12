// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The grouping scope over one two-package workspace that holds a copy of
//! every test-only form, copies under a named directory, and copies in
//! integration-test and example targets.

use std::collections::BTreeSet;

use raf_census::{Census, CensusOptions, Group, GroupingScope, Job, JobForbidden, MatchClass};
#[path = "support/scope_fixture.rs"]
mod fixture;

use fixture::tree;

/// The pinned default-scope baseline and index of [`tree`]. The complete scope
/// adds no assertion to either, so these bytes hold with or without a scope API.
const OFF: &str = include_str!("fixtures/reports/scope-off.ttl");

fn census(scope: GroupingScope) -> Census {
    load(&tree(), scope)
}

fn load(tree: &raf_source::MemoryTree, scope: GroupingScope) -> Census {
    let mut options = CensusOptions::default();
    options.scope = scope;
    Census::load(tree, &["."], &options).expect("the workspace loads")
}

/// The member symbols of every group, each group in member order.
fn groups(census: &Census) -> BTreeSet<Vec<String>> {
    symbols(census.groups().iter())
}

fn symbols<'a>(groups: impl Iterator<Item = &'a Group>) -> BTreeSet<Vec<String>> {
    groups
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
/// of that name that an integration test reaches keeps its group. The
/// excluded copy is still a unit that a job finds, so it still counts as a
/// copy toward the job.
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
    let copy = census
        .unit("one::generated::copy")
        .and_then(|unit| unit.print.as_ref())
        .and_then(|print| print.body.clone())
        .expect("the excluded copy is a unit with a body fingerprint");
    let job = Job::new("copy", "two::copy_again").forbidding(JobForbidden {
        fingerprints: vec![copy],
        constants: Vec::new(),
    });
    let report = census.match_jobs(&[job]).remove(0);
    assert!(
        report
            .matches
            .iter()
            .any(|found| found.symbol == "one::generated::copy" && found.class == MatchClass::Copy),
        "{report:?}"
    );
    assert_eq!(report.copies, 2);
    let baseline = census.baseline();
    assert_eq!(
        baseline.scope.excluded_dirs().collect::<Vec<_>>(),
        ["generated"]
    );
    let text = baseline.to_turtle().unwrap();
    assert_eq!(raf_census::Baseline::from_turtle(&text).unwrap(), baseline);
}

/// A unit stays in scope while any one of its reaches does. The file that the
/// library reaches only under `cfg(test)` and an integration test reaches
/// through `#[path]` keeps its body group when test items are excluded; a rule
/// that required every reach to stay in scope would drop it. Shim grouping
/// applies the same scope: the test-only forwarders group without the
/// exclusion and leave with it, while the library forwarders keep theirs.
#[test]
fn a_unit_with_any_kept_reach_still_groups_and_test_only_forwarders_leave_shim_groups() {
    let tree = fixture::reach_tree();
    let both = expected(&[&["one::shared::both", "two::both"]]);
    let relay = ["one::relay", "two::relay"];
    let forward = ["one::tests::forward", "two::tests::forward"];

    let complete = load(&tree, GroupingScope::default());
    assert_eq!(symbols(complete.body_groups()), both);
    assert_eq!(
        symbols(complete.shim_groups()),
        expected(&[&relay, &forward])
    );

    let excluded = load(&tree, GroupingScope::default().exclude_test_items(true));
    assert_eq!(symbols(excluded.body_groups()), both);
    assert_eq!(symbols(excluded.shim_groups()), expected(&[&relay]));
    assert!(has_unit(&excluded, "one::tests::forward"));
}
