// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The seeds of the `load-workspace` fuzz target are replayed under `cargo
//! test` through the same input code the fuzz target runs, so a seed that
//! stops reaching the index, or a build in which no input could, fails here.

use std::fs;
use std::path::Path;

use raf_source::Workspace;

#[path = "../../../fuzz/fuzz_targets/workspace_input.rs"]
mod workspace_input;

#[test]
fn every_load_workspace_seed_loads_with_items_through_the_fuzz_input_code() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fuzz/corpus/load-workspace");
    let mut seeds: Vec<_> = fs::read_dir(&dir)
        .expect("the seed directory exists")
        .map(|entry| entry.expect("directory entry").path())
        .collect();
    seeds.sort();
    assert!(seeds.len() >= 4, "{seeds:?}");
    for seed in seeds {
        let data = fs::read(&seed).expect("read the seed");
        let tree = workspace_input::tree(&data)
            .unwrap_or_else(|| panic!("{} is not a tree", seed.display()));
        let workspace = Workspace::load(&tree, &["."])
            .unwrap_or_else(|error| panic!("{} does not load: {error}", seed.display()));
        assert!(
            workspace_input::exercise(&workspace) >= 1,
            "{} has no items",
            seed.display()
        );
    }
}

#[test]
fn the_self_check_the_fuzz_target_runs_first_passes_beside_an_input_that_is_not_a_tree() {
    workspace_input::self_check();
    let tree =
        workspace_input::tree(b"").expect("an empty input describes a package without targets");
    let workspace = Workspace::load(&tree, &["."]).expect("a package without targets loads");
    assert_eq!(workspace_input::exercise(&workspace), 0);
    assert!(workspace_input::tree(&[0xff, 0xfe]).is_none());
    assert!(workspace_input::tree(b"\0../outside.rs\nfn f() {}").is_none());
}
