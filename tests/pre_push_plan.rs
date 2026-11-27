// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Which lines of a push the pre-push source gates check.
//!
//! `.githooks/pre-push-plan` decides the lines; the hook runs the gates only on
//! the lines it plans. These tests feed the planner synthetic stdin in the form git
//! passes to a pre-push hook. The planner runs nothing on that input: the ref
//! names and object names are data.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

const ZERO: &str = "0000000000000000000000000000000000000000";
const SHA: &str = "851bd2cb3d9a87b49fa8eec75c8eafdb0dd41772";
const MAIN: &str = "2e21813266c551f3174794b0233a8eeb23af4248";

/// Runs `pre_push_plan` on `lines` and returns the planned lines.
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "a planner that cannot run or exits non-zero fails the test with its output"
)]
fn plan(lines: &[&str]) -> Vec<String> {
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
    let output = child.wait_with_output().expect("the planner finishes");
    if !output.status.success() {
        panic!(
            "pre_push_plan exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    String::from_utf8(output.stdout)
        .expect("the planner prints UTF-8")
        .lines()
        .map(str::to_owned)
        .collect()
}

#[test]
fn a_branch_or_tag_destination_is_checked_whatever_its_local_name() {
    // A source commit pushed from a local notes-named ref to main must be checked,
    // and so must a real tag and a plain branch.
    let planned = plan(&[
        &format!("refs/notes/x {MAIN} refs/heads/main"),
        &format!("refs/heads/y {SHA} refs/tags/v0.9.0"),
        &format!("refs/heads/z {SHA} refs/heads/feature"),
    ]);
    assert_eq!(
        planned,
        [
            format!("refs/notes/x {MAIN} refs/heads/main"),
            format!("refs/heads/y {SHA} refs/tags/v0.9.0"),
            format!("refs/heads/z {SHA} refs/heads/feature"),
        ]
    );
}

#[test]
fn a_metadata_destination_is_skipped() {
    // Notes, ghprsq audit refs and the ghprsq evidence tag namespace carry metadata.
    let planned = plan(&[
        &format!("refs/notes/ghprsq {SHA} refs/notes/ghprsq"),
        &format!("refs/heads/z {SHA} refs/ghprsq/pr/455/head"),
        &format!("refs/heads/z {SHA} refs/tags/ghprsq/artifacts/{SHA}"),
    ]);
    assert_eq!(planned, Vec::<String>::new());
}

#[test]
fn look_alike_destinations_are_checked() {
    // The namespaces match only at their own separators: a neighbouring name, and
    // another tag under ghprsq, are source destinations.
    let planned = plan(&[
        &format!("refs/heads/a {SHA} refs/tags/ghprsq/artifactsx"),
        &format!("refs/heads/a {SHA} refs/tags/ghprsq/other"),
        &format!("refs/heads/a {SHA} refs/tags/ghprsq/artifacts"),
        &format!("refs/heads/a {SHA} refs/notesx/foo"),
        &format!("refs/heads/a {SHA} refs/ghprsqx/foo"),
    ]);
    assert_eq!(
        planned.len(),
        5,
        "every look-alike line is planned: {planned:?}"
    );
}

#[test]
fn a_deletion_is_skipped() {
    // A deletion sends the zero object and no commit to check.
    let planned = plan(&[&format!("(delete) {ZERO} refs/heads/old {ZERO}")]);
    assert_eq!(planned, Vec::<String>::new());
}

#[test]
fn a_mixed_push_plans_each_line_alone_in_either_order() {
    // One push carries a metadata line and a branch line; only the branch line is
    // planned, whichever comes first.
    let notes = format!("refs/notes/ghprsq {SHA} refs/notes/ghprsq");
    let branch = format!("refs/heads/main {MAIN} refs/heads/main");
    assert_eq!(plan(&[&notes, &branch]), [branch.clone()]);
    assert_eq!(plan(&[&branch, &notes]), [branch]);
}
