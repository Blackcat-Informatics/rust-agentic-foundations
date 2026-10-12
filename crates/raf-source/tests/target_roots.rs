// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The crate root of every target is a module item at its crate path.

use std::path::Path;

use raf_source::{DiskTree, MemoryTree, Namespace, ResolveError, SymbolKind, TestRole, Workspace};

const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";

fn all_kinds() -> Workspace {
    let tree: MemoryTree = [
        ("Cargo.toml", DEMO),
        ("src/lib.rs", "//! The library.\npub fn go() {}\n"),
        ("src/main.rs", "fn main() {}\n"),
        ("src/bin/x.rs", "fn main() {}\n"),
        ("tests/it.rs", "#[test]\nfn case() {}\n"),
        ("benches/b.rs", "fn bench() {}\n"),
        ("examples/e.rs", "fn main() {}\n"),
        ("build.rs", "fn main() {}\n"),
    ]
    .into_iter()
    .collect();
    Workspace::load(&tree, &["."]).expect("loads")
}

#[test]
fn every_target_kind_resolves_at_its_crate_path_as_a_module_item_of_its_root_file() {
    let ws = all_kinds();
    let cases = [
        ("demo", "src/lib.rs", true),
        ("demo::bin_crate::demo", "src/main.rs", true),
        ("demo::bin_crate::x", "src/bin/x.rs", true),
        ("demo::test_crate::it", "tests/it.rs", false),
        ("demo::bench_crate::b", "benches/b.rs", false),
        ("demo::example_crate::e", "examples/e.rs", false),
        ("demo::build_script", "build.rs", true),
    ];
    for (path, file, shipping) in cases {
        let resolution = ws.resolve(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(resolution.namespace, Namespace::Type, "{path}");
        assert_eq!(resolution.sites.len(), 1, "{path}");
        let item = ws.item(resolution.sites[0]).expect("item");
        assert_eq!(item.kind, SymbolKind::Mod, "{path}");
        assert_eq!(item.path, path);
        assert_eq!(item.file.as_str(), file, "{path}");
        assert_eq!(item.name_line, 1, "{path}");
        assert_eq!(item.span.start_byte, 0, "{path}");
        assert_eq!(
            item.span.end_byte,
            ws.source(&item.file).expect("text").len(),
            "{path}"
        );
        assert_eq!(ws.is_shipping(item), shipping, "{path}");
        // Only the library has inner documentation.
        assert_eq!(item.documented, path == "demo", "{path}");
    }
    let roles = |path: &str| {
        ws.item(ws.resolve(path).expect("resolves").sites[0])
            .expect("item")
            .test
    };
    assert_eq!(roles("demo::test_crate::it"), TestRole::CfgTest);
    assert_eq!(roles("demo"), TestRole::None);
    // An item inside a root still resolves through it.
    assert!(ws.resolve("demo::test_crate::it::case").is_ok());
    assert!(ws.resolve("demo::go").is_ok());
}

#[test]
fn the_namespace_segments_are_not_items_and_a_first_party_module_of_that_name_collides() {
    let ws = all_kinds();
    for segment in [
        "demo::test_crate",
        "demo::bench_crate",
        "demo::example_crate",
        "demo::bin_crate",
    ] {
        assert!(
            matches!(ws.resolve(segment), Err(ResolveError::NotFound(_))),
            "{segment}: {:?}",
            ws.resolve(segment)
        );
    }
    // A target root is a type; the value namespace has nothing at its path.
    assert!(ws.resolve_in("demo", Namespace::Value).is_err());
    // With a first-party module of that name the shared prefix stays ambiguous.
    let tree: MemoryTree = [
        ("Cargo.toml", DEMO),
        ("src/lib.rs", "mod test_crate;\n"),
        ("src/test_crate.rs", "pub fn first_party() {}\n"),
        ("tests/it.rs", "pub fn case() {}\n"),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["."]).expect("loads");
    assert!(matches!(
        ws.resolve("demo::test_crate::it::case"),
        Err(ResolveError::Ambiguous(_))
    ));
    assert!(ws.resolve("demo::test_crate").is_ok());
}

#[test]
fn this_repository_resolves_a_test_crate_root_and_a_library_root() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ws = Workspace::load(&DiskTree::new(repository).expect("tree"), &["crates"])
        .unwrap_or_else(|e| panic!("{e}"));
    let library = ws.resolve("raf_okf").expect("the library root");
    let item = ws.item(library.sites[0]).expect("item");
    assert_eq!(item.file.as_str(), "crates/raf-okf/src/lib.rs");
    let test = ws
        .resolve("raf_okf::test_crate::refusals")
        .expect("a test target root");
    let item = ws.item(test.sites[0]).expect("item");
    assert_eq!(item.file.as_str(), "crates/raf-okf/tests/refusals.rs");
    assert!(!ws.is_shipping(item));
}

#[test]
fn a_first_party_module_at_a_binary_crate_path_is_ambiguous_beside_a_binary_that_is_not() {
    let tree: MemoryTree = [
        ("Cargo.toml", DEMO),
        (
            "src/lib.rs",
            "pub mod bin_crate { pub mod demo { pub fn f() {} } }\n",
        ),
        ("src/main.rs", "fn main() {}\n"),
        ("src/bin/x.rs", "fn main() {}\n"),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["."]).expect("loads");
    // The module and the binary root share a path.
    match ws.resolve("demo::bin_crate::demo") {
        Err(ResolveError::Ambiguous(ids)) => {
            assert_eq!(ids.len(), 2);
            let files: Vec<_> = ids
                .iter()
                .map(|id| ws.item(*id).expect("item").file.as_str())
                .collect();
            assert!(files.contains(&"src/lib.rs") && files.contains(&"src/main.rs"));
        }
        other => panic!("{other:?}"),
    }
    // The module's own item still resolves.
    assert!(ws.resolve("demo::bin_crate::demo::f").is_ok());
    // The same layout without the first-party `bin_crate` module has no collision.
    let tree: MemoryTree = [
        ("Cargo.toml", DEMO),
        ("src/lib.rs", "pub mod demo { pub fn f() {} }\n"),
        ("src/main.rs", "fn main() {}\n"),
        ("src/bin/x.rs", "fn main() {}\n"),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["."]).expect("loads");
    assert_eq!(
        ws.resolve("demo::bin_crate::demo")
            .expect("the root")
            .sites
            .len(),
        1
    );
    assert_eq!(
        ws.resolve("demo::bin_crate::x")
            .expect("resolves")
            .sites
            .len(),
        1
    );
    assert!(ws.resolve("demo::demo::f").is_ok());
}
