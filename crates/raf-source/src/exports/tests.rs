// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The work of listing a surface grows in step with the names it holds.
//!
//! Each test loads a crate at three sizes, one, two and three times `UNIT`,
//! outside the measure, then lists it inside one. Linear growth means the
//! third step cannot add more than the second did, which needs no constant
//! to be tuned: a step that does work per pair of names makes each size add
//! more than the one before.

use std::fmt::Write as _;

use super::Exports;
use crate::work::{Work, measure_kind};
use crate::{LoadOptions, MemoryTree, TargetKind, Workspace};

/// The size of the smallest crate.
const UNIT: usize = 500;

/// The most the kind of work a test asserts on may count while one listing
/// runs. A listing that does work per pair of names passes it at once instead
/// of running to the end. The other kinds, such as the lookups the settled
/// tables answer, are counted without a limit.
const CEILING: usize = 64 * UNIT;

const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

#[test]
fn native_surface_storage_preserves_physical_failure_and_budget_ownership() {
    use crate::meter::{Meter, NativeAdmission, push_item_work};
    use crate::{Budget, ExportError, LoadErrorKind, RelPath};
    use purrdf_lex::{allocation::Memory, walk::WorkList};

    let file = RelPath::new("src/lib.rs").expect("path");
    let meter = Meter::new(&LoadOptions::default());
    let mut admission = NativeAdmission::items(&meter, &file, Some(7), 0);
    let mut values = Vec::<u64>::new();
    let error = Memory::new(&mut admission)
        .reserve(&mut values, usize::MAX)
        .expect_err("impossible native layout");
    assert_eq!(values, []);
    assert_eq!(
        meter.usage().item_bytes,
        0,
        "layout fails before allocation or admission"
    );
    for carried in [None, Some("demo::Alias".to_owned())] {
        let error = admission.graph_error(error.into());
        assert_eq!((error.file.as_str(), error.line), ("src/lib.rs", Some(7)));
        let mapped = ExportError::storage(error, carried);
        assert!(matches!(mapped, ExportError::Storage(_)));
        assert!(!mapped.to_string().contains("over budget"));
        assert!(std::error::Error::source(&mapped).is_some());
    }
    let meter = Meter::new(&LoadOptions {
        max_item_bytes: 0,
        ..LoadOptions::default()
    });
    let mut work = WorkList::<_, 1>::new();
    push_item_work(&mut work, 1, &meter, &file, Some(8)).expect("inline neighbor");
    let error = push_item_work(&mut work, 2, &meter, &file, Some(8))
        .expect_err("actual native spill budget");
    assert!(matches!(
        error.kind,
        LoadErrorKind::Budget {
            budget: Budget::ItemBytes,
            limit: 0
        }
    ));
    assert!(matches!(
        ExportError::storage(error, None),
        ExportError::Budget(_)
    ));
    let error =
        push_item_work(&mut work, 2, &meter, &file, Some(8)).expect_err("same spill refused again");
    assert!(matches!(
        ExportError::storage(error, Some("demo::Alias".to_owned())),
        ExportError::CarryBudget { .. }
    ));
    assert_eq!(
        work.pop(),
        Some(1),
        "refused push preserves existing payload"
    );
}

fn tree(lib: &str) -> MemoryTree {
    [("Cargo.toml", DEMO), ("src/lib.rs", lib)]
        .into_iter()
        .collect()
}

fn load(lib: &str) -> Workspace {
    Workspace::load(&tree(lib), &["."]).unwrap_or_else(|e| panic!("{e}"))
}

fn list(ws: &Workspace) -> Exports {
    let target = ws
        .target_ids()
        .find(|id| ws.target(*id).is_some_and(|t| t.kind == TargetKind::Lib))
        .expect("a library target");
    ws.exported_items(target).unwrap_or_else(|e| panic!("{e}"))
}

/// What listing `lib` counts of `kind`, after `check` has looked at the
/// listing.
fn counted(kind: Work, lib: &str, check: impl FnOnce(&Exports)) -> usize {
    let ws = load(lib);
    let (listed, tally) = measure_kind(kind, CEILING, || list(&ws));
    check(&listed);
    tally.get(kind)
}

/// The counts of `kind` for the crates `build` makes at one, two and three
/// units, and the asserted growth: the count is nonzero and the third unit
/// adds no more than the second.
fn assert_linear(kind: Work, build: impl Fn(usize) -> String, check: impl Fn(&Exports, usize)) {
    let counts = [1, 2, 3].map(|units| {
        let size = units * UNIT;
        counted(kind, &build(size), |listed| check(listed, size))
    });
    let [one, two, three] = counts;
    assert!(one > 0, "{kind:?} counted nothing: {counts:?}");
    assert!(
        // The same as three - two <= two - one, without a subtraction below zero.
        three + one <= 2 * two,
        "{kind:?} grows faster than its names: {counts:?}"
    );
}

/// A module of `n` globs of modules that each hold a function.
fn pub_globs(n: usize) -> String {
    let mut lib = String::new();
    for i in 0..n {
        writeln!(lib, "mod m{i} {{ pub fn f{i}() {{}} }}\npub use m{i}::*;").expect("write");
    }
    lib
}

/// A module of `n` globs of modules that each carry a re-export, and `n`
/// globs of modules that all bind two names, one of which the reader shadows.
fn carrying_globs(n: usize) -> String {
    let mut lib = String::new();
    for i in 0..n {
        writeln!(
            lib,
            "mod m{i} {{ pub use std::cell::Cell as C{i}; }}\npub use m{i}::*;"
        )
        .expect("write");
    }
    for i in 0..n {
        writeln!(
            lib,
            "mod k{i} {{ pub use std::rc::Rc; pub use std::fmt::Debug; }}\npub use k{i}::*;"
        )
        .expect("write");
    }
    lib.push_str("pub struct Debug;\n");
    lib
}

/// The listing of `carrying_globs(size)` holds each carried name, and `Rc`
/// once from each module that has it, and not the name the reader shadows.
fn check_carrying(listed: &Exports, size: usize) {
    assert_eq!(listed.items.len(), 1);
    assert_eq!(listed.reexports.len(), 2 * size);
    assert!(
        listed
            .reexports
            .iter()
            .all(|reexport| reexport.path != "demo::Debug")
    );
}

#[test]
fn a_module_of_pub_globs_is_listed_in_view_reads_linear_in_its_names() {
    // The globs that bring in a name are found by the name; reading every
    // glob of the module for each name grows with the square of the names.
    assert_linear(Work::ViewReads, pub_globs, |listed, size| {
        assert_eq!(listed.items.len(), size);
    });
}

#[test]
fn a_module_of_pub_globs_that_each_carry_a_re_export_is_listed_in_view_reads_linear_in_its_names() {
    assert_linear(Work::ViewReads, carrying_globs, check_carrying);
}

#[test]
fn a_module_of_pub_globs_that_each_carry_a_re_export_is_listed_in_walk_steps_linear_in_its_names() {
    // The reach is walked once per reader, not once per carried import.
    assert_linear(Work::WalkSteps, carrying_globs, check_carrying);
}

/// A reader with `n` globs of modules that each carry a re-export of their
/// own name, beside `n` globs of modules that each bind that name privately.
fn sibling_shadows(n: usize) -> String {
    let mut lib = String::new();
    for i in 0..n {
        writeln!(
            lib,
            "mod m{i} {{ pub use std::cell::Cell as C{i}; }}\nmod s{i} {{ fn C{i}() {{}} }}\npub use m{i}::*;\npub use s{i}::*;"
        )
        .expect("write");
    }
    lib
}

#[test]
fn readers_of_carried_names_beside_sibling_shadows_list_in_linear_work_and_bounded_bytes() {
    let check = |listed: &Exports, size: usize| assert_eq!(listed.reexports.len(), size);
    // Walking the reach again for each shadowed name grows with the square
    // of the names.
    assert_linear(Work::ViewReads, sibling_shadows, check);
    assert_linear(Work::WalkSteps, sibling_shadows, check);
    // The walks are charged to the item-bytes budget, and 3000 of them list
    // within 32 MiB beyond what the load spent.
    let lib = sibling_shadows(3000);
    let spent = load(&lib).usage().item_bytes;
    let options = LoadOptions {
        max_item_bytes: spent + (32 << 20),
        ..LoadOptions::default()
    };
    let ws = Workspace::load_with(&tree(&lib), &["."], &options).expect("the load fits");
    assert_eq!(list(&ws).reexports.len(), 3000);
}
