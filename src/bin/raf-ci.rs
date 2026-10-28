// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Execute a package's documented standalone examples through the native verifier.

use std::error::Error;

use raf_workplan::bound::{BoundOutcome, Executor, Plan};
use raf_workplan::exec::{Programs, SystemRunner};
use raf_workplan::{BindingKind, Gate};

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args().skip(1);
    let package = arguments.next().ok_or("usage: raf-ci <package>")?;
    if arguments.next().is_some() {
        return Err("usage: raf-ci <package>".into());
    }
    run_examples(&package)
}

fn run_examples(package: &str) -> Result<(), Box<dyn Error>> {
    let root = rust_agentic_foundations::gate_root();
    let opened = Gate::workspace(&root).open()?;
    let source = opened.workspace();
    if !source.packages().iter().any(|item| item.name == package) {
        return Err(format!("unknown workspace package `{package}`").into());
    }
    let workplan = opened.workplan();
    let findings = workplan.check();
    if !findings.is_empty() {
        return Err(findings
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            .into());
    }
    let runner = SystemRunner::new(Programs::new(&root).discover_git());
    let executor = Executor::new(source, runner)?;
    let mut executed = Vec::new();
    let mut failed = false;
    for binding in workplan.graph().bindings() {
        if binding.kind() != BindingKind::Example {
            continue;
        }
        let plan = match executor.plan(binding)? {
            Plan::Example(plan) => plan,
            // Package tests already execute all documentation tests.
            Plan::DocTest(_) => continue,
            other => return Err(format!("unexpected standalone example plan: {other}").into()),
        };
        if plan.package().as_str() != package || executed.contains(&plan) {
            continue;
        }
        println!("running example {} in {package}", plan.example().as_str());
        let evidence = executor.run(binding)?;
        println!("{}: {:?}", evidence.path(), evidence.outcome());
        match evidence.outcome() {
            BoundOutcome::Passed => executed.push(plan),
            BoundOutcome::NotApplicable { .. } => {}
            _ => failed = true,
        }
    }
    println!(
        "verified {} standalone examples for {package}",
        executed.len()
    );
    if failed {
        return Err("a standalone example failed native verification".into());
    }
    Ok(())
}
