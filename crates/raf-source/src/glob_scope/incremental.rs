// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The covert reads of the glob tables and the changes their views take as
//! they come, held to the answers of the reference mode (see
//! [`reference`](super::reference)): each fixture is a crate whose answers
//! one of those paths could get wrong, loaded in every order of its
//! declarations in both modes, its answers compared and the expected ones
//! asserted. One fixture follows the tables through the steps the settling
//! makes, to show that a covert read is listed before its names arrive.

use super::{Index, Level, Nobody, reference};
use crate::item::ItemId;
use crate::meter::Meter;
use crate::options::LoadOptions;
use crate::{MemoryTree, Namespace, ResolveError, Workspace};

/// The manifest of each crate of the fixtures.
const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

/// Every order of `blocks`.
fn permutations(blocks: &[&str]) -> Vec<Vec<String>> {
    if blocks.len() <= 1 {
        return vec![blocks.iter().map(|block| (*block).to_owned()).collect()];
    }
    let mut all = Vec::new();
    for (at, first) in blocks.iter().enumerate() {
        let mut rest = blocks.to_vec();
        rest.remove(at);
        for mut tail in permutations(&rest) {
            tail.insert(0, (*first).to_owned());
            all.push(tail);
        }
    }
    all
}

/// What `self::<name>` resolves to in each module of a load of `lib`, for
/// each name the text holds, in each name space: one line for each, with the
/// path of the item, or of each item of an ambiguity, or the error.
fn every_answer(lib: &str) -> Vec<String> {
    let tree: MemoryTree = [("Cargo.toml", MANIFEST), ("src/lib.rs", lib)]
        .into_iter()
        .collect();
    let ws = Workspace::load(&tree, &["."]).unwrap_or_else(|error| panic!("{lib}: {error}"));
    let mut names: Vec<&str> = lib
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .filter(|word| word.starts_with(|ch: char| ch.is_ascii_uppercase()))
        .collect();
    names.sort_unstable();
    names.dedup();
    let mut answers = Vec::new();
    for module in &ws.index().modules {
        for name in &names {
            for ns in [Namespace::Type, Namespace::Value, Namespace::Macro] {
                let found = ws.resolve_from_in(&module.path, &format!("self::{name}"), ns);
                answers.push(format!(
                    "{} {name} {ns:?}: {}",
                    module.path,
                    shown(&ws, found)
                ));
            }
        }
    }
    answers
}

/// An answer as the fixtures print it: the path of the item, or of each item
/// of an ambiguity, or the error.
fn shown(ws: &Workspace, found: Result<crate::Resolution, ResolveError>) -> String {
    match found {
        Ok(found) => format!("ok {}", found.path),
        Err(ResolveError::Ambiguous(ids)) => {
            let paths: Vec<&str> = ids
                .iter()
                .map(|id| ws.item(*id).expect("an item of the load").path.as_str())
                .collect();
            format!("ambiguous {}", paths.join(" "))
        }
        Err(error) => format!("err {error:?}"),
    }
}

/// What each of `probes`, a module and a path, resolves to in the type name
/// space in a load of `lib`: the path of the item, or of each item of an
/// ambiguity, or the error.
fn answers(lib: &str, probes: &[(&str, &str)]) -> Vec<String> {
    let tree: MemoryTree = [("Cargo.toml", MANIFEST), ("src/lib.rs", lib)]
        .into_iter()
        .collect();
    let ws = Workspace::load(&tree, &["."]).unwrap_or_else(|error| panic!("{lib}: {error}"));
    probes
        .iter()
        .map(|(module, path)| shown(&ws, ws.resolve_from_in(module, path, Namespace::Type)))
        .collect()
}

/// Load each of `variants`, the blocks of one crate, in every order, out of
/// the reference mode and in it, and assert that `probes` answer the same in
/// both and as `expected` says. Returns the loads made in each mode.
fn every_order(variants: &[&[&str]], probes: &[(&str, &str)], expected: &[&str]) -> usize {
    let mut loads = 0;
    for blocks in variants {
        for order in permutations(blocks) {
            let lib = order.concat();
            let settled = answers(&lib, probes);
            let exact = reference::exact(|| answers(&lib, probes));
            assert_eq!(settled, exact, "the two modes differ on\n{lib}");
            assert_eq!(settled, expected, "{lib}");
            loads += 1;
        }
    }
    loads
}

#[test]
fn a_module_whose_covert_read_turns_active_is_no_longer_looked_at_as_a_covert_reader_when_the_view_grows()
 {
    // `a::m` globs `t` at `pub(super)`, which the view of `m` for `r`, a
    // module outside `a`, cannot access: the read is covert while it is the
    // only view, and while `t`'s view is empty the view of `m` does not know
    // what it cannot access, so `m` is an open reader of `t`'s view. The
    // view of `m` for `q`, inside `a`, can access the glob, so the read
    // turns active: `m` leaves the covert readers of `t`'s view, and its
    // open readers too, beside `b::n`, a covert reader that stays. When
    // `t`'s view gains `X`, it reaches `m` through the active read only.
    let blocks: &[&str] = &[
        "pub mod r { pub use crate::a::m::*; }\n",
        "pub mod s { pub use crate::b::n::*; }\n",
        "pub mod a { pub mod m { pub(super) use crate::t::*; } pub mod q { pub use super::m::*; } }\n",
        "pub mod b { pub mod n { pub(super) use crate::t::*; } }\n",
        "pub mod t { pub use crate::d::*; }\n",
        "pub mod d { pub struct X; }\n",
    ];
    let loads = every_order(
        &[blocks],
        &[
            ("demo::a::q", "self::X"),
            ("demo::a::m", "self::X"),
            ("demo::r", "self::X"),
            ("demo::s", "self::X"),
        ],
        &[
            "ok demo::d::X",
            "ok demo::d::X",
            "err NotFound(\"self::X\")",
            "err NotFound(\"self::X\")",
        ],
    );
    assert_eq!(loads, 720);
}

#[test]
fn an_item_that_a_view_read_through_a_glob_its_class_cannot_access_holds_makes_the_first_item_it_can_access_under_the_name_ambiguous()
 {
    // The view of `t` for `u` knows that `t`'s private glob of `h` brings in
    // what `u` cannot access before `d::X` first arrives: it reads `X` again
    // from the globs, as `h`'s view holds the name, and `u` reads `X` as
    // ambiguous, as rustc keeps the ambiguity of two globs; without `h`,
    // whose view never holds `X`, it takes `d::X` as it comes.
    let probes = &[("demo::u", "self::X")];
    let expected = &["ambiguous demo::d::X demo::h::X"];
    let loads = every_order(
        &[
            &[
                "pub mod u { pub use crate::t::*; }\n",
                "pub mod t { use crate::h::*; pub use crate::d::*; }\n",
                "pub mod h { pub struct X; }\n",
                "pub mod d { pub struct X; }\n",
            ],
            &[
                "pub mod u { pub use crate::t::*; }\n",
                "pub mod t { pub use crate::d::*; use crate::h::*; }\n",
                "pub mod h { pub struct X; }\n",
                "pub mod d { pub struct X; }\n",
            ],
        ],
        probes,
        expected,
    );
    assert_eq!(loads, 48);
    every_order(
        &[&[
            "pub mod u { pub use crate::t::*; }\n",
            "pub mod t { use crate::h::*; pub use crate::d::*; }\n",
            "pub mod h { pub struct Y; }\n",
            "pub mod d { pub struct X; }\n",
        ]],
        probes,
        &["ok demo::d::X"],
    );
}

#[test]
fn an_item_its_class_cannot_access_that_arrived_under_a_name_its_view_did_not_hold_makes_the_first_item_it_can_access_ambiguous()
 {
    // `a::s::X` is `pub(super)`, so `a::m`'s `pub` glob of `s` brings it in
    // at a level that `r`, outside `a`, cannot access: arriving before
    // `d::X`, it finds nothing to stand beside in `m`'s view for `r`, which
    // records the name, and `d::X` then reads `X` again from the globs, so
    // `r` reads it as ambiguous; arriving after, it is exposed beside
    // `d::X`. Where `m` reaches `d` through `e`, `d::X` can come after the
    // view is made from what `s` brought in, which records the name as it
    // gathers. Without `s::X`, `d::X` is taken as it comes.
    let probes = &[("demo::r", "self::X"), ("demo::a::m", "self::X")];
    let expected = &[
        "ambiguous demo::a::s::X demo::d::X",
        "ambiguous demo::a::s::X demo::d::X",
    ];
    let loads = every_order(
        &[
            &[
                "pub mod r { pub use crate::a::m::*; }\n",
                "pub mod a { pub mod s { pub(super) struct X; } pub mod m { pub use super::s::*; pub use crate::d::*; } }\n",
                "pub mod d { pub struct X; }\n",
            ],
            &[
                "pub mod r { pub use crate::a::m::*; }\n",
                "pub mod a { pub mod m { pub use crate::d::*; pub use super::s::*; } pub mod s { pub(super) struct X; } }\n",
                "pub mod d { pub struct X; }\n",
            ],
            &[
                "pub mod r { pub use crate::a::m::*; }\n",
                "pub mod a { pub mod s { pub(super) struct X; } pub mod m { pub use super::s::*; pub use crate::e::*; } }\n",
                "pub mod e { pub use crate::d::*; }\n",
                "pub mod d { pub struct X; }\n",
            ],
        ],
        probes,
        expected,
    );
    assert_eq!(loads, 36);
    every_order(
        &[&[
            "pub mod r { pub use crate::a::m::*; }\n",
            "pub mod a { pub mod s { pub(super) struct Y; } pub mod m { pub use super::s::*; pub use crate::d::*; } }\n",
            "pub mod d { pub struct X; }\n",
        ]],
        &[("demo::r", "self::X")],
        &["ok demo::d::X"],
    );
}

/// The module at `path` of the crate of `index`.
fn module(index: &Index, path: &str) -> usize {
    index
        .modules
        .iter()
        .position(|module| module.path == path)
        .unwrap_or_else(|| panic!("a module {path}"))
}

/// The item at `path`.
fn item(index: &Index, path: &str) -> ItemId {
    index
        .items
        .iter()
        .find(|item| item.path == path)
        .unwrap_or_else(|| panic!("an item {path}"))
        .id
}

/// The glob import of `from` that reaches `to`, and `to`.
fn glob(index: &Index, from: &str, to: &str) -> (usize, usize) {
    let (from, to) = (module(index, from), module(index, to));
    let glob = index
        .glob_imports
        .iter()
        .copied()
        .find(|position| {
            index.uses[*position].module == from && index.glob_reach[*position].contains(&to)
        })
        .unwrap_or_else(|| panic!("a glob of module {from} that reaches module {to}"));
    (glob, to)
}

/// The items the view of the module at `path` holds under `P` in the type
/// name space, with whether each is a companion.
fn held(index: &Index, path: &str) -> Vec<(ItemId, bool)> {
    let view = index.globs.exports[module(index, path)][0];
    index.globs.views[view]
        .names
        .get("P")
        .map_or_else(Vec::new, |lists| {
            lists[0]
                .iter()
                .map(|(id, level): &(ItemId, Level)| (*id, level.is_companion()))
                .collect()
        })
}

#[test]
fn a_covert_read_is_listed_before_its_names_arrive_so_an_item_its_view_gains_while_they_arrive_reaches_the_module()
 {
    // The tables are built read by read, as the settling builds them while
    // `y`'s import of the function `P` is still undecided in the type name
    // space. `y` is kept from its globs under `P` there while its private
    // glob of `z` brings `z::P` in, so its view for `t` does not learn that
    // it brings in what `t` cannot access; once the import is decided, the
    // name is read again with nothing `t` can access under it, and `s::P`
    // then arrives at that view through `m` and is taken as it comes. `m`'s
    // private glob of `t` is covert: `t`'s view holds `P` beside the view of
    // `m`, so `w::P` arrives as a companion of `s::P`. Passed on through
    // `y`, it reads `P` again there, which exposes `z::P`, and that reaches
    // `t`'s view while `m`'s covert read still passes its names on: `m`
    // holds it only because the read was listed among the covert readers of
    // `t`'s view first. `s::P` and `w::P` reach `m` and `t` through modules
    // that do not bind `P` themselves, so no companion is dropped on the way
    // (see [`Index::redelivered`]).
    let lib = "pub mod m { use crate::t::*; pub use crate::s2::*; }\n\
               pub mod y { pub use crate::m::*; use crate::z::*; use crate::f::P; }\n\
               pub mod t { pub use crate::y::*; pub use crate::v::*; }\n\
               pub mod s2 { pub use crate::s::*; }\npub mod v { pub use crate::w::*; }\n\
               pub mod s { pub struct P; }\npub mod z { pub struct P; }\n\
               pub mod w { pub struct P; }\npub mod f { pub fn P() {} }\n";
    let tree: MemoryTree = [("Cargo.toml", MANIFEST), ("src/lib.rs", lib)]
        .into_iter()
        .collect();
    let loaded = Workspace::load(&tree, &["."])
        .expect("loads")
        .index()
        .clone();
    let (s, z, w) = (
        item(&loaded, "demo::s::P"),
        item(&loaded, "demo::z::P"),
        item(&loaded, "demo::w::P"),
    );
    let settled = built(&loaded, s);
    let exact = reference::exact(|| built(&loaded, s));
    // Out of the reference mode the read is covert, and in it active.
    assert!(settled.covert && !exact.covert);
    assert_eq!(settled.y, exact.y);
    assert_eq!(settled.m, exact.m);
    assert!(settled.y.contains(&(z, true)), "{:?}", settled.y);
    let mut expected = vec![(s, false), (z, true), (w, true)];
    expected.sort_unstable();
    assert_eq!(settled.m, expected);
}

/// What the views of `y` and `m` of the fixture above hold under `P` once
/// `m` has read `t`, and whether that read is covert.
struct Built {
    y: Vec<(ItemId, bool)>,
    m: Vec<(ItemId, bool)>,
    covert: bool,
}

/// Build the tables of the fixture above from `loaded`, read by read, and
/// check the state before `m` reads `t`, where only `s` stands under `P`
/// in the views of `y` and `m`.
fn built(loaded: &Index, s: ItemId) -> Built {
    let mut index = loaded.clone();
    let meter = Meter::new(&LoadOptions::default());
    // As the first settle of the load builds them: no drop is recorded yet.
    index.clear_globs();
    index.globs.drops.clear();
    index.globs.dropped = 0;
    index
        .reset_globs(&meter)
        .expect("the tables are built again");
    let import = index.imports_of(module(&index, "demo::y"), "P")[0];
    let read = |index: &mut Index, from: &str, to: &str| {
        let (glob, target) = glob(index, from, to);
        index
            .read_glob(glob, target, &mut Nobody, &meter)
            .expect("read");
    };
    // The import of `P` in `y` is undecided in the type name space.
    index.use_free[import][0] = false;
    read(&mut index, "demo::y", "demo::m");
    read(&mut index, "demo::t", "demo::y");
    read(&mut index, "demo::v", "demo::w");
    read(&mut index, "demo::t", "demo::v");
    read(&mut index, "demo::s2", "demo::s");
    read(&mut index, "demo::y", "demo::z");
    index.use_free[import][0] = true;
    index.use_decided[import][0] = true;
    index
        .refresh(module(&index, "demo::y"), "P", &mut Nobody, &meter)
        .expect("read again");
    read(&mut index, "demo::m", "demo::s2");
    assert_eq!(held(&index, "demo::y"), vec![(s, false)]);
    assert_eq!(held(&index, "demo::m"), vec![(s, false)]);
    read(&mut index, "demo::m", "demo::t");
    let t_view = index.globs.exports[module(&index, "demo::t")][0];
    Built {
        y: held(&index, "demo::y"),
        m: held(&index, "demo::m"),
        covert: index.globs.views[t_view]
            .unseen()
            .covert
            .contains_key(&module(&index, "demo::m")),
    }
}

#[test]
fn a_view_made_when_its_module_has_left_the_open_readers_makes_it_open_again_until_it_learns_what_it_cannot_access()
 {
    // `a::b::m` reads `t` through a `pub(super)` glob, which neither its
    // view for `r`, outside `a`, nor its view for `a::q` can access. The
    // view for `r` already knows what it cannot access, from `a::x::Z`, so
    // the covert read leaves `m` out of the open readers of `t`'s view; the
    // view for `q`, which can access `Z`, does not, as `t`'s view holds only
    // `K`, which `m` binds, so it makes `m` an open reader again. When `t`'s
    // view gains `y::Y`, the views that hold `Y` are fewer than the covert
    // readers, the four `n<i>` and `m`, and lead to none of them: `m` is
    // found as an open reader, and its view for `q` learns that `y::Y`
    // stands under `Y` where it cannot access it. So `w::Y`, arriving
    // through `e` later, is read as ambiguous by `q` as by `r`.
    let blocks: &[&str] = &[
        "pub mod r { pub use crate::a::b::m::*; }\n\
         pub mod a { pub mod x { pub(super) struct Z; } \
         pub mod b { pub mod m { pub use crate::a::x::*; pub(super) use crate::t::*; \
         pub use crate::e::*; pub struct K; } } pub mod q { pub use super::b::m::*; } }\n",
        "pub mod n { pub mod n1 { pub(super) use crate::t::*; } pub mod n2 { pub(super) use crate::t::*; } \
         pub mod n3 { pub(super) use crate::t::*; } pub mod n4 { pub(super) use crate::t::*; } }\n\
         pub mod o { pub use crate::n::n1::*; pub use crate::n::n2::*; pub use crate::n::n3::*; \
         pub use crate::n::n4::*; }\n",
        "pub mod t { pub struct K; pub use crate::g::*; }\npub mod g { pub use crate::y::*; }\n\
         pub mod y { pub struct Y; }\n",
        "pub mod e { pub use crate::w::*; }\npub mod w { pub struct Y; }\n",
    ];
    let loads = every_order(
        &[blocks],
        &[("demo::a::q", "self::Y"), ("demo::r", "self::Y")],
        &[
            "ambiguous demo::w::Y demo::y::Y",
            "ambiguous demo::w::Y demo::y::Y",
        ],
    );
    assert_eq!(loads, 24);
}

/// Assert that every answer of a load of `lib` is the reference mode's, and
/// that `module` reads `self::<name>` in the name space `ns` as `expected`.
fn as_the_reference_mode(lib: &str, (module, name, ns): (&str, &str, Namespace), expected: &str) {
    let settled = every_answer(lib);
    let exact = reference::exact(|| every_answer(lib));
    let differ: Vec<String> = settled
        .iter()
        .zip(&exact)
        .filter(|(left, right)| left != right)
        .map(|(left, right)| format!("{left}\n  in the reference mode {right}"))
        .collect();
    assert!(differ.is_empty(), "{}", differ.join("\n"));
    assert_eq!(settled.len(), exact.len());
    let probe = format!("{module} {name} {ns:?}: ");
    let found = settled
        .iter()
        .find_map(|answer| answer.strip_prefix(probe.as_str()))
        .unwrap_or_else(|| panic!("no answer for {probe}"));
    assert_eq!(found, expected, "{probe}");
}

#[test]
fn a_covert_reader_is_woken_in_the_turn_an_active_read_of_it_would_have_so_the_imports_that_wait_on_the_name_are_queued_again_in_that_order()
 {
    // A crate of globs in cycles and imports that rename through them, in
    // an order of its declarations in which waking a covert reader only
    // after every active reader of a gain, and not in its turn, queues the
    // imports that wait on a name again in another order, which decides
    // them otherwise: `D` is then one item, `m6::s1::D`, where it is
    // ambiguous among three.
    let lib = "pub(crate) struct A;\n\
         mod m5 {\n\
         pub(crate) mod s0 {\n\
         pub use crate::m3::s1::*;\n\
         pub(in crate) use crate::m3::*;\n\
         pub use crate::m6::s0::*;\n\
         use crate::m7::s0::*;\n\
         }\n\
         pub mod s1 {\n\
         use crate::m6::s0::*;\n\
         pub(crate) use crate::m10::*;\n\
         }\n\
         }\n\
         mod m7 {\n\
         pub mod s0 {\n\
         pub(super) use crate::m3::*;\n\
         pub use crate::m0::*;\n\
         }\n\
         }\n\
         pub(crate) mod m9 {\n\
         pub use crate::m5::s0::A as D;\n\
         }\n\
         pub mod m4 {\n\
         pub(super) use crate::m7::s0::A as A;\n\
         }\n\
         pub(crate) mod m3 {\n\
         pub(in crate) use crate::m4::*;\n\
         pub mod s1 {\n\
         pub(crate) struct A;\n\
         }\n\
         pub use crate::m5::s0::*;\n\
         }\n\
         pub mod m6 {\n\
         pub mod s0 {\n\
         pub use crate::m5::s1::D;\n\
         }\n\
         pub mod s1 {\n\
         pub(crate) struct D;\n\
         }\n\
         }\n\
         pub mod m10 {\n\
         pub use crate::m3::*;\n\
         pub use crate::m6::s1::*;\n\
         pub(in crate::m10) use crate::m9::*;\n\
         use crate::*;\n\
         }\n\
         mod m0 {\n\
         pub use crate::m10::*;\n\
         }\n";
    as_the_reference_mode(
        lib,
        ("demo::m5::s0", "D", Namespace::Type),
        "ambiguous demo::A demo::m3::s1::A demo::m6::s1::D",
    );
}

#[test]
fn a_covert_reader_takes_a_gain_in_the_turn_an_active_read_of_it_would_have_so_the_companions_of_a_name_are_kept_as_its_drops_allow()
 {
    // Globs of two macros of one name in cycles, in an order of their
    // declarations in which a gain arriving through a covert read only
    // after the gains through every active reader, and not in its turn,
    // loses a companion of `D` on the way, as the reading again of a name
    // with a companion reads the views as they stand when it runs: `m7::s0`
    // then reads `D` as one macro, `m13::s0::D`, where it is ambiguous.
    let lib = "pub mod m3 {\n\
         pub(crate) use crate::m14::*;\n\
         }\n\
         pub mod m4 {\n\
         mod s1 {\n\
         use super::*;\n\
         pub use crate::m4::s0::*;\n\
         }\n\
         pub(in crate::m4) use crate::m3::*;\n\
         pub(super) use crate::m4::s1::*;\n\
         pub(crate) mod s0 {\n\
         pub(in crate) use crate::m13::s0::*;\n\
         pub(crate) use crate::m4::*;\n\
         }\n\
         }\n\
         mod m10 {\n\
         pub use crate::m4::s0::*;\n\
         }\n\
         pub(crate) mod m7 {\n\
         pub(crate) mod s0 {\n\
         pub(in crate::m7) use crate::m13::s0::*;\n\
         use crate::m10::*;\n\
         }\n\
         }\n\
         mod m6 {\n\
         pub mod s0 {\n\
         macro_rules! D { () => {} }\n\
         }\n\
         pub use crate::m14::s0::*;\n\
         }\n\
         mod m13 {\n\
         pub(crate) mod s0 {\n\
         macro_rules! D { () => {} }\n\
         }\n\
         }\n\
         mod m14 {\n\
         pub use crate::m6::*;\n\
         }\n";
    as_the_reference_mode(
        lib,
        ("demo::m7::s0", "D", Namespace::Macro),
        "ambiguous demo::m13::s0::D demo::m6::s0::D",
    );
}
