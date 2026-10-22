// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Domain refusal and graph projection neighbours.

use super::*;
use raf_source::{LoadOptions, MemoryTree, Workspace};

#[path = "../../../raf-workplan/tests/support/strict.rs"]
#[allow(
    dead_code,
    reason = "this test uses the capability from the shared fixture module"
)]
mod strict;

fn reach_item_budget_error(limit: usize) -> String {
    format!(
        "the walk of the bound items failed: the walk is over budget: src/lib.rs:1: exceeds the budget for bytes of index records ({limit})"
    )
}

fn golden_at_item_budget(golden: &Graph, limit: usize) -> Graph {
    let message = |limit| Triple {
        subject: local("#result/0/error/0"),
        predicate: iri("message"),
        object: TermValue::simple_literal(reach_item_budget_error(limit)),
    };
    let mut expected = Builder::from_graph(golden);
    assert!(
        expected
            .remove(message(22_810))
            .expect("the representative fixture message is valid")
    );
    expected
        .insert(message(limit))
        .expect("the configured item ceiling is a valid message");
    expected
        .finish()
        .expect("all other golden assertions remain")
}

fn sorted_triples(text: &str) -> String {
    let mut triples: Vec<&str> = text.lines().collect();
    triples.sort_unstable();
    format!("{}\n", triples.join("\n"))
}

#[test]
fn actual_reach_budget_refusal_projects_a_lines_error_without_partial_rows() {
    let tree = MemoryTree::from([
        ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\n"),
        ("src/lib.rs", "pub fn run() { helper(); }\nfn helper() {}\n"),
    ]);
    let source = Workspace::load(&tree, &["."]).unwrap();
    let capability = strict::capability();
    let docs = MemoryTree::from([("docs/capabilities/demo.md", capability.as_str())]);
    let valid = raf_workplan::Workplan::load(&docs, &source).lines_report();
    assert_eq!(valid.errors(), Vec::<String>::new());
    assert_ne!(valid.crates(), []);
    let mut options = LoadOptions::default();
    // One spare byte cannot hold a reach slot and varies the golden ceiling.
    options.max_item_bytes = source.usage().item_bytes + 1;
    let bounded = Workspace::load_with(&tree, &["."], &options).unwrap();
    let refused = raf_workplan::Workplan::load(&docs, &bounded).lines_report();
    assert_eq!(
        refused.errors(),
        vec![reach_item_budget_error(options.max_item_bytes)]
    );
    assert_eq!(refused.crates(), []);
    assert_eq!(refused.orphans(), []);
    for (report, expected) in [(valid, false), (refused, true)] {
        let result = CommandResult::Lines(report);
        let graph = graph("coverage lines", &[&result]).unwrap();
        assert_eq!(
            graph.assertions().any(|t| t.object == iri("LinesError")),
            expected
        );
        for format in [raf_rdf::Format::Turtle, raf_rdf::Format::NTriples] {
            let text = graph.to_text(format).unwrap();
            assert_eq!(
                Graph::from_text(&text, format)
                    .unwrap()
                    .to_text(format)
                    .unwrap(),
                text
            );
            if expected {
                let suffix = if format == raf_rdf::Format::Turtle {
                    "ttl"
                } else {
                    "nt"
                };
                let expected = std::fs::read_to_string(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join(format!("tests/fixtures/reports/lines-error.{suffix}")),
                )
                .unwrap();
                let golden = Graph::from_text(&expected, format)
                    .expect("the representative golden has the closed report profile");
                assert_eq!(golden.to_text(format).unwrap(), expected);
                let instantiated = golden_at_item_budget(&golden, options.max_item_bytes)
                    .to_text(format)
                    .expect("the same golden with the actual configured ceiling renders");
                assert_eq!(text, instantiated);
                if format == raf_rdf::Format::NTriples {
                    assert_eq!(
                        sorted_triples(&expected),
                        include_str!("../../tests/fixtures/reports/lines-error.oracle.nt")
                    );
                    assert_eq!(sorted_triples(&text), sorted_triples(&instantiated));
                }
            }
        }
    }
}
