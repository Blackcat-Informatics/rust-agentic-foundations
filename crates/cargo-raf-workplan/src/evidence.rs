// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Authored evidence metadata queries; no execution authority.

use crate::{
    exit::Failure,
    render::{Format, Record, Report},
    result::Result as CommandResult,
};
use raf_okf::{DocType, EvidenceVerdict};
use raf_workplan::{Gate, evidence::Support};
use std::path::Path;

/// `evidence claims <id>`: each statement that rests on the record, by
/// capability and line. Writes nothing and starts no process; always exit 0 when it has an answer.
pub(crate) fn claims(root: &Path, id: &str, format: Format) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    if format != Format::Text {
        return Ok(Report::claims_dataset(plan.evidence_claims_dataset(id)?));
    }
    let claims = plan.evidence_claims(id)?;
    let evidence = evidence_id(id);
    let empty = claims.is_empty();
    let mut report = Report::claim_statements(claims);
    if empty {
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
        CommandResult::EvidenceSupport {
            capability: capability.into(),
            support: entry.clone(),
        },
        format!(
            "{}  {}  {}  ({listed}; {claims})",
            entry.evidence,
            verdict.unwrap_or("not loaded"),
            entry.path
        ),
    )
}
