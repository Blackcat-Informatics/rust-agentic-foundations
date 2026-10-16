// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Writing records: a record built with the constructors, or read from a
//! fixture, renders to a document the profile reads back as the same value,
//! with its keys in table order; a value the profile refuses is refused when
//! the text is read back, and a value the frontmatter cannot hold is refused
//! by the writer.

use std::fs;
use std::path::Path;

use raf_okf::{
    Actor, AssessmentRecord, AssessmentVerdict, CapabilitySection, ConstructError, CoverageRecord,
    DocType, Document, EvidenceRecord, EvidenceSubject, EvidenceVerdict, Exclusion, ExclusionItem,
    Input, InputKind, Measure, Number, Profile, RafDocument, Rule, SectionRef,
};

const DIGEST: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

/// Read `text` as a document of `path` and validate it.
fn read(text: &str, path: &str) -> RafDocument {
    let document = Document::parse(text).unwrap_or_else(|error| panic!("{error}: {text}"));
    Profile
        .validate(&document, path)
        .unwrap_or_else(|found| panic!("{path} does not validate: {found:#?}\n{text}"))
}

fn number(text: &str) -> Number {
    Number::parse(text).unwrap_or_else(|| panic!("{text} is a number"))
}

fn evidence() -> EvidenceRecord {
    let mut record = EvidenceRecord::new(
        "memory-bound",
        EvidenceSubject::Command,
        vec![
            Input::new(InputKind::Symbol, "raf_okf::Document::parse", DIGEST),
            Input::new(InputKind::Symbol, "raf_okf::Body::claims", DIGEST),
        ],
        EvidenceVerdict::Pass,
        Actor::Agent,
        vec![
            "cap:okf-profile".to_string(),
            "cap:source-index".to_string(),
        ],
    );
    record.command = Some("cargo bench --bench parse -- \"10 MiB\"".to_string());
    record.revision = Some("b499578a1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f".to_string());
    record.values = vec![
        Measure::new("peak_mib", number("41.5")),
        Measure::new("seconds", number("-0.25")),
        Measure::new("count", number("0012")),
    ];
    record.limits = vec![Measure::new("peak_mib", number("64"))];
    record.header.tags = Some(vec!["bench".to_string(), "memory".to_string()]);
    record.header.timestamp = Some("2026-03-01T12:00:00Z".to_string());
    record
}

fn assessment() -> AssessmentRecord {
    AssessmentRecord::new(
        "reconcile-explanation",
        SectionRef::new("cap:workplan-reconcile", CapabilitySection::Explanation),
        vec![
            Input::new(
                InputKind::Section,
                "cap:workplan-reconcile#explanation",
                DIGEST,
            ),
            Input::new(InputKind::Record, "evidence:memory-bound", DIGEST),
        ],
        AssessmentVerdict::Rejected,
        Actor::Human,
    )
}

fn coverage() -> CoverageRecord {
    CoverageRecord::new(
        vec!["raf-workplan".to_string(), "cargo-workplan".to_string()],
        vec![
            Exclusion::new(
                ExclusionItem::Prefix("raf_workplan::exec::tests".to_string()),
                "Unit tests of the process rows: they guard internals.",
            ),
            Exclusion::new(
                ExclusionItem::Item("raf_workplan::Workplan::empty".to_string()),
                "Built by the load only.",
            ),
        ],
    )
}

#[test]
fn a_built_evidence_record_renders_to_a_record_that_reads_back_equal() {
    let record = evidence();
    assert_eq!(record.header.id, "evidence:memory-bound");
    let text = record.to_document().expect("the record writes").render();
    assert_eq!(
        read(&text, "work/evidence/memory-bound.md"),
        RafDocument::Evidence(record)
    );
    // Numbers are written as the record holds them, unquoted.
    assert!(text.contains("\n  peak_mib: 41.5\n"), "{text}");
    assert!(text.contains("\n  count: 0012\n"), "{text}");
    // A record has no body: the text ends with the closing fence.
    assert!(text.ends_with("\n---\n"), "{text}");
}

#[test]
fn a_built_assessment_and_coverage_record_read_back_equal() {
    let record = assessment();
    let text = record.to_document().expect("the record writes").render();
    assert_eq!(
        read(&text, "work/assessments/reconcile-explanation.md"),
        RafDocument::Assessment(record)
    );
    let record = coverage();
    assert_eq!(record.header.id, "coverage:default");
    let text = record.to_document().expect("the record writes").render();
    assert!(
        text.contains("item: raf_workplan::exec::tests::*"),
        "{text}"
    );
    assert_eq!(
        read(&text, "work/coverage/default.md"),
        RafDocument::Coverage(record)
    );
}

#[test]
fn empty_lists_and_absent_keys_are_written_as_the_profile_reads_them() {
    // An empty scope and an empty exclusion list are `[]`, which opts out
    // visibly.
    let record = CoverageRecord::new(Vec::new(), Vec::new());
    let text = record.to_document().expect("the record writes").render();
    assert_eq!(
        text,
        "---\ntype: Coverage\nid: coverage:default\nscope: []\nexclusions: []\n---\n"
    );
    assert_eq!(
        read(&text, "work/coverage/default.md"),
        RafDocument::Coverage(record)
    );
    // Evidence from a bound test: no command, no revision and no measures,
    // so none of the optional keys is written.
    let record = EvidenceRecord::new(
        "unit",
        EvidenceSubject::Binding("raf_okf::test_crate::records::unit".to_string()),
        vec![Input::new(InputKind::Symbol, "raf_okf::Body", DIGEST)],
        EvidenceVerdict::Fail,
        Actor::Human,
        vec!["cap:okf-profile".to_string()],
    );
    let text = record.to_document().expect("the record writes").render();
    for key in [
        "command",
        "revision",
        "values",
        "limits",
        "tags",
        "timestamp",
    ] {
        assert!(!text.contains(&format!("\n{key}:")), "{key} in {text}");
    }
    assert!(text.contains("\nsubject: binding:raf_okf::test_crate::records::unit\n"));
    assert_eq!(
        read(&text, "work/evidence/unit.md"),
        RafDocument::Evidence(record)
    );
}

#[test]
fn the_keys_are_written_in_the_order_of_the_key_table() {
    for (doc_type, text) in [
        (
            DocType::Evidence,
            evidence().to_document().expect("writes").render(),
        ),
        (
            DocType::Assessment,
            assessment().to_document().expect("writes").render(),
        ),
        (
            DocType::Coverage,
            coverage().to_document().expect("writes").render(),
        ),
    ] {
        let written: Vec<&str> = text
            .lines()
            .filter_map(|line| line.split_once(':').map(|(key, _)| key))
            .filter(|key| !key.starts_with(' ') && !key.starts_with('-') && !key.is_empty())
            .collect();
        let table: Vec<&str> = doc_type
            .keys()
            .iter()
            .map(|spec| spec.key)
            .filter(|key| written.contains(key))
            .collect();
        assert_eq!(written, table, "{text}");
    }
}

#[test]
fn every_record_fixture_reads_back_equal_after_writing() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for (name, path) in [
        ("evidence.md", "work/evidence/ledger-sync-bound.md"),
        (
            "assessment.md",
            "work/assessments/ledger-sync-explanation.md",
        ),
        ("coverage.md", "work/coverage/default.md"),
    ] {
        let text = fs::read_to_string(fixtures.join(name)).expect("the fixture reads");
        let typed = read(&text, path);
        let written = match &typed {
            RafDocument::Evidence(record) => record.to_document(),
            RafDocument::Assessment(record) => record.to_document(),
            RafDocument::Coverage(record) => record.to_document(),
            other => panic!("{name} is not a record: {other:?}"),
        }
        .expect("the record writes")
        .render();
        assert_eq!(read(&written, path), typed, "{name}");
        // Writing the written text again gives the same bytes.
        assert_eq!(Document::parse(&written).expect("reads").render(), written);
    }
}

#[test]
fn a_value_the_profile_refuses_is_written_and_refused_when_it_is_read_back() {
    // The constructors do not judge values; the profile does, on the text.
    let mut record = evidence();
    record.inputs[0].digest = "ABC".to_string();
    let text = record.to_document().expect("the text is writable").render();
    let document = Document::parse(&text).expect("reads");
    let found = Profile
        .validate(&document, "work/evidence/memory-bound.md")
        .expect_err("the digest is refused");
    assert!(
        found
            .iter()
            .any(|violation| matches!(violation.rule(), Rule::BadDigest { .. })),
        "{found:#?}"
    );
    // The neighbour with a digest of the right form reads back.
    let record = evidence();
    let text = record.to_document().expect("writes").render();
    assert_eq!(
        read(&text, "work/evidence/memory-bound.md"),
        RafDocument::Evidence(record)
    );
}

#[test]
fn a_value_the_frontmatter_cannot_hold_is_refused_by_the_writer() {
    let mut record = evidence();
    record.command = Some("cargo bench\u{7}".to_string());
    assert!(matches!(
        record.to_document(),
        Err(ConstructError::Unrepresentable { character: '\u{7}' })
    ));
    let mut record = evidence();
    record.values = vec![Measure::new("not a key", number("1"))];
    assert!(matches!(
        record.to_document(),
        Err(ConstructError::InvalidKey { .. })
    ));
    // A command with a line break is escaped, not refused, and reads back.
    let mut record = evidence();
    record.command = Some("cargo bench\nsecond line".to_string());
    let text = record.to_document().expect("writes").render();
    assert_eq!(
        read(&text, "work/evidence/memory-bound.md"),
        RafDocument::Evidence(record)
    );
}

#[test]
fn verdict_and_subject_names_are_the_values_of_their_keys() {
    assert_eq!(
        EvidenceVerdict::ALL.map(EvidenceVerdict::name),
        ["pass", "fail"]
    );
    assert_eq!(
        AssessmentVerdict::ALL.map(AssessmentVerdict::name),
        ["accepted", "rejected"]
    );
    assert_eq!(EvidenceSubject::Command.text(), "command");
    assert_eq!(
        EvidenceSubject::Binding("a::b".to_string()).text(),
        "binding:a::b"
    );
    assert_eq!(
        SectionRef::new("cap:a", CapabilitySection::Usage).text(),
        "cap:a#usage"
    );
}
