// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Literal captures must enter the real reachability graph with actual owners.

use raf_source::{Budget, Edge, LoadErrorKind, MemoryTree, Namespace, Node, ReachError, Workspace};
use std::fmt::Write as _;

mod support;

fn load(text: &str) -> Workspace {
    Workspace::load(
        &MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n",
            ),
            ("src/lib.rs", text),
        ]),
        &["."],
    )
    .unwrap_or_else(|error| panic!("{error}"))
}

fn value(workspace: &Workspace, path: &str) -> Node {
    Node::Item(
        workspace
            .resolve_in(&format!("demo::{path}"), Namespace::Value)
            .expect(path)
            .sites[0],
    )
}

fn captured(workspace: &Workspace, root: &str, name: &str, expected: bool) {
    let root = value(workspace, root);
    let reach = workspace.reachable(&[root]).expect("reachable");
    let capture = value(workspace, name);
    assert_eq!(reach.contains(capture), expected, "{root:?} -> {name}");
    if expected {
        let found = reach.get(capture).expect("the capture");
        assert_eq!((found.edge, found.from), (Edge::References, Some(root)));
    }
}

#[test]
fn a_cfg_alternative_sibling_alias_does_not_move_the_declared_modules_prelude_attribute() {
    let workspace = load(
        "pub mod b { pub const VALUE:u32=7; pub fn f(){format!(\"{VALUE}\");} } \
         #[cfg(any())] #[no_implicit_prelude] mod a {} \
         #[cfg(not(any()))] use self::b as a; \
         #[no_implicit_prelude] mod isolated { pub const VALUE:u32=7; \
         pub fn f(){format!(\"{VALUE}\");} }",
    );
    captured(&workspace, "b::f", "b::VALUE", true);
    captured(&workspace, "isolated::f", "isolated::VALUE", false);
}

#[test]
fn implicit_captures_and_named_counts_are_value_edges_of_the_actual_function() {
    let source = r#"
const VALUE: f64 = 3.0;
const WIDTH: usize = 8;
const PRECISION: usize = 2;
pub fn implicit() -> String { format!("{VALUE:WIDTH$.PRECISION$}") }
pub fn qualified() -> String { std::format!("{VALUE}") }
pub fn absolute() -> String { ::std::format!("{VALUE}") }
pub fn escaped() -> String { format!("{{VALUE}}") }
pub fn ordinary() -> &'static str { "{VALUE}" }
pub fn positional() -> String { format!("{:8.2}", 3.0) }
"#;
    let diagnostics = support::rustc::compile(source);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.level != "error"),
        "{diagnostics:?}"
    );
    let workspace = load(source);
    for name in ["VALUE", "WIDTH", "PRECISION"] {
        captured(&workspace, "implicit", name, true);
    }
    for root in ["qualified", "absolute"] {
        captured(&workspace, root, "VALUE", true);
    }
    for root in ["escaped", "ordinary", "positional"] {
        captured(&workspace, root, "VALUE", false);
    }
}

#[test]
fn named_bindings_suppress_capture_and_left_hand_path_without_hiding_the_expression() {
    let source = r#"
const VALUE: u32 = 7;
const OTHER: u32 = 11;
pub fn named() -> String { format!("{VALUE}", VALUE = OTHER) }
pub fn direct() -> String { format!("{}", VALUE) }
pub fn complex() -> String { format!("{VALUE}", VALUE = (|x, y| x + y)(OTHER, 1)) }
"#;
    let diagnostics = support::rustc::compile(source);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.level != "error"),
        "{diagnostics:?}"
    );
    let workspace = load(source);
    for root in ["named", "complex"] {
        captured(&workspace, root, "VALUE", false);
        captured(&workspace, root, "OTHER", true);
    }
    captured(&workspace, "direct", "VALUE", true);
    captured(&workspace, "direct", "OTHER", false);
}

#[test]
fn direct_import_aliases_and_block_imports_are_positive_provenance() {
    let source = r#"
use std::format as render;
const VALUE: u32 = 7;
pub fn alias() -> String { render!("{VALUE}") }
pub fn local() -> String { let out = local_render!("{VALUE}"); use std::format as local_render; out }
mod child {
    pub fn not_inherited() { render!("{VALUE}"); }
    pub fn own() -> String { std::format!("{VALUE}", VALUE = super::VALUE) }
}
"#;
    let workspace = load(source);
    for root in ["alias", "local"] {
        captured(&workspace, root, "VALUE", true);
    }
    captured(&workspace, "child::not_inherited", "VALUE", false);
    captured(&workspace, "child::own", "VALUE", true);
}

#[test]
fn textual_macro_shadows_begin_after_the_declaration_and_do_not_leak_from_a_block() {
    let source = r#"
const VALUE: u32 = 7;
pub fn before() -> String { format!("{VALUE}") }
macro_rules! format { ($($input:tt)*) => { String::new() } }
pub fn after() -> String { format!("{VALUE}") }
pub fn qualified() -> String { std::format!("{VALUE}") }
pub fn block() -> String {
    { macro_rules! format { ($($input:tt)*) => { String::new() } } let _ = format!("{VALUE}"); }
    std::format!("{VALUE}")
}
"#;
    let diagnostics = support::rustc::compile(source);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.level != "error"),
        "{diagnostics:?}"
    );
    let workspace = load(source);
    captured(&workspace, "before", "VALUE", true);
    captured(&workspace, "after", "VALUE", false);
    captured(&workspace, "qualified", "VALUE", true);
    captured(&workspace, "block", "VALUE", true);
}

#[test]
fn unknown_named_and_glob_imports_never_become_standard_macro_certificates() {
    for import in ["use vendor::format;", "use vendor::*;"] {
        let workspace = load(&format!(
            "{import}\nconst VALUE:u32=7; pub fn f() {{ format!(\"{{VALUE}}\"); }}"
        ));
        captured(&workspace, "f", "VALUE", false);
    }
    let workspace = load(
        "const VALUE:u32=7; mod data { pub const OTHER:u32=1; } use data::*; pub fn f()->String { format!(\"{VALUE}\") }",
    );
    captured(&workspace, "f", "VALUE", true);
}

#[test]
fn prelude_changes_and_std_head_shadows_refuse_inferred_captures() {
    for attribute in [
        "#![no_std]",
        "#![no_implicit_prelude]",
        "#![cfg_attr(any(),no_implicit_prelude)]",
    ] {
        let workspace = load(&format!(
            "{attribute}\nconst VALUE:u32=7; pub fn f() {{ format!(\"{{VALUE}}\"); }}"
        ));
        captured(&workspace, "f", "VALUE", false);
    }
    let workspace = load(
        "const VALUE:u32=7; mod std {} pub fn f() { std::format!(\"{VALUE}\"); } pub fn absolute()->String { ::std::format!(\"{VALUE}\") }",
    );
    captured(&workspace, "f", "VALUE", false);
    captured(&workspace, "absolute", "VALUE", true);
    let workspace = load(
        "extern crate core as std; const VALUE:u32=7; pub fn absolute(){ ::std::format!(\"{VALUE}\"); }",
    );
    captured(&workspace, "absolute", "VALUE", false);
}

#[test]
fn cooked_raw_unicode_keyword_and_count_bindings_use_decoded_names() {
    let source = r##"
const VALUE: u32 = 7;
const r#type: u32 = 11;
const Δ: u32 = 13;
const WIDTH: usize = 8;
const OTHER: usize = 2;
pub fn cooked() -> String { format!("\u{7b}VALUE\u{7d}") }
pub fn raw() -> String { format!(r#"{VALUE}"#) }
pub fn keyword() -> String { format!("{type}") }
pub fn unicode() -> String { format!("{Δ}") }
pub fn count() -> String { format!("{VALUE:WIDTH$}", WIDTH = OTHER) }
pub fn escaped() -> String { format!("\\u0085 {{VALUE}}") }
"##;
    let diagnostics = support::rustc::compile(source);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.level != "error"),
        "{diagnostics:?}"
    );
    let workspace = load(source);
    for root in ["cooked", "raw", "count"] {
        captured(&workspace, root, "VALUE", true);
    }
    captured(&workspace, "keyword", "type", true);
    captured(&workspace, "unicode", "Δ", true);
    captured(&workspace, "count", "WIDTH", false);
    captured(&workspace, "count", "OTHER", true);
    captured(&workspace, "escaped", "VALUE", false);
}

#[test]
fn malformed_standard_format_input_is_actionable_while_a_custom_macro_keeps_its_grammar() {
    let workspace =
        load("pub fn bad() { format!(\"{\"); } pub fn good()->String { format!(\"{{\") }");
    let error = workspace
        .reachable(&[value(&workspace, "bad")])
        .expect_err("an invalid standard format field");
    assert!(error.to_string().contains("format fields"), "{error}");
    assert!(workspace.reachable(&[value(&workspace, "good")]).is_ok());
    let workspace =
        load("macro_rules! format {($($t:tt)*)=>{()}} pub fn custom(){format!(\"{\");}");
    assert!(workspace.reachable(&[value(&workspace, "custom")]).is_ok());
}

#[test]
fn conditional_or_module_prelude_changes_do_not_disable_an_unrelated_sibling() {
    let workspace = load(
        "const VALUE:u32=7; #[no_implicit_prelude] mod isolated { pub fn f(){format!(\"{VALUE}\");} } mod regular { pub fn f()->String {format!(\"{VALUE}\",VALUE=super::VALUE)} }",
    );
    captured(&workspace, "isolated::f", "VALUE", false);
    captured(&workspace, "regular::f", "VALUE", true);
}

#[test]
fn nested_standard_expression_macros_are_transparent_but_custom_token_inputs_are_not_calls() {
    let source = r#"
const VALUE:u32=7;
macro_rules! discard { ($($input:tt)*) => { () } }
pub fn discarded() { discard!(format!("{VALUE}")); }
pub fn discarded_invalid() { discard!(format!("{")); }
pub fn asserted() { assert_eq!(format!("{VALUE}"), "7"); }
pub fn nested() -> String { format!("{}", format!("{VALUE}")) }
pub fn custom_shadow() { macro_rules! assert_eq { ($($input:tt)*) => { () } } assert_eq!(format!("{VALUE}"), "7"); }
"#;
    let diagnostics = support::rustc::compile(source);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.level != "error"),
        "{diagnostics:?}"
    );
    let workspace = load(source);
    for root in ["discarded", "discarded_invalid", "custom_shadow"] {
        captured(&workspace, root, "VALUE", false);
    }
    for root in ["asserted", "nested"] {
        captured(&workspace, root, "VALUE", true);
    }
}

#[test]
fn block_local_std_aliases_are_checked_at_the_imports_own_scope() {
    let source = r#"
const VALUE:u32=7;
macro_rules! custom_format { ($($input:tt)*)=>{String::new()} }
mod custom { pub(crate) use custom_format as format; }
pub fn custom() -> String {
    use crate::custom as std;
    use std::format as render;
    render!("{VALUE}")
}

pub fn standard() -> String {
    use std::format as render;
    { mod std {} render!("{VALUE}") }
}
"#;
    let diagnostics = support::rustc::compile(source);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.level != "error"),
        "{diagnostics:?}"
    );
    let workspace = load(source);
    captured(&workspace, "custom", "VALUE", false);
    captured(&workspace, "standard", "VALUE", true);
}

#[test]
fn recursive_format_expressions_are_admitted_before_parsing_and_custom_tokens_stay_opaque() {
    let expressions = |operators| format!("{}0", "0 + ".repeat(operators));
    let source = format!(
        "pub fn refused()->String {{ format!(\"{{}}\", {}) }}\npub fn admitted()->String {{ format!(\"{{}}\", {}) }}\nmacro_rules! custom {{($($t:tt)*)=>{{()}}}} pub fn opaque(){{custom!(\"{{}}\", {});}}",
        expressions(Budget::Recursion.limit() + 1),
        expressions(Budget::Recursion.limit()),
        expressions(Budget::Recursion.limit() + 1),
    );
    let workspace = load(&source);
    let error = workspace
        .reachable(&[value(&workspace, "refused")])
        .expect_err("one excess recursion unit");
    match error {
        ReachError::Source(error) => {
            assert_eq!(error.file.as_str(), "src/lib.rs");
            assert_eq!(error.line, Some(1));
            assert!(
                matches!(error.kind, LoadErrorKind::Budget { budget: Budget::Recursion, limit } if limit == Budget::Recursion.limit())
            );
        }
        other => panic!("unexpected refusal: {other}"),
    }
    for root in ["admitted", "opaque"] {
        assert!(
            workspace.reachable(&[value(&workspace, root)]).is_ok(),
            "{root}"
        );
    }
}

#[test]
fn one_physical_file_is_recognized_per_target_module_edition_and_root_order() {
    let tree = MemoryTree::from([
        ("Cargo.toml", "[workspace]\nmembers=['old','new']\n"),
        (
            "old/Cargo.toml",
            "[package]\nname='old'\nversion='0.1.0'\nedition='2015'\n",
        ),
        (
            "new/Cargo.toml",
            "[package]\nname='new'\nversion='0.1.0'\nedition='2024'\n",
        ),
        (
            "old/src/lib.rs",
            "#![no_std]\n#[path=\"../../shared.rs\"] pub mod shared;",
        ),
        (
            "new/src/lib.rs",
            "#[path=\"../../shared.rs\"] pub mod shared;",
        ),
        (
            "shared.rs",
            "pub const VALUE:u32=7; pub fn f(){format!(\"{VALUE}\");}",
        ),
    ]);
    for roots in [
        vec!["old"],
        vec!["new"],
        vec!["old", "new"],
        vec!["new", "old"],
    ] {
        let workspace = Workspace::load(&tree, &roots).expect("shared source");
        for (package, expected) in [("old", false), ("new", true)] {
            if !roots.contains(&package) {
                continue;
            }
            let resolve = |tail| {
                Node::Item(
                    workspace
                        .resolve_in(&format!("{package}::shared::{tail}"), Namespace::Value)
                        .expect("shared value")
                        .sites[0],
                )
            };
            let reach = workspace.reachable(&[resolve("f")]).expect("shared reach");
            assert_eq!(
                reach.contains(resolve("VALUE")),
                expected,
                "{roots:?}: {package}"
            );
        }
    }
}

#[test]
fn a_dependency_named_std_and_a_resolved_workspace_macro_are_not_the_sysroot() {
    let tree = MemoryTree::from([
        ("Cargo.toml", "[workspace]\nmembers=['app','vendor']\n"),
        (
            "app/Cargo.toml",
            "[package]\nname='app'\nversion='0.1.0'\nedition='2021'\n[dependencies]\nstd={package='vendor',path='../vendor'}\n",
        ),
        (
            "vendor/Cargo.toml",
            "[package]\nname='vendor'\nversion='0.1.0'\nedition='2021'\n",
        ),
        (
            "vendor/src/lib.rs",
            "#[macro_export] macro_rules! format {($($t:tt)*)=>{()}}",
        ),
        (
            "app/src/lib.rs",
            "pub const VALUE:u32=7; pub fn f(){std::format!(\"{VALUE}\");} pub fn absolute(){::std::format!(\"{VALUE}\");}",
        ),
    ]);
    let workspace = Workspace::load(&tree, &["."]).expect("dependency std");
    let value = |name| {
        Node::Item(
            workspace
                .resolve_in(&format!("app::{name}"), Namespace::Value)
                .expect(name)
                .sites[0],
        )
    };
    for root in ["f", "absolute"] {
        assert!(
            !workspace
                .reachable(&[value(root)])
                .expect("reachability")
                .contains(value("VALUE")),
            "{root}"
        );
    }
    let workspace = load(
        "const VALUE:u32=7; #[macro_export] macro_rules! custom {($($t:tt)*)=>{()}} mod macros {pub use crate::custom as format;} use macros::*; pub fn f(){format!(\"{VALUE}\");}",
    );
    captured(&workspace, "f", "VALUE", false);
}

#[test]
fn every_admitted_enclosing_standard_macro_accepts_nested_format_expressions() {
    let mut source = String::from("use std::fmt::Write; const VALUE:u32=7;\n");
    let expressions = [
        ("format", "format!(\"{}\",format!(\"{VALUE}\"))"),
        ("format_args", "format_args!(\"{}\",format!(\"{VALUE}\"))"),
        ("print", "print!(\"{}\",format!(\"{VALUE}\"))"),
        ("println", "println!(\"{}\",format!(\"{VALUE}\"))"),
        ("eprint", "eprint!(\"{}\",format!(\"{VALUE}\"))"),
        ("eprintln", "eprintln!(\"{}\",format!(\"{VALUE}\"))"),
        (
            "write",
            "write!(&mut String::new(),\"{}\",format!(\"{VALUE}\"))",
        ),
        (
            "writeln",
            "writeln!(&mut String::new(),\"{}\",format!(\"{VALUE}\"))",
        ),
        ("panic", "panic!(\"{}\",format!(\"{VALUE}\"))"),
        ("assert", "assert!(format!(\"{VALUE}\")==\"7\")"),
        ("assert_eq", "assert_eq!(format!(\"{VALUE}\"),\"7\")"),
        ("assert_ne", "assert_ne!(format!(\"{VALUE}\"),\"8\")"),
        ("debug_assert", "debug_assert!(format!(\"{VALUE}\")==\"7\")"),
        (
            "debug_assert_eq",
            "debug_assert_eq!(format!(\"{VALUE}\"),\"7\")",
        ),
        (
            "debug_assert_ne",
            "debug_assert_ne!(format!(\"{VALUE}\"),\"8\")",
        ),
        ("vec", "vec![format!(\"{VALUE}\")]"),
    ];
    for (name, expression) in expressions {
        writeln!(source, "pub fn probe_{name}() {{ {expression}; }}").expect("writing a String");
    }
    let diagnostics = support::rustc::compile(&source);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.level != "error"),
        "{diagnostics:?}"
    );
    let workspace = load(&source);
    for (name, _) in expressions {
        captured(&workspace, &format!("probe_{name}"), "VALUE", true);
    }
}

#[test]
fn a_parent_files_textual_macro_also_shadows_an_imported_standard_alias() {
    let workspace = Workspace::load(&MemoryTree::from([
        ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n"),
        ("src/lib.rs", "macro_rules! render {($($t:tt)*)=>{()}} pub mod child;"),
        ("src/child.rs", "use std::format as render; pub const VALUE:u32=7; pub fn f(){render!(\"{VALUE}\");} pub fn explicit()->String{std::format!(\"{VALUE}\")}"),
    ]), &["."]).expect("external child");
    captured(&workspace, "child::f", "child::VALUE", false);
    captured(&workspace, "child::explicit", "child::VALUE", true);
}
