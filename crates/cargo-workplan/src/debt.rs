// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `debt --write`: `Gate::write_debt`.

use std::path::Path;

use crate::result::Result as CommandResult;
use raf_workplan::Finding;
use raf_workplan::debt::DebtError;
use raf_workplan::record::RecordOptions;

use crate::cli::DebtArgs;
use crate::command::gate;
use crate::exit::Failure;
use crate::render::{Record, Report};

/// `debt --write`: write the debt record and say what was done, or print the
/// findings that stop the write. A write is an outcome and exits 0, `created`,
/// `replaced` or `unchanged`. A debt that grew, or a finding that no row can
/// hold, is a finding of the run and exits 1; nothing is written. A repository
/// that cannot be opened and a record that cannot be written are exit 2.
pub(crate) fn debt(root: &Path, args: &DebtArgs) -> Result<Report, Failure> {
    let mut options = RecordOptions::new();
    if let Some(copyright) = &args.copyright {
        options = options.copyright(copyright.as_str());
    }
    if let Some(licence) = &args.licence {
        options = options.licence(licence.as_str());
    }
    match gate(root).record_options(options).write_debt() {
        Ok(written) => {
            let text = format!(
                "{}: {} ({} finding(s), {} item(s) with orphan lines)",
                written.change().name(),
                written.path(),
                written.findings(),
                written.lines()
            );
            Ok(Report::of([Record::outcome(
                CommandResult::DebtWritten(written),
                text,
            )]))
        }
        Err(DebtError::Growth(findings)) => Ok(refused(
            &findings,
            "the debt would grow, so nothing is written; a debt record only ever shrinks, so pay these findings",
        )),
        Err(DebtError::Findings(findings)) => Ok(refused(
            &findings,
            "findings that no debt record can hold stand, so nothing is written; pay them first",
        )),
        Err(other) => Err(Failure::from(other)),
    }
}

/// The findings that stop the write, with the reason as a note.
fn refused(findings: &[Finding], note: &str) -> Report {
    let mut report = Report::of(findings.iter().map(Record::of_finding));
    report.note(note);
    report
}
