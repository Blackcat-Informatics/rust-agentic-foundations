// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Prints the baseline and the index report of a workspace.
//!
//! With no argument it reports a built-in workspace with one body group, one
//! shim group and one job. With a directory argument it reports the workspace
//! under that directory, with no jobs; an absolute directory and the same
//! directory as a relative path from its parent give the same output. The
//! output is byte-stable: running it twice prints the same bytes.

use std::process::ExitCode;

#[path = "../tests/support/builtin.rs"]
mod builtin;

fn main() -> ExitCode {
    let output = match std::env::args().nth(1) {
        None => builtin::builtin(),
        Some(dir) => builtin::of_dir(&dir),
    };
    match output {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("baseline: {error}");
            ExitCode::FAILURE
        }
    }
}
