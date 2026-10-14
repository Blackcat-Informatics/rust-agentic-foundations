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
//!   capabilities, and the code symbols, tests and examples the bindings
//!   resolve to.
//! * **Edges** ([`Edge`]): bindings from a capability to a path of the source
//!   index, `depends` between capabilities, `affects` from a change to a
//!   section of a capability, `changes` from a change to a capability, and
//!   Markdown links. A link edge is section-granular: it runs from the
//!   section that holds the link to the section its anchor names, or to the
//!   target document when the link has no anchor.
//!
//! The queries are [`Graph::code_of`], [`Graph::capabilities_of`],
//! [`Graph::tests_for`], [`Graph::sections_of`] and
//! [`Graph::linking_sections`]. Each returns typed values in a fixed order.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use raf_okf::{
    Architecture, Body, Capability, CapabilitySection, Change, LinkTarget, Location, Outcome,
};
use raf_source::{ItemId, ResolveError, Workspace};

use crate::bindings::{self, ExampleKind};
use crate::finding::kind;
use crate::{Finding, Loaded};

/// One of the seven sections of a capability, named by the capability id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionId {
    capability: String,
    section: CapabilitySection,
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
fn position(section: CapabilitySection) -> usize {
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
    /// A source item bound as code.
    Symbol,
    /// A source item bound as a test.
    Test,
    /// A source item bound as an example.
    Example,
}

impl NodeKind {
    /// Every kind of node.
    pub const ALL: [Self; 8] = [
        Self::Capability,
        Self::Section,
        Self::Change,
        Self::Architecture,
        Self::Proposal,
        Self::Symbol,
        Self::Test,
        Self::Example,
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
    /// A source item bound as code.
    Symbol(ItemId),
    /// A source item bound as a test.
    Test(ItemId),
    /// A source item bound as an example.
    Example(ItemId),
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
    /// outside code fences and PROPOSED blocks holds the bound path in a code
    /// span, and Verification when there is none. A code binding
    /// substantiates no section.
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
}

impl EdgeKind {
    /// Every kind of edge.
    pub const ALL: [Self; 5] = [
        Self::Binding,
        Self::Depends,
        Self::Affects,
        Self::Changes,
        Self::Link,
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
    /// The source path of every item a binding resolved to.
    item_paths: BTreeMap<ItemId, String>,
}

impl Graph {
    /// Every node: the capabilities, their sections, the changes, the
    /// architecture pages, the proposals, and the symbols, tests and examples
    /// bound by a resolving binding, each group sorted.
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
        for (kind, make) in [
            (BindingKind::Code, Node::Symbol as fn(ItemId) -> Node),
            (BindingKind::Test, Node::Test),
            (BindingKind::Example, Node::Example),
        ] {
            nodes.extend(self.bound_items(kind).into_iter().map(make));
        }
        nodes
    }

    /// Every edge: the bindings, then `depends`, `affects`, `changes` and
    /// links, each group sorted.
    #[must_use]
    pub fn edges(&self) -> Vec<Edge> {
        let mut edges: Vec<Edge> = self.bindings.iter().cloned().map(Edge::Binding).collect();
        edges.extend(self.depends.iter().cloned().map(Edge::Depends));
        edges.extend(self.affects.iter().cloned().map(Edge::Affects));
        edges.extend(self.changes_edges.iter().cloned().map(Edge::Changes));
        edges.extend(self.links.iter().cloned().map(Edge::Link));
        edges
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

/// Build the graph of the loaded documents, and the findings of the rules
/// between documents: a reference to a document that does not exist, and a
/// `depends` cycle.
///
/// `present` holds the ids of the documents that exist as files, loaded or
/// not: a reference to a document that is there but broken is reported by the
/// findings of that document, not as a missing one.
pub(crate) fn build(
    capabilities: &[Loaded<Capability>],
    changes: &[Loaded<Change>],
    architecture: &[Loaded<Architecture>],
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
    (graph, findings)
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

/// The sections a test or example binding substantiates.
fn substantiated(body: &Body, path: &str) -> Vec<CapabilitySection> {
    let holding: Vec<CapabilitySection> = CapabilitySection::ALL
        .into_iter()
        .filter(|section| *section != CapabilitySection::Verification)
        .filter(|section| {
            body.sections()
                .iter()
                .find(|s| s.slug() == section.slug())
                .and_then(|s| prose_of(body, s).map(|text| names_in_code_span(&text, path)))
                .unwrap_or(false)
        })
        .collect();
    if holding.is_empty() {
        vec![CapabilitySection::Verification]
    } else {
        holding
    }
}

/// The text of a section with its fenced code blocks, PROPOSED blocks and
/// generated blocks blanked out, line breaks kept. A generated block is
/// derived from this very text, so it must not count as prose.
fn prose_of(body: &Body, section: &raf_okf::Section) -> Option<String> {
    let text = body.section_text(section)?;
    let start = section.content().bytes.start;
    let blanked: Vec<(usize, usize)> = body
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
        .collect();
    Some(
        text.char_indices()
            .map(|(at, c)| {
                let inside = blanked.iter().any(|(from, to)| (*from..*to).contains(&at));
                if inside && c != '\n' && c != '\r' {
                    ' '
                } else {
                    c
                }
            })
            .collect(),
    )
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
