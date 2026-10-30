// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Document-owned declarations carried by native RDF and native presentation.

mod domain;

use super::{
    DiagramError,
    semantic::{VOCAB, Writer, doc},
};
use crate::{Workplan, architecture::AuthoredModel, digest::Digest};
use purrdf_core::{RdfDataset, TermId};
use purrdf_rdf::viz::{
    self, VizComponent, VizDisplay, VizInteraction, VizNavigation, VizParticipant, VizPresentation,
    VizProjection, VizRegionKind, VizScenario, VizScenarioRegion, VizSceneLabel, VizSemanticRef,
    VizSpec, VizTermId, VizTermValue, VizValueRef,
};
use raf_okf::{ComponentKind, ModelBindings, Scenario, ScenarioRegionKind};
use std::collections::BTreeSet;
use std::sync::Arc;

const LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";

pub(super) struct ModelScene {
    pub dataset: Arc<RdfDataset>,
    pub projection: VizProjection,
    pub presentation: VizPresentation,
}

fn identity(owner: &str, family: &str, id: &str) -> Result<String, DiagramError> {
    Ok(format!("{}/model/{family}/{id}", doc(owner)?))
}

fn source_identity(path: &str) -> String {
    format!("{VOCAB}source/{}", super::semantic::hash(path))
}

fn link(writer: &mut Writer, subject: TermId, kind: &str, iri: &str) -> Result<(), DiagramError> {
    let object = writer.iri(iri)?;
    writer.put(subject, &format!("{VOCAB}{kind}"), object)
}

fn entity(
    writer: &mut Writer,
    model: AuthoredModel<'_>,
    family: &str,
    id: &str,
    label: &str,
) -> Result<TermId, DiagramError> {
    let subject = writer.iri(&identity(model.owner, family, id)?)?;
    let text = writer.literal(label)?;
    writer.put(subject, LABEL, text)?;
    writer.text(subject, "claimStrength", "declared")?;
    writer.text(subject, "declarationPath", model.path)?;
    link(writer, subject, "declaredBy", &doc(model.owner)?)?;
    Ok(subject)
}

fn bindings(
    writer: &mut Writer,
    plan: &Workplan<'_>,
    subject: TermId,
    bindings: &ModelBindings,
) -> Result<(), DiagramError> {
    for (kind, paths) in [
        ("code", &bindings.code),
        ("publicInterface", &bindings.interfaces),
        ("test", &bindings.tests),
        ("example", &bindings.examples),
    ] {
        source_bindings(writer, plan, subject, kind, paths)?;
    }
    for capability in &bindings.capabilities {
        link(writer, subject, "capability", &doc(capability)?)?;
        let target = plan
            .capabilities()
            .iter()
            .find(|target| target.value().header.id == *capability)
            .ok_or_else(|| {
                DiagramError::refused(format!("capability `{capability}` did not load"))
            })?;
        let digest = Digest::section(target.body(), raf_okf::CapabilitySection::Contract)
            .ok_or_else(|| {
                DiagramError::refused(format!("capability `{capability}` has no contract"))
            })?;
        let node = writer.iri(&doc(capability)?)?;
        writer.text(node, "contractDigest", &digest.to_string())?;
        for (kind, paths) in [
            ("code", &target.value().bindings.code),
            ("test", &target.value().bindings.tests),
            ("example", &target.value().bindings.examples),
        ] {
            source_bindings(writer, plan, node, kind, paths)?;
        }
        writer.text(
            node,
            "declarationDigest",
            &Digest::capability_declaration(target.value()).to_string(),
        )?;
    }
    Ok(())
}

fn source_bindings(
    writer: &mut Writer,
    plan: &Workplan<'_>,
    subject: TermId,
    kind: &str,
    paths: &[String],
) -> Result<(), DiagramError> {
    for path in paths {
        let iri = source_identity(path);
        link(writer, subject, kind, &iri)?;
        let source = writer.iri(&iri)?;
        writer.text(source, "bindingPath", path)?;
        let digest = Digest::symbol(plan.source(), path).map_err(DiagramError::data)?;
        writer.text(source, "sourceDigest", &digest.to_string())?;
    }
    Ok(())
}

fn evidence(
    writer: &mut Writer,
    plan: &Workplan<'_>,
    subject: TermId,
    evidence: &[String],
) -> Result<(), DiagramError> {
    for id in evidence {
        link(writer, subject, "associatedVerification", &doc(id)?)?;
        let record = plan
            .evidence()
            .iter()
            .find(|record| record.value().header.id == *id)
            .ok_or_else(|| {
                DiagramError::refused(format!("verification record `{id}` did not load"))
            })?;
        let verification = writer.iri(&doc(id)?)?;
        writer.text(
            verification,
            "verificationRecordDigest",
            &Digest::record(&record.text()).to_string(),
        )?;
        for (key, value) in super::semantic::verification_fields(plan, record.value())? {
            writer.text(verification, &key, &value)?;
        }
        writer.text(
            subject,
            "verificationBoundary",
            "stored outcome over recorded inputs; no event, order or concurrency witness",
        )?;
    }
    Ok(())
}

fn participant(model: AuthoredModel<'_>, id: &str) -> Result<String, DiagramError> {
    let family = if model.declarations.components.iter().any(|row| row.id == id) {
        "component"
    } else if model.declarations.actors.iter().any(|row| row.id == id) {
        "actor"
    } else {
        return Err(DiagramError::refused(format!(
            "participant `{id}` is absent"
        )));
    };
    identity(model.owner, family, id)
}

fn dataset(
    plan: &Workplan<'_>,
    model: AuthoredModel<'_>,
    limit: usize,
) -> Result<Arc<RdfDataset>, DiagramError> {
    let mut writer = Writer::new(limit);
    let owner = writer.iri(&doc(model.owner)?)?;
    let declaration = plan
        .capabilities()
        .iter()
        .find(|target| target.value().header.id == model.owner)
        .map_or_else(
            || Digest::declaration(model.declarations),
            |target| Digest::capability_declaration(target.value()),
        );
    writer.text(owner, "declarationDigest", &declaration.to_string())?;
    for row in &model.declarations.components {
        let subject = entity(&mut writer, model, "component", &row.id, &row.label)?;
        writer.text(
            subject,
            "componentKind",
            match row.kind {
                ComponentKind::Component => "component",
                ComponentKind::Boundary => "boundary",
            },
        )?;
        writer.text(subject, "responsibility", &row.responsibility)?;
        bindings(&mut writer, plan, subject, &row.bindings)?;
    }
    for row in &model.declarations.actors {
        let subject = entity(&mut writer, model, "actor", &row.id, &row.label)?;
        writer.text(subject, "responsibility", &row.responsibility)?;
    }
    for row in &model.declarations.artifacts {
        let subject = entity(&mut writer, model, "artifact", &row.id, &row.label)?;
        writer.text(subject, "artifactType", &row.artifact_type)?;
        bindings(&mut writer, plan, subject, &row.bindings)?;
    }
    for row in &model.declarations.relations {
        let subject = entity(&mut writer, model, "relation", &row.id, &row.label)?;
        let from = writer.iri(&participant(model, &row.from)?)?;
        let to = writer.iri(&participant(model, &row.to)?)?;
        let statement = writer
            .native
            .intern_triple(from, subject, to)
            .map_err(DiagramError::data)?;
        writer
            .native
            .push_reifier(subject, statement)
            .map_err(DiagramError::data)?;
        if let Some(protocol) = &row.protocol {
            writer.text(subject, "protocol", protocol)?;
        }
        if let Some(artifact) = &row.artifact {
            link(
                &mut writer,
                subject,
                "artifact",
                &identity(model.owner, "artifact", artifact)?,
            )?;
        }
    }
    for scenario in &model.declarations.scenarios {
        scenario_facts(&mut writer, plan, model, scenario)?;
    }
    domain::write(&mut writer, model)?;
    writer.native.freeze().map_err(DiagramError::data)
}

fn scenario_facts(
    writer: &mut Writer,
    plan: &Workplan<'_>,
    model: AuthoredModel<'_>,
    scenario: &Scenario,
) -> Result<(), DiagramError> {
    let subject = entity(writer, model, "scenario", &scenario.id, &scenario.label)?;
    evidence(writer, plan, subject, &scenario.evidence)?;
    for (position, id) in scenario.participants.iter().enumerate() {
        link(
            writer,
            subject,
            &format!("participant{position}"),
            &participant(model, id)?,
        )?;
    }
    for region in &scenario.regions {
        let id = format!("{}/{}", scenario.id, region.id);
        let region_subject = entity(writer, model, "region", &id, &region.label)?;
        link(
            writer,
            region_subject,
            "scenario",
            &identity(model.owner, "scenario", &scenario.id)?,
        )?;
        writer.text(region_subject, "regionKind", region.kind.as_str())?;
        writer.text(region_subject, "order", &region.order.to_string())?;
        if let Some(parent) = &region.parent {
            link(
                writer,
                region_subject,
                "region",
                &identity(model.owner, "region", &format!("{}/{parent}", scenario.id))?,
            )?;
        }
    }
    for step in &scenario.steps {
        let id = format!("{}/{}", scenario.id, step.id);
        let step_subject = entity(writer, model, "step", &id, &step.label)?;
        link(
            writer,
            step_subject,
            "scenario",
            &identity(model.owner, "scenario", &scenario.id)?,
        )?;
        link(
            writer,
            step_subject,
            "relationship",
            &identity(model.owner, "relation", &step.relation)?,
        )?;
        writer.text(step_subject, "order", &step.order.to_string())?;
        if let Some(region) = &step.region {
            link(
                writer,
                step_subject,
                "region",
                &identity(model.owner, "region", &format!("{}/{region}", scenario.id))?,
            )?;
        }
        bindings(writer, plan, step_subject, &step.bindings)?;
        evidence(writer, plan, step_subject, &step.evidence)?;
    }
    Ok(())
}

fn term(projection: &VizProjection, iri: &str) -> Result<VizTermId, DiagramError> {
    projection
        .terms
        .iter()
        .find(|term| matches!(&term.value, VizTermValue::Iri { value } if value == iri))
        .map(|term| term.id.clone())
        .ok_or_else(|| {
            DiagramError::refused(format!(
                "model identity `{iri}` is absent from the native projection"
            ))
        })
}

fn scenario(
    model: AuthoredModel<'_>,
    projection: &VizProjection,
    row: &Scenario,
) -> Result<VizScenario, DiagramError> {
    let participants = row
        .participants
        .iter()
        .map(|id| {
            let iri = participant(model, id)?;
            let (label, role) = model
                .declarations
                .components
                .iter()
                .find(|component| component.id == *id)
                .map(|component| (component.label.as_str(), "component"))
                .or_else(|| {
                    model
                        .declarations
                        .actors
                        .iter()
                        .find(|actor| actor.id == *id)
                        .map(|actor| (actor.label.as_str(), "external actor"))
                })
                .ok_or_else(|| DiagramError::refused(format!("participant `{id}` is absent")))?;
            Ok(VizParticipant {
                term: term(projection, &iri)?,
                label: VizSceneLabel::plain(label),
                role: Some(role.into()),
            })
        })
        .collect::<Result<_, DiagramError>>()?;
    let regions = row
        .regions
        .iter()
        .map(|region| VizScenarioRegion {
            id: region.id.clone(),
            parent: region.parent.clone(),
            order: region.order,
            kind: match region.kind {
                ScenarioRegionKind::Sequence => VizRegionKind::Sequence,
                ScenarioRegionKind::Alternative => VizRegionKind::Alternative,
                ScenarioRegionKind::Branch => VizRegionKind::Branch,
                ScenarioRegionKind::Error => VizRegionKind::Error,
                ScenarioRegionKind::Parallel => VizRegionKind::Parallel,
                ScenarioRegionKind::Lane => VizRegionKind::Lane,
            },
            label: VizSceneLabel::plain(&region.label),
        })
        .collect();
    let interactions = row
        .steps
        .iter()
        .map(|step| {
            let relation = model
                .declarations
                .relations
                .iter()
                .find(|relation| relation.id == step.relation)
                .ok_or_else(|| {
                    DiagramError::refused(format!(
                        "step relationship `{}` is absent",
                        step.relation
                    ))
                })?;
            let (binding, from, to) = relation_binding(model, projection, relation)?;
            Ok(VizInteraction {
                id: step.id.clone(),
                binding,
                from,
                to,
                region: step.region.clone(),
                order: step.order,
                label: Some(VizSceneLabel::plain(&format!("DECLARED: {}", step.label))),
            })
        })
        .collect::<Result<_, DiagramError>>()?;
    Ok(VizScenario {
        participants,
        regions,
        interactions,
    })
}

fn relation_binding(
    model: AuthoredModel<'_>,
    projection: &VizProjection,
    relation: &raf_okf::DeclaredRelation,
) -> Result<(VizSemanticRef, VizTermId, VizTermId), DiagramError> {
    let predicate = term(
        projection,
        &identity(model.owner, "relation", &relation.id)?,
    )?;
    let from = term(projection, &participant(model, &relation.from)?)?;
    let to = term(projection, &participant(model, &relation.to)?)?;
    let statement = projection
        .statements
        .iter()
        .find(|statement| {
            statement.predicate == predicate
                && statement.subject == VizValueRef::Term { id: from.clone() }
                && statement.object == VizValueRef::Term { id: to.clone() }
        })
        .ok_or_else(|| {
            DiagramError::refused("declared interaction has no native structural statement")
        })?;
    Ok((VizSemanticRef::Statement(statement.id.clone()), from, to))
}

pub(super) fn scene(
    plan: &Workplan<'_>,
    model: AuthoredModel<'_>,
    selected: Option<&str>,
    spec: &VizSpec,
) -> Result<ModelScene, DiagramError> {
    let dataset = dataset(plan, model, spec.max_statements)?;
    let projection = viz::project_dataset(&dataset, spec)?;
    let mut presentation = VizPresentation::default();
    for row in &model.declarations.components {
        let binding = VizSemanticRef::Term(term(
            &projection,
            &identity(model.owner, "component", &row.id)?,
        )?);
        presentation.components.push(VizComponent {
            id: row.id.clone(),
            binding: binding.clone(),
            parent: row.parent.clone(),
            members: Vec::new(),
            label: VizSceneLabel::plain(&row.label),
        });
        presentation.displays.push(VizDisplay {
            binding,
            label: Some(VizSceneLabel::plain(&row.label)),
            description: Some(format!("DECLARED responsibility: {}", row.responsibility)),
            role: Some(
                match row.kind {
                    ComponentKind::Component => "declared component",
                    ComponentKind::Boundary => "declared boundary",
                }
                .into(),
            ),
            navigation: None,
        });
    }
    for (family, id, label, description) in model
        .declarations
        .actors
        .iter()
        .map(|row| {
            (
                "actor",
                row.id.as_str(),
                row.label.as_str(),
                row.responsibility.clone(),
            )
        })
        .chain(model.declarations.artifacts.iter().map(|row| {
            (
                "artifact",
                row.id.as_str(),
                row.label.as_str(),
                format!("declared type: {}", row.artifact_type),
            )
        }))
        .chain(model.declarations.relations.iter().map(|row| {
            (
                "relation",
                row.id.as_str(),
                row.label.as_str(),
                row.protocol.as_ref().map_or_else(
                    || "declared interaction".into(),
                    |protocol| format!("declared protocol: {protocol}"),
                ),
            )
        }))
    {
        presentation.displays.push(VizDisplay {
            binding: VizSemanticRef::Term(term(&projection, &identity(model.owner, family, id)?)?),
            label: Some(VizSceneLabel::plain(label)),
            description: Some(description),
            role: Some(format!("declared {family}")),
            navigation: None,
        });
    }
    source_displays(plan, model, &projection, &mut presentation)?;
    if let Some(id) = selected {
        let row = model
            .declarations
            .scenarios
            .iter()
            .find(|row| row.id == id)
            .ok_or_else(|| {
                DiagramError::refused(format!("scenario `{id}` is absent from {}", model.owner))
            })?;
        presentation.scenario = Some(scenario(model, &projection, row)?);
    }
    Ok(ModelScene {
        dataset,
        projection,
        presentation,
    })
}

fn source_displays(
    plan: &Workplan<'_>,
    model: AuthoredModel<'_>,
    projection: &VizProjection,
    presentation: &mut VizPresentation,
) -> Result<(), DiagramError> {
    let mut paths = BTreeSet::new();
    for bindings in model.declarations.bindings() {
        paths.extend(binding_paths(plan, bindings)?);
    }
    for path in paths {
        let resolved = plan.source().resolve(path).map_err(DiagramError::data)?;
        let [id] = resolved.sites.as_slice() else {
            return Err(DiagramError::refused(format!(
                "source navigation for `{path}` needs one definition"
            )));
        };
        let item = plan
            .source()
            .item(*id)
            .ok_or_else(|| DiagramError::refused("source navigation has no indexed definition"))?;
        presentation.displays.push(VizDisplay {
            binding: VizSemanticRef::Term(term(projection, &source_identity(path))?),
            label: Some(VizSceneLabel::plain(path)),
            description: Some(format!(
                "resolved identity at {}:{}; no execution claim",
                item.file, item.name_line
            )),
            role: Some("source binding".into()),
            navigation: Some(VizNavigation::parse(&format!(
                "../../../../{}#L{}",
                purrdf_lex::percent::encode(
                    &item.file.to_string(),
                    purrdf_lex::percent::UNRESERVED_SLASH
                ),
                item.name_line
            ))?),
        });
    }
    Ok(())
}

pub(crate) fn validate(
    plan: &Workplan<'_>,
    model: AuthoredModel<'_>,
) -> Result<(), (String, DiagramError)> {
    if model.declarations == &raf_okf::ArchitectureModel::default() {
        return Ok(());
    }
    let spec = VizSpec {
        max_statements: 10000,
        max_terms: 30000,
        ..VizSpec::default()
    };
    let overview = scene(plan, model, None, &spec).map_err(|error| ("model".into(), error))?;
    viz::build_presented_scene(&overview.projection, &spec, &overview.presentation)
        .map_err(|error| (location(model, None, &error), error.into()))?;
    for (index, row) in model.declarations.scenarios.iter().enumerate() {
        let mut presentation = overview.presentation.clone();
        presentation.scenario = Some(
            scenario(model, &overview.projection, row)
                .map_err(|error| (format!("model.scenarios[{index}]"), error))?,
        );
        viz::build_presented_scene(&overview.projection, &spec, &presentation)
            .map_err(|error| (location(model, Some(index), &error), error.into()))?;
    }
    Ok(())
}

fn location(model: AuthoredModel<'_>, scenario: Option<usize>, error: &viz::VizError) -> String {
    if let viz::VizError::Presentation(error) = error {
        if let Some(index) = scenario {
            let row = &model.declarations.scenarios[index];
            if let Some(position) = row
                .regions
                .iter()
                .position(|row| row.id == error.instruction)
            {
                return format!("model.scenarios[{index}].regions[{position}]");
            }
            if let Some(position) = row.steps.iter().position(|row| row.id == error.instruction) {
                return format!("model.scenarios[{index}].steps[{position}]");
            }
        } else if let Some(position) = model
            .declarations
            .components
            .iter()
            .position(|row| row.id == error.instruction)
        {
            return format!("model.components[{position}]");
        }
    }
    scenario.map_or_else(
        || "model".into(),
        |index| format!("model.scenarios[{index}]"),
    )
}

pub(super) fn bundle(
    plan: &Workplan<'_>,
    view: &super::NamedDiagramView,
    scope: &str,
) -> Result<super::DiagramBundle, DiagramError> {
    let (owner, scenario_id) = if view.kind() == super::DiagramKind::Scenario {
        let (owner, scenario) = scope.split_once('#').ok_or_else(|| {
            DiagramError::refused(
                "scenario scope must be a document id followed by # and its local scenario id",
            )
        })?;
        (owner, Some(scenario))
    } else {
        (scope, None)
    };
    let model = plan
        .architecture_models()
        .find(|model| model.owner == owner)
        .ok_or_else(|| DiagramError::refused(format!("model owner `{owner}` did not load")))?;
    if let Some(finding) = plan.findings().iter().find(|finding| {
        finding.file == model.path && finding.kind == crate::finding::kind::MODEL_INVALID
    }) {
        return Err(DiagramError::refused(format!(
            "{}: {}; {}",
            finding.file, finding.message, finding.fix
        )));
    }
    if model.declarations == &raf_okf::ArchitectureModel::default() {
        return Err(DiagramError::refused(format!(
            "document `{owner}` has no model declarations"
        )));
    }
    if !view.focus.is_empty()
        || !view.relations.is_empty()
        || view.grouping != super::DiagramGrouping::None
    {
        return Err(DiagramError::refused(
            "document model views retain their complete declared containment and provenance; select a scenario detail through document#scenario with grouping none and empty focus/relations",
        ));
    }
    build_bundle(plan, model, view, scenario_id, scope)
}

fn build_bundle(
    plan: &Workplan<'_>,
    model: AuthoredModel<'_>,
    view: &super::NamedDiagramView,
    selected: Option<&str>,
    scope: &str,
) -> Result<super::DiagramBundle, DiagramError> {
    use domain::Layer;
    use purrdf_lex::json::{self, record::ToJson as _};
    let full = scene(plan, model, None, &view.spec)?;
    let full_hash = viz::stable_hash_hex(&json::write_compact(&full.projection.to_json()));
    // Audit detail retains every RDF fact and native source display. Compound
    // architectural frames belong to the domain views, not the RDF audit canvas.
    let audit = VizPresentation {
        displays: full.presentation.displays.clone(),
        components: Vec::new(),
        scenario: None,
    };
    if view.kind() == super::DiagramKind::ModelProvenance {
        return super::DiagramBundle::finish(
            view,
            full.dataset,
            &full.projection,
            Some(&audit),
            Vec::new(),
            scope,
            Some(&full_hash),
        );
    }
    let layer = if view.kind() == super::DiagramKind::Artifacts {
        if model.declarations.artifacts.is_empty() {
            return Err(DiagramError::refused(
                "artifact view requires declared typed producer/consumer flows",
            ));
        }
        Layer::Artifacts
    } else {
        Layer::Components
    };
    let provenance = provenance_detail(view, model, &full, &audit, &full_hash)?;
    let mut primary = view.clone();
    primary.spec = domain::spec(model, layer, &view.spec)?;
    let projection = viz::project_dataset(&full.dataset, &primary.spec)?;
    let mut presentation = domain::presentation(
        plan,
        model,
        &projection,
        &primary.spec,
        &full.presentation,
        layer,
        &full_hash,
    )?;
    let mut details = vec![provenance];
    if let Some(id) = selected {
        let declaration = model
            .declarations
            .scenarios
            .iter()
            .find(|row| row.id == id)
            .ok_or_else(|| {
                DiagramError::refused(format!("scenario `{id}` is absent from {}", model.owner))
            })?;
        presentation.scenario = Some(scenario(model, &projection, declaration)?);
    } else if layer == Layer::Components {
        for declaration in &model.declarations.scenarios {
            let detail_scope = format!("{}#{}", model.owner, declaration.id);
            let mut detail = super::NamedDiagramView::new(
                &format!("{}-scenario-{}", view.name, declaration.id),
                super::DiagramScope::Automatic {
                    kind: super::DiagramKind::Scenario,
                    scope: detail_scope.clone(),
                },
            )?;
            detail.spec = primary.spec.clone();
            detail.owner.clone_from(&view.owner);
            let mut selected = presentation.clone();
            selected.scenario = Some(scenario(model, &projection, declaration)?);
            details.push(super::DiagramBundle::finish(
                &detail,
                Arc::clone(&full.dataset),
                &projection,
                Some(&selected),
                Vec::new(),
                &detail_scope,
                Some(&full_hash),
            )?);
        }
    }
    super::DiagramBundle::finish(
        &primary,
        full.dataset,
        &projection,
        Some(&presentation),
        details,
        scope,
        Some(&full_hash),
    )
}

fn provenance_detail(
    view: &super::NamedDiagramView,
    model: AuthoredModel<'_>,
    full: &ModelScene,
    audit: &VizPresentation,
    hash: &str,
) -> Result<super::DiagramBundle, DiagramError> {
    let mut named = super::NamedDiagramView::new(
        &format!("{}-provenance", view.name),
        super::DiagramScope::Automatic {
            kind: super::DiagramKind::ModelProvenance,
            scope: model.owner.into(),
        },
    )?;
    named.spec = domain::spec(model, domain::Layer::Provenance, &view.spec)?;
    named.owner.clone_from(&view.owner);
    super::DiagramBundle::finish(
        &named,
        Arc::clone(&full.dataset),
        &full.projection,
        Some(audit),
        Vec::new(),
        model.owner,
        Some(hash),
    )
}

fn binding_paths<'a>(
    plan: &'a Workplan<'_>,
    bindings: &'a ModelBindings,
) -> Result<BTreeSet<&'a String>, DiagramError> {
    let mut paths: BTreeSet<_> = bindings
        .code
        .iter()
        .chain(&bindings.interfaces)
        .chain(&bindings.tests)
        .chain(&bindings.examples)
        .collect();
    for id in &bindings.capabilities {
        let target = plan
            .capabilities()
            .iter()
            .find(|target| target.value().header.id == *id)
            .ok_or_else(|| DiagramError::refused(format!("capability `{id}` did not load")))?;
        paths.extend(
            target
                .value()
                .bindings
                .code
                .iter()
                .chain(&target.value().bindings.tests)
                .chain(&target.value().bindings.examples),
        );
    }
    Ok(paths)
}

fn binding_navigation(
    plan: &Workplan<'_>,
    bindings: &ModelBindings,
    full: &VizPresentation,
) -> Result<Option<VizNavigation>, DiagramError> {
    if let Some(path) = binding_paths(plan, bindings)?.first() {
        let display = full
            .displays
            .iter()
            .find(|display| {
                display
                    .label
                    .as_ref()
                    .is_some_and(|label| label.full_text == path.as_str())
            })
            .ok_or_else(|| {
                DiagramError::refused(format!(
                    "source navigation for `{path}` is absent from full provenance"
                ))
            })?;
        return Ok(display.navigation.clone());
    }
    if let Some(id) = bindings.capabilities.iter().min() {
        let target = plan
            .capabilities()
            .iter()
            .find(|target| target.value().header.id == *id)
            .ok_or_else(|| DiagramError::refused(format!("capability `{id}` did not load")))?;
        return Ok(Some(VizNavigation::parse(&format!(
            "../../../../{}",
            purrdf_lex::percent::encode(target.path(), purrdf_lex::percent::UNRESERVED_SLASH)
        ))?));
    }
    Ok(None)
}
