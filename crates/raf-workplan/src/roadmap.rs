// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The roadmap: every capability by state, with the open changes that list it
//! and, for an implementing capability, the gaps in its bindings.
//!
//! [`Workplan::status`](crate::Workplan::status) computes a [`Roadmap`] once
//! from the graph. The value carries everything a status listing or the
//! `catalogue` block of `docs/capabilities/index.md` shows, so what is built
//! and what is not has one derived answer. Printing it and rendering the block
//! are separate from the value.

use std::fmt;

use raf_okf::{Capability, Change, State};

use crate::graph::{BindingKind, Graph};
use crate::model::Loaded;

/// What an implementing capability still lacks in its `bindings`.
///
/// A gap is a list with no entry, or an entry whose path does not resolve in
/// the source index. A binding that resolves is never a gap, whatever its role.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Gap {
    /// The list has no entries.
    EmptyList(BindingKind),
    /// An entry of the list names a path that does not resolve. The reason is
    /// on the binding itself, in [`Graph::bindings_of`].
    Unresolved {
        /// The list the entry is in.
        list: BindingKind,
        /// The path as the document writes it.
        binding: String,
    },
}

impl Gap {
    /// The list the gap is in.
    #[must_use]
    pub fn list(&self) -> BindingKind {
        match self {
            Self::EmptyList(list) | Self::Unresolved { list, .. } => *list,
        }
    }

    /// The binding path, for a gap that is an entry that does not resolve.
    #[must_use]
    pub fn binding(&self) -> Option<&str> {
        match self {
            Self::EmptyList(_) => None,
            Self::Unresolved { binding, .. } => Some(binding),
        }
    }
}

impl fmt::Display for Gap {
    /// The gap as a phrase for a row: "bindings.tests is empty" or
    /// "bindings.code `a::b` does not resolve".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyList(list) => write!(f, "bindings.{} is empty", list.key()),
            Self::Unresolved { list, binding } => {
                write!(f, "bindings.{} `{binding}` does not resolve", list.key())
            }
        }
    }
}

/// One capability in the roadmap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoadmapEntry {
    id: String,
    slug: String,
    title: String,
    description: String,
    state: State,
    path: String,
    open_changes: Vec<String>,
    gaps: Vec<Gap>,
}

impl RoadmapEntry {
    /// The capability id, with its prefix: `cap:<slug>`.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The slug of the id, which is the file name without `.md`.
    #[must_use]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// The title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// The one-line description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// The state.
    #[must_use]
    pub fn state(&self) -> State {
        self.state
    }

    /// The repository-relative path of the document, with `/` separators.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The link target of the document as written in
    /// `docs/capabilities/index.md`, which sits beside it: the file name.
    #[must_use]
    pub fn link_target(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }

    /// The ids of the changes that list the capability in `changes`, sorted.
    #[must_use]
    pub fn open_changes(&self) -> &[String] {
        &self.open_changes
    }

    /// The slugs of the changes that list the capability, without the
    /// `change:` prefix, sorted.
    #[must_use]
    pub fn open_change_slugs(&self) -> Vec<&str> {
        self.open_changes
            .iter()
            .map(|id| crate::lookup::bare_slug(id))
            .collect()
    }

    /// The gaps in the bindings, in list order (`code`, `tests`, `examples`)
    /// and then by path. Empty unless the state is
    /// [`Implementing`](State::Implementing): the other states make no claim
    /// about their bindings.
    #[must_use]
    pub fn gaps(&self) -> &[Gap] {
        &self.gaps
    }
}

/// A change record that loaded: the open change a status listing names.
///
/// The value is read from the record; it is not the record. Build the list
/// with [`Workplan::open_changes`](crate::Workplan::open_changes).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpenChange {
    slug: String,
    id: String,
    path: String,
    capabilities: Vec<String>,
}

impl OpenChange {
    /// The open changes of the loaded records, sorted by slug (then id and
    /// path), each with its capabilities sorted.
    pub(crate) fn build(changes: &[Loaded<Change>]) -> Vec<Self> {
        let mut open: Vec<Self> = changes
            .iter()
            .map(|loaded| {
                let change = loaded.value();
                let mut capabilities = change.changes.clone();
                capabilities.sort();
                Self {
                    slug: change.header.slug.clone(),
                    id: change.header.id.clone(),
                    path: loaded.path().to_string(),
                    capabilities,
                }
            })
            .collect();
        open.sort();
        open
    }

    /// The slug of the change, without the `change:` prefix.
    #[must_use]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// The change id, with its prefix: `change:<slug>`.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The repository-relative path of the record.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The ids of the capabilities the change lists, sorted.
    #[must_use]
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }
}

/// The order of the groups: the order of the `catalogue` block.
const ORDER: [State; 4] = [
    State::Delivered,
    State::Implementing,
    State::Designed,
    State::Considered,
];

/// Every capability grouped by state, computed once from the graph.
///
/// The groups are in the order of [`Roadmap::ORDER`] and each is sorted by
/// slug. The value holds no timestamp and is a pure function of the documents
/// and the source index.
///
/// A capability document that fails validation is not an entry; its findings
/// are reported by [`Workplan::check`](crate::Workplan::check), so a caller
/// that relies on the roadmap runs `check` too.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roadmap {
    /// Sorted by the position of the state in `ORDER`, then by slug.
    entries: Vec<RoadmapEntry>,
}

impl Roadmap {
    /// The states in the order the groups are listed: `delivered`,
    /// `implementing`, `designed`, `considered`.
    pub const ORDER: [State; 4] = ORDER;

    pub(crate) fn build(capabilities: &[Loaded<Capability>], graph: &Graph) -> Self {
        let mut entries: Vec<RoadmapEntry> = capabilities
            .iter()
            .map(|loaded| entry(loaded, graph))
            .collect();
        // The loader yields documents in file-name order, which includes the
        // `.md` suffix (`a-b.md` before `a.md`), so slug order is made here.
        entries.sort_by(|a, b| (rank(a.state), &a.slug).cmp(&(rank(b.state), &b.slug)));
        Self { entries }
    }

    /// Every entry: the groups in [`Roadmap::ORDER`], each sorted by slug.
    #[must_use]
    pub fn entries(&self) -> &[RoadmapEntry] {
        &self.entries
    }

    /// The number of capabilities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the repository has no capability.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entries in `state`, sorted by slug; empty when there are none.
    #[must_use]
    pub fn group(&self, state: State) -> &[RoadmapEntry] {
        let start = self
            .entries
            .partition_point(|e| rank(e.state) < rank(state));
        let end = self
            .entries
            .partition_point(|e| rank(e.state) <= rank(state));
        &self.entries[start..end]
    }

    /// The four groups, in [`Roadmap::ORDER`], each with its state. A state
    /// with no capability is present with an empty slice.
    #[must_use]
    pub fn groups(&self) -> [(State, &[RoadmapEntry]); 4] {
        ORDER.map(|state| (state, self.group(state)))
    }

    /// The entry of the capability with this slug.
    #[must_use]
    pub fn get(&self, slug: &str) -> Option<&RoadmapEntry> {
        self.entries.iter().find(|e| e.slug == slug)
    }

    /// The planned capabilities: the `designed` group followed by the
    /// `considered` group, each sorted by slug.
    ///
    /// This is a view over two of the four groups for readers that want
    /// "planned, implementing, delivered". It is not a fifth state, and
    /// [`Roadmap::entries`] and [`Roadmap::groups`] never contain it.
    #[must_use]
    pub fn planned(&self) -> Vec<&RoadmapEntry> {
        self.group(State::Designed)
            .iter()
            .chain(self.group(State::Considered))
            .collect()
    }
}

/// The position of a state in [`ORDER`].
fn rank(state: State) -> usize {
    match state {
        State::Delivered => 0,
        State::Implementing => 1,
        State::Designed => 2,
        State::Considered => 3,
    }
}

fn entry(loaded: &Loaded<Capability>, graph: &Graph) -> RoadmapEntry {
    let capability = loaded.value();
    let id = capability.header.id.clone();
    let mut open_changes: Vec<String> = graph
        .changes_edges()
        .iter()
        .filter(|edge| edge.capability() == id)
        .map(|edge| edge.change().to_string())
        .collect();
    open_changes.sort();
    open_changes.dedup();
    let gaps = if capability.state == State::Implementing {
        gaps(&id, graph)
    } else {
        Vec::new()
    };
    RoadmapEntry {
        slug: capability.header.slug.clone(),
        title: capability.header.title.clone(),
        description: capability.header.description.clone(),
        state: capability.state,
        path: loaded.path().to_string(),
        id,
        open_changes,
        gaps,
    }
}

/// The empty lists and the unresolved entries of a capability's bindings,
/// whatever its state: the roadmap keeps them for an implementing capability,
/// and `reconcile` reads them for a delivered one too.
pub(crate) fn gaps(id: &str, graph: &Graph) -> Vec<Gap> {
    let mut gaps = Vec::new();
    for list in [BindingKind::Code, BindingKind::Test, BindingKind::Example] {
        let mut entries = graph
            .bindings_of(id)
            .filter(|binding| binding.kind() == list)
            .peekable();
        if entries.peek().is_none() {
            gaps.push(Gap::EmptyList(list));
        }
        gaps.extend(
            entries
                .filter(|b| b.resolution().is_err())
                .map(|b| Gap::Unresolved {
                    list,
                    binding: b.path().to_string(),
                }),
        );
    }
    gaps
}
