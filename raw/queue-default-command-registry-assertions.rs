// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Command vocabulary fixtures exercise actual callers and preserve native bytes.

#![cfg(unix)]

mod support;

use raf_rdf::{Atom, Format, Graph, Iri, Object, vocab};
use std::collections::BTreeSet;
use support::{Repo, Run, completion, history, strict};

fn terms(graph: &Graph, found: &mut BTreeSet<String>) {
    fn iri(value: &Iri, found: &mut BTreeSet<String>) {
        if let Some(name) = value.as_str().strip_prefix(vocab::REPORT) {
            found.insert(name.into());
        }
    }
    fn atom(value: &Atom, found: &mut BTreeSet<String>) {
        match value {
            Atom::Iri(v) => iri(v, found),
            Atom::Literal { datatype, .. } => iri(datatype, found),
        }
    }
    for triple in graph.triples() {
        iri(&triple.subject, found);
        iri(&triple.predicate, found);
        match &triple.object {
            Object::Atom(v) => atom(v, found),
            Object::Triple(statement) => {
                iri(&statement.subject, found);
                iri(&statement.predicate, found);
                atom(&statement.object, found);
            }
        }
    }
}

fn observe(name: &str, run: Run, found: &mut BTreeSet<String>) {
    let stdout = run.stdout;
    let graph = Graph::from_text(&stdout, Format::Turtle).unwrap();
    terms(&graph, found);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reports");
    assert_eq!(
        stdout,
        std::fs::read_to_string(root.join(format!("{name}.ttl"))).unwrap(),
        "{name}: Turtle golden"
    );
    if name == "check" {
        assert!(include_str!("../../../docs/capabilities/workplan-cli.md").contains(&stdout));
    }
    let nt = graph.to_text(Format::NTriples).unwrap();
    assert_eq!(Graph::from_text(&nt, Format::NTriples).unwrap(), graph);
    assert_eq!(
        nt,
        std::fs::read_to_string(root.join(format!("{name}.nt"))).unwrap(),
        "{name}: N-Triples golden"
    );
}

fn roadmap(found: &mut BTreeSet<String>) {
    let repo = Repo::synced("registry-roadmap", true);
    observe(
        "check",
        repo.run(&["--format", "turtle", "check"]).expect(1),
        found,
    );
    observe(
        "status",
        repo.run(&["--format", "turtle", "status"]).expect(0),
        found,
    );
    observe(
        "preview",
        repo.run(&["--format", "turtle", "docs", "preview", "grow"])
            .expect(0),
        found,
    );
    repo.edit(
        "docs/capabilities/gamma.md",
        "description: A description of gamma.",
        "description: Gamma reads the stream.",
    );
    observe(
        "sync",
        repo.run(&["--format", "turtle", "docs", "sync"]).expect(0),
        found,
    );
    observe(
        "new",
        repo.run(&[
            "--format",
            "turtle",
            "change",
            "new",
            "watch",
            "--capability",
            "cap:alpha",
            "--affects",
            "cap:alpha#contract",
        ])
        .expect(0),
        found,
    );
    repo.edit(
        "docs/capabilities/gamma.md",
        "Text for Usage of gamma.",
        "Text for Usage of gamma, see [it](alpha.md#contract).",
    );
    observe(
        "impact",
        repo.run(&["--format", "turtle", "change", "impact", "watch"])
            .expect(0),
        found,
    );
    observe(
        "coverage-code",
        repo.run(&["--format", "turtle", "coverage", "code"])
            .expect(0),
        found,
    );
    observe(
        "coverage-tests",
        repo.run(&["--format", "turtle", "coverage", "tests"])
            .expect(0),
        found,
    );
}

fn lifecycle(found: &mut BTreeSet<String>) {
    let refused = completion::shaped(
        "registry-refused",
        "implementing",
        "[demo::tests::fails]",
        completion::GOOD,
        "items:\n  - claim: \"fixture is shaped\"",
    );
    observe(
        "reconcile",
        refused
            .run(&["--format", "turtle", "change", "reconcile", "shape"])
            .expect(1),
        found,
    );
    observe(
        "refused",
        refused
            .run(&["--format", "turtle", "change", "complete", "shape"])
            .expect(1),
        found,
    );
    let ready = completion::ready("registry-completed");
    observe(
        "completed",
        ready
            .run(&["--format", "turtle", "change", "complete", "shape"])
            .expect(0),
        found,
    );
    let source = Repo::strict("registry-verified", &strict::capability(), "");
    observe(
        "verify",
        source.run(&["--format", "turtle", "verify"]).expect(0),
        found,
    );
    let dated = history::dated("registry-history");
    observe(
        "history",
        dated
            .run(&["--format", "turtle", "history", "cap:alpha"])
            .expect(0),
        found,
    );
}

fn evidence_and_assessments(found: &mut BTreeSet<String>) {
    let repo = Repo::synced("registry-evidence", false);
    observe(
        "recorded",
        repo.run(&[
            "--format",
            "turtle",
            "evidence",
            "record",
            "--id",
            "peak",
            "--for",
            "cap:alpha",
            "--by",
            "agent",
            "--limit",
            "peak=64",
            "--revision",
            "0123456789abcdef0123456789abcdef01234567",
            "--",
            "/bin/sh",
            "-c",
            "echo peak=80; echo rss=3.5",
        ])
        .expect(1),
        found,
    );
    repo.edit(
        "docs/capabilities/alpha.md",
        "Text for Contract of alpha.",
        "Text for Contract of alpha.\n<!-- raf:claim evidence=peak -->",
    );
    observe(
        "claims",
        repo.run(&["--format", "turtle", "evidence", "claims", "peak"])
            .expect(0),
        found,
    );
    observe(
        "support",
        repo.run(&["--format", "turtle", "evidence", "for", "cap:alpha"])
            .expect(0),
        found,
    );
    observe(
        "context",
        repo.run(&[
            "--format",
            "turtle",
            "assess",
            "cap:alpha#contract",
            "--context",
        ])
        .expect(0),
        found,
    );
    repo.edit(
        "docs/capabilities/beta.md",
        "[demo::real]",
        "[demo::missing]",
    );
    observe(
        "context-unresolved",
        repo.run(&[
            "--format",
            "turtle",
            "assess",
            "cap:beta#contract",
            "--context",
        ])
        .expect(0),
        found,
    );
    observe(
        "assessed",
        repo.run(&[
            "--format",
            "turtle",
            "assess",
            "cap:alpha#contract",
            "--verdict",
            "accepted",
            "--by",
            "agent",
        ])
        .expect(0),
        found,
    );
    repo.edit(
        "docs/capabilities/alpha.md",
        "Text for Contract of alpha.",
        "Alpha states a changed contract.",
    );
    observe(
        "assessment",
        repo.run(&["--format", "turtle", "assess", "--list"])
            .expect(0),
        found,
    );
}

fn line_coverage_and_debt(found: &mut BTreeSet<String>) {
    let record = "ceilings:\n  - crate: demo\n    lines: 10\n    of: shipping\n  - crate: demo\n    percent: 100\n    of: test\n";
    let repo = Repo::strict("registry-lines", &strict::capability(), record);
    repo.edit(
        strict::LIB,
        "pub fn run() {}\n",
        "pub fn run() {}\nfn helper() {}\n",
    );
    observe(
        "lines",
        repo.run(&["--format", "turtle", "coverage", "lines"])
            .expect(1),
        found,
    );
    repo.put(strict::COVERAGE, &strict::coverage(&format!("{record}exceptions:\n  - item: demo::helper\n    reason: Kept for a demonstrated caller.\n")));
    observe(
        "excepted",
        repo.run(&["--format", "turtle", "coverage", "lines"])
            .expect(0),
        found,
    );
    observe(
        "claim-coverage",
        repo.run(&["--format", "turtle", "coverage", "claims"])
            .expect(0),
        found,
    );
    let thin = Repo::strict("registry-debt", &strict::thin("Examples"), "");
    observe(
        "debt",
        thin.run(&["--format", "turtle", "debt", "--write"])
            .expect(0),
        found,
    );
    let error = Graph::from_text(
        include_str!("fixtures/reports/lines-error.ttl"),
        Format::Turtle,
    )
    .unwrap();
    terms(&error, found);
}

#[test]
fn command_extension_registry_matches_the_documented_terms() {
    let mut found = BTreeSet::new();
    roadmap(&mut found);
    lifecycle(&mut found);
    evidence_and_assessments(&mut found);
    line_coverage_and_debt(&mut found);
    let base: BTreeSet<String> = raf_workplan::report::TERMS
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    let registered: BTreeSet<String> = raf_workplan::report::COMMAND_TERMS
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    assert_eq!(registered.len(), raf_workplan::report::COMMAND_TERMS.len());
    assert!(registered.is_disjoint(&base));
    let document = include_str!("../../../docs/capabilities/rdf-report-profile.md");
    let table = document
        .split("| Command class | Properties and object types |")
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
        .filter(|s| !base.contains(s))
        .collect();
    assert_eq!(registered, documented);
    let emitted: BTreeSet<_> = found.difference(&base).cloned().collect();
    assert_eq!(emitted, registered);
}
