// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Count one resident workspace with 64 sibling modules and 64 functions each.
//!
//! `parallel_cost collect <new-output-directory>` runs isolated serial and
//! two-worker children through native inherited instruction counters. Fixture
//! construction precedes both windows. The allocator window covers only loading,
//! through parser destruction, TLS cleanup and complete joining, with the index
//! retained at close. The instruction window additionally includes allocator
//! window bookkeeping and perf enable/disable protocol boundary overhead. Report
//! validation and index destruction follow both windows. Counts are observations,
//! without a ratio gate or a speed claim.

#[path = "support/workspace_report.rs"]
mod workspace_report;

use std::ffi::OsString;
use std::fmt::Write as _;
use std::path::Path;
use std::process::ExitCode;

use purrdf_alloc_probe::{CountingAllocator, WholeProcessWindow};
use purrdf_testkit::bench::counter::{CounterCommand, CounterControl, collect_instructions};
use raf_source::{LoadOptions, MemoryTree, ParallelOptions, Workspace};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn fixture() -> MemoryTree {
    let mut files = vec![(
        "Cargo.toml".to_owned(),
        "[package]\nname='wide'\nversion='0.1.0'\nedition='2021'\n".to_owned(),
    )];
    let mut root = String::new();
    for module in 0..64 {
        writeln!(root, "pub mod m{module:02};").expect("write");
        let mut text = String::new();
        for function in 0..64 {
            writeln!(text, "pub fn f{function:02}() -> usize {{ {function} }}").expect("write");
        }
        files.push((format!("src/m{module:02}.rs"), text));
    }
    files.push(("src/lib.rs".to_owned(), root));
    files.into_iter().collect()
}

fn measure(mode: &str) -> Result<(), String> {
    if !matches!(mode, "serial" | "parallel") {
        return Err("work mode must be serial or parallel".to_owned());
    }
    let mut control = CounterControl::required().map_err(|error| error.to_string())?;
    let tree = fixture();
    let options = LoadOptions::default();
    let parallel = ParallelOptions::default();
    control
        .command(CounterCommand::Enable)
        .map_err(|error| error.to_string())?;
    let window = WholeProcessWindow::open();
    let loaded = if mode == "serial" {
        Workspace::load_with(&tree, &["."], &options)
    } else {
        Workspace::par_load(&tree, &["."], &options, &parallel)
    };
    let allocations = window.close();
    let disabled = control.command(CounterCommand::Disable);
    let workspace = loaded.map_err(|error| error.to_string())?;
    disabled.map_err(|error| error.to_string())?;
    println!("answer {}", workspace_report::report(&workspace));
    println!(
        "allocations={} requested_bytes={} retained_bytes={} peak_working_bytes={}",
        allocations.allocations,
        allocations.requested_bytes,
        allocations.retained_bytes,
        allocations.peak_working_bytes
    );
    println!(
        "parallel_usage={:?} semantic_usage={:?}",
        workspace.parallel_usage(),
        workspace.usage()
    );
    Ok(())
}

fn answer(output: &str) -> Result<&str, String> {
    let mut answers = output.lines().filter(|line| line.starts_with("answer "));
    let answer = answers
        .next()
        .ok_or_else(|| "missing workspace answer".to_owned())?;
    if answers.next().is_some() {
        return Err("duplicate workspace answers".to_owned());
    }
    Ok(answer)
}

fn collect(directory: &Path) -> Result<(), String> {
    std::fs::create_dir(directory).map_err(|error| format!("{}: {error}", directory.display()))?;
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut outputs = Vec::new();
    for mode in ["serial", "parallel"] {
        let record = directory.join(mode);
        let sample = collect_instructions(
            &executable,
            &[OsString::from("work"), OsString::from(mode)],
            &record,
        )?;
        let output = std::fs::read_to_string(record.join("workload.stdout"))
            .map_err(|error| error.to_string())?;
        answer(&output)?;
        println!(
            "{mode} instructions={} enabled={} running={} coverage={}",
            sample.instructions, sample.enabled, sample.running, sample.displayed_running_percent
        );
        print!("{output}");
        outputs.push(output);
    }
    if answer(&outputs[0])?.as_bytes() != answer(&outputs[1])?.as_bytes() {
        return Err("serial and parallel workspace report bytes differ".to_owned());
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [command, mode] if command == "work" => measure(mode),
        [command, directory] if command == "collect" => collect(Path::new(directory)),
        _ => Err("usage: parallel_cost collect <new-output-directory>".to_owned()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("parallel_cost: {error}");
            ExitCode::FAILURE
        }
    }
}
