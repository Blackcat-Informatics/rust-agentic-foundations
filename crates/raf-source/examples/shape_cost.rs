// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Load one shape of glob imports of a size in memory and resolve a few
//! paths through it, so that the instructions a load of the shape costs can
//! be counted from outside.
//!
//! ```text
//! cargo build --release -p raf-source --example shape_cost
//! perf stat -e instructions:u target/release/examples/shape_cost <shape> <n>
//! ```
//!
//! The shapes are those of `tests/support/shapes/globs.rs`, by name:
//! `chain`, `many-items`, `pub-fan-in`, `private-fan-in`,
//! `companion-fan-in`, `prelude`, `hub`, `ambiguous-hub`, `lookups` and
//! `holder-scan`. The
//! shape is the library of one package in a `MemoryTree`, loaded once with
//! the default options, and the paths the shape names are resolved from the
//! crate root; each answer is printed, one a line. The indexing caller uses
//! `Workspace::load`, `MemoryTree` and `Workspace::resolve`, so it also builds
//! against an older resolver for comparison. Numeric arguments use the shared
//! native adapter. A refused load, or an unknown shape or size, ends the
//! run with a failure. No clock is read.
//! With no arguments, the example loads the four-link chain through that same path.

#[path = "../tests/support/shapes/globs.rs"]
#[allow(dead_code, reason = "the example selects one shape by its name")]
mod shapes;

#[path = "../tests/support/number.rs"]
mod number;

use std::fmt::Write as _;
use std::process::ExitCode;

use raf_source::{MemoryTree, Workspace};

fn run(args: &[String]) -> Result<String, String> {
    let names: Vec<&str> = shapes::ALL.iter().map(|shape| shape.name).collect();
    let usage = format!("usage: shape_cost <{}> <n>", names.join("|"));
    let [name, size] = args else {
        return Err(usage);
    };
    let shape = shapes::named(name).ok_or_else(|| format!("unknown shape {name:?}; {usage}"))?;
    let n: usize = number::unsigned(size).ok_or_else(|| {
        format!("{size:?} is not a size: invalid or overflowing unsigned integer; {usage}")
    })?;
    if n < 2 {
        return Err(format!("the size must be at least 2; {usage}"));
    }
    let lib = (shape.text)(n);
    let tree: MemoryTree = [
        ("Cargo.toml", shapes::MANIFEST),
        ("src/lib.rs", lib.as_str()),
    ]
    .into_iter()
    .collect();
    let ws =
        Workspace::load(&tree, &["."]).map_err(|error| format!("the load is refused: {error}"))?;
    let mut out = String::new();
    for path in (shape.paths)(n) {
        let answer = ws.resolve(&path).map(|found| found.path);
        writeln!(out, "{path} -> {answer:?}").expect("write");
    }
    Ok(out)
}

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        args = vec!["chain".to_owned(), "4".to_owned()];
    }
    match run(&args) {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("shape_cost: {error}");
            ExitCode::FAILURE
        }
    }
}
