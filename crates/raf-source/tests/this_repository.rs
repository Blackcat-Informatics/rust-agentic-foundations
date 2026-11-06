// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The workspace this crate lives in loads through a real directory, members
//! that inherit their edition included.

use std::path::Path;

use raf_source::{DiskTree, Namespace, ResolveError, SymbolKind, TargetKind, Vis, Workspace};

/// Protects the public syntax boundary for single-file and corpus consumers.
/// Alternatives considered: a type-only check misses ownership and callbacks;
/// workspace loading hides explicitly selected inactive files; a copied parser
/// would not protect the public API's native errors and worker lifetime.
#[test]
fn the_reexported_syn_types_are_the_types_received_by_the_parse_visitor() {
    use raf_source::syn as indexed_syn;

    fn count(file: &indexed_syn::File) -> usize {
        file.items.len()
    }
    let file = raf_source::RelPath::new("src/example.rs").expect("path");
    assert_eq!(
        raf_source::parse_source(&file, "fn ordinary() {}", count).expect("valid source"),
        1
    );
    let error = raf_source::parse_source(&file, "fn {", count).expect_err("malformed source");
    assert!(matches!(
        error.kind,
        raf_source::LoadErrorKind::Syntax { .. }
    ));
    assert_eq!(error.file, file);
    owned_corpus_keeps_admission_and_worker_failures_atomic();
}

fn owned_corpus_keeps_admission_and_worker_failures_atomic() {
    use raf_source::{Budget, LoadErrorKind, LoadOptions, ParsedSource, RelPath, parse_sources};
    use std::sync::atomic::{AtomicBool, Ordering};

    let inputs = vec![
        (
            RelPath::new("src/alpha.rs").expect("path"),
            "fn alpha() {}".to_owned(),
        ),
        (
            RelPath::new("inactive/beta.rs").expect("path"),
            "fn beta() {}".to_owned(),
        ),
    ];
    let mut exact = LoadOptions::default();
    exact.max_file_bytes = inputs
        .iter()
        .map(|(_, text)| text.len() as u64)
        .max()
        .expect("inputs");
    exact.max_total_bytes = inputs.iter().map(|(_, text)| text.len()).sum();
    exact.max_file_tokens = 6;
    exact.max_item_bytes = inputs.len() * size_of::<ParsedSource>();
    let names = parse_sources(inputs.clone(), &exact, |files| {
        // The consumer takes ownership across files and makes a second pass.
        let mut items = Vec::new();
        for file in files {
            assert_ne!(file.text, "");
            assert_ne!(file.path.as_str(), "");
            items.extend(file.syntax.items);
        }
        items
            .into_iter()
            .map(|item| match item {
                syn::Item::Fn(function) => function.sig.ident.to_string(),
                _ => panic!("fixture contains functions"),
            })
            .collect::<Vec<_>>()
    })
    .expect("exact grants admit the complete corpus");
    assert_eq!(names, ["alpha", "beta"]);
    for budget in [Budget::TotalBytes, Budget::FileTokens, Budget::ItemBytes] {
        let mut short = exact.clone();
        match budget {
            Budget::TotalBytes => short.max_total_bytes -= 1,
            Budget::FileTokens => short.max_file_tokens -= 1,
            Budget::ItemBytes => short.max_item_bytes -= 1,
            _ => unreachable!("only the three tested budgets"),
        }
        let error = parse_sources(inputs.clone(), &short, |_| panic!("partial callback"))
            .expect_err("short grant");
        assert!(
            matches!(error.kind, LoadErrorKind::Budget { budget: actual, .. } if actual == budget)
        );
    }
    let mut short = exact.clone();
    short.max_file_bytes -= 1;
    let error = parse_sources(inputs.clone(), &short, |_| panic!("oversized callback"))
        .expect_err("file bytes");
    assert!(
        matches!(error.kind, LoadErrorKind::Tree(source) if matches!(*source, raf_source::TreeError::TooLarge { .. }))
    );
    let mut malformed = inputs;
    "fn {".clone_into(&mut malformed[1].1);
    let called = AtomicBool::new(false);
    let error = parse_sources(malformed, &LoadOptions::default(), |_| {
        called.store(true, Ordering::SeqCst);
    })
    .expect_err("later source is invalid");
    assert!(!called.load(Ordering::SeqCst));
    assert_eq!(error.file.as_str(), "inactive/beta.rs");
    assert!(matches!(error.kind, LoadErrorKind::Syntax { .. }));
    assert_eq!(
        parse_sources(Vec::new(), &exact, |files| files.len()).expect("empty corpus"),
        0
    );
    let panic = std::panic::catch_unwind(|| {
        let _: Result<(), _> = parse_sources(Vec::new(), &exact, |_| {
            std::panic::panic_any(String::from("corpus visitor failed"));
        });
    })
    .expect_err("visitor panic propagated");
    assert_eq!(
        panic.downcast_ref::<String>().map(String::as_str),
        Some("corpus visitor failed")
    );
}

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

#[test]
fn a_dependency_re_exported_by_its_crate_name_is_not_found_and_not_a_cycle() {
    let ws = this_repository();
    // `raf-source` re-exports `syn` with `pub use syn;`, a crate outside the
    // workspace.
    for path in ["raf_source::syn", "raf_source::syn::File"] {
        assert_eq!(
            ws.resolve(path),
            Err(ResolveError::NotFound(path.to_owned())),
            "{path}"
        );
    }
    // Beside it, a re-export of the crate's own item resolves.
    assert_eq!(
        ws.resolve("raf_source::DiskTree")
            .unwrap_or_else(|e| panic!("{e}"))
            .path,
        "raf_source::tree::disk::DiskTree"
    );
}

#[test]
fn the_surface_of_each_library_and_binary_of_this_repository_lists_its_public_items() {
    let ws = this_repository();
    let (mut listed, mut binaries) = (0, 0);
    for id in ws.target_ids() {
        let target = ws.target(id).expect("target");
        if !matches!(target.kind, TargetKind::Lib | TargetKind::Bin) {
            continue;
        }
        let exports = ws
            .exported_items(id)
            .unwrap_or_else(|e| panic!("{}: {e}", target.crate_path));
        listed += exports.items.len();
        if target.crate_path == "raf_source" {
            let paths: Vec<&str> = exports.items.iter().map(|e| e.path.as_str()).collect();
            for public in [
                "raf_source::Workspace",
                "raf_source::Workspace::exported_items",
                "raf_source::DiskTree",
                "raf_source::ExportedItem",
            ] {
                assert!(paths.contains(&public), "{public} is not listed");
            }
            // No item of a private module is listed under that module, nor is
            // the dependency `syn`, which is a re-export of another crate.
            let private: Vec<String> = ws
                .items_under("raf_source")
                .filter(|item| {
                    item.kind == SymbolKind::Mod
                        && item.visibility == Vis::Inherited
                        && item.path.matches("::").count() == 1
                })
                .map(|item| format!("{}::", item.path))
                .collect();
            assert!(private.len() > 10, "{private:?}");
            for path in &paths {
                assert!(
                    !private
                        .iter()
                        .any(|module| path.starts_with(module.as_str())),
                    "{path}"
                );
            }
            assert!(
                exports
                    .reexports
                    .iter()
                    .any(|reexport| reexport.path == "raf_source::syn" && reexport.item.is_none())
            );
            let disk = exports
                .items
                .iter()
                .find(|e| e.path == "raf_source::DiskTree")
                .expect("listed");
            assert_eq!(
                ws.item(disk.item).expect("item").path,
                "raf_source::tree::disk::DiskTree"
            );
        }
        // The command line binary keeps its items in its crate.
        if target.crate_path == "cargo_workplan::bin_crate::cargo_workplan" {
            assert_eq!(exports.items, []);
            binaries += 1;
        }
    }
    assert_eq!(binaries, 1);
    assert!(listed > 100, "{listed}");
}
