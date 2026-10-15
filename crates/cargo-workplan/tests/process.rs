// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The processes the binary starts, seen through stub programs: the exact
//! command of root discovery from `$CARGO` and from `PATH`, a slug that starts
//! with `-` refused before any process starts, a missing `git` and a
//! `build.rustc` program that discovery does not run.

#![cfg(unix)]

mod support;

use std::ffi::OsString;
use std::fs;
use std::io::Write as _;
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard, PoisonError};

use support::{Repo, Run, Scratch, Shape};

/// A stub is written and started while other tests start processes; a forked
/// child briefly holding the descriptor that wrote the file makes the start
/// fail with "text file busy". Every test holds this lock from its first write
/// to its last spawn.
static SPAWN: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SPAWN.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The first `name` on `PATH` that is a file.
fn which(name: &str) -> PathBuf {
    let path = std::env::var_os("PATH").expect("a PATH");
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| panic!("no `{name}` on PATH"))
}

/// The cargo of this test run.
fn real_cargo() -> PathBuf {
    std::env::var_os("CARGO").map_or_else(|| which("cargo"), PathBuf::from)
}

/// Write an executable script.
fn script(path: &Path, body: &str) {
    fs::create_dir_all(path.parent().expect("a directory")).expect("a directory");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o755)
        .open(path)
        .expect("a script file");
    write!(file, "#!/bin/sh\n{body}\n").expect("a script body");
}

/// A stub that appends its working directory and one `arg=` line for each
/// argument to `record`, then runs `real` with the same arguments.
fn recording(path: &Path, record: &Path, real: &Path) {
    script(
        path,
        &format!(
            "{{ printf 'cwd=%s\\n' \"$PWD\"; for a in \"$@\"; do printf 'arg=%s\\n' \"$a\"; done; \
             printf 'end\\n'; }} >> '{}'\nexec '{}' \"$@\"",
            record.display(),
            real.display()
        ),
    );
}

/// The arguments of each recorded call, in order.
fn calls(record: &Path) -> Vec<Vec<String>> {
    let Ok(text) = fs::read_to_string(record) else {
        return Vec::new();
    };
    let mut calls = Vec::new();
    let mut current = Vec::new();
    for line in text.lines() {
        if line == "end" {
            calls.push(std::mem::take(&mut current));
        } else if let Some(argument) = line.strip_prefix("arg=") {
            current.push(argument.to_string());
        }
    }
    calls
}

/// A directory of stubs for `cargo` (a recording one) and `git` (a recording
/// one), the files they record to, and the environment that puts them in
/// front.
struct Stubs {
    scratch: Scratch,
    cargo_record: PathBuf,
    git_record: PathBuf,
}

impl Stubs {
    fn new(tag: &str) -> Self {
        let scratch = Scratch::new(tag);
        let bin = scratch.path().join("bin");
        let cargo_record = scratch.path().join("cargo.record");
        let git_record = scratch.path().join("git.record");
        recording(&bin.join("cargo"), &cargo_record, &real_cargo());
        recording(&bin.join("git"), &git_record, &which("git"));
        Self {
            scratch,
            cargo_record,
            git_record,
        }
    }

    fn bin(&self) -> PathBuf {
        self.scratch.path().join("bin")
    }

    /// `PATH` with the stubs in front.
    fn path(&self) -> OsString {
        let mut dirs = vec![self.bin()];
        dirs.extend(std::env::split_paths(
            &std::env::var_os("PATH").expect("a PATH"),
        ));
        std::env::join_paths(dirs).expect("a PATH")
    }

    /// The binary in `dir` with the stubs in front on `PATH` and `$CARGO` set
    /// to the cargo stub, or unset when `cargo_var` is false.
    fn command(&self, dir: &Path, shape: Shape, args: &[&str], cargo_var: bool) -> Command {
        let mut command = support::command(dir, shape, args);
        command.env("PATH", self.path());
        if cargo_var {
            command.env("CARGO", self.bin().join("cargo"));
        } else {
            command.env_remove("CARGO");
        }
        command
    }

    fn run(&self, dir: &Path, shape: Shape, args: &[&str], cargo_var: bool) -> Run {
        support::finish(&mut self.command(dir, shape, args, cargo_var))
    }

    fn cargo_calls(&self) -> Vec<Vec<String>> {
        calls(&self.cargo_record)
    }

    fn git_calls(&self) -> Vec<Vec<String>> {
        calls(&self.git_record)
    }
}

const METADATA: [&str; 4] = ["metadata", "--format-version", "1", "--no-deps"];

fn metadata_argv() -> Vec<String> {
    let mut argv: Vec<String> = METADATA.iter().map(ToString::to_string).collect();
    argv.push("--frozen".to_string());
    argv
}

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
