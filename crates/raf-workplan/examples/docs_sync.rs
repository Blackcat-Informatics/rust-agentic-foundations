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
//! The documents are read through a disk tree rooted at `ROOT`, and the source
//! index is built from the workspace at `ROOT`. Links are judged against the
//! tracked paths from `git ls-files` when `ROOT` holds a `.git` entry. When it
//! does not, every file of the tree stands for a tracked path; that only
//! changes the link findings of `check`, which sync does not read.

use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use raf_source::{DiskTree, RelPath, Workspace};
use raf_workplan::{LinkContext, SyncError, SyncOptions, Workplan};

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

/// The tracked paths of the checkout at `root`, as `git ls-files` lists them.
fn tracked(root: &Path) -> Result<Vec<RelPath>, Box<dyn Error>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|error| format!("cannot run `git ls-files` in {}: {error}", root.display()))?;
    if !output.status.success() {
        return Err(format!(
            "`git ls-files` failed in {}: {}",
            root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    let mut paths = Vec::new();
    for name in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
    {
        let text = std::str::from_utf8(name)
            .map_err(|error| format!("a tracked path is not UTF-8: {error}"))?;
        paths.push(RelPath::new(text).map_err(|error| format!("`{text}` is not a path: {error}"))?);
    }
    Ok(paths)
}

fn run() -> Result<ExitCode, Box<dyn Error>> {
    let args = parse(std::env::args().skip(1))?;
    let tree = DiskTree::new(&args.root)?;
    let source = Workspace::load(&tree, &["."])?;
    let context = if args.root.join(".git").exists() {
        LinkContext::new(&tree, tracked(&args.root)?)
    } else {
        LinkContext::of_tree(&tree)
    };
    let plan = Workplan::load_with(&tree, &source, &context);
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
