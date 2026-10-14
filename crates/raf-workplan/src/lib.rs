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
mod load;
mod model;

pub use finding::Finding;
pub use model::{Loaded, Workplan};
