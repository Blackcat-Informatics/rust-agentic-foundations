// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Static receipt creation and admission at the caller's build boundary.

use crate::cli::LintArgs;
use raf_census::{Error, Gate, MAX_LEDGER_BYTES, ProductionInputs, ProductionLintRun};
use raf_source::DiskTree;
use std::path::Path;

mod capture;
pub(crate) use capture::capture;

pub(crate) fn inputs(gate: &Gate) -> Result<ProductionInputs, Error> {
    inputs_at(&gate.root()?)
}

fn inputs_at(root: &Path) -> Result<ProductionInputs, Error> {
    let tree = DiskTree::new(root).map_err(|source| Error::Tree {
        file: root.display().to_string(),
        source,
    })?;
    ProductionInputs::read(&tree)
}

pub(crate) fn receipt(
    gate: &Gate,
    before: &str,
    compiler: &str,
    status: u8,
    diagnostics: &Path,
    arguments: &[String],
) -> Result<String, Error> {
    let arguments: Vec<_> = arguments.iter().map(String::as_str).collect();
    let diagnostics = crate::command::read(diagnostics, MAX_LEDGER_BYTES)?;
    ProductionLintRun::capture(
        before,
        &inputs(gate)?,
        &arguments,
        compiler.to_owned(),
        status,
        diagnostics,
    )?
    .to_turtle()
}

pub(crate) fn configure(gate: Gate, args: &LintArgs) -> Result<Gate, Error> {
    if args.no_production_lints {
        return Ok(gate.without_production_lints());
    }
    let build_receipt = std::env::var_os("RAF_CENSUS_LINT_EVIDENCE");
    let Some(path) = args
        .lint_evidence
        .as_deref()
        .or_else(|| build_receipt.as_deref().map(Path::new))
    else {
        return Ok(gate);
    };
    let run = ProductionLintRun::read_external(&gate.root()?, path)?;
    Ok(gate.with_production_lint_run(run))
}
