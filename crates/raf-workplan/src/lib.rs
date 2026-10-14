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

mod finding;
mod graph;
mod links;
mod load;
mod model;
pub mod plain;
pub mod refs;
mod rules;

pub use finding::Finding;
pub use graph::{
    AffectsEdge, Binding, BindingKind, ChangesEdge, DependsEdge, Edge, EdgeKind, Graph, LinkEdge,
    LinkEnd, Node, NodeKind, Proposal, SectionId,
};
pub use links::LinkContext;
pub use model::{Loaded, Workplan};
