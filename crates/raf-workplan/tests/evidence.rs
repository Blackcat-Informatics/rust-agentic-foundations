// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Evidence and claims: the dependency views, a record whose verdict comes
//! from a bound test run by a real `cargo`, the refusals of a request, the
//! staleness of a record when its bound code moves (and not when anything else
//! does), and `evidence-changed` against the merge base in real git
//! repositories.

mod support;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use raf_okf::{Actor, CapabilitySection, EvidenceSubject, EvidenceVerdict};
use raf_source::{MemoryTree, Workspace};
use raf_workplan::evidence::{
    EvidenceError, EvidenceLookupError, EvidenceRequest, Failed, Support, VerdictSource,
};
use raf_workplan::exec::{
    ExecError, FakeRunner, Invocation, Limits, Output, Programs, Runner, SystemRunner,
};
use raf_workplan::git::GitError;
use raf_workplan::lifecycle::{Lifecycle, LifecycleError, Side};
use raf_workplan::obligation::{ObligationKind, Reconciled};
use raf_workplan::{Gate, Workplan};
use support::Repo;
use support::changes::{DONE, OMEGA, SHAPE, capability, record, replaced, repo, without};
use support::checkout::{Checkout, serial};
use support::disk::{Scratch, put, snapshot};
use support::evidence::{Evidence, claim, digest_of};
use support::synced::with_plan;

/// `omega` owns the demonstration code, test and real example. Dependency
/// queries and reconciliation inspect those bindings without running them.
const BOUND: [&str; 3] = [
    "[demo::real]",
    "[demo::test_crate::it::case]",
    "[demo::example_crate::show]",
];

/// `omega` delivered with `extra` in its sections, `shape` listing it with the
/// effect `none` (so no section is compared), and the records given.
fn claimed(extra: &[(&str, &str)], records: &[&Evidence]) -> Repo {
    let mut document = capability("omega", "delivered", BOUND, extra);
    // The authored claim precedes the automatically supplied proof marker of
    // the same Contract paragraph. Their order is part of the real grammar.
    if let Some((_, text)) = extra.iter().find(|(section, _)| *section == "Contract")
        && let Some((first, _)) = text.split_once('\n')
        && first.starts_with("<!-- raf:claim ")
    {
        let tested = "<!-- raf:tested-by demo::test_crate::it::case -->\n";
        let before = format!("{tested}{first}\n");
        assert!(document.contains(&before));
        document = document.replacen(&before, &format!("{first}\n{tested}"), 1);
    }
    let mut files: Vec<(String, String)> = vec![
        (OMEGA.to_string(), document),
        (
            SHAPE.to_string(),
            record(
                "shape",
                "[cap:omega]",
                &["cap:omega#contract"],
                "none",
                DONE,
            ),
        ),
    ];
    files.extend(records.iter().map(|record| (record.path(), record.text())));
    let files: Vec<(&str, String)> = files
        .iter()
        .map(|(path, text)| (path.as_str(), text.clone()))
        .collect();
    repo(&files)
}

// --- the dependency views -----------------------------------------------------

#[test]
fn the_views_list_the_claims_of_a_record_and_the_records_of_a_capability() {
    let bound = Evidence::passing("bound", "[cap:omega]");
    let other = Evidence::passing("other", "[cap:omega]").verdict("fail");
    let usage = format!(
        "{}\n- It holds too.\n{}",
        claim("bound,ghost"),
        claim("other")
    );
    let repo = claimed(
        &[("Contract", &claim("bound")), ("Usage", &usage)],
        &[&bound, &other],
    );
    let contract_line = support::line_of(&repo[OMEGA], "raf:claim evidence=bound -->");
    let usage_line = support::line_of(&repo[OMEGA], "evidence=bound,ghost");
    let item_line = support::line_of(&repo[OMEGA], "evidence=other");
    with_plan(&repo, |plan| {
        let claims = plan.evidence_claims("evidence:bound").expect("claims");
        let rows: Vec<(&str, &str, Option<CapabilitySection>, usize, &str)> = claims
            .iter()
            .map(|c| {
                (
                    c.capability.as_str(),
                    c.path.as_str(),
                    c.section,
                    c.line,
                    c.statement.as_str(),
                )
            })
            .collect();
        assert_eq!(
            rows,
            [
                (
                    "cap:omega",
                    OMEGA,
                    Some(CapabilitySection::Contract),
                    contract_line,
                    "**Result.** Text for Contract of omega.\nThis demonstration describes the supported behavior of its library, tests and examples, which readers can inspect together with the generated document relationships.\n"
                ),
                (
                    "cap:omega",
                    OMEGA,
                    Some(CapabilitySection::Usage),
                    usage_line,
                    "Text for Usage of omega.\nThis demonstration describes the supported behavior of its library, tests and examples, which readers can inspect together with the generated document relationships.\nCall `demo::example_crate::show` through its documented executable example.\n"
                ),
            ]
        );
        // The slug alone names the same record.
        assert_eq!(plan.evidence_claims("bound").expect("claims"), claims);
        // A claim on a record that is not there still rests on it.
        assert_eq!(plan.evidence_claims("ghost").expect("claims").len(), 1);
        assert_semantic_claims(plan, "bound");
        assert_semantic_claims(plan, "ghost");
        let item = plan.evidence_claims("other").expect("claims");
        assert_eq!(item[0].statement, "- It holds too.\n");
        assert_eq!(item[0].line, item_line);

        let support = plan.evidence_for("cap:omega").expect("support");
        let row = |s: &Support| (s.evidence.clone(), s.verdict, s.listed, s.claims.clone());
        assert_eq!(
            support.iter().map(row).collect::<Vec<_>>(),
            [
                (
                    "evidence:bound".to_string(),
                    Some(EvidenceVerdict::Pass),
                    true,
                    vec![contract_line, usage_line]
                ),
                ("evidence:ghost".to_string(), None, false, vec![usage_line]),
                (
                    "evidence:other".to_string(),
                    Some(EvidenceVerdict::Fail),
                    true,
                    vec![item_line]
                ),
            ]
        );
        assert_eq!(support[0].path, "work/evidence/bound.md");
    });
    let mut repeated = claimed(&[], &[&bound]);
    let before_sections = "Repeated statement.\n<!-- raf:claim evidence=bound -->\n\nRepeated statement.\n<!-- raf:claim evidence=bound -->\n\n";
    let document = repeated.get_mut(OMEGA).unwrap();
    *document = document.replacen("## Purpose", &format!("{before_sections}## Purpose"), 1);
    with_plan(&repeated, |plan| {
        let claims = plan.evidence_claims("bound").unwrap();
        assert_eq!(claims.len(), 2);
        assert_eq!(claims[0].statement, claims[1].statement);
        assert!(claims.iter().all(|claim| claim.section.is_none()));
        let graph = plan.evidence_claims_dataset("bound").unwrap();
        assert_eq!(graph.reifiers().count(), 2);
        assert_semantic_claims(plan, "bound");
    });
}

fn assert_semantic_claims(plan: &Workplan<'_>, evidence: &str) {
    use purrdf_core::{RdfDataset, TermId, TermRef};
    use raf_okf::DocType;
    use raf_workplan::evidence::CLAIMS_VOCAB;
    let graph = plan.evidence_claims_dataset(evidence).unwrap();
    assert_diagram_bundle(plan, evidence, &graph);
    let claims = plan.evidence_claims(evidence).unwrap();
    let evidence = DocType::Evidence
        .document_iri(&format!("evidence:{evidence}"))
        .unwrap();
    let value = |graph: &RdfDataset, subject: TermId, name: &str| {
        let property = format!("{CLAIMS_VOCAB}{name}");
        graph
            .quads()
            .filter(|q| q.s == subject && graph.term_value(q.p).as_iri() == Some(property.as_str()))
            .map(|q| graph.term_value(q.o))
            .collect::<Vec<_>>()
    };
    let selection = graph
        .quads()
        .find(|q| {
            graph.term_value(q.p).as_iri() == Some(format!("{CLAIMS_VOCAB}evidence").as_str())
        })
        .unwrap();
    assert_eq!(
        graph.term_value(selection.o).as_iri(),
        Some(evidence.as_str())
    );
    assert_eq!(graph.reifiers().count(), claims.len());
    assert_eq!(value(&graph, selection.s, "occurrence").len(), claims.len());
    for (occurrence, target) in graph.reifiers() {
        let TermRef::Triple { s, p, o } = graph.resolve(target) else {
            panic!("claim must be a triple term")
        };
        assert_eq!(
            graph.term_value(p).as_iri(),
            Some(format!("{CLAIMS_VOCAB}claimedStatement").as_str())
        );
        assert!(
            !graph.quads().any(|q| q.s == s && q.p == p && q.o == o),
            "quoted statement must remain unasserted"
        );
        let purrdf_core::TermValue::Literal { lexical_form, .. } =
            &value(&graph, occurrence, "line")[0]
        else {
            panic!("line must be an integer literal")
        };
        let line = lexical_form.parse::<usize>().unwrap();
        let claim = claims.iter().find(|claim| claim.line == line).unwrap();
        assert_eq!(
            graph.term_value(s).as_iri(),
            Some(
                DocType::Capability
                    .document_iri(&claim.capability)
                    .unwrap()
                    .as_str()
            )
        );
        assert_eq!(
            graph.term_value(o),
            purrdf_core::TermValue::simple_literal(&claim.statement)
        );
        assert_eq!(
            value(&graph, occurrence, "path"),
            [purrdf_core::TermValue::simple_literal(&claim.path)]
        );
        assert_eq!(
            value(&graph, occurrence, "supportedBy"),
            [purrdf_core::TermValue::iri(&evidence)]
        );
        assert_eq!(
            value(&graph, occurrence, "section"),
            claim
                .section
                .map(|section| purrdf_core::TermValue::simple_literal(section.slug()))
                .into_iter()
                .collect::<Vec<_>>()
        );
    }
}

fn assert_diagram_bundle(plan: &Workplan<'_>, evidence: &str, graph: &purrdf_core::RdfDataset) {
    use purrdf_lex::json::{self, record::FromJson as _};
    use purrdf_rdf::viz::{VizExport, VizRelation, VizTermValue};
    use raf_workplan::diagram::{DiagramError, DiagramScope, NamedDiagramView};
    let view = NamedDiagramView::new(
        "claim-references",
        DiagramScope::Evidence {
            id: evidence.to_owned(),
        },
    )
    .unwrap();
    let bundle = plan.diagram(&view).unwrap();
    let alias = NamedDiagramView::new("claim-references", view.scope().clone()).unwrap();
    let repeated = plan.diagram(&alias).unwrap();
    assert_eq!(bundle.svg(), repeated.svg());
    assert_eq!(bundle.json(), repeated.json());
    assert_eq!(bundle.text(), repeated.text());
    let encode = |dataset: &purrdf_core::RdfDataset| {
        purrdf_rdf::serialize_dataset_with(
            dataset,
            purrdf_rdf::NativeRdfFormat::NTriples,
            None,
            &purrdf_rdf::SerializeOptions {
                selection: purrdf_rdf::SerializeGraph::Dataset,
                statement_layer: purrdf_rdf::StatementLayer::Emit,
                jsonld_options: None,
            },
        )
        .unwrap()
        .bytes
    };
    assert_eq!(
        encode(bundle.dataset()),
        encode(graph),
        "diagram changed its semantic selection"
    );
    let envelope = json::read(bundle.json()).unwrap();
    assert_eq!(
        VizExport::from_json(&envelope["export"]).unwrap(),
        *bundle.export()
    );
    assert!(bundle.svg().contains(&bundle.export().model_hash));
    let claims = plan.evidence_claims(evidence).unwrap();
    assert_eq!(
        bundle
            .export()
            .model
            .relations
            .iter()
            .filter(|relation| matches!(relation, VizRelation::Reifies { .. }))
            .count(),
        claims.len()
    );
    if !claims.is_empty() {
        let predicate = bundle.export().model.terms.iter().find(|term| matches!(&term.value,
            VizTermValue::Iri { value } if value == &format!("{}claimedStatement", raf_workplan::evidence::CLAIMS_VOCAB))).unwrap();
        let quoted: Vec<_> = bundle
            .export()
            .model
            .statements
            .iter()
            .filter(|statement| statement.predicate == predicate.id)
            .collect();
        assert_ne!(quoted.len(), 0, "authored claim has no native statement");
        assert!(
            quoted
                .iter()
                .all(|statement| statement.asserted_in.is_empty()),
            "authored claims became assertions"
        );
    }
    for edge in &bundle.export().scene.edges {
        assert!(
            bundle.text().contains(&edge.accessibility.description),
            "readable alternative lost a relationship"
        );
    }
    let limited = view
        .clone()
        .with_limits(std::num::NonZeroUsize::MIN, std::num::NonZeroUsize::MIN);
    assert!(
        matches!(plan.diagram(&limited), Err(DiagramError::Native(_))),
        "an exceeded limit returned a partial bundle"
    );
    assert!(NamedDiagramView::new("../invalid", view.scope().clone()).is_err());
    assert!(
        NamedDiagramView::new(
            "valid-name",
            DiagramScope::Evidence {
                id: "Bad Id".to_owned()
            }
        )
        .is_err()
    );
}

#[test]
fn a_record_supports_a_capability_by_its_for_list_without_a_claim() {
    let bound = Evidence::passing("bound", "[cap:omega]");
    with_plan(&claimed(&[], &[&bound]), |plan| {
        let support = plan.evidence_for("cap:omega").expect("support");
        assert_eq!(support.len(), 1);
        assert!(support[0].listed && support[0].claims.is_empty());
        // No claim names it, and the record is there.
        assert_eq!(plan.evidence_claims("bound").expect("claims"), []);
        assert_semantic_claims(plan, "bound");
    });
    // Naming the same result in ordinary prose creates no inferred edge.
    let prose = claimed(
        &[(
            "Contract",
            "\nThe measurement evidence:bound supports omega.\n",
        )],
        &[&bound],
    );
    with_plan(&prose, |plan| {
        assert_eq!(plan.evidence_claims("bound").expect("claims"), []);
        let support = plan.evidence_for("cap:omega").expect("support");
        assert_eq!(support.len(), 1);
        assert!(support[0].listed && support[0].claims.is_empty());
    });
}

#[test]
fn a_handwritten_record_can_exceed_the_writer_value_limit_but_not_the_document_bound() {
    let mut evidence = Evidence::passing("bound", "[cap:omega]");
    evidence.values = (0..1025)
        .map(|index| (format!("v{index}"), "1".to_string()))
        .collect();
    let mut repo = claimed(&[], &[&evidence]);
    with_plan(&repo, |plan| {
        let records = plan.evidence();
        assert_eq!(records.len(), 1);
        let values = &records[0].value().values;
        assert_eq!(values.len(), 1025);
        assert_eq!(
            values
                .iter()
                .map(|value| value.name.as_str())
                .collect::<Vec<_>>(),
            evidence
                .values
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>()
        );
        assert!(values.iter().all(|value| value.value.text() == "1"));
    });
    let path = evidence.path();
    let text = repo.get_mut(&path).expect("the authored record");
    let padding = raf_okf::MAX_TEXT_BYTES - text.len() - 9;
    text.push_str("\n<!--");
    text.extend(std::iter::repeat_n('x', padding));
    text.push_str("-->\n");
    assert_eq!(text.len(), raf_okf::MAX_TEXT_BYTES);
    with_plan(&repo, |plan| {
        assert_eq!(plan.evidence().len(), 1);
        assert_eq!(plan.evidence()[0].value().values.len(), 1025);
    });
    let text = repo.get_mut(&path).expect("the authored record");
    text.insert(text.len() - 4, 'x');
    assert_eq!(text.len(), raf_okf::MAX_TEXT_BYTES + 1);
    with_plan(&repo, |plan| {
        assert_eq!(plan.evidence(), []);
        let findings: Vec<_> = plan
            .check()
            .into_iter()
            .filter(|finding| finding.file == path)
            .collect();
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].kind, "okf-parse-too-large");
    });
}

#[test]
fn a_view_of_what_does_not_exist_is_an_error_and_its_neighbour_answers() {
    let bound = Evidence::passing("bound", "[cap:omega]");
    with_plan(&claimed(&[], &[&bound]), |plan| {
        assert!(matches!(
            plan.evidence_claims("evidence:Bad Id"),
            Err(EvidenceLookupError::BadId { .. })
        ));
        assert!(matches!(
            plan.evidence_claims("nothing"),
            Err(EvidenceLookupError::UnknownEvidence { .. })
        ));
        assert!(matches!(
            plan.evidence_for("cap:ghost"),
            Err(EvidenceLookupError::Capability {
                not_loaded: false,
                ..
            })
        ));
        assert!(plan.evidence_for("cap:omega").is_ok());
    });
    let broken = replaced(
        &claimed(&[], &[]),
        OMEGA,
        "state: delivered",
        "state: unknown",
    );
    with_plan(&broken, |plan| {
        assert!(matches!(
            plan.evidence_for("cap:omega"),
            Err(EvidenceLookupError::Capability {
                not_loaded: true,
                ..
            })
        ));
    });
}

#[test]
fn the_claims_of_a_record_are_refused_while_a_capability_document_does_not_load() {
    let bound = Evidence::passing("bound", "[cap:omega]");
    let clean = claimed(&[("Contract", &claim("bound"))], &[&bound]);
    // A second capability that does not load may hold a claim on the record,
    // so no answer about the claims is complete: each id is refused.
    let mut broken = clean.clone();
    broken.insert(
        "docs/capabilities/zeta.md".to_string(),
        capability("zeta", "unknown", BOUND, &[]),
    );
    with_plan(&broken, |plan| {
        for id in ["bound", "evidence:bound", "nothing"] {
            let error = plan.evidence_claims(id).expect_err("refused");
            assert!(matches!(
                plan.evidence_claims_dataset(id),
                Err(raf_workplan::evidence::ClaimsDatasetError::Lookup(
                    EvidenceLookupError::UnloadedCapabilities { .. }
                ))
            ));
            assert_eq!(
                error,
                EvidenceLookupError::UnloadedCapabilities {
                    ids: vec!["cap:zeta".to_string()]
                },
                "{id}"
            );
            assert!(
                error.to_string().contains("cap:zeta did not load"),
                "{error}"
            );
        }
        // A bad id is still a bad id.
        assert!(matches!(
            plan.evidence_claims("Bad Id"),
            Err(EvidenceLookupError::BadId { .. })
        ));
        assert!(matches!(
            plan.evidence_claims_dataset("Bad Id"),
            Err(raf_workplan::evidence::ClaimsDatasetError::Lookup(
                EvidenceLookupError::BadId { .. }
            ))
        ));
    });
    // A change record or an evidence record that does not load holds no
    // claim: the answer stands.
    let change = replaced(&clean, SHAPE, "outcome: implemented", "outcome: maybe");
    let record = {
        let mut next = clean.clone();
        next.insert(
            "work/evidence/other.md".to_string(),
            Evidence::passing("other", "[cap:omega]")
                .verdict("maybe")
                .text(),
        );
        next
    };
    for (name, repo) in [("clean", &clean), ("change", &change), ("record", &record)] {
        with_plan(repo, |plan| {
            // The neighbours do not load what they break.
            assert_eq!(
                plan.changes().len(),
                usize::from(name != "change"),
                "{name}"
            );
            assert_eq!(plan.evidence().len(), 1, "{name}");
            let claims = plan.evidence_claims("bound").expect(name);
            assert_eq!(claims.len(), 1, "{name}");
            assert_eq!(claims[0].capability, "cap:omega", "{name}");
        });
    }
}

// --- a record from a bound test -------------------------------------------------

/// The `cargo` that runs this test binary.
fn cargo() -> PathBuf {
    std::env::var_os("CARGO")
        .map(PathBuf::from)
        .or_else(|| option_env!("CARGO").map(PathBuf::from))
        .expect("cargo test sets CARGO")
}

const LIB: &str = "\
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[cfg(test)]
mod tests {
    #[test]
    fn passes() {
        assert_eq!(super::add(1, 2), 3);
    }

    #[test]
    fn fails() {
        panic!(\"this test fails\");
    }
}
";

/// A crate with its own workspace, lock file and target directory, and the
/// capability `fixture` that binds `add` and both tests.
fn crate_fixture() -> Scratch {
    let scratch = Scratch::new("evidence-binding");
    let root = scratch.path();
    put(
        root,
        "Cargo.toml",
        "[package]\nname = \"fx-ev\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
    );
    put(
        root,
        "Cargo.lock",
        "version = 4\n\n[[package]]\nname = \"fx-ev\"\nversion = \"0.1.0\"\n",
    );
    put(root, ".cargo/config.toml", &scratch.cargo_config());
    put(root, "src/lib.rs", LIB);
    put(
        root,
        "docs/capabilities/fixture.md",
        &capability(
            "fixture",
            "implementing",
            [
                "[fx_ev::add]",
                "[fx_ev::tests::passes, fx_ev::tests::fails]",
                "[]",
            ],
            &[],
        ),
    );
    scratch
}

fn from_binding(path: &str) -> EvidenceRequest {
    EvidenceRequest::new(
        "bound",
        vec!["cap:fixture".to_string()],
        Actor::Human,
        VerdictSource::Binding(path.to_string()),
    )
}

fn record_in(
    root: &Path,
    runner: &dyn Runner,
    request: &EvidenceRequest,
) -> Result<raf_workplan::evidence::Recorded, EvidenceError> {
    let opened = Gate::workspace(root)
        .tracked_paths(Vec::new())
        .open()
        .expect("the fixture opens");
    opened.workplan().record_evidence(root, runner, request)
}

#[test]
fn a_bound_test_gives_the_verdict_and_a_path_that_names_no_binding_starts_nothing() {
    let scratch = crate_fixture();
    let root = scratch.path();
    let docs = |files: &std::collections::BTreeMap<String, Vec<u8>>| {
        files
            .iter()
            .filter(|(path, _)| path.starts_with("docs/"))
            .map(|(path, bytes)| (path.clone(), bytes.clone()))
            .collect::<Vec<_>>()
    };
    let before = snapshot(root);

    // A path that is no test or example of the capability: nothing starts.
    let runner = FakeRunner::new();
    for path in [
        "fx_ev::add",
        "fx_ev::tests::elsewhere",
        "other::tests::passes",
    ] {
        let error = record_in(root, &runner, &from_binding(path)).expect_err(path);
        assert!(
            matches!(&error, EvidenceError::NoSuchBinding { bound, .. }
                if bound == &["fx_ev::tests::fails".to_string(), "fx_ev::tests::passes".to_string()]),
            "{path}: {error}"
        );
    }
    assert_eq!(runner.calls().len(), 0);

    let real = SystemRunner::new(Programs::with_cargo_var(root, Some(cargo().into())));
    let passed = record_in(root, &real, &from_binding("fx_ev::tests::passes")).expect("recorded");
    assert_eq!(passed.record.verdict, EvidenceVerdict::Pass);
    assert_eq!(
        passed.record.subject,
        EvidenceSubject::Binding("fx_ev::tests::passes".to_string())
    );
    assert_eq!(passed.record.command, None);
    assert_eq!(passed.record.by, Actor::Human);
    let source = Workspace::load(
        &MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname = \"fx-ev\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
            ),
            ("src/lib.rs", LIB),
        ]),
        &["."],
    )
    .expect("the library loads");
    assert_eq!(passed.record.inputs.len(), 1);
    assert_eq!(
        passed.record.inputs[0].digest,
        digest_of(&source, "fx_ev::add")
    );
    let text = std::fs::read_to_string(root.join("work/evidence/bound.md")).expect("written");
    assert!(
        text.contains("subject: binding:fx_ev::tests::passes\n"),
        "{text}"
    );
    assert!(!text.contains("command:"), "{text}");

    let failed = record_in(
        root,
        &real,
        &from_binding("fx_ev::tests::fails").mode(raf_workplan::evidence::WriteMode::Replace),
    )
    .expect("a failing test is recorded");
    assert_eq!(failed.record.verdict, EvidenceVerdict::Fail);
    assert!(matches!(failed.failed.as_slice(), [Failed::Binding(_)]));

    // Recording changed no document.
    assert_eq!(docs(&snapshot(root)), docs(&before));
}

// --- staleness from the source --------------------------------------------------

/// The demo workspace with its library replaced by `lib`.
fn demo_with(lib: &str) -> Workspace {
    let files: MemoryTree = support::demo_files()
        .into_iter()
        .map(|(path, text)| (path, if path == "src/lib.rs" { lib } else { text }))
        .collect();
    Workspace::load(&files, &["."]).expect("the demo loads")
}

/// The obligations of `shape` over `repo` and the source `source`.
fn reconcile_over(repo: &Repo, source: &Workspace) -> Reconciled {
    let tree: MemoryTree = repo.iter().collect();
    Workplan::load(&tree, source)
        .reconcile("shape")
        .expect("reconcile answers")
}

#[test]
fn an_edit_of_a_bound_symbol_makes_its_record_stale_and_nothing_else_does() {
    let bound = Evidence::passing("bound", "[cap:omega]");
    let repo = claimed(&[("Contract", &claim("bound"))], &[&bound]);
    let lib = support::DEMO_LIB;
    let stale = |source: &Workspace| -> Vec<String> {
        reconcile_over(&repo, source)
            .of_kind(ObligationKind::EvidenceStale)
            .map(ToString::to_string)
            .collect()
    };
    assert_eq!(stale(&demo_with(lib)), Vec::<String>::new());

    // The body of the bound function changes: the record is stale, naming it.
    let edited = lib.replace("pub fn real() {}", "pub fn real() { let _ = 1; }");
    assert_ne!(edited, lib);
    assert_eq!(
        stale(&demo_with(&edited)),
        [
            "cap:omega  document  evidence-stale  the evidence recorded again with evidence record (`evidence:bound` was made against `demo::real`, which has changed since)"
        ]
    );
    // Its documentation is part of its text too.
    let documented = lib.replace("/// A documented function.", "/// A function, documented.");
    assert_eq!(stale(&demo_with(&documented)).len(), 1);
    let renamed = lib.replace("pub fn real() {}", "pub fn renamed() {}");
    let missing = stale(&demo_with(&renamed));
    assert_eq!(missing.len(), 1, "{missing:?}");
    assert!(
        missing[0].contains("`demo::real`, which does not resolve now:"),
        "{missing:?}"
    );

    // Revision is reader metadata. Pure in-memory reconciliation has no Git
    // runner and judges the same current source under either accepted width.
    for revision in ["a".repeat(40), "b".repeat(64)] {
        let tagged = replaced(
            &repo,
            &bound.path(),
            "by: agent\n",
            &format!("by: agent\nrevision: {revision}\n"),
        );
        for (text, expected) in [
            (lib, Vec::new()),
            (edited.as_str(), stale(&demo_with(&edited))),
        ] {
            let actual: Vec<_> = reconcile_over(&tagged, &demo_with(text))
                .of_kind(ObligationKind::EvidenceStale)
                .map(ToString::to_string)
                .collect();
            assert_eq!(actual, expected);
        }
    }

    // Another item edited, an item added before it, and every line ending
    // turned to CRLF leave it current.
    let other = lib.replace(
        "pub fn bare() { generated(); }",
        "pub fn bare() { generated(); let _ = 2; }",
    );
    let before = format!("pub fn first() {{}}\n{lib}");
    let crlf = lib.replace('\n', "\r\n");
    for neighbour in [other, before, crlf] {
        assert_eq!(stale(&demo_with(&neighbour)), Vec::<String>::new());
    }
}

/// The demo workspace with the first `from` of `path` replaced by `to`.
fn demo_editing(path: &str, from: &str, to: &str) -> Workspace {
    let files: Vec<(&str, String)> = support::demo_files()
        .into_iter()
        .map(|(file, text)| {
            if file == path {
                assert!(text.contains(from), "{file} has no `{from}`");
                (file, text.replacen(from, to, 1))
            } else {
                (file, text.to_string())
            }
        })
        .collect();
    let tree: MemoryTree = files.iter().map(|(p, t)| (*p, t.as_str())).collect();
    Workspace::load(&tree, &["."]).expect("the demo loads")
}

#[test]
fn an_edit_inside_a_bound_module_makes_its_record_stale_and_a_sibling_edit_does_not() {
    // `demo::body` is a file module (`src/body/mod.rs`) with the child module
    // `blocks` in its own file; `demo::inner` is an inline sibling.
    let demo = support::demo_workspace();
    let bound = Evidence::passing("bound", "[cap:omega]").inputs(vec![(
        "demo::body".to_string(),
        digest_of(&demo, "demo::body"),
    )]);
    let repo = claimed(&[("Contract", &claim("bound"))], &[&bound]);
    let stale = |source: &Workspace| -> Vec<String> {
        reconcile_over(&repo, source)
            .of_kind(ObligationKind::EvidenceStale)
            .map(ToString::to_string)
            .collect()
    };
    assert_eq!(stale(&demo), Vec::<String>::new());
    let named = "cap:omega  document  evidence-stale  the evidence recorded again with evidence record (`evidence:bound` was made against `demo::body`, which has changed since)";
    for (path, from, to) in [
        (
            "src/body/mod.rs",
            "pub fn in_body() {}",
            "pub fn in_body() { let _ = 1; }",
        ),
        (
            "src/body/blocks.rs",
            "pub(crate) struct FenceBlock;",
            "pub(crate) struct FenceBlock(u8);",
        ),
    ] {
        assert_eq!(stale(&demo_editing(path, from, to)), [named], "{path}");
    }
    let sibling = demo_editing("src/lib.rs", "pub fn go() {}", "pub fn go() { let _ = 2; }");
    assert_eq!(stale(&sibling), Vec::<String>::new());
}

#[test]
fn a_stale_record_of_a_capability_the_change_does_not_list_is_not_owed() {
    // `bound` is for `sigma`, which `shape` does not list.
    let stale = Evidence::passing("bound", "[cap:sigma]").inputs(vec![(
        "demo::real".to_string(),
        support::evidence::OTHER_DIGEST.to_string(),
    )]);
    let mut files = claimed(&[], &[&stale]);
    files.insert(
        "docs/capabilities/sigma.md".to_string(),
        capability("sigma", "delivered", BOUND, &[]),
    );
    let files = support::synced::synced(&files);
    let list = reconcile_over(&files, &support::demo_workspace());
    assert_eq!(
        list.of_kind(ObligationKind::EvidenceStale).count(),
        0,
        "{list}"
    );
    // Listed, it is owed.
    let listed = replaced(
        &files,
        SHAPE,
        "changes: [cap:omega]",
        "changes: [cap:omega, cap:sigma]",
    );
    let list = reconcile_over(&listed, &support::demo_workspace());
    assert_eq!(
        list.of_kind(ObligationKind::EvidenceStale).count(),
        1,
        "{list}"
    );
}

#[test]
fn hostile_record_text_is_read_as_text_and_reaches_no_process() {
    let mut hostile = Evidence::passing("bound", "[cap:omega]")
        .verdict("fail")
        .text();
    hostile = hostile.replace(
        "command: \"cargo bench\"",
        "command: \"$(printf substituted); `printf substituted` | cat\"",
    );
    let mut repo = claimed(&[("Contract", &claim("bound"))], &[]);
    repo.insert("work/evidence/bound.md".to_string(), hostile);
    let list = with_plan(&repo, |plan| plan.reconcile("shape").expect("answers"));
    assert_eq!(
        list.of_kind(ObligationKind::ClaimUnsupported)
            .map(|o| o.evidence.to_string())
            .collect::<Vec<_>>(),
        [
            "a passing record within its limits for each evidence id the claim names, or the claim reworded or removed (`evidence:bound` has the verdict fail)"
        ]
    );
}

// --- evidence-changed against the merge base -------------------------------------

/// The passing record of the git fixtures: a value within its limit.
fn bounded() -> Evidence {
    Evidence::passing("bound", "[cap:omega]")
        .values(&[("peak", "41")])
        .limits(&[("peak", "64")])
}

/// A record that no claim names and that never changes.
fn steady() -> Evidence {
    Evidence::passing("steady", "[cap:omega]")
}

fn git_base() -> Repo {
    claimed(&[("Contract", &claim("bound"))], &[&bounded(), &steady()])
}

fn lifecycle(checkout: &Checkout) -> Lifecycle {
    Lifecycle::open(real_programs(checkout))
        .unwrap_or_else(|error| panic!("the checkout opens: {error}"))
}

/// Reconciliation runs the actual declared example. Its dependency-free demo
/// has a frozen lock and an isolated build directory outside the judged tree.
fn real_programs(checkout: &Checkout) -> Programs {
    checkout.put(
        "Cargo.lock",
        "version = 4\n\n[[package]]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    );
    let target = checkout.scratch().join("cargo-target");
    checkout.put(
        ".cargo/config.toml",
        &format!(
            "[build]\ntarget-dir = '{0}'\nbuild-dir = '{0}'\n",
            target.display()
        ),
    );
    Programs::new(checkout.root())
        .discover_git()
        .with_cargo(cargo())
}

fn changed_lines(checkout: &Checkout) -> Vec<String> {
    lifecycle(checkout)
        .reconcile("shape")
        .unwrap_or_else(|error| panic!("reconcile answers: {error}"))
        .of_kind(ObligationKind::EvidenceChanged)
        .map(ToString::to_string)
        .collect()
}

const REVIEW: &str = "the claim reviewed against the new result: the evidence passes within the limits of the merge base again, or the claim no longer names it";

#[test]
fn each_change_of_a_passing_record_since_the_merge_base_reopens_its_claim() {
    let _lock = serial();
    let base = git_base();
    let line = support::line_of(&base[OMEGA], "raf:claim");
    let at = |detail: &str| {
        format!("cap:omega  contract  evidence-changed  line {line}: {REVIEW} ({detail})")
    };
    let checkout = Checkout::new("evidence-changed", &base);
    let path = bounded().path();

    // Nothing changed: nothing is owed, and the list is the same twice.
    assert_eq!(changed_lines(&checkout), Vec::<String>::new());
    let first = lifecycle(&checkout).reconcile("shape").expect("answers");
    assert_eq!(
        lifecycle(&checkout).reconcile("shape").expect("answers"),
        first
    );

    for (record, want) in [
        (
            bounded().verdict("fail"),
            vec![at(
                "`evidence:bound` passed at the merge base and fails now",
            )],
        ),
        (
            bounded().verdict("fail").values(&[("peak", "80")]),
            vec![
                at("`evidence:bound` passed at the merge base and fails now"),
                at("`evidence:bound`: `peak` is 80, over its limit 64"),
            ],
        ),
        (
            bounded().limits(&[]),
            vec![at(
                "`evidence:bound` no longer has the limit `peak` it had at the merge base",
            )],
        ),
        // A limit loosened past the value: over the limit of the merge base.
        (
            bounded()
                .values(&[("peak", "80")])
                .limits(&[("peak", "100")]),
            vec![at("`evidence:bound`: `peak` is 80, over its limit 64")],
        ),
    ] {
        checkout.put(&path, &record.text());
        let mut got = changed_lines(&checkout);
        got.sort();
        let mut want = want;
        want.sort();
        assert_eq!(got, want, "{}", record.text());
        // Committed on the branch, it is still a change since the merge base.
        checkout.commit("a new result");
        let mut got = changed_lines(&checkout);
        got.sort();
        assert_eq!(got, want);
    }

    // Removed: owed as changed, and unsupported too.
    std::fs::remove_file(checkout.root().join(&path)).expect("removed");
    let list = lifecycle(&checkout).reconcile("shape").expect("answers");
    let lines: Vec<String> = list
        .of_kind(ObligationKind::EvidenceChanged)
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        lines,
        [at(
            "`evidence:bound` passed at the merge base and is not a loaded record now"
        )]
    );
    assert_eq!(list.of_kind(ObligationKind::ClaimUnsupported).count(), 1);

    // The passing neighbour: the record of the merge base again.
    checkout.put(&path, &bounded().text());
    assert_eq!(changed_lines(&checkout), Vec::<String>::new());
    // A claim that no longer names the record owes nothing for it.
    checkout.put(&path, &bounded().verdict("fail").text());
    assert_eq!(changed_lines(&checkout).len(), 1);
    let text = std::fs::read_to_string(checkout.root().join(OMEGA)).expect("omega");
    assert!(text.contains(&claim("bound")));
    checkout.put(OMEGA, &text.replacen(&claim("bound"), "", 1));
    assert_eq!(changed_lines(&checkout), Vec::<String>::new());
}

#[test]
fn a_record_that_failed_at_the_merge_base_or_was_not_there_is_compared_with_nothing() {
    let _lock = serial();
    let failing = claimed(
        &[("Contract", &claim("bound"))],
        &[&bounded().verdict("fail")],
    );
    let checkout = Checkout::new("evidence-failing-base", &failing);
    checkout.put(
        &bounded().path(),
        &bounded().verdict("fail").values(&[("peak", "90")]).text(),
    );
    assert_eq!(changed_lines(&checkout), Vec::<String>::new());

    let absent = claimed(&[("Contract", &claim("bound"))], &[]);
    let checkout = Checkout::new("evidence-new", &absent);
    checkout.put(&bounded().path(), &bounded().verdict("fail").text());
    assert_eq!(changed_lines(&checkout), Vec::<String>::new());
}

#[test]
fn a_record_at_the_merge_base_that_does_not_read_is_an_error_and_not_a_pass() {
    let _lock = serial();
    let mut broken = git_base();
    broken.insert(
        bounded().path(),
        bounded().text().replace("verdict: pass", "verdict: maybe"),
    );
    let checkout = Checkout::new("evidence-broken-base", &broken);
    checkout.put(&bounded().path(), &bounded().text());
    let error = lifecycle(&checkout)
        .reconcile("shape")
        .expect_err("an unreadable record at the merge base");
    assert!(
        matches!(&error, LifecycleError::Document { path, side: Side::MergeBase, .. }
            if path == "work/evidence/bound.md"),
        "{error:?}"
    );
    // The neighbour: committed whole, it reads.
    let checkout = Checkout::new("evidence-good-base", &git_base());
    assert!(lifecycle(&checkout).reconcile("shape").is_ok());
}

#[test]
fn evidence_invalidation_needs_history_only_when_a_claim_names_a_record() {
    let _lock = serial();
    // Isolate evidence invalidation from the separate token guard and
    // section-untouched rules, which also legitimately read history.
    let base = replaced(
        &git_base(),
        SHAPE,
        "contract_effect: none",
        "contract_effect: changes",
    );
    let base = replaced(
        &base,
        SHAPE,
        "affects:\n  - cap:omega#contract\n",
        "affects: []\n",
    );
    let checkout = Checkout::new("evidence-no-origin", &base);
    checkout.git(&["symbolic-ref", "--delete", "refs/remotes/origin/HEAD"]);
    let error = lifecycle(&checkout)
        .reconcile("shape")
        .expect_err("no default branch");
    assert!(
        matches!(
            &error,
            LifecycleError::Git(GitError::NoDefaultBranch { .. })
        ),
        "{error:?}"
    );
    // Without a claim nothing needs the merge base.
    let unclaimed = without(&base, &bounded().path());
    let unclaimed = replaced(&unclaimed, OMEGA, &claim("bound"), "");
    let checkout = Checkout::new("evidence-no-origin-unclaimed", &unclaimed);
    checkout.git(&["symbolic-ref", "--delete", "refs/remotes/origin/HEAD"]);
    lifecycle(&checkout)
        .reconcile("shape")
        .expect("without a claim evidence invalidation needs no history");
}

/// A runner that runs through the real one and records each command.
struct Recording {
    inner: SystemRunner,
    commands: Mutex<Vec<String>>,
}

impl Runner for Recording {
    fn run(&self, invocation: &Invocation, limits: &Limits) -> Result<Output, ExecError> {
        self.commands
            .lock()
            .expect("a lock")
            .push(invocation.to_string());
        self.inner.run(invocation, limits)
    }
}

#[test]
fn each_named_record_is_read_once_at_the_merge_base_with_the_blob_row() {
    let _lock = serial();
    // Two claims on one record, a record no claim names, and a workspace
    // below the top of its repository.
    let base = claimed(
        &[("Contract", &claim("bound")), ("Usage", &claim("bound"))],
        &[&bounded(), &steady()],
    );
    let checkout = Checkout::below("evidence-argv", &base, "services/api");
    let programs = real_programs(&checkout);
    let runner = Recording {
        inner: SystemRunner::new(programs.clone()),
        commands: Mutex::new(Vec::new()),
    };
    let gate = Gate::workspace(checkout.root()).programs(programs);
    let lifecycle = Lifecycle::open_with(&gate, runner).expect("opens");
    let owed = lifecycle.reconcile("shape").expect("answers");
    assert_eq!(owed.of_kind(ObligationKind::EvidenceChanged).count(), 0);
    let commands = lifecycle.runner().commands.lock().expect("a lock").clone();
    let base = checkout.git(&["rev-parse", "refs/remotes/origin/main"]);
    let mut expected = vec![
        "git --version".to_string(),
        "git rev-parse --show-prefix".to_string(),
        "git rev-parse --is-inside-work-tree".to_string(),
        "git ls-files -z".to_string(),
        "git symbolic-ref refs/remotes/origin/HEAD".to_string(),
        "git rev-parse --verify --end-of-options refs/remotes/origin/main^{commit}".to_string(),
        format!("git merge-base HEAD {base}"),
        format!("git cat-file blob {base}:services/api/work/evidence/bound.md"),
        format!("git ls-tree -r -z --full-tree {base}"),
    ];
    // The independent contract-effect guard reads the real prior source.
    // Neither repeated claims nor the unclaimed steady record add reads.
    for path in [
        "Cargo.toml",
        "src/lib.rs",
        "src/body/mod.rs",
        "src/body/blocks.rs",
        "tests/it.rs",
        "examples/flagged.rs",
        "examples/show.rs",
    ] {
        expected.push(format!("git cat-file blob {base}:services/api/{path}"));
    }
    expected.push("cargo run --frozen --package=demo --example=show".to_string());
    assert_eq!(commands, expected);
}

#[test]
fn a_claim_on_a_value_over_its_limit_by_less_than_a_float_tells_apart_is_unsupported() {
    let one_e400 = format!("1{}", "0".repeat(400));
    let two_e400 = format!("2{}", "0".repeat(400));
    let cases: [(&str, &str, &str); 4] = [
        ("9007199254740993", "9007199254740992", "9007199254740991"),
        ("0.30000000000000001", "0.3", "0.29999999999999999"),
        ("1.0000000000000000001", "1", "0.9999999999999999999"),
        (&two_e400, &one_e400, &one_e400[..400]),
    ];
    let unsupported = |verdict: &'static str, value: &str, limit: &str| -> Vec<String> {
        let record = Evidence::passing("bound", "[cap:omega]")
            .verdict(verdict)
            .values(&[("peak", value)])
            .limits(&[("peak", limit)]);
        let steady = Evidence::passing("steady", "[cap:omega]");
        let uncited = Evidence::passing("uncited", "[cap:omega]").verdict("fail");
        let answer = |names: &str| -> Vec<String> {
            let repo = claimed(
                &[("Contract", &claim(names))],
                &[&record, &steady, &uncited],
            );
            with_plan(&repo, |plan| {
                assert_eq!(plan.evidence().len(), 3, "{:#?}", plan.check());
                plan.reconcile("shape")
                    .expect("answers")
                    .of_kind(ObligationKind::ClaimUnsupported)
                    .map(|o| o.evidence.to_string())
                    .collect()
            })
        };
        let forward = answer("bound,steady");
        assert_eq!(
            forward,
            answer("steady,bound"),
            "each named record is judged, in either order"
        );
        forward
    };
    for (over, limit, under) in cases {
        let lines = unsupported("fail", over, limit);
        assert_eq!(lines.len(), 2, "{lines:#?}");
        assert!(
            lines[1].ends_with(&format!("`peak` is {over}, over its limit {limit})")),
            "{lines:#?}"
        );
        // Passing records at the limit and just under it are supported.
        assert_eq!(unsupported("pass", limit, limit), Vec::<String>::new());
        assert_eq!(unsupported("pass", under, limit), Vec::<String>::new());
    }
}

#[test]
fn an_opposite_host_binding_cannot_publish_execution_evidence() {
    let scratch = crate_fixture();
    let root = scratch.path();
    let guard = if cfg!(windows) { "unix" } else { "windows" };
    put(
        root,
        "src/lib.rs",
        &format!(
            "{LIB}\n#[cfg({guard})]\n#[test]\nfn opposite_host() {{ panic!(\"must not run\"); }}\n"
        ),
    );
    let path = "docs/capabilities/fixture.md";
    let document = std::fs::read_to_string(root.join(path)).expect("fixture document");
    put(
        root,
        path,
        &document.replace(
            "[fx_ev::tests::passes, fx_ev::tests::fails]",
            "[fx_ev::tests::passes, fx_ev::tests::fails, fx_ev::opposite_host]",
        ),
    );
    let before = snapshot(root);
    let runner = Recording {
        inner: SystemRunner::new(Programs::with_cargo_var(root, Some(cargo().into()))),
        commands: Mutex::new(Vec::new()),
    };
    let error = record_in(root, &runner, &from_binding("fx_ev::opposite_host"))
        .expect_err("nonexecution cannot become an evidence verdict");
    let EvidenceError::BindingNotApplicable { path, reason } = &error else {
        panic!("expected typed non-applicability: {error}");
    };
    assert_eq!(path, "fx_ev::opposite_host");
    assert_ne!(reason, "");
    assert!(
        error
            .to_string()
            .contains("no execution evidence was recorded")
    );
    assert_eq!(
        *runner.commands.lock().expect("recorded compiler query"),
        ["cargo rustc --frozen --package=fx-ev --lib -- --print=cfg"]
    );
    assert_eq!(snapshot(root), before);
}
