// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Cargo entry point for the Census command line.

use raf_census::Gate;
use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    cargo_census::run(
        std::env::args_os(),
        None::<fn(Gate) -> Gate>,
        &mut io::stdout(),
        &mut io::stderr(),
    )
}
