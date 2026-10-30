// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Constructive replay and its actual reconstruction boundary.

use super::*;
use crate::glob_scope::reference;
use crate::{Budget, ExportOptions, LoadErrorKind, LoadOptions, MemoryTree, Namespace, Workspace};
use std::collections::{BTreeMap, BTreeSet};

const STAR: &str = "pub mod a { pub struct X; pub struct Unique; }
    pub mod b { pub struct X; }
    pub mod hidden { pub struct X; }
    pub mod hub { pub use crate::a::*; pub use crate::b::*;
        use crate::hidden::*; pub use crate::relay::*; }
    pub mod relay { pub use crate::hub::*; }";

fn load(source: &str) -> Workspace {
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[package]\nname=\"demo\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
        ),
        ("src/lib.rs", source),
    ]
    .into_iter()
    .collect();
    Workspace::load(&tree, &["."]).expect("literal source loads")
}

fn resident_meter(index: &Index, options: &LoadOptions) -> Meter {
    let meter = Meter::new(options);
    let decl = &index.uses[index.glob_imports[0]];
    meter
        .globs(index.globs.charged, &decl.file, Some(decl.line))
        .expect("resident tables admitted");
    meter
}

fn name_inputs<'a>(
    proof: &Ranked<'a, '_>,
    name: &str,
    topology: &Topology<'a>,
    memory: &Transient<'_>,
) -> NameInputs<'a> {
    let mut cursors = proof.cursors(memory).expect("admitted frozen cursors");
    for next in proof.names().expect("complete ordered names") {
        if next == name {
            return proof
                .name_inputs(next, topology, memory, &mut cursors)
                .expect("actual name inputs");
        }
        let prior_memory = memory.child();
        drop(
            proof
                .name_inputs(next, topology, &prior_memory, &mut cursors)
                .expect("preceding frozen inputs"),
        );
    }
    panic!("the requested input name is absent: {name}");
}

type Payload = BTreeMap<(usize, Vec<bool>), BTreeMap<String, [Vec<Held>; 3]>>;

fn payload(index: &Index) -> Payload {
    index
        .globs
        .views
        .iter()
        .map(|view| {
            (
                (view.target, view.class.to_vec()),
                view.names
                    .iter()
                    .map(|(name, held)| (name.to_string(), *held.clone()))
                    .collect(),
            )
        })
        .collect()
}

fn compare_replay(ws: &Workspace) {
    let options = LoadOptions::default();
    let index = ws.index();
    let meter = resident_meter(index, &options);
    let mut replay = index.clone();
    replay.lookups.built(false);
    replay
        .replay_globs_for_measurement(&meter)
        .expect("actual from-definitions reconstruction");
    assert_eq!(
        payload(index),
        payload(&replay),
        "complete Level and companion payloads"
    );
    let export_options = ExportOptions::default();
    let hidden = BTreeSet::new();
    for target in ws.target_ids() {
        let surface = |index| {
            format!(
                "{:?}",
                crate::exports::Surface::new(
                    index,
                    target,
                    "demo",
                    &export_options,
                    &Meter::query(&options),
                    &hidden,
                )
                .list()
            )
        };
        assert_eq!(
            surface(index),
            surface(&replay),
            "actual exported surface including ambiguity"
        );
    }
    for path in [
        "demo::hub::X",
        "demo::relay::X",
        "demo::relay::Unique",
        "demo::a::Unique",
    ] {
        for ns in [Namespace::Type, Namespace::Value] {
            assert_eq!(
                format!(
                    "{:?}",
                    index.resolve(path, Some(ns), None, &Meter::query(&options))
                ),
                format!(
                    "{:?}",
                    replay.resolve(path, Some(ns), None, &Meter::query(&options))
                ),
                "query, identity and chain at {path} in {ns:?}"
            );
        }
    }
}

#[test]
fn a_mixed_relay_and_a_removed_anchor_match_actual_reconstruction() {
    for source in [
        "pub mod a { pub struct X; } pub mod b { pub struct X; } pub mod c { pub struct X; }
         pub mod hub { pub use crate::a::*; pub use crate::b::*; pub use crate::relay::*; }
         pub mod relay { use crate::hub::*; pub use crate::c::*; }",
        "pub mod a { pub struct X; } pub mod c { pub struct X; }
         pub mod hub { pub use crate::a::*; pub use crate::relay::*; }
         pub mod relay { use crate::hub::*; pub use crate::c::*; }",
    ] {
        compare_replay(&load(source));
    }
}

#[test]
fn an_impossible_aggregate_reservation_is_a_typed_layout_refusal_before_allocation() {
    let ws = load(STAR);
    let index = ws.index();
    let meter = resident_meter(index, &LoadOptions::default());
    let proof = Ranked {
        index,
        meter: &meter,
        memory: Transient::new(&meter),
        site: index.glob_imports[0],
    };
    let mut cells = Vec::<Held>::new();
    let error = proof
        .reserve(&mut cells, usize::MAX, &proof.memory)
        .expect_err("overflow count cannot allocate");
    assert!(
        matches!(
            error.kind,
            LoadErrorKind::Graph(purrdf_core::graph::GraphError::Storage(
                purrdf_lex::allocation::StorageError::SizeOverflow
            ))
        ),
        "{error:?}"
    );
    assert_eq!(cells.capacity(), 0);
}

#[test]
fn public_peer_rows_are_rooted_without_a_second_table_construction() {
    let before = reference::builds()[0];
    let ws = load(STAR);
    assert_eq!(
        reference::builds()[0] - before,
        1,
        "actual constructor omitted"
    );
    assert!(
        !ws.index().globs.drops.is_empty(),
        "real recorded returning rows"
    );
    let index = ws.index().clone();
    index.lookups.built(false);
    assert!(
        index
            .ranked_glob_tables(&resident_meter(&index, &LoadOptions::default()))
            .expect("rank and closure")
    );
    assert_eq!(
        ws.resolve("demo::relay::Unique")
            .expect("successful chain")
            .path,
        "demo::a::Unique"
    );
    compare_replay(&ws);
}

#[test]
fn complete_public_peer_discovery_omits_only_its_redundant_postpublication_pass() {
    let before = reference::discoveries();
    let ws = load(STAR);
    assert_eq!(
        reference::discoveries() - before,
        1,
        "one actual complete publication pass"
    );
    let index = ws.index();
    assert!(!index.globs.drops.is_empty(), "actual replacement rows");
    // Admit the owned comparison tables before copying them, then independently
    // execute exactly the omitted production pass against the retained rows.
    let options = LoadOptions::default();
    let meter = resident_meter(index, &options);
    let mut repeated = index.clone();
    repeated.lookups.built(false);
    assert!(
        repeated
            .ranked_glob_tables(&meter)
            .expect("actual rooted closure")
    );
    assert!(
        repeated
            .discovery_is_row_independent(&meter)
            .expect("global static input checks")
    );
    let rows = repeated.globs.drops.clone();
    assert!(
        rows.values()
            .flat_map(|names| names.values())
            .any(|spaces| !spaces[0].is_empty())
    );
    assert!(
        rows.values()
            .flat_map(|names| names.values())
            .any(|spaces| !spaces[1].is_empty())
    );
    let held = payload(&repeated);
    let before = reference::discoveries();
    assert_eq!(
        repeated
            .record_drops(&meter)
            .expect("actual omitted discovery"),
        0
    );
    assert_eq!(reference::discoveries() - before, 1);
    assert_eq!(
        repeated.globs.drops, rows,
        "all exact read/name/namespace/id/Level selections"
    );
    assert_eq!(
        payload(&repeated),
        held,
        "raw resident levels and companion flags"
    );
    compare_replay(&ws);
}

#[test]
fn roots_outside_resident_targets_still_require_the_second_discovery() {
    for definition in ["struct Private;", "pub(crate) struct Private;"] {
        let before = reference::discoveries();
        let ws = load(&format!("{STAR} pub mod unused {{ {definition} }}"));
        assert_eq!(
            reference::discoveries() - before,
            2,
            "rank alone cannot omit discovery"
        );
        let index = ws.index().clone();
        index.lookups.built(false);
        let meter = resident_meter(&index, &LoadOptions::default());
        assert!(
            index
                .ranked_glob_tables(&meter)
                .expect("unused root is outside resident targets")
        );
        assert!(
            !index
                .discovery_is_row_independent(&meter)
                .expect("all definition_supply roots checked")
        );
        compare_replay(&ws);
    }
}

#[test]
fn named_extern_alias_and_qualified_glob_neighbors_do_not_certify_independence() {
    for source in [
        format!("{STAR} pub mod unused {{ pub use crate::a::Unique; }}"),
        format!("{STAR} pub extern crate self as other;"),
        format!("{STAR} pub mod unused {{ pub type Alias = u8; }}"),
        format!("{STAR} pub mod unused {{ pub(in crate::unused) struct Private; }}"),
        STAR.replacen("pub use crate::a::*", "pub(crate) use crate::a::*", 1),
    ] {
        let ws = load(&source);
        let index = ws.index().clone();
        index.lookups.built(false);
        assert!(
            !index
                .discovery_is_row_independent(&resident_meter(&index, &LoadOptions::default()))
                .expect("unsupported discovery dependency")
        );
        compare_replay(&ws);
    }
}

#[test]
fn discovery_independence_scans_preserve_the_original_work_refusal() {
    let ws = load(STAR);
    let index = ws.index().clone();
    index.lookups.built(false);
    let options = LoadOptions::default();
    let meter = resident_meter(&index, &options);
    assert!(
        index
            .discovery_is_row_independent(&meter)
            .expect("measure actual static scans")
    );
    let spent = meter.usage().work;
    assert!(spent > 0);
    let exact = LoadOptions {
        max_work: spent,
        ..options
    };
    assert!(
        index
            .discovery_is_row_independent(&resident_meter(&index, &exact))
            .expect("exact original work grant")
    );
    let short = LoadOptions {
        max_work: spent - 1,
        ..exact
    };
    let error = index
        .discovery_is_row_independent(&resident_meter(&index, &short))
        .expect_err("one less refuses the actual scan charge");
    assert!(
        matches!(error.kind, LoadErrorKind::Budget { budget: Budget::Work, limit } if limit == spent - 1),
        "{error}"
    );
    let decl = &index.uses[index.glob_imports[0]];
    assert_eq!(error.file, decl.file);
    assert_eq!(error.line, Some(decl.line));
}

#[test]
fn an_exact_empty_namespace_proves_bottom_without_any_transfer() {
    let empty = load("pub mod a { pub use crate::b::*; } pub mod b { pub use crate::a::*; }");
    assert!(
        empty
            .index()
            .ranked_glob_tables(&resident_meter(empty.index(), &LoadOptions::default()))
            .expect("empty peer-glob graph has no terminal name to certify")
    );
    let ws = load(STAR);
    let index = ws.index().clone();
    index.lookups.built(false);
    let meter = resident_meter(&index, &LoadOptions::default());
    let proof = Ranked {
        index: &index,
        meter: &meter,
        memory: Transient::new(&meter),
        site: index.glob_imports[0],
    };
    let memory = proof.memory.child();
    let topology = proof.topology(&memory).expect("actual read topology");
    let mut incomplete = proof.cursors(&memory).expect("admitted frozen cursors");
    assert!(matches!(
        proof.complete_cursors(&mut incomplete).unwrap_err().kind,
        LoadErrorKind::Internal { .. }
    ));
    assert!(matches!(
        proof
            .name_inputs("X", &topology, &memory, &mut incomplete)
            .err()
            .expect("an earlier resident name cannot be skipped")
            .kind,
        LoadErrorKind::Internal { .. }
    ));
    let mut complete = proof.cursors(&memory).expect("admitted frozen cursors");
    for name in proof.names().expect("complete ordered names") {
        let name_memory = memory.child();
        drop(
            proof
                .name_inputs(name, &topology, &name_memory, &mut complete)
                .expect("ordered frozen input"),
        );
    }
    proof
        .complete_cursors(&mut complete)
        .expect("every frozen row consumed exactly once");
    let named = name_inputs(&proof, "X", &topology, &memory);
    let inputs = named.namespace(&topology, 2);
    assert!(inputs.bottom);
    assert!((0..inputs.rows.len()).all(|view| {
        let row = inputs.row(view);
        row.held.is_empty() && row.defs.is_none()
    }));
    let mut scratch = Scratch::default();
    assert!(
        proof
            .certify(&inputs, &mut scratch, &memory)
            .expect("exact bottom")
    );
    assert_eq!(scratch.closure_calls, 0);
    assert_eq!(scratch.seen.capacity(), 0);
    assert_eq!(scratch.hidden.capacity(), 0);
    assert!(meter.usage().work > 0, "the emptiness scans are billed");
}

#[test]
fn a_complete_definition_and_relay_rank_is_already_its_full_closure() {
    let ws = load(
        "pub mod a { pub struct X; } pub mod relay { pub use crate::a::*; }
        pub mod hub { pub use crate::relay::*; } pub mod reader { pub use crate::hub::*; }",
    );
    let index = ws.index().clone();
    index.lookups.built(false);
    let meter = resident_meter(&index, &LoadOptions::default());
    let proof = Ranked {
        index: &index,
        meter: &meter,
        memory: Transient::new(&meter),
        site: index.glob_imports[0],
    };
    let memory = proof.memory.child();
    let topology = proof.topology(&memory).expect("actual read topology");
    let named = name_inputs(&proof, "X", &topology, &memory);
    let inputs = named.namespace(&topology, 0);
    let mut scratch = Scratch::default();
    assert!(
        proof
            .certify(&inputs, &mut scratch, &memory)
            .expect("rooted complete ranks")
    );
    assert_eq!(
        scratch.closure_calls, 0,
        "each successful rank had its actual complete input"
    );
    assert!(scratch.seen.capacity() > 0, "actual transfers executed");
    compare_replay(&ws);

    let distinct = load(
        "pub mod a { pub type X = (); pub const X: () = (); }
         pub mod relay { pub use crate::a::*; }
         pub mod reader { pub use crate::relay::*; }",
    );
    assert!(
        distinct
            .index()
            .ranked_glob_tables(&resident_meter(distinct.index(), &LoadOptions::default()))
            .expect("different type and value identities certify independently")
    );
    compare_replay(&distinct);
}

#[test]
fn an_incomplete_successful_rank_still_checks_its_later_complete_input() {
    let ws = load(
        "pub mod a { pub struct X; } pub mod b { pub struct X; }
        pub mod relay { pub use crate::b::*; }
        pub mod hub { pub use crate::a::*; pub use crate::relay::*; }
        pub mod reader { pub use crate::hub::*; }",
    );
    let mut index = ws.index().clone();
    index.lookups.built(false);
    let hub = index
        .globs
        .views
        .iter()
        .position(|view| index.modules[view.target].path.as_str() == "demo::hub")
        .expect("hub resident");
    let relay = index
        .globs
        .views
        .iter()
        .position(|view| index.modules[view.target].path.as_str() == "demo::relay")
        .expect("relay resident");
    let a = index
        .items
        .iter()
        .position(|item| item.path.as_str() == "demo::a::X")
        .expect("genuine anchor");
    // Deliberately incomplete retained output: it can be rooted from a alone,
    // but the actual relay contributes b before full closure finishes.
    index.globs.views[hub].names.get_mut("X").expect("hub X")[0].retain(|entry| entry.0.0 == a);
    let meter = resident_meter(&index, &LoadOptions::default());
    let proof = Ranked {
        index: &index,
        meter: &meter,
        memory: Transient::new(&meter),
        site: index.glob_imports[0],
    };
    let memory = proof.memory.child();
    let topology = proof.topology(&memory).expect("actual read topology");
    let named = name_inputs(&proof, "X", &topology, &memory);
    let inputs = named.namespace(&topology, 0);
    let mut marks = vec![
        Mark {
            certified: true,
            ..Mark::default()
        };
        inputs.rows.len()
    ];
    marks[relay].certified = false;
    let mut scratch = Scratch::default();
    let complete = proof
        .transfer(hub, &inputs, &marks, &mut scratch, &memory)
        .expect("partial rank transfer");
    assert!(
        proof
            .same(&scratch.seen, inputs.row(hub).held)
            .expect("partial equality")
    );
    assert!(
        !complete,
        "unmarked actual relay prevents a closure witness"
    );
    marks[hub].closed = complete;
    marks[relay].certified = true;
    for view in 0..marks.len() {
        if view != hub {
            assert!(
                proof
                    .transfer(view, &inputs, &marks, &mut scratch, &memory)
                    .expect("other complete transfer")
            );
            assert!(
                proof
                    .same(&scratch.seen, inputs.row(view).held)
                    .expect("other actual closure")
            );
            marks[view].closed = true;
        }
    }
    assert!(
        !proof
            .closure(&inputs, &marks, &mut scratch, &memory)
            .expect("later complete input differs")
    );
    assert_eq!(
        scratch.closure_calls, 1,
        "the incomplete rank receives its mandatory full transfer"
    );
}

#[test]
fn interior_levels_named_imports_and_glob_prefixes_keep_ordinary_reconstruction() {
    for source in [
        "pub mod a { pub struct X; } pub mod b { pub struct X; } pub mod outer { pub mod p { pub(super) use crate::b::*; } pub mod m { pub use crate::a::*; use crate::b::*; pub use super::p::*; } pub mod observer { pub use super::m::*; } }",
        "pub mod a { pub struct X; } pub mod hub { pub use crate::a::X; pub use crate::a::*; }",
        "pub mod a { pub mod inner { pub struct X; } } pub mod prefix { pub use crate::a::*; } pub mod hub { pub use crate::prefix::inner::*; }",
    ] {
        let ws = load(source);
        let index = ws.index().clone();
        index.lookups.built(false);
        assert!(
            !index
                .ranked_glob_tables(&resident_meter(&index, &LoadOptions::default()))
                .expect("conservative proof boundary")
        );
        compare_replay(&ws);
    }
}

#[test]
fn absent_cells_and_ungrounded_cyclic_residents_cannot_pass_closure_or_rank() {
    let ws = load(STAR);
    let mut index = ws.index().clone();
    index.lookups.built(false);
    for view in &mut index.globs.views {
        view.names.clear();
    }
    assert!(
        !index
            .ranked_glob_tables(&resident_meter(&index, &LoadOptions::default()))
            .expect("absent own definitions checked")
    );

    let mut index = ws.index().clone();
    index.lookups.built(false);
    for scope in &mut index.scopes {
        scope.remove("X");
    }
    assert!(
        !index
            .ranked_glob_tables(&resident_meter(&index, &LoadOptions::default()))
            .expect("cyclic facts need genuine roots")
    );

    for at in [0, 1] {
        let mut index = ws.index().clone();
        index.lookups.built(false);
        for view in &mut index.globs.views {
            if let Some(held) = view.names.get_mut("X") {
                held[at].clear();
            }
        }
        assert!(
            !index
                .ranked_glob_tables(&resident_meter(&index, &LoadOptions::default()))
                .expect("each namespace must retain its own required cells")
        );

        let mut index = ws.index().clone();
        index.lookups.built(false);
        for scope in &mut index.scopes {
            if let Some(entry) = scope.get_mut("X") {
                entry.defs[at] = Bound::default();
            }
        }
        assert!(
            !index
                .ranked_glob_tables(&resident_meter(&index, &LoadOptions::default()))
                .expect("each namespace needs its own genuine roots")
        );
    }

    // Here the returning carrier's private companion must be removed: the
    // reader supplies X independently and is its only route to the carrier.
    let returning = load(
        "pub mod a { pub struct X; } pub mod hidden { pub struct X; }
         pub mod reader { pub use crate::a::*; pub use crate::carrier::*; }
         pub mod carrier { pub use crate::reader::*; use crate::hidden::*; }",
    );
    let mut index = returning.index().clone();
    index.lookups.built(false);
    assert!(
        index
            .ranked_glob_tables(&resident_meter(&index, &LoadOptions::default()))
            .expect("actual returning selections certify both namespaces")
    );
    assert!(
        index
            .globs
            .drops
            .values()
            .any(|names| { names.values().any(|spaces| !spaces[1].is_empty()) })
    );
    for names in index.globs.drops.values_mut() {
        for spaces in names.values_mut() {
            spaces[1].clear();
        }
    }
    assert!(
        !index
            .ranked_glob_tables(&resident_meter(&index, &LoadOptions::default()))
            .expect("different recorded selections need their own namespace proof")
    );
}

#[test]
fn proof_scratch_and_work_refuse_one_less_than_the_actual_original_grant() {
    let ws = load(STAR);
    let index = ws.index().clone();
    index.lookups.built(false);
    assert!(
        index
            .fixed_glob_paths(&resident_meter(&index, &LoadOptions::default()))
            .expect("actual fixed-path prerequisite")
    );
    // Isolate the new ranked scratch: the shared path walk correctly owns
    // its separate transient peak at the actual import being walked.
    let ranked = |meter: &Meter| {
        Ranked {
            index: &index,
            meter,
            memory: Transient::new(meter),
            site: index.glob_imports[0],
        }
        .tables()
    };
    let meter = resident_meter(&index, &LoadOptions::default());
    assert!(ranked(&meter).expect("measure actual proof"));
    let usage = meter.usage();
    assert!(usage.glob_bytes > index.globs.charged);
    assert!(usage.work > 0);
    let exact = LoadOptions {
        max_glob_bytes: usage.glob_bytes,
        max_work: usage.work,
        ..LoadOptions::default()
    };
    assert!(ranked(&resident_meter(&index, &exact)).expect("exact physical and work grant"));
    for (options, budget, limit) in [
        (
            LoadOptions {
                max_glob_bytes: usage.glob_bytes - 1,
                ..exact.clone()
            },
            Budget::GlobBytes,
            usage.glob_bytes - 1,
        ),
        (
            LoadOptions {
                max_work: usage.work - 1,
                ..exact.clone()
            },
            Budget::Work,
            usage.work - 1,
        ),
    ] {
        let error = ranked(&resident_meter(&index, &options)).expect_err("one less refuses");
        assert!(
            matches!(error.kind, LoadErrorKind::Budget { budget: actual, limit: applied } if actual == budget && applied == limit),
            "{error:?}"
        );
        let decl = &index.uses[index.glob_imports[0]];
        assert_eq!(error.file, decl.file);
        assert_eq!(error.line, Some(decl.line));
    }
}

#[test]
fn homogeneous_returning_carriers_share_an_actual_holder_retrieval() {
    let source = format!(
        "{} pub mod relay2 {{ pub use crate::hub::*; }} \
         pub mod relay3 {{ pub use crate::hub::*; }}",
        STAR.replacen(
            "pub use crate::relay::*;",
            "pub use crate::relay::*; pub use crate::relay2::*; pub use crate::relay3::*;",
            1,
        ),
    );
    let before = reference::holder_retrievals();
    let batched = load(&source);
    let batch_reads = reference::holder_retrievals() - before;
    let before = reference::holder_retrievals();
    let single = reference::with_single_returning(|| load(&source));
    let single_reads = reference::holder_retrievals() - before;
    assert!(batch_reads > 0, "actual holder proof requested");
    assert!(
        single_reads > batch_reads,
        "actual repeated retrievals omitted"
    );
    assert!(
        !batched.index().globs.drops.is_empty(),
        "real selected rows"
    );
    assert_eq!(batched.index().globs.drops, single.index().globs.drops);
    assert_eq!(payload(batched.index()), payload(single.index()));
    compare_replay(&batched);
    compare_replay(&single);
    compare_cut_proofs(batched.index());
    // A reader can hold another name on an independent route. Cutting its
    // whole module would erase that route and invent returning dominance.
    let renamed = load(
        "pub mod a { pub struct X; }
        pub mod b { pub struct X; }
        pub mod hub {
            pub use crate::a::X as Alias;
            pub use crate::relay::Y as X;
            pub use crate::a::*; pub use crate::b::*; pub use crate::relay::*;
        }
        pub mod relay { pub use crate::hub::*; pub use crate::hub::Alias as Y; }",
    );
    compare_cut_proofs(renamed.index());
    let mut sparse = source;
    for at in 0..512 {
        use std::fmt::Write as _;
        writeln!(sparse, "pub mod untouched_{at} {{}}").expect("fixture text");
    }
    let sparse = load(&sparse);
    let reader = sparse
        .index()
        .globs
        .reads
        .iter()
        .position(|reads| reads.len() >= 2)
        .expect("the actual hub remains the only discovery reader");
    compare_cut_proofs_for(
        sparse.index(),
        &[reader, sparse.index().modules.len()].into_iter(),
    );
}

/// Compare real cut membership with independently built full dominator proofs.
fn compare_cut_proofs(index: &Index) {
    compare_cut_proofs_for(index, &(0..=index.modules.len()));
}

fn compare_cut_proofs_for(index: &Index, readers: &(impl Iterator<Item = usize> + Clone)) {
    // These are load-construction APIs. Reconstruct their proofs on an owned
    // clone, preserving the completed workspace's post-load lookup guard.
    let settled = index;
    let constructing = settled.clone();
    constructing.lookups.built(false);
    let index = &constructing;
    let options = LoadOptions::default();
    let meter = resident_meter(index, &options);
    let first = index.glob_imports[0];
    let decl = &index.uses[first];
    let charge = |units| meter.work(units, &decl.file, Some(decl.line));
    let mut supply = index.supply(&meter, first).expect("actual supplier routes");
    assert!(supply.cut_reader.is_some(), "one actual discovery reader");
    let queries: BTreeSet<_> = index
        .globs
        .views
        .iter()
        .flat_map(|view| {
            view.names.iter().flat_map(|(name, spaces)| {
                spaces.iter().enumerate().flat_map(move |(at, held)| {
                    held.iter().flat_map(move |(id, level)| {
                        [false, true].map(|carried| (*id, name.clone(), at, level.plain(), carried))
                    })
                })
            })
        })
        .collect();
    assert!(!queries.is_empty());
    let mut present = 0;
    let mut absent = 0;
    let mut independent = 0;
    let mut returning = 0;
    for query in queries {
        let full = index
            .retained_holding(&mut supply, &charge, &query, None)
            .expect("full rooted graph");
        let expected: Vec<_> = readers
            .clone()
            .map(|reader| {
                (
                    reader,
                    (0..=index.modules.len())
                        .map(|from| {
                            (
                                full.holds(from),
                                full.holds(from) && full.dominates(reader, from),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        for (reader, rows) in expected {
            let proof = index
                .retained_cut(&mut supply, &charge, reader, &query, None)
                .expect("exact reader cut");
            for (from, (held, dominated)) in rows.into_iter().enumerate() {
                assert_eq!(
                    proof
                        .answer(from, true, &charge)
                        .expect("returning query admitted"),
                    dominated,
                    "{query:?}, reader {reader}, holder {from}"
                );
                assert_eq!(
                    proof
                        .answer(from, false, &charge)
                        .expect("independent query admitted"),
                    held && !dominated,
                    "{query:?}, reader {reader}, holder {from}"
                );
                present += usize::from(held);
                absent += usize::from(!held);
                returning += usize::from(dominated);
                independent += usize::from(held && !dominated);
            }
        }
    }
    index
        .settle_lookups(&meter, first)
        .expect("construction lookup work is charged");
    assert_eq!(
        settled.lookups.get(),
        0,
        "the completed index keeps its query-only account"
    );
    assert!(present > 0 && absent > 0 && independent > 0 && returning > 0);
}
