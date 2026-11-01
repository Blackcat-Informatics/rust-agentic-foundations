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

fn query_budget(error: &CodeResolveError) -> Option<Budget> {
    match error {
        CodeResolveError::Resolve(raf_source::ResolveError::Budget { budget, .. }) => Some(*budget),
        CodeResolveError::Query(error) => match error.kind {
            LoadErrorKind::Budget { budget, .. } => Some(budget),
            _ => None,
        },
        _ => None,
    }
}

#[test]
fn external_library_query_scratch_uses_the_original_native_grants() {
    let files = MemoryTree::from([
        ("Cargo.toml", "[workspace]\nmembers=['consumer','provider']"),
        (
            "consumer/Cargo.toml",
            "[package]\nname='consumer'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nprovider={path='../provider'}",
        ),
        (
            "consumer/src/lib.rs",
            "extern crate provider; pub fn reader(){}",
        ),
        (
            "provider/Cargo.toml",
            "[package]\nname='provider'\nversion='1.0.0'\nedition='2024'",
        ),
        ("provider/src/lib.rs", "pub fn selected(){}"),
    ]);
    let baseline = Workspace::load(&files, &["."]).unwrap();
    let allowance = baseline.code_resolver().unwrap().usage();
    let check = |options: &LoadOptions| {
        let workspace = Workspace::load_with(&files, &["."], options).unwrap();
        let reader = workspace
            .items()
            .find(|item| item.path == "consumer::reader")
            .unwrap()
            .id;
        let byte = workspace
            .item(reader)
            .unwrap()
            .body
            .as_ref()
            .unwrap()
            .start_byte
            + 1;
        let code = workspace.code_resolver().unwrap();
        code.extern_libraries_at(reader, byte)
            .unwrap()
            .find(|library| library.rust_name() == "provider")
            .unwrap()
            .resolve("selected", Namespace::Value)
    };
    assert!(check(&LoadOptions::default()).is_ok());
    for budget in [Budget::Work, Budget::ItemBytes] {
        let mut options = LoadOptions::default();
        match budget {
            Budget::Work => options.max_query_work = 0,
            Budget::ItemBytes => options.max_item_bytes = allowance.item_bytes,
            _ => unreachable!(),
        }
        let error = check(&options).unwrap_err();
        assert!(
            matches!(error, CodeResolveError::Query(ref error) if matches!(error.kind, LoadErrorKind::Budget { budget: actual, .. } if actual == budget)),
            "{error}"
        );
    }
}

#[test]
fn external_library_receipts_keep_actual_names_exports_and_definition_identity() {
    for (dependency, rust_name) in [
        ("renamed={package='provider',path='../provider'}", "renamed"),
        ("provider={path='../provider'}", "native_library"),
    ] {
        let manifest = format!(
            "[package]\nname='consumer'\nversion='1.0.0'\nedition='2024'\n[dependencies]\n{dependency}"
        );
        let source = format!("extern crate {rust_name} as external; pub fn reader(){{}}");
        let files = MemoryTree::from([
            ("Cargo.toml", "[workspace]\nmembers=['consumer','provider']"),
            ("consumer/Cargo.toml", manifest.as_str()),
            ("consumer/src/lib.rs", source.as_str()),
            (
                "provider/Cargo.toml",
                "[package]\nname='provider'\nversion='1.0.0'\nedition='2024'\n[lib]\nname='native_library'",
            ),
            (
                "provider/src/lib.rs",
                "mod implementation { pub fn selected(){} pub fn concealed(){} } pub use implementation::selected; #[doc(hidden)] pub fn hidden(){}",
            ),
        ]);
        let workspace = Workspace::load(&files, &["."]).unwrap();
        let reader = owner(&workspace, "consumer::reader");
        let byte = workspace
            .item(reader)
            .unwrap()
            .body
            .as_ref()
            .unwrap()
            .start_byte
            + 1;
        let code = workspace.code_resolver().unwrap();
        let mut names = Vec::new();
        for library in code.extern_libraries_at(reader, byte).unwrap() {
            names.push(library.rust_name().to_owned());
            assert_eq!(
                workspace.package(library.package()).unwrap().name,
                "provider"
            );
            assert_eq!(
                workspace.target(library.target()).unwrap().crate_path,
                "native_library"
            );
            let public = library.resolve("selected", Namespace::Value).unwrap();
            let defining = library
                .resolve("implementation::selected", Namespace::Value)
                .unwrap();
            assert_eq!(public.sites, defining.sites);
            assert_eq!(public.sites.len(), 1);
            assert!(library.resolve("hidden", Namespace::Value).is_ok());
            assert!(matches!(
                library.resolve("implementation::concealed", Namespace::Value),
                Err(CodeResolveError::NotExported { .. })
            ));
        }
        names.sort();
        assert_eq!(names, ["external", rust_name]);
    }
}

#[test]
fn library_receipts_preserve_effective_name_ambiguity_and_deduplicate_one_root() {
    for (source, symbol, ambiguous) in [
        (
            "extern crate other as same; pub fn reader(){}",
            "consumer::reader",
            true,
        ),
        (
            "extern crate same; pub fn reader(){}",
            "consumer::reader",
            false,
        ),
        (
            "pub mod child {extern crate other as same; pub fn reader(){}}",
            "consumer::child::reader",
            false,
        ),
    ] {
        let files = MemoryTree::from([
            (
                "Cargo.toml",
                "[workspace]\nmembers=['consumer','provider','competitor']",
            ),
            (
                "consumer/Cargo.toml",
                "[package]\nname='consumer'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nsame={package='provider',path='../provider'}\nother={package='competitor',path='../competitor'}",
            ),
            ("consumer/src/lib.rs", source),
            (
                "provider/Cargo.toml",
                "[package]\nname='provider'\nversion='1.0.0'\nedition='2024'",
            ),
            ("provider/src/lib.rs", "pub fn selected(){}"),
            (
                "competitor/Cargo.toml",
                "[package]\nname='competitor'\nversion='1.0.0'\nedition='2024'",
            ),
            ("competitor/src/lib.rs", "pub fn selected(){}"),
        ]);
        let workspace = Workspace::load(&files, &["."]).unwrap();
        let reader = owner(&workspace, symbol);
        let byte = workspace
            .item(reader)
            .unwrap()
            .body
            .as_ref()
            .unwrap()
            .start_byte
            + 1;
        let code = workspace.code_resolver().unwrap();
        let mut observed = 0;
        for library in code
            .extern_libraries_at(reader, byte)
            .unwrap()
            .filter(|library| library.rust_name() == "same")
        {
            observed += 1;
            let result = library.resolve("selected", Namespace::Value);
            if ambiguous && library.route_kind() == raf_source::ExternRouteKind::Prelude {
                assert!(
                    matches!(result, Err(CodeResolveError::Resolve(raf_source::ResolveError::Ambiguous(ref sites))) if sites.len() == 2),
                    "{result:?}"
                );
            } else {
                assert!(result.is_ok(), "{result:?}");
            }
        }
        assert!(observed > 0);
    }
}

#[test]
fn code_associated_calls_prefer_inherent_methods_without_erasing_symbol_inventory() {
    let source = "pub trait Tr{fn run()->u32;} pub struct S; impl Tr for S{fn run()->u32{1}} impl S{pub fn run()->u32{2}} pub struct OnlyTrait; impl Tr for OnlyTrait{fn run()->u32{3}} pub fn reader(){S::run(); OnlyTrait::run();}";
    let workspace = Workspace::load(&tree("2024", source), &["."]).unwrap();
    assert!(matches!(
        workspace.resolve_in("demo::S::run", Namespace::Value),
        Err(raf_source::ResolveError::Ambiguous(_))
    ));
    let reader = owner(&workspace, "demo::reader");
    let code = workspace.code_resolver().unwrap();
    for (path, kind) in [
        ("S::run", raf_source::SymbolKind::InherentMethod),
        ("OnlyTrait::run", raf_source::SymbolKind::TraitImplMethod),
    ] {
        let byte = source.find(&format!("{path}();")).unwrap();
        let resolved = code
            .resolve_from_item_at(reader, byte, path, Namespace::Value)
            .unwrap();
        assert_eq!(resolved.sites.len(), 1);
        assert_eq!(workspace.item(resolved.sites[0]).unwrap().kind, kind);
    }
    for source in [
        "pub struct S<T>(T); pub trait Tr{fn run()->u32;} impl Tr for S<u16>{fn run()->u32{1}} impl S<u8>{pub fn run()->u32{2}} pub fn reader(){S::<u16>::run();}",
        "pub struct S<T=u8>(T); pub trait Tr{fn run()->u32;} impl Tr for S<u16>{fn run()->u32{1}} impl S{pub fn run()->u32{2}} pub fn reader(){S::<u16>::run();}",
        "pub struct S; pub type Alias=S; pub trait Tr{fn run()->u32;} impl Tr for S{fn run()->u32{1}} impl Alias{pub fn run()->u32{2}} pub fn reader(){S::run();}",
    ] {
        let workspace = Workspace::load(&tree("2024", source), &["."]).unwrap();
        let reader = owner(&workspace, "demo::reader");
        let byte = workspace
            .item(reader)
            .unwrap()
            .body
            .as_ref()
            .unwrap()
            .start_byte
            + 1;
        assert!(matches!(
            workspace.code_resolver().unwrap().resolve_from_item_at(
                reader,
                byte,
                "S::run",
                Namespace::Value
            ),
            Err(CodeResolveError::Resolve(
                raf_source::ResolveError::Ambiguous(_)
            ))
        ));
    }
}

#[test]
fn alias_members_preserve_definition_chains_cycles_and_original_query_grants() {
    let source = "pub struct State<const N:usize>; impl<const N:usize> State<N>{pub fn read()->usize{N}} pub type A=State<3>; pub type B=A; pub fn reader(){B::read();}";
    let files = tree("2024", source);
    let workspace = Workspace::load(&files, &["."]).unwrap();
    let reader = owner(&workspace, "demo::reader");
    let byte = source.find("B::read();").unwrap();
    let code = workspace.code_resolver().unwrap();
    let resolved = code
        .resolve_from_item_at(reader, byte, "B::read", Namespace::Value)
        .unwrap();
    assert_eq!(resolved.path, "demo::State::read");
    assert_eq!(
        resolved
            .type_aliases
            .iter()
            .map(|id| workspace.item(*id).unwrap().path.as_str())
            .collect::<Vec<_>>(),
        ["demo::B", "demo::A"]
    );
    let mut options = LoadOptions::default();
    options.max_item_bytes = code.usage().item_bytes;
    let bounded = Workspace::load_with(&files, &["."], &options).unwrap();
    let failure = bounded
        .code_resolver()
        .unwrap()
        .resolve_from_item_at(
            owner(&bounded, "demo::reader"),
            byte,
            "B::read",
            Namespace::Value,
        )
        .unwrap_err();
    assert_eq!(query_budget(&failure), Some(Budget::ItemBytes), "{failure}");
    let mut query_options = LoadOptions::default();
    query_options.max_query_work = 0;
    let query_limited = Workspace::load_with(&files, &["."], &query_options).unwrap();
    let query_reader = query_limited
        .items()
        .find(|item| item.path == "demo::reader")
        .unwrap()
        .id;
    let query_failure = query_limited
        .code_resolver()
        .unwrap()
        .resolve_from_item_at(query_reader, byte, "B::read", Namespace::Value)
        .unwrap_err();
    assert_eq!(
        query_budget(&query_failure),
        Some(Budget::Work),
        "{query_failure}"
    );
    for (source, cyclic) in [
        ("pub type A=B; pub type B=A; pub fn reader(){}", true),
        (
            "pub struct State<const N:usize>; impl<const N:usize> State<N>{pub fn read()->usize{N}} pub type A=State<{1+2}>; pub fn reader(){}",
            false,
        ),
    ] {
        let workspace = Workspace::load(&tree("2024", source), &["."]).unwrap();
        let reader = owner(&workspace, "demo::reader");
        let byte = workspace
            .item(reader)
            .unwrap()
            .body
            .as_ref()
            .unwrap()
            .start_byte
            + 1;
        let result = workspace.code_resolver().unwrap().resolve_from_item_at(
            reader,
            byte,
            "A::read",
            Namespace::Value,
        );
        if cyclic {
            assert!(matches!(
                result,
                Err(CodeResolveError::Resolve(raf_source::ResolveError::Cyclic(
                    _
                )))
            ));
        } else {
            assert!(matches!(result, Err(CodeResolveError::AliasMember { .. })));
        }
    }
}

#[test]
fn inherent_method_applicability_is_local_to_the_selected_signature() {
    for (methods, plain) in [
        ("pub fn run(){} pub fn generic<T>(){}", true),
        ("pub fn generic<T>(){} pub fn run(){}", true),
        ("pub fn run(); pub fn generic<T>(){}", true),
        ("pub fn run<T>(){} pub fn ordinary(){}", false),
        ("pub fn run<T>(); pub fn ordinary(){}", false),
        ("pub fn run() where Self:Copy {} pub fn ordinary(){}", false),
    ] {
        let source = format!(
            "#[derive(Clone,Copy)] pub struct S; pub trait Tr{{fn run();}} impl Tr for S{{fn run(){{}}}} impl S{{{methods}}} pub fn reader(){{}}"
        );
        let workspace = Workspace::load(&tree("2024", &source), &["."]).unwrap();
        let reader = owner(&workspace, "demo::reader");
        let byte = workspace
            .item(reader)
            .unwrap()
            .body
            .as_ref()
            .unwrap()
            .start_byte
            + 1;
        let code = workspace.code_resolver().unwrap();
        let method = workspace
            .items()
            .find(|item| item.name == "run" && item.kind == raf_source::SymbolKind::InherentMethod)
            .unwrap()
            .id;
        assert_eq!(
            code.require_unconditional_inherent_method(method).is_ok(),
            plain,
            "{source}"
        );
        let result = code.resolve_from_item_at(reader, byte, "S::run", Namespace::Value);
        if plain {
            assert_eq!(result.unwrap().sites, [method]);
        } else {
            assert!(
                matches!(
                    result,
                    Err(CodeResolveError::Resolve(
                        raf_source::ResolveError::Ambiguous(_)
                    ))
                ),
                "{source}: {result:?}"
            );
        }
    }
}

#[test]
fn complete_module_paths_return_actual_type_sites_without_becoming_values() {
    let workspace = Workspace::load(
        &tree("2024", "pub mod child { pub fn reader() {} }"),
        &["."],
    )
    .unwrap();
    let reader = owner(&workspace, "demo::child::reader");
    let byte = workspace
        .item(reader)
        .unwrap()
        .body
        .as_ref()
        .unwrap()
        .start_byte
        + 1;
    let code = workspace.code_resolver().unwrap();
    for (path, expected) in [
        ("crate", "demo"),
        ("self", "demo::child"),
        ("super", "demo"),
    ] {
        let resolved = code
            .resolve_from_item_at(reader, byte, path, Namespace::Type)
            .unwrap();
        assert_eq!(resolved.path, expected);
        assert_eq!(resolved.sites.len(), 1);
        assert!(
            resolved
                .sites
                .iter()
                .all(|site| workspace.item(*site).unwrap().kind == raf_source::SymbolKind::Mod)
        );
        assert!(
            code.resolve_from_item_at(reader, byte, path, Namespace::Value)
                .is_err()
        );
    }
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
    let reader = owner(&workspace, "demo::reader");
    let item = workspace.item(reader).unwrap();
    assert!(matches!(
        code.with_configured_syntax(item.target, &item.file, |_, _| ()),
        Err(CodeResolveError::SyntaxContext {
            reason: "supply its exact declared configuration",
            ..
        })
    ));
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
