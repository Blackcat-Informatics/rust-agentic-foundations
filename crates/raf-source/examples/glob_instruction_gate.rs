// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Compare paired release shape drivers using retired user instructions.
//!
//! Both drivers must use the same shape generator, caller and compiler. Run
//! `glob_instruction_gate <baseline-driver> <candidate-driver> <new-output-dir>`.
//! The output directory retains every counter record, answer and diagnostic.
//! A missing measurement, failed child, reported partial coverage, excessive growth
//! or excessive candidate/baseline ratio fails the gate. No clock is read.
//! Answer differences are reported for separate native semantic qualification;
//! an older resolver can have incorrect answers. Perf rounds its coverage field.
//! With no arguments, the example demonstrates the real counter parser and
//! integer bounds using explicitly unmeasured sample data, without creating files.

#[path = "../tests/support/number.rs"]
mod number;

use std::fs::{self, File};
use std::path::Path;
use std::process::{Command, ExitCode};

const SIZES: [usize; 4] = [100, 200, 400, 800];
const SHAPES: [(&str, u128); 9] = [
    ("chain", 215),
    ("many-items", 215),
    ("pub-fan-in", 215),
    ("private-fan-in", 215),
    ("companion-fan-in", 215),
    ("prelude", 340),
    ("hub", 430),
    ("ambiguous-hub", 430),
    ("lookups", 440),
];

pub(crate) fn counter(text: &str) -> Result<u64, String> {
    let mut counts = None;
    for line in text
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
    {
        let fields: Vec<_> = line.split(',').collect();
        if fields.get(2) != Some(&"instructions:u") {
            return Err(format!("unexpected counter record: {line}"));
        }
        if fields.get(4) != Some(&"100.00") {
            return Err(format!("instruction counter lacks full coverage: {line}"));
        }
        let count = number::unsigned::<u64>(fields[0])
            .filter(|count| *count != 0)
            .ok_or_else(|| format!("invalid instruction count: {line}"))?;
        if counts.replace(count).is_some() {
            return Err("duplicate instruction counter".to_owned());
        }
    }
    counts.ok_or_else(|| "missing instruction counter".to_owned())
}

fn measure(driver: &str, label: &str, shape: &str, n: usize, out: &Path) -> Result<u64, String> {
    let stem = out.join(format!("{label}-{shape}-{n}"));
    let csv = stem.with_extension("csv");
    let answers = File::create(stem.with_extension("answers")).map_err(|e| e.to_string())?;
    let errors = File::create(stem.with_extension("stderr")).map_err(|e| e.to_string())?;
    let status = Command::new("perf")
        .env("LC_ALL", "C")
        .args(["stat", "--no-big-num", "-x,", "-e", "instructions:u", "-o"])
        .arg(&csv)
        .arg("--")
        .arg(driver)
        .arg(shape)
        .arg(n.to_string())
        .stdout(answers)
        .stderr(errors)
        .status()
        .map_err(|e| format!("cannot run perf for {label} {shape} {n}: {e}"))?;
    if !status.success() {
        return Err(format!(
            "{label} {shape} {n} failed: {status}; see {}",
            stem.display()
        ));
    }
    counter(&fs::read_to_string(&csv).map_err(|e| format!("{}: {e}", csv.display()))?)
}

pub(crate) fn bounds(baseline: &[u64; 4], candidate: &[u64; 4], growth: u128) -> Vec<String> {
    let mut failures = Vec::new();
    if u128::from(candidate[3]) * 2 > u128::from(baseline[3]) * 3 {
        failures.push("candidate/baseline at 800 exceeds 1.5".to_owned());
    }
    if u128::from(candidate[3]) * 100 > u128::from(candidate[2]) * growth {
        failures.push(format!("growth from 400 to 800 exceeds {growth}/100"));
    }
    failures
}

fn run(args: &[String]) -> Result<(), String> {
    let [baseline, candidate, directory] = args else {
        return Err(
            "usage: glob_instruction_gate <baseline-driver> <candidate-driver> <new-output-dir>"
                .to_owned(),
        );
    };
    let out = Path::new(directory);
    fs::create_dir(out).map_err(|e| format!("cannot create fresh {}: {e}", out.display()))?;
    let mut failures = Vec::new();
    for (shape, growth) in SHAPES {
        let mut before = [0; 4];
        let mut after = [0; 4];
        for (at, n) in SIZES.into_iter().enumerate() {
            before[at] = measure(baseline, "baseline", shape, n, out)?;
            after[at] = measure(candidate, "candidate", shape, n, out)?;
            let old = out.join(format!("baseline-{shape}-{n}.answers"));
            let new = out.join(format!("candidate-{shape}-{n}.answers"));
            if fs::read(&old).map_err(|e| format!("{}: {e}", old.display()))?
                != fs::read(&new).map_err(|e| format!("{}: {e}", new.display()))?
            {
                eprintln!(
                    "{shape} {n}: driver answers differ; inspect retained files for semantic qualification"
                );
            }
            println!("{shape},{n},{},{}", before[at], after[at]);
        }
        failures.extend(
            bounds(&before, &after, growth)
                .into_iter()
                .map(|error| format!("{shape}: {error}")),
        );
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let outcome = if args.is_empty() {
        sample()
    } else {
        run(&args)
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("glob_instruction_gate: {error}");
            ExitCode::FAILURE
        }
    }
}

fn sample() -> Result<(), String> {
    let count = counter("1200,,instructions:u,0,100.00,,\n")?;
    let failures = bounds(&[100, 200, 400, 800], &[150, 300, 600, 1200], 215);
    if !failures.is_empty() {
        return Err(failures.join("\n"));
    }
    println!("sample (unmeasured): counted event {count}; exact ratio boundary 1.5");
    Ok(())
}
