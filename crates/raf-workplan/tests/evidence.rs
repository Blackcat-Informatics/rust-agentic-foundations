// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Evidence and claims: the dependency views, a record whose verdict comes
//! from a bound test run by a real `cargo`, the refusals of a request, the
//! staleness of a record when its bound code moves (and not when anything else
//! does), and `evidence-changed` against the merge base in real git
//! repositories.

mod support;

use raf_okf::{CapabilitySection, EvidenceVerdict};
use raf_workplan::Workplan;
use raf_workplan::evidence::{EvidenceLookupError, Support};
use support::Repo;
use support::changes::{DONE, OMEGA, SHAPE, capability, record, replaced, repo};
use support::evidence::{Evidence, claim};
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
