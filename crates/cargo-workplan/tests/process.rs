// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The processes the binary starts, seen through stub programs: the exact
//! absence of Cargo calls during root discovery, a slug that starts
//! with `-` refused before any process starts, an id, a day or a value of the
//! history, evidence, assessment and coverage commands refused the same way,
//! a missing `git`, a Git too old for the history, and a `build.rustc`
//! program that discovery does not run.

#![cfg(unix)]

mod support;

use std::fs;
use std::process::Command;

use support::stubs::{Stubs, real_cargo, script, serial};
use support::{Repo, Scratch, Shape};

#[test]
fn discovery_starts_no_cargo_from_the_cargo_variable() {
    let _lock = serial();
    let repo = Repo::synced("process-cargo-var", false);
    let stubs = Stubs::new("process-cargo-var-stubs");
    for shape in [Shape::Direct, Shape::Cargo] {
        let run = stubs
            .run(&repo.root().join("src"), shape, &["status"], true)
            .expect(0);
        assert!(run.stdout.starts_with("delivered"), "{run:#?}");
        let calls = stubs.cargo_calls();
        assert!(calls.is_empty(), "root discovery starts no cargo process");
    }
}

#[test]
fn discovery_starts_no_cargo_on_path_when_the_variable_is_unset() {
    let _lock = serial();
    let repo = Repo::synced("process-cargo-path", false);
    let stubs = Stubs::new("process-cargo-path-stubs");
    for shape in [Shape::Direct, Shape::Cargo] {
        stubs.run(repo.root(), shape, &["check"], false).expect(0);
        let calls = stubs.cargo_calls();
        assert!(calls.is_empty(), "root discovery starts no cargo process");
    }
}

#[test]
fn root_starts_no_cargo_at_all() {
    let _lock = serial();
    let repo = Repo::synced("process-root", false);
    let stubs = Stubs::new("process-root-stubs");
    let target = repo.root().to_string_lossy().into_owned();
    for (cargo_var, args) in [
        (true, vec!["--root", &target, "check"]),
        (false, vec!["--root", &target, "status"]),
        (true, vec!["--root", &target, "docs", "preview", "grow"]),
        (true, vec!["--root", &target, "docs", "sync"]),
    ] {
        stubs
            .run(&support::elsewhere(), Shape::Direct, &args, cargo_var)
            .expect(0);
    }
    assert_eq!(stubs.cargo_calls(), Vec::<Vec<String>>::new());
    // The stubs are live: the repository was asked about by git.
    assert_eq!(stubs.git_calls(), Vec::<Vec<String>>::new());
}

#[test]
fn a_slug_that_starts_with_a_dash_is_refused_before_any_process_starts() {
    let _lock = serial();
    let repo = Repo::synced("process-dash", false);
    let stubs = Stubs::new("process-dash-stubs");
    let target = repo.root().to_string_lossy().into_owned();
    for slug in [
        "-rf",
        "--since=x",
        "-",
        "--output=/tmp/x",
        "-Zunstable-options",
    ] {
        for shape in [Shape::Direct, Shape::Cargo] {
            for args in [
                // No root given: discovery would inspect manifests.
                vec!["docs", "preview", "--", slug],
                vec!["docs", "preview", slug],
                // A root given: the repository would be opened by git.
                vec!["--root", &target, "docs", "preview", "--", slug],
                vec!["--root", &target, "docs", "preview", slug],
            ] {
                let run = stubs.run(repo.root(), shape, &args, true).expect(2);
                assert_eq!(run.stdout, "", "{args:?}");
                assert!(!run.stderr.is_empty(), "{args:?}");
            }
        }
    }
    assert_eq!(stubs.cargo_calls(), Vec::<Vec<String>>::new());
    assert_eq!(stubs.git_calls(), Vec::<Vec<String>>::new());

    // The neighbour: a slug that is a slug starts the processes.
    stubs
        .run(
            repo.root(),
            Shape::Direct,
            &["docs", "preview", "grow"],
            true,
        )
        .expect(0);
    assert_eq!(stubs.cargo_calls(), Vec::<Vec<String>>::new());
    assert_eq!(stubs.git_calls(), Vec::<Vec<String>>::new());
}

#[test]
fn a_slug_that_is_not_a_slug_is_refused_before_any_process_starts() {
    let _lock = serial();
    let repo = Repo::synced("process-bad-slug", false);
    let stubs = Stubs::new("process-bad-slug-stubs");
    for slug in ["Grow", "../grow", "grow/x", "change:grow", ""] {
        stubs
            .run(repo.root(), Shape::Direct, &["docs", "preview", slug], true)
            .expect(2);
    }
    assert_eq!(stubs.cargo_calls(), Vec::<Vec<String>>::new());
    assert_eq!(stubs.git_calls(), Vec::<Vec<String>>::new());
}

#[test]
fn a_usage_error_starts_no_process() {
    let _lock = serial();
    let repo = Repo::synced("process-usage", false);
    let stubs = Stubs::new("process-usage-stubs");
    for args in [
        &["frobnicate"][..],
        &["check", "--bogus"],
        &["--format", "yaml", "check"],
        &["docs", "export", "missing.md"],
        &["--format", "text", "docs", "export", "missing.md"],
        &["--format", "turtle", "-q", "docs", "export", "missing.md"],
        &["--format", "turtle", "-q", "evidence", "claims", "peak"],
        &["--format", "ntriples", "-q", "evidence", "claims", "peak"],
    ] {
        stubs.run(repo.root(), Shape::Direct, args, true).expect(2);
    }
    assert_eq!(stubs.cargo_calls(), Vec::<Vec<String>>::new());
    assert_eq!(stubs.git_calls(), Vec::<Vec<String>>::new());
    for args in [
        &["docs", "export", "missing.md"][..],
        &["--format", "turtle", "-q", "docs", "export", "missing.md"],
    ] {
        let run = support::run_in(&support::elsewhere(), Shape::Direct, args).expect(2);
        assert!(run.stderr.contains("requires --format"), "{run:#?}");
        assert_eq!(run.stdout, "");
    }
}

#[test]
#[cfg(target_os = "linux")]
fn help_and_version_output_failures_exit_two_beside_successful_output() {
    let _lock = serial();
    let repo = Repo::synced("process-help-output", false);
    repo.edit(
        "docs/capabilities/alpha.md",
        "Text for Contract of alpha.",
        "Text for Contract of alpha.\n<!-- raf:claim evidence=peak -->\n",
    );
    let path = "docs/architecture/row-keys.md";
    repo.put(
        path,
        include_str!("../../raf-okf/tests/fixtures/architecture.md"),
    );
    for (args, expected) in [
        (&["--help"][..], "cannot write command-line response"),
        (&["--version"][..], "cannot write command-line response"),
        (
            &["--format", "ntriples", "docs", "export", path][..],
            "cannot write the report",
        ),
        (
            &["--format", "turtle", "docs", "export", path][..],
            "cannot write the report",
        ),
        (
            &["--format", "turtle", "evidence", "claims", "peak"][..],
            "cannot write the report",
        ),
        (
            &["--format", "ntriples", "evidence", "claims", "peak"][..],
            "cannot write the report",
        ),
    ] {
        for shape in [Shape::Direct, Shape::Cargo] {
            let before = repo.snapshot();
            let mut command = support::command(repo.root(), shape, args);
            command.stdout(
                fs::OpenOptions::new()
                    .write(true)
                    .open("/dev/full")
                    .unwrap(),
            );
            let output = command.output().unwrap();
            assert_eq!(
                output.status.code(),
                Some(2),
                "{args:?} {shape:?} {output:?}"
            );
            assert!(purrdf_testkit::text::stderr_utf8(&output).contains(expected));
            let successful = support::run_in(repo.root(), shape, args).expect(0);
            assert_ne!(successful.stdout, "");
            assert_eq!(
                repo.snapshot(),
                before,
                "{args:?} changed repository inputs"
            );
        }
    }
}

#[test]
fn discovery_succeeds_when_cargo_cannot_be_started() {
    let _lock = serial();
    let repo = Repo::synced("process-no-cargo", false);
    let mut command = support::command(repo.root(), Shape::Direct, &["check"]);
    command.env("CARGO", repo.root().join("no-such-cargo"));
    support::finish(&mut command).expect(0);
}

#[test]
fn a_missing_git_binary_does_not_prevent_native_reads() {
    let _lock = serial();
    let repo = Repo::synced("process-no-git", false);
    let empty = Scratch::new("process-no-git-path");
    let target = repo.root().to_string_lossy().into_owned();
    for shape in [Shape::Direct, Shape::Cargo] {
        let mut command =
            support::command(&support::elsewhere(), shape, &["--root", &target, "check"]);
        command.env("PATH", empty.path());
        let run = support::finish(&mut command).expect(0);
        assert_eq!(run.stderr, "");
    }
    // The neighbour: with git on PATH the same run checks.
    let run = support::run_in(
        &support::elsewhere(),
        Shape::Direct,
        &["--root", &target, "check"],
    );
    run.expect(0);
}

#[test]
fn discovery_does_not_run_the_build_rustc_program_of_the_workspace() {
    let _lock = serial();
    let repo = Repo::synced("process-build-rustc", false);
    let scratch = Scratch::new("process-build-rustc-script");
    let marker = scratch.path().join("ran");
    let rustc = scratch.path().join("rustc-script");
    script(
        &rustc,
        &format!("echo ran >> '{}'\nexec rustc \"$@\"", marker.display()),
    );
    repo.put(
        ".cargo/config.toml",
        &format!("[build]\nrustc = \"{}\"\n", rustc.display()),
    );
    for shape in [Shape::Direct, Shape::Cargo] {
        support::run_in(repo.root(), shape, &["status"]).expect(0);
    }
    assert!(!marker.exists(), "discovery ran the build.rustc program");
}

/// Each command that takes a slug, as the words before it and the words after
/// it, and a slug that is allowed.
fn slug_commands() -> Vec<(Vec<&'static str>, Vec<&'static str>, &'static str)> {
    vec![
        (vec!["docs", "preview"], vec![], "grow"),
        (vec!["change", "impact"], vec![], "grow"),
        (vec!["change", "reconcile"], vec![], "grow"),
        (vec!["change", "complete"], vec![], "grow"),
        (vec!["verify"], vec![], "grow"),
        (
            vec!["change", "new"],
            vec!["--capability", "cap:beta"],
            "fresh",
        ),
    ]
}

#[test]
fn a_slug_that_starts_with_a_dash_is_refused_by_every_command_before_any_process_starts() {
    let _lock = serial();
    let repo = Repo::synced("process-dash-all", false);
    repo.checkout();
    let stubs = Stubs::new("process-dash-all-stubs");
    let target = repo.root().to_string_lossy().into_owned();
    let before = repo.snapshot();
    for slug in ["-rf", "--since=x", "-", "--output=/tmp/x"] {
        for shape in [Shape::Direct, Shape::Cargo] {
            for (head, tail, _) in slug_commands() {
                // As an argument, as a value after `--`, with a root given
                // (git would be asked) and without (cargo would be).
                for (rooted, dashed) in [(false, false), (false, true), (true, false), (true, true)]
                {
                    let mut args: Vec<&str> = Vec::new();
                    if rooted {
                        args.extend(["--root", &target]);
                    }
                    args.extend(&head);
                    args.extend(&tail);
                    if dashed {
                        args.push("--");
                    }
                    args.push(slug);
                    let run = stubs.run(repo.root(), shape, &args, true).expect(2);
                    assert_eq!(run.stdout, "", "{args:?}");
                    assert!(!run.stderr.is_empty(), "{args:?}");
                }
            }
        }
    }
    assert_eq!(stubs.cargo_calls(), Vec::<Vec<String>>::new());
    assert_eq!(stubs.git_calls(), Vec::<Vec<String>>::new());
    assert_eq!(repo.snapshot(), before);

    // Valid neighboring slugs reach static command behavior without starting tools.
    for (head, tail, slug) in slug_commands() {
        let mut args = vec!["--root", &target];
        args.extend(&head);
        args.push(slug);
        args.extend(&tail);
        let run = stubs.run(repo.root(), Shape::Direct, &args, true);
        assert_ne!(run.code, 2, "{args:?}: {run:?}");
        assert!(stubs.git_calls().is_empty(), "{args:?} started Git");
        assert!(stubs.cargo_calls().is_empty(), "{args:?} started Cargo");
    }
}

#[test]
fn a_usage_error_of_the_change_commands_starts_no_process() {
    let _lock = serial();
    let repo = Repo::synced("process-change-usage", false);
    let stubs = Stubs::new("process-change-usage-stubs");
    for args in [
        &["change"][..],
        &["change", "frobnicate"],
        &["change", "new", "fresh"],
        &["change", "new", "--capability", "cap:beta"],
        &[
            "change",
            "new",
            "fresh",
            "--capability",
            "cap:beta",
            "--copyright",
            "x",
        ],
        &[
            "change",
            "new",
            "fresh",
            "--capability",
            "cap:beta",
            "--licence",
            "x",
        ],
        &["change", "impact"],
        &["change", "reconcile"],
        &["change", "complete"],
        &["change", "complete", "grow", "extra"],
        &["verify", "grow", "extra"],
        &["verify", "--bogus"],
    ] {
        stubs.run(repo.root(), Shape::Direct, args, true).expect(2);
    }
    assert_eq!(stubs.cargo_calls(), Vec::<Vec<String>>::new());
    assert_eq!(stubs.git_calls(), Vec::<Vec<String>>::new());
}

/// A `git` that answers `--version` with `version`, records every call, and
/// fails any other.
#[test]
fn cargo_dispatches_workplan_to_the_binary_on_path_and_passes_the_exit_code_back() {
    let _lock = serial();
    let repo = Repo::synced("process-dispatch", false);
    repo.checkout();
    let scratch = Scratch::new("process-dispatch-bin");
    let bin = scratch.path().join("bin");
    let record = scratch.path().join("argv.record");
    // A `cargo-workplan` that records the words cargo gave it, then is the
    // binary.
    script(
        &bin.join("cargo-workplan"),
        &format!(
            "for a in \"$@\"; do printf 'arg=%s\\n' \"$a\"; done >> '{}'\nprintf 'end\\n' >> '{}'\nexec '{}' \"$@\"",
            record.display(),
            record.display(),
            support::BINARY
        ),
    );
    let path = {
        let mut dirs = vec![bin.clone()];
        dirs.extend(std::env::split_paths(
            &std::env::var_os("PATH").expect("a PATH"),
        ));
        std::env::join_paths(dirs).expect("a PATH")
    };
    let cargo = |args: &[&str]| -> support::Run {
        let mut command = Command::new(real_cargo());
        command
            .arg("workplan")
            .args(args)
            .current_dir(repo.root())
            .env("PATH", &path)
            .env_remove("CARGO_TARGET_DIR")
            .stdin(std::process::Stdio::null());
        for name in [
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_COMMON_DIR",
            "GIT_INDEX_FILE",
        ] {
            command.env_remove(name);
        }
        support::finish(&mut command)
    };
    let direct = |args: &[&str]| support::run_in(repo.root(), Shape::Direct, args);

    // Each command, with its exit code: 0, 1 and 2.
    for (args, code) in [
        (vec!["check"], 0),
        (vec!["status"], 0),
        (vec!["change", "impact", "grow"], 0),
        (vec!["change", "reconcile", "grow"], 1),
        (vec!["change", "complete", "grow"], 1),
        (vec!["change", "impact", "nowhere"], 2),
        (vec!["--format", "turtle", "change", "reconcile", "grow"], 1),
        (vec!["-q", "change", "reconcile", "grow"], 1),
        (
            vec!["change", "new", "fresh", "--capability", "cap:beta"],
            0,
        ),
        (vec!["change", "new", "-rf", "--capability", "cap:beta"], 2),
    ] {
        let through = cargo(&args);
        assert_eq!(through.code, code, "cargo workplan {args:?}: {through:#?}");
        if args[0] != "change" || args[1] != "new" {
            assert_eq!(through, direct(&args), "{args:?}");
        }
    }
    assert!(repo.root().join("work/changes/fresh.md").is_file());

    // Cargo gave the binary the word `workplan` first, then the arguments.
    let calls = fs::read_to_string(&record).expect("a record");
    assert!(
        calls.starts_with("arg=workplan\narg=check\nend\n"),
        "{calls}"
    );
    assert!(
        calls.contains("arg=workplan\narg=change\narg=reconcile\narg=grow\nend\n"),
        "{calls}"
    );
}
