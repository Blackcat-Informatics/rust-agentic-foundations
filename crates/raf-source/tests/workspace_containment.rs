// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The containment cases of `DiskTree`, run through `Workspace::load`. Each
//! fixture sits in a fresh temporary directory beside an outside sentinel that
//! is a FIFO wherever one is needed: reading it would block, and every load
//! runs under a watchdog, so a test that passes never read it.

#![cfg(unix)]

mod support;

use raf_source::{DiskTree, LoadError, LoadErrorKind, Origin, TargetProblem, TreeError, Workspace};
use support::{Scratch, within_watchdog};

const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";

/// `root/` is the workspace with a package; `outside/sentinel.rs` is a FIFO.
fn fixture() -> Scratch {
    let scratch = Scratch::new();
    scratch.file("root/Cargo.toml", DEMO);
    scratch.file("root/src/lib.rs", "");
    scratch.fifo("outside/sentinel.rs");
    scratch
}

fn load(scratch: &Scratch) -> Result<Workspace, LoadError> {
    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    within_watchdog(move || Workspace::load(&tree, &["."]))
}

fn outside_root(error: &LoadError) -> bool {
    matches!(&error.kind, LoadErrorKind::Tree(e) if matches!(**e, TreeError::OutsideRoot { .. }))
}

#[test]
fn a_symlinked_module_file_to_outside_is_refused_naming_the_module_beside_an_in_root_link() {
    let scratch = fixture();
    scratch.file("root/src/lib.rs", "mod inner;");
    scratch.link("root/src/inner.rs", scratch.path("outside/sentinel.rs"));
    let error = load(&scratch).expect_err("outside");
    assert!(outside_root(&error), "{error}");
    assert!(
        matches!(error.origin.as_deref(), Some(Origin::ModDecl { module }) if module == "inner"),
        "{error}"
    );
    assert_eq!(error.file.as_str(), "src/lib.rs");
    assert_eq!(error.line, Some(1));

    // A link to a file inside the root is accepted.
    let accepted = fixture();
    accepted.file("root/src/lib.rs", "mod alias;");
    accepted.file("root/src/real.rs", "pub fn go() {}");
    accepted.link("root/src/alias.rs", accepted.path("root/src/real.rs"));
    let ws = load(&accepted).expect("in-root symlink loads");
    assert_eq!(ws.files().count(), 2);
}

#[test]
fn a_path_attribute_through_an_outside_directory_symlink_with_a_parent_segment_is_refused() {
    let scratch = fixture();
    scratch.dir("outside/inner");
    scratch.fifo("outside/x.rs");
    scratch.link("root/src/vendored", scratch.path("outside/inner"));
    scratch.file(
        "root/src/lib.rs",
        "#[path = \"vendored/../x.rs\"]\nmod escaped;",
    );
    scratch.file("root/x.rs", "the lexical clamp would read this");
    let error = load(&scratch).expect_err("outside");
    assert!(outside_root(&error), "{error}");
    assert!(
        matches!(error.origin.as_deref(), Some(Origin::PathAttr { module, .. }) if module == "escaped"),
        "{error}"
    );
    assert_eq!(error.line, Some(2));

    // Through a real directory the same spelling is accepted.
    let accepted = fixture();
    accepted.dir("root/src/plain");
    accepted.file("root/src/x.rs", "pub fn go() {}");
    accepted.file("root/src/lib.rs", "#[path = \"plain/../x.rs\"]\nmod fine;");
    let ws = load(&accepted).expect("real directory loads");
    assert!(ws.files().any(|f| f.path.as_str() == "src/x.rs"));
}

#[test]
fn a_manifest_target_path_that_leaves_the_root_is_refused_naming_the_key() {
    for path in ["../outside.rs", "/tmp/outside.rs"] {
        let scratch = fixture();
        scratch.file(
            "root/Cargo.toml",
            &format!("{DEMO}\n[[bin]]\nname = \"tool\"\npath = \"{path}\"\n"),
        );
        scratch.file("outside.rs", "");
        let error = load(&scratch).expect_err(path);
        assert!(outside_root(&error), "{error}");
        assert!(
            matches!(error.origin.as_deref(), Some(Origin::ManifestKey { key }) if key == "bin.path"),
            "{error}"
        );
    }
    let scratch = fixture();
    scratch.file(
        "root/Cargo.toml",
        &format!("{DEMO}\n[[bin]]\nname = \"tool\"\npath = \"tools/tool.rs\"\n"),
    );
    scratch.file("root/tools/tool.rs", "");
    assert!(load(&scratch).is_ok());
}

#[test]
fn a_target_file_that_is_a_symlink_to_outside_is_refused_naming_the_key() {
    let scratch = fixture();
    scratch.file(
        "root/Cargo.toml",
        &format!("{DEMO}\n[[bin]]\nname = \"tool\"\npath = \"tool.rs\"\n"),
    );
    scratch.link("root/tool.rs", scratch.path("outside/sentinel.rs"));
    let error = load(&scratch).expect_err("outside");
    assert!(outside_root(&error), "{error}");
    assert!(
        matches!(error.origin.as_deref(), Some(Origin::ManifestKey { key }) if key == "bin.path"),
        "{error}"
    );
}

#[test]
fn a_symlinked_manifest_to_outside_is_refused_beside_one_to_a_file_inside() {
    let scratch = Scratch::new();
    scratch.dir("root/pkg");
    scratch.fifo("outside/Cargo.toml");
    scratch.link("root/pkg/Cargo.toml", scratch.path("outside/Cargo.toml"));
    let error = load(&scratch).expect_err("outside");
    assert!(outside_root(&error), "{error}");
    assert_eq!(error.file.as_str(), "pkg/Cargo.toml");

    let accepted = Scratch::new();
    accepted.file("root/real/Cargo.toml", DEMO);
    accepted.file("root/real/src/lib.rs", "");
    accepted.dir("root/pkg");
    accepted.link("root/pkg/Cargo.toml", accepted.path("root/real/Cargo.toml"));
    accepted.file("root/pkg/src/lib.rs", "");
    // Two packages with one name: the link is read and found to repeat `demo`.
    let error = load(&accepted).expect_err("duplicate name through the link");
    assert!(matches!(error.kind, LoadErrorKind::Target(_)), "{error}");
    let packages = within_watchdog({
        let tree = DiskTree::new(accepted.path("root/real")).expect("tree");
        move || Workspace::load(&tree, &["."]).map(|ws| ws.packages().len())
    });
    assert_eq!(packages.expect("loads"), 1);
}

#[test]
fn a_directory_symlink_loop_ends_with_an_error_and_a_plain_directory_symlink_is_refused() {
    let scratch = fixture();
    scratch.dir("root/crates");
    scratch.link("root/crates/a", "b");
    scratch.link("root/crates/b", "a");
    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    let error = within_watchdog(move || Workspace::load(&tree, &["crates"])).expect_err("loop");
    assert!(
        matches!(&error.kind, LoadErrorKind::Tree(e) if matches!(**e, TreeError::SymlinkLoop { .. })),
        "{error}"
    );

    let scratch = fixture();
    scratch.dir("root/crates/real");
    scratch.link("root/crates/link", scratch.path("root/crates/real"));
    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    let error = within_watchdog(move || Workspace::load(&tree, &["crates"])).expect_err("symlink");
    assert!(matches!(error.kind, LoadErrorKind::DirSymlink), "{error}");
    assert_eq!(error.file.as_str(), "crates/link");

    // A symlink to a file is not a directory and is left alone.
    let scratch = fixture();
    scratch.dir("root/crates/real");
    scratch.file("root/crates/readme.txt", "text");
    scratch.link(
        "root/crates/readme-link",
        scratch.path("root/crates/readme.txt"),
    );
    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    assert!(within_watchdog(move || Workspace::load(&tree, &["crates"])).is_ok());
}

#[test]
fn a_fifo_named_as_a_module_file_is_refused_without_blocking() {
    let scratch = fixture();
    scratch.file("root/src/lib.rs", "mod pipe;");
    scratch.fifo("root/src/pipe.rs");
    let error = load(&scratch).expect_err("fifo");
    assert!(
        matches!(&error.kind, LoadErrorKind::Tree(e) if matches!(**e, TreeError::NotRegular { .. })),
        "{error}"
    );
    assert!(
        matches!(error.origin.as_deref(), Some(Origin::ModDecl { module }) if module == "pipe"),
        "{error}"
    );
}

#[test]
fn a_target_root_that_is_a_fifo_is_refused_without_blocking() {
    let scratch = fixture();
    std::fs::remove_file(scratch.path("root/src/lib.rs")).expect("remove");
    scratch.fifo("root/src/lib.rs");
    let error = load(&scratch).expect_err("fifo");
    assert!(
        matches!(&error.kind, LoadErrorKind::Tree(e) if matches!(**e, TreeError::NotRegular { .. })),
        "{error}"
    );
}

#[test]
fn a_missing_explicit_target_file_is_a_target_error_on_disk_too() {
    let scratch = fixture();
    scratch.file(
        "root/Cargo.toml",
        &format!("{DEMO}\n[[bin]]\nname = \"tool\"\npath = \"gone.rs\"\n"),
    );
    let error = load(&scratch).expect_err("missing");
    assert!(
        matches!(&error.kind, LoadErrorKind::Target(p) if matches!(**p, TargetProblem::MissingFile { .. })),
        "{error}"
    );
}

#[test]
fn disk_and_memory_trees_give_the_same_workspace_for_regular_files() {
    use raf_source::MemoryTree;
    let files = [
        ("Cargo.toml", DEMO),
        ("src/lib.rs", "mod a;\n#[path = \"b/c.rs\"] mod c;"),
        ("src/a.rs", "pub fn go() {}"),
        ("src/b/c.rs", "pub struct C;"),
        ("tests/it.rs", "fn case() {}"),
    ];
    let scratch = Scratch::new();
    for (name, text) in files {
        scratch.file(&format!("root/{name}"), text);
    }
    let disk =
        Workspace::load(&DiskTree::new(scratch.path("root")).expect("tree"), &["."]).expect("disk");
    let memory =
        Workspace::load(&files.into_iter().collect::<MemoryTree>(), &["."]).expect("memory");
    let describe = |ws: &Workspace| {
        ws.files()
            .map(|f| (f.path.to_string(), f.text.clone(), f.reached.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(describe(&disk), describe(&memory));
    assert_eq!(disk.packages(), memory.packages());
    assert_eq!(
        disk.targets().cloned().collect::<Vec<_>>(),
        memory.targets().cloned().collect::<Vec<_>>()
    );
}

#[test]
fn a_link_to_a_symlink_that_points_outside_is_refused_naming_the_module() {
    let scratch = fixture();
    scratch.file("root/src/lib.rs", "mod inner;");
    scratch.link("root/src/hop.rs", scratch.path("outside/sentinel.rs"));
    scratch.link("root/src/inner.rs", scratch.path("root/src/hop.rs"));
    let error = load(&scratch).expect_err("outside");
    assert!(outside_root(&error), "{error}");
    assert!(
        matches!(error.origin.as_deref(), Some(Origin::ModDecl { module }) if module == "inner"),
        "{error}"
    );
    // The same chain inside the root is accepted.
    let accepted = fixture();
    accepted.file("root/src/lib.rs", "mod inner;");
    accepted.file("root/src/real.rs", "");
    accepted.link("root/src/hop.rs", accepted.path("root/src/real.rs"));
    accepted.link("root/src/inner.rs", accepted.path("root/src/hop.rs"));
    assert!(load(&accepted).is_ok());
}

/// Load `crates/` of a scratch tree that holds one good package and `link`.
fn discover_with(link: impl FnOnce(&Scratch)) -> Result<Workspace, LoadError> {
    let scratch = fixture();
    scratch.dir("root/crates/real");
    scratch.file("root/crates/a/Cargo.toml", "[package]\nname = \"a\"\n");
    scratch.file("root/crates/a/src/lib.rs", "");
    scratch.file("root/crates/file.txt", "text");
    link(&scratch);
    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    within_watchdog(move || Workspace::load(&tree, &["crates"]))
}

#[test]
fn discovery_classifies_a_symlink_by_what_it_resolves_to() {
    // A regular file, inside the root: left alone.
    assert!(
        discover_with(|s| {
            s.link("root/crates/to-file", s.path("root/crates/file.txt"));
        })
        .is_ok()
    );
    // A directory: DirSymlink.
    let error = discover_with(|s| {
        s.link("root/crates/to-dir", s.path("root/crates/real"));
    })
    .expect_err("directory");
    assert!(matches!(error.kind, LoadErrorKind::DirSymlink), "{error}");
    // A FIFO inside the root: NotRegular, not DirSymlink.
    let error = discover_with(|s| {
        s.fifo("root/crates/pipe");
        s.link("root/crates/to-pipe", s.path("root/crates/pipe"));
    })
    .expect_err("fifo");
    assert!(
        matches!(&error.kind, LoadErrorKind::Tree(e) if matches!(**e, TreeError::NotRegular { .. })),
        "{error}"
    );
    // Outside the root: OutsideRoot, for a file and for a directory.
    let error = discover_with(|s| {
        s.link("root/crates/to-outside", s.path("outside/sentinel.rs"));
    })
    .expect_err("outside file");
    assert!(outside_root(&error), "{error}");
    let error = discover_with(|s| {
        s.dir("outside/dir");
        s.link("root/crates/to-outside-dir", s.path("outside/dir"));
    })
    .expect_err("outside directory");
    assert!(outside_root(&error), "{error}");
    // Dangling: NotFound.
    let error = discover_with(|s| {
        s.link("root/crates/dangling", "nowhere");
    })
    .expect_err("dangling");
    assert!(
        matches!(&error.kind, LoadErrorKind::Tree(e) if matches!(**e, TreeError::NotFound { .. })),
        "{error}"
    );
}
