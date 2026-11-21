// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Read the repository's Census verdict or complete index, or record its debt, through its own locked policy.

#[path = "support/census.rs"]
mod policy;

use std::{error::Error, io::Write as _, process::ExitCode};

use raf_census::{Error as CensusError, RegisteredCheck, RegisteredOperation};
use raf_source::DiskTree;

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let operation = RegisteredOperation::from_args(std::env::args_os().skip(1))?;
    let root = rust_agentic_foundations::gate_root();
    let gate = if matches!(operation, RegisteredOperation::Check) {
        policy::checked_gate(&root)?
    } else {
        policy::gate(&root).without_production_lints()
    };
    let tree = DiskTree::new(&root)?;
    let locked = || {
        policy::locked_report(&gate, &tree).map_err(|error| CensusError::Options(error.to_string()))
    };
    let (text, code) = match operation {
        RegisteredOperation::Check => {
            let response = RegisteredCheck::from_check(&locked())?;
            (response.to_ntriples()?, response.exit_code())
        }
        RegisteredOperation::Index => {
            let report = locked()?;
            (report.census().index(report.jobs())?.to_ntriples()?, 0)
        }
        RegisteredOperation::WriteDebt => {
            let written = gate.write_debt_registry(&policy::registry(&tree)?)?;
            (format!("{written}\n"), 0)
        }
    };
    std::io::stdout().lock().write_all(text.as_bytes())?;
    Ok(ExitCode::from(code))
}
