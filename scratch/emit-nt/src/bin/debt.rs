// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0
use raf_rdf::{Atom, Format, Graph, Iri, Triple, vocab};
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let mut graph = Graph::new();
    let local = |id| Iri::local(id).unwrap();
    let named = |id| Iri::absolute(format!("{}{id}", vocab::REPORT)).unwrap();
    for (subject, class) in [("", "DebtRecord"), ("#debt/body/demo_a::len,demo_b::len", "BodyDebt"), ("#job/hex-encode", "JobDebt")] {
        graph.insert(Triple { subject: local(subject), predicate: Iri::absolute(vocab::TYPE).unwrap(), object: Atom::Iri(named(class)).into() }).unwrap();
    }
    for (subject, property, atom) in [
        ("", "normalForm", Atom::integer(1).unwrap()),
        ("", "reportForm", Atom::integer(2).unwrap()),
        ("#debt/body/demo_a::len,demo_b::len", "memberSymbol", Atom::string("demo_a::len")),
        ("#debt/body/demo_a::len,demo_b::len", "memberSymbol", Atom::string("demo_b::len")),
        ("#job/hex-encode", "copies", Atom::integer(3).unwrap()),
        ("#job/hex-encode", "jobId", Atom::string("hex-encode")),
    ] {
        graph.insert(Triple { subject: local(subject), predicate: named(property), object: atom.into() }).unwrap();
    }
    let text = graph.to_text(Format::Turtle).unwrap();
    let document = std::fs::read_to_string(&path).unwrap();
    let start = document.find("```turtle\n").unwrap() + "```turtle\n".len();
    let end = start + document[start..].find("```").unwrap();
    std::fs::write(path, format!("{}{}{}", &document[..start], text, &document[end..])).unwrap();
}
