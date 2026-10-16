// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The records in the document model: the load of `work/evidence`,
//! `work/assessments` and `work/coverage`, the record nodes and edges of the
//! graph, the claims of the capabilities, and the writer.

mod support;

use std::fmt::Write as _;
use std::fs;
use std::time::{Duration, Instant};

use raf_okf::{
    Actor, AssessmentRecord, AssessmentVerdict, CapabilitySection, CoverageRecord, EvidenceRecord,
    EvidenceSubject, EvidenceVerdict, Exclusion, ExclusionItem, Input, InputKind, Location,
    MAX_TEXT_BYTES, SectionRef,
};
use raf_source::MemoryTree;
use raf_workplan::record::{self, Record, RecordError, RecordOptions};
use raf_workplan::{Edge, EdgeKind, InputTarget, Node, SectionId, Workplan};
use support::cases::{DIGEST, assessment_record, coverage_record, evidence_record, with_record};
use support::disk::{Scratch, on_disk, put, read, snapshot, write_repo};
use support::fixtures::{ALPHA, BETA, GAMMA, valid};
use support::{Repo, assert_clean, check, demo_workspace, key, line_of, without_sync_findings};

const EVIDENCE: &str = "work/evidence/bound.md";
const ASSESSMENT: &str = "work/assessments/review.md";
const COVERAGE: &str = "work/coverage/default.md";

/// The valid repository with one record of each type.
fn with_records() -> Repo {
    let mut repo = valid();
    for record in [evidence_record(), assessment_record(), coverage_record()] {
        let (with, path) = with_record(&record);
        repo.insert(path.clone(), with[&path].clone());
    }
    repo
}

/// Require that a list holds nothing.
fn assert_none<T: std::fmt::Debug>(items: impl AsRef<[T]>) {
    let items = items.as_ref();
    assert_eq!(items.len(), 0, "{items:?}");
}

fn on_plan<R>(repo: &Repo, f: impl FnOnce(&Workplan<'_>) -> R) -> R {
    let tree: MemoryTree = repo.iter().collect();
    let workspace = demo_workspace();
    let plan = Workplan::load(&tree, &workspace);
    f(&plan)
}

/// Replace the first `from` of `path` in `repo`.
fn edit(repo: &mut Repo, path: &str, from: &str, to: &str) {
    let text = repo.get_mut(path).expect("the repository holds the file");
    assert!(text.contains(from), "{path} has no `{from}`");
    *text = text.replacen(from, to, 1);
}

/// `repo` with a claim marker after the Purpose text of alpha.
fn claiming(mut repo: Repo, evidence: &str) -> Repo {
    edit(
        &mut repo,
        ALPHA,
        "Text for Purpose.\n",
        &format!("Text for Purpose.\n<!-- raf:claim evidence={evidence} -->\n"),
    );
    repo
}

fn kinds(repo: &Repo) -> Vec<(&'static str, String)> {
    check(repo)
        .into_iter()
        .map(|finding| (finding.kind, finding.file))
        .collect()
}

// ---------------------------------------------------------------------------
// The load
// ---------------------------------------------------------------------------

#[test]
fn each_record_type_loads_from_its_directory_into_the_model() {
    let repo = with_records();
    assert_clean(&check(&repo));
    on_plan(&repo, |plan| {
        let [evidence] = plan.evidence() else {
            panic!("one evidence record: {:?}", plan.evidence());
        };
        assert_eq!(evidence.path(), EVIDENCE);
        assert_eq!(evidence.value().header.id, "evidence:bound");
        assert_eq!(evidence.value().verdict, EvidenceVerdict::Pass);
        assert_eq!(evidence.value().covers, ["cap:alpha"]);
        assert_none(evidence.body().text());
        let [assessment] = plan.assessments() else {
            panic!("one assessment: {:?}", plan.assessments());
        };
        assert_eq!(assessment.path(), ASSESSMENT);
        assert_eq!(
            assessment.value().subject,
            SectionRef::new("cap:alpha", CapabilitySection::Contract)
        );
        let coverage = plan.coverage().expect("the coverage record loads");
        assert_eq!(coverage.path(), COVERAGE);
        assert_eq!(coverage.value().scope, ["demo"]);
        // The documents load as before.
        assert_eq!(plan.capabilities().len(), 3);
        assert_eq!(plan.changes().len(), 1);
    });
}

#[test]
fn a_repository_without_record_directories_has_no_records_and_no_new_findings() {
    let repo = valid();
    assert_clean(&check(&repo));
    on_plan(&repo, |plan| {
        assert!(plan.evidence().is_empty() && plan.assessments().is_empty());
        assert!(plan.coverage().is_none());
        assert!(plan.graph().evidence().is_empty() && plan.graph().coverage().is_none());
        for kind in [
            EdgeKind::Covers,
            EdgeKind::Claim,
            EdgeKind::Assesses,
            EdgeKind::Scope,
        ] {
            assert!(
                !plan.graph().edges().iter().any(|edge| edge.kind() == kind),
                "{kind:?}"
            );
        }
    });
    // Record directories that hold no Markdown hold no record either.
    let mut repo = valid();
    for path in ["work/evidence/.keep", "work/assessments/.keep"] {
        repo.insert(path.to_string(), String::new());
    }
    assert_clean(&check(&repo));
    on_plan(&repo, |plan| assert_none(plan.evidence()));
}

#[test]
fn a_broken_record_is_a_finding_and_not_in_the_model() {
    let (repo, path) = with_record(&evidence_record().replace("verdict: pass", "verdict: maybe"));
    assert_eq!(kinds(&repo), [("okf-bad-verdict", path)]);
    on_plan(&repo, |plan| assert_none(plan.evidence()));
    // A record without frontmatter is a stray page, as in the other
    // directories.
    let mut repo = valid();
    repo.insert(
        "work/assessments/notes.md".to_string(),
        "# Notes\n\nNo frontmatter.\n".to_string(),
    );
    assert_eq!(
        kinds(&repo),
        [("stray-markdown", "work/assessments/notes.md".to_string())]
    );
}

#[test]
fn a_record_in_the_directory_of_another_type_is_wrong_directory() {
    // Evidence among the assessments, and a capability among the evidence.
    let mut repo = valid();
    repo.insert("work/assessments/bound.md".to_string(), evidence_record());
    let found = kinds(&repo);
    assert_eq!(
        found,
        [
            ("wrong-directory", "work/assessments/bound.md".to_string()),
            (
                "okf-wrong-directory",
                "work/assessments/bound.md".to_string()
            ),
        ]
    );
    let mut repo = valid();
    let alpha = repo.remove(ALPHA).expect("alpha");
    repo.insert("work/evidence/alpha.md".to_string(), alpha);
    assert!(
        check(&repo)
            .iter()
            .any(|f| f.kind == "wrong-directory" && f.file == "work/evidence/alpha.md"),
        "{:#?}",
        check(&repo)
    );
    // The neighbour in its own directory is clean.
    let (repo, _) = with_record(&evidence_record());
    assert_clean(&check(&repo));
}

#[test]
fn a_record_under_another_file_name_and_a_nested_record_are_refused() {
    let mut repo = valid();
    repo.insert("work/evidence/other.md".to_string(), evidence_record());
    let findings = check(&repo);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(
        (
            findings[0].kind,
            findings[0].file.as_str(),
            &findings[0].location
        ),
        ("okf-id-file-name", "work/evidence/other.md", &key("id"))
    );
    let mut repo = valid();
    repo.insert(
        "work/assessments/old/review.md".to_string(),
        assessment_record(),
    );
    assert_eq!(
        kinds(&repo),
        [(
            "nested-directory",
            "work/assessments/old/review.md".to_string()
        )]
    );
}

#[test]
fn work_coverage_holds_only_the_coverage_record() {
    // Any other file there is a finding and is not read, whatever it holds.
    for (path, text) in [
        ("work/coverage/other.md", coverage_record()),
        ("work/coverage/index.md", "# Coverage\n".to_string()),
        ("work/coverage/notes.txt", "notes".to_string()),
        ("work/coverage/default.md.bak", coverage_record()),
        ("work/coverage/.default.md.41.0.sync.tmp", coverage_record()),
    ] {
        let (mut repo, _) = with_record(&coverage_record());
        repo.insert(path.to_string(), text);
        let findings = check(&repo);
        assert_eq!(findings.len(), 1, "{path}: {findings:#?}");
        assert_eq!(
            (
                findings[0].kind,
                findings[0].file.as_str(),
                &findings[0].location
            ),
            ("singleton-other-file", path, &Location::Document)
        );
        assert!(
            findings[0].message.contains("work/coverage"),
            "{findings:#?}"
        );
        assert!(findings[0].fix.contains("interrupted"), "{findings:#?}");
        on_plan(&repo, |plan| {
            assert_eq!(
                plan.coverage().map(raf_workplan::Loaded::path),
                Some(COVERAGE),
                "{path}"
            );
        });
    }
    // Without the record, the other file is still the only finding.
    let mut repo = valid();
    repo.insert("work/coverage/other.md".to_string(), coverage_record());
    assert_eq!(
        kinds(&repo),
        [("singleton-other-file", "work/coverage/other.md".to_string())]
    );
    // The record itself under another slug.
    let (repo, path) =
        with_record(&coverage_record().replace("id: coverage:default", "id: coverage:other"));
    assert_eq!(path, COVERAGE);
    assert_eq!(kinds(&repo), [("okf-singleton-id", path)]);
    // Markdown below it is nested.
    let mut repo = valid();
    repo.insert(
        "work/coverage/old/default.md".to_string(),
        coverage_record(),
    );
    assert_eq!(
        kinds(&repo),
        [(
            "nested-directory",
            "work/coverage/old/default.md".to_string()
        )]
    );
}

#[test]
fn a_record_that_names_what_does_not_exist_is_a_finding_and_its_neighbour_is_clean() {
    let cases = [
        (
            evidence_record().replace("for: [cap:alpha]", "for: [cap:ghost, cap:alpha]"),
            "evidence-for-missing",
            "for",
        ),
        (
            assessment_record().replace("cap:alpha#contract", "cap:ghost#contract"),
            "assessment-subject-missing",
            "subject",
        ),
        (
            coverage_record().replace("scope: [demo]", "scope: [ghost]"),
            "scope-crate-missing",
            "scope",
        ),
    ];
    for (text, kind, at) in cases {
        let (repo, path) = with_record(&text);
        let findings = check(&repo);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(
            (
                findings[0].kind,
                findings[0].file.as_str(),
                &findings[0].location
            ),
            (kind, path.as_str(), &key(at))
        );
        assert!(findings[0].message.contains("ghost"), "{findings:#?}");
    }
    // A scope crate is named by its package name, so a crate path name is
    // refused and the message lists the packages.
    let (repo, _) = with_record(&coverage_record().replace("scope: [demo]", "scope: [demo_x]"));
    let findings = check(&repo);
    assert!(
        findings[0].message.contains("the packages are demo"),
        "{findings:#?}"
    );
    // A name whose document is there and broken is explained by that
    // document's own findings.
    let (mut repo, _) =
        with_record(&assessment_record().replace("cap:alpha#contract", "cap:broken#contract"));
    repo.insert(
        "docs/capabilities/broken.md".to_string(),
        "---\ntype: Capability\nid: cap:broken\n---\n".to_string(),
    );
    assert!(
        check(&repo)
            .iter()
            .all(|f| f.file == "docs/capabilities/broken.md"),
        "{:#?}",
        check(&repo)
    );
}

// ---------------------------------------------------------------------------
// Claims
// ---------------------------------------------------------------------------

#[test]
fn a_claim_on_evidence_that_covers_its_capability_is_clean_and_an_edge() {
    let repo = claiming(with_records(), "bound");
    assert_clean(&check(&repo));
    on_plan(&repo, |plan| {
        let [claim] = plan.graph().claims() else {
            panic!("one claim: {:?}", plan.graph().claims());
        };
        assert_eq!(claim.capability(), "cap:alpha");
        assert_eq!(claim.evidence(), "evidence:bound");
        assert_eq!(
            claim.section(),
            Some(SectionId::new("cap:alpha", CapabilitySection::Purpose))
        );
        assert_eq!(claim.line(), line_of(&repo[ALPHA], "raf:claim"));
        assert_eq!(plan.graph().claims_on("evidence:bound"), [claim]);
        assert_none(plan.graph().claims_on("evidence:other"));
    });
}

#[test]
fn a_claim_on_unknown_evidence_names_the_file_and_the_line() {
    let repo = claiming(with_records(), "bound,ghost");
    let findings = check(&repo);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let line = line_of(&repo[ALPHA], "raf:claim");
    assert_eq!(
        (
            findings[0].kind,
            findings[0].file.as_str(),
            &findings[0].location
        ),
        ("claim-evidence-missing", ALPHA, &Location::Line(line))
    );
    assert!(
        findings[0].message.contains("evidence:ghost"),
        "{findings:#?}"
    );
    assert!(
        findings[0].fix.contains("work/evidence/ghost.md"),
        "{findings:#?}"
    );
    // The record it does name is an edge.
    on_plan(&repo, |plan| assert_eq!(plan.graph().claims().len(), 1));
    // A claim on a record that is there and broken is explained by the
    // record's findings.
    let mut repo = claiming(valid(), "bound");
    repo.insert(
        EVIDENCE.to_string(),
        evidence_record().replace("by: agent", "by: Ada"),
    );
    assert_eq!(kinds(&repo), [("okf-bad-actor", EVIDENCE.to_string())]);
}

#[test]
fn a_claim_on_evidence_for_another_capability_is_a_finding() {
    let mut repo = claiming(with_records(), "bound");
    edit(&mut repo, EVIDENCE, "for: [cap:alpha]", "for: [cap:gamma]");
    let findings = check(&repo);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "claim-evidence-not-for");
    assert_eq!(findings[0].file, ALPHA);
    assert!(findings[0].fix.contains("cap:alpha"), "{findings:#?}");
    // The citation is still an edge, and the record covers gamma.
    on_plan(&repo, |plan| {
        assert_eq!(plan.graph().claims().len(), 1);
        assert_eq!(plan.graph().evidence_of("cap:gamma"), ["evidence:bound"]);
        assert_none(plan.graph().evidence_of("cap:alpha"));
    });
}

#[test]
fn a_claim_inside_a_proposed_block_is_not_a_claim_until_it_is_resolved() {
    // Beta is considered and holds a PROPOSED block in its Purpose.
    let mut repo = with_records();
    edit(
        &mut repo,
        BETA,
        "> The candidate contract.\n",
        "> The candidate contract.\n> <!-- raf:claim evidence=ghost -->\n",
    );
    assert_clean(&check(&repo));
    on_plan(&repo, |plan| assert_none(plan.graph().claims()));
}

#[test]
fn claims_are_section_granular_and_one_edge_per_named_record() {
    let mut repo = with_records();
    let (second, path) = with_record(
        &evidence_record()
            .replace("evidence:bound", "evidence:second")
            .replace("for: [cap:alpha]", "for: [cap:alpha, cap:gamma]"),
    );
    repo.insert(path.clone(), second[&path].clone());
    edit(
        &mut repo,
        ALPHA,
        "Text for Usage.\n",
        "Text for Usage.\n<!-- raf:claim evidence=second,bound -->\n",
    );
    edit(
        &mut repo,
        GAMMA,
        "Text for Constraints.\n",
        "- It holds.\n<!-- raf:claim evidence=second -->\n\nText for Constraints.\n",
    );
    assert_clean(&check(&repo));
    on_plan(&repo, |plan| {
        let claims: Vec<(&str, Option<SectionId>, &str)> = plan
            .graph()
            .claims()
            .iter()
            .map(|c| (c.capability(), c.section(), c.evidence()))
            .collect();
        let s = |cap: &str, section| Some(SectionId::new(cap, section));
        assert_eq!(
            claims,
            [
                (
                    "cap:alpha",
                    s("cap:alpha", CapabilitySection::Usage),
                    "evidence:bound"
                ),
                (
                    "cap:alpha",
                    s("cap:alpha", CapabilitySection::Usage),
                    "evidence:second"
                ),
                (
                    "cap:gamma",
                    s("cap:gamma", CapabilitySection::Constraints),
                    "evidence:second"
                ),
            ]
        );
        assert_eq!(plan.graph().claims_on("evidence:second").len(), 2);
        assert_eq!(
            plan.graph().evidence_of("cap:alpha"),
            ["evidence:bound", "evidence:second"]
        );
    });
}

// ---------------------------------------------------------------------------
// The graph
// ---------------------------------------------------------------------------

#[test]
fn the_records_are_nodes_with_their_edges() {
    on_plan(&with_records(), |plan| {
        let graph = plan.graph();
        let nodes = graph.nodes();
        for node in [
            Node::Evidence("evidence:bound".into()),
            Node::Assessment("assessment:review".into()),
            Node::Coverage("coverage:default".into()),
            Node::Crate("demo".into()),
        ] {
            assert!(nodes.contains(&node), "{node:?}");
        }
        assert_eq!(graph.evidence(), ["evidence:bound"]);
        assert_eq!(graph.assessments(), ["assessment:review"]);
        assert_eq!(graph.coverage(), Some("coverage:default"));
        let [covers] = graph.covers() else {
            panic!("{:?}", graph.covers());
        };
        assert_eq!(
            (covers.evidence(), covers.capability()),
            ("evidence:bound", "cap:alpha")
        );
        let [assesses] = graph.assesses() else {
            panic!("{:?}", graph.assesses());
        };
        let contract = SectionId::new("cap:alpha", CapabilitySection::Contract);
        assert_eq!(assesses.assessment(), "assessment:review");
        assert_eq!(assesses.section(), &contract);
        assert_eq!(graph.assessments_of(&contract), ["assessment:review"]);
        assert_none(graph.assessments_of(&SectionId::new("cap:alpha", CapabilitySection::Usage)));
        let [scope] = graph.scope() else {
            panic!("{:?}", graph.scope());
        };
        assert_eq!(
            (scope.coverage(), scope.package()),
            ("coverage:default", "demo")
        );
        let edges = graph.edges();
        assert!(edges.iter().any(|e| matches!(e, Edge::Covers(_))));
        assert!(edges.iter().any(|e| matches!(e, Edge::Assesses(_))));
        assert!(edges.iter().any(|e| matches!(e, Edge::Scope(_))));
    });
}

/// The valid repository with the records, and an assessment of alpha's
/// Contract made against a section, two symbols and the evidence record, one
/// input of each kind naming nothing the repository holds.
fn with_inputs() -> Repo {
    let mut repo = with_records();
    let row = |kind: &str, name: &str| {
        format!("  - kind: {kind}\n    name: {name}\n    digest: {DIGEST}\n")
    };
    let rows = [
        row("section", "cap:alpha#contract"),
        row("section", "cap:ghost#usage"),
        row("symbol", "demo::bare"),
        row("symbol", "demo::shown"),
        row("symbol", "demo::gone"),
        row("record", "evidence:bound"),
        row("record", "evidence:ghost"),
    ]
    .concat();
    repo.insert(
        "work/assessments/inputs.md".to_string(),
        format!(
            "---\ntype: Assessment\nid: assessment:inputs\nsubject: cap:alpha#contract\n\
             inputs:\n{rows}verdict: accepted\nby: agent\n---\n"
        ),
    );
    repo
}

#[test]
fn each_input_of_a_record_that_resolves_is_an_edge_and_one_that_does_not_is_no_finding() {
    let repo = with_inputs();
    // An input that names nothing now is not a finding of the load.
    assert_clean(&check(&repo));
    let workspace = demo_workspace();
    let item = |path: &str| workspace.resolve(path).expect(path).sites;
    on_plan(&repo, |plan| {
        let graph = plan.graph();
        let inputs: Vec<(InputKind, &str, InputTarget)> = graph
            .inputs_of("assessment:inputs")
            .into_iter()
            .map(|edge| (edge.kind(), edge.name(), edge.target().clone()))
            .collect();
        assert_eq!(
            inputs,
            [
                (
                    InputKind::Section,
                    "cap:alpha#contract",
                    InputTarget::Section(SectionId::new("cap:alpha", CapabilitySection::Contract))
                ),
                (
                    InputKind::Symbol,
                    "demo::bare",
                    InputTarget::Symbol(item("demo::bare"))
                ),
                // A re-export leads to its definition.
                (
                    InputKind::Symbol,
                    "demo::shown",
                    InputTarget::Symbol(item("demo::inner::go"))
                ),
                (
                    InputKind::Record,
                    "evidence:bound",
                    InputTarget::Record("evidence:bound".to_string())
                ),
            ]
        );
        // The evidence record's own symbol input and the first assessment's
        // section input are edges too.
        assert_eq!(graph.inputs_of("evidence:bound").len(), 1);
        assert_eq!(graph.inputs_of("assessment:review").len(), 1);
        assert_eq!(graph.inputs().len(), 6);
        assert!(graph.edges().iter().any(|e| matches!(e, Edge::Input(_))));
        // A symbol an input names is a node, though no capability binds it.
        let nodes = graph.nodes();
        assert!(nodes.contains(&Node::Symbol(item("demo::bare")[0])));
        let symbols = nodes
            .iter()
            .filter(|n| matches!(n, Node::Symbol(_)))
            .count();
        // demo::real (bound by gamma and named by the evidence), demo::bare
        // and demo::inner::go, once each.
        assert_eq!(symbols, 3, "{nodes:?}");
    });
    // Without records there is no input edge.
    on_plan(&valid(), |plan| assert_none(plan.graph().inputs()));
}

#[test]
fn a_reference_that_does_not_resolve_makes_no_edge() {
    let mut repo = with_records();
    edit(&mut repo, EVIDENCE, "for: [cap:alpha]", "for: [cap:ghost]");
    edit(&mut repo, COVERAGE, "scope: [demo]", "scope: [ghost]");
    on_plan(&repo, |plan| {
        assert_none(plan.graph().covers());
        assert_none(plan.graph().scope());
        assert!(
            !plan
                .graph()
                .nodes()
                .iter()
                .any(|n| matches!(n, Node::Crate(_)))
        );
        // The records are nodes still.
        assert_eq!(plan.graph().evidence(), ["evidence:bound"]);
    });
}

// ---------------------------------------------------------------------------
// The writer
// ---------------------------------------------------------------------------

fn evidence() -> EvidenceRecord {
    let mut record = EvidenceRecord::new(
        "parse-bound",
        EvidenceSubject::Command,
        vec![Input::new(InputKind::Symbol, "demo::real", DIGEST)],
        EvidenceVerdict::Pass,
        Actor::Agent,
        vec!["cap:alpha".to_string()],
    );
    record.command = Some("cargo bench".to_string());
    record
}

fn assessment() -> AssessmentRecord {
    AssessmentRecord::new(
        "alpha-contract",
        SectionRef::new("cap:alpha", CapabilitySection::Contract),
        vec![Input::new(InputKind::Section, "cap:alpha#contract", DIGEST)],
        AssessmentVerdict::Accepted,
        Actor::Human,
    )
}

fn coverage() -> CoverageRecord {
    CoverageRecord::new(
        vec!["demo".to_string()],
        vec![Exclusion::new(
            ExclusionItem::Prefix("demo::inner".to_string()),
            "Internals with no contract.",
        )],
    )
}

fn spdx() -> RecordOptions {
    RecordOptions::new()
        .copyright("2026 Example Org <dev@example.org>")
        .licence("CC-BY-4.0")
}

/// A scratch repository on disk: the demo workspace and the valid documents.
fn repository(tag: &str) -> Scratch {
    let scratch = Scratch::new(tag);
    write_repo(scratch.path(), &valid());
    scratch
}

#[test]
fn a_written_record_reloads_with_no_finding_as_the_same_value() {
    let scratch = repository("record-create");
    let root = scratch.path();
    let records = [
        Record::Evidence(evidence()),
        Record::Assessment(assessment()),
        Record::Coverage(coverage()),
    ];
    let mut written = Vec::new();
    for record in &records {
        let path = record::create(root, record, &spdx()).expect("the record is written");
        assert_eq!(path, record.path().expect("a path"));
        let text = read(root, path.as_str());
        assert!(
            text.starts_with(concat!(
                "---\n# SPDX-",
                "FileCopyrightText: 2026 Example Org"
            )),
            "{text}"
        );
        assert!(text.contains(concat!("\n# SPDX-", "License-Identifier: CC-BY-4.0\n")));
        written.push(path.to_string());
    }
    assert_eq!(
        written,
        [
            "work/evidence/parse-bound.md",
            "work/assessments/alpha-contract.md",
            "work/coverage/default.md"
        ]
    );
    on_disk(root, |plan| {
        assert_clean(&without_sync_findings(plan.check()));
        assert_eq!(plan.evidence()[0].value(), &evidence());
        assert_eq!(plan.assessments()[0].value(), &assessment());
        assert_eq!(
            plan.coverage().map(raf_workplan::Loaded::value),
            Some(&coverage())
        );
    });
    // Without SPDX options the record opens with its fence.
    let scratch = repository("record-plain");
    let path = record::create(
        scratch.path(),
        &Record::Evidence(evidence()),
        &RecordOptions::new(),
    )
    .expect("written");
    assert!(read(scratch.path(), path.as_str()).starts_with("---\ntype: Evidence\n"));
    on_disk(scratch.path(), |plan| {
        assert_clean(&without_sync_findings(plan.check()));
    });
}

#[test]
fn create_never_replaces_a_file() {
    let scratch = repository("record-exists");
    let root = scratch.path();
    put(root, "work/evidence/parse-bound.md", "mine\n");
    let before = snapshot(root);
    let error =
        record::create(root, &Record::Evidence(evidence()), &spdx()).expect_err("a file is there");
    assert!(matches!(error, RecordError::Exists { .. }), "{error}");
    assert_eq!(snapshot(root), before);
    // A second create of the same record is refused the same way.
    let scratch = repository("record-twice");
    let record = Record::Assessment(assessment());
    record::create(scratch.path(), &record, &spdx()).expect("the first is written");
    let before = snapshot(scratch.path());
    assert!(matches!(
        record::create(scratch.path(), &record, &spdx()),
        Err(RecordError::Exists { .. })
    ));
    assert_eq!(snapshot(scratch.path()), before);
}

#[test]
fn replace_puts_new_text_in_place_of_an_existing_record_only() {
    let scratch = repository("record-replace");
    let root = scratch.path();
    let mut record = assessment();
    // Nothing to replace yet.
    let before = snapshot(root);
    assert!(matches!(
        record::replace(root, &Record::Assessment(record.clone()), &spdx()),
        Err(RecordError::Missing { .. })
    ));
    assert_eq!(snapshot(root), before);
    record::create(root, &Record::Assessment(record.clone()), &spdx()).expect("created");
    record.verdict = AssessmentVerdict::Rejected;
    let path =
        record::replace(root, &Record::Assessment(record.clone()), &spdx()).expect("replaced");
    assert!(read(root, path.as_str()).contains("\nverdict: rejected\n"));
    on_disk(root, |plan| {
        assert_clean(&without_sync_findings(plan.check()));
        assert_eq!(plan.assessments()[0].value(), &record);
    });
    // No temporary file is left beside it.
    let names: Vec<String> = fs::read_dir(root.join("work/assessments"))
        .expect("lists")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(names, ["alpha-contract.md"]);
    // A directory at the path is not a record file.
    let scratch = repository("record-replace-dir");
    fs::create_dir_all(scratch.path().join("work/assessments/alpha-contract.md")).expect("dir");
    assert!(matches!(
        record::replace(scratch.path(), &Record::Assessment(assessment()), &spdx()),
        Err(RecordError::Missing { .. })
    ));
}

#[test]
fn a_record_that_would_not_load_clean_is_not_written() {
    let scratch = repository("record-invalid");
    let root = scratch.path();
    let before = snapshot(root);
    let mut bad = evidence();
    bad.inputs[0].digest = "abc".to_string();
    let error = record::create(root, &Record::Evidence(bad), &spdx()).expect_err("refused");
    let RecordError::Invalid { path, findings } = &error else {
        panic!("{error}");
    };
    assert_eq!(path.as_str(), "work/evidence/parse-bound.md");
    assert_eq!(findings[0].kind, "okf-bad-digest");
    // A command that names an issue would be an issue reference.
    let mut referencing = evidence();
    referencing.command = Some(concat!("cargo bench # see ", "#", "12").to_string());
    let error = record::create(root, &Record::Evidence(referencing), &spdx()).expect_err("refused");
    assert!(
        matches!(&error, RecordError::Invalid { findings, .. } if findings[0].kind == "issue-reference"),
        "{error}"
    );
    // A slug that is no file name, and SPDX given by halves.
    let mut slug = evidence();
    slug.header.slug = "../escape".to_string();
    assert!(matches!(
        record::create(root, &Record::Evidence(slug), &spdx()),
        Err(RecordError::BadSlug { .. })
    ));
    for options in [
        RecordOptions::new().copyright("2026 Example"),
        RecordOptions::new().licence("CC-BY-4.0"),
    ] {
        assert!(matches!(
            record::create(root, &Record::Evidence(evidence()), &options),
            Err(RecordError::SpdxUnset { .. })
        ));
    }
    assert!(matches!(
        record::create(
            root,
            &Record::Evidence(evidence()),
            &RecordOptions::new()
                .copyright("two\nlines")
                .licence("CC-BY-4.0")
        ),
        Err(RecordError::SpdxInvalid {
            field: "copyright",
            ..
        })
    ));
    assert_eq!(snapshot(root), before, "nothing is written");
    // The neighbour is written.
    record::create(root, &Record::Evidence(evidence()), &spdx()).expect("written");
}

#[test]
fn the_record_directory_is_created_below_the_root_only() {
    let scratch = repository("record-dirs");
    let root = scratch.path();
    assert!(!root.join("work/coverage").exists());
    record::create(root, &Record::Coverage(coverage()), &spdx()).expect("written");
    assert!(root.join("work/coverage/default.md").is_file());
    // A file where the directory belongs is refused, and left alone.
    let scratch = repository("record-dir-file");
    put(scratch.path(), "work/evidence", "a file\n");
    let before = snapshot(scratch.path());
    let error = record::create(scratch.path(), &Record::Evidence(evidence()), &spdx())
        .expect_err("refused");
    assert!(
        matches!(&error, RecordError::Write { path, .. } if path == "work/evidence"),
        "{error}"
    );
    assert_eq!(snapshot(scratch.path()), before);
}

#[cfg(unix)]
#[test]
fn a_symbolic_link_on_the_way_to_the_record_is_refused() {
    let outside = Scratch::new("record-outside");
    let scratch = repository("record-link");
    fs::create_dir_all(scratch.path().join("work")).expect("work");
    std::os::unix::fs::symlink(outside.path(), scratch.path().join("work/evidence"))
        .expect("a link");
    assert!(matches!(
        record::create(scratch.path(), &Record::Evidence(evidence()), &spdx()),
        Err(RecordError::Write { .. })
    ));
    assert!(
        fs::read_dir(outside.path())
            .expect("lists")
            .next()
            .is_none()
    );
    // A record file that is a link is not replaced through it.
    let scratch = repository("record-link-file");
    put(outside.path(), "target.md", "outside\n");
    fs::create_dir_all(scratch.path().join("work/evidence")).expect("dir");
    std::os::unix::fs::symlink(
        outside.path().join("target.md"),
        scratch.path().join("work/evidence/parse-bound.md"),
    )
    .expect("a link");
    assert!(record::replace(scratch.path(), &Record::Evidence(evidence()), &spdx()).is_err());
    assert_eq!(read(outside.path(), "target.md"), "outside\n");
}

// ---------------------------------------------------------------------------
// Hostile input
// ---------------------------------------------------------------------------

/// What the load of the hostile repositories below may take in a release
/// build; a debug build is not timed.
const RELEASE_BOUND: Duration = Duration::from_secs(5);

fn timed_check(repo: &Repo) -> (Duration, Vec<raf_workplan::Finding>) {
    let started = Instant::now();
    let findings = check(repo);
    (started.elapsed(), findings)
}

fn within_bound(elapsed: Duration, what: &str) {
    if !cfg!(debug_assertions) {
        assert!(elapsed < RELEASE_BOUND, "{what} took {elapsed:?}");
    }
}

#[test]
fn a_record_at_the_size_bound_with_the_most_measures_loads() {
    let head = evidence_record()
        .split("values:")
        .next()
        .expect("the record has values")
        .to_string();
    let mut values = String::from("values:\n");
    let mut limits = String::from("limits:\n");
    let mut count = 0;
    while head.len() + values.len() + limits.len() + 40 < MAX_TEXT_BYTES {
        let _ = writeln!(values, "  m{count}: 1");
        let _ = writeln!(limits, "  m{count}: 2");
        count += 1;
    }
    let text = format!("{head}{values}{limits}---\n");
    assert!(text.len() <= MAX_TEXT_BYTES && count > 40_000);
    let (repo, _) = with_record(&text);
    let (elapsed, findings) = timed_check(&repo);
    assert_clean(&findings);
    within_bound(elapsed, &format!("a record with {count} values and limits"));
    // One byte over the bound is refused by the reader, and a file the read
    // bound stops is refused by name.
    let mut repo = valid();
    repo.insert(EVIDENCE.to_string(), "a".repeat(MAX_TEXT_BYTES + 1));
    assert_eq!(
        kinds(&repo),
        [("okf-parse-too-large", EVIDENCE.to_string())]
    );
    repo.insert(EVIDENCE.to_string(), "a".repeat(MAX_TEXT_BYTES + 2));
    assert_eq!(kinds(&repo), [("unreadable-file", EVIDENCE.to_string())]);
}

#[test]
fn a_hundred_thousand_claims_load_in_time_linear_in_their_number() {
    // Four capabilities at the size bound, each holding as many claims as
    // fit, every claim naming a record that exists and one that does not.
    let mut repo = with_records();
    let mut claims = 0;
    let mut documents = 0;
    while claims < 100_000 {
        let slug = format!("bulk{documents}");
        let mut text = support::fixtures::capability(&slug, "designed", "[]", "");
        let at = text.find("Text for Purpose.\n").expect("a Purpose") + "Text for Purpose.\n".len();
        let mut inserted = String::new();
        let claim = "\nx\n<!-- raf:claim evidence=bound,ghost -->\n";
        while text.len() + inserted.len() + claim.len() < MAX_TEXT_BYTES && claims < 100_000 {
            inserted.push_str(claim);
            claims += 1;
        }
        text.insert_str(at, &inserted);
        repo.insert(format!("docs/capabilities/{slug}.md"), text);
        documents += 1;
    }
    let mut change = repo["work/changes/add-alpha.md"].clone();
    let listed: Vec<String> = (0..documents).map(|n| format!("cap:bulk{n}")).collect();
    change = change.replacen(
        "changes: [cap:alpha, cap:beta]",
        &format!("changes: [cap:alpha, cap:beta, {}]", listed.join(", ")),
        1,
    );
    repo.insert("work/changes/add-alpha.md".to_string(), change);
    edit(
        &mut repo,
        EVIDENCE,
        "for: [cap:alpha]",
        &format!("for: [cap:alpha, {}]", listed.join(", ")),
    );
    let (elapsed, findings) = timed_check(&repo);
    assert_eq!(claims, 100_000);
    assert!(documents >= 4, "{documents} documents");
    assert_eq!(findings.len(), claims, "one finding per unknown citation");
    assert!(findings.iter().all(|f| f.kind == "claim-evidence-missing"));
    within_bound(
        elapsed,
        &format!("{claims} claims in {documents} capabilities"),
    );
    on_plan(&repo, |plan| {
        assert_eq!(plan.graph().claims().len(), claims);
        assert_eq!(plan.graph().claims_on("evidence:bound").len(), claims);
    });
}

/// `replace` writes a temporary file beside the target and renames it over
/// the target: the target is a new file once it succeeds, and a directory
/// that refuses a new file leaves the target as it was, with no temporary file.
#[cfg(unix)]
#[test]
fn replace_renames_a_new_file_over_the_target_and_never_writes_into_it() {
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};

    let scratch = repository("record-atomic");
    let root = scratch.path();
    let mut record = assessment();
    record::create(root, &Record::Assessment(record.clone()), &spdx()).expect("created");
    let target = root.join("work/assessments/alpha-contract.md");
    let directory = root.join("work/assessments");
    let before = fs::metadata(&target).expect("the record").ino();
    record.verdict = AssessmentVerdict::Rejected;
    record::replace(root, &Record::Assessment(record.clone()), &spdx()).expect("replaced");
    assert_ne!(
        fs::metadata(&target).expect("the record").ino(),
        before,
        "the new text is a new file renamed into place, not written into the target"
    );
    // A directory that refuses a new file: the replace fails before the
    // target is touched.
    let old = fs::read(&target).expect("the record");
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o555)).expect("read-only");
    let enforced = fs::File::create(directory.join("probe")).is_err();
    let result = if enforced {
        record.verdict = AssessmentVerdict::Accepted;
        Some(record::replace(root, &Record::Assessment(record), &spdx()))
    } else {
        // Permissions do not bind this user; the rename above still proves
        // the target is never written in place.
        let _ = fs::remove_file(directory.join("probe"));
        None
    };
    let names: Vec<String> = fs::read_dir(&directory)
        .expect("lists")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o755)).expect("writable");
    if let Some(result) = result {
        assert!(
            matches!(result, Err(RecordError::Write { .. })),
            "{result:?}"
        );
        assert_eq!(fs::read(&target).expect("the record"), old);
        assert_eq!(names, ["alpha-contract.md"], "no temporary file is left");
    }
}
