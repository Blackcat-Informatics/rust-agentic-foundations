// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A native report and named RDF 1.2 source statement in both formats.
use raf_rdf::{Builder, Format, Graph, TermBox, TermValue as Term, Triple, vocab};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::new();
    builder.insert(Triple {
        subject: Term::iri(""),
        predicate: Term::iri("urn:example:value"),
        object: Term::integer(7),
    })?;
    builder.insert(Triple {
        subject: Term::iri("#proof"),
        predicate: Term::iri(vocab::REIFIES),
        object: Term::Triple {
            s: TermBox::new(Term::iri("")),
            p: TermBox::new(Term::iri("urn:example:value")),
            o: TermBox::new(Term::integer(7)),
        },
    })?;
    let graph = builder.finish()?;
    for format in [Format::Turtle, Format::NTriples] {
        let text = graph.to_text(format)?;
        assert_eq!(
            Graph::from_text(&text, format)?.snapshot()?,
            graph.snapshot()?
        );
        print!("{text}");
    }
    Ok(())
}
