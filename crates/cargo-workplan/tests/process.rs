// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The processes the binary starts, seen through stub programs: the exact
//! command of root discovery from `$CARGO` and from `PATH`, a slug that starts
//! with `-` refused before any process starts, a missing `git` and a
//! `build.rustc` program that discovery does not run.

#![cfg(unix)]

mod support;

use std::fs;
use std::process::Command;

use support::stubs::{Stubs, metadata_argv, real_cargo, script, serial};
use support::{Repo, Scratch, Shape};

#[test]
fn discovery_runs_exactly_cargo_metadata_from_the_cargo_variable() {
    let _lock = serial();
    let repo = Repo::synced("process-cargo-var", false);
    let stubs = Stubs::new("process-cargo-var-stubs");
    for shape in [Shape::Direct, Shape::Cargo] {
        let before = stubs.cargo_calls().len();
        let run = stubs
            .run(&repo.root().join("src"), shape, &["status"], true)
            .expect(0);
        assert!(run.stdout.starts_with("delivered"), "{run:#?}");
        let calls = stubs.cargo_calls();
        assert_eq!(calls.len(), before + 1, "one cargo process per run");
        assert_eq!(calls[before], metadata_argv());
    }
}

#[test]
fn discovery_runs_the_cargo_on_path_when_the_variable_is_unset() {
    let _lock = serial();
    let repo = Repo::synced("process-cargo-path", false);
    let stubs = Stubs::new("process-cargo-path-stubs");
    for shape in [Shape::Direct, Shape::Cargo] {
        let before = stubs.cargo_calls().len();
        stubs.run(repo.root(), shape, &["check"], false).expect(0);
        let calls = stubs.cargo_calls();
        assert_eq!(calls.len(), before + 1);
        assert_eq!(calls[before], metadata_argv());
    }
    // The working directory of the process is the directory of the run.
    let record = fs::read_to_string(&stubs.cargo_record).expect("a record");
    assert!(
        record.contains(&format!("cwd={}", repo.root().display())),
        "{record}"
    );
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
    assert_ne!(stubs.git_calls().len(), 0);
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
                // No root given: discovery would start cargo.
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
    assert_eq!(stubs.cargo_calls(), [metadata_argv()]);
    assert_ne!(stubs.git_calls().len(), 0);
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
    ] {
        stubs.run(repo.root(), Shape::Direct, args, true).expect(2);
    }
    assert_eq!(stubs.cargo_calls(), Vec::<Vec<String>>::new());
    assert_eq!(stubs.git_calls(), Vec::<Vec<String>>::new());
}

#[test]
fn a_cargo_that_fails_is_exit_two_with_what_it_said() {
    let _lock = serial();
    let repo = Repo::synced("process-cargo-fails", false);
    let scratch = Scratch::new("process-cargo-fails-stub");
    let stub = scratch.path().join("cargo");
    script(&stub, "echo 'the stub refuses' >&2\nexit 101");
    let mut command = support::command(repo.root(), Shape::Direct, &["check"]);
    command.env("CARGO", &stub);
    let run = support::finish(&mut command).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("the stub refuses"), "{run:#?}");
    assert!(run.stderr.contains("exit 101"), "{run:#?}");
}

#[test]
fn output_from_cargo_that_is_not_the_metadata_is_exit_two() {
    let _lock = serial();
    let repo = Repo::synced("process-cargo-garbage", false);
    let scratch = Scratch::new("process-cargo-garbage-stub");
    for (name, body) in [
        ("text", "echo 'not json'"),
        ("no-root", "echo '{\"packages\":[]}'"),
        ("relative", "echo '{\"workspace_root\":\"relative/dir\"}'"),
    ] {
        let stub = scratch.path().join(name);
        script(&stub, body);
        let mut command = support::command(repo.root(), Shape::Direct, &["check"]);
        command.env("CARGO", &stub);
        let run = support::finish(&mut command).expect(2);
        assert_eq!(run.stdout, "", "{name}");
        assert!(run.stderr.contains("cargo-workplan:"), "{name}: {run:#?}");
    }
    // The neighbour: a stub that names the root.
    let stub = scratch.path().join("good");
    script(
        &stub,
        &format!(
            "echo '{{\"workspace_root\":\"{}\"}}'",
            repo.root().display()
        ),
    );
    let mut command = support::command(&support::elsewhere(), Shape::Direct, &["check"]);
    command.env("CARGO", &stub);
    support::finish(&mut command).expect(0);
}

#[test]
fn a_cargo_that_cannot_be_started_is_exit_two() {
    let _lock = serial();
    let repo = Repo::synced("process-no-cargo", false);
    let mut command = support::command(repo.root(), Shape::Direct, &["check"]);
    command.env("CARGO", repo.root().join("no-such-cargo"));
    let run = support::finish(&mut command).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("no-such-cargo"), "{run:#?}");
}

#[test]
fn a_missing_git_is_exit_two_and_names_it() {
    let _lock = serial();
    let repo = Repo::synced("process-no-git", false);
    let empty = Scratch::new("process-no-git-path");
    let target = repo.root().to_string_lossy().into_owned();
    for shape in [Shape::Direct, Shape::Cargo] {
        let mut command =
            support::command(&support::elsewhere(), shape, &["--root", &target, "check"]);
        command.env("PATH", empty.path());
        let run = support::finish(&mut command).expect(2);
        assert_eq!(run.stdout, "");
        assert!(run.stderr.contains("git"), "{run:#?}");
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
    // The neighbour: the configuration is live, a build of the workspace runs it.
    let build = Command::new(real_cargo())
        .args(["build", "--offline", "--quiet"])
        .current_dir(repo.root())
        .env("CARGO_TARGET_DIR", scratch.path().join("target"))
        .output()
        .expect("cargo build runs");
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(marker.exists(), "the configuration is not live");
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

    // The neighbour of each command: a slug that is a slug starts a process.
    for (head, tail, slug) in slug_commands() {
        let before = stubs.git_calls().len();
        let mut args = vec!["--root", &target];
        args.extend(&head);
        args.push(slug);
        args.extend(&tail);
        stubs.run(repo.root(), Shape::Direct, &args, true);
        assert!(
            stubs.git_calls().len() > before,
            "{args:?} started no process"
        );
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
fn old_git(path: &std::path::Path, record: &std::path::Path, version: &str) {
    script(
        path,
        &format!(
            "{{ for a in \"$@\"; do printf 'arg=%s\\n' \"$a\"; done; printf 'end\\n'; }} >> '{}'\n\
             if [ \"$1\" = --version ]; then echo 'git version {version}'; exit 0; fi\n\
             echo 'git: the stub answers only --version' >&2\nexit 1",
            record.display()
        ),
    );
}

#[test]
fn a_git_older_than_2_30_is_exit_two_before_any_other_process() {
    let _lock = serial();
    let repo = Repo::synced("process-old-git", false);
    let scratch = Scratch::new("process-old-git-stubs");
    let bin = scratch.path().join("bin");
    let record = scratch.path().join("git.record");
    old_git(&bin.join("git"), &record, "2.29.2");
    let cargo_record = scratch.path().join("cargo.record");
    let cargo = bin.join("cargo");
    script(
        &cargo,
        &format!("echo \"$@\" >> '{}'\nexit 1", cargo_record.display()),
    );
    let target = repo.root().to_string_lossy().into_owned();
    for shape in [Shape::Direct, Shape::Cargo] {
        for command in [["change", "reconcile"], ["change", "complete"]] {
            let before = fs::read_to_string(&record).unwrap_or_default();
            let mut run = support::command(
                &support::elsewhere(),
                shape,
                &["--root", &target, command[0], command[1], "grow"],
            );
            run.env("PATH", bin.clone()).env("CARGO", &cargo);
            let run = support::finish(&mut run).expect(2);
            assert_eq!(run.stdout, "", "{command:?}");
            assert!(run.stderr.contains("2.30"), "{command:?}: {run:#?}");
            let after = fs::read_to_string(&record).expect("a record");
            // One call, and it is `git --version`.
            assert_eq!(
                after[before.len()..],
                *"arg=--version\nend\n",
                "{command:?}"
            );
        }
    }
    assert!(!cargo_record.exists(), "cargo was started");

    // The neighbour: 2.30 passes the gate and the next question is asked.
    let newer = Scratch::new("process-new-git-stubs");
    let newer_record = newer.path().join("git.record");
    old_git(&newer.path().join("bin/git"), &newer_record, "2.30.0");
    let mut run = support::command(
        &support::elsewhere(),
        Shape::Direct,
        &["--root", &target, "change", "reconcile", "grow"],
    );
    run.env("PATH", newer.path().join("bin"))
        .env("CARGO", &cargo);
    support::finish(&mut run).expect(2);
    let calls = fs::read_to_string(&newer_record).expect("a record");
    assert!(calls.starts_with("arg=--version\nend\n"), "{calls}");
    assert!(calls.len() > "arg=--version\nend\n".len(), "{calls}");
}

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
        (vec!["--format", "json", "change", "reconcile", "grow"], 1),
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
