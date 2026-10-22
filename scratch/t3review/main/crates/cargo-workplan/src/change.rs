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

use raf_workplan::exec::{Programs, SystemRunner};
use raf_workplan::lifecycle::{
    self, Completion, Failure as Refusal, Impact, Lifecycle, NewChange, NewChangeRequest,
};
use raf_workplan::obligation::Obligation;
use raf_workplan::{Gate, Workplan};
use serde_json::{Value, json};

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
    let affects: Vec<String> = created.affects.iter().map(ToString::to_string).collect();
    let guidance: Vec<Value> = created
        .guidance
        .iter()
        .map(|entry| {
            json!({
                "capability": entry.capability,
                "state": entry.state.name(),
                "advice": entry.advice.name(),
            })
        })
        .collect();
    Record::outcome(
        "new-change",
        vec![
            ("slug", Value::from(created.slug.as_str())),
            ("path", Value::from(created.path.as_str())),
            ("capabilities", json!(created.capabilities)),
            ("affects", json!(affects)),
            ("template", Value::from(created.template.as_str())),
            ("guidance", Value::Array(guidance)),
            ("sync_needed", Value::from(created.sync_needed)),
        ],
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
    let bindings: Vec<Value> = impact
        .bindings
        .iter()
        .map(|binding| {
            let reason = binding
                .resolution()
                .err()
                .map(ToString::to_string)
                .map_or(Value::Null, Value::from);
            json!({
                "capability": binding.capability(),
                "list": binding.kind().key(),
                "path": binding.path(),
                "resolves": binding.resolution().is_ok(),
                "reason": reason,
            })
        })
        .collect();
    let sections: Vec<String> = impact.sections.iter().map(ToString::to_string).collect();
    let dependants: Vec<Value> = impact
        .dependants
        .iter()
        .map(|dependant| {
            json!({
                "capability": dependant.capability,
                "depends_on": dependant.depends_on,
            })
        })
        .collect();
    let linking: Vec<Value> = impact
        .linking
        .iter()
        .map(|link| json!({ "from": link.from.to_string(), "target": link.target.to_string() }))
        .collect();
    Record::outcome(
        "impact",
        vec![
            ("change", Value::from(impact.change.as_str())),
            ("capabilities", json!(impact.capabilities)),
            ("sections", json!(sections)),
            ("bindings", Value::Array(bindings)),
            ("dependants", Value::Array(dependants)),
            ("linking", Value::Array(linking)),
        ],
        impact.to_string(),
    )
}

/// The lifecycle of the repository at `root`: the Git version is asked first,
/// then the repository is opened.
fn open(root: &Path) -> Result<Lifecycle, Failure> {
    Ok(Lifecycle::open(Programs::new(root))?)
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
        "obligation",
        vec![
            ("obligation", Value::from(obligation.kind.name())),
            ("subject", Value::from(obligation.subject.to_string())),
            ("section", Value::from(obligation.section.to_string())),
            ("path", Value::from(obligation.location.path.as_str())),
            (
                "line",
                obligation.location.line.map_or(Value::Null, Value::from),
            ),
            ("evidence", Value::from(obligation.evidence.to_string())),
        ],
        obligation.to_string(),
    )
}

/// `change complete <slug>`: the completion gate. A refusal lists every
/// failure and writes nothing; a completion has removed the record.
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
        Completion::Completed { removed, rewritten } => {
            let mut records = vec![Record::outcome(
                "completed",
                vec![("path", Value::from(removed.as_str()))],
                completion.headline(),
            )];
            records.extend(rewritten.iter().map(|path| {
                Record::outcome(
                    "rewrote",
                    vec![("path", Value::from(path.as_str()))],
                    format!("rewrote {path}"),
                )
            }));
            Ok(records)
        }
        Completion::Refused { failures } => {
            let mut records = vec![Record::finding(
                "refused",
                vec![
                    ("failures", Value::from(failures.len())),
                    ("message", Value::from(completion.headline())),
                ],
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
    let condition = failure.condition();
    let repeated = failure.is_repeated_in(all);
    let message = failure.to_string();
    let text = if repeated {
        String::new()
    } else {
        message.clone()
    };
    Record::finding(
        "refusal",
        vec![
            ("condition", Value::from(condition.name())),
            (
                "number",
                condition.number().map_or(Value::Null, Value::from),
            ),
            ("source", Value::from(failure.name())),
            ("message", Value::from(message)),
            ("repeated", Value::from(repeated)),
        ],
        text,
    )
}

/// `verify [<slug>]`: run the bound tests and examples of the change, or of
/// every capability, and print one line for each binding. Nothing is written.
/// Git is not asked anything.
pub(crate) fn verify(root: &Path, slug: Option<&str>) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let runner = SystemRunner::new(Programs::new(root));
    let verified = lifecycle::verify(&opened, &runner, slug)?;
    let records: Vec<Record> = verified
        .evidence()
        .iter()
        .zip(verified.lines())
        .map(|(evidence, line)| {
            let fields = vec![
                ("outcome", Value::from(evidence.outcome().name())),
                ("capability", Value::from(evidence.capability())),
                ("list", Value::from(evidence.kind().key())),
                ("path", Value::from(evidence.path())),
                ("proof", Value::from(evidence.proof().to_string())),
                (
                    "reason",
                    evidence.outcome().reason().map_or(Value::Null, Value::from),
                ),
                ("message", Value::from(line.as_str())),
            ];
            if evidence.passed() {
                Record::outcome("binding", fields, line)
            } else {
                Record::finding("binding", fields, line)
            }
        })
        .collect();
    let mut report = Report::of(records);
    if verified.is_empty() {
        report.note("there is no bound test or example to run");
    }
    Ok(report)
}
