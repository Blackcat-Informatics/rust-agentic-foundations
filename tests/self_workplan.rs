// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The workplan gate over this repository, from disk.
//!
//! `Gate::workspace` opens the gate root the way a dependent crate's own test
//! does. The repository must load at least one capability, `Workplan::check`
//! must report nothing, and every assessment record must be current and
//! accepted, so a section that changed after it was assessed fails here until
//! it is assessed again. Inside the commit hook the root is a checkout of the
//! index without `.git`, and the gate takes every file of it for a tracked
//! path.

use std::fmt::Write as _;

use raf_okf::AssessmentVerdict;
use raf_workplan::bound::Executor;
use raf_workplan::exec::FakeRunner;
use raf_workplan::{BindingKind, Gate};

#[allow(
    dead_code,
    reason = "only gate_root is used here; the other binaries use the rest of the shared support"
)]
mod support;

use support::gate_root;

#[test]
fn repository_workplan_is_clean_and_assessed() {
    let opened = match Gate::workspace(gate_root()).open() {
        Ok(opened) => opened,
        Err(error) => panic!("workplan gate: {error}"),
    };
    let workplan = opened.workplan();
    assert!(
        !workplan.capabilities().is_empty(),
        "the workplan loaded no capability document"
    );

    let findings = workplan.check();
    let mut report = String::new();
    for finding in &findings {
        let _ = writeln!(report, "  {finding}");
    }
    assert!(
        findings.is_empty(),
        "workplan gate: {} finding(s) in {}:\n{report}",
        findings.len(),
        opened.dir().display()
    );

    let runner = FakeRunner::new();
    let executor = Executor::new(opened.workspace(), &runner).expect("native binding planner");
    let unplannable: Vec<_> = workplan
        .graph()
        .bindings()
        .iter()
        .filter(|binding| matches!(binding.kind(), BindingKind::Test | BindingKind::Example))
        .filter_map(|binding| {
            executor
                .plan(binding)
                .err()
                .map(|error| format!("{} {}: {error}", binding.capability(), binding.path()))
        })
        .collect();
    assert!(
        unplannable.is_empty(),
        "unplannable bindings:\n{}",
        unplannable.join("\n")
    );
    assert!(
        runner.calls().is_empty(),
        "binding planning started a process"
    );

    let offending: Vec<String> = workplan
        .list_assessments()
        .iter()
        .filter(|state| {
            !state.currency.is_current() || state.verdict != AssessmentVerdict::Accepted
        })
        .map(ToString::to_string)
        .collect();
    assert!(
        offending.is_empty(),
        "{} assessment(s) are stale or not accepted:\n  {}",
        offending.len(),
        offending.join("\n  ")
    );
}
