// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Actual finding and coverage projections retain typed fields and provenance.

use raf_okf::Location;
use raf_rdf::{Format, Graph, TermValue, vocab};
use raf_source::{MemoryTree, Workspace};
use raf_workplan::{Workplan, coverage::GapKind, report};

fn literals<'a>(graph: &'a Graph, property: &str) -> Vec<&'a str> {
    let property = format!("{}{property}", vocab::REPORT);
    let mut nodes: Vec<_> = graph.nodes().collect();
    nodes.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    nodes
        .into_iter()
        .flat_map(|(_, node)| node.values(&property))
        .filter_map(|value| value.literal().map(|(lexical, _)| lexical))
        .collect()
}
fn both_formats(graph: &Graph) {
    for format in [Format::Turtle, Format::NTriples] {
        let text = graph.to_text(format).unwrap();
        assert_eq!(
            Graph::from_text(&text, format)
                .unwrap()
                .to_text(format)
                .unwrap(),
            text
        );
        assert!(!text.contains("_:"));
    }
}

fn pinned(name: &str, graph: &Graph) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reports");
    for (suffix, format) in [("ttl", Format::Turtle), ("nt", Format::NTriples)] {
        let expected = std::fs::read_to_string(root.join(format!("{name}.{suffix}"))).unwrap();
        assert_eq!(graph.to_text(format).unwrap(), expected);
    }
    let nt = graph.to_text(Format::NTriples).unwrap();
    let mut triples: Vec<_> = nt.lines().collect();
    triples.sort_unstable();
    let oracle = std::fs::read_to_string(root.join(format!("{name}.oracle.nt"))).unwrap();
    assert_eq!(format!("{}\n", triples.join("\n")), oracle);
}
#[test]
fn findings_preserve_all_location_variants_and_refuse_unknown_kinds() {
    let source = Workspace::load(
        &MemoryTree::from([
            ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\n"),
            ("src/lib.rs", ""),
        ]),
        &["."],
    )
    .unwrap();
    let docs = MemoryTree::from([("docs/architecture/broken.md", "not a document\n")]);
    let actual = Workplan::load(&docs, &source).check();
    assert_ne!(actual.len(), 0);
    let graph = report::findings_graph(&actual).unwrap();
    pinned("findings", &graph);
    assert!(literals(&graph, "kind").contains(&"stray-markdown"));
    both_formats(&graph);
    let findings: Vec<_> = [
        Location::Document,
        Location::Key("binding".into()),
        Location::Section("Contract".into()),
        Location::Line(17),
    ]
    .into_iter()
    .map(|location| {
        let mut finding = actual[0].clone();
        finding.kind = "stray-markdown";
        finding.file = "é.md".into();
        finding.location = location;
        finding.message = "message\n\0".into();
        finding.fix = "a concrete fix".into();
        finding
    })
    .collect();
    let graph = report::findings_graph(&findings).unwrap();
    assert_eq!(literals(&graph, "ordinal"), ["0", "1", "2", "3"]);
    assert_eq!(literals(&graph, "key"), ["binding"]);
    assert_eq!(literals(&graph, "section"), ["Contract"]);
    assert_eq!(literals(&graph, "line"), ["17"]);
    assert_eq!(literals(&graph, "message"), ["message\n\0"; 4]);
    both_formats(&graph);
    let mut invalid = findings;
    invalid[0].location = Location::Line(0);
    assert!(report::findings_graph(&invalid).is_err());
    invalid[0].location = Location::Line(1);
    assert!(report::findings_graph(&invalid).is_ok());
    invalid[0].kind = "unregistered";
    assert!(report::findings_graph(&invalid).is_err());
}
#[test]
fn real_coverage_keeps_packages_gaps_exclusions_and_named_source_provenance() {
    let tree = MemoryTree::from([
        ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\n"),
        (
            "src/lib.rs",
            "pub fn open() {}\npub fn excluded() {}\n#[cfg(test)] mod tests { #[test] fn case() {} }\n",
        ),
    ]);
    let source = Workspace::load(&tree, &["."]).unwrap();
    let docs = MemoryTree::from([(
        "work/coverage/default.md",
        "---\ntype: Coverage\nid: coverage:default\nscope: [demo]\nexclusions:\n  - item: demo::excluded\n    reason: A deliberate example.\n---\n",
    )]);
    let coverage = Workplan::load(&docs, &source).coverage_report();
    assert_eq!(coverage.code().len(), 1);
    assert_eq!(coverage.excluded().len(), 1);
    assert_eq!(coverage.tests().len(), 1);
    let graph = report::coverage_graph(&coverage, GapKind::PublicItem).unwrap();
    pinned("coverage-code", &graph);
    assert!(literals(&graph, "item").contains(&"demo::open"));
    assert!(literals(&graph, "item").contains(&"demo::excluded"));
    assert_eq!(literals(&graph, "reason"), ["A deliberate example."]);
    assert!(literals(&graph, "inScope").iter().all(|v| *v == "true"));
    assert_eq!(
        graph
            .assertions()
            .filter(|t| t.predicate == TermValue::iri(vocab::REIFIES))
            .count(),
        2
    );
    assert!(
        !graph
            .assertions()
            .any(|t| matches!(&t.predicate, TermValue::Iri(p) if p.contains("run")))
    );
    both_formats(&graph);
    let tests = report::coverage_graph(&coverage, GapKind::Test).unwrap();
    pinned("coverage-tests", &tests);
    assert!(
        literals(&tests, "item")
            .iter()
            .any(|v| v.ends_with("tests::case"))
    );
    assert_eq!(literals(&tests, "reason"), [] as [&str; 0]);
    both_formats(&tests);
}
#[test]
fn unreadable_public_surface_retains_its_structured_failure() {
    let source = Workspace::load(&MemoryTree::from([
        ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\n"),
        ("src/lib.rs", "pub mod left { pub fn dup() {} } pub mod right { pub fn dup() {} } pub use left::*; pub use right::*;"),
    ]), &["."]).unwrap();
    let coverage = Workplan::load(&MemoryTree::default(), &source).coverage_report();
    assert_eq!(coverage.errors().len(), 1);
    let graph = report::coverage_graph(&coverage, GapKind::PublicItem).unwrap();
    pinned("coverage-error", &graph);
    assert_eq!(literals(&graph, "failure"), ["unresolved"]);
    assert_eq!(literals(&graph, "target"), ["demo"]);
    assert_eq!(literals(&graph, "root"), ["src/lib.rs"]);
    assert_eq!(literals(&graph, "inScope"), ["false", "false"]);
    both_formats(&graph);
}

#[test]
fn unresolved_reexports_keep_declaration_observations_without_claiming_a_definition() {
    let source = Workspace::load(&MemoryTree::from([
        ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n"),
        ("src/lib.rs", "mod generated { macro_rules! make { () => { pub struct Made; }; } make!(); }\npub use generated::Made;\npub struct Plain;\n"),
    ]), &["."]).unwrap();
    let coverage = Workplan::load(&MemoryTree::default(), &source).coverage_report();
    assert!(
        coverage
            .code()
            .iter()
            .any(|gap| gap.kind() == GapKind::Reexport)
    );
    assert!(
        coverage
            .code()
            .iter()
            .any(|gap| gap.kind() == GapKind::PublicItem)
    );
    let graph = report::coverage_graph(&coverage, GapKind::PublicItem).unwrap();
    assert!(literals(&graph, "item").contains(&"demo::generated::Made"));
    assert!(graph.assertions().any(|t| matches!(&t.object, TermValue::Triple { o, .. } if matches!(o.as_ref(), TermValue::Literal { lexical_form, .. } if lexical_form == "demo::generated::Made"))));
    assert!(
        !graph
            .assertions()
            .any(|t| matches!(&t.predicate, TermValue::Iri(p) if p.ends_with("#definition")))
    );
    both_formats(&graph);
}
