// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Test-owned Git repositories, written by the installed Git and read by RAF
//! only through its native library.

use std::path::Path;
use std::process::Command;

/// Run Git in `dir` with no user or system configuration, so hooks, signing
/// and ambient repository selectors cannot affect the fixture.
pub(crate) fn git(dir: &Path, args: &[&str]) -> String {
    let mut command = Command::new("git");
    for name in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_PREFIX",
    ] {
        command.env_remove(name);
    }
    let output = command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .args([
            "-c",
            "user.name=Example",
            "-c",
            "user.email=example@example.test",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
            "-C",
        ])
        .arg(dir)
        .args(args)
        .output()
        .expect("required Git");
    assert!(output.status.success(), "git {args:?}: {output:?}");
    String::from_utf8(output.stdout)
        .expect("UTF-8 Git output")
        .trim()
        .to_owned()
}

/// Create a repository at `dir`.
pub(crate) fn init(dir: &Path) {
    std::fs::create_dir_all(dir).expect("repository directory");
    git(dir, &["init", "--quiet"]);
}

/// Commit every file of the checkout and return the full commit ID.
pub(crate) fn commit_all(dir: &Path) -> String {
    git(dir, &["add", "--all"]);
    git(
        dir,
        &["commit", "--quiet", "--allow-empty", "-m", "fixture"],
    );
    git(dir, &["rev-parse", "HEAD"])
}
