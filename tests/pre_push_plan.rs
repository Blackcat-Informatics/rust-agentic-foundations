// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Which pushed lines the pre-push source gates check, and how the hook refuses
//! when its planner cannot decide every line.
//!
//! The planner tests feed `.githooks/pre-push-plan` synthetic lines in the form git
//! passes to a pre-push hook. The hook tests run the real `.githooks/pre-push` in a
//! fixture repository under the test target directory, with the real planner or a
//! broken one. Every push either is refused before the first check, plans only
//! metadata destinations, or reaches the static gate's manifest step for a commit
//! that holds no tree. No workspace check runs, and no payload is executed: the
//! inputs are ref names and object names.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const ZERO: &str = "0000000000000000000000000000000000000000";
const SHA: &str = "851bd2cb3d9a87b49fa8eec75c8eafdb0dd41772";
const MAIN: &str = "2e21813266c551f3174794b0233a8eeb23af4248";
/// The empty tree, which every repository holds. `git archive` exports it as a
/// tree with no files, so the static gate stops at its manifest step. No commit or
/// ref is created, so the fixture writes no history.
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const HOOK: &str = include_str!("../.githooks/pre-push");
const PLANNER: &str = include_str!("../.githooks/pre-push-plan");
const FAILING_PLANNER: &str = "pre_push_plan() {\n    return 1\n}\n";
const SILENT_PLANNER: &str = "pre_push_plan() {\n    :\n}\n";

#[expect(
    clippy::expect_used,
    reason = "a planner that cannot start fails the test with the message"
)]
fn planner_output(lines: &[&str]) -> Output {
    let planner = Path::new(env!("CARGO_MANIFEST_DIR")).join(".githooks/pre-push-plan");
    let mut child = Command::new("bash")
        .arg("-c")
        .arg("source \"$1\" && pre_push_plan")
        .arg("bash")
        .arg(&planner)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("bash starts the pre-push planner");
    {
        let mut stdin = child.stdin.take().expect("the planner's stdin is piped");
        for line in lines {
            writeln!(stdin, "{line}").expect("the planner reads a synthetic line");
        }
    }
    child.wait_with_output().expect("the planner finishes")
}

/// The planned decisions for `lines`, or the planner's stderr when it refuses.
fn plan(lines: &[&str]) -> Result<Vec<String>, String> {
    let output = planner_output(lines);
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let text = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    Ok(text.lines().map(str::to_owned).collect())
}

/// The planned decisions for `lines`. A planner that refuses fails the test.
#[expect(
    clippy::panic,
    reason = "the test fails with the planner's stderr when the planner refuses"
)]
fn decided(lines: &[&str]) -> Vec<String> {
    match plan(lines) {
        Ok(planned) => planned,
        Err(stderr) => panic!("the planner refused a line it should decide: {stderr}"),
    }
}

#[expect(
    clippy::expect_used,
    reason = "a git command that cannot start fails the fixture with the message"
)]
fn git(dir: &Path, args: &[&str]) -> String {
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
    String::from_utf8(output.stdout)
        .expect("git prints UTF-8")
        .trim()
        .to_owned()
}

/// A repository whose `.githooks` holds the real pre-push hook and the given planner
/// (`None` leaves the planner out). It is rebuilt on each call.
#[expect(
    clippy::expect_used,
    reason = "a fixture that cannot be written fails the test"
)]
fn fixture(name: &str, planner: Option<&str>) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("pre-push")
        .join(name);
    if root.exists() {
        fs::remove_dir_all(&root).expect("the previous fixture is removed");
    }
    let hooks = root.join(".githooks");
    fs::create_dir_all(&hooks).expect("the fixture hooks directory is created");
    fs::write(hooks.join("pre-push"), HOOK).expect("the fixture hook is written");
    if let Some(text) = planner {
        fs::write(hooks.join("pre-push-plan"), text).expect("the fixture planner is written");
    }
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

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn a_branch_or_tag_destination_is_checked_whatever_its_local_name() {
    // A source commit pushed from a local notes-named ref to main is checked, and so
    // are a real tag, a plain branch and a local ghprsq-named ref.
    let planned = decided(&[
        &format!("refs/notes/x {MAIN} refs/heads/main {ZERO}"),
        &format!("refs/heads/y {SHA} refs/tags/v0.9.0 {ZERO}"),
        &format!("refs/heads/z {SHA} refs/heads/feature {ZERO}"),
        &format!("refs/ghprsq/local {SHA} refs/heads/feature {ZERO}"),
    ]);
    assert_eq!(
        planned,
        [
            format!("check refs/notes/x {MAIN} refs/heads/main"),
            format!("check refs/heads/y {SHA} refs/tags/v0.9.0"),
            format!("check refs/heads/z {SHA} refs/heads/feature"),
            format!("check refs/ghprsq/local {SHA} refs/heads/feature"),
        ]
    );
}

#[test]
fn a_metadata_destination_is_skipped() {
    // Notes, ghprsq audit refs and the ghprsq evidence tag namespace carry metadata.
    let planned = decided(&[
        &format!("refs/notes/ghprsq {SHA} refs/notes/ghprsq {ZERO}"),
        &format!("refs/heads/z {SHA} refs/ghprsq/pr/455/head {ZERO}"),
        &format!("refs/heads/z {SHA} refs/tags/ghprsq/artifacts/{SHA} {ZERO}"),
    ]);
    assert_eq!(
        planned,
        [
            format!("skip refs/notes/ghprsq {SHA} refs/notes/ghprsq"),
            format!("skip refs/heads/z {SHA} refs/ghprsq/pr/455/head"),
            format!("skip refs/heads/z {SHA} refs/tags/ghprsq/artifacts/{SHA}"),
        ]
    );
}

#[test]
fn look_alike_destinations_are_checked() {
    // The namespaces match only at their own separators, at the start of the name:
    // neighbouring names, another tag under ghprsq, and a namespace nested inside
    // another name are all source destinations.
    let planned = decided(&[
        &format!("refs/heads/a {SHA} refs/tags/ghprsq/artifactsx {ZERO}"),
        &format!("refs/heads/a {SHA} refs/tags/ghprsq/other {ZERO}"),
        &format!("refs/heads/a {SHA} refs/tags/ghprsq/artifacts {ZERO}"),
        &format!("refs/heads/a {SHA} refs/notesx/foo {ZERO}"),
        &format!("refs/heads/a {SHA} refs/ghprsqx/foo {ZERO}"),
        &format!("refs/heads/a {SHA} refs/heads/refs/notes/x {ZERO}"),
        &format!("refs/heads/a {SHA} refs/tags/x/refs/ghprsq/y {ZERO}"),
    ]);
    assert!(
        planned.iter().all(|line| line.starts_with("check ")),
        "every look-alike line is checked: {planned:?}"
    );
    assert_eq!(planned.len(), 7);
}

#[test]
fn a_deletion_is_skipped() {
    // A deletion sends the zero object and no commit to check.
    let planned = decided(&[&format!("(delete) {ZERO} refs/heads/old {ZERO}")]);
    assert_eq!(planned, [format!("skip (delete) {ZERO} refs/heads/old")]);
}

#[test]
fn a_four_field_line_is_planned_by_its_first_three_fields() {
    // Git sends the remote object as the fourth field; the planner reads the first
    // three and ignores the rest.
    let planned = decided(&[&format!("refs/heads/x {SHA} refs/heads/main {MAIN}")]);
    assert_eq!(
        planned,
        [format!("check refs/heads/x {SHA} refs/heads/main")]
    );
}

#[test]
fn a_mixed_push_decides_each_line_alone_in_either_order() {
    // One push carries a metadata line and a branch line. Each line gets exactly one
    // decision, in input order, whichever line comes first.
    let notes = format!("refs/notes/ghprsq {SHA} refs/notes/ghprsq {ZERO}");
    let branch = format!("refs/heads/main {MAIN} refs/heads/main {ZERO}");
    assert_eq!(
        decided(&[&notes, &branch]),
        [
            format!("skip refs/notes/ghprsq {SHA} refs/notes/ghprsq"),
            format!("check refs/heads/main {MAIN} refs/heads/main"),
        ]
    );
    assert_eq!(
        decided(&[&branch, &notes]),
        [
            format!("check refs/heads/main {MAIN} refs/heads/main"),
            format!("skip refs/notes/ghprsq {SHA} refs/notes/ghprsq"),
        ]
    );
}

#[test]
fn a_malformed_line_makes_the_planner_refuse() {
    // A line with no remote ref cannot be decided, so the planner returns non-zero.
    let refused = plan(&[&format!("refs/heads/x {SHA}")]);
    assert!(refused.is_err(), "a malformed line is refused: {refused:?}");
}

#[test]
fn a_notes_destination_through_the_real_hook_is_not_checked() {
    let root = fixture("notes-skip", Some(PLANNER));
    let sha = EMPTY_TREE;
    let output = run_hook(
        &root,
        &format!("refs/notes/ghprsq {sha} refs/notes/ghprsq {ZERO}\n"),
    );
    assert!(
        output.status.success(),
        "a metadata push is not refused: {}",
        stderr_of(&output)
    );
    assert!(
        !stderr_of(&output).contains("checking"),
        "a metadata push runs no check: {}",
        stderr_of(&output)
    );
}

#[test]
fn a_branch_destination_through_the_real_hook_reaches_the_static_gate() {
    // The empty commit holds no Cargo.toml, so the gate refuses it at its manifest
    // step. The refusal shows that the line reached the gate.
    let root = fixture("branch-check", Some(PLANNER));
    let sha = EMPTY_TREE;
    let output = run_hook(
        &root,
        &format!("refs/heads/main {sha} refs/heads/main {ZERO}\n"),
    );
    assert!(!output.status.success(), "the empty commit is refused");
    assert!(
        stderr_of(&output).contains("checking refs/heads/main"),
        "the branch line reached the gate: {}",
        stderr_of(&output)
    );
}

#[test]
fn a_failing_planner_refuses_the_push_before_any_check() {
    let root = fixture("failing-planner", Some(FAILING_PLANNER));
    let sha = EMPTY_TREE;
    let output = run_hook(
        &root,
        &format!("refs/heads/main {sha} refs/heads/main {ZERO}\n"),
    );
    assert!(
        !output.status.success(),
        "a failing planner refuses the push"
    );
    assert!(
        !stderr_of(&output).contains("checking"),
        "no check runs after a failing planner: {}",
        stderr_of(&output)
    );
}

#[test]
fn a_planner_that_prints_nothing_for_a_branch_line_refuses_the_push() {
    // The planner decides none of the lines, so the counts differ and the hook
    // refuses before any check.
    let root = fixture("silent-planner", Some(SILENT_PLANNER));
    let sha = EMPTY_TREE;
    let output = run_hook(
        &root,
        &format!("refs/heads/main {sha} refs/heads/main {ZERO}\n"),
    );
    assert!(
        !output.status.success(),
        "a silent planner refuses the branch push"
    );
    assert!(
        stderr_of(&output).contains("decided 0 of 1"),
        "the refusal names the decision count: {}",
        stderr_of(&output)
    );
    assert!(
        !stderr_of(&output).contains("checking"),
        "no check runs after a silent planner: {}",
        stderr_of(&output)
    );
}

#[test]
fn a_missing_planner_refuses_the_push() {
    let root = fixture("missing-planner", None);
    let sha = EMPTY_TREE;
    let output = run_hook(
        &root,
        &format!("refs/heads/main {sha} refs/heads/main {ZERO}\n"),
    );
    assert!(
        !output.status.success(),
        "a missing planner refuses the push"
    );
    assert!(
        !stderr_of(&output).contains("checking"),
        "no check runs without a planner: {}",
        stderr_of(&output)
    );
}
