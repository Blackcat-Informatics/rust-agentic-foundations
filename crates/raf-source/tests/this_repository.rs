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
    shared_tokens_keep_original_prefix_positions();
    prepared_source_keeps_counts_bound_to_its_input();
    lexed_source_keeps_admission_before_syntax();
}

fn lexed_source_keeps_admission_before_syntax() {
    use raf_source::{LoadErrorKind, RelPath, prepare_source};
    let path = RelPath::new("lexed.rs").expect("fixture path");
    let refused = prepare_source(&path, "fn ;")
        .expect("guard-admitted syntax data")
        .with_lexed(|lexed| {
            assert!(!lexed.tokens().is_empty());
            Err::<(), _>("parser admission refused")
        })
        .expect("lexing does not run the syntax parser");
    assert_eq!(refused, Err("parser admission refused"));

    let parsed = prepare_source(&path, "\u{feff}#!data\n/// doc\nfn café<'a>() {}")
        .expect("guard-admitted valid neighbour")
        .with_lexed(|lexed| {
            let expected = native_nodes(lexed.tokens().clone());
            lexed.parse_with_tokens(|syntax, original| {
                assert_eq!(syntax.shebang.as_deref(), Some("#!data"));
                let syn::Item::Fn(function) = &syntax.items[0] else {
                    panic!("the valid neighbour is a function")
                };
                assert_eq!(function.sig.fn_token.span.start().line, 3);
                assert_eq!(native_nodes(original), expected);
                syntax.items.len()
            })
        })
        .expect("lexical ticket")
        .expect("the admitted ticket parses");
    assert_eq!(parsed, 1);

    let error = prepare_source(&path, "fn ;")
        .expect("guard-admitted invalid syntax")
        .with_lexed(|lexed| lexed.parse(|_| panic!("invalid syntax cannot reach the visitor")))
        .expect("lexically valid ticket")
        .expect_err("syntax failure belongs to consuming parse");
    assert!(matches!(error.kind, LoadErrorKind::Syntax { .. }));
    assert_eq!(error.file, path);
}

fn native_nodes(tokens: proc_macro2::TokenStream) -> usize {
    tokens
        .into_iter()
        .map(|token| {
            1 + match token {
                proc_macro2::TokenTree::Group(group) => native_nodes(group.stream()),
                _ => 0,
            }
        })
        .sum()
}

fn prepared_source_keeps_counts_bound_to_its_input() {
    use raf_source::{Budget, LoadErrorKind, RelPath, prepare_source};
    let path = RelPath::new("prepared.rs").expect("path");
    for separator in ["\u{2003}", "\u{200e}", "\u{200f}", "\u{0085}"] {
        let macro_source = format!("m!({});", vec!["a"; 64].join(separator));
        let prepared = prepare_source(&path, &macro_source).expect("Unicode-separated macro data");
        let bound = prepared.native_token_bound();
        prepared
            .parse_with_tokens(|syntax, original| {
                assert_eq!(syntax.items.len(), 1);
                assert!(native_nodes(original) <= bound);
            })
            .expect("macro tokens are inspected without expansion");

        // Never parse this deep source: failure must happen during preparation.
        let deep = format!(
            "fn deep() {{ {}0; }}",
            format!("return{separator}").repeat(Budget::Recursion.limit() + 1)
        );
        let error = prepare_source(&path, &deep).expect_err("Unicode cannot hide recursion");
        assert!(matches!(
            error.kind,
            LoadErrorKind::Budget {
                budget: Budget::Recursion,
                ..
            }
        ));
        let small = format!("fn small() {{ return{separator}return{separator}0; }}");
        assert_eq!(
            prepare_source(&path, &small)
                .expect("small Unicode-separated neighbour")
                .parse(|syntax| syntax.items.len())
                .expect("admitted neighbour parses"),
            1
        );
    }
    for text in [
        "const X:f64=1e2.3;",
        "\u{feff}#!data\n/// doc\nfn café<'a>() {}",
    ] {
        let prepared = prepare_source(&path, text).expect("admitted source data");
        let bound = prepared.native_token_bound();
        prepared
            .parse_with_tokens(|_, original| assert!(native_nodes(original) <= bound))
            .expect("native nodes fit the prepared ceiling");
    }
    let malformed = prepare_source(&path, "fn {").expect("guards are not syntax validation");
    assert!(matches!(
        malformed.parse(|_| panic!("invalid syntax cannot reach the visitor")),
        Err(raf_source::LoadError {
            kind: LoadErrorKind::Syntax { .. },
            ..
        })
    ));
}

fn check_tokens(source: &str, original: proc_macro2::TokenStream) {
    for token in original {
        match token {
            proc_macro2::TokenTree::Group(group) => {
                let (open, close) = match group.delimiter() {
                    proc_macro2::Delimiter::Brace => ("{", "}"),
                    proc_macro2::Delimiter::Bracket => ("[", "]"),
                    proc_macro2::Delimiter::Parenthesis => ("(", ")"),
                    proc_macro2::Delimiter::None => {
                        panic!("native file input has no invisible group")
                    }
                };
                assert_eq!(source.get(group.span_open().byte_range()), Some(open));
                assert_eq!(source.get(group.span_close().byte_range()), Some(close));
                check_tokens(source, group.stream());
            }
            token => assert_eq!(
                source.get(token.span().byte_range()),
                Some(token.to_string().as_str())
            ),
        }
    }
}

fn shared_tokens_keep_original_prefix_positions() {
    use raf_source::{Budget, LoadErrorKind, RelPath, parse_source_with_tokens};
    use std::sync::atomic::{AtomicBool, Ordering};
    use syn::spanned::Spanned;

    let path = RelPath::new("original.rs").unwrap();
    for (prefix, header) in [
        ("", None),
        ("\u{feff}", None),
        ("#!fixture-data\n", Some("#!fixture-data")),
        (
            "\u{feff}#! fixture données\r\n",
            Some("#! fixture données\r"),
        ),
        (
            "#! /* nested /* ordinary */ comment */ [allow(unused)]\n",
            None,
        ),
        ("#! // ordinary\n[allow(unused)]\n", None),
        ("#!\u{200e}[allow(unused)]\n", None),
        (
            "#!/** documentation */ ignored\n",
            Some("#!/** documentation */ ignored"),
        ),
        ("#!/// documentation\n", Some("#!/// documentation")),
    ] {
        let source = format!("{prefix}fn café(input:i32) {{ input + 1; }}");
        parse_source_with_tokens(&path, &source, |syntax, original| {
            assert_eq!(syntax.shebang.as_deref(), header);
            let syn::Item::Fn(function) = &syntax.items[0] else {
                panic!("fixture function");
            };
            assert_eq!(
                source.get(function.sig.ident.span().byte_range()),
                Some("café")
            );
            assert_eq!(
                source.get(function.block.span().byte_range()),
                Some("{ input + 1; }")
            );
            check_tokens(&source, original);
        })
        .unwrap();
    }
    // Ignored header delimiters never spend a Rust file's recursion grant.
    let source = format!(
        "#!fixture-data {}\nfn valid() {{}}",
        "[".repeat(Budget::Delimiters.limit() + 1)
    );
    assert_eq!(
        parse_source_with_tokens(&path, &source, |syntax, _| syntax.items.len()).unwrap(),
        1
    );
    for source in ["#!only-header", "\u{feff}#!only-header"] {
        parse_source_with_tokens(&path, source, |syntax, tokens| {
            assert!(syntax.items.is_empty());
            assert!(tokens.is_empty());
            assert_eq!(syntax.shebang.as_deref(), Some("#!only-header"));
        })
        .unwrap();
    }
    let called = AtomicBool::new(false);
    for source in [
        "#!fixture-data\nfn {",
        "\u{feff}#!fixture-data\r\n\r\nfn 1 {}",
    ] {
        let error =
            parse_source_with_tokens(&path, source, |_, _| called.store(true, Ordering::SeqCst))
                .unwrap_err();
        assert!(matches!(error.kind, LoadErrorKind::Syntax { .. }));
        assert!(!called.load(Ordering::SeqCst));
    }
    let deep = format!(
        "fn invalid() {{ let _ = {}1{}; }}",
        "(".repeat(Budget::Delimiters.limit()),
        ")".repeat(Budget::Delimiters.limit())
    );
    let error = parse_source_with_tokens(&path, &deep, |_, _| called.store(true, Ordering::SeqCst))
        .unwrap_err();
    assert!(matches!(
        error.kind,
        LoadErrorKind::Budget {
            budget: Budget::Delimiters,
            ..
        }
    ));
    assert!(!called.load(Ordering::SeqCst));
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
        .resolve("raf_okf::body::blocks::Line")
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(resolution.path, "raf_okf::body::blocks::Line");
    assert_eq!(
        ws.item(resolution.sites[0]).expect("item").file.as_str(),
        "crates/raf-okf/src/body/blocks.rs"
    );
    assert_eq!(
        ws.resolve("raf_okf::body::Line").map(|r| r.path),
        Err(ResolveError::NotFound("raf_okf::body::Line".to_owned()))
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
