// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native policy for an owned reference package and its module home.

use raf_census::{NORMAL_FORM, REPORT_FORM};
use raf_rdf::{Builder, Format, TermValue as Term, Triple, vocab};

pub(crate) fn ledger(reference: &str, home: &str, job: &str) -> String {
    let mut graph = Builder::new();
    let iri = |name| Term::iri(format!("{}{name}", vocab::REPORT));
    for (subject, property, object) in [
        ("", vocab::TYPE.into(), iri("Ledger")),
        (
            "",
            format!("{}normalForm", vocab::REPORT),
            Term::integer(i128::from(NORMAL_FORM)),
        ),
        (
            "",
            format!("{}reportForm", vocab::REPORT),
            Term::integer(i128::from(REPORT_FORM)),
        ),
        ("", format!("{}minTokens", vocab::REPORT), Term::integer(1)),
        (
            "",
            format!("{}root", vocab::REPORT),
            Term::simple_literal("."),
        ),
        (
            "",
            format!("{}reference", vocab::REPORT),
            Term::simple_literal(reference),
        ),
        ("", format!("{}job", vocab::REPORT), Term::iri("#job")),
        ("#job", vocab::TYPE.into(), iri("LedgerJob")),
        (
            "#job",
            format!("{}jobId", vocab::REPORT),
            Term::simple_literal(job),
        ),
        (
            "#job",
            format!("{}summary", vocab::REPORT),
            Term::simple_literal("Shared dependency implementations"),
        ),
        (
            "#job",
            format!("{}homeSymbol", vocab::REPORT),
            Term::simple_literal(home),
        ),
        (
            "#job",
            format!("{}enforced", vocab::REPORT),
            Term::boolean(false),
        ),
    ] {
        graph
            .insert(Triple {
                subject: Term::iri(subject),
                predicate: Term::iri(property),
                object,
            })
            .unwrap();
    }
    graph.finish().unwrap().to_text(Format::Turtle).unwrap()
}
