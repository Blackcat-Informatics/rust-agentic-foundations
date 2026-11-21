// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `coverage code|tests [--crate <package>]`: one direction of
//! `Workplan::coverage_report`, printed as a human table or a typed RDF graph.
//! `coverage claims [--capability
//! cap:<slug>]`: `Workplan::claims_report`, printed the same way. `coverage
//! lines [--crate <package>]`, also spelled `coverage --reverse`:
//! `Workplan::lines_report`, printed the same way.

use std::path::Path;

use raf_workplan::Gate;
use raf_workplan::coverage::lines::{CrateLines, LinesReport};
use raf_workplan::coverage::{CoverageReport, GapKind, PackageCoverage, render};

use crate::cli::{ClaimsArgs, CoverageArgs};
use crate::exit::Failure;
use crate::render::{Class, Report};
use crate::result::Result as CommandResult;

/// A coverage report of one direction, of the whole workspace or of the
/// package `--crate` names. The report is a finding when it names a gap, or
/// in the code direction a surface that cannot be listed, so the run exits 1;
/// otherwise it is an outcome and exits 0. It starts no process beyond the
/// `git` that lists the tracked files, and writes nothing.
pub(crate) fn coverage(root: &Path, args: &CoverageArgs, kind: GapKind) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    let whole = plan.coverage_report();
    let report = match &args.package {
        None => whole,
        Some(name) => selected(&whole, name)?,
    };
    let errors = kind.is_code() && !report.errors().is_empty();
    let class = if report.gaps(kind).is_empty() && !errors {
        Class::Outcome
    } else {
        Class::Finding
    };
    Ok(Report::block(
        class,
        render::table(&report, kind),
        CommandResult::Coverage {
            report,
            direction: kind,
        },
    ))
}

/// The report of the package `name`, given as cargo names it (`raf-okf`) or
/// as its crate is named (`raf_okf`).
fn selected(whole: &CoverageReport, name: &str) -> Result<CoverageReport, Failure> {
    let found = whole
        .packages()
        .iter()
        .find(|package| package.name() == name || package.name().replace('-', "_") == name);
    found
        .and_then(|package| whole.package(package.name()))
        .ok_or_else(|| {
            let names: Vec<&str> = whole.packages().iter().map(PackageCoverage::name).collect();
            Failure::new(format!(
                "no package of the workspace is named `{name}`; the packages are {}",
                names.join(", ")
            ))
        })
}

/// The claims of every delivered capability, or of the capability
/// `--capability` names, with the tests their markers name. The report is a
/// finding when a claim names no test, so the run exits 1; otherwise it is an
/// outcome and exits 0. A capability that no loaded document has is a run
/// that cannot answer, exit 2. It starts
/// no process beyond the `git` that lists the tracked files, and writes
/// nothing.
pub(crate) fn claims(root: &Path, args: &ClaimsArgs) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    let whole = plan.claims_report();
    let report = match &args.capability {
        None => whole,
        Some(id) => {
            let known = plan
                .capabilities()
                .iter()
                .any(|loaded| loaded.value().header.id == *id);
            if !known {
                let ids: Vec<&str> = plan
                    .capabilities()
                    .iter()
                    .map(|loaded| loaded.value().header.id.as_str())
                    .collect();
                return Err(Failure::new(format!(
                    "no loaded capability is `{id}`; the capabilities are {}",
                    ids.join(", ")
                )));
            }
            whole.capability(id)
        }
    };
    let class = if report.is_complete() {
        Class::Outcome
    } else {
        Class::Finding
    };
    Ok(Report::block(
        class,
        render::claims_table(&report),
        CommandResult::Claims(report),
    ))
}

/// The reverse coverage of every package, or of the package `--crate` names:
/// the lines that no capability explains. The report is a finding when it
/// prints an orphan line or could not count the lines, so the run exits 1;
/// otherwise it is an outcome and exits 0. A package that is not one is a run
/// that cannot answer, exit 2. Every complete check judges the ceilings. It
/// starts no process beyond the `git` that lists the tracked files, and writes
/// nothing.
pub(crate) fn lines(root: &Path, args: &CoverageArgs) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    let whole = plan.lines_report();
    // Lines that could not be counted hold no row of any package, so the
    // whole report, with its error, is what is printed.
    let report = match &args.package {
        Some(name) if whole.errors().is_empty() => selected_lines(&whole, name)?,
        _ => whole,
    };
    let class = if report.is_complete() {
        Class::Outcome
    } else {
        Class::Finding
    };
    Ok(Report::block(
        class,
        render::lines_table(&report),
        CommandResult::Lines(report),
    ))
}

/// The reverse coverage of the package `name`, as cargo names it or as its
/// crate is named.
fn selected_lines(whole: &LinesReport, name: &str) -> Result<LinesReport, Failure> {
    whole
        .crates()
        .iter()
        .find(|row| row.package() == name || row.package().replace('-', "_") == name)
        .and_then(|row| whole.package(row.package()))
        .ok_or_else(|| {
            let names: Vec<&str> = whole.crates().iter().map(CrateLines::package).collect();
            Failure::new(format!(
                "no package of the workspace is named `{name}`; the packages are {}",
                names.join(", ")
            ))
        })
}
