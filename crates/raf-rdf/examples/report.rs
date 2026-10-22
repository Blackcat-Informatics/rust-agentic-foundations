// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0
//! A closed ground report with typed fields and named statement provenance.
use raf_rdf::{Builder, Format, Graph, TermBox, TermValue, Triple, vocab};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph = Builder::new();
    let root = TermValue::iri("");
    let predicate = TermValue::iri("urn:example:value");
    for atom in [
        TermValue::simple_literal("é雪\n\0\"\\\u{85}"),
        TermValue::simple_literal("literal \\u0085"),
        TermValue::integer(123),
        TermValue::boolean(true),
        TermValue::typed_literal("\tcustom", "urn:example:datatype"),
    ] {
        graph.insert(Triple {
            subject: root.clone(),
            predicate: predicate.clone(),
            object: atom,
        })?;
    }
    let proof = TermValue::iri("#proof");
    graph.insert(Triple {
        subject: root.clone(),
        predicate: TermValue::iri("urn:example:proof"),
        object: proof.clone(),
    })?;
    graph.insert(Triple {
        subject: proof,
        predicate: TermValue::iri(vocab::REIFIES),
        object: TermValue::Triple {
            s: TermBox::new(root),
            p: TermBox::new(predicate),
            o: TermBox::new(TermValue::integer(123)),
        },
    })?;
    let graph = graph.finish()?;
    let mut args = std::env::args().skip(1);
    let format = match args.next().as_deref() {
        None => Format::Turtle,
        Some("nt") => Format::NTriples,
        Some(_) => return Err("usage: report [nt]".into()),
    };
    if args.next().is_some() {
        return Err("usage: report [nt]".into());
    }
    let text = graph.to_text(format)?;
    assert_eq!(Graph::from_text(&text, format)?.to_text(format)?, text);
    print!("{text}");
    Ok(())
}
