// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Stored evidence is data and grants no execution authority.

mod support;
use support::Repo;

#[test]
fn stored_command_text_is_never_executed_by_checks_or_dependency_views() {
    use raf_okf::{Actor, EvidenceRecord, EvidenceSubject, EvidenceVerdict, Input, InputKind};
    use raf_workplan::Gate;
    use raf_workplan::digest::Digest;
    use raf_workplan::record::{Record, RecordOptions};

    let repo = Repo::synced("stored-command-text", false);
    let opened = Gate::workspace(repo.root())
        .open()
        .expect("the real fixture workspace");
    let digest = Digest::symbol(opened.workspace(), "demo::real").expect("the actual code input");
    let mut record = EvidenceRecord::new(
        "stored",
        EvidenceSubject::Command,
        vec![Input::new(
            InputKind::Symbol,
            "demo::real",
            digest.to_string(),
        )],
        EvidenceVerdict::Fail,
        Actor::Agent,
        vec!["cap:alpha".to_string()],
    );
    record.command = Some(
        "sh -c 'printf ran > stored-command-ran'; $(printf ran > substituted-command-ran)"
            .to_string(),
    );
    repo.put(
        "work/evidence/stored.md",
        &Record::Evidence(record)
            .text(&RecordOptions::new())
            .unwrap(),
    );
    let before = repo.snapshot();
    for args in [
        vec!["check"],
        vec!["evidence", "for", "cap:alpha"],
        vec!["evidence", "claims", "stored"],
    ] {
        repo.both(&args).expect(0);
        assert_eq!(repo.snapshot(), before, "{args:?}");
        assert!(!repo.root().join("stored-command-ran").exists());
        assert!(!repo.root().join("substituted-command-ran").exists());
    }
}
