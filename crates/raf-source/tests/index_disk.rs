// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The same tree on disk and in memory gives an equal index.

mod support;

use raf_source::{DiskTree, MemoryTree, Namespace, Workspace};
use support::Scratch;

const FILES: [(&str, &str); 6] = [
    (
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    ),
    (
        "src/lib.rs",
        "mod inner;\npub use inner::go as run;\npub use inner::*;\npub struct Thing;\nimpl Thing { pub fn new() {} }\n#[cfg(unix)] pub fn twin() {}\n#[cfg(windows)] pub fn twin() {}\n",
    ),
    (
        "src/inner.rs",
        "/// Docs.\npub fn go() {}\npub fn other() {}\n",
    ),
    (
        "src/main.rs",
        "extern crate demo as d;\nuse d::inner::go;\nfn main() { go() }\n#[test] fn t() {}\n",
    ),
    (
        "tests/it.rs",
        "use demo::Thing;\n#[test] fn case() { Thing::new() }\n",
    ),
    ("benches/b.rs", "fn bench() {}\n"),
];

#[test]
fn disk_and_memory_trees_give_equal_workspaces_and_equal_resolutions() {
    let scratch = Scratch::new();
    for (name, text) in FILES {
        scratch.file(&format!("root/{name}"), text);
    }
    let disk =
        Workspace::load(&DiskTree::new(scratch.path("root")).expect("tree"), &["."]).expect("disk");
    let memory =
        Workspace::load(&FILES.into_iter().collect::<MemoryTree>(), &["."]).expect("memory");
    assert_eq!(disk, memory);
    for path in [
        "demo::run",
        "demo::other",
        "demo::twin",
        "demo::Thing::new",
        "demo::test_crate::it::case",
        "demo::nope",
    ] {
        assert_eq!(disk.resolve(path), memory.resolve(path), "{path}");
    }
    assert_eq!(
        disk.resolve_in("demo::go", Namespace::Value),
        memory.resolve_in("demo::go", Namespace::Value)
    );
    assert!(disk.resolve("demo::run").is_ok());
}

#[test]
fn a_reload_observes_changed_source_while_the_previous_workspace_keeps_its_snapshot() {
    let scratch = Scratch::new();
    scratch.file("root/Cargo.toml", FILES[0].1);
    scratch.file("root/src/lib.rs", "pub fn before() {}\n");
    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    let before = Workspace::load(&tree, &["."]).expect("first snapshot");
    let original = before.resolve("demo::before").expect("original item");
    let file = before.item(original.sites[0]).expect("item").file.clone();
    assert_eq!(before.source(&file), Some("pub fn before() {}\n"));

    scratch.file("root/src/lib.rs", "pub fn after() {}\n");
    let after = Workspace::load(&tree, &["."]).expect("reloaded snapshot");
    after.resolve("demo::after").expect("new item");
    assert!(after.resolve("demo::before").is_err());
    assert_eq!(after.source(&file), Some("pub fn after() {}\n"));
    before
        .resolve("demo::before")
        .expect("held snapshot is unchanged");
    assert!(before.resolve("demo::after").is_err());
    assert_eq!(before.source(&file), Some("pub fn before() {}\n"));
}
