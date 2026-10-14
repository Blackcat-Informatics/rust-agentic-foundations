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
//! capability document is an error, not a clean verdict. [`Gate::open`] is that
//! load on its own: it returns an [`Opened`] repository (the tree, the source
//! workspace and the tracked paths), whose [`Opened::workplan`] loads the
//! documents as they are on disk at the call. Every caller that reads a
//! repository goes through it, and [`Gate::programs`] names the programs it
//! starts.
//!
//! [`Workplan::change`] finds the change record a slug names, and says whether
//! the slug is unknown or its record did not load ([`ChangeLookupError`]).
//!
//! [`lifecycle`] opens and reads a change. [`Workplan::new_change`] writes the
//! record `work/changes/<slug>.md` of a new change, exclusively and as the only
//! file it writes, validates the slug with the profile's slug pattern, checks
//! that every listed capability is loaded and every `affects` entry names a
//! listed capability and one of the seven sections, reads the record back
//! through the loader, and returns a [`lifecycle::NewChange`]: the PROPOSED
//! block template to place and, for each listed capability, whether to set
//! `state: implementing` in the same commit (`delivered` and `designed`) or
//! nothing (`considered` and `implementing`). [`Workplan::impact`] returns one
//! [`lifecycle::Impact`]: the capabilities and sections of a change, their
//! bindings, the capabilities that depend on them and the sections that link
//! to the affected sections, in a fixed order and without writing anything.
//!
//! # Processes
//!
//! [`exec`] is the one place a `cargo` or `git` process is started. A process is
//! an [`exec::Invocation`] of fixed text and values validated by the patterns of
//! the completion gate, started with no shell, null standard input, bounded
//! output and a wall-clock kill by an [`exec::Runner`]; [`exec::FakeRunner`]
//! answers in process for tests.
//!
//! [`git`] asks `git` its questions through a runner: the version gate
//! ([`git::require_version`] refuses a Git older than 2.30), the tracked paths
//! of a checkout, and the four commands that read a file as it was at the merge
//! base of the current branch and the default branch. [`root::discover`] finds
//! the workspace root with `cargo metadata --format-version 1 --no-deps
//! --frozen`, reading nothing but its `workspace_root` field.
//!
//! [`bound`] runs what a capability binds: the tests, documentation tests and
//! examples of `tests` and `examples`, through a runner and only the argv forms of the
//! completion gate. An [`bound::Executor`] decides from the source index what a
//! [`Binding`] runs as (refusing a function that is not a test, a target without the
//! libtest harness, a target with required features, and any value that fails its
//! pattern, before a process starts), counts a test as passed only on the proof its
//! kind requires, and returns an [`bound::Evidence`] without writing anything.
//!
//! # Preview
//!
//! [`Workplan::preview`] returns each capability document a change lists as it
//! reads once that change is reconciled: the PROPOSED blocks of the change are
//! replaced by their content and nothing else moves. It returns text and
//! writes nothing; see [`Preview`] and [`PreviewError`].
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
pub mod bound;
pub mod exec;
mod finding;
mod gate;
mod generated;
pub mod git;
mod graph;
pub mod lifecycle;
mod links;
mod load;
mod lookup;
mod model;
pub mod plain;
mod preview;
pub mod refs;
mod roadmap;
pub mod root;
mod rules;
mod sync;

pub use bindings::ExampleKind;
pub use finding::Finding;
pub use gate::{Gate, GateError, Opened};
pub use graph::{
    AffectsEdge, Binding, BindingKind, ChangesEdge, DependsEdge, Edge, EdgeKind, Graph, LinkEdge,
    LinkEnd, Node, NodeKind, Proposal, SectionId,
};
pub use links::LinkContext;
pub use lookup::ChangeLookupError;
pub use model::{Loaded, Workplan};
pub use preview::{Preview, PreviewError};
pub use roadmap::{Gap, Roadmap, RoadmapEntry};
pub use sync::{SyncError, SyncOptions, SyncReport};
