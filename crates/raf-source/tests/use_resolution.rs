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

#[test]
fn extern_crate_renames_and_leading_colon_paths_reach_the_library_from_other_targets() {
    let ws = load(&[
        ("src/lib.rs", "pub mod inner { pub fn go() {} }\n"),
        (
            "src/main.rs",
            "extern crate demo as d;\npub use d::inner::go;\npub use ::demo::inner::go as g;\nfn main() {}\n",
        ),
    ])
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
    let ws = load(&[
        ("src/lib.rs", "pub mod inner { pub fn go() {} }\n"),
        ("tests/it.rs", "use demo::inner::go;\n"),
    ])
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
