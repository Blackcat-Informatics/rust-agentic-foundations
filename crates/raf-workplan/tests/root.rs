// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Root discovery from workspace roots, members and nested directories on disk.

use std::fs;
use std::path::{Path, PathBuf};

use raf_workplan::root::{RootError, discover};

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

    fn put(&self, path: &str, text: &str) {
        let target = self.path().join(path);
        fs::create_dir_all(target.parent().expect("a file has a directory"))
            .expect("the directory of a file");
        fs::write(target, text).expect("a file");
    }
}

/// A workspace of one package at its root, with its own `[workspace]` and no
/// dependency, so that nothing is fetched and no enclosing workspace is read.
fn single(scratch: &Scratch) {
    scratch.put(
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
    );
    scratch.put(
        "Cargo.lock",
        "version = 4\n\n[[package]]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    );
    scratch.put("src/lib.rs", "");
}

/// A virtual workspace of two members.
fn virtual_workspace(scratch: &Scratch) {
    scratch.put(
        "Cargo.toml",
        "[workspace]\nresolver = \"2\"\nmembers = [\"crates/one\", \"crates/two\"]\n",
    );
    for name in ["one", "two"] {
        scratch.put(
            &format!("crates/{name}/Cargo.toml"),
            &format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        );
        scratch.put(&format!("crates/{name}/src/lib.rs"), "");
    }
    scratch.put(
        "Cargo.lock",
        "version = 4\n\n[[package]]\nname = \"one\"\nversion = \"0.1.0\"\n\n[[package]]\nname = \"two\"\nversion = \"0.1.0\"\n",
    );
}

fn discovered(dir: &Path) -> Result<PathBuf, RootError> {
    discover(dir)
}

fn same_path(left: &Path, right: &Path) -> bool {
    fs::canonicalize(left).expect("left exists") == fs::canonicalize(right).expect("right exists")
}

#[test]
fn the_root_is_found_from_the_root_a_member_and_a_subdirectory() {
    let single_scratch = Scratch::new();
    single(&single_scratch);
    let root = discovered(single_scratch.path()).expect("a root");
    assert!(same_path(&root, single_scratch.path()), "{root:?}");
    assert!(root.is_absolute());
    // From a directory below the root that is not a package.
    single_scratch.put("src/deep/note.txt", "");
    let from_below = discovered(&single_scratch.path().join("src/deep")).expect("a root");
    assert!(
        same_path(&from_below, single_scratch.path()),
        "{from_below:?}"
    );

    let members = Scratch::new();
    virtual_workspace(&members);
    for start in ["", "crates/one", "crates/two/src"] {
        let found = discovered(&members.path().join(start)).expect("a root");
        assert!(same_path(&found, members.path()), "{start}: {found:?}");
    }
    let sibling = Scratch::new();
    sibling.put(
        "workspace/Cargo.toml",
        "[workspace]\nmembers=['../package']",
    );
    sibling.put(
        "package/Cargo.toml",
        "[package]\nname='demo'\nworkspace='../workspace'",
    );
    let root = discovered(&sibling.path().join("package")).expect("the explicit sibling workspace");
    assert!(
        same_path(&root, &sibling.path().join("workspace")),
        "{root:?}"
    );
}

#[test]
fn a_directory_outside_any_workspace_is_an_actionable_error() {
    let scratch = Scratch::new();
    scratch.put("note.txt", "no manifest here");
    let error = discovered(scratch.path()).expect_err("no workspace");
    assert!(matches!(error, RootError::Source(_)));
    assert!(error.to_string().contains("Cargo.toml"), "{error}");
}

#[test]
fn a_nonexistent_directory_is_refused() {
    let scratch = Scratch::new();
    assert!(matches!(
        discovered(&scratch.path().join("missing")),
        Err(RootError::Directory(_))
    ));
}

#[test]
fn memory_tree_membership_discovery_needs_no_disk_adapter() {
    let tree = raf_source::MemoryTree::from([
        ("Cargo.toml", "[workspace]\nmembers=['member']"),
        ("member/Cargo.toml", "[package]\nname='demo'"),
        ("member/src/note", ""),
    ]);
    let start = raf_source::RelPath::new("member/src").unwrap();
    assert_eq!(
        raf_source::discover_root(&tree, &start).unwrap(),
        raf_source::RelPath::root()
    );
}

#[cfg(unix)]
#[test]
fn a_dangling_manifest_is_a_finding_instead_of_an_ancestor_fallback() {
    let scratch = Scratch::new();
    single(&scratch);
    scratch.put("nested/note", "");
    std::os::unix::fs::symlink("missing.toml", scratch.path().join("nested/Cargo.toml")).unwrap();
    let error = discovered(&scratch.path().join("nested"))
        .expect_err("the manifest exists but cannot be read");
    assert!(error.to_string().contains("nested/Cargo.toml"), "{error}");
    // Without that entry, discovery can use the ancestor workspace.
    fs::remove_file(scratch.path().join("nested/Cargo.toml")).unwrap();
    assert!(same_path(
        &discovered(&scratch.path().join("nested")).unwrap(),
        scratch.path()
    ));
}

#[cfg(unix)]
#[test]
fn tree_directory_aliases_fail_instead_of_selecting_lexical_ancestors() {
    let scratch = Scratch::new();
    virtual_workspace(&scratch);
    std::os::unix::fs::symlink("crates/one", scratch.path().join("alias")).unwrap();
    let tree = raf_source::DiskTree::new(scratch.path()).unwrap();
    let alias = raf_source::RelPath::new("alias/src").unwrap();
    let error = raf_source::discover_root(&tree, &alias).unwrap_err();
    assert!(error.to_string().contains("directory symlinks"), "{error}");
    // The disk adapter canonicalizes first, so its public path-based entry works.
    assert!(same_path(
        &discovered(&scratch.path().join("alias/src")).unwrap(),
        scratch.path()
    ));
}
