// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Read the repository's Census verdict or complete index through its own policy.

#[path = "support/census.rs"]
mod policy;

use std::{error::Error, io::Write as _, process::ExitCode};

use raf_census::{RegisteredCheck, RegisteredOperation};

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let operation = RegisteredOperation::from_args(std::env::args_os().skip(1))?;
    let gate = policy::gate(rust_agentic_foundations::gate_root());
    let (text, code) = match operation {
        RegisteredOperation::Check => {
            let response = RegisteredCheck::from_check(&gate.check())?;
            (response.to_ntriples()?, response.exit_code())
        }
        RegisteredOperation::Index => {
            let report = gate.check()?;
            (report.census().index(report.jobs())?.to_ntriples()?, 0)
        }
        RegisteredOperation::WriteDebt => {
            return Err(
                "repository_census is read-only and does not support debt publication".into(),
            );
        }
    };
    std::io::stdout().lock().write_all(text.as_bytes())?;
    Ok(ExitCode::from(code))
}
