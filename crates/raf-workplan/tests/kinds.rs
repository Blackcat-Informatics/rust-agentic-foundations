// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The finding vocabulary: every kind has a named fixture, every kind a fixture
//! produces is a kind, and the kinds follow the rules and the errors of
//! `raf-okf`.

mod support;

use std::collections::BTreeSet;

use raf_okf::{ErrorKind, Rule};
use raf_workplan::Finding;
use support::cases::all_cases;

/// The kinds that name a rule of this crate.
const OWN_KINDS: [&str; 62] = [
    "diagram-invalid",
    "diagram-stale",
    "wrong-directory",
    "nested-directory",
    "stray-markdown",
    "unreadable-file",
    "unreadable-directory",
    "depends-missing",
    "changes-missing",
    "moved-missing",
    "singleton-other-file",
    "claim-evidence-missing",
    "claim-evidence-not-for",
    "claim-test-unbound",
    "evidence-for-missing",
    "assessment-subject-missing",
    "assessment-id-mismatch",
    "scope-crate-missing",
    "coverage-record-missing",
    "coverage-item-unbound",
    "coverage-test-unbound",
    "coverage-reexport-unbound",
    "coverage-exclusion-unmatched",
    "coverage-surface-unreadable",
    "coverage-exception-unmatched",
    "coverage-lines-over-ceiling",
    "architecture-page-unlinked",
    "decision-conditions-missing",
    "explanation-conditions-missing",
    "depends-cycle",
    "proposed-missing-change",
    "proposed-change-not-listing",
    "proposed-section-not-affected",
    "state-considered-no-proposal",
    "state-designed-proposal",
    "state-delivered-open-record",
    "state-delivered-proposal",
    "no-open-record",
    "link-broken",
    "anchor-broken",
    "link-outside",
    "link-symlink",
    "issue-reference",
    "binding-unresolved",
    "binding-ambiguous",
    "binding-wrong-role",
    "binding-macro-generated",
    "index-missing",
    "generated-stale",
    "generated-unknown-kind",
    "generated-misplaced",
    "index-hand-written",
    "mixed-line-endings",
    "strict-example-missing",
    "strict-matrix-missing",
    "strict-claim-uncovered",
    "strict-claim-unit-collapsed",
    "strict-section-thin",
    "strict-usage-unbound",
    "strict-limitations-missing",
    "debt-resolved",
    "debt-entry-invalid",
];

#[test]
fn kinds_are_unique_kebab_case_names() {
    let mut seen = BTreeSet::new();
    for kind in Finding::KINDS {
        assert!(
            !kind.is_empty()
                && kind
                    .split('-')
                    .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_lowercase())),
            "{kind}"
        );
        assert!(seen.insert(*kind), "{kind} is listed twice");
    }
}

#[test]
fn every_rule_and_every_reader_error_has_its_own_kind() {
    let mut want: BTreeSet<String> = OWN_KINDS.iter().map(ToString::to_string).collect();
    for tag in Rule::ALL {
        want.insert(format!("okf-{}", tag.name()));
    }
    for tag in ErrorKind::ALL {
        // A page that does not begin with a frontmatter fence is a stray file,
        // and text in the generated index is a hand-written index.
        if tag.name() != "missing-fence" && tag.name() != "index-text" {
            want.insert(format!("okf-parse-{}", tag.name()));
        }
    }
    let have: BTreeSet<String> = Finding::KINDS.iter().map(ToString::to_string).collect();
    assert_eq!(have, want);
    assert_eq!(
        Finding::KINDS.len(),
        Rule::ALL.len() + ErrorKind::ALL.len() + OWN_KINDS.len() - 2
    );
}

#[test]
fn every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind() {
    let mut names = BTreeSet::new();
    let mut produced: BTreeSet<&'static str> = BTreeSet::new();
    let mut failed = Vec::new();
    for case in all_cases() {
        assert!(
            names.insert(case.name.clone()),
            "{} is named twice",
            case.name
        );
        // Judge every real fixture before failing the registry, so one broken
        // neighbor does not hide the other actionable assertion failures.
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| case.assert_exact())).is_err() {
            failed.push(case.name);
            continue;
        }
        produced.extend(case.run().iter().map(|f| f.kind));
    }
    assert!(
        failed.is_empty(),
        "fixture assertions failed: {failed:?}; see their assertion output"
    );
    let kinds: BTreeSet<&'static str> = Finding::KINDS.iter().copied().collect();
    let unproduced: Vec<&&str> = kinds.difference(&produced).collect();
    assert!(unproduced.is_empty(), "no fixture produces {unproduced:?}");
    let unknown: Vec<&&str> = produced.difference(&kinds).collect();
    assert!(
        unknown.is_empty(),
        "fixtures produce unlisted kinds {unknown:?}"
    );
}

#[test]
fn every_finding_names_a_fix_and_a_message() {
    for case in all_cases() {
        for finding in case.run() {
            assert!(!finding.message.is_empty(), "{finding:#?}");
            assert!(!finding.fix.is_empty(), "{finding:#?}");
        }
    }
}
