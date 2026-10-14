// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The input of the `load-workspace` fuzz target: how bytes become a
//! workspace in memory and what is asked of the loaded index. The fuzz target
//! and the test that replays its seeds share this file, so the seeds are
//! judged through the same code the fuzzer runs.
//!
//! The input is UTF-8 text cut at NUL characters. The first piece is appended
//! to the manifest of a package named `demo`, so it can add targets,
//! dependencies, members or a `[workspace]` table. Each later piece is one
//! file: its first line is the key and the rest is the text.

use raf_source::{MemoryTree, Namespace, RelPath, Workspace};

/// The start of the root manifest; the first piece of the input follows it.
const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

/// Paths resolved in every workspace that loads, beside every item path.
const QUERIES: [&str; 10] = [
    "demo",
    "demo::a",
    "demo::a::f",
    "demo::f",
    "demo::S",
    "demo::S::m",
    "demo::T",
    "demo::bin_crate::x",
    "c",
    "c::f",
];

/// The tree an input describes, or `None` for bytes that are not UTF-8 and
/// for keys the tree refuses.
pub(crate) fn tree(data: &[u8]) -> Option<MemoryTree> {
    let text = std::str::from_utf8(data).ok()?;
    let mut pieces = text.split('\0');
    let manifest = format!("{MANIFEST}{}", pieces.next().unwrap_or_default());
    let mut files = vec![("Cargo.toml", manifest.as_str())];
    files.extend(pieces.map(|piece| piece.split_once('\n').unwrap_or((piece, ""))));
    MemoryTree::try_from_iter(files).ok()
}

/// Ask the index what a tool asks: every query and every item path in each
/// namespace and package, the items of each file and under each query, and
/// the test functions. Returns the number of items.
pub(crate) fn exercise(workspace: &Workspace) -> usize {
    let packages: Vec<String> = workspace
        .packages()
        .iter()
        .map(|package| package.name.clone())
        .collect();
    let mut paths: Vec<String> = QUERIES.iter().map(|query| (*query).to_owned()).collect();
    paths.extend(workspace.items().map(|item| item.path.to_string()));
    for path in &paths {
        drop(workspace.resolve(path));
        for namespace in [Namespace::Type, Namespace::Value, Namespace::Macro] {
            drop(workspace.resolve_in(path, namespace));
        }
        for package in &packages {
            drop(workspace.resolve_in_package(package, path));
        }
        std::hint::black_box(workspace.items_under(path).count());
    }
    let files: Vec<RelPath> = workspace.files().map(|file| file.path.clone()).collect();
    for file in &files {
        std::hint::black_box(workspace.items_in(file).count());
    }
    drop(workspace.test_fns());
    workspace.items().count()
}

/// A seed with items: it must load, and its index must hold items. A build
/// whose parser lost the positions of what it parses fails here, before any
/// input is judged.
pub(crate) fn self_check() {
    let seed = "\0src/lib.rs\npub mod a {\n    pub fn f() {}\n}\npub struct S;\n";
    let tree = tree(seed.as_bytes()).expect("the self-check seed is a tree");
    let workspace = match Workspace::load(&tree, &["."]) {
        Ok(workspace) => workspace,
        Err(error) => panic!("the self-check seed does not load: {error}"),
    };
    assert!(
        exercise(&workspace) >= 3,
        "the self-check seed has no items"
    );
}
