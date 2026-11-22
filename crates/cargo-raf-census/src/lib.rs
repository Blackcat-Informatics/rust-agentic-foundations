// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Command-line Census hosted by Cargo or a repository-owned executable.

mod cli;
mod command;
mod embedded;
mod error;
mod lints;

use raf_census::Gate;
use std::{ffi::OsString, io::Write, process::ExitCode};

/// Run the Census command line and return its process exit status.
///
/// Pass the executable name as the first argument. Both direct and Cargo
/// subcommand invocations are accepted. Reports go to `output`, and refusals
/// go to `diagnostics`. Explicit report destinations use the selected command's
/// normal file-writing rules.
///
/// A repository binary supplies `Some(configure)` to install executable rules
/// and ledger extension validators on the Gate for the selected workspace.
/// Configuration runs only after argument validation. In this mode,
/// `baseline` validates registrations before rendering the default Census
/// baseline. The Cap'n Proto persistence, reader admission and reader evidence commands use
/// pinned Git independently of Gate registration. Use `None::<fn(Gate) -> Gate>`
/// for the stock command line.
pub fn run(
    args: impl IntoIterator<Item = OsString>,
    configure: Option<impl FnOnce(Gate) -> Gate>,
    output: &mut dyn Write,
    diagnostics: &mut dyn Write,
) -> ExitCode {
    let args = match cli::parse(args) {
        Ok(args) => args,
        Err(error) => {
            let status = u8::try_from(error.exit_code()).unwrap_or(2);
            let printed = if error.use_stderr() {
                write!(diagnostics, "{error}")
            } else {
                write!(output, "{error}")
            };
            return if printed.is_ok() {
                ExitCode::from(status)
            } else {
                ExitCode::FAILURE
            };
        }
    };
    match command::run(&args, configure, output, diagnostics) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            let _ = writeln!(diagnostics, "cargo-raf-census: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}
