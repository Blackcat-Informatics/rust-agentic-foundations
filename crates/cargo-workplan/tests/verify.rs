// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `verify` through the built binary and the real `cargo`: one line for each
//! binding, the exit 0 of a run in which every binding passes and the exit 1 of
//! one in which a binding does not, the exit 2 of a slug with no record, `-q`,
//! the typed RDF results, and that it writes nothing.

mod support;
use support::rdf::Rdf;

use support::{BLOCKS, Repo, Run, capability, change};

/// A repository with the capability `fixture` bound as given (tests, then
/// examples) and the change `shape` that lists it.
fn fixture(tag: &str, tests: &str, examples: &str) -> Repo {
    let repo = Repo::real(tag);
    repo.put(
        "docs/capabilities/fixture.md",
        &capability(
            "fixture",
            "implementing",
            ["[demo::add]", tests, examples],
            "[]",
            &[("Verification", BLOCKS)],
        ),
    );
    repo.put(
        "work/changes/shape.md",
        &change("shape", "[cap:fixture]", &["cap:fixture#contract"]),
    );
    repo.sync();
    repo
}

/// The lines of a run with the output of `cargo` cut from a failing line: what
/// it holds past the exit code (timings, a lock message, paths of the build)
/// is not the same twice.
fn steady(run: &Run) -> Vec<String> {
    run.lines().into_iter().map(cut).collect()
}

/// One line with the output of `cargo` cut from it when it is a failing one.
fn cut(line: &str) -> String {
    match line.find("exited with code ") {
        Some(at) if line.starts_with("fail ") => {
            let end = line[at..].find(':').map_or(line.len(), |colon| at + colon);
            line[..end].to_string()
        }
        _ => line.to_string(),
    }
}

/// Run `args` in both shapes and require the same code and the same steady
/// lines. The direct run is returned.
fn both_steady(repo: &Repo, args: &[&str]) -> Run {
    let direct = repo.run(args);
    let cargo = support::run_in(repo.root(), support::Shape::Cargo, args);
    assert_eq!(direct.code, cargo.code, "{args:?}");
    assert_eq!(steady(&direct), steady(&cargo), "{args:?}");
    direct
}

const PASS: &str =
    "pass cap:fixture tests `demo::tests::passes` (1 listed, 1 passed, 0 failed, 0 ignored)";

#[test]
fn one_passing_and_one_failing_test_print_one_line_each_and_exit_one() {
    let repo = fixture(
        "verify-mixed",
        "[demo::tests::passes, demo::tests::fails]",
        "[]",
    );
    let before = repo.snapshot();
    let run = repo.run(&["verify", "shape"]).expect(1);
    let lines = run.lines();
    assert_eq!(lines.len(), 2, "{run:#?}");
    // Sorted by capability, list and path: `fails` before `passes`.
    assert!(
        lines[0].starts_with("fail cap:fixture tests `demo::tests::fails`: "),
        "{}",
        lines[0]
    );
    assert!(lines[0].contains("1 failed"), "{}", lines[0]);
    for cause in ["stdout:", "this test fails", "stderr:", "test failed"] {
        assert!(lines[0].contains(cause), "{}", lines[0]);
    }
    assert_eq!(lines[1], PASS);
    assert_eq!(run.stderr, "");
    // Both shapes agree, and nothing was written: the build goes to the target
    // directory the fixture names, outside the tree.
    assert_eq!(
        steady(&both_steady(&repo, &["verify", "shape"])),
        steady(&run)
    );
    assert_eq!(repo.snapshot(), before, "verify wrote");
}

#[test]
fn every_binding_passing_exits_zero() {
    let repo = fixture(
        "verify-pass",
        "[demo::tests::passes]",
        "[demo::example_crate::good]",
    );
    let run = repo.run(&["verify", "shape"]).expect(0);
    assert_eq!(
        run.lines(),
        [
            PASS,
            "pass cap:fixture examples `demo::example_crate::good` (no counts)"
        ]
    );
}

#[cfg(unix)]
#[test]
fn verification_uses_only_tracked_path_git_calls_and_frozen_cargo_without_a_commit() {
    use support::Shape;
    use support::stubs::{Stubs, serial};

    let _lock = serial();
    let repo = fixture("verify-tracked-only", "[demo::tests::passes]", "[]");
    repo.git(&["init", "-q"]);
    repo.git(&["add", "."]);
    let before = repo.snapshot();
    let stubs = Stubs::new("verify-tracked-only-stubs");
    let run = stubs
        .run(repo.root(), Shape::Direct, &["verify", "shape"], true)
        .expect(0);
    assert_eq!(run.lines(), [PASS]);
    assert_eq!(run.stderr, "");
    assert_eq!(
        stubs.git_calls(),
        [
            vec!["rev-parse", "--is-inside-work-tree"],
            vec!["ls-files", "-z"],
        ]
    );
    assert_eq!(
        stubs.cargo_calls(),
        [
            vec![
                "test",
                "--frozen",
                "--package=demo",
                "--lib",
                "--",
                "--list",
                "--exact",
                "--",
                "tests::passes",
            ],
            vec![
                "test",
                "--frozen",
                "--package=demo",
                "--lib",
                "--",
                "--exact",
                "--",
                "tests::passes",
            ],
        ]
    );
    assert_eq!(repo.snapshot(), before, "verify wrote to the fixture");
}

#[test]
fn a_failing_example_and_a_binding_that_proves_nothing_are_failures() {
    let repo = fixture(
        "verify-example",
        "[demo::tests::passes]",
        "[demo::example_crate::exiting, demo::example_crate::good, demo::real]",
    );
    let run = repo.run(&["verify", "shape"]).expect(1);
    let lines = run.lines();
    assert_eq!(lines.len(), 4, "{run:#?}");
    assert_eq!(lines[0], PASS);
    assert!(
        lines[1].starts_with("fail cap:fixture examples `demo::example_crate::exiting`: ")
            && lines[1].contains("exited with code 3"),
        "{}",
        lines[1]
    );
    assert_eq!(
        lines[2],
        "pass cap:fixture examples `demo::example_crate::good` (no counts)"
    );
    // A documented item with no runnable example runs nothing.
    assert!(
        lines[3].starts_with("fail cap:fixture examples `demo::real`: unresolved binding: "),
        "{}",
        lines[3]
    );
}

#[test]
fn no_slug_runs_every_capability_and_a_slug_only_the_ones_the_change_lists() {
    let repo = fixture("verify-all", "[demo::tests::passes]", "[]");
    repo.put(
        "docs/capabilities/other.md",
        &capability(
            "other",
            "implementing",
            ["[demo::add]", "[demo::tests::fails]", "[]"],
            "[]",
            &[("Verification", BLOCKS)],
        ),
    );
    repo.sync();
    // The change lists `fixture` only: its test passes.
    repo.run(&["verify", "shape"]).expect(0);
    // Without a slug the failing test of `other` is run too.
    let all = repo.run(&["verify"]).expect(1);
    let lines = all.lines();
    assert_eq!(lines.len(), 2, "{all:#?}");
    assert_eq!(lines[0], PASS);
    assert!(
        lines[1].starts_with("fail cap:other tests `demo::tests::fails`: "),
        "{}",
        lines[1]
    );
}

#[test]
fn a_slug_with_no_record_and_a_document_that_does_not_load_are_exit_two() {
    let repo = fixture("verify-two", "[demo::tests::passes]", "[]");
    let run = repo.both(&["verify", "nowhere"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.contains("no change record `nowhere`"),
        "{run:#?}"
    );

    // A capability document that does not load: what it binds is unknown, so a
    // run over every capability is not a pass.
    repo.edit(
        "docs/capabilities/fixture.md",
        "state: implementing",
        "state: bogus",
    );
    let run = repo.both(&["verify"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("did not load"), "{run:#?}");
}

#[test]
fn a_hostile_binding_is_a_failure_and_never_reaches_cargo() {
    let repo = fixture("verify-hostile", "[demo::tests::passes; touch pwned]", "[]");
    let root = repo.root().to_string_lossy().into_owned();
    // A cargo that does not exist: a run that started it would be exit 2.
    let mut command = support::command(
        &support::elsewhere(),
        support::Shape::Direct,
        &["--root", &root, "verify", "shape"],
    );
    command.env("CARGO", repo.root().join("no-such-cargo"));
    let run = support::finish(&mut command).expect(1);
    assert_eq!(run.lines().len(), 1, "{run:#?}");
    assert!(
        run.stdout.starts_with(
            "fail cap:fixture tests `demo::tests::passes; touch pwned`: unresolved binding: "
        ),
        "{run:#?}"
    );
    assert!(!repo.root().join("pwned").exists());
    let mut machine = support::command(
        &support::elsewhere(),
        support::Shape::Direct,
        &["--root", &root, "--format", "turtle", "verify", "shape"],
    );
    machine.env("CARGO", repo.root().join("no-such-cargo"));
    let refused = support::finish(&mut machine).expect(1);
    let rdf = Rdf::of(&refused);
    assert_eq!(rdf.rows("Binding").len(), 1);
    assert!(rdf.rows("VerificationRun").is_empty());
}

#[test]
fn nothing_to_run_is_exit_zero_with_a_note_that_quiet_hides() {
    let repo = fixture("verify-empty", "[]", "[]");
    let run = repo.both(&["verify", "shape"]).expect(0);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr
            .contains("cargo-workplan: there is no bound test or example to run"),
        "{run:#?}"
    );
    let quiet = repo.run(&["-q", "verify", "shape"]).expect(0);
    assert_eq!((quiet.stdout.as_str(), quiet.stderr.as_str()), ("", ""));
    let json = repo
        .run(&["--format", "turtle", "verify", "shape"])
        .expect(0);
    assert_eq!(Rdf::of(&json).result_count(), 0);
}

#[test]
fn quiet_prints_only_the_failing_bindings_and_keeps_the_exit() {
    let repo = fixture(
        "verify-quiet",
        "[demo::tests::passes, demo::tests::fails]",
        "[]",
    );
    let loud = repo.run(&["verify", "shape"]).expect(1);
    let quiet = both_steady(&repo, &["-q", "verify", "shape"]).expect(1);
    assert_eq!(quiet.lines().len(), 1, "{quiet:#?}");
    assert_eq!(steady(&quiet)[0], steady(&loud)[0]);
    assert!(quiet.lines().iter().all(|line| !line.starts_with("pass ")));

    // With every binding passing `-q` prints nothing and the exit is zero.
    let passing = fixture("verify-quiet-pass", "[demo::tests::passes]", "[]");
    let run = both_steady(&passing, &["--quiet", "verify", "shape"]).expect(0);
    assert_eq!(run.stdout, "");
}

#[test]
fn rdf_preserves_real_binding_outcomes_proof_counts_and_quiet_filtering() {
    let repo = fixture(
        "verify-rdf",
        "[demo::tests::passes, demo::tests::fails]",
        "[demo::real]",
    );
    let run = repo
        .run(&["--format", "turtle", "verify", "shape"])
        .expect(1);
    let cargo = support::run_in(
        repo.root(),
        support::Shape::Cargo,
        &["--format", "turtle", "verify", "shape"],
    )
    .expect(1);
    let outcomes = |run: &Run| {
        let rdf = Rdf::of(run);
        rdf.rows("Binding")
            .iter()
            .map(|r| {
                (
                    r.required("path").to_owned(),
                    r.required("outcome").to_owned(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(outcomes(&run), outcomes(&cargo));
    let rdf = Rdf::of(&run);
    let bindings = rdf.rows("Binding");
    assert_eq!(bindings.len(), 3);
    let (failed, passed, unresolved) = (bindings[0], bindings[1], bindings[2]);
    assert_eq!(failed.required("outcome"), "failed");
    assert_eq!(failed.required("path"), "demo::tests::fails");
    assert_eq!(failed.required("list"), "tests");
    assert_eq!(failed.required("capability"), "cap:fixture");
    assert_eq!(
        (
            failed.number("listed"),
            failed.number("passed"),
            failed.number("failed"),
            failed.number("ignored")
        ),
        (1, 0, 1, 0)
    );
    assert!(failed.required("reason").contains("exited with code 101"));
    assert_eq!(passed.required("outcome"), "passed");
    assert!(passed.text("reason").is_none());
    assert_eq!(passed.required("message"), PASS);
    let runs = passed.children("run");
    assert!(!runs.is_empty());
    assert!(
        runs.iter()
            .all(|run| run.required("programKind") == "cargo")
    );
    assert!(runs.iter().all(|run| run.required("ended") == "exited"));
    assert!(runs.iter().all(|run| run.number("exitCode") == 0));
    assert!(runs.iter().all(|run| !run.children("argument").is_empty()));
    assert_eq!(unresolved.required("outcome"), "unresolved");
    assert_eq!(unresolved.required("list"), "examples");
    assert!(
        unresolved
            .required("reason")
            .contains("no documentation test")
    );
    let text = repo.run(&["verify", "shape"]);
    let messages = bindings
        .iter()
        .map(|r| cut(r.required("message")))
        .collect::<Vec<_>>();
    assert_eq!(steady(&text), messages);
    let quiet = repo
        .run(&["-q", "--format", "turtle", "verify", "shape"])
        .expect(1);
    let rdf = Rdf::of(&quiet);
    let kept = rdf.rows("Binding");
    assert_eq!(kept.len(), 2);
    assert!(kept.iter().all(|r| r.required("outcome") != "passed"));
}

#[test]
fn host_nonapplicability_is_exit_zero_and_preserved_in_machine_output() {
    let repo = fixture("verify-host", "[demo::opposite_host]", "[]");
    let guard = if cfg!(windows) { "unix" } else { "windows" };
    repo.put("src/lib.rs", &format!(
        "pub fn add(a: u32, b: u32) -> u32 {{ a + b }}\n#[cfg({guard})]\n#[test]\nfn opposite_host() {{ panic!(\"must not run\"); }}\n"
    ));
    repo.sync();
    let before = repo.snapshot();
    let run = repo.run(&["verify", "shape"]).expect(0);
    assert!(
        run.stdout
            .starts_with("not-applicable cap:fixture tests `demo::opposite_host`: "),
        "{run:#?}"
    );
    assert!(!run.stdout.contains("1 passed"));
    let machine = repo
        .run(&["--format", "turtle", "verify", "shape"])
        .expect(0);
    let rdf = Rdf::of(&machine);
    let bindings = rdf.rows("Binding");
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].required("outcome"), "not-applicable");
    assert_eq!(bindings[0].required("path"), "demo::opposite_host");
    assert_ne!(bindings[0].required("reason"), "");
    assert!(bindings[0].text("passed").is_none());
    let observations = bindings[0].children("run");
    assert_eq!(observations.len(), 1);
    assert!(
        observations[0]
            .children("argument")
            .iter()
            .any(|arg| arg.required("text") == "--print=cfg")
    );
    let quiet = repo.run(&["-q", "verify", "shape"]).expect(0);
    assert_eq!(quiet.stdout, "");
    assert_eq!(repo.snapshot(), before);
}
