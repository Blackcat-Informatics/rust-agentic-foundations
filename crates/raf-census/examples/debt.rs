// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Exercise the real debt writer in an owned scratch workspace.
//!
//! Adoption records present copies, an unchanged write preserves exact bytes,
//! growth and non-debt findings refuse without changing the file, and shrinkage
//! lowers it. A different normal-form record takes the explicit adoption path.
//! All tracked files of the checkout remain untouched.

#[path = "support/locked_debt.rs"]
mod locked_debt;
#[path = "support/registration.rs"]
mod registration;

use std::{collections::BTreeMap, error::Error, fs, path::Path};

use purrdf_testkit::TempDir;
use raf_census::{DEBT_FILE, DebtRecord, Gate, LEDGER_FILE, NORMAL_FORM, REPORT_FORM};
use raf_rdf::{Builder, Format, TermValue as Term, Triple, vocab};

fn put(graph: &mut Builder, node: &str, key: &str, object: Term) -> Result<(), raf_rdf::Error> {
    graph.insert(Triple {
        subject: Term::iri(node),
        predicate: Term::iri(if key == "type" {
            vocab::TYPE.into()
        } else {
            format!("{}{key}", vocab::REPORT)
        }),
        object,
    })
}

fn ledger(enforced: bool) -> Result<String, raf_rdf::Error> {
    let mut graph = Builder::new();
    put(
        &mut graph,
        "",
        "type",
        Term::iri(format!("{}Ledger", vocab::REPORT)),
    )?;
    put(
        &mut graph,
        "",
        "normalForm",
        Term::typed_literal(NORMAL_FORM.to_string(), vocab::INTEGER),
    )?;
    put(
        &mut graph,
        "",
        "reportForm",
        Term::typed_literal(REPORT_FORM.to_string(), vocab::INTEGER),
    )?;
    put(&mut graph, "", "job", Term::iri("#number"))?;
    put(
        &mut graph,
        "#number",
        "type",
        Term::iri(format!("{}LedgerJob", vocab::REPORT)),
    )?;
    for (key, value) in [
        ("jobId", "number"),
        ("summary", "Use the approved constant"),
        ("homeSymbol", "demo::home"),
        ("forbiddenFingerprint", "rule:no-local-copy"),
    ] {
        put(&mut graph, "#number", key, Term::simple_literal(value))?;
    }
    put(&mut graph, "#number", "enforced", Term::boolean(enforced))?;
    put(
        &mut graph,
        "#number",
        "forbiddenConstant",
        Term::typed_literal("7", vocab::INTEGER),
    )?;
    graph.finish()?.to_text(Format::Turtle)
}

fn source(root: &Path, names: &[&str]) -> Result<(), std::io::Error> {
    let mut text = "pub fn home() -> u32 { 7 }\n".to_owned();
    for name in names {
        text.push_str("pub fn ");
        text.push_str(name);
        text.push_str("() -> u32 { 7 + 1 }\n");
    }
    fs::write(root.join("src/lib.rs"), text)
}

fn gate(root: &Path) -> Gate {
    registration::rule_gate(root.join("src"))
}

fn main() -> Result<(), Box<dyn Error>> {
    let scratch = TempDir::new_in(std::env::temp_dir())?;
    let root = scratch.path();
    fs::create_dir(root.join("src"))?;
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\n[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )?;
    fs::write(root.join(LEDGER_FILE), ledger(false)?)?;
    source(root, &["first", "second"])?;
    let adopted = gate(root).write_debt()?;
    assert_eq!(adopted.jobs, 1);
    gate(root).assert_clean();
    let original = fs::read(root.join(DEBT_FILE))?;
    gate(root).write_debt()?;
    assert_eq!(fs::read(root.join(DEBT_FILE))?, original);

    source(root, &["first", "second", "third"])?;
    assert!(gate(root).write_debt().is_err());
    assert_eq!(fs::read(root.join(DEBT_FILE))?, original);
    source(root, &["first"])?;
    assert!(!gate(root).check()?.is_clean());
    gate(root).write_debt()?;
    gate(root).assert_clean();
    let reduced = fs::read(root.join(DEBT_FILE))?;

    source(root, &["local_copy"])?;
    assert!(gate(root).write_debt().is_err());
    assert_eq!(fs::read(root.join(DEBT_FILE))?, reduced);
    source(root, &[])?;
    assert!(gate(root).write_debt().is_err());
    assert_eq!(fs::read(root.join(DEBT_FILE))?, reduced);
    fs::write(root.join(LEDGER_FILE), ledger(true)?)?;
    gate(root).write_debt()?;
    gate(root).assert_clean();

    let foreign = DebtRecord::new(NORMAL_FORM + 1, [], BTreeMap::new())?;
    fs::write(root.join(DEBT_FILE), foreign.to_turtle()?)?;
    assert!(gate(root).check().is_err());
    gate(root).write_debt()?;
    gate(root).assert_clean();
    println!(
        "adoption, exact bytes, growth refusal, shrinkage, non-debt refusal and version replacement pass"
    );
    locked_debt::exercise()?;
    Ok(())
}
