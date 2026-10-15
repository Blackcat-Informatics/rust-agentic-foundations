// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A real project in a real git repository, for the scenario fixtures: a
//! cargo crate with its own `[workspace]`, a template `Cargo.lock`, no
//! dependency and a pinned edition, the documents that bind it, and a git
//! history with a remote-tracking default branch.
//!
//! Every project of a test binary builds into one target directory, named in
//! the project's own `.cargo/config.toml` so the environment of the test
//! process is not touched. `cargo` is the one that started the test binary
//! (`$CARGO`), given to the library explicitly. Nothing here is a shell
//! script, so the fixtures run on every platform the workspace tests run on.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use raf_workplan::exec::Programs;
use raf_workplan::lifecycle::Lifecycle;
use raf_workplan::{Finding, Gate, SyncOptions, Workplan};

use super::checkout::{git, git_exact};
use super::disk::{Scratch, put, read, snapshot};

/// No project may take longer than this, builds included.
pub(crate) const BUDGET: Duration = Duration::from_secs(300);

/// The copyright text of the SPDX lines the fixtures write.
pub(crate) const COPYRIGHT: &str = "2026 Example Org <dev@example.org>";

/// The licence identifier of the SPDX lines the fixtures write.
pub(crate) const LICENCE: &str = "CC-BY-4.0";

/// The `cargo` that runs this test binary: `$CARGO` as `cargo test` sets it.
pub(crate) fn cargo() -> PathBuf {
    std::env::var_os("CARGO")
        .map(PathBuf::from)
        .or_else(|| option_env!("CARGO").map(PathBuf::from))
        .expect("cargo test sets CARGO")
}

/// The target directory every project of this test binary builds into.
fn shared_target() -> &'static Path {
    static TARGET: OnceLock<PathBuf> = OnceLock::new();
    TARGET.get_or_init(|| {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("scenario-fixtures");
        std::fs::create_dir_all(&dir).expect("the shared target directory");
        dir
    })
}

/// The two SPDX comment lines of a Markdown file, without the tags written
/// out in this source.
pub(crate) fn spdx_comments() -> String {
    format!(
        "<!-- {}{COPYRIGHT} -->\n<!-- {}{LICENCE} -->\n",
        concat!("SPDX-", "FileCopyrightText: "),
        concat!("SPDX-", "License-Identifier: "),
    )
}

/// A project in a scratch directory, removed when dropped.
pub(crate) struct Project {
    scratch: Scratch,
    started: Instant,
}

impl Project {
    /// A project holding the crate `package` (edition 2021, no dependency)
    /// and `files`, as a git repository on `main` with nothing committed.
    pub(crate) fn create(tag: &str, package: &str, files: &[(String, String)]) -> Self {
        let scratch = Scratch::new(tag);
        let root = scratch.path();
        put(
            root,
            "Cargo.toml",
            &format!(
                "[package]\nname = \"{package}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n"
            ),
        );
        put(
            root,
            "Cargo.lock",
            &format!("version = 4\n\n[[package]]\nname = \"{package}\"\nversion = \"0.1.0\"\n"),
        );
        put(
            root,
            ".cargo/config.toml",
            &format!("[build]\ntarget-dir = '{}'\n", shared_target().display()),
        );
        for (path, text) in files {
            put(root, path, text);
        }
        git(root, &["init", "-q", "-b", "main"]);
        Self {
            scratch,
            started: Instant::now(),
        }
    }

    /// The root of the project and of its git work tree.
    pub(crate) fn root(&self) -> &Path {
        self.scratch.path()
    }

    /// Run `git` at the root.
    pub(crate) fn git(&self, args: &[&str]) -> String {
        git(self.root(), args)
    }

    /// Write a file.
    pub(crate) fn put(&self, path: &str, text: &str) {
        put(self.root(), path, text);
    }

    /// The text of a file.
    pub(crate) fn text(&self, path: &str) -> String {
        read(self.root(), path)
    }

    /// Whether a file is there.
    pub(crate) fn exists(&self, path: &str) -> bool {
        self.root().join(path).exists()
    }

    /// Remove a file.
    pub(crate) fn remove(&self, path: &str) {
        std::fs::remove_file(self.root().join(path))
            .unwrap_or_else(|error| panic!("cannot remove {path}: {error}"));
    }

    /// Replace the first `from` in the file `path` by `to`.
    pub(crate) fn edit(&self, path: &str, from: &str, to: &str) {
        let text = self.text(path);
        assert!(text.contains(from), "{path} has no `{from}`");
        self.put(path, &text.replacen(from, to, 1));
    }

    /// Every file below the root except the git directory, with its bytes.
    pub(crate) fn snapshot(&self) -> BTreeMap<String, Vec<u8>> {
        snapshot(self.root())
            .into_iter()
            .filter(|(path, _)| !path.starts_with(".git/"))
            .collect()
    }

    /// The `docs sync` of the documents as they are: written to the disk.
    pub(crate) fn sync(&self) {
        let opened = Gate::workspace(self.root())
            .open()
            .unwrap_or_else(|error| panic!("the project opens: {error}"));
        let report = opened
            .workplan()
            .sync(&SyncOptions::new(COPYRIGHT, LICENCE))
            .unwrap_or_else(|error| panic!("the project syncs: {error}"));
        Workplan::write(&report, self.root()).expect("the sync is written");
    }

    /// Every finding of `check`, judged against the files git tracks.
    pub(crate) fn findings(&self) -> Vec<Finding> {
        Gate::workspace(self.root()).check()
    }

    /// Stage everything, require that the repository passes its gate as the
    /// staged tree (what a pre-commit hook judges), and commit. Returns the
    /// commit id.
    pub(crate) fn commit(&self, message: &str) -> String {
        self.git(&["add", "-A"]);
        let findings = self.findings();
        assert!(
            findings.is_empty(),
            "the tree to commit for `{message}` has findings: {findings:#?}"
        );
        self.git(&["commit", "-q", "-m", message]);
        self.git(&["rev-parse", "HEAD"])
    }

    /// Make the current commit the default branch of `origin`, as a clone
    /// leaves it: `refs/remotes/origin/main`, and `refs/remotes/origin/HEAD`
    /// naming it.
    pub(crate) fn publish(&self) {
        self.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        self.git(&[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ]);
    }

    /// Start `branch` at the current commit and switch to it.
    pub(crate) fn fork(&self, branch: &str) {
        self.git(&["checkout", "-q", "-b", branch]);
    }

    /// Fast-forward `main` to `branch`, switch to it and publish it.
    pub(crate) fn land(&self, branch: &str) {
        self.git(&["checkout", "-q", "main"]);
        self.git(&["merge", "-q", "--ff-only", branch]);
        self.publish();
    }

    /// The file `path` as it is at `revision`, byte for byte.
    pub(crate) fn show(&self, revision: &str, path: &str) -> String {
        git_exact(self.root(), &["show", &format!("{revision}:{path}")])
    }

    /// The tracked paths at `revision`, sorted.
    pub(crate) fn tracked_at(&self, revision: &str) -> Vec<String> {
        self.git(&["ls-tree", "-r", "--name-only", revision])
            .lines()
            .map(str::to_string)
            .collect()
    }

    /// Check out `revision` as a second work tree at `into`, which the caller
    /// owns and removes.
    pub(crate) fn checkout(&self, revision: &str, into: &Path) {
        let target = into.to_string_lossy().into_owned();
        self.git(&["worktree", "add", "-q", "--detach", &target, revision]);
    }

    /// The programs of the project: the real `git` and the `cargo` of this
    /// test binary, named explicitly.
    pub(crate) fn programs(&self) -> Programs {
        Programs::with_cargo_var(self.root(), Some(cargo().into()))
    }

    /// The lifecycle of the project as it is now. The open freezes the source
    /// index, so a lifecycle is opened again after each edit.
    pub(crate) fn lifecycle(&self) -> Lifecycle {
        Lifecycle::open(self.programs())
            .unwrap_or_else(|error| panic!("the project opens: {error}"))
    }

    /// Require that the project, builds included, is within its budget.
    pub(crate) fn within_budget(&self) {
        assert!(
            self.started.elapsed() < BUDGET,
            "the project took {:?}, over its budget of {BUDGET:?}",
            self.started.elapsed()
        );
    }

    /// The time since the project was created.
    pub(crate) fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
}
