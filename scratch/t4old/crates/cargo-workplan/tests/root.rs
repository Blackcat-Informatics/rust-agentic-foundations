// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Root discovery through the built binary and the real `cargo`: the root is
//! found from a subdirectory with no `--root`, `--root` overrides the
//! discovery, and a directory outside any workspace is exit 2.

mod support;

use support::{Repo, Run, Shape, run_in};

/// The run of `args` from the directory `below` the root of the repository.
fn from_below(repo: &Repo, below: &str, shape: Shape, args: &[&str]) -> Run {
    run_in(&repo.root().join(below), shape, args)
}

#[test]
fn the_root_is_found_from_the_root_and_from_any_directory_below_it() {
    let repo = Repo::synced("root-below", true);
    let expected = repo.gate_lines();
    assert_ne!(expected.len(), 0);
    for below in [
        "",
        "src",
        "docs",
        "docs/capabilities",
        "work/changes",
        "tests",
    ] {
        for shape in [Shape::Direct, Shape::Cargo] {
            let run = from_below(&repo, below, shape, &["check"]).expect(1);
            assert_eq!(run.lines(), expected, "from `{below}` as {shape:?}");
        }
    }
}

#[test]
fn every_command_finds_the_root_from_a_subdirectory() {
    let repo = Repo::synced("root-commands", false);
    for shape in [Shape::Direct, Shape::Cargo] {
        let check = from_below(&repo, "src", shape, &["check"]).expect(0);
        assert_eq!(check.stdout, "");
        let status = from_below(&repo, "src", shape, &["status"]).expect(0);
        assert!(
            status.stdout.starts_with("delivered     1  alpha"),
            "{status:#?}"
        );
        let preview = from_below(&repo, "src", shape, &["docs", "preview", "grow"]).expect(0);
        assert!(
            preview.stdout.contains("Delta reads streams"),
            "{preview:#?}"
        );
        let sync = from_below(&repo, "src", shape, &["docs", "sync"]).expect(0);
        assert_eq!(sync.stdout, "");
    }
}

#[test]
fn sync_from_a_subdirectory_writes_at_the_root_and_nowhere_else() {
    let repo = Repo::roadmap("root-sync", false);
    let before = repo.snapshot();
    let run = from_below(
        &repo,
        "src",
        Shape::Direct,
        &[
            "docs",
            "sync",
            "--copyright",
            support::COPYRIGHT,
            "--licence",
            support::LICENCE,
        ],
    )
    .expect(0);
    assert_eq!(run.lines(), ["rewrote docs/capabilities/index.md"]);
    let after = repo.snapshot();
    let new: Vec<&String> = after
        .keys()
        .filter(|path| !before.contains_key(*path))
        .collect();
    assert_eq!(new, ["docs/capabilities/index.md"]);
}

#[test]
fn root_overrides_the_discovery_and_the_current_directory_is_not_read() {
    // The current directory is a clean workspace; `--root` names one with
    // findings. The findings are the ones of the named root.
    let clean = Repo::synced("root-override-clean", false);
    let broken = Repo::synced("root-override-broken", true);
    let expected = broken.gate_lines();
    assert_ne!(expected.len(), 0);
    let target = broken.root().to_string_lossy().into_owned();
    for shape in [Shape::Direct, Shape::Cargo] {
        let run = run_in(clean.root(), shape, &["--root", &target, "check"]).expect(1);
        assert_eq!(run.lines(), expected);
        // The neighbour: without `--root` the current directory is read.
        run_in(clean.root(), shape, &["check"]).expect(0);
    }
}

#[test]
fn a_relative_root_is_taken_from_the_current_directory() {
    let repo = Repo::synced("root-relative", true);
    let expected = repo.gate_lines();
    let run = run_in(
        &repo.root().join("src"),
        Shape::Direct,
        &["--root", "..", "check"],
    )
    .expect(1);
    assert_eq!(run.lines(), expected);
}

#[test]
fn a_directory_outside_any_workspace_is_exit_two_and_names_the_command() {
    let scratch = support::Scratch::new("root-outside");
    for shape in [Shape::Direct, Shape::Cargo] {
        let run = run_in(scratch.path(), shape, &["check"]).expect(2);
        assert_eq!(run.stdout, "");
        assert!(
            run.stderr.starts_with("cargo-workplan: ")
                && run
                    .stderr
                    .contains("cargo metadata --format-version 1 --no-deps --frozen"),
            "{run:#?}"
        );
        // `--root` is the way out, whatever the directory.
        let repo = Repo::synced("root-outside-root", false);
        let target = repo.root().to_string_lossy().into_owned();
        run_in(scratch.path(), shape, &["--root", &target, "check"]).expect(0);
    }
}
