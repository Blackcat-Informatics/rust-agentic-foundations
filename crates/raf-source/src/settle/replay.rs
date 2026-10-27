// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Boundary of replay elimination for genuinely fixed glob paths.

use super::*;
use crate::{LoadOptions, MemoryTree, Namespace, Workspace};

fn load(source: &str) -> Workspace {
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        ),
        ("src/lib.rs", source),
    ]
    .into_iter()
    .collect();
    Workspace::load(&tree, &["."]).expect("source loads")
}

fn fixed(ws: &Workspace) -> bool {
    // This private proof runs during settling, before query accounting is
    // enabled. Recreate that phase on an owned copy of the loaded index.
    let index = ws.index().clone();
    index.lookups.built(false);
    index
        .fixed_glob_paths(&Meter::new(&LoadOptions::default()))
        .expect("resident path proof")
}

#[test]
fn genuine_paths_are_fixed_but_glob_prefixes_and_named_shadows_require_replay() {
    let genuine = load(
        "pub mod source { pub mod inner { pub struct X; } } pub mod prefix { pub mod inner { pub struct X; } } pub use crate::prefix::inner::*;",
    );
    assert!(fixed(&genuine));
    assert_eq!(
        genuine.resolve("demo::X").expect("genuine target").path,
        "demo::prefix::inner::X"
    );
    let globbed = load(
        "pub mod source { pub mod inner { pub struct X; } } pub mod prefix { pub use crate::source::*; } pub use crate::prefix::inner::*;",
    );
    assert!(!fixed(&globbed));
    assert_eq!(
        globbed
            .resolve("demo::X")
            .expect("glob-derived target")
            .path,
        "demo::source::inner::X"
    );
    let shadowed = load(
        "pub mod a { pub struct X; } pub mod b { pub struct X; } use crate::a::X; pub use crate::b::*;",
    );
    assert!(!fixed(&shadowed));
    assert_eq!(
        shadowed.resolve("demo::X").expect("named shadow").path,
        "demo::a::X"
    );
    let unshadowed =
        load("pub mod a { pub struct X; } pub mod b { pub struct X; } pub use crate::b::*;");
    assert!(fixed(&unshadowed));
    assert_eq!(
        unshadowed.resolve("demo::X").expect("ordinary glob").path,
        "demo::b::X"
    );
}

#[test]
fn path_visibility_and_unmatched_or_open_resident_targets_keep_replay() {
    let ws = load(
        "pub mod source { pub struct X; } pub mod outer { pub(in crate::outer) use crate::source::*; }",
    );
    assert!(!fixed(&ws));
    assert_eq!(
        ws.resolve_from_in("demo::outer", "self::X", Namespace::Type)
            .expect("local path visibility")
            .path,
        "demo::source::X"
    );
    let ws = load("pub mod source { pub struct X; } pub use crate::source::*;");
    assert!(fixed(&ws));
    let mut index = ws.index().clone();
    index.lookups.built(false);
    let glob = index.glob_imports[0];
    index.glob_reach[glob].clear();
    assert!(
        !index
            .fixed_glob_paths(&Meter::new(&LoadOptions::default()))
            .expect("stale target certificate")
    );
    let mut index = ws.index().clone();
    index.lookups.built(false);
    index.glob_closed[glob] = false;
    assert!(
        !index
            .fixed_glob_paths(&Meter::new(&LoadOptions::default()))
            .expect("open target certificate")
    );
}
