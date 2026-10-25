// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Actual migration and debt commands, with read-only inputs and refusal neighbours.

use raf_census::{Baseline, Census, CensusOptions, DebtRecord, Gate, Ledger, MAX_LEDGER_BYTES};
use raf_rdf::{Builder, Format, TermValue as Term, Triple, vocab};
use raf_source::DiskTree;
use std::{
    fmt::Write as _,
    fs,
    path::Path,
    process::{Command, Output},
};

fn put(graph: &mut Builder, subject: &str, key: &str, object: Term) {
    graph
        .insert(Triple {
            subject: Term::iri(subject),
            predicate: Term::iri(if key == "type" {
                vocab::TYPE.into()
            } else {
                format!("{}{key}", vocab::REPORT)
            }),
            object,
        })
        .unwrap();
}

fn ledger(version: u32) -> String {
    let mut graph = Builder::new();
    put(
        &mut graph,
        "",
        "type",
        Term::iri(format!("{}Ledger", vocab::REPORT)),
    );
    for (key, value) in [
        ("normalForm", version),
        ("reportForm", raf_census::REPORT_FORM),
    ] {
        put(
            &mut graph,
            "",
            key,
            Term::typed_literal(value.to_string(), vocab::INTEGER),
        );
    }
    put(&mut graph, "", "job", Term::iri("#job"));
    put(
        &mut graph,
        "#job",
        "type",
        Term::iri(format!("{}LedgerJob", vocab::REPORT)),
    );
    for (key, value) in [
        ("jobId", "number"),
        ("summary", "The approved constant"),
        ("homeSymbol", "demo::home"),
    ] {
        put(&mut graph, "#job", key, Term::simple_literal(value));
    }
    put(&mut graph, "#job", "enforced", Term::boolean(false));
    put(
        &mut graph,
        "#job",
        "forbiddenConstant",
        Term::typed_literal("7", vocab::INTEGER),
    );
    graph.finish().unwrap().to_text(Format::Turtle).unwrap()
}

fn fixture() -> purrdf_testkit::TempDir {
    let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    fs::create_dir(scratch.path().join("src")).unwrap();
    fs::write(
        scratch.path().join("Cargo.toml"),
        "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n[workspace]\n",
    )
    .unwrap();
    source(scratch.path(), 2);
    fs::write(scratch.path().join("helpers-ledger.ttl"), ledger(1)).unwrap();
    scratch
}

fn source(root: &Path, copies: usize) {
    let mut text = "/// The operation.\npub fn home()->u32{7}\n".to_owned();
    for index in 0..copies {
        writeln!(text, "fn copy_{index}()->u32{{7}}").unwrap();
    }
    fs::write(root.join("src/lib.rs"), text).unwrap();
}

fn run(root: &Path, words: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cargo-census"))
        .current_dir(root)
        .args(words)
        .output()
        .unwrap()
}

#[test]
fn debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate() {
    let scratch = fixture();
    let root = scratch.path();
    let first = run(root, &["census", "debt", "--write"]);
    assert!(first.status.success(), "{first:?}");
    let path = root.join("census-debt.ttl");
    let before = fs::read_to_string(&path).unwrap();
    let policy = DebtRecord::from_turtle(&before).unwrap();
    assert_eq!(policy.job_copies()["number"], 2);
    Gate::workspace(root).write_debt().unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    source(root, 3);
    let growth = run(root, &["debt", "--write"]);
    assert_eq!(growth.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&growth.stderr).contains("growth cannot be recorded"));
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    source(root, 1);
    let shrink = run(root, &["debt", "--write"]);
    assert!(shrink.status.success(), "{shrink:?}");
    assert!(Gate::workspace(root).check().unwrap().is_clean());
    let lower = fs::read_to_string(&path).unwrap();
    assert_eq!(
        DebtRecord::from_turtle(&lower).unwrap().job_copies()["number"],
        1
    );
    fs::write(
        root.join("helpers-ledger.ttl"),
        ledger(1).replace("demo::home", "demo::gone"),
    )
    .unwrap();
    let stale = run(root, &["debt", "--write"]);
    assert_eq!(stale.status.code(), Some(1));
    assert_eq!(fs::read_to_string(path).unwrap(), lower);
}

#[test]
fn migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy() {
    let scratch = fixture();
    let root = scratch.path();
    let old = ledger(0);
    fs::write(root.join("old.ttl"), &old).unwrap();
    let tree = DiskTree::new(root).unwrap();
    let census = Census::load(&tree, &["."], &CensusOptions::default()).unwrap();
    let mut index = census.index(&[]).unwrap();
    index.normal_form = 0;
    let previous = index.to_turtle().unwrap();
    fs::write(root.join("old-index.ttl"), &previous).unwrap();
    let result = run(
        root,
        &["migrate", "--ledger", "old.ttl", "--index", "old-index.ttl"],
    );
    assert!(result.status.success(), "{result:?}");
    let text = purrdf_testkit::text::stdout_utf8(&result);
    assert_eq!(
        Ledger::from_turtle(&text).unwrap().jobs()[0].job.id,
        "number"
    );
    assert_eq!(fs::read_to_string(root.join("old.ttl")).unwrap(), old);
    assert_eq!(
        fs::read_to_string(root.join("old-index.ttl")).unwrap(),
        previous
    );
    let foreign = "[census]\n[[job]]\nid='number'\nsummary='The approved constant'\nhome='demo::home'\nentry_points=[]\nenforced=true\n";
    let baseline = r#"{"min_tokens":20,"units":3,"body_groups":[],"shim_groups":[],"hex_tables":[],"shared_constants":[]}"#;
    fs::write(root.join("old.toml"), foreign).unwrap();
    fs::write(root.join("old.json"), baseline).unwrap();
    let words = [
        "migrate",
        "--legacy",
        "--ledger",
        "old.toml",
        "--baseline",
        "old.json",
    ];
    let result = run(root, &words);
    assert!(result.status.success(), "{result:?}");
    Ledger::from_turtle(&purrdf_testkit::text::stdout_utf8(&result)).unwrap();
    assert_eq!(fs::read_to_string(root.join("old.toml")).unwrap(), foreign);
    assert_eq!(fs::read_to_string(root.join("old.json")).unwrap(), baseline);
    fs::write(
        root.join("old.toml"),
        format!("{foreign}unrecognized='original'\n"),
    )
    .unwrap();
    let unmapped = run(root, &words);
    assert_eq!(unmapped.status.code(), Some(1));
    assert!(purrdf_testkit::text::stdout_utf8(&unmapped).contains("# unmapped:"));
}

#[test]
fn command_usage_and_bounded_input_failures_do_not_publish_a_record() {
    baseline_discovery();
    let scratch = fixture();
    let root = scratch.path();
    for words in [
        vec!["debt"],
        vec!["migrate", "--legacy", "--ledger", "missing"],
        vec!["migrate", "--ledger", "missing"],
        vec!["check"],
        vec!["baseline", "--format", "json"],
        vec!["baseline", "--out"],
    ] {
        assert_eq!(run(root, &words).status.code(), Some(2), "{words:?}");
    }
    for words in [vec!["--help"], vec!["census", "--version"]] {
        assert!(run(root, &words).status.success());
    }
    fs::write(root.join("large.toml"), " ".repeat(MAX_LEDGER_BYTES + 1)).unwrap();
    let result = run(
        root,
        &[
            "migrate",
            "--legacy",
            "--ledger",
            "large.toml",
            "--baseline",
            "missing",
        ],
    );
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("large.toml"));
    assert_eq!(result.stdout, [] as [u8; 0]);
    assert!(!root.join("census-debt.ttl").exists());
}

fn baseline_discovery() {
    let scratch = fixture();
    let root = scratch.path();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn first(x:u32)->u32{let a=x.wrapping_add(1);let b=a.wrapping_mul(3);b.wrapping_sub(2)}\n\
         pub fn second(y:u32)->u32{let c=y.wrapping_add(1);let d=c.wrapping_mul(3);d.wrapping_sub(2)}\n",
    )
    .unwrap();
    // Discovery never treats a missing or malformed policy as a clean verdict.
    fs::write(root.join("helpers-ledger.ttl"), "not a ledger").unwrap();
    let paths = ["Cargo.toml", "src/lib.rs", "helpers-ledger.ttl"];
    let before = paths.map(|path| fs::read(root.join(path)).unwrap());
    let native = Census::load(
        &DiskTree::new(root).unwrap(),
        &["."],
        &CensusOptions::default(),
    )
    .unwrap()
    .baseline();
    assert_ne!(native.body_groups, [] as [raf_census::BaselineGroup; 0]);
    let result = run(root, &["baseline"]);
    assert!(result.status.success(), "{result:?}");
    assert_eq!(result.stdout, native.to_ntriples().unwrap().as_bytes());
    assert_eq!(run(root, &["baseline"]).stdout, result.stdout);
    assert_eq!(
        Baseline::from_ntriples(std::str::from_utf8(&result.stdout).unwrap()).unwrap(),
        native
    );
    let caller = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let turtle = Command::new(env!("CARGO_BIN_EXE_cargo-census"))
        .current_dir(caller.path())
        .args(["--workspace"])
        .arg(root)
        .args(["baseline", "--format", "turtle", "--out", "report.ttl"])
        .output()
        .unwrap();
    assert!(turtle.status.success(), "{turtle:?}");
    assert_eq!(turtle.stdout, [] as [u8; 0]);
    let text = fs::read_to_string(caller.path().join("report.ttl")).unwrap();
    assert_eq!(text, native.to_turtle().unwrap());
    assert_eq!(Baseline::from_turtle(&text).unwrap(), native);
    assert_eq!(paths.map(|path| fs::read(root.join(path)).unwrap()), before);
    assert!(!root.join("census-debt.ttl").exists());
    baseline_refusals(root, caller.path());
}

fn baseline_refusals(root: &Path, caller: &Path) {
    let destination = caller.join("report.ttl");
    let manifest = fs::read(root.join("Cargo.toml")).unwrap();
    fs::write(root.join("Cargo.toml"), "not a manifest [").unwrap();
    for present in [true, false] {
        if present {
            fs::write(&destination, "preserve prior output").unwrap();
        } else {
            fs::remove_file(&destination).unwrap();
        }
        let result = Command::new(env!("CARGO_BIN_EXE_cargo-census"))
            .current_dir(root)
            .arg("baseline")
            .arg("--out")
            .arg(&destination)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1), "{result:?}");
        assert_eq!(result.stdout, [] as [u8; 0]);
        assert!(String::from_utf8_lossy(&result.stderr).contains("Cargo.toml"));
        if present {
            assert_eq!(fs::read(&destination).unwrap(), b"preserve prior output");
        } else {
            assert!(!destination.exists());
        }
    }
    fs::write(root.join("Cargo.toml"), manifest).unwrap();
    let result = run(root, &["baseline", "--out", "."]);
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert_eq!(result.stdout, [] as [u8; 0]);
    assert_ne!(result.stderr, [] as [u8; 0]);
    assert!(run(root, &["baseline"]).status.success());
}

#[test]
fn registered_code_is_required_explicitly_without_recording_partial_debt() {
    let scratch = fixture();
    let root = scratch.path();
    for kind in ["rule", "extension", "variant"] {
        let policy = Ledger::from_turtle(&ledger(1)).unwrap();
        let mut graph = Builder::from_graph(policy.graph());
        if kind == "extension" {
            put(&mut graph, "#job", "extension", Term::iri("#extension"));
            put(
                &mut graph,
                "#extension",
                "type",
                Term::iri(format!("{}Extension", vocab::REPORT)),
            );
            graph
                .insert(Triple {
                    subject: Term::iri("#extension"),
                    predicate: Term::iri("urn:example:owner"),
                    object: Term::simple_literal("runtime"),
                })
                .unwrap();
        } else if kind == "variant" {
            put(&mut graph, "#job", "variant", Term::iri("#variant"));
            put(
                &mut graph,
                "#variant",
                "type",
                Term::iri(format!("{}Variant", vocab::REPORT)),
            );
            for (key, value) in [
                ("symbol", "demo::copy_0"),
                ("file", "src/lib.rs"),
                ("detector", "custom-check"),
                ("criterion", "layering"),
                ("reason", "Independent fixture implementation"),
            ] {
                put(&mut graph, "#variant", key, Term::simple_literal(value));
            }
            put(
                &mut graph,
                "#variant",
                "anchor",
                Term::iri("urn:raf:code:demo::home"),
            );
        } else {
            put(
                &mut graph,
                "#job",
                "forbiddenFingerprint",
                Term::simple_literal("rule:custom-check"),
            );
        }
        fs::write(
            root.join("helpers-ledger.ttl"),
            graph.finish().unwrap().to_text(Format::Turtle).unwrap(),
        )
        .unwrap();
        let result = run(root, &["debt", "--write"]);
        assert_eq!(result.status.code(), Some(2), "{result:?}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("Gate::write_debt"));
        assert!(!root.join("census-debt.ttl").exists());
    }
}
