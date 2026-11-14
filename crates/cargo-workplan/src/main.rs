// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Static Workplan commands and explicit owned documentation operations.
//!
//! Ordinary analysis never starts a process. Verification reports declarations
//! and static findings; completion judges a prospective document overlay.
//! Existing documentation artifacts are imported data. Only the private
//! documentation build accepts invocation-scoped trust for build-time code.

mod assess;
mod change;
mod cli;
mod command;
mod coverage;
mod debt;
mod docs;
mod evidence;
mod exit;
mod history;
mod mdbook;
mod render;
mod result;
mod result_graph;
mod review;
mod trusted_docs;

use std::io::{self, Write};
use std::process::ExitCode;

use exit::{Exit, Failure};

fn main() -> ExitCode {
    let args = match cli::parse(std::env::args_os()) {
        Ok(args) => args,
        Err(error) => {
            // Help and the version go to standard output with code 0, a usage
            // error to standard error with code 2.
            let code = error.exit_code();
            let response = error.print().and_then(|()| {
                if error.use_stderr() {
                    io::stderr().lock().flush()
                } else {
                    io::stdout().lock().flush()
                }
            });
            if let Err(error) = response {
                return ExitCode::from(
                    fail(
                        &Failure::new(format!("cannot write command-line response: {error}")),
                        &mut io::stderr().lock(),
                    )
                    .code(),
                );
            }
            return ExitCode::from(u8::try_from(code).unwrap_or(2));
        }
    };
    ExitCode::from(run(&args, &mut io::stdout().lock(), &mut io::stderr().lock()).code())
}

/// Run the command, write its complete output carrier and return how it ended.
fn run(args: &cli::Args, out: &mut dyn Write, diagnostics: &mut dyn Write) -> Exit {
    let report = match command::run(args) {
        Ok(command::Output::Report(report)) => report,
        Ok(command::Output::Fragment(fragment)) => {
            return match out.write_all(fragment.as_bytes()).and_then(|()| out.flush()).and_then(|()| {
                writeln!(diagnostics, "cargo-workplan: static source index and imported Cargo metadata; compiler configuration, link resolution and package membership are not verified")
            }) {
                Ok(()) => Exit::Clean,
                Err(error) => fail(&Failure::new(format!("cannot write the documentation fragment: {error}")), diagnostics),
            };
        }
        Ok(command::Output::Mdbook(response)) => {
            return match response.write(out) {
                Ok(()) => response.exit(),
                Err(error) => fail(
                    &Failure::new(format!("cannot write the mdBook response: {error}")),
                    diagnostics,
                ),
            };
        }
        Err(failure) => return fail(&failure, diagnostics),
    };
    if let Err(error) = report.print(args.format, args.quiet, out, diagnostics) {
        return fail(
            &Failure::new(format!("cannot write the report: {error}")),
            diagnostics,
        );
    }
    report.exit()
}

/// Write a failure to standard error: exit 2.
fn fail(failure: &Failure, diagnostics: &mut dyn Write) -> Exit {
    // A message that cannot be written changes nothing: the exit is 2 either way.
    let message = raf_workplan::escape::shown_lines(&failure.to_string());
    let _ = writeln!(diagnostics, "cargo-workplan: {message}");
    Exit::Failed
}
