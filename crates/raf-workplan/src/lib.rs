// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Static source and documentation contracts for a repository.
//!
//! [`Workplan::load`] reads capability, architecture and change documents and
//! their authored evidence, assessment, review, coverage and debt records through
//! a bounded [`raf_source::Tree`]. Its graph relates declarations to the source
//! index. [`Workplan::check`] reports typed findings without executing code.
//!
//! Tests and examples are declarations to resolve, never programs to run.
//! Binding validation checks existence, role and declared configuration. It does
//! not establish test success, macro expansion or compiler acceptance. Stored
//! evidence remains authored metadata; static reference and digest checks do not
//! turn its verdict into an observed execution result.
//!
//! [`Gate::workspace`] opens a disk workspace and reads tracked paths through a
//! native read-only Git repository. [`git`] and [`history`] read local objects,
//! references and history without helpers or fetching. Missing or unreadable
//! required data is an error. [`root::discover`] reads bounded ancestor manifests.
//!
//! [`lifecycle::Lifecycle`] reconciles changes and completes them through static
//! obligations, current assessments and a prospective in-memory document overlay.
//! Completion starts no process and creates no execution worktree. Its explicit
//! writes are owned generated documentation and removal of the completed record.
//!
//! [`record`], [`assessment`], [`review`], [`debt`] and [`Workplan::write`] expose
//! explicit data publication. Analysis never edits inspected source or Git state.
//! [`Workplan::preview`] and generated synchronization compute text before writes.
//!
//! [`diagram`] computes bounded diagram data and documentation artifacts. Reading
//! compiler or Rustdoc artifacts never starts a compiler or grants execution
//! authority. Explicit trusted documentation builds belong to the command-line
//! component, outside these analysis libraries.
//!
//! ```rust
//! use raf_source::{MemoryTree, Workspace};
//! use raf_workplan::Workplan;
//!
//! let tree = MemoryTree::from([
//!     ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\n"),
//!     ("src/lib.rs", "pub fn declared() {}"),
//! ]);
//! let source = Workspace::load(&tree, &["."])?;
//! let plan = Workplan::load(&tree, &source);
//! let findings = plan.check();
//! # let _ = findings;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod architecture;
pub mod assessment;
mod bindings;
pub mod coverage;
pub mod debt;
pub mod diagram;
pub mod digest;
pub mod escape;
pub mod evidence;
mod files;
mod finding;
mod gate;
mod generated;
pub mod git;
pub mod identifiers;

mod graph;
pub mod history;
pub mod lifecycle;
mod links;
mod load;
mod lookup;
mod model;
pub mod obligation;
pub mod plain;
mod preview;
mod rationale;
pub mod record;
pub mod refs;
pub mod report;
pub mod review;
mod roadmap;
pub mod root;
mod rules;
mod spdx;
mod staged;
pub mod strict;
mod sync;
#[cfg(test)]
mod unsigned;
mod work;

pub use bindings::ExampleKind;
pub use finding::Finding;
pub use gate::{Gate, GateError, Opened};
pub use graph::{
    AffectsEdge, AssessesEdge, Binding, BindingKind, ChangesEdge, ClaimEdge, CoversEdge,
    DependsEdge, Edge, EdgeKind, Graph, InputEdge, InputTarget, LinkEdge, LinkEnd, Node, NodeKind,
    Proposal, ScopeEdge, SectionId, TestedByEdge,
};
pub use links::LinkContext;
pub use lookup::ChangeLookupError;
pub use model::{Loaded, Workplan};
pub use preview::{Preview, PreviewError};
pub use roadmap::{Gap, OpenChange, Roadmap, RoadmapEntry};
pub use sync::{SyncError, SyncOptions, SyncReport};
