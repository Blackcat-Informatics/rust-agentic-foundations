// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0
//! Canonical negative report inputs, not changes to production code.
use raf_rdf::{Atom, Format, Graph, Iri, Object, Triple, vocab};
fn property(name: &str) -> Iri { Iri::absolute(format!("{}{name}", vocab::REPORT)).unwrap() }
fn main() {
    let baseline = Graph::from_text(&std::fs::read_to_string("crates/raf-census/tests/fixtures/reports/baseline.ttl").unwrap(), Format::Turtle).unwrap();
    let index = Graph::from_text(&std::fs::read_to_string("crates/raf-census/tests/fixtures/reports/index.ttl").unwrap(), Format::Turtle).unwrap();
    for case in ["unknown", "copies", "form", "orphan", "missing", "mistyped", "class", "name", "individual", "datatype", "repeated", "single"] {
        let source = if matches!(case, "copies" | "individual") { &index } else { &baseline };
        let mut graph = Graph::new();
        for mut triple in source.triples().cloned() {
            if (case == "orphan" && triple.predicate == property("holds")) || (case == "missing" && triple.predicate == property("minTokens")) { continue; }
            if case == "single" && triple.predicate == property("member") && matches!(&triple.object, Object::Atom(Atom::Iri(iri)) if iri.as_str().contains("demo::beta@")) { continue; }
            match case {
                "copies" if triple.predicate == property("copies") => triple.object = Atom::integer(3).unwrap().into(),
                "form" if triple.predicate == property("reportForm") => triple.object = Atom::integer(1).unwrap().into(),
                "mistyped" if triple.predicate == property("normalForm") => triple.object = Atom::string("one").into(),
                "class" if triple.object == Object::Atom(Atom::Iri(property("Unit"))) => triple.object = Atom::Iri(property("Widget")).into(),
                "name" if triple.predicate == property("line") => triple.object = Atom::integer(99).unwrap().into(),
                "individual" if triple.predicate == property("matchClass") => triple.object = Atom::Iri(property("Duplicate")).into(),
                "datatype" if triple.predicate == property("minTokens") => triple.object = Atom::Literal { lexical: "20".into(), datatype: Iri::absolute("urn:unsupported").unwrap() }.into(),
                _ => {}
            }
            graph.insert(triple).unwrap();
        }
        if matches!(case, "unknown" | "repeated") {
            graph.insert(Triple { subject: Iri::local("").unwrap(), predicate: property(if case == "unknown" { "extra" } else { "minTokens" }), object: Atom::integer(21).unwrap().into() }).unwrap();
        }
        assert_ne!(&graph, source);
        let text = graph.to_text(Format::Turtle).unwrap();
        assert!(Graph::from_text(&text, Format::Turtle).is_ok());
        std::fs::write(format!("fuzz/corpus/census-report-read/{case}.ttl"), text).unwrap();
    }
}
