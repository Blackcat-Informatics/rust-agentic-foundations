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
use raf_workplan::exec::{Programs, SystemRunner};
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

/// The lifecycle of the repository at `root`, judged under the complete policy:
/// the Git version is asked first, then the repository is opened.
fn open(root: &Path) -> Result<Lifecycle, Failure> {
    let programs = Programs::new(root);
    let gate = crate::command::gate(root).programs(programs.clone());
    Ok(Lifecycle::open_with(&gate, SystemRunner::new(programs))?)
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
/// Trusted bound programs can have their own filesystem effects.
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
    let receipts = || {
        completion.evidence().iter().filter_map(|evidence| {
            if !matches!(
                evidence.outcome(),
                raf_workplan::bound::BoundOutcome::NotApplicable { .. }
            ) {
                return None;
            }
            let line = evidence.to_string();
            Some(Record::outcome(
                CommandResult::Binding {
                    evidence: evidence.clone(),
                    message: line.clone(),
                },
                line,
            ))
        })
    };
    match completion {
        Completion::Completed {
            removed, rewritten, ..
        } => {
            let mut records = vec![Record::outcome(
                CommandResult::Completed {
                    path: removed.clone(),
                    evidence: completion.evidence().to_vec(),
                },
                completion.headline(),
            )];
            records.extend(rewritten.iter().map(|path| {
                Record::outcome(
                    CommandResult::Rewritten { path: path.clone() },
                    format!("rewrote {path}"),
                )
            }));
            records.extend(receipts());
            Ok(records)
        }
        Completion::Refused { failures, .. } => {
            let mut records = vec![Record::finding(
                CommandResult::Refused {
                    failures: failures.len(),
                    message: completion.headline(),
                    evidence: completion.evidence().to_vec(),
                },
                completion.headline(),
            )];
            records.extend(
                failures
                    .iter()
                    .map(|failure| refusal_record(failure, failures)),
            );
            records.extend(receipts());
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

/// `verify [<slug>]`: run the bound tests and examples of the change, or of
/// every capability, and print one line for each binding. The tool publishes no
/// repository files; trusted Cargo builds, tests and examples can write files.
/// Git is used only to identify the worktree and list its tracked paths.
pub(crate) fn verify(root: &Path, slug: Option<&str>) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let runner = SystemRunner::new(Programs::new(root));
    let verified = lifecycle::verify(&opened, &runner, slug)?;
    let records: Vec<Record> = verified
        .evidence()
        .iter()
        .zip(verified.lines())
        .map(|(evidence, line)| {
            let result = CommandResult::Binding {
                evidence: evidence.clone(),
                message: line.clone(),
            };
            if matches!(
                evidence.outcome(),
                raf_workplan::bound::BoundOutcome::Passed
                    | raf_workplan::bound::BoundOutcome::NotApplicable { .. }
            ) {
                Record::outcome(result, line)
            } else {
                Record::finding(result, line)
            }
        })
        .collect();
    let mut report = Report::of(records);
    if verified.is_empty() {
        report.note("there is no bound test or example to run");
    }
    Ok(report)
}
