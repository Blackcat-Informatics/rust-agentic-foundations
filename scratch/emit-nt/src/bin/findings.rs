// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

use raf_rdf::{Format, Graph, Properties, TermValue, vocab};
use std::collections::BTreeMap;

fn literal<'a>(properties: &'a Properties, key: &str) -> &'a str {
    match properties.get(&format!("{}{key}", vocab::REPORT)).map(Vec::as_slice) {
        Some([TermValue::Literal { lexical_form, datatype, language: None, direction: None }])
            if datatype == vocab::STRING => lexical_form,
        other => panic!("expected one string {key}, got {other:?}"),
    }
}

fn main() {
    let input = std::env::args().nth(1).expect("canonical check output path");
    let text = std::fs::read_to_string(input).expect("check output");
    let graph = Graph::from_text(&text, Format::Turtle).expect("canonical check graph");
    let nodes = graph.into_nodes();
    assert_eq!(literal(&nodes[""], "command"), "check");
    let mut kinds = BTreeMap::<&str, usize>::new();
    let mut files = BTreeMap::<&str, usize>::new();
    for (id, node) in &nodes {
        if id.is_empty() { continue; }
        assert_eq!(node.get(vocab::TYPE).map(Vec::as_slice), Some([TermValue::iri(format!("{}Finding", vocab::REPORT))].as_slice()));
        let kind = literal(node, "kind");
        let file = literal(node, "file");
        *kinds.entry(kind).or_default() += 1;
        *files.entry(file).or_default() += 1;
        println!("FINDING\t{}\t{}\t{}\t{}", file.escape_debug(), kind.escape_debug(), literal(node, "message").escape_debug(), literal(node, "fix").escape_debug());
    }
    println!("TOTAL\t{}", kinds.values().sum::<usize>());
    for (kind,count) in kinds { println!("KIND\t{kind}\t{count}"); }
    for (file,count) in files { println!("FILE\t{file}\t{count}"); }
}
