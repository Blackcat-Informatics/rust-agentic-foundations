// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Complete typed coverage projection and vocabulary ownership checks.

use super::{CoverageReport, GapKind, PackageCoverage, SurfaceError, SurfaceFailure};
use raf_rdf::{TermBox, TermValue};

fn collect_terms(graph: &raf_rdf::Graph, terms: &mut std::collections::BTreeSet<String>) {
    fn iri(value: &str, terms: &mut std::collections::BTreeSet<String>) {
        if let Some(name) = value.strip_prefix(raf_rdf::vocab::REPORT) {
            terms.insert(name.into());
        }
    }
    for t in graph.assertions() {
        for atom in [&t.subject, &t.predicate, &t.object] {
            let _ = atom.visit_terms(|v| {
                match v {
                    TermValue::Iri(v) => iri(v, terms),
                    TermValue::Literal { datatype, .. } => iri(datatype, terms),
                    _ => {}
                }
                std::ops::ControlFlow::<()>::Continue(())
            });
        }
    }
}
#[test]
fn emitted_application_iris_belong_to_the_documented_registry() {
    use crate::{Workplan, report};
    use raf_okf::Location;
    use raf_source::{MemoryTree, Workspace};
    use std::collections::BTreeSet;

    let document = include_str!("../../../../docs/capabilities/rdf-report-profile.md");
    let table = document
        .split("| Class | Properties and object types |")
        .nth(1)
        .unwrap()
        .split("\n\n")
        .next()
        .unwrap();
    let documented: BTreeSet<String> = table
        .split('\x60')
        .enumerate()
        .filter(|(i, text)| i % 2 == 1 && !text.contains(':'))
        .map(|(_, text)| text.to_owned())
        .collect();
    assert_eq!(
        documented,
        report::TERMS.iter().map(|s| (*s).to_owned()).collect()
    );
    let mut emitted = BTreeSet::new();
    let source = Workspace::load(&MemoryTree::from([
        ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n"),
        ("src/lib.rs", "pub fn open() {}\npub fn excluded() {}\n#[cfg(test)] mod tests { #[test] fn case() {} }\n"),
    ]), &["."]).unwrap();
    let docs = MemoryTree::from([
        ("docs/architecture/broken.md", "not a document\n"),
        (
            "work/coverage/default.md",
            "---\ntype: Coverage\nid: coverage:default\nscope: [demo]\nexclusions:\n  - item: demo::excluded\n    reason: A deliberate example.\n---\n",
        ),
    ]);
    let plan = Workplan::load(&docs, &source);
    let actual = plan.check();
    assert_ne!(actual, []);
    let findings: Vec<_> = [
        Location::Document,
        Location::Key("binding".into()),
        Location::Section("Contract".into()),
        Location::Line(17),
    ]
    .into_iter()
    .map(|location| {
        let mut finding = actual[0].clone();
        finding.location = location;
        finding
    })
    .collect();
    collect_terms(&report::findings_graph(&findings).unwrap(), &mut emitted);
    let coverage = plan.coverage_report();
    assert_eq!(coverage.excluded().len(), 1);
    assert_eq!(coverage.tests().len(), 1);
    for direction in [GapKind::PublicItem, GapKind::Test] {
        collect_terms(
            &report::coverage_graph(&coverage, direction).unwrap(),
            &mut emitted,
        );
    }
    let reexports = Workspace::load(&MemoryTree::from([
        ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n"),
        ("src/lib.rs", "mod generated { macro_rules! make { () => { pub struct Made; }; } make!(); }\npub use generated::Made;\npub struct Plain;\n"),
    ]), &["."]).unwrap();
    let plan = Workplan::load(&MemoryTree::default(), &reexports);
    let coverage = plan.coverage_report();
    assert!(
        coverage
            .code()
            .iter()
            .any(|gap| gap.kind() == GapKind::Reexport)
    );
    collect_terms(
        &report::coverage_graph(&coverage, GapKind::PublicItem).unwrap(),
        &mut emitted,
    );
    for report in surface_reports() {
        for direction in [GapKind::PublicItem, GapKind::Test] {
            collect_terms(
                &report::coverage_graph(&report, direction).unwrap(),
                &mut emitted,
            );
        }
    }
    assert_eq!(emitted, documented);
}

#[test]
fn registry_collection_visits_quoted_atoms_and_both_datatype_positions() {
    use raf_rdf::{Builder, Triple, vocab};
    let named = |name: &str| TermValue::iri(format!("{}{name}", vocab::REPORT));
    let mut graph = Builder::new();
    graph
        .insert(Triple {
            subject: TermValue::iri(""),
            predicate: named("outer"),
            object: TermValue::Triple {
                s: TermBox::new(named("subject")),
                p: TermBox::new(named("predicate")),
                o: TermBox::new(TermValue::typed_literal(
                    "value",
                    format!("{}nestedDatatype", vocab::REPORT),
                )),
            },
        })
        .unwrap();
    graph
        .insert(Triple {
            subject: TermValue::iri(""),
            predicate: named("literal"),
            object: TermValue::typed_literal("value", format!("{}datatype", vocab::REPORT)),
        })
        .unwrap();
    graph
        .insert(Triple {
            subject: TermValue::iri(""),
            predicate: named("iri"),
            object: named("object"),
        })
        .unwrap();
    graph
        .insert(Triple {
            subject: TermValue::iri(""),
            predicate: named("quotedIri"),
            object: TermValue::Triple {
                s: TermBox::new(named("subject")),
                p: TermBox::new(named("predicate")),
                o: TermBox::new(named("nestedObject")),
            },
        })
        .unwrap();
    let graph = graph.finish().unwrap();
    let mut found = std::collections::BTreeSet::new();
    collect_terms(&graph, &mut found);
    assert_eq!(
        found,
        [
            "outer",
            "subject",
            "predicate",
            "nestedDatatype",
            "literal",
            "datatype",
            "iri",
            "object",
            "quotedIri",
            "nestedObject"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
}

fn surface_reports() -> Vec<CoverageReport> {
    [
        SurfaceFailure::Budget,
        SurfaceFailure::CarryBudget {
            path: "demo::target::Item".into(),
        },
        SurfaceFailure::Unresolved,
        SurfaceFailure::Other,
    ]
    .into_iter()
    .map(|failure| CoverageReport {
        packages: vec![PackageCoverage {
            name: "demo".into(),
            in_scope: true,
            items: 0,
            covered_items: 0,
            tests: 0,
            covered_tests: 0,
        }],
        errors: vec![SurfaceError {
            package: "demo".into(),
            target: "demo".into(),
            root: "src/lib.rs".into(),
            failure,
            message: "surface unavailable".into(),
        }],
        ..CoverageReport::default()
    })
    .collect()
}

#[test]
fn surface_failure_projection_preserves_optional_paths_and_direction() {
    for (failure, name, path) in [
        (SurfaceFailure::Budget, "budget", None),
        (
            SurfaceFailure::CarryBudget {
                path: "demo::target::Item".into(),
            },
            "carry-budget",
            Some("demo::target::Item"),
        ),
        (SurfaceFailure::Unresolved, "unresolved", None),
        (SurfaceFailure::Other, "other", None),
    ] {
        let report = CoverageReport {
            packages: vec![PackageCoverage {
                name: "demo".into(),
                in_scope: true,
                items: 0,
                covered_items: 0,
                tests: 0,
                covered_tests: 0,
            }],
            errors: vec![SurfaceError {
                package: "demo".into(),
                target: "demo".into(),
                root: "src/lib.rs".into(),
                failure,
                message: "surface unavailable".into(),
            }],
            ..CoverageReport::default()
        };
        let graph = crate::report::coverage_graph(&report, GapKind::PublicItem).unwrap();
        assert!(graph.assertions().any(
            |t| matches!(&t.predicate, TermValue::Iri(p) if p.ends_with("#failure"))
                && t.object == TermValue::simple_literal(name)
        ));
        let paths: Vec<_> = graph
            .assertions()
            .filter(|t| matches!(&t.predicate, TermValue::Iri(p) if p.ends_with("#failurePath")))
            .map(|t| t.object)
            .collect();
        let expected: Vec<_> = path.map(TermValue::simple_literal).into_iter().collect();
        assert_eq!(paths, expected);
        let tests = crate::report::coverage_graph(&report, GapKind::Test).unwrap();
        assert!(
            !tests
                .assertions()
                .any(|t| matches!(&t.predicate, TermValue::Iri(p) if p.ends_with("#failure")))
        );
    }
}
