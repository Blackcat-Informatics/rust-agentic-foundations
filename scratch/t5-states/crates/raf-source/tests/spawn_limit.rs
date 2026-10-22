// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A parse thread that cannot start is a typed `Spawn` error naming the file.
//!
//! The test binary runs itself in a child process whose address space is
//! limited (`ulimit -v`, set by `sh` before it executes the binary), so the
//! operating system refuses the parse thread its stack. A child without a limit
//! loads the workspace, as the control; children under limits far below the
//! stack of the parse thread must end in the typed refusal.

#![cfg(target_os = "linux")]

use std::env;
use std::process::{Command, Stdio};

use raf_source::{LoadErrorKind, MemoryTree, Workspace};

/// Set in a child; its value is the mode the child runs.
const MODE: &str = "RAF_SOURCE_SPAWN_MODE";
/// Marks the one result line of a child.
const MARK: &str = "RAF_SPAWN ";
/// The test a child runs.
const TEST: &str = "child_main";

const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

/// In a child: load a one-file workspace and print what became of it.
#[test]
fn child_main() {
    if env::var_os(MODE).is_none() {
        return;
    }
    let tree: MemoryTree = [("Cargo.toml", DEMO), ("src/lib.rs", "pub fn f() {}\n")]
        .into_iter()
        .collect();
    match Workspace::load(&tree, &["."]) {
        Ok(_) => println!("{MARK}loaded"),
        Err(error) => match &error.kind {
            LoadErrorKind::Spawn(_) => println!("{MARK}spawn {} | {error}", error.file),
            _ => println!("{MARK}other {error}"),
        },
    }
}

/// Run `child_main` as a child, under `limit_kib` of address space when given,
/// and return its result line, or `None` when the child printed none (the
/// system could not even start it under the limit).
fn run(limit_kib: Option<u64>) -> (Option<String>, String) {
    let executable = env::current_exe().expect("the test binary has a path");
    // `ulimit -v` applies to the shell and what it executes; the limit is the
    // first argument, the program and its arguments follow.
    let output = Command::new("sh")
        .args([
            "-c",
            "ulimit -v \"$1\" && shift && exec \"$@\"",
            "sh",
            &limit_kib.map_or_else(|| "unlimited".to_owned(), |kib| kib.to_string()),
        ])
        .arg(executable)
        .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
        .env(MODE, "1")
        .env_remove("RUST_MIN_STACK")
        .stdin(Stdio::null())
        .output()
        .expect("start the child");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let line = stdout
        .lines()
        .find_map(|line| line.split_once(MARK))
        .map(|(_, rest)| rest.to_owned());
    let detail = format!(
        "status {:?}; stdout: {stdout}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    (line, detail)
}

#[test]
fn a_parse_thread_the_system_refuses_a_stack_is_a_spawn_error_naming_the_file() {
    // The control: without a limit the same load succeeds.
    let (free, detail) = run(None);
    let free = free.unwrap_or_else(|| panic!("the unlimited child gave no result: {detail}"));
    assert!(free.starts_with("loaded"), "{free}");

    // Under a limit well below what the parse stack needs, the child either
    // cannot start at all or ends in the typed refusal; it must never load.
    // The address space a child maps at start varies with the limit it runs
    // under, so a few limits are tried and at least two must reach the load,
    // which a band of limits too narrow to start the binary cannot satisfy.
    let mib = 1024;
    let mut refused = 0;
    for limit in (16..=48).step_by(8).map(|steps| steps * mib) {
        let (line, detail) = run(Some(limit));
        let Some(line) = line else { continue };
        let (file, message) = line
            .strip_prefix("spawn ")
            .and_then(|rest| rest.split_once(" | "))
            .unwrap_or_else(|| panic!("limit {limit} KiB: not a spawn refusal: {line}"));
        assert_eq!(file, "src/lib.rs", "{detail}");
        assert!(message.contains("src/lib.rs"), "{message}");
        assert!(
            message.contains("cannot start the parse thread"),
            "{message}"
        );
        refused += 1;
    }
    assert!(
        refused >= 2,
        "only {refused} of the limits let the child reach the load"
    );
}
