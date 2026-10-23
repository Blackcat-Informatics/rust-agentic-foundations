// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Module-level code lookup retains actual owner context and import differences.

use raf_source::{
    Budget, CodeResolveError, LoadErrorKind, LoadOptions, MemoryTree, Namespace, Workspace,
};

fn tree(edition: &str, source: &str) -> MemoryTree {
    [
        (
            "Cargo.toml",
            format!("[package]\nname='demo'\nversion='0.1.0'\nedition='{edition}'\n"),
        ),
        ("src/lib.rs", source.to_owned()),
    ]
    .into_iter()
    .collect()
}

fn owner(workspace: &Workspace, symbol: &str) -> raf_source::ItemId {
    let resolution = workspace.resolve_in(symbol, Namespace::Value).unwrap();
    assert_eq!(resolution.sites.len(), 1);
    resolution.sites[0]
}

#[test]
fn code_heads_read_the_edition_2015_module_where_use_heads_read_the_crate_root() {
    let source =
        "pub fn selected(){} pub mod child { pub fn selected(){} pub fn reader(){selected();} }";
    let workspace = Workspace::load(&tree("2015", source), &["."]).unwrap();
    let reader = owner(&workspace, "demo::child::reader");
    let byte = source.find("selected();").unwrap();
    let code = workspace.code_resolver().unwrap();
    assert_eq!(code.reading_module(reader, byte).unwrap(), "demo::child");
    assert_eq!(
        code.resolve_from_item_at(reader, byte, "selected", Namespace::Value)
            .unwrap()
            .path,
        "demo::child::selected"
    );
    assert_eq!(
        workspace
            .resolve_from_in("demo::child", "selected", Namespace::Value)
            .unwrap()
            .path,
        "demo::selected"
    );
    assert!(matches!(
        code.resolve_from_item_at(reader, 0, "selected", Namespace::Value),
        Err(CodeResolveError::OutsideItem(_, 0))
    ));
}

#[test]
fn method_and_trait_members_read_their_defining_module_instead_of_their_symbol_parent() {
    let source = "pub mod types { pub struct S; } pub mod engine { use crate::types::S; pub fn chosen(){} impl S { pub fn reader(){chosen();} } pub trait Tr { fn defaulted(){chosen();} } }";
    let workspace = Workspace::load(&tree("2021", source), &["."]).unwrap();
    let code = workspace.code_resolver().unwrap();
    for symbol in ["demo::types::S::reader", "demo::engine::Tr::defaulted"] {
        let reader = owner(&workspace, symbol);
        let byte = workspace
            .item(reader)
            .unwrap()
            .body
            .as_ref()
            .unwrap()
            .start_byte
            + 1;
        assert_eq!(code.reading_module(reader, byte).unwrap(), "demo::engine");
        let engine = workspace
            .resolve_in("demo::engine", Namespace::Type)
            .unwrap()
            .sites[0];
        let types = workspace
            .resolve_in("demo::types", Namespace::Type)
            .unwrap()
            .sites[0];
        assert!(code.is_defined_in(reader, engine).unwrap());
        assert!(!code.is_defined_in(reader, types).unwrap());
        assert_eq!(
            code.resolve_from_item_at(reader, byte, "chosen", Namespace::Value)
                .unwrap()
                .path,
            "demo::engine::chosen"
        );
    }
}

#[test]
fn a_module_body_reads_itself_while_its_declaration_reads_its_parent() {
    let source = "pub fn chosen(){} pub mod child { pub fn chosen(){} invoke!(chosen); }";
    let workspace = Workspace::load(&tree("2021", source), &["."]).unwrap();
    let child = workspace
        .resolve_in("demo::child", Namespace::Type)
        .unwrap()
        .sites[0];
    let code = workspace.code_resolver().unwrap();
    let declaration = workspace.item(child).unwrap().span.start_byte;
    assert_eq!(code.reading_module(child, declaration).unwrap(), "demo");
    let body = source.find("invoke!").unwrap();
    assert_eq!(code.reading_module(child, body).unwrap(), "demo::child");
    assert_eq!(
        code.resolve_from_item_at(child, body, "chosen", Namespace::Value)
            .unwrap()
            .path,
        "demo::child::chosen"
    );
}

#[test]
fn code_context_tables_spend_only_the_original_load_allowance_and_preserve_usage() {
    let files = tree("2021", "pub fn reader(){}");
    let workspace = Workspace::load(&files, &["."]).unwrap();
    let code = workspace.code_resolver().unwrap();
    let mut expected = workspace.usage();
    assert!(code.usage().item_bytes > expected.item_bytes);
    expected.item_bytes = code.usage().item_bytes;
    assert_eq!(code.usage(), expected);
    let mut limits = LoadOptions::default();
    limits.max_item_bytes = workspace.usage().item_bytes;
    let bounded = Workspace::load_with(&files, &["."], &limits).unwrap();
    let Err(failure) = bounded.code_resolver() else {
        panic!("the reading table must not receive a fresh allowance");
    };
    assert!(
        matches!(
            failure.kind,
            LoadErrorKind::Budget {
                budget: Budget::ItemBytes,
                ..
            }
        ),
        "{failure}"
    );
}

#[test]
fn exported_macros_keep_their_lexical_module_beside_their_crate_root_export() {
    let source = "pub fn selected(){} pub mod inner { pub fn selected(){} #[macro_export] macro_rules! exported {()=>{selected();}} macro_rules! ordinary {()=>{selected();}} }";
    let workspace = Workspace::load(&tree("2021", source), &["."]).unwrap();
    let code = workspace.code_resolver().unwrap();
    for symbol in ["demo::exported", "demo::inner::ordinary"] {
        let id = workspace
            .resolve_in(symbol, Namespace::Macro)
            .unwrap()
            .sites[0];
        let byte = workspace.item(id).unwrap().span.start_byte;
        assert_eq!(code.reading_module(id, byte).unwrap(), "demo::inner");
        let inner = workspace
            .resolve_in("demo::inner", Namespace::Type)
            .unwrap()
            .sites[0];
        assert!(code.is_defined_in(id, inner).unwrap());
        assert_eq!(
            code.resolve_from_item_at(id, byte, "selected", Namespace::Value)
                .unwrap()
                .path,
            "demo::inner::selected"
        );
        let byte = workspace
            .item(id)
            .unwrap()
            .body
            .as_ref()
            .unwrap()
            .start_byte
            + 1;
        assert!(
            matches!(code.resolve_from_item_at(id, byte, "selected", Namespace::Value),
            Err(CodeResolveError::MacroDefinition(found)) if found == id)
        );
    }
}

#[test]
fn pattern_binding_observations_exclude_names_introduced_by_guard_expressions() {
    let source = "fn reader(value:Option<u8>){match value {Some(bound) if {let leaked=1; leaked==bound} => (), _=>()}}";
    let names = raf_source::parse_source(
        &raf_source::RelPath::new("src/lib.rs").unwrap(),
        source,
        |file| {
            let syn::Item::Fn(function) = &file.items[0] else {
                panic!("function fixture")
            };
            let syn::Stmt::Expr(syn::Expr::Match(expression), _) = &function.block.stmts[0] else {
                panic!("match fixture")
            };
            let mut names = Vec::new();
            raf_source::visit_pattern_bindings(&expression.arms[0].pat, |name| {
                names.push(name.to_string());
            });
            names
        },
    )
    .unwrap();
    assert_eq!(names, ["bound"]);
}
