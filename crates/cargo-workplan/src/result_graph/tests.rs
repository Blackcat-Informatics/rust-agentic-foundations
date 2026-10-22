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
    options.max_item_bytes = source.usage().item_bytes;
    let bounded = Workspace::load_with(&tree, &["."], &options).unwrap();
    let refused = raf_workplan::Workplan::load(&docs, &bounded).lines_report();
    assert_eq!(refused.errors().len(), 1);
    assert!(refused.errors()[0].contains("budget"));
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
                assert_eq!(text, expected);
                if format == raf_rdf::Format::NTriples {
                    let mut triples: Vec<&str> = text.lines().collect();
                    triples.sort_unstable();
                    assert_eq!(
                        format!("{}\n", triples.join("\n")),
                        include_str!("../../tests/fixtures/reports/lines-error.oracle.nt")
                    );
                }
            }
        }
    }
}
