// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Dispatch and report string-producing measurement examples.

use std::process::ExitCode;

pub(crate) fn run(
    name: &str,
    sample: fn() -> Result<String, String>,
    explicit: fn(&[String]) -> Result<String, String>,
) -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let outcome = if args.is_empty() {
        sample()
    } else {
        explicit(&args)
    };
    match outcome {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{name}: {error}");
            ExitCode::FAILURE
        }
    }
}
