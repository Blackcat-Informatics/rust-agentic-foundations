// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The answers of loads settled in the reference mode (see
//! [`reference`](super::reference)) against those of loads settled out of
//! it, over the programs of the differential test against `rustc`: the
//! programs of both seed families and the committed corpus, each as written
//! and in more orders of its declarations. Every answer of
//! `tests/support/answers.rs` is compared: each probe in the three name
//! spaces, `self::<name>` from every module with a `use` declaration, and
//! the exported surface. While the loads settle, each view entry added is
//! checked to be new or wider than the one it takes the place of.

#[path = "../../tests/support/answers.rs"]
mod answers;
#[path = "../../tests/support/program.rs"]
#[allow(dead_code, reason = "the comparison uses part of the generator")]
mod program;
#[path = "../../tests/support/random.rs"]
#[allow(dead_code, reason = "the comparison uses part of the generator")]
mod random;

use std::path::Path;

use super::reference;
use crate::{DiskTree, EntryKind, MemoryTree, Namespace, RelPath, TargetKind, Tree, Workspace};
use answers::Counted;
use program::{Program, random_program};
use random::Draw;

/// The programs drawn from each seed family.
const PROGRAMS: usize = 64;

/// The bases of the two seed families of the differential test.
const BASES: [u64; 2] = [0x0a11_ce00, 0xc0ff_ee00];

/// The orders of the declarations each program is loaded in besides the
/// order as written.
const ORDERS: usize = 2;

/// The seed the orders of each program are drawn from.
const ORDER_SEED: u64 = 0x0de2_0001;

/// The programs compared, with their labels: [`PROGRAMS`] of each family,
/// the seeds `base + 1` to `base + 8` in turn, and the committed corpus.
fn programs() -> Vec<(String, Program)> {
    let mut all = Vec::new();
    for base in BASES {
        let seeds: Vec<u64> = (1..=8).map(|n| base + n).collect();
        let mut draws: Vec<Draw> = seeds.iter().map(|seed| Draw::seeded(*seed)).collect();
        for n in 0..PROGRAMS {
            let which = n % seeds.len();
            let program = random_program(&mut draws[which]);
            all.push((
                format!("seed {:#x} program {}", seeds[which], n / seeds.len()),
                program,
            ));
        }
    }
    // The corpus is read through the disk backend, the one module of the
    // crate that names the file system.
    let corpus =
        DiskTree::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fuzz/corpus/rustc-oracle"))
            .expect("the corpus directory opens");
    let mut names: Vec<String> = corpus
        .list(&RelPath::root())
        .expect("the corpus directory lists")
        .into_iter()
        .filter(|entry| entry.kind == EntryKind::File)
        .map(|entry| entry.name)
        .collect();
    names.sort();
    for name in names {
        let path = RelPath::new(&name).expect("a corpus file name is a path");
        let text = corpus
            .read(&path, 1 << 20)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        let program = Program::parse(&text).unwrap_or_else(|error| panic!("{name}: {error}"));
        all.push((format!("corpus {name}"), program));
    }
    all
}

/// The first answer that differs between `settled` and `reference`, the
/// answers of one load out of the reference mode and in it, if one does:
/// the label of the load, the line and both answers.
fn difference(label: &str, settled: &str, reference: &str) -> Option<String> {
    let (mut lines, mut others) = (settled.lines(), reference.lines());
    let mut line = 0;
    loop {
        line += 1;
        match (lines.next(), others.next()) {
            (None, None) => return None,
            (left, right) if left == right => {}
            (left, right) => {
                return Some(format!(
                    "{label}, answer {line}: settled {left:?}, in the reference mode {right:?}"
                ));
            }
        }
    }
}

/// What the comparison of some programs judged.
#[derive(Debug, Default)]
struct Judged {
    /// The programs compared.
    programs: usize,
    /// The loads compared, each made in both modes.
    loads: usize,
    /// What the answers out of the reference mode held.
    counted: Counted,
    /// The answers that differ, one for each load.
    differences: Vec<String>,
}

/// Compare the answers of each of `programs`, as written and in [`ORDERS`]
/// more orders, out of the reference mode and in it.
fn judge(programs: &[(String, Program)]) -> Judged {
    let mut judged = Judged::default();
    for (label, program) in programs {
        judged.programs += 1;
        let mut draw = Draw::seeded(ORDER_SEED);
        let orders =
            std::iter::once((label.clone(), program.clone())).chain((0..ORDERS).map(|order| {
                (
                    format!("{label} order {order}"),
                    program.shuffled(&mut draw),
                )
            }));
        for (label, program) in orders.collect::<Vec<_>>() {
            let (settled, counted) = answers::answers(&program);
            let (exact, _) = reference::exact(|| answers::answers(&program));
            judged.loads += 1;
            judged.counted = judged.counted.add(counted);
            judged
                .differences
                .extend(difference(&label, &settled, &exact));
        }
    }
    judged
}

#[test]
fn the_settling_gives_the_answers_of_the_reference_mode_to_every_generated_and_corpus_program_in_three_orders()
 {
    let programs = programs();
    let corpus = programs
        .iter()
        .filter(|(label, _)| label.starts_with("corpus "))
        .count();
    let before = reference::builds();
    let judged = judge(&programs);
    let after = reference::builds();
    assert!(
        judged.differences.is_empty(),
        "{} of {} loads differ:\n{}",
        judged.differences.len(),
        judged.loads,
        judged.differences.join("\n")
    );
    // The comparison judged every program, in each order, and the answers
    // it compared hold resolutions, ambiguities and refused surfaces, so a
    // run that compares nothing, or only one kind of answer, fails.
    assert!(corpus > 0, "the corpus holds programs");
    assert_eq!(judged.programs, BASES.len() * PROGRAMS + corpus);
    assert_eq!(judged.loads, judged.programs * (1 + ORDERS));
    assert_eq!(judged.counted.loads, judged.loads);
    assert!(
        judged.counted.answers > 100 * judged.loads,
        "{:?}",
        judged.counted
    );
    assert!(judged.counted.ambiguous > 0, "{:?}", judged.counted);
    assert!(judged.counted.refused_surfaces > 0, "{:?}", judged.counted);
    // The loads of each mode built their glob tables in that mode.
    for mode in 0..2 {
        assert!(
            after[mode] > before[mode],
            "no glob tables were built in mode {mode}: {before:?} then {after:?}"
        );
    }
}

#[test]
fn an_answer_that_differs_between_the_two_modes_is_reported_with_its_load_and_both_answers() {
    let (label, program) = programs()
        .into_iter()
        .find(|(_, program)| program.to_text().contains("::*;"))
        .expect("a program with a glob import");
    let (settled, _) = answers::answers(&program);
    let (exact, _) = reference::exact(|| answers::answers(&program));
    assert_eq!(difference(&label, &settled, &exact), None);
    // One answer changed, and one more or one fewer, are each reported.
    let changed = exact.replacen("ok ", "err ", 1);
    let reported = difference(&label, &settled, &changed).expect("a changed answer is reported");
    assert!(
        reported.starts_with(&format!("{label}, answer ")) && reported.contains("err "),
        "{reported}"
    );
    let longer = format!("{exact}one more\n");
    assert!(
        difference(&label, &settled, &longer).is_some_and(|reported| reported.contains("one more"))
    );
    let shorter = exact.split_once('\n').map_or("", |(_, rest)| rest);
    assert!(difference(&label, &settled, shorter).is_some());
    // The mode ends with the run it was asked for, and so does a run that
    // panics in it.
    let [out, in_mode] = reference::builds();
    drop(std::panic::catch_unwind(|| {
        reference::exact(|| panic!("a run that fails in the reference mode"))
    }));
    answers::answers(&program);
    let [out_after, in_after] = reference::builds();
    assert!(out_after > out, "{out} then {out_after}");
    assert_eq!(in_after, in_mode);
}
