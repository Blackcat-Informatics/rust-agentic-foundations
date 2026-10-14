// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The model of a repository's workplan documents.

use std::collections::BTreeSet;

use raf_okf::{Architecture, Body, Capability, Change, DocType};
use raf_source::Workspace;

use crate::Finding;
use crate::graph::{self, Graph};
use crate::links::LinkContext;
use crate::load;
use crate::rules;

/// A document that satisfies the profile: where it is, what it says in its
/// frontmatter, and its body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded<T> {
    path: String,
    value: T,
    body: Body,
}

impl<T> Loaded<T> {
    pub(crate) fn new(path: String, value: T, body: Body) -> Self {
        Self { path, value, body }
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
        self.graph = graph;
        self.findings.extend(findings);
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

    /// Every finding of every check over the loaded documents.
    #[must_use]
    pub fn check(&self) -> Vec<Finding> {
        self.findings.clone()
    }
}
