// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Static completion publishes documents only after the prospective gate passes.

mod support;
use raf_workplan::{
    Gate, SyncOptions, Workplan,
    lifecycle::{Completion, Condition, Lifecycle},
};
use support::{
    changes::{BOUND, DONE, OMEGA, SHAPE, capability, record, repo},
    checkout::{Checkout, serial},
    disk::{read, snapshot},
};

fn fixture(state: &str, items: &str) -> Checkout {
    let documents = repo(&[
        (OMEGA, capability("omega", state, BOUND, &[])),
        (
            SHAPE,
            record("shape", "[cap:omega]", &[], "none", items)
                .replace("affects:\n", "affects: []\n"),
        ),
    ]);
    Checkout::new("static-completion", &documents)
}

fn complete(checkout: &Checkout) -> Completion {
    Lifecycle::open(checkout.root())
        .unwrap()
        .complete("shape")
        .unwrap()
}

#[test]
fn prospective_completion_refuses_an_implementing_capability_and_an_undone_item() {
    let _lock = serial();
    for (state, items, condition) in [
        ("implementing", DONE, Condition::AfterRemoval),
        (
            "delivered",
            "  - claim: still undone\n",
            Condition::Outcomes,
        ),
    ] {
        let checkout = fixture(state, items);
        let before = snapshot(checkout.root());
        let completion = complete(&checkout);
        assert!(!completion.is_completed(), "{completion}");
        assert!(
            completion.of_condition(condition).next().is_some(),
            "{completion}"
        );
        assert_eq!(snapshot(checkout.root()), before);
        assert!(checkout.root().join(SHAPE).is_file());
    }
}

#[test]
fn completion_rewrites_owned_documents_and_leaves_source_and_git_untouched() {
    let _lock = serial();
    let checkout = fixture("delivered", DONE);
    // A panicking body is source data. Structural verification never calls it.
    let original = read(checkout.root(), "tests/it.rs");
    checkout.put(
        "tests/it.rs",
        &original.replace("fn case() {", "fn case() { panic!(\"must not execute\");"),
    );
    let before = snapshot(checkout.root());
    let completion = complete(&checkout);
    assert!(completion.is_completed(), "{completion}");
    assert!(!checkout.root().join(SHAPE).exists());
    let after = snapshot(checkout.root());
    for (path, bytes) in &before {
        if path.starts_with(".git/")
            || std::path::Path::new(path).extension() == Some(std::ffi::OsStr::new("rs"))
            || path.ends_with("Cargo.toml")
            || path.ends_with("Cargo.lock")
        {
            assert_eq!(after.get(path), Some(bytes), "completion changed {path}");
        }
    }
    assert_eq!(
        Gate::workspace(checkout.root()).try_check().unwrap(),
        Vec::new()
    );
}

#[test]
fn forged_sync_reports_cannot_overwrite_source_or_manifests() {
    let _lock = serial();
    let checkout = fixture("delivered", DONE);
    for path in ["src/lib.rs", "Cargo.toml", ".git/config"] {
        let before = snapshot(checkout.root());
        let mut report = raf_workplan::SyncReport::default();
        report.edits.push((
            raf_source::RelPath::new(path).unwrap(),
            "replacement".into(),
        ));
        assert!(Workplan::write(&report, checkout.root()).is_err());
        assert_eq!(snapshot(checkout.root()), before);
    }
    checkout.put(
        "docs/capabilities/source.md",
        "pub fn source_in_markdown_path() {}\n",
    );
    let mut report = raf_workplan::SyncReport::default();
    report.edits.push((
        raf_source::RelPath::new("docs/capabilities/source.md").unwrap(),
        "replacement".into(),
    ));
    let before = snapshot(checkout.root());
    assert!(Workplan::write(&report, checkout.root()).is_err());
    assert_eq!(snapshot(checkout.root()), before);
    std::fs::remove_file(checkout.root().join("docs/capabilities/source.md")).unwrap();
    let opened = Gate::workspace(checkout.root()).open().unwrap();
    Workplan::write(
        &opened.workplan().sync(&SyncOptions::default()).unwrap(),
        checkout.root(),
    )
    .unwrap();
}

#[test]
fn an_agent_assessment_satisfies_condition_six_and_a_rejected_one_by_either_never_does() {
    use raf_okf::{Actor, AssessmentVerdict};
    use support::assessed::assess_on_disk_as;
    let _lock = serial();
    for actor in [Actor::Human, Actor::Agent] {
        for verdict in [AssessmentVerdict::Accepted, AssessmentVerdict::Rejected] {
            let documents = repo(&[
                (OMEGA, capability("omega", "delivered", BOUND, &[])),
                (
                    SHAPE,
                    record(
                        "shape",
                        "[cap:omega]",
                        &["cap:omega#contract"],
                        "none",
                        DONE,
                    ),
                ),
            ]);
            let checkout = Checkout::new("static-assessments", &documents);
            assert_eq!(
                assess_on_disk_as(checkout.root(), "shape", verdict, actor).len(),
                1
            );
            let opened = Gate::workspace(checkout.root()).open().unwrap();
            Workplan::write(
                &opened.workplan().sync(&SyncOptions::default()).unwrap(),
                checkout.root(),
            )
            .unwrap();
            let before = snapshot(checkout.root());
            let completion = complete(&checkout);
            if verdict == AssessmentVerdict::Accepted {
                assert!(completion.is_completed(), "{completion}");
            } else {
                assert!(
                    completion
                        .of_condition(Condition::Assessments)
                        .next()
                        .is_some(),
                    "{completion}"
                );
                assert_eq!(snapshot(checkout.root()), before);
            }
        }
    }
}
