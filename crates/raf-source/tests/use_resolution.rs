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
fn a_cycle_of_explicit_imports_is_cyclic_whatever_its_globs_hold_in_every_order() {
    // The explicit imports of `X` form a cycle, and explicit imports shadow
    // globs: rustc cannot determine either import, and neither glob answers.
    let blocks: Vec<String> = [
        "mod a { pub use crate::b::X; pub use crate::c::*; }",
        "mod b { pub use crate::a::X; pub use crate::d::*; }",
        "mod c { pub struct X; }",
        "mod d { pub struct X; }",
    ]
    .map(str::to_owned)
    .to_vec();
    let mut orders = 0;
    for order in permutations(&blocks) {
        let ws = lib(&order.join("\n"));
        for path in ["demo::a::X", "demo::b::X"] {
            assert_eq!(
                ws.resolve(path),
                Err(ResolveError::Cyclic(path.to_owned())),
                "{order:?}"
            );
        }
        orders += 1;
    }
    assert_eq!(orders, 24);
    // Without the import that takes `a::X` from `b` there is no cycle: the
    // glob of `a` answers, and `b` imports that.
    let ws = lib(
        "mod a { pub use crate::c::*; }\nmod b { pub use crate::a::X; pub use crate::d::*; }\n\
         mod c { pub struct X; }\nmod d { pub struct X; }\n",
    );
    assert_eq!(path_of(&ws, "demo::a::X"), "demo::c::X");
    assert_eq!(path_of(&ws, "demo::b::X"), "demo::c::X");
    // An import that waits on the cycle never settles either: its own glob
    // does not answer for its name, which resolves `Cyclic`, and so does the
    // name in a module that only globs it, which the glob may still bring in.
    let ws = lib("mod a { pub use crate::b::X; pub use crate::c::*; }\n\
         mod b { pub use crate::a::X; pub use crate::d::*; }\n\
         mod c { pub struct X; }\nmod d { pub struct X; }\n\
         mod e { pub use crate::a::X; pub use crate::c::*; }\nmod f { pub use crate::e::*; }\n");
    assert_eq!(
        ws.resolve("demo::e::X"),
        Err(ResolveError::Cyclic("demo::e::X".to_owned()))
    );
    assert_eq!(
        ws.resolve("demo::f::X"),
        Err(ResolveError::Cyclic("demo::f::X".to_owned()))
    );
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

/// Load every order of `blocks` with each of `variants` among them, at
/// every position, and assert that `paths` come out the same in each.
/// Returns that outcome and the number of loads.
fn one_outcome_in_every_order(
    blocks: &[&str],
    variants: &[&str],
    paths: &[&str],
) -> (Vec<String>, usize) {
    let mut expected: Option<Vec<String>> = None;
    let mut loads = 0;
    for variant in variants {
        let mut all: Vec<String> = blocks.iter().map(|block| (*block).to_owned()).collect();
        all.push((*variant).to_owned());
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
    (expected.expect("loaded"), loads)
}

#[test]
fn resolution_does_not_depend_on_the_order_of_the_declarations() {
    // `m` has two uses, in either order, and sits at every position.
    let (expected, loads) = one_outcome_in_every_order(
        &[
            "mod x { pub struct A; pub fn only_x() {} }",
            "mod y { pub struct A; }",
            "mod p { pub use crate::y::A; }",
            "mod n { pub use crate::m::A; pub use crate::m::only_x; }",
            "mod both { pub use crate::x::*; pub use crate::y::*; }",
        ],
        &[
            "mod m { pub use crate::x::*; pub use crate::p::A; }",
            "mod m { pub use crate::p::A; pub use crate::x::*; }",
        ],
        &[
            "demo::n::A",
            "demo::n::only_x",
            "demo::m::A",
            "demo::m::only_x",
            "demo::p::A",
            "demo::both::A",
            "demo::both::only_x",
            "demo::x::A",
            "demo::nope",
        ],
    );
    assert_eq!(loads, 1440);
    assert_eq!(expected[0], "demo::y::A sites=1 glob=false");
    assert_eq!(expected[5], "ambiguous 2");
    assert_eq!(expected[6], "demo::x::only_x sites=1 glob=true");
    assert_eq!(expected[8], "NotFound(\"demo::nope\")");
    // A cycle of explicit imports, an import that waits on it, a chain that
    // ends and a failing import beside globs: the imports with no way out are
    // decided as a cycle whatever order they are met in. `p` has its two uses
    // in either order, and sits at every position.
    let (cycle, loads) = one_outcome_in_every_order(
        &[
            "mod x { pub struct A; pub fn only_x() {} }",
            "mod y { pub struct A; }",
            "mod q { pub use crate::p::A; pub use crate::y::*; }",
            "mod n { pub use crate::q::A; pub use crate::x::only_x; pub use crate::y::*; }",
            "mod m { pub use crate::x::*; pub use crate::nothing::B; pub use crate::x::A as C; }",
        ],
        &[
            "mod p { pub use crate::q::A; pub use crate::x::*; }",
            "mod p { pub use crate::x::*; pub use crate::q::A; }",
        ],
        &[
            "demo::p::A",
            "demo::q::A",
            "demo::n::A",
            "demo::n::only_x",
            "demo::m::A",
            "demo::m::B",
            "demo::m::C",
        ],
    );
    assert_eq!(loads, 1440);
    assert_eq!(
        cycle,
        [
            "Cyclic(\"demo::p::A\")",
            "Cyclic(\"demo::q::A\")",
            "Cyclic(\"demo::n::A\")",
            "demo::x::only_x sites=1 glob=false",
            "demo::x::A sites=1 glob=true",
            "NotFound(\"demo::m::B\")",
            "demo::x::A sites=1 glob=false",
        ]
        .map(str::to_owned)
    );
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
}

#[test]
fn an_explicit_import_shadows_the_glob_whether_or_not_the_index_sees_what_it_binds() {
    // `serde::A` is an item of a dependency, which the index cannot see. The
    // explicit import still shadows the glob's `A`, as in rustc, so neither
    // `m::A` nor `n::A` reads the glob.
    let ws = load_with(
        "2021",
        "[dependencies]\nserde = \"1\"\n",
        "mod x { pub struct A; }\nmod m { pub use crate::x::*; pub use serde::A; }\n\
         mod n { pub use crate::m::A; }\n",
    );
    not_found(&ws, "demo::m::A");
    not_found(&ws, "demo::n::A");
    // An import that fails shadows the glob too: rustc refuses it rather than
    // read the glob.
    let ws = lib(
        "mod x { pub struct A; }\nmod m { pub use crate::x::*; pub use crate::nothing::A; }\n\
         mod n { pub use crate::m::A; }\n",
    );
    not_found(&ws, "demo::m::A");
    not_found(&ws, "demo::n::A");
    // Without the explicit import the glob answers.
    let ws = lib(
        "mod x { pub struct A; }\nmod m { pub use crate::x::*; }\nmod n { pub use crate::m::A; }\n",
    );
    assert_eq!(path_of(&ws, "demo::m::A"), "demo::x::A");
    assert_eq!(path_of(&ws, "demo::n::A"), "demo::x::A");
    // An import binds its name only in the name spaces where its path
    // reaches something, and leaves the others to the globs, as in rustc: the
    // struct `y::A` in the type name space, the glob's function `x::A` in the
    // value name space.
    let ws = lib(
        "mod x { pub fn A() {} }\nmod y { pub struct A {} }\nmod m { pub use crate::x::*; pub use crate::y::A; }\n",
    );
    let ty = ws.resolve_in("demo::m::A", Namespace::Type).expect("type");
    assert_eq!((ty.path.as_str(), ty.via_glob), ("demo::y::A", false));
    let value = ws
        .resolve_in("demo::m::A", Namespace::Value)
        .expect("value");
    assert_eq!((value.path.as_str(), value.via_glob), ("demo::x::A", true));
    // A module that globs `m` sees both.
    let ws = lib(
        "mod x { pub fn A() {} }\nmod y { pub struct A {} }\nmod m { pub use crate::x::*; pub use crate::y::A; }\n\
         mod n { pub use crate::m::*; }\n",
    );
    assert_eq!(path_of(&ws, "demo::n::A"), "demo::y::A");
    let value = ws
        .resolve_in("demo::n::A", Namespace::Value)
        .expect("value");
    assert_eq!(value.path, "demo::x::A");
    // Beside a private glob of `m`, which `n` cannot name, `n` sees only the
    // import.
    let ws = lib(
        "mod x { pub fn A() {} }\nmod y { pub struct A {} }\nmod m { use crate::x::*; pub use crate::y::A; }\n\
         mod n { pub use crate::m::*; }\n",
    );
    assert_eq!(path_of(&ws, "demo::n::A"), "demo::y::A");
    not_found_in(&ws, "demo::n::A", Namespace::Value);
}

#[test]
fn an_import_that_binds_its_name_in_one_name_space_leaves_the_others_to_the_globs() {
    // `use super::m::m;` imports the function `m::m`, and `use super::*;`
    // brings in the module `m`: the import binds the value name space and
    // the glob answers in the type name space.
    let ws =
        lib("pub mod m { pub fn m() {} }\npub mod observed { use super::m::m; use super::*; }\n");
    let module = ws
        .resolve_in("demo::observed::m", Namespace::Type)
        .expect("type");
    assert_eq!((module.path.as_str(), module.via_glob), ("demo::m", true));
    let value = ws
        .resolve_in("demo::observed::m", Namespace::Value)
        .expect("value");
    assert_eq!((value.path.as_str(), value.via_glob), ("demo::m::m", false));
    // A path that goes on through the name reads the module.
    assert_eq!(path_of(&ws, "demo::observed::m::m"), "demo::m::m");
    // Without the explicit import the glob answers in the type name space
    // too.
    let ws = lib("pub mod m { pub fn m() {} }\npub mod observed { use super::*; }\n");
    assert_eq!(path_of(&ws, "demo::observed::m"), "demo::m");
    // An import of an item the index cannot see keeps the name from the
    // globs in every name space: it may bind it in each.
    let ws = load_with(
        "2021",
        "[dependencies]\nserde = \"1\"\n",
        "pub mod m { pub fn m() {} }\npub mod observed { use serde::m; use super::*; }\n",
    );
    not_found_in(&ws, "demo::observed::m", Namespace::Type);
}

/// Assert that `path` names nothing in `ns`.
fn not_found_in(ws: &Workspace, path: &str, ns: Namespace) {
    assert_eq!(
        ws.resolve_in(path, ns),
        Err(ResolveError::NotFound(path.to_owned()))
    );
}

/// Load `text` as `demo`'s library, beside a manifest of `edition` with the
/// tables `extra`.
fn load_with(edition: &str, extra: &str, text: &str) -> Workspace {
    let manifest = format!("{DEMO}edition = \"{edition}\"\n{extra}");
    let tree: MemoryTree = [("Cargo.toml", manifest.as_str()), ("src/lib.rs", text)]
        .into_iter()
        .collect();
    Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"))
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
fn a_cycle_of_imports_through_a_crate_name_and_a_glob_is_cyclic_in_every_order() {
    // Each import's first segment is the name the other binds, and an
    // explicit import shadows both a crate and a glob, so neither import may
    // start at the library `shared` or read the glob: rustc cannot determine
    // either of them.
    let prefix = "pub mod k {\n\
         pub mod inn { pub mod inner { pub mod other { pub fn x() {} } } }\n\
         pub mod other { pub fn x() {} pub trait T { fn m(&self); } }\n}\n\
         impl self::k::other::T for String { fn m(&self) {} }";
    let key = raf_source::TraitKey::Canonical("demo::k::other::T".to_owned());
    for (blocks, paths) in [
        (
            ["use shared::inner as inn;", "use inn::inner as shared;"],
            ["demo::inn", "demo::shared"],
        ),
        (
            ["pub use inn::inner::other;", "pub use other as inn;"],
            ["demo::other", "demo::inn"],
        ),
    ] {
        let mut all = blocks.to_vec();
        all.push("use self::k::*;");
        for path in paths {
            let (resolved, found, orders) = every_order(prefix, None, &all, path);
            assert_eq!(orders, 6);
            assert_eq!(resolved, Err(vec![format!("Cyclic({path:?})")]), "{all:?}");
            assert_eq!(found, key);
        }
    }
    // Without one edge of the cycle, the other import reads the glob.
    let (resolved, found, orders) = every_order(
        prefix,
        None,
        &["pub use inn::inner::other;", "use self::k::*;"],
        "demo::other::x",
    );
    assert_eq!(orders, 2);
    assert_eq!(resolved, Ok("demo::k::inn::inner::other::x".to_owned()));
    assert_eq!(found, key);
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
fn an_import_binds_what_a_name_it_read_gains_later_in_every_order() {
    // `sys` has two `cfg` alternatives. The second starts at the crate
    // `shared`, which a module with a glob reads only once the globs have
    // brought in what they can, so `pub use sys::x;` may first read `sys`
    // with the alternative that has no `x`. It is read again when `sys`
    // gains the other, whatever the order of the declarations.
    let prefix = "pub mod unix { pub fn only_unix() {} }\npub mod g {}\nuse self::g::*;\n\
         impl shared::inner::other::T for String { fn m(&self) {} }";
    let (resolved, key, orders) = every_order_in(
        beside_two,
        prefix,
        None,
        &[
            "#[cfg(unix)] use self::unix as sys;",
            "#[cfg(windows)] use shared::inner::other as sys;",
            "pub use sys::x;",
        ],
        "demo::x",
    );
    assert_eq!(orders, 6);
    assert_eq!(resolved, Ok("shared::inner::other::x".to_owned()));
    assert_eq!(
        key,
        raf_source::TraitKey::Canonical("shared::inner::other::T".to_owned())
    );
    // Through the alternative that has the name, and not through the other.
    let ws = beside_two(&format!(
        "{prefix}\n#[cfg(unix)] use self::unix as sys;\n\
         #[cfg(windows)] use shared::inner::other as sys;\npub use sys::only_unix;\n"
    ));
    assert_eq!(path_of(&ws, "demo::only_unix"), "demo::unix::only_unix");
    not_found(&ws, "demo::sys::nope");
}

#[test]
fn a_crate_name_an_explicit_import_binds_is_never_read_as_the_crate_in_every_order() {
    // `pub use k::other;` could bind `other` only once the glob of `k` brings
    // it in, and that glob goes through `inn`, whose own path starts at
    // `other`. The explicit import shadows the crate `other`, so `inn` never
    // reads the crate: the two imports and the glob wait on one another, a
    // cycle, and neither import binds. rustc refuses the import (E0432); the
    // index answers the same whatever the order of the declarations.
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
    assert_eq!(other, Err(vec!["Cyclic(\"demo::other\")".to_owned()]));
}

/// The answer to `path` in `ns`, or in every name space, with the items of
/// an ambiguity by path.
fn answer(ws: &Workspace, path: &str, ns: Option<Namespace>) -> String {
    let result = match ns {
        Some(ns) => ws.resolve_in(path, ns),
        None => ws.resolve(path),
    };
    match result {
        Ok(resolution) => resolution.path,
        Err(ResolveError::Ambiguous(ids)) => format!(
            "ambiguous {:?}",
            ids.iter()
                .map(|id| ws.item(*id).expect("item").path.to_string())
                .collect::<Vec<_>>()
        ),
        Err(other) => format!("{other:?}"),
    }
}

/// Load every order of `blocks` with `load`, and assert that the answers to
/// `queries` and the trait keys of the load are the same in each. Returns
/// them, the keys last, and the number of orders.
fn one_answer_in_every_order(
    load: fn(&str) -> Workspace,
    blocks: &[&str],
    queries: &[(&str, Option<Namespace>)],
) -> (Vec<String>, usize) {
    let mut expected: Option<Vec<String>> = None;
    let mut orders = 0;
    for order in permutations(&blocks.iter().map(|b| (*b).to_owned()).collect::<Vec<_>>()) {
        let text = order.join("\n");
        let ws = load(&text);
        let mut found: Vec<String> = queries
            .iter()
            .map(|(path, ns)| answer(&ws, path, *ns))
            .collect();
        let mut keys: Vec<String> = ws
            .items()
            .filter_map(|item| {
                item.trait_key
                    .as_deref()
                    .map(|key| format!("{} {key:?}", item.path))
            })
            .collect();
        keys.sort();
        found.push(keys.join("; "));
        match &expected {
            None => expected = Some(found),
            Some(first) => assert_eq!(&found, first, "{text}"),
        }
        orders += 1;
    }
    (expected.expect("an order"), orders)
}

#[test]
fn an_import_keeps_the_globs_out_only_in_the_name_spaces_it_binds_in_every_order() {
    // `m` imports the struct `y::A` and globs the function `x::A`; `n` globs
    // `m` and the function `z::A`. In the value name space `n` sees both
    // functions, rustc's E0659, and in the type name space the struct.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "mod x { pub fn A() {} }",
            "mod y { pub struct A {} }",
            "mod z { pub fn A() {} }",
            "mod m { pub use crate::x::*; pub use crate::y::A; }",
            "mod n { pub use crate::m::*; pub use crate::z::*; }",
        ],
        &[
            ("demo::n::A", Some(Namespace::Value)),
            ("demo::n::A", Some(Namespace::Type)),
            ("demo::m::A", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 120);
    assert_eq!(
        found,
        [
            "ambiguous [\"demo::x::A\", \"demo::z::A\"]",
            "demo::y::A",
            "demo::x::A",
            "",
        ]
    );
    // The same with traits, and a trait impl through `n`: rustc's E0659,
    // and the key has both traits.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "mod x { pub trait T { fn m(&self); } }",
            "mod y { pub fn T() {} }",
            "mod z { pub trait T { fn m(&self); } }",
            "mod m { pub use crate::x::*; pub use crate::y::T; }",
            "mod n { pub use crate::m::*; pub use crate::z::*; }",
            "impl n::T for String { fn m(&self) {} }",
        ],
        &[("demo::n::T", Some(Namespace::Type))],
    );
    assert_eq!(orders, 720);
    assert_eq!(
        found,
        [
            "ambiguous [\"demo::x::T\", \"demo::z::T\"]",
            "n::T::m Unresolved { written: \"n::T\", candidates: [\"demo::x::T\", \"demo::z::T\"] }",
        ]
    );
    // An import of a function `b::B` beside a glob of a struct `c::B`, read
    // through another module: the type name space has the struct.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod a { pub use crate::b::B; pub use crate::c::*; }",
            "pub mod b { pub fn B() {} }",
            "pub mod c { pub struct B {} }",
            "pub mod e { pub use crate::a::B; }",
        ],
        &[
            ("demo::e::B", Some(Namespace::Type)),
            ("demo::e::B", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 24);
    assert_eq!(found, ["demo::c::B", "demo::b::B", ""]);
}

#[test]
fn an_import_of_a_value_named_like_a_crate_leaves_the_crate_and_the_globs_in_every_order() {
    // A function imported under the name of the library `shared` keeps the
    // crate only from the value name space, so a `use` path through the
    // name reads the crate.
    let (found, orders) = one_answer_in_every_order(
        beside_shared,
        &[
            "pub mod vals { pub fn shared() {} }",
            "pub mod m { use crate::vals::shared; pub use shared::x; }",
        ],
        &[("demo::m::x", None)],
    );
    assert_eq!(orders, 2);
    assert_eq!(found, ["shared::x", ""]);
    // Where a glob also brings in a module `shared` through another module,
    // the path through the name is the glob's module and the crate, rustc's
    // E0659.
    let (found, orders) = one_answer_in_every_order(
        beside_shared,
        &[
            "pub mod vals { pub fn shared() {} }",
            "pub mod k { pub mod shared { pub fn x() {} } }",
            "pub mod p { pub use crate::vals::shared; pub use crate::k::*; }",
            "pub mod n { use crate::p::*; pub use shared::x; }",
        ],
        &[
            ("demo::n::x", None),
            ("demo::p::shared", Some(Namespace::Type)),
        ],
    );
    assert_eq!(orders, 24);
    assert_eq!(
        found,
        [
            "ambiguous [\"demo::k::shared\", \"shared\"]",
            "demo::k::shared",
            "",
        ]
    );
}

#[test]
fn a_cycle_of_imports_with_one_glob_on_it_resolves_through_the_glob_in_every_order() {
    // rustc resolves an import without the import itself: in a cycle where
    // one module has a glob that brings the name in, the import that reads
    // that module reads the glob, and the rest of the cycle follows it.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod b { pub trait T { fn m(&self); } }",
            "pub mod e { pub use crate::d::T; }",
            "pub mod d { pub use crate::e::T; pub use crate::b::*; }",
            "pub mod w { pub use crate::e::T as W; }",
            "impl e::T for String { fn m(&self) {} }",
        ],
        &[
            ("demo::e::T", None),
            ("demo::d::T", None),
            ("demo::w::W", None),
            ("demo::e::T", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 120);
    assert_eq!(
        found,
        [
            "demo::b::T",
            "demo::b::T",
            "demo::b::T",
            "NotFound(\"demo::e::T\")",
            "demo::b::T::m Canonical(\"demo::b::T\")",
        ]
    );
    // Three imports, one glob.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod g { pub struct X; }",
            "pub mod a { pub use crate::b::X; pub use crate::g::*; }",
            "pub mod b { pub use crate::c::X; }",
            "pub mod c { pub use crate::a::X; }",
        ],
        &[
            ("demo::a::X", None),
            ("demo::b::X", None),
            ("demo::c::X", None),
        ],
    );
    assert_eq!(orders, 24);
    assert_eq!(found, ["demo::g::X", "demo::g::X", "demo::g::X", ""]);
    // Two globs on the cycle, of one module or of two, or renamed imports
    // in it, leave every import of the cycle undetermined, as in rustc.
    let cyclic = |path: &str| format!("Cyclic({path:?})");
    for (blocks, paths) in [
        (
            [
                "pub mod g { pub struct X; }",
                "pub mod a { pub use crate::b::X; pub use crate::g::*; }",
                "pub mod b { pub use crate::a::X; pub use crate::g::*; }",
            ],
            vec!["demo::a::X", "demo::b::X"],
        ),
        (
            [
                "pub mod g { pub struct X; }",
                "pub mod a { pub use crate::b::X; pub use crate::g::*; }",
                "pub mod b { pub use crate::c::X; pub use crate::g::*; }\n\
                 pub mod c { pub use crate::a::X; }",
            ],
            vec!["demo::a::X", "demo::b::X", "demo::c::X"],
        ),
        (
            [
                "pub mod g { pub struct A; pub struct B; }",
                "pub mod a { pub use self::B as A; pub use self::A as B; pub use crate::g::*; }",
                "pub mod h { pub use crate::a::A; }",
            ],
            vec!["demo::a::A", "demo::a::B", "demo::h::A"],
        ),
    ] {
        let queries: Vec<(&str, Option<Namespace>)> =
            paths.iter().map(|path| (*path, None)).collect();
        let (found, orders) = one_answer_in_every_order(lib, &blocks, &queries);
        assert_eq!(orders, 6);
        let mut expected: Vec<String> = paths.iter().map(|path| cyclic(path)).collect();
        expected.push(String::new());
        assert_eq!(found, expected, "{blocks:?}");
    }
}

#[test]
fn an_import_that_bound_a_name_before_the_crates_opened_binds_the_crate_later_in_every_order() {
    // `pub use other as inn;` binds the glob's function `other` in the
    // value name space before the crates of the extern prelude are read,
    // and the library `other` in the type name space once they are.
    let (found, orders) = one_answer_in_every_order(
        beside_two,
        &[
            "pub mod vals { pub fn other() {} }",
            "pub mod a { pub use other as inn; pub use crate::e::*; }",
            "pub mod e { pub use crate::vals::other; }",
        ],
        &[
            ("demo::a::inn::x", None),
            ("demo::a::inn", Some(Namespace::Type)),
            ("demo::a::inn", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 6);
    assert_eq!(found, ["other::x", "other", "demo::vals::other", ""]);
    // The same where the glob's function is a definition, so that nothing
    // the import read changes once the crates are read: it is read again
    // all the same.
    let (found, orders) = one_answer_in_every_order(
        beside_two,
        &[
            "pub mod a { pub use other as inn; pub use crate::e::*; }",
            "pub mod e { pub fn other() {} }",
        ],
        &[
            ("demo::a::inn::x", None),
            ("demo::a::inn", Some(Namespace::Type)),
            ("demo::a::inn", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 2);
    assert_eq!(found, ["other::x", "other", "demo::e::other", ""]);
}

#[test]
fn imports_that_wait_on_one_another_through_globs_are_decided_as_rustc_does_in_every_order() {
    // A cycle `a::B` -> `b::B` -> `e::B` -> `a::B` whose one glob, of `c`,
    // brings in a function `B`, while `c` also globs `e`, whose `B` is the
    // cycle's: the cycle resolves to the function in the value name space and
    // binds nothing in the type name space, as rustc has it.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod c { pub use crate::e::*; pub fn B() {} }",
            "pub mod e { pub use crate::a::B as B; }",
            "pub mod a { pub use crate::c::*; pub use crate::b::B; }",
            "pub mod b { pub use crate::a::B as A; pub use crate::e::B as B; }",
        ],
        &[
            ("demo::a::B", Some(Namespace::Value)),
            ("demo::b::A", Some(Namespace::Value)),
            ("demo::e::B", Some(Namespace::Type)),
        ],
    );
    assert_eq!(orders, 24);
    assert_eq!(
        found,
        ["demo::c::B", "demo::c::B", "NotFound(\"demo::e::B\")", ""]
    );
    // `c` imports the struct `e::A`, and `e` globs `c`: in the value name
    // space the import reads only its own name through the glob, which rustc
    // leaves out, so it binds nothing there and `a`'s renamed import of
    // `e::A` leaves the value name space of `B` to the glob's function.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod c { pub use crate::d::*; pub use crate::e::A; pub fn B() {} }",
            "pub mod d {}",
            "pub mod e { pub use crate::c::*; pub struct A {} }",
            "pub mod a { pub use crate::e::*; pub use crate::e::A as B; }",
        ],
        &[
            ("demo::a::B", Some(Namespace::Value)),
            ("demo::a::B", Some(Namespace::Type)),
        ],
    );
    assert_eq!(orders, 24);
    assert_eq!(found, ["demo::c::B", "demo::e::A", ""]);
}

#[test]
fn imports_whose_heads_are_names_their_globs_bring_in_are_decided_alike_in_every_order() {
    // Each of `b` and `c` imports `shared::inner as shared` and globs the
    // other, so each import's first segment is a name the other module's
    // import may still bring in through the glob: neither may start at the
    // crate `shared` before the other is settled, so the two wait on one
    // another and neither binds. rustc refuses both (E0659, E0432); the index
    // gives one answer whatever the order.
    let (found, orders) = one_answer_in_every_order(
        beside_two,
        &[
            "pub mod b { pub use shared::inner as shared; pub use super::c::*; }",
            "pub mod c { pub use super::b::*; pub use shared::inner as shared; }",
        ],
        &[("demo::b::shared", None), ("demo::c::shared", None)],
    );
    assert_eq!(orders, 2);
    assert_eq!(
        found,
        [
            "Cyclic(\"demo::b::shared\")",
            "Cyclic(\"demo::c::shared\")",
            ""
        ]
    );
    // A glob whose first segment is a crate that a glob of its module also
    // brings in brings in nothing, also once the globs are rebuilt against
    // the settled tables, so a module that imports through it finds nothing.
    let (found, orders) = one_answer_in_every_order(
        beside_two,
        &[
            "pub mod a { pub use shared::inner as shared; }",
            "pub mod c { pub use super::a::*; pub use shared::*; }",
            "pub mod b { pub use super::c::inner; }",
        ],
        &[
            ("demo::c::inner", None),
            ("demo::b::inner", None),
            ("demo::c::shared", None),
        ],
    );
    assert_eq!(orders, 6);
    assert_eq!(
        found,
        [
            "NotFound(\"demo::c::inner\")",
            "NotFound(\"demo::b::inner\")",
            "shared::inner",
            "",
        ]
    );
}

#[test]
fn a_name_space_binds_nothing_only_once_the_globs_that_may_bring_it_in_are_settled_in_every_order()
{
    // `m` imports `t::X`, a struct, and globs a function `X` of `g`. `t`
    // globs `q`, whose own glob of `late` resolves only once `u` leaves the
    // type name space of `late` to its glob of `w`: that glob brings the
    // function `late::X` into `t`, so the import binds it in the value name
    // space too, and the glob of `g` never answers there, as in rustc.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod v { pub fn late() {} }\npub mod w { pub mod late { pub fn X() {} } }\n\
             pub mod u { pub use crate::v::late; pub use crate::w::*; }",
            "pub mod q { pub use crate::u::late; pub use self::late::*; }",
            "pub mod t { pub use crate::q::*; pub struct X {} }\npub mod g { pub fn X() {} }",
            "pub mod m { pub use crate::t::X; pub use crate::g::*; }\n\
             pub mod r { pub use crate::m::X as R; }",
        ],
        &[
            ("demo::r::R", Some(Namespace::Value)),
            ("demo::r::R", Some(Namespace::Type)),
            ("demo::m::X", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 24);
    assert_eq!(
        found,
        ["demo::w::late::X", "demo::t::X", "demo::w::late::X", ""]
    );
}

#[test]
fn the_rebuilt_globs_read_each_glob_path_in_the_settled_tables_in_every_order() {
    // `c` globs `a`, which brings in `inner` from `l2` and an alias `shared`
    // of the library's `inner`, and globs `shared::*`, whose first segment
    // is then ambiguous with the library `shared`: it brings in nothing, so
    // `c::inner` is `l2::inner` alone. Read against tables whose globs are
    // cleared, the path would start at the library and bring in its
    // `inner` as well.
    let (found, orders) = one_answer_in_every_order(
        beside_two,
        &[
            "pub mod l2 { pub mod inner { pub mod other { pub fn x() {} } } }",
            "pub mod a { pub use shared::inner as shared; pub use super::l2::*; }",
            "pub mod c { pub use super::a::*; pub use shared::*; }",
            "pub mod b { pub use super::c::inner::other; }",
        ],
        &[("demo::b::other", None), ("demo::c::inner", None)],
    );
    assert_eq!(orders, 24);
    assert_eq!(found, ["demo::l2::inner::other", "demo::l2::inner", ""]);
}

/// The key of the one trait impl on `u8` in `ws`.
fn string_or_u8_key(ws: &Workspace) -> raf_source::TraitKey {
    let keys: Vec<raf_source::TraitKey> = ws
        .items()
        .filter(|item| item.self_type.as_deref() == Some("u8"))
        .filter_map(|item| item.trait_key.as_deref().cloned())
        .collect();
    match keys.as_slice() {
        [key] => key.clone(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn globs_that_glob_one_another_beside_names_bound_twice_settle_in_every_order() {
    // Five modules glob one another and bind `A` and `B` twice (rustc's
    // E0252 and E0255): what a glob may still bring in is found from the
    // imports that keep a name open, never from the globs alone, so the
    // tables settle whatever the order.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod d { pub fn A() {} pub use crate::e::*; pub use crate::d::A; pub use crate::c::*; }",
            "pub mod e { pub use crate::b::*; pub use crate::e::B; pub struct B {} pub use crate::a::*; }",
            "pub mod b { pub struct A {} pub struct B {} pub use crate::b::A; }",
            "pub mod c { pub use crate::d::*; pub use crate::a::B; pub use crate::e::*; }",
            "pub mod a { pub use crate::b::A as B; pub struct B {} pub use crate::b::B; pub use crate::c::*; }",
        ],
        &[("demo::c::A", None), ("demo::c::B", None)],
    );
    assert_eq!(orders, 120);
    assert_eq!(
        found,
        [
            "demo::b::A",
            "ambiguous [\"demo::a::B\", \"demo::b::A\", \"demo::b::B\"]",
            "",
        ]
    );
    // A module whose globs both start at crates: each glob's path reads its
    // first segment without the glob itself, and the globs settle.
    let ws = beside_two("use shared::*;\nuse other::*;\npub use x as y;\n");
    assert_eq!(path_of(&ws, "demo::y"), "other::x");
}

#[test]
fn a_glob_whose_path_turns_out_ambiguous_brings_in_nothing_in_every_order() {
    // `c` globs `shared::*` and `a`, which brings in an alias `shared` of
    // the library's `inner`: once `a`'s glob has brought it in, the first
    // segment of `shared::*` is ambiguous with the library `shared`, so the
    // glob brings in nothing, also where its path read the library before
    // that and brought names in: the tables are settled again with it held.
    let (found, orders) = one_answer_in_every_order(
        beside_two,
        &[
            "pub mod a { pub use q::q as p; pub use shared::inner as shared; \
             pub use super::c::shared::X as Y; pub use super::l2::*; }",
            "pub mod b { pub use super::c as shared; pub use shared::inner::other; \
             pub use super::a::shared::X as Y; }",
            "pub mod l2 { pub struct X; pub mod inner { pub mod other { pub fn x() {} } } }",
            "pub mod c { pub use shared::shared as q; pub use self::shared::X; \
             pub use shared::*; pub use super::a::*; }",
        ],
        &[("demo::b::other", None), ("demo::c::inner", None)],
    );
    assert_eq!(orders, 24);
    assert_eq!(found, ["demo::l2::inner::other", "demo::l2::inner", ""]);
}

#[test]
fn a_glob_brings_in_what_its_target_binds_later_whatever_its_target_holds_under_its_first_segment()
{
    // `shared::prelude` binds `shared` only through a cycle of renamed
    // imports, which never settles, and `T` through a chain of re-exports
    // that may settle after the glob of `demo` first reads it. The glob's
    // own path is read without what the glob itself may bring in, so it
    // brings in `T` once the chain settles.
    let lib_text = "pub mod c1 { pub use super::c2::T; }\npub mod c2 { pub use super::c3::T; }\n\
                    pub mod c3 { pub trait T { fn m(&self); } }\n\
                    pub mod prelude { pub use self::s2 as shared; pub use self::shared as s2; \
                    pub use crate::c1::T; }\n";
    let tree: MemoryTree = [
        ("Cargo.toml", "[workspace]\nmembers = [\"one\", \"demo\"]\n"),
        (
            "one/Cargo.toml",
            "[package]\nname = \"p-one\"\nedition = \"2021\"\n\n[lib]\nname = \"shared\"\n",
        ),
        ("one/src/lib.rs", lib_text),
        (
            "demo/Cargo.toml",
            "[package]\nname = \"demo\"\nedition = \"2021\"\n\n[dependencies]\np-one = { path = \"../one\" }\n",
        ),
        (
            "demo/src/lib.rs",
            "use shared::prelude::*;\nimpl T for u8 { fn m(&self) {} }\n",
        ),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        string_or_u8_key(&ws),
        raf_source::TraitKey::Canonical("shared::c3::T".to_owned())
    );
}

#[test]
fn an_import_whose_glob_reads_it_starts_at_the_crate_once_no_glob_may_bring_the_name_in_every_order()
 {
    // `use shared::inner as inn;` reads the crate `shared` once `e`'s import
    // of the function `vals::shared` leaves the type name space, and `use
    // inn::*;` then reads it: the glob, which waits on the import, does not
    // hold the import's first segment back, as rustc reads it.
    let (found, orders) = one_answer_in_every_order(
        beside_two,
        &[
            "pub mod vals { pub fn shared() {} }",
            "pub mod e { pub use crate::vals::shared; }",
            "pub mod b { pub use crate::e::*; pub use shared::inner as inn; pub use inn::*; }",
        ],
        &[
            ("demo::b::inn", Some(Namespace::Type)),
            ("demo::b::other::x", None),
            ("demo::b::shared", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 6);
    assert_eq!(
        found,
        [
            "shared::inner",
            "shared::inner::other::x",
            "demo::vals::shared",
            "",
        ]
    );
    // The same where `vals` globs an empty module, so that `e`'s import is
    // decided only once the queue is empty: the first segment of `b`'s
    // import still does not wait on `b`'s glob that reads the import.
    let (found, orders) = one_answer_in_every_order(
        beside_two,
        &[
            "pub mod z {}",
            "pub mod vals { pub fn shared() {} pub use crate::z::*; }",
            "pub mod e { pub use crate::vals::shared; }",
            "pub mod b { pub use crate::e::*; pub use shared::inner as inn; pub use inn::*; }",
        ],
        &[
            ("demo::b::inn", Some(Namespace::Type)),
            ("demo::b::other::x", None),
        ],
    );
    assert_eq!(orders, 24);
    assert_eq!(found, ["shared::inner", "shared::inner::other::x", ""]);
}

#[test]
fn an_item_a_glob_reaches_by_a_public_and_a_private_route_is_visible_in_every_order() {
    // `m1` brings in `m2::X` through its public glob of `m2` and through its
    // private glob of `m4`, whose `X` is `m3::X`, which `m3` brings in from
    // `m1`. Whichever route is kept, the item is as visible as its most
    // visible route, so `m3`, which globs `m1`, sees it in every order, as
    // rustc does.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod m3 { pub use crate::m1::*; }",
            "pub mod m4 { pub use crate::m3::X; }",
            "pub mod m1 { pub use crate::m2::*; use crate::m4::*; }",
            "pub mod m2 { pub fn X() {} }",
        ],
        &[("demo::m3::X", None), ("demo::m4::X", None)],
    );
    assert_eq!(orders, 24);
    assert_eq!(found, ["demo::m2::X", "demo::m2::X", ""]);
    // Through the private route alone, `m3` cannot see it.
    let ws = lib(
        "pub mod m3 { pub use crate::m1::*; }\npub mod m1 { use crate::m2::*; }\n\
                  pub mod m2 { pub fn X() {} }\n",
    );
    not_found(&ws, "demo::m3::X");
}

#[test]
fn a_cycle_through_a_glob_and_a_renamed_import_is_decided_in_each_name_space_in_every_order() {
    // `a::B` and `b::B` import each other. In the type name space the glob
    // of `d` in `a` answers, `d::B` being `e::B`, the renamed `b::A`; in the
    // value name space no glob answers and nothing binds. The name spaces
    // are decided apart, so the cycle resolves in the one and binds nothing
    // in the other, as rustc does.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod d { pub use crate::e::B; }",
            "pub mod b { pub use crate::a::B; pub use crate::c::*; pub struct A {} }",
            "pub mod e { pub use crate::b::A as B; }",
            "pub mod c { pub use crate::e::B as A; }",
            "pub mod a { pub use crate::d::*; pub use crate::b::B; }",
        ],
        &[
            ("demo::a::B", Some(Namespace::Type)),
            ("demo::b::B", Some(Namespace::Type)),
            ("demo::a::B", Some(Namespace::Value)),
            ("demo::c::A", Some(Namespace::Type)),
        ],
    );
    assert_eq!(orders, 120);
    assert_eq!(
        found,
        [
            "demo::b::A",
            "demo::b::A",
            "NotFound(\"demo::a::B\")",
            "demo::b::A",
            ""
        ]
    );
}

#[test]
fn an_import_of_its_own_module_reads_what_the_globs_bring_in_in_every_order() {
    // `c` imports `c::A`, its own name: rustc reads it without the import,
    // through the glob of `d`, whose `A` is the renamed `a::B`, a value. So
    // the import binds the value and nothing in the type name space, which
    // `e`'s renamed import of `a::A` leaves to `e`'s glob of `c`: `c::B`.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod c { pub struct B {} pub use crate::d::*; pub use crate::c::A; }",
            "pub mod d { pub use crate::a::B as A; pub const fn B() -> u32 { 4 } }",
            "pub mod e { pub use crate::c::*; pub use crate::a::A as B; }",
            "pub mod a { pub use crate::d::*; pub use crate::e::A; }",
        ],
        &[
            ("demo::e::B", Some(Namespace::Type)),
            ("demo::e::B", Some(Namespace::Value)),
            ("demo::c::A", Some(Namespace::Value)),
            ("demo::c::A", Some(Namespace::Type)),
        ],
    );
    assert_eq!(orders, 24);
    assert_eq!(
        found,
        [
            "demo::c::B",
            "demo::d::B",
            "demo::d::B",
            "NotFound(\"demo::c::A\")",
            ""
        ]
    );
}

#[test]
fn a_path_through_an_import_a_cycle_leaves_undecided_in_another_name_space_is_cyclic() {
    // `c::B` and `x::B` import each other and nothing else answers, so the
    // cycle never settles. `e` imports `d::B`, a function in the value name
    // space and, in the type name space, whatever `d`'s glob of `c` brings
    // in, which waits on the cycle. rustc refuses the crate, failing an
    // import of the cycle (E0432). The index answers nothing through an
    // import a cycle leaves undecided in a name space, in any name space,
    // nor through one that imports from it.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod d { pub fn B() {} pub use crate::c::*; }",
            "pub mod c { pub use crate::x::B; }",
            "pub mod x { pub use crate::c::B; }",
            "pub mod e { pub use crate::d::B as A; }",
            "pub mod y { pub use crate::e::A as Q; }",
        ],
        &[
            ("demo::d::B", Some(Namespace::Value)),
            ("demo::e::A", Some(Namespace::Value)),
            ("demo::e::A", Some(Namespace::Type)),
            ("demo::y::Q", Some(Namespace::Value)),
            ("demo::c::B", Some(Namespace::Type)),
        ],
    );
    assert_eq!(orders, 120);
    assert_eq!(
        found,
        [
            "demo::d::B",
            "Cyclic(\"demo::e::A\")",
            "Cyclic(\"demo::e::A\")",
            "Cyclic(\"demo::y::Q\")",
            "Cyclic(\"demo::c::B\")",
            ""
        ]
    );
    // Where the cycle runs through globs, rustc reports each import that
    // waits on it as one it cannot determine, whatever it bound in another
    // name space: here `e`'s import of `d::B`, a function `b` defines in the
    // value name space and, in the type name space, what `b`'s import of
    // `a::B` binds, which waits on the globs of `a`, `d` and `b`.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod b { pub use crate::a::B; pub use crate::a::*; pub use crate::e::*; pub fn B() {} }",
            "pub mod e { pub struct B {} pub use crate::d::B as A; }",
            "pub mod a { pub use crate::d::*; pub struct A {} }",
            "pub mod d { pub use crate::b::*; }",
        ],
        &[
            ("demo::e::A", Some(Namespace::Value)),
            ("demo::e::A", Some(Namespace::Type)),
        ],
    );
    assert_eq!(orders, 24);
    assert_eq!(
        found,
        ["Cyclic(\"demo::e::A\")", "Cyclic(\"demo::e::A\")", ""]
    );
    // The same import, once the cycle is gone, binds in the value name
    // space.
    let ws = lib(
        "pub mod d { pub fn B() {} pub use crate::c::*; }\npub mod c {}\n\
                  pub mod e { pub use crate::d::B as A; }\n",
    );
    assert_eq!(answer(&ws, "demo::e::A", None), "demo::d::B");
}

#[test]
fn a_cycle_whose_import_reads_its_own_name_through_another_glob_is_cyclic_in_every_order() {
    // `m0` imports `m3::Z`, which `m3`'s import of `m4::Z` keeps from
    // `m3`'s globs; `m4`'s `Z` is what its glob of `m3` brings in, so the
    // import of `m4::Z` reads its own name through another module's glob.
    // The cycle leaves `m3::Z` to `m3`'s globs while `m0`'s import reads it,
    // but `m4`'s glob does not bring the name in from `m3` until `m3`'s
    // import is decided, which never happens: rustc never determines that
    // import (E0432), and the index answers `Cyclic` through it, and through
    // `m0`'s import, which waits on it in the type name space, whatever the
    // order.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod m7 { pub fn Z() {} }",
            "pub mod m6 { pub use crate::m0::*; }",
            "pub mod m4 { pub use crate::m3::*; }",
            "pub mod m3 { pub use crate::m4::Z; pub use crate::m7::*; pub use crate::m6::*; use crate::m2::*; }",
            "pub mod m0 { pub use crate::m3::Z; }",
            "pub mod m2 { pub fn Z() {} }",
        ],
        &[
            ("demo::m0::Z", Some(Namespace::Value)),
            ("demo::m3::Z", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 720);
    assert_eq!(
        found,
        ["Cyclic(\"demo::m0::Z\")", "Cyclic(\"demo::m3::Z\")", ""]
    );
    // A cycle through renamed imports and globs whose one glob answer, at
    // `e::B`, leaves `b`'s and `c`'s renamed imports waiting on each other:
    // rustc reads a place through the glob only while the cycle's imports
    // resolve without it, which they never do here, so it cannot determine
    // the imports (E0432), and the index decides none of them.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod c { pub use crate::b::B as A; }",
            "pub mod e { pub use crate::b::B; pub use crate::d::*; pub use crate::b::B as A; }",
            "pub mod a { pub use crate::d::*; pub use crate::e::*; pub use crate::e::B; }",
            "pub mod d { pub struct B {} pub const fn A() -> u32 { 2 } pub use crate::c::*; }",
            "pub mod b { pub use crate::d::A as B; pub use crate::a::*; }",
        ],
        &[
            ("demo::a::B", Some(Namespace::Type)),
            ("demo::a::B", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 120);
    assert_eq!(
        found,
        ["Cyclic(\"demo::a::B\")", "Cyclic(\"demo::a::B\")", ""]
    );
    // Where the name read back is one an import of the cycle binds, the path
    // reads it through that import and not through the glob: `e`'s import of
    // `c::B` keeps `e::B` from `e`'s glob of `d`, and `c::B` is `c`'s import
    // of `e::B`, though `c` globs `e` too. The cycle resolves through `e`'s
    // glob, as in rustc: `d::B` for both.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod e { pub use crate::d::*; pub use crate::c::B as B; }",
            "pub mod c { pub use crate::e::*; pub use crate::e::B; }",
            "pub mod d { pub use crate::c::*; pub struct B {} }",
            "pub mod b { pub const fn B() -> u32 { 1 } }",
        ],
        &[
            ("demo::c::B", Some(Namespace::Type)),
            ("demo::e::B", Some(Namespace::Type)),
        ],
    );
    assert_eq!(orders, 24);
    assert_eq!(found, ["demo::d::B", "demo::d::B", ""]);
}

/// [`one_answer_in_every_order`] for blocks too many to load in every order:
/// each rotation of the blocks, forward and reversed.
fn one_answer_in_every_rotation(
    blocks: &[&str],
    queries: &[(&str, Option<Namespace>)],
) -> (Vec<String>, usize) {
    one_answer_in_every_rotation_of(lib, blocks, queries)
}

/// [`one_answer_in_every_rotation`] loaded with `load`.
fn one_answer_in_every_rotation_of(
    load: fn(&str) -> Workspace,
    blocks: &[&str],
    queries: &[(&str, Option<Namespace>)],
) -> (Vec<String>, usize) {
    let mut expected: Option<Vec<String>> = None;
    let mut orders = 0;
    for reversed in [false, true] {
        let mut order: Vec<&str> = blocks.to_vec();
        if reversed {
            order.reverse();
        }
        for _ in 0..order.len() {
            order.rotate_left(1);
            let text = order.join("\n");
            let ws = load(&text);
            let found: Vec<String> = queries
                .iter()
                .map(|(path, ns)| answer(&ws, path, *ns))
                .collect();
            match &expected {
                None => expected = Some(found),
                Some(first) => assert_eq!(&found, first, "{text}"),
            }
            orders += 1;
        }
    }
    (expected.expect("an order"), orders)
}

#[test]
fn a_name_space_binds_nothing_only_once_the_globs_its_globs_reach_are_settled() {
    // `user` imports `m::T`, which `m` defines as a function and which, in
    // the type name space, `m`'s glob of `a` brings in once `a`'s glob path
    // `h::inner` resolves, through a cycle that resolves through `j`'s glob.
    // `m`'s glob is not settled while `a`'s is not, so the import's type
    // name space is not left to `user`'s glob of `o` meanwhile: it binds
    // `z::inner::T`, as rustc has it, and `r`, which imports it, too.
    let (found, orders) = one_answer_in_every_rotation(
        &[
            "pub mod z { pub mod inner { pub struct T {} } }",
            "pub mod j { pub use crate::h::inner; pub use crate::z::*; }",
            "pub mod h { pub use crate::j::inner; }",
            "pub mod a { pub use crate::h::inner::*; }",
            "pub mod m { pub use crate::a::*; pub fn T() {} }",
            "pub mod o { pub struct T {} }",
            "pub mod user { pub use crate::m::T; pub use crate::o::*; }",
            "pub mod r { pub use crate::user::T; }",
        ],
        &[
            ("demo::user::T", Some(Namespace::Type)),
            ("demo::r::T", Some(Namespace::Type)),
            ("demo::r::T", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 16);
    assert_eq!(
        found,
        ["demo::z::inner::T", "demo::z::inner::T", "demo::m::T"]
    );
}

#[test]
fn a_cycle_through_a_renamed_import_that_has_bound_nothing_is_cyclic_in_every_order() {
    // `a` imports `b::X as Y` and globs `g`, whose `Y` would answer `a::Y`;
    // `b` imports `a::Y as X`. rustc cannot read `a::Y` without `a`'s
    // renamed import, which has bound nothing in any name space, and cannot
    // determine either import (E0432); the index decides neither.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod a { pub use crate::b::X as Y; pub use crate::g::*; }",
            "pub mod b { pub use crate::a::Y as X; }",
            "pub mod g { pub struct Y; }",
        ],
        &[("demo::a::Y", None), ("demo::b::X", None)],
    );
    assert_eq!(orders, 6);
    assert_eq!(
        found,
        ["Cyclic(\"demo::a::Y\")", "Cyclic(\"demo::b::X\")", ""]
    );
}

#[test]
fn a_cycle_resolves_through_its_glob_when_a_place_an_import_keeps_also_waits_on_a_glob() {
    // `e::B`, `b::B` and `d::B` import one another in a cycle that `d`'s
    // glob of `c` answers with the function `c::B`; `a`'s renamed import
    // of `e::B` waits on it, and `e`'s glob of `a` may bring `a::B` back
    // into `e`. That loop runs through `e::B`, which `e`'s import binds once
    // the cycle resolves, so it ends with the cycle: every `B` is `c::B` in
    // the value name space, as rustc has them.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod a { pub struct A {} pub use crate::e::B as B; }",
            "pub mod e { pub use crate::b::B; pub use crate::a::*; pub struct A {} }",
            "pub mod c { pub use crate::a::*; pub const fn B() -> u32 { 3 } pub const fn A() -> u32 { 2 } }",
            "pub mod d { pub use crate::e::B; pub use crate::c::*; }",
            "pub mod b { pub use crate::d::B; }",
        ],
        &[
            ("demo::a::B", Some(Namespace::Value)),
            ("demo::b::B", Some(Namespace::Value)),
            ("demo::d::B", Some(Namespace::Value)),
            ("demo::e::B", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 120);
    assert_eq!(
        found,
        ["demo::c::B", "demo::c::B", "demo::c::B", "demo::c::B", ""]
    );
}

#[test]
fn an_import_through_an_import_that_keeps_its_name_keeps_its_own_from_the_globs_in_every_order() {
    // `m`'s import of `m2::N` reaches a module whose glob the index cannot
    // resolve, so it binds nothing and keeps `N` in the type name space.
    // `r`'s import of `m::N` binds the function `m::N` and, in the type name
    // space, reads that kept name: it keeps `r::N` from `r`'s glob of `g`
    // there too, and nothing answers it, whatever the order.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod m2 { pub use crate::nonexistent::*; }",
            "pub mod m { pub use crate::m2::N; pub fn N() {} }",
            "pub mod g { pub struct N {} }",
            "pub mod r { pub use crate::m::N; pub use crate::g::*; }",
        ],
        &[
            ("demo::r::N", Some(Namespace::Type)),
            ("demo::r::N", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 24);
    assert_eq!(found, ["NotFound(\"demo::r::N\")", "demo::m::N", ""]);
}

#[test]
fn a_cycle_through_a_glob_the_importer_cannot_access_is_no_cycle_in_every_order() {
    // `a` imports `b::B`, a function, and globs `c`; `b` globs `a` with a
    // private glob, which `a` cannot access. rustc resolves `a`'s import
    // without what `a` cannot access, so there is no cycle: `a::B` is the
    // function, and `c::B` in the type name space.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod c { pub struct B {} }",
            "pub mod a { pub use self::b::B; pub use crate::c::*; mod b { pub fn B() {} use super::*; } }",
        ],
        &[
            ("demo::a::B", Some(Namespace::Type)),
            ("demo::a::B", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 2);
    assert_eq!(found, ["demo::c::B", "demo::a::b::B", ""]);
    // The same with `b` beside `a` and its glob of `a` private to it.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod c { pub struct B {} }",
            "pub mod a { pub use crate::b::B; pub use crate::c::*; }",
            "pub mod b { pub fn B() {} use crate::a::*; }",
        ],
        &[
            ("demo::a::B", Some(Namespace::Type)),
            ("demo::a::B", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 6);
    assert_eq!(found, ["demo::c::B", "demo::b::B", ""]);
    // With `b`'s glob public, `a`'s import reads `b::B` back through it:
    // rustc cannot determine the import (E0432), and the index decides
    // nothing for it.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod c { pub struct B {} }",
            "pub mod a { pub use self::b::B; pub use crate::c::*; pub mod b { pub fn B() {} pub use super::*; } }",
        ],
        &[
            ("demo::a::B", Some(Namespace::Type)),
            ("demo::a::B", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 2);
    assert_eq!(
        found,
        ["Cyclic(\"demo::a::B\")", "Cyclic(\"demo::a::B\")", ""]
    );
}

#[test]
fn an_import_a_cycle_leaves_to_the_globs_binds_what_its_readers_could_access_in_every_order() {
    // `m2` and `m3` import `Y` from each other, and `m2`'s globs answer
    // `m2::Y`: its private glob of `m0` with the struct `m0::Y`, which has no
    // value, and its public glob of `m1` with the unit struct `m1::Y`, which
    // has both. The value name space holds `m1::Y` only, and both imports
    // bind it, for `m3`, outside `m2`, as for `m2`. The type name space
    // holds two structs: `m3` cannot access the glob of `m0`, but rustc keeps
    // the name ambiguous in `m2` for every reader, so `m2::Y` and `m3::Y` are
    // ambiguous there, and not `m1::Y`. rustc itself reads this crate in two
    // ways, by the order of the two globs in `m2` and by nothing else: with
    // the private glob first it accepts the crate and drops the type name
    // space of the imports (`m3::Y` names no type), and with the public glob
    // first it reports E0659. The index answers as rustc does wherever rustc
    // answers, in every order, and holds the type name space ambiguous.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod m3 { pub use crate::m2::Y; }",
            "pub mod m0 { pub struct Y {} }",
            "pub mod m2 { use crate::m0::*; pub use crate::m3::Y; pub use crate::m1::*; }",
            "pub mod m1 { pub struct Y; }",
        ],
        &[
            ("demo::m2::Y", Some(Namespace::Type)),
            ("demo::m2::Y", Some(Namespace::Value)),
            ("demo::m3::Y", Some(Namespace::Type)),
            ("demo::m3::Y", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 24);
    let both = "ambiguous [\"demo::m0::Y\", \"demo::m1::Y\"]";
    assert_eq!(found, [both, "demo::m1::Y", both, "demo::m1::Y", ""]);
}

#[test]
fn a_name_an_import_the_reader_cannot_access_binds_is_hidden_from_it_in_every_order() {
    // `c` imports `shared::inner as inn` privately and globs `d`, whose `inn`
    // is the crate `other`. `b`'s import of `c::inn` cannot access `c`'s
    // import, which hides the glob's `inn` all the same, so it binds nothing
    // whatever the order (rustc: E0603), and does not bind the glob's crate
    // before `c`'s import is decided.
    let (found, orders) = one_answer_in_every_order(
        beside_two,
        &[
            "pub mod c { use shared::inner as inn; pub use crate::d::*; }",
            "pub mod d { pub use other as inn; }",
            "pub mod b { pub use crate::c::inn; }",
        ],
        &[("demo::b::inn", None), ("demo::c::inn", None)],
    );
    assert_eq!(orders, 6);
    assert_eq!(found, ["NotFound(\"demo::b::inn\")", "shared::inner", ""]);
    // The program the order search found it in, in every rotation.
    let (found, orders) = one_answer_in_every_rotation_of(
        beside_two,
        &[
            "pub mod vals { pub fn shared() {} pub fn other() {} }",
            "pub mod k { pub mod other { pub fn x() {} pub trait T { fn m(&self); } } }",
            "pub mod c { pub use other::x; use shared::inner as inn; pub use crate::vals::other; pub use crate::d::*; }",
            "pub mod e {  }",
            "pub mod b { pub use crate::c::inn; pub use crate::vals::other; pub use shared::inner::other; pub use other::x; }",
            "pub mod d { pub use other as inn; impl shared::T for u16 { fn m(&self) {} } pub use crate::vals::shared; }",
            "pub mod a {  }",
        ],
        &[("demo::b::inn", None)],
    );
    assert_eq!(orders, 14);
    assert_eq!(found, ["NotFound(\"demo::b::inn\")"]);
}

#[test]
fn an_import_that_reads_a_name_only_through_a_glob_it_cannot_access_finds_nothing_in_every_order() {
    // `c` imports `a::A`, which only `a`'s private glob of `c` could bring
    // in: `c` cannot access that glob, so its import finds nothing and waits
    // on nothing (rustc: E0432 for it), and `e`'s import of `d::A`, which
    // `d`'s glob of `c` would bring in, finds nothing either.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod a { use crate::c::*; }",
            "pub mod e { pub use crate::d::A; }",
            "pub mod c { pub use crate::a::A; pub use crate::e::*; }",
            "pub mod d { pub use crate::c::*; }",
        ],
        &[
            ("demo::a::A", None),
            ("demo::c::A", None),
            ("demo::e::A", None),
        ],
    );
    assert_eq!(orders, 24);
    assert_eq!(
        found,
        [
            "NotFound(\"demo::a::A\")",
            "NotFound(\"demo::c::A\")",
            "NotFound(\"demo::e::A\")",
            ""
        ]
    );
}

#[test]
fn a_glob_the_importer_cannot_access_holds_nothing_back_in_every_order() {
    // `m` defines the function `N` and globs a path through the cycle of
    // `x` and `y` with a private glob, which stays unsettled. `f` cannot
    // access that glob, so its import of `m::N` binds the function and
    // leaves the type name space to `f`'s glob of `g`, as rustc does (which
    // refuses only the cycle, E0432).
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub mod x { pub use crate::y::Q; }",
            "pub mod y { pub use crate::x::Q; }",
            "pub mod m { pub fn N() {} use crate::x::Q::*; }",
            "pub mod g { pub struct N {} }",
            "pub mod f { pub use crate::m::N; pub use crate::g::*; }",
        ],
        &[
            ("demo::f::N", Some(Namespace::Type)),
            ("demo::f::N", Some(Namespace::Value)),
        ],
    );
    assert_eq!(orders, 120);
    assert_eq!(found, ["demo::g::N", "demo::m::N", ""]);
}

#[test]
fn cycles_through_private_globs_resolve_as_each_reader_can_access_them_in_every_order() {
    // Two programs rustc accepts. In the first, `a`'s import of `d::B` and
    // `d`'s import of `a::B` wait on each other; `a`'s private glob of `b`
    // would answer `a::B`, but not for `d`, and `d`'s public glob of `b`
    // answers `d::B`: the cycle resolves through that one glob, to `b::B`.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub trait Id { const ID: u32; }",
            "pub mod b { pub struct B {} impl crate::Id for B { const ID: u32 = 4; } pub const fn A() -> u32 { 3 } }",
            "pub mod e { use crate::b::*; }",
            "pub mod a { pub struct A {} impl crate::Id for A { const ID: u32 = 1; } pub const fn A() -> u32 { 2 } use crate::b::*; pub use crate::d::B as B; }",
            "pub mod d { use crate::a::*; pub use crate::b::*; pub use crate::a::B; }",
        ],
        &[
            ("demo::a::B", Some(Namespace::Type)),
            ("demo::d::B", Some(Namespace::Type)),
        ],
    );
    assert_eq!(orders, 120);
    assert_eq!(found, ["demo::b::B", "demo::b::B", ""]);
    // In the second, `d`'s and `b`'s imports of `e::A` read the struct; in
    // the value name space `e`'s glob of `d` brings in only what `e` can
    // access of `d`, not what `d`'s private globs of `b` and `c` bring in,
    // so neither import waits on the other there.
    let (found, orders) = one_answer_in_every_order(
        lib,
        &[
            "pub trait Id { const ID: u32; }",
            "pub mod b { pub use crate::e::A; pub const fn B() -> u32 { 1 } }",
            "pub mod c {  }",
            "pub mod d { use crate::c::*; use crate::b::*; pub use crate::e::*; pub use crate::e::A as A; }",
            "pub mod e { pub use crate::d::*; pub struct A {} impl crate::Id for A { const ID: u32 = 2; } }",
        ],
        &[
            ("demo::b::A", Some(Namespace::Type)),
            ("demo::d::A", Some(Namespace::Type)),
        ],
    );
    assert_eq!(orders, 120);
    assert_eq!(found, ["demo::e::A", "demo::e::A", ""]);
}

/// The orders of `blocks` that start at each of them, forward and backward.
fn rotations<'a>(blocks: &[&'a str]) -> Vec<Vec<&'a str>> {
    let mut orders = Vec::new();
    for start in 0..blocks.len() {
        let order: Vec<&str> = blocks[start..]
            .iter()
            .chain(&blocks[..start])
            .copied()
            .collect();
        let mut backward = order.clone();
        backward.reverse();
        orders.push(order);
        orders.push(backward);
    }
    orders
}

#[test]
fn a_cycle_left_to_a_glob_reads_all_that_the_glob_brings_in_once_its_round_is_decided() {
    // `m5` and `m21` import `Y` from each other, and the cycle is left to the
    // glob of `m21`. Through it `m25` brings in `m58::Y` at once and `m33::Y`
    // once `m38`'s import of `Y`, which binds a module, is decided to bind
    // nothing in the value name space, in the round that leaves the cycle to
    // the glob. Both imports read both functions: rustc reports E0659 for
    // each.
    let blocks = [
        "pub mod m5 { pub use crate::m21::Y as Y; }",
        "pub mod m38 { pub use crate::m50::*; pub use crate::m43::X as Y; }",
        "pub mod m59 {}",
        "pub mod m31 { pub use crate::m49::*; }",
        "pub mod m11 { pub use crate::m38::*; }",
        "pub mod m49 { pub use crate::m11::*; }",
        "pub mod m25 { pub use crate::m31::*; pub use crate::m58::*; }",
        "pub mod m21 { pub use crate::m47::*; pub use crate::m5::Y as Y; }",
        "pub mod m23 { pub use crate::m25::*; }",
        "pub mod m33 { pub fn Y() {} }",
        "pub mod m47 { pub use crate::m23::*; }",
        "pub mod m50 { pub use crate::m33::*; }",
        "pub mod m58 { pub fn Y() {} }",
        "pub mod m43 { pub mod X { use super::*; } pub use crate::m59::*; }",
    ];
    let orders = rotations(&blocks);
    for order in &orders {
        let ws = lib(&order.join("\n"));
        for path in ["demo::m5::Y", "demo::m21::Y", "demo::m25::Y"] {
            assert_eq!(
                answer(&ws, path, Some(Namespace::Value)),
                "ambiguous [\"demo::m33::Y\", \"demo::m58::Y\"]",
                "{path} in {order:?}"
            );
        }
        assert_eq!(
            answer(&ws, "demo::m38::Y", Some(Namespace::Value)),
            "demo::m33::Y",
            "{order:?}"
        );
    }
    assert_eq!(orders.len(), 28);
}

#[test]
fn imports_that_read_a_name_through_a_glob_agree_with_the_glob_on_an_ambiguous_re_export() {
    // `m58` re-exports `Y` ambiguously, `m46::Y` and `m0::Y`, which rustc
    // only warns about (`ambiguous_glob_reexports`); `m41` globs it and
    // `m28`, and `m46` imports `m41::Y`. The cycle of `m1` and `m49` is left
    // to the glob of `m49`, which reads `m46::Y`: it stays ambiguous, as
    // what `m43` and `m46` bring in is, whatever the order. rustc's own
    // answer here depends on the order of the declarations: in the orders at
    // even places of `rotations` it compiles, binding `m28::Y`, and in those at
    // odd places it reports E0659. `Ambiguous` in every order is deliberate:
    // it agrees with rustc wherever rustc reports an error and does not
    // depend on the order, so it is not to be changed to match one order.
    let blocks = [
        "pub mod m38 { pub use crate::m15::*; }",
        "pub mod m58 { pub use crate::m46::*; pub use crate::m42::*; }",
        "pub mod m43 { pub use crate::m46::*; }",
        "pub mod m46 { pub use crate::m54::*; pub use crate::m41::Y; }",
        "pub mod m9 { pub use crate::m0::*; }",
        "pub mod m0 { pub use crate::m41::*; pub fn Y() {} }",
        "pub mod m54 { pub use crate::m9::*; }",
        "pub mod m28 { pub fn Y() {} }",
        "pub mod m30 { pub use crate::m9::*; pub use crate::m47::Y as Y; }",
        "pub mod m4 { pub use crate::m38::*; }",
        "pub mod m41 { pub use crate::m11::*; pub use crate::m28::*; }",
        "pub mod m47 { use crate::m4::*; pub use crate::m4::Y as Y; }",
        "pub mod m49 { pub use crate::m1::Y; pub use crate::m43::*; }",
        "pub mod m27 { pub use crate::m58::*; }",
        "pub mod m1 { pub use crate::m49::Y; }",
        "pub mod m42 { pub use crate::m30::*; }",
        "pub mod m15 { pub mod Y { use super::*; } }",
        "pub mod m11 { pub use crate::m27::*; }",
    ];
    let orders = rotations(&blocks);
    for order in &orders {
        let ws = lib(&order.join("\n"));
        for path in [
            "demo::m1::Y",
            "demo::m49::Y",
            "demo::m43::Y",
            "demo::m46::Y",
        ] {
            assert_eq!(
                answer(&ws, path, Some(Namespace::Value)),
                "ambiguous [\"demo::m0::Y\", \"demo::m28::Y\"]",
                "{path} in {order:?}"
            );
        }
        assert_eq!(
            answer(&ws, "demo::m42::Y", Some(Namespace::Value)),
            "demo::m0::Y",
            "{order:?}"
        );
    }
    assert_eq!(orders.len(), 36);
}

/// A two-package workspace whose `p-two` library is `text` and depends on
/// `p-one`, which defines `X` and `inner::go`.
fn beside_p_one(text: &str) -> Result<Workspace, LoadError> {
    let tree: MemoryTree = [
        ("Cargo.toml", "[workspace]\nmembers = [\"one\", \"two\"]\n"),
        (
            "one/Cargo.toml",
            "[package]\nname = \"p-one\"\nedition = \"2021\"\n",
        ),
        ("one/src/lib.rs", "pub struct X;\npub mod inner { pub fn go() {} }\n"),
        (
            "two/Cargo.toml",
            "[package]\nname = \"p-two\"\nedition = \"2021\"\n\n[dependencies]\np-one = { path = \"../one\" }\n",
        ),
        ("two/src/lib.rs", text),
    ]
    .into_iter()
    .collect();
    Workspace::load(&tree, &["."])
}

#[test]
fn a_use_group_with_leading_colon_members_binds_as_the_separate_uses_do() {
    // syn has no node for a group member that starts with `::`; rustc accepts
    // it, and each member binds as its own `use` would.
    let grouped = "pub mod a { pub fn f() {} pub fn g() {} }\n\
                   #[allow(unused_imports)]\n\
                   pub use {\n    ::p_one::X as D,\n    crate::a::f,\n    ::p_one::inner::{self, go as run},\n    { ::std::fmt::Debug as Dbg },\n};\n\
                   pub mod m { pub use { ::p_one::X, super::a::g as h }; }\n";
    let separate = "pub mod a { pub fn f() {} pub fn g() {} }\n\
                    pub use ::p_one::X as D;\npub use crate::a::f;\n\
                    pub use ::p_one::inner::{self, go as run};\npub use ::std::fmt::Debug as Dbg;\n\
                    pub mod m { pub use ::p_one::X; pub use super::a::g as h; }\n";
    let grouped = beside_p_one(grouped).unwrap_or_else(|e| panic!("{e}"));
    let separate = beside_p_one(separate).unwrap_or_else(|e| panic!("{e}"));
    for (path, expected) in [
        ("p_two::D", "p_one::X"),
        ("p_two::f", "p_two::a::f"),
        ("p_two::inner::go", "p_one::inner::go"),
        ("p_two::run", "p_one::inner::go"),
        ("p_two::m::X", "p_one::X"),
        ("p_two::m::h", "p_two::a::g"),
    ] {
        assert_eq!(path_of(&grouped, path), expected, "{path}");
        assert_eq!(path_of(&separate, path), expected, "{path}");
    }
    // A member that names a crate outside the workspace is not found, as
    // the separate `use` is not.
    not_found(&grouped, "p_two::Dbg");
    not_found(&separate, "p_two::Dbg");
    not_found(&grouped, "p_two::Missing");
}

#[test]
fn a_leading_colon_member_of_a_use_group_reads_the_crate_root_in_edition_2015() {
    let ws = load_in(
        "2015",
        &[(
            "src/lib.rs",
            "pub mod inner { pub fn go() {} }\n\
             pub mod user { pub use { ::inner::go as colon, super::inner }; }\n",
        )],
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(path_of(&ws, "demo::user::colon"), "demo::inner::go");
    assert_eq!(path_of(&ws, "demo::user::inner::go"), "demo::inner::go");
}

#[test]
fn a_crate_root_inside_a_group_under_a_path_is_a_syntax_error() {
    // rustc refuses `a::{::b}` too (E0433), and syn does not parse it; the
    // load names the file and line.
    let error = load(&[(
        "src/lib.rs",
        "pub mod a { pub fn b() {} }\npub use { ::std::fmt, a::{::b} };\n",
    )])
    .expect_err("a crate root under a path");
    assert!(
        matches!(error.kind, LoadErrorKind::Syntax { .. }),
        "{error}"
    );
    assert_eq!((error.file.as_str(), error.line), ("src/lib.rs", Some(2)));
}

/// What `module` reads `path` as in `ns` when it writes it in a `use`: the
/// canonical path, or the kind of refusal.
fn read_as(ws: &Workspace, module: &str, path: &str, ns: Namespace) -> String {
    match ws.resolve_from_in(module, path, ns) {
        Ok(found) => found.path,
        Err(ResolveError::NotFound(_)) => "not found".to_owned(),
        Err(ResolveError::Ambiguous(ids)) => format!(
            "ambiguous {:?}",
            ids.iter()
                .map(|id| ws.item(*id).expect("item").path.to_string())
                .collect::<Vec<_>>()
        ),
        Err(other) => format!("{other:?}"),
    }
}

/// A question to a reader: the module, the `use` path and the name space.
type Read = (&'static str, &'static str, Namespace);

/// Load the crate in every order of its top-level modules, `others` and the
/// `modules` of `(header, declarations)`, with the declarations inside each
/// of those modules in every order too, and assert that every reader's
/// answer to `reads` is the same in each. Returns the answers and the number
/// of loads.
fn reads_in_every_order(
    others: &[&str],
    modules: &[(&str, &[&str])],
    reads: &[Read],
) -> (Vec<String>, usize) {
    let mut expected: Option<Vec<String>> = None;
    let mut loads = 0;
    let insides: Vec<Vec<Vec<String>>> = modules
        .iter()
        .map(|(_, inner)| permutations(&inner.iter().map(|d| (*d).to_owned()).collect::<Vec<_>>()))
        .collect();
    let mut choice = vec![0; modules.len()];
    loop {
        let mut blocks: Vec<String> = others.iter().map(|b| (*b).to_owned()).collect();
        for (at, (header, _)) in modules.iter().enumerate() {
            blocks.push(format!(
                "{header} {{ {} }}",
                insides[at][choice[at]].join(" ")
            ));
        }
        for order in permutations(&blocks) {
            let ws = lib(&order.join("\n"));
            let results: Vec<String> = reads
                .iter()
                .map(|(module, path, ns)| read_as(&ws, module, path, *ns))
                .collect();
            match &expected {
                None => expected = Some(results),
                Some(first) => assert_eq!(&results, first, "{order:?}"),
            }
            loads += 1;
        }
        // The next combination of orders inside the modules.
        let mut at = 0;
        while at < choice.len() {
            choice[at] += 1;
            if choice[at] < insides[at].len() {
                break;
            }
            choice[at] = 0;
            at += 1;
        }
        if at == choice.len() {
            break;
        }
    }
    (expected.expect("loaded"), loads)
}

const TYPE: Namespace = Namespace::Type;
const VALUE: Namespace = Namespace::Value;

#[test]
fn a_pub_use_of_a_name_read_through_a_private_glob_is_seen_no_wider_than_the_glob_in_every_order() {
    // `m` reads the struct `A` of `x` through a private glob and defines the
    // function `A`. `c` re-exports `super::A`: rustc keeps the struct `A` of
    // `c` visible within `m` only, and the function `pub`. `r` globs `c`, so
    // it reads the function and not the struct, and `m` reads both through
    // `c`. In `m` the one name `A` stands in both name spaces: for the struct
    // through the glob, and for the function it defines.
    let reads: &[Read] = &[
        ("demo::r", "self::A", TYPE),
        ("demo::r", "self::A", VALUE),
        ("demo::r", "crate::m::c::A", TYPE),
        ("demo::r", "crate::m::c::A", VALUE),
        ("demo::m", "self::c::A", TYPE),
        ("demo::m", "self::c::A", VALUE),
        ("demo::m::c", "self::A", TYPE),
        ("demo::m::c", "self::A", VALUE),
    ];
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod x { pub struct A {} }",
            "pub mod r { pub use crate::m::c::*; }",
        ],
        &[(
            "mod m",
            &[
                "use crate::x::*;",
                "pub fn A() {}",
                "pub mod c { pub use super::A; }",
            ],
        )],
        reads,
    );
    assert_eq!(
        found,
        [
            "not found",
            "demo::m::A",
            "not found",
            "demo::m::A",
            "demo::x::A",
            "demo::m::A",
            "demo::x::A",
            "demo::m::A",
        ]
    );
    assert_eq!(loads, 36);

    // Neighbours: the same re-export of a binding the glob makes accessible
    // as widely as the re-export stays as visible as it, for a `pub` glob and
    // for a `pub(crate)` glob read from inside the crate.
    for glob in ["pub use crate::x::*;", "pub(crate) use crate::x::*;"] {
        let (found, loads) = reads_in_every_order(
            &[
                "pub mod x { pub struct A {} }",
                "pub mod r { pub use crate::m::c::*; }",
            ],
            &[(
                "mod m",
                &[glob, "pub fn A() {}", "pub mod c { pub use super::A; }"],
            )],
            reads,
        );
        assert_eq!(
            found,
            [
                "demo::x::A",
                "demo::m::A",
                "demo::x::A",
                "demo::m::A",
                "demo::x::A",
                "demo::m::A",
                "demo::x::A",
                "demo::m::A",
            ],
            "{glob}"
        );
        assert_eq!(loads, 36);
    }
}

#[test]
fn a_chain_of_two_re_exports_through_two_private_globs_is_capped_twice_in_every_order() {
    // `c1` re-exports what `m1` reads of `x` through a private glob; `m2`
    // reads `c1` through a private glob and defines the struct `A`, so `c2`
    // re-exports its own struct, which is `pub`, and the function `m2`
    // reads through the glob, which is visible within `m2` only. `r` reads
    // the struct and not the function.
    let reads: &[Read] = &[
        ("demo::r", "self::A", TYPE),
        ("demo::r", "self::A", VALUE),
        ("demo::m2", "self::c2::A", TYPE),
        ("demo::m2", "self::c2::A", VALUE),
        ("demo::m2", "self::A", TYPE),
        ("demo::m2", "self::A", VALUE),
        ("demo::m1", "self::c1::A", TYPE),
        ("demo::m1", "self::c1::A", VALUE),
        ("demo::m2::c2", "self::A", VALUE),
    ];
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod x { pub struct A {} }",
            "pub mod r { pub use crate::m2::c2::*; }",
        ],
        &[
            (
                "mod m1",
                &[
                    "use crate::x::*;",
                    "pub fn A() {}",
                    "pub mod c1 { pub use super::A; }",
                ],
            ),
            (
                "mod m2",
                &[
                    "use crate::m1::c1::*;",
                    "pub struct A {}",
                    "pub mod c2 { pub use super::A; }",
                ],
            ),
        ],
        reads,
    );
    assert_eq!(
        found,
        [
            "demo::m2::A",
            "not found",
            "demo::m2::A",
            "demo::m1::A",
            "demo::m2::A",
            "demo::m1::A",
            "demo::x::A",
            "demo::m1::A",
            "demo::m1::A",
        ]
    );
    assert_eq!(loads, 864);
}

#[test]
fn an_item_reached_by_a_capped_re_export_and_by_a_wider_glob_is_seen_as_widely_as_the_wider_in_every_order()
 {
    // `n` reads the struct `A` through `c`'s re-export, visible within `m`,
    // and through a glob of `x` of its own. With a `pub(crate)` glob the item
    // is visible in the crate, the wider of the two, whichever is met first;
    // with a private glob, the wider is `m`, and a reader outside `m` finds
    // no struct. The function reaches `n` through `c` only and is `pub`.
    let reads: &[Read] = &[
        ("demo::r", "self::A", TYPE),
        ("demo::r", "self::A", VALUE),
        ("demo::m::n", "self::A", TYPE),
        ("demo::m", "self::n::A", TYPE),
    ];
    let wide = [
        "pub mod n { pub use super::c::*; pub(crate) use crate::x::*; }",
        "pub mod n { pub(crate) use crate::x::*; pub use super::c::*; }",
    ];
    for n in wide {
        let (found, loads) = reads_in_every_order(
            &[
                "pub mod x { pub struct A {} }",
                "pub mod r { pub use crate::m::n::*; }",
            ],
            &[(
                "mod m",
                &[
                    "use crate::x::*;",
                    "pub fn A() {}",
                    "pub mod c { pub use super::A; }",
                    n,
                ],
            )],
            reads,
        );
        assert_eq!(
            found,
            ["demo::x::A", "demo::m::A", "demo::x::A", "demo::x::A"],
            "{n}"
        );
        assert_eq!(loads, 144);
    }
    let narrow = [
        "pub mod n { pub use super::c::*; use crate::x::*; }",
        "pub mod n { use crate::x::*; pub use super::c::*; }",
    ];
    for n in narrow {
        let (found, loads) = reads_in_every_order(
            &[
                "pub mod x { pub struct A {} }",
                "pub mod r { pub use crate::m::n::*; }",
            ],
            &[(
                "mod m",
                &[
                    "use crate::x::*;",
                    "pub fn A() {}",
                    "pub mod c { pub use super::A; }",
                    n,
                ],
            )],
            reads,
        );
        assert_eq!(
            found,
            ["not found", "demo::m::A", "demo::x::A", "demo::x::A"],
            "{n}"
        );
        assert_eq!(loads, 144);
    }
}

#[test]
fn a_glob_re_export_of_a_private_item_is_seen_no_wider_than_the_item_in_every_order() {
    // `s0` globs its parent, whose private struct `B` it may read; its glob
    // is `pub` and the binding it makes is visible within `m0` only, so a
    // glob of `s0` from outside `m0` brings in nothing, and a glob from
    // inside brings in the struct.
    let reads: &[Read] = &[
        ("demo", "self::m2::B", TYPE),
        ("demo::m0::s0", "self::B", TYPE),
        ("demo::m0::s1", "crate::m0::s0::B", TYPE),
        ("demo::m0::s1", "self::B", TYPE),
    ];
    let (found, loads) = reads_in_every_order(
        &["pub mod m2 { pub use crate::m0::s0::*; }"],
        &[(
            "pub mod m0",
            &[
                "pub mod s0 { pub use super::*; }",
                "pub mod s1 { pub use super::s0::*; }",
                "struct B {}",
            ],
        )],
        reads,
    );
    assert_eq!(
        found,
        ["not found", "demo::m0::B", "demo::m0::B", "demo::m0::B"]
    );
    assert_eq!(loads, 12);
}

/// The answer of a reader who finds the name ambiguous between `first` and
/// `second`.
fn ambiguous_between(first: &str, second: &str) -> String {
    format!("ambiguous [\"{first}\", \"{second}\"]")
}

#[test]
fn a_name_two_globs_bring_in_as_different_items_is_ambiguous_to_a_reader_of_one_glob_in_every_order()
 {
    // `m` globs `a` privately and `b` with a glob the readers can access, and
    // both modules hold a struct `Y`. rustc keeps the name ambiguous in `m`
    // whichever glob a reader can access (E0659 for each of the probes
    // below, in every order of the declarations of the crate and of `m`),
    // and refuses it to a reader that cannot access the private glob as
    // much as to one that can: the index does not answer with `b::Y`.
    let reads: &[Read] = &[
        ("demo", "crate::m::Y", TYPE),
        ("demo", "crate::m::Y", VALUE),
        ("demo::s", "crate::m::Y", TYPE),
        ("demo::m", "self::Y", TYPE),
        ("demo::m::c", "super::Y", TYPE),
        ("demo::m::c", "super::Y", VALUE),
    ];
    let both = ambiguous_between("demo::a::Y", "demo::b::Y");
    for glob in ["pub use crate::b::*;", "pub(crate) use crate::b::*;"] {
        let (found, loads) = reads_in_every_order(
            &[
                "pub mod a { pub struct Y; }",
                "pub mod b { pub struct Y; }",
                "pub mod s {}",
            ],
            &[("mod m", &["use crate::a::*;", glob, "pub mod c {}"])],
            reads,
        );
        assert_eq!(found, vec![both.clone(); reads.len()], "{glob}");
        assert_eq!(loads, 144);
    }
    // The question as the public entry point asks it: ambiguous, with both
    // items, for a reader outside `m` that cannot access the private glob.
    let ws = lib("pub mod a { pub struct Y; }\npub mod b { pub struct Y; }\n\
         mod m { use crate::a::*; pub use crate::b::*; }\n");
    match ws.resolve_from("demo", "crate::m::Y") {
        Err(ResolveError::Ambiguous(ids)) => {
            let mut paths: Vec<String> = ids
                .iter()
                .map(|id| ws.item(*id).expect("item").path.to_string())
                .collect();
            paths.sort();
            assert_eq!(paths, ["demo::a::Y", "demo::b::Y"]);
        }
        other => panic!("{other:?}"),
    }
    // Neighbours. With both globs private the name is ambiguous inside `m`
    // and found nowhere outside it (rustc refuses both reads); with both
    // public it is ambiguous everywhere.
    let (found, loads) = reads_in_every_order(
        &["pub mod a { pub struct Y; }", "pub mod b { pub struct Y; }"],
        &[(
            "mod m",
            &["use crate::a::*;", "use crate::b::*;", "pub mod c {}"],
        )],
        &[
            ("demo", "crate::m::Y", TYPE),
            ("demo::m", "self::Y", TYPE),
            ("demo::m::c", "super::Y", TYPE),
        ],
    );
    assert_eq!(found, ["not found".to_owned(), both.clone(), both.clone()]);
    assert_eq!(loads, 36);
    let (found, loads) = reads_in_every_order(
        &["pub mod a { pub struct Y; }", "pub mod b { pub struct Y; }"],
        &[(
            "mod m",
            &[
                "pub use crate::a::*;",
                "pub use crate::b::*;",
                "pub mod c {}",
            ],
        )],
        &[
            ("demo", "crate::m::Y", TYPE),
            ("demo::m::c", "super::Y", TYPE),
        ],
    );
    assert_eq!(found, [both.clone(), both]);
    assert_eq!(loads, 36);
}

#[test]
fn a_module_and_a_struct_of_one_name_through_two_globs_are_ambiguous_in_the_type_name_space_in_every_order()
 {
    // The shape rustc reports as E0573 beside E0659 in a type position: the
    // private glob brings in a module `Y`, the public one a unit struct `Y`.
    // The type name space is ambiguous; the value name space holds the
    // struct only, which rustc resolves for a reader of the public glob.
    let (found, loads) = reads_in_every_order(
        &["pub mod a { pub mod Y {} }", "pub mod b { pub struct Y; }"],
        &[(
            "mod m",
            &["use crate::a::*;", "pub use crate::b::*;", "pub mod c {}"],
        )],
        &[
            ("demo", "crate::m::Y", TYPE),
            ("demo", "crate::m::Y", VALUE),
            ("demo::m", "self::Y", TYPE),
            ("demo::m::c", "super::Y", TYPE),
            ("demo::m::c", "super::Y", VALUE),
        ],
    );
    let both = ambiguous_between("demo::a::Y", "demo::b::Y");
    assert_eq!(
        found,
        [
            both.clone(),
            "demo::b::Y".to_owned(),
            both.clone(),
            both,
            "demo::b::Y".to_owned()
        ]
    );
    assert_eq!(loads, 36);
}

#[test]
fn the_refusal_of_an_ambiguous_name_in_one_name_space_leaves_the_other_resolved_in_every_order() {
    // A struct `Y` through the private glob and another through the public
    // one are ambiguous in the type name space, and the function `Y` of a
    // third public glob is the only value: rustc refuses the type and
    // resolves the value, for readers outside `m` and inside it.
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod a { pub struct Y {} }",
            "pub mod b { pub struct Y {} }",
            "pub mod d { pub fn Y() {} }",
        ],
        &[(
            "mod m",
            &[
                "use crate::a::*;",
                "pub use crate::b::*;",
                "pub use crate::d::*;",
            ],
        )],
        &[
            ("demo", "crate::m::Y", TYPE),
            ("demo", "crate::m::Y", VALUE),
            ("demo::m", "self::Y", TYPE),
            ("demo::m", "self::Y", VALUE),
        ],
    );
    assert_eq!(
        found,
        [
            ambiguous_between("demo::a::Y", "demo::b::Y"),
            "demo::d::Y".to_owned(),
            ambiguous_between("demo::a::Y", "demo::b::Y"),
            "demo::d::Y".to_owned(),
        ]
    );
    assert_eq!(loads, 144);
    // The other way: two functions are ambiguous and the struct resolves.
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod a { pub fn Y() {} }",
            "pub mod b { pub fn Y() {} }",
            "pub mod d { pub struct Y {} }",
        ],
        &[(
            "mod m",
            &[
                "use crate::a::*;",
                "pub use crate::b::*;",
                "pub use crate::d::*;",
            ],
        )],
        &[
            ("demo", "crate::m::Y", TYPE),
            ("demo", "crate::m::Y", VALUE),
            ("demo::m", "self::Y", TYPE),
            ("demo::m", "self::Y", VALUE),
        ],
    );
    assert_eq!(
        found,
        [
            "demo::d::Y".to_owned(),
            ambiguous_between("demo::a::Y", "demo::b::Y"),
            "demo::d::Y".to_owned(),
            ambiguous_between("demo::a::Y", "demo::b::Y"),
        ]
    );
    assert_eq!(loads, 144);
}

#[test]
fn names_the_private_glob_does_not_make_ambiguous_still_resolve_for_a_reader_outside_in_every_order()
 {
    // One item reached through both globs is one candidate.
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod a { pub struct Y; }",
            "pub mod b { pub use crate::a::Y; }",
        ],
        &[(
            "mod m",
            &["use crate::a::*;", "pub use crate::b::*;", "pub mod c {}"],
        )],
        &[
            ("demo", "crate::m::Y", TYPE),
            ("demo", "crate::m::Y", VALUE),
            ("demo::m::c", "super::Y", TYPE),
        ],
    );
    assert_eq!(found, vec!["demo::a::Y"; 3]);
    assert_eq!(loads, 36);
    // A function and a struct of one name are two name spaces: the reader
    // outside finds the struct of the public glob, and the function of the
    // private one only inside.
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod a { pub fn Y() {} }",
            "pub mod b { pub struct Y {} }",
        ],
        &[(
            "mod m",
            &["use crate::a::*;", "pub use crate::b::*;", "pub mod c {}"],
        )],
        &[
            ("demo", "crate::m::Y", TYPE),
            ("demo", "crate::m::Y", VALUE),
            ("demo::m::c", "super::Y", TYPE),
            ("demo::m::c", "super::Y", VALUE),
        ],
    );
    assert_eq!(
        found,
        ["demo::b::Y", "not found", "demo::b::Y", "demo::a::Y"]
    );
    assert_eq!(loads, 36);
    // A definition of `m` shadows both globs, and so does an explicit
    // import.
    let (found, loads) = reads_in_every_order(
        &["pub mod a { pub struct Y; }", "pub mod b { pub struct Y; }"],
        &[(
            "mod m",
            &[
                "use crate::a::*;",
                "pub use crate::b::*;",
                "pub struct Y;",
                "pub mod c {}",
            ],
        )],
        &[
            ("demo", "crate::m::Y", TYPE),
            ("demo", "crate::m::Y", VALUE),
            ("demo::m::c", "super::Y", TYPE),
        ],
    );
    assert_eq!(found, vec!["demo::m::Y"; 3]);
    assert_eq!(loads, 144);
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod a { pub struct Y; }",
            "pub mod b { pub struct Y; }",
            "pub mod e { pub struct Y; }",
        ],
        &[(
            "mod m",
            &[
                "use crate::a::*;",
                "pub use crate::b::*;",
                "pub use crate::e::Y;",
            ],
        )],
        &[
            ("demo", "crate::m::Y", TYPE),
            ("demo", "crate::m::Y", VALUE),
        ],
    );
    assert_eq!(found, vec!["demo::e::Y"; 2]);
    assert_eq!(loads, 144);
}

#[test]
fn a_glob_of_a_module_whose_name_is_ambiguous_carries_the_ambiguity_to_its_readers_in_every_order()
{
    // `r` globs `m`, where the private glob of `a` and the public glob of
    // `b` bring in different structs `Y`. The name is ambiguous in `r`
    // as in `m` (rustc refuses each read in every order), for a reader that
    // cannot access the private glob of `m` as well as for `r` itself; a
    // reader that cannot access the glob of `r` finds nothing there.
    let both = ambiguous_between("demo::a::Y", "demo::b::Y");
    for (glob, outside) in [
        ("pub use crate::m::*;", both.as_str()),
        ("pub(crate) use crate::m::*;", both.as_str()),
        ("use crate::m::*;", "not found"),
    ] {
        let (found, loads) = reads_in_every_order(
            &[
                "pub mod a { pub struct Y; }",
                "pub mod b { pub struct Y; }",
                &format!("pub mod r {{ {glob} }}"),
            ],
            &[("mod m", &["use crate::a::*;", "pub use crate::b::*;"])],
            &[
                ("demo", "crate::r::Y", TYPE),
                ("demo", "crate::r::Y", VALUE),
                ("demo::r", "self::Y", TYPE),
            ],
        );
        assert_eq!(found, [outside, outside, both.as_str()], "{glob}");
        assert_eq!(loads, 48);
    }
}

#[test]
fn an_import_a_reader_cannot_name_still_keeps_its_name_from_the_globs_in_every_order() {
    // `m0` globs `m2`, whose functions `C` and `A` it would pass on, and
    // imports `m2::C` as `A`. The import binds `A` in `m0` whoever can name
    // it: a reader that cannot (rustc: E0603 for `m0::A`, and nothing in the
    // glob of `m0` that `m1` makes) does not get the glob's `A` instead, and
    // the glob does not carry it before the import has bound it: `m1`'s
    // import of `self::A` binds nothing (E0432). With a `pub` import every
    // reader names it.
    let reads: &[Read] = &[
        ("demo::m1", "self::A", VALUE),
        ("demo", "crate::m1::A", VALUE),
        ("demo", "crate::m0::A", VALUE),
        ("demo::m0", "self::A", VALUE),
        ("demo", "crate::m1::seen", VALUE),
    ];
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod m1 { pub use crate::m0::*; pub use self::A as seen; }",
            "pub mod m2 { pub fn C() {} pub fn A() {} }",
        ],
        &[(
            "mod m0",
            &["pub use crate::m2::*;", "use crate::m2::C as A;"],
        )],
        reads,
    );
    assert_eq!(
        found,
        [
            "not found",
            "not found",
            "not found",
            "demo::m2::C",
            "not found"
        ]
    );
    assert_eq!(loads, 12);
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod m1 { pub use crate::m0::*; pub use self::A as seen; }",
            "pub mod m2 { pub fn C() {} pub fn A() {} }",
        ],
        &[(
            "pub mod m0",
            &["pub use crate::m2::*;", "pub use crate::m2::C as A;"],
        )],
        reads,
    );
    assert_eq!(found, vec!["demo::m2::C"; 5]);
    assert_eq!(loads, 12);
}

#[test]
fn an_import_of_a_value_a_reader_cannot_name_leaves_the_type_of_its_name_to_the_globs_in_every_order()
 {
    // `s1` imports the function `m2::C` privately and globs `m0`, which
    // brings in the struct `m1::C`; `s0` globs `s1` and `m2`, and `s1`
    // re-exports `s0::C` as `B`, which `m2` imports. The private import binds
    // `C` in the value name space only, so once it is settled the glob's
    // struct reaches `s0`, which cannot name the import: `s0::C` is the
    // struct and the function, and so are `s1::B` and `m2::B`, never the
    // struct `m1::B` that the glob of `s1` brings in under `B`. rustc accepts
    // each read below, and refuses `B` at the root (E0432), in every order.
    // Neighbour: where `m2` defines a struct `C` too, the private import
    // binds it in both name spaces, keeps the glob's struct from `s0`, and
    // `s0` reads `m2`'s struct through its other glob.
    let reads: &[Read] = &[
        ("demo::m2", "self::B", TYPE),
        ("demo::m2", "self::B", VALUE),
        ("demo::m0::s1", "self::B", TYPE),
        ("demo::m0::s1", "self::C", TYPE),
        ("demo::m0::s1", "self::C", VALUE),
        ("demo::m0::s0", "self::C", TYPE),
        ("demo", "self::B", TYPE),
        ("demo", "self::C", TYPE),
        ("demo", "self::C", VALUE),
    ];
    let s0 = ["pub use crate::m0::s1::*;", "pub use crate::m2::*;"];
    let s1 = [
        "use crate::m2::C;",
        "pub(crate) use super::*;",
        "pub use crate::m0::s0::C as B;",
    ];
    // Both orders of `s0` and each declaration of `s1` first, in the first
    // shape; the neighbour in the written order of the two.
    let inner = |all: bool| {
        let mut orders = vec![(s0, s1)];
        if all {
            for (s0, s1) in [
                (s0, [s1[1], s1[2], s1[0]]),
                (s0, [s1[2], s1[0], s1[1]]),
                ([s0[1], s0[0]], s1),
                ([s0[1], s0[0]], [s1[1], s1[2], s1[0]]),
                ([s0[1], s0[0]], [s1[2], s1[0], s1[1]]),
            ] {
                orders.push((s0, s1));
            }
        }
        orders
    };
    for (all, m2, expected) in [
        (
            true,
            &["use crate::m0::s1::B;", "pub fn C() {}"][..],
            [
                "demo::m1::C",
                "demo::m2::C",
                "demo::m1::C",
                "demo::m1::C",
                "demo::m2::C",
                "demo::m1::C",
                "not found",
                "not found",
                "demo::m2::C",
            ],
        ),
        (
            false,
            &["use crate::m0::s1::B;", "pub fn C() {}", "pub struct C {}"][..],
            [
                "demo::m2::C",
                "demo::m2::C",
                "demo::m2::C",
                "demo::m2::C",
                "demo::m2::C",
                "demo::m2::C",
                "not found",
                "demo::m2::C",
                "demo::m2::C",
            ],
        ),
    ] {
        for (s0, s1) in inner(all) {
            let m0 = [
                "pub use crate::m1::*;".to_owned(),
                format!("pub mod s0 {{ {} }}", s0.join(" ")),
                format!("pub mod s1 {{ {} }}", s1.join(" ")),
            ];
            let m0: Vec<&str> = m0.iter().map(String::as_str).collect();
            let (found, loads) = reads_in_every_order(
                &[
                    "pub use crate::m2::*;",
                    "pub mod m1 { pub struct C {} pub struct B {} }",
                ],
                &[("mod m2", m2), ("pub mod m0", &m0)],
                reads,
            );
            assert_eq!(found, expected, "{m2:?} {s0:?} {s1:?}");
            assert_eq!(loads, 24 * (1..=m2.len()).product::<usize>() * 6);
        }
    }
}

#[test]
fn a_hidden_item_does_not_make_a_name_ambiguous_where_a_glob_cycle_delivers_the_item_held_in_every_order()
 {
    // `m` globs the root, which globs `x` and `m`, and globs `p` privately:
    // `A` is `x::A`, delivered back to `m` by the root, and `p::A`, which the
    // root cannot access. The root already holds `x::A` through its own
    // glob of `x`, so rustc drops the ambiguity there and accepts `A` from
    // the root and from `m::c`, in every order; inside `m`, and through `m`
    // from the root, it refuses.
    let reads: &[Read] = &[
        ("demo", "crate::A", TYPE),
        ("demo::m::c", "crate::A", TYPE),
        ("demo::m", "self::A", TYPE),
        ("demo::m", "crate::A", TYPE),
        ("demo", "crate::m::A", TYPE),
    ];
    let both = ambiguous_between("demo::p::A", "demo::x::A");
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod x { pub struct A; }",
            "pub mod p { pub struct A; }",
            "use crate::m::*;",
            "pub use crate::x::*;",
        ],
        &[(
            "pub mod m",
            &["pub use super::*;", "use crate::p::*;", "pub mod c {}"],
        )],
        reads,
    );
    assert_eq!(
        found,
        [
            "demo::x::A".to_owned(),
            "demo::x::A".to_owned(),
            both.clone(),
            "demo::x::A".to_owned(),
            both
        ]
    );
    assert_eq!(loads, 720);
}

#[test]
fn an_ambiguity_a_glob_cycle_carries_beside_a_glob_of_one_of_its_items_is_one_answer_in_every_order()
 {
    // `m2` globs the root, whose private struct `A` it reads, and its own
    // child `s1` privately, which brings in `s0::A` (a struct and a
    // function) through `m3`: `A` is ambiguous in `m2` in the type name
    // space. `m0` globs `m2` privately, and `m3` with a `pub` glob, which
    // brings in `s0::A` alone; `m1` globs `m0`. rustc refuses `A` in `m2`
    // in every order (E0659). In `m0` and `m1` its answer depends on the
    // order of the declarations: by which of its globs brought the name into
    // `m2` first, `m0` reads the type `A` as `s0::A` or refuses it (E0659),
    // and `m1` reads it as `s0::A` or finds no type `A`. The index reads the
    // ambiguity of `m2` as visible as its most visible item, the root's
    // struct, which `m0` can access, so `m0` holds the two items through
    // two globs, and `m1` reads both, in every order. The function is
    // `s0::A` everywhere.
    let reads: &[Read] = &[
        ("demo::m2", "self::A", TYPE),
        ("demo::m2", "self::A", VALUE),
        ("demo::m0", "self::A", TYPE),
        ("demo::m0", "self::A", VALUE),
        ("demo::m1", "self::A", TYPE),
        ("demo::m1", "self::A", VALUE),
    ];
    let m2 = [
        "mod s1 { pub(crate) use crate::m3::*; }",
        "use crate::m2::s1::*;",
        "pub use crate::*;",
        "pub mod s0 { pub struct A {} pub fn A() {} }",
    ];
    let both = ambiguous_between("demo::A", "demo::m2::s0::A");
    for first in 0..m2.len() {
        let rotated: Vec<&str> = m2[first..].iter().chain(&m2[..first]).copied().collect();
        let m2 = format!("mod m2 {{ {} }}", rotated.join(" "));
        let (found, loads) = reads_in_every_order(
            &[
                "struct A {}",
                "mod m3 { pub(crate) use crate::m2::s0::*; }",
                &m2,
                "mod m1 { pub(crate) use crate::m0::*; }",
            ],
            &[("mod m0", &["use crate::m2::*;", "pub use crate::m3::*;"])],
            reads,
        );
        assert_eq!(
            found,
            [
                both.clone(),
                "demo::m2::s0::A".to_owned(),
                both.clone(),
                "demo::m2::s0::A".to_owned(),
                both.clone(),
                "demo::m2::s0::A".to_owned(),
            ],
            "{m2}"
        );
        assert_eq!(loads, 240);
    }
}

#[test]
fn a_hidden_module_a_glob_cycle_carries_back_beside_the_module_itself_is_no_ambiguity_in_every_order()
 {
    // `m2` globs `m1` privately, which brings in the module `m1::s0`, and the
    // root, which globs `m3::s0`, which globs `m0::s0`, which globs `m2` and
    // its parent `m0`: the root and `m2` read `s0` as the module `m0::s0`
    // carried around the cycle, and in `m2` it is ambiguous with `m1::s0`.
    // `m0::s0` reads `m0::s0` itself through its glob of `m0`, so rustc drops
    // the ambiguity there, and every reader outside `m2` reads `s0` as
    // `m0::s0` and the root's `s0::C` as the function of `m2`, with no type
    // `C`, in every order.
    let reads: &[Read] = &[
        ("demo::m3::s0", "crate::s0::C", TYPE),
        ("demo::m3::s0", "crate::s0::C", VALUE),
        ("demo", "crate::s0", TYPE),
        ("demo::m3::s0", "self::s0", TYPE),
        ("demo::m0::s0", "self::s0", TYPE),
        ("demo::m2", "self::s0", TYPE),
    ];
    let s0 = [
        "pub use crate::m2::*; pub(crate) use super::*;",
        "pub(crate) use super::*; pub use crate::m2::*;",
    ];
    for s0 in s0 {
        let m0 = format!("pub mod m0 {{ pub mod s0 {{ {s0} }} }}");
        let (found, loads) = reads_in_every_order(
            &[
                "pub mod m3 { pub mod s0 { pub use crate::m0::s0::*; } }",
                &m0,
                "pub mod m1 { pub mod s0 {} }",
                "pub use crate::m3::s0::*;",
            ],
            &[(
                "mod m2",
                &[
                    "use crate::m1::*;",
                    "pub(crate) use super::*;",
                    "pub const fn C() -> u32 { 8 }",
                ],
            )],
            reads,
        );
        assert_eq!(
            found,
            [
                "not found".to_owned(),
                "demo::m2::C".to_owned(),
                "demo::m0::s0".to_owned(),
                "demo::m0::s0".to_owned(),
                "demo::m0::s0".to_owned(),
                ambiguous_between("demo::m0::s0", "demo::m1::s0"),
            ],
            "{s0}"
        );
        assert_eq!(loads, 720);
    }
}

#[test]
fn an_item_a_glob_cycle_carries_back_through_the_module_itself_keeps_its_companion_in_every_order()
{
    // `m0` globs `m2` publicly and `m1::s1` privately, and each holds a
    // struct `B`: `B` is ambiguous in `m0`, and a reader outside `m0` can
    // access `m2::B` only, so `m1::s1::B` rides along with it. `m1::s0` globs
    // `m0` and the root, and the root globs `m1::s0`: the cycle carries
    // `m2::B` back to `m1::s0` through the root only because `m1::s0` passed
    // it on, so it drops nothing there, whichever comes first. The root and
    // `m1::s0` each import `self::B` as `probe`, which a debug build checks
    // against the settled tables after every load: what an import bound
    // while they settled is what they reach. The index reads `B` and each
    // `probe` as the two structs, in every order, and rustc agrees: it
    // refuses `B` and `probe` as ambiguous (E0659) in the root, in `m1::s0`
    // and in `m0`.
    let reads: &[Read] = &[
        ("demo", "self::B", TYPE),
        ("demo", "self::probe", TYPE),
        ("demo::m1::s0", "self::B", TYPE),
        ("demo::m1::s0", "self::probe", TYPE),
        ("demo::m0", "self::B", TYPE),
        ("demo", "crate::m0::B", TYPE),
    ];
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod m1 { pub mod s1 { pub struct B {} } \
             pub mod s0 { pub use crate::m0::*; pub use crate::*; use self::B as probe; } }",
            "pub mod m2 { pub struct B {} }",
            "pub use crate::m1::s0::*;",
            "use self::B as probe;",
        ],
        &[(
            "pub mod m0",
            &["pub use crate::m2::*;", "use crate::m1::s1::*;"],
        )],
        reads,
    );
    let both = ambiguous_between("demo::m1::s1::B", "demo::m2::B");
    assert_eq!(found, vec![both; reads.len()]);
    assert_eq!(loads, 240);
}

#[test]
fn a_path_through_a_module_two_globs_bring_in_as_different_modules_is_ambiguous_in_every_order() {
    // `m` globs `a` and `b`, both publicly, and each holds a module `s`.
    // rustc refuses a path through `s` (E0659), also for a name only one of
    // the two modules has; the same module reached through both globs is one
    // candidate, and the path resolves.
    let reads: &[Read] = &[
        ("demo", "crate::m::s::X", TYPE),
        ("demo::m::c", "super::s::X", TYPE),
        ("demo", "crate::m::s::Z", TYPE),
    ];
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod a { pub mod s { pub struct X; } }",
            "pub mod b { pub mod s { pub struct Z; } }",
        ],
        &[(
            "pub mod m",
            &[
                "pub use crate::a::*;",
                "pub use crate::b::*;",
                "pub mod c {}",
            ],
        )],
        reads,
    );
    assert_eq!(
        found,
        vec![ambiguous_between("demo::a::s", "demo::b::s"); 3]
    );
    assert_eq!(loads, 36);
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod a { pub mod s { pub struct X; } }",
            "pub mod b { pub use crate::a::s; }",
        ],
        &[(
            "pub mod m",
            &[
                "pub use crate::a::*;",
                "pub use crate::b::*;",
                "pub mod c {}",
            ],
        )],
        &reads[..2],
    );
    assert_eq!(found, vec!["demo::a::s::X"; 2]);
    assert_eq!(loads, 36);
}

#[test]
fn an_item_an_explicit_import_supplies_beside_a_glob_that_carries_it_with_a_companion_is_one_answer_in_every_order()
 {
    // `m0` globs `m2` publicly and `m1::s1` privately, and each holds a
    // struct `B`: `B` is ambiguous in `m0`, and a reader outside `m0` can
    // access `m2::B` only, so `m1::s1::B` rides along with it. `w` imports
    // `m2::B` by name, and `mm` globs `m0` and `w`: `w` binds `B` itself and
    // delivers `m2::B`, which `mm` also reads through `m0`, so `mm` drops
    // the companion, whether the import of `w` binds before or after a path
    // of `mm` reads `B`. `mm` imports `self::B` as `probe`, which a debug
    // build checks against the settled tables after every load: what an
    // import bound while they settled is what they reach. The index reads
    // `B` and `probe` in `mm` as `m2::B` in every order. rustc's own answer
    // depends on the order: it keeps the first glob binding of `B` that
    // reaches `mm`, so it refuses `B` as ambiguous (E0659) when `m0` comes
    // before `w` and accepts `m2::B` when `w` comes first. The index takes
    // the answer of the orders rustc accepts, the one a crate that compiles
    // has.
    let reads: &[Read] = &[
        ("demo::mm", "self::B", TYPE),
        ("demo::mm", "self::probe", TYPE),
    ];
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod m1 { pub mod s1 { pub struct B {} } }",
            "pub mod m2 { pub struct B {} }",
            "pub mod m0 { pub use crate::m2::*; use crate::m1::s1::*; }",
            "pub mod w { pub use crate::m2::B; }",
            "pub mod mm { pub use crate::m0::*; pub use crate::w::*; use self::B as probe; }",
        ],
        &[],
        reads,
    );
    assert_eq!(found, vec!["demo::m2::B"; reads.len()]);
    assert_eq!(loads, 120);
}

#[test]
fn a_crate_visible_import_or_glob_that_supplies_an_item_beside_its_companion_is_one_answer_in_every_order()
 {
    // The shape above with a `pub(crate)` import of `m2::B` in `w`, or a
    // `pub(crate)` glob of `w` in `mm`: `mm` can access what `w` delivers
    // either way, so it drops the companion as before, and the index reads
    // `B` and `probe` in `mm` as `m2::B` in every order. rustc again keeps
    // the first glob binding that reaches `mm`: it refuses `B` as ambiguous
    // (E0659) when `m0` comes before `w` and accepts `m2::B` when `w` comes
    // first.
    let reads: &[Read] = &[
        ("demo::mm", "self::B", TYPE),
        ("demo::mm", "self::probe", TYPE),
    ];
    for (w, mm) in [
        (
            "pub mod w { pub(crate) use crate::m2::B; }",
            "pub mod mm { pub use crate::m0::*; pub use crate::w::*; use self::B as probe; }",
        ),
        (
            "pub mod w { pub use crate::m2::B; }",
            "pub mod mm { pub use crate::m0::*; pub(crate) use crate::w::*; use self::B as probe; }",
        ),
        (
            "pub mod w { pub(crate) use crate::m2::B; }",
            "pub mod mm { pub use crate::m0::*; pub(crate) use crate::w::*; use self::B as probe; }",
        ),
    ] {
        let (found, loads) = reads_in_every_order(
            &[
                "pub mod m1 { pub mod s1 { pub struct B {} } }",
                "pub mod m2 { pub struct B {} }",
                "pub mod m0 { pub use crate::m2::*; use crate::m1::s1::*; }",
                w,
                mm,
            ],
            &[],
            reads,
        );
        assert_eq!(found, vec!["demo::m2::B"; reads.len()], "{w} {mm}");
        assert_eq!(loads, 120);
    }
}

#[test]
fn a_companion_a_glob_cycle_carries_beside_an_explicit_import_of_its_item_leaves_no_view_with_it_in_every_order()
 {
    // The shape above, where `m0` also globs `mm` privately, so a cycle of
    // globs runs through `m0` and `mm`, and `rr` globs `mm` and imports
    // `self::B` as `probe2`. The view of `mm` that `rr` reads holds what
    // `mm` delivers once the companion is dropped, and a debug build checks
    // the imports of both modules against the settled tables. The index
    // reads `B` and each `probe` as `m2::B` in every order. rustc's own
    // answer depends on the order: it refuses `B` as ambiguous (E0659) when
    // `m0` comes before `w` and accepts `m2::B` when `w` comes first.
    let reads: &[Read] = &[
        ("demo::mm", "self::B", TYPE),
        ("demo::mm", "self::probe", TYPE),
        ("demo::rr", "self::B", TYPE),
        ("demo::rr", "self::probe2", TYPE),
    ];
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod m1 { pub mod s1 { pub struct B {} } }",
            "pub mod m2 { pub struct B {} }",
            "pub mod m0 { pub use crate::m2::*; use crate::m1::s1::*; use crate::mm::*; }",
            "pub mod w { pub use crate::m2::B; }",
            "pub mod mm { pub use crate::m0::*; pub use crate::w::*; use self::B as probe; }",
            "pub mod rr { pub use crate::mm::*; use self::B as probe2; }",
        ],
        &[],
        reads,
    );
    assert_eq!(found, vec!["demo::m2::B"; reads.len()]);
    assert_eq!(loads, 720);
}

#[test]
fn a_module_an_explicit_import_supplies_beside_a_glob_that_carries_it_with_a_companion_is_one_answer_in_every_order()
 {
    // The shape where `w` supplies `m2::B` beside `m0`, whose globs carry
    // `m1::s1::B` with it as a companion, with `B` a module of each and the
    // path going on through it: `mm` imports `self::B::X` as `probe`. While
    // the companion is still delivered, the path through `B` is ambiguous
    // and the import is held; once the settled tables drop the companion, the
    // imports are held again only as the tables with the drop show them
    // ambiguous, and a debug build checks that every held import still walks
    // an ambiguous path. The index reads `probe` and `self::B::X` in `mm` as
    // `m2::B::X` in every order. rustc's own answer depends on the order: it
    // refuses `B` as ambiguous (E0659) when `m0` comes before `w` and accepts
    // `m2::B::X` when `w` comes first.
    let reads: &[Read] = &[
        ("demo::mm", "self::probe", TYPE),
        ("demo::mm", "self::B::X", TYPE),
    ];
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod m1 { pub mod s1 { pub mod B { pub struct X {} } } }",
            "pub mod m2 { pub mod B { pub struct X {} } }",
            "pub mod m0 { pub use crate::m2::*; use crate::m1::s1::*; }",
            "pub mod w { pub use crate::m2::B; }",
            "pub mod mm { pub use crate::m0::*; pub use crate::w::*; use self::B::X as probe; }",
        ],
        &[],
        reads,
    );
    assert_eq!(found, vec!["demo::m2::B::X"; reads.len()]);
    assert_eq!(loads, 120);
}

#[test]
fn an_import_that_reads_its_name_through_the_glob_whose_companion_it_would_drop_keeps_the_ambiguity_in_every_order()
 {
    // The shape where `w` imports `B` by name beside `m0`, whose globs carry
    // `m1::s1::B` with `m2::B` as a companion, but `w` imports `crate::mm::B`:
    // it reads the name through the glob of `m0` in `mm`, so it binds the
    // companion as well, and what it delivers back to `mm` does not justify
    // dropping the companion there. The index reads `B` and `probe` in `mm`
    // and `B` in `w` as ambiguous between the two structs in every order,
    // and rustc agrees: it refuses `B` as ambiguous (E0659) in every order.
    let reads: &[Read] = &[
        ("demo::mm", "self::B", TYPE),
        ("demo::mm", "self::probe", TYPE),
        ("demo::w", "self::B", TYPE),
    ];
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod m1 { pub mod s1 { pub struct B {} } }",
            "pub mod m2 { pub struct B {} }",
            "pub mod m0 { pub use crate::m2::*; use crate::m1::s1::*; }",
            "pub mod w { pub use crate::mm::B; }",
            "pub mod mm { pub use crate::m0::*; pub use crate::w::*; use self::B as probe; }",
        ],
        &[],
        reads,
    );
    let both = ambiguous_between("demo::m1::s1::B", "demo::m2::B");
    assert_eq!(found, vec![both; reads.len()]);
    assert_eq!(loads, 120);
}

#[test]
fn a_companion_that_arrives_where_the_reader_cannot_access_it_is_dropped_as_its_glob_drops_it_in_every_order()
 {
    // `m1` globs `m2` privately and `m0::s0` publicly, and each holds a
    // struct `B`: `B` is ambiguous in `m1`, and `m2::s1`, which globs `m1`
    // privately, can access `m0::s0::B` only, so `m2::B` rides along with it.
    // `m2::s1` also globs `m0::s0` publicly, which defines `B`, so it drops
    // the companion. `m2::s0` globs `m2::s1` and cannot access what `m2::s1`
    // brings in through its private glob: the companion arrives at the view
    // `m2::s0` reads as an item it cannot access beside one it holds, and that
    // view drops it as the glob it came through drops it, whether it came in
    // before or after `m0::s0::B`. The index reads `B` in `m2::s0` as
    // `m0::s0::B` in every order. rustc's own answer depends on the order: it
    // accepts `m0::s0::B` in 54 of the 72 orders and refuses `B` as ambiguous
    // (E0659) in the others.
    let reads: &[Read] = &[("demo::m2::s0", "self::B", TYPE)];
    let (found, loads) = reads_in_every_order(
        &["mod m0 { pub mod s0 { pub(crate) struct B {} } }"],
        &[
            (
                "mod m2",
                &[
                    "pub mod s1 { pub use crate::m0::s0::*; use crate::m1::*; }",
                    "pub mod s0 { pub use crate::m2::s1::*; }",
                    "pub struct B {}",
                ],
            ),
            (
                "mod m1",
                &["use crate::m2::*;", "pub use crate::m0::s0::*;"],
            ),
        ],
        reads,
    );
    assert_eq!(found, ["demo::m0::s0::B"]);
    assert_eq!(loads, 72);
}

#[test]
fn a_companion_a_chain_of_globs_carries_to_a_reader_that_cannot_access_it_is_dropped_in_every_order()
 {
    // `m0::s0` globs `m2::s1` privately and `m1::s0` publicly, and each holds
    // a struct `B`; `m2` globs `m0::s0` privately, so it holds `m1::s0::B`
    // with `m2::s1::B` as a companion, and `m2::s0` globs `m2` and `m1::s0`,
    // which defines `B`, so it drops the companion. The crate root globs
    // `m2::s0` privately, `m3` globs the root and `m1` globs `m3`. The
    // companion arrives at the view of `m2::s0` that the root reads as an
    // item the root cannot access beside `m1::s0::B`, and the view drops it
    // as the glob of `m2` it came through drops it, whether it came in before
    // or after. The index reads `crate::m1::B` from `m3::s0` and from `m3` as
    // `m1::s0::B` in every order, and rustc agrees in every order.
    let reads: &[Read] = &[
        ("demo::m3::s0", "crate::m1::B", TYPE),
        ("demo::m3", "crate::m1::B", TYPE),
    ];
    let (found, loads) = reads_in_every_order(
        &[
            "use crate::m2::s0::*;",
            "pub mod m1 { pub mod s0 { pub struct B {} } pub(crate) use crate::m3::*; }",
            "mod m3 { pub(crate) use super::*; pub mod s0 {} }",
            "mod m0 { pub mod s0 { use crate::m2::s1::*; pub use crate::m1::s0::*; } }",
        ],
        &[(
            "pub mod m2",
            &[
                "use crate::m0::s0::*;",
                "pub mod s0 { use super::*; pub use crate::m1::s0::*; }",
                "pub mod s1 { pub(crate) struct B {} }",
            ],
        )],
        reads,
    );
    assert_eq!(found, vec!["demo::m1::s0::B"; reads.len()]);
    assert_eq!(loads, 720);
}

/// [`reads_in_every_order`] for a library of `edition`, where a header of
/// `modules` may open nested modules, as `mod m { pub mod s`: the
/// declarations go into the innermost, and each module the header leaves
/// open is closed after them.
fn reads_in_every_order_in(
    edition: &str,
    others: &[&str],
    modules: &[(&str, &[&str])],
    reads: &[Read],
) -> (Vec<String>, usize) {
    let mut expected: Option<Vec<String>> = None;
    let mut loads = 0;
    let insides: Vec<Vec<Vec<String>>> = modules
        .iter()
        .map(|(_, inner)| permutations(&inner.iter().map(|d| (*d).to_owned()).collect::<Vec<_>>()))
        .collect();
    let mut choice = vec![0; modules.len()];
    loop {
        let mut blocks: Vec<String> = others.iter().map(|b| (*b).to_owned()).collect();
        for (at, (header, _)) in modules.iter().enumerate() {
            let opened = header.matches('{').count() - header.matches('}').count();
            let closes = " }".repeat(opened);
            blocks.push(format!(
                "{header} {{ {} }}{closes}",
                insides[at][choice[at]].join(" ")
            ));
        }
        for order in permutations(&blocks) {
            let text = order.join("\n");
            let ws = load_in(edition, &[("src/lib.rs", text.as_str())])
                .unwrap_or_else(|e| panic!("{e}"));
            let results: Vec<String> = reads
                .iter()
                .map(|(module, path, ns)| read_as(&ws, module, path, *ns))
                .collect();
            match &expected {
                None => expected = Some(results),
                Some(first) => assert_eq!(&results, first, "{order:?}"),
            }
            loads += 1;
        }
        let mut at = 0;
        while at < choice.len() {
            choice[at] += 1;
            if choice[at] < insides[at].len() {
                break;
            }
            choice[at] = 0;
            at += 1;
        }
        if at == choice.len() {
            break;
        }
    }
    (expected.expect("loaded"), loads)
}

#[test]
fn an_item_a_glob_cycle_carries_back_beside_a_glob_that_holds_it_from_elsewhere_drops_its_companion_in_every_order()
 {
    // `m3` globs `m3::s0` privately and `m0`, and `m0` globs `m3` and
    // `m3::s1`, which globs `m1`: the function `A` of `m1` reaches `m3`
    // through `m0`, beside `m3::s0::A`, so `A` is ambiguous in `m3`, and to
    // `m0`, which cannot access `m3::s0::A`, it rides along as a companion of
    // `m1::A`. The glob of `m3::s1` in `m0` delivers `m1::A` from a module
    // whose globs do not reach `m0`, so `m0` holds it apart from the cycle
    // that carries it back, and drops the companion, though `m3::s1` does not
    // bind `A` itself. The index reads `crate::m0::A` from `m3::s1` as
    // `m1::A` in the value name space in every order, and rustc agrees in
    // every order, in the editions 2021 and 2024.
    let reads: &[Read] = &[
        ("demo::m3::s1", "crate::m0::A", VALUE),
        ("demo::m3::s1", "crate::m0::A", TYPE),
    ];
    let (found, loads) = reads_in_every_order_in(
        "2024",
        &["pub mod m1 { pub const fn A() -> u32 { 7 } }"],
        &[
            (
                "pub mod m0",
                &["pub use crate::m3::s1::*;", "use crate::m3::*;"],
            ),
            (
                "mod m3",
                &[
                    "pub mod s0 { pub const fn A() -> u32 { 15 } }",
                    "use crate::m3::s0::*;",
                    "pub(crate) use crate::m0::*;",
                    "pub mod s1 { pub use crate::m1::*; }",
                ],
            ),
        ],
        reads,
    );
    assert_eq!(found, ["demo::m1::A", "not found"]);
    assert_eq!(loads, 288);
}

#[test]
fn a_module_a_glob_cycle_carries_back_beside_a_glob_that_holds_it_from_elsewhere_starts_a_path_in_every_order()
 {
    // `m0::s0` globs `m1::s0` privately and the crate root publicly, and the
    // root globs `m1` privately, so it holds the module `m1::s0`. `m1::s0`
    // globs `m0` privately and `m2::s0`, which globs `m0::s0`: what
    // `m1::s0` brings in under `s0` is `m1::s0`, carried back around the
    // cycle, with `m0::s0` riding along hidden. The root delivers `m1::s0`
    // to `m0::s0` from globs that do not reach `m0::s0`, so `m0::s0` drops
    // the companion and the bare first segment `s0` of a path in `m0::s0`
    // names `m1::s0` alone. The index reads `s0::B` from `m0::s0` as
    // `m1::s0::B` in every order, and rustc agrees in every order, in the
    // editions 2021 and 2024.
    let reads: &[Read] = &[
        ("demo::m0::s0", "s0::B", TYPE),
        ("demo::m0::s0", "s0::B", VALUE),
    ];
    let (found, loads) = reads_in_every_order_in(
        "2024",
        &[
            "pub mod m0 { pub mod s0 { use crate::m1::s0::*; pub use crate::*; } }",
            "use self::m1::*;",
            "pub mod m2 { pub mod s0 { pub(crate) use crate::m0::s0::*; } }",
        ],
        &[(
            "mod m1 { pub mod s0",
            &[
                "use crate::m0::*;",
                "pub struct B {}",
                "pub use crate::m2::s0::*;",
            ],
        )],
        reads,
    );
    assert_eq!(found, ["demo::m1::s0::B", "not found"]);
    assert_eq!(loads, 144);
}

#[test]
fn an_item_a_glob_brings_in_from_its_definition_apart_from_a_cycle_drops_the_companion_the_cycle_carries_in_every_order()
 {
    // The shapes above where the glob that holds the item apart from the
    // cycle reaches its definition through another glob. `m3::s1` globs
    // `m1`, which defines `A`, and `m0` privately: what it delivers to `m0`
    // comes from `m1` without passing through `m0`, so `m0` drops the
    // companion `m3::s0::A` that its glob of `m3` carries beside `m1::A`.
    // And the crate root globs `m1::s0`, which globs `m1`, where `B` is
    // defined, and `m2::s1`, which globs the root and, privately, `m2`, which
    // defines another `B`: the root drops `m2::B`, which rides along with
    // `m1::B` carried back around the cycle. The index reads `crate::m0::A`
    // from `m3::s1` as `m1::A` and `crate::B` from `m2::s1` as `m1::B`, in the
    // value name space, in every order, and rustc agrees in every order, in
    // the editions 2021 and 2024.
    let (found, loads) = reads_in_every_order_in(
        "2024",
        &[
            "pub mod m1 { pub const fn A() -> u32 { 7 } }",
            "pub mod m0 { pub use crate::m3::s1::*; use crate::m3::*; }",
        ],
        &[(
            "mod m3",
            &[
                "pub mod s0 { pub const fn A() -> u32 { 15 } }",
                "use crate::m3::s0::*;",
                "pub mod s1 { use crate::m0::*; pub use crate::m1::*; }",
                "pub use crate::m0::*;",
            ],
        )],
        &[("demo::m3::s1", "crate::m0::A", VALUE)],
    );
    assert_eq!(found, ["demo::m1::A"]);
    assert_eq!(loads, 144);
    let (found, loads) = reads_in_every_order_in(
        "2024",
        &[
            "pub use crate::m1::s0::*;",
            "pub mod m1 { pub mod s0 { pub use crate::m1::*; } pub(crate) const fn B() -> u32 { 1 } }",
            "use crate::m2::s1::*;",
        ],
        &[(
            "mod m2",
            &[
                "pub const fn B() -> u32 { 6 }",
                "pub mod s1 { pub use crate::*; use crate::m2::*; }",
            ],
        )],
        &[("demo::m2::s1", "crate::B", VALUE)],
    );
    assert_eq!(found, ["demo::m1::B"]);
    assert_eq!(loads, 48);
}
