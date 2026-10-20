// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The symbol index: items, their kinds and fields, impl members, accessors
//! and determinism.

use std::fmt::Write as _;

use raf_source::{
    Item, LoadError, LoadErrorKind, MemoryTree, Namespace, RelPath, ResolveError, SelfType,
    SymbolKind, TargetKind, TestRole, Vis, Workspace,
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
fn an_inherent_impl_on_a_type_the_index_cannot_see_is_indexed_under_its_written_self_path() {
    for (written, path) in [
        ("impl Missing { pub fn f() {} }", "demo::Missing"),
        ("impl Vec<u8> { pub fn f() {} }", "demo::Vec"),
        ("impl<T> T { pub fn f() {} }", "demo::T"),
        ("impl &Foo { pub fn f() {} }", "demo::&Foo"),
        ("impl [u8] { pub fn f() {} }", "demo::[u8]"),
        ("impl crate::Missing { pub fn f() {} }", "demo::Missing"),
    ] {
        let text = format!("pub struct Foo;\n{written}\nimpl Foo {{ pub fn g() {{}} }}\n");
        let ws = lib(&text);
        let blocks: Vec<_> = ws.unresolved_impls().collect();
        assert_eq!(blocks.len(), 1, "{written}");
        let block = blocks[0];
        assert_eq!(block.path, path, "{written}");
        assert_eq!((block.file.as_str(), block.line), ("src/lib.rs", 2));
        assert_eq!(block.test, TestRole::None);
        let member = ws.item(block.members[0]).expect("member");
        assert_eq!(member.path, format!("{path}::f"), "{written}");
        assert_eq!(member.kind, SymbolKind::InherentMethod);
        assert_eq!(member.self_key, Some(SelfType::Unresolved), "{written}");
        assert_eq!(member.name_line, 2);
        // The marker is on the item wherever it is read.
        let found: Vec<_> = ws.items().filter(|item| item.id == member.id).collect();
        assert_eq!(found[0].self_key, Some(SelfType::Unresolved));
        let at = ws.item_at(&member.file, member.span.start_byte);
        assert_eq!(at.len(), 1);
        assert_eq!(at[0].self_key, Some(SelfType::Unresolved));
        // The neighbouring impl on a local type is placed under its type.
        let neighbour = item(&ws, "demo::Foo::g", Namespace::Value);
        assert_eq!(neighbour.self_key, Some(SelfType::Local));
    }
    // A path through the self path resolves to the typed error, and the self
    // path itself, a type the index cannot see, names nothing.
    let ws = lib("impl Missing { pub fn f() {} }\nimpl self::Missing {}\n");
    assert_eq!(ws.unresolved_impls().count(), 2);
    for path in ["demo::Missing::f", "demo::Missing::other"] {
        assert_eq!(
            ws.resolve(path),
            Err(ResolveError::UnresolvedSelf(path.to_owned())),
            "{path}"
        );
    }
    assert!(matches!(
        ws.resolve("demo::Missing"),
        Err(ResolveError::NotFound(_))
    ));
}

#[test]
fn a_super_path_above_the_root_in_an_unresolved_self_type_is_still_an_error() {
    let error = load(&[("src/lib.rs", "impl super::Missing {}\n")]).expect_err("above");
    assert!(
        matches!(error.kind, LoadErrorKind::SuperAboveRoot { .. }),
        "{error}"
    );
    assert_eq!(error.line, Some(1));
    // One `super` from a module stays inside the crate.
    let ws = lib("mod m { impl super::Missing { fn f() {} } }\n");
    assert_eq!(
        ws.unresolved_impls().next().expect("listed").path,
        "demo::Missing"
    );
}

#[test]
fn a_bitflags_style_macro_type_with_an_impl_loads_with_its_members_marked_unresolved() {
    let ws = ok(&[
        (
            "src/lib.rs",
            "#[macro_use]\nmod flags;\npub mod kinds;\npub fn plain() {}\n",
        ),
        (
            "src/flags.rs",
            "macro_rules! bitflags {\n    (pub struct $name:ident: $t:ty { $(const $f:ident = $v:expr;)* }) => {\n        pub struct $name($t);\n    };\n}\n",
        ),
        (
            "src/kinds.rs",
            "bitflags! {\n    pub struct Kind: u8 {\n        const A = 1;\n    }\n}\n\nimpl Kind {\n    pub fn is_a(&self) -> bool { true }\n    pub const NONE: u8 = 0;\n}\n\n#[cfg(test)]\nmod tests {\n    impl super::Kind { fn helper() {} }\n}\n",
        ),
    ]);
    let blocks: Vec<_> = ws.unresolved_impls().collect();
    let paths: Vec<_> = blocks
        .iter()
        .map(|block| (block.path.as_str(), block.line, block.test))
        .collect();
    assert_eq!(
        paths,
        [
            ("demo::kinds::Kind", 7, TestRole::None),
            ("demo::kinds::Kind", 14, TestRole::CfgTest),
        ]
    );
    assert_eq!(blocks[0].self_type, "Kind");
    assert_eq!(blocks[1].self_type, "super::Kind");
    let members: Vec<_> = blocks[0]
        .members
        .iter()
        .map(|id| {
            let member = ws.item(*id).expect("member");
            (member.path.as_str(), member.kind, member.self_key)
        })
        .collect();
    assert_eq!(
        members,
        [
            (
                "demo::kinds::Kind::is_a",
                SymbolKind::InherentMethod,
                Some(SelfType::Unresolved)
            ),
            (
                "demo::kinds::Kind::NONE",
                SymbolKind::AssocConst,
                Some(SelfType::Unresolved)
            ),
        ]
    );
    assert_eq!(
        ws.resolve("demo::kinds::Kind::is_a"),
        Err(ResolveError::UnresolvedSelf(
            "demo::kinds::Kind::is_a".to_owned()
        ))
    );
    assert_eq!(
        ws.resolve_in("demo::kinds::Kind::NONE", Namespace::Value),
        Err(ResolveError::UnresolvedSelf(
            "demo::kinds::Kind::NONE".to_owned()
        ))
    );
    // The rest of the crate is indexed and resolves.
    assert!(ws.resolve("demo::plain").is_ok());
    assert!(matches!(
        ws.resolve("demo::kinds::Other::is_a"),
        Err(ResolveError::NotFound(_))
    ));
}

#[test]
fn an_impl_of_a_local_trait_object_loads_with_its_members_marked_unresolved() {
    let ws = lib("pub trait Shape { fn area(&self) -> u8; }\n\
         impl dyn Shape + Send { pub fn describe(&self) {} }\n\
         impl dyn Shape { pub fn twice(&self) {} }\n");
    let blocks: Vec<_> = ws
        .unresolved_impls()
        .map(|block| (block.path.as_str(), block.self_type.as_str(), block.line))
        .collect();
    assert_eq!(
        blocks,
        [
            ("demo::Shape", "dynShape+Send", 2),
            ("demo::Shape", "dynShape", 3)
        ]
    );
    for (path, line) in [("demo::Shape::describe", 2), ("demo::Shape::twice", 3)] {
        let members: Vec<_> = ws.items().filter(|item| item.path == path).collect();
        assert_eq!(members.len(), 1, "{path}");
        assert_eq!(members[0].self_key, Some(SelfType::Unresolved));
        assert_eq!(members[0].name_line, line);
        assert_eq!(
            ws.resolve(path),
            Err(ResolveError::UnresolvedSelf(path.to_owned()))
        );
    }
    // The trait's own method still resolves to the trait.
    let area = item(&ws, "demo::Shape::area", Namespace::Value);
    assert_eq!(area.kind, SymbolKind::TraitRequiredMethod);
    assert_eq!(area.self_key, None);
}

#[test]
fn a_glob_of_enum_variants_beside_an_impl_loads() {
    // `http` writes `use self::Inner::*;` beside `impl PartialEq<Method> for
    // str`: the glob brings in the enum's variants, none of which is a trait.
    let ws = lib("pub struct Method(Inner);\n\
         enum Inner { Get, Post }\n\
         use self::Inner::*;\n\
         impl PartialEq<Method> for str { fn eq(&self, _: &Method) -> bool { true } }\n\
         pub fn first() -> Method { Method(Get) }\n");
    let eq = item(&ws, "std::cmp::PartialEq::eq", Namespace::Value);
    assert_eq!(eq.self_key, Some(SelfType::Trait));
    assert_eq!(eq.self_type.as_deref(), Some("str"));
    // A glob of an enum in another module, through `crate`.
    let ws = ok(&[
        (
            "src/lib.rs",
            "pub mod kinds;\nuse crate::kinds::Kind::*;\nstruct S;\nimpl Clone for S { fn clone(&self) -> Self { S } }\n",
        ),
        ("src/kinds.rs", "pub enum Kind { A, B }\n"),
    ]);
    assert!(ws.resolve("std::clone::Clone::clone").is_err());
    assert!(ws.resolve("demo::S::clone").is_ok());
    // A glob whose first segment is no local module and no crate is still
    // refused when a trait key consults it.
    let error = load(&[("src/lib.rs", "use bogus::Kind::*;\nimpl Tr for str {}\n")])
        .expect_err("an unknown glob");
    assert!(
        matches!(&error.kind, LoadErrorKind::UnresolvedTrait { path } if path == "bogus::Kind::*"),
        "{error}"
    );
}

#[test]
fn a_glob_of_a_local_path_the_index_cannot_see_is_a_candidate_that_cannot_be_confirmed() {
    // `nix` globs an enum that `libc_enum!` generates; `zellij-utils` globs a
    // module that `include!` generates.
    let ws = ok(&[
        (
            "src/lib.rs",
            "macro_rules! generate { ($name:ident) => { pub enum $name { A } } }\n\
             pub mod signal;\npub mod event;\n",
        ),
        (
            "src/signal.rs",
            "generate!(Signal);\n\
             impl AsRef<str> for Signal { fn as_ref(&self) -> &str { \"\" } }\n\
             pub use self::Signal::*;\n",
        ),
        (
            "src/event.rs",
            "use super::generated::api::*;\n\
             pub struct Event;\n\
             impl ProtobufMessage for Vec<Event> { fn encode(&self) {} }\n",
        ),
    ]);
    let key = |path: &str| {
        let found: Vec<_> = ws
            .items()
            .filter(|item| item.path == path)
            .map(|item| item.trait_key.as_deref().cloned())
            .collect();
        assert_eq!(found.len(), 1, "{path}");
        found[0].clone().expect("a trait key")
    };
    assert_eq!(
        key("AsRef::as_ref"),
        raf_source::TraitKey::Unresolved {
            written: "AsRef".to_owned(),
            candidates: vec![
                "demo::signal::Signal::AsRef".to_owned(),
                "std::convert::AsRef".to_owned()
            ],
        }
    );
    assert_eq!(
        key("demo::generated::api::ProtobufMessage::encode"),
        raf_source::TraitKey::Canonical("demo::generated::api::ProtobufMessage".to_owned())
    );
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
fn a_first_party_test_crate_module_and_a_test_target_are_ambiguous_only_where_their_paths_meet() {
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
    // The first-party item and the module resolve, and so does the test
    // target's item, which nothing in the library shares a path with.
    assert!(ws.resolve("demo::test_crate::first_party").is_ok());
    assert!(ws.resolve("demo::test_crate").is_ok());
    let case = ws
        .resolve("demo::test_crate::it::case")
        .expect("the test item");
    assert_eq!(
        ws.item(case.sites[0]).expect("item").file.as_str(),
        "tests/it.rs"
    );
    // Where the library's module reaches the test target's paths, each shared
    // path names both, in either order of the files.
    for (lib_text, file) in [
        ("mod test_crate;\n", "src/test_crate.rs"),
        (
            "#[path = \"a_first.rs\"]\nmod test_crate;\n",
            "src/a_first.rs",
        ),
    ] {
        let ws = ok(&[
            ("src/lib.rs", lib_text),
            (
                file,
                "pub mod it { pub fn case() {} pub fn lib_only() {} }\n",
            ),
            ("tests/it.rs", "pub fn case() {}\npub fn test_only() {}\n"),
        ]);
        for shared in ["demo::test_crate::it::case", "demo::test_crate::it"] {
            match ws.resolve(shared) {
                Err(ResolveError::Ambiguous(ids)) => {
                    let mut files: Vec<_> = ids
                        .iter()
                        .map(|id| ws.item(*id).expect("item").file.as_str())
                        .collect();
                    files.sort_unstable();
                    assert_eq!(files, [file, "tests/it.rs"], "{shared}");
                }
                other => panic!("{shared}: {other:?}"),
            }
        }
        let only = |path: &str| {
            let found = ws.resolve(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            ws.item(found.sites[0]).expect("item").file.to_string()
        };
        assert_eq!(only("demo::test_crate::it::lib_only"), file);
        assert_eq!(only("demo::test_crate::it::test_only"), "tests/it.rs");
    }
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
fn items_under_matches_whole_path_segments() {
    let ws = ok(&[
        (
            "src/lib.rs",
            "pub mod a;\npub mod ab;\npub mod a1;\npub fn a_fn() {}\n",
        ),
        ("src/a.rs", "pub struct A;\nimpl A { pub fn m() {} }\n"),
        ("src/ab.rs", "pub struct B;\n"),
        ("src/a1.rs", "pub struct C;\n"),
    ]);
    let under = |prefix: &str| -> Vec<String> {
        ws.items_under(prefix)
            .map(|i| i.path.as_str().to_owned())
            .collect()
    };
    assert_eq!(under("demo::a"), ["demo::a", "demo::a::A", "demo::a::A::m"]);
    assert_eq!(under("demo::ab"), ["demo::ab", "demo::ab::B"]);
    assert_eq!(under("demo::a::A"), ["demo::a::A", "demo::a::A::m"]);
    assert_eq!(under("demo::a::"), Vec::<String>::new());
    assert_eq!(under("demo::nope"), Vec::<String>::new());
    assert_eq!(under("").len(), ws.items().count());
}

#[test]
fn a_file_reached_by_targets_of_two_packages_belongs_to_the_first_target_in_target_order() {
    let tree: MemoryTree = [
        ("Cargo.toml", "[workspace]\nmembers = [\"p1\", \"p2\"]\n"),
        (
            "p1/Cargo.toml",
            "[package]\nname = \"zeta\"\nversion = \"0.1.0\"\n",
        ),
        (
            "p2/Cargo.toml",
            "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n",
        ),
        ("p1/src/lib.rs", "#[path = \"../../shared.rs\"]\nmod s;\n"),
        ("p2/src/lib.rs", "#[path = \"../../shared.rs\"]\nmod s;\n"),
        ("shared.rs", "pub fn go() {}\n"),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["."]).expect("loads");
    let first = ws.targets().next().expect("a target");
    let shared = ws
        .file(&RelPath::new("shared.rs").expect("path"))
        .expect("the shared file");
    assert_eq!(shared.reached.len(), 2);
    assert_eq!(shared.package, first.package);
    // Packages are in manifest-path order, not name order.
    assert_eq!(first.package, "zeta");
}

#[test]
fn an_unnamed_const_is_not_in_the_index_beside_a_named_one() {
    let ws = lib("const _: () = ();\npub const NAMED: u8 = 1;\n");
    assert!(ws.items().all(|i| i.name != "_"));
    assert!(ws.resolve_in("demo::NAMED", Namespace::Value).is_ok());
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

#[test]
fn items_on_one_long_line_of_wide_characters_get_exact_byte_offsets() {
    // Twenty thousand functions on one line, each name holding a character of
    // two bytes, so a column is not a byte offset.
    let mut text = String::new();
    let mut starts = Vec::new();
    for index in 0..20_000 {
        starts.push(text.len());
        write!(text, "pub fn é{index}() {{}} ").expect("write to a string");
    }
    text.push('\n');
    let ws = lib(&text);
    for (index, start) in starts.iter().enumerate().step_by(997) {
        let found = item(&ws, &format!("demo::é{index}"), Namespace::Value);
        assert_eq!(found.span.start_byte, *start, "é{index}");
        assert_eq!(found.span.lines, 1..2);
        let body = found.body.clone().expect("a function body");
        assert_eq!(
            body.end_byte,
            start + format!("pub fn é{index}() {{}}").len()
        );
    }
}

#[test]
fn bodiless_functions_and_types_of_attribute_macros_are_indexed_where_they_are_written() {
    // The input of an attribute macro that syn has no node for: a function
    // with no body, at item level and in an impl block, and a type with no
    // definition.
    let text = "pub struct S;\n\
                #[ref_cast_custom]\n\
                #[inline]\n\
                pub(crate) const fn free(x: &str) -> &S;\n\
                impl S {\n    #[ref_cast_custom]\n    pub fn new(s: &str) -> &S;\n    pub fn body() {}\n}\n\
                /// A type with two states.\n\
                #[ty(cast)]\n\
                pub type bool;\n\
                pub mod m {\n    #[cfg(test)] pub fn inner();\n}\n";
    let ws = lib(text);
    let at = |needle: &str| text.find(needle).expect("in the text");
    let free = item(&ws, "demo::free", Namespace::Value);
    assert_eq!(free.kind, SymbolKind::Fn);
    assert_eq!((free.name_line, free.span.lines.clone()), (4, 2..5));
    assert_eq!(free.span.start_byte, at("#[ref_cast_custom]"));
    assert_eq!(free.span.end_byte, at("impl S") - 1);
    assert!(free.body.is_none());
    assert_eq!(free.visibility, Vis::Restricted("crate".to_owned()));
    let new = item(&ws, "demo::S::new", Namespace::Value);
    assert_eq!(new.kind, SymbolKind::InherentMethod);
    assert_eq!((new.name_line, new.span.lines.clone()), (7, 6..8));
    assert_eq!(
        new.span.start_byte,
        at("#[ref_cast_custom]\n    pub fn new")
    );
    assert!(new.body.is_none());
    assert!(item(&ws, "demo::S::body", Namespace::Value).body.is_some());
    let ty = item(&ws, "demo::bool", Namespace::Type);
    assert_eq!(ty.kind, SymbolKind::TypeAlias);
    assert_eq!((ty.name_line, ty.span.lines.clone()), (12, 10..13));
    assert!(ty.documented);
    let inner = item(&ws, "demo::m::inner", Namespace::Value);
    assert_eq!((inner.kind, inner.name_line), (SymbolKind::Fn, 14));
    assert_eq!(inner.test, TestRole::CfgTest);
    assert!(matches!(
        ws.resolve_in("demo::inner", Namespace::Value),
        Err(ResolveError::NotFound(_))
    ));
}

#[test]
fn a_bodiless_const_or_another_verbatim_item_is_still_a_typed_error_naming_file_and_line() {
    // rustc parses each of these (they compile under `#[cfg(any())]`); syn
    // has no node for them and the index does not read them.
    for (text, line) in [
        ("pub fn ok() {}\n#[cfg(any())]\nconst C: u8;\n", 2),
        (
            "pub struct S;\nimpl S {\n    #[cfg(any())]\n    const K: u8;\n}\n",
            3,
        ),
        ("pub fn ok() {}\n#[cfg(any())]\ntype T: Bound;\n", 2),
    ] {
        let error = load(&[("src/lib.rs", text)]).expect_err(text);
        assert!(
            matches!(error.kind, LoadErrorKind::UnsupportedItem { .. }),
            "{text}: {error}"
        );
        assert_eq!(
            (error.file.as_str(), error.line),
            ("src/lib.rs", Some(line)),
            "{text}"
        );
    }
    // The bodiless function beside them loads.
    assert!(load(&[("src/lib.rs", "#[cfg(any())]\nfn f();\n")]).is_ok());
}

/// What the tests of `Workspace::impls` read of each block, as a line: its
/// line, its lines, its module, its written self type and trait (`-` for
/// none), the paths of its self type key and of its members.
fn block_rows(ws: &Workspace) -> Vec<String> {
    let paths = |ids: &[raf_source::ItemId]| {
        ids.iter()
            .map(|id| ws.item(*id).expect("an item").path.to_string())
            .collect::<Vec<_>>()
            .join(",")
    };
    ws.impls()
        .map(|block| {
            format!(
                "{} {:?} {} {} {} [{}] [{}]",
                block.line,
                block.span.lines,
                block.module,
                block.self_type,
                block.trait_path.as_deref().unwrap_or("-"),
                paths(&block.self_items),
                paths(&block.members),
            )
        })
        .collect()
}

#[test]
fn every_impl_block_is_listed_in_file_order_with_its_span_module_self_type_key_and_members() {
    let text = "pub struct Foo;\n\
         pub type Alias = Foo;\n\
         pub trait Tr { fn run(&self); }\n\
         impl Foo { pub fn new() {} const C: u8 = 1; }\n\
         impl Alias { fn via_alias() {} }\n\
         impl Tr for Foo { fn run(&self) {} }\n\
         impl Tr for String { fn on_string(&self) {} }\n\
         impl<T> Tr for T { fn blanket(&self) {} }\n\
         impl Tr for &Foo { fn by_ref(&self) {} }\n\
         impl Missing { fn lost() {} }\n\
         #[cfg(test)]\n\
         impl Foo {\n    fn checked() {}\n}\n\
         mod inner { impl super::Foo { fn from_inner() {} } }\n\
         fn body() { struct Local; impl Local { fn hidden() {} } }\n";
    let ws = lib(text);
    assert_eq!(
        block_rows(&ws),
        [
            "4 4..5 demo Foo - [demo::Foo] [demo::Foo::new,demo::Foo::C]",
            "5 5..6 demo Alias - [demo::Foo] [demo::Foo::via_alias]",
            "6 6..7 demo Foo Tr [demo::Foo] [demo::Foo::run]",
            "7 7..8 demo String Tr [] [demo::Tr::on_string]",
            "8 8..9 demo T Tr [] [demo::Tr::blanket]",
            "9 9..10 demo &Foo Tr [] [demo::Tr::by_ref]",
            "10 10..11 demo Missing - [] [demo::Missing::lost]",
            "12 11..15 demo Foo - [demo::Foo] [demo::Foo::checked]",
            "15 15..16 demo::inner super::Foo - [demo::Foo] [demo::Foo::from_inner]",
        ]
    );
    // The span runs from the attribute to the closing brace, in bytes too.
    let checked = ws.impls().nth(7).expect("the test block");
    assert_eq!(checked.test, TestRole::CfgTest);
    let start = text.find("#[cfg(test)]").expect("the attribute");
    let body = "    fn checked() {}\n}";
    let end = text.find(body).expect("the block") + body.len();
    assert_eq!(
        (checked.span.start_byte, checked.span.end_byte),
        (start, end)
    );
    // The id is the place in the order, and finds the block again.
    for (position, block) in ws.impls().enumerate() {
        assert_eq!(block.id.index(), position);
        assert_eq!(ws.impl_block(block.id), Some(block));
        assert_eq!(block.file.as_str(), "src/lib.rs");
    }
    // An impl block that names a type the index cannot place is listed as
    // well as among the unresolved impls.
    assert_eq!(ws.unresolved_impls().count(), 1);
}

#[test]
fn a_negative_impl_keeps_its_bang_in_the_trait_path_beside_a_positive_one() {
    let ws = lib("pub struct G<T>(T);\n\
         impl !Sync for G<u8> {}\n\
         unsafe impl Sync for G<u16> {}\n");
    assert_eq!(
        block_rows(&ws),
        [
            "2 2..3 demo G<u8> !Sync [demo::G] []",
            "3 3..4 demo G<u16> Sync [demo::G] []",
        ]
    );
}

#[test]
fn an_impl_block_of_a_file_reached_under_two_module_paths_is_listed_once_for_each() {
    let ws = ok(&[
        (
            "src/lib.rs",
            "#[path = \"shared.rs\"]\npub mod a;\n#[path = \"shared.rs\"]\npub mod b;\n",
        ),
        (
            "src/shared.rs",
            "pub struct S;\nimpl S { pub fn new() {} }\n",
        ),
    ]);
    assert_eq!(
        block_rows(&ws),
        [
            "2 2..3 demo::a S - [demo::a::S] [demo::a::S::new]",
            "2 2..3 demo::b S - [demo::b::S] [demo::b::S::new]",
        ]
    );
    assert!(ws.impls().all(|block| block.line == 2));
    // A library without impl blocks lists none.
    assert_eq!(lib("pub fn f() {}\n").impls().count(), 0);
}
