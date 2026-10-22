// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The crate root of every target is a module item at its crate path.

use std::path::Path;

use raf_source::{
    DiskTree, LoadErrorKind, LoadOptions, MemoryTree, Namespace, Origin, ResolveError, SymbolKind,
    TargetKind, TestRole, TreeError, Workspace,
};

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
    // A first-party module of that name is an item, and it collides with a
    // target only at a path that both define.
    let tree: MemoryTree = [
        ("Cargo.toml", DEMO),
        ("src/lib.rs", "mod test_crate;\n"),
        (
            "src/test_crate.rs",
            "pub mod it {}\npub fn first_party() {}\n",
        ),
        ("tests/it.rs", "pub fn case() {}\n"),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["."]).expect("loads");
    assert!(matches!(
        ws.resolve("demo::test_crate::it"),
        Err(ResolveError::Ambiguous(_))
    ));
    assert!(ws.resolve("demo::test_crate::it::case").is_ok());
    assert!(ws.resolve("demo::test_crate").is_ok());
}

/// The file of the one definition `path` resolves to.
fn file_of(ws: &Workspace, path: &str) -> String {
    let resolution = ws.resolve(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    assert_eq!(resolution.sites.len(), 1, "{path}");
    ws.item(resolution.sites[0]).expect("item").file.to_string()
}

/// The files of the items `path` is ambiguous between, sorted.
fn ambiguous_files(ws: &Workspace, path: &str) -> Vec<String> {
    match ws.resolve(path) {
        Err(ResolveError::Ambiguous(ids)) => {
            let mut files: Vec<String> = ids
                .iter()
                .map(|id| ws.item(*id).expect("item").file.to_string())
                .collect();
            files.sort();
            files
        }
        other => panic!("{path}: {other:?}"),
    }
}

#[test]
fn a_lib_module_named_bin_crate_stays_in_the_library() {
    for (segment, file) in [
        ("bin_crate", "src/bin/x.rs"),
        ("test_crate", "tests/x.rs"),
        ("bench_crate", "benches/x.rs"),
        ("example_crate", "examples/x.rs"),
    ] {
        let lib = format!("pub mod {segment} {{ pub mod x {{ pub fn lib_fn() {{}} }} }}\n");
        let tree: MemoryTree = [
            ("Cargo.toml", DEMO),
            ("src/lib.rs", lib.as_str()),
            (file, "pub fn bin_fn() {}\nfn main() {}\n"),
        ]
        .into_iter()
        .collect();
        let ws = Workspace::load(&tree, &["."]).expect("loads");
        let base = format!("demo::{segment}::x");
        assert_eq!(file_of(&ws, &format!("{base}::bin_fn")), file, "{segment}");
        assert_eq!(file_of(&ws, &format!("{base}::lib_fn")), "src/lib.rs");
        let lib_fn = ws
            .items()
            .find(|item| item.name == "lib_fn")
            .expect("indexed");
        assert_eq!(lib_fn.path, format!("{base}::lib_fn"));
        assert_eq!(
            ws.target(lib_fn.target).expect("target").kind,
            TargetKind::Lib
        );
        assert!(matches!(
            ws.resolve(&format!("{base}::main")),
            Ok(found) if found.sites.len() == 1
        ));
        // The library's module and the target's root share their path.
        let mut both = vec!["src/lib.rs".to_owned(), file.to_owned()];
        both.sort();
        assert_eq!(ambiguous_files(&ws, &base), both);
    }
    let tree: MemoryTree = [
        ("Cargo.toml", DEMO),
        (
            "src/lib.rs",
            "pub mod build_script { pub fn lib_fn() {} }\n",
        ),
        ("build.rs", "pub fn bin_fn() {}\nfn main() {}\n"),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["."]).expect("loads");
    assert_eq!(file_of(&ws, "demo::build_script::bin_fn"), "build.rs");
    assert_eq!(file_of(&ws, "demo::build_script::lib_fn"), "src/lib.rs");
    assert_eq!(
        ambiguous_files(&ws, "demo::build_script"),
        ["build.rs", "src/lib.rs"]
    );
}

#[test]
fn a_build_script_main_beside_a_lib_module_build_script_is_ambiguous() {
    // `build.rs` sorts before `src/lib.rs` and `zz/build.rs` after it.
    let named = format!("{DEMO}build = \"zz/build.rs\"\n");
    for (manifest, script) in [(DEMO, "build.rs"), (named.as_str(), "zz/build.rs")] {
        let tree: MemoryTree = [
            ("Cargo.toml", manifest),
            ("src/lib.rs", "pub mod build_script { pub fn main() {} }\n"),
            (script, "fn main() {}\n"),
        ]
        .into_iter()
        .collect();
        let ws = Workspace::load(&tree, &["."]).expect("loads");
        let mut expected = vec![script.to_owned(), "src/lib.rs".to_owned()];
        expected.sort();
        assert_eq!(
            ambiguous_files(&ws, "demo::build_script::main"),
            expected,
            "{script}"
        );
    }
    // Without the library's module the build script's `main` is one symbol.
    let tree: MemoryTree = [
        ("Cargo.toml", DEMO),
        ("src/lib.rs", "pub mod tools { pub fn main() {} }\n"),
        ("build.rs", "fn main() {}\n"),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["."]).expect("loads");
    assert_eq!(file_of(&ws, "demo::build_script::main"), "build.rs");
}

#[test]
fn an_explicit_path_bin_beside_a_lib_module_bin_crate_is_ambiguous_in_both_file_orders() {
    // `a/x.rs` sorts before `src/lib.rs` and `tools/x.rs` after it.
    for path in ["a/x.rs", "tools/x.rs"] {
        let manifest = format!("{DEMO}\n[[bin]]\nname = \"x\"\npath = \"{path}\"\n");
        let tree: MemoryTree = [
            ("Cargo.toml", manifest.as_str()),
            (
                "src/lib.rs",
                "pub mod bin_crate { pub mod x { pub fn main() {} } }\n",
            ),
            (path, "fn main() {}\n"),
        ]
        .into_iter()
        .collect();
        let ws = Workspace::load(&tree, &["."]).expect("loads");
        let mut expected = vec![path.to_owned(), "src/lib.rs".to_owned()];
        expected.sort();
        assert_eq!(
            ambiguous_files(&ws, "demo::bin_crate::x::main"),
            expected,
            "{path}"
        );
        // The same layout with a library module of another name is one
        // symbol in each order.
        let tree: MemoryTree = [
            ("Cargo.toml", manifest.as_str()),
            (
                "src/lib.rs",
                "pub mod tools { pub mod x { pub fn main() {} } }\n",
            ),
            (path, "fn main() {}\n"),
        ]
        .into_iter()
        .collect();
        let ws = Workspace::load(&tree, &["."]).expect("loads");
        assert_eq!(file_of(&ws, "demo::bin_crate::x::main"), path);
        assert_eq!(file_of(&ws, "demo::tools::x::main"), "src/lib.rs");
    }
}

#[test]
fn an_over_cap_inferred_lib_names_discovery_not_lib_path() {
    let mut options = LoadOptions::default();
    // Every manifest below fits the cap; the library is over it.
    options.max_file_bytes = 200;
    let big = format!("{}\n", "/".repeat(300));
    for (manifest, origin) in [
        (DEMO.to_owned(), Origin::Discovery),
        (
            format!("{DEMO}\n[lib]\nname = \"demo\"\n"),
            Origin::ManifestKey {
                key: "lib".to_owned(),
            },
        ),
        (
            format!("{DEMO}\n[lib]\npath = \"src/lib.rs\"\n"),
            Origin::ManifestKey {
                key: "lib.path".to_owned(),
            },
        ),
    ] {
        let tree: MemoryTree = [
            ("Cargo.toml", manifest.as_str()),
            ("src/lib.rs", big.as_str()),
        ]
        .into_iter()
        .collect();
        let error = Workspace::load_with(&tree, &["."], &options).expect_err("over the cap");
        assert_eq!(error.file.as_str(), "src/lib.rs");
        assert!(
            matches!(&error.kind, LoadErrorKind::Tree(inner) if matches!(**inner, TreeError::TooLarge { .. })),
            "{error}"
        );
        assert_eq!(error.origin.as_deref(), Some(&origin), "{error}");
        let text = error.to_string();
        assert_eq!(
            text.contains("lib.path"),
            origin
                == Origin::ManifestKey {
                    key: "lib.path".to_owned()
                },
            "{text}"
        );
    }
    // A library under the cap loads under each manifest.
    let tree: MemoryTree = [("Cargo.toml", DEMO), ("src/lib.rs", "pub fn go() {}\n")]
        .into_iter()
        .collect();
    assert!(Workspace::load_with(&tree, &["."], &options).is_ok());
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
