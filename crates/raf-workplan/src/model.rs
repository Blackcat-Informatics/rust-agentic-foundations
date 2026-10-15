// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The model of a repository's workplan documents.

use std::collections::BTreeSet;

use raf_okf::{Architecture, Body, Capability, Change, DocType, Index};
use raf_source::Workspace;

use crate::Finding;
use crate::bindings;
use crate::generated;
use crate::graph::{self, BindingKind, Graph};
use crate::links::LinkContext;
use crate::load;
use crate::roadmap::{OpenChange, Roadmap};
use crate::rules;

/// A document that satisfies the profile: where it is, what it says in its
/// frontmatter, and its body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded<T> {
    path: String,
    value: T,
    body: Body,
    /// The text before the body: both frontmatter fences and the frontmatter,
    /// byte for byte, so a rewritten body can be joined to it unchanged.
    head: String,
}

impl<T> Loaded<T> {
    /// A loaded document. `text` is the whole file and `body` is read from it,
    /// so the body starts where its span says.
    pub(crate) fn new(path: String, value: T, body: Body, text: &str) -> Self {
        let head = text[..body.span().bytes.start].to_string();
        Self {
            path,
            value,
            body,
            head,
        }
    }

    /// The file text up to the body.
    pub(crate) fn head(&self) -> &str {
        &self.head
    }

    /// The repository-relative path of the file, with `/` separators.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The typed frontmatter.
    #[must_use]
    pub fn value(&self) -> &T {
        &self.value
    }

    /// The body.
    #[must_use]
    pub fn body(&self) -> &Body {
        &self.body
    }
}

/// What the load found at `docs/capabilities/index.md`.
#[derive(Debug, Clone)]
pub(crate) enum IndexSlot {
    /// There is no such file.
    Absent,
    /// The file reads as a generated index.
    Valid(Index),
    /// The file is there and cannot be used: it did not read, or it is
    /// hand-written. The finding says why.
    Invalid(Finding),
}

/// The capability, change and architecture documents of a repository, loaded
/// through a [`Tree`](raf_source::Tree), with every finding the load made.
///
/// A document that breaks the profile is not part of the model; its findings
/// are. A file the tree refuses to give is a finding that names it.
#[derive(Debug)]
pub struct Workplan<'s> {
    pub(crate) capabilities: Vec<Loaded<Capability>>,
    pub(crate) changes: Vec<Loaded<Change>>,
    pub(crate) architecture: Vec<Loaded<Architecture>>,
    pub(crate) findings: Vec<Finding>,
    /// The ids of the documents that exist as files, loaded or not.
    present: BTreeSet<String>,
    /// The PROPOSED blocks of capability documents that failed validation:
    /// file, change slug and line.
    pub(crate) unvalidated_blocks: Vec<(String, String, usize)>,
    /// The capability documents that did not load only because a delivered
    /// capability binds nothing in a list it must bind: file and lists.
    pub(crate) unbound: Vec<(String, Vec<BindingKind>)>,
    pub(crate) index: IndexSlot,
    graph: Graph,
    source: &'s Workspace,
}

impl<'s> Workplan<'s> {
    /// Load the documents under `docs/capabilities`, `docs/architecture` and
    /// `work/changes` of `docs`, and relate them to the source index `source`.
    ///
    /// The load never fails and never panics: every problem is a [`Finding`].
    /// Each directory holds its documents directly. A typed document in the
    /// directory of another type is `wrong-directory`, a Markdown file below a
    /// document directory is `nested-directory`, a Markdown file other than
    /// `index.md` that has no frontmatter is `stray-markdown`, and a file or a
    /// directory the tree refuses is `unreadable-file` or
    /// `unreadable-directory`. A directory that does not exist holds nothing.
    ///
    /// Links are judged against every file `docs` lists, which is the set of
    /// tracked paths only for a tree that holds nothing else. On a disk tree
    /// the listing includes untracked and ignored files, build output and
    /// `.git`, so a link to an untracked file passes; a caller that checks a
    /// working copy passes its tracked list through [`Workplan::load_with`]
    /// and [`LinkContext::new`]. See [`LinkContext::of_tree`].
    #[must_use]
    pub fn load(docs: &dyn raf_source::Tree, source: &'s Workspace) -> Self {
        load::load(docs, source, &LinkContext::of_tree(docs))
    }

    /// As [`Workplan::load`], with the links of the documents judged against
    /// `links`: the tracked paths, the tracked symlinks, and the tree the
    /// anchors are read from.
    #[must_use]
    pub fn load_with(
        docs: &dyn raf_source::Tree,
        source: &'s Workspace,
        links: &LinkContext<'_>,
    ) -> Self {
        load::load(docs, source, links)
    }

    pub(crate) fn empty(source: &'s Workspace) -> Self {
        Self {
            capabilities: Vec::new(),
            changes: Vec::new(),
            architecture: Vec::new(),
            findings: Vec::new(),
            present: BTreeSet::new(),
            unvalidated_blocks: Vec::new(),
            unbound: Vec::new(),
            index: IndexSlot::Absent,
            graph: Graph::default(),
            source,
        }
    }

    /// Record that the file at `path` exists as a document of `doc_type`.
    pub(crate) fn note_present(&mut self, doc_type: DocType, path: &str) {
        let name = path.rsplit('/').next().unwrap_or(path);
        let stem = name.strip_suffix(".md").unwrap_or(name);
        self.present
            .insert(format!("{}{stem}", doc_type.id_prefix()));
    }

    /// Whether the repository has a capability document, loaded or not: the
    /// capability index is required exactly then.
    pub(crate) fn has_capability_files(&self) -> bool {
        self.present.iter().any(|id| id.starts_with("cap:"))
    }

    /// The ids of the capability and change documents that exist as files and
    /// did not load, sorted. Their findings are in `check`.
    pub(crate) fn unloaded(&self) -> Vec<String> {
        let loaded: BTreeSet<&str> = self
            .capabilities
            .iter()
            .map(|cap| cap.value().header.id.as_str())
            .chain(
                self.changes
                    .iter()
                    .map(|change| change.value().header.id.as_str()),
            )
            .collect();
        self.present
            .iter()
            .filter(|id| id.starts_with("cap:") || id.starts_with("change:"))
            .filter(|id| !loaded.contains(id.as_str()))
            .cloned()
            .collect()
    }

    /// Build the graph of the loaded documents and add the findings of the
    /// rules between documents.
    pub(crate) fn relate(&mut self) {
        let (graph, findings) = graph::build(
            &self.capabilities,
            &self.changes,
            &self.architecture,
            &self.present,
            self.source,
        );
        self.findings.extend(findings);
        self.findings
            .extend(bindings::check(&graph, &self.capabilities, self.source));
        self.graph = graph;
        self.findings.extend(rules::check(
            &self.capabilities,
            &self.changes,
            &self.present,
            &self.unvalidated_blocks,
        ));
    }

    /// The graph of the documents and the source they are bound to.
    #[must_use]
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// The source index the documents are related to.
    #[must_use]
    pub fn source(&self) -> &'s Workspace {
        self.source
    }

    /// The capability documents, in directory order.
    #[must_use]
    pub fn capabilities(&self) -> &[Loaded<Capability>] {
        &self.capabilities
    }

    /// The change records, in directory order.
    #[must_use]
    pub fn changes(&self) -> &[Loaded<Change>] {
        &self.changes
    }

    /// The architecture pages, in directory order.
    #[must_use]
    pub fn architecture(&self) -> &[Loaded<Architecture>] {
        &self.architecture
    }

    /// The findings the load made.
    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    /// The roadmap: every capability grouped by state, with its open changes
    /// and, for an implementing capability, the gaps in its bindings.
    ///
    /// The value is computed from the graph on each call and is a pure
    /// function of the loaded documents and the source index. Its capabilities
    /// are those that loaded; a document that failed validation is a finding
    /// of [`Workplan::check`], not an entry.
    #[must_use]
    pub fn status(&self) -> Roadmap {
        Roadmap::build(&self.capabilities, &self.graph)
    }

    /// The change records that loaded, sorted by slug, each with the
    /// capabilities it lists. A record that did not load is a finding of
    /// [`Workplan::check`], not an entry.
    #[must_use]
    pub fn open_changes(&self) -> Vec<OpenChange> {
        OpenChange::build(&self.changes)
    }

    /// Every finding of every check over the loaded documents: the findings
    /// of the load, then those of the generated blocks (`index-missing`,
    /// `generated-stale`, `generated-unknown-kind`, `generated-misplaced` and
    /// `mixed-line-endings`), which are judged by the renderers that
    /// [`Workplan::sync`] writes with, so a clean `check` is a `sync` with
    /// nothing to write.
    #[must_use]
    pub fn check(&self) -> Vec<Finding> {
        let mut findings = self.findings.clone();
        findings.extend(generated::analyze(self).into_findings());
        findings
    }

    /// The kinds of generated block `raf-workplan` owns, in the order they are
    /// documented: `verification-matrix` and `relationships` for a capability
    /// document, `catalogue` for `docs/capabilities/index.md`.
    pub const GENERATED_KINDS: [&'static str; 3] = generated::KINDS;
}
