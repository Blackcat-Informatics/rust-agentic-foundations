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
