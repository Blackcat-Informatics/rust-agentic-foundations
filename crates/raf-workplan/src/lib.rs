// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Loads the capability, change and architecture documents of a repository
//! into a typed model and reports what is wrong with them.
//!
//! [`Workplan::load`] reads `docs/capabilities`, `docs/architecture` and
//! `work/changes` through a [`Tree`](raf_source::Tree), parses and validates
//! each document with `raf-okf`, and keeps the typed documents and their bodies.
//! Everything that goes wrong is a [`Finding`] with a stable kind from
//! [`Finding::KINDS`], a location and a fix; nothing is skipped and nothing
//! panics. [`Workplan::check`] returns the findings.
//!
//! The rules between documents are part of `check`: a PROPOSED block names a
//! change that lists its capability and affects its section, each state fixes
//! the blocks and the open change records a capability may have, links resolve
//! against a set of tracked paths and the anchors of their targets
//! ([`LinkContext`]), and no document holds an issue or pull request reference
//! ([`refs`]). Markdown outside the document directories, such as the README,
//! is judged by the same rules through [`plain::check`].
//!
//! The load also builds the [`Graph`] of the documents and the source they are
//! bound to, available as [`Workplan::graph`], and reports the references that
//! name a document that does not exist and the `depends` cycles.
//!
//! Each entry of a capability's `bindings` is resolved through the source
//! index once, when the graph is built, and the result is kept on its
//! [`Binding`] ([`Graph::bindings_of`]). `check` judges it: a path that does
//! not resolve, or names more than one symbol, or that a macro generates, and
//! an item that does not fit its list (`code` a shipping item, `tests` a
//! `#[test]` function, `examples` an example target or a documented item of a
//! library whose documentation tests run, see [`ExampleKind`]).
//!
//! [`Workplan::status`] groups the capabilities by state into a [`Roadmap`]:
//! the open changes of each, and for an implementing capability the gaps in its
//! bindings. A document that failed validation is a finding, not an entry.
//!
//! [`Gate::workspace`] is the entry point for a test of a repository: it opens a
//! checkout on disk, loads its documents and its source, and
//! [`Gate::assert_clean`] panics with every finding. A repository without a
//! capability document is an error, not a clean verdict.
//!
//! # Generated blocks and sync
//!
//! Three kinds of generated block are owned here: `verification-matrix`
//! (`Item | Kind | Section`, one row per `tests` and `examples` binding, sorted
//! by path), `relationships` (`Relation | Capability`: depends on, depended on
//! by, changed by) and `catalogue` (one `## <State>` table per state, in the
//! order delivered, implementing, designed, considered, with the gaps of an
//! implementing capability). `verification-matrix` and `relationships` sit in
//! a section of a capability document; `catalogue` sits in
//! `docs/capabilities/index.md`, the only index, which holds nothing but its
//! SPDX comment and generated blocks.
//!
//! [`Workplan::sync`] returns the whole new text of every stale or missing
//! file as a [`SyncReport`], without writing; [`Workplan::write`] puts a report
//! on a disk through a temporary file and a rename. `check` judges staleness
//! with the renderers that sync writes with (`generated-stale`), so a clean
//! `check` is a sync with nothing to write. A missing index is `index-missing`,
//! and sync creates it with the SPDX text of [`SyncOptions`].
//!
//! Text that comes from a document and is written into a table cell is
//! escaped by one rule: `\` becomes `\\`, `|` becomes `\|`, `[` and `]`
//! become `\[` and `\]`, `<` becomes `&lt;` (so a description can never open
//! a comment or forge a marker), and each line break becomes one space.
//!
//! A block is written in the line ending of the body it sits in, read from the
//! first line break of that body, so the frontmatter's ending does not matter.
//! A body that mixes CRLF and LF is `mixed-line-endings` and sync refuses it.
//! The index is created with LF line endings. A sync stops with
//! [`SyncError::Unloaded`] while a capability or change document does not load,
//! since the tables that list it would be wrong.
//!
//! ```rust
//! use raf_source::{MemoryTree, Workspace};
//! use raf_workplan::Workplan;
//!
//! let docs = MemoryTree::from([(
//!     "docs/architecture/overview.md",
//!     "---\ntype: Architecture\nid: arch:overview\ntitle: Overview\n\
//!      description: How it fits.\n---\n\n# Overview\n\nBody text.\n",
//! )]);
//! let source = Workspace::load(
//!     &MemoryTree::from([
//!         ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
//!         ("src/lib.rs", ""),
//!     ]),
//!     &["."],
//! )?;
//! let plan = Workplan::load(&docs, &source);
//! assert!(plan.check().is_empty());
//! assert_eq!(plan.architecture().len(), 1);
//!
//! // A page that is not a document is a finding, not a skipped file.
//! let docs = MemoryTree::from([("docs/architecture/notes.md", "# Notes\n")]);
//! let plan = Workplan::load(&docs, &source);
//! assert_eq!(plan.check()[0].kind, "stray-markdown");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod bindings;
mod finding;
mod gate;
mod generated;
mod graph;
mod links;
mod load;
mod model;
pub mod plain;
pub mod refs;
mod roadmap;
mod rules;
mod sync;

pub use bindings::ExampleKind;
pub use finding::Finding;
pub use gate::{Gate, GateError};
pub use graph::{
    AffectsEdge, Binding, BindingKind, ChangesEdge, DependsEdge, Edge, EdgeKind, Graph, LinkEdge,
    LinkEnd, Node, NodeKind, Proposal, SectionId,
};
pub use links::LinkContext;
pub use model::{Loaded, Workplan};
pub use roadmap::{Gap, Roadmap, RoadmapEntry};
pub use sync::{SyncError, SyncOptions, SyncReport};
