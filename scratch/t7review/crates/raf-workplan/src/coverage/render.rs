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
//!
//! [`claims_table`] and [`claims_json`] write the claims report of strict mode
//! ([`ClaimsReport`]) the same two ways: `| Capability | Section | Claim | Line
//! | Tested by |`, one row per claim with the slug of its section and its
//! tests as code spans, and one `{"kind":"coverage-claim", ...}` object per
//! claim with its `capability`, `section`, `claim`, `line` and `tests`.
//!
//! [`lines_table`] and [`lines_json`] write the reverse coverage of strict mode
//! ([`LinesReport`]) the same two ways. The text is three tables, each after a
//! blank line from the one before: `| Crate | Enforced | Counted | Covered |
//! Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |`, one row
//! per package; `| Crate | Module | Counted | Orphan |`, one row per module that
//! holds an orphan line; and `| Crate | Item | Kind | File | Orphan lines |`,
//! one row per unit with orphan lines, as merged `a-b` ranges. When there are
//! any, the lines the exceptions take out follow (`| Crate | Item | Kind | File
//! | Excepted lines | Exception | Reason |`), and the reasons the lines could not
//! be counted (`| Error |`). The JSON holds one object per row, in that order:
//! `lines-crate` (`crate`, `enforced`, `counted`, `covered`, `excepted`,
//! `orphan`, `shipping_orphan`, `test_orphan`, `ceiling`, `debt`),
//! `lines-module` (`crate`, `module`, `counted`, `orphan`), `lines-orphan`
//! (`crate`, `module`, `item`, `unit`, `file`, `ranges` as `[[a,b],...]`),
//! `lines-excepted` (those fields, then `exception` and `reason`) and
//! `lines-error` (`message`).

use super::lines::{CrateLines, ExceptedLines, LinesReport, ModuleLines, UnitLines};
use super::{CoverageReport, Excluded, Gap, GapKind, SurfaceError, SurfaceFailure};
use crate::history::render::{string, table_cell, table_code};
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
/// assert_eq!(render::claims_json(&ClaimsReport::default()), "[]\n");
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

/// The claims report as one JSON array, one object to a line; `[]` when it
/// holds nothing.
#[must_use]
pub fn claims_json(report: &ClaimsReport) -> String {
    let objects: Vec<String> = report
        .claims()
        .iter()
        .map(|claim| {
            let tests: Vec<String> = claim.tests().iter().map(|path| string(path)).collect();
            format!(
                "{{\"kind\":\"coverage-claim\",\"capability\":{},\"section\":{},\"claim\":{},\"line\":{},\"tests\":[{}]}}",
                string(claim.capability()),
                string(claim.section().slug()),
                string(claim.claim()),
                claim.line(),
                tests.join(",")
            )
        })
        .collect();
    if objects.is_empty() {
        "[]\n".to_string()
    } else {
        format!("[\n{}\n]\n", objects.join(",\n"))
    }
}

fn crate_row(row: &CrateLines) -> String {
    format!(
        "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
        table_cell(row.package()),
        if row.enforced() { "yes" } else { "no" },
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
///     "| Crate | Enforced | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |\n\
///      | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n\n\
///      | Crate | Module | Counted | Orphan |\n| --- | --- | --- | --- |\n\n\
///      | Crate | Item | Kind | File | Orphan lines |\n| --- | --- | --- | --- | --- |\n"
/// );
/// assert_eq!(render::lines_json(&LinesReport::default()), "[]\n");
/// ```
#[must_use]
pub fn lines_table(report: &LinesReport) -> String {
    let mut out = String::from(
        "| Crate | Enforced | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n",
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

fn unit_fields(unit: &UnitLines) -> String {
    let ranges: Vec<String> = unit
        .ranges()
        .iter()
        .map(|(first, last)| format!("[{first},{last}]"))
        .collect();
    format!(
        "\"crate\":{},\"module\":{},\"item\":{},\"unit\":{},\"file\":{},\"ranges\":[{}]",
        string(unit.package()),
        string(unit.module()),
        string(unit.item()),
        string(unit.kind()),
        string(unit.file()),
        ranges.join(",")
    )
}

/// The reverse coverage as one JSON array, one object to a line; `[]` when it
/// holds nothing.
#[must_use]
pub fn lines_json(report: &LinesReport) -> String {
    let crates = report.crates().iter().map(|row| {
        format!(
            "{{\"kind\":\"lines-crate\",\"crate\":{},\"enforced\":{},\"counted\":{},\"covered\":{},\"excepted\":{},\"orphan\":{},\"shipping_orphan\":{},\"test_orphan\":{},\"ceiling\":{},\"debt\":{}}}",
            string(row.package()),
            row.enforced(),
            row.counted(),
            row.covered(),
            row.excepted(),
            row.orphan(),
            row.shipping_orphan(),
            row.test_orphan(),
            string(&row.ceiling_text()),
            row.debt()
        )
    });
    let modules = report.modules().iter().map(|row| {
        format!(
            "{{\"kind\":\"lines-module\",\"crate\":{},\"module\":{},\"counted\":{},\"orphan\":{}}}",
            string(row.package()),
            string(row.module()),
            row.counted(),
            row.orphan()
        )
    });
    let orphans = report
        .orphans()
        .iter()
        .map(|unit| format!("{{\"kind\":\"lines-orphan\",{}}}", unit_fields(unit)));
    let excepted = report.excepted().iter().map(|row| {
        format!(
            "{{\"kind\":\"lines-excepted\",{},\"exception\":{},\"reason\":{}}}",
            unit_fields(row.lines()),
            string(row.exception()),
            string(row.reason())
        )
    });
    let errors = report
        .errors()
        .iter()
        .map(|error| format!("{{\"kind\":\"lines-error\",\"message\":{}}}", string(error)));
    let objects: Vec<String> = crates
        .chain(modules)
        .chain(orphans)
        .chain(excepted)
        .chain(errors)
        .collect();
    if objects.is_empty() {
        "[]\n".to_string()
    } else {
        format!("[\n{}\n]\n", objects.join(",\n"))
    }
}
