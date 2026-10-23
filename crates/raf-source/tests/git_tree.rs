// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Actual object bytes establish the snapshot; labels and backend identities do not.

use std::error::Error;

use raf_source::{
    CommittedGitTree, EntryKind, LoadErrorKind, LoadOptions, LockedCatalog, MemoryTree, RelPath,
    Tree, TreeError, Workspace,
};

#[path = "support/git_objects.rs"]
mod git_objects;
mod support;
use git_objects::{GitObjectId, GitObjectKind, Objects, entry};

/// Remove only a fixture object, including Git's Windows read-only flag.
fn remove_object(objects: &Objects, id: GitObjectId) {
    let path = objects.object_path(id);
    #[cfg(windows)]
    {
        let mut permissions = std::fs::metadata(&path)
            .expect("owned object")
            .permissions();
        permissions.set_readonly(false);
        std::fs::set_permissions(&path, permissions).expect("owned object permissions");
    }
    std::fs::remove_file(path).expect("remove only the owned loose object");
}

fn fixture() -> (Objects, GitObjectId, GitObjectId) {
    let mut objects = Objects::default();
    let manifest = objects.insert(
        GitObjectKind::Blob,
        b"[package]\nname='demo'\nversion='1.0.0'\n".to_vec(),
    );
    let lib = objects.insert(GitObjectKind::Blob, b"pub fn provided() {}\r\n".to_vec());
    let src = objects.insert(
        GitObjectKind::Tree,
        [
            entry("100644", "lib.rs", lib),
            entry("100644", "twin.rs", lib),
        ]
        .concat(),
    );
    let link = objects.insert(GitObjectKind::Blob, b"src".to_vec());
    let root = objects.insert(
        GitObjectKind::Tree,
        [
            entry("100644", "Cargo.toml", manifest),
            entry("120000", "link", link),
            entry("40000", "src", src),
        ]
        .concat(),
    );
    let commit = objects.commit(root);
    (objects, commit, lib)
}

fn locked(commit: GitObjectId) -> LockedCatalog {
    let text = format!(
        "version=4\n[[package]]\nname='demo'\nversion='1.0.0'\nsource='git+https://example.test/source?rev={commit}#{commit}'\n"
    );
    LockedCatalog::load(
        &MemoryTree::from([("Cargo.lock", text.as_str())]),
        &RelPath::new("Cargo.lock").expect("path"),
        &LoadOptions::default(),
    )
    .expect("catalog")
}

#[test]
fn checked_snapshot_serves_real_bytes_and_existing_loader_with_distinct_path_identity() {
    let (objects, commit, _) = fixture();
    let catalog = locked(commit);
    let id = catalog.packages().next().expect("row").id();
    let tree = CommittedGitTree::open(&catalog, id, objects.path(), &LoadOptions::default())
        .expect("verified snapshot");
    assert_eq!(tree.source().revision(), Some(commit.to_string().as_str()));
    let path = RelPath::new("src/lib.rs").expect("path");
    assert_eq!(
        tree.read(&path, 100).expect("blob"),
        "pub fn provided() {}\r\n"
    );
    assert!(matches!(
        tree.read(&path, 1),
        Err(TreeError::TooLarge { .. })
    ));
    assert_ne!(
        tree.identity(&path).expect("identity"),
        tree.identity(&RelPath::new("src/twin.rs").expect("path"))
            .expect("twin identity")
    );
    assert_eq!(
        tree.identity(&path).expect("identity"),
        tree.identity(&RelPath::new("src/../src/lib.rs").expect("path"))
            .expect("parent within source")
    );
    let listing = tree.list(&RelPath::root()).expect("complete listing");
    assert_eq!(
        listing
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>(),
        ["Cargo.toml", "link", "src"]
    );
    assert_eq!(listing[1].kind, EntryKind::Symlink);
    assert!(
        tree.read(&RelPath::new("link/../src/lib.rs").expect("path"), 100)
            .is_err()
    );
    assert_eq!(
        tree.entry(&RelPath::new("absent.rs").expect("path"))
            .expect("checked absence"),
        None
    );
    // Untracked checkout files cannot appear: this Tree reads only committed objects.
    let ws = Workspace::load(&tree, &["."]).expect("ordinary source loader");
    assert_eq!(
        ws.resolve("demo::provided")
            .expect("actual definition")
            .path,
        "demo::provided"
    );
}

#[test]
fn missing_commit_repository_and_wrong_object_kind_refuse_beside_the_exact_commit() {
    let (objects, commit, blob) = fixture();
    let catalog = locked(commit);
    let row = catalog.packages().next().expect("row").id();
    CommittedGitTree::open(&catalog, row, objects.path(), &LoadOptions::default()).expect("valid");
    let absent = objects.path().join("not-a-repository");
    let error = CommittedGitTree::open(&catalog, row, &absent, &LoadOptions::default())
        .expect_err("missing repository");
    assert!(error.source().is_some(), "{error}");
    for id in [blob, GitObjectId([7; 20])] {
        let wrong = locked(id);
        let row = wrong.packages().next().expect("row").id();
        assert!(
            CommittedGitTree::open(&wrong, row, objects.path(), &LoadOptions::default()).is_err()
        );
    }
    CommittedGitTree::open(&catalog, row, objects.path(), &LoadOptions::default())
        .expect("valid neighbor");
}

#[test]
fn missing_or_substituted_blob_is_a_hard_failure_even_when_seen_would_skip_text() {
    let (mut objects, commit, blob) = fixture();
    let catalog = locked(commit);
    let row = catalog.packages().next().expect("row").id();
    let tree = CommittedGitTree::open(&catalog, row, objects.path(), &LoadOptions::default())
        .expect("valid");
    let path = RelPath::new("src/lib.rs").expect("path");
    tree.read(&path, 100).expect("valid neighbor");
    let alternate = objects.insert(GitObjectKind::Blob, b"pub fn invented() {}\r\n".to_vec());
    remove_object(&objects, blob);
    std::fs::copy(objects.object_path(alternate), objects.object_path(blob))
        .expect("substitute only an owned object");
    let error = tree
        .read_unseen(&path, 100, &|_| true)
        .expect_err("different committed blob cannot become Seen");
    assert!(!matches!(error, TreeError::NotFound { .. }), "{error}");
    remove_object(&objects, blob);
    let error = tree
        .read_unseen(&path, 100, &|_| true)
        .expect_err("missing committed bytes cannot become Seen");
    assert!(!matches!(error, TreeError::NotFound { .. }), "{error}");
    assert!(error.source().is_some(), "{error}");
}

#[test]
fn malformed_tree_and_submodule_refuse_with_an_ordinary_tree_neighbor() {
    let mut objects = Objects::default();
    let blob = objects.insert(GitObjectKind::Blob, b"hello\n".to_vec());
    let valid = objects.insert(GitObjectKind::Tree, entry("100644", "file", blob));
    let valid = objects.commit(valid);
    for body in [
        b"100644 broken".to_vec(),
        entry("160000", "submodule", valid),
        [entry("100644", "same", blob), entry("100644", "same", blob)].concat(),
    ] {
        let tree = objects.insert(GitObjectKind::Tree, body);
        let commit = objects.commit(tree);
        let catalog = locked(commit);
        let row = catalog.packages().next().expect("row").id();
        assert!(
            CommittedGitTree::open(&catalog, row, objects.path(), &LoadOptions::default()).is_err()
        );
    }
    let catalog = locked(valid);
    let row = catalog.packages().next().expect("row").id();
    let tree = CommittedGitTree::open(&catalog, row, objects.path(), &LoadOptions::default())
        .expect("valid neighbor");
    assert_eq!(
        tree.read(&RelPath::new("file").expect("path"), 6)
            .expect("exact"),
        "hello\n"
    );
    assert!(matches!(
        tree.read(&RelPath::new("file").expect("path"), 5),
        Err(TreeError::TooLarge { .. })
    ));
}

#[test]
fn committed_bytes_ignore_worktree_edits_and_replacement_refs() {
    let (mut objects, commit, _) = fixture();
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(objects.path())
        .args(["read-tree", &commit.to_string()])
        .status()
        .expect("owned checkout");
    assert!(status.success());
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(objects.path())
        .args(["checkout-index", "--all"])
        .status()
        .expect("owned indexed checkout");
    assert!(status.success());
    std::fs::write(objects.path().join("src/lib.rs"), "forged checkout source")
        .expect("edit owned checkout");
    let other = objects.insert(GitObjectKind::Tree, Vec::new());
    let other = objects.commit(other);
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(objects.path())
        .args(["replace", &commit.to_string(), &other.to_string()])
        .status()
        .expect("owned Git");
    assert!(status.success());
    std::fs::write(objects.path().join("src.rs"), "forged untracked source")
        .expect("owned worktree neighbor");
    let catalog = locked(commit);
    let row = catalog.packages().next().expect("row").id();
    let tree = CommittedGitTree::open(&catalog, row, objects.path(), &LoadOptions::default())
        .expect("exact committed tree");
    assert_eq!(
        tree.read(&RelPath::new("src/lib.rs").expect("path"), 100)
            .expect("original"),
        "pub fn provided() {}\r\n"
    );
    assert_eq!(
        tree.entry(&RelPath::new("src.rs").expect("path"))
            .expect("absence"),
        None
    );
}

#[test]
fn missing_installed_git_is_an_actionable_dependency_failure() {
    const REPO: &str = "RAF_GIT_MISSING_TOOL_REPOSITORY";
    if let Some(repository) = std::env::var_os(REPO) {
        let commit = std::env::var("RAF_GIT_MISSING_TOOL_COMMIT").expect("parent commit");
        let text = format!(
            "version=4\n[[package]]\nname='demo'\nversion='1.0.0'\nsource='git+https://example.test/source#{commit}'\n"
        );
        let catalog = LockedCatalog::load(
            &MemoryTree::from([("Cargo.lock", text.as_str())]),
            &RelPath::new("Cargo.lock").expect("path"),
            &LoadOptions::default(),
        )
        .expect("catalog");
        let row = catalog.packages().next().expect("row").id();
        let error = CommittedGitTree::open(&catalog, row, repository, &LoadOptions::default())
            .expect_err("no installed Git");
        assert!(
            matches!(&error.kind,LoadErrorKind::GitIo {source,..} if source.kind()==std::io::ErrorKind::NotFound),
            "{error}"
        );
        assert!(error.source().is_some());
        return;
    }
    let (objects, commit, _) = fixture();
    let empty = support::Scratch::new();
    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args([
            "--exact",
            "missing_installed_git_is_an_actionable_dependency_failure",
        ])
        .env(REPO, objects.path())
        .env("RAF_GIT_MISSING_TOOL_COMMIT", commit.to_string())
        .env("PATH", empty.base())
        .output()
        .expect("isolated no-tool child");
    assert!(output.status.success(), "{output:?}");
}
