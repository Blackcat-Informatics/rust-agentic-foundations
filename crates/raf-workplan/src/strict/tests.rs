// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The counted work of strict mode: under `default` neither the load nor
//! `check` judges a unit of it, and under strict `check` judges each delivered
//! capability and each package that strict mode alone enforces once.

#[allow(
    dead_code,
    reason = "the unit tests read a part of the fixture the integration tests share"
)]
#[path = "../../tests/support/strict.rs"]
pub(crate) mod fixture;

use raf_source::{MemoryTree, Workspace};

use crate::work::{Work, measure_kind};
use crate::{Strictness, Workplan};

fn tree(files: &[(&'static str, String)]) -> MemoryTree {
    files
        .iter()
        .map(|(path, text)| (*path, text.as_str()))
        .collect()
}

/// The strict-clean documents with a thin Examples section, under a record
/// whose `strictness` line is `record`.
fn documents(record: &str) -> MemoryTree {
    tree(&[
        (fixture::CAPABILITY, fixture::thin("Examples")),
        (fixture::COVERAGE, fixture::coverage(record)),
    ])
}

#[test]
fn under_default_no_unit_of_strict_mode_is_judged_and_under_strict_each_is_judged_once() {
    let source = Workspace::load(&tree(&fixture::source_files()), &["."]).expect("the source");
    let docs = documents("");
    // The ceiling of 0 stops the run at the first unit of strict mode.
    let (plan, load) = measure_kind(Work::StrictUnits, 0, || Workplan::load(&docs, &source));
    assert_eq!(load.get(Work::StrictUnits), 0);
    let (found, tally) = measure_kind(Work::StrictUnits, 0, || plan.check());
    assert_eq!(tally.get(Work::StrictUnits), 0);
    assert!(
        !found
            .iter()
            .any(|finding| finding.kind.starts_with("strict-")),
        "{found:#?}"
    );

    // A record that says strict loads with no more work, and its check judges
    // the one delivered capability and the one package it binds.
    let strict_docs = documents("strictness: strict\n");
    let (plan, load) = measure_kind(Work::StrictUnits, 0, || {
        Workplan::load(&strict_docs, &source)
    });
    assert_eq!(load.get(Work::StrictUnits), 0);
    let (found, tally) = measure_kind(Work::StrictUnits, 2, || plan.check());
    assert_eq!(tally.get(Work::StrictUnits), 2);
    assert!(
        found
            .iter()
            .any(|finding| finding.kind == "strict-section-thin"),
        "{found:#?}"
    );

    // A floor gives the same judgement as the record.
    let plan = Workplan::load(&docs, &source).with_strictness(Strictness::Strict);
    let (_, tally) = measure_kind(Work::StrictUnits, 2, || plan.check());
    assert_eq!(tally.get(Work::StrictUnits), 2);
}
