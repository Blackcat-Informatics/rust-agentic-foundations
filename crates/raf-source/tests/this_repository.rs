// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The workspace this crate lives in loads through a real directory, members
//! that inherit their edition included.

use std::path::Path;

use raf_source::{DiskTree, Namespace, Workspace};

#[test]
fn this_repository_loads_from_its_root_with_the_crates_directory_as_the_root() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ws = Workspace::load(&DiskTree::new(repository).expect("tree"), &["crates"])
        .unwrap_or_else(|e| panic!("{e}"));
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
