// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The symbol index: items, their kinds and fields, impl members, accessors
//! and determinism.

use raf_source::{
    Item, LoadError, LoadErrorKind, MemoryTree, Namespace, RelPath, ResolveError, SymbolKind,
    TargetKind, TestRole, Vis, Workspace,
};

const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";

fn load(files: &[(&str, &str)]) -> Result<Workspace, LoadError> {
    let mut all = vec![("Cargo.toml", DEMO)];
    all.extend_from_slice(files);
    let tree: MemoryTree = all.into_iter().collect();
    Workspace::load(&tree, &["."])
}

fn ok(files: &[(&str, &str)]) -> Workspace {
    load(files).unwrap_or_else(|e| panic!("{e}"))
}

fn lib(text: &str) -> Workspace {
    ok(&[("src/lib.rs", text)])
}

fn item<'a>(ws: &'a Workspace, path: &str, namespace: Namespace) -> &'a Item {
    let resolution = ws
        .resolve_in(path, namespace)
        .unwrap_or_else(|e| panic!("{path}: {e}"));
    ws.item(resolution.sites[0]).expect("the item exists")
}

const KINDS: &str = "pub fn f() {}\n\
pub(crate) struct S { x: u8 }\n\
struct Unit;\n\
pub struct Tup(u8);\n\
pub enum E {}\n\
pub union U { a: u8 }\n\
pub trait T {\n    fn req(&self);\n    fn dflt(&self) {}\n    const K: u8;\n    type A;\n}\n\
pub type Al = u8;\n\
pub const C: u8 = 1;\n\
pub static ST: u8 = 1;\n\
pub mod m {}\n\
macro_rules! private_mac { () => {} }\n\
#[macro_export]\nmacro_rules! exported { () => {} }\n\
extern crate alloc;\n\
extern \"C\" {\n    fn foreign_fn();\n    static FS: u8;\n    type FT;\n}\n\
mod hidden { #[macro_export] macro_rules! deep { () => {} } }\n";

type KindCase = (&'static str, Namespace, SymbolKind, usize, Vis);

#[allow(clippy::too_many_lines, reason = "the function is one table of cases")]
fn kind_cases() -> Vec<KindCase> {
    vec![
        ("demo::f", Namespace::Value, SymbolKind::Fn, 1, Vis::Public),
        (
            "demo::S",
            Namespace::Type,
            SymbolKind::Struct,
            2,
            Vis::Restricted("crate".to_owned()),
        ),
        (
            "demo::Unit",
            Namespace::Type,
            SymbolKind::Struct,
            3,
            Vis::Inherited,
        ),
        (
            "demo::Tup",
            Namespace::Type,
            SymbolKind::Struct,
            4,
            Vis::Public,
        ),
        ("demo::E", Namespace::Type, SymbolKind::Enum, 5, Vis::Public),
        (
            "demo::U",
            Namespace::Type,
            SymbolKind::Union,
            6,
            Vis::Public,
        ),
        (
            "demo::T",
            Namespace::Type,
            SymbolKind::Trait,
            7,
            Vis::Public,
        ),
        (
            "demo::T::req",
            Namespace::Value,
            SymbolKind::TraitRequiredMethod,
            8,
            Vis::Inherited,
        ),
        (
            "demo::T::dflt",
            Namespace::Value,
            SymbolKind::TraitDefaultMethod,
            9,
            Vis::Inherited,
        ),
        (
            "demo::T::K",
            Namespace::Value,
            SymbolKind::AssocConst,
            10,
            Vis::Inherited,
        ),
        (
            "demo::T::A",
            Namespace::Type,
            SymbolKind::AssocType,
            11,
            Vis::Inherited,
        ),
        (
            "demo::Al",
            Namespace::Type,
            SymbolKind::TypeAlias,
            13,
            Vis::Public,
        ),
        (
            "demo::C",
            Namespace::Value,
            SymbolKind::Const,
            14,
            Vis::Public,
        ),
        (
            "demo::ST",
            Namespace::Value,
            SymbolKind::Static,
            15,
            Vis::Public,
        ),
        ("demo::m", Namespace::Type, SymbolKind::Mod, 16, Vis::Public),
        (
            "demo::private_mac",
            Namespace::Macro,
            SymbolKind::MacroRules,
            17,
            Vis::Inherited,
        ),
        (
            "demo::exported",
            Namespace::Macro,
            SymbolKind::MacroRules,
            19,
            Vis::Inherited,
        ),
        (
            "demo::alloc",
            Namespace::Type,
            SymbolKind::ExternCrate,
            20,
            Vis::Inherited,
        ),
        (
            "demo::foreign_fn",
            Namespace::Value,
            SymbolKind::ForeignFn,
            22,
            Vis::Inherited,
        ),
        (
            "demo::FS",
            Namespace::Value,
            SymbolKind::ForeignStatic,
            23,
            Vis::Inherited,
        ),
        (
            "demo::FT",
            Namespace::Type,
            SymbolKind::ForeignType,
            24,
            Vis::Inherited,
        ),
        // An exported macro is indexed at the crate root, wherever it is written.
        (
            "demo::deep",
            Namespace::Macro,
            SymbolKind::MacroRules,
            26,
            Vis::Inherited,
        ),
    ]
}

#[test]
fn every_kind_is_indexed_with_its_kind_line_file_and_visibility() {
    let ws = lib(KINDS);
    let file = RelPath::new("src/lib.rs").expect("path");
    let cases = kind_cases();
    for (path, namespace, kind, line, visibility) in cases {
        let found = item(&ws, path, namespace);
        assert_eq!(found.kind, kind, "{path}");
        assert_eq!(found.file, file, "{path}");
        assert_eq!(found.name_line, line, "{path}");
        assert_eq!(found.visibility, visibility, "{path}");
        assert_eq!(found.path, path);
    }
    // The macro-exported item is not found under the module it is written in.
    assert!(matches!(
        ws.resolve_in("demo::hidden::deep", Namespace::Macro),
        Err(ResolveError::NotFound(_))
    ));
}

#[test]
fn unit_and_tuple_structs_are_also_values_and_named_structs_are_not() {
    let ws = lib("pub struct Unit;\npub struct Tup(u8);\npub struct Named { x: u8 }\n");
    for (name, value) in [("Unit", true), ("Tup", true), ("Named", false)] {
        let path = format!("demo::{name}");
        assert!(ws.resolve_in(&path, Namespace::Type).is_ok(), "{name}");
        assert_eq!(
            ws.resolve_in(&path, Namespace::Value).is_ok(),
            value,
            "{name}"
        );
    }
    let unit = item(&ws, "demo::Unit", Namespace::Value);
    assert_eq!(unit.kind, SymbolKind::Struct);
    assert!(unit.in_namespace(Namespace::Value) && unit.in_namespace(Namespace::Type));
}

#[test]
fn a_module_a_function_and_a_macro_of_one_name_coexist_and_each_selects_by_namespace() {
    let ws = lib("pub mod select {}\npub fn select() {}\nmacro_rules! select { () => {} }\n");
    for (namespace, kind) in [
        (Namespace::Type, SymbolKind::Mod),
        (Namespace::Value, SymbolKind::Fn),
        (Namespace::Macro, SymbolKind::MacroRules),
    ] {
        let resolution = ws.resolve_in("demo::select", namespace).expect("resolves");
        assert_eq!(resolution.namespace, namespace);
        assert_eq!(ws.item(resolution.sites[0]).expect("item").kind, kind);
    }
    // Without a namespace the type namespace comes first.
    assert_eq!(
        ws.resolve("demo::select").expect("resolves").namespace,
        Namespace::Type
    );
    // A name that only the macro namespace has resolves there.
    let only_macro = lib("macro_rules! only { () => {} }");
    assert_eq!(
        only_macro
            .resolve("demo::only")
            .expect("resolves")
            .namespace,
        Namespace::Macro
    );
}

#[test]
fn documentation_counts_text_and_include_str_but_not_hidden_or_blank_comments() {
    let ws = lib("/// Documented.\npub fn documented() {}\n\
         #[doc(hidden)]\npub fn hidden() {}\n\
         ///\npub fn blank() {}\n\
         #[doc = include_str!(\"guide.md\")]\npub fn guide() {}\n\
         /// Text.\n#[doc(hidden)]\npub fn both() {}\n\
         pub fn bare() {}\n");
    let flags = |name: &str| {
        let found = item(&ws, &format!("demo::{name}"), Namespace::Value);
        (found.documented, found.doc_hidden)
    };
    assert_eq!(flags("documented"), (true, false));
    assert_eq!(flags("hidden"), (false, true));
    assert_eq!(flags("blank"), (false, false));
    assert_eq!(flags("guide"), (true, false));
    assert_eq!(flags("both"), (true, true));
    assert_eq!(flags("bare"), (false, false));
}

#[test]
fn a_module_in_its_own_file_is_documented_by_the_inner_docs_of_that_file() {
    let ws = ok(&[
        (
            "src/lib.rs",
            "mod with_docs;\nmod without;\n/// Own docs.\nmod outer;\n",
        ),
        ("src/with_docs.rs", "//! Inner docs.\n"),
        ("src/without.rs", "pub fn f() {}\n"),
        ("src/outer.rs", ""),
    ]);
    for (name, documented) in [("with_docs", true), ("without", false), ("outer", true)] {
        let found = item(&ws, &format!("demo::{name}"), Namespace::Type);
        assert_eq!(found.documented, documented, "{name}");
    }
}

#[test]
fn impl_members_are_recorded_under_the_type_or_under_the_trait_by_the_shape_of_the_impl() {
    let ws = lib("pub struct Foo;\n\
         pub type Alias = Foo;\n\
         pub trait Tr { fn run(&self); }\n\
         impl Foo { pub fn new() {} const C: u8 = 1; }\n\
         impl Alias { fn via_alias() {} }\n\
         impl Tr for Foo { fn run(&self) {} }\n\
         impl Tr for String { fn on_string(&self) {} }\n\
         impl<T> Tr for T { fn blanket(&self) {} }\n\
         impl Tr for &Foo { fn by_ref(&self) {} }\n\
         impl Tr for [u8] { fn on_slice(&self) {} }\n\
         impl Tr for (u8, u8) { fn on_tuple(&self) {} }\n");
    let cases = [
        ("demo::Foo::new", SymbolKind::InherentMethod, Some("Foo")),
        (
            "demo::Foo::via_alias",
            SymbolKind::InherentMethod,
            Some("Alias"),
        ),
        ("demo::Foo::run", SymbolKind::TraitImplMethod, Some("Foo")),
        (
            "demo::Tr::on_string",
            SymbolKind::TraitImplMethod,
            Some("String"),
        ),
        ("demo::Tr::blanket", SymbolKind::TraitImplMethod, Some("T")),
        (
            "demo::Tr::by_ref",
            SymbolKind::TraitImplMethod,
            Some("&Foo"),
        ),
        (
            "demo::Tr::on_slice",
            SymbolKind::TraitImplMethod,
            Some("[u8]"),
        ),
        (
            "demo::Tr::on_tuple",
            SymbolKind::TraitImplMethod,
            Some("(u8,u8)"),
        ),
    ];
    for (path, kind, self_type) in cases {
        let found = item(&ws, path, Namespace::Value);
        assert_eq!(found.kind, kind, "{path}");
        assert_eq!(found.self_type.as_deref(), self_type, "{path}");
        let in_trait_impl = kind == SymbolKind::TraitImplMethod;
        assert_eq!(
            found.trait_path.as_deref(),
            in_trait_impl.then_some("Tr"),
            "{path}"
        );
    }
    let constant = item(&ws, "demo::Foo::C", Namespace::Value);
    assert_eq!(constant.kind, SymbolKind::AssocConst);
    // The trait's own method is a different item from the impl's.
    assert_eq!(
        item(&ws, "demo::Tr::run", Namespace::Value).kind,
        SymbolKind::TraitRequiredMethod
    );
    // An impl member is not found by a path that names no type.
    assert!(matches!(
        ws.resolve_in("demo::on_string", Namespace::Value),
        Err(ResolveError::NotFound(_))
    ));
}

#[test]
fn an_inherent_impl_on_a_type_that_does_not_resolve_is_an_error_naming_file_and_line() {
    for bad in [
        "impl Missing {}",
        "impl Vec<u8> {}",
        "impl &Foo {}",
        "impl [u8] {}",
        "impl<T> T {}",
    ] {
        let text = format!("pub struct Foo;\n{bad}\n");
        let error = load(&[("src/lib.rs", &text)]).expect_err(bad);
        assert!(
            matches!(error.kind, LoadErrorKind::InherentImplSelf { .. }),
            "{bad}: {error}"
        );
        assert_eq!(error.file.as_str(), "src/lib.rs");
        assert_eq!(error.line, Some(2), "{bad}");
    }
    // The neighbouring impl on a local type loads.
    assert!(load(&[("src/lib.rs", "pub struct Foo;\nimpl Foo {}\n")]).is_ok());
}

#[test]
fn a_method_in_another_file_than_its_type_is_recorded_under_the_type() {
    let ws = ok(&[
        ("src/lib.rs", "mod ty;\nmod ops;\n"),
        ("src/ty.rs", "pub struct Thing;\n"),
        (
            "src/ops.rs",
            "use crate::ty::Thing;\nimpl Thing { pub fn act() {} }\n",
        ),
    ]);
    let method = item(&ws, "demo::ty::Thing::act", Namespace::Value);
    assert_eq!(method.file.as_str(), "src/ops.rs");
    assert_eq!(method.name_line, 2);
}

#[test]
fn cfg_twins_are_one_symbol_with_two_sorted_sites() {
    let ws = lib("#[cfg(unix)]\npub fn a() {}\n#[cfg(windows)]\npub fn a() {}\n");
    let resolution = ws.resolve("demo::a").expect("twins resolve");
    assert_eq!(resolution.sites.len(), 2);
    assert!(!resolution.via_glob);
    let lines: Vec<_> = resolution
        .sites
        .iter()
        .map(|id| ws.item(*id).expect("item").name_line)
        .collect();
    assert_eq!(lines, [2, 4]);
    // One definition resolves to one site.
    assert_eq!(
        lib("pub fn a() {}")
            .resolve("demo::a")
            .expect("resolves")
            .sites
            .len(),
        1
    );
}

#[test]
fn files_items_in_and_item_at_answer_by_position() {
    let ws = ok(&[
        ("src/lib.rs", "mod inner;\npub fn first() {}\n"),
        ("src/inner.rs", "pub fn go() {}\npub struct S { f: u8 }\n"),
    ]);
    let files: Vec<_> = ws.files().map(|f| f.path.as_str()).collect();
    assert_eq!(files, ["src/inner.rs", "src/lib.rs"]);
    let file = RelPath::new("src/inner.rs").expect("path");
    let names: Vec<_> = ws.items_in(&file).map(|i| i.name.as_str()).collect();
    assert_eq!(names, ["go", "S"]);
    let source = ws.source(&file).expect("text");
    let offset = source.find("pub struct").expect("offset");
    let at = ws.item_at(&file, offset + 4);
    assert_eq!(at.len(), 1);
    assert_eq!(at[0].name, "S");
    assert_eq!(ws.item_at(&file, source.len() + 10).len(), 0);
    // Identity is the file and the start of the span.
    assert_eq!(at[0].source_key(), (&file, offset));
    let lib_file = ws
        .file(&RelPath::new("src/lib.rs").expect("path"))
        .expect("file");
    assert_eq!(lib_file.package, "demo");
    assert!(lib_file.shipping && !lib_file.test);
}

#[test]
fn shipping_is_the_target_and_not_a_test_role() {
    let ws = ok(&[
        (
            "src/lib.rs",
            "pub fn real() {}\n#[cfg(test)]\nmod tests { pub fn helper() {} }\n#[test]\nfn unit() {}\n",
        ),
        ("tests/it.rs", "fn case() {}\n"),
    ]);
    let flags = |path: &str, ns| ws.is_shipping(item(&ws, path, ns));
    assert!(flags("demo::real", Namespace::Value));
    assert!(!flags("demo::tests::helper", Namespace::Value));
    assert!(!flags("demo::unit", Namespace::Value));
    assert!(!flags("demo::test_crate::it::case", Namespace::Value));
    let test_file = ws
        .file(&RelPath::new("tests/it.rs").expect("path"))
        .expect("file");
    assert!(!test_file.shipping);
}

#[test]
fn test_functions_carry_the_package_target_and_crate_relative_name() {
    let ws = ok(&[
        ("src/lib.rs", "#[test]\nfn in_lib() {}\n"),
        ("src/main.rs", "#[test]\nfn in_bin() {}\nfn main() {}\n"),
        (
            "tests/it.rs",
            "#[test]\nfn case() {}\nmod inner { #[test] fn deep() {} }\n#[cfg(test)] fn helper() {}\n",
        ),
    ]);
    let mut found: Vec<_> = ws
        .test_fns()
        .expect("test paths start with their crate path")
        .into_iter()
        .map(|t| (t.package, t.kind, t.target, t.name))
        .collect();
    found.sort();
    let expect = |kind, target: &str, name: &str| {
        ("demo".to_owned(), kind, target.to_owned(), name.to_owned())
    };
    let mut want = vec![
        expect(TargetKind::Lib, "demo", "in_lib"),
        expect(TargetKind::Bin, "demo", "in_bin"),
        expect(TargetKind::Test, "it", "case"),
        expect(TargetKind::Test, "it", "inner::deep"),
    ];
    want.sort();
    assert_eq!(found, want);
}

#[test]
fn a_first_party_test_crate_module_beside_a_test_target_makes_the_shared_path_ambiguous() {
    let files = [
        ("src/lib.rs", "mod test_crate;\n"),
        ("src/test_crate.rs", "pub fn first_party() {}\n"),
        ("tests/it.rs", "pub fn case() {}\n"),
    ];
    let ws = ok(&files);
    // Both files load, under their own paths.
    let reached = |path: &str| {
        ws.file(&RelPath::new(path).expect("path"))
            .expect("loaded")
            .reached[0]
            .module_path
            .clone()
    };
    assert_eq!(reached("src/test_crate.rs"), "demo::test_crate");
    assert_eq!(reached("tests/it.rs"), "demo::test_crate::it");
    // The first-party item and the module resolve.
    assert!(ws.resolve("demo::test_crate::first_party").is_ok());
    assert!(ws.resolve("demo::test_crate").is_ok());
    // A path through the shared name is ambiguous between the two.
    match ws.resolve("demo::test_crate::it::case") {
        Err(ResolveError::Ambiguous(ids)) => assert!(ids.len() >= 2),
        other => panic!("{other:?}"),
    }
    // Without the first-party module the same test path resolves.
    let clean = ok(&[("src/lib.rs", ""), ("tests/it.rs", "pub fn case() {}\n")]);
    assert!(clean.resolve("demo::test_crate::it::case").is_ok());
}

#[test]
fn two_loads_are_equal_and_items_and_targets_come_in_strictly_increasing_order() {
    let files = [
        (
            "src/lib.rs",
            "mod a;\npub fn z() {}\npub fn b() {}\n#[cfg(unix)] pub fn t() {}\n#[cfg(windows)] pub fn t() {}\n",
        ),
        ("src/a.rs", "pub struct A;\nimpl A { pub fn m() {} }\n"),
        ("src/main.rs", "fn main() {}\n"),
        ("tests/it.rs", "fn case() {}\n"),
    ];
    let first = ok(&files);
    let second = ok(&files);
    assert_eq!(first, second);
    let keys: Vec<_> = first
        .items()
        .map(|i| {
            (
                i.path.clone(),
                i.namespace,
                i.file.clone(),
                i.span.start_byte,
            )
        })
        .collect();
    assert!(keys.windows(2).all(|pair| pair[0] < pair[1]), "{keys:?}");
    let targets: Vec<_> = first
        .targets()
        .map(|t| (t.package.clone(), t.kind, t.name.clone()))
        .collect();
    assert!(
        targets.windows(2).all(|pair| pair[0] < pair[1]),
        "{targets:?}"
    );
    // A prefix is a contiguous run.
    let under: Vec<_> = first
        .items_under("demo::a")
        .map(|i| i.path.as_str())
        .collect();
    assert!(under.iter().all(|p| p.starts_with("demo::a")));
    assert_eq!(
        under,
        ["demo::a", "demo::a::A", "demo::a::A::m"]
            .map(str::to_owned)
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    );
}

#[test]
fn an_unnamed_const_is_in_the_facts_and_not_in_the_index() {
    let ws = lib("const _: () = ();\npub const NAMED: u8 = 1;\n");
    assert!(ws.items().all(|i| i.name != "_"));
    assert!(ws.resolve_in("demo::NAMED", Namespace::Value).is_ok());
    let file = ws
        .file(&RelPath::new("src/lib.rs").expect("path"))
        .expect("file");
    assert!(file.facts.body.items.iter().any(|i| i.name == "_"));
}

#[test]
fn test_roles_follow_the_module_tree_and_the_item() {
    let ws = ok(&[
        ("src/lib.rs", "pub fn plain() {}\n#[cfg(test)] mod t;\n"),
        ("src/t.rs", "pub fn helper() {}\n#[test] fn marked() {}\n"),
    ]);
    let role = |path: &str| item(&ws, path, Namespace::Value).test;
    assert_eq!(role("demo::plain"), TestRole::None);
    assert_eq!(role("demo::t::helper"), TestRole::CfgTest);
    assert_eq!(role("demo::t::marked"), TestRole::TestFn);
}

#[test]
fn an_item_the_index_does_not_model_is_a_typed_error_naming_file_and_line() {
    let error =
        load(&[("src/lib.rs", "pub fn ok() {}\ntrait Alias = Send;\n")]).expect_err("unmodelled");
    assert!(
        matches!(error.kind, LoadErrorKind::UnsupportedItem { .. }),
        "{error}"
    );
    assert_eq!(error.file.as_str(), "src/lib.rs");
    assert_eq!(error.line, Some(2));
    assert!(load(&[("src/lib.rs", "pub fn ok() {}\npub trait Plain {}\n")]).is_ok());
}

#[test]
fn items_that_share_a_source_item_are_grouped_by_its_identity() {
    let ws = ok(&[
        ("src/lib.rs", ""),
        (
            "tests/a.rs",
            "mod common;\n#[test] fn a() { common::helper() }\n",
        ),
        (
            "tests/b.rs",
            "mod common;\n#[test] fn b() { common::helper() }\n",
        ),
        ("tests/common/mod.rs", "pub fn helper() {}\n"),
    ]);
    let helpers: Vec<&Item> = ws.items().filter(|i| i.name == "helper").collect();
    assert_eq!(
        helpers.len(),
        2,
        "one item for each way the file is reached"
    );
    assert_eq!(helpers[0].source_key(), helpers[1].source_key());
    assert_ne!(helpers[0].path, helpers[1].path);
    let group = ws.same_source(helpers[0]);
    assert_eq!(group.len(), 2);
    let paths: Vec<_> = group
        .iter()
        .map(|id| ws.item(*id).expect("item").path.as_str())
        .collect();
    assert_eq!(
        paths,
        [
            "demo::test_crate::a::common::helper",
            "demo::test_crate::b::common::helper"
        ]
    );
    // Every other source item is alone in its group, and groups come in order.
    let groups: Vec<_> = ws.items_by_source().collect();
    assert_eq!(
        groups.iter().filter(|(_, _, ids)| ids.len() == 2).count(),
        1
    );
    assert!(
        groups
            .windows(2)
            .all(|pair| (pair[0].0, pair[0].1) < (pair[1].0, pair[1].1))
    );
    let covered: usize = groups.iter().map(|(_, _, ids)| ids.len()).sum();
    // Crate roots are whole files and are not grouped.
    let roots = ws.targets().count();
    assert_eq!(covered + roots, ws.items().count());
    let lone = ws.items().find(|i| i.name == "a").expect("test a");
    assert_eq!(ws.same_source(lone).len(), 1);
}
