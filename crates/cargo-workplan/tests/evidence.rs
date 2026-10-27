// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `evidence record`, `evidence claims` and `evidence for` through the built
//! binary: a command after `--` run with no shell and judged by its exit
//! status and its `name=number` lines, exactly one record written, the exit 1
//! of a `fail` record and the exit 2 of a run that writes none, the options
//! after `--` left to the command, a bound test as the source of the verdict,
//! typed RDF results, a real `cargo workplan` dispatch, and a claim that
//! reconciles clean until its bound code moves.

#![cfg(unix)]

#[cfg(target_os = "linux")]
#[path = "evidence/supervisor.rs"]
mod supervisor;
mod support;
use support::rdf::Rdf;

use std::process::Command;

use support::stubs::{Stubs, real_cargo, script, serial};
use support::{COPYRIGHT, LICENCE, Repo, Scratch, Shape, capability, change, changed};

const RECORD: &str = "work/evidence/peak.md";

/// `evidence record` of the record `peak` for `alpha` by an agent with the
/// limit `peak=64`, then `tail`.
fn record<'a>(tail: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
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
    ];
    args.extend_from_slice(tail);
    args
}

fn semantic_claims(repo: &Repo, id: &str, format: &str) -> std::sync::Arc<purrdf_rdf::RdfDataset> {
    use purrdf_rdf::{
        NativeRdfFormat, ParseOptions, SerializeGraph, SerializeOptions, StatementLayer,
        parse_dataset_with, serialize_dataset_with,
    };
    let before = repo.snapshot();
    let args = ["--format", format, "evidence", "claims", id];
    let run = repo.both(&args).expect(0);
    assert_eq!(run.stderr, "");
    let (native, media) = if format == "turtle" {
        (NativeRdfFormat::Turtle, "text/turtle")
    } else {
        (NativeRdfFormat::NTriples, "application/n-triples")
    };
    let parsed =
        parse_dataset_with(run.stdout.as_bytes(), media, None, &ParseOptions::default()).unwrap();
    let opened = raf_workplan::Gate::workspace(repo.root())
        .programs(raf_workplan::exec::Programs::new(repo.root()).discover_git())
        .open()
        .unwrap();
    let direct = opened.workplan().evidence_claims_dataset(id).unwrap();
    let serialized = serialize_dataset_with(
        direct.as_ref(),
        native,
        None,
        &SerializeOptions {
            selection: SerializeGraph::Dataset,
            statement_layer: StatementLayer::Emit,
            jsonld_options: None,
        },
    )
    .unwrap();
    assert_eq!(run.stdout.as_bytes(), serialized.bytes);
    assert_eq!(
        repo.both(&args).expect(0).stdout,
        run.stdout,
        "deterministic replay"
    );
    assert_eq!(
        repo.snapshot(),
        before,
        "semantic query changed repository inputs"
    );
    parsed.dataset
}

#[test]
fn a_command_that_passes_writes_exactly_one_record_and_exits_zero() {
    let repo = Repo::synced("evidence-pass", false);
    let before = repo.snapshot();
    let run = repo
        .both_writing(&record(&[
            "--",
            "/bin/sh",
            "-c",
            "echo peak=41; printf 'kept%s\\n' ' nowhere'; printf 'err%s\\n' 'or text' >&2",
        ]))
        .expect(0);
    assert_eq!(run.stdout, format!("recorded: {RECORD} (pass)\n"));
    assert_eq!(run.stderr, "");
    assert_eq!(changed(&before, &repo.snapshot()), [RECORD]);
    let text = repo.read(RECORD);
    for line in [
        "id: evidence:peak",
        "subject: command",
        "verdict: pass",
        "by: agent",
        "for: [cap:alpha]",
        "command: /bin/sh -c 'echo peak=41; printf '\\''kept%s\\n'\\'' '\\'' nowhere'\\''; printf '\\''err%s\\n'\\'' '\\''or text'\\'' >&2'",
    ] {
        assert!(text.contains(line), "{line}\n{text}");
    }
    assert!(
        text.contains("peak: 41") && text.contains("peak: 64"),
        "{text}"
    );
    // No other output is kept.
    assert!(
        !text.contains("kept nowhere") && !text.contains("error text"),
        "{text}"
    );
    // The record loads: the repository stays clean.
    repo.both(&["check"]).expect(0);
    // No claim names it yet: an empty answer, its note naming the id.
    let none = repo.both(&["evidence", "claims", "peak"]).expect(0);
    assert_eq!(none.stdout, "");
    assert!(
        none.stderr
            .contains("no claim of a capability names evidence:peak"),
        "{none:#?}"
    );

    // Under `-q` a passing record is written and nothing is printed.
    let quiet = Repo::synced("evidence-pass-quiet", false);
    let run = quiet
        .both_writing(&{
            let mut args = vec!["-q"];
            args.extend(record(&["--", "/bin/sh", "-c", "echo peak=41"]));
            args
        })
        .expect(0);
    assert_eq!(run.stdout, "");
    assert!(quiet.root().join(RECORD).is_file());
}

#[test]
fn stored_command_text_is_never_executed_by_checks_or_dependency_views() {
    use raf_okf::{Actor, EvidenceRecord, EvidenceSubject, EvidenceVerdict, Input, InputKind};
    use raf_workplan::Gate;
    use raf_workplan::digest::Digest;
    use raf_workplan::record::{Record, RecordOptions};

    let repo = Repo::synced("stored-command-text", false);
    let opened = Gate::workspace(repo.root())
        .programs(raf_workplan::exec::Programs::new(repo.root()).discover_git())
        .open()
        .expect("the real fixture workspace");
    let digest = Digest::symbol(opened.workspace(), "demo::real").expect("the actual code input");
    let mut record = EvidenceRecord::new(
        "stored",
        EvidenceSubject::Command,
        vec![Input::new(
            InputKind::Symbol,
            "demo::real",
            digest.to_string(),
        )],
        EvidenceVerdict::Fail,
        Actor::Agent,
        vec!["cap:alpha".to_string()],
    );
    record.command = Some(
        "sh -c 'printf ran > stored-command-ran'; $(printf ran > substituted-command-ran)"
            .to_string(),
    );
    repo.put(
        "work/evidence/stored.md",
        &Record::Evidence(record)
            .text(&RecordOptions::new())
            .unwrap(),
    );
    let before = repo.snapshot();
    for args in [
        vec!["check"],
        vec!["evidence", "for", "cap:alpha"],
        vec!["evidence", "claims", "stored"],
    ] {
        repo.both(&args).expect(0);
        assert_eq!(repo.snapshot(), before, "{args:?}");
        assert!(!repo.root().join("stored-command-ran").exists());
        assert!(!repo.root().join("substituted-command-ran").exists());
    }
}

#[test]
fn the_value_limit_exits_two_at_1025_and_records_all_1024_beside_it() {
    const MODE: &str = "RAF_EVIDENCE_VALUE_BOUNDARY";
    const NAME: &str = "the_value_limit_exits_two_at_1025_and_records_all_1024_beside_it";
    if let Ok(mode) = std::env::var(MODE) {
        let count = match mode.as_str() {
            "at" => 1024,
            "over" => 1025,
            other => panic!("unrecognized value fixture: {other:?}"),
        };
        println!();
        for number in 0..count {
            println!("value{number}=1");
        }
        return;
    }
    let executable = std::env::current_exe().unwrap();
    for mode in ["over", "at"] {
        for shape in [Shape::Direct, Shape::Cargo] {
            let repo = Repo::synced("value-count-boundary", false);
            let before = repo.snapshot();
            let args = [
                "evidence",
                "record",
                "--id",
                "peak",
                "--for",
                "cap:alpha",
                "--by",
                "agent",
                "--",
                executable.to_str().unwrap(),
                "--exact",
                NAME,
                "--nocapture",
            ];
            let run = support::finish(support::command(repo.root(), shape, &args).env(MODE, mode))
                .expect(if mode == "at" { 0 } else { 2 });
            if mode == "over" {
                assert_eq!(run.stdout, "");
                assert!(run.stderr.contains("1024"), "{run:#?}");
                assert_eq!(repo.snapshot(), before);
            } else {
                assert_eq!(changed(&before, &repo.snapshot()), [RECORD]);
                let opened = raf_workplan::Gate::workspace(repo.root())
                    .programs(raf_workplan::exec::Programs::new(repo.root()).discover_git())
                    .open()
                    .unwrap();
                let plan = opened.workplan();
                let [record] = plan.evidence() else {
                    panic!("one recorded execution")
                };
                assert_eq!(record.value().values.len(), 1024);
                assert_eq!(record.value().verdict, raf_okf::EvidenceVerdict::Pass);
                assert!(
                    record
                        .value()
                        .values
                        .iter()
                        .all(|value| value.value.text() == "1")
                );
            }
        }
    }
}

#[test]
fn a_failing_command_or_a_value_over_its_limit_records_fail_and_exits_one() {
    let repo = Repo::synced("evidence-fail", false);
    let before = repo.snapshot();
    let run = repo
        .both_writing(&record(&["--", "/bin/sh", "-c", "echo peak=12; exit 3"]))
        .expect(1);
    assert_eq!(
        run.stdout,
        format!("recorded: {RECORD} (fail): the command exited with 3\n")
    );
    assert_eq!(changed(&before, &repo.snapshot()), [RECORD]);
    assert!(repo.read(RECORD).contains("verdict: fail"));

    // A value over its limit replaces the record with `--update` and is
    // `fail` too; `-q` keeps the line, since a failing record is a finding.
    let replaced = repo.snapshot();
    let run = repo
        .both_writing(&{
            let mut args = vec!["-q"];
            args.extend(record(&["--update", "--", "/bin/sh", "-c", "echo peak=80"]));
            args
        })
        .expect(1);
    assert_eq!(
        run.stdout,
        format!("recorded: {RECORD} (fail): over the limit: peak\n")
    );
    assert_eq!(changed(&replaced, &repo.snapshot()), [RECORD]);
    let text = repo.read(RECORD);
    assert!(
        text.contains("peak: 80") && text.contains("verdict: fail"),
        "{text}"
    );

    // The neighbour: within its limit the replacement passes.
    repo.both_writing(&record(&[
        "--update",
        "--",
        "/bin/sh",
        "-c",
        "echo peak=64",
    ]))
    .expect(0);
    assert!(repo.read(RECORD).contains("verdict: pass"));
}

#[test]
fn a_replacement_holds_the_spdx_lines_of_its_own_run_only() {
    let repo = Repo::synced("evidence-spdx", false);
    let spdx = format!(
        "---\n# SPDX-FileCopyrightText: {COPYRIGHT}\n# SPDX-License-Identifier: {LICENCE}\ntype: Evidence\n"
    );
    let echo = ["--", "/bin/sh", "-c", "echo peak=1"];
    let pair = ["--copyright", COPYRIGHT, "--licence", LICENCE];
    repo.both_writing(&record(&[&pair[..], &echo[..]].concat()))
        .expect(0);
    assert!(
        repo.read(RECORD).starts_with(&spdx),
        "{}",
        repo.read(RECORD)
    );
    // `--update` without the pair writes a record without SPDX lines: the
    // replacement holds only what its run gives.
    repo.both_writing(&record(&[&["--update"][..], &echo[..]].concat()))
        .expect(0);
    assert!(
        repo.read(RECORD).starts_with("---\ntype: Evidence\n"),
        "{}",
        repo.read(RECORD)
    );
    // With the pair again, the lines are back.
    repo.both_writing(&record(&[&["--update"][..], &pair[..], &echo[..]].concat()))
        .expect(0);
    assert!(
        repo.read(RECORD).starts_with(&spdx),
        "{}",
        repo.read(RECORD)
    );
}

#[test]
fn output_the_tool_cannot_read_is_exit_two_and_writes_no_record() {
    let repo = Repo::synced("evidence-unreadable", false);
    let before = repo.snapshot();
    for (limit, script) in [
        ("peak=64", "echo peak=1e-3"),
        ("peak=64", "echo peak=1; echo peak=2"),
        ("other=5", "echo peak=1"),
        ("peak=64", "head -c 17000000 /dev/zero; echo peak=1"),
    ] {
        let mut args = vec![
            "evidence",
            "record",
            "--id",
            "peak",
            "--for",
            "cap:alpha",
            "--by",
            "human",
            "--limit",
            limit,
            "--",
            "/bin/sh",
            "-c",
            script,
        ];
        // The kill for too much output gives the time the command ran, which
        // differs between the two shapes.
        let run = repo.both_timed(&args).expect(2);
        assert_eq!(run.stdout, "", "{script}");
        assert!(
            run.stderr.contains("no record was written")
                || run.stderr.contains("nothing was recorded"),
            "{script}: {run:#?}"
        );
        if script.starts_with("head") {
            assert!(
                run.stderr.contains(
                    "wrote more than 16777216 bytes to standard output in <t> s and was killed"
                ),
                "{run:#?}"
            );
        }
        assert_eq!(repo.snapshot(), before, "{script}");
        args.insert(0, "--format=json");
        assert_eq!(repo.both_timed(&args).expect(2).stdout, "", "{script}");
    }
    // A program that cannot be started.
    let run = repo
        .both(&record(&["--", "/no/such/program", "x"]))
        .expect(2);
    assert!(run.stderr.contains("/no/such/program"), "{run:#?}");
    assert_eq!(repo.snapshot(), before);
    // The neighbour: the same limit and one number within it.
    repo.both_writing(&record(&["--", "/bin/sh", "-c", "echo peak=1"]))
        .expect(0);
}

#[test]
fn a_request_that_cannot_be_recorded_runs_nothing() {
    let repo = Repo::synced("evidence-refused", false);
    let marker = "ran.txt";
    let touch = ["--", "/bin/sh", "-c", "touch ran.txt; echo peak=1"];
    let head = |covers: &'static str| {
        vec![
            "evidence", "record", "--id", "peak", "--for", covers, "--by", "agent",
        ]
    };
    let before = repo.snapshot();
    for (args, says) in [
        // No such capability, and one that binds no code.
        (
            [head("cap:nowhere"), touch.to_vec()].concat(),
            "cap:nowhere",
        ),
        (
            [head("cap:gamma"), touch.to_vec()].concat(),
            "binds no code",
        ),
        // A replacement of a record that is not there.
        (
            [head("cap:alpha"), vec!["--update"], touch.to_vec()].concat(),
            RECORD,
        ),
        // A limit with a bound test as the source.
        (
            [
                head("cap:alpha"),
                vec![
                    "--limit",
                    "peak=1",
                    "--from-binding",
                    "demo::test_crate::it::case",
                ],
            ]
            .concat(),
            "limit",
        ),
        // A path that is no test or example of the capability.
        (
            [head("cap:alpha"), vec!["--from-binding", "demo::nowhere"]].concat(),
            "demo::test_crate::it::case",
        ),
    ] {
        let run = repo.both(&args).expect(2);
        assert!(run.stderr.contains(says), "{args:?}: {run:#?}");
        assert!(
            !repo.root().join(marker).exists(),
            "{args:?} ran the command"
        );
        assert_eq!(repo.snapshot(), before, "{args:?}");
    }
    // A record that is there is never replaced without `--update`.
    repo.both_writing(&record(&["--", "/bin/sh", "-c", "echo peak=1"]))
        .expect(0);
    let kept = repo.snapshot();
    let run = repo
        .both(&[head("cap:alpha"), touch.to_vec()].concat())
        .expect(2);
    assert!(run.stderr.contains(RECORD), "{run:#?}");
    assert!(!repo.root().join(marker).exists());
    assert_eq!(repo.snapshot(), kept);
}

#[test]
fn the_words_after_the_separator_are_the_command_options_included() {
    let repo = Repo::synced("evidence-trailing", false);
    let run = repo
        .both_writing(&record(&[
            "--",
            "/bin/sh",
            "-c",
            "printf '%s\\n' \"$@\" > argv.txt; echo peak=2",
            "sh",
            "-q",
            "--format=json",
            "--by",
            "human",
            "--",
            "x y",
        ]))
        .expect(0);
    // Neither `-q` nor `--format=json` reached the tool, and `--by human`
    // did not change who recorded it.
    assert_eq!(run.stdout, format!("recorded: {RECORD} (pass)\n"));
    assert!(repo.read(RECORD).contains("by: agent"));
    // The command got them as its own words, one argument each, with no shell
    // splitting them.
    assert_eq!(
        repo.read("argv.txt"),
        "-q\n--format=json\n--by\nhuman\n--\nx y\n"
    );
}

#[test]
fn rdf_is_one_evidence_recorded_object_with_its_fields() {
    let repo = Repo::synced("evidence-json", false);
    let run = repo
        .both_writing(&{
            let mut args = vec!["--format", "turtle"];
            args.extend(record(&[
                "--revision",
                "0123456789abcdef0123456789abcdef01234567",
                "--",
                "/bin/sh",
                "-c",
                "echo peak=80; echo rss=3.5",
            ]));
            args
        })
        .expect(1);
    let rdf = Rdf::of(&run);
    let recorded = rdf.rows("EvidenceRecorded");
    assert_eq!(recorded.len(), 1);
    let row = recorded[0];
    assert_eq!(row.required("path"), RECORD);
    assert_eq!(row.required("id"), "evidence:peak");
    assert_eq!(row.required("verdict"), "fail");
    assert_eq!(row.required("subject"), "command");
    assert_eq!(
        row.required("command"),
        "/bin/sh -c 'echo peak=80; echo rss=3.5'"
    );
    assert_eq!(
        row.required("revision"),
        "0123456789abcdef0123456789abcdef01234567"
    );
    assert_eq!(row.required("by"), "agent");
    assert_eq!(row.children("for")[0].required("id"), "cap:alpha");
    let values = row.children("value");
    assert_eq!(
        values
            .iter()
            .map(|m| (m.required("name"), m.required("decimal")))
            .collect::<Vec<_>>(),
        [("peak", "80"), ("rss", "3.5")]
    );
    let limits = row.children("limit");
    assert_eq!(limits.len(), 1);
    assert_eq!(
        (limits[0].required("name"), limits[0].required("decimal")),
        ("peak", "64")
    );
    let input = row.children("input")[0];
    assert_eq!(input.required("kind"), "symbol");
    assert_eq!(input.required("name"), "demo::real");
    assert_eq!(input.required("digest").len(), 64);
    let failed = row.children("failed");
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].required("reason"), "over-limit");
    assert!(failed[0].text("exitCode").is_none());
    assert_eq!(failed[0].children("measure")[0].required("name"), "peak");
    assert_eq!(
        row.required("message"),
        format!("recorded: {RECORD} (fail): over the limit: peak")
    );
}

/// A workspace the real `cargo` runs, with the capability `fixture` bound to
/// `demo::add` and to a test that passes and one that fails.
fn bound(tag: &str) -> Repo {
    let repo = Repo::real(tag);
    repo.put(
        "docs/capabilities/fixture.md",
        &capability(
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
        &change("shape", "[cap:fixture]", &["cap:fixture#contract"]),
    );
    repo.sync();
    repo
}

#[test]
fn a_bound_test_gives_the_verdict_and_a_path_that_names_none_starts_nothing() {
    let _lock = serial();
    let repo = bound("evidence-binding");
    let stubs = Stubs::new("evidence-binding-stubs");
    let target = repo.root().to_string_lossy().into_owned();
    let from = |id: &'static str, path: &'static str| {
        vec![
            "--root",
            target.as_str(),
            "evidence",
            "record",
            "--id",
            id,
            "--for",
            "cap:fixture",
            "--by",
            "human",
            "--from-binding",
            path,
        ]
    };
    // A path that is no test or example of the capability: nothing starts.
    let before = repo.snapshot();
    let run = stubs.run(
        repo.root(),
        Shape::Direct,
        &from("nowhere", "demo::tests::nowhere"),
        true,
    );
    let run = run.expect(2);
    assert!(run.stderr.contains("demo::tests::passes"), "{run:#?}");
    assert_eq!(stubs.cargo_calls(), Vec::<Vec<String>>::new());
    assert_eq!(repo.snapshot(), before);

    // The neighbours: a passing test is `pass`, a failing one `fail`.
    let run = stubs
        .run(
            repo.root(),
            Shape::Direct,
            &from("passes", "demo::tests::passes"),
            true,
        )
        .expect(0);
    assert_eq!(run.stdout, "recorded: work/evidence/passes.md (pass)\n");
    assert!(
        repo.read("work/evidence/passes.md")
            .contains("subject: binding:demo::tests::passes")
    );
    assert_ne!(stubs.cargo_calls().len(), 0, "the test was run by cargo");
    let run = stubs
        .run(
            repo.root(),
            Shape::Cargo,
            &from("fails", "demo::tests::fails"),
            true,
        )
        .expect(1);
    assert!(
        run.stdout
            .starts_with("recorded: work/evidence/fails.md (fail)"),
        "{run:#?}"
    );
    assert_eq!(
        changed(&before, &repo.snapshot()),
        ["work/evidence/fails.md", "work/evidence/passes.md"]
    );
}

#[test]
fn recorded_binding_failures_keep_failed_and_unresolved_outcomes_distinct() {
    let _lock = serial();
    for (tag, binding, expected) in [
        ("binding-failed-rdf", "demo::tests::fails", Some("failed")),
        (
            "binding-ignored-rdf",
            "demo::tests::passes",
            Some("unresolved"),
        ),
        ("binding-passed-rdf", "demo::tests::passes", None),
    ] {
        let repo = bound(tag);
        if expected == Some("unresolved") {
            repo.edit(
                "src/lib.rs",
                "    fn passes()",
                "    #[ignore]\n    fn passes()",
            );
        }
        let run = repo
            .run(&[
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
                binding,
            ])
            .expect(i32::from(expected.is_some()));
        let rdf = Rdf::of(&run);
        let failures = rdf.rows("EvidenceRecorded")[0].children("failed");
        let opened = raf_workplan::Gate::workspace(repo.root())
            .programs(raf_workplan::exec::Programs::new(repo.root()).discover_git())
            .open()
            .unwrap();
        let digest = raf_workplan::digest::Digest::symbol(opened.workspace(), "demo::add")
            .unwrap()
            .to_string();
        for format in [raf_rdf::Format::Turtle, raf_rdf::Format::NTriples] {
            let parsed = rdf.reparsed(format);
            let recorded = parsed.rows("EvidenceRecorded")[0];
            let inputs = recorded.children("input");
            assert_eq!(inputs.len(), 1);
            assert_eq!(inputs[0].required("name"), "demo::add");
            assert_eq!(inputs[0].required("digest"), digest);
            let assertion = inputs[0].reified("verdict");
            assert_eq!(assertion.required("subject"), format!("binding:{binding}"));
            assert_eq!(assertion.required("id"), "evidence:bound");
            assert_eq!(
                assertion.required("verdict"),
                if expected.is_some() { "fail" } else { "pass" }
            );
            assert!(parsed.rows("VerificationRun").is_empty());
        }
        if let Some(expected) = expected {
            assert_eq!(failures.len(), 1);
            assert_eq!(failures[0].required("reason"), "binding");
            assert_eq!(failures[0].required("outcome"), expected);
            assert_ne!(failures[0].required("message"), "");
        } else {
            assert!(failures.is_empty());
        }
        assert!(rdf.rows("VerificationRun").is_empty());
    }
}

/// The roadmap repository as a git checkout with the change `calm`, which
/// lists the delivered `alpha`, declares its contract unaffected and owes
/// nothing.
fn calm(tag: &str) -> Repo {
    let repo = Repo::synced(tag, false);
    repo.put(
        "work/changes/calm.md",
        &change("calm", "[cap:alpha]", &["cap:alpha#contract"])
            .replace("contract_effect: changes", "contract_effect: none"),
    );
    repo.sync();
    repo.assess("calm");
    repo.checkout();
    repo
}

const CLAIM: &str = "Text for Contract of alpha.\n<!-- raf:claim evidence=peak -->\n";

#[test]
fn a_claim_on_a_recorded_result_reconciles_clean_until_its_bound_code_moves() {
    let repo = calm("evidence-claim");
    repo.both(&["change", "reconcile", "calm"]).expect(0);
    repo.both_writing(&record(&["--", "/bin/sh", "-c", "echo peak=41"]))
        .expect(0);
    // The contract of alpha now rests on the record, and is assessed again.
    repo.edit(
        "docs/capabilities/alpha.md",
        "Text for Contract of alpha.",
        CLAIM,
    );
    repo.sync();
    repo.assess("calm");
    repo.git(&["add", "-A"]);
    let run = repo.both(&["change", "reconcile", "calm"]).expect(0);
    assert_eq!(run.stdout, "");

    // The views: the statement that rests on the record, and the record that
    // supports the capability.
    let line = repo
        .read("docs/capabilities/alpha.md")
        .lines()
        .position(|l| l.starts_with("<!-- raf:claim"))
        .expect("the marker")
        + 1;
    let claims = repo.both(&["evidence", "claims", "peak"]).expect(0);
    assert_eq!(
        claims.stdout,
        format!(
            "docs/capabilities/alpha.md:{line}: cap:alpha#contract\n    **Result.** Text for Contract of alpha.\n"
        )
    );
    assert_eq!(repo.both(&["evidence", "claims", "evidence:peak"]), claims);
    let support = repo.both(&["evidence", "for", "cap:alpha"]).expect(0);
    assert_eq!(
        support.stdout,
        format!("evidence:peak  pass  {RECORD}  (in its for list; claims at lines {line})\n")
    );

    // An edit of the bound code makes the record stale, naming the symbol.
    repo.edit(
        "src/lib.rs",
        "pub fn real() {}",
        "pub fn real() {\n    let _ = 1;\n}",
    );
    let run = repo.both(&["change", "reconcile", "calm"]).expect(1);
    assert!(
        run.lines()
            .iter()
            .any(|l| l.contains("evidence-stale") && l.contains("demo::real")),
        "{run:#?}"
    );
}

#[test]
fn the_views_print_rdf_and_refuse_what_does_not_exist() {
    let repo = Repo::synced("evidence-views", false);
    repo.both_writing(&record(&["--", "/bin/sh", "-c", "echo peak=41"]))
        .expect(0);
    repo.edit(
        "docs/capabilities/alpha.md",
        "Text for Contract of alpha.",
        CLAIM,
    );
    repo.sync();
    let before = repo.snapshot();
    for format in ["turtle", "ntriples"] {
        assert_eq!(semantic_claims(&repo, "peak", format).reifiers().count(), 1);
    }
    let support = repo
        .both(&["--format", "turtle", "evidence", "for", "cap:alpha"])
        .expect(0);
    let rdf = Rdf::of(&support);
    let rows = rdf.rows("EvidenceSupport");
    assert_eq!(rows.len(), 1);
    let row = rows[0];
    assert_eq!(row.required("verdict"), "pass");
    assert!(row.boolean("listed"));
    assert_eq!(row.required("capability"), "cap:alpha");
    assert_eq!(row.required("evidence"), "evidence:peak");
    assert!(row.children("claim")[0].number("line") > 0);
    // `-q` hides the views, which are not findings.
    assert_eq!(
        repo.both(&["-q", "evidence", "claims", "peak"])
            .expect(0)
            .stdout,
        ""
    );
    assert_eq!(
        repo.both(&["-q", "evidence", "for", "cap:alpha"])
            .expect(0)
            .stdout,
        ""
    );
    // A capability no record supports is an empty answer with a note.
    let none = repo.both(&["evidence", "for", "cap:beta"]).expect(0);
    assert_eq!(none.stdout, "");
    assert!(
        none.stderr.contains("no evidence record supports cap:beta"),
        "{none:#?}"
    );
    // What does not exist is exit 2.
    for args in [
        &["evidence", "claims", "nowhere"][..],
        &["evidence", "for", "cap:nowhere"],
    ] {
        let run = repo.both(args).expect(2);
        assert_eq!(run.stdout, "", "{args:?}");
    }
    // The views write nothing.
    assert_eq!(repo.snapshot(), before);
    for format in ["turtle", "ntriples"] {
        let run = repo
            .both(&["--format", format, "evidence", "claims", "nowhere"])
            .expect(2);
        assert_eq!(run.stdout, "");
        assert_eq!(repo.snapshot(), before);
    }
    repo.edit(
        "docs/capabilities/alpha.md",
        "<!-- raf:claim evidence=peak -->",
        "",
    );
    for format in ["turtle", "ntriples"] {
        let empty = semantic_claims(&repo, "peak", format);
        assert_eq!(empty.reifiers().count(), 0);
        assert!(
            empty.quads().any(|q| empty.term_value(q.p).as_iri()
                == Some("https://blackcatinformatics.ca/raf/workplan/1#evidence")),
            "explicit empty selection"
        );
    }
}

#[test]
fn claims_exit_two_naming_a_capability_that_does_not_load_and_answer_beside_other_records() {
    let claimed = || {
        let repo = Repo::synced("evidence-claims-unloaded", false);
        repo.edit(
            "docs/capabilities/alpha.md",
            "Text for Contract of alpha.",
            CLAIM,
        );
        repo.sync();
        repo
    };
    let answer = |repo: &Repo| {
        let before = repo.snapshot();
        let run = repo.both(&["evidence", "claims", "peak"]).expect(0);
        assert!(
            run.stdout.starts_with("docs/capabilities/alpha.md:")
                && run.stdout.contains("cap:alpha#contract"),
            "{run:#?}"
        );
        for format in ["turtle", "ntriples"] {
            assert_eq!(semantic_claims(repo, "peak", format).reifiers().count(), 1);
        }
        assert_eq!(repo.snapshot(), before);
    };
    // The clean neighbour answers.
    answer(&claimed());
    // A capability document that does not load may hold a claim on the
    // record: exit 2, naming it, with nothing on standard output and nothing
    // written, for an id with claims and for one without.
    let broken = claimed();
    broken.edit(
        "docs/capabilities/beta.md",
        "state: implementing",
        "state: unknown",
    );
    let before = broken.snapshot();
    for id in ["peak", "evidence:peak", "nowhere"] {
        let run = broken.both(&["evidence", "claims", id]).expect(2);
        assert_eq!(run.stdout, "", "{run:#?}");
        assert!(
            run.stderr.contains("cap:beta did not load"),
            "{id}: {run:#?}"
        );
    }
    for format in ["turtle", "ntriples"] {
        let run = broken
            .both(&["--format", format, "evidence", "claims", "peak"])
            .expect(2);
        assert!(run.stderr.contains("cap:beta"), "{run:#?}");
        assert_eq!(run.stdout, "");
    }
    assert_eq!(broken.snapshot(), before, "evidence claims wrote");
    // A change record or an evidence record that does not load holds no
    // claim: the answer stands, though `check` reports the record.
    for (path, text) in [
        (
            "work/changes/broken.md",
            change("broken", "[cap:beta]", &["cap:beta#contract"])
                .replace("items: []", "items: [x]"),
        ),
        (
            "work/evidence/broken.md",
            "---\ntype: Evidence\nid: evidence:broken\n---\n".to_string(),
        ),
    ] {
        let repo = claimed();
        repo.put(path, &text);
        assert!(
            repo.both(&["check"]).expect(1).stdout.contains(path),
            "{path}"
        );
        answer(&repo);
    }
}

#[test]
fn cargo_passes_the_command_after_the_separator_to_the_binary_unchanged() {
    let _lock = serial();
    let repo = Repo::synced("evidence-dispatch", false);
    let scratch = Scratch::new("evidence-dispatch-bin");
    let bin = scratch.path().join("bin");
    script(
        &bin.join("cargo-workplan"),
        &format!("exec '{}' \"$@\"", support::BINARY),
    );
    let path = {
        let mut dirs = vec![bin.clone()];
        dirs.extend(std::env::split_paths(
            &std::env::var_os("PATH").expect("a PATH"),
        ));
        std::env::join_paths(dirs).expect("a PATH")
    };
    let mut command = Command::new(real_cargo());
    command
        .arg("workplan")
        .args(record(&[
            "--",
            "/bin/sh",
            "-c",
            "printf '%s\\n' \"$@\" > argv.txt; echo peak=41",
            "sh",
            "--",
            "-q",
        ]))
        .current_dir(repo.root())
        .env("PATH", &path)
        .env_remove("CARGO_TARGET_DIR")
        .stdin(std::process::Stdio::null());
    let run = support::finish(&mut command).expect(0);
    assert_eq!(run.stdout, format!("recorded: {RECORD} (pass)\n"));
    assert_eq!(repo.read("argv.txt"), "--\n-q\n");
}

#[test]
fn a_claimed_statement_reaches_the_terminal_escaped_and_rdf_escapes_it_as_rdf() {
    let hostile = "\u{1b}[2J\u{1b}]0;PWNED\u{7} \u{202e}tfel";
    let repo = Repo::synced("evidence-claims-escaped", false);
    repo.edit(
        "docs/capabilities/alpha.md",
        "Text for Contract of alpha.",
        &format!(
            "Text for Contract of alpha.{hostile}\n\tIndented.\n<!-- raf:claim evidence=peak -->\n"
        ),
    );
    repo.sync();
    let line = repo
        .read("docs/capabilities/alpha.md")
        .lines()
        .position(|l| l.starts_with("<!-- raf:claim"))
        .expect("the marker")
        + 1;
    let run = repo.both(&["evidence", "claims", "peak"]).expect(0);
    assert_eq!(
        run.stdout,
        format!(
            "docs/capabilities/alpha.md:{line}: cap:alpha#contract\n    **Result.** Text for Contract of alpha.\\u{{1b}}[2J\\u{{1b}}]0;PWNED\\u{{7}} \\u{{202e}}tfel\n    \tIndented.\n"
        )
    );
    let before = repo.snapshot();
    for format in ["turtle", "ntriples"] {
        let dataset = semantic_claims(&repo, "peak", format);
        let (_, triple) = dataset.reifiers().next().unwrap();
        let purrdf_rdf::TermRef::Triple { o, .. } = dataset.resolve(triple) else {
            panic!("triple term")
        };
        assert_eq!(
            dataset.term_value(o),
            raf_rdf::TermValue::simple_literal(format!(
                "**Result.** Text for Contract of alpha.{hostile}\n\tIndented.\n"
            ))
        );
    }
    assert_eq!(repo.snapshot(), before);
}

#[test]
fn an_opposite_host_binding_is_exit_two_and_cannot_create_or_replace_evidence() {
    let repo = bound("evidence-host-nonexecution");
    let original = repo.read("src/lib.rs");
    repo.put("src/lib.rs", &format!(
        "{original}\n#[cfg(windows)]\n#[test]\nfn opposite_host() {{ panic!(\"must not run on this Unix host\"); }}\n"
    ));
    repo.edit(
        "docs/capabilities/fixture.md",
        "[demo::tests::passes, demo::tests::fails]",
        "[demo::tests::passes, demo::tests::fails, demo::opposite_host]",
    );
    repo.sync();
    let args = [
        "evidence",
        "record",
        "--id",
        "host",
        "--for",
        "cap:fixture",
        "--by",
        "human",
        "--from-binding",
        "demo::opposite_host",
    ];
    let before = repo.snapshot();
    for shape in [Shape::Direct, Shape::Cargo] {
        let run = support::run_in(repo.root(), shape, &args).expect(2);
        assert_eq!(run.stdout, "", "{run:#?}");
        for text in [
            "demo::opposite_host",
            "not applicable on this host",
            "no execution evidence was recorded",
            "--print=cfg",
        ] {
            assert!(run.stderr.contains(text), "{text}: {run:#?}");
        }
        assert!(!repo.root().join("work/evidence/host.md").exists());
        assert_eq!(repo.snapshot(), before);
    }
    // The same publication path accepts an actual passing bound execution.
    let mut passing = args;
    passing[9] = "demo::tests::passes";
    repo.run(&passing).expect(0);
    assert!(repo.read("work/evidence/host.md").contains("verdict: pass"));
    let kept = repo.snapshot();
    for shape in [Shape::Direct, Shape::Cargo] {
        let mut update = args.to_vec();
        update.push("--update");
        let run = support::run_in(repo.root(), shape, &update).expect(2);
        assert_eq!(run.stdout, "", "{run:#?}");
        assert!(
            run.stderr.contains("no execution evidence was recorded"),
            "{run:#?}"
        );
        assert_eq!(repo.snapshot(), kept);
    }
}
