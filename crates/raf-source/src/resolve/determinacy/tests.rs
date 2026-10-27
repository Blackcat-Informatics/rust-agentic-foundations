// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Actual query and surface admission boundaries of selected-route proofs.

use super::*;
use crate::exports::Surface;
use crate::meter::Meter;
use crate::resolve::Source;
use crate::{Budget, ExportError, ExportOptions, LoadOptions, MemoryTree, RelPath, Workspace};
use std::fmt::Write;

fn workspace() -> Workspace {
    workspace_with_suffix("")
}

fn workspace_with_suffix(extra: &str) -> Workspace {
    let source = include_str!("../../../../../fuzz/corpus/rustc-oracle/import_cycle_a_private_glob_does_not_finalize_a_returning_type.rs")
        .replace("pub(crate) const fn A() -> u32 { 1 }", "pub const fn A() -> u32 { 1 }")
        .replace("pub mod m2 {", "pub mod m2 {\npub use crate::m1::A as Bound;");
    let source = format!("{source}\npub use m2::Bound as Named;\n{extra}\n");
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        ),
        ("src/lib.rs", source.as_str()),
    ]
    .into_iter()
    .collect();
    Workspace::load(&tree, &["."]).expect("late import fixture")
}

fn query(ws: &Workspace, meter: &Meter, surface: bool) -> Result<(), ResolveError> {
    if surface {
        let target = ws.target_ids().next().expect("library");
        match Surface::new(
            ws.index(),
            target,
            "demo",
            &ExportOptions::default(),
            meter,
            &BTreeSet::new(),
        )
        .list()
        {
            Err(ExportError::Resolve(error)) => Err(error),
            other => panic!("expected semantic refusal from actual surface: {other:?}"),
        }
    } else {
        ws.index()
            .resolve("demo::Named", Some(Namespace::Value), None, meter)
            .map(|_| ())
    }
}

#[test]
fn selected_query_and_surface_proofs_admit_exact_work_and_release_all_scratch() {
    unresolved_import_read_refusals();
    let ws = workspace();
    for surface in [false, true] {
        let options = LoadOptions::default();
        let meter = Meter::query(&options);
        assert!(matches!(
            query(&ws, &meter, surface),
            Err(ResolveError::IndeterminateImport { .. })
        ));
        let usage = meter.usage();
        assert!(usage.work > 0 && usage.glob_bytes > 0);
        // Every proof's scratch has died even on semantic refusal.
        let memory = Transient::new(&meter);
        memory
            .charge(options.max_glob_bytes, &RelPath::root(), None)
            .expect("no proof scratch remains live");
        drop(memory);
        let mut exact = options;
        exact.max_query_work = usage.work;
        exact.max_glob_bytes = usage.glob_bytes;
        assert!(matches!(
            query(&ws, &Meter::query(&exact), surface),
            Err(ResolveError::IndeterminateImport { .. })
        ));
        let mut short_work = exact.clone();
        short_work.max_query_work -= 1;
        assert!(matches!(
            query(&ws, &Meter::query(&short_work), surface),
            Err(ResolveError::Budget {
                budget: Budget::Work,
                ..
            })
        ));
        exact.max_glob_bytes -= 1;
        assert!(matches!(
            query(&ws, &Meter::query(&exact), surface),
            Err(ResolveError::Budget {
                budget: Budget::GlobBytes,
                ..
            })
        ));
    }
}

fn unresolved_import_read_refusals() {
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        ),
        (
            "src/lib.rs",
            "pub use crate::missing as A; pub fn good() {}",
        ),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["."]).expect("a settled missing import loads");
    assert_eq!(
        ws.resolve("demo::good").expect("valid neighbour").path,
        "demo::good"
    );
    let index = ws.index();
    let root = index
        .module_named("demo", None)
        .expect("root lookup")
        .expect("root");
    let (file, line) = index.module_site(root);
    let missing = [Source {
        module: root,
        ns: Namespace::Type,
        name: "A".into(),
    }];
    for limited in [false, true] {
        let mut options = LoadOptions::default();
        if limited {
            options.max_item_bytes = 1;
        }
        let meter = Meter::query(&options);
        let error = index.unbound(
            &missing,
            &[],
            "demo::A",
            Steps::QueryItems {
                meter: &meter,
                file: &file,
                line,
            },
        );
        if limited {
            assert!(
                matches!(
                    error,
                    ResolveError::Budget {
                        budget: Budget::ItemBytes,
                        limit: 1,
                        ..
                    }
                ),
                "{error:?}"
            );
        } else {
            assert!(matches!(error, ResolveError::NotFound(_)), "{error:?}");
        }
    }
}

#[test]
fn native_query_layout_failure_is_reported_as_storage_without_allocating() {
    let meter = Meter::query(&LoadOptions::default());
    let memory = Transient::new(&meter);
    let file = RelPath::new("src/lib.rs").expect("source path");
    let mut admission = NativeAdmission::new(Steps::QueryAt {
        memory: &memory,
        file: &file,
        line: Some(13),
    });
    let mut values = Vec::<usize>::new();
    let error = purrdf_lex::allocation::Memory::new(&mut admission)
        .reserve(&mut values, usize::MAX)
        .expect_err("impossible native layout");
    let error = ResolveError::storage(admission.graph_error(error.into()));
    assert!(
        matches!(error, ResolveError::Storage { file: at, line: Some(13), what } if at == file && !what.is_empty())
    );
    assert_eq!(values, [] as [usize; 0]);
    assert_eq!(values.capacity(), 0);
    assert_eq!(meter.usage().glob_bytes, 0);
}

fn spill_workspace() -> Workspace {
    let mut source = "pub mod spill_source { pub fn x() {} }\n".to_owned();
    for route in 0..9 {
        writeln!(
            source,
            "pub mod route{route} {{ pub use crate::spill_source::*; }}"
        )
        .expect("write fixture module");
    }
    source.push_str("pub mod fan {\n");
    for route in 0..9 {
        writeln!(source, "pub use crate::route{route}::*;").expect("write fixture import");
    }
    source.push_str("}\n");
    workspace_with_suffix(&source)
}

#[test]
fn selected_path_proof_spills_on_real_routes_and_releases_its_scratch() {
    let ws = spill_workspace();
    let index = ws.index();
    assert!(index.has_late_imports);
    let root = index
        .module_named("demo", None)
        .expect("root lookup")
        .expect("root");
    let (file, line) = index.module_site(root);
    let options = LoadOptions::default();
    assert_eq!(
        index
            .resolve(
                "demo::fan::x",
                Some(Namespace::Value),
                None,
                &Meter::query(&options),
            )
            .expect("unrelated late import does not poison a genuine route")
            .path,
        "demo::spill_source::x"
    );
    let meter = Meter::query(&options);
    let memory = Transient::new(&meter);
    {
        let mut proof = Proof::new(
            index,
            Steps::QueryAt {
                memory: &memory,
                file: &file,
                line,
            },
        );
        proof
            .path(
                root,
                &["fan".to_owned(), "x".to_owned()],
                false,
                Namespace::Value,
                Heads::Local,
                None,
            )
            .expect("actual path seed");
        assert_eq!(proof.finish().expect("independent route proof"), None);
        assert!(proof.pending.is_empty());
        assert!(
            proof.pending.heap_capacity() > 0,
            "nine routes exceed inline storage"
        );
    }
    drop(memory);
    Transient::new(&meter)
        .charge(options.max_glob_bytes, &file, line)
        .expect("all proof and native spill storage was released");
}

struct SpillRun {
    result: Result<(), LoadError>,
    attempted: usize,
    pending: usize,
    capacity: usize,
    peak: usize,
}

fn select_real_routes(ws: &Workspace, limit: usize) -> SpillRun {
    let index = ws.index();
    let source = index
        .module_named("demo::spill_source", None)
        .expect("source lookup")
        .expect("source");
    let id = *index.scopes[source]["x"].defs[slot(Namespace::Value)]
        .keys()
        .next()
        .expect("genuine function");
    let fan = index
        .module_named("demo::fan", None)
        .expect("fan lookup")
        .expect("fan");
    let routes: Vec<_> = (0..9)
        .map(|route| {
            index
                .module_named(&format!("demo::route{route}"), None)
                .expect("route lookup")
                .expect("route")
        })
        .collect();
    let (file, line) = index.module_site(fan);
    let options = LoadOptions {
        max_glob_bytes: limit,
        ..LoadOptions::default()
    };
    let meter = Meter::query(&options);
    let memory = Transient::new(&meter);
    let steps = Steps::QueryAt {
        memory: &memory,
        file: &file,
        line,
    };
    let mut proof = Proof::new(index, steps);
    let mut attempted = 0;
    let result = routes.into_iter().try_for_each(|module| {
        attempted += 1;
        assert!(proof.select_at(module, Namespace::Value, "x", None, Some(fan), &[id], steps)?);
        Ok(())
    });
    let run = SpillRun {
        result,
        attempted,
        pending: proof.pending.len(),
        capacity: proof.pending.heap_capacity(),
        peak: meter.usage().glob_bytes,
    };
    drop(proof);
    drop(memory);
    Transient::new(&meter)
        .charge(limit, &file, line)
        .expect("successful and refused native pushes release all scratch");
    run
}

#[test]
fn selected_routes_admit_the_first_native_spill_exactly_and_refuse_before_mutation() {
    let ws = spill_workspace();
    let full = select_real_routes(&ws, usize::MAX);
    full.result.expect("unrestricted real selections");
    assert_eq!((full.attempted, full.pending), (9, 9));
    assert!(full.capacity > 0 && full.peak > 0);
    let exact = select_real_routes(&ws, full.peak);
    exact.result.expect("exact first-spill peak");
    assert_eq!(
        (exact.attempted, exact.pending, exact.peak),
        (9, 9, full.peak)
    );
    assert!(exact.capacity > 0);
    let short = select_real_routes(&ws, full.peak - 1);
    assert!(matches!(short.result, Err(LoadError {
        kind: crate::LoadErrorKind::Budget { budget: Budget::GlobBytes, limit }, ..
    }) if limit == full.peak - 1));
    assert_eq!((short.attempted, short.pending, short.capacity), (9, 8, 0));
}
