// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Both backends enforce the byte cap on a read.

mod support;

use raf_source::{
    DEFAULT_MAX_FILE_BYTES, DiskTree, LoadOptions, MemoryTree, RelPath, Tree, TreeError,
};

const MIB: usize = 1024 * 1024;

fn exactly(len: usize) -> String {
    "a".repeat(len)
}

/// Run the cap cases against one backend that holds `exact.rs` of exactly
/// the default cap and `over.rs` one byte longer.
fn check_cap(tree: &dyn Tree) {
    let cap = DEFAULT_MAX_FILE_BYTES;
    let exact = RelPath::new("exact.rs").expect("path");
    let over = RelPath::new("over.rs").expect("path");

    assert_eq!(
        tree.read(&exact, cap).expect("exactly the cap").len(),
        8 * MIB
    );

    let refused = tree.read(&over, cap).expect_err("one byte over");
    assert!(
        matches!(refused, TreeError::TooLarge { limit, .. } if limit == cap),
        "{refused:?}"
    );
    assert!(refused.to_string().contains("over.rs"));

    // A lowered cap is honoured, with the limit it was given.
    let lowered = tree.read(&exact, 100).expect_err("over the lowered cap");
    assert!(
        matches!(lowered, TreeError::TooLarge { limit: 100, .. }),
        "{lowered:?}"
    );
    assert_eq!(
        tree.read(&exact, u64::try_from(8 * MIB).expect("fits"))
            .expect("fits")
            .len(),
        8 * MIB
    );
}

#[test]
fn memory_tree_enforces_the_cap_at_eight_mib() {
    let tree = MemoryTree::from([
        ("exact.rs", exactly(8 * MIB).as_str()),
        ("over.rs", exactly(8 * MIB + 1).as_str()),
    ]);
    check_cap(&tree);
}

#[test]
fn disk_tree_enforces_the_cap_at_eight_mib() {
    let scratch = support::Scratch::new();
    let root = scratch.dir("root");
    scratch.file("root/exact.rs", &exactly(8 * MIB));
    scratch.file("root/over.rs", &exactly(8 * MIB + 1));
    check_cap(&DiskTree::new(&root).expect("tree"));
}

#[test]
fn the_default_cap_is_eight_mib() {
    assert_eq!(DEFAULT_MAX_FILE_BYTES, 8 * 1024 * 1024);
    assert_eq!(
        LoadOptions::default().max_file_bytes,
        DEFAULT_MAX_FILE_BYTES
    );
}

#[test]
fn a_zero_cap_refuses_any_content_and_accepts_an_empty_file() {
    let scratch = support::Scratch::new();
    let root = scratch.dir("root");
    scratch.file("root/empty.rs", "");
    scratch.file("root/one.rs", "x");
    let disk = DiskTree::new(&root).expect("tree");
    let memory = MemoryTree::from([("empty.rs", ""), ("one.rs", "x")]);
    for tree in [&disk as &dyn Tree, &memory] {
        assert_eq!(
            tree.read(&RelPath::new("empty.rs").expect("path"), 0)
                .expect("empty"),
            ""
        );
        assert!(matches!(
            tree.read(&RelPath::new("one.rs").expect("path"), 0),
            Err(TreeError::TooLarge { limit: 0, .. })
        ));
    }
}
