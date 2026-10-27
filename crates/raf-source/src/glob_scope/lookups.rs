// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The lookup steps of what the globs of a module bring in, charged to the
//! work budgets: each glob read a lookup looks at, and each view that holds
//! the name looked at to find those reads. During the load they are counted
//! as they are made and charged at its checkpoints, so the shape of many
//! named imports through many globs is refused by a small limit at the
//! first checkpoint past it; after the load each query counts its own
//! against `max_query_work`, from zero. Nothing here is timed: every count
//! is a count of steps.

use super::shapes::{MANIFEST, holder_scan, lookups, prelude};
use crate::meter::charges;
use crate::work::{Tally, Work, measure};
use crate::{
    Budget, ExportError, LoadError, LoadErrorKind, LoadOptions, MemoryTree, RelPath, ResolveError,
    Workspace,
};

/// The library of one package whose source is `lib`.
fn tree(lib: &str) -> MemoryTree {
    [("Cargo.toml", MANIFEST), ("src/lib.rs", lib)]
        .into_iter()
        .collect()
}

/// Load `lib` with `options`.
fn load_with(lib: &str, options: &LoadOptions) -> Result<Workspace, LoadError> {
    Workspace::load_with(&tree(lib), &["."], options)
}

/// Load `lib` with the work of the load limited to `max_work`.
fn load(lib: &str, max_work: usize) -> Result<Workspace, LoadError> {
    load_with(
        lib,
        &LoadOptions {
            max_work,
            ..LoadOptions::default()
        },
    )
}

/// Load `lib` with the work of each query limited to `max_query_work`.
fn queried(lib: &str, max_query_work: usize) -> Workspace {
    load_with(
        lib,
        &LoadOptions {
            max_query_work,
            ..LoadOptions::default()
        },
    )
    .unwrap_or_else(|error| panic!("loads: {error}"))
}

/// A glob that brings `A` into `m`, where a private import of `A` that the
/// index cannot follow keeps it from the globs of `m`: a path of `u`, which
/// globs `m`, that reads `A` asks whether the globs of `m` bring `A` in at
/// all, as the import keeps it from every importer only while they do.
const ANY: &str = "pub mod g { pub struct A; }\n\
                   pub mod m { use crate::missing::A; pub use crate::g::*; }\n\
                   pub mod u { use crate::m::*; use A as B; }\n";

/// A public surface whose lookups are made both by the walk of a glob's
/// path (`u` globs `crate::a::c`, and `c` is brought into `a` by a glob)
/// and by the resolving of a public import the index cannot follow (`m`'s
/// import of `A` asks whether the globs of `x` bring `A` in, in each name
/// space).
const SURFACE: &str = "pub mod g { pub struct A; }\n\
                       pub mod x { use other::A; pub use crate::g::*; }\n\
                       pub mod m { pub use crate::x::*; pub use another::A; }\n\
                       pub mod a { pub use crate::b::*; }\n\
                       pub mod b { pub mod c { pub struct X; } }\n\
                       pub mod u { pub use crate::a::c::*; }\n";

/// What `ws`'s debug check of its settled tables counts, which a debug build
/// runs at the end of the settle and which is no work of the load: it is
/// run again on the built index, the same tables, and its tallies are taken
/// out of those of the load. Nothing in a release build.
fn checked(ws: &Workspace) -> Tally {
    #[cfg(debug_assertions)]
    {
        let index = ws.index();
        let ((), tally) = measure(usize::MAX, || {
            index.lookups.built(false);
            index.check_settled();
            index.lookups.take();
            index.lookups.built(true);
        });
        tally
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = ws;
        Tally::default()
    }
}

/// The lookup steps a tally saw made: each glob read looked at and each
/// view looked at to find them.
fn made(tally: &Tally) -> usize {
    tally.get(Work::LookupReads) + tally.get(Work::LookupScans)
}

/// Assert that `error` is the refusal of the work budget at `limit`.
fn refused_at(error: &LoadError, limit: usize) {
    assert!(
        matches!(
            error.kind,
            LoadErrorKind::Budget { budget: Budget::Work, limit: refused } if refused == limit
        ),
        "at {limit}: {error}"
    );
}

#[test]
fn the_lookups_of_many_named_imports_through_many_globs_are_charged_and_refused_by_a_small_limit_at_the_first_checkpoint_past_it()
 {
    // The n named imports of `t` each look `X` up in what the n globs of
    // `t` bring in, every time they are walked: the steps grow with the
    // square of the imports, and the load is charged them beside the steps
    // of its tables.
    for n in [8, 16] {
        let lib = lookups(n);
        charges::take();
        let ws = load(&lib, Budget::Work.limit()).unwrap_or_else(|error| panic!("{n}: {error}"));
        let all = charges::take();
        let total = ws.usage().work;
        assert!(total > 0);
        assert_eq!(all.iter().sum::<usize>(), total, "{n}");
        // Accepted at its work, refused one step below it.
        assert_eq!(
            load(&lib, total).map(|ws| ws.usage().work).ok(),
            Some(total)
        );
        let error = load(&lib, total - 1).expect_err("one step below its work");
        refused_at(&error, total - 1);
        // Include all newly metered import work. A small limit refuses at
        // the first charge past it, with exactly the executed charge prefix.
        let limit = total / 10;
        charges::take();
        let error = load(&lib, limit).expect_err("a small work budget");
        refused_at(&error, limit);
        assert_eq!(error.file.as_str(), "src/lib.rs");
        assert!(
            lib.lines()
                .nth(error.line.expect("responsible import") - 1)
                .is_some_and(|text| text.contains("use ")),
            "{error}"
        );
        let made = charges::take();
        let passing = all
            .iter()
            .scan(0, |sum, charge| {
                *sum += charge;
                Some(*sum)
            })
            .position(|sum| sum > limit)
            .expect("a charge passes the limit");
        assert_eq!(made, all[..=passing], "{n}");
        assert!(made.iter().sum::<usize>() < total, "{n}");
    }
}

#[test]
fn every_limit_below_the_work_of_the_lookups_shape_is_refused_by_the_work_budget_and_by_nothing_else()
 {
    // The lookups answer as they always do until the checkpoint refuses,
    // so no limit makes the settling reach a state its own checks reject:
    // every limit below the work of the load is the refusal of the work
    // budget, and the work itself is accepted.
    let lib = lookups(4);
    let total = load(&lib, Budget::Work.limit())
        .expect("loads")
        .usage()
        .work;
    for limit in 0..total {
        match load(&lib, limit) {
            Err(error) => refused_at(&error, limit),
            Ok(ws) => panic!("accepted at {limit}, below {}", ws.usage().work),
        }
    }
    assert_eq!(
        load(&lib, total).map(|ws| ws.usage().work).ok(),
        Some(total)
    );
}

#[test]
fn every_glob_read_a_lookup_of_the_load_looks_at_is_charged_once() {
    // Each read and view a lookup of the load looks at is a step charged
    // at a checkpoint, once: what the tallies saw the lookups look at, the
    // debug check of the settled tables left out, is what the checkpoints
    // charged. The imports of `ANY` read whether the globs of `m` bring `A`
    // in at all.
    for (shape, lib) in [
        ("lookups", lookups(8)),
        ("whether globs bring in", ANY.to_owned()),
    ] {
        let (ws, tally) = measure(usize::MAX, || {
            Workspace::load(&tree(&lib), &["."]).unwrap_or_else(|error| panic!("{error}"))
        });
        let check = checked(&ws);
        assert!(
            made(&tally) > made(&check),
            "{shape}: the load looks names up"
        );
        assert_eq!(
            tally.get(Work::LookupCharges),
            made(&tally) - made(&check),
            "{shape}"
        );
        if shape != "lookups" {
            assert!(tally.get(Work::AnyLookups) > 0, "{shape}");
        }
        // Nothing is left on the count of the load once it is built.
        assert_eq!(ws.index().lookups.get(), 0, "{shape}");
    }
}

/// The path of the many named imports through many globs a query resolves:
/// `X`, which the n globs of `t` bring in.
const QUERIED: &str = "demo::t::X";

/// The lookup steps of resolving [`QUERIED`] at `n`: the walk looks at the
/// n reads of `t`, all of whose views hold `X`, the chain of the answer
/// looks at them again to find the route of the binding, and at the one
/// read of `g0` for the next link of the chain.
const fn query_steps(n: usize) -> usize {
    2 * n + 1
}

#[test]
fn a_query_after_the_load_is_charged_its_own_lookup_steps_against_the_limit_of_one_query() {
    for n in [8, 16] {
        let lib = lookups(n);
        let steps = query_steps(n);
        // The query makes its steps, each a read or view the tallies saw a
        // lookup look at, and charges each once.
        let ws = queried(&lib, Budget::Work.limit());
        let (answer, tally) = measure(usize::MAX, || ws.resolve(QUERIED));
        assert_eq!(
            answer.map(|found| found.path).ok(),
            Some("demo::d::X".to_owned())
        );
        assert_eq!(made(&tally), steps, "{n}");
        assert_eq!(tally.get(Work::LookupCharges), steps, "{n}");
        // At its steps it answers, also a second time: each query counts
        // from zero, and the lookups of the load, many more, are not its.
        let ws = queried(&lib, steps);
        assert!(ws.usage().work > steps);
        for turn in ["first", "second"] {
            assert_eq!(
                ws.resolve(QUERIED).map(|found| found.path).ok(),
                Some("demo::d::X".to_owned()),
                "{n}, the {turn} time"
            );
        }
        // One step below, it is refused with the typed error of the work
        // budget, the second time as the first. The step that passes the
        // limit is the chain's look at the read of `g0`, declared on the
        // third line.
        let ws = queried(&lib, steps - 1);
        for turn in ["first", "second"] {
            assert_eq!(
                ws.resolve(QUERIED),
                Err(ResolveError::Budget {
                    budget: Budget::Work,
                    limit: steps - 1,
                    file: RelPath::new("src/lib.rs").expect("a relative path"),
                    line: Some(3),
                }),
                "{n}, the {turn} time"
            );
        }
    }
}

#[test]
fn every_limit_below_the_steps_of_a_query_refuses_it_and_the_refusal_comes_before_any_answer() {
    let lib = lookups(8);
    let steps = query_steps(8);
    for limit in 0..steps {
        let answer = queried(&lib, limit).resolve(QUERIED);
        assert!(
            matches!(
                answer,
                Err(ResolveError::Budget { budget: Budget::Work, limit: refused, .. })
                    if refused == limit
            ),
            "at {limit}: {answer:?}"
        );
    }
}

#[test]
fn every_query_counts_the_lookups_of_its_walks_and_of_the_views_it_looks_at_to_find_them() {
    // The reads and views each query looks at are charged once to it: a
    // path of the module whose names many views hold and two of the
    // prelude look views up to find the reads, and a `use` path read from
    // `u` asks whether the globs of `m` bring `A` in at all.
    let mut scanned = 0;
    let mut any = 0;
    for (lib, module, path) in [
        (holder_scan(8), None, "demo::u0::X0"),
        (prelude(8), None, "demo::T0"),
        (prelude(8), None, "demo::c1::T0"),
        (ANY.to_owned(), Some("demo::u"), "A"),
    ] {
        let ws = queried(&lib, Budget::Work.limit());
        let (_, tally) = measure(usize::MAX, || match module {
            Some(module) => ws.resolve_from(module, path).map(|_| ()),
            None => ws.resolve(path).map(|_| ()),
        });
        assert!(made(&tally) > 0, "{path}");
        assert_eq!(tally.get(Work::LookupCharges), made(&tally), "{path}");
        scanned += tally.get(Work::LookupScans);
        any += tally.get(Work::AnyLookups);
        // Each refuses at one step below what it charged.
        let ws = queried(&lib, made(&tally) - 1);
        let answer = match module {
            Some(module) => ws.resolve_from(module, path),
            None => ws.resolve(path),
        };
        assert!(
            matches!(answer, Err(ResolveError::Budget { .. })),
            "{path}: {answer:?}"
        );
    }
    assert!(scanned > 0 && any > 0, "views {scanned}, whether {any}");
}

#[test]
fn every_way_of_resolving_a_path_is_a_query_with_its_own_limit() {
    let ws = queried(&lookups(8), 0);
    let package = ws.packages().first().expect("one package").name.clone();
    let lib = ws.target_ids().next().expect("the library");
    for (way, answer) in [
        ("resolve", ws.resolve(QUERIED)),
        ("resolve_in", ws.resolve_in(QUERIED, crate::Namespace::Type)),
        (
            "resolve_in_package",
            ws.resolve_in_package(&package, QUERIED),
        ),
        ("resolve_from", ws.resolve_from("demo::t", "self::X")),
        (
            "resolve_from_in",
            ws.resolve_from_in("demo::t", "self::X", crate::Namespace::Type),
        ),
    ] {
        assert!(
            matches!(answer, Err(ResolveError::Budget { limit: 0, .. })),
            "{way}: {answer:?}"
        );
    }
    // A path whose lookups look at no glob read is no work.
    assert!(ws.resolve("demo::d::X").is_ok());
    // Canonical listing resolves each glob-supplied public name too.
    match ws.exported_items(lib) {
        Err(ExportError::Budget(error)) => refused_at(&error, 0),
        other => panic!("the glob surface must consume query work: {other:?}"),
    }
    let definitions = queried("pub mod d { pub struct X {} }", 0);
    let lib = definitions.target_ids().next().expect("the library");
    assert!(definitions.exported_items(lib).is_ok());
}

#[test]
fn the_public_surface_is_one_query_whose_lookups_and_those_of_the_paths_it_resolves_share_its_limit()
 {
    // Listing the surface walks the path of `u`'s glob, one step, and
    // resolves `m`'s public import of `A`, which asks whether the globs of
    // `x` bring `A` in, a step in each name space: four path reads. Listing
    // also resolves a::c and u::X canonically in all three namespaces.
    let steps = 4 + 2 * 3;
    let ws = queried(SURFACE, Budget::Work.limit());
    let lib = ws.target_ids().next().expect("the library");
    let (listed, tally) = measure(usize::MAX, || ws.exported_items(lib).map(|_| ()));
    assert!(listed.is_ok());
    assert_eq!(tally.get(Work::LookupCharges), steps);
    assert_eq!(made(&tally), steps);
    assert!(queried(SURFACE, steps).exported_items(lib).is_ok());
    match queried(SURFACE, steps - 1).exported_items(lib) {
        Err(ExportError::Budget(error)) => refused_at(&error, steps - 1),
        other => panic!("{other:?}"),
    }
}

#[test]
fn queries_on_other_threads_each_count_from_zero() {
    let lib = lookups(8);
    let steps = query_steps(8);
    let ws = queried(&lib, steps);
    let answers: Vec<Result<String, ResolveError>> = std::thread::scope(|scope| {
        let ws = &ws;
        let workers: Vec<_> = (0..4)
            .map(|_| scope.spawn(move || ws.resolve(QUERIED).map(|found| found.path)))
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().expect("the query runs"))
            .collect()
    });
    assert_eq!(answers, vec![Ok("demo::d::X".to_owned()); 4]);
}
