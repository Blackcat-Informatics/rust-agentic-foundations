// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Actual delivery ownership, native spill admission and visibility merging.

use super::*;
use crate::{Budget, LoadErrorKind, LoadOptions, MemoryTree, RelPath, Workspace};

fn retained_rows_tree() -> MemoryTree {
    let source = "pub mod a { pub struct X; pub struct Y; }
        pub mod b { pub struct X; pub struct Y; }
        pub mod hidden { pub struct X; pub struct Y; }
        pub mod hub { pub use crate::a::*; pub use crate::b::*;
            use crate::hidden::*; pub use crate::relay::*; }
        pub mod relay { pub use crate::hub::*; }";
    [("Cargo.toml", shapes::MANIFEST), ("src/lib.rs", source)]
        .into_iter()
        .collect()
}

#[test]
fn recorded_name_rows_survive_real_reconstruction_with_new_equal_interned_names() {
    let ws = Workspace::load(&retained_rows_tree(), &["."]).expect("returning names");
    let options = LoadOptions::default();
    let meter = Meter::new(&options);
    let decl = &ws.index().uses[ws.index().glob_imports[0]];
    meter
        .globs(ws.index().globs.charged, &decl.file, Some(decl.line))
        .expect("resident cloned tables admitted");
    let mut index = ws.index().clone();
    index.lookups.built(false);
    let retained = index.globs.drops.clone();
    assert!(!retained.is_empty());
    index
        .replay_globs_for_measurement(&meter)
        .expect("actual clearing and from-definitions reconstruction");
    assert_eq!(
        index.globs.drops, retained,
        "the authoritative rows survive unchanged"
    );
    let mut names_checked = 0;
    for (&(glob, target), names) in &retained {
        let read = index
            .reads_of(index.uses[glob].module)
            .iter()
            .find(|read| read.glob == glob && read.target == target)
            .expect("actual reconstructed read");
        for (name, spaces) in names {
            let current = index
                .globs
                .strings
                .get(name.as_str())
                .expect("new canonical name");
            assert_eq!(current, name);
            assert_ne!(
                current.as_ptr(),
                name.as_ptr(),
                "clearing replaced the interner allocation"
            );
            let independent = SharedStr::from(name.as_str());
            assert_ne!(independent.as_ptr(), name.as_ptr());
            assert_ne!(independent.as_ptr(), current.as_ptr());
            for (at, held) in spaces.iter().enumerate() {
                assert_eq!(index.replacements(read, independent.as_str(), at), held);
                assert!(
                    held.windows(2).all(|pair| pair[0].0 < pair[1].0),
                    "identities remain sorted"
                );
                if at < 2 {
                    assert_eq!(
                        held.len(),
                        2,
                        "two independent identities in Type and Value"
                    );
                    assert!(held.iter().all(|(_, level)| *level == Level::PUBLIC));
                } else {
                    assert!(held.is_empty(), "no Macro selection invented");
                }
            }
            names_checked += 1;
        }
    }
    assert_eq!(
        names_checked, 2,
        "both retained identifier rows were queried"
    );
}

#[test]
fn recorded_name_hash_storage_loads_at_exact_item_and_glob_grants_and_refuses_one_less() {
    let tree = retained_rows_tree();
    let ws = Workspace::load(&tree, &["."]).expect("measure actual original grants");
    let options = LoadOptions {
        max_item_bytes: ws.usage().item_bytes,
        max_glob_bytes: ws.usage().glob_bytes,
        ..LoadOptions::default()
    };
    Workspace::load_with(&tree, &["."], &options).expect("exact overlapping actual grants");
    for (short, budget, limit) in [
        (
            LoadOptions {
                max_item_bytes: options.max_item_bytes - 1,
                ..options.clone()
            },
            Budget::ItemBytes,
            options.max_item_bytes - 1,
        ),
        (
            LoadOptions {
                max_glob_bytes: options.max_glob_bytes - 1,
                ..options.clone()
            },
            Budget::GlobBytes,
            options.max_glob_bytes - 1,
        ),
    ] {
        let error = Workspace::load_with(&tree, &["."], &short).expect_err("one less refuses");
        assert!(
            matches!(error.kind, LoadErrorKind::Budget { budget: actual, limit: applied } if actual == budget && applied == limit),
            "{error:?}"
        );
        assert_eq!(
            error.file,
            RelPath::new("src/lib.rs").expect("fixture source")
        );
        assert!(
            error.line.is_some(),
            "responsible import retains source context"
        );
    }
}

fn declaration() -> UseDecl {
    UseDecl {
        module: 0,
        file: RelPath::new("src/lib.rs").expect("source path"),
        line: 1,
        leading_colon: false,
        vis: Vis::Public,
        test: false,
        doc_hidden: false,
        kind: crate::index::UseKind::Glob {
            segs: vec!["source".to_owned()],
        },
    }
}

fn delta<'m>(meter: &'m Meter, id: usize, payload: bool, decl: &UseDecl) -> Grown<'m> {
    let memory = Transient::new(meter);
    let mut ids = Vec::new();
    if payload {
        reserve_next(
            &mut ids,
            Steps::LoadAt {
                memory: &memory,
                file: &decl.file,
                line: decl.line,
            },
        )
        .expect("delta payload admission");
        ids.push((ItemId(id), Level::PUBLIC));
    }
    Grown::new(
        id,
        SharedStr::from("X"),
        0,
        Added {
            ids,
            _memory: memory,
        },
    )
}

fn assert_released(meter: &Meter) {
    Transient::new(meter)
        .charge(meter.options().max_glob_bytes, &RelPath::root(), None)
        .expect("the entire live-byte grant is available after delivery dies");
}

#[test]
fn native_delivery_spill_is_lifo_and_failed_push_releases_only_the_incoming_delta() {
    let decl = declaration();
    let meter = Meter::new(&LoadOptions::default());
    let mut queue = Queue::new(&meter);
    queue
        .push(delta(&meter, 0, true, &decl), &decl)
        .expect("inline delta");
    assert_eq!(queue.work.heap_capacity(), 0);
    let payload = meter.usage().glob_bytes;
    assert!(payload > 0);
    queue
        .push(delta(&meter, 1, true, &decl), &decl)
        .expect("first spill");
    let spill = queue.work.heap_capacity() * size_of::<Grown<'_>>();
    assert!(spill > 0);
    assert_eq!(queue.work.pop().expect("spill entry").view, 1);
    assert_eq!(queue.work.pop().expect("inline entry").view, 0);
    assert!(queue.work.is_empty());
    assert!(
        queue.work.heap_capacity() > 0,
        "pop retains the real spill allocation"
    );
    drop(queue);
    assert_released(&meter);

    for short in [false, true] {
        let options = LoadOptions {
            max_glob_bytes: 2 * payload + spill - usize::from(short),
            ..LoadOptions::default()
        };
        let meter = Meter::new(&options);
        let mut queue = Queue::new(&meter);
        queue
            .push(delta(&meter, 0, true, &decl), &decl)
            .expect("inline owner");
        let result = queue.push(delta(&meter, 1, true, &decl), &decl);
        if short {
            assert!(matches!(
                result,
                Err(LoadError {
                    kind: LoadErrorKind::Budget {
                        budget: Budget::GlobBytes,
                        ..
                    },
                    ..
                })
            ));
            assert_eq!(queue.work.len(), 1);
            assert_eq!(queue.work.heap_capacity(), 0);
            // Failed publication dropped the incoming Added owner, while
            // the original inline owner's actual payload is still held.
            Transient::new(&meter)
                .charge(
                    options.max_glob_bytes - payload,
                    &decl.file,
                    Some(decl.line),
                )
                .expect("only the first payload remains live");
        } else {
            result.expect("the exact first-spill peak is admitted");
            assert_eq!(queue.work.len(), 2);
        }
        drop(queue);
        assert_released(&meter);
    }
}

#[test]
fn native_delivery_growth_admits_old_and_new_spill_buffers_together() {
    let decl = declaration();
    let meter = Meter::new(&LoadOptions::default());
    let mut queue = Queue::new(&meter);
    queue
        .push(delta(&meter, 0, false, &decl), &decl)
        .expect("inline");
    queue
        .push(delta(&meter, 1, false, &decl), &decl)
        .expect("spill");
    let capacity = queue.work.heap_capacity();
    let next = purrdf_lex::allocation::growth_capacity::<Grown<'_>>(capacity, capacity + 1)
        .expect("native next capacity");
    drop(queue);
    assert_released(&meter);
    for short in [false, true] {
        let options = LoadOptions {
            max_glob_bytes: (capacity + next) * size_of::<Grown<'_>>() - usize::from(short),
            ..LoadOptions::default()
        };
        let meter = Meter::new(&options);
        let mut queue = Queue::new(&meter);
        for id in 0..=capacity {
            queue
                .push(delta(&meter, id, false, &decl), &decl)
                .expect("fill inline and old spill");
        }
        let result = queue.push(delta(&meter, capacity + 1, false, &decl), &decl);
        if short {
            assert!(matches!(
                result,
                Err(LoadError {
                    kind: LoadErrorKind::Budget {
                        budget: Budget::GlobBytes,
                        ..
                    },
                    ..
                })
            ));
            assert_eq!(queue.work.heap_capacity(), capacity);
            assert_eq!(queue.work.len(), capacity + 1);
        } else {
            result.expect("exact coexisting physical buffers are admitted");
            assert_eq!(queue.work.heap_capacity(), next);
            assert_eq!(queue.work.pop().expect("new entry").view, capacity + 1);
        }
        for expected in (0..=capacity).rev() {
            assert_eq!(queue.work.pop().expect("original entry").view, expected);
        }
        drop(queue);
        assert_released(&meter);
    }
}

#[test]
fn widest_routes_merge_firm_evidence_in_place_and_refuse_before_mutation() {
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        ),
        (
            "src/lib.rs",
            "mod source { pub struct X; } pub use source::*;",
        ),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["."]).expect("indexed glob");
    let index = ws.index();
    let site = index.glob_imports[0];
    let input = vec![
        (ItemId(2), Level::within(0).companion()),
        (ItemId(0), Level::PUBLIC.companion()),
        (ItemId(1), Level::PUBLIC.companion()),
        (ItemId(0), Level::within(0)),
        (ItemId(2), Level::PUBLIC),
        (ItemId(1), Level::within(0).companion()),
    ];
    let mut routes = input.clone();
    let allocation = (routes.as_ptr(), routes.capacity());
    index
        .widest(&mut routes, site, &Meter::new(&LoadOptions::default()))
        .expect("merge routes");
    assert_eq!(
        routes,
        [
            (ItemId(0), Level::PUBLIC),
            (ItemId(1), Level::PUBLIC.companion()),
            (ItemId(2), Level::PUBLIC)
        ]
    );
    assert_eq!((routes.as_ptr(), routes.capacity()), allocation);
    let options = LoadOptions {
        max_work: 0,
        ..LoadOptions::default()
    };
    let mut refused = input.clone();
    assert!(matches!(
        index.widest(&mut refused, site, &Meter::new(&options)),
        Err(LoadError {
            kind: LoadErrorKind::Budget {
                budget: Budget::Work,
                ..
            },
            ..
        })
    ));
    assert_eq!(refused, input);
}

#[test]
fn maximal_public_replacements_avoid_repeat_proofs_while_narrower_ones_are_widened() {
    let source = shapes::ambiguous_hub(8);
    let tree: MemoryTree = [
        ("Cargo.toml", shapes::MANIFEST),
        ("src/lib.rs", source.as_str()),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["."]).expect("replacement-positive hub");
    let mut index = ws.index().clone();
    assert!(index.drop_count() > 0);
    let mut selected = None;
    for (&(glob, target), names) in &index.globs.drops {
        for (name, spaces) in names {
            for (at, entries) in spaces.iter().enumerate() {
                for &(id, level) in entries {
                    assert_eq!(
                        level,
                        Level::PUBLIC,
                        "the control contains only maximal records"
                    );
                    selected.get_or_insert((glob, target, name.clone(), at, id));
                }
            }
        }
    }
    let meter = Meter::new(&LoadOptions::default());
    let (recorded, counts) = crate::work::measure(usize::MAX, || {
        index.record_drops(&meter).expect("maximal replay")
    });
    assert_eq!(recorded, 0);
    // Any remaining negative proofs are the control population. Widening
    // one real unsaturated selection must require additional routed proof.
    let negative_proof_steps = counts.get(Work::SupplySteps);
    assert_eq!(
        negative_proof_steps, 0,
        "maximal selections and competitors defeated by firm evidence need no graph proof"
    );
    let (glob, target, name, at, id) = selected.expect("a replacement exists");
    let reader = index.uses[glob].module;
    let entries = &mut index
        .globs
        .drops
        .get_mut(&(glob, target))
        .expect("recorded carrier")
        .get_mut(&name)
        .expect("recorded name")[at];
    let place = entries
        .binary_search_by_key(&id, |entry| entry.0)
        .expect("recorded candidate");
    entries[place].1 = Level::within(reader);
    let meter = Meter::new(&LoadOptions::default());
    let (recorded, counts) = crate::work::measure(usize::MAX, || {
        index
            .record_drops(&meter)
            .expect("wider support is still discovered")
    });
    assert_eq!(recorded, 1);
    assert!(
        counts.get(Work::SupplySteps) > negative_proof_steps,
        "widening one unsaturated member requires additional actual route proof"
    );
    assert_eq!(
        index.globs.drops[&(glob, target)][&name][at][place].1,
        Level::PUBLIC
    );
}

#[test]
fn one_returning_carrier_retains_multiple_independently_selected_identities() {
    let source = "pub mod a { pub struct X; }
        pub mod b { pub struct X; }
        pub mod hidden { pub struct X; }
        pub mod hub {
            pub use crate::a::*;
            pub use crate::b::*;
            use crate::hidden::*;
            pub use crate::relay::*;
        }
        pub mod relay { pub use crate::hub::*; }";
    let tree: MemoryTree = [("Cargo.toml", shapes::MANIFEST), ("src/lib.rs", source)]
        .into_iter()
        .collect();
    let ws = Workspace::load(&tree, &["."]).expect("multiple independent selections");
    let index = ws.index();
    let mut both = 0;
    for names in index.globs.drops.values() {
        let Some(spaces) = names.get("X") else {
            continue;
        };
        for entries in spaces {
            let paths: Vec<_> = entries
                .iter()
                .map(|(id, level)| {
                    assert_eq!(*level, Level::PUBLIC);
                    index.items[id.0].path.as_str()
                })
                .collect();
            if paths == ["demo::a::X", "demo::b::X"] {
                both += 1;
            }
        }
    }
    assert_eq!(
        both, 2,
        "one carrier selects both identities in both namespaces"
    );
    let options = LoadOptions {
        max_glob_bytes: ws.usage().glob_bytes,
        ..LoadOptions::default()
    };
    Workspace::load_with(&tree, &["."], &options).expect("exact overlapping scratch grant");
    let refused = Workspace::load_with(
        &tree,
        &["."],
        &LoadOptions {
            max_glob_bytes: options.max_glob_bytes - 1,
            ..options
        },
    )
    .expect_err("one byte below the actual peak is refused");
    assert!(matches!(
        refused.kind,
        LoadErrorKind::Budget {
            budget: Budget::GlobBytes,
            ..
        }
    ));
}
