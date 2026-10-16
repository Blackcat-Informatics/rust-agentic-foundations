// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `assess`: `Workplan::assess_context` (`--context`),
//! `Workplan::record_assessment` (`--verdict`) and
//! `Workplan::list_assessments` (`--list`).

use std::path::Path;

use raf_okf::{EvidenceVerdict, Measure};
use raf_workplan::Gate;
use raf_workplan::assessment::{
    AssessContext, Assessed, AssessmentRequest, AssessmentState, Currency, InputChange, InputNow,
    MovedInput,
};
use raf_workplan::record::RecordOptions;
use serde_json::{Value, json};

use crate::cli::{AssessArgs, Verdict};
use crate::exit::Failure;
use crate::render::{Record, Report};

/// `assess`: the context, a recorded verdict, or the list. Each prints
/// outcomes only, so each exits 0 when it has an answer: a rejected verdict
/// was recorded, and a stale or rejected assessment in the list is shown with
/// its currency and left to `change reconcile`, which lists what it owes. A
/// section given to `--list` must be one `--context` answers for, so an
/// unknown capability is exit 2 in all three modes.
pub(crate) fn assess(root: &Path, args: &AssessArgs) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    if args.list {
        // A section to list is one the library can assess: a capability or a
        // section that no loaded document has is an error, as for `--context`.
        if let Some(subject) = &args.subject {
            plan.assess_context(subject)?;
        }
        let states = plan.list_assessments();
        let shown: Vec<&AssessmentState> = states
            .iter()
            .filter(|state| args.subject.as_ref().is_none_or(|s| *s == state.subject))
            .collect();
        let mut report = Report::of(shown.iter().map(|state| state_record(state)));
        if shown.is_empty() {
            report.note(match &args.subject {
                Some(subject) => format!("no assessment of {subject} loaded"),
                None => "no assessment record loaded".to_string(),
            });
        }
        return Ok(report);
    }
    let subject = args
        .subject
        .as_deref()
        .ok_or_else(|| Failure::new("give the section to assess, `cap:<slug>#<section>`"))?;
    let Some(verdict) = args.verdict else {
        let context = plan.assess_context(subject)?;
        return Ok(Report::of([context_record(&context)]));
    };
    let by = args
        .by
        .ok_or_else(|| Failure::new("--verdict needs --by human or --by agent: who assessed"))?;
    let mut options = RecordOptions::new();
    if let Some(copyright) = &args.copyright {
        options = options.copyright(copyright.as_str());
    }
    if let Some(licence) = &args.licence {
        options = options.licence(licence.as_str());
    }
    let request =
        AssessmentRequest::new(subject, Verdict::verdict(verdict), by.actor()).options(options);
    let assessed = plan.record_assessment(root, &request)?;
    Ok(Report::of([assessed_record(&assessed)]))
}

/// The record of the context of a section. Its text is the library's.
fn context_record(context: &AssessContext) -> Record {
    let symbols: Vec<Value> = context
        .symbols
        .iter()
        .map(|symbol| {
            let definitions: Vec<Value> = symbol
                .definitions
                .iter()
                .map(|definition| json!({ "file": definition.file, "text": definition.text }))
                .collect();
            json!({
                "path": symbol.path,
                "named": symbol.named,
                "definitions": definitions,
                "unresolved": symbol.unresolved,
            })
        })
        .collect();
    let evidence: Vec<Value> = context
        .evidence
        .iter()
        .map(|evidence| {
            let measures = |list: &[Measure]| -> Vec<Value> {
                list.iter()
                    .map(|m| json!({ "name": m.name, "value": m.value.text() }))
                    .collect()
            };
            json!({
                "id": evidence.id,
                "path": evidence.path,
                "verdict": evidence.verdict.map(EvidenceVerdict::name),
                "values": measures(&evidence.values),
                "limits": measures(&evidence.limits),
            })
        })
        .collect();
    let inputs: Vec<Value> = context
        .inputs
        .iter()
        .map(|input| {
            let (digest, unresolved) = match &input.now {
                InputNow::Digest(digest) => (Some(digest.to_string()), None),
                InputNow::Unresolved(reason) => (None, Some(reason.clone())),
                _ => (None, None),
            };
            json!({
                "kind": input.kind.name(),
                "name": input.name,
                "digest": digest,
                "unresolved": unresolved,
            })
        })
        .collect();
    Record::outcome(
        "assess-context",
        vec![
            ("subject", Value::from(context.subject.as_str())),
            ("record", Value::from(context.record.as_str())),
            ("record_path", Value::from(context.record_path.as_str())),
            ("document", Value::from(context.document.as_str())),
            ("section", Value::from(context.section.as_str())),
            ("symbols", Value::Array(symbols)),
            ("evidence", Value::Array(evidence)),
            ("inputs", Value::Array(inputs)),
        ],
        context.to_string(),
    )
}

/// The record of a written assessment. Its text is the library's.
fn assessed_record(assessed: &Assessed) -> Record {
    let record = &assessed.record;
    let inputs: Vec<Value> = record
        .inputs
        .iter()
        .map(|input| json!({ "kind": input.kind.name(), "name": input.name, "digest": input.digest }))
        .collect();
    Record::outcome(
        "assessment-recorded",
        vec![
            ("path", Value::from(assessed.path.as_str())),
            ("id", Value::from(record.header.id.as_str())),
            ("subject", Value::from(record.subject.text())),
            ("verdict", Value::from(record.verdict.name())),
            ("by", Value::from(record.by.name())),
            ("replaced", Value::from(assessed.replaced)),
            ("inputs", Value::Array(inputs)),
        ],
        assessed.to_string(),
    )
}

/// The name of how an input moved, as the JSON form writes it.
fn change_name(change: &InputChange) -> &'static str {
    match change {
        InputChange::Changed => "changed",
        InputChange::Unresolved(_) => "unresolved",
        InputChange::NoLongerInput => "no-longer-input",
        InputChange::NewInput => "new-input",
        _ => "other",
    }
}

/// A moved input as a JSON object.
fn moved(input: &MovedInput) -> Value {
    json!({
        "kind": input.kind.name(),
        "name": input.name,
        "change": change_name(&input.change),
        "message": input.to_string(),
    })
}

/// The record of one assessment in the list. Its text is the library's line.
fn state_record(state: &AssessmentState) -> Record {
    let moved: Vec<Value> = match &state.currency {
        Currency::Stale(inputs) => inputs.iter().map(moved).collect(),
        _ => Vec::new(),
    };
    Record::outcome(
        "assessment",
        vec![
            ("id", Value::from(state.id.as_str())),
            ("path", Value::from(state.path.as_str())),
            ("subject", Value::from(state.subject.as_str())),
            ("verdict", Value::from(state.verdict.name())),
            ("by", Value::from(state.by.name())),
            ("current", Value::from(state.currency.is_current())),
            ("moved", Value::Array(moved)),
        ],
        state.to_string(),
    )
}
