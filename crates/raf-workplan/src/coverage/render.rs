// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Human tables of typed item, claim and reverse-line coverage reports.
//! Machine output is projected from the domain values through the RDF profile.

use super::lines::{CrateLines, ExceptedLines, LinesReport, ModuleLines, UnitLines};
use super::{CoverageReport, Excluded, Gap, GapKind, SurfaceError};
use crate::history::render::{table_cell, table_code};
use crate::strict::{ClaimCoverage, ClaimsReport};

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

fn claim_row(claim: &ClaimCoverage) -> String {
    let tests: Vec<String> = claim.tests().iter().map(|path| table_code(path)).collect();
    format!(
        "| {} | {} | {} | {} | {} |",
        table_cell(claim.capability()),
        claim.section().slug(),
        table_cell(claim.claim()),
        claim.line(),
        tests.join(", ")
    )
}

/// The claims report as text: one row per claim, in the order of the report.
///
/// ```rust
/// use raf_workplan::coverage::render;
/// use raf_workplan::strict::ClaimsReport;
///
/// assert_eq!(
///     render::claims_table(&ClaimsReport::default()),
///     "| Capability | Section | Claim | Line | Tested by |\n| --- | --- | --- | --- | --- |\n"
/// );
/// ```
#[must_use]
pub fn claims_table(report: &ClaimsReport) -> String {
    let mut out = String::from(
        "| Capability | Section | Claim | Line | Tested by |\n| --- | --- | --- | --- | --- |\n",
    );
    for claim in report.claims() {
        out.push_str(&claim_row(claim));
        out.push('\n');
    }
    out
}

fn crate_row(row: &CrateLines) -> String {
    format!(
        "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
        table_cell(row.package()),
        row.counted(),
        row.covered(),
        row.excepted(),
        row.orphan(),
        row.shipping_orphan(),
        row.test_orphan(),
        table_cell(&row.ceiling_text()),
        row.debt()
    )
}

fn module_row(row: &ModuleLines) -> String {
    format!(
        "| {} | {} | {} | {} |",
        table_cell(row.package()),
        table_code(row.module()),
        row.counted(),
        row.orphan()
    )
}

fn unit_cells(unit: &UnitLines) -> String {
    format!(
        "| {} | {} | {} | {} | {} |",
        table_cell(unit.package()),
        table_code(unit.item()),
        unit.kind(),
        table_cell(unit.file()),
        unit.ranges_text()
    )
}

fn excepted_row(row: &ExceptedLines) -> String {
    format!(
        "{} {} | {} |",
        unit_cells(row.lines()),
        table_code(row.exception()),
        table_cell(row.reason())
    )
}

/// The reverse coverage as text: the crate, module and item tables, then the
/// excepted lines and the errors when there are any.
///
/// ```rust
/// use raf_workplan::coverage::lines::LinesReport;
/// use raf_workplan::coverage::render;
///
/// assert_eq!(
///     render::lines_table(&LinesReport::default()),
///     "| Crate | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |\n\
///      | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n\n\
///      | Crate | Module | Counted | Orphan |\n| --- | --- | --- | --- |\n\n\
///      | Crate | Item | Kind | File | Orphan lines |\n| --- | --- | --- | --- | --- |\n"
/// );
/// ```
#[must_use]
pub fn lines_table(report: &LinesReport) -> String {
    let mut out = String::from(
        "| Crate | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- |\n",
    );
    for row in report.crates() {
        out.push_str(&crate_row(row));
        out.push('\n');
    }
    out.push_str("\n| Crate | Module | Counted | Orphan |\n| --- | --- | --- | --- |\n");
    for row in report.modules() {
        out.push_str(&module_row(row));
        out.push('\n');
    }
    out.push_str(
        "\n| Crate | Item | Kind | File | Orphan lines |\n| --- | --- | --- | --- | --- |\n",
    );
    for unit in report.orphans() {
        out.push_str(&unit_cells(unit));
        out.push('\n');
    }
    if !report.excepted().is_empty() {
        out.push_str(
            "\n| Crate | Item | Kind | File | Excepted lines | Exception | Reason |\n| --- | --- | --- | --- | --- | --- | --- |\n",
        );
        for row in report.excepted() {
            out.push_str(&excepted_row(row));
            out.push('\n');
        }
    }
    if !report.errors().is_empty() {
        out.push_str("\n| Error |\n| --- |\n");
        for error in report.errors() {
            out.push_str("| ");
            out.push_str(&table_cell(error));
            out.push_str(" |\n");
        }
    }
    out
}
