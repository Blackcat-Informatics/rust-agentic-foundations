// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native policy fixture shared by the runnable example and command tests.

use raf_rdf::{Builder, Format, TermValue as Term, Triple, vocab};

pub(crate) fn ledger(owner: &str) -> String {
    let mut graph = Builder::new();
    let iri = |name: &str| Term::iri(format!("{}{name}", vocab::REPORT));
    let mut put = |subject: &str, key: &str, object| {
        graph
            .insert(Triple {
                subject: Term::iri(subject),
                predicate: Term::iri(if key == "type" {
                    vocab::TYPE.into()
                } else if key.starts_with("urn:") {
                    key.into()
                } else {
                    format!("{}{key}", vocab::REPORT)
                }),
                object,
            })
            .unwrap();
    };
    put("", "type", iri("Ledger"));
    for (key, value) in [
        ("normalForm", raf_census::NORMAL_FORM),
        ("reportForm", raf_census::REPORT_FORM),
    ] {
        put("", key, Term::integer(value));
    }
    put("", "job", Term::iri("#job"));
    put("#job", "type", iri("LedgerJob"));
    for (key, value) in [
        ("jobId", "number"),
        ("summary", "Use home"),
        ("homeSymbol", "demo::home"),
        ("forbiddenFingerprint", "rule:no-local-copy"),
    ] {
        put("#job", key, Term::simple_literal(value));
    }
    put("#job", "enforced", Term::boolean(true));
    put("#job", "extension", Term::iri("#ext"));
    put("#ext", "type", iri("Extension"));
    put("#ext", "urn:example:owner", Term::simple_literal(owner));
    graph.finish().unwrap().to_text(Format::Turtle).unwrap()
}
