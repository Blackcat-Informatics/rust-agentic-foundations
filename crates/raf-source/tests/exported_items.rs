// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The public surface of a target: which items another crate can name through
//! its crate root, under which paths, and what is refused.

mod support;

use std::fmt::Write as _;

use support::random::{Draw, random_crate};

use raf_source::{
    Budget, ExportError, ExportOptions, Exports, LoadErrorKind, LoadOptions, MemoryTree,
    ResolveError, SymbolKind, TargetId, TargetKind, Workspace,
};

const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

fn tree(files: &[(&str, &str)]) -> MemoryTree {
    let mut all = vec![("Cargo.toml", DEMO)];
    all.extend_from_slice(files);
    all.into_iter().collect()
}

fn load(files: &[(&str, &str)]) -> Workspace {
    Workspace::load(&tree(files), &["."]).unwrap_or_else(|e| panic!("{e}"))
}

/// The id of the first target of `kind`.
fn target(ws: &Workspace, kind: TargetKind) -> TargetId {
    ws.target_ids()
        .find(|id| ws.target(*id).is_some_and(|target| target.kind == kind))
        .unwrap_or_else(|| panic!("no {kind:?} target"))
}

fn exports(ws: &Workspace, kind: TargetKind, include_hidden: bool) -> Exports {
    let mut options = ExportOptions::default();
    options.include_hidden = include_hidden;
    ws.exported_items_with(target(ws, kind), &options)
        .unwrap_or_else(|e| panic!("{e}"))
}

/// Each exported item as its exported path and the path of the item that
/// defines it.
fn surface(ws: &Workspace) -> Vec<(String, String)> {
    exports(ws, TargetKind::Lib, false)
        .items
        .iter()
        .map(|export| {
            let item = ws.item(export.item).expect("the defining item exists");
            (export.path.clone(), item.path.to_string())
        })
        .collect()
}

/// The exported paths of the library of a crate whose root is `lib`.
fn paths_of(lib: &str) -> Vec<String> {
    surface(&load(&[("src/lib.rs", lib)]))
        .into_iter()
        .map(|(path, _)| path)
        .collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
        .collect()
}

fn strings(list: &[&str]) -> Vec<String> {
    list.iter().map(|text| (*text).to_owned()).collect()
}

#[test]
fn a_pub_function_is_exported_beside_private_and_restricted_ones_which_are_not() {
    let ws = load(&[(
        "src/lib.rs",
        "pub fn open() {}\nfn private() {}\npub(crate) fn krate() {}\npub mod m {\n    pub(super) fn sup() {}\n    pub(in crate::m) fn inside() {}\n    pub(self) fn own() {}\n    pub fn deep() {}\n}\n",
    )]);
    assert_eq!(
        surface(&ws),
        pairs(&[
            ("demo::m", "demo::m"),
            ("demo::m::deep", "demo::m::deep"),
            ("demo::open", "demo::open"),
        ])
    );
    let listed = exports(&ws, TargetKind::Lib, false);
    let open = listed
        .items
        .iter()
        .find(|export| export.path == "demo::open")
        .expect("listed");
    assert_eq!(open.kind, SymbolKind::Fn);
    assert_eq!(open.file.as_str(), "src/lib.rs");
    assert_eq!(open.span.lines, 1..2);
    assert!(!open.doc_hidden);
    assert_eq!(listed.reexports, []);
}

#[test]
fn a_pub_item_of_a_private_module_is_exported_only_through_a_pub_use_and_under_its_rename() {
    let hidden = load(&[(
        "src/lib.rs",
        "mod inner {\n    pub fn go() {}\n    pub fn other() {}\n}\n",
    )]);
    assert_eq!(surface(&hidden), pairs(&[]));
    let ws = load(&[(
        "src/lib.rs",
        "mod inner {\n    pub fn go() {}\n    pub fn other() {}\n}\npub use inner::go;\npub use inner::go as run;\n",
    )]);
    assert_eq!(
        surface(&ws),
        pairs(&[
            ("demo::go", "demo::inner::go"),
            ("demo::run", "demo::inner::go")
        ])
    );
    // The neighbour: the same module made `pub` exports both, under its own
    // path.
    let open = load(&[(
        "src/lib.rs",
        "pub mod inner {\n    pub fn go() {}\n    pub fn other() {}\n}\n",
    )]);
    assert_eq!(
        surface(&open),
        pairs(&[
            ("demo::inner", "demo::inner"),
            ("demo::inner::go", "demo::inner::go"),
            ("demo::inner::other", "demo::inner::other"),
        ])
    );
}

#[test]
fn a_glob_re_export_brings_in_the_pub_items_of_its_module_only() {
    let ws = load(&[(
        "src/lib.rs",
        "mod inner {\n    pub fn a() {}\n    pub(crate) fn b() {}\n    fn c() {}\n    pub struct S;\n    pub mod deeper { pub fn d() {} }\n}\npub use inner::*;\n",
    )]);
    assert_eq!(
        surface(&ws),
        pairs(&[
            ("demo::S", "demo::inner::S"),
            ("demo::a", "demo::inner::a"),
            ("demo::deeper", "demo::inner::deeper"),
            ("demo::deeper::d", "demo::inner::deeper::d"),
        ])
    );
    // A private glob of the same module exports nothing.
    assert_eq!(
        paths_of("mod inner {\n    pub fn a() {}\n}\nuse inner::*;\n"),
        strings(&[])
    );
}

#[test]
fn a_chain_of_re_exports_through_nested_modules_is_followed_and_a_restricted_hop_stops_it() {
    let ws = load(&[(
        "src/lib.rs",
        "mod a {\n    pub mod b {\n        pub(crate) mod c { pub fn deep() {} }\n        pub use self::c::deep as shallow;\n    }\n}\npub use a::b;\nmod x { pub use crate::y::*; }\nmod y { pub fn yy() {} }\npub use x::*;\n",
    )]);
    assert_eq!(
        surface(&ws),
        pairs(&[
            ("demo::b", "demo::a::b"),
            ("demo::b::shallow", "demo::a::b::c::deep"),
            ("demo::yy", "demo::y::yy"),
        ])
    );
    // A glob that is not `pub` on the way keeps the name in the crate, as
    // rustc brings a name in no more visible than the binding it reads.
    assert_eq!(
        paths_of(
            "mod p { pub(crate) use crate::q::*; }\nmod q { pub fn qq() {} }\npub use p::*;\nmod r { pub(crate) use crate::q::qq as rq; }\npub use r::*;\n"
        ),
        strings(&[])
    );
    // A route that is `pub` all the way exports the item although another
    // glob brings it in through a restricted hop.
    assert_eq!(
        surface(&load(&[(
            "src/lib.rs",
            "mod d { pub fn x() {} }\nmod p { pub use crate::d::x; }\nmod q { pub(crate) use crate::d::x; }\npub use q::*;\npub use p::*;\n",
        )])),
        pairs(&[("demo::x", "demo::d::x")])
    );
}

#[test]
fn a_restricted_or_private_re_export_is_not_exported_beside_a_pub_one_that_is() {
    for restricted in [
        "pub(crate) use inner::go;",
        "use inner::go;",
        "pub(self) use inner::go;",
        "pub(in crate) use inner::go;",
        "pub(crate) use inner::*;",
        "use inner::*;",
    ] {
        let lib = format!("mod inner {{ pub fn go() {{}} }}\n{restricted}\n");
        assert_eq!(paths_of(&lib), strings(&[]), "{restricted}");
    }
    // A `pub` re-export of an item that is not `pub` itself, which rustc
    // refuses, exports nothing either.
    assert_eq!(
        paths_of("mod inner { pub(crate) fn go() {} }\npub use inner::go;\n"),
        strings(&[])
    );
    assert_eq!(
        paths_of(
            "pub mod m {\n    mod inner { pub fn go() {} }\n    pub(super) use self::inner::go;\n}\n"
        ),
        strings(&["demo::m"])
    );
    assert_eq!(
        paths_of("mod inner { pub fn go() {} }\npub use inner::go;\n"),
        strings(&["demo::go"])
    );
}

#[test]
fn a_re_export_of_another_crate_is_a_re_export_and_not_an_item() {
    let mut manifest = DEMO.to_owned();
    manifest.push_str("[dependencies]\nserde = \"1\"\np_one = { path = \"p_one\" }\n");
    let tree: MemoryTree = [
        ("Cargo.toml", manifest.as_str()),
        (
            "src/lib.rs",
            "pub use serde::Serialize;\npub use serde;\npub use std::fmt::*;\npub use p_one::X;\npub use p_one::inner::*;\npub use p_one as one;\npub fn own() {}\nuse serde::Deserialize;\npub extern crate alloc;\npub extern crate self as me;\n",
        ),
        (
            "p_one/Cargo.toml",
            "[package]\nname = \"p_one\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        ),
        (
            "p_one/src/lib.rs",
            "pub struct X;\npub mod inner { pub fn f() {} }\n",
        ),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &[".", "p_one"]).unwrap_or_else(|e| panic!("{e}"));
    let listed = exports(&ws, TargetKind::Lib, false);
    let items: Vec<&str> = listed
        .items
        .iter()
        .map(|export| export.path.as_str())
        .collect();
    assert_eq!(items, ["demo::me", "demo::own"]);
    assert_eq!(listed.items[0].kind, SymbolKind::ExternCrate);
    let reexports: Vec<(&str, &str, Option<String>, usize)> = listed
        .reexports
        .iter()
        .map(|reexport| {
            (
                reexport.path.as_str(),
                reexport.written.as_str(),
                reexport
                    .item
                    .and_then(|id| ws.item(id))
                    .map(|item| item.path.to_string()),
                reexport.line,
            )
        })
        .collect();
    assert_eq!(
        reexports,
        [
            ("demo::*", "std::fmt::*", None, 3),
            ("demo::Serialize", "serde::Serialize", None, 1),
            ("demo::X", "p_one::X", Some("p_one::X".to_owned()), 4),
            ("demo::alloc", "alloc", None, 9),
            (
                "demo::f",
                "p_one::inner::*",
                Some("p_one::inner::f".to_owned()),
                5
            ),
            ("demo::one", "p_one", Some("p_one".to_owned()), 6),
            ("demo::serde", "serde", None, 2),
        ]
    );
    assert!(listed.reexports.iter().all(|reexport| !reexport.doc_hidden));
    assert_eq!(listed.reexports[0].file.as_str(), "src/lib.rs");
}

#[test]
fn a_re_export_of_a_path_the_index_cannot_follow_is_listed_as_a_re_export() {
    let ws = load(&[(
        "src/lib.rs",
        "macro_rules! make { () => { pub mod generated { pub struct Thing; } } }\nmake!();\npub use self::generated::Thing;\npub fn own() {}\n",
    )]);
    let listed = exports(&ws, TargetKind::Lib, false);
    assert_eq!(surface(&ws), pairs(&[("demo::own", "demo::own")]));
    let reexports: Vec<(&str, &str, Option<_>)> = listed
        .reexports
        .iter()
        .map(|reexport| {
            (
                reexport.path.as_str(),
                reexport.written.as_str(),
                reexport.item,
            )
        })
        .collect();
    assert_eq!(reexports, [("demo::Thing", "self::generated::Thing", None)]);
}

#[test]
fn enum_variants_and_struct_fields_are_not_items_of_the_surface() {
    assert_eq!(
        paths_of(
            "pub enum E { A, B(u8), C { x: u8 } }\npub struct S { pub x: u8, y: u8 }\npub struct Unit;\npub struct Tuple(pub u8);\npub union U { pub a: u8 }\npub use self::E::*;\n"
        ),
        strings(&["demo::E", "demo::S", "demo::Tuple", "demo::U", "demo::Unit"])
    );
}

#[test]
fn the_pub_methods_of_a_pub_type_are_exported_and_those_of_a_private_type_are_not() {
    let ws = load(&[(
        "src/lib.rs",
        "pub struct T;\nimpl T {\n    pub fn new() -> Self { T }\n    fn private(&self) {}\n    pub(crate) fn krate(&self) {}\n    pub const K: u8 = 1;\n}\nimpl Clone for T {\n    fn clone(&self) -> Self { T }\n}\nmod imp {\n    impl super::T {\n        pub fn elsewhere(&self) {}\n    }\n}\nstruct U;\nimpl U {\n    pub fn new() -> Self { U }\n}\nmod m {\n    pub struct V;\n    impl V { pub fn make() {} }\n}\npub use m::V as W;\n#[cfg(test)]\nimpl T {\n    pub fn only_in_tests(&self) {}\n}\n",
    )]);
    assert_eq!(
        surface(&ws),
        pairs(&[
            ("demo::T", "demo::T"),
            ("demo::T::K", "demo::T::K"),
            ("demo::T::elsewhere", "demo::T::elsewhere"),
            ("demo::T::new", "demo::T::new"),
            ("demo::W", "demo::m::V"),
            ("demo::W::make", "demo::m::V::make"),
        ])
    );
    let kinds: Vec<SymbolKind> = exports(&ws, TargetKind::Lib, false)
        .items
        .iter()
        .map(|export| export.kind)
        .collect();
    assert_eq!(
        kinds,
        [
            SymbolKind::Struct,
            SymbolKind::AssocConst,
            SymbolKind::InherentMethod,
            SymbolKind::InherentMethod,
            SymbolKind::Struct,
            SymbolKind::InherentMethod,
        ]
    );
}

#[test]
fn the_members_of_a_pub_trait_are_exported_and_those_of_a_private_trait_are_not() {
    let ws = load(&[(
        "src/lib.rs",
        "pub trait Tr {\n    fn req(&self);\n    fn dflt(&self) {}\n    const K: u8;\n    type A;\n}\ntrait Private { fn x(); }\npub struct S;\nimpl Tr for S {\n    fn req(&self) {}\n    const K: u8 = 1;\n    type A = u8;\n}\n",
    )]);
    assert_eq!(
        surface(&ws),
        pairs(&[
            ("demo::S", "demo::S"),
            ("demo::Tr", "demo::Tr"),
            ("demo::Tr::A", "demo::Tr::A"),
            ("demo::Tr::K", "demo::Tr::K"),
            ("demo::Tr::dflt", "demo::Tr::dflt"),
            ("demo::Tr::req", "demo::Tr::req"),
        ])
    );
}

#[test]
fn doc_hidden_items_are_left_out_by_default_and_flagged_when_included() {
    let ws = load(&[
        (
            "src/lib.rs",
            "#[doc(hidden)]\npub fn h() {}\npub fn v() {}\n#[doc(hidden)]\npub mod hm { pub fn inside() {} }\nmod inner { pub fn r() {} pub fn shown() {} }\n#[doc(hidden)]\npub use inner::r;\npub use inner::shown;\npub struct S;\nimpl S {\n    #[doc(hidden)]\n    pub fn secret(&self) {}\n    pub fn open(&self) {}\n}\npub mod filed;\nmod globbed { pub fn g() {} }\n#[doc(hidden)]\npub use globbed::*;\n",
        ),
        (
            "src/filed.rs",
            "#![doc(hidden)]\npub fn in_hidden_file() {}\n",
        ),
    ]);
    assert_eq!(
        surface(&ws),
        pairs(&[
            ("demo::S", "demo::S"),
            ("demo::S::open", "demo::S::open"),
            ("demo::shown", "demo::inner::shown"),
            ("demo::v", "demo::v"),
        ])
    );
    let all: Vec<(String, bool)> = exports(&ws, TargetKind::Lib, true)
        .items
        .into_iter()
        .map(|export| (export.path, export.doc_hidden))
        .collect();
    assert_eq!(
        all,
        [
            ("demo::S".to_owned(), false),
            ("demo::S::open".to_owned(), false),
            ("demo::S::secret".to_owned(), true),
            ("demo::filed".to_owned(), true),
            ("demo::filed::in_hidden_file".to_owned(), true),
            ("demo::g".to_owned(), true),
            ("demo::h".to_owned(), true),
            ("demo::hm".to_owned(), true),
            ("demo::hm::inside".to_owned(), true),
            ("demo::r".to_owned(), true),
            ("demo::shown".to_owned(), false),
            ("demo::v".to_owned(), false),
        ]
    );
    // A module that a hidden route and a shown one both reach is read under
    // the shown one.
    let both = load(&[(
        "src/lib.rs",
        "mod outer { pub mod m { pub fn f() {} } }\n#[doc(hidden)]\npub use outer::m as a;\npub use outer::m as b;\n",
    )]);
    let listed: Vec<(String, bool)> = exports(&both, TargetKind::Lib, true)
        .items
        .into_iter()
        .map(|export| (export.path, export.doc_hidden))
        .collect();
    assert_eq!(
        listed,
        [
            ("demo::a".to_owned(), true),
            ("demo::b".to_owned(), false),
            ("demo::b::f".to_owned(), false),
        ]
    );
}

#[test]
fn test_code_is_not_part_of_the_surface_beside_the_shipped_items() {
    let ws = load(&[
        (
            "src/lib.rs",
            "#[cfg(test)]\npub mod tests { pub fn t() {} }\n#[cfg(test)]\npub fn only_test() {}\npub fn shipped() {}\nmod inner { pub fn x() {} pub fn y() {} }\n#[cfg(test)]\npub use inner::x;\n#[cfg(test)]\npub use inner::*;\n#[test]\nfn case() {}\n#[cfg(all(test, unix))]\npub mod unix_tests;\n",
        ),
        ("src/unix_tests.rs", "pub fn in_a_test_file() {}\n"),
    ]);
    assert_eq!(surface(&ws), pairs(&[("demo::shipped", "demo::shipped")]));
    // The neighbour: the same module and re-exports outside test code.
    assert_eq!(
        paths_of(
            "pub mod tests { pub fn t() {} }\nmod inner { pub fn x() {} }\npub use inner::x;\n"
        ),
        strings(&["demo::tests", "demo::tests::t", "demo::x"])
    );
}

#[test]
fn an_exported_macro_is_exported_and_a_local_or_generated_one_is_not() {
    let ws = load(&[(
        "src/lib.rs",
        "#[macro_export]\nmacro_rules! exported { () => {} }\nmacro_rules! local { () => {} }\npub(crate) use local;\nmacro_rules! make { () => { pub fn generated() {} } }\nmake!();\nmod deep {\n    #[macro_export]\n    macro_rules! from_deep { () => {} }\n}\n",
    )]);
    assert_eq!(
        surface(&ws),
        pairs(&[
            ("demo::exported", "demo::exported"),
            ("demo::from_deep", "demo::from_deep"),
        ])
    );
    assert!(
        exports(&ws, TargetKind::Lib, false)
            .items
            .iter()
            .all(|export| export.kind == SymbolKind::MacroRules)
    );
    // The flag the listing reads is the item's own.
    let flags: Vec<(&str, bool)> = ws
        .items()
        .filter(|item| item.kind == SymbolKind::MacroRules)
        .map(|item| (item.path.as_str(), item.macro_export))
        .collect();
    assert_eq!(
        flags,
        [
            ("demo::exported", true),
            ("demo::from_deep", true),
            ("demo::local", false),
            ("demo::make", false),
        ]
    );
}

#[test]
fn a_binary_lists_its_own_pub_items_and_a_test_target_is_refused() {
    let ws = load(&[
        ("src/lib.rs", "pub fn f() {}\n"),
        (
            "src/main.rs",
            "pub fn helper() {}\nfn main() {}\nmod m { pub fn x() {} }\npub use m::x;\npub use demo::f;\n",
        ),
        ("tests/it.rs", "pub fn in_test() {}\n"),
    ]);
    let bin = exports(&ws, TargetKind::Bin, false);
    let items: Vec<&str> = bin
        .items
        .iter()
        .map(|export| export.path.as_str())
        .collect();
    assert_eq!(
        items,
        ["demo::bin_crate::demo::helper", "demo::bin_crate::demo::x"]
    );
    let reexports: Vec<&str> = bin
        .reexports
        .iter()
        .map(|reexport| reexport.path.as_str())
        .collect();
    assert_eq!(reexports, ["demo::bin_crate::demo::f"]);
    let test = target(&ws, TargetKind::Test);
    assert!(matches!(
        ws.exported_items(test),
        Err(ExportError::NotLibraryOrBinary(id)) if id == test
    ));
    let error = ws
        .exported_items(test)
        .expect_err("a test target")
        .to_string();
    assert!(error.contains("not a library or a binary"), "{error}");
    // The neighbour: the library of the same workspace lists its item.
    assert_eq!(surface(&ws), pairs(&[("demo::f", "demo::f")]));
}

#[test]
fn a_module_reached_by_two_pub_paths_is_listed_under_each_and_read_under_the_first() {
    assert_eq!(
        surface(&load(&[(
            "src/lib.rs",
            "pub mod a { pub fn f() {} }\npub use a as b;\n",
        )])),
        pairs(&[
            ("demo::a", "demo::a"),
            ("demo::a::f", "demo::a::f"),
            ("demo::b", "demo::a"),
        ])
    );
    // A module that globs its parent, which globs it, is read once.
    assert_eq!(
        paths_of("pub mod a { pub use super::*; }\npub use a::*;\npub fn f() {}\n"),
        strings(&["demo::a", "demo::a::a", "demo::a::f", "demo::f"])
    );
}

#[test]
fn the_surface_is_the_same_in_every_load_and_in_every_order_of_the_declarations() {
    let decls = [
        "pub mod a { pub fn f() {} pub struct S; impl S { pub fn m() {} } }",
        "mod b { pub fn g() {} pub trait T { fn t(); } }",
        "pub use b::*;",
        "pub use a::f as renamed;",
    ];
    let mut first: Option<Exports> = None;
    for order in [[0, 1, 2, 3], [3, 2, 1, 0], [2, 0, 3, 1], [1, 3, 0, 2]] {
        let mut lib = String::new();
        for at in order {
            writeln!(lib, "{}", decls[at]).expect("write");
        }
        let ws = load(&[("src/lib.rs", lib.as_str())]);
        let listed = exports(&ws, TargetKind::Lib, true);
        assert_eq!(listed, exports(&ws, TargetKind::Lib, true));
        let paths: Vec<&str> = listed.items.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "demo::T",
                "demo::T::t",
                "demo::a",
                "demo::a::S",
                "demo::a::S::m",
                "demo::a::f",
                "demo::g",
                "demo::renamed",
            ]
        );
        assert!(listed.items.windows(2).all(|w| w[0].path <= w[1].path));
        match &first {
            Some(known) => {
                let shape = |e: &Exports| -> Vec<(String, SymbolKind)> {
                    e.items.iter().map(|i| (i.path.clone(), i.kind)).collect()
                };
                assert_eq!(shape(known), shape(&listed));
            }
            None => first = Some(listed),
        }
    }
}

#[test]
fn an_ambiguous_a_cyclic_or_an_unresolved_public_name_fails_beside_a_neighbour_that_lists() {
    let ambiguous = load(&[(
        "src/lib.rs",
        "mod a { pub fn x() {} }\nmod b { pub fn x() {} }\npub use a::*;\npub use b::*;\n",
    )]);
    assert!(matches!(
        ambiguous.exported_items(target(&ambiguous, TargetKind::Lib)),
        Err(ExportError::Resolve(ResolveError::Ambiguous(_)))
    ));
    assert_eq!(
        paths_of(
            "mod a { pub fn x() {} }\nmod b { pub fn y() {} }\npub use a::*;\npub use b::*;\n"
        ),
        strings(&["demo::x", "demo::y"])
    );
    let cyclic = load(&[(
        "src/lib.rs",
        "pub mod a { pub use crate::b::X; }\npub mod b { pub use crate::a::X; }\n",
    )]);
    let error = cyclic
        .exported_items(target(&cyclic, TargetKind::Lib))
        .expect_err("a cycle of imports");
    assert!(
        matches!(&error, ExportError::Resolve(ResolveError::Cyclic(path)) if path.ends_with("::X")),
        "{error}"
    );
    assert_eq!(
        paths_of("pub mod a { pub use crate::b::X; }\npub mod b { pub struct X; }\n"),
        strings(&["demo::a", "demo::a::X", "demo::b", "demo::b::X"])
    );
    let broken = load(&[("src/lib.rs", "pub mod m { pub use self::missing::*; }\n")]);
    let error = broken
        .exported_items(target(&broken, TargetKind::Lib))
        .expect_err("a glob of nothing");
    assert!(
        matches!(&error, ExportError::Resolve(ResolveError::UnresolvedGlob { glob, module, .. })
            if glob == "self::missing" && module == "demo::m"),
        "{error}"
    );
    // A private glob of nothing beside the module's own item does not read
    // into the surface.
    assert_eq!(
        paths_of("pub mod m { use self::missing::*; pub fn f() {} }\n"),
        strings(&["demo::m", "demo::m::f"])
    );
}

#[test]
fn a_surface_over_the_item_budget_left_by_the_load_is_refused_beside_one_within_it() {
    let mut lib = String::new();
    for n in 0..200 {
        writeln!(lib, "pub fn function_with_a_long_name_{n}() {{}}").expect("write");
    }
    let files = [("src/lib.rs", lib.as_str())];
    let loaded = load(&files);
    let spent = loaded.usage().item_bytes;
    let mut options = LoadOptions::default();
    options.max_item_bytes = spent + 1000;
    let tight = Workspace::load_with(&tree(&files), &["."], &options).expect("the load fits");
    let error = tight
        .exported_items(target(&tight, TargetKind::Lib))
        .expect_err("over the budget");
    match &error {
        ExportError::Budget(load) => {
            assert!(
                matches!(
                    load.kind,
                    LoadErrorKind::Budget {
                        budget: Budget::ItemBytes,
                        limit
                    } if limit == spent + 1000
                ),
                "{load}"
            );
            assert_eq!(load.file.as_str(), "src/lib.rs");
            assert!(load.line.is_some());
        }
        other => panic!("{other}"),
    }
    assert!(error.to_string().contains("over budget"), "{error}");
    options.max_item_bytes = spent + 200_000;
    let roomy = Workspace::load_with(&tree(&files), &["."], &options).expect("the load fits");
    assert_eq!(
        roomy
            .exported_items(target(&roomy, TargetKind::Lib))
            .expect("within the budget")
            .items
            .len(),
        200
    );
}

#[test]
fn twenty_thousand_chained_re_exports_are_listed_on_a_two_mib_thread() {
    let mut lib = String::from("mod m { pub struct T; }\npub use m::T as A0;\n");
    for n in 1..20_000 {
        writeln!(lib, "pub use A{} as A{n};", n - 1).expect("write");
    }
    let listed = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let ws = load(&[("src/lib.rs", lib.as_str())]);
            let listed = ws
                .exported_items(target(&ws, TargetKind::Lib))
                .unwrap_or_else(|e| panic!("{e}"));
            (listed.items.len(), listed.items[0].path.clone())
        })
        .expect("spawn")
        .join()
        .expect("no overflow");
    assert_eq!(listed, (20_000, "demo::A0".to_owned()));
}

#[test]
fn a_chain_of_twenty_thousand_pub_globs_is_listed_on_a_two_mib_thread() {
    let mut lib = String::from("pub use m0::*;\n");
    for n in 0..20_000 {
        writeln!(lib, "mod m{n} {{ pub use super::m{}::*; }}", n + 1).expect("write");
    }
    writeln!(lib, "mod m20000 {{ pub fn last() {{}} }}").expect("write");
    let listed = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let ws = load(&[("src/lib.rs", lib.as_str())]);
            ws.exported_items(target(&ws, TargetKind::Lib))
                .unwrap_or_else(|e| panic!("{e}"))
                .items
                .into_iter()
                .map(|export| export.path)
                .collect::<Vec<_>>()
        })
        .expect("spawn")
        .join()
        .expect("no overflow");
    assert_eq!(listed, ["demo::last"]);
}

#[test]
fn the_example_of_the_contract_lists_one_item_and_more_through_a_glob_or_a_pub_module() {
    let inner = ("src/inner.rs", "pub fn go() {}\npub fn unused() {}\n");
    let root = "mod inner;\npub use inner::go as run;\npub(crate) fn helper() {}\n";
    assert_eq!(
        surface(&load(&[("src/lib.rs", root), inner])),
        pairs(&[("demo::run", "demo::inner::go")])
    );
    let globbed = format!("{root}pub use inner::*;\n");
    assert_eq!(
        surface(&load(&[("src/lib.rs", globbed.as_str()), inner])),
        pairs(&[
            ("demo::go", "demo::inner::go"),
            ("demo::run", "demo::inner::go"),
            ("demo::unused", "demo::inner::unused"),
        ])
    );
    let open = root.replace("mod inner;", "pub mod inner;");
    assert_eq!(
        surface(&load(&[("src/lib.rs", open.as_str()), inner])),
        pairs(&[
            ("demo::inner", "demo::inner"),
            ("demo::inner::go", "demo::inner::go"),
            ("demo::inner::unused", "demo::inner::unused"),
            ("demo::run", "demo::inner::go"),
        ])
    );
}

/// Each re-export as its path, written path and whether it is hidden.
fn reexports_of(ws: &Workspace, include_hidden: bool) -> Vec<(String, String, bool)> {
    exports(ws, TargetKind::Lib, include_hidden)
        .reexports
        .into_iter()
        .map(|reexport| (reexport.path, reexport.written, reexport.doc_hidden))
        .collect()
}

fn triples(list: &[(&str, &str, bool)]) -> Vec<(String, String, bool)> {
    list.iter()
        .map(|(a, b, c)| ((*a).to_owned(), (*b).to_owned(), *c))
        .collect()
}

const OUTSIDE: &str = "pub use std::fmt::Debug;\n    pub use std::collections::*;\n    pub use std::fmt::Display as Disp;\n    pub(crate) use std::fmt::Write;\n    pub fn own() {}\n";

#[test]
fn a_re_export_of_an_outside_crate_that_a_pub_glob_carries_is_a_re_export_of_the_reader() {
    // The neighbour that names it: an explicit `pub use a::Debug`.
    let named = load(&[(
        "src/lib.rs",
        "mod a { pub use std::fmt::Debug; }\npub use a::Debug;\n",
    )]);
    assert_eq!(
        reexports_of(&named, false),
        triples(&[("demo::Debug", "a::Debug", false)])
    );
    // A glob of a private module carries the module's re-exports, renamed
    // ones under their new name, and not a `pub(crate)` one.
    let globbed = load(&[(
        "src/lib.rs",
        format!("mod a {{\n    {OUTSIDE}}}\npub use a::*;\n").as_str(),
    )]);
    assert_eq!(surface(&globbed), pairs(&[("demo::own", "demo::a::own")]));
    assert_eq!(
        reexports_of(&globbed, false),
        triples(&[
            ("demo::*", "std::collections::*", false),
            ("demo::Debug", "std::fmt::Debug", false),
            ("demo::Disp", "std::fmt::Display", false),
        ])
    );
    // A `pub` module is listed under its own path and under the glob's.
    let open = load(&[(
        "src/lib.rs",
        format!("pub mod a {{\n    {OUTSIDE}}}\npub use a::*;\n").as_str(),
    )]);
    assert_eq!(
        reexports_of(&open, false),
        triples(&[
            ("demo::*", "std::collections::*", false),
            ("demo::Debug", "std::fmt::Debug", false),
            ("demo::Disp", "std::fmt::Display", false),
            ("demo::a::*", "std::collections::*", false),
            ("demo::a::Debug", "std::fmt::Debug", false),
            ("demo::a::Disp", "std::fmt::Display", false),
        ])
    );
    // Through two globs, and hidden by the reader's hidden glob; a hidden glob
    // further on hides nothing, as for an item.
    let chained = load(&[(
        "src/lib.rs",
        "mod b { pub use std::fmt::Debug; }\nmod a { pub use crate::b::*; }\npub use a::*;\nmod c { pub use std::fmt::Pointer; }\nmod d { pub use crate::c::*; }\n#[doc(hidden)]\npub use d::*;\nmod e { pub use std::fmt::Write; }\nmod f { #[doc(hidden)] pub use crate::e::*; }\npub use f::*;\n",
    )]);
    assert_eq!(
        reexports_of(&chained, false),
        triples(&[
            ("demo::Debug", "std::fmt::Debug", false),
            ("demo::Write", "std::fmt::Write", false),
        ])
    );
    assert_eq!(
        reexports_of(&chained, true),
        triples(&[
            ("demo::Debug", "std::fmt::Debug", false),
            ("demo::Pointer", "std::fmt::Pointer", true),
            ("demo::Write", "std::fmt::Write", false),
        ])
    );
}

#[test]
fn a_carried_outside_re_export_is_kept_out_by_a_restricted_glob_or_a_name_on_the_way() {
    for lib in [
        // A glob that is not `pub` carries nothing out.
        "mod a { pub use std::fmt::Debug; }\npub(crate) use a::*;\n",
        "mod a { pub use std::fmt::Debug; }\nuse a::*;\n",
        // A re-export that is not `pub` is not carried.
        "mod a { pub(crate) use std::fmt::Debug; }\npub use a::*;\n",
        // A name the reader binds itself shadows what the glob brings in.
        "mod a { pub use std::fmt::Debug; }\npub use a::*;\nfn Debug() {}\n",
        // So does one that a module on the way binds.
        "mod b { pub use std::fmt::Debug; }\nmod a { pub use crate::b::*; pub(crate) struct Debug; }\npub use a::*;\n",
        // And a test re-export or glob carries nothing.
        "mod a { #[cfg(test)] pub use std::fmt::Debug; }\npub use a::*;\n",
        "mod a { pub use std::fmt::Debug; }\n#[cfg(test)]\npub use a::*;\n",
    ] {
        let ws = load(&[("src/lib.rs", lib)]);
        assert_eq!(reexports_of(&ws, true), triples(&[]), "{lib}");
    }
}

#[test]
fn a_named_re_export_of_an_enum_variant_is_a_re_export_and_a_glob_of_an_enum_adds_nothing() {
    let ws = load(&[(
        "src/lib.rs",
        "pub mod p { pub enum F { G, H } }\npub use p::F::G;\npub use p::F::*;\n",
    )]);
    assert_eq!(
        surface(&ws),
        pairs(&[("demo::p", "demo::p"), ("demo::p::F", "demo::p::F")])
    );
    assert_eq!(
        reexports_of(&ws, false),
        triples(&[("demo::G", "p::F::G", false)])
    );
}

#[test]
fn a_module_reached_by_a_hidden_and_a_shown_glob_carries_its_re_export_shown() {
    let ws = load(&[(
        "src/lib.rs",
        "mod c { pub use std::cell::Cell; }\nmod h { pub use crate::c::*; }\nmod s { pub use crate::c::*; }\n#[doc(hidden)]\npub use h::*;\npub use s::*;\n",
    )]);
    assert_eq!(
        reexports_of(&ws, false),
        triples(&[("demo::Cell", "std::cell::Cell", false)])
    );
    // The neighbour: with the shown route gone, the re-export is hidden.
    let hidden = load(&[(
        "src/lib.rs",
        "mod c { pub use std::cell::Cell; }\nmod h { pub use crate::c::*; }\n#[doc(hidden)]\npub use h::*;\n",
    )]);
    assert_eq!(reexports_of(&hidden, false), triples(&[]));
    assert_eq!(
        reexports_of(&hidden, true),
        triples(&[("demo::Cell", "std::cell::Cell", true)])
    );
}

#[test]
fn a_carried_re_export_is_kept_out_exactly_where_every_route_passes_a_module_that_binds_its_name() {
    let carried = |lib: &str| -> bool {
        let ws = load(&[("src/lib.rs", lib)]);
        let listed = reexports_of(&ws, true);
        assert!(listed.len() <= 1, "{lib}: {listed:?}");
        listed.len() == 1
    };
    let cell = "mod c { pub use std::cell::Cell; }\nmod z {}\n";
    for (shape, expected) in [
        // A binder on the one route dominates the carrier.
        (
            "mod a { fn Cell() {} pub use crate::c::*; }\npub use a::*;\n",
            false,
        ),
        // A binder beside a free route.
        (
            "mod a { fn Cell() {} pub use crate::c::*; }\npub use a::*;\npub use c::*;\n",
            true,
        ),
        // A binder with routes of its own that lead elsewhere.
        (
            "mod a { fn Cell() {} pub use crate::z::*; }\npub use a::*;\npub use c::*;\n",
            true,
        ),
        // Two binders, one on each of the two routes: no free route.
        (
            "mod a { fn Cell() {} pub use crate::c::*; }\nmod b { struct Cell; pub use crate::c::*; }\npub use a::*;\npub use b::*;\n",
            false,
        ),
        // Two binders, a third route free of both.
        (
            "mod a { fn Cell() {} pub use crate::c::*; }\nmod b { struct Cell; pub use crate::c::*; }\nmod f { pub use crate::c::*; }\npub use a::*;\npub use b::*;\npub use f::*;\n",
            true,
        ),
        // Two binders on one route behind a free module.
        (
            "mod a { fn Cell() {} pub use crate::b::*; }\nmod b { struct Cell; pub use crate::c::*; }\nmod f { pub use crate::a::*; pub use crate::c::*; }\npub use f::*;\n",
            true,
        ),
        // The reader binds the name.
        ("pub use c::*;\nfn Cell() {}\n", false),
    ] {
        let lib = format!("{cell}{shape}");
        assert_eq!(carried(&lib), expected, "{lib}");
    }
}

#[test]
fn an_item_and_a_carried_outside_re_export_of_one_name_are_both_listed_and_not_ambiguous() {
    let ws = load(&[(
        "src/lib.rs",
        "mod a { pub struct Rc; }\nmod b { pub use std::rc::Rc; }\npub use a::*;\npub use b::*;\n",
    )]);
    assert_eq!(surface(&ws), pairs(&[("demo::Rc", "demo::a::Rc")]));
    assert_eq!(
        reexports_of(&ws, false),
        triples(&[("demo::Rc", "std::rc::Rc", false)])
    );
}

#[test]
fn searches_for_names_that_several_binders_keep_from_a_long_route_are_refused_by_the_item_budget() {
    // A chain of globs to a module carrying many names, each of which two
    // modules reached beside the chain bind: each name needs a search of its
    // own along the chain.
    let chain = |links: usize, names: usize| {
        let mut lib = String::from("mod z {}\npub use c0::*;\n");
        for i in 0..links {
            writeln!(lib, "mod c{i} {{ pub use crate::c{}::*; }}", i + 1).expect("write");
        }
        writeln!(
            lib,
            "mod c{links} {{ pub use crate::m::*; pub use crate::a::*; pub use crate::b::*; }}"
        )
        .expect("write");
        let (mut carried, mut bound) = (String::new(), String::new());
        for i in 0..names {
            writeln!(carried, "pub use std::cell::Cell as C{i};").expect("write");
            writeln!(bound, "fn C{i}() {{}}").expect("write");
        }
        writeln!(
            lib,
            "mod m {{ {carried} }}\nmod a {{ {bound} pub use crate::z::*; }}\nmod b {{ {bound} pub use crate::z::*; }}"
        )
        .expect("write");
        lib
    };
    let large = chain(5000, 2000);
    let ws = load(&[("src/lib.rs", large.as_str())]);
    let error = ws
        .exported_items(target(&ws, TargetKind::Lib))
        .expect_err("over the item budget");
    // The refusal names the re-export being decided and its declaration.
    let ExportError::CarryBudget {
        path,
        error: refusal,
    } = &error
    else {
        panic!("{error}");
    };
    assert!(
        matches!(
            refusal.kind,
            LoadErrorKind::Budget {
                budget: Budget::ItemBytes,
                ..
            }
        ),
        "{refusal}"
    );
    assert!(path.starts_with("demo::C"), "{path}");
    let name = path.strip_prefix("demo::").expect("a path of the reader");
    let declaration = format!("pub use std::cell::Cell as {name};");
    let line = large
        .lines()
        .position(|text| text.trim_start_matches("mod m {").trim() == declaration)
        .expect("the carried declaration")
        + 1;
    assert_eq!(
        (refusal.file.as_str(), refusal.line),
        ("src/lib.rs", Some(line))
    );
    assert!(error.to_string().contains(&format!("`{path}`")), "{error}");
    // The neighbour: a short chain lists every name.
    let small = chain(50, 200);
    let ws = load(&[("src/lib.rs", small.as_str())]);
    assert_eq!(
        ws.exported_items(target(&ws, TargetKind::Lib))
            .unwrap_or_else(|e| panic!("{e}"))
            .reexports
            .len(),
        200
    );
}

#[test]
fn a_re_export_that_only_a_hidden_glob_carries_past_a_shadow_is_hidden_as_an_item_would_be() {
    let shape = |carried: &str| {
        format!(
            "mod c {{ {carried} }}\nmod p {{ pub use crate::c::*; struct Cell; }}\nmod q {{ pub use crate::c::*; }}\npub use p::*;\n#[doc(hidden)]\npub use q::*;\n"
        )
    };
    // The item: the shown route through `p` is shadowed by its `Cell`.
    let item = load(&[("src/lib.rs", shape("pub struct Cell;").as_str())]);
    assert_eq!(surface(&item), pairs(&[]));
    let all: Vec<(String, bool)> = exports(&item, TargetKind::Lib, true)
        .items
        .into_iter()
        .map(|export| (export.path, export.doc_hidden))
        .collect();
    assert_eq!(all, [("demo::Cell".to_owned(), true)]);
    // The re-export of a crate outside the workspace, alike.
    let outside = load(&[("src/lib.rs", shape("pub use std::cell::Cell;").as_str())]);
    assert_eq!(reexports_of(&outside, false), triples(&[]));
    assert_eq!(
        reexports_of(&outside, true),
        triples(&[("demo::Cell", "std::cell::Cell", true)])
    );
    // Two shadowed shown routes and a free hidden third.
    let third = load(&[(
        "src/lib.rs",
        "mod c { pub use std::cell::Cell; }\nmod p { pub use crate::c::*; struct Cell; }\nmod r { pub use crate::c::*; fn Cell() {} }\nmod q { pub use crate::c::*; }\npub use p::*;\npub use r::*;\n#[doc(hidden)]\npub use q::*;\n",
    )]);
    assert_eq!(
        reexports_of(&third, true),
        triples(&[("demo::Cell", "std::cell::Cell", true)])
    );
    // The neighbour: a free shown route shows it.
    let free = load(&[(
        "src/lib.rs",
        "mod c { pub use std::cell::Cell; }\nmod p { pub use crate::c::*; struct Cell; }\nmod q { pub use crate::c::*; }\npub use p::*;\n#[doc(hidden)]\npub use q::*;\npub use c::*;\n",
    )]);
    assert_eq!(
        reexports_of(&free, false),
        triples(&[("demo::Cell", "std::cell::Cell", false)])
    );
}

#[test]
fn a_carrier_with_pub_globs_of_its_own_is_not_its_own_binder() {
    let ws = load(&[(
        "src/lib.rs",
        "mod z { pub fn other() {} }\nmod c { pub use std::cell::Cell; pub use crate::z::*; }\npub use c::*;\n",
    )]);
    assert_eq!(
        reexports_of(&ws, false),
        triples(&[("demo::Cell", "std::cell::Cell", false)])
    );
    assert_eq!(surface(&ws), pairs(&[("demo::other", "demo::z::other")]));
}

#[test]
fn of_two_carriers_of_one_name_only_the_one_with_a_free_route_is_listed_with_its_own_line() {
    // `c1` is reached only through the binders `a` and `b`; `c2` also
    // directly. Both orders of the declarations.
    for (first, second) in [("c1", "c2"), ("c2", "c1")] {
        let lib = format!(
            "mod {first} {{ pub use std::cell::Cell; }}\nmod {second} {{ pub use std::cell::Cell; }}\nmod a {{ struct Cell; pub use crate::c1::*; pub use crate::c2::*; }}\nmod b {{ struct Cell; pub use crate::c1::*; }}\npub use a::*;\npub use b::*;\npub use c2::*;\n"
        );
        let ws = load(&[("src/lib.rs", lib.as_str())]);
        let listed: Vec<(String, usize)> = exports(&ws, TargetKind::Lib, true)
            .reexports
            .into_iter()
            .map(|reexport| (reexport.path, reexport.line))
            .collect();
        let line = if second == "c2" { 2 } else { 1 };
        assert_eq!(listed, [("demo::Cell".to_owned(), line)], "{lib}");
    }
}

#[test]
fn a_binder_in_an_irreducible_cycle_that_dominates_nothing_leaves_its_name_carried() {
    // The carrier `c` and the modules `b` and `d` glob one another: the cycle
    // is entered at `b` and at `d`, and only `b` binds the name, so the route
    // through `d` carries it. A first pass over the cycle alone would find
    // `b` on every route to `c`.
    let ws = load(&[(
        "src/lib.rs",
        "mod a {}\nmod c { pub use std::cell::Cell; pub use crate::a::*; pub use crate::b::*; pub use crate::d::*; }\nmod b { struct Cell; pub use crate::c::*; }\nmod d { pub use crate::c::*; }\npub use a::*;\npub use b::*;\npub use d::*;\n",
    )]);
    assert_eq!(
        reexports_of(&ws, false),
        triples(&[("demo::Cell", "std::cell::Cell", false)])
    );
    // The neighbour: with the route through `d` gone, `b` is on every route.
    let ws = load(&[(
        "src/lib.rs",
        "mod a {}\nmod c { pub use std::cell::Cell; pub use crate::a::*; pub use crate::b::*; }\nmod b { struct Cell; pub use crate::c::*; }\npub use a::*;\npub use b::*;\n",
    )]);
    assert_eq!(reexports_of(&ws, true), triples(&[]));
}

#[test]
fn a_hidden_glob_partway_along_a_route_hides_a_carried_re_export_no_more_than_an_item() {
    for carried in ["pub struct Cell;", "pub use std::cell::Cell;"] {
        for lib in [
            format!(
                "mod c {{ {carried} }}\nmod q {{ #[doc(hidden)] pub use crate::c::*; }}\npub use q::*;\n"
            ),
            format!(
                "mod c {{ {carried} }}\nmod p {{ pub use crate::c::*; struct Cell; }}\nmod q {{ #[doc(hidden)] pub use crate::c::*; }}\npub use p::*;\npub use q::*;\n"
            ),
        ] {
            let ws = load(&[("src/lib.rs", lib.as_str())]);
            assert_eq!(cells(&ws), [("demo::Cell".to_owned(), false)], "{lib}");
        }
    }
}

/// Every path of the library's surface that ends in `Cell`, item or
/// re-export, hidden ones included, with whether it is hidden; or the error.
fn cells(ws: &Workspace) -> Vec<(String, bool)> {
    let mut options = ExportOptions::default();
    options.include_hidden = true;
    match ws.exported_items_with(target(ws, TargetKind::Lib), &options) {
        Ok(listed) => listed
            .items
            .iter()
            .map(|item| (item.path.clone(), item.doc_hidden))
            .chain(
                listed
                    .reexports
                    .iter()
                    .map(|reexport| (reexport.path.clone(), reexport.doc_hidden)),
            )
            .filter(|(path, _)| path.ends_with("::Cell"))
            .collect(),
        Err(error) => vec![(format!("error: {error}"), false)],
    }
}

/// One crate of the differential family: a carrier `c` of `Cell`, reached
/// from the reader through `depth` private modules of `pub` globs, with a
/// binder, a `#[doc(hidden)]` and an extra hidden route placed as the
/// parameters say.
fn carried_case(
    carrier: &str,
    depth: usize,
    binder: &str,
    hidden: &str,
    in_module: bool,
    hidden_alt: bool,
) -> Option<String> {
    let mid = |what: &str| depth > 0 && hidden == what;
    if (hidden == "mid glob" || hidden == "mid module") && depth == 0
        || binder == "on route" && depth == 0 && in_module
        || hidden == "reader module" && !in_module
    {
        return None;
    }
    let mut lib = String::new();
    let use_attr = if hidden == "use" {
        "#[doc(hidden)] "
    } else {
        ""
    };
    writeln!(lib, "mod c {{ {use_attr}{carrier} }}").expect("write");
    for at in 1..=depth {
        let next = if at == depth {
            "c".to_owned()
        } else {
            format!("m{}", at + 1)
        };
        let glob_attr = if at == 1 && mid("mid glob") {
            "#[doc(hidden)] "
        } else {
            ""
        };
        let module_attr = if at == 1 && mid("mid module") {
            "#[doc(hidden)]\n"
        } else {
            ""
        };
        let bound = if at == 1 && binder == "on route" {
            " struct Cell;"
        } else {
            ""
        };
        writeln!(
            lib,
            "{module_attr}mod m{at} {{ {glob_attr}pub use crate::{next}::*;{bound} }}"
        )
        .expect("write");
    }
    writeln!(
        lib,
        "mod s {{ struct Cell; }}\nmod alt {{ pub use crate::c::*; }}"
    )
    .expect("write");
    let first = if depth == 0 { "c" } else { "m1" };
    let mut reader = String::new();
    let reader_attr = if hidden == "reader glob" {
        "#[doc(hidden)] "
    } else {
        ""
    };
    writeln!(reader, "{reader_attr}pub use crate::{first}::*;").expect("write");
    if binder == "on route" && depth == 0 {
        reader.push_str("struct Cell;\n");
    }
    if binder == "off route" {
        reader.push_str("pub use crate::s::*;\n");
    }
    if hidden_alt {
        reader.push_str("#[doc(hidden)] pub use crate::alt::*;\n");
    }
    if in_module {
        let module_attr = if hidden == "reader module" {
            "#[doc(hidden)]\n"
        } else {
            ""
        };
        writeln!(lib, "{module_attr}pub mod r {{\n{reader}}}").expect("write");
    } else {
        lib.push_str(&reader);
    }
    Some(lib)
}

#[test]
fn a_carried_re_export_is_listed_hidden_or_absent_exactly_as_an_item_of_the_same_shape() {
    let mut cases = 0;
    for depth in 0..=2 {
        for binder in ["none", "on route", "off route"] {
            for hidden in [
                "none",
                "reader glob",
                "mid glob",
                "use",
                "mid module",
                "reader module",
            ] {
                for in_module in [false, true] {
                    for hidden_alt in [false, true] {
                        let Some(item) = carried_case(
                            "pub struct Cell;",
                            depth,
                            binder,
                            hidden,
                            in_module,
                            hidden_alt,
                        ) else {
                            continue;
                        };
                        let outside = carried_case(
                            "pub use std::cell::Cell;",
                            depth,
                            binder,
                            hidden,
                            in_module,
                            hidden_alt,
                        )
                        .expect("the same shape");
                        let as_item = cells(&load(&[("src/lib.rs", item.as_str())]));
                        let as_reexport = cells(&load(&[("src/lib.rs", outside.as_str())]));
                        assert_eq!(as_reexport, as_item, "{outside}");
                        cases += 1;
                    }
                }
            }
        }
    }
    assert!(cases > 150, "{cases}");
}

#[test]
fn only_a_route_that_carries_a_name_out_decides_whether_it_is_hidden_for_items_and_re_exports() {
    // `m0` keeps `Cell` in behind a private import, and `m1` brings it in
    // only at `pub(crate)`: in both, the one route that exports `Cell` is the
    // hidden glob of `c`, so `Cell` is hidden, item or re-export, at the root
    // and in a `pub` module.
    for shadow in [
        "mod m0 { pub use crate::c::*; use std::rc::Rc as Cell; }\npub use m0::*;",
        "mod m1 { pub(crate) use crate::c::*; }\npub use m1::*;",
    ] {
        for carried in ["pub struct Cell;", "pub use std::cell::Cell;"] {
            for in_module in [false, true] {
                let reader = format!("{shadow}\n#[doc(hidden)]\npub use c::*;\n")
                    .replace("pub use m", "pub use crate::m")
                    .replace("pub use c::*", "pub use crate::c::*");
                let (reader, path) = if in_module {
                    let (modules, uses): (Vec<&str>, Vec<&str>) =
                        reader.lines().partition(|line| line.starts_with("mod "));
                    (
                        format!(
                            "{}\npub mod r {{\n{}\n}}\n",
                            modules.join("\n"),
                            uses.join("\n")
                        ),
                        "demo::r::Cell",
                    )
                } else {
                    (reader, "demo::Cell")
                };
                let lib = format!("mod c {{ {carried} }}\n{reader}");
                let ws = load(&[("src/lib.rs", lib.as_str())]);
                assert_eq!(cells(&ws), [(path.to_owned(), true)], "{lib}");
                // Nothing is listed by default.
                let shown = exports(&ws, TargetKind::Lib, false);
                assert!(
                    shown.items.iter().all(|e| !e.path.ends_with("::Cell"))
                        && shown.reexports.iter().all(|e| !e.path.ends_with("::Cell")),
                    "{lib}"
                );
            }
        }
    }
}

#[test]
fn in_random_small_crates_a_carried_re_export_is_listed_hidden_or_absent_as_the_item_is() {
    let mut outcomes = std::collections::BTreeSet::new();
    for seed in [0x5eed_0001_u64, 0x5eed_0002, 0x5eed_0003] {
        let mut draw = Draw(seed);
        for _ in 0..150 {
            // Both versions are drawn from one state, so they share a shape.
            let state = draw.0;
            let item = random_crate(&mut draw, "pub struct Cell;");
            draw.0 = state;
            let outside = random_crate(&mut draw, "pub use std::cell::Cell;");
            let as_item = cells(&load(&[("src/lib.rs", item.as_str())]));
            let as_reexport = cells(&load(&[("src/lib.rs", outside.as_str())]));
            assert_eq!(as_reexport, as_item, "{outside}");
            outcomes.insert(format!("{as_item:?}"));
        }
    }
    // The family reaches every kind of answer.
    for kind in ["[]", "Cell\", false", "Cell\", true"] {
        assert!(
            outcomes
                .iter()
                .any(|outcome| outcome.contains(kind) || outcome == kind),
            "{kind}: {outcomes:?}"
        );
    }
}

#[test]
fn a_re_export_two_globs_carry_is_written_as_the_first_glob_that_carries_it() {
    let mut manifest = DEMO.to_owned();
    manifest.push_str("[dependencies]\np_one = { path = \"p_one\" }\n");
    let site = |lib: &str| {
        let tree: MemoryTree = [
            ("Cargo.toml", manifest.as_str()),
            ("src/lib.rs", lib),
            (
                "p_one/Cargo.toml",
                "[package]\nname = \"p_one\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            (
                "p_one/src/lib.rs",
                "pub mod inner { pub fn f() {} }\npub mod again { pub use crate::inner::f; }\n",
            ),
        ]
        .into_iter()
        .collect();
        let ws = Workspace::load(&tree, &[".", "p_one"]).unwrap_or_else(|e| panic!("{e}"));
        exports(&ws, TargetKind::Lib, false)
            .reexports
            .iter()
            .map(|reexport| {
                let item = reexport
                    .item
                    .and_then(|id| ws.item(id))
                    .map(|item| item.path.to_string());
                (
                    reexport.path.clone(),
                    reexport.written.clone(),
                    item,
                    reexport.line,
                )
            })
            .collect::<Vec<_>>()
    };
    let f = Some("p_one::inner::f".to_owned());
    // Both globs bring in the same `f`: the re-export is written as the glob
    // that comes first, whichever of the two it is.
    assert_eq!(
        site("pub fn own() {}\npub use p_one::inner::*;\npub use p_one::again::*;\n"),
        [(
            "demo::f".to_owned(),
            "p_one::inner::*".to_owned(),
            f.clone(),
            2
        )]
    );
    assert_eq!(
        site("pub fn own() {}\npub use p_one::again::*;\npub use p_one::inner::*;\n"),
        [(
            "demo::f".to_owned(),
            "p_one::again::*".to_owned(),
            f.clone(),
            2
        )]
    );
    // The neighbour with one glob is written as that glob.
    assert_eq!(
        site("pub fn own() {}\n\npub use p_one::again::*;\n"),
        [("demo::f".to_owned(), "p_one::again::*".to_owned(), f, 3)]
    );
}

/// Every order of `items`.
fn orders(items: &[String]) -> Vec<Vec<String>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut all = Vec::new();
    for position in 0..items.len() {
        let mut rest = items.to_vec();
        let head = rest.remove(position);
        for mut tail in orders(&rest) {
            tail.insert(0, head.clone());
            all.push(tail);
        }
    }
    all
}

/// The surface of the crate made of `outer` and of `mod m` holding `inner`,
/// in every order of both, which is the same in each. Returns it and the
/// number of loads.
fn surface_in_every_order(outer: &[&str], inner: &[&str]) -> (Vec<(String, String)>, usize) {
    let mut found: Option<Vec<(String, String)>> = None;
    let mut loads = 0;
    let inner: Vec<String> = inner.iter().map(|d| (*d).to_owned()).collect();
    for inside in orders(&inner) {
        let mut blocks: Vec<String> = outer.iter().map(|d| (*d).to_owned()).collect();
        blocks.push(format!("mod m {{ {} }}", inside.join(" ")));
        for order in orders(&blocks) {
            let text = order.join("\n");
            let listed = surface(&load(&[("src/lib.rs", text.as_str())]));
            match &found {
                None => found = Some(listed),
                Some(first) => assert_eq!(&listed, first, "{text}"),
            }
            loads += 1;
        }
    }
    (found.expect("loaded"), loads)
}

#[test]
fn an_item_reached_only_by_a_pub_use_of_a_private_glob_binding_is_not_exported_in_every_order() {
    // `m` reads the struct `A` of the private module `x` through a private
    // glob, and `c` re-exports `super::A` with the function `A` that `m`
    // defines: rustc keeps the struct no more visible than the glob, so the
    // crate's `pub use` of `c` exports the function and not the struct.
    let (listed, loads) = surface_in_every_order(
        &["mod x { pub struct A {} }", "pub use m::c::*;"],
        &[
            "use crate::x::*;",
            "pub fn A() {}",
            "pub mod c { pub use super::A; }",
        ],
    );
    assert_eq!(listed, pairs(&[("demo::A", "demo::m::A")]));
    assert_eq!(loads, 36);

    // Neighbour: with a `pub` glob in `m` the struct is exported beside the
    // function.
    let (listed, loads) = surface_in_every_order(
        &["mod x { pub struct A {} }", "pub use m::c::*;"],
        &[
            "pub use crate::x::*;",
            "pub fn A() {}",
            "pub mod c { pub use super::A; }",
        ],
    );
    assert_eq!(
        listed,
        pairs(&[("demo::A", "demo::m::A"), ("demo::A", "demo::x::A")])
    );
    assert_eq!(loads, 36);
}

/// The number of loads of the crate made of `outer` and of `mod m` holding
/// `inner`, in every order of both, for each of which the surface is refused
/// as ambiguous.
fn ambiguous_surface_in_every_order(outer: &[&str], inner: &[&str]) -> usize {
    let mut loads = 0;
    let inner: Vec<String> = inner.iter().map(|d| (*d).to_owned()).collect();
    for inside in orders(&inner) {
        let mut blocks: Vec<String> = outer.iter().map(|d| (*d).to_owned()).collect();
        blocks.push(format!("mod m {{ {} }}", inside.join(" ")));
        for order in orders(&blocks) {
            let text = order.join("\n");
            let ws = load(&[("src/lib.rs", text.as_str())]);
            assert!(
                matches!(
                    ws.exported_items(target(&ws, TargetKind::Lib)),
                    Err(ExportError::Resolve(ResolveError::Ambiguous(_)))
                ),
                "{text}"
            );
            loads += 1;
        }
    }
    loads
}

#[test]
fn a_name_two_globs_bring_in_as_different_items_one_through_a_private_glob_is_refused_in_every_order()
 {
    // `m` globs `a` privately and `b` publicly, and both hold a struct `Y`.
    // The name is ambiguous in `m` for the crate root as well, so a `pub use`
    // of `m`, by glob or by name, has no one item to export, and the surface
    // is refused as ambiguous, not listed with `b::Y`. rustc refuses the
    // named re-export in every order; for the glob it depends on the order of
    // the two globs of `m` (a downstream crate finds `Y` missing, or is
    // refused), and the index answers the refusal in each.
    let inner = ["use crate::a::*;", "pub use crate::b::*;"];
    for reexport in ["pub use m::*;", "pub use m::Y;"] {
        let loads = ambiguous_surface_in_every_order(
            &[
                "mod a { pub struct Y; }",
                "mod b { pub struct Y; }",
                reexport,
            ],
            &inner,
        );
        assert_eq!(loads, 48, "{reexport}");
    }

    // Neighbour: the private glob brings in another name, and the surface
    // lists what the public glob does and nothing of the private one.
    let (listed, loads) = surface_in_every_order(
        &[
            "mod a { pub struct X; }",
            "mod b { pub struct Y; }",
            "pub use m::*;",
        ],
        &inner,
    );
    assert_eq!(listed, pairs(&[("demo::Y", "demo::b::Y")]));
    assert_eq!(loads, 48);

    // Neighbour: one item through both globs is not ambiguous.
    let (listed, loads) = surface_in_every_order(
        &[
            "mod a { pub struct Y; }",
            "mod b { pub use crate::a::Y; }",
            "pub use m::*;",
        ],
        &inner,
    );
    assert_eq!(listed, pairs(&[("demo::Y", "demo::a::Y")]));
    assert_eq!(loads, 48);
}
