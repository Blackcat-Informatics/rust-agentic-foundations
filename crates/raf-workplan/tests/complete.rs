// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Static completion publishes documents only after the prospective gate passes.

mod support;
use raf_workplan::{
    Gate, SyncOptions, SyncReport, Workplan,
    lifecycle::{Completion, Condition, Lifecycle, NewChangeRequest},
};
use support::evidence::Evidence;
use support::{
    changes::{BOUND, DONE, OMEGA, SHAPE, capability, record, repo},
    checkout::{Checkout, serial},
    disk::{read, snapshot},
    synced::INDEX,
};

fn fixture(state: &str, items: &str, bindings: [&str; 3]) -> Checkout {
    let documents = repo(&[
        (OMEGA, delivery_capability(state, bindings)),
        (
            SHAPE,
            record("shape", "[cap:omega]", &[], "none", items)
                .replace("affects:\n", "affects: []\n"),
        ),
    ]);
    Checkout::new("static-completion", &documents)
}

fn delivery_capability(state: &str, bindings: [&str; 3]) -> String {
    capability("omega", "delivered", bindings, &[]).replacen(
        "state: delivered",
        &format!("state: {state}"),
        1,
    )
}

fn complete(checkout: &Checkout) -> Completion {
    Lifecycle::open(checkout.root())
        .unwrap()
        .complete("shape")
        .unwrap()
}

#[test]
fn prospective_completion_refuses_missing_bindings_or_an_undone_item() {
    let _lock = serial();
    for (bindings, items, condition) in [
        (["[]", BOUND[1], BOUND[2]], DONE, Condition::Obligations),
        ([BOUND[0], BOUND[1], "[]"], DONE, Condition::AfterRemoval),
        (BOUND, "  - claim: still undone\n", Condition::Outcomes),
    ] {
        let checkout = fixture("implementing", items, bindings);
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

/// The completion fixture whose capability requires measured evidence, with
/// `records` beside it.
fn requiring(records: &[&Evidence]) -> Checkout {
    let mut files = vec![
        (
            OMEGA.to_string(),
            capability("omega", "delivered", BOUND, &[]).replacen(
                "depends:",
                "measured_evidence_required: true\ndepends:",
                1,
            ),
        ),
        (
            SHAPE.to_string(),
            record("shape", "[cap:omega]", &[], "none", DONE)
                .replace("affects:\n", "affects: []\n"),
        ),
    ];
    files.extend(records.iter().map(|record| (record.path(), record.text())));
    let files: Vec<(&str, String)> = files
        .iter()
        .map(|(path, text)| (path.as_str(), text.clone()))
        .collect();
    Checkout::new("measured-completion", &repo(&files))
}

#[test]
fn completion_waits_for_the_measured_evidence_a_capability_requires() {
    let _lock = serial();
    let bare = Evidence::passing("bare", "[cap:omega]").values(&[("peak", "41")]);
    let measured = Evidence::passing("measured", "[cap:omega]")
        .values(&[("peak", "41")])
        .units(&[("peak", "MiB")])
        .conditions("release build on the reference host");
    // Neither no record nor a record of values alone states a measurement.
    for records in [vec![], vec![&bare]] {
        let checkout = requiring(&records);
        let completion = complete(&checkout);
        assert!(!completion.is_completed(), "{completion}");
        assert!(
            completion
                .of_condition(Condition::AfterRemoval)
                .next()
                .is_some(),
            "{completion}"
        );
        assert!(checkout.root().join(SHAPE).is_file());
    }
    let checkout = requiring(&[&measured]);
    let completion = complete(&checkout);
    assert!(completion.is_completed(), "{completion}");
}

#[test]
fn completion_rewrites_owned_documents_and_leaves_source_and_git_untouched() {
    let _lock = serial();
    let checkout = fixture("implementing", DONE, BOUND);
    let original_capability = read(checkout.root(), OMEGA).replacen(
        "state: implementing",
        "state: \"implementing\"  # Delivery state.",
        1,
    );
    checkout.put(OMEGA, &original_capability);
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
    assert!(read(checkout.root(), OMEGA).contains("state: \"delivered\"  # Delivery state.\n"));
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

/// An implementing capability becomes delivered only when its last owning
/// record is completed.
#[test]
fn completion_retains_implementing_while_a_second_record_owns_the_capability() {
    let _lock = serial();
    let second = "work/changes/second.md";
    let documents = repo(&[
        (OMEGA, delivery_capability("implementing", BOUND)),
        (
            SHAPE,
            record("shape", "[cap:omega]", &[], "none", DONE)
                .replace("affects:\n", "affects: []\n"),
        ),
        (
            second,
            record("second", "[cap:omega]", &[], "none", DONE)
                .replace("affects:\n", "affects: []\n"),
        ),
    ]);
    let checkout = Checkout::new("completion-other-owner", &documents);
    let other = read(checkout.root(), second);
    let mut stale_index = detached_sync(&checkout);
    stale_index.edits.retain(|(path, _)| path.as_str() == INDEX);
    assert!(complete(&checkout).is_completed());
    assert_eq!(read(checkout.root(), second), other);
    assert!(read(checkout.root(), OMEGA).contains("state: implementing\n"));
    assert_stale(&checkout, &stale_index);
    let last = Lifecycle::open(checkout.root())
        .unwrap()
        .complete("second")
        .unwrap();
    assert!(last.is_completed(), "{last}");
    assert!(read(checkout.root(), OMEGA).contains("state: delivered\n"));
    assert_eq!(
        Gate::workspace(checkout.root()).try_check().unwrap(),
        Vec::new()
    );
}

/// Prepare a real split sync report without leaving the active fixture out of sync.
fn detached_sync(checkout: &Checkout) -> SyncReport {
    let root = checkout.root();
    let page = read(root, OMEGA);
    let catalogue = read(root, INDEX);
    let marker = "<!-- raf:generated:relationships -->";
    checkout.put(
        OMEGA,
        &page.replace(marker, &format!("{marker}\nStale content.")),
    );
    std::fs::remove_file(root.join(INDEX)).unwrap();
    let opened = Gate::workspace(root).open().unwrap();
    let report = opened.workplan().sync(&support::synced::options()).unwrap();
    checkout.put(OMEGA, &page);
    checkout.put(INDEX, &catalogue);
    assert!([OMEGA, INDEX].iter().all(|expected| {
        report
            .edits
            .iter()
            .any(|(path, _)| path.as_str() == *expected)
    }));
    report
}

fn assert_stale(checkout: &Checkout, report: &SyncReport) {
    let before = snapshot(checkout.root());
    let error = Workplan::write(report, checkout.root())
        .expect_err("the report has old lifecycle metadata");
    assert!(
        error
            .to_string()
            .contains("compute a fresh docs sync report"),
        "{error}"
    );
    assert_eq!(snapshot(checkout.root()), before);
}

/// Split publication cannot write across a native close or reopen, even when
/// the old report is limited to the catalogue or copied into a manual edit.
#[test]
fn split_sync_refuses_stale_publication_after_completion_or_reopening() {
    let _lock = serial();
    let checkout = fixture("implementing", DONE, BOUND);
    let root = checkout.root();
    let before_completion = detached_sync(&checkout);
    assert!(complete(&checkout).is_completed());
    assert_stale(&checkout, &before_completion);

    let before_opening = detached_sync(&checkout);
    Gate::workspace(root)
        .open()
        .unwrap()
        .workplan()
        .new_change(
            root,
            &NewChangeRequest::new("reopen").capability("cap:omega"),
        )
        .unwrap();
    assert_stale(&checkout, &before_opening);
    let manual = SyncReport::new(
        before_opening
            .edits
            .into_iter()
            .filter(|(path, _)| path.as_str() == OMEGA)
            .collect(),
    );
    assert_stale(&checkout, &manual);

    let current = Gate::workspace(root)
        .open()
        .unwrap()
        .workplan()
        .sync(&SyncOptions::default())
        .unwrap();
    Workplan::write(&current, root).expect("the neighboring current report publishes");
    assert!(read(root, OMEGA).contains("state: implementing\n"));
    assert!(read(root, OMEGA).contains("reopen.md"));
    assert!(read(root, INDEX).contains("## Implementing"));
}

#[test]
fn forged_sync_reports_cannot_overwrite_source_or_manifests() {
    let _lock = serial();
    let checkout = fixture("delivered", DONE, BOUND);
    for path in ["src/lib.rs", "Cargo.toml", ".git/config"] {
        let before = snapshot(checkout.root());
        let mut report = SyncReport::default();
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
    let mut report = SyncReport::default();
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
