// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Which pushed refs the pre-push hook checks and which it skips, and how it refuses
//! an object name that is not hexadecimal.
//!
//! Each test runs the real `.githooks/pre-push` in a fixture repository under the
//! test target directory. A checked line reaches the static gate. The gate exports the
//! empty tree, which holds no manifest, so it stops at once, and the line appears as a
//! `checking` message on stderr. A skipped line exits 0 with no such message. The
//! inputs are ref names and object names; nothing is executed from them, and no
//! workspace check runs.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const ZERO: &str = "0000000000000000000000000000000000000000";
/// The empty tree, which every repository holds. Its export has no manifest, so the
/// static gate refuses it at once. No commit or ref is created.
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const HOOK: &str = include_str!("../.githooks/pre-push");

/// Remote refs whose pushes carry metadata, not a source tree.
const SKIPPED: &[&str] = &[
    "refs/notes/ghprsq",
    "refs/notes/x/y",
    "refs/ghprsq/pr/455/head",
    "refs/tags/ghprsq/artifacts/851bd2cb3d9a87b49fa8eec75c8eafdb0dd41772",
];

/// Remote refs that are branches or tags, including look-alikes of the skipped
/// namespaces, which match only at their own separators.
const CHECKED: &[&str] = &[
    "refs/heads/main",
    "refs/tags/v0.9.0",
    "refs/tags/ghprsq/artifactsx",
    "refs/tags/ghprsq/other",
    "refs/tags/ghprsq/artifacts",
    "refs/notesx/foo",
    "refs/ghprsqx/foo",
    "refs/heads/refs/notes/x",
    "refs/tags/x/refs/ghprsq/y",
];

#[expect(
    clippy::expect_used,
    reason = "a git command that cannot start fails the fixture with the message"
)]
fn git(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("git starts");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// A fresh repository whose `core.hooksPath` is the real pre-push hook.
#[expect(
    clippy::expect_used,
    reason = "a fixture that cannot be written fails the test with the message"
)]
fn fixture(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("pre-push")
        .join(name);
    if root.exists() {
        fs::remove_dir_all(&root).expect("the previous fixture is removed");
    }
    let hooks = root.join(".githooks");
    fs::create_dir_all(&hooks).expect("the fixture hooks directory is created");
    fs::write(hooks.join("pre-push"), HOOK).expect("the fixture hook is written");
    git(&root, &["init", "-q"]);
    git(&root, &["config", "core.hooksPath", ".githooks"]);
    root
}

#[expect(
    clippy::expect_used,
    reason = "a hook that cannot start fails the test with the message"
)]
fn run_hook(root: &Path, pushed: &str) -> Output {
    let mut child = Command::new("bash")
        .arg(root.join(".githooks/pre-push"))
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("bash starts the fixture hook");
    child
        .stdin
        .take()
        .expect("the hook's stdin is piped")
        .write_all(pushed.as_bytes())
        .expect("the hook reads the pushed lines");
    child.wait_with_output().expect("the hook finishes")
}

/// One pushed line as git sends it: local ref, local object, remote ref, remote object.
fn pushed_line(local_ref: &str, local_sha: &str, remote_ref: &str) -> String {
    format!("{local_ref} {local_sha} {remote_ref} {ZERO}\n")
}

/// The local refs the hook announced it would check, read from its stderr.
fn checked(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .filter_map(|line| line.strip_prefix("pre-push: checking "))
        .filter_map(|rest| rest.split(" at ").next())
        .map(str::to_owned)
        .collect()
}

#[test]
fn metadata_and_deletion_destinations_are_skipped() {
    let root = fixture("skipped");
    for remote in SKIPPED {
        let output = run_hook(&root, &pushed_line("refs/heads/source", EMPTY_TREE, remote));
        assert!(output.status.success(), "{remote} is not refused");
        assert!(checked(&output).is_empty(), "{remote} runs no check");
    }
    let deletion = run_hook(&root, &format!("(delete) {ZERO} refs/heads/old {ZERO}\n"));
    assert!(deletion.status.success(), "a deletion is not refused");
    assert!(checked(&deletion).is_empty(), "a deletion runs no check");
}

#[test]
fn branch_tag_and_look_alike_destinations_reach_the_gate() {
    let root = fixture("checked");
    for remote in CHECKED {
        let output = run_hook(&root, &pushed_line("refs/heads/source", EMPTY_TREE, remote));
        assert!(
            !output.status.success(),
            "{remote} reaches the gate and is refused there"
        );
        assert_eq!(
            checked(&output),
            ["refs/heads/source"],
            "{remote} is checked"
        );
    }
}

#[test]
fn a_metadata_local_name_pushed_to_a_branch_is_checked() {
    // The destination decides: a source commit under a notes or ghprsq local name is
    // still checked when it lands on a branch.
    let root = fixture("local-names");
    for local in ["refs/notes/x", "refs/ghprsq/y"] {
        let output = run_hook(&root, &pushed_line(local, EMPTY_TREE, "refs/heads/main"));
        assert_eq!(checked(&output), [local], "{local} to main is checked");
    }
}

#[test]
fn a_mixed_push_checks_only_the_branch_in_either_order() {
    let root = fixture("mixed");
    let metadata = pushed_line("refs/notes/ghprsq", EMPTY_TREE, "refs/notes/ghprsq");
    let branch = pushed_line("refs/heads/main", EMPTY_TREE, "refs/heads/main");
    for pushed in [format!("{metadata}{branch}"), format!("{branch}{metadata}")] {
        let output = run_hook(&root, &pushed);
        assert_eq!(
            checked(&output),
            ["refs/heads/main"],
            "only the branch is checked in {pushed:?}"
        );
    }
}

#[test]
fn an_object_name_that_is_not_hexadecimal_is_refused_before_any_check() {
    // A value that begins with a dash would be read by git archive as an option, so
    // the hook accepts only a full hexadecimal object name.
    let root = fixture("object-name");
    for bad in [
        "--output=/nonexistent",
        "851bd2cb3d9a87b49fa8eec75c8eafdb0dd4177",
    ] {
        let output = run_hook(
            &root,
            &pushed_line("refs/heads/source", bad, "refs/heads/main"),
        );
        assert!(!output.status.success(), "{bad} is refused");
        assert!(
            checked(&output).is_empty(),
            "{bad} is refused before any check"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("is not an object name"),
            "the refusal names the object: {bad}"
        );
    }
}

#[test]
fn an_empty_push_checks_nothing() {
    let root = fixture("empty");
    let output = run_hook(&root, "");
    assert!(output.status.success(), "an empty push is not refused");
    assert!(checked(&output).is_empty(), "an empty push runs no check");
}
