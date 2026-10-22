// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The git questions against the real `git` in repositories written to a
//! scratch directory (the tracked paths, the default branch, the merge base,
//! a file at the merge base, a linked work tree), and against a stub `git` that
//! records exactly what it was started with.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use raf_okf::MAX_TEXT_BYTES;
use raf_workplan::exec::{Commit, Programs, RefName, SourcePath, SystemRunner};
use raf_workplan::git::{
    GitError, GitVersion, MergeBase, blob_at, default_branch_ref, merge_base, require_version,
    resolve_commit, show_prefix, tracked_paths,
};

/// A stub is written and started while other tests start `git`; a forked child
/// briefly holding the descriptor that wrote the file makes the start fail with
/// "text file busy". Every test holds this lock from its first process to its
/// last, so no two tests start processes at once.
static SPAWN: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SPAWN.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A scratch directory, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        // The name is the process id and a counter; a directory left by an
        // earlier process of the same id is skipped, not reused.
        let dir = loop {
            let dir = std::env::temp_dir().join(format!(
                "raf-workplan-git-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&dir) {
                Ok(()) => break dir,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("a scratch directory: {error}"),
            }
        };
        Self(fs::canonicalize(dir).expect("a canonical scratch directory"))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Run `git` in `dir` as a fixture step: the variables of an enclosing git
/// command and the user's configuration are out of the way, so a step never
/// touches the repository of a hook that runs the test. Repository-local hooks and
/// signing settings remain in force. Returns standard output without its last
/// line ending.
fn git(dir: &Path, args: &[&str]) -> String {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(dir)
        .args(["-c", "core.excludesFile="])
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        // The user's own ignore file (under the config home) must not hide fixture files.
        .env("XDG_CONFIG_HOME", dir)
        .env("HOME", dir)
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
    let output = command.output().expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).expect("UTF-8 output");
    text.strip_suffix('\n').unwrap_or(&text).to_string()
}

/// A work tree with a repository in it.
struct Repo {
    dir: PathBuf,
}

impl Repo {
    /// A new repository on the branch `main` in `dir`, which is created.
    fn init(dir: PathBuf) -> Self {
        fs::create_dir_all(&dir).expect("a repository directory");
        git(&dir, &["init", "-q", "-b", "main"]);
        Self { dir }
    }

    fn put(&self, path: &str, bytes: &[u8]) {
        let target = self.dir.join(path);
        fs::create_dir_all(target.parent().expect("a file has a directory"))
            .expect("the directory of a file");
        fs::write(target, bytes).expect("a file");
    }

    fn git(&self, args: &[&str]) -> String {
        git(&self.dir, args)
    }

    /// Write the file, add everything, commit and return the commit id.
    fn commit(&self, path: &str, bytes: &[u8], message: &str) -> String {
        self.put(path, bytes);
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", message]);
        self.git(&["rev-parse", "HEAD"])
    }

    fn runner(&self) -> SystemRunner {
        runner_in(&self.dir)
    }
}

fn runner_in(dir: &Path) -> SystemRunner {
    SystemRunner::new(Programs::new(dir))
}

fn commit(text: &str) -> Commit {
    Commit::new(text).expect("a commit id")
}

fn path(text: &str) -> SourcePath {
    SourcePath::new(text).expect("a source path")
}

/// The merge base of `HEAD` of `repo` and the commit `ancestor`, which is the
/// only way to get a revision to read a file at.
fn base_at(repo: &Repo, ancestor: &str) -> MergeBase {
    merge_base(&repo.runner(), &commit(ancestor)).expect("a merge base")
}

/// A repository cloned from a bare origin, so that `refs/remotes/origin/HEAD`
/// names the default branch as a clone sets it. Returns the clone and the
/// commit of `main` at the clone.
fn clone_of_origin(scratch: &Scratch) -> (Repo, String) {
    let seed = Repo::init(scratch.path().join("seed"));
    let first = seed.commit("README.md", b"# Seed\n", "first");
    let bare = scratch.path().join("origin.git");
    git(
        scratch.path(),
        &["clone", "-q", "--bare", "seed", "origin.git"],
    );
    assert!(bare.is_dir());
    git(scratch.path(), &["clone", "-q", "origin.git", "work"]);
    let work = Repo {
        dir: scratch.path().join("work"),
    };
    (work, first)
}

// ------------------------------------------------------------- tracked paths

#[test]
fn the_tracked_paths_are_the_index_and_not_the_files_on_disk() {
    let _lock = serial();
    let scratch = Scratch::new();
    let repo = Repo::init(scratch.path().join("repo"));
    repo.put("docs/guide.md", b"# Guide\n");
    repo.put("docs/a b.md", b"spaces\n");
    repo.put("docs/\u{e9}t\u{e9}.md", b"unicode\n");
    repo.put("src/lib.rs", b"");
    repo.git(&["add", "-A"]);
    repo.put("untracked.md", b"not added\n");
    let paths = tracked_paths(&repo.runner())
        .expect("a listing")
        .expect("inside a work tree");
    let names: Vec<&str> = paths.iter().map(raf_source::RelPath::as_str).collect();
    assert_eq!(
        names,
        [
            "docs/a b.md",
            "docs/guide.md",
            "docs/\u{e9}t\u{e9}.md",
            "src/lib.rs"
        ]
    );
}

#[test]
fn the_tracked_paths_of_a_subdirectory_are_relative_to_it() {
    let _lock = serial();
    let scratch = Scratch::new();
    let repo = Repo::init(scratch.path().join("repo"));
    repo.put("top.md", b"top\n");
    repo.put("sub/inner.md", b"inner\n");
    repo.put("sub/deep/leaf.md", b"leaf\n");
    repo.git(&["add", "-A"]);
    let paths = tracked_paths(&runner_in(&repo.dir.join("sub")))
        .expect("a listing")
        .expect("inside a work tree");
    let names: Vec<&str> = paths.iter().map(raf_source::RelPath::as_str).collect();
    assert_eq!(names, ["deep/leaf.md", "inner.md"]);
}

#[test]
fn a_directory_that_is_no_work_tree_has_no_listing_and_a_broken_index_is_an_error() {
    let _lock = serial();
    let scratch = Scratch::new();
    // Outside any repository: the documented fallback.
    let loose = scratch.path().join("loose");
    fs::create_dir(&loose).expect("a directory");
    assert_eq!(tracked_paths(&runner_in(&loose)), Ok(None));

    // Inside the `.git` directory: git answers `false`.
    let repo = Repo::init(scratch.path().join("repo"));
    repo.commit("a.md", b"a\n", "first");
    assert_eq!(tracked_paths(&runner_in(&repo.dir.join(".git"))), Ok(None));

    // A neighbour inside the work tree lists.
    assert_eq!(
        tracked_paths(&repo.runner())
            .expect("a listing")
            .map(|paths| paths.len()),
        Some(1)
    );

    // A work tree whose index cannot be read is an error, not a walk.
    fs::write(repo.dir.join(".git/index"), "this is not an index").expect("a bad index");
    let error = tracked_paths(&repo.runner()).expect_err("a corrupt index");
    assert!(matches!(error, GitError::Failed { .. }), "{error:?}");
    assert!(error.to_string().contains("git ls-files"), "{error}");
}

#[test]
fn a_program_that_does_not_exist_is_an_error_and_not_a_fallback() {
    let _lock = serial();
    let scratch = Scratch::new();
    let programs = Programs::new(scratch.path()).with_git(scratch.path().join("no-such-git"));
    let error = tracked_paths(&SystemRunner::new(programs.clone())).expect_err("no git");
    assert!(matches!(error, GitError::Exec(_)), "{error:?}");
    assert!(error.to_string().contains("cannot start"), "{error}");
    let error = require_version(&SystemRunner::new(programs)).expect_err("no git");
    assert!(matches!(error, GitError::Exec(_)), "{error:?}");
}

// ----------------------------------------------------------------- version

#[test]
fn the_installed_git_is_new_enough() {
    let _lock = serial();
    let scratch = Scratch::new();
    let version = require_version(&runner_in(scratch.path())).expect("a supported git");
    assert!(version >= GitVersion::MINIMUM, "{version}");
}

// ----------------------------------------------------------- default branch

#[test]
fn the_default_branch_of_a_clone_is_the_full_remote_tracking_name() {
    let _lock = serial();
    let scratch = Scratch::new();
    let (work, first) = clone_of_origin(&scratch);
    let name = default_branch_ref(&work.runner()).expect("a default branch");
    assert_eq!(name.as_str(), "refs/remotes/origin/main");
    let default = resolve_commit(&work.runner(), &name).expect("a commit");
    assert_eq!(default.as_str(), first);
    // On the default branch itself the merge base is `HEAD`.
    let base = merge_base(&work.runner(), &default).expect("a merge base");
    assert_eq!(base.as_str(), first);
}

#[test]
fn a_repository_without_origin_head_has_no_default_branch_and_the_fix_is_named() {
    let _lock = serial();
    let scratch = Scratch::new();
    // No remote at all.
    let alone = Repo::init(scratch.path().join("alone"));
    alone.commit("a.md", b"a\n", "first");
    let error = default_branch_ref(&alone.runner()).expect_err("no remote");
    assert!(
        matches!(error, GitError::NoDefaultBranch { .. }),
        "{error:?}"
    );
    let message = error.to_string();
    assert!(
        message.contains("git remote set-head origin --auto"),
        "{message}"
    );
    assert!(message.contains("no default branch"), "{message}");

    // A remote whose `origin/HEAD` was removed, then restored by the fix the
    // message names.
    let (work, _) = clone_of_origin(&scratch);
    work.git(&["remote", "set-head", "origin", "-d"]);
    assert!(matches!(
        default_branch_ref(&work.runner()),
        Err(GitError::NoDefaultBranch { .. })
    ));
    work.git(&["remote", "set-head", "origin", "--auto"]);
    assert_eq!(
        default_branch_ref(&work.runner())
            .expect("restored")
            .as_str(),
        "refs/remotes/origin/main"
    );

    // A `refs/remotes/origin/HEAD` that is a plain ref and not a symbolic one
    // names no branch.
    let plain = Repo::init(scratch.path().join("plain"));
    let tip = plain.commit("a.md", b"a\n", "first");
    plain.git(&["update-ref", "refs/remotes/origin/HEAD", &tip]);
    assert!(matches!(
        default_branch_ref(&plain.runner()),
        Err(GitError::NoDefaultBranch { .. })
    ));
}

#[test]
fn a_local_branch_named_origin_main_does_not_stand_in_for_the_remote_one() {
    let _lock = serial();
    let scratch = Scratch::new();
    let (work, remote_tip) = clone_of_origin(&scratch);
    // A branch literally named `origin/main`, one commit ahead.
    work.git(&["checkout", "-q", "-b", "origin/main"]);
    let local_tip = work.commit("local.md", b"local\n", "local");
    assert_ne!(local_tip, remote_tip);
    // The short name now means the local branch.
    assert_eq!(work.git(&["rev-parse", "origin/main"]), local_tip);
    // The full name the default branch is read as does not.
    let name = default_branch_ref(&work.runner()).expect("a default branch");
    assert_eq!(name.as_str(), "refs/remotes/origin/main");
    let resolved = resolve_commit(&work.runner(), &name).expect("a commit");
    assert_eq!(resolved.as_str(), remote_tip);
    // And the merge base of the local branch with it is the remote tip.
    assert_eq!(
        merge_base(&work.runner(), &resolved)
            .expect("a merge base")
            .as_str(),
        remote_tip
    );
}

#[test]
fn a_default_branch_named_like_an_option_is_a_ref_and_never_an_option() {
    let _lock = serial();
    let scratch = Scratch::new();
    let repo = Repo::init(scratch.path().join("repo"));
    let tip = repo.commit("a.md", b"a\n", "first");
    repo.git(&["update-ref", "refs/remotes/origin/--output=x", &tip]);
    repo.git(&[
        "symbolic-ref",
        "refs/remotes/origin/HEAD",
        "refs/remotes/origin/--output=x",
    ]);
    let name = default_branch_ref(&repo.runner()).expect("a default branch");
    assert_eq!(name.as_str(), "refs/remotes/origin/--output=x");
    let resolved = resolve_commit(&repo.runner(), &name).expect("resolved as a ref");
    assert_eq!(resolved.as_str(), tip);
    assert_eq!(
        merge_base(&repo.runner(), &resolved)
            .expect("a merge base")
            .as_str(),
        tip
    );
    // Nothing was written as `x`: it was never read as `--output=x`.
    assert!(!repo.dir.join("x").exists());
    assert!(!std::env::current_dir().expect("a cwd").join("x").exists());

    // The neighbour: a name that is an option on its own is no reference name,
    // so it cannot be asked about.
    assert!(RefName::new("--output=x").is_err());
    assert!(RefName::new("-h").is_err());
}

// -------------------------------------------------------------- merge base

/// A clone with `main` at two commits and a feature branch of its own that
/// forked from the first. Returns the clone, the fork point and the tip of
/// `main` at the remote.
fn forked(scratch: &Scratch) -> (Repo, String, String) {
    let (work, fork) = clone_of_origin(scratch);
    work.commit("shared.md", b"shared\n", "second on main");
    work.git(&["push", "-q", "origin", "main"]);
    let main_tip = work.git(&["rev-parse", "HEAD"]);
    work.git(&["checkout", "-q", "-b", "feature", &fork]);
    work.commit("feature.md", b"feature\n", "on the feature branch");
    (work, fork, main_tip)
}

#[test]
fn the_merge_base_is_where_the_branch_left_the_default_branch() {
    let _lock = serial();
    let scratch = Scratch::new();
    let (work, fork, main_tip) = forked(&scratch);
    let name = default_branch_ref(&work.runner()).expect("a default branch");
    let default = resolve_commit(&work.runner(), &name).expect("a commit");
    assert_eq!(default.as_str(), main_tip);
    let base = merge_base(&work.runner(), &default).expect("a merge base");
    assert_eq!(base.as_str(), fork);
    assert_ne!(base.as_str(), work.git(&["rev-parse", "HEAD"]));
}

#[test]
fn histories_with_no_common_ancestor_have_no_merge_base() {
    let _lock = serial();
    let scratch = Scratch::new();
    let (work, _, main_tip) = forked(&scratch);
    work.git(&["checkout", "-q", "--orphan", "unrelated"]);
    work.git(&["rm", "-rfq", "."]);
    work.commit("island.md", b"island\n", "an unrelated root");
    let error = merge_base(&work.runner(), &commit(&main_tip)).expect_err("no common ancestor");
    assert!(matches!(&error, GitError::NoMergeBase { default } if default.as_str() == main_tip));
    assert!(error.to_string().contains("no common ancestor"), "{error}");
}

// ------------------------------------------------------------------ prefix

#[test]
fn the_prefix_is_where_the_directory_is_below_the_top_of_its_work_tree() {
    let _lock = serial();
    let scratch = Scratch::new();
    let repo = Repo::init(scratch.path().join("repo"));
    repo.put("top.md", b"top\n");
    repo.put("services/api/inner.md", b"inner\n");
    repo.commit("services/api/more.md", b"more\n", "first");
    let prefix = |dir: &Path| show_prefix(&runner_in(dir)).expect("a prefix");
    assert!(prefix(&repo.dir).is_top());
    assert_eq!(prefix(&repo.dir.join("services")).as_str(), "services/");
    assert_eq!(
        prefix(&repo.dir.join("services/api")).as_str(),
        "services/api/"
    );

    // A file is read at the prefix, and at the bare path it is absent.
    let tip = repo.git(&["rev-parse", "HEAD"]);
    let base = base_at(&repo, &tip);
    let named = prefix(&repo.dir.join("services/api"))
        .join(&path("inner.md"))
        .expect("a joined path");
    assert_eq!(named.as_str(), "services/api/inner.md");
    assert_eq!(
        blob_at(&repo.runner(), &base, &named).expect("an answer"),
        Some(b"inner\n".to_vec())
    );
    assert_eq!(
        blob_at(&repo.runner(), &base, &path("inner.md")).expect("an answer"),
        None
    );

    // A directory outside any work tree has no prefix.
    let loose = scratch.path().join("loose");
    fs::create_dir(&loose).expect("a directory");
    assert!(matches!(
        show_prefix(&runner_in(&loose)),
        Err(GitError::Failed { .. })
    ));
}

#[test]
fn a_linked_work_tree_is_its_own_top() {
    let _lock = serial();
    let scratch = Scratch::new();
    let (work, _, _) = forked(&scratch);
    let linked = scratch.path().join("linked");
    work.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "from-main",
        linked.to_str().expect("a UTF-8 path"),
        "main",
    ]);
    assert!(show_prefix(&runner_in(&linked)).expect("a prefix").is_top());
}

// ----------------------------------------------------------------- blob_at

#[test]
fn a_file_is_read_as_it_was_at_the_merge_base() {
    let _lock = serial();
    let scratch = Scratch::new();
    let seed = Repo::init(scratch.path().join("repo"));
    let fork = seed.commit("doc.md", b"as it was\n", "first");
    seed.commit("doc.md", b"edited on main\n", "second");
    seed.git(&["checkout", "-q", "-b", "feature", &fork]);
    seed.commit("doc.md", b"edited on the branch\n", "branch edit");
    seed.commit("added.md", b"new on the branch\n", "branch add");
    let base = base_at(&seed, &fork);
    assert_eq!(base.as_str(), fork);

    let read = |name: &str| blob_at(&seed.runner(), &base, &path(name)).expect("an answer");
    assert_eq!(read("doc.md").as_deref(), Some(b"as it was\n".as_slice()));
    // Absent at the merge base: new on the branch, and never there.
    assert_eq!(read("added.md"), None);
    assert_eq!(read("nowhere/at/all.md"), None);
    // Present in the working tree but not at the merge base.
    seed.put("only-on-disk.md", b"on disk\n");
    assert_eq!(read("only-on-disk.md"), None);
}

#[test]
fn a_blob_is_returned_byte_for_byte() {
    let _lock = serial();
    let scratch = Scratch::new();
    let repo = Repo::init(scratch.path().join("repo"));
    let bytes: Vec<u8> = (0..=255u8).chain([b'\r', b'\n', 0, 0xff]).collect();
    let tip = repo.commit("blob.bin", &bytes, "binary");
    repo.commit("empty.md", b"", "empty");
    let base = base_at(&repo, &tip);
    assert_eq!(
        blob_at(&repo.runner(), &base, &path("blob.bin")).expect("a blob"),
        Some(bytes)
    );
    // The empty file is a file, not an absent one.
    let head = base_at(&repo, &repo.git(&["rev-parse", "HEAD"]));
    assert_eq!(
        blob_at(&repo.runner(), &head, &path("empty.md")).expect("a blob"),
        Some(Vec::new())
    );
}

#[test]
fn a_blob_over_one_mebibyte_is_an_error_and_exactly_one_mebibyte_is_read() {
    let _lock = serial();
    let scratch = Scratch::new();
    let repo = Repo::init(scratch.path().join("repo"));
    repo.put("exact.md", &vec![b'a'; MAX_TEXT_BYTES]);
    repo.put("over.md", &vec![b'a'; MAX_TEXT_BYTES + 1]);
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "big files"]);
    let tip = base_at(&repo, &repo.git(&["rev-parse", "HEAD"]));

    let exact = blob_at(&repo.runner(), &tip, &path("exact.md"))
        .expect("the bound itself passes")
        .expect("a blob");
    assert_eq!(exact.len(), MAX_TEXT_BYTES);

    let over = blob_at(&repo.runner(), &tip, &path("over.md")).expect_err("one byte over");
    assert!(
        matches!(&over, GitError::BlobTooLarge { path, limit } if path.as_str() == "over.md" && *limit == MAX_TEXT_BYTES),
        "{over:?}"
    );
    assert!(over.to_string().contains("over.md"), "{over}");
}

#[test]
fn a_directory_is_an_error_and_not_an_absent_file() {
    let _lock = serial();
    let scratch = Scratch::new();
    let repo = Repo::init(scratch.path().join("repo"));
    let tip = repo.commit("dir/file.md", b"x\n", "first");
    let error =
        blob_at(&repo.runner(), &base_at(&repo, &tip), &path("dir")).expect_err("a directory");
    assert!(matches!(error, GitError::Failed { .. }), "{error:?}");
    // The neighbour: the file in the directory reads.
    assert_eq!(
        blob_at(&repo.runner(), &base_at(&repo, &tip), &path("dir/file.md")).expect("a blob"),
        Some(b"x\n".to_vec())
    );
}

// --------------------------------------------------------- a linked worktree

#[test]
fn a_linked_work_tree_asks_about_its_own_head_and_shares_the_remote_refs() {
    let _lock = serial();
    let scratch = Scratch::new();
    let (work, fork, main_tip) = forked(&scratch);
    // The main work tree is on `feature`; the linked one is on its own branch
    // from `main`, so the two `HEAD`s differ.
    let linked = scratch.path().join("linked");
    work.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "from-main",
        linked.to_str().expect("a UTF-8 path"),
        "main",
    ]);
    let in_linked = Repo {
        dir: linked.clone(),
    };
    assert!(
        linked.join(".git").is_file(),
        "a linked work tree has a `.git` file"
    );
    let own_tip = in_linked.commit("linked.md", b"linked\n", "in the linked work tree");

    // The default branch is the shared remote-tracking ref.
    let name = default_branch_ref(&in_linked.runner()).expect("a default branch");
    assert_eq!(name.as_str(), "refs/remotes/origin/main");
    let default = resolve_commit(&in_linked.runner(), &name).expect("a commit");
    assert_eq!(default.as_str(), main_tip);

    // `HEAD` is the linked work tree's: its merge base is `main`'s tip, where
    // the main work tree's is the fork point.
    let linked_base = merge_base(&in_linked.runner(), &default).expect("a merge base");
    assert_eq!(linked_base.as_str(), main_tip);
    assert_ne!(linked_base.as_str(), own_tip);
    let main_base = merge_base(&work.runner(), &default).expect("a merge base");
    assert_eq!(main_base.as_str(), fork);

    // A file as the linked merge base had it, and as the other had it.
    assert_eq!(
        blob_at(&in_linked.runner(), &linked_base, &path("shared.md")).expect("an answer"),
        Some(b"shared\n".to_vec())
    );
    assert_eq!(
        blob_at(&in_linked.runner(), &main_base, &path("shared.md")).expect("an answer"),
        None
    );

    // The tracked paths are the linked work tree's own.
    let tracked = tracked_paths(&in_linked.runner())
        .expect("a listing")
        .expect("inside a work tree");
    let names: Vec<&str> = tracked.iter().map(raf_source::RelPath::as_str).collect();
    assert_eq!(names, ["README.md", "linked.md", "shared.md"]);
}

// -------------------------------------------------------------- stub `git`

#[cfg(unix)]
mod stubs {
    use super::*;
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;

    const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
    const MERGE_BASE: &str = "fedcba9876543210fedcba9876543210fedcba98";

    /// A `git` that records each call (its argv, one record to a line, the
    /// elements separated by the unit separator 0x1f, and the directory and
    /// locale it ran with) and answers the questions of this module.
    fn recording_git(version: &str, default_ref: &str) -> String {
        format!(
            r#"dir=$(dirname "$0")
printf '%s\037' "$@" >> "$dir/argv"
printf '\n' >> "$dir/argv"
pwd -P >> "$dir/cwd"
printf '%s\n' "${{LC_ALL-<unset>}}" >> "$dir/lc_all"
case "$1" in
--version) echo '{version}' ;;
rev-parse)
  case "$2" in
  --is-inside-work-tree) echo true ;;
  --show-prefix) echo ;;
  *) echo {COMMIT} ;;
  esac ;;
ls-files) printf 'docs/a.md\0src/lib.rs\0' ;;
symbolic-ref) echo '{default_ref}' ;;
merge-base) echo {MERGE_BASE} ;;
cat-file) printf 'blob text' ;;
*) echo "unexpected $*" >&2; exit 64 ;;
esac"#
        )
    }

    struct Stub {
        scratch: Scratch,
        git: PathBuf,
    }

    impl Stub {
        fn new(version: &str, default_ref: &str) -> Self {
            let scratch = Scratch::new();
            fs::create_dir(scratch.path().join("bin")).expect("a bin directory");
            fs::create_dir(scratch.path().join("work")).expect("a work directory");
            let git = scratch.path().join("bin/git");
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o755)
                .open(&git)
                .expect("a stub file");
            write!(file, "#!/bin/sh\n{}\n", recording_git(version, default_ref))
                .expect("a stub body");
            drop(file);
            Self { scratch, git }
        }

        fn runner(&self) -> SystemRunner {
            SystemRunner::new(Programs::new(self.work()).with_git(&self.git))
        }

        fn work(&self) -> PathBuf {
            self.scratch.path().join("work")
        }

        fn lines(&self, name: &str) -> Vec<String> {
            let text =
                fs::read_to_string(self.scratch.path().join("bin").join(name)).unwrap_or_default();
            text.lines().map(str::to_string).collect()
        }

        /// Every recorded call, each as its argv.
        fn calls(&self) -> Vec<Vec<String>> {
            self.lines("argv")
                .iter()
                .map(|line| {
                    let mut parts: Vec<&str> = line.split('\u{1f}').collect();
                    assert_eq!(parts.pop(), Some(""), "the record ends in a separator");
                    parts.into_iter().map(str::to_string).collect()
                })
                .collect()
        }
    }

    #[test]
    fn every_question_reaches_git_as_exactly_its_argv_in_the_work_directory() {
        let _lock = serial();
        let stub = Stub::new(
            "git version 2.45.1.windows.1",
            "refs/remotes/origin/--output=x",
        );
        let runner = stub.runner();

        let version = require_version(&runner).expect("a version");
        assert_eq!(
            (version.major(), version.minor(), version.patch()),
            (2, 45, 1)
        );
        let tracked = tracked_paths(&runner)
            .expect("a listing")
            .expect("inside a work tree");
        assert_eq!(tracked.len(), 2);
        assert!(show_prefix(&runner).expect("a prefix").is_top());
        let name = default_branch_ref(&runner).expect("a default branch");
        assert_eq!(name.as_str(), "refs/remotes/origin/--output=x");
        let default = resolve_commit(&runner, &name).expect("a commit");
        assert_eq!(default.as_str(), COMMIT);
        let base = merge_base(&runner, &default).expect("a merge base");
        assert_eq!(base.as_str(), MERGE_BASE);
        let blob =
            blob_at(&runner, &base, &path("docs/capabilities/workplan-cli.md")).expect("a blob");
        assert_eq!(blob.as_deref(), Some(b"blob text".as_slice()));

        let expected: Vec<Vec<String>> = [
            vec!["--version"],
            vec!["rev-parse", "--is-inside-work-tree"],
            vec!["ls-files", "-z"],
            vec!["rev-parse", "--show-prefix"],
            vec!["symbolic-ref", "refs/remotes/origin/HEAD"],
            vec![
                "rev-parse",
                "--verify",
                "--end-of-options",
                "refs/remotes/origin/--output=x^{commit}",
            ],
            vec!["merge-base", "HEAD", COMMIT],
            vec![
                "cat-file",
                "blob",
                &format!("{MERGE_BASE}:docs/capabilities/workplan-cli.md"),
            ],
        ]
        .into_iter()
        .map(|argv| argv.into_iter().map(str::to_string).collect())
        .collect();
        assert_eq!(stub.calls(), expected);

        // A branch named like an option is never an argument of its own: the
        // only words that begin with a dash are the fixed options.
        let fixed = [
            "--version",
            "--is-inside-work-tree",
            "--show-prefix",
            "-z",
            "--verify",
            "--end-of-options",
        ];
        for call in stub.calls() {
            for arg in call.iter().filter(|arg| arg.starts_with('-')) {
                assert!(fixed.contains(&arg.as_str()), "{arg} in {call:?}");
            }
        }

        // Every call ran in the work directory and in the C locale.
        let work = fs::canonicalize(stub.work()).expect("a canonical work directory");
        let cwd = work.to_str().expect("a UTF-8 path");
        assert!(stub.lines("cwd").iter().all(|line| line == cwd));
        assert_eq!(stub.lines("cwd").len(), 8);
        assert!(stub.lines("lc_all").iter().all(|line| line == "C"));
    }

    #[test]
    fn the_version_gate_over_a_real_process_refuses_2_29_and_accepts_2_30() {
        let _lock = serial();
        for (text, accepted) in [
            ("git version 2.29.9", false),
            ("git version 2.29.2.windows.1", false),
            ("git version 2.29.3 (Apple Git-130)", false),
            ("git version 2.30.0", true),
            ("git version 2.45.1", true),
            ("git version 2.45.1.windows.1", true),
            ("git version 2.39.5 (Apple Git-154)", true),
        ] {
            let stub = Stub::new(text, "refs/remotes/origin/main");
            let result = require_version(&stub.runner());
            if accepted {
                result.unwrap_or_else(|error| panic!("{text}: {error}"));
            } else {
                let error = result.expect_err(text);
                assert!(
                    matches!(error, GitError::VersionTooOld { .. }),
                    "{text}: {error:?}"
                );
            }
            assert_eq!(stub.calls(), [["--version"]], "{text}: one question");
        }
        let stub = Stub::new("this is not git", "refs/remotes/origin/main");
        assert!(matches!(
            require_version(&stub.runner()),
            Err(GitError::VersionUnreadable { .. })
        ));
    }

    #[test]
    fn an_answer_that_is_not_a_full_reference_name_stops_the_chain_before_any_commit_is_asked_for()
    {
        let _lock = serial();
        for answer in ["--output=x", "main", "origin/main"] {
            let stub = Stub::new("git version 2.45.1", answer);
            let error = default_branch_ref(&stub.runner()).expect_err(answer);
            assert!(
                matches!(error, GitError::Answer { .. }),
                "{answer}: {error:?}"
            );
            assert_eq!(stub.calls().len(), 1, "{answer}");
        }
    }

    #[test]
    fn a_stub_that_exits_with_a_failure_is_an_error_with_its_message() {
        let _lock = serial();
        let scratch = Scratch::new();
        let git = scratch.path().join("git");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o755)
            .open(&git)
            .expect("a stub file");
        write!(file, "#!/bin/sh\necho 'fatal: boom' >&2\nexit 128\n").expect("a stub body");
        drop(file);
        let runner = SystemRunner::new(Programs::new(scratch.path()).with_git(&git));
        let error = require_version(&runner).expect_err("a failing git");
        assert!(
            matches!(&error, GitError::Failed { code: Some(128), stderr, .. } if stderr == "fatal: boom"),
            "{error:?}"
        );
        let error = tracked_paths(&runner).expect_err("a failing git");
        assert!(matches!(error, GitError::Failed { .. }), "{error:?}");
    }
}
