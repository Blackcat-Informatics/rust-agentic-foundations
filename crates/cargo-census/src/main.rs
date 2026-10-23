// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Ledger migration and exact debt recording through the shared Census gate.

mod cli;
mod command;

use std::io::{self, Write as _};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = match cli::parse(std::env::args_os()) {
        Ok(args) => args,
        Err(error) => {
            let status = u8::try_from(error.exit_code()).unwrap_or(2);
            return if error.print().is_ok() {
                ExitCode::from(status)
            } else {
                ExitCode::FAILURE
            };
        }
    };
    match command::run(&args, &mut io::stdout().lock()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            let code = if matches!(error, raf_census::Error::RegistrationRequired { .. }) {
                2
            } else {
                1
            };
            let _ = writeln!(io::stderr().lock(), "cargo-census: {error}");
            ExitCode::from(code)
        }
    }
}
