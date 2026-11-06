// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native Git reads over repositories written to a scratch directory: tracked
//! paths, default branches, merge bases, historical files and linked worktrees.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard, PoisonError};

use raf_okf::MAX_TEXT_BYTES;
use raf_workplan::git::{
    GitError, MergeBase, Repository, blob_at, default_branch_ref, merge_base, resolve_commit,
    resolve_commit_id, show_prefix, tracked_paths,
};
use raf_workplan::identifiers::{Commit, RefName, SourcePath};

/// Fixture hooks are written while other tests start `git`; a forked child
/// briefly holding the write descriptor can prevent executing the hook.
/// Serialize fixture processes to prevent that descriptor inheritance race.
static SPAWN: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SPAWN.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A scratch directory, removed when dropped.
struct Scratch(purrdf_testkit::TempDir);

impl Scratch {
    fn new() -> Self {
        let parent = fs::canonicalize(std::env::temp_dir()).expect("a canonical temporary parent");
        Self(purrdf_testkit::TempDir::new_in(parent).expect("a scratch directory"))
    }

    fn path(&self) -> &Path {
        self.0.path()
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
    let text = purrdf_testkit::text::stdout_utf8(&output);
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

    fn runner(&self) -> Repository {
        runner_in(&self.dir)
    }
}

fn runner_in(dir: &Path) -> Repository {
    Repository::open(dir).expect("native repository")
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
    assert!(
        Repository::discover(&loose)
            .expect("outside discovery")
            .is_none()
    );

    // Inside the `.git` directory: git answers `false`.
    let repo = Repo::init(scratch.path().join("repo"));
    repo.commit("a.md", b"a\n", "first");
    assert!(
        tracked_paths(&runner_in(&repo.dir.join(".git")))
            .expect("Git directory selection")
            .is_none()
    );

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
    assert!(
        matches!(error, GitError::Native(_) | GitError::Answer { .. }),
        "{error:?}"
    );
    assert!(error.to_string().contains("index"), "{error}");
}

// ----------------------------------------------------------------- version

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
    let base = merge_base(&work.runner(), default.commit()).expect("a merge base");
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
    assert!(
        message.contains("no symbolic origin default branch"),
        "{message}"
    );

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
        merge_base(&work.runner(), resolved.commit())
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
        merge_base(&repo.runner(), resolved.commit())
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
    let base = merge_base(&work.runner(), default.commit()).expect("a merge base");
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
        blob_at(&repo.runner(), base.resolved(), &named).expect("an answer"),
        Some(b"inner\n".to_vec())
    );
    assert_eq!(
        blob_at(&repo.runner(), base.resolved(), &path("inner.md")).expect("an answer"),
        None
    );

    // A directory outside any work tree has no prefix.
    let loose = scratch.path().join("loose");
    fs::create_dir(&loose).expect("a directory");
    assert!(
        Repository::discover(&loose)
            .expect("outside discovery")
            .is_none()
    );
}

#[test]
fn a_linked_work_tree_is_its_own_top() {
    let _lock = serial();
    let scratch = Scratch::new();
    let (work, _, _) = forked(&scratch);
    let linked = scratch.path().join("linked");
    // Keep the Git argument relative to `work`, rather than passing the
    // canonical filesystem spelling (a verbatim path on Windows).
    work.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "from-main",
        "../linked",
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
    let tip = seed.commit("added.md", b"new on the branch\n", "branch add");
    let base = base_at(&seed, &fork);
    assert_eq!(base.as_str(), fork);

    let read =
        |name: &str| blob_at(&seed.runner(), base.resolved(), &path(name)).expect("an answer");
    assert_eq!(read("doc.md").as_deref(), Some(b"as it was\n".as_slice()));
    // Absent at the merge base: new on the branch, and never there.
    assert_eq!(read("added.md"), None);
    assert_eq!(read("nowhere/at/all.md"), None);
    // Present in the working tree but not at the merge base.
    seed.put("only-on-disk.md", b"on disk\n");
    assert_eq!(read("only-on-disk.md"), None);
    resolved_historical_files_preserve_admission(&seed, &tip, &base);
}

fn resolved_historical_files_preserve_admission(repo: &Repo, tip: &str, base: &MergeBase) {
    let resolved = resolve_commit_id(&repo.runner(), &commit(tip)).expect("an explicit commit");
    assert_ne!(resolved.as_str(), base.as_str());
    assert_eq!(
        blob_at(&repo.runner(), &resolved, &path("doc.md")).unwrap(),
        Some(b"edited on the branch\n".to_vec())
    );
    assert_eq!(
        blob_at(&repo.runner(), &resolved, &path("only-on-disk.md")).unwrap(),
        None
    );
    let reference = RefName::new("refs/heads/feature").unwrap();
    assert_eq!(
        resolve_commit(&repo.runner(), &reference).unwrap(),
        resolved
    );
    let blob = repo.git(&["rev-parse", &format!("{tip}:doc.md")]);
    for refused in [blob.as_str(), "0000000000000000000000000000000000000000"] {
        assert!(matches!(
            resolve_commit_id(&repo.runner(), &commit(refused)),
            Err(GitError::Native(_) | GitError::Answer { .. })
        ));
    }
    repo.git(&["tag", "-a", "reviewed", "-m", "reviewed commit", tip]);
    let tag = RefName::new("refs/tags/reviewed").unwrap();
    assert_eq!(resolve_commit(&repo.runner(), &tag).unwrap(), resolved);
    let tag_id = repo.git(&["rev-parse", "refs/tags/reviewed"]);
    assert!(matches!(
        resolve_commit_id(&repo.runner(), &commit(&tag_id)),
        Err(GitError::Native(_))
    ));
    assert_eq!(
        fs::read(repo.dir.join("only-on-disk.md")).unwrap(),
        b"on disk\n"
    );
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
        blob_at(&repo.runner(), base.resolved(), &path("blob.bin")).expect("a blob"),
        Some(bytes)
    );
    // The empty file is a file, not an absent one.
    let head = base_at(&repo, &repo.git(&["rev-parse", "HEAD"]));
    assert_eq!(
        blob_at(&repo.runner(), head.resolved(), &path("empty.md")).expect("a blob"),
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

    let exact = blob_at(&repo.runner(), tip.resolved(), &path("exact.md"))
        .expect("the bound itself passes")
        .expect("a blob");
    assert_eq!(exact.len(), MAX_TEXT_BYTES);

    let over =
        blob_at(&repo.runner(), tip.resolved(), &path("over.md")).expect_err("one byte over");
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
    let error = blob_at(
        &repo.runner(),
        base_at(&repo, &tip).resolved(),
        &path("dir"),
    )
    .expect_err("a directory");
    assert!(
        matches!(error, GitError::Native(_) | GitError::Answer { .. }),
        "{error:?}"
    );
    // The neighbour: the file in the directory reads.
    assert_eq!(
        blob_at(
            &repo.runner(),
            base_at(&repo, &tip).resolved(),
            &path("dir/file.md")
        )
        .expect("a blob"),
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
    // The sibling path denotes the same target without a Windows verbatim
    // filesystem prefix in Git's path argument.
    work.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "from-main",
        "../linked",
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
    let linked_base = merge_base(&in_linked.runner(), default.commit()).expect("a merge base");
    assert_eq!(linked_base.as_str(), main_tip);
    assert_ne!(linked_base.as_str(), own_tip);
    let main_base = merge_base(&work.runner(), default.commit()).expect("a merge base");
    assert_eq!(main_base.as_str(), fork);

    // A file as the linked merge base had it, and as the other had it.
    assert_eq!(
        blob_at(
            &in_linked.runner(),
            linked_base.resolved(),
            &path("shared.md")
        )
        .expect("an answer"),
        Some(b"shared\n".to_vec())
    );
    assert_eq!(
        blob_at(
            &in_linked.runner(),
            main_base.resolved(),
            &path("shared.md")
        )
        .expect("an answer"),
        None
    );

    // The tracked paths are the linked work tree's own.
    let tracked = tracked_paths(&in_linked.runner())
        .expect("a listing")
        .expect("inside a work tree");
    let names: Vec<&str> = tracked.iter().map(raf_source::RelPath::as_str).collect();
    assert_eq!(names, ["README.md", "linked.md", "shared.md"]);
}
