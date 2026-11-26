// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A file reached under two spellings is one physical parent, and each of its
//! declarations owns the children it declares, whichever spelling it arrived by.

#![cfg(unix)]

mod support;

use raf_source::{DiskTree, RelPath, Workspace};
use support::Scratch;

#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
#[test]
fn a_child_of_a_parent_reached_under_two_spellings_is_owned_by_both_declarations() {
    // INVARIANT: `src/alias.rs` is a symbolic link to `src/parent.rs`, so both
    // module paths reach one physical file. The canonical spelling is the one
    // that sorts first, `src/alias.rs`. Each `mod child;` in that file is one
    // declaration of `src/child.rs` under its module path, and the child file's
    // declaring items name both module paths, each at the declaration's line.
    let scratch = Scratch::new();
    scratch.dir("root");
    scratch.file(
        "root/Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    scratch.file(
        "root/src/lib.rs",
        "#[path = \"parent.rs\"]\npub mod parent;\n#[path = \"alias.rs\"]\npub mod alias_mod;\n",
    );
    scratch.file("root/src/parent.rs", "pub mod child;\n");
    scratch.file("root/src/child.rs", "use serde_json::Value;\n");
    scratch.link("root/src/alias.rs", scratch.path("root/src/parent.rs"));
    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    let workspace = Workspace::load(&tree, &["."]).expect("workspace");

    let child = RelPath::new("src/child.rs").unwrap();
    let mut owners: Vec<_> = workspace
        .declaring_modules(&child)
        .into_iter()
        .map(|item| (item.path.to_string(), item.file.to_string(), item.name_line))
        .collect();
    owners.sort();
    assert_eq!(
        owners,
        [
            (
                "demo::alias_mod::child".to_owned(),
                "src/alias.rs".to_owned(),
                1
            ),
            (
                "demo::parent::child".to_owned(),
                "src/alias.rs".to_owned(),
                1
            ),
        ],
    );
}
