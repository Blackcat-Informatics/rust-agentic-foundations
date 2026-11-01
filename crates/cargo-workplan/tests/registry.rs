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
        if self.key == "originalTracked" {
            TermValue::integer(0)
        } else {
            TermValue::simple_literal(self.subject.map_or_else(
                || format!("@{}@", self.key),
                |subject| format!("@{}:{subject}@", self.key),
            ))
        }
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

fn completion_facts(
    repo: &Repo,
    run: &Run,
    original: &[(&'static str, TermValue)],
) -> Vec<Dynamic> {
    let query = support::rdf::Rdf::of(run);
    let runs = query.rows("VerificationRun");
    assert!(
        !runs.is_empty(),
        "bound completion retains actual execution"
    );
    let first = runs[0];
    let digest = first.required("viewDigest");
    assert_eq!(digest.len(), 64);
    assert!(
        digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    let physical = std::path::Path::new(first.required("physicalRoot"));
    assert!(physical.starts_with(repo.root().join(".worktrees")));
    assert_ne!(physical, repo.root());
    assert!(!physical.exists(), "native context was released");
    assert_eq!(
        std::path::Path::new(first.required("logicalRoot")),
        repo.root()
    );
    assert_eq!(first.required("filesystemBackend"), "bwrap");
    let common = std::fs::canonicalize(
        repo.root()
            .join(repo.git(&["rev-parse", "--git-common-dir"])),
    )
    .unwrap();
    let owned = common.join("worktrees").join(physical.file_name().unwrap());
    assert!(!owned.exists(), "owned administration was released");
    let exposed_owned = repo.root().join(".git");
    let exposed_common = exposed_owned.join("raf-shared-common");
    let exposed_index = exposed_owned.join("index");
    let mut values = original
        .iter()
        .map(|(key, value)| Dynamic {
            subject: None,
            key,
            value: value.clone(),
        })
        .collect::<Vec<_>>();
    for (key, value) in [
        ("logicalRoot", first.required("logicalRoot")),
        ("physicalRoot", physical.to_str().unwrap()),
        ("viewDigest", digest),
        ("isolatedRoot", repo.root().to_str().unwrap()),
        ("commonGitDirectory", exposed_common.to_str().unwrap()),
        ("ownedGitDirectory", exposed_owned.to_str().unwrap()),
        ("ownedIndexPath", exposed_index.to_str().unwrap()),
    ] {
        values.push(Dynamic {
            subject: None,
            key,
            value: TermValue::simple_literal(value),
        });
    }
    for run in runs {
        assert_eq!(run.required("viewKind"), "materialized");
        assert_eq!(run.required("filesystemBackend"), "bwrap");
        for field in &values {
            let key = field.key;
            let value = &field.value;
            if key == "originalTracked" {
                assert_eq!(
                    TermValue::integer(i128::try_from(run.number(key)).unwrap()),
                    *value
                );
            } else {
                assert_eq!(
                    TermValue::simple_literal(run.required(key)),
                    *value,
                    "fixture fact {key}"
                );
            }
        }
    }
    values
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
    let reference = ready.git(&["symbolic-ref", "refs/remotes/origin/HEAD"]);
    let tracked = ready
        .git(&["ls-files", "-z"])
        .split('\0')
        .filter(|path| !path.is_empty())
        .count();
    let original_admin = ready.git(&["rev-parse", "--absolute-git-dir"]);
    let original_common = std::fs::canonicalize(
        ready
            .root()
            .join(ready.git(&["rev-parse", "--git-common-dir"])),
    )
    .unwrap();
    let original_index_path =
        std::fs::canonicalize(
            ready
                .root()
                .join(ready.git(&["rev-parse", "--git-path", "index"])),
        )
        .unwrap();
    let original = [
        (
            "originalGitDirectory",
            TermValue::simple_literal(original_admin),
        ),
        (
            "originalCommonDirectory",
            TermValue::simple_literal(original_common.to_str().unwrap()),
        ),
        (
            "originalIndexPath",
            TermValue::simple_literal(original_index_path.to_str().unwrap()),
        ),
        (
            "originalIndex",
            TermValue::simple_literal(ready.git(&["write-tree"])),
        ),
        (
            "originalHead",
            TermValue::simple_literal(ready.git(&["rev-parse", "HEAD"])),
        ),
        ("defaultReference", TermValue::simple_literal(&reference)),
        (
            "defaultCommit",
            TermValue::simple_literal(ready.git(&["rev-parse", &reference])),
        ),
        (
            "mergeBase",
            TermValue::simple_literal(ready.git(&["merge-base", "HEAD", &reference])),
        ),
        (
            "originalTracked",
            TermValue::integer(i128::try_from(tracked).unwrap()),
        ),
    ];
    let run = ready
        .run(&["--format", "turtle", "change", "complete", "shape"])
        .expect(0);
    let dynamic = completion_facts(&ready, &run, &original);
    observe_with("completed", run, found, &dynamic);
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
}

fn history_snapshot(repo: &Repo, found: &mut BTreeSet<String>) {
    let log = repo.git(&["rev-list", "--reverse", "HEAD"]);
    let commits: Vec<_> = log.lines().collect();
    assert_eq!(commits.len(), 5, "the complete signed dated fixture");
    for commit in &commits {
        raf_workplan::exec::Commit::new(commit).unwrap();
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
        "Text for Contract of alpha.\n<!-- raf:claim evidence=peak -->\n",
    );
    observe(
        "support",
        repo.run(&["--format", "turtle", "evidence", "for", "cap:alpha"])
            .expect(0),
        found,
    );
    assessments(&repo, found);
    recorded_binding_outcomes(found);
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

fn recorded_binding_outcomes(found: &mut BTreeSet<String>) {
    for (name, path, ignored) in [
        ("recorded-failed-binding", "demo::tests::fails", false),
        ("recorded-unresolved-binding", "demo::tests::passes", true),
    ] {
        let repo = Repo::real(name);
        repo.put(
            "docs/capabilities/fixture.md",
            &support::capability(
                "fixture",
                "implementing",
                [
                    "[demo::add]",
                    "[demo::tests::passes, demo::tests::fails]",
                    "[]",
                ],
                "[]",
                &[],
            ),
        );
        repo.put(
            "work/changes/shape.md",
            &support::change("shape", "[cap:fixture]", &["cap:fixture#contract"]),
        );
        repo.sync();
        if ignored {
            repo.edit(
                "src/lib.rs",
                "    fn passes()",
                "    #[ignore]\n    fn passes()",
            );
        } else {
            // A process failure retains real stdout without libtest's variable
            // panic PID and duration in this exact RDF representation fixture.
            repo.edit(
                "src/lib.rs",
                "panic!(\"this test fails\");",
                "std::process::exit(1);",
            );
        }
        // Preserve the real test output and status while controlling the
        // build diagnostics that otherwise embed paths and variable durations.
        // Parallel libtest output omits the unfinished test's progress prefix;
        // retain that mode even when the enclosing suite runs serially.
        let cargo = repo.root().join("cargo-without-build-diagnostics");
        support::stubs::script(
            &cargo,
            "RUST_TEST_THREADS=2 exec \"$RAF_REGISTRY_CARGO\" \"$@\" 2>/dev/null",
        );
        let mut command = support::command(
            repo.root(),
            support::Shape::Direct,
            &[
                "--format",
                "turtle",
                "evidence",
                "record",
                "--id",
                "bound",
                "--for",
                "cap:fixture",
                "--by",
                "agent",
                "--from-binding",
                path,
            ],
        );
        command
            .env("CARGO", cargo)
            .env("RAF_REGISTRY_CARGO", support::stubs::real_cargo());
        observe(name, support::finish(&mut command).expect(1), found);
    }
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
