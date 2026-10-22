// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A git checkout on a disk for the fixtures that read the merge base: a
//! repository whose first commit is a fixture repository, a remote-tracking
//! `origin/main` at that commit as a clone leaves it, and the helpers that
//! write files and commit.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use raf_workplan::exec::Programs;
use raf_workplan::lifecycle::{Lifecycle, LifecycleError};

use super::Repo;
use super::disk::{Scratch, put, write_repo};

/// A stub is written and started while other tests start `git`; a forked child
/// briefly holding the descriptor that wrote the file makes the start fail
/// with "text file busy". Every test that starts a process holds this lock
/// from its first write to its last spawn.
static SPAWN: Mutex<()> = Mutex::new(());

pub(crate) fn serial() -> MutexGuard<'static, ()> {
    SPAWN.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A directory that holds nothing, which stands in for the home and the
/// configuration directory of the user while a fixture runs `git`.
fn empty_home() -> &'static Path {
    static HOME: OnceLock<PathBuf> = OnceLock::new();
    HOME.get_or_init(|| {
        let dir = std::env::temp_dir().join("raf-workplan-empty-home");
        std::fs::create_dir_all(&dir).expect("an empty home directory");
        dir
    })
}

/// A `git` command for a fixture step in `dir`, cut off from the user: no
/// global, system or home configuration, no global ignore file (neither
/// `core.excludesFile` nor the default `$XDG_CONFIG_HOME/git/ignore`, which a
/// user may have set to hide `*.md` or `work/`), no hooks, no signing, and none
/// of the variables of an enclosing git command.
pub(crate) fn git_command(dir: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.excludesFile=",
        ])
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("HOME", empty_home())
        .env("USERPROFILE", empty_home())
        .env("XDG_CONFIG_HOME", empty_home())
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid");
    for variable in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_COMMON_DIR",
        "GIT_PREFIX",
    ] {
        command.env_remove(variable);
    }
    command
}

/// Run `git` in `dir` as a fixture step ([`git_command`]) and return standard
/// output exactly as git wrote it.
pub(crate) fn git_exact(dir: &Path, args: &[&str]) -> String {
    let output = git_command(dir).args(args).output().expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("UTF-8 output")
}

/// As [`git_exact`], without the last line ending of standard output.
pub(crate) fn git(dir: &Path, args: &[&str]) -> String {
    let text = git_exact(dir, args);
    text.strip_suffix('\n').unwrap_or(&text).to_string()
}

/// A repository on the branch `feature`, forked from `main` at the commit of
/// the fixture. `refs/remotes/origin/main` is at that commit and
/// `refs/remotes/origin/HEAD` names it, as in a clone.
pub(crate) struct Checkout {
    scratch: Scratch,
    /// The top of the work tree.
    top: PathBuf,
    /// The workspace: the top, or a directory below it.
    root: PathBuf,
}

impl Checkout {
    /// A checkout whose first commit holds the demo workspace and `repo`.
    pub(crate) fn new(tag: &str, repo: &Repo) -> Self {
        Self::below(tag, repo, "")
    }

    /// As [`Checkout::new`], with the workspace in the directory `below` of the
    /// work tree (`""` for the top, `"services/api"` for two levels down), so
    /// that `git` names its files from a directory above it.
    pub(crate) fn below(tag: &str, repo: &Repo, below: &str) -> Self {
        let scratch = Scratch::new(tag);
        let top = scratch.path().join("work");
        let root = if below.is_empty() {
            top.clone()
        } else {
            top.join(below)
        };
        std::fs::create_dir_all(&root).expect("a work directory");
        write_repo(&root, repo);
        git(&top, &["init", "-q", "-b", "main"]);
        git(&top, &["add", "-A"]);
        git(&top, &["commit", "-q", "-m", "base"]);
        let checkout = Self { scratch, top, root };
        checkout.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        checkout.git(&[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ]);
        checkout.git(&["checkout", "-q", "-b", "feature"]);
        checkout
    }

    /// The root of the workspace.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// The top of the work tree: the root, or a directory above it.
    pub(crate) fn top(&self) -> &Path {
        &self.top
    }

    /// The directory the checkout and its neighbours live in.
    pub(crate) fn scratch(&self) -> &Path {
        self.scratch.path()
    }

    /// Run `git` at the top of the work tree.
    pub(crate) fn git(&self, args: &[&str]) -> String {
        git(&self.top, args)
    }

    /// Write a file below the root.
    pub(crate) fn put(&self, path: &str, text: &str) {
        put(&self.root, path, text);
    }

    /// Write the files of `repo`, over what is there.
    pub(crate) fn put_all(&self, repo: &Repo) {
        for (path, text) in repo {
            self.put(path, text);
        }
    }

    /// Add everything and commit.
    pub(crate) fn commit(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", message]);
    }

    /// The lifecycle of the checkout, over the real `git` and a `cargo` that
    /// does not exist, so a fixture that starts `cargo` fails as an error.
    pub(crate) fn lifecycle(&self) -> Result<Lifecycle, LifecycleError> {
        Lifecycle::open(self.programs())
    }

    /// The programs of the checkout: the real `git`, no `cargo`.
    pub(crate) fn programs(&self) -> Programs {
        Programs::new(&self.root).with_cargo(self.scratch.path().join("no-such-cargo"))
    }
}
