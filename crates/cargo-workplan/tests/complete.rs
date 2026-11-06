// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Static completion through the shipped CLI preserves source and repository state.

mod support;
use support::{
    changed,
    completion::{CAP, DONE, RECORD, ready, shaped},
};

#[test]
fn completion_prints_the_owned_changes_and_leaves_a_clean_static_check() {
    let repo = ready("static-cli-completion");
    let before = repo.worktree();
    let run = repo.run(&["change", "complete", "shape"]).expect(0);
    assert!(
        run.stdout
            .starts_with("completed: removed work/changes/shape.md\n")
    );
    assert!(!repo.root().join(RECORD).exists());
    assert!(
        changed(&before, &repo.worktree())
            .iter()
            .all(|path| path == CAP || path == RECORD)
    );
    repo.both(&["check"]).expect(0);
}

#[test]
fn failing_consumer_bodies_do_not_prevent_structurally_valid_completion() {
    let repo = shaped(
        "static-failing-consumer",
        "delivered",
        "[demo::tests::fails]",
        "[demo::example_crate::exiting]",
        DONE,
    );
    let before = repo.worktree();
    repo.run(&["change", "complete", "shape"]).expect(0);
    for (path, bytes) in before {
        if std::path::Path::new(&path).extension() == Some(std::ffi::OsStr::new("rs"))
            || path.ends_with("Cargo.toml")
            || path.ends_with("Cargo.lock")
        {
            assert_eq!(repo.worktree().get(&path), Some(&bytes), "changed {path}");
        }
    }
}

#[test]
fn prospective_refusal_retains_the_record_and_every_input_byte() {
    let repo = shaped(
        "static-cli-refusal",
        "implementing",
        "[demo::tests::passes]",
        "[demo::example_crate::good]",
        DONE,
    );
    let before = repo.worktree();
    let run = repo.both(&["change", "complete", "shape"]).expect(1);
    assert!(run.stdout.contains("no-open-record"), "{run:?}");
    assert_eq!(repo.worktree(), before);
}
