// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `coverage code|tests [--crate <package>]`: one direction of
//! `Workplan::coverage_report`, printed as the table and the JSON array of
//! `raf_workplan::coverage::render`.

use std::path::Path;

use raf_workplan::Gate;
use raf_workplan::coverage::{CoverageReport, GapKind, PackageCoverage, render};

use crate::cli::CoverageArgs;
use crate::exit::Failure;
use crate::render::{Class, Report};

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
    Ok(Report::rendered(
        class,
        render::table(&report, kind),
        render::json(&report, kind),
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
