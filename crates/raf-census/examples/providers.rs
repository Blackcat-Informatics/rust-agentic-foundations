// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Configured provider proof and competing-table detection in an owned source fixture.
//!
//! The selected public encoder is the approved implementation. Its adapter
//! returns the actual provider result; another helper in that package receives
//! no exemption merely because the adapter was proved.

use std::error::Error;

use raf_census::{Gate, GateReport, NORMAL_FORM, REPORT_FORM};
use raf_rdf::{Builder, Format, TermValue as Term, Triple, vocab};
use raf_source::{
    DeclaredCfg, DeclaredConfiguration, LoadOptions, LockedCatalog, LockedEdge, LockedManifest,
    MemoryTree, RelPath, TargetKind,
};

fn ledger() -> Result<String, raf_rdf::Error> {
    let term = |name| Term::iri(format!("{}{name}", vocab::REPORT));
    let mut graph = Builder::new();
    for (subject, predicate, object) in [
        ("", Term::iri(vocab::TYPE), term("Ledger")),
        (
            "",
            term("normalForm"),
            Term::typed_literal(NORMAL_FORM.to_string(), vocab::INTEGER),
        ),
        (
            "",
            term("reportForm"),
            Term::typed_literal(REPORT_FORM.to_string(), vocab::INTEGER),
        ),
        ("", term("job"), Term::iri("#digits")),
        ("#digits", Term::iri(vocab::TYPE), term("LedgerJob")),
        ("#digits", term("jobId"), Term::simple_literal("digits")),
        (
            "#digits",
            term("summary"),
            Term::simple_literal("Use the approved digit encoder"),
        ),
        (
            "#digits",
            term("homeSymbol"),
            Term::simple_literal("native_provider::encode"),
        ),
        ("#digits", term("enforced"), Term::boolean(true)),
        (
            "#digits",
            term("forbiddenFingerprint"),
            Term::simple_literal("rule:hex-table"),
        ),
        ("#digits", term("preferredProvider"), Term::iri("#provider")),
        (
            "#provider",
            Term::iri(vocab::TYPE),
            term("PreferredProvider"),
        ),
        (
            "#provider",
            term("package"),
            Term::simple_literal("provider"),
        ),
        (
            "#provider",
            term("entryPoint"),
            Term::simple_literal("native_provider::encode"),
        ),
        (
            "#provider",
            term("consumer"),
            Term::simple_literal("consumer::run"),
        ),
    ] {
        graph.insert(Triple {
            subject: Term::iri(subject),
            predicate,
            object,
        })?;
    }
    graph.finish()?.to_text(Format::Turtle)
}

fn context(name: &str) -> DeclaredConfiguration<'_> {
    DeclaredConfiguration::new(
        DeclaredCfg::complete("declared-example-target", &[]),
        &[],
        TargetKind::Lib,
        name,
    )
}

fn check(competitor: bool) -> Result<GateReport, Box<dyn Error>> {
    let ledger = ledger()?;
    let consumer = format!(
        "pub fn run(value:u8)->char {{renamed::encode(value)}}\n{}",
        if competitor {
            "pub fn other(value:u8)->char {let digits=b\"0123456789abcdef\"; digits[usize::from(value & 15)] as char}"
        } else {
            ""
        }
    );
    let tree = MemoryTree::from([
        ("Cargo.toml", "[workspace]\nmembers=['consumer','provider']"),
        (
            "consumer/Cargo.toml",
            "[package]\nname='consumer'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nrenamed={package='provider',path='../provider',version='1'}",
        ),
        ("consumer/src/lib.rs", consumer.as_str()),
        (
            "provider/Cargo.toml",
            "[package]\nname='provider'\nversion='1.0.0'\nedition='2024'\n[lib]\nname='native_provider'",
        ),
        (
            "provider/src/lib.rs",
            "pub fn encode(value:u8)->char {b\"0123456789abcdef\"[usize::from(value & 15)] as char}",
        ),
        (
            "Cargo.lock",
            "version=4\n[[package]]\nname='consumer'\nversion='1.0.0'\ndependencies=['provider']\n[[package]]\nname='provider'\nversion='1.0.0'",
        ),
        ("helpers-ledger.ttl", ledger.as_str()),
    ]);
    let catalog =
        LockedCatalog::load(&tree, &RelPath::new("Cargo.lock")?, &LoadOptions::default())?;
    let row = |name| {
        catalog
            .packages()
            .find(|row| row.name() == name)
            .map(raf_source::LockedPackage::id)
            .ok_or_else(|| format!("missing locked package {name}"))
    };
    let edges = [LockedEdge::new("dependencies.renamed", None)];
    let manifests = [
        LockedManifest::new(
            row("consumer")?,
            RelPath::new("consumer/Cargo.toml")?,
            &edges,
        )
        .with_configuration(context("consumer")),
        LockedManifest::new(row("provider")?, RelPath::new("provider/Cargo.toml")?, &[])
            .with_configuration(context("native_provider")),
    ];
    Ok(Gate::workspace(".").check_locked(&tree, &catalog, &manifests, &[])?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let clean = check(false)?;
    assert!(clean.is_clean(), "{:?}", clean.findings());
    let competing = check(true)?;
    assert!(
        competing
            .findings()
            .iter()
            .any(|finding| finding.contains("rule hex-table:")),
        "{:?}",
        competing.findings()
    );
    println!("configured provider: clean adapter accepted; competing table found");
    Ok(())
}
