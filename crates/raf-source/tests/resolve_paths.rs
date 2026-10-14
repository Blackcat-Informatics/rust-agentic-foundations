// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! What `resolve` accepts as a canonical path, and what it answers for a path
//! whose lookups read a glob that names nothing.

use std::time::{Duration, Instant};

use raf_source::{MemoryTree, Namespace, ResolveError, Workspace};

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

#[test]
fn a_path_of_fifty_thousand_segments_resolves_in_linear_time() {
    let ws = lib("pub mod m { pub use crate::m; pub fn end() {} }\n");
    let missing = format!("demo{}", "::a".repeat(50_000));
    let deep = format!("demo{}::end", "::m".repeat(50_000));
    let started = Instant::now();
    assert_eq!(
        ws.resolve(&missing),
        Err(ResolveError::NotFound(missing.clone()))
    );
    assert_eq!(path_of(&ws, &deep), "demo::m::end");
    let took = started.elapsed();
    // Under 100 ms in a release build; a debug build is several times slower.
    let bound = if cfg!(debug_assertions) {
        Duration::from_secs(1)
    } else {
        Duration::from_millis(100)
    };
    assert!(took < bound, "took {took:?}");
}
