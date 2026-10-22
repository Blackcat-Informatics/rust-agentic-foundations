// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `status` through the built binary: one capability in each state, grouped in
//! the order of the catalogue, the gaps of the implementing one, the open
//! changes, the exit code, `-q`, and that it writes nothing.

mod support;

use support::Repo;

#[test]
fn the_roadmap_groups_one_capability_in_each_state_with_the_gaps_and_the_change() {
    let repo = Repo::synced("status-groups", true);
    let before = repo.snapshot();
    let run = repo.both(&["status"]).expect(0);
    assert_eq!(
        run.lines(),
        [
            "delivered     1  alpha",
            "implementing  1  beta (gap: bindings.code `demo::missing` does not resolve; \
             gap: bindings.tests is empty; gap: bindings.examples is empty; changes: grow)",
            "designed      1  gamma (changes: grow)",
            "considered    1  delta (changes: grow)",
            "open changes  1  grow (cap:beta, cap:delta, cap:gamma)",
        ],
        "{run:#?}"
    );
    // The gaps are not violations of the status command, and the one finding
    // of the documents is a note on standard error.
    assert!(run.stderr.contains("1 finding(s)"), "{run:#?}");
    assert!(run.stderr.contains("cargo workplan check"), "{run:#?}");
    assert_eq!(repo.snapshot(), before, "status wrote");
}

#[test]
fn a_state_with_no_capability_is_a_row_with_a_count_of_zero() {
    let repo = Repo::bare("status-empty-states");
    repo.put(
        "docs/capabilities/alpha.md",
        &support::capability(
            "alpha",
            "delivered",
            ["[demo::real]", "[demo::test_crate::it::case]", "[]"],
            "[]",
            &[],
        ),
    );
    repo.sync();
    let run = repo.both(&["status"]).expect(0);
    assert_eq!(
        run.lines(),
        [
            "delivered     1  alpha",
            "implementing  0",
            "designed      0",
            "considered    0",
            "open changes  0",
        ]
    );
    assert_eq!(run.stderr, "");
}

#[test]
fn an_implementing_capability_with_every_binding_resolved_has_no_gap_beyond_its_empty_lists() {
    let repo = Repo::synced("status-no-unresolved", false);
    let run = repo.both(&["status"]).expect(0);
    assert!(
        !run.stdout.contains("does not resolve"),
        "the binding resolves: {run:#?}"
    );
    assert!(
        run.stdout.contains("gap: bindings.tests is empty"),
        "{run:#?}"
    );
    assert_eq!(run.stderr, "");
}

#[test]
fn the_roadmap_is_the_same_by_root_from_another_directory() {
    let repo = Repo::synced("status-rooted", false);
    let here = repo.both(&["status"]).expect(0);
    let rooted = repo.rooted(&["status"]).expect(0);
    assert_eq!(here.stdout, rooted.stdout);
}

#[test]
fn quiet_leaves_standard_output_empty_because_the_roadmap_has_no_finding() {
    let repo = Repo::synced("status-quiet", true);
    let run = repo.both(&["-q", "status"]).expect(0);
    assert_eq!(
        (run.stdout.as_str(), run.stderr.as_str()),
        ("", ""),
        "{run:#?}"
    );
}

#[test]
fn a_directory_that_is_not_a_repository_is_exit_two() {
    let repo = Repo::bare("status-not-a-repository");
    let run = repo.rooted(&["status"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("docs/capabilities"), "{run:#?}");
}
