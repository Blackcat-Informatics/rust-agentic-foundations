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
const OWN_KINDS: [&str; 64] = [
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
    "review-findings-unreadable",
    "review-missing",
    "review-stale",
    "review-unavailable",
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

/// Explicit policy has missing/stale neighbors; a findings verdict is not clean.
fn required_review_neighbors() -> Vec<&'static str> {
    use raf_okf::{
        Actor, DocType, RecordHeader, ReviewKind, ReviewRecord, ReviewScope, ReviewVerdict,
    };
    use raf_source::MemoryTree;
    use raf_workplan::Workplan;
    use support::strict as fixture;
    let source = support::workspace_of(&fixture::source_files());
    let mut repo: support::Repo = fixture::documents()
        .into_iter()
        .map(|(path, text)| (path.to_string(), text))
        .collect();
    assert_eq!(support::check_in(&repo, &source), []);
    let cap = repo.get_mut(fixture::CAPABILITY).unwrap();
    *cap = cap.replace("depends:", "review_required: [security]\ndepends:");
    let kinds = review_finding_kinds;
    assert_eq!(kinds(&repo, &source), ["review-missing"]);
    assert_eq!(
        review_obligations(&repo, &source).unwrap(),
        ["review-missing"]
    );
    let tree: MemoryTree = repo.iter().collect();
    let scope = ReviewScope::Capability("cap:demo".to_string());
    let inputs = Workplan::load(&tree, &source)
        .review_inputs(&scope)
        .unwrap();
    let mut header = RecordHeader::new(DocType::Review, "demo-security");
    header.timestamp = Some("2026-01-01T00:00:00Z".to_string());
    let mut record = ReviewRecord::new(
        header,
        scope,
        "a".repeat(40),
        ReviewKind::Security,
        ReviewVerdict::Findings,
        Actor::Human,
        inputs,
    );
    record.findings_ref = Some("https://example.invalid/review".to_string());
    let path = "work/reviews/demo-security.md".to_string();
    repo.insert(path.clone(), record.to_document().unwrap().render());
    assert_eq!(kinds(&repo, &source), Vec::<&str>::new());
    assert_eq!(
        review_obligations(&repo, &source).unwrap(),
        Vec::<&str>::new()
    );
    // A completed findings review satisfies review coverage, not a clean verdict.
    let tree: MemoryTree = repo.iter().collect();
    assert_eq!(
        Workplan::load(&tree, &source).review_list().unwrap()[0]
            .value()
            .verdict,
        ReviewVerdict::Findings
    );
    for (verdict, kind) in [
        (ReviewVerdict::Blocked, ReviewKind::Security),
        (ReviewVerdict::Findings, ReviewKind::Design),
    ] {
        record.verdict = verdict;
        record.kind = kind;
        repo.insert(path.clone(), record.to_document().unwrap().render());
        assert_eq!(kinds(&repo, &source), ["review-missing"]);
    }
    record.verdict = ReviewVerdict::Findings;
    record.kind = ReviewKind::Security;
    repo.insert(path.clone(), record.to_document().unwrap().render());
    let mut files = fixture::source_files();
    let lib = files
        .iter_mut()
        .find(|(path, _)| *path == fixture::LIB)
        .unwrap();
    lib.1 = lib
        .1
        .replace("pub fn run() {}", "pub fn run() { let _value = 1; }");
    let changed = support::workspace_of(&files);
    assert_eq!(kinds(&repo, &changed), ["review-stale"]);
    assert_eq!(
        review_obligations(&repo, &changed).unwrap(),
        ["review-stale"]
    );
    record.findings_ref = Some("work/findings/missing.md".to_string());
    repo.insert(path, record.to_document().unwrap().render());
    let unavailable = kinds(&repo, &source);
    assert_eq!(
        unavailable,
        ["review-findings-unreadable", "review-unavailable"]
    );
    assert!(matches!(
        review_obligations(&repo, &source),
        Err(raf_workplan::obligation::ReconcileError::Review(_))
    ));
    unavailable
        .into_iter()
        .chain(["review-missing", "review-stale"])
        .collect()
}

fn review_finding_kinds(repo: &support::Repo, source: &raf_source::Workspace) -> Vec<&'static str> {
    let mut found: Vec<_> = support::check_in(repo, source)
        .into_iter()
        .map(|finding| finding.kind)
        .collect();
    purrdf_lex::walk::try_sort_unstable_by(&mut found, |left, right| {
        Ok::<_, std::convert::Infallible>(left.cmp(right))
    })
    .unwrap_or_else(|never| match never {});
    purrdf_lex::walk::try_dedup_by(&mut found, |left, right| {
        Ok::<_, std::convert::Infallible>(left == right)
    })
    .unwrap_or_else(|never| match never {});
    found
}

fn review_obligations(
    repo: &support::Repo,
    source: &raf_source::Workspace,
) -> Result<Vec<&'static str>, raf_workplan::obligation::ReconcileError> {
    let mut repo = repo.clone();
    repo.insert("work/changes/review-demo.md".to_string(),
        "---\ntype: Change\nid: change:review-demo\ntitle: Review demo\ndescription: Review the demo.\nchanges: [cap:demo]\naffects: []\ncontract_effect: none\nitems:\n  - claim: Review the demo.\n    outcome: implemented\n---\n\n## Summary\n\nThe demo requires its current code review.\n".to_string());
    let tree: raf_source::MemoryTree = repo.iter().collect();
    let owed = raf_workplan::Workplan::load(&tree, source).reconcile("review-demo")?;
    Ok(owed
        .obligations()
        .iter()
        .filter(|owed| {
            matches!(
                owed.kind,
                raf_workplan::obligation::ObligationKind::ReviewMissing
                    | raf_workplan::obligation::ObligationKind::ReviewStale
            )
        })
        .map(|owed| owed.kind.name())
        .collect())
}

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
    produced.extend(required_review_neighbors());
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
