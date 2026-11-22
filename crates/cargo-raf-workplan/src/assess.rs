// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `assess`: `Workplan::assess_context` (`--context`),
//! `Workplan::record_assessment` (`--verdict`) and
//! `Workplan::list_assessments` (`--list`).

use std::path::Path;

use crate::result::Result as CommandResult;
use raf_workplan::Gate;
use raf_workplan::assessment::{AssessContext, Assessed, AssessmentRequest, AssessmentState};
use raf_workplan::escape::shown_lines;
use raf_workplan::record::RecordOptions;

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

/// The record of the context of a section. Its text is the library's, with
/// each control and bidirectional formatting character but the line feed and
/// the tab as its escape (`raf_workplan::escape::shown_lines`), since it holds
/// the text of the section and of the bound code; the RDF form retains the typed text.
fn context_record(context: &AssessContext) -> Record {
    Record::outcome(
        CommandResult::AssessmentContext(context.clone()),
        shown_lines(&context.to_string()),
    )
}

/// The record of a written assessment. Its text is the library's.
fn assessed_record(assessed: &Assessed) -> Record {
    Record::outcome(
        CommandResult::AssessmentRecorded(assessed.clone()),
        assessed.to_string(),
    )
}

/// The record of one assessment in the list. Its text is the library's line.
fn state_record(state: &AssessmentState) -> Record {
    Record::outcome(CommandResult::Assessment(state.clone()), state.to_string())
}
