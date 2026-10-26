// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The workspace source index: packages and targets, the module tree of each
//! target, a symbol index and a `use`-alias resolver, over a [`Tree`] seam with
//! a disk and a memory backend and under canonical path containment.
//!
//! Load a workspace from a real directory and ask where a path is defined. The
//! block needs a real tree, so it is compiled and not run:
//!
//! ```rust,no_run
//! use raf_source::{DiskTree, Workspace};
//!
//! let ws = Workspace::load(&DiskTree::new(".")?, &["crates"])?;
//! let resolution = ws.resolve("raf_okf::Document")?;
//! for site in &resolution.sites {
//!     let item = ws.item(*site).expect("a site is an item of this workspace");
//!     println!("{} at {}:{}", item.path, item.file, item.name_line);
//! }
//! for target in ws.targets().filter(|t| !t.shipping) {
//!     println!("{} is a non-shipping target", target.name);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [`Workspace::exported_items`] lists the public surface of a library or a
//! binary: the items another crate can name through its crate root, each under
//! the path it is exported by and with the item that defines it, and the public
//! re-exports of other crates, which are not items of the target. Over a real
//! tree, each library's surface maps every exported path back to its
//! definition. The block needs a real tree, so it is compiled and not run:
//!
//! ```rust,no_run
//! use raf_source::{DiskTree, TargetKind, Workspace};
//!
//! let ws = Workspace::load(&DiskTree::new(".")?, &["crates"])?;
//! for id in ws.target_ids() {
//!     let target = ws.target(id).expect("an id of this workspace");
//!     if target.kind != TargetKind::Lib {
//!         continue;
//!     }
//!     let exports = ws.exported_items(id)?;
//!     for export in &exports.items {
//!         let item = ws.item(export.item).expect("an item of this workspace");
//!         println!("{} is {} in {}", export.path, item.path, export.file);
//!     }
//!     for reexport in &exports.reexports {
//!         println!("{} re-exports {}", reexport.path, reexport.written);
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! With [`MemoryTree`] a fixture needs no disk:
//!
//! ```rust
//! use raf_source::{MemoryTree, Workspace};
//!
//! let tree = MemoryTree::from([
//!     ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
//!     ("src/lib.rs", "mod inner;\npub use inner::go as run;\n"),
//!     ("src/inner.rs", "pub fn go() {}\n"),
//! ]);
//! let ws = Workspace::load(&tree, &["."])?;
//! assert_eq!(ws.resolve("demo::run")?.path, "demo::inner::go");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! A `pub` item of a private module is exported only through a `pub` re-export:
//!
//! ```rust
//! use raf_source::{MemoryTree, Workspace};
//!
//! let tree = MemoryTree::from([
//!     ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
//!     ("src/lib.rs", "mod inner;\npub use inner::go as run;\npub(crate) fn helper() {}\n"),
//!     ("src/inner.rs", "pub fn go() {}\npub fn unused() {}\n"),
//! ]);
//! let ws = Workspace::load(&tree, &["."])?;
//! let lib = ws.target_ids().next().expect("the library");
//! let exports = ws.exported_items(lib)?;
//! assert_eq!(exports.items.len(), 1);
//! assert_eq!(exports.items[0].path, "demo::run");
//! assert_eq!(
//!     ws.item(exports.items[0].item).map(|item| item.path.as_str()),
//!     Some("demo::inner::go")
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
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
//! [`Workspace::load`] takes explicit roots, finds the packages and targets under
//! them, and follows every target's `mod` declarations as rustc does:
//!
//! ```rust
//! use raf_source::{MemoryTree, TargetKind, Workspace};
//!
//! let tree = MemoryTree::from([
//!     ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
//!     ("src/lib.rs", "mod inner;\n"),
//!     ("src/inner.rs", "pub fn go() {}\n"),
//!     ("tests/it.rs", "fn case() {}\n"),
//! ]);
//! let ws = Workspace::load(&tree, &["."])?;
//! let targets: Vec<_> = ws.targets().map(|t| (t.kind, t.crate_path.as_str())).collect();
//! assert_eq!(
//!     targets,
//!     [(TargetKind::Lib, "demo"), (TargetKind::Test, "demo::test_crate::it")]
//! );
//! assert_eq!(ws.files().count(), 3);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Only [`TreeError::NotFound`] means that nothing is there. Every other
//! error is a refusal or a failure and is never read as absence.

mod budget;
mod cfg;
mod configuration;
pub use configuration::{
    ActiveConfiguration, CfgAtom, CfgSelection, DeclaredCfg, DeclaredConfiguration,
};
mod code;
mod cost;
mod discovery;
mod error;
mod exports;
mod facts;
mod glob;
mod glob_scope;
mod impls;
mod index;
mod intern;
mod item;
mod locked;
mod manifest;
mod members;
mod meter;
mod modtree;
mod options;
mod parse;
mod path;
mod platform;
mod prescan;
mod reach;
mod resolve;
mod root;
mod settle;
mod target;
mod tree;
mod verbatim;
mod work;
mod workspace;

pub use budget::{Budget, PARSE_BYTES_PER_TOKEN, RECURSION_UNIT_BYTES};
pub use code::{CodeResolveError, CodeResolver, ExternLibrary, ExternRouteKind};
pub use error::{LoadError, LoadErrorKind, Origin, TargetProblem, TreeError};
pub use exports::{ExportError, ExportOptions, ExportedItem, Exports, Reexport};
pub use facts::visit_pattern_bindings;
pub use facts::{FieldShape, SpanFact, TestRole, Vis};
pub use item::{
    ImplBlock, ImplId, Item, ItemId, Namespace, SelfType, SharedStr, SymbolKind, TraitKey,
    UnresolvedImpl,
};
pub use locked::{
    LockedCatalog, LockedGitReference, LockedPackage, LockedPackageId, LockedSource,
    LockedSourceKind, RegistryIndex,
};
pub use manifest::{DependencyDeclaration, DependencyKind, DependencySpec, InheritedDependency};
pub use meter::Usage;
pub use modtree::{Reach, TargetId};
pub use options::{DEFAULT_MAX_FILE_BYTES, LoadOptions};
pub use parse::{PARSE_STACK_BYTES, parse_source};
pub use path::{PathError, RelPath, SourceAuthorityId};
pub use platform::{CompilerPlatform, CompilerPlatformError, TestPlatformGuard};
pub use reach::{Edge, Node, ReachError, Reachability, Reached};
pub use resolve::{Resolution, ResolveError};
pub use root::discover_root;
/// The `syn` that produced the facts, re-exported so a consumer parses with the
/// same version; a `syn` major bump is a breaking change of this crate.
pub use syn;
pub use target::{Package, PackageId, Target, TargetKind};
pub use tree::{
    CommittedGitTree, DiskTree, Entry, EntryKind, FileId, MemoryTree, Opened, OpenedBytes,
    SourceTrees, Tree,
};
pub use workspace::{
    GitPackageRoots, LockedBinding, LockedEdge, LockedManifest, SourceFile, TestFn, Workspace,
};
