// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Command vocabulary fixtures exercise actual callers and preserve native bytes.

#![cfg(unix)]

mod support;

use raf_rdf::{Builder, Format, Graph, TermValue, vocab};
use std::collections::BTreeSet;
use support::{Repo, Run, completion, history, strict};

fn terms(graph: &Graph, found: &mut BTreeSet<String>) {
    fn iri(value: &str, found: &mut BTreeSet<String>) {
        if let Some(name) = value.strip_prefix(vocab::REPORT) {
            found.insert(name.into());
        }
    }
    for triple in graph.assertions() {
        for atom in [&triple.subject, &triple.predicate, &triple.object] {
            let _ = atom.visit_terms(|term| {
                match term {
                    TermValue::Iri(v) => iri(v, found),
                    TermValue::Literal { datatype, .. } => iri(datatype, found),
                    _ => {}
                }
                std::ops::ControlFlow::<()>::Continue(())
            });
        }
    }
}
fn observe(name: &str, run: Run, found: &mut BTreeSet<String>) {
    observe_with(name, run, found, &[]);
}
struct Dynamic {
    subject: Option<&'static str>,
    key: &'static str,
    value: TermValue,
}
impl Dynamic {
    fn marker(&self) -> TermValue {
        TermValue::simple_literal(self.subject.map_or_else(
            || format!("@{}@", self.key),
            |subject| format!("@{}:{subject}@", self.key),
        ))
    }
}
fn observe_with(name: &str, run: Run, found: &mut BTreeSet<String>, dynamic: &[Dynamic]) {
    let stdout = run.stdout;
    let graph = Graph::from_text(&stdout, Format::Turtle).unwrap();
    terms(&graph, found);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reports");
    assert_eq!(
        stdout,
        golden(&root, name, "ttl", Format::Turtle, dynamic),
        "{name}: Turtle golden"
    );
    if name == "check" {
        assert!(include_str!("../../../docs/capabilities/workplan-cli.md").contains(&stdout));
    }
    let nt = graph.to_text(Format::NTriples).unwrap();
    assert_eq!(
        Graph::from_text(&nt, Format::NTriples)
            .unwrap()
            .to_text(Format::NTriples)
            .unwrap(),
        nt
    );
    assert_eq!(
        nt,
        golden(&root, name, "nt", Format::NTriples, dynamic),
        "{name}: N-Triples golden"
    );
    let mut triples: Vec<&str> = nt.lines().collect();
    triples.sort_unstable();
    let sorted = format!("{}\n", triples.join("\n"));
    assert_eq!(
        sorted,
        {
            let mut rows: Vec<_> = golden(&root, name, "oracle.nt", Format::NTriples, dynamic)
                .lines()
                .map(str::to_owned)
                .collect();
            rows.sort_unstable();
            format!("{}\n", rows.join("\n"))
        },
        "{name}: distinct native RDF 1.2 parser revision graph pin"
    );
}

fn golden(
    root: &std::path::Path,
    name: &str,
    suffix: &str,
    format: Format,
    dynamic: &[Dynamic],
) -> String {
    let bytes = std::fs::read_to_string(root.join(format!("{name}.{suffix}"))).unwrap();
    if dynamic.is_empty() {
        return bytes;
    }
    let canonical = if suffix == "oracle.nt" {
        let canonical = std::fs::read_to_string(root.join(format!("{name}.nt"))).unwrap();
        let mut rows: Vec<_> = canonical.lines().collect();
        rows.sort_unstable();
        assert_eq!(
            format!("{}\n", rows.join("\n")),
            bytes,
            "independently parsed template pin"
        );
        canonical
    } else {
        bytes
    };
    let graph = Graph::from_text(&canonical, format).expect("native dynamic golden template");
    instantiate(&graph, dynamic).to_text(format).unwrap()
}

fn instantiate(graph: &Graph, dynamic: &[Dynamic]) -> Graph {
    let mut builder = Builder::from_graph(graph);
    let mut seen = vec![false; dynamic.len()];
    for triple in graph.assertions() {
        let TermValue::Iri(predicate) = &triple.predicate else {
            continue;
        };
        let Some(key) = predicate.strip_prefix(vocab::REPORT) else {
            continue;
        };
        let key = key.to_string();
        let Some((index, field)) = dynamic.iter().enumerate().find(|(_, field)| {
            field.key == key
                && field
                    .subject
                    .is_none_or(|subject| triple.subject.as_iri() == Some(subject))
        }) else {
            continue;
        };
        let marker = field.marker();
        assert_eq!(triple.object, marker, "validated dynamic term {key}");
        assert!(builder.remove(triple.clone()).unwrap());
        let mut replacement = triple;
        replacement.object = field.value.clone();
        builder.insert(replacement).unwrap();
        seen[index] = true;
    }
    assert!(
        seen.iter().all(|value| *value),
        "every declared dynamic field is retained"
    );
    builder.finish().unwrap()
}

fn roadmap(found: &mut BTreeSet<String>) {
    let repo = Repo::synced("registry-roadmap", true);
    observe(
        "check",
        repo.run(&["--format", "turtle", "check"]).expect(1),
        found,
    );
    observe(
        "verify-findings",
        repo.run(&["--format", "turtle", "verify"]).expect(1),
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
    history_snapshot(&dated, found);
}

fn documentation(found: &mut BTreeSet<String>) {
    let repo = Repo::synced("registry-documentation", false);
    let path = "docs/capabilities/alpha.md";
    let text = repo.read(path).replacen(
        "bindings:\n",
        "diagrams:\n  - name: dependencies\n    kind: workspace-dependencies\n    scope: workspace\nbindings:\n",
        1,
    );
    repo.put(
        path,
        &format!(
            "{text}\n<!-- raf:generated:diagram-dependencies -->\n<!-- raf:generated:end -->\n"
        ),
    );
    repo.put("published/index.txt", "unrelated host output");
    let run = repo
        .run(&[
            "--format",
            "turtle",
            "docs",
            "build",
            "standalone",
            "--view",
            "cap:alpha#dependencies",
            "--output-dir",
            "published",
        ])
        .expect(0);
    terms(
        &Graph::from_text(&run.stdout, Format::Turtle).unwrap(),
        found,
    );
    assert_eq!(repo.read("published/index.txt"), "unrelated host output");
    imported_documentation(found);
}

fn imported_documentation(found: &mut BTreeSet<String>) {
    let repo = Repo::bare("registry-imported-documentation");
    repo.put(
        "build.rs",
        "fn main() { panic!(\"must never execute\"); }\n",
    );
    let output = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let target = output.path().canonicalize().unwrap();
    std::fs::create_dir_all(target.join("doc/demo")).unwrap();
    let entry = target.join("doc/demo/index.html");
    std::fs::write(
        &entry,
        "<!doctype html><title>existing documentation</title>",
    )
    .unwrap();
    let json = |text: &str| purrdf_lex::json::write_compact(&purrdf_lex::json::Value::from(text));
    let manifest = json(repo.root().join("Cargo.toml").to_str().unwrap());
    let source = json(repo.root().join("src/lib.rs").to_str().unwrap());
    let library = format!(
        "{{\"name\":\"demo\",\"kind\":[\"lib\"],\"crate_types\":[\"lib\"],\"src_path\":{source},\"doc\":true}}"
    );
    let metadata = format!(
        "{{\"version\":1,\"workspace_root\":{},\"target_directory\":{},\"workspace_members\":[\"opaque-demo\"],\"packages\":[{{\"id\":\"opaque-demo\",\"name\":\"demo\",\"version\":\"0.1.0\",\"manifest_path\":{manifest},\"source\":null,\"targets\":[{library}]}}],\"resolve\":{{\"nodes\":[{{\"id\":\"opaque-demo\",\"features\":[]}}]}}}}",
        json(repo.root().to_str().unwrap()),
        json(target.to_str().unwrap())
    );
    let messages = format!(
        "{{\"reason\":\"compiler-artifact\",\"package_id\":\"opaque-demo\",\"manifest_path\":{manifest},\"target\":{library},\"features\":[],\"filenames\":[{}],\"fresh\":false}}\n{{\"reason\":\"build-finished\",\"success\":true}}\n",
        json(entry.to_str().unwrap())
    );
    repo.put("metadata.json", &metadata);
    repo.put("messages.ndjson", &messages);
    let before = repo.worktree();
    let artifacts_before = support::worktree(&target);
    let run = repo
        .run(&[
            "--format",
            "turtle",
            "docs",
            "artifacts",
            "--package",
            "demo",
            "--target-directory",
            target.to_str().unwrap(),
            "--metadata",
            "metadata.json",
            "--messages",
            "messages.ndjson",
        ])
        .expect(0);
    observe_with(
        "documentation-artifact",
        run,
        found,
        &[Dynamic {
            subject: Some("#result/0"),
            key: "entry",
            value: TermValue::simple_literal(entry.to_str().unwrap()),
        }],
    );
    assert_eq!(repo.worktree(), before);
    assert_eq!(support::worktree(&target), artifacts_before);
}

fn history_snapshot(repo: &Repo, found: &mut BTreeSet<String>) {
    let log = repo.git(&["rev-list", "--reverse", "HEAD"]);
    let commits: Vec<_> = log.lines().collect();
    assert_eq!(commits.len(), 5, "the complete signed dated fixture");
    for commit in &commits {
        raf_workplan::identifiers::Commit::new(commit).unwrap();
    }
    let dynamic: Vec<_> = [
        ("#result/0/entry/0", "commit", 2),
        ("#result/0/entry/1", "commit", 1),
        ("#result/0/entry/2", "commit", 0),
        ("#result/0/removed/0", "commit", 3),
        ("#result/0/removed/0", "lastResolved", 2),
    ]
    .into_iter()
    .map(|(subject, key, index)| Dynamic {
        subject: Some(subject),
        key,
        value: TermValue::simple_literal(commits[index]),
    })
    .collect();
    let run = repo
        .run(&["--format", "turtle", "history", "cap:alpha"])
        .expect(0);
    let rdf = support::rdf::Rdf::of(&run);
    for field in &dynamic {
        let node = rdf.graph().get(field.subject.unwrap()).unwrap();
        let values: Vec<_> = node
            .values(&format!("{}{}", vocab::REPORT, field.key))
            .map(raf_rdf::Value::to_owned)
            .collect();
        assert_eq!(
            values.as_slice(),
            std::slice::from_ref(&field.value),
            "the emitted dated row names its actual Git commit"
        );
    }
    observe_with("history", run, found, &dynamic);
}

fn evidence_and_assessments(found: &mut BTreeSet<String>) {
    let repo = Repo::synced("registry-evidence", false);
    let opened = raf_workplan::Gate::workspace(repo.root()).open().unwrap();
    let digest = raf_workplan::digest::Digest::symbol(opened.workspace(), "demo::real").unwrap();
    let mut metadata = raf_okf::EvidenceRecord::new(
        "peak",
        raf_okf::EvidenceSubject::Command,
        vec![raf_okf::Input::new(
            raf_okf::InputKind::Symbol,
            "demo::real",
            digest.to_string(),
        )],
        raf_okf::EvidenceVerdict::Fail,
        raf_okf::Actor::Agent,
        vec!["cap:alpha".into()],
    );
    metadata.command = Some("authored command text; never executed".into());
    metadata.values = vec![
        raf_okf::Measure::new("peak", raf_okf::Number::parse("80").unwrap()),
        raf_okf::Measure::new("rss", raf_okf::Number::parse("3.5").unwrap()),
    ];
    metadata.limits = vec![raf_okf::Measure::new(
        "peak",
        raf_okf::Number::parse("64").unwrap(),
    )];
    raf_workplan::record::create(
        repo.root(),
        &raf_workplan::record::Record::Evidence(metadata),
        &raf_workplan::record::RecordOptions::new(),
    )
    .unwrap();
    repo.edit(
        "docs/capabilities/alpha.md",
        "Text for Contract of alpha.",
        "Text for Contract of alpha.\n<!-- raf:claim evidence=peak -->\n",
    );
    observe(
        "support",
        repo.run(&["--format", "turtle", "evidence", "for", "cap:alpha"])
            .expect(0),
        found,
    );
    assessments(&repo, found);
}

fn assessments(repo: &Repo, found: &mut BTreeSet<String>) {
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
    repo.edit("src/lib.rs", "pub fn real()", "pub fn moved()");
    observe(
        "assessment-unresolved",
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

/// Exercise each review query projection through its actual CLI consumer.
fn review_queries(found: &mut BTreeSet<String>) {
    let repo = Repo::bare("review-query-registry");
    repo.put(
        "docs/capabilities/scope.md",
        &support::capability("scope", "considered", ["[]", "[]", "[]"], "[]", &[]),
    );
    repo.sync();
    repo.checkout();
    let recorded = repo
        .run(&[
            "--format",
            "turtle",
            "review",
            "record",
            "--id",
            "audit",
            "--scope",
            "crate:demo",
            "--date",
            "2030-01-01T00:00:00Z",
            "--kind",
            "correctness",
            "--verdict",
            "clean",
            "--by",
            "human",
            "--copyright",
            support::COPYRIGHT,
            "--licence",
            support::LICENCE,
        ])
        .expect(0);
    terms(
        &Graph::from_text(&recorded.stdout, Format::Turtle).unwrap(),
        found,
    );
    repo.edit(
        "src/lib.rs",
        "pub fn real() {}",
        "pub fn real() { let _value = 1; }",
    );
    let before = repo.worktree();
    for (arguments, code) in [
        (
            &["--format", "turtle", "review", "status", "crate:demo"][..],
            1,
        ),
        (
            &[
                "--format", "turtle", "review", "coverage", "--crate", "demo",
            ][..],
            0,
        ),
        (
            &[
                "--format",
                "turtle",
                "review",
                "unreviewed",
                "--crate",
                "demo",
            ][..],
            1,
        ),
    ] {
        let output = repo.run(arguments).expect(code);
        let graph = Graph::from_text(&output.stdout, Format::Turtle).unwrap();
        terms(&graph, found);
    }
    assert_eq!(repo.worktree(), before);
}

#[test]
fn command_extension_registry_matches_the_documented_terms() {
    let mut found = BTreeSet::new();
    documentation(&mut found);
    roadmap(&mut found);
    lifecycle(&mut found);
    evidence_and_assessments(&mut found);
    line_coverage_and_debt(&mut found);
    review_queries(&mut found);
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
