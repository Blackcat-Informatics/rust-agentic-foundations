// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The model of a repository's workplan documents.

use std::collections::BTreeSet;

use raf_okf::{
    Architecture, AssessmentRecord, Body, Capability, Change, CoverageRecord, DebtRecord, DocType,
    EvidenceRecord, Index,
};
use raf_source::Workspace;

use crate::Finding;
use crate::bindings;
use crate::coverage;
use crate::generated;
use crate::graph::{self, BindingKind, Graph};
use crate::links::LinkContext;
use crate::load;
use crate::rationale;
use crate::roadmap::{OpenChange, Roadmap};
use crate::rules;
use crate::strict;

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

    /// The whole text of the file as it was read: the frontmatter and the
    /// body, byte for byte.
    pub(crate) fn text(&self) -> String {
        format!("{}{}", self.head, self.body.text())
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

/// The capability, change and architecture documents of a repository and its
/// evidence, assessment and coverage records, loaded through a
/// [`Tree`](raf_source::Tree), with every finding the load made.
///
/// A document that breaks the profile is not part of the model; its findings
/// are. A file the tree refuses to give is a finding that names it.
#[derive(Debug)]
pub struct Workplan<'s> {
    pub(crate) capabilities: Vec<Loaded<Capability>>,
    pub(crate) changes: Vec<Loaded<Change>>,
    pub(crate) architecture: Vec<Loaded<Architecture>>,
    pub(crate) evidence: Vec<Loaded<EvidenceRecord>>,
    pub(crate) assessments: Vec<Loaded<AssessmentRecord>>,
    /// The coverage record: only `work/coverage/default.md` is read, so there
    /// is at most one.
    pub(crate) coverage: Option<Loaded<CoverageRecord>>,
    /// The debt record: only `work/debt/default.md` is read, so there is at
    /// most one.
    pub(crate) debt: Option<Loaded<DebtRecord>>,
    pub(crate) findings: Vec<Finding>,
    // Native loading, graph and binding findings precede delivery-state rules.
    // Reconciled previews do not mutate states merely to render their graph.
    pub(crate) semantic_findings: usize,
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
    pub(crate) diagram_assets: crate::diagram::assets::Stored,
}

impl<'s> Workplan<'s> {
    /// Load the documents under `docs/capabilities`, `docs/architecture` and
    /// `work/changes` of `docs` and the records under `work/evidence`,
    /// `work/assessments`, `work/coverage` and `work/debt`, and relate them to
    /// the source index `source`.
    ///
    /// The load never fails and never panics: every problem is a [`Finding`].
    /// Each directory holds its documents directly. A typed document in the
    /// directory of another type is `wrong-directory`, a Markdown file below a
    /// document directory is `nested-directory`, a Markdown file other than
    /// `index.md` that has no frontmatter is `stray-markdown`, and a file or a
    /// directory the tree refuses is `unreadable-file` or
    /// `unreadable-directory`. `work/coverage` holds the one coverage record,
    /// `default.md`, and `work/debt` the one debt record, `default.md`; any
    /// other file there is `singleton-other-file` and is not read. A directory that does not exist holds nothing, so a
    /// repository without records loads as it did before they existed.
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
            evidence: Vec::new(),
            assessments: Vec::new(),
            coverage: None,
            debt: None,
            findings: Vec::new(),
            semantic_findings: 0,
            present: BTreeSet::new(),
            unvalidated_blocks: Vec::new(),
            unbound: Vec::new(),
            index: IndexSlot::Absent,
            graph: Graph::default(),
            source,
            diagram_assets: crate::diagram::assets::Stored::default(),
        }
    }

    /// Record that the file at `path` exists as a document of `doc_type`.
    pub(crate) fn note_present(&mut self, doc_type: DocType, path: &str) {
        let name = path.rsplit('/').next().unwrap_or(path);
        let stem = name.strip_suffix(".md").unwrap_or(name);
        self.present
            .insert(format!("{}{stem}", doc_type.id_prefix()));
    }

    /// Whether a file of the document directories exists under the id, loaded
    /// or not.
    pub(crate) fn is_present(&self, id: &str) -> bool {
        self.present.contains(id)
    }

    /// Whether the repository has a capability document, loaded or not: the
    /// capability index is required exactly then.
    pub(crate) fn has_capability_files(&self) -> bool {
        self.present.iter().any(|id| id.starts_with("cap:"))
    }

    /// The ids of capability, change and architecture documents that exist as files and
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
            .chain(
                self.architecture
                    .iter()
                    .map(|page| page.value().header.id.as_str()),
            )
            .collect();
        self.present
            .iter()
            .filter(|id| {
                id.starts_with("cap:") || id.starts_with("change:") || id.starts_with("arch:")
            })
            .filter(|id| !loaded.contains(id.as_str()))
            .cloned()
            .collect()
    }

    /// Whether a capability, change or architecture document exists as a file
    /// and did not load: its links are not in the graph, and its findings say
    /// what is wrong with it.
    ///
    /// A file is noted under the id of its type's prefix and its file stem
    /// ([`Self::note_present`]), and that id is compared with the header ids of
    /// the loaded documents. The comparison is exact because the profile rule
    /// `okf-id-file-name` refuses a document whose id is not its file stem: a
    /// loaded document is always noted under its own id, and a file whose
    /// header names another id does not load, so it counts as unloaded.
    pub(crate) fn has_unloaded_documents(&self) -> bool {
        let loaded: BTreeSet<&str> = self
            .capabilities
            .iter()
            .map(|cap| cap.value().header.id.as_str())
            .chain(self.changes.iter().map(|c| c.value().header.id.as_str()))
            .chain(
                self.architecture
                    .iter()
                    .map(|a| a.value().header.id.as_str()),
            )
            .collect();
        self.present.iter().any(|id| {
            ["cap:", "change:", "arch:"]
                .iter()
                .any(|prefix| id.starts_with(prefix))
                && !loaded.contains(id.as_str())
        })
    }

    /// Build the graph of the loaded documents and add the findings of the
    /// rules between documents.
    pub(crate) fn relate(&mut self) {
        let (graph, findings) = graph::build(
            &self.capabilities,
            &self.changes,
            &self.architecture,
            &graph::Records {
                evidence: &self.evidence,
                assessments: &self.assessments,
                coverage: self.coverage.as_ref(),
            },
            &self.present,
            self.source,
        );
        self.findings.extend(findings);
        self.findings
            .extend(bindings::check(&graph, &self.capabilities, self.source));
        self.graph = graph;
        self.semantic_findings = self.findings.len();
        self.findings.extend(rules::check(
            &self.capabilities,
            &self.changes,
            &self.present,
            &self.unvalidated_blocks,
        ));
        let coverage = coverage::check(self);
        self.findings.extend(coverage);
        let rationale = rationale::check(self);
        self.findings.extend(rationale);
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

    /// The evidence records of `work/evidence`, in directory order.
    #[must_use]
    pub fn evidence(&self) -> &[Loaded<EvidenceRecord>] {
        &self.evidence
    }

    /// The assessment records of `work/assessments`, in directory order.
    #[must_use]
    pub fn assessments(&self) -> &[Loaded<AssessmentRecord>] {
        &self.assessments
    }

    /// The coverage record `work/coverage/default.md`, when it is there and
    /// loaded.
    #[must_use]
    pub fn coverage(&self) -> Option<&Loaded<CoverageRecord>> {
        self.coverage.as_ref()
    }

    /// The debt record `work/debt/default.md`, when it is there and loaded.
    #[must_use]
    pub fn debt(&self) -> Option<&Loaded<DebtRecord>> {
        self.debt.as_ref()
    }

    /// The findings of loading and relating documents. Complete policy
    /// findings are judged by [`Workplan::check`], never by the load.
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
    ///
    /// Coverage, prose, claims and debt are judged on every check.
    ///
    /// ```rust
    /// use raf_source::{MemoryTree, Workspace};
    /// use raf_workplan::Workplan;
    ///
    /// let source = Workspace::load(&MemoryTree::from([
    ///     ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ///     ("src/lib.rs", ""),
    /// ]), &["."])?;
    /// let record = "---\ntype: Coverage\nid: coverage:default\nscope: []\nexclusions: []\n---\n";
    /// let docs = MemoryTree::from([("work/coverage/default.md", record)]);
    /// assert_eq!(Workplan::load(&docs, &source).check(), []);
    ///
    /// let removed_key = record.replace("scope:", "strictness: default\nscope:");
    /// let docs = MemoryTree::from([("work/coverage/default.md", removed_key.as_str())]);
    /// let found = Workplan::load(&docs, &source).check();
    /// assert_eq!(found.len(), 1);
    /// assert_eq!(found[0].kind, "okf-unknown-key");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn check(&self) -> Vec<Finding> {
        let mut findings = self.base_findings();
        findings.extend(strict::check(self));
        findings
    }

    /// The findings of the load and of the generated blocks: the findings of
    /// document loading and generated-block consistency.
    pub(crate) fn base_findings(&self) -> Vec<Finding> {
        let mut findings = self.findings.clone();
        findings.extend(generated::analyze(self).into_findings());
        findings
    }

    /// The kinds of generated block `raf-workplan` owns, in the order they are
    /// documented: `verification-matrix` and `relationships` for a capability
    /// document, `catalogue` for `docs/capabilities/index.md`.
    pub const GENERATED_KINDS: [&'static str; 3] = generated::KINDS;
}
