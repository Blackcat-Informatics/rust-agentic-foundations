// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The evidence commands: `evidence record` (`Workplan::record_evidence`),
//! `evidence claims` (`Workplan::evidence_claims`) and `evidence for`
//! (`Workplan::evidence_for`).
//!
//! The command of `evidence record -- <program> <arg>...` is the
//! `TrustedArgv` that `raf-workplan` builds from `std::env::args_os()`: the
//! words after the first `--` of the command line of this process. Clap reads
//! the same words as the trailing arguments of the command; [`source`] refuses
//! the run when the two differ, so a `--` that an option took as its value
//! can never shift what runs.

use std::ffi::{OsStr, OsString};
use std::path::Path;

use raf_okf::{CapabilitySection, DocType, EvidenceVerdict, Measure};
use raf_workplan::Gate;
use raf_workplan::escape::shown_lines;
use raf_workplan::evidence::{
    ClaimStatement, EvidenceRequest, Failed, Recorded, Support, VerdictSource, WriteMode,
};
use raf_workplan::exec::{Programs, SystemRunner, TrustedArgv};
use raf_workplan::record::RecordOptions;
use serde_json::{Value, json};

use crate::cli::RecordArgs;
use crate::exit::Failure;
use crate::render::{Record, Report};

/// The word that starts the command of `evidence record`.
const SEPARATOR: &str = "--";

/// The source of the verdict of `evidence record`: the bound test or example
/// `--from-binding` names, or the command after `--`. Nothing is read and
/// nothing is started here, so a refusal comes before root discovery.
///
/// # Errors
///
/// A [`Failure`] when the program name of the process is `--`, when
/// `--from-binding` is given beside a `--`, when the command line holds no
/// command after `--`, its program is the empty word or a word of it is not
/// UTF-8, or when the words after its first `--` are not the command the
/// parser read.
pub(crate) fn source(args: &RecordArgs) -> Result<VerdictSource, Failure> {
    line(std::env::args_os(), args.from_binding.is_some())?;
    if let Some(path) = &args.from_binding {
        return Ok(VerdictSource::Binding(path.clone()));
    }
    let trusted = TrustedArgv::from_command_line(std::env::args_os())?;
    let words: Vec<&OsStr> = trusted
        .words()
        .iter()
        .map(|word| OsStr::new(word.as_str()))
        .collect();
    agree(&words, &args.command)?;
    Ok(VerdictSource::Command(trusted))
}

/// Refuse a command line whose program name is `--`, which the first-`--`
/// reading would take as the start of the command, and one that gives
/// `--from-binding` and a `--`, even with nothing after it.
fn line(words: impl IntoIterator<Item = OsString>, from_binding: bool) -> Result<(), Failure> {
    let mut words = words.into_iter();
    if words.next().is_some_and(|program| program == SEPARATOR) {
        return Err(Failure::new(
            "the program name of this process is `--`, which would read as the start of the command; run cargo-workplan under its own name; nothing was run",
        ));
    }
    if from_binding && words.any(|word| word == SEPARATOR) {
        return Err(Failure::new(
            "`--from-binding` and a command after `--` exclude each other; give one of them; nothing was run",
        ));
    }
    Ok(())
}

/// Refuse the command when the words after the first `--` differ from the
/// trailing words the parser read.
fn agree(words: &[&OsStr], parsed: &[OsString]) -> Result<(), Failure> {
    let read: Vec<&OsStr> = parsed.iter().map(OsString::as_os_str).collect();
    if words == read.as_slice() {
        Ok(())
    } else {
        Err(Failure::new(
            "the words after the first `--` of the command line are not the command the arguments end with; give each option its own value and put the command after one `--`; nothing was run",
        ))
    }
}

/// `evidence record`: run the source of the verdict and write the record. A
/// record written with the verdict `pass` is an outcome; one written with
/// `fail` is a finding, so the run exits 1. A run that writes no record is a
/// failure (exit 2).
pub(crate) fn record(
    root: &Path,
    args: &RecordArgs,
    source: VerdictSource,
) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    let mut options = RecordOptions::new();
    if let Some(copyright) = &args.copyright {
        options = options.copyright(copyright.as_str());
    }
    if let Some(licence) = &args.licence {
        options = options.licence(licence.as_str());
    }
    let mut request = EvidenceRequest::new(
        args.id.as_str(),
        args.covers.clone(),
        args.by.actor(),
        source,
    )
    .limits(args.limit.clone())
    .options(options);
    if let Some(revision) = &args.revision {
        request = request.revision(revision.as_str());
    }
    if args.update {
        request = request.mode(WriteMode::Replace);
    }
    let runner = SystemRunner::new(Programs::new(root));
    let recorded = plan.record_evidence(root, &runner, &request)?;
    Ok(Report::of([recorded_record(&recorded)]))
}

/// A measure as a JSON object; the number is its decimal text, never a float.
fn measure(measure: &Measure) -> Value {
    json!({ "name": measure.name, "value": measure.value.text() })
}

/// Why a verdict is `fail`, as a JSON object.
fn failed(reason: &Failed) -> Value {
    let (name, code, names): (&str, Option<i32>, &[String]) = match reason {
        Failed::Exited(code) => ("exited", *code, &[]),
        Failed::TimedOut => ("timed-out", None, &[]),
        Failed::OverLimit(names) => ("over-limit", None, names),
        Failed::Binding(_) => ("binding", None, &[]),
        _ => ("other", None, &[]),
    };
    json!({ "reason": name, "code": code, "names": names })
}

/// The record of a written evidence record.
fn recorded_record(recorded: &Recorded) -> Record {
    let record = &recorded.record;
    let inputs: Vec<Value> = record
        .inputs
        .iter()
        .map(|input| json!({ "kind": input.kind.name(), "name": input.name, "digest": input.digest }))
        .collect();
    let fields = vec![
        ("path", Value::from(recorded.path.as_str())),
        ("id", Value::from(record.header.id.as_str())),
        ("verdict", Value::from(record.verdict.name())),
        ("subject", Value::from(record.subject.text())),
        (
            "command",
            record.command.as_deref().map_or(Value::Null, Value::from),
        ),
        (
            "revision",
            record.revision.as_deref().map_or(Value::Null, Value::from),
        ),
        ("by", Value::from(record.by.name())),
        ("for", json!(record.covers)),
        ("values", record.values.iter().map(measure).collect()),
        ("limits", record.limits.iter().map(measure).collect()),
        ("inputs", Value::Array(inputs)),
        ("failed", recorded.failed.iter().map(failed).collect()),
        ("message", Value::from(recorded.to_string())),
    ];
    if recorded.failed.is_empty() {
        Record::outcome("evidence-recorded", fields, recorded.to_string())
    } else {
        Record::finding("evidence-recorded", fields, recorded.to_string())
    }
}

/// `evidence claims <id>`: each statement that rests on the record, by
/// capability and line. Writes nothing and starts no process beyond the `git`
/// that lists the tracked files; always exit 0 when it has an answer.
pub(crate) fn claims(root: &Path, id: &str) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    let claims = plan.evidence_claims(id)?;
    let evidence = evidence_id(id);
    let mut report = Report::of(claims.iter().map(|claim| claim_record(&evidence, claim)));
    if claims.is_empty() {
        report.note(format!("no claim of a capability names {evidence}"));
    }
    Ok(report)
}

/// The evidence id `evidence:<slug>` of an id given with or without its
/// prefix.
fn evidence_id(id: &str) -> String {
    let prefix = DocType::Evidence.id_prefix();
    if id.starts_with(prefix) {
        id.to_string()
    } else {
        format!("{prefix}{id}")
    }
}

/// The record of one claim: `<path>:<line>: <capability>#<section>` and the
/// statement below it, each line indented by four spaces, with each control
/// and bidirectional formatting character but the line feed and the tab as
/// its escape
/// (`raf_workplan::escape::shown_lines`); the JSON form escapes as JSON does.
fn claim_record(evidence: &str, claim: &ClaimStatement) -> Record {
    let section = claim.section.map(CapabilitySection::slug);
    let at = section.map_or_else(
        || claim.capability.clone(),
        |slug| format!("{}#{slug}", claim.capability),
    );
    let mut text = format!("{}:{}: {at}", claim.path, claim.line);
    for line in claim.statement.lines() {
        text.push_str("\n    ");
        text.push_str(line);
    }
    // The statement is document text: no control of it reaches the terminal.
    let text = shown_lines(&text);
    Record::outcome(
        "evidence-claim",
        vec![
            ("evidence", Value::from(evidence)),
            ("capability", Value::from(claim.capability.as_str())),
            ("path", Value::from(claim.path.as_str())),
            ("section", section.map_or(Value::Null, Value::from)),
            ("line", Value::from(claim.line)),
            ("statement", Value::from(claim.statement.as_str())),
        ],
        text,
    )
}

/// `evidence for <cap:slug>`: each record that supports the capability.
/// Writes nothing; always exit 0 when it has an answer.
pub(crate) fn support(root: &Path, capability: &str) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    let found = plan.evidence_for(capability)?;
    let mut report = Report::of(found.iter().map(|entry| support_record(capability, entry)));
    if found.is_empty() {
        report.note(format!("no evidence record supports {capability}"));
    }
    Ok(report)
}

/// The record of one supporting record:
/// `<id>  <verdict>  <path>  (<listed>; <claims>)`.
fn support_record(capability: &str, entry: &Support) -> Record {
    let verdict = entry.verdict.map(EvidenceVerdict::name);
    let listed = if entry.listed {
        "in its for list"
    } else {
        "not in its for list"
    };
    let lines: Vec<String> = entry.claims.iter().map(ToString::to_string).collect();
    let claims = if lines.is_empty() {
        "no claim".to_string()
    } else {
        format!("claims at lines {}", lines.join(", "))
    };
    Record::outcome(
        "evidence-for",
        vec![
            ("capability", Value::from(capability)),
            ("evidence", Value::from(entry.evidence.as_str())),
            ("path", Value::from(entry.path.as_str())),
            ("verdict", verdict.map_or(Value::Null, Value::from)),
            ("listed", Value::from(entry.listed)),
            ("claims", json!(entry.claims)),
        ],
        format!(
            "{}  {}  {}  ({listed}; {claims})",
            entry.evidence,
            verdict.unwrap_or("not loaded"),
            entry.path
        ),
    )
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};

    use super::{agree, line};

    fn os(words: &[&str]) -> Vec<OsString> {
        words.iter().map(OsString::from).collect()
    }

    #[test]
    fn a_program_named_the_separator_and_a_binding_beside_one_are_refused() {
        let program = line(os(&["--", "evidence", "record", "--", "true"]), false)
            .expect_err("a program named `--`");
        assert!(program.to_string().contains("program name"), "{program}");
        let both = line(
            os(&["cargo-workplan", "--from-binding", "a::t", "--"]),
            true,
        )
        .expect_err("a binding beside `--`");
        assert!(both.to_string().contains("exclude each other"), "{both}");
        // The neighbours: a command after `--`, and a binding with none.
        assert!(line(os(&["cargo-workplan", "--", "true"]), false).is_ok());
        assert!(line(os(&["cargo-workplan", "--from-binding", "a::t"]), true).is_ok());
    }

    #[test]
    fn the_command_runs_only_when_both_readings_of_the_line_agree() {
        let words = [OsStr::new("sh"), OsStr::new("-c"), OsStr::new("echo -- x")];
        let parsed: Vec<OsString> = words.iter().map(|w| w.to_os_string()).collect();
        assert!(agree(&words, &parsed).is_ok());
        // A reading that starts one word later, as when an option took the
        // first `--`, is refused, and so is one that holds a word more.
        assert!(agree(&words, &parsed[1..]).is_err());
        let mut longer = parsed.clone();
        longer.push(OsString::from("--"));
        assert!(agree(&words, &longer).is_err());
        assert!(agree(&[], &[]).is_ok());
    }
}
