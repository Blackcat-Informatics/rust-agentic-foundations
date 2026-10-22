// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Rewrites the generated blocks of a repository, or checks that none is stale.
//!
//! ```text
//! docs_sync [ROOT] [--check] [--copyright <text>] [--licence <id>]
//! ```
//!
//! * `ROOT` is the repository root and defaults to `.`. It is the only
//!   argument with a default.
//! * `--copyright` and `--licence` are the SPDX text of the first line pair of
//!   `docs/capabilities/index.md`. They are read only when sync has to create
//!   the index; then each is required, and the error names the missing flag.
//!   The header of a new index is never guessed.
//! * Without `--check` the example syncs, writes each file that changed and
//!   prints `rewrote <path>` for it.
//! * With `--check` it writes nothing and prints `stale <path>` for each file a
//!   sync would write.
//!
//! Exit codes: 0 when the sync succeeded and, under `--check`, nothing is
//! stale; 1 under `--check` when a file is stale; 2 for any failure (a bad
//! argument, a document that does not load, a file sync refuses, a write that
//! fails), with a message on standard error.
//!
//! The repository is opened with `Gate::open`, the load the gate runs: the
//! documents are read through a disk tree rooted at `ROOT`, and the source index
//! is built from the workspace at `ROOT`. Links are judged against the tracked
//! paths from `git ls-files` when `ROOT` is inside a git work tree. When `git`
//! answers that it is not, every file of the tree stands for a tracked path; that
//! only changes the link findings of `check`, which sync does not read. A `ROOT`
//! with no `docs/capabilities` directory, or none that holds a capability
//! document, is exit 2 with the gate's message.

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use raf_workplan::{Gate, SyncError, SyncOptions, Workplan};

/// The command line.
struct Args {
    root: PathBuf,
    check: bool,
    options: SyncOptions,
}

const USAGE: &str = "usage: docs_sync [ROOT] [--check] [--copyright <text>] [--licence <id>]";

fn parse(mut args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut root = None;
    let mut check = false;
    let mut copyright = None;
    let mut licence = None;
    while let Some(arg) = args.next() {
        let mut value = |name: &str| {
            args.next()
                .ok_or_else(|| format!("{name} needs a value; {USAGE}"))
        };
        match arg.as_str() {
            "--check" => check = true,
            "--copyright" => copyright = Some(value("--copyright")?),
            "--licence" => licence = Some(value("--licence")?),
            option if option.starts_with("--") => {
                return Err(format!("unknown option {option}; {USAGE}"));
            }
            _ if root.is_some() => return Err(format!("more than one ROOT; {USAGE}")),
            _ => root = Some(PathBuf::from(arg)),
        }
    }
    Ok(Args {
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        check,
        options: {
            let mut options = SyncOptions::default();
            options.copyright = copyright;
            options.licence = licence;
            options
        },
    })
}

fn run() -> Result<ExitCode, Box<dyn Error>> {
    let args = parse(std::env::args().skip(1))?;
    let opened = Gate::workspace(&args.root).open()?;
    let plan = opened.workplan();
    let report = plan.sync(&args.options).map_err(|error| match error {
        SyncError::SpdxUnset { field } => format!(
            "docs/capabilities/index.md must be created and --{field} is not given; {USAGE}"
        )
        .into(),
        other => Box::new(other) as Box<dyn Error>,
    })?;
    if args.check {
        for (path, _) in &report.edits {
            println!("stale {path}");
        }
        return Ok(if report.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        });
    }
    Workplan::write(&report, &args.root)?;
    for (path, _) in &report.edits {
        println!("rewrote {path}");
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("docs_sync: {error}");
            ExitCode::from(2)
        }
    }
}
