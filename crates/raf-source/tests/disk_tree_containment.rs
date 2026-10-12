// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A `DiskTree` never reads outside its root. Each fixture sits in a fresh
//! temporary directory beside an outside sentinel. The sentinel is a FIFO
//! wherever one is needed: a read of it would block, and every call runs
//! under a watchdog, so a test that passes never read it.

#![cfg(unix)]

mod support;

use raf_source::{DiskTree, EntryKind, Opened, RelPath, Tree, TreeError};
use support::{Scratch, within_watchdog};

const CAP: u64 = 1024;

fn path(text: &str) -> RelPath {
    RelPath::new(text).expect("valid path")
}

/// `root/` is the tree; `outside/sentinel.rs` is a FIFO that must never be read.
fn fixture() -> (Scratch, DiskTree) {
    let scratch = Scratch::new();
    scratch.dir("root");
    scratch.fifo("outside/sentinel.rs");
    scratch.file("root/real.rs", "real");
    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    (scratch, tree)
}

fn read_within_watchdog(tree: DiskTree, text: &'static str) -> Result<String, TreeError> {
    within_watchdog(move || tree.read(&path(text), CAP))
}

#[test]
fn a_symlink_to_a_file_outside_is_refused_beside_an_in_root_symlink_that_is_accepted() {
    let (scratch, tree) = fixture();
    scratch.link("root/out.rs", scratch.path("outside/sentinel.rs"));
    scratch.link("root/alias.rs", scratch.path("root/real.rs"));
    scratch.link("root/relative.rs", "real.rs");

    let refused = read_within_watchdog(tree, "out.rs").expect_err("outside");
    assert!(
        matches!(refused, TreeError::OutsideRoot { .. }),
        "{refused:?}"
    );
    assert!(refused.to_string().contains("out.rs"));

    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    assert_eq!(tree.read(&path("alias.rs"), CAP).expect("alias"), "real");
    assert_eq!(
        tree.read(&path("relative.rs"), CAP).expect("relative"),
        "real"
    );
}

#[test]
fn a_symlink_to_a_symlink_that_leaves_the_root_is_refused_beside_a_chain_that_stays() {
    let (scratch, tree) = fixture();
    scratch.link("root/second.rs", scratch.path("outside/sentinel.rs"));
    scratch.link("root/first.rs", scratch.path("root/second.rs"));
    scratch.link("root/c.rs", scratch.path("root/real.rs"));
    scratch.link("root/b.rs", scratch.path("root/c.rs"));

    let refused = read_within_watchdog(tree, "first.rs").expect_err("outside");
    assert!(
        matches!(refused, TreeError::OutsideRoot { .. }),
        "{refused:?}"
    );

    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    assert_eq!(tree.read(&path("b.rs"), CAP).expect("chain"), "real");
}

#[test]
fn a_parent_segment_after_an_outside_directory_symlink_is_resolved_on_disk_and_refused() {
    let (scratch, tree) = fixture();
    // `vendored/..` is `outside`, not the root: the text and the disk disagree.
    scratch.dir("outside/inner");
    scratch.link("root/vendored", scratch.path("outside/inner"));
    scratch.file("root/x.rs", "the lexical clamp would read this");
    scratch.fifo("outside/x.rs");
    scratch.dir("root/plain");

    let refused = read_within_watchdog(tree, "vendored/../x.rs").expect_err("outside");
    assert!(
        matches!(refused, TreeError::OutsideRoot { .. }),
        "{refused:?}"
    );
    assert!(refused.to_string().contains("vendored/../x.rs"));

    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    assert_eq!(
        tree.read(&path("plain/../x.rs"), CAP)
            .expect("real directory"),
        "the lexical clamp would read this"
    );
    assert!(matches!(
        tree.entry(&path("vendored/../x.rs")),
        Err(TreeError::OutsideRoot { .. })
    ));
}

#[test]
fn a_directory_symlink_loop_ends_with_an_error_and_is_listed_but_never_entered() {
    let (scratch, tree) = fixture();
    scratch.link("root/a", "b");
    scratch.link("root/b", "a");
    scratch.link("root/selfie", ".");
    scratch.dir("root/real_dir");
    scratch.file("root/real_dir/inner.rs", "inner");

    let through = tree.read(&path("a/x.rs"), CAP).expect_err("loop");
    assert!(
        matches!(through, TreeError::SymlinkLoop { .. }),
        "{through:?}"
    );
    assert!(matches!(
        tree.list(&path("a")),
        Err(TreeError::SymlinkLoop { .. })
    ));

    let names: Vec<(String, EntryKind)> = tree
        .list(&RelPath::root())
        .expect("list root")
        .into_iter()
        .map(|entry| (entry.name, entry.kind))
        .collect();
    assert_eq!(
        names,
        [
            ("a".to_owned(), EntryKind::Symlink),
            ("b".to_owned(), EntryKind::Symlink),
            ("real.rs".to_owned(), EntryKind::File),
            ("real_dir".to_owned(), EntryKind::Dir),
            ("selfie".to_owned(), EntryKind::Symlink),
        ]
    );

    // A symlink to a directory inside the root is not entered either.
    assert!(matches!(
        tree.list(&path("selfie")),
        Err(TreeError::NotRegular { .. })
    ));
    assert_eq!(tree.list(&path("real_dir")).expect("real dir").len(), 1);
}

#[test]
fn a_fifo_named_as_a_file_is_refused_without_blocking_beside_a_regular_file() {
    let (scratch, _tree) = fixture();
    scratch.fifo("root/pipe.rs");
    let tree = DiskTree::new(scratch.path("root")).expect("tree");

    let (refused, kind, accepted) = within_watchdog(move || {
        (
            tree.read(&path("pipe.rs"), CAP),
            tree.entry(&path("pipe.rs")),
            tree.read(&path("real.rs"), CAP),
        )
    });

    assert!(
        matches!(refused, Err(TreeError::NotRegular { .. })),
        "{refused:?}"
    );
    assert_eq!(kind.expect("entry"), Some(EntryKind::Other));
    assert_eq!(accepted.expect("regular"), "real");
}

#[test]
fn a_directory_named_as_a_file_is_refused() {
    let (scratch, tree) = fixture();
    scratch.dir("root/sub");
    assert!(matches!(
        tree.read(&path("sub"), CAP),
        Err(TreeError::NotRegular { .. })
    ));
    assert!(matches!(
        tree.list(&path("real.rs")),
        Err(TreeError::NotRegular { .. })
    ));
}

#[test]
fn a_missing_root_is_an_error_beside_an_existing_one() {
    let scratch = Scratch::new();
    scratch.dir("root");
    let missing = DiskTree::new(scratch.path("absent")).expect_err("missing root");
    assert!(matches!(missing, TreeError::NotFound { .. }), "{missing:?}");
    assert!(missing.to_string().contains("absent"));
    assert!(DiskTree::new(scratch.path("root")).is_ok());

    scratch.file("a_file", "x");
    assert!(matches!(
        DiskTree::new(scratch.path("a_file")),
        Err(TreeError::NotRegular { .. })
    ));
}

#[test]
fn a_symlinked_root_is_followed_to_its_target_and_containment_is_judged_there() {
    let (scratch, _tree) = fixture();
    scratch.link("root/to_outside.rs", scratch.path("outside/sentinel.rs"));
    scratch.link("alias_root", scratch.path("root"));
    let tree = DiskTree::new(scratch.path("alias_root")).expect("tree through the alias");

    assert_eq!(tree.read(&path("real.rs"), CAP).expect("read"), "real");
    let refused = read_within_watchdog(tree, "to_outside.rs").expect_err("outside");
    assert!(
        matches!(refused, TreeError::OutsideRoot { .. }),
        "{refused:?}"
    );
}

#[test]
fn a_file_that_is_missing_is_not_found_and_one_that_leaves_the_root_is_outside_root() {
    let (scratch, tree) = fixture();
    scratch.link("root/out.rs", scratch.path("outside/sentinel.rs"));
    scratch.link("root/dangling.rs", "nothing_here.rs");

    let missing = tree.read(&path("missing.rs"), CAP).expect_err("missing");
    assert!(matches!(missing, TreeError::NotFound { .. }), "{missing:?}");
    let dangling = tree.read(&path("dangling.rs"), CAP).expect_err("dangling");
    assert!(
        matches!(dangling, TreeError::NotFound { .. }),
        "{dangling:?}"
    );
    let tree2 = DiskTree::new(scratch.path("root")).expect("tree");
    let outside = read_within_watchdog(tree2, "out.rs").expect_err("outside");
    assert!(
        matches!(outside, TreeError::OutsideRoot { .. }),
        "{outside:?}"
    );
}

#[test]
fn a_file_that_is_not_utf8_is_refused_beside_one_that_is() {
    let (scratch, tree) = fixture();
    std::fs::write(scratch.path("root/bad.rs"), [b'a', 0xFF, b'b']).expect("write");
    let refused = tree.read(&path("bad.rs"), CAP).expect_err("not UTF-8");
    assert!(matches!(refused, TreeError::NotUtf8 { .. }), "{refused:?}");
    assert!(refused.to_string().contains("bad.rs"));
    assert_eq!(tree.read(&path("real.rs"), CAP).expect("UTF-8"), "real");
}

#[test]
fn a_non_utf8_entry_name_fails_the_listing_beside_a_listing_without_one() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt as _;

    let (scratch, tree) = fixture();
    scratch.dir("root/clean");
    scratch.file("root/clean/ok.rs", "ok");
    scratch.dir("root/dirty");
    std::fs::write(
        scratch
            .path("root/dirty")
            .join(OsStr::from_bytes(b"bad\xff.rs")),
        "x",
    )
    .expect("write a file with a non-UTF-8 name");

    let refused = tree.list(&path("dirty")).expect_err("non-UTF-8 name");
    assert!(
        matches!(&refused, TreeError::NonUtf8Name { path, .. } if path == "dirty"),
        "{refused:?}"
    );
    assert_eq!(tree.list(&path("clean")).expect("clean").len(), 1);
}

#[test]
fn entry_kinds_cover_file_directory_symlink_other_and_absent() {
    let (scratch, tree) = fixture();
    scratch.dir("root/sub");
    scratch.fifo("root/pipe");
    scratch.link("root/link", scratch.path("outside/sentinel.rs"));

    for (name, expected) in [
        ("real.rs", Some(EntryKind::File)),
        ("sub", Some(EntryKind::Dir)),
        ("pipe", Some(EntryKind::Other)),
        ("link", Some(EntryKind::Symlink)),
        ("absent", None),
        ("absent/deeper.rs", None),
        ("real.rs/deeper.rs", None),
        ("", Some(EntryKind::Dir)),
        ("sub/..", Some(EntryKind::Dir)),
    ] {
        assert_eq!(tree.entry(&path(name)).expect(name), expected, "{name}");
    }
}

#[test]
fn listings_are_sorted_by_name() {
    let (scratch, tree) = fixture();
    for name in ["m.rs", "B.rs", "a.rs", "z.rs"] {
        scratch.file(&format!("root/{name}"), "x");
    }
    let names: Vec<String> = tree
        .list(&RelPath::root())
        .expect("list")
        .into_iter()
        .map(|entry| entry.name)
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    assert_eq!(names, ["B.rs", "a.rs", "m.rs", "real.rs", "z.rs"]);
}

#[test]
fn the_identity_of_a_file_is_the_device_and_inode_of_its_verified_handle() {
    let (scratch, tree) = fixture();
    scratch.link("root/alias.rs", scratch.path("root/real.rs"));
    std::fs::hard_link(scratch.path("root/real.rs"), scratch.path("root/hard.rs"))
        .expect("hard link");
    scratch.dir("root/sub");
    scratch.link("root/through", scratch.path("root/sub"));
    scratch.file("root/sub/other.rs", "other");
    let real = tree.identity(&path("real.rs")).expect("a file");
    // A symlink, a hard link and a path through a directory symlink name the
    // file they lead to; a different file differs.
    assert_eq!(tree.identity(&path("alias.rs")).expect("alias"), real);
    assert_eq!(tree.identity(&path("hard.rs")).expect("hard link"), real);
    assert_eq!(
        tree.identity(&path("sub/../real.rs")).expect("parent"),
        real
    );
    let other = tree.identity(&path("sub/other.rs")).expect("other");
    assert_ne!(other, real);
    assert_eq!(
        tree.identity(&path("through/other.rs")).expect("through"),
        other
    );
    // What `read` refuses, `identity` refuses, without reading.
    assert!(matches!(
        tree.identity(&path("sub")),
        Err(TreeError::NotRegular { .. })
    ));
    assert!(matches!(
        tree.identity(&path("missing.rs")),
        Err(TreeError::NotFound { .. })
    ));
    scratch.link("root/out.rs", scratch.path("outside/sentinel.rs"));
    let outside = within_watchdog(move || tree.identity(&path("out.rs")));
    assert!(
        matches!(outside, Err(TreeError::OutsideRoot { .. })),
        "{outside:?}"
    );
}

#[test]
fn one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen() {
    let (scratch, tree) = fixture();
    scratch.link("root/alias.rs", scratch.path("root/real.rs"));
    let real = tree.identity(&path("real.rs")).expect("a file");
    match tree.read_unseen(&path("alias.rs"), CAP, &|_| false) {
        Ok(Opened::Read { id, text }) => assert_eq!((id, text.as_str()), (real.clone(), "real")),
        other => panic!("{other:?}"),
    }
    let seen = real.clone();
    assert_eq!(
        tree.read_unseen(&path("alias.rs"), 0, &move |id| *id == seen)
            .expect("seen"),
        Opened::Seen(real)
    );
    assert!(matches!(
        tree.read_unseen(&path("real.rs"), 1, &|_| false),
        Err(TreeError::TooLarge { .. })
    ));
}
