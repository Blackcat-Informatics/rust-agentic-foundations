// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native named-graph selections over the one authoritative declaration dataset.

use super::{AuthoredModel, DiagramError, VOCAB, Workplan, Writer, identity, participant};
use purrdf_rdf::viz::{
    self, VizDisplay, VizGraphPolicy, VizPresentation, VizProjection, VizSceneLabel,
    VizSemanticRef, VizSpec, VizTermValue,
};
use raf_okf::ComponentKind;
use std::collections::BTreeSet;
use std::fmt::Write as _;

const TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Layer {
    Components,
    Artifacts,
    Provenance,
}

fn graph(model: AuthoredModel<'_>, layer: Layer) -> Result<String, DiagramError> {
    identity(
        model.owner,
        "graph",
        match layer {
            Layer::Components => "declared-components",
            Layer::Artifacts => "declared-artifacts",
            Layer::Provenance => {
                return Err(DiagramError::refused(
                    "complete provenance selects every native graph",
                ));
            }
        },
    )
}

pub(super) fn spec(
    model: AuthoredModel<'_>,
    layer: Layer,
    original: &VizSpec,
) -> Result<VizSpec, DiagramError> {
    let mut spec = original.clone();
    spec.graph_policy = if layer == Layer::Provenance {
        VizGraphPolicy::All
    } else {
        VizGraphPolicy::Include(vec![graph(model, layer)?])
    };
    Ok(spec)
}

pub(super) fn write(writer: &mut Writer, model: AuthoredModel<'_>) -> Result<(), DiagramError> {
    let components = writer.iri(&graph(model, Layer::Components)?)?;
    let artifacts = writer.iri(&graph(model, Layer::Artifacts)?)?;
    let mut connected = BTreeSet::new();
    for row in &model.declarations.components {
        if let Some(parent) = &row.parent {
            let child = writer.iri(&participant(model, &row.id)?)?;
            let parent_node = writer.iri(&participant(model, parent)?)?;
            for graph in [components, artifacts] {
                writer.put_in_graph(
                    child,
                    &format!("{VOCAB}containedBy"),
                    parent_node,
                    Some(graph),
                )?;
            }
            connected.insert(row.id.as_str());
            connected.insert(parent.as_str());
        }
    }
    for row in &model.declarations.relations {
        let from = writer.iri(&participant(model, &row.from)?)?;
        let to = writer.iri(&participant(model, &row.to)?)?;
        writer.put_in_graph(
            from,
            &identity(model.owner, "relation", &row.id)?,
            to,
            Some(components),
        )?;
        connected.insert(row.from.as_str());
        connected.insert(row.to.as_str());
    }
    for row in &model.declarations.artifacts {
        let artifact = writer.iri(&identity(model.owner, "artifact", &row.id)?)?;
        for producer in &row.producers {
            let producer = writer.iri(&participant(model, producer)?)?;
            writer.put_in_graph(
                producer,
                &format!("{VOCAB}produces"),
                artifact,
                Some(artifacts),
            )?;
        }
        for consumer in &row.consumers {
            let consumer = writer.iri(&participant(model, consumer)?)?;
            writer.put_in_graph(
                artifact,
                &format!("{VOCAB}consumedBy"),
                consumer,
                Some(artifacts),
            )?;
        }
    }
    // A disconnected declared participant still needs a genuine RDF statement
    // to enter native projection; type is its authored fact, never an inferred edge.
    for (id, kind) in model
        .declarations
        .components
        .iter()
        .map(|row| {
            (
                row.id.as_str(),
                match row.kind {
                    ComponentKind::Component => "Component",
                    ComponentKind::Boundary => "Boundary",
                },
            )
        })
        .chain(
            model
                .declarations
                .actors
                .iter()
                .map(|row| (row.id.as_str(), "ExternalActor")),
        )
    {
        if !connected.contains(id) {
            let subject = writer.iri(&participant(model, id)?)?;
            let class = writer.iri(&format!("{VOCAB}{kind}"))?;
            writer.put_in_graph(subject, TYPE, class, Some(components))?;
        }
    }
    Ok(())
}

pub(super) fn presentation(
    plan: &Workplan<'_>,
    model: AuthoredModel<'_>,
    projection: &VizProjection,
    spec: &VizSpec,
    full: &VizPresentation,
    layer: Layer,
    full_model_hash: &str,
) -> Result<VizPresentation, DiagramError> {
    let scene = viz::build_scene(projection, spec)?;
    let nodes: BTreeSet<_> = scene
        .nodes
        .iter()
        .flat_map(|row| row.bindings.iter().cloned())
        .collect();
    let visible: BTreeSet<_> = nodes
        .iter()
        .cloned()
        .chain(
            scene
                .edges
                .iter()
                .flat_map(|row| row.bindings.iter().cloned()),
        )
        .chain(
            scene
                .edges
                .iter()
                .filter_map(|row| row.anchor.as_ref())
                .flat_map(|row| row.bindings.iter().cloned()),
        )
        .collect();
    let mut presentation = VizPresentation {
        displays: full
            .displays
            .iter()
            .filter(|row| visible.contains(&row.binding))
            .cloned()
            .collect(),
        components: full
            .components
            .iter()
            .filter(|row| nodes.contains(&row.binding))
            .cloned()
            .collect(),
        scenario: None,
    };
    for row in &model.declarations.components {
        let iri = identity(model.owner, "component", &row.id)?;
        if let Some(display) = display(&mut presentation, projection, &iri) {
            display.navigation = super::binding_navigation(plan, &row.bindings, full)?;
        }
    }
    for row in &model.declarations.artifacts {
        let iri = identity(model.owner, "artifact", &row.id)?;
        if let Some(display) = display(&mut presentation, projection, &iri) {
            display.role = Some(format!("declared artifact; {}", row.artifact_type));
            display.navigation = super::binding_navigation(plan, &row.bindings, full)?;
        }
    }
    if layer == Layer::Components {
        for row in &model.declarations.relations {
            let (binding, _, _) = super::relation_binding(model, projection, row)?;
            let mut role = "DECLARED".to_owned();
            if let Some(protocol) = &row.protocol {
                write!(role, "; {protocol}").expect("writing to String cannot fail");
            }
            if let Some(id) = &row.artifact {
                let artifact = model
                    .declarations
                    .artifacts
                    .iter()
                    .find(|artifact| artifact.id == *id)
                    .ok_or_else(|| DiagramError::refused(format!("artifact `{id}` is absent")))?;
                write!(role, "; {}", artifact.artifact_type)
                    .expect("writing to String cannot fail");
            }
            presentation.displays.push(VizDisplay {
                binding,
                label: Some(VizSceneLabel::plain(&row.label)),
                description: Some("Authored relationship; no execution witness".into()),
                role: Some(role),
                navigation: None,
            });
        }
    }
    if let Some(display) = presentation.displays.first_mut() {
        let description = display.description.get_or_insert_with(String::new);
        write!(
            description,
            "\nFull RDF provenance model hash: {full_model_hash}"
        )
        .expect("writing to String cannot fail");
    }
    Ok(presentation)
}

fn display<'a>(
    presentation: &'a mut VizPresentation,
    projection: &VizProjection,
    iri: &str,
) -> Option<&'a mut VizDisplay> {
    let term = projection
        .terms
        .iter()
        .find(|term| matches!(&term.value, VizTermValue::Iri { value } if value == iri))?;
    presentation
        .displays
        .iter_mut()
        .find(|row| row.binding == VizSemanticRef::Term(term.id.clone()))
}
