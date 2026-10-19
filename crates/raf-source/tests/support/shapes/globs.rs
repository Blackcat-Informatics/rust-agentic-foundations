// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The shapes of glob imports whose settling the work counts and the
//! instruction counts measure, as generators of the source of one library
//! crate named `demo`, each of a size `n`.
//!
//! The file names nothing of `raf_source`, so the unit tests of the glob
//! tables, the integration tests and the examples include the same text.

use std::fmt::Write as _;

/// The manifest of the crate each shape is the library of.
pub(crate) const MANIFEST: &str =
    "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

/// One shape: its name, the source of its library at a size, and the paths
/// a load of it is asked to resolve.
pub(crate) struct GlobShape {
    /// The name the shape is selected by.
    pub(crate) name: &'static str,
    /// The source of `src/lib.rs` at a size.
    pub(crate) text: fn(usize) -> String,
    /// The paths resolved after a load at a size.
    pub(crate) paths: fn(usize) -> Vec<String>,
}

/// Every shape, in a stable order.
pub(crate) const ALL: [GlobShape; 9] = [
    GlobShape {
        name: "chain",
        text: chain,
        paths: |n| vec!["demo::m0::f".to_owned(), format!("demo::m{}::f", n / 2)],
    },
    GlobShape {
        name: "many-items",
        text: many_items,
        paths: |n| {
            vec![
                "demo::g0::f0".to_owned(),
                format!("demo::g{0}::f{0}", n - 1),
            ]
        },
    },
    GlobShape {
        name: "pub-fan-in",
        text: pub_fan_in,
        paths: |n| vec!["demo::u::T0".to_owned(), format!("demo::t::T{}", n - 1)],
    },
    GlobShape {
        name: "private-fan-in",
        text: private_fan_in,
        paths: |_| vec!["demo::u::T0".to_owned(), "demo::m0::T0".to_owned()],
    },
    GlobShape {
        name: "companion-fan-in",
        text: companion_fan_in,
        paths: |_| vec!["demo::u::X0".to_owned(), "demo::t::X1".to_owned()],
    },
    GlobShape {
        name: "prelude",
        text: prelude,
        paths: |n| {
            vec![
                "demo::T0".to_owned(),
                format!("demo::T{}", n - 1),
                "demo::c1::T0".to_owned(),
            ]
        },
    },
    GlobShape {
        name: "hub",
        text: hub,
        paths: |_| {
            vec![
                "demo::hub::T0".to_owned(),
                "demo::s1::T2".to_owned(),
                "demo::hub::T1".to_owned(),
            ]
        },
    },
    GlobShape {
        name: "ambiguous-hub",
        text: ambiguous_hub,
        paths: |_| {
            vec![
                "demo::hub::T0".to_owned(),
                "demo::s2::T0".to_owned(),
                "demo::s0::T1".to_owned(),
            ]
        },
    },
    GlobShape {
        name: "lookups",
        text: lookups,
        paths: |n| {
            vec![
                "demo::d::X".to_owned(),
                "demo::g0::X".to_owned(),
                format!("demo::g{}::X", n - 1),
            ]
        },
    },
];

/// The shape named `name`, if there is one.
pub(crate) fn named(name: &str) -> Option<&'static GlobShape> {
    ALL.iter().find(|shape| shape.name == name)
}

/// A chain of `n` globs: each module globs the next, and the last defines
/// one function.
pub(crate) fn chain(n: usize) -> String {
    let mut lib = String::new();
    for i in 0..n {
        writeln!(lib, "mod m{i} {{ pub use super::m{}::*; }}", i + 1).expect("write");
    }
    writeln!(lib, "pub mod m{n} {{ pub fn f() {{}} }}").expect("write");
    lib
}

/// A module of `n` public functions, which each of `n` modules globs
/// publicly.
pub(crate) fn many_items(n: usize) -> String {
    let mut lib = String::from("pub mod d {");
    for i in 0..n {
        write!(lib, " pub fn f{i}() {{}}").expect("write");
    }
    lib.push_str(" }\n");
    for i in 0..n {
        writeln!(lib, "pub mod g{i} {{ pub use crate::d::*; }}").expect("write");
    }
    lib
}

/// A fan-in of `n` `pub` globs onto `t`, each of a module of one struct,
/// and `u`, declared after `t`, globbing `t`.
pub(crate) fn pub_fan_in(n: usize) -> String {
    fan_in(n, "pub ")
}

/// A fan-in of `n` private globs onto `t`, each of a module of one struct,
/// and `u`, declared after `t`, globbing `t`.
pub(crate) fn private_fan_in(n: usize) -> String {
    fan_in(n, "")
}

fn fan_in(n: usize, vis: &str) -> String {
    let mut lib = String::from("pub mod t {");
    for i in 0..n {
        write!(lib, " {vis}use crate::m{i}::*;").expect("write");
    }
    lib.push_str(" }\npub mod u { pub use crate::t::*; }\n");
    for i in 0..n {
        writeln!(lib, "pub mod m{i} {{ pub struct T{i}; }}").expect("write");
    }
    lib
}

/// A fan-in of `n` `pub` globs onto `t`, each of a module `a<i>` that brings
/// in a struct `X<i>` through a `pub` glob and another struct `X<i>`
/// through a private one, so that each carries its name with a companion,
/// and `u` globbing `t`.
pub(crate) fn companion_fan_in(n: usize) -> String {
    let mut lib = String::from("pub mod t {");
    for i in 0..n {
        write!(lib, " pub use crate::a{i}::*;").expect("write");
    }
    lib.push_str(" }\npub mod u { pub use crate::t::*; }\n");
    for i in 0..n {
        writeln!(
            lib,
            "pub mod a{i} {{ pub use crate::d{i}::*; use crate::p{i}::*; }}\n\
             pub mod d{i} {{ pub struct X{i}; }}\npub mod p{i} {{ pub struct X{i}; }}"
        )
        .expect("write");
    }
    lib
}

/// The prelude pattern: the crate root globs each of `n` children, and each
/// child, which defines one struct, globs its parent.
pub(crate) fn prelude(n: usize) -> String {
    let mut lib = String::new();
    for i in 0..n {
        writeln!(
            lib,
            "pub use c{i}::*;\npub mod c{i} {{ use super::*; pub struct T{i}; }}"
        )
        .expect("write");
    }
    lib
}

/// A hub that `n` sibling modules glob and that globs each of them back,
/// every other one privately; each sibling defines one struct.
pub(crate) fn hub(n: usize) -> String {
    hub_of(n, 1)
}

/// The hub of [`hub`], where each two siblings define a struct of one name:
/// the hub brings each name in from a sibling it globs publicly and from one
/// it globs privately, so every name the siblings read through the hub
/// carries a companion.
pub(crate) fn ambiguous_hub(n: usize) -> String {
    hub_of(n, 2)
}

/// A hub of `n` siblings, the sibling `i` defining the struct `T<i / per>`.
fn hub_of(n: usize, per: usize) -> String {
    let mut lib = String::from("pub mod hub {");
    for i in 0..n {
        let vis = if i % 2 == 0 { "pub " } else { "" };
        write!(lib, " {vis}use crate::s{i}::*;").expect("write");
    }
    lib.push_str(" }\n");
    for i in 0..n {
        writeln!(
            lib,
            "pub mod s{i} {{ pub use crate::hub::*; pub struct T{}; }}",
            i / per
        )
        .expect("write");
    }
    lib
}

/// A module `t`, declared first, with `n` private globs of modules that
/// each glob `d`, which defines `X`, and `n` named imports of `X` through
/// them: each import looks the name up in what the globs of `t` bring in.
/// The imports are private, as in the reproducer: rustc refuses to
/// re-export a name that private globs bring in.
pub(crate) fn lookups(n: usize) -> String {
    let mut lib = String::from("pub mod t {");
    for i in 0..n {
        write!(lib, " use crate::g{i}::*;").expect("write");
    }
    for k in 0..n {
        write!(lib, " use self::X as Y{k};").expect("write");
    }
    lib.push_str(" }\npub mod d { pub struct X; }\n");
    for i in 0..n {
        writeln!(lib, "pub mod g{i} {{ pub use crate::d::*; }}").expect("write");
    }
    lib
}
