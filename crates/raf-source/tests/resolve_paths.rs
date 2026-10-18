// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! What `resolve` accepts as a canonical path, and what it answers for a path
//! whose lookups read a glob that names nothing.

use raf_source::{MemoryTree, Namespace, ResolveError, Workspace};

mod support;

use support::orders::{Read, ambiguous_between, in_every_order, read_as, resolved_as};

fn lib(text: &str) -> Workspace {
    lib_in("2021", text)
}

fn lib_in(edition: &str, text: &str) -> Workspace {
    let manifest = format!(
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"{edition}\"\n\
         [dependencies]\nserde = \"1\"\n"
    );
    let tree: MemoryTree = [("Cargo.toml", manifest.as_str()), ("src/lib.rs", text)]
        .into_iter()
        .collect();
    Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"))
}

/// The answer in each name space, then in all of them.
fn answers(ws: &Workspace, path: &str) -> Vec<Result<String, ResolveError>> {
    let mut all: Vec<_> = Namespace::ALL
        .iter()
        .map(|ns| ws.resolve_in(path, *ns).map(|found| found.path))
        .collect();
    all.push(ws.resolve(path).map(|found| found.path));
    all
}

fn everywhere(answer: &ResolveError) -> Vec<Result<String, ResolveError>> {
    vec![Err(answer.clone()); 4]
}

fn path_of(ws: &Workspace, path: &str) -> String {
    ws.resolve(path)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .path
}

fn glob_error(path: &str, glob: &str, module: &str) -> ResolveError {
    ResolveError::UnresolvedGlob {
        path: path.to_owned(),
        glob: glob.to_owned(),
        module: module.to_owned(),
    }
}

#[test]
fn a_path_keyword_in_any_position_is_invalid_beside_the_canonical_path() {
    let ws = lib("pub mod a { pub fn x() {} pub struct S; }\npub fn x() {}\n");
    for bad in [
        "demo::crate::a::x",
        "demo::super::x",
        "demo::self::a::x",
        "demo::a::self",
        "demo::a::Self",
        "demo::a::S::Self",
        "demo::r#crate::a::x",
        "demo::r#super::x",
        "crate::a::x",
        "self::x",
        "super::x",
        "Self",
    ] {
        assert_eq!(
            ws.resolve(bad),
            Err(ResolveError::Invalid(bad.to_owned())),
            "{bad}"
        );
        for ns in Namespace::ALL {
            assert_eq!(
                ws.resolve_in(bad, ns),
                Err(ResolveError::Invalid(bad.to_owned())),
                "{bad} in {ns:?}"
            );
        }
    }
    assert_eq!(path_of(&ws, "demo::a::x"), "demo::a::x");
    assert_eq!(path_of(&ws, "demo::x"), "demo::x");
    assert_eq!(path_of(&ws, "demo::a::S"), "demo::a::S");
    // A name that only contains a keyword is a name.
    let ws = lib("pub mod crates { pub fn selfish() {} }\npub struct Selves;\n");
    assert_eq!(
        path_of(&ws, "demo::crates::selfish"),
        "demo::crates::selfish"
    );
    assert_eq!(path_of(&ws, "demo::Selves"), "demo::Selves");
}

#[test]
fn a_glob_of_a_missing_local_module_loads_and_fails_each_lookup_that_reads_it() {
    let ws = lib(concat!(
        "pub mod a {\n",
        "    pub use self::missing::*;\n",
        "    pub struct Here;\n",
        "    pub mod inner { pub fn deep() {} }\n",
        "}\n",
        "pub mod b { pub use crate::a::*; }\n",
        "pub mod c { pub use crate::a::gone::*; }\n",
        "pub mod d { pub use super::a::inner::nothing::*; pub fn f() {} }\n",
        "pub use self::a::gone as alias;\n",
        "pub struct Sibling;\n",
    ));
    // Each lookup that reads a glob whose path names nothing fails with the
    // glob, its module and the path asked for.
    for (path, glob, module) in [
        ("demo::a::gone", "self::missing", "demo::a"),
        ("demo::a::gone::deeper", "self::missing", "demo::a"),
        ("demo::b::gone", "self::missing", "demo::a"),
        ("demo::c::x", "crate::a::gone", "demo::c"),
        ("demo::d::x", "super::a::inner::nothing", "demo::d"),
        ("demo::alias", "self::missing", "demo::a"),
    ] {
        assert_eq!(
            ws.resolve(path),
            Err(glob_error(path, glob, module)),
            "{path}"
        );
    }
    // A name the module binds itself does not read its globs.
    assert_eq!(path_of(&ws, "demo::a::Here"), "demo::a::Here");
    assert_eq!(path_of(&ws, "demo::a::inner::deep"), "demo::a::inner::deep");
    assert_eq!(
        ws.resolve_in("demo::d::f", Namespace::Value)
            .map(|found| found.path),
        Ok("demo::d::f".to_owned())
    );
    // A sibling path that reads none of them resolves.
    assert_eq!(path_of(&ws, "demo::Sibling"), "demo::Sibling");
    assert_eq!(path_of(&ws, "demo::b"), "demo::b");
    // A name space whose lookup reads the glob fails, though another has the
    // name: the glob could bring in a type `f`.
    assert_eq!(
        ws.resolve("demo::d::f"),
        Err(glob_error(
            "demo::d::f",
            "super::a::inner::nothing",
            "demo::d"
        ))
    );
}

#[test]
fn a_glob_the_index_cannot_look_into_leaves_a_lookup_not_found() {
    let ws = lib(concat!(
        "pub mod a { pub use std::io::*; pub struct Here; }\n",
        "pub mod b { pub use serde::*; }\n",
        "pub mod d { pub use std::collections::*; }\n",
        "pub mod e { pub use crate::d::hash_map::*; }\n",
        "pub mod f { pub use serde::de; }\n",
        "pub mod g { pub use crate::f::de::*; }\n",
        "pub mod h { pub use crate::e::*; pub use crate::g::*; }\n",
        "pub mod k { pub use ::unknown::m::*; }\n",
    ));
    for path in [
        "demo::a::gone",
        "demo::b::gone",
        "demo::e::x",
        "demo::g::x",
        "demo::h::x",
        "demo::k::x",
    ] {
        assert_eq!(
            ws.resolve(path),
            Err(ResolveError::NotFound(path.to_owned())),
            "{path}"
        );
    }
    assert_eq!(path_of(&ws, "demo::a::Here"), "demo::a::Here");
}

#[test]
fn globs_of_dependencies_in_one_module_leave_a_missing_name_not_found() {
    let not_found = |path: &str| everywhere(&ResolveError::NotFound(path.to_owned()));
    // Each glob's first segment is a crate that only the other glob could
    // also bring in: neither waits on the other.
    for (edition, text) in [
        (
            "2021",
            "pub mod a { pub use std::io::*; pub use serde::*; pub struct Here; }\n\
             pub mod b { pub use crate::a::*; }\n",
        ),
        (
            "2021",
            "pub mod a { pub use std::io::*; pub use std::fmt::*; pub struct Here; }\n\
             pub mod b { pub use crate::a::*; }\n",
        ),
        (
            "2021",
            "pub mod a { pub use serde::*; pub use tokio::*; pub struct Here; }\n\
             pub mod b { pub use crate::a::*; }\n",
        ),
        (
            "2015",
            "pub mod a { pub use std::io::*; pub use std::fmt::*; pub struct Here; }\n\
             pub mod b { pub use a::*; }\n",
        ),
    ] {
        let ws = lib_in(edition, text);
        for path in ["demo::a::gone", "demo::b::gone"] {
            assert_eq!(
                answers(&ws, path),
                not_found(path),
                "{edition} {text}: {path}"
            );
        }
        for path in ["demo::a::Here", "demo::b::Here"] {
            assert_eq!(
                answers(&ws, path),
                [
                    Ok("demo::a::Here".to_owned()),
                    Ok("demo::a::Here".to_owned()),
                    Err(ResolveError::NotFound(path.to_owned())),
                    Ok("demo::a::Here".to_owned()),
                ],
                "{edition} {text}: {path}"
            );
        }
    }
    // The same at the crate root, in both editions.
    for edition in ["2015", "2021"] {
        let ws = lib_in(
            edition,
            "pub use std::io::*;\npub use std::fmt::*;\npub struct Top {}\n",
        );
        assert_eq!(
            answers(&ws, "demo::gone"),
            not_found("demo::gone"),
            "{edition}"
        );
        assert_eq!(path_of(&ws, "demo::Top"), "demo::Top", "{edition}");
    }
    // A glob of a dependency could bring in the module a glob beside it
    // names, so that glob is not known to name nothing; a glob of a local
    // module that does not have the name is.
    let ws = lib(concat!(
        "pub mod a { pub use std::io::*; pub use self::missing::*; }\n",
        "pub mod b { pub use std::io::*; pub use crate::other::missing::*; }\n",
        "pub mod other { pub struct There; }\n",
    ));
    assert_eq!(answers(&ws, "demo::a::gone"), not_found("demo::a::gone"));
    assert_eq!(
        answers(&ws, "demo::b::gone"),
        everywhere(&glob_error(
            "demo::b::gone",
            "crate::other::missing",
            "demo::b"
        ))
    );
    assert_eq!(path_of(&ws, "demo::other::There"), "demo::other::There");
}

#[test]
fn a_cycle_of_globs_stays_cyclic_beside_globs_that_resolve() {
    let ws = lib(concat!(
        "pub mod a { pub use crate::b::x::*; }\n",
        "pub mod b { pub use crate::a::*; }\n",
        "pub mod c { pub use crate::d::*; pub struct C; }\n",
        "pub mod d { pub use crate::c::*; }\n",
    ));
    for path in ["demo::a::gone", "demo::b::gone"] {
        assert_eq!(
            answers(&ws, path),
            everywhere(&ResolveError::Cyclic(path.to_owned())),
            "{path}"
        );
    }
    assert_eq!(path_of(&ws, "demo::d::C"), "demo::c::C");
    assert_eq!(
        answers(&ws, "demo::c::gone"),
        everywhere(&ResolveError::NotFound("demo::c::gone".to_owned()))
    );
}

#[test]
fn a_cycle_is_reported_before_a_glob_that_names_nothing() {
    // `gone` is bound by a cycle of imports in `c`, which the glob of `c`
    // reads, beside a glob of a missing module: the cycle decides.
    let ws = lib(concat!(
        "pub mod a { pub use self::missing::*; pub use crate::c::*; }\n",
        "pub mod c { pub use crate::d::gone; }\n",
        "pub mod d { pub use crate::c::gone; }\n",
    ));
    for path in ["demo::a::gone", "demo::c::gone"] {
        assert_eq!(
            answers(&ws, path),
            everywhere(&ResolveError::Cyclic(path.to_owned())),
            "{path}"
        );
    }
    // Two globs that bring in different items under one name are ambiguous
    // beside a glob of a missing module.
    let ws = lib(concat!(
        "pub mod a { pub use self::missing::*; pub use crate::c::*; pub use crate::d::*; }\n",
        "pub mod c { pub struct X {} }\n",
        "pub mod d { pub struct X {} }\n",
    ));
    assert!(
        matches!(
            ws.resolve_in("demo::a::X", Namespace::Type),
            Err(ResolveError::Ambiguous(ref ids)) if ids.len() == 2
        ),
        "{:?}",
        ws.resolve_in("demo::a::X", Namespace::Type)
    );
}

#[test]
fn a_name_found_through_a_glob_beside_a_glob_that_names_nothing_fails() {
    let ws = lib(concat!(
        "pub mod a { pub use self::missing::*; pub use crate::c::*; }\n",
        "pub mod c { pub struct C; pub mod inner { pub fn z() {} } }\n",
        "pub mod e { pub use self::m::missing::*; pub mod m {} }\n",
        "pub use crate::missing::*;\n",
        "pub struct Top {}\n",
    ));
    for (path, glob, module) in [
        ("demo::a::C", "self::missing", "demo::a"),
        ("demo::a::inner::z", "self::missing", "demo::a"),
        ("demo::e::gone", "self::m::missing", "demo::e"),
        ("demo::gone", "crate::missing", "demo"),
    ] {
        assert_eq!(
            answers(&ws, path),
            everywhere(&glob_error(path, glob, module)),
            "{path}"
        );
    }
    assert_eq!(path_of(&ws, "demo::c::C"), "demo::c::C");
    assert_eq!(path_of(&ws, "demo::c::inner::z"), "demo::c::inner::z");
    assert_eq!(
        ws.resolve_in("demo::Top", Namespace::Type)
            .map(|found| found.path),
        Ok("demo::Top".to_owned())
    );
}

/// The path the answer names, or the error.
fn from(ws: &Workspace, module: &str, path: &str) -> Result<String, ResolveError> {
    ws.resolve_from(module, path).map(|found| found.path)
}

#[test]
fn a_use_path_counts_only_what_the_reading_module_can_access() {
    let ws = lib(
        "mod a { pub fn open() {} fn closed() {} mod inner { pub fn deep() {} } }\n\
                  mod b { pub(crate) fn sibling() {} }\n",
    );
    // The neighbour that reads: the module itself, a child of it, and a
    // public item from a sibling.
    assert_eq!(
        from(&ws, "demo::a", "self::closed"),
        Ok("demo::a::closed".to_owned())
    );
    assert_eq!(
        from(&ws, "demo::a::inner", "super::closed"),
        Ok("demo::a::closed".to_owned())
    );
    assert_eq!(
        from(&ws, "demo::b", "crate::a::open"),
        Ok("demo::a::open".to_owned())
    );
    assert_eq!(
        from(&ws, "demo::a", "crate::b::sibling"),
        Ok("demo::b::sibling".to_owned())
    );
    // The refusals: a private item of a module read from outside it, and a
    // module that is private to the one that declares it.
    assert_eq!(
        from(&ws, "demo::b", "crate::a::closed"),
        Err(ResolveError::NotFound("crate::a::closed".to_owned()))
    );
    assert_eq!(
        from(&ws, "demo::b", "crate::a::inner::deep"),
        Err(ResolveError::NotFound("crate::a::inner::deep".to_owned()))
    );
}

#[test]
fn a_use_path_is_read_in_one_name_space_or_in_each_in_turn() {
    let ws = lib("mod a { pub struct Both {} pub fn Both() {} pub fn only() {} }\nmod b {}\n");
    assert_eq!(
        ws.resolve_from_in("demo::b", "crate::a::Both", Namespace::Type)
            .map(|found| found.sites.len()),
        Ok(1)
    );
    let value = ws
        .resolve_from_in("demo::b", "crate::a::Both", Namespace::Value)
        .expect("the function");
    let ty = ws
        .resolve_from_in("demo::b", "crate::a::Both", Namespace::Type)
        .expect("the struct");
    assert_ne!(value.sites, ty.sites);
    assert_eq!(
        from(&ws, "demo::b", "crate::a::only"),
        Ok("demo::a::only".to_owned())
    );
}

#[test]
fn a_module_that_is_not_in_the_workspace_and_a_text_that_is_no_path_are_refused() {
    let ws = lib("mod a { pub fn go() {} }\n");
    assert_eq!(
        ws.resolve_from("demo::gone", "crate::a::go"),
        Err(ResolveError::UnknownModule("demo::gone".to_owned()))
    );
    assert_eq!(
        ws.resolve_from("demo::a", "crate::a::"),
        Err(ResolveError::Invalid("crate::a::".to_owned()))
    );
    assert_eq!(
        ws.resolve_from("demo::a", "super::super::go"),
        Err(ResolveError::Invalid("super::super::go".to_owned()))
    );
    // The neighbour: the same module reads a real path.
    assert_eq!(
        from(&ws, "demo::a", "self::go"),
        Ok("demo::a::go".to_owned())
    );
}

const TYPE: Namespace = Namespace::Type;
const VALUE: Namespace = Namespace::Value;

/// What each reader makes of its path, in every order of the declarations.
fn reads_in_every_order(
    others: &[&str],
    modules: &[(&str, &[&str])],
    reads: &[Read],
) -> (Vec<String>, usize) {
    in_every_order(lib, others, modules, |ws| {
        reads
            .iter()
            .map(|(module, path, ns)| read_as(ws, module, path, *ns))
            .collect()
    })
}

#[test]
fn a_path_that_starts_with_a_name_two_globs_bring_in_as_different_modules_is_ambiguous_in_every_order()
 {
    // `m` globs `p` publicly and `q` privately, and both hold a module `s`.
    // rustc refuses a path that starts with `s` (E0659, or E0432 and E0433
    // where the private glob is out of the reader's reach), also for a name
    // only one of the two modules has, in the type and the value name space,
    // in a `use` and in code, in every order and in both editions. The
    // readers are `m` itself, its child `k` that globs it, and the module `r`
    // that globs `m`.
    let reads: &[Read] = &[
        ("demo::m", "s::A", TYPE),
        ("demo::m", "s::A", VALUE),
        ("demo::m", "s::B", TYPE),
        ("demo::m::k", "s::A", TYPE),
        ("demo::m::k", "s::B", TYPE),
        ("demo::r", "s::A", TYPE),
        ("demo::r", "s::A", VALUE),
        ("demo::r", "s::B", TYPE),
        // The same names through `self` and `super` and with the full path
        // were refused before.
        ("demo::m", "self::s::A", TYPE),
        ("demo::m::k", "super::s::A", TYPE),
        ("demo::r", "crate::m::s::A", TYPE),
    ];
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod p { pub mod s { pub struct A; } }",
            "pub mod q { pub mod s { pub struct B; } }",
            "pub mod r { use crate::m::*; }",
        ],
        &[(
            "pub mod m",
            &[
                "pub use crate::p::*;",
                "use crate::q::*;",
                "pub mod k { use super::*; }",
            ],
        )],
        reads,
    );
    assert_eq!(
        found,
        vec![ambiguous_between("demo::p::s", "demo::q::s"); reads.len()]
    );
    assert_eq!(loads, 144);
}

#[test]
fn a_path_that_starts_with_a_name_two_globs_bring_in_as_one_module_still_resolves_in_every_order() {
    // The neighbour: `b` re-exports the module `s` of `p`, so the two globs
    // of `m` bring in one module, and rustc accepts the paths that start
    // with it (in `m`, in its child and in the reader that globs it). So it
    // does for a module with the one glob and for a module of its own.
    let reads: &[Read] = &[
        ("demo::m", "s::A", TYPE),
        ("demo::m", "s::A", VALUE),
        ("demo::m::k", "s::A", TYPE),
        ("demo::r", "s::A", TYPE),
        ("demo::n", "s::A", TYPE),
        ("demo::n", "own::A", TYPE),
        ("demo::m", "s::B", TYPE),
    ];
    let (found, loads) = reads_in_every_order(
        &[
            "pub mod p { pub mod s { pub struct A; } }",
            "pub mod b { pub use crate::p::s; }",
            "pub mod r { use crate::m::*; }",
            "pub mod n { use crate::p::*; pub mod own { pub struct A; } use own::A as _a; }",
        ],
        &[(
            "pub mod m",
            &[
                "pub use crate::p::*;",
                "use crate::b::*;",
                "pub mod k { use super::*; }",
            ],
        )],
        reads,
    );
    assert_eq!(
        found,
        [
            "demo::p::s::A",
            "demo::p::s::A",
            "demo::p::s::A",
            "demo::p::s::A",
            "demo::p::s::A",
            "demo::n::own::A",
            "not found",
        ]
    );
    assert_eq!(loads, 720);
}

#[test]
fn an_import_that_starts_with_a_name_two_globs_bring_in_as_different_modules_is_refused_in_every_order()
 {
    // `r` imports `s::A` and `s::B` where `s` is ambiguous in `r`, and rustc
    // refuses both imports (E0659, or E0432 where it cannot tell): nothing
    // binds `X` or `Y`. `t` imports through a name that stands for one
    // module, and its imports bind.
    let paths = ["demo::r::X", "demo::r::Y", "demo::t::X"];
    let (found, loads) = in_every_order(
        lib,
        &[
            "pub mod p { pub mod s { pub struct A; } }",
            "pub mod q { pub mod s { pub struct B; } }",
            "pub mod t { use crate::p::*; pub use s::A as X; }",
        ],
        &[
            ("pub mod m", &["pub use crate::p::*;", "use crate::q::*;"]),
            (
                "pub mod r",
                &[
                    "use crate::m::*;",
                    "pub use s::A as X;",
                    "pub use s::B as Y;",
                ],
            ),
        ],
        |ws| paths.iter().map(|path| resolved_as(ws, path)).collect(),
    );
    let both = ambiguous_between("demo::p::s", "demo::q::s");
    assert_eq!(found, [both.clone(), both, "demo::p::s::A".to_owned()]);
    assert_eq!(loads, 120 * 2 * 6);
}

fn two_libraries_sharing_a_crate_name() -> Workspace {
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"one\", \"two\", \"three\"]\n",
        ),
        (
            "one/Cargo.toml",
            "[package]\nname = \"p-one\"\nedition = \"2021\"\n\n[lib]\nname = \"shared\"\n",
        ),
        (
            "one/src/lib.rs",
            "pub mod m { pub struct X; pub fn only_one() {} }\n",
        ),
        (
            "two/Cargo.toml",
            "[package]\nname = \"p-two\"\nedition = \"2021\"\n\n[lib]\nname = \"shared\"\n",
        ),
        (
            "two/src/lib.rs",
            "pub mod m { pub struct X; pub fn y() {} }\n",
        ),
        (
            "three/Cargo.toml",
            "[package]\nname = \"p-three\"\nedition = \"2021\"\n\n[lib]\nname = \"alone\"\n",
        ),
        (
            "three/src/lib.rs",
            "pub mod m { pub fn go() {} pub struct X; }\npub mod n {}\n",
        ),
    ]
    .into_iter()
    .collect();
    Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn a_module_of_a_crate_name_two_libraries_share_is_ambiguous_to_a_reader_as_to_a_resolve() {
    let ws = two_libraries_sharing_a_crate_name();
    let roots = |error: ResolveError| match error {
        ResolveError::Ambiguous(ids) => ids
            .iter()
            .map(|id| ws.item(*id).expect("item").file.to_string())
            .collect::<Vec<_>>(),
        other => panic!("{other:?}"),
    };
    let expected = ["one/src/lib.rs", "two/src/lib.rs"];
    // The canonical path, and the reads from a module of either crate, a
    // name that only one of them has included, answer with both crates.
    for path in ["shared::m::X", "shared::m::only_one", "shared::m::y"] {
        assert_eq!(roots(ws.resolve(path).expect_err(path)), expected, "{path}");
    }
    for (module, path) in [
        ("shared::m", "self::X"),
        ("shared::m", "self::only_one"),
        ("shared::m", "self::y"),
        ("shared::m", "crate::m::X"),
        ("shared", "self::m::X"),
        ("shared::gone", "self::X"),
    ] {
        assert_eq!(
            roots(ws.resolve_from(module, path).expect_err(path)),
            expected,
            "{module} {path}"
        );
        for ns in Namespace::ALL {
            assert_eq!(
                ws.resolve_from_in(module, path, ns),
                ws.resolve_from(module, path),
                "{module} {path} {ns:?}"
            );
        }
    }
    // The neighbours: a module of a crate no other library shares reads as
    // before, and a module that no crate has is unknown.
    assert_eq!(
        from(&ws, "alone::m", "self::go"),
        Ok("alone::m::go".to_owned())
    );
    assert_eq!(
        from(&ws, "alone::n", "crate::m::X"),
        Ok("alone::m::X".to_owned())
    );
    assert_eq!(
        ws.resolve_from("nothing::m", "self::X"),
        Err(ResolveError::UnknownModule("nothing::m".to_owned()))
    );
    assert_eq!(
        ws.resolve_from("alone::gone", "self::X"),
        Err(ResolveError::UnknownModule("alone::gone".to_owned()))
    );
    // The package names which crate is meant for a canonical path.
    assert_eq!(
        ws.resolve_in_package("p-two", "shared::m::y")
            .map(|found| found.path),
        Ok("shared::m::y".to_owned())
    );
}

#[test]
fn definitions_and_imports_of_one_name_under_cfg_are_alternatives_at_any_segment_of_a_path() {
    // A module, or an import of one, that exists twice under `cfg` is not an
    // ambiguity between globs: a path goes through every alternative, at the
    // start and in the middle, as a canonical path and as a `use` path.
    let ws = lib(concat!(
        "pub mod a {\n",
        "    #[cfg(unix)] pub mod sys { pub fn common() {} }\n",
        "    #[cfg(windows)] pub mod sys { pub fn common() {} pub fn extra() {} }\n",
        "}\n",
        "#[cfg(unix)] pub mod b { pub fn f() {} }\n",
        "#[cfg(windows)] pub mod b { pub fn f() {} }\n",
        "mod u { pub fn only_u() {} }\n",
        "mod w { pub fn only_w() {} }\n",
        "mod c {\n",
        "    #[cfg(unix)] use crate::u as sys;\n",
        "    #[cfg(windows)] use crate::w as sys;\n",
        "}\n",
    ));
    for (path, expected) in [
        ("demo::a::sys::common", 2),
        ("demo::a::sys::extra", 1),
        ("demo::b::f", 2),
    ] {
        assert_eq!(
            ws.resolve(path).map(|found| found.sites.len()),
            Ok(expected),
            "{path}"
        );
    }
    for (module, path, expected) in [
        ("demo::a", "sys::common", 2),
        ("demo", "a::sys::common", 2),
        ("demo", "crate::a::sys::extra", 1),
        ("demo", "b::f", 2),
        ("demo", "crate::b::f", 2),
        ("demo::c", "sys::only_u", 1),
        ("demo::c", "sys::only_w", 1),
    ] {
        assert_eq!(
            ws.resolve_from(module, path).map(|found| found.sites.len()),
            Ok(expected),
            "{module} {path}"
        );
    }
}
