// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Evaluate one native pattern against approved and unsanctioned source.

use raf_census::Gate;
use raf_rdf::{Builder, Format, TermValue as Term, Triple, vocab};
use raf_source::MemoryTree;

fn ledger() -> Result<String, raf_rdf::Error> {
    let mut graph = Builder::new();
    for (subject, key, object) in [
        ("", "type", Term::iri(format!("{}Ledger", vocab::REPORT))),
        ("", "normalForm", Term::typed_literal("1", vocab::INTEGER)),
        ("", "reportForm", Term::integer(raf_census::REPORT_FORM)),
        ("", "job", Term::iri("#release")),
        ("", "pattern", Term::iri("#direct-drop")),
        (
            "#release",
            "type",
            Term::iri(format!("{}LedgerJob", vocab::REPORT)),
        ),
        ("#release", "jobId", Term::simple_literal("release")),
        (
            "#release",
            "summary",
            Term::simple_literal("Release values through the approved wrapper"),
        ),
        (
            "#release",
            "homeSymbol",
            Term::simple_literal("demo::release"),
        ),
        ("#release", "enforced", Term::boolean(true)),
        (
            "#release",
            "forbiddenPattern",
            Term::simple_literal("direct-drop"),
        ),
        (
            "#direct-drop",
            "type",
            Term::iri(format!("{}Pattern", vocab::REPORT)),
        ),
        (
            "#direct-drop",
            "patternId",
            Term::simple_literal("direct-drop"),
        ),
        (
            "#direct-drop",
            "summary",
            Term::simple_literal("A direct release call outside its wrapper"),
        ),
        (
            "#direct-drop",
            "call",
            Term::simple_literal("std::mem::drop"),
        ),
    ] {
        graph.insert(Triple {
            subject: Term::iri(subject),
            predicate: Term::iri(if key == "type" {
                vocab::TYPE.into()
            } else {
                format!("{}{key}", vocab::REPORT)
            }),
            object,
        })?;
    }
    graph.finish()?.to_text(Format::Turtle)
}

fn tree(other: &str, policy: &str) -> MemoryTree {
    [
        (
            "Cargo.toml",
            "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n".to_owned(),
        ),
        (
            "src/lib.rs",
            format!("pub fn release(){{std::mem::drop(0);}} pub fn other(){{{other}}}"),
        ),
        ("helpers-ledger.ttl", policy.to_owned()),
    ]
    .into_iter()
    .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gate = Gate::workspace(".");
    let policy = ledger()?;
    let refused = gate.check_tree(&tree("std::mem::drop(1);", &policy))?;
    assert_eq!(refused.jobs()[0].copies, 1);
    assert!(!refused.is_clean());
    assert!(
        refused
            .findings()
            .iter()
            .any(|finding| finding.contains("direct-drop"))
    );
    let approved = gate.check_tree(&tree("release();", &policy))?;
    assert!(approved.is_clean(), "{:?}", approved.findings());
    println!("native pattern execution, copy reporting and approved wrapper calls pass");
    Ok(())
}
