// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The workspace this crate lives in loads through a real directory, members
//! that inherit their edition included.

use std::path::Path;

use raf_source::{DiskTree, Namespace, ResolveError, Workspace};

/// This repository, loaded with the crates directory as the root.
fn this_repository() -> Workspace {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Workspace::load(&DiskTree::new(repository).expect("tree"), &["crates"])
        .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn this_repository_loads_from_its_root_with_the_crates_directory_as_the_root() {
    let ws = this_repository();
    let names: Vec<_> = ws.packages().iter().map(|p| p.name.as_str()).collect();
    assert!(
        names.contains(&"raf-okf") && names.contains(&"raf-source"),
        "{names:?}"
    );
    let resolution = ws
        .resolve_in("raf_source::Workspace", Namespace::Type)
        .expect("this crate's own type resolves");
    assert_eq!(
        ws.item(resolution.sites[0]).expect("item").file.as_str(),
        "crates/raf-source/src/workspace.rs"
    );
}

#[test]
fn an_item_of_a_module_file_that_sorts_before_its_mod_rs_is_found_under_its_own_module() {
    let ws = this_repository();
    // `body/blocks.rs` sorts before `body/mod.rs`, which declares it.
    let resolution = ws
        .resolve("raf_okf::body::blocks::FenceBlock")
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(resolution.path, "raf_okf::body::blocks::FenceBlock");
    assert_eq!(
        ws.item(resolution.sites[0]).expect("item").file.as_str(),
        "crates/raf-okf/src/body/blocks.rs"
    );
    assert_eq!(
        ws.resolve("raf_okf::body::FenceBlock").map(|r| r.path),
        Err(ResolveError::NotFound(
            "raf_okf::body::FenceBlock".to_owned()
        ))
    );
}
