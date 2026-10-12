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
//! [`parse_source`] parses Rust text under bounds. A token pre-scan refuses a file
//! that passes a recursion [`Budget`] before `syn` sees it, and the parse and a
//! caller's visitor then run on a thread named after the file, with a stack of
//! [`PARSE_STACK_BYTES`]:
//!
//! ```rust
//! use raf_source::{Budget, LoadErrorKind, RelPath, parse_source};
//!
//! let file = RelPath::new("src/lib.rs")?;
//! let items = parse_source(&file, "pub fn go() {}\n", |tree| tree.items.len())?;
//! assert_eq!(items, 1);
//!
//! let deep = format!("type T = {}u8{};", "A<".repeat(65), ">".repeat(65));
//! let refused = parse_source(&file, &deep, |_| ()).unwrap_err();
//! assert!(matches!(
//!     refused.kind,
//!     LoadErrorKind::Budget { budget: Budget::Angles, limit: 64 }
//! ));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Only [`TreeError::NotFound`] means that nothing is there. Every other
//! error is a refusal or a failure and is never read as absence.

mod budget;
mod error;
mod options;
mod parse;
mod path;
mod prescan;
mod tree;

pub use budget::Budget;
pub use error::{LoadError, LoadErrorKind, Origin, TreeError};
pub use options::{DEFAULT_MAX_FILE_BYTES, LoadOptions};
pub use parse::{PARSE_STACK_BYTES, parse_source};
pub use path::{PathError, RelPath};
pub use syn;
pub use tree::{DiskTree, Entry, EntryKind, MemoryTree, Tree};
