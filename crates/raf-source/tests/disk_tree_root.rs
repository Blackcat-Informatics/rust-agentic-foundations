// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `DiskTree` and `MemoryTree` agree on a tree of regular files, on any
//! platform.

mod support;

use raf_source::{DiskTree, MemoryTree, RelPath, Tree, TreeError};
use support::Scratch;

#[test]
fn disk_and_memory_trees_agree_on_regular_files() {
    let scratch = Scratch::new();
    let files = [
        ("Cargo.toml", "[package]\n"),
        ("src/lib.rs", "mod a;\n"),
        ("src/a/mod.rs", "// a\n"),
    ];
    for (name, text) in files {
        scratch.file(&format!("root/{name}"), text);
    }
    let disk = DiskTree::new(scratch.path("root")).expect("tree");
    let memory = MemoryTree::from(files);

    for directory in ["", "src", "src/a"] {
        let at = RelPath::new(directory).expect("path");
        assert_eq!(
            disk.list(&at).expect("disk"),
            memory.list(&at).expect("memory"),
            "{directory}"
        );
    }
    for name in [
        "Cargo.toml",
        "src/lib.rs",
        "src/a/mod.rs",
        "src/none.rs",
        "src",
    ] {
        let at = RelPath::new(name).expect("path");
        assert_eq!(
            disk.entry(&at).expect("disk"),
            memory.entry(&at).expect("memory"),
            "{name}"
        );
        assert_eq!(
            disk.read(&at, 64).map_err(|e| std::mem::discriminant(&e)),
            memory.read(&at, 64).map_err(|e| std::mem::discriminant(&e)),
            "{name}"
        );
    }
}

#[test]
fn a_root_that_is_missing_is_not_found_on_every_platform() {
    let scratch = Scratch::new();
    let error = DiskTree::new(scratch.path("absent")).expect_err("missing root");
    assert!(matches!(error, TreeError::NotFound { .. }));
}

#[cfg(unix)]
#[test]
fn root_discovery_refuses_directory_aliases_and_unsafe_parent_reductions_beside_real_starts() {
    let scratch = Scratch::new();
    scratch.file("root/workspace/Cargo.toml", "[workspace]\n");
    scratch.dir("root/workspace/notes");
    std::os::unix::fs::symlink("workspace", scratch.path("root/alias")).expect("directory alias");
    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    let real = RelPath::new("workspace/notes").expect("real start");
    assert_eq!(
        raf_source::discover_root(&tree, &real).expect("real directory"),
        RelPath::new("workspace").expect("workspace")
    );
    let parent_start = RelPath::new("workspace/notes/..").expect("safe parent");
    assert_eq!(
        raf_source::discover_root(&tree, &parent_start).expect("real parent is reduced"),
        RelPath::new("workspace").expect("workspace")
    );
    for start in [
        "alias",
        "alias/notes",
        "alias/../workspace",
        "missing/../workspace",
    ] {
        let error = raf_source::discover_root(&tree, &RelPath::new(start).expect("alias path"))
            .expect_err("lexical ancestors of a directory alias are not workspace ancestors");
        let message = error.to_string();
        assert!(message.contains("directory symlinks"), "{message}");
        assert!(message.contains("canonicalize"), "{message}");
    }
}

#[cfg(unix)]
#[test]
fn a_dangling_manifest_is_refused_without_falling_back_to_its_workspace_ancestor() {
    let scratch = Scratch::new();
    scratch.file(
        "root/workspace/Cargo.toml",
        "[workspace]\nmembers=['pkg']\n",
    );
    scratch.dir("root/workspace/pkg");
    let manifest = scratch.path("root/workspace/pkg/Cargo.toml");
    std::os::unix::fs::symlink("missing.toml", &manifest).expect("dangling manifest");
    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    let start = RelPath::new("workspace/pkg").expect("package directory");
    let error =
        raf_source::discover_root(&tree, &start).expect_err("present but unreadable manifest");
    assert!(
        matches!(error.kind, raf_source::LoadErrorKind::Tree(ref error)
        if matches!(error.as_ref(), TreeError::NotFound { .. }))
    );
    assert_eq!(error.file.as_str(), "workspace/pkg/Cargo.toml");

    std::fs::remove_file(&manifest).expect("remove dangling link");
    scratch.file(
        "root/workspace/pkg/Cargo.toml",
        "[package]\nname='pkg'\nversion='0.1.0'\n",
    );
    assert_eq!(
        raf_source::discover_root(&tree, &start).expect("readable member manifest"),
        RelPath::new("workspace").expect("workspace")
    );
}
