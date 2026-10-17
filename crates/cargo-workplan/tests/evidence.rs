// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `evidence record`, `evidence claims` and `evidence for` through the built
//! binary: a command after `--` run with no shell and judged by its exit
//! status and its `name=number` lines, exactly one record written, the exit 1
//! of a `fail` record and the exit 2 of a run that writes none, the options
//! after `--` left to the command, a bound test as the source of the verdict,
//! the JSON objects, a real `cargo workplan` dispatch, and a claim that
//! reconciles clean until its bound code moves.

#![cfg(unix)]

mod support;

use std::process::Command;

use support::stubs::{Stubs, real_cargo, script, serial};
use support::{
    COPYRIGHT, LICENCE, Repo, Scratch, Shape, array, capability, change, changed, member_names,
    strings,
};

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

/// The members of an `evidence-recorded` object, in order.
const RECORDED: [&str; 14] = [
    "kind", "path", "id", "verdict", "subject", "command", "revision", "by", "for", "values",
    "limits", "inputs", "failed", "message",
];

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
        let run = repo.both(&args).expect(2);
        assert_eq!(run.stdout, "", "{script}");
        assert!(
            run.stderr.contains("no record was written")
                || run.stderr.contains("nothing was recorded"),
            "{script}: {run:#?}"
        );
        assert_eq!(repo.snapshot(), before, "{script}");
        args.insert(0, "--format=json");
        assert_eq!(repo.both(&args).expect(2).stdout, "", "{script}");
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
fn json_is_one_evidence_recorded_object_with_its_fields() {
    let repo = Repo::synced("evidence-json", false);
    let run = repo
        .both_writing(&{
            let mut args = vec!["--format", "json"];
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
    assert_eq!(member_names(&run), [strings(&RECORDED)]);
    let objects = array(&run);
    let object = &objects[0];
    assert_eq!(object["kind"], "evidence-recorded");
    assert_eq!(object["path"], RECORD);
    assert_eq!(object["id"], "evidence:peak");
    assert_eq!(object["verdict"], "fail");
    assert_eq!(object["subject"], "command");
    assert_eq!(object["command"], "/bin/sh -c 'echo peak=80; echo rss=3.5'");
    assert_eq!(
        object["revision"],
        "0123456789abcdef0123456789abcdef01234567"
    );
    assert_eq!(object["by"], "agent");
    assert_eq!(object["for"], serde_json::json!(["cap:alpha"]));
    assert_eq!(
        object["values"],
        serde_json::json!([{"name": "peak", "value": "80"}, {"name": "rss", "value": "3.5"}])
    );
    assert_eq!(
        object["limits"],
        serde_json::json!([{"name": "peak", "value": "64"}])
    );
    assert_eq!(object["inputs"][0]["kind"], "symbol");
    assert_eq!(object["inputs"][0]["name"], "demo::real");
    assert_eq!(
        object["failed"],
        serde_json::json!([{"reason": "over-limit", "code": null, "names": ["peak"]}])
    );
    assert_eq!(
        object["message"],
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

const CLAIM: &str = "Text for Contract of alpha.\n<!-- raf:claim evidence=peak -->";

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
            "docs/capabilities/alpha.md:{line}: cap:alpha#contract\n    Text for Contract of alpha.\n"
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
fn the_views_print_json_and_refuse_what_does_not_exist() {
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
    let claims = repo
        .both(&["--format", "json", "evidence", "claims", "peak"])
        .expect(0);
    assert_eq!(
        member_names(&claims),
        [strings(&[
            "kind",
            "evidence",
            "capability",
            "path",
            "section",
            "line",
            "statement"
        ])]
    );
    let object = &array(&claims)[0];
    assert_eq!(object["kind"], "evidence-claim");
    assert_eq!(object["evidence"], "evidence:peak");
    assert_eq!(object["capability"], "cap:alpha");
    assert_eq!(object["section"], "contract");
    assert_eq!(object["statement"], "Text for Contract of alpha.\n");
    let support = repo
        .both(&["--format", "json", "evidence", "for", "cap:alpha"])
        .expect(0);
    assert_eq!(
        member_names(&support),
        [strings(&[
            "kind",
            "capability",
            "evidence",
            "path",
            "verdict",
            "listed",
            "claims"
        ])]
    );
    let object = &array(&support)[0];
    assert_eq!(object["kind"], "evidence-for");
    assert_eq!(object["verdict"], "pass");
    assert_eq!(object["listed"], true);
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
        let run = repo.both(&["evidence", "claims", "peak"]).expect(0);
        assert!(
            run.stdout.starts_with("docs/capabilities/alpha.md:")
                && run.stdout.contains("cap:alpha#contract"),
            "{run:#?}"
        );
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
    let json = broken
        .both(&["--format", "json", "evidence", "claims", "peak"])
        .expect(2);
    assert!(json.stderr.contains("cap:beta"), "{json:#?}");
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
fn a_claimed_statement_reaches_the_terminal_escaped_and_json_escapes_it_as_json() {
    let hostile = "\u{1b}[2J\u{1b}]0;PWNED\u{7} \u{202e}tfel";
    let repo = Repo::synced("evidence-claims-escaped", false);
    repo.edit(
        "docs/capabilities/alpha.md",
        "Text for Contract of alpha.",
        &format!(
            "Text for Contract of alpha.{hostile}\n\tIndented.\n<!-- raf:claim evidence=peak -->"
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
            "docs/capabilities/alpha.md:{line}: cap:alpha#contract\n    Text for Contract of alpha.\\u{{1b}}[2J\\u{{1b}}]0;PWNED\\u{{7}} \\u{{202e}}tfel\n    \tIndented.\n"
        )
    );
    let json = repo
        .both(&["--format", "json", "evidence", "claims", "peak"])
        .expect(0);
    assert!(
        !json.stdout.contains('\u{1b}') && json.stdout.contains("\\u001b"),
        "{json:#?}"
    );
    assert_eq!(
        array(&json)[0]["statement"],
        format!("Text for Contract of alpha.{hostile}\n\tIndented.\n")
    );
}
