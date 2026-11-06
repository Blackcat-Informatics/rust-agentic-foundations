// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The change commands: `change new`, `change impact`, `change reconcile`,
//! `change complete` and `verify`.
//!
//! Each one calls the function of `raf-workplan` that does the work
//! (`Workplan::new_change`, `Workplan::impact`, `Lifecycle::reconcile`,
//! `Lifecycle::complete`, `lifecycle::verify`) and turns what it returns into
//! records. The text of a record is the `Display` of the library value, so
//! what the command prints is what the library says.

use std::path::Path;

use crate::result::Result as CommandResult;
use raf_workplan::lifecycle::{
    self, Completion, Failure as Refusal, Impact, Lifecycle, NewChange, NewChangeRequest,
};
use raf_workplan::obligation::Obligation;
use raf_workplan::{Gate, Workplan};

use crate::cli::NewArgs;
use crate::exit::Failure;
use crate::render::{Record, Report};

/// `change new <slug>`: create the record and print what to do next.
pub(crate) fn new(root: &Path, args: &NewArgs) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan: Workplan<'_> = opened.workplan();
    let mut request = NewChangeRequest::new(args.slug.as_str());
    for capability in &args.capability {
        request = request.capability(capability.as_str());
    }
    for entry in &args.affects {
        request = request.affects(entry.as_str());
    }
    if let Some(copyright) = &args.copyright {
        request = request.copyright(copyright.as_str());
    }
    if let Some(licence) = &args.licence {
        request = request.licence(licence.as_str());
    }
    let created = plan.new_change(root, &request)?;
    Ok(Report::of([new_change_record(&created)]))
}

/// The record of a created change.
fn new_change_record(created: &NewChange) -> Record {
    Record::outcome(
        CommandResult::NewChange(created.clone()),
        created.to_string(),
    )
}

/// `change impact <slug>`: what the change reaches. Nothing is written and no
/// process is started.
pub(crate) fn impact(root: &Path, slug: &str) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    let impact = plan.impact(slug)?;
    Ok(Report::of([impact_record(&impact)]))
}

/// The record of an impact.
fn impact_record(impact: &Impact) -> Record {
    Record::outcome(CommandResult::Impact(impact.clone()), impact.to_string())
}

/// Open the native repository and complete static document policy.
fn open(root: &Path) -> Result<Lifecycle, Failure> {
    Ok(Lifecycle::open_with(&Gate::workspace(root))?)
}

/// `change reconcile <slug>`: one line for each obligation, in the order the
/// library sorts them. The run exits 1 exactly when the list is not empty.
pub(crate) fn reconcile(root: &Path, slug: &str) -> Result<Report, Failure> {
    let owed = open(root)?.reconcile(slug)?;
    Ok(Report::of(owed.obligations().iter().map(obligation_record)))
}

/// The record of an obligation. The text is its line in the shape of the
/// reconcile contract.
fn obligation_record(obligation: &Obligation) -> Record {
    Record::finding(
        CommandResult::Obligation(obligation.clone()),
        obligation.to_string(),
    )
}

/// `change complete <slug>`: the completion gate. A refusal lists every
/// failure without tool-owned publication; a completion has removed the record.
/// Consumer source bodies remain data during completion.
pub(crate) fn complete(root: &Path, slug: &str) -> Result<Report, Failure> {
    let completion = open(root)?.complete(slug)?;
    Ok(Report::of(completion_records(&completion)?))
}

/// The records of a completion.
///
/// # Errors
///
/// A completion of a kind this version does not know.
fn completion_records(completion: &Completion) -> Result<Vec<Record>, Failure> {
    match completion {
        Completion::Completed {
            removed, rewritten, ..
        } => {
            let mut records = vec![Record::outcome(
                CommandResult::Completed {
                    path: removed.clone(),
                },
                completion.headline(),
            )];
            records.extend(rewritten.iter().map(|path| {
                Record::outcome(
                    CommandResult::Rewritten { path: path.clone() },
                    format!("rewrote {path}"),
                )
            }));
            Ok(records)
        }
        Completion::Refused { failures, .. } => {
            let mut records = vec![Record::finding(
                CommandResult::Refused {
                    failures: failures.len(),
                    message: completion.headline(),
                },
                completion.headline(),
            )];
            records.extend(
                failures
                    .iter()
                    .map(|failure| refusal_record(failure, failures)),
            );
            Ok(records)
        }
        _ => Err(Failure::new(
            "the library gave a completion this version of cargo-workplan does not know; it is not a result to print",
        )),
    }
}

/// The record of one failure of a refusal, with the condition it is a fault
/// of. A failure whose line another failure already prints has no text.
fn refusal_record(failure: &Refusal, all: &[Refusal]) -> Record {
    let repeated = failure.is_repeated_in(all);
    let text = if repeated {
        String::new()
    } else {
        failure.to_string()
    };
    Record::finding(
        CommandResult::Refusal {
            failure: failure.clone(),
            repeated,
        },
        text,
    )
}

/// Validate declared source bindings and document contracts without executing code.
pub(crate) fn verify(root: &Path, slug: Option<&str>) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let verified = lifecycle::verify(&opened, slug)?;
    let text = verified.to_string();
    let valid = verified.is_valid();
    let result = CommandResult::Verification(verified);
    Ok(Report::of([if valid {
        Record::outcome(result, text)
    } else {
        Record::finding(result, text)
    }]))
}
