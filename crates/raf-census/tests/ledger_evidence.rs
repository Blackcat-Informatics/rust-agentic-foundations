// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A job's test-vector and bench links are checked against the workspace.

use raf_census::{Gate, Ledger, NORMAL_FORM, REPORT_FORM};
use raf_rdf::{Builder, Format, Graph, TermValue as Term, Triple, vocab};
use raf_source::{DiskTree, MemoryTree};

#[path = "support/reference_ledger.rs"]
mod reference_ledger;

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[[bench]]\nname = \"speed\"\nharness = false\n";
const PLAIN_MANIFEST: &str =
    "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
const SOURCE: &str = "/// The one implementation.\npub fn home() -> u32 { 7 }\n";

fn put(graph: &mut Builder, node: &str, key: &str, object: Term) {
    let subject = Term::iri(node);
    if key == "type" {
        graph
            .insert(Triple {
                subject,
                predicate: Term::iri(vocab::TYPE),
                object,
            })
            .unwrap();
        return;
    }
    let predicate = Term::iri(format!("{}{key}", vocab::REPORT));
    let inserted = graph.insert(Triple {
        subject,
        predicate,
        object,
    });
    assert!(inserted.is_ok(), "{key}: {inserted:?}");
}

/// A ledger with one enforced job homed at `demo::home` carrying the links.
fn ledger(vectors: &[&str], benches: &[&str]) -> Builder {
    let mut graph = Builder::new();
    let iri = |name: &str| Term::iri(format!("{}{name}", vocab::REPORT));
    put(&mut graph, "", "type", iri("Ledger"));
    put(
        &mut graph,
        "",
        "normalForm",
        Term::integer(i128::from(NORMAL_FORM)),
    );
    put(
        &mut graph,
        "",
        "reportForm",
        Term::integer(i128::from(REPORT_FORM)),
    );
    put(&mut graph, "", "job", Term::iri("#job"));
    put(&mut graph, "#job", "type", iri("LedgerJob"));
    for (key, value) in [
        ("jobId", "number"),
        ("summary", "The approved constant"),
        ("homeSymbol", "demo::home"),
    ] {
        put(&mut graph, "#job", key, Term::simple_literal(value));
    }
    put(&mut graph, "#job", "enforced", Term::boolean(true));
    for vector in vectors {
        put(
            &mut graph,
            "#job",
            "vectorPath",
            Term::simple_literal(*vector),
        );
    }
    for bench in benches {
        put(
            &mut graph,
            "#job",
            "benchTarget",
            Term::simple_literal(*bench),
        );
    }
    graph
}

fn text(graph: Builder) -> String {
    graph.finish().unwrap().to_text(Format::Turtle).unwrap()
}

fn tree(manifest: &str, files: &[(&str, &str)], ledger: Builder) -> MemoryTree {
    let mut all = vec![
        ("Cargo.toml", manifest.to_owned()),
        ("src/lib.rs", SOURCE.to_owned()),
        ("helpers-ledger.ttl", text(ledger)),
    ];
    all.extend(files.iter().map(|(path, body)| (*path, (*body).to_owned())));
    all.into_iter().collect()
}

const LINKED: [(&str, &str); 2] = [
    ("vectors/add.json", "[]"),
    ("benches/speed.rs", "fn main() {}"),
];

#[test]
fn a_job_naming_an_existing_vector_and_bench_is_clean() {
    let report = Gate::workspace(".")
        .check_tree(&tree(
            MANIFEST,
            &LINKED,
            ledger(&["vectors/add.json"], &["demo:speed"]),
        ))
        .unwrap();
    assert!(report.is_clean(), "{:?}", report.findings());
}

#[test]
fn a_renamed_vector_or_removed_bench_is_a_finding_naming_the_job_and_link() {
    let renamed = Gate::workspace(".")
        .check_tree(&tree(
            MANIFEST,
            &[
                ("vectors/sum.json", "[]"),
                ("benches/speed.rs", "fn main() {}"),
            ],
            ledger(&["vectors/add.json"], &["demo:speed"]),
        ))
        .unwrap();
    assert_eq!(renamed.findings().len(), 1, "{:?}", renamed.findings());
    let finding = &renamed.findings()[0];
    assert!(
        finding.contains("\"number\"") && finding.contains("vectors/add.json"),
        "{finding}"
    );
    let removed = Gate::workspace(".")
        .check_tree(&tree(
            PLAIN_MANIFEST,
            &[("vectors/add.json", "[]")],
            ledger(&["vectors/add.json"], &["demo:speed"]),
        ))
        .unwrap();
    assert_eq!(removed.findings().len(), 1, "{:?}", removed.findings());
    let finding = &removed.findings()[0];
    assert!(
        finding.contains("\"number\"") && finding.contains("demo:speed"),
        "{finding}"
    );
    let directory = Gate::workspace(".")
        .check_tree(&tree(
            MANIFEST,
            &LINKED,
            ledger(&["vectors"], &["demo:speed"]),
        ))
        .unwrap();
    assert_eq!(directory.findings().len(), 1, "{:?}", directory.findings());
    let finding = &directory.findings()[0];
    assert!(
        finding.contains("\"number\"")
            && finding.contains("\"vectors\"")
            && finding.contains("not a regular file"),
        "{finding}"
    );
    let other_package = Gate::workspace(".")
        .check_tree(&tree(
            MANIFEST,
            &LINKED,
            ledger(&["vectors/add.json"], &["other:speed"]),
        ))
        .unwrap();
    assert!(
        other_package.findings()[0].contains("other:speed"),
        "{:?}",
        other_package.findings()
    );
}

#[test]
fn a_link_that_escapes_the_workspace_root_is_refused() {
    for (escaping, cause) in [
        ("../outside.json", "climbs above the root"),
        ("/etc/passwd", "is absolute"),
        ("vectors/../../outside.json", "climbs above the root"),
        ("vectors\\add.json", "holds a backslash"),
    ] {
        let error = Ledger::from_turtle(&text(ledger(&[escaping], &[])))
            .unwrap_err()
            .to_string();
        assert!(error.contains("vectorPath"), "{escaping}: {error}");
        assert!(error.contains(cause), "{escaping}: {error}");
    }
    Ledger::from_turtle(&text(ledger(&["vectors/add.json"], &["demo:speed"]))).unwrap();
    for malformed in ["speed", "demo:", ":speed", "a:b:c"] {
        let error = Ledger::from_turtle(&text(ledger(&[], &[malformed])))
            .unwrap_err()
            .to_string();
        assert!(error.contains("benchTarget"), "{malformed}: {error}");
    }
}

#[cfg(unix)]
#[test]
fn a_vector_symlink_leading_out_of_the_root_is_refused_beside_an_inside_link() {
    let scratch = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let root = scratch.path().join("workspace");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join("vectors")).unwrap();
    std::fs::write(scratch.path().join("outside.json"), "[]").unwrap();
    std::fs::write(root.join("vectors/inside.json"), "[]").unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        format!("{PLAIN_MANIFEST}[workspace]\n"),
    )
    .unwrap();
    std::fs::write(root.join("src/lib.rs"), SOURCE).unwrap();
    let check = |vector: &str| {
        std::fs::write(
            root.join("helpers-ledger.ttl"),
            text(ledger(&[vector], &[])),
        )
        .unwrap();
        Gate::workspace(".").check_tree(&DiskTree::new(&root).unwrap())
    };
    assert!(check("vectors/inside.json").unwrap().is_clean());
    std::os::unix::fs::symlink("../../outside.json", root.join("vectors/out.json")).unwrap();
    let error = check("vectors/out.json").unwrap_err().to_string();
    assert!(
        error.contains("vectors/out.json") && error.contains("leads outside the root"),
        "{error}"
    );
}

#[test]
fn a_job_homed_in_a_reference_dependency_is_refused_when_it_declares_links() {
    let files = |ledger: String| {
        MemoryTree::from_iter([
            ("Cargo.toml", "[workspace]\nmembers=['app','helper']\n[workspace.package]\nversion='1.0.0'\n[workspace.dependencies]\nrenamed={package='helper',path='helper',version='1'}\n".to_owned()),
            ("app/Cargo.toml", "[package]\nname='app'\nversion.workspace=true\nedition='2024'\n[dependencies]\nrenamed.workspace=true\n".to_owned()),
            ("app/src/lib.rs", String::new()),
            ("helper/Cargo.toml", "[package]\nname='helper'\nversion.workspace=true\nedition='2024'\n".to_owned()),
            ("helper/src/lib.rs", "pub mod api { pub fn first() -> u32 { 1 } }".to_owned()),
            ("vectors/add.json", "[]".to_owned()),
            ("Cargo.lock", "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['helper']\n[[package]]\nname='helper'\nversion='1.0.0'\n".to_owned()),
            ("helpers-ledger.ttl", ledger),
        ])
    };
    let plain = reference_ledger::ledger("helper", "helper::api", "arithmetic");
    Gate::workspace(".")
        .check_tree(&files(plain.clone()))
        .expect("the same reference job without links is evaluated");
    let mut linked = Builder::from_graph(&Graph::from_text(&plain, Format::Turtle).unwrap());
    put(
        &mut linked,
        "#job",
        "vectorPath",
        Term::simple_literal("vectors/add.json"),
    );
    let error = Gate::workspace(".")
        .check_tree(&files(text(linked)))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("arithmetic") && error.contains("reference dependency"),
        "{error}"
    );
}
