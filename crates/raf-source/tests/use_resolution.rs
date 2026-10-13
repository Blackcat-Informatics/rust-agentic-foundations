// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Resolving paths through `use` declarations, globs and re-export chains.

use std::fmt::Write as _;

use raf_source::{LoadError, LoadErrorKind, MemoryTree, Namespace, ResolveError, Workspace};

const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";

fn load(files: &[(&str, &str)]) -> Result<Workspace, LoadError> {
    let mut all = vec![("Cargo.toml", DEMO)];
    all.extend_from_slice(files);
    let tree: MemoryTree = all.into_iter().collect();
    Workspace::load(&tree, &["."])
}

fn lib(text: &str) -> Workspace {
    load(&[("src/lib.rs", text)]).unwrap_or_else(|e| panic!("{e}"))
}

/// The canonical path a path resolves to.
fn path_of(ws: &Workspace, path: &str) -> String {
    ws.resolve(path)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .path
}

fn not_found(ws: &Workspace, path: &str) {
    assert!(
        matches!(ws.resolve(path), Err(ResolveError::NotFound(_))),
        "{path}: {:?}",
        ws.resolve(path)
    );
}

#[test]
fn a_rename_a_nested_group_and_a_plain_use_each_resolve_to_the_definition() {
    let ws = lib(
        "mod inner { pub fn go() {} pub fn stop() {} pub mod deep { pub fn dive() {} } }\n\
         pub use inner::go as run;\n\
         pub use inner::{stop, deep::{self, dive as plunge}};\n",
    );
    assert_eq!(path_of(&ws, "demo::run"), "demo::inner::go");
    assert_eq!(path_of(&ws, "demo::stop"), "demo::inner::stop");
    assert_eq!(path_of(&ws, "demo::plunge"), "demo::inner::deep::dive");
    assert_eq!(path_of(&ws, "demo::deep::dive"), "demo::inner::deep::dive");
    // Neighbouring negatives: the original names are not re-exported.
    not_found(&ws, "demo::go");
    not_found(&ws, "demo::dive");
}

#[test]
fn a_glob_resolves_with_via_glob_and_the_direct_path_does_not() {
    let ws = lib("pub mod inner { pub fn go() {} }\npub mod outer { pub use super::inner::*; }\n");
    let direct = ws.resolve("demo::inner::go").expect("direct");
    assert!(!direct.via_glob);
    let globbed = ws.resolve("demo::outer::go").expect("through the glob");
    assert!(globbed.via_glob);
    assert_eq!(globbed.path, "demo::inner::go");
    assert_eq!(
        globbed.chain,
        ["demo::outer::go", "demo::inner::go"].map(str::to_owned)
    );
    assert_eq!(direct.chain, ["demo::inner::go".to_owned()]);
}

#[test]
fn definitions_and_explicit_imports_shadow_globs() {
    let ws = lib("pub mod a { pub fn x() {} pub fn y() {} }\n\
         pub mod b { pub fn x() {} pub fn y() {} }\n\
         pub mod own { pub use super::a::*; pub fn x() {} }\n\
         pub mod picked { pub use super::a::*; pub use super::b::y; }\n");
    // A definition beats the glob.
    let own = ws.resolve("demo::own::x").expect("definition");
    assert_eq!(own.path, "demo::own::x");
    assert!(!own.via_glob);
    // The glob still supplies what the module does not define.
    assert_eq!(path_of(&ws, "demo::own::y"), "demo::a::y");
    // An explicit import beats the glob of the same name.
    assert_eq!(path_of(&ws, "demo::picked::y"), "demo::b::y");
    assert_eq!(path_of(&ws, "demo::picked::x"), "demo::a::x");
}

#[test]
fn explicit_imports_shadow_globs_for_the_modules_that_glob_the_importer() {
    // `mid` imports `b::y` explicitly and globs `a`, so `top` sees `b::y` only.
    let ws = lib("pub mod a { pub fn y() {} }\npub mod b { pub fn y() {} }\n\
         pub mod mid { pub use super::a::*; pub use super::b::y; }\n\
         pub mod top { pub use super::mid::*; }\n");
    assert_eq!(path_of(&ws, "demo::top::y"), "demo::b::y");
}

#[test]
fn two_globs_that_bring_in_different_items_are_ambiguous_beside_two_that_bring_in_one() {
    let ws = lib("pub mod a { pub fn x() {} }\npub mod b { pub fn x() {} }\n\
         pub mod both { pub use super::a::*; pub use super::b::*; }\n\
         pub mod one { pub use super::a::*; }\n\
         pub mod twice { pub use super::a::*; pub use super::one::*; }\n");
    match ws.resolve("demo::both::x") {
        Err(ResolveError::Ambiguous(ids)) => assert_eq!(ids.len(), 2),
        other => panic!("{other:?}"),
    }
    assert_eq!(path_of(&ws, "demo::one::x"), "demo::a::x");
    // The same item through two globs is one symbol.
    assert_eq!(path_of(&ws, "demo::twice::x"), "demo::a::x");
}

#[test]
fn a_mutual_super_glob_resolves_a_real_item_in_both_directions() {
    let ws = lib("pub mod a { pub use super::b::*; pub fn only_a() {} }\n\
         pub mod b { pub use super::a::*; pub fn only_b() {} }\n");
    assert_eq!(path_of(&ws, "demo::a::only_b"), "demo::b::only_b");
    assert_eq!(path_of(&ws, "demo::b::only_a"), "demo::a::only_a");
    assert!(ws.resolve("demo::a::only_b").expect("resolves").via_glob);
    not_found(&ws, "demo::a::missing");
}

#[test]
fn crate_self_and_super_paths_resolve_from_the_module_they_are_written_in() {
    let ws = lib("pub fn root_fn() {}\n\
         pub mod outer {\n    pub fn up() {}\n    pub mod inner {\n        pub fn leaf() {}\n\
         pub use crate::root_fn as via_crate;\n\
         pub use self::leaf as via_self;\n\
         pub use super::up as via_super;\n\
         pub use super::super::root_fn as via_super_super;\n    }\n}\n");
    for (alias, target) in [
        ("via_crate", "demo::root_fn"),
        ("via_self", "demo::outer::inner::leaf"),
        ("via_super", "demo::outer::up"),
        ("via_super_super", "demo::root_fn"),
    ] {
        assert_eq!(
            path_of(&ws, &format!("demo::outer::inner::{alias}")),
            target,
            "{alias}"
        );
    }
}

#[test]
fn super_above_the_crate_root_is_an_error_naming_file_and_line() {
    let error =
        load(&[("src/lib.rs", "pub fn ok() {}\nuse super::missing;\n")]).expect_err("above");
    assert!(
        matches!(error.kind, LoadErrorKind::SuperAboveRoot { .. }),
        "{error}"
    );
    assert_eq!(error.file.as_str(), "src/lib.rs");
    assert_eq!(error.line, Some(2));
    let error = load(&[("src/lib.rs", "mod m { use super::super::x; }\n")]).expect_err("two up");
    assert!(
        matches!(error.kind, LoadErrorKind::SuperAboveRoot { .. }),
        "{error}"
    );
    assert!(
        load(&[(
            "src/lib.rs",
            "mod m { pub fn x() {} }\nmod n { use super::m::x; }\n"
        )])
        .is_ok()
    );
}

/// Load `files` beside a manifest of `edition`.
fn load_in(edition: &str, files: &[(&str, &str)]) -> Result<Workspace, LoadError> {
    let manifest = format!("{DEMO}edition = \"{edition}\"\n");
    let mut all = vec![("Cargo.toml", manifest.as_str())];
    all.extend_from_slice(files);
    let tree: MemoryTree = all.into_iter().collect();
    Workspace::load(&tree, &["."])
}

#[test]
fn extern_crate_renames_and_leading_colon_paths_reach_the_library_from_other_targets() {
    let ws = load_in(
        "2021",
        &[
            ("src/lib.rs", "pub mod inner { pub fn go() {} }\n"),
            (
                "src/main.rs",
                "extern crate demo as d;\npub use d::inner::go;\npub use ::demo::inner::go as g;\nfn main() {}\n",
            ),
        ],
    )
    .expect("loads");
    assert_eq!(path_of(&ws, "demo::bin_crate::demo::go"), "demo::inner::go");
    assert_eq!(path_of(&ws, "demo::bin_crate::demo::g"), "demo::inner::go");
    // The extern crate item itself resolves to the item.
    let alias = ws.resolve("demo::bin_crate::demo::d").expect("alias");
    assert_eq!(
        ws.item(alias.sites[0]).expect("item").kind,
        raf_source::SymbolKind::ExternCrate
    );
    // A plain use of the library by name works from a test target.
    let ws = load_in(
        "2021",
        &[
            ("src/lib.rs", "pub mod inner { pub fn go() {} }\n"),
            ("tests/it.rs", "use demo::inner::go;\n"),
        ],
    )
    .expect("loads");
    assert_eq!(path_of(&ws, "demo::test_crate::it::go"), "demo::inner::go");
}

#[test]
fn a_three_hop_chain_resolves_and_lists_each_hop() {
    let ws = lib("mod a { pub fn real() {} }\n\
         mod b { pub use super::a::real as hop2; }\n\
         mod c { pub use super::b::hop2 as hop1; }\n\
         pub use c::hop1 as top;\n");
    let resolution = ws.resolve("demo::top").expect("resolves");
    assert_eq!(resolution.path, "demo::a::real");
    assert_eq!(
        resolution.chain,
        [
            "demo::top",
            "demo::c::hop1",
            "demo::b::hop2",
            "demo::a::real"
        ]
        .map(str::to_owned)
    );
}

#[test]
fn a_cycle_of_imports_is_cyclic_beside_a_chain_that_ends() {
    let ws = lib(
        "pub mod a { pub use super::b::x; }\npub mod b { pub use super::a::x; }\n\
         pub mod c { pub fn real() {} }\npub mod d { pub use super::c::real as x; }\n",
    );
    assert!(
        matches!(ws.resolve("demo::a::x"), Err(ResolveError::Cyclic(_))),
        "{:?}",
        ws.resolve("demo::a::x")
    );
    assert!(matches!(
        ws.resolve("demo::b::x"),
        Err(ResolveError::Cyclic(_))
    ));
    assert_eq!(path_of(&ws, "demo::d::x"), "demo::c::real");
    // A self-referencing import is also a cycle.
    let ws = lib("pub mod a { pub use self::x as y; pub use self::y as x; }\n");
    assert!(matches!(
        ws.resolve("demo::a::x"),
        Err(ResolveError::Cyclic(_))
    ));
}

#[test]
fn missing_names_dependency_items_and_macro_generated_items_are_not_found() {
    let ws = lib("pub use serde::Serialize;\n\
         macro_rules! make { () => { pub fn made() {} } }\n\
         make!();\n\
         pub fn present() {}\n");
    not_found(&ws, "demo::nope");
    not_found(&ws, "demo::Serialize");
    not_found(&ws, "demo::made");
    not_found(&ws, "serde::Serialize");
    not_found(&ws, "demo::present::deeper");
    // Beside them: the macro and the plain function resolve.
    assert_eq!(path_of(&ws, "demo::make"), "demo::make");
    assert_eq!(path_of(&ws, "demo::present"), "demo::present");
}

#[test]
fn raw_identifiers_resolve_under_both_spellings() {
    let ws = load(&[(
        "src/lib.rs",
        "pub mod r#match { pub fn r#type() {} }\npub use r#match::r#type as r#loop;\n",
    )])
    .expect("loads");
    let plain = ws.resolve("demo::match::type").expect("plain");
    let raw = ws.resolve("demo::r#match::r#type").expect("raw");
    assert_eq!(plain, raw);
    assert_eq!(path_of(&ws, "demo::loop"), "demo::match::type");
    assert_eq!(path_of(&ws, "demo::r#loop"), "demo::match::type");
}

#[test]
fn a_long_chain_of_re_exports_resolves_in_either_order_without_a_cap() {
    let hops = 5000;
    let mut forward = String::from("mod origin { pub fn end() {} }\npub use origin::end as h0;\n");
    for hop in 1..hops {
        writeln!(forward, "pub use self::h{} as h{hop};", hop - 1).expect("write");
    }
    let ws = lib(&forward);
    let last = format!("demo::h{}", hops - 1);
    let resolution = ws.resolve(&last).expect("resolves");
    assert_eq!(resolution.path, "demo::origin::end");
    assert_eq!(resolution.chain.len(), hops + 1);
    // The same chain declared last-to-first.
    let mut backward = String::from("mod origin { pub fn end() {} }\n");
    for hop in (1..hops).rev() {
        writeln!(backward, "pub use self::h{} as h{hop};", hop - 1).expect("write");
    }
    backward.push_str("pub use origin::end as h0;\n");
    let ws = lib(&backward);
    assert_eq!(path_of(&ws, &last), "demo::origin::end");
}

#[test]
fn the_same_symbol_reached_through_a_type_alias_chain_and_a_method_path_resolves() {
    let ws =
        lib("pub struct Thing;\nimpl Thing { pub fn new() {} }\npub use self::Thing as Alias;\n");
    assert_eq!(path_of(&ws, "demo::Alias"), "demo::Thing");
    assert_eq!(path_of(&ws, "demo::Alias::new"), "demo::Thing::new");
    not_found(&ws, "demo::Thing::missing");
}

#[test]
fn an_invalid_path_is_an_invalid_error_beside_a_valid_one() {
    let ws = lib("pub fn f() {}");
    for bad in ["", "demo::", "::demo::f", "demo::f()", "demo f"] {
        assert!(
            matches!(ws.resolve(bad), Err(ResolveError::Invalid(_))),
            "{bad:?}"
        );
    }
    assert!(ws.resolve("demo::f").is_ok());
    assert!(ws.resolve_in("demo::f", Namespace::Type).is_err());
}

#[test]
fn a_cfg_twin_module_is_one_symbol_and_its_contents_merge() {
    let ws = lib("#[cfg(unix)] pub mod sys { pub fn common() {} }\n\
         #[cfg(windows)] pub mod sys { pub fn common() {} pub fn extra() {} }\n");
    let module = ws.resolve("demo::sys").expect("module");
    assert_eq!(module.sites.len(), 2);
    let function = ws.resolve("demo::sys::common").expect("twin function");
    assert_eq!(function.sites.len(), 2);
    assert_eq!(
        ws.resolve("demo::sys::extra")
            .expect("one site")
            .sites
            .len(),
        1
    );
}

#[test]
fn cfg_attr_alternatives_index_every_file_under_the_one_module_path() {
    let ws = load(&[
        (
            "src/lib.rs",
            "#[cfg_attr(unix, path = \"unix.rs\")]\n#[cfg_attr(windows, path = \"win.rs\")]\nmod sys;\n",
        ),
        ("src/unix.rs", "pub fn go() {}\n"),
        ("src/win.rs", "pub fn go() {}\n"),
    ])
    .expect("loads");
    let resolution = ws.resolve("demo::sys::go").expect("twins");
    assert_eq!(resolution.sites.len(), 2);
}

/// The result of one resolution, as plain comparable data.
fn outcome(ws: &Workspace, path: &str) -> String {
    match ws.resolve(path) {
        Ok(resolution) => format!(
            "{} sites={} glob={}",
            resolution.path,
            resolution.sites.len(),
            resolution.via_glob
        ),
        Err(ResolveError::Ambiguous(ids)) => format!("ambiguous {}", ids.len()),
        Err(other) => format!("{other:?}"),
    }
}

#[test]
fn a_name_is_not_read_from_a_glob_while_an_explicit_import_of_it_is_unsettled() {
    // `m` globs `x` and explicitly imports `p::A`; `n` imports `m::A`. The
    // explicit import must win for `n` as it does for `m`, whatever the order.
    let ws = lib("mod x { pub struct A; }\nmod y { pub struct A; }\n\
         mod m { pub use crate::x::*; pub use crate::p::A; }\n\
         mod n { pub use crate::m::A; }\n\
         mod p { pub use crate::y::A; }\n");
    assert_eq!(path_of(&ws, "demo::m::A"), "demo::y::A");
    assert_eq!(path_of(&ws, "demo::n::A"), "demo::y::A");
    // The reverse declaration order.
    let ws = lib("mod p { pub use crate::y::A; }\n\
         mod n { pub use crate::m::A; }\n\
         mod m { pub use crate::p::A; pub use crate::x::*; }\n\
         mod y { pub struct A; }\nmod x { pub struct A; }\n");
    assert_eq!(path_of(&ws, "demo::n::A"), "demo::y::A");
}

fn permutations(items: &[String]) -> Vec<Vec<String>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut all = Vec::new();
    for position in 0..items.len() {
        let mut rest = items.to_vec();
        let head = rest.remove(position);
        for mut tail in permutations(&rest) {
            tail.insert(0, head.clone());
            all.push(tail);
        }
    }
    all
}

#[test]
fn resolution_does_not_depend_on_the_order_of_the_declarations() {
    let blocks: Vec<String> = [
        "mod x { pub struct A; pub fn only_x() {} }",
        "mod y { pub struct A; }",
        "mod p { pub use crate::y::A; }",
        "mod n { pub use crate::m::A; pub use crate::m::only_x; }",
        "mod both { pub use crate::x::*; pub use crate::y::*; }",
    ]
    .map(str::to_owned)
    .to_vec();
    let paths = [
        "demo::n::A",
        "demo::n::only_x",
        "demo::m::A",
        "demo::m::only_x",
        "demo::p::A",
        "demo::both::A",
        "demo::both::only_x",
        "demo::x::A",
        "demo::nope",
    ];
    let mut expected: Option<Vec<String>> = None;
    let mut loads = 0;
    // `m` has two uses, in either order, and sits at every position.
    for first_uses in [
        "mod m { pub use crate::x::*; pub use crate::p::A; }",
        "mod m { pub use crate::p::A; pub use crate::x::*; }",
    ] {
        let mut all = blocks.clone();
        all.push(first_uses.to_owned());
        for order in permutations(&all) {
            let ws = lib(&order.join("\n"));
            let results: Vec<String> = paths.iter().map(|p| outcome(&ws, p)).collect();
            match &expected {
                None => expected = Some(results),
                Some(first) => assert_eq!(&results, first, "{order:?}"),
            }
            loads += 1;
        }
    }
    assert_eq!(loads, 1440);
    // A glob and a library of the workspace that share a name: whichever is
    // declared first, the import through the name is ambiguous.
    let (resolved, _, orders) = every_order(
        "pub mod k { pub mod shared { pub fn x() {} pub trait T { fn m(&self); } } }",
        None,
        &[
            "use k::*;",
            "pub use shared::x;",
            "use shared::T;\nimpl T for String { fn m(&self) {} }",
            "pub mod later { pub use crate::x as y; }",
        ],
        "demo::later::y",
    );
    assert_eq!(orders, 24);
    assert_eq!(
        resolved,
        Err(vec!["demo::k::shared".to_owned(), "shared".to_owned()])
    );
    let expected = expected.expect("loaded");
    assert_eq!(expected[0], "demo::y::A sites=1 glob=false");
    assert_eq!(expected[5], "ambiguous 2");
    assert_eq!(expected[6], "demo::x::only_x sites=1 glob=true");
    assert_eq!(expected[8], "NotFound(\"demo::nope\")");
}

#[test]
fn an_import_that_can_never_resolve_still_shadows_nothing_it_cannot_bind() {
    // `serde::A` does not exist, so the explicit import settles as failed and
    // the glob's `A` is read afterwards.
    let ws = lib(
        "mod x { pub struct A; }\nmod m { pub use crate::x::*; pub use serde::A; }\nmod n { pub use crate::m::A; }\n",
    );
    assert_eq!(path_of(&ws, "demo::n::A"), "demo::x::A");
    assert_eq!(path_of(&ws, "demo::m::A"), "demo::x::A");
}

#[test]
fn a_route_without_a_glob_decides_via_glob_for_an_item_reached_both_ways() {
    let ws = lib("mod a { pub fn f() {} }\nmod m { pub use crate::a::*; pub use crate::a::f; }\n");
    let resolution = ws.resolve("demo::m::f").expect("resolves");
    assert!(!resolution.via_glob);
    assert_eq!(resolution.path, "demo::a::f");
    let only_glob = lib("mod a { pub fn f() {} }\nmod m { pub use crate::a::*; }\n");
    assert!(only_glob.resolve("demo::m::f").expect("resolves").via_glob);
}

#[test]
fn globs_import_only_the_names_the_importer_can_name() {
    // A private `Foo` and a public `Foo` from two globs: the public one.
    let ws = lib("mod a { struct Foo; }\nmod b { pub struct Foo; }\n\
         mod m { pub use crate::a::*; pub use crate::b::*; }\n");
    assert_eq!(path_of(&ws, "demo::m::Foo"), "demo::b::Foo");
    // A private item reaches the descendants of its module through a glob.
    let ws = lib(
        "mod a { struct Foo; pub mod child { pub use super::*; } }\nmod outside { pub use crate::a::*; }\n",
    );
    assert_eq!(path_of(&ws, "demo::a::child::Foo"), "demo::a::Foo");
    not_found(&ws, "demo::outside::Foo");
    // Restricted visibility.
    let ws = lib(
        "mod a {\n    pub(crate) struct C;\n    pub mod inner { pub(super) struct S; pub(in crate::a) struct T; }\n\
         pub mod peer { pub use super::inner::*; }\n}\n\
         mod outside { pub use crate::a::*; pub use crate::a::inner::*; }\n",
    );
    assert_eq!(path_of(&ws, "demo::outside::C"), "demo::a::C");
    assert_eq!(path_of(&ws, "demo::a::peer::S"), "demo::a::inner::S");
    assert_eq!(path_of(&ws, "demo::a::peer::T"), "demo::a::inner::T");
    not_found(&ws, "demo::outside::S");
    not_found(&ws, "demo::outside::T");
}

#[test]
fn use_with_only_self_is_an_error_and_use_path_self_imports_the_module() {
    let error = load(&[("src/lib.rs", "pub fn ok() {}\nuse {self};\n")]).expect_err("no path");
    assert!(
        matches!(error.kind, LoadErrorKind::UnsupportedItem { .. }),
        "{error}"
    );
    assert_eq!(error.line, Some(2));
    let ws = lib(
        "mod a { pub fn f() {} }\nmod b { pub use crate::a::{self}; }\nmod c { pub use crate::a::{self as renamed}; }\n",
    );
    assert_eq!(path_of(&ws, "demo::b::a::f"), "demo::a::f");
    assert_eq!(path_of(&ws, "demo::c::renamed::f"), "demo::a::f");
}

#[test]
fn trait_impls_on_foreign_types_are_keyed_by_the_canonical_trait_path_in_every_spelling() {
    let manifest =
        "[package]\nname = \"demo\"\nedition = \"2021\"\n\n[dependencies]\nserde = \"1\"\n";
    let text = "use std::fmt;\nuse std::fmt::Display;\npub struct X;\n\
         impl fmt::Display for Vec<X> { fn fmt(&self) {} }\n\
         impl Display for Option<X> { fn fmt(&self) {} }\n\
         impl std::fmt::Display for [X] { fn fmt(&self) {} }\n\
         extern crate serde as s;\n\
         impl s::Serialize for Vec<X> { fn serialize(&self) {} }\n\
         impl Clone for Vec<X> { fn clone(&self) {} }\n\
         impl ::core::fmt::Debug for Vec<X> { fn fmt(&self) {} }\n";
    let tree: MemoryTree = [("Cargo.toml", manifest), ("src/lib.rs", text)]
        .into_iter()
        .collect();
    let ws = Workspace::load(&tree, &["."]).expect("loads");
    let display = ws.resolve("std::fmt::Display::fmt").expect("one key");
    assert_eq!(display.sites.len(), 3);
    assert_eq!(display.path, "std::fmt::Display::fmt");
    assert_eq!(
        path_of(&ws, "serde::Serialize::serialize"),
        "serde::Serialize::serialize"
    );
    assert_eq!(
        path_of(&ws, "std::clone::Clone::clone"),
        "std::clone::Clone::clone"
    );
    // `core` and `std` spell one trait.
    assert_eq!(path_of(&ws, "std::fmt::Debug::fmt"), "std::fmt::Debug::fmt");
    // The spelling stays on the item.
    let first = ws.item(display.sites[0]).expect("item");
    assert!(first.trait_path.is_some());
}

#[test]
fn a_trait_path_that_cannot_be_made_absolute_is_a_typed_error() {
    for bad in [
        "impl crate::missing::Tr for Vec<u8> {}",
        "impl Unknown for Vec<u8> {}",
        "impl self::Nope for Vec<u8> {}",
    ] {
        let text = format!("pub fn ok() {{}}\n{bad}\n");
        let error = load(&[("src/lib.rs", &text)]).expect_err(bad);
        assert!(
            matches!(error.kind, LoadErrorKind::UnresolvedTrait { .. }),
            "{bad}: {error}"
        );
        assert_eq!(error.line, Some(2), "{bad}");
    }
    assert!(
        load(&[(
            "src/lib.rs",
            "pub trait Tr {}\nimpl Tr for Vec<u8> {}\nimpl Default for Vec<u8> {}\n"
        )])
        .is_ok()
    );
}

/// Two packages whose libraries are both named `shared`, a package that
/// depends on one by path and on the other through `[workspace.dependencies]`
/// under another name, and one that renames a path dependency.
const SHARED: [(&str, &str); 14] = [
    (
        "Cargo.toml",
        "[workspace]\nmembers = [\"one\", \"two\", \"three\", \"four\"]\n\n\
         [workspace.dependencies]\nalias = { path = \"two\", package = \"p-two\" }\n",
    ),
    (
        "one/Cargo.toml",
        "[package]\nname = \"p-one\"\nedition = \"2021\"\n\n[lib]\nname = \"shared\"\n",
    ),
    (
        "one/src/lib.rs",
        "pub struct X;\npub fn only_one() {}\npub trait Tr {}\n",
    ),
    ("one/tests/it.rs", "pub use shared::only_one;\n"),
    ("one/build.rs", "pub use shared::only_one;\nfn main() {}\n"),
    (
        "two/Cargo.toml",
        "[package]\nname = \"p-two\"\nedition = \"2021\"\n\n[lib]\nname = \"shared\"\n",
    ),
    ("two/src/lib.rs", "pub struct X;\npub trait Tr {}\n"),
    (
        "three/Cargo.toml",
        "[package]\nname = \"p-three\"\nedition = \"2021\"\n\n\
         [dependencies]\np-one = { path = \"../one\" }\n\n\
         [dev-dependencies]\nalias = { workspace = true }\n",
    ),
    (
        "three/src/lib.rs",
        "pub use shared::X;\npub use shared::only_one;\npub use alias::X as NotHere;\n\
         #[cfg(test)]\nmod tests {\n    pub use alias::X as Two;\n}\n",
    ),
    (
        "three/tests/it.rs",
        "pub use alias::X as Two;\npub use shared::X;\n",
    ),
    (
        "four/Cargo.toml",
        "[package]\nname = \"p-four\"\nedition = \"2021\"\n\n\
         [dependencies]\nrenamed = { path = \"../two\", package = \"p-two\" }\n",
    ),
    (
        "four/src/lib.rs",
        "pub use renamed::X;\npub use shared::X as Shared;\nextern crate renamed as again;\npub use again::X as Again;\npub mod m {\n    pub use again::X;\n}\n",
    ),
    ("four/src/main.rs", "pub use p_four::X;\nfn main() {}\n"),
    ("four/tests/it.rs", "pub use p_four::X;\n"),
];

fn shared() -> Workspace {
    let tree: MemoryTree = SHARED.into_iter().collect();
    Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"))
}

/// The file of the definition a path resolves to.
fn file_of(ws: &Workspace, resolution: &raf_source::Resolution) -> String {
    ws.item(resolution.sites[0])
        .expect("the site is an item")
        .file
        .to_string()
}

#[test]
fn two_libraries_with_one_crate_name_are_ambiguous_until_the_package_is_named() {
    let ws = shared();
    for path in ["shared::X", "shared"] {
        match ws.resolve(path) {
            Err(ResolveError::Ambiguous(ids)) => {
                let files: Vec<_> = ids
                    .iter()
                    .map(|id| ws.item(*id).expect("item").file.as_str())
                    .collect();
                assert_eq!(files, ["one/src/lib.rs", "two/src/lib.rs"], "{path}");
            }
            other => panic!("{path}: {other:?}"),
        }
    }
    let two = ws
        .resolve_in_package("p-two", "shared::X")
        .expect("in p-two");
    assert_eq!(two.path, "shared::X");
    assert_eq!(file_of(&ws, &two), "two/src/lib.rs");
    let one = ws
        .resolve_in_package("p-one", "shared::X")
        .expect("in p-one");
    assert_eq!(file_of(&ws, &one), "one/src/lib.rs");
    assert!(matches!(
        ws.resolve_in_package("p-two", "shared::only_one"),
        Err(ResolveError::NotFound(_))
    ));
    assert_eq!(
        ws.resolve_in_package("p-nine", "shared::X"),
        Err(ResolveError::UnknownPackage("p-nine".to_owned()))
    );
}

#[test]
fn a_crate_name_in_an_import_is_the_dependency_its_target_declares() {
    let ws = shared();
    // A path dependency names its package's library by the library's name.
    let x = ws
        .resolve("p_three::X")
        .expect("through the path dependency");
    assert_eq!(file_of(&ws, &x), "one/src/lib.rs");
    assert_eq!(x.chain, ["p_three::X", "shared::X"]);
    assert_eq!(
        file_of(&ws, &ws.resolve("p_three::only_one").expect("resolves")),
        "one/src/lib.rs"
    );
    // A dev-dependency renamed in `[workspace.dependencies]` is named by its
    // new name, in test code only.
    let two = ws.resolve("p_three::tests::Two").expect("test code");
    assert_eq!(file_of(&ws, &two), "two/src/lib.rs");
    not_found(&ws, "p_three::NotHere");
    let test_target = ws
        .resolve("p_three::test_crate::it::Two")
        .expect("a test target names its dev-dependencies");
    assert_eq!(file_of(&ws, &test_target), "two/src/lib.rs");
    assert_eq!(
        file_of(
            &ws,
            &ws.resolve("p_three::test_crate::it::X").expect("resolves")
        ),
        "one/src/lib.rs"
    );
    // A renamed path dependency is named by the name it is declared under,
    // and not by its library's name; so is the crate of an `extern crate`.
    let renamed = ws.resolve("p_four::X").expect("renamed");
    assert_eq!(file_of(&ws, &renamed), "two/src/lib.rs");
    let again = ws.resolve("p_four::Again").expect("extern crate");
    assert_eq!(file_of(&ws, &again), "two/src/lib.rs");
    // An `extern crate` item at the crate root adds its name to the extern
    // prelude, so every module of the crate names it.
    let nested = ws.resolve("p_four::m::X").expect("from a module");
    assert_eq!(file_of(&ws, &nested), "two/src/lib.rs");
    not_found(&ws, "p_four::Shared");
    // A test target and a binary name their own package's library; the
    // build script does not.
    let own = ws
        .resolve("p_one::test_crate::it::only_one")
        .expect("its own library");
    assert_eq!(file_of(&ws, &own), "one/src/lib.rs");
    not_found(&ws, "p_one::build_script::only_one");
    for path in ["p_four::bin_crate::p_four::X", "p_four::test_crate::it::X"] {
        assert_eq!(
            file_of(&ws, &ws.resolve(path).expect("own library")),
            "two/src/lib.rs",
            "{path}"
        );
    }
}

#[test]
fn extern_crate_self_names_the_crate_itself_from_every_module() {
    let ws = lib("extern crate self as demo;\n\
         pub mod inner { pub fn go() {} }\n\
         pub mod user { pub use demo::inner::go; }\n\
         pub mod traits { pub trait Tr {} }\n\
         mod impls { impl demo::traits::Tr for u8 {} }\n");
    assert_eq!(path_of(&ws, "demo::user::go"), "demo::inner::go");
    // Without the `extern crate` item the library cannot name itself.
    let ws = lib("pub mod inner { pub fn go() {} }\n\
         pub mod user { pub use demo::inner::go; }\n");
    not_found(&ws, "demo::user::go");
}

/// A package whose library names `shared` for one package and whose test
/// code names `shared` for another as well: cargo builds its library and
/// reports the name as ambiguous in its test code.
const SPLIT: [(&str, &str); 8] = [
    (
        "one/Cargo.toml",
        "[package]\nname = \"p-one\"\nedition = \"2021\"\n\n[lib]\nname = \"shared\"\n",
    ),
    (
        "one/src/lib.rs",
        "pub struct X;\npub fn only_one() {}\npub trait Tr {}\n",
    ),
    (
        "two/Cargo.toml",
        "[package]\nname = \"p-two\"\nedition = \"2021\"\n",
    ),
    ("two/src/lib.rs", "pub struct X;\npub trait Tr {}\n"),
    (
        "three/Cargo.toml",
        "[package]\nname = \"p-three\"\nedition = \"2021\"\n\n\
         [dependencies]\np-one = { path = \"../one\" }\n\n\
         [dev-dependencies]\nshared = { path = \"../two\", package = \"p-two\" }\n",
    ),
    (
        "three/src/lib.rs",
        "pub use shared::X as A;\npub use shared::only_one as L;\n\
         impl shared::Tr for Vec<u8> { fn lib_side() {} }\n\
         #[cfg(test)]\nimpl shared::Tr for Vec<u16> { fn test_side() {} }\n\
         extern crate shared as one;\n\
         pub mod d {\n    pub use one::X as D;\n}\n\
         #[cfg(test)]\nmod tests {\n    pub use shared::X as B;\n    pub use shared::only_one as T;\n    pub use shared as S;\n}\n\
         #[cfg(test)]\nextern crate shared as sh;\n\
         #[cfg(test)]\nmod t2 {\n    pub use sh::X as C;\n    pub use sh::only_one as E;\n}\n\
         #[cfg(test)]\npub mod t {\n    pub use shared;\n    pub use shared::*;\n}\n",
    ),
    (
        "three/tests/it.rs",
        "pub use shared::X;\npub use shared::only_one;\n",
    ),
    ("three/src/main.rs", "pub use shared::X;\nfn main() {}\n"),
];

#[test]
fn a_dev_dependency_is_named_only_by_test_code_even_under_a_normal_dependencys_name() {
    let tree: MemoryTree = SPLIT.into_iter().collect();
    let ws = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"));
    // Code that is not test code names the normal dependency only.
    for path in [
        "p_three::A",
        "p_three::L",
        "p_three::d::D",
        "p_three::bin_crate::p_three::X",
    ] {
        let found = ws.resolve(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(file_of(&ws, &found), "one/src/lib.rs", "{path}");
    }
    // Test code names both, as rustc does, so a path through the name, or
    // through an `extern crate` of it, is ambiguous whatever follows it, even
    // where one library alone defines the rest; it names both crate roots.
    for path in [
        "p_three::tests::B",
        "p_three::tests::T",
        "p_three::tests::S",
        "p_three::tests::S::only_one",
        "p_three::t2::C",
        "p_three::t2::E",
        "p_three::test_crate::it::X",
        "p_three::test_crate::it::only_one",
    ] {
        match ws.resolve(path) {
            Err(ResolveError::Ambiguous(ids)) => {
                let mut files: Vec<_> = ids
                    .iter()
                    .map(|id| {
                        let item = ws.item(*id).expect("item");
                        assert_eq!(item.kind, raf_source::SymbolKind::Mod, "{path}");
                        item.file.as_str()
                    })
                    .collect();
                files.sort_unstable();
                assert_eq!(files, ["one/src/lib.rs", "two/src/lib.rs"], "{path}");
            }
            other => panic!("{path}: {other:?}"),
        }
    }
    // An import that goes on into the crates names something in each name
    // space, so it is ambiguous in the value name space too.
    for path in ["p_three::tests::T", "p_three::tests::B"] {
        assert!(
            matches!(
                ws.resolve_in(path, Namespace::Value),
                Err(ResolveError::Ambiguous(_))
            ),
            "{path}: {:?}",
            ws.resolve_in(path, Namespace::Value)
        );
    }
    // A glob of the name brings in nothing, even through an import of it, so
    // a name only that glob could bring is not found; the import itself is
    // ambiguous.
    assert_eq!(
        ws.resolve("p_three::t::only_one"),
        Err(ResolveError::NotFound("p_three::t::only_one".to_owned()))
    );
    assert!(matches!(
        ws.resolve("p_three::t::shared::only_one"),
        Err(ResolveError::Ambiguous(_))
    ));
    // A trait through the name is keyed under the one library in library
    // code, and in test code under the path in each library, unresolved.
    let key = |name: &str| {
        let keys: Vec<_> = ws
            .items()
            .filter(|item| item.name == name)
            .map(|item| item.trait_key.as_deref().cloned())
            .collect();
        assert_eq!(keys.len(), 1, "{name}");
        keys[0].clone().expect("a trait key")
    };
    assert_eq!(
        key("lib_side"),
        raf_source::TraitKey::Canonical("shared::Tr".to_owned())
    );
    assert_eq!(
        key("test_side"),
        raf_source::TraitKey::Unresolved {
            written: "shared::Tr".to_owned(),
            candidates: vec!["p_two::Tr".to_owned(), "shared::Tr".to_owned()],
        }
    );
}

#[test]
fn a_name_bound_to_several_modules_gives_a_trait_key_through_each() {
    let manifest = "[package]\nname = \"demo\"\nedition = \"2021\"\n\n\
                    [dependencies]\nserde = \"1\"\nother = \"1\"\n";
    // `m` is brought in by two globs; rustc reports it as ambiguous, and the
    // trait key lists the path through each module, never the first.
    let tree: MemoryTree = [
        ("Cargo.toml", manifest),
        (
            "src/lib.rs",
            "pub mod a { pub mod m { pub use serde::Tr; } }\n\
             pub mod b { pub mod m { pub use other::Tr; } }\n\
             pub mod user {\n\
                 use crate::a::*;\n    use crate::b::*;\n\
                 impl m::Tr for u8 { fn f() {} }\n\
             }\n\
             pub mod unseen {\n\
                 use crate::a::*;\n    use crate::b::*;\n    use m::Gen::*;\n\
                 impl Hidden for u8 { fn g() {} }\n\
             }\n",
        ),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"));
    let key = |path: &str| {
        let keys: Vec<_> = ws
            .items()
            .filter(|item| item.path == path)
            .map(|item| item.trait_key.as_deref().cloned())
            .collect();
        assert_eq!(keys.len(), 1, "{path}");
        keys[0].clone().expect("a trait key")
    };
    assert_eq!(
        key("m::Tr::f"),
        raf_source::TraitKey::Unresolved {
            written: "m::Tr".to_owned(),
            candidates: vec!["other::Tr".to_owned(), "serde::Tr".to_owned()],
        }
    );
    assert_eq!(
        key("Hidden::g"),
        raf_source::TraitKey::Unresolved {
            written: "Hidden".to_owned(),
            candidates: vec![
                "demo::a::m::Gen::Hidden".to_owned(),
                "demo::b::m::Gen::Hidden".to_owned()
            ],
        }
    );
}

/// Load `text` as the library of a package of `edition` that depends on
/// `serde`, a crate outside the workspace.
fn with_serde(edition: &str, text: &str) -> Result<Workspace, LoadError> {
    let manifest = format!("{DEMO}edition = \"{edition}\"\n\n[dependencies]\nserde = \"1\"\n");
    let tree: MemoryTree = [("Cargo.toml", manifest.as_str()), ("src/lib.rs", text)]
        .into_iter()
        .collect();
    Workspace::load(&tree, &["."])
}

#[test]
fn a_single_segment_import_of_a_dependency_is_an_external_alias_and_never_its_own_cycle() {
    // The imports, the alias a module imports `Item` through, and the paths
    // through them.
    let cases: [(&str, &str, &[&str]); 5] = [
        (
            "pub use serde;",
            "serde",
            &["demo::serde", "demo::serde::Serialize"],
        ),
        (
            "pub use std;",
            "std",
            &["demo::std", "demo::std::fmt::Display"],
        ),
        (
            "pub use serde::{self, Serialize};",
            "serde",
            &["demo::serde", "demo::serde::Serialize", "demo::Serialize"],
        ),
        (
            "pub use std::{self as s2};",
            "s2",
            &["demo::s2", "demo::s2::fmt"],
        ),
        (
            "pub use std;\npub mod m { pub use std; }\npub use std::{self as s2};",
            "s2",
            &[
                "demo::std::fmt",
                "demo::m::std",
                "demo::m::std::fmt",
                "demo::s2::fmt",
            ],
        ),
    ];
    for edition in ["2018", "2021", "2024"] {
        for (imports, alias, paths) in cases {
            let text = format!(
                "{imports}\npub mod inner {{ pub fn go() {{}} }}\npub use inner::go;\n\
                 pub mod through {{ pub use crate::{alias}::Item; }}\n",
            );
            let ws = with_serde(edition, &text).unwrap_or_else(|e| panic!("{imports}: {e}"));
            for path in paths.iter().copied().chain(["demo::through::Item"]) {
                assert_eq!(
                    ws.resolve(path),
                    Err(ResolveError::NotFound(path.to_owned())),
                    "{edition}: {imports}: {path}"
                );
            }
            // Beside them, a local re-export resolves.
            assert_eq!(path_of(&ws, "demo::go"), "demo::inner::go", "{imports}");
        }
    }
}

#[test]
fn a_single_segment_import_of_a_workspace_library_binds_its_crate_root() {
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"one\", \"two\"]\n",
        ),
        (
            "one/Cargo.toml",
            "[package]\nname = \"p-one\"\nedition = \"2021\"\n",
        ),
        ("one/src/lib.rs", "pub struct X;\npub mod inner { pub fn go() {} }\n"),
        (
            "two/Cargo.toml",
            "[package]\nname = \"p-two\"\nedition = \"2021\"\n\n[dependencies]\np-one = { path = \"../one\" }\n",
        ),
        (
            "two/src/lib.rs",
            "pub use p_one;\npub use p_one::{self as renamed};\npub mod m { pub use crate::p_one::inner::go; }\n",
        ),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(path_of(&ws, "p_two::p_one::X"), "p_one::X");
    assert_eq!(
        path_of(&ws, "p_two::renamed::inner::go"),
        "p_one::inner::go"
    );
    assert_eq!(path_of(&ws, "p_two::m::go"), "p_one::inner::go");
    // The name alone is the library's crate root, a module.
    let root = ws.resolve("p_two::p_one").expect("the crate root");
    assert_eq!(root.path, "p_one");
    assert_eq!(
        ws.item(root.sites[0]).expect("item").file.as_str(),
        "one/src/lib.rs"
    );
    // A name the library does not define is not found through it.
    not_found(&ws, "p_two::p_one::Missing");
}

#[test]
fn a_use_path_of_edition_2015_starts_at_the_crate_root() {
    let text = "pub mod inner { pub fn go() {} }\n\
                pub mod user {\n    pub use inner::go;\n    pub use ::inner::go as colon;\n    pub use inner;\n}\n";
    let old = load_in("2015", &[("src/lib.rs", text)]).expect("2015");
    for path in [
        "demo::user::go",
        "demo::user::colon",
        "demo::user::inner::go",
    ] {
        assert_eq!(path_of(&old, path), "demo::inner::go", "2015: {path}");
    }
    // From edition 2018 on, a `use` path starts in its module or at a crate.
    let new = load_in("2018", &[("src/lib.rs", text)]).expect("2018");
    for path in ["demo::user::go", "demo::user::colon", "demo::user::inner"] {
        not_found(&new, path);
    }
    // An `extern crate` item at the root is what a 2015 `use` path names a
    // dependency by: `use serde;` in a module is that item, and a path
    // through it goes outside the workspace.
    let ws = with_serde(
        "2015",
        "extern crate serde;\npub mod de {\n    pub use serde;\n    pub use serde::Serialize;\n}\n",
    )
    .expect("2015 with serde");
    let item = ws
        .resolve("demo::de::serde")
        .expect("the extern crate item");
    assert_eq!(item.path, "demo::serde");
    assert_eq!(
        ws.item(item.sites[0]).expect("item").kind,
        raf_source::SymbolKind::ExternCrate
    );
    for path in ["demo::de::serde::Serialize", "demo::de::Serialize"] {
        not_found(&ws, path);
    }
    // A test target of edition 2015 names its library through an `extern
    // crate` item, and not by the name alone.
    let ws = load_in(
        "2015",
        &[
            ("src/lib.rs", "pub mod inner { pub fn go() {} }\n"),
            ("tests/bare.rs", "pub use demo::inner::go;\n"),
            (
                "tests/item.rs",
                "extern crate demo;\npub use demo::inner::go;\n",
            ),
        ],
    )
    .expect("2015 test targets");
    not_found(&ws, "demo::test_crate::bare::go");
    assert_eq!(
        path_of(&ws, "demo::test_crate::item::go"),
        "demo::inner::go"
    );
}

#[test]
fn a_use_head_from_a_glob_that_is_also_an_extern_prelude_crate_is_ambiguous() {
    let text = "pub mod local { pub mod serde { pub struct Serialize; } pub fn serde_fn() {} }\n\
                pub mod values { #[allow(non_snake_case)] pub fn serde() {} }\n\
                pub mod globbed { pub use crate::local::*; pub use serde::Serialize as S; }\n\
                pub mod alone { pub use crate::local::*; pub use serde; }\n\
                pub mod via { pub use crate::globbed::S as T; }\n\
                pub mod defined { pub mod serde { pub struct Serialize; } pub use serde::Serialize as S; }\n\
                pub mod valued { pub use crate::values::*; pub use serde::Serialize as S; }\n";
    let ws = with_serde("2021", text).expect("loads");
    let local = path_of(&ws, "demo::local::serde");
    for path in ["demo::globbed::S", "demo::via::T", "demo::alone::serde"] {
        match ws.resolve(path) {
            Err(ResolveError::Ambiguous(ids)) => {
                let paths: Vec<_> = ids
                    .iter()
                    .map(|id| ws.item(*id).expect("item").path.to_string())
                    .collect();
                assert_eq!(paths.as_slice(), std::slice::from_ref(&local), "{path}");
            }
            other => panic!("{path}: {other:?}"),
        }
    }
    // A module's own item shadows the crate, as rustc has it.
    assert_eq!(
        path_of(&ws, "demo::defined::S"),
        "demo::defined::serde::Serialize"
    );
    // A glob that brings in only a value does not collide with a crate.
    not_found(&ws, "demo::valued::S");
    // In edition 2015 the path starts at the crate root, which binds no
    // `serde`.
    let ws = with_serde("2015", text).expect("loads in 2015");
    not_found(&ws, "demo::globbed::S");
    not_found(&ws, "demo::via::T");
}

/// A workspace where `demo` (edition 2021) depends on `p-one`, whose library
/// is `shared`, with `text` as `demo`'s library.
fn beside_shared(text: &str) -> Workspace {
    let tree: MemoryTree = [
        ("Cargo.toml", "[workspace]\nmembers = [\"one\", \"demo\"]\n"),
        (
            "one/Cargo.toml",
            "[package]\nname = \"p-one\"\nedition = \"2021\"\n\n[lib]\nname = \"shared\"\n",
        ),
        (
            "one/src/lib.rs",
            "pub fn x() {}\npub trait T { fn m(&self); }\n",
        ),
        (
            "demo/Cargo.toml",
            "[package]\nname = \"demo\"\nedition = \"2021\"\n\n[dependencies]\np-one = { path = \"../one\" }\n",
        ),
        ("demo/src/lib.rs", text),
    ]
    .into_iter()
    .collect();
    Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{text}: {e}"))
}

/// The paths a resolution names: its definition, or the ids of an
/// ambiguity.
fn named(ws: &Workspace, path: &str) -> Result<String, Vec<String>> {
    match ws.resolve(path) {
        Ok(resolution) => Ok(resolution.path),
        Err(ResolveError::Ambiguous(ids)) => Err(ids
            .iter()
            .map(|id| ws.item(*id).expect("item").path.to_string())
            .collect()),
        Err(other) => Err(vec![format!("{other:?}")]),
    }
}

/// The key of the trait impl on `String` in `ws`.
fn string_impl_key(ws: &Workspace) -> raf_source::TraitKey {
    let keys: Vec<raf_source::TraitKey> = ws
        .items()
        .filter(|item| item.self_type.as_deref() == Some("String"))
        .filter_map(|item| item.trait_key.as_deref().cloned())
        .collect();
    match keys.as_slice() {
        [key] => key.clone(),
        other => panic!("{other:?}"),
    }
}

/// For every order of `blocks` in a module: what `path` resolves to and the
/// key of the trait impl on `String`, which must be one answer.
fn every_order(
    prefix: &str,
    module: Option<&str>,
    blocks: &[&str],
    path: &str,
) -> (Result<String, Vec<String>>, raf_source::TraitKey, usize) {
    every_order_in(beside_shared, prefix, module, blocks, path)
}

/// [`every_order`] over the workspace `load` builds around `demo`'s library.
fn every_order_in(
    load: fn(&str) -> Workspace,
    prefix: &str,
    module: Option<&str>,
    blocks: &[&str],
    path: &str,
) -> (Result<String, Vec<String>>, raf_source::TraitKey, usize) {
    let mut answer = None;
    let mut orders = 0;
    for order in permutations(&blocks.iter().map(|b| (*b).to_owned()).collect::<Vec<_>>()) {
        let body = order.join("\n");
        let text = match module {
            Some(name) => format!("{prefix}\npub mod {name} {{\n{body}\n}}\n"),
            None => format!("{prefix}\n{body}\n"),
        };
        let ws = load(&text);
        let found = (named(&ws, path), string_impl_key(&ws));
        match &answer {
            None => answer = Some(found),
            Some(first) => assert_eq!(&found, first, "{text}"),
        }
        orders += 1;
    }
    let (resolved, key) = answer.expect("an order");
    (resolved, key, orders)
}

#[test]
fn a_glob_against_a_workspace_library_is_ambiguous_in_every_declaration_order() {
    // The crate name of a library of the workspace, which a glob also brings
    // in: rustc's E0659 whichever comes first.
    let (resolved, key, orders) = every_order(
        "pub mod k { pub mod shared { pub fn x() {} pub trait T { fn m(&self); } } }",
        None,
        &[
            "use k::*;",
            "pub use shared::x;",
            "use shared::T;\nimpl T for String { fn m(&self) {} }",
        ],
        "demo::x",
    );
    assert_eq!(orders, 6);
    assert_eq!(
        resolved,
        Err(vec!["demo::k::shared".to_owned(), "shared".to_owned()])
    );
    assert_eq!(
        key,
        raf_source::TraitKey::Unresolved {
            written: "T".to_owned(),
            candidates: vec!["demo::k::shared::T".to_owned(), "shared::T".to_owned()],
        }
    );
    // The same through an `extern crate` item of the library at the crate
    // root, named from a module.
    let (resolved, key, orders) = every_order(
        "extern crate shared as lib;\n\
         pub mod k { pub mod lib { pub fn x() {} pub trait T { fn m(&self); } } }",
        Some("user"),
        &[
            "use crate::k::*;",
            "pub use lib::x;",
            "use lib::T;\nimpl T for String { fn m(&self) {} }",
        ],
        "demo::user::x",
    );
    assert_eq!(orders, 6);
    assert_eq!(
        resolved,
        Err(vec!["demo::k::lib".to_owned(), "shared".to_owned()])
    );
    assert_eq!(
        key,
        raf_source::TraitKey::Unresolved {
            written: "T".to_owned(),
            candidates: vec!["demo::k::lib::T".to_owned(), "shared::T".to_owned()],
        }
    );
    // Beside them, without the glob the crate is read in every order, and
    // with the module's own item the item shadows the crate.
    let (resolved, key, _) = every_order(
        "",
        None,
        &[
            "pub use shared::x;",
            "use shared::T;\nimpl T for String { fn m(&self) {} }",
        ],
        "demo::x",
    );
    assert_eq!(resolved, Ok("shared::x".to_owned()));
    assert_eq!(key, raf_source::TraitKey::Canonical("shared::T".to_owned()));
    let (resolved, key, _) = every_order(
        "pub mod shared { pub fn x() {} pub trait T { fn m(&self); } }",
        None,
        &[
            "pub use shared::x;",
            "use shared::T;\nimpl T for String { fn m(&self) {} }",
        ],
        "demo::x",
    );
    assert_eq!(resolved, Ok("demo::shared::x".to_owned()));
    assert_eq!(
        key,
        raf_source::TraitKey::Canonical("demo::shared::T".to_owned())
    );
}

#[test]
fn a_glob_that_brings_in_an_import_of_another_crate_collides_with_the_crate() {
    let ws = with_serde(
        "2021",
        "pub mod m { pub use std as serde; }\n\
         pub mod user { use crate::m::*; pub use serde::Serialize; }\n\
         pub mod same { pub use serde; }\n\
         pub mod fine { use crate::same::*; pub use serde::Serialize; }\n\
         pub mod parent {\n    pub use serde::serde;\n    \
         pub mod tests { use super::*; pub use serde::Serialize; }\n}\n",
    )
    .expect("loads");
    // `std as serde` and the crate `serde` are two things: rustc's E0659.
    assert_eq!(
        named(&ws, "demo::user::Serialize"),
        Err(vec!["demo::m".to_owned()])
    );
    // A glob that brings in the crate itself does not collide with it, and
    // neither does one that brings in an item of the crate under its name,
    // whose name space the index cannot tell (as `use quote::quote;` brings
    // in a macro).
    not_found(&ws, "demo::fine::Serialize");
    not_found(&ws, "demo::parent::tests::Serialize");
}

#[test]
fn a_no_std_crate_does_not_name_std_without_an_extern_crate_item() {
    let body = "pub mod k { pub mod std { pub mod fmt { pub fn f() {} } } }\n\
                pub mod user { use crate::k::*; pub use std::fmt; }\n";
    // Under `#![no_std]` the glob's `std` is the only one.
    let ws = with_serde("2021", &format!("#![no_std]\n{body}")).expect("no_std");
    assert_eq!(
        named(&ws, "demo::user::fmt"),
        Ok("demo::k::std::fmt".to_owned())
    );
    // With `std` in the extern prelude, unconditionally or as a `cfg_attr`
    // leaves it under some configuration, the glob collides with it.
    for prefix in [
        "",
        "#![cfg_attr(not(feature = \"std\"), no_std)]\n",
        "#![no_std]\nextern crate std;\n",
    ] {
        let ws = with_serde("2021", &format!("{prefix}{body}")).expect("std");
        assert_eq!(
            named(&ws, "demo::user::fmt"),
            Err(vec!["demo::k::std".to_owned()]),
            "{prefix}"
        );
    }
}

/// A workspace where `demo` (edition 2021) depends on `p-one`, whose library
/// `shared` has a module `inner::other`, and on `p-two`, whose library is
/// `other`, with `text` as `demo`'s library.
fn beside_two(text: &str) -> Workspace {
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"one\", \"two\", \"demo\"]\n",
        ),
        (
            "one/Cargo.toml",
            "[package]\nname = \"p-one\"\nedition = \"2021\"\n\n[lib]\nname = \"shared\"\n",
        ),
        (
            "one/src/lib.rs",
            "pub mod inner { pub mod other { pub fn x() {} pub trait T { fn m(&self); } } }\n",
        ),
        (
            "two/Cargo.toml",
            "[package]\nname = \"p-two\"\nedition = \"2021\"\n\n[lib]\nname = \"other\"\n",
        ),
        (
            "two/src/lib.rs",
            "pub fn x() {}\npub trait T { fn m(&self); }\n\
             pub mod inner { pub mod other { pub fn x() {} } }\n",
        ),
        (
            "demo/Cargo.toml",
            "[package]\nname = \"demo\"\nedition = \"2021\"\n\n[dependencies]\n\
             p-one = { path = \"../one\" }\np-two = { path = \"../two\" }\n",
        ),
        ("demo/src/lib.rs", text),
    ]
    .into_iter()
    .collect();
    Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{text}: {e}"))
}

#[test]
fn a_glob_whose_own_path_starts_at_a_crate_is_ambiguous_with_a_crate_in_every_order() {
    // The glob's path goes through an import of a crate, and the glob brings
    // in `other`, which is also a library of the workspace: whatever the
    // order, every import through `other` is ambiguous, as rustc's E0659.
    let (resolved, key, orders) = every_order_in(
        beside_two,
        "",
        None,
        &[
            "use shared::inner as inn;",
            "use inn::*;",
            "pub use other::x;",
            "use other::T;\nimpl T for String { fn m(&self) {} }",
        ],
        "demo::x",
    );
    assert_eq!(orders, 24);
    assert_eq!(
        resolved,
        Err(vec!["other".to_owned(), "shared::inner::other".to_owned()])
    );
    assert_eq!(
        key,
        raf_source::TraitKey::Unresolved {
            written: "T".to_owned(),
            candidates: vec!["other::T".to_owned(), "shared::inner::other::T".to_owned()],
        }
    );
    // Without the glob, the crate is read in every order.
    let (resolved, key, _) = every_order_in(
        beside_two,
        "",
        None,
        &[
            "use shared::inner as inn;",
            "pub use other::x;",
            "use other::T;\nimpl T for String { fn m(&self) {} }",
        ],
        "demo::x",
    );
    assert_eq!(resolved, Ok("other::x".to_owned()));
    assert_eq!(key, raf_source::TraitKey::Canonical("other::T".to_owned()));
}

#[test]
fn an_explicit_import_shadows_a_workspace_library_in_every_declaration_order() {
    // `other` is a library of the workspace and also the name an explicit
    // import binds: the import shadows the crate whatever the order, as
    // rustc has it, in a module without globs and in one with a glob.
    for prefix in ["", "pub mod g { pub fn unrelated() {} }\nuse g::*;"] {
        let (resolved, key, orders) = every_order_in(
            beside_two,
            prefix,
            None,
            &[
                "pub use other::x;",
                "pub use shared::inner::other;",
                "use other::T;\nimpl T for String { fn m(&self) {} }",
                "pub mod later { pub use crate::x as y; }",
            ],
            "demo::x",
        );
        assert_eq!(orders, 24);
        assert_eq!(
            resolved,
            Ok("shared::inner::other::x".to_owned()),
            "{prefix}"
        );
        assert_eq!(
            key,
            raf_source::TraitKey::Canonical("shared::inner::other::T".to_owned()),
            "{prefix}"
        );
    }
}

#[test]
fn a_crate_read_before_the_explicit_import_of_its_name_binds_is_read_again_in_every_order() {
    // `pub use k::other;` binds `other` only once the glob of `k` brings it
    // in, and that glob goes through `inn`, whose own path starts at
    // `other`: when the import first fails, `inn` may read `other` as the
    // crate, after which the import would bind the name. rustc refuses the
    // import (E0432); the index answers the same whatever the order of the
    // declarations, and never through both readings of `other`.
    let blocks = [
        "pub use k::other;".to_owned(),
        "pub use other::inner as inn;".to_owned(),
        "pub mod k { pub use crate::inn::*; }".to_owned(),
    ];
    let mut answer = None;
    for order in permutations(&blocks) {
        let ws = beside_two(&order.join("\n"));
        let found = (named(&ws, "demo::other"), named(&ws, "demo::inn"));
        match &answer {
            None => answer = Some(found),
            Some(first) => assert_eq!(&found, first, "{order:?}"),
        }
    }
    let (other, _) = answer.expect("an order");
    assert_eq!(other, Err(vec!["NotFound(\"demo::other\")".to_owned()]));
}
