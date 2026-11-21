// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Stub programs for the tests that look at the processes the binary starts:
//! scripts that record their arguments and then run the real program, or that
//! answer in place of it.

use std::ffi::OsString;
use std::fs;
use std::io::Write as _;
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard, PoisonError};

use super::{Run, Scratch, Shape};

/// A stub is written and started while other tests start processes; a forked
/// child briefly holding the descriptor that wrote the file makes the start
/// fail with "text file busy". Every test holds this lock from its first write
/// to its last spawn.
pub(crate) static SPAWN: Mutex<()> = Mutex::new(());

pub(crate) fn serial() -> MutexGuard<'static, ()> {
    SPAWN.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The first `name` on `PATH` that is a file.
pub(crate) fn which(name: &str) -> PathBuf {
    let path = std::env::var_os("PATH").expect("a PATH");
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| panic!("no `{name}` on PATH"))
}

/// The cargo of this test run.
pub(crate) fn real_cargo() -> PathBuf {
    std::env::var_os("CARGO").map_or_else(|| which("cargo"), PathBuf::from)
}

/// Write an executable script.
pub(crate) fn script(path: &Path, body: &str) {
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
pub(crate) fn recording(path: &Path, record: &Path, real: &Path) {
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
pub(crate) fn calls(record: &Path) -> Vec<Vec<String>> {
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
pub(crate) struct Stubs {
    pub(crate) scratch: Scratch,
    pub(crate) cargo_record: PathBuf,
    pub(crate) git_record: PathBuf,
}

impl Stubs {
    pub(crate) fn new(tag: &str) -> Self {
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

    pub(crate) fn bin(&self) -> PathBuf {
        self.scratch.path().join("bin")
    }

    /// `PATH` with the stubs in front.
    pub(crate) fn path(&self) -> OsString {
        let mut dirs = vec![self.bin()];
        dirs.extend(std::env::split_paths(
            &std::env::var_os("PATH").expect("a PATH"),
        ));
        std::env::join_paths(dirs).expect("a PATH")
    }

    /// The binary in `dir` with the stubs in front on `PATH` and `$CARGO` set
    /// to the cargo stub, or unset when `cargo_var` is false.
    pub(crate) fn command(
        &self,
        dir: &Path,
        shape: Shape,
        args: &[&str],
        cargo_var: bool,
    ) -> Command {
        let mut command = super::command(dir, shape, args);
        command.env("PATH", self.path());
        if cargo_var {
            command.env("CARGO", self.bin().join("cargo"));
        } else {
            command.env_remove("CARGO");
        }
        command
    }

    pub(crate) fn run(&self, dir: &Path, shape: Shape, args: &[&str], cargo_var: bool) -> Run {
        super::finish(&mut self.command(dir, shape, args, cargo_var))
    }

    pub(crate) fn cargo_calls(&self) -> Vec<Vec<String>> {
        calls(&self.cargo_record)
    }

    pub(crate) fn git_calls(&self) -> Vec<Vec<String>> {
        calls(&self.git_record)
    }
}
