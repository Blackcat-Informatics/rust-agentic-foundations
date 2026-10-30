// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Source-bound authored architecture, separate from observed execution facts.

mod json;
mod read;

pub(super) use read::check;

/// Bindings identify implementations and verification; they do not prove interactions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelBindings {
    /// Implementation symbols, including actual crate and module roots.
    pub code: Vec<String>,
    /// Public exported interfaces, checked against the native source index.
    pub interfaces: Vec<String>,
    /// Capability identities whose documented contract participates.
    pub capabilities: Vec<String>,
    /// Associated test symbols; a passing test does not observe a scenario step.
    pub tests: Vec<String>,
    /// Associated example symbols.
    pub examples: Vec<String>,
}

/// A component implementation or an enclosing architectural boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentKind {
    /// A source-bound architectural component.
    Component,
    /// A responsibility boundary containing components.
    Boundary,
}

/// A named component or boundary in an authored containment forest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    /// Stable local kebab identifier, independent of source spelling.
    pub id: String,
    /// Human-facing name.
    pub label: String,
    /// Whether this row implements a component or encloses components.
    pub kind: ComponentKind,
    /// Parent component/boundary local identifier, absent at the model root.
    pub parent: Option<String>,
    /// The component's authored responsibility.
    pub responsibility: String,
    /// Explicit source, public-interface, capability and verification associations.
    pub bindings: ModelBindings,
}

/// A participant outside the declared component containment tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalActor {
    /// Stable local kebab identifier.
    pub id: String,
    /// Human-facing participant name.
    pub label: String,
    /// The actor's declared role or responsibility.
    pub responsibility: String,
}

/// A labelled declared relationship, never inferred from a package dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredRelation {
    /// Stable local kebab identifier.
    pub id: String,
    /// Sending/using component or actor local identifier.
    pub from: String,
    /// Receiving/providing component or actor local identifier.
    pub to: String,
    /// Authored architectural meaning.
    pub label: String,
    /// Optional labelled communication protocol.
    pub protocol: Option<String>,
    /// Optional explicitly transferred artifact local identifier.
    pub artifact: Option<String>,
}

/// Explicit producer/consumer flow of a named artifact type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactFlow {
    /// Stable local kebab identifier.
    pub id: String,
    /// Human-facing artifact name.
    pub label: String,
    /// Authored transferred type, independent of package dependency edges.
    pub artifact_type: String,
    /// Explicit producer component/actor identifiers.
    pub producers: Vec<String>,
    /// Explicit consumer component/actor identifiers.
    pub consumers: Vec<String>,
    /// Source and public-interface bindings for the artifact type.
    pub bindings: ModelBindings,
}

/// Authored structural grammar; parallel lanes declare no inter-lane order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioRegionKind {
    /// Ordered children.
    Sequence,
    /// Two or more guarded branch/error arms.
    Alternative,
    /// An ordinary guarded arm.
    Branch,
    /// A declared error arm.
    Error,
    /// Two or more parallel lanes.
    Parallel,
    /// Ordered interactions within one parallel lane.
    Lane,
}

/// A flat region declaration; nesting is explicit and bounded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioRegion {
    /// Stable local kebab identifier within the scenario.
    pub id: String,
    /// Parent region identifier, absent for the implicit root sequence.
    pub parent: Option<String>,
    /// Explicit zero-based position among sibling regions and steps.
    pub order: u32,
    /// The region's execution grammar, declared rather than observed.
    pub kind: ScenarioRegionKind,
    /// Branch/error condition, lane name or sequence explanation.
    pub label: String,
}

/// A declared occurrence of a model relationship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioStep {
    /// Stable local kebab identifier within the scenario.
    pub id: String,
    /// Existing declared relationship identifier.
    pub relation: String,
    /// Parent sequence, arm or lane, absent at the scenario root.
    pub region: Option<String>,
    /// Explicit zero-based position among sibling steps and regions.
    pub order: u32,
    /// Authored interaction label.
    pub label: String,
    /// Implementation, capability and verification associations.
    pub bindings: ModelBindings,
    /// Independent execution-record references, never event/order witnesses.
    pub evidence: Vec<String>,
}

/// A focused declared scenario over named participants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    /// Stable local kebab identifier.
    pub id: String,
    /// Human-facing scenario name.
    pub label: String,
    /// Participant component/actor identifiers in explicit presentation order.
    pub participants: Vec<String>,
    /// Flat structural regions.
    pub regions: Vec<ScenarioRegion>,
    /// Interaction occurrences with explicit sibling positions.
    pub steps: Vec<ScenarioStep>,
    /// Independent current execution-record references.
    pub evidence: Vec<String>,
}

/// The optional closed `model` key shared by capability and architecture documents.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArchitectureModel {
    /// Component and boundary declarations.
    pub components: Vec<Component>,
    /// External actors, outside component containment.
    pub actors: Vec<ExternalActor>,
    /// Labelled declared relationships.
    pub relations: Vec<DeclaredRelation>,
    /// Explicit typed producer/consumer flows.
    pub artifacts: Vec<ArtifactFlow>,
    /// Focused scenarios with declared structural semantics.
    pub scenarios: Vec<Scenario>,
}

impl ArchitectureModel {
    /// Explicit associations on components, artifacts and scenario steps.
    pub fn bindings(&self) -> impl Iterator<Item = &ModelBindings> {
        self.components
            .iter()
            .map(|row| &row.bindings)
            .chain(self.artifacts.iter().map(|row| &row.bindings))
            .chain(
                self.scenarios
                    .iter()
                    .flat_map(|row| row.steps.iter().map(|step| &step.bindings)),
            )
    }

    /// Independent execution records associated with scenarios and steps.
    pub fn evidence(&self) -> impl Iterator<Item = &str> {
        self.scenarios
            .iter()
            .flat_map(|row| {
                row.evidence
                    .iter()
                    .chain(row.steps.iter().flat_map(|step| step.evidence.iter()))
            })
            .map(String::as_str)
    }
}
