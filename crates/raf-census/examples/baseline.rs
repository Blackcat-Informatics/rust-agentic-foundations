// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Prints the baseline and the index report of a workspace.
//!
//! With no argument it reports a built-in workspace with one body group, one
//! shim group and one job. With a directory argument it reports the workspace
//! under that directory, with no jobs. The output is byte-stable: running it
//! twice prints the same bytes.

use std::path::Path;
use std::process::ExitCode;

use raf_source::DiskTree;

#[path = "../tests/support/builtin.rs"]
mod builtin;

fn main() -> ExitCode {
    let output = match std::env::args().nth(1) {
        None => builtin::builtin(),
        Some(dir) => {
            // An absolute path is its own tree; a relative one is a root of
            // the tree at the current directory.
            let (base, root) = if Path::new(&dir).is_absolute() {
                (dir.as_str(), ".")
            } else {
                (".", dir.as_str())
            };
            match DiskTree::new(base) {
                Ok(tree) => builtin::of(&tree, root),
                Err(error) => {
                    eprintln!("baseline: {dir}: {error}");
                    return ExitCode::FAILURE;
                }
            }
        }
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
