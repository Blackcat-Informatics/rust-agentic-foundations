// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The two forms of one direction of a [`CoverageReport`]: a table for a
//! reader and JSON for a tool, carrying the same entries in the same order.
//!
//! [`table`] writes `| Gap | Crate | Path | Location |`, one row per gap of the
//! direction (`public item`, `unresolved re-export` or `test`, the package
//! name, the path as a code span, and `file:line`); [`GapKind::PublicItem`] and
//! [`GapKind::Reexport`] both select the code direction, then, when there are any, the items or tests the
//! exclusions take out (`| Excluded | Crate | Path | Location | Exclusion |
//! Reason |`) and, for the code direction, the surfaces that could not be
//! listed (`| Crate | Target | Error |`), each table after a blank line. Cell
//! text is escaped as the history tables escape it: by the rule of the
//! generated tables (`\`, `|`, `[`, `]` and `<`), with each control character
//! and bidirectional formatting character written as its `\u{..}` escape, so a
//! reason can neither break the table nor reach the terminal.
//!
//! [`json`] writes one array, one object to a line: `{"kind":"coverage-gap",
//! ...}` per gap, then `{"kind":"coverage-excluded", ...}` per exclusion hit,
//! then, for the code direction, `{"kind":"coverage-error", ...}` per surface
//! that could not be listed. Each object of a gap names the `gap` kind, the
//! `crate`, whether it is `in_scope`, the `path`, the defining `item`, the
//! `file` and the `line`.

use super::{CoverageReport, Excluded, Gap, GapKind, SurfaceError, SurfaceFailure};
use crate::history::render::{string, table_cell, table_code};

fn location(gap: &Gap) -> String {
    format!("{}:{}", gap.file(), gap.line())
}

fn gap_row(gap: &Gap) -> String {
    format!(
        "| {} | {} | {} | {} |",
        gap.kind(),
        table_cell(gap.package()),
        table_code(gap.path()),
        table_cell(&location(gap))
    )
}

fn excluded_row(excluded: &Excluded) -> String {
    let gap = excluded.gap();
    format!(
        "| {} | {} | {} | {} | {} | {} |",
        gap.kind(),
        table_cell(gap.package()),
        table_code(gap.path()),
        table_cell(&location(gap)),
        table_code(excluded.exclusion()),
        table_cell(excluded.reason())
    )
}

fn error_row(error: &SurfaceError) -> String {
    format!(
        "| {} | {} | {} |",
        table_cell(error.package()),
        table_code(error.target()),
        table_cell(error.message())
    )
}

fn excluded_of(report: &CoverageReport, kind: GapKind) -> impl Iterator<Item = &Excluded> {
    report
        .excluded()
        .iter()
        .filter(move |excluded| excluded.gap().kind().is_code() == kind.is_code())
}

fn errors_of(report: &CoverageReport, kind: GapKind) -> &[SurfaceError] {
    if kind.is_code() { report.errors() } else { &[] }
}

/// One direction of the report as text: the gaps, then the exclusion hits and
/// the surfaces that could not be listed when there are any.
#[must_use]
pub fn table(report: &CoverageReport, kind: GapKind) -> String {
    let mut out = String::from("| Gap | Crate | Path | Location |\n| --- | --- | --- | --- |\n");
    for gap in report.gaps(kind) {
        out.push_str(&gap_row(gap));
        out.push('\n');
    }
    let mut excluded = excluded_of(report, kind).peekable();
    if excluded.peek().is_some() {
        out.push_str(
            "\n| Excluded | Crate | Path | Location | Exclusion | Reason |\n| --- | --- | --- | --- | --- | --- |\n",
        );
        for hit in excluded {
            out.push_str(&excluded_row(hit));
            out.push('\n');
        }
    }
    let errors = errors_of(report, kind);
    if !errors.is_empty() {
        out.push_str("\n| Crate | Target | Error |\n| --- | --- | --- |\n");
        for error in errors {
            out.push_str(&error_row(error));
            out.push('\n');
        }
    }
    out
}

fn in_scope(report: &CoverageReport, package: &str) -> bool {
    report
        .packages()
        .iter()
        .any(|p| p.name() == package && p.in_scope())
}

fn gap_fields(report: &CoverageReport, gap: &Gap) -> String {
    format!(
        "\"gap\":{},\"crate\":{},\"in_scope\":{},\"path\":{},\"item\":{},\"file\":{},\"line\":{}",
        string(gap.kind().name()),
        string(gap.package()),
        in_scope(report, gap.package()),
        string(gap.path()),
        string(gap.item()),
        string(gap.file()),
        gap.line()
    )
}

/// One direction of the report as one JSON array, one object to a line; `[]`
/// when it holds nothing.
#[must_use]
pub fn json(report: &CoverageReport, kind: GapKind) -> String {
    let gaps = report
        .gaps(kind)
        .iter()
        .map(|gap| format!("{{\"kind\":\"coverage-gap\",{}}}", gap_fields(report, gap)));
    let excluded = excluded_of(report, kind).map(|hit| {
        format!(
            "{{\"kind\":\"coverage-excluded\",{},\"exclusion\":{},\"reason\":{}}}",
            gap_fields(report, hit.gap()),
            string(hit.exclusion()),
            string(hit.reason())
        )
    });
    let errors = errors_of(report, kind).iter().map(|error| {
        let path = match error.failure() {
            SurfaceFailure::CarryBudget { path } => string(path),
            _ => "null".to_string(),
        };
        format!(
            "{{\"kind\":\"coverage-error\",\"crate\":{},\"in_scope\":{},\"target\":{},\"root\":{},\"failure\":{},\"reexport\":{path},\"message\":{}}}",
            string(error.package()),
            in_scope(report, error.package()),
            string(error.target()),
            string(error.root()),
            string(error.failure().name()),
            string(error.message())
        )
    });
    let objects: Vec<String> = gaps.chain(excluded).chain(errors).collect();
    if objects.is_empty() {
        "[]\n".to_string()
    } else {
        format!("[\n{}\n]\n", objects.join(",\n"))
    }
}
