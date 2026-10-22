// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Counted work of the complete policy: loading judges no content unit,
//! while every complete check judges each delivered capability and each
//! binding-reached package once.

#[allow(
    dead_code,
    reason = "the unit tests read a part of the fixture the integration tests share"
)]
#[path = "../../tests/support/strict.rs"]
pub(crate) mod fixture;

use raf_source::{MemoryTree, Workspace};

use crate::Workplan;
use crate::work::{Work, measure_kind};

fn tree(files: &[(&'static str, String)]) -> MemoryTree {
    files
        .iter()
        .map(|(path, text)| (*path, text.as_str()))
        .collect()
}

/// The otherwise complete documents with a thin Examples section and the
/// supplied coverage-record fields.
fn documents(record: &str) -> MemoryTree {
    tree(&[
        (fixture::CAPABILITY, fixture::thin("Examples")),
        (fixture::COVERAGE, fixture::coverage(record)),
    ])
}

#[test]
fn the_default_check_judges_each_policy_unit_once_without_policy_work_during_load() {
    let source = Workspace::load(&tree(&fixture::source_files()), &["."]).expect("the source");
    let docs = documents("");
    let (plan, load) = measure_kind(Work::StrictUnits, 0, || Workplan::load(&docs, &source));
    assert_eq!(load.get(Work::StrictUnits), 0);
    let (found, tally) = measure_kind(Work::StrictUnits, 2, || plan.check());
    assert_eq!(tally.get(Work::StrictUnits), 2);
    assert!(
        found
            .iter()
            .any(|finding| finding.kind == "strict-section-thin"),
        "{found:#?}"
    );
}
