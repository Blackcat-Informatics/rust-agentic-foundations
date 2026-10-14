// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The committed fuzz corpus is replayed under `cargo test`, so a file that
//! once aborted or slipped past a budget fails here without running the
//! fuzzer. Every file is judged in a child process and must end in a typed
//! refusal or in the index it records; an abort fails the replay.

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use support::child::{Outcome, problem, run_child, run_child_if_requested};
use support::shapes::{self, EXPECTED_NAMES};

/// The selector prefix that names a corpus file.
const PREFIX: &str = "corpus/";

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fuzz/corpus/parse_source")
}

fn corpus_file(name: &str) -> PathBuf {
    corpus_dir().join(format!("{name}.rs"))
}

fn text_of(selector: &str) -> String {
    let name = selector
        .strip_prefix(PREFIX)
        .unwrap_or_else(|| panic!("selector {selector} does not name a corpus file"));
    fs::read_to_string(corpus_file(name)).unwrap_or_else(|e| panic!("cannot read {name}: {e}"))
}

/// Selected by the parent process; an ordinary run of it does nothing.
#[test]
fn child_main() {
    run_child_if_requested(text_of);
}

/// The file names in the corpus directory, sorted.
fn names_on_disk() -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(corpus_dir())
        .expect("the corpus directory exists")
        .map(|entry| {
            entry
                .expect("directory entry")
                .file_name()
                .into_string()
                .expect("corpus file names are UTF-8")
        })
        .collect();
    names.sort();
    names
}

#[test]
fn every_named_shape_has_a_corpus_file_and_no_file_is_unrecorded() {
    let on_disk = names_on_disk();
    for name in EXPECTED_NAMES {
        assert!(
            on_disk.contains(&format!("{name}.rs")),
            "the corpus lacks {name}.rs"
        );
    }
    for file in &on_disk {
        let stem = file.strip_suffix(".rs").unwrap_or(file);
        assert!(
            shapes::all().iter().any(|shape| shape.name == stem),
            "{file} has no recorded symbol list; record it in the shape table"
        );
    }
    assert_eq!(on_disk.len(), EXPECTED_NAMES.len());
}

#[test]
fn the_shape_table_matches_the_expected_names() {
    let built: Vec<&str> = shapes::all().iter().map(|shape| shape.name).collect();
    assert_eq!(built, EXPECTED_NAMES);
}

#[test]
fn corpus_files_hold_exactly_the_text_of_their_shapes() {
    for shape in shapes::all() {
        let on_disk = fs::read_to_string(corpus_file(shape.name))
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", shape.name));
        assert!(
            on_disk == shape.text,
            "{} differs from its shape; regenerate with \
             `cargo test -p raf-source --test corpus_replay -- --ignored write_corpus`",
            shape.name
        );
    }
}

#[test]
fn every_corpus_file_is_refused_or_indexed_with_its_recorded_symbols_and_none_aborts() {
    for shape in shapes::all() {
        let outcome = run_child("child_main", &format!("{PREFIX}{}", shape.name));
        let named = format!("{PREFIX}{}.rs", shape.name);
        assert_eq!(
            problem(&outcome, &named, shape.symbols),
            None,
            "{}: {outcome:?}",
            shape.name
        );
    }
}

#[test]
fn files_at_a_budget_exit_zero_and_files_over_it_are_refused_naming_file_and_budget() {
    for (at, over, budget) in [
        (
            "budget_delimiters_at",
            "budget_delimiters_over",
            "delimiter nesting",
        ),
        ("budget_angles_at", "budget_angles_over", "nested `<`"),
        (
            "budget_prefix_at",
            "budget_prefix_over",
            "prefix operator run",
        ),
        (
            "budget_closure_bars_at",
            "budget_closure_bars_over",
            "closure bar chain",
        ),
        (
            "budget_else_if_at",
            "budget_else_if_over",
            "`else if` chain",
        ),
        (
            "budget_recursion_at",
            "budget_recursion_over",
            "parser recursion",
        ),
    ] {
        let symbols: Vec<String> = shapes::all()
            .into_iter()
            .find(|shape| shape.name == at)
            .map(|shape| shape.symbols.iter().map(|s| (*s).to_owned()).collect())
            .expect("shape exists");
        assert_eq!(
            run_child("child_main", &format!("{PREFIX}{at}")),
            Outcome::Index(symbols),
            "{at}"
        );
        assert_eq!(
            run_child("child_main", &format!("{PREFIX}{over}")),
            Outcome::Refused {
                file: format!("{PREFIX}{over}.rs"),
                budget: budget.to_owned(),
            },
            "{over}"
        );
    }
}

#[test]
fn evasion_shapes_are_refused_naming_their_budget_beside_their_at_budget_neighbours() {
    for (at, over, budget) in [
        (
            "evasion_else_if_block_condition_at",
            "evasion_else_if_block_condition_20000",
            "`else if` chain",
        ),
        (
            "evasion_ref_lifetime_at",
            "evasion_ref_lifetime_20000",
            "prefix operator run",
        ),
    ] {
        let symbols: Vec<String> = shapes::all()
            .into_iter()
            .find(|shape| shape.name == at)
            .map(|shape| shape.symbols.iter().map(|s| (*s).to_owned()).collect())
            .expect("shape exists");
        assert_eq!(
            run_child("child_main", &format!("{PREFIX}{at}")),
            Outcome::Index(symbols),
            "{at}"
        );
        assert_eq!(
            run_child("child_main", &format!("{PREFIX}{over}")),
            Outcome::Refused {
                file: format!("{PREFIX}{over}.rs"),
                budget: budget.to_owned(),
            },
            "{over}"
        );
    }
}

#[test]
fn a_recorded_list_that_differs_from_the_real_index_fails_the_replay() {
    let outcome = run_child("child_main", &format!("{PREFIX}budget_prefix_at"));
    assert_eq!(
        problem(&outcome, "corpus/budget_prefix_at.rs", &["f"]),
        None
    );
    let wrong = problem(&outcome, "corpus/budget_prefix_at.rs", &["g"]);
    assert!(wrong.is_some(), "a wrong list must fail: {outcome:?}");
    let partial = problem(&outcome, "corpus/budget_prefix_at.rs", &[]);
    assert!(partial.is_some(), "a missing symbol must fail: {outcome:?}");
}

/// Writes the committed corpus from the shape table.
#[test]
#[ignore = "writes the committed corpus files from the shape table"]
fn write_corpus() {
    fs::create_dir_all(corpus_dir()).expect("create the corpus directory");
    for shape in shapes::all() {
        fs::write(corpus_file(shape.name), &shape.text).expect("write a corpus file");
    }
}
