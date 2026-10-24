// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native mechanics over an explicit owner-reviewed, frozen subject list.

use std::{error::Error, io, path::PathBuf};

use raf_okf::{Actor, AssessmentVerdict};
use raf_workplan::{Gate, assessment::AssessmentRequest, record::RecordOptions};

fn request(subject: &str) -> AssessmentRequest {
    AssessmentRequest::new(subject, AssessmentVerdict::Accepted, Actor::Agent).options(
        RecordOptions::new()
            .copyright("2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>")
            .licence("CC-BY-4.0"),
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let mode = arguments.next().ok_or("supply context or record-reviewed")?;
    if mode != "context" && mode != "record-reviewed" {
        return Err("mode must be context or record-reviewed".into());
    }
    let root = PathBuf::from(arguments.next().ok_or("supply actual checkout root")?);
    let subjects: Vec<String> = arguments
        .map(|subject| subject.into_string().map_err(|_| io::Error::other("subject is not UTF-8")))
        .collect::<Result<_, _>>()?;
    if subjects.is_empty() {
        return Err("explicit reviewed subject list is empty".into());
    }
    for (index, subject) in subjects.iter().enumerate() {
        if subjects[..index].contains(subject) {
            return Err(format!("duplicate subject {subject}").into());
        }
    }
    let opened = Gate::workspace(&root).open()?;
    let plan = opened.workplan();
    if !plan.findings().is_empty() {
        return Err(format!("native document load findings: {:?}", plan.findings()).into());
    }
    for subject in &subjects {
        let context = plan.assess_context(subject)?;
        if mode == "context" {
            println!("{context}");
        } else {
            plan.assessment_record(&request(subject))?;
        }
    }
    if mode == "context" {
        return Ok(());
    }
    // This mode accepts only the explicitly named subjects already reviewed by
    // the owner. It never selects stale sections or treats preflight as review.
    // Freeze source, authored documents and evidence throughout the batch.
    for subject in &subjects {
        println!("{}", plan.record_assessment(&root, &request(subject))?);
    }
    let refreshed = opened.workplan();
    if !refreshed.findings().is_empty() {
        return Err(format!("native refreshed load findings: {:?}", refreshed.findings()).into());
    }
    let states = refreshed.list_assessments();
    for subject in &subjects {
        let mut found = states.iter().filter(|state| &state.subject == subject);
        let state = found.next().ok_or("written assessment did not reload")?;
        if found.next().is_some()
            || !state.currency.is_current()
            || state.verdict != AssessmentVerdict::Accepted
            || state.by != Actor::Agent
        {
            return Err(format!("written assessment is not exact/current: {subject}").into());
        }
    }
    Ok(())
}
