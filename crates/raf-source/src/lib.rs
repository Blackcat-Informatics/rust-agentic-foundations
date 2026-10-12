// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The workspace source index: a [`Tree`] seam with a disk and a memory
//! backend, under canonical path containment.
//!
//! Every file the crate looks at goes through [`Tree`]. [`DiskTree`] reads a
//! directory on disk and refuses anything that leads out of its root, or is
//! not a regular file, or is larger than the caller allows. [`MemoryTree`]
//! holds the same contents in a map so that a fixture needs no disk:
//!
//! ```rust
//! use raf_source::{EntryKind, MemoryTree, RelPath, Tree, TreeError};
//!
//! let tree = MemoryTree::from([("src/lib.rs", "pub fn go() {}\n")]);
//! let lib = RelPath::new("src/lib.rs")?;
//! assert_eq!(tree.read(&lib, 1024)?, "pub fn go() {}\n");
//! assert_eq!(tree.entry(&RelPath::new("src")?)?, Some(EntryKind::Dir));
//! assert!(matches!(
//!     tree.read(&RelPath::new("src/missing.rs")?, 1024),
//!     Err(TreeError::NotFound { .. })
//! ));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Only [`TreeError::NotFound`] means that nothing is there. Every other
//! error is a refusal or a failure and is never read as absence.

mod error;
mod options;
mod path;
mod tree;

pub use error::TreeError;
pub use options::{DEFAULT_MAX_FILE_BYTES, LoadOptions};
pub use path::{PathError, RelPath};
pub use tree::{DiskTree, Entry, EntryKind, MemoryTree, Tree};
