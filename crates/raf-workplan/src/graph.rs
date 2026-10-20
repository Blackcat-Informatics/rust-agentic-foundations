// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The semantic graph of the documents and the source they are bound to.
//!
//! The graph is built once, when a [`Workplan`](crate::Workplan) is loaded,
//! from the typed documents and the source index. Rules, projections and
//! queries read the graph; nothing re-reads a document.
//!
//! * **Nodes** ([`Node`]): capabilities, the seven sections of each
//!   capability, changes, architecture pages, the PROPOSED blocks of the
//!   capabilities, the code symbols, tests and examples the bindings
//!   resolve to, the evidence, assessment and coverage records, and the
//!   crates the coverage record scopes.
//! * **Edges** ([`Edge`]): bindings from a capability to a path of the source
//!   index, `depends` between capabilities, `affects` from a change to a
//!   section of a capability, `changes` from a change to a capability,
//!   Markdown links, `for` from an evidence record to a capability it covers,
//!   claims from a section of a capability to the evidence it rests on, the
//!   subject of an assessment, the `scope` of the coverage record, and the
//!   inputs of a record to the section, symbol or evidence record each names,
//!   when it resolves. Links
//!   and claims are section-granular: a link runs from the section that holds
//!   it to the section its anchor names, or to the target document when the
//!   link has no anchor, and a claim runs from the section that holds its
//!   marker.
//!
//! The queries are [`Graph::code_of`], [`Graph::capabilities_of`],
//! [`Graph::tests_for`], [`Graph::sections_of`],
//! [`Graph::linking_sections`], [`Graph::evidence_of`],
//! [`Graph::claims_on`], [`Graph::assessments_of`] and [`Graph::inputs_of`].
//! Each returns typed
//! values in a fixed order.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;

use raf_okf::{
    Architecture, AssessmentRecord, Body, Capability, CapabilitySection, Change, CoverageRecord,
    DocType, EvidenceRecord, Input, InputKind, LinkTarget, Location, Outcome,
};
use raf_source::{ItemId, ResolveError, Workspace};

use crate::bindings::{self, ExampleKind};
use crate::finding::kind;
use crate::work::{Counted, Work, add};
use crate::{Finding, Loaded};

/// One of the seven sections of a capability, named by the capability id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionId {
    capability: String,
    section: CapabilitySection,
}

impl fmt::Display for SectionId {
    /// `cap:<slug>#<section>`, the form of an `affects` entry.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.capability, self.section.slug())
    }
}

impl SectionId {
    /// The section `section` of the capability with the id `capability`.
    #[must_use]
    pub fn new(capability: impl Into<String>, section: CapabilitySection) -> Self {
        Self {
            capability: capability.into(),
            section,
        }
    }

    /// The id of the capability, `cap:<slug>`.
    #[must_use]
    pub fn capability(&self) -> &str {
        &self.capability
    }

    /// Which of the seven sections.
    #[must_use]
    pub fn section(&self) -> CapabilitySection {
        self.section
    }

    fn rank(&self) -> usize {
        position(self.section)
    }
}

impl PartialOrd for SectionId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SectionId {
    /// By capability id, then in the order the capability holds its sections.
    fn cmp(&self, other: &Self) -> Ordering {
        self.capability
            .cmp(&other.capability)
            .then_with(|| self.rank().cmp(&other.rank()))
    }
}

/// The place of a section in the order a capability holds them.
pub(crate) fn position(section: CapabilitySection) -> usize {
    CapabilitySection::ALL
        .iter()
        .position(|candidate| *candidate == section)
        .unwrap_or(CapabilitySection::ALL.len())
}

/// A PROPOSED block of a capability document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    capability: String,
    change: String,
    section: Option<CapabilitySection>,
    ordinal: usize,
}

impl Proposal {
    /// The id of the capability that holds the block.
    #[must_use]
    pub fn capability(&self) -> &str {
        &self.capability
    }

    /// The slug of the change that proposes the text.
    #[must_use]
    pub fn change(&self) -> &str {
        &self.change
    }

    /// The section that holds the block.
    #[must_use]
    pub fn section(&self) -> Option<CapabilitySection> {
        self.section
    }

    /// The place of the block among the blocks of the document, from zero.
    #[must_use]
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
}

/// The kinds of node of the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NodeKind {
    /// A capability document.
    Capability,
    /// One of the seven sections of a capability.
    Section,
    /// A change record.
    Change,
    /// An architecture page.
    Architecture,
    /// A PROPOSED block.
    Proposal,
    /// A source item bound as code, or named by a `symbol` input of a record.
    Symbol,
    /// A source item bound as a test.
    Test,
    /// A source item bound as an example.
    Example,
    /// An evidence record.
    Evidence,
    /// An assessment record.
    Assessment,
    /// The coverage record.
    Coverage,
    /// A crate in the `scope` of the coverage record.
    Crate,
}

impl NodeKind {
    /// Every kind of node.
    pub const ALL: [Self; 12] = [
        Self::Capability,
        Self::Section,
        Self::Change,
        Self::Architecture,
        Self::Proposal,
        Self::Symbol,
        Self::Test,
        Self::Example,
        Self::Evidence,
        Self::Assessment,
        Self::Coverage,
        Self::Crate,
    ];
}

/// A node of the graph.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Node {
    /// A capability, by id.
    Capability(String),
    /// A section of a capability.
    Section(SectionId),
    /// A change, by id.
    Change(String),
    /// An architecture page, by id.
    Architecture(String),
    /// A PROPOSED block.
    Proposal(Proposal),
    /// A source item bound as code, or named by a `symbol` input of a record.
    Symbol(ItemId),
    /// A source item bound as a test.
    Test(ItemId),
    /// A source item bound as an example.
    Example(ItemId),
    /// An evidence record, by id.
    Evidence(String),
    /// An assessment record, by id.
    Assessment(String),
    /// The coverage record, by id.
    Coverage(String),
    /// A crate of the source workspace, by package name.
    Crate(String),
}

impl Node {
    /// The kind of the node.
    #[must_use]
    pub fn kind(&self) -> NodeKind {
        match self {
            Self::Capability(_) => NodeKind::Capability,
            Self::Section(_) => NodeKind::Section,
            Self::Change(_) => NodeKind::Change,
            Self::Architecture(_) => NodeKind::Architecture,
            Self::Proposal(_) => NodeKind::Proposal,
            Self::Symbol(_) => NodeKind::Symbol,
            Self::Test(_) => NodeKind::Test,
            Self::Example(_) => NodeKind::Example,
            Self::Evidence(_) => NodeKind::Evidence,
            Self::Assessment(_) => NodeKind::Assessment,
            Self::Coverage(_) => NodeKind::Coverage,
            Self::Crate(_) => NodeKind::Crate,
        }
    }
}

/// Which list of `bindings` an entry came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum BindingKind {
    /// `bindings.code`.
    Code,
    /// `bindings.tests`.
    Test,
    /// `bindings.examples`.
    Example,
}

impl BindingKind {
    /// The list a key of `bindings` names.
    pub(crate) fn from_key(key: &str) -> Option<Self> {
        [Self::Code, Self::Test, Self::Example]
            .into_iter()
            .find(|list| list.key() == key)
    }

    /// The key of the list in `bindings`: `code`, `tests` or `examples`.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Test => "tests",
            Self::Example => "examples",
        }
    }
}

/// A binding edge: an entry of a capability's `bindings`, and what the source
/// index made of its path.
///
/// The edge records a path that does not resolve as well as one that does;
/// the binding rules that judge the result are in `check`, and read this
/// record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    capability: String,
    kind: BindingKind,
    path: String,
    resolution: Result<Vec<ItemId>, ResolveError>,
    sections: Vec<CapabilitySection>,
    examples: Vec<(ItemId, ExampleKind)>,
}

impl Binding {
    /// The id of the capability that lists the binding.
    #[must_use]
    pub fn capability(&self) -> &str {
        &self.capability
    }

    /// The list the binding is in.
    #[must_use]
    pub fn kind(&self) -> BindingKind {
        self.kind
    }

    /// The path as the document writes it.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The items the path resolves to, or why it does not. The items are the
    /// definitions, sorted by path and then by id.
    ///
    /// # Errors
    ///
    /// The [`ResolveError`] the source index gave for the path.
    pub fn resolution(&self) -> Result<&[ItemId], &ResolveError> {
        self.resolution.as_ref().map(Vec::as_slice)
    }

    /// The items the path resolves to; empty when it does not resolve.
    #[must_use]
    pub fn items(&self) -> &[ItemId] {
        self.resolution.as_deref().unwrap_or_default()
    }

    /// How a resolved item is an example, for a binding in `examples`: the
    /// crate root of an example target or a documentation test item. `None`
    /// for an item that is not an example, for an item the binding does not
    /// resolve to, and for a binding in another list.
    #[must_use]
    pub fn example_kind(&self, item: ItemId) -> Option<ExampleKind> {
        self.examples
            .iter()
            .find(|(id, _)| *id == item)
            .map(|(_, kind)| *kind)
    }

    /// The sections the binding substantiates, in document order. A test or an
    /// example substantiates every section other than Verification whose text
    /// outside code fences, PROPOSED blocks and HTML comments holds the bound
    /// path in a code span or holds a tested-by marker that names it, and
    /// Verification when there is none. A code binding substantiates no section.
    #[must_use]
    pub fn sections(&self) -> &[CapabilitySection] {
        &self.sections
    }
}

/// A `depends` edge between two capabilities that both exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependsEdge {
    from: String,
    to: String,
}

impl DependsEdge {
    /// The capability that depends.
    #[must_use]
    pub fn from(&self) -> &str {
        &self.from
    }

    /// The capability it depends on.
    #[must_use]
    pub fn to(&self) -> &str {
        &self.to
    }
}

/// An `affects` edge from a change to a section of a capability.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AffectsEdge {
    change: String,
    section: SectionId,
}

impl AffectsEdge {
    /// The id of the change.
    #[must_use]
    pub fn change(&self) -> &str {
        &self.change
    }

    /// The section it affects.
    #[must_use]
    pub fn section(&self) -> &SectionId {
        &self.section
    }
}

/// A `changes` edge from a change to a capability it delivers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangesEdge {
    change: String,
    capability: String,
}

impl ChangesEdge {
    /// The id of the change.
    #[must_use]
    pub fn change(&self) -> &str {
        &self.change
    }

    /// The id of the capability.
    #[must_use]
    pub fn capability(&self) -> &str {
        &self.capability
    }
}

/// One end of a link edge.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LinkEnd {
    /// A section of a capability.
    Section(SectionId),
    /// A capability as a whole, by id.
    Capability(String),
    /// A change as a whole, by id.
    Change(String),
    /// An architecture page, by id.
    Architecture(String),
}

impl LinkEnd {
    fn sort_key(&self) -> (u8, &str, usize) {
        match self {
            Self::Section(id) => (0, id.capability(), id.rank()),
            Self::Capability(id) => (1, id, 0),
            Self::Change(id) => (2, id, 0),
            Self::Architecture(id) => (3, id, 0),
        }
    }
}

/// A Markdown link between two documents of the model.
///
/// The source is the section that holds the link when the document is a
/// capability and the link is under one of its sections, and the document
/// otherwise. The target is the section the anchor names, or the document
/// when the link has no anchor. A link to an external site, to a file that is
/// not a loaded document, or to an anchor no heading has is not an edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkEdge {
    from: LinkEnd,
    to: LinkEnd,
    line: usize,
}

impl LinkEdge {
    /// Where the link is written.
    #[must_use]
    pub fn from(&self) -> &LinkEnd {
        &self.from
    }

    /// Where the link leads.
    #[must_use]
    pub fn to(&self) -> &LinkEnd {
        &self.to
    }

    /// The 1-based line of the link in its document.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }
}

/// A `for` edge from an evidence record to a capability it covers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoversEdge {
    evidence: String,
    capability: String,
}

impl CoversEdge {
    /// The id of the evidence record.
    #[must_use]
    pub fn evidence(&self) -> &str {
        &self.evidence
    }

    /// The id of the capability.
    #[must_use]
    pub fn capability(&self) -> &str {
        &self.capability
    }
}

/// A claim: a statement of a capability that rests on an evidence record,
/// named by a claim marker. One edge for each record the marker names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimEdge {
    capability: String,
    section: Option<CapabilitySection>,
    evidence: String,
    line: usize,
}

impl ClaimEdge {
    /// The id of the capability whose document holds the marker.
    #[must_use]
    pub fn capability(&self) -> &str {
        &self.capability
    }

    /// The section that holds the marker, or `None` for a marker before the
    /// first section.
    #[must_use]
    pub fn section(&self) -> Option<SectionId> {
        self.section
            .map(|section| SectionId::new(self.capability.clone(), section))
    }

    /// The id of the evidence record, `evidence:<slug>`.
    #[must_use]
    pub fn evidence(&self) -> &str {
        &self.evidence
    }

    /// The 1-based line of the marker in the capability document.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }
}

/// A tested-by edge: a statement of a capability mapped by a tested-by
/// marker to a test or example the capability binds. One edge for each path
/// of the marker that is a path of the capability's `bindings.tests` or
/// `bindings.examples`; a path that is neither is the finding
/// `claim-test-unbound` and makes no edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestedByEdge {
    capability: String,
    section: Option<CapabilitySection>,
    path: String,
    line: usize,
}

impl TestedByEdge {
    /// The id of the capability whose document holds the marker.
    #[must_use]
    pub fn capability(&self) -> &str {
        &self.capability
    }

    /// The section that holds the marker, or `None` for a marker outside the
    /// seven sections.
    #[must_use]
    pub fn section(&self) -> Option<SectionId> {
        self.section
            .map(|section| SectionId::new(self.capability.clone(), section))
    }

    /// The path of the binding, as the marker and the binding write it.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The 1-based line of the marker in the capability document.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }
}

/// An assessment and the section it assesses, its subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssessesEdge {
    assessment: String,
    section: SectionId,
}

impl AssessesEdge {
    /// The id of the assessment record.
    #[must_use]
    pub fn assessment(&self) -> &str {
        &self.assessment
    }

    /// The section it assesses.
    #[must_use]
    pub fn section(&self) -> &SectionId {
        &self.section
    }
}

/// A `scope` edge from the coverage record to a crate whose coverage it
/// enforces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeEdge {
    coverage: String,
    package: String,
}

impl ScopeEdge {
    /// The id of the coverage record.
    #[must_use]
    pub fn coverage(&self) -> &str {
        &self.coverage
    }

    /// The package name of the crate.
    #[must_use]
    pub fn package(&self) -> &str {
        &self.package
    }
}

/// What an input of a record leads to.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum InputTarget {
    /// A section of a capability that exists.
    Section(SectionId),
    /// The definitions a symbol path resolves to, sorted by path and then by
    /// id.
    Symbol(Vec<ItemId>),
    /// An evidence record that loaded, by id.
    Record(String),
}

/// An input of a record: the row the record was made against and what its
/// name resolves to now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputEdge {
    record: String,
    kind: InputKind,
    name: String,
    to: InputTarget,
}

impl InputEdge {
    /// The id of the record that holds the input.
    #[must_use]
    pub fn record(&self) -> &str {
        &self.record
    }

    /// The kind of the input row.
    #[must_use]
    pub fn kind(&self) -> InputKind {
        self.kind
    }

    /// The name the input row writes.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What the name resolves to.
    #[must_use]
    pub fn target(&self) -> &InputTarget {
        &self.to
    }
}

/// The kinds of edge of the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EdgeKind {
    /// A capability bound to a source path.
    Binding,
    /// A capability depends on a capability.
    Depends,
    /// A change affects a section of a capability.
    Affects,
    /// A change delivers a capability.
    Changes,
    /// A Markdown link.
    Link,
    /// An evidence record covers a capability.
    Covers,
    /// A statement of a capability rests on an evidence record.
    Claim,
    /// A statement of a capability is shown by a test or example it binds.
    TestedBy,
    /// An assessment assesses a section of a capability.
    Assesses,
    /// The coverage record enforces the coverage of a crate.
    Scope,
    /// A record was made against an input: a section, a symbol or an
    /// evidence record.
    Input,
}

impl EdgeKind {
    /// Every kind of edge.
    pub const ALL: [Self; 11] = [
        Self::Binding,
        Self::Depends,
        Self::Affects,
        Self::Changes,
        Self::Link,
        Self::Covers,
        Self::Claim,
        Self::TestedBy,
        Self::Assesses,
        Self::Scope,
        Self::Input,
    ];
}

/// An edge of the graph.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Edge {
    /// A binding.
    Binding(Binding),
    /// A `depends` edge.
    Depends(DependsEdge),
    /// An `affects` edge.
    Affects(AffectsEdge),
    /// A `changes` edge.
    Changes(ChangesEdge),
    /// A link edge.
    Link(LinkEdge),
    /// A `for` edge of an evidence record.
    Covers(CoversEdge),
    /// A claim.
    Claim(ClaimEdge),
    /// A tested-by edge.
    TestedBy(TestedByEdge),
    /// The subject of an assessment.
    Assesses(AssessesEdge),
    /// A `scope` edge of the coverage record.
    Scope(ScopeEdge),
    /// An input of a record.
    Input(InputEdge),
}

impl Edge {
    /// The kind of the edge.
    #[must_use]
    pub fn kind(&self) -> EdgeKind {
        match self {
            Self::Binding(_) => EdgeKind::Binding,
            Self::Depends(_) => EdgeKind::Depends,
            Self::Affects(_) => EdgeKind::Affects,
            Self::Changes(_) => EdgeKind::Changes,
            Self::Link(_) => EdgeKind::Link,
            Self::Covers(_) => EdgeKind::Covers,
            Self::Claim(_) => EdgeKind::Claim,
            Self::TestedBy(_) => EdgeKind::TestedBy,
            Self::Assesses(_) => EdgeKind::Assesses,
            Self::Scope(_) => EdgeKind::Scope,
            Self::Input(_) => EdgeKind::Input,
        }
    }
}

/// The graph of a loaded repository. Its nodes and edges are listed by [`Graph::nodes`] and [`Graph::edges`].
#[derive(Debug, Default)]
pub struct Graph {
    capabilities: Vec<String>,
    changes: Vec<String>,
    architecture: Vec<String>,
    proposals: Vec<Proposal>,
    bindings: Vec<Binding>,
    depends: Vec<DependsEdge>,
    affects: Vec<AffectsEdge>,
    changes_edges: Vec<ChangesEdge>,
    links: Vec<LinkEdge>,
    evidence: Vec<String>,
    assessments: Vec<String>,
    coverage: Vec<String>,
    crates: Vec<String>,
    covers: Vec<CoversEdge>,
    claims: Vec<ClaimEdge>,
    tested_by: Vec<TestedByEdge>,
    assesses: Vec<AssessesEdge>,
    scope: Vec<ScopeEdge>,
    inputs: Vec<InputEdge>,
    /// The source path of every item a binding resolved to.
    item_paths: BTreeMap<ItemId, String>,
}

impl Graph {
    /// Every node: the capabilities, their sections, the changes, the
    /// architecture pages, the proposals, the symbols, tests and examples
    /// bound by a resolving binding, the evidence, assessment and coverage
    /// records, and the crates of the coverage scope, each group sorted.
    #[must_use]
    pub fn nodes(&self) -> Vec<Node> {
        let mut nodes: Vec<Node> = self
            .capabilities
            .iter()
            .cloned()
            .map(Node::Capability)
            .collect();
        for id in &self.capabilities {
            nodes.extend(
                CapabilitySection::ALL
                    .into_iter()
                    .map(|section| Node::Section(SectionId::new(id.clone(), section))),
            );
        }
        nodes.extend(self.changes.iter().cloned().map(Node::Change));
        nodes.extend(self.architecture.iter().cloned().map(Node::Architecture));
        nodes.extend(self.proposals.iter().cloned().map(Node::Proposal));
        let input_symbols = self.inputs.iter().flat_map(|edge| match &edge.to {
            InputTarget::Symbol(items) => items.as_slice(),
            InputTarget::Section(_) | InputTarget::Record(_) => &[],
        });
        let symbols = self.sorted_items(
            self.bound_items(BindingKind::Code)
                .into_iter()
                .chain(input_symbols.copied()),
        );
        nodes.extend(symbols.into_iter().map(Node::Symbol));
        for (kind, make) in [
            (BindingKind::Test, Node::Test as fn(ItemId) -> Node),
            (BindingKind::Example, Node::Example),
        ] {
            nodes.extend(self.bound_items(kind).into_iter().map(make));
        }
        nodes.extend(self.evidence.iter().cloned().map(Node::Evidence));
        nodes.extend(self.assessments.iter().cloned().map(Node::Assessment));
        nodes.extend(self.coverage.iter().cloned().map(Node::Coverage));
        nodes.extend(self.crates.iter().cloned().map(Node::Crate));
        nodes
    }

    /// Every edge: the bindings, then `depends`, `affects`, `changes`, links,
    /// `for`, claims, tested-by edges, assessment subjects, `scope` and the
    /// inputs, each group sorted.
    #[must_use]
    pub fn edges(&self) -> Vec<Edge> {
        let mut edges: Vec<Edge> = self.bindings.iter().cloned().map(Edge::Binding).collect();
        edges.extend(self.depends.iter().cloned().map(Edge::Depends));
        edges.extend(self.affects.iter().cloned().map(Edge::Affects));
        edges.extend(self.changes_edges.iter().cloned().map(Edge::Changes));
        edges.extend(self.links.iter().cloned().map(Edge::Link));
        edges.extend(self.covers.iter().cloned().map(Edge::Covers));
        edges.extend(self.claims.iter().cloned().map(Edge::Claim));
        edges.extend(self.tested_by.iter().cloned().map(Edge::TestedBy));
        edges.extend(self.assesses.iter().cloned().map(Edge::Assesses));
        edges.extend(self.scope.iter().cloned().map(Edge::Scope));
        edges.extend(self.inputs.iter().cloned().map(Edge::Input));
        edges
    }

    /// The evidence record ids, sorted.
    #[must_use]
    pub fn evidence(&self) -> &[String] {
        &self.evidence
    }

    /// The assessment record ids, sorted.
    #[must_use]
    pub fn assessments(&self) -> &[String] {
        &self.assessments
    }

    /// The id of the coverage record, when it loaded.
    #[must_use]
    pub fn coverage(&self) -> Option<&str> {
        self.coverage.first().map(String::as_str)
    }

    /// The `for` edges to capabilities that exist, by evidence record and
    /// then by capability.
    #[must_use]
    pub fn covers(&self) -> &[CoversEdge] {
        &self.covers
    }

    /// The claims that name an evidence record that loaded, by capability,
    /// line and record.
    #[must_use]
    pub fn claims(&self) -> &[ClaimEdge] {
        &self.claims
    }

    /// The tested-by edges, by capability, line and path.
    #[must_use]
    pub fn tested_by(&self) -> &[TestedByEdge] {
        &self.tested_by
    }

    /// The subjects of the assessments, to sections of capabilities that
    /// exist, by assessment.
    #[must_use]
    pub fn assesses(&self) -> &[AssessesEdge] {
        &self.assesses
    }

    /// The `scope` edges of the coverage record to crates of the source
    /// workspace, sorted by package name.
    #[must_use]
    pub fn scope(&self) -> &[ScopeEdge] {
        &self.scope
    }

    /// The input edges of the records: each input row whose section,
    /// symbol or evidence record resolves, by record, kind and name. An input
    /// that names nothing the repository holds now makes no edge and no
    /// finding: an input that moved is a matter of the record's currency, not
    /// of the load.
    #[must_use]
    pub fn inputs(&self) -> &[InputEdge] {
        &self.inputs
    }

    /// The input edges of the record `record`, by kind and name.
    #[must_use]
    pub fn inputs_of(&self, record: &str) -> Vec<&InputEdge> {
        self.inputs
            .iter()
            .filter(|edge| edge.record == record)
            .collect()
    }

    /// The evidence records whose `for` lists the capability, sorted by id.
    #[must_use]
    pub fn evidence_of(&self, capability: &str) -> Vec<&str> {
        self.covers
            .iter()
            .filter(|edge| edge.capability == capability)
            .map(|edge| edge.evidence.as_str())
            .collect()
    }

    /// The claims that name the evidence record `evidence`, by capability and
    /// line.
    #[must_use]
    pub fn claims_on(&self, evidence: &str) -> Vec<&ClaimEdge> {
        self.claims
            .iter()
            .filter(|edge| edge.evidence == evidence)
            .collect()
    }

    /// The assessments whose subject is `section`, sorted by id.
    #[must_use]
    pub fn assessments_of(&self, section: &SectionId) -> Vec<&str> {
        self.assesses
            .iter()
            .filter(|edge| &edge.section == section)
            .map(|edge| edge.assessment.as_str())
            .collect()
    }

    /// The capability ids, sorted.
    #[must_use]
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    /// The change ids, sorted.
    #[must_use]
    pub fn changes(&self) -> &[String] {
        &self.changes
    }

    /// The architecture page ids, sorted.
    #[must_use]
    pub fn architecture(&self) -> &[String] {
        &self.architecture
    }

    /// The seven sections of the capability `capability`, in document order;
    /// empty when the graph has no such capability.
    #[must_use]
    pub fn sections(&self, capability: &str) -> Vec<SectionId> {
        if self.capabilities.iter().any(|id| id == capability) {
            CapabilitySection::ALL
                .into_iter()
                .map(|section| SectionId::new(capability, section))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// The PROPOSED blocks, by capability and then by place in the document.
    #[must_use]
    pub fn proposals(&self) -> &[Proposal] {
        &self.proposals
    }

    /// The binding edges, by capability, then code, tests and examples, then
    /// by path.
    #[must_use]
    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    /// The bindings of one capability, with the result of resolving each, by
    /// list (code, tests, examples) and then by path. A binding that did not
    /// resolve is here with its error, so a reader never resolves again.
    pub fn bindings_of<'a>(&'a self, capability: &'a str) -> impl Iterator<Item = &'a Binding> {
        self.bindings
            .iter()
            .filter(move |binding| binding.capability == capability)
    }

    /// The `depends` edges between capabilities that exist, sorted.
    #[must_use]
    pub fn depends(&self) -> &[DependsEdge] {
        &self.depends
    }

    /// The `affects` edges to sections of capabilities that exist, sorted.
    #[must_use]
    pub fn affects(&self) -> &[AffectsEdge] {
        &self.affects
    }

    /// The `changes` edges to capabilities that exist, sorted.
    #[must_use]
    pub fn changes_edges(&self) -> &[ChangesEdge] {
        &self.changes_edges
    }

    /// The link edges, by source, target and line.
    #[must_use]
    pub fn links(&self) -> &[LinkEdge] {
        &self.links
    }

    /// The items that realise a capability: what its `bindings.code` entries
    /// resolve to, sorted by source path and then by id.
    #[must_use]
    pub fn code_of(&self, capability: &str) -> Vec<ItemId> {
        let items = self
            .bindings
            .iter()
            .filter(|b| b.capability == capability && b.kind == BindingKind::Code)
            .flat_map(|b| b.items().iter().copied());
        self.sorted_items(items)
    }

    /// The capabilities bound to an item by any of their lists, sorted by id.
    #[must_use]
    pub fn capabilities_of(&self, symbol: ItemId) -> Vec<&str> {
        let found: BTreeSet<&str> = self
            .bindings
            .iter()
            .filter(|b| b.items().contains(&symbol))
            .map(|b| b.capability.as_str())
            .collect();
        found.into_iter().collect()
    }

    /// The test bindings of a capability that substantiate one of its
    /// sections, sorted by path. See [`Binding::sections`] for the rule.
    #[must_use]
    pub fn tests_for(&self, capability: &str, section: CapabilitySection) -> Vec<&Binding> {
        self.bindings
            .iter()
            .filter(|b| {
                b.capability == capability
                    && b.kind == BindingKind::Test
                    && b.sections.contains(&section)
            })
            .collect()
    }

    /// The sections a change affects, sorted and without repeats.
    #[must_use]
    pub fn sections_of(&self, change: &str) -> Vec<SectionId> {
        let found: BTreeSet<&SectionId> = self
            .affects
            .iter()
            .filter(|edge| edge.change == change)
            .map(|edge| &edge.section)
            .collect();
        found.into_iter().cloned().collect()
    }

    /// The sections that hold a link to a section, sorted and without repeats.
    /// A link whose target is a document as a whole is not a link to one of
    /// its sections, and a link written outside every section has no source
    /// section.
    #[must_use]
    pub fn linking_sections(&self, section: &SectionId) -> Vec<SectionId> {
        let found: BTreeSet<&SectionId> = self
            .links
            .iter()
            .filter(|edge| matches!(&edge.to, LinkEnd::Section(target) if target == section))
            .filter_map(|edge| match &edge.from {
                LinkEnd::Section(source) => Some(source),
                _ => None,
            })
            .collect();
        found.into_iter().cloned().collect()
    }

    fn bound_items(&self, kind: BindingKind) -> Vec<ItemId> {
        let items = self
            .bindings
            .iter()
            .filter(|b| b.kind == kind)
            .flat_map(|b| b.items().iter().copied());
        self.sorted_items(items)
    }

    /// The items without repeats, by source path and then by id.
    fn sorted_items(&self, items: impl Iterator<Item = ItemId>) -> Vec<ItemId> {
        let ordered: BTreeSet<(&str, ItemId)> = items
            .map(|id| (self.item_paths.get(&id).map_or("", String::as_str), id))
            .collect();
        ordered.into_iter().map(|(_, id)| id).collect()
    }
}

/// The records of a load.
pub(crate) struct Records<'a> {
    pub(crate) evidence: &'a [Loaded<EvidenceRecord>],
    pub(crate) assessments: &'a [Loaded<AssessmentRecord>],
    pub(crate) coverage: Option<&'a Loaded<CoverageRecord>>,
}

/// Build the graph of the loaded documents, and the findings of the rules
/// between documents: a reference to a document or a record that does not
/// exist, a claim on evidence that does not cover its capability, and a
/// `depends` cycle.
///
/// `present` holds the ids of the documents that exist as files, loaded or
/// not: a reference to a document that is there but broken is reported by the
/// findings of that document, not as a missing one.
pub(crate) fn build(
    capabilities: &[Loaded<Capability>],
    changes: &[Loaded<Change>],
    architecture: &[Loaded<Architecture>],
    records: &Records<'_>,
    present: &BTreeSet<String>,
    source: &Workspace,
) -> (Graph, Vec<Finding>) {
    let mut graph = Graph::default();
    let mut findings = Vec::new();
    let mut caps: Vec<&Loaded<Capability>> = capabilities.iter().collect();
    caps.sort_by(|a, b| a.value().header.id.cmp(&b.value().header.id));
    let mut chgs: Vec<&Loaded<Change>> = changes.iter().collect();
    chgs.sort_by(|a, b| a.value().header.id.cmp(&b.value().header.id));
    graph.capabilities = caps.iter().map(|c| c.value().header.id.clone()).collect();
    graph.changes = chgs.iter().map(|c| c.value().header.id.clone()).collect();
    graph.architecture = architecture
        .iter()
        .map(|a| a.value().header.id.clone())
        .collect();
    graph.architecture.sort();
    let known: BTreeSet<String> = graph.capabilities.iter().cloned().collect();

    for cap in &caps {
        add_capability(&mut graph, cap, &known, present, source, &mut findings);
    }
    for change in &chgs {
        add_change(&mut graph, change, &known, present, &mut findings);
    }
    findings.extend(cycles(&caps, &graph.depends));
    graph
        .depends
        .sort_by(|a, b| (&a.from, &a.to).cmp(&(&b.from, &b.to)));
    graph.depends.dedup();
    graph.affects.sort();
    graph.affects.dedup();
    graph
        .changes_edges
        .sort_by(|a, b| (&a.change, &a.capability).cmp(&(&b.change, &b.capability)));
    graph.changes_edges.dedup();
    add_links(&mut graph, capabilities, changes, architecture);
    add_records(&mut graph, records, &known, present, source, &mut findings);
    add_claims(&mut graph, &caps, records.evidence, present, &mut findings);
    graph
        .tested_by
        .sort_by(|a, b| (&a.capability, a.line, &a.path).cmp(&(&b.capability, b.line, &b.path)));
    (graph, findings)
}

/// The record nodes, the `for`, subject and `scope` edges, and the findings
/// for a capability or a crate a record names and that does not exist.
fn add_records(
    graph: &mut Graph,
    records: &Records<'_>,
    known: &BTreeSet<String>,
    present: &BTreeSet<String>,
    source: &Workspace,
    findings: &mut Vec<Finding>,
) {
    for evidence in records.evidence {
        let id = &evidence.value().header.id;
        graph.evidence.push(id.clone());
        for capability in &evidence.value().covers {
            if known.contains(capability) {
                graph.covers.push(CoversEdge {
                    evidence: id.clone(),
                    capability: capability.clone(),
                });
            } else if !present.contains(capability) {
                findings.push(Finding::new(
                    kind::EVIDENCE_FOR_MISSING,
                    evidence.path(),
                    Location::Key("for".to_string()),
                    format!("`for` names `{capability}`, which is not a capability document"),
                    "name a capability the evidence covers, or remove the entry",
                ));
            }
        }
    }
    for assessment in records.assessments {
        let id = &assessment.value().header.id;
        graph.assessments.push(id.clone());
        let subject = &assessment.value().subject;
        findings.extend(assessment_id_mismatch(assessment));
        if known.contains(&subject.capability) {
            graph.assesses.push(AssessesEdge {
                assessment: id.clone(),
                section: SectionId::new(subject.capability.clone(), subject.section),
            });
        } else if !present.contains(&subject.capability) {
            findings.push(Finding::new(
                kind::ASSESSMENT_SUBJECT_MISSING,
                assessment.path(),
                Location::Key("subject".to_string()),
                format!(
                    "the subject `{}` is a section of `{}`, which is not a capability document",
                    subject.text(),
                    subject.capability
                ),
                "name a section of an existing capability, or remove the record",
            ));
        }
    }
    if let Some(coverage) = records.coverage {
        let id = &coverage.value().header.id;
        graph.coverage.push(id.clone());
        let packages: BTreeSet<&str> = source
            .packages()
            .iter()
            .map(|package| package.name.as_str())
            .collect();
        for package in &coverage.value().scope {
            if packages.contains(package.as_str()) {
                graph.crates.push(package.clone());
                graph.scope.push(ScopeEdge {
                    coverage: id.clone(),
                    package: package.clone(),
                });
            } else {
                // A crate name (`raf_okf`) is not the package name (`raf-okf`):
                // say which package it is, rather than guess.
                let crate_form = |name: &str| name.replace('-', "_");
                let named: Vec<&str> = packages
                    .iter()
                    .copied()
                    .filter(|known| crate_form(known) == crate_form(package))
                    .collect();
                let known = if let [only] = named.as_slice() {
                    format!(
                        "`scope` names packages by the `name` of their `[package]`, and `{package}` is the crate name of the package `{only}`"
                    )
                } else if packages.is_empty() {
                    "the source workspace has no package".to_string()
                } else {
                    format!(
                        "the packages are {}",
                        packages.iter().copied().collect::<Vec<_>>().join(", ")
                    )
                };
                findings.push(Finding::new(
                    kind::SCOPE_CRATE_MISSING,
                    coverage.path(),
                    Location::Key("scope".to_string()),
                    format!("`scope` names `{package}`, which is no package of the source workspace; {known}"),
                    "name the crate by the `name` of its `[package]`, or remove the entry",
                ));
            }
        }
    }
    graph.evidence.sort();
    graph.assessments.sort();
    graph.crates.sort();
    graph.crates.dedup();
    graph
        .covers
        .sort_by(|a, b| (&a.evidence, &a.capability).cmp(&(&b.evidence, &b.capability)));
    graph
        .assesses
        .sort_by(|a, b| (&a.assessment, &a.section).cmp(&(&b.assessment, &b.section)));
    graph.scope.sort_by(|a, b| a.package.cmp(&b.package));
    add_inputs(graph, records, known, source);
}

/// The `assessment-id-mismatch` finding of an assessment whose id is not the
/// one id of its subject, `assessment:<capability slug>-<section>`.
fn assessment_id_mismatch(assessment: &Loaded<AssessmentRecord>) -> Option<Finding> {
    let value = assessment.value();
    let subject = &value.subject;
    let wanted = crate::assessment::record_slug(&subject.capability, subject.section);
    if wanted.as_deref() == Some(value.header.slug.as_str()) {
        return None;
    }
    let wanted = wanted.map_or_else(
        || format!("`{}` is not a capability id", subject.capability),
        |slug| {
            format!(
                "the id of an assessment of `{}` is `assessment:{slug}`",
                subject.text()
            )
        },
    );
    Some(Finding::new(
        kind::ASSESSMENT_ID_MISMATCH,
        assessment.path(),
        Location::Key("id".to_string()),
        format!(
            "the record `{}` assesses `{}`, and {wanted}, so it is not the assessment of its subject",
            value.header.id,
            subject.text()
        ),
        "name the record after its subject, `<capability slug>-<section>`, or record the assessment again with `assess`",
    ))
}

/// The input edges of the evidence records and the assessments. The evidence
/// ids are sorted already, so an input that names a record finds it.
fn add_inputs(
    graph: &mut Graph,
    records: &Records<'_>,
    known: &BTreeSet<String>,
    source: &Workspace,
) {
    let inputs = records
        .evidence
        .iter()
        .map(|record| (&record.value().header.id, &record.value().inputs))
        .chain(
            records
                .assessments
                .iter()
                .map(|record| (&record.value().header.id, &record.value().inputs)),
        );
    for (id, rows) in inputs {
        for input in rows {
            add_input(graph, id, input, known, source);
        }
    }
    graph
        .inputs
        .sort_by(|a, b| (&a.record, a.kind, &a.name).cmp(&(&b.record, b.kind, &b.name)));
}

/// The edge of one input row of the record `record`, when its name resolves:
/// a section of a capability that exists, a symbol of the source index, or an
/// evidence record that loaded.
fn add_input(
    graph: &mut Graph,
    record: &str,
    input: &Input,
    known: &BTreeSet<String>,
    source: &Workspace,
) {
    let to = match input.kind {
        InputKind::Section => input.name.split_once('#').and_then(|(capability, slug)| {
            let section = CapabilitySection::from_slug(slug)?;
            known
                .contains(capability)
                .then(|| InputTarget::Section(SectionId::new(capability, section)))
        }),
        InputKind::Symbol => source.resolve(&input.name).ok().map(|found| {
            for id in &found.sites {
                if let Some(item) = source.item(*id) {
                    graph.item_paths.insert(*id, item.path.to_string());
                }
            }
            InputTarget::Symbol(graph.sorted_items(found.sites.into_iter()))
        }),
        InputKind::Record => graph
            .evidence
            .binary_search(&input.name)
            .is_ok()
            .then(|| InputTarget::Record(input.name.clone())),
    };
    if let Some(to) = to {
        graph.inputs.push(InputEdge {
            record: record.to_string(),
            kind: input.kind,
            name: input.name.clone(),
            to,
        });
    }
}

/// The claim edges of the capability documents, and the findings for a claim
/// on a record that does not exist or does not cover the capability.
fn add_claims(
    graph: &mut Graph,
    caps: &[&Loaded<Capability>],
    evidence: &[Loaded<EvidenceRecord>],
    present: &BTreeSet<String>,
    findings: &mut Vec<Finding>,
) {
    let loaded = Counted::new(
        Work::ClaimLookups,
        evidence
            .iter()
            .inspect(|_| add(Work::ClaimChecks, 1))
            .map(|record| record.value().header.id.as_str())
            .collect::<HashSet<&str>>(),
    );
    let covered = Counted::new(
        Work::ClaimLookups,
        evidence
            .iter()
            .flat_map(|record| {
                record
                    .value()
                    .covers
                    .iter()
                    .map(|capability| (record.value().header.id.as_str(), capability.as_str()))
            })
            .inspect(|_| add(Work::ClaimChecks, 1))
            .collect::<HashSet<(&str, &str)>>(),
    );
    let present = Counted::new(Work::ClaimLookups, present);
    let prefix = DocType::Evidence.id_prefix();
    for cap in caps {
        let capability = cap.value().header.id.as_str();
        for claim in cap.body().claims() {
            let section = claim.section().and_then(CapabilitySection::from_slug);
            for slug in claim.evidence() {
                add(Work::ClaimChecks, 1);
                let id = format!("{prefix}{slug}");
                if loaded.contains(id.as_str()) {
                    if !covered.contains(&(id.as_str(), capability)) {
                        findings.push(Finding::new(
                            kind::CLAIM_EVIDENCE_NOT_FOR,
                            cap.path(),
                            Location::Line(claim.line()),
                            format!(
                                "the claim names `{id}`, whose `for` does not list `{capability}`"
                            ),
                            format!(
                                "add `{capability}` to the `for` of the record, or name evidence that covers the capability"
                            ),
                        ));
                    }
                    graph.claims.push(ClaimEdge {
                        capability: capability.to_string(),
                        section,
                        evidence: id,
                        line: claim.line(),
                    });
                } else if !present.contains(&id) {
                    findings.push(Finding::new(
                        kind::CLAIM_EVIDENCE_MISSING,
                        cap.path(),
                        Location::Line(claim.line()),
                        format!("the claim names `{id}`, which is not an evidence record"),
                        format!(
                            "record the evidence as {}/{slug}.md, or name an existing record",
                            DocType::Evidence.directory()
                        ),
                    ));
                }
            }
        }
    }
    graph.claims.sort_by(|a, b| {
        (&a.capability, a.line, &a.evidence).cmp(&(&b.capability, b.line, &b.evidence))
    });
}

/// A document that can hold or receive a link.
struct Doc<'a> {
    whole: LinkEnd,
    body: &'a Body,
    capability: Option<&'a str>,
}

/// The section of a capability body that holds a line.
fn section_at(capability: &str, body: &Body, line: usize) -> Option<SectionId> {
    body.sections()
        .iter()
        .find(|s| s.line() <= line && line < s.content().lines.end)
        .and_then(|s| CapabilitySection::from_slug(s.slug()))
        .map(|section| SectionId::new(capability, section))
}

fn add_links(
    graph: &mut Graph,
    capabilities: &[Loaded<Capability>],
    changes: &[Loaded<Change>],
    architecture: &[Loaded<Architecture>],
) {
    let mut docs: BTreeMap<&str, Doc<'_>> = BTreeMap::new();
    for cap in capabilities {
        let id = cap.value().header.id.as_str();
        docs.insert(
            cap.path(),
            Doc {
                whole: LinkEnd::Capability(id.to_string()),
                body: cap.body(),
                capability: Some(id),
            },
        );
    }
    for change in changes {
        docs.insert(
            change.path(),
            Doc {
                whole: LinkEnd::Change(change.value().header.id.clone()),
                body: change.body(),
                capability: None,
            },
        );
    }
    for page in architecture {
        docs.insert(
            page.path(),
            Doc {
                whole: LinkEnd::Architecture(page.value().header.id.clone()),
                body: page.body(),
                capability: None,
            },
        );
    }
    let mut links = Vec::new();
    for (path, doc) in &docs {
        for link in doc.body.links() {
            let (target_path, anchor) = match link.target_in(path) {
                Ok(LinkTarget::Fragment(anchor)) => (*path, Some(anchor)),
                Ok(LinkTarget::Relative { path, anchor }) => {
                    match docs.get_key_value(path.as_str()) {
                        Some((key, _)) => (*key, anchor),
                        None => continue,
                    }
                }
                // An external link, or a target that is not a path.
                Ok(_) | Err(_) => continue,
            };
            let target = &docs[target_path];
            let to = match (anchor, target.capability) {
                (Some(anchor), _) if !target.body.has_anchor(&anchor) => continue,
                (Some(anchor), Some(id)) => target
                    .body
                    .headings()
                    .iter()
                    .find(|h| h.anchor() == anchor)
                    .and_then(|h| section_at(id, target.body, h.line()))
                    .map_or_else(|| target.whole.clone(), LinkEnd::Section),
                (None, _) | (Some(_), None) => target.whole.clone(),
            };
            let from = doc
                .capability
                .and_then(|id| section_at(id, doc.body, link.line()))
                .map_or_else(|| doc.whole.clone(), LinkEnd::Section);
            links.push(LinkEdge {
                from,
                to,
                line: link.line(),
            });
        }
    }
    links.sort_by(|a, b| {
        (a.from.sort_key(), a.to.sort_key(), a.line).cmp(&(
            b.from.sort_key(),
            b.to.sort_key(),
            b.line,
        ))
    });
    links.dedup();
    graph.links = links;
}

fn add_capability(
    graph: &mut Graph,
    cap: &Loaded<Capability>,
    known: &BTreeSet<String>,
    present: &BTreeSet<String>,
    source: &Workspace,
    findings: &mut Vec<Finding>,
) {
    let id = &cap.value().header.id;
    for (index, block) in cap.body().proposed_blocks().iter().enumerate() {
        graph.proposals.push(Proposal {
            capability: id.clone(),
            change: block.slug().to_string(),
            section: block.section().and_then(CapabilitySection::from_slug),
            ordinal: index,
        });
    }
    for target in &cap.value().depends {
        if known.contains(target) {
            graph.depends.push(DependsEdge {
                from: id.clone(),
                to: target.clone(),
            });
        } else if !present.contains(target) {
            findings.push(Finding::new(
                kind::DEPENDS_MISSING,
                cap.path(),
                Location::Key("depends".to_string()),
                format!("`depends` names `{target}`, which is not a capability document"),
                "name an existing capability, or remove the entry",
            ));
        }
    }
    let lists = [
        (BindingKind::Code, &cap.value().bindings.code),
        (BindingKind::Test, &cap.value().bindings.tests),
        (BindingKind::Example, &cap.value().bindings.examples),
    ];
    let mut bindings = Vec::new();
    for (kind, paths) in lists {
        for path in paths {
            bindings.push(bind(id, kind, path, cap.body(), source, graph));
        }
    }
    bindings.sort_by(|a, b| (a.kind, &a.path).cmp(&(b.kind, &b.path)));
    graph.bindings.extend(bindings);
    add_tested_by(graph, cap, findings);
}

/// The tested-by edges of one capability, and `claim-test-unbound` for each
/// path of a marker that is no path of its `tests` or `examples` bindings.
fn add_tested_by(graph: &mut Graph, cap: &Loaded<Capability>, findings: &mut Vec<Finding>) {
    let id = &cap.value().header.id;
    let bindings = &cap.value().bindings;
    for marker in cap.body().tested_by() {
        let section = marker.section().and_then(CapabilitySection::from_slug);
        for path in marker.paths() {
            if bindings.tests.contains(path) || bindings.examples.contains(path) {
                graph.tested_by.push(TestedByEdge {
                    capability: id.clone(),
                    section,
                    path: path.clone(),
                    line: marker.line(),
                });
            } else {
                findings.push(Finding::new(
                    kind::CLAIM_TEST_UNBOUND,
                    cap.path(),
                    Location::Line(marker.line()),
                    format!(
                        "the tested-by marker names `{path}`, which is no path of the `tests` or `examples` bindings of `{id}`"
                    ),
                    "name a path of the capability's `bindings.tests` or `bindings.examples`, or bind the test there first",
                ));
            }
        }
    }
}

fn bind(
    capability: &str,
    kind: BindingKind,
    path: &str,
    body: &Body,
    source: &Workspace,
    graph: &mut Graph,
) -> Binding {
    let resolution = source.resolve(path).map(|found| {
        let mut sites = found.sites;
        for id in &sites {
            if let Some(item) = source.item(*id) {
                graph.item_paths.insert(*id, item.path.to_string());
            }
        }
        sites.sort_by(|a, b| {
            let name = |id: &ItemId| graph.item_paths.get(id).cloned().unwrap_or_default();
            (name(a), *a).cmp(&(name(b), *b))
        });
        sites.dedup();
        sites
    });
    let examples = match (&resolution, kind) {
        (Ok(sites), BindingKind::Example) => sites
            .iter()
            .filter_map(|id| {
                let item = source.item(*id)?;
                Some((*id, bindings::example_kind(source, item)?))
            })
            .collect(),
        _ => Vec::new(),
    };
    let sections = match kind {
        BindingKind::Code => Vec::new(),
        BindingKind::Test | BindingKind::Example => substantiated(body, path),
    };
    Binding {
        capability: capability.to_string(),
        kind,
        path: path.to_string(),
        resolution,
        sections,
        examples,
    }
}

fn add_change(
    graph: &mut Graph,
    change: &Loaded<Change>,
    known: &BTreeSet<String>,
    present: &BTreeSet<String>,
    findings: &mut Vec<Finding>,
) {
    let value = change.value();
    let id = &value.header.id;
    for target in &value.changes {
        if known.contains(target) {
            graph.changes_edges.push(ChangesEdge {
                change: id.clone(),
                capability: target.clone(),
            });
        } else if !present.contains(target) {
            findings.push(Finding::new(
                kind::CHANGES_MISSING,
                change.path(),
                Location::Key("changes".to_string()),
                format!("`changes` names `{target}`, which is not a capability document"),
                "name an existing capability, or remove the entry",
            ));
        }
    }
    for entry in &value.affects {
        if known.contains(&entry.capability) {
            graph.affects.push(AffectsEdge {
                change: id.clone(),
                section: SectionId::new(entry.capability.clone(), entry.section),
            });
        }
    }
    for item in &value.items {
        if let Some(Outcome::Moved(slug)) = &item.outcome {
            let target = format!("change:{slug}");
            if !present.contains(&target) {
                findings.push(Finding::new(
                    kind::MOVED_MISSING,
                    change.path(),
                    Location::Key("items".to_string()),
                    format!("an item has the outcome `moved:{target}`, but no such change record exists"),
                    "name the change record that now carries the claim, or give the item another outcome",
                ));
            }
        }
    }
}

/// The `depends` cycles: one finding for each group of capabilities that
/// depend on each other, named in the file of the first of them.
fn cycles(caps: &[&Loaded<Capability>], depends: &[DependsEdge]) -> Vec<Finding> {
    let ids: Vec<&str> = caps.iter().map(|c| c.value().header.id.as_str()).collect();
    let index: BTreeMap<&str, usize> = ids.iter().enumerate().map(|(i, id)| (*id, i)).collect();
    let mut adjacent: Vec<Vec<usize>> = vec![Vec::new(); ids.len()];
    for edge in depends {
        if let (Some(&from), Some(&to)) =
            (index.get(edge.from.as_str()), index.get(edge.to.as_str()))
        {
            adjacent[from].push(to);
        }
    }
    let mut findings = Vec::new();
    for mut group in strongly_connected(&adjacent) {
        let cyclic = group.len() > 1 || adjacent[group[0]].contains(&group[0]);
        if !cyclic {
            continue;
        }
        group.sort_unstable();
        let members: Vec<&str> = group.iter().map(|i| ids[*i]).collect();
        let message = if members.len() == 1 {
            format!("`{}` depends on itself", members[0])
        } else {
            format!(
                "the capabilities {} depend on each other",
                members
                    .iter()
                    .map(|id| format!("`{id}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        findings.push(Finding::new(
            kind::DEPENDS_CYCLE,
            caps[group[0]].path(),
            Location::Key("depends".to_string()),
            message,
            "remove a `depends` entry so the capabilities form no cycle",
        ));
    }
    findings
}

/// The strongly connected components of a graph (Tarjan, without recursion),
/// each as a list of node numbers, ordered by their smallest node.
fn strongly_connected(adjacent: &[Vec<usize>]) -> Vec<Vec<usize>> {
    const UNSEEN: usize = usize::MAX;
    let n = adjacent.len();
    let mut order = vec![UNSEEN; n];
    let mut low = vec![0; n];
    let mut on_stack = vec![false; n];
    let mut stack = Vec::new();
    let mut counter = 0;
    let mut groups = Vec::new();
    for root in 0..n {
        if order[root] != UNSEEN {
            continue;
        }
        let mut walk = vec![(root, 0_usize)];
        order[root] = counter;
        low[root] = counter;
        counter += 1;
        stack.push(root);
        on_stack[root] = true;
        while let Some(&(node, next)) = walk.last() {
            if let Some(&target) = adjacent[node].get(next) {
                if let Some(top) = walk.last_mut() {
                    top.1 += 1;
                }
                if order[target] == UNSEEN {
                    order[target] = counter;
                    low[target] = counter;
                    counter += 1;
                    stack.push(target);
                    on_stack[target] = true;
                    walk.push((target, 0));
                } else if on_stack[target] {
                    low[node] = low[node].min(order[target]);
                }
            } else {
                walk.pop();
                if let Some(&(parent, _)) = walk.last() {
                    low[parent] = low[parent].min(low[node]);
                }
                if low[node] == order[node] {
                    let mut group = Vec::new();
                    while let Some(member) = stack.pop() {
                        on_stack[member] = false;
                        group.push(member);
                        if member == node {
                            break;
                        }
                    }
                    groups.push(group);
                }
            }
        }
    }
    for group in &mut groups {
        group.sort_unstable();
    }
    groups.sort();
    groups
}

/// The sections a test or example binding substantiates: each section other
/// than Verification that names the path in a code span or holds a tested-by
/// marker that names it, and Verification when there is none. A code span
/// inside an HTML comment does not count: a comment is no text of the section.
fn substantiated(body: &Body, path: &str) -> Vec<CapabilitySection> {
    let holding: Vec<CapabilitySection> = CapabilitySection::ALL
        .into_iter()
        .filter(|section| *section != CapabilitySection::Verification)
        .filter(|section| {
            let spanned = body
                .sections()
                .iter()
                .find(|s| s.slug() == section.slug())
                .and_then(|s| {
                    prose_of(body, s).map(|text| names_in_code_span(&without_comments(&text), path))
                })
                .unwrap_or(false);
            spanned || marks(body, *section, path)
        })
        .collect();
    if holding.is_empty() {
        vec![CapabilitySection::Verification]
    } else {
        holding
    }
}

/// Whether a tested-by marker in `section` names `path`.
fn marks(body: &Body, section: CapabilitySection, path: &str) -> bool {
    body.tested_by().iter().any(|marker| {
        marker.section() == Some(section.slug()) && marker.paths().iter().any(|p| p == path)
    })
}

/// `text` with each HTML comment outside a code span removed, its line breaks
/// kept. Code spans are read per paragraph as [`code_spans`] reads them: a run
/// of backticks opens a span that the next run of the same length in the
/// paragraph closes. A `<!--` that no `-->` closes runs to the end.
fn without_comments(text: &str) -> String {
    let bytes = text.as_bytes();
    let run = |from: usize| bytes[from..].iter().take_while(|b| **b == b'`').count();
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'`' {
            let width = run(at);
            let mut scan = at + width;
            let mut close = None;
            while scan < bytes.len() && !bytes[scan..].starts_with(b"\n\n") {
                if bytes[scan] == b'`' {
                    let other = run(scan);
                    if other == width {
                        close = Some(scan + width);
                        break;
                    }
                    scan += other;
                } else {
                    scan += 1;
                }
            }
            let end = close.unwrap_or(at + width);
            out.push_str(&text[at..end]);
            at = end;
        } else if text[at..].starts_with("<!--") {
            let body = &text[at + 4..];
            let end = body.find("-->").map_or(body.len(), |close| close + 3);
            out.extend(body[..end].chars().filter(|c| *c == '\n'));
            at += 4 + end;
        } else {
            let next = text[at..].chars().next().map_or(1, char::len_utf8);
            out.push_str(&text[at..at + next]);
            at += next;
        }
    }
    out
}

/// The text of a section with its fenced code blocks, PROPOSED blocks and
/// generated blocks blanked out, line breaks kept. A generated block is
/// derived from this very text, so it must not count as prose.
///
/// The blocks are merged into sorted, disjoint ranges once and the text is
/// read in one pass, so the time is linear in the text and the blocks.
pub(crate) fn prose_of(body: &Body, section: &raf_okf::Section) -> Option<String> {
    let text = body.section_text(section)?;
    let start = section.content().bytes.start;
    let mut blanked: Vec<(usize, usize)> = body
        .fenced_blocks()
        .iter()
        .map(raf_okf::FencedBlock::span)
        .chain(
            body.proposed_blocks()
                .iter()
                .map(raf_okf::ProposedBlock::span),
        )
        .chain(
            body.generated_blocks()
                .iter()
                .map(raf_okf::GeneratedBlock::outer),
        )
        .map(|span| {
            (
                span.bytes.start.saturating_sub(start),
                span.bytes.end.saturating_sub(start),
            )
        })
        .filter(|(from, to)| from < to)
        .collect();
    blanked.sort_unstable();
    let mut merged: Vec<(usize, usize)> = Vec::with_capacity(blanked.len());
    for (from, to) in blanked {
        match merged.last_mut() {
            Some(last) if from <= last.1 => last.1 = last.1.max(to),
            _ => merged.push((from, to)),
        }
    }
    let mut ranges = merged.into_iter().peekable();
    Some(
        text.char_indices()
            .map(|(at, c)| {
                add(Work::ProseChars, 1);
                while ranges.peek().is_some_and(|(_, to)| *to <= at) {
                    ranges.next();
                    add(Work::BlankedSteps, 1);
                }
                let inside = ranges.peek().is_some_and(|(from, _)| *from <= at);
                if inside && c != '\n' && c != '\r' {
                    ' '
                } else {
                    c
                }
            })
            .collect(),
    )
}

/// Every path a code span of `text` names as a whole path, by the rule of
/// [`names_in_code_span`]: for each paragraph, each code span, and in it each
/// run of identifiers joined by `::` that no identifier character or `:`
/// precedes and no identifier character or `::` follows.
///
/// A path `p` is in the set exactly when `names_in_code_span(text, p)`, and
/// the set is found in one pass over the text, so a section that names many
/// symbols is judged in time linear in its length.
pub(crate) fn code_span_paths(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut paragraph = String::new();
    for line in text.lines().chain(std::iter::once("")) {
        add(Work::ContractLines, 1);
        if line.trim().is_empty() {
            for span in code_spans(&paragraph) {
                paths_in_span(span, &mut found);
            }
            paragraph.clear();
        } else {
            paragraph.push_str(line);
            paragraph.push('\n');
        }
    }
    found
}

/// The whole paths of one code span (see [`code_span_paths`]).
fn paths_in_span(span: &str, found: &mut BTreeSet<String>) {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    let chars: Vec<(usize, char)> = span.char_indices().collect();
    let mut index = 0;
    while index < chars.len() {
        let (at, c) = chars[index];
        let before = index.checked_sub(1).map(|i| chars[i].1);
        if !ident(c) || before.is_some_and(|b| ident(b) || b == ':') {
            index += 1;
            continue;
        }
        // The longest run `ident (:: ident)*` from here.
        let mut end = index;
        loop {
            while end < chars.len() && ident(chars[end].1) {
                end += 1;
            }
            let joined = end + 2 < chars.len()
                && chars[end].1 == ':'
                && chars[end + 1].1 == ':'
                && ident(chars[end + 2].1);
            if joined {
                end += 2;
            } else {
                break;
            }
        }
        let stop = chars.get(end).map_or(span.len(), |(at, _)| *at);
        let after = &span[stop..];
        if !after.starts_with("::") {
            found.insert(span[at..stop].to_string());
        }
        index = end.max(index + 1);
    }
}

/// Whether a code span of `text` names `path`: the path stands in the span as
/// a whole path, not as a part of a longer identifier or of a longer path.
fn names_in_code_span(text: &str, path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    let mut paragraph = String::new();
    for line in text.lines().chain(std::iter::once("")) {
        if line.trim().is_empty() {
            if code_spans(&paragraph).iter().any(|span| names(span, path)) {
                return true;
            }
            paragraph.clear();
        } else {
            paragraph.push_str(line);
            paragraph.push('\n');
        }
    }
    false
}

/// The contents of the code spans of one paragraph: a run of backticks opens
/// a span that the next run of the same length closes.
fn code_spans(paragraph: &str) -> Vec<&str> {
    add(Work::ParagraphBytes, paragraph.len());
    let bytes = paragraph.as_bytes();
    let run = |from: usize| bytes[from..].iter().take_while(|b| **b == b'`').count();
    let mut spans = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] != b'`' {
            at += 1;
            continue;
        }
        let width = run(at);
        let open_end = at + width;
        let mut scan = open_end;
        let mut close = None;
        while scan < bytes.len() {
            if bytes[scan] == b'`' {
                let other = run(scan);
                if other == width {
                    close = Some(scan);
                    break;
                }
                scan += other;
            } else {
                scan += 1;
            }
        }
        if let Some(close) = close {
            spans.push(&paragraph[open_end..close]);
            at = close + width;
        } else {
            at = open_end;
        }
    }
    spans
}

fn names(span: &str, path: &str) -> bool {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    span.match_indices(path).any(|(at, _)| {
        let before = span[..at].chars().next_back();
        let after = &span[at + path.len()..];
        !before.is_some_and(|c| ident(c) || c == ':')
            && !after.chars().next().is_some_and(ident)
            && !after.starts_with("::")
    })
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use raf_source::{MemoryTree, Workspace};

    use super::{code_span_paths, names_in_code_span, without_comments};
    use crate::Workplan;
    use crate::work::{Work, measure};

    /// The set of paths a text names is exactly the paths `names_in_code_span`
    /// accepts, for the paths of the text and for paths it does not hold.
    #[test]
    fn the_paths_of_the_code_spans_are_the_paths_names_in_code_span_accepts() {
        let texts = [
            "Call `demo::run` and `demo::run()`.",
            "A `demo::run::inner` path and `demo::run::*` glob.",
            "`x.demo::run` and `::demo::run` and `ademo::run`.",
            "`demo::run:x` with one colon, `demo::runner` longer.",
            "A span that opens `demo::run\nand closes` on the next line.",
            "A span over a blank line `demo::run\n\nis` not a span.",
            "``demo::run`` doubled, ``a`b`` with a tick, `` ` `` alone.",
            "`ünïcode::päth` and `demo::_hidden` and `demo::r2d2`.",
            "Unclosed `demo::run and nothing else.",
            "`a::a::a` and `a::a`.",
            "",
        ];
        let probes = [
            "demo::run",
            "demo::run::inner",
            "demo",
            "run",
            "demo::runner",
            "ünïcode::päth",
            "demo::_hidden",
            "demo::r2d2",
            "a::a",
            "a::a::a",
            "a",
        ];
        for text in texts {
            let found = code_span_paths(text);
            for probe in probes
                .iter()
                .copied()
                .chain(found.iter().map(String::as_str))
            {
                assert_eq!(
                    found.contains(probe),
                    names_in_code_span(text, probe),
                    "{probe} in {text:?}"
                );
            }
        }
    }

    /// The most any kind of work may count while one section is read in a
    /// growth test: a read that does work per block and per character stops
    /// here at once instead of running to the end.
    const CEILING: usize = 1_000_000;

    /// The paragraphs a section of `count` of them holds, each beside a
    /// fenced block.
    fn paragraphs(count: usize) -> String {
        let paragraph = "`demo::kept` x.\n\n```\n`demo::fenced`\n```\n\n";
        format!("# Title\n\n## Contract\n\n{}", paragraph.repeat(count))
    }

    /// A section of many paragraphs, each beside a fenced block, is read in
    /// work linear in its text and its blocks: the blocks are blanked, the
    /// code spans outside them are read, and none inside them is. The merged
    /// ranges are stepped past once each, however many characters follow
    /// them, and every character is read once. The section is built at one,
    /// two and three units; the work of the third adds no more than the
    /// second did.
    #[test]
    fn a_section_of_many_blocks_is_read_in_work_linear_in_its_blocks() {
        const UNIT: usize = 5_000;
        let read = |units: usize| {
            let text = paragraphs(units * UNIT);
            let body = raf_okf::Body::parse(&text).expect("a body");
            assert_eq!(body.fenced_blocks().len(), units * UNIT);
            let section = &body.sections()[0];
            let ((prose, found), tally) = measure(CEILING, || {
                let prose = super::prose_of(&body, section).expect("the section");
                let found = code_span_paths(&prose);
                (prose, found)
            });
            assert!(found.contains("demo::kept"));
            assert!(!found.contains("demo::fenced"));
            let whole = body.section_text(section).expect("text");
            assert_eq!(prose.len(), whole.len());
            assert_eq!(tally.get(Work::ProseChars), whole.chars().count());
            assert!(tally.get(Work::BlankedSteps) <= units * UNIT);
            tally
        };
        let [one, two, three] = [1, 2, 3].map(read);
        for work in [Work::BlankedSteps, Work::ProseChars] {
            let (one, two, three) = (one.get(work), two.get(work), three.get(work));
            assert!(one > 0, "{work:?}");
            assert!(three + one <= 2 * two, "{work:?}: {one}, {two}, {three}");
        }
    }

    /// The text of a capability with the Purpose `purpose`.
    fn capability(id: &str, purpose: &str) -> String {
        let mut sections = String::new();
        for name in [
            "Purpose",
            "Contract",
            "Usage",
            "Examples",
            "Constraints",
            "Explanation",
            "Verification",
        ] {
            let text = if name == "Purpose" {
                purpose
            } else {
                "Text.\n"
            };
            write!(sections, "## {name}\n\n{text}\n").expect("a string takes the text");
        }
        format!(
            "---\ntype: Capability\nid: cap:{id}\ntitle: T\ndescription: D.\n\
             state: designed\nbindings:\n  code: []\n  tests: []\n  examples: []\n\
             depends: []\n---\n\n# T\n\n{sections}"
        )
    }

    /// An evidence record `slug` that covers `covers`.
    fn evidence(slug: &str, covers: &[String]) -> String {
        format!(
            "---\ntype: Evidence\nid: evidence:{slug}\nsubject: command\ninputs:\n  \
             - kind: symbol\n    name: demo::real\n    digest: {DIGEST}\n\
             verdict: pass\nby: agent\nfor: [{list}]\ncommand: \"cargo bench\"\n\
             revision: b499578a1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f\n---\n",
            list = covers
                .iter()
                .map(|id| format!("cap:{id}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    const DIGEST: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

    /// What loading `capabilities` capabilities with `claims` claims each,
    /// every claim naming `slugs` records that exist or not, costs the claim
    /// judgment, and the findings it gives.
    fn judge_claims(
        capabilities: usize,
        claims: usize,
        records: usize,
        slugs: &[String],
    ) -> (usize, usize, usize) {
        let ids: Vec<String> = (0..capabilities).map(|n| format!("c{n}")).collect();
        let marker = format!("<!-- raf:claim evidence={} -->\n", slugs.join(","));
        let purpose = format!("Purpose.\n{}", format!("\nx\n{marker}").repeat(claims));
        let mut files: Vec<(String, String)> = ids
            .iter()
            .map(|id| {
                (
                    format!("docs/capabilities/{id}.md"),
                    capability(id, &purpose),
                )
            })
            .collect();
        for n in 0..records {
            files.push((
                format!("work/evidence/r{n}.md"),
                evidence(&format!("r{n}"), &ids),
            ));
        }
        let tree: MemoryTree = files
            .iter()
            .map(|(path, text)| (path.as_str(), text.as_str()))
            .collect();
        let source = Workspace::load(
            &MemoryTree::from([
                (
                    "Cargo.toml",
                    "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
                ),
                ("src/lib.rs", "pub fn real() {}\n"),
            ]),
            &["."],
        )
        .expect("a workspace");
        // The slugs of the claims, and the records entered into the sets once.
        let ceiling = 2 * capabilities * claims * slugs.len() + records * (capabilities + 1);
        let ((claim_edges, missing), tally) = measure(ceiling, || {
            let plan = Workplan::load(&tree, &source);
            let missing = plan
                .check()
                .iter()
                .filter(|finding| finding.kind == "claim-evidence-missing")
                .count();
            (plan.graph().claims().len(), missing)
        });
        assert_eq!(
            claim_edges,
            capabilities * claims * records_named(slugs, records)
        );
        (
            tally.get(Work::ClaimChecks),
            tally.get(Work::ClaimLookups),
            missing,
        )
    }

    /// How many of `slugs` name one of the first `records` records.
    fn records_named(slugs: &[String], records: usize) -> usize {
        slugs
            .iter()
            .filter(|slug| {
                slug.strip_prefix('r')
                    .and_then(|n| n.parse::<usize>().ok())
                    .is_some_and(|n| n < records)
            })
            .count()
    }

    /// The claims of the capabilities are judged against sets that are built
    /// once: each record is entered into them once, each `for` entry once,
    /// and each slug of each claim is looked up once, however many claims
    /// there are.
    #[test]
    fn a_comment_outside_a_code_span_is_removed_and_one_inside_a_span_is_kept() {
        assert_eq!(
            without_comments("Call <!-- `demo::run` --> it.\n"),
            "Call  it.\n"
        );
        assert_eq!(
            without_comments("Write `<!-- x -->` and ``a ` <!-- b``.\n"),
            "Write `<!-- x -->` and ``a ` <!-- b``.\n"
        );
        // A comment keeps its line breaks, and one never closed runs to the end.
        assert_eq!(without_comments("a <!-- b\nc --> d"), "a \n d");
        assert_eq!(without_comments("a <!-- b\n\nc"), "a \n\n");
        // A backtick that its paragraph does not close opens no span, so the
        // comment after it is removed; text around characters of several bytes
        // is kept whole.
        assert_eq!(
            without_comments("Héllo … <!-- `x` -->\n\n`y\n\nz <!-- c --> …"),
            "Héllo … \n\n`y\n\nz  …"
        );
        assert!(!names_in_code_span(
            &without_comments("Run <!-- `demo::run` --> it.\n"),
            "demo::run"
        ));
        assert!(names_in_code_span(
            &without_comments("Run `demo::run` <!-- note --> now.\n"),
            "demo::run"
        ));
    }

    #[test]
    fn the_claims_of_many_capabilities_are_judged_against_sets_built_once() {
        let slugs: Vec<String> = ["r0", "r1", "ghost"].map(String::from).to_vec();
        for (capabilities, claims, records) in [(4, 500, 3), (8, 250, 40)] {
            let (checks, lookups, missing) = judge_claims(capabilities, claims, records, &slugs);
            let total_slugs = capabilities * claims * slugs.len();
            // Each record enters `loaded` once and `covered` once for each
            // capability it covers.
            let entered = records + records * capabilities;
            assert_eq!(checks, total_slugs + entered, "{capabilities} x {claims}");
            assert_eq!(missing, capabilities * claims, "one finding per ghost");
            // Each slug is looked up among the loaded records, and then among
            // the records the capability is covered by or among those that
            // exist as files: two lookups for each slug.
            assert_eq!(lookups, 2 * total_slugs, "{capabilities} x {claims}");
        }
    }

    #[test]
    fn ten_thousand_named_paths_are_found_in_one_pass() {
        let text = (0..10_000).fold(String::new(), |mut text, n| {
            let _ = write!(text, "`demo::item_{n}` ");
            text
        });
        let found = code_span_paths(&text);
        assert_eq!(found.len(), 10_000);
        assert!(found.contains("demo::item_9999"));
    }
}
