// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Actual object bytes establish the snapshot; labels and backend identities do not.

use std::cell::Cell;
use std::error::Error;

use purrdf_hash::hex;
use raf_source::{
    Budget, EntryKind, GitObjectId, GitObjectKind, LoadErrorKind, LoadOptions, LockedCatalog,
    MemoryTree, Opened, RelPath, Tree, TreeError, VerifiedGitTree, Workspace,
};

#[path = "support/git_objects.rs"]
mod git_objects;
use git_objects::{Objects, entry};

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
    let tree = VerifiedGitTree::new(&catalog, id, &objects, &LoadOptions::default())
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
fn wrong_commit_wrong_kind_and_corrupt_bodies_refuse_beside_the_exact_objects() {
    let (mut objects, commit, _) = fixture();
    let catalog = locked(commit);
    let id = catalog.packages().next().expect("row").id();
    VerifiedGitTree::new(&catalog, id, &objects, &LoadOptions::default()).expect("valid");
    let actual = objects.0.get(&commit).expect("commit").clone();
    objects.0.get_mut(&commit).expect("commit").0 = GitObjectKind::Tree;
    assert!(
        VerifiedGitTree::new(&catalog, id, &objects, &LoadOptions::default())
            .expect_err("wrong kind")
            .to_string()
            .contains("wrong kind")
    );
    objects.0.insert(commit, actual.clone());
    objects.0.get_mut(&commit).expect("commit").1.push(b'x');
    assert!(
        VerifiedGitTree::new(&catalog, id, &objects, &LoadOptions::default())
            .expect_err("corrupt body")
            .to_string()
            .contains("hash")
    );
    objects.0.insert(commit, actual);
    let wrong = locked(GitObjectId::from_sha1([7; 20]));
    assert!(
        VerifiedGitTree::new(
            &wrong,
            wrong.packages().next().expect("row").id(),
            &objects,
            &LoadOptions::default()
        )
        .is_err()
    );
    VerifiedGitTree::new(&catalog, id, &objects, &LoadOptions::default()).expect("restored valid");
}

#[test]
fn corrupt_blob_cannot_hide_behind_seen_or_a_missing_object_become_path_absence() {
    let (mut objects, commit, blob) = fixture();
    let catalog = locked(commit);
    let id = catalog.packages().next().expect("row").id();
    let path = RelPath::new("src/lib.rs").expect("path");
    let original = objects.0.get(&blob).expect("blob").clone();
    objects.0.get_mut(&blob).expect("blob").1.push(b'x');
    let tree = VerifiedGitTree::new(&catalog, id, &objects, &LoadOptions::default())
        .expect("commit/tree valid");
    let seen = Cell::new(false);
    assert!(matches!(
        tree.read_unseen(&path, 0, &|_| {
            seen.set(true);
            true
        }),
        Err(TreeError::GitSource { .. })
    ));
    assert!(!seen.get());
    objects.0.remove(&blob);
    let tree = VerifiedGitTree::new(&catalog, id, &objects, &LoadOptions::default())
        .expect("commit/tree valid");
    let error = tree.read(&path, 100).expect_err("missing evidence");
    assert!(matches!(error, TreeError::GitSource { .. }));
    assert!(error.source().is_some());
    objects.0.insert(blob, original);
    let tree =
        VerifiedGitTree::new(&catalog, id, &objects, &LoadOptions::default()).expect("restored");
    assert!(matches!(
        tree.read_unseen(&path, 0, &|_| true),
        Ok(Opened::Seen(_))
    ));
}

#[test]
fn complete_fallible_decode_refuses_trailing_corruption_duplicate_names_and_path_escapes() {
    let (mut objects, _, blob) = fixture();
    for tree_body in [
        [
            entry("100644", "first.rs", blob),
            b"bad trailing entry".to_vec(),
        ]
        .concat(),
        [
            entry("100644", "same.rs", blob),
            entry("100644", "same.rs", blob),
        ]
        .concat(),
        entry("100644", "../escape.rs", blob),
        entry("160000", "submodule", blob),
    ] {
        let root = objects.insert(GitObjectKind::Tree, tree_body);
        let commit = objects.commit(root);
        let catalog = locked(commit);
        let error = VerifiedGitTree::new(
            &catalog,
            catalog.packages().next().expect("row").id(),
            &objects,
            &LoadOptions::default(),
        )
        .expect_err("complete directory refusal");
        assert!(matches!(
            error.kind,
            LoadErrorKind::GitDecode(_) | LoadErrorKind::Locked { .. }
        ));
    }
    let good_root = objects.insert(GitObjectKind::Tree, entry("100644", "first.rs", blob));
    let malformed_commit = objects.insert(
        GitObjectKind::Commit,
        format!("tree {good_root}\nauthor broken\n").into_bytes(),
    );
    let catalog = locked(malformed_commit);
    let error = VerifiedGitTree::new(
        &catalog,
        catalog.packages().next().expect("row").id(),
        &objects,
        &LoadOptions::default(),
    )
    .expect_err("must decode after first tree header");
    assert!(matches!(error.kind, LoadErrorKind::GitDecode(_)));
    assert!(error.source().is_some());
}

#[test]
fn native_hash_matches_git_blob_vector_and_object_processing_keeps_original_grants() {
    let mut vector = Objects::default();
    let id = vector.insert(GitObjectKind::Blob, b"hello\n".to_vec());
    assert_eq!(
        format!("{}", hex::Lower(id.sha1())),
        "ce013625030ba8dba906f756967f9e9ca394464a"
    );
    let (objects, commit, _) = fixture();
    let catalog = locked(commit);
    let id = catalog.packages().next().expect("row").id();
    let baseline =
        VerifiedGitTree::new(&catalog, id, &objects, &LoadOptions::default()).expect("baseline");
    let usage = baseline.usage();
    let mut options = LoadOptions::default();
    options.max_work = usage.work;
    options.max_item_bytes = usage.item_bytes;
    options.max_total_bytes = usage.total_bytes;
    VerifiedGitTree::new(&catalog, id, &objects, &options).expect("exact grants");
    for budget in [Budget::Work, Budget::ItemBytes, Budget::TotalBytes] {
        let mut short = options.clone();
        match budget {
            Budget::Work => short.max_work -= 1,
            Budget::ItemBytes => short.max_item_bytes -= 1,
            Budget::TotalBytes => short.max_total_bytes -= 1,
            _ => unreachable!("three varied grants"),
        }
        let error = VerifiedGitTree::new(&catalog, id, &objects, &short).expect_err("one short");
        assert!(
            matches!(error.kind, LoadErrorKind::Budget { budget: actual, .. } if actual == budget),
            "{error}"
        );
    }
    let mut tiny = LoadOptions::default();
    tiny.max_file_bytes = 1;
    let error = VerifiedGitTree::new(&catalog, id, &objects, &tiny)
        .expect_err("bounded object acquisition");
    assert!(matches!(error.kind, LoadErrorKind::Tree(_)));
    assert!(error.to_string().contains("limit of 1 bytes"));
}

#[test]
fn other_hash_formats_and_local_rows_do_not_gain_git_admission() {
    let (objects, commit, _) = fixture();
    for source in [
        String::new(),
        format!(
            "source='git+https://example.test/source#{}'",
            "b".repeat(64)
        ),
    ] {
        let text = format!("version=4\n[[package]]\nname='demo'\nversion='1.0.0'\n{source}\n");
        let catalog = LockedCatalog::load(
            &MemoryTree::from([("Cargo.lock", text.as_str())]),
            &RelPath::new("Cargo.lock").expect("path"),
            &LoadOptions::default(),
        )
        .expect("valid catalog observation");
        let error = VerifiedGitTree::new(
            &catalog,
            catalog.packages().next().expect("row").id(),
            &objects,
            &LoadOptions::default(),
        )
        .expect_err("no supported Git admission");
        assert!(matches!(error.kind, LoadErrorKind::Locked { .. }));
    }
    let catalog = locked(commit);
    VerifiedGitTree::new(
        &catalog,
        catalog.packages().next().expect("row").id(),
        &objects,
        &LoadOptions::default(),
    )
    .expect("actual SHA-1 neighbor");
}
