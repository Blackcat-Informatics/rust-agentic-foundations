// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The one helper every fixture that reconciles or completes a change uses to
//! give the authored sections the change affects a current assessment.
//!
//! The record is built by the real [`Workplan::assessment_record`], and on a
//! disk written by the real [`Workplan::record_assessment`], from the
//! documents as they are when the helper runs, so the assessment is current
//! for exactly that state: an edit made afterwards makes it stale, as it does
//! for an author. A section whose assessment cannot be recorded (a bound
//! symbol that does not resolve, a claim on a record that does not load, a
//! capability that does not load) gets none, and what it owes stays visible.
//!
//! The file needs nothing else of the support module, so the tests of the
//! command-line crate include it as it is. The fixtures over an in-memory tree
//! reach it through `synced::assessed`.

use std::path::Path;

use raf_okf::{Actor, AssessmentVerdict};
use raf_workplan::assessment::AssessmentRequest;
use raf_workplan::record::RecordOptions;
use raf_workplan::{Gate, Workplan};

/// The path and text of an assessment by `by` with `verdict` of each authored
/// section in the `affects` of `change`, for the documents of `plan`.
pub(crate) fn records_of(
    plan: &Workplan<'_>,
    change: &str,
    verdict: AssessmentVerdict,
    by: Actor,
) -> Vec<(String, String)> {
    let Ok(impact) = plan.impact(change) else {
        return Vec::new();
    };
    impact
        .sections
        .iter()
        .filter_map(|section| {
            let request = AssessmentRequest::new(section.to_string(), verdict, by);
            let record = plan.assessment_record(&request).ok()?;
            let path = record.path().ok()?.to_string();
            let text = record.text(&RecordOptions::new()).ok()?;
            Some((path, text))
        })
        .collect()
}

/// Record a current accepted assessment by an agent of each authored section
/// `change` affects in the repository at `root`, through
/// [`Workplan::record_assessment`], and return the paths written.
pub(crate) fn assess_on_disk(root: &Path, change: &str) -> Vec<String> {
    assess_on_disk_as(root, change, AssessmentVerdict::Accepted, Actor::Agent)
}

/// As [`assess_on_disk`], with the verdict `verdict` by `by`.
pub(crate) fn assess_on_disk_as(
    root: &Path,
    change: &str,
    verdict: AssessmentVerdict,
    by: Actor,
) -> Vec<String> {
    let opened = Gate::workspace(root)
        .tracked_paths(Vec::new())
        .open()
        .unwrap_or_else(|error| panic!("the fixture opens: {error}"));
    let plan = opened.workplan();
    let Ok(impact) = plan.impact(change) else {
        return Vec::new();
    };
    impact
        .sections
        .iter()
        .filter_map(|section| {
            let request = AssessmentRequest::new(section.to_string(), verdict, by);
            plan.record_assessment(root, &request)
                .ok()
                .map(|assessed| assessed.path.to_string())
        })
        .collect()
}
