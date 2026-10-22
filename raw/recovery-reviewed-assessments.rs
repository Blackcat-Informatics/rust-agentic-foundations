// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Capture or publish only explicitly reviewed subjects through production APIs.

use std::collections::BTreeSet;
use std::error::Error;
use std::path::PathBuf;

use raf_okf::{Actor, AssessmentVerdict};
use raf_workplan::Gate;
use raf_workplan::assessment::AssessmentRequest;
use raf_workplan::record::RecordOptions;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let mode = args.next().ok_or("expected stale, capture or publish")?;
    if mode == "stale" {
        if args.next().is_some() {
            return Err("unexpected argument".into());
        }
        let opened = Gate::workspace(std::env::current_dir()?).open()?;
        for state in opened.workplan().list_assessments() {
            if !state.currency.is_current() {
                println!("{}", state.subject);
            }
        }
        return Ok(());
    }
    if !matches!(mode.as_str(), "capture" | "publish") {
        return Err("expected capture or publish".into());
    }
    let selected = args.next().ok_or("expected reviewed-subject list")?;
    let output = PathBuf::from(args.next().ok_or("expected context directory")?);
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let text = std::fs::read_to_string(selected)?;
    let mut subjects = BTreeSet::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if line != line.trim() || !subjects.insert(line.to_owned()) {
            return Err(format!("invalid or duplicate reviewed subject: {line:?}").into());
        }
    }
    if subjects.is_empty() {
        return Err("no reviewed subjects selected".into());
    }
    let root = std::env::current_dir()?;
    let opened = Gate::workspace(&root).open()?;
    let plan = opened.workplan();
    if mode == "capture" {
        std::fs::create_dir_all(&output)?;
    }
    // Validate every current context before publishing any record.
    let mut contexts = Vec::new();
    for subject in subjects {
        let context = plan.assess_context(&subject)?;
        let path = output.join(format!("{}.txt", context.record.trim_start_matches("assessment:")));
        let rendered = context.to_string();
        if mode == "capture" {
            std::fs::write(path, &rendered)?;
        } else if std::fs::read_to_string(&path)? != rendered {
            return Err(format!("context changed since review: {subject}").into());
        }
        contexts.push(context);
    }
    for context in contexts {
        if mode == "publish" {
            let options = RecordOptions::new()
                .copyright("2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>")
                .licence("CC-BY-4.0");
            let request = AssessmentRequest::new(
                &context.subject,
                AssessmentVerdict::Accepted,
                Actor::Agent,
            ).options(options);
            plan.record_assessment(&root, &request)?;
        }
        println!("{mode}: {}", context.subject);
    }
    if mode == "publish" {
        let fresh = raf_workplan::Workplan::load_with(
            opened.tree(), opened.workspace(), &opened.links(),
        );
        let states = fresh.list_assessments();
        println!("currency: {} current / {} total",
            states.iter().filter(|state| state.currency.is_current()).count(), states.len());
        for state in states {
            println!("{state}");
        }
    }
    Ok(())
}
