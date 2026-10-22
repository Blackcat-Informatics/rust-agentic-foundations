// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The in-memory backend.

use raf_source::{Entry, EntryKind, FileId, MemoryTree, Opened, RelPath, Tree, TreeError};

fn path(text: &str) -> RelPath {
    RelPath::new(text).expect("valid path")
}

fn sample() -> MemoryTree {
    MemoryTree::from([
        ("Cargo.toml", "[package]\n"),
        ("src/lib.rs", "mod a;\n"),
        ("src/a/mod.rs", "// a\n"),
        ("src/z.rs", "// z\n"),
    ])
}

#[test]
fn a_file_reads_back_and_a_missing_file_is_not_found() {
    let tree = sample();
    assert_eq!(
        tree.read(&path("src/lib.rs"), 1024).expect("read"),
        "mod a;\n"
    );
    let missing = tree.read(&path("src/none.rs"), 1024).expect_err("missing");
    assert!(matches!(missing, TreeError::NotFound { .. }));
    assert!(missing.to_string().contains("src/none.rs"));
}

#[test]
fn a_directory_is_not_readable_as_a_file_and_a_file_is_not_listable() {
    let tree = sample();
    assert!(matches!(
        tree.read(&path("src"), 1024),
        Err(TreeError::NotRegular { .. })
    ));
    assert!(matches!(
        tree.list(&path("src/lib.rs")),
        Err(TreeError::NotRegular { .. })
    ));
    assert!(matches!(
        tree.list(&path("nowhere")),
        Err(TreeError::NotFound { .. })
    ));
}

#[test]
fn directories_are_inferred_and_listings_are_sorted_with_their_kinds() {
    let tree = sample();
    assert_eq!(
        tree.list(&RelPath::root()).expect("list"),
        [
            Entry::new("Cargo.toml", EntryKind::File),
            Entry::new("src", EntryKind::Dir),
        ]
    );
    assert_eq!(
        tree.list(&path("src")).expect("list"),
        [
            Entry::new("a", EntryKind::Dir),
            Entry::new("lib.rs", EntryKind::File),
            Entry::new("z.rs", EntryKind::File),
        ]
    );
}

#[test]
fn entry_reports_file_dir_and_absence_without_an_error() {
    let tree = sample();
    assert_eq!(
        tree.entry(&path("src/lib.rs")).expect("entry"),
        Some(EntryKind::File)
    );
    assert_eq!(
        tree.entry(&path("src/a")).expect("entry"),
        Some(EntryKind::Dir)
    );
    assert_eq!(
        tree.entry(&RelPath::root()).expect("entry"),
        Some(EntryKind::Dir)
    );
    assert_eq!(tree.entry(&path("src/none.rs")).expect("entry"), None);
}

#[test]
fn a_parent_segment_is_applied_to_the_trees_own_directories() {
    let tree = sample();
    assert_eq!(
        tree.read(&path("src/a/../lib.rs"), 1024).expect("read"),
        "mod a;\n"
    );
    assert!(matches!(
        tree.read(&path("src/a/../none.rs"), 1024),
        Err(TreeError::NotFound { .. })
    ));
}

#[test]
fn keys_that_leave_the_root_are_refused_with_outside_root_beside_a_valid_key() {
    for key in ["../a", "/a", "x/../../a"] {
        let error = MemoryTree::try_from_iter([(key, "text")]).expect_err(key);
        assert!(
            matches!(error, TreeError::OutsideRoot { .. }),
            "{key}: {error:?}"
        );
        assert!(error.to_string().contains(key));
    }
    assert!(MemoryTree::try_from_iter([("a", "text")]).is_ok());
}

#[test]
fn a_tree_built_infallibly_from_a_bad_key_reports_the_error_from_every_call() {
    for key in ["../a", "/a"] {
        let tree = MemoryTree::from([("ok.rs", "fine"), (key, "text")]);
        let read = tree.read(&path("ok.rs"), 1024).expect_err("read");
        assert!(matches!(read, TreeError::OutsideRoot { .. }), "{read:?}");
        assert_eq!(read.path(), key);
        assert!(matches!(
            tree.list(&RelPath::root()),
            Err(TreeError::OutsideRoot { .. })
        ));
        assert!(matches!(
            tree.entry(&path("ok.rs")),
            Err(TreeError::OutsideRoot { .. })
        ));
        for error in [
            tree.identity(&path("ok.rs")).expect_err("identity"),
            tree.read_unseen(&path("ok.rs"), 1024, &|_| true)
                .expect_err("even a seen identity cannot bypass an invalid tree"),
        ] {
            assert!(matches!(error, TreeError::OutsideRoot { .. }));
            assert_eq!(error.path(), key);
        }
    }
}

#[test]
fn read_and_unseen_read_apply_the_exact_byte_limit_beside_a_seen_identity() {
    let text = "é🦀";
    let tree = MemoryTree::from([("wide.rs", text)]);
    let file = path("wide.rs");
    let bytes = u64::try_from(text.len()).expect("small fixture");
    assert_eq!(tree.read(&file, bytes).expect("exact byte cap"), text);
    assert!(matches!(
        tree.read_unseen(&file, bytes, &|_| false).expect("exact unseen cap"),
        Opened::Read { text: actual, .. } if actual == text
    ));
    for error in [
        tree.read(&file, bytes - 1).expect_err("one byte below"),
        tree.read_unseen(&file, bytes - 1, &|_| false)
            .expect_err("one byte below for unseen read"),
    ] {
        assert!(matches!(error, TreeError::TooLarge { limit, .. } if limit == bytes - 1));
        assert_eq!(error.path(), "wide.rs");
    }
    let identity = tree.identity(&file).expect("identity does not read");
    assert_eq!(
        tree.read_unseen(&file, 0, &|id| *id == identity)
            .expect("seen is not read"),
        Opened::Seen(identity)
    );
}

#[test]
fn a_workspace_loads_through_an_object_safe_tree_with_caller_local_state() {
    use std::cell::Cell;
    use std::rc::Rc;

    struct LocalTree {
        files: MemoryTree,
        calls: Rc<Cell<usize>>,
    }

    impl Tree for LocalTree {
        fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
            self.calls.set(self.calls.get() + 1);
            self.files.read(path, max_bytes)
        }

        fn read_unseen(
            &self,
            path: &RelPath,
            max_bytes: u64,
            seen: &dyn Fn(&FileId) -> bool,
        ) -> Result<Opened, TreeError> {
            self.calls.set(self.calls.get() + 1);
            self.files.read_unseen(path, max_bytes, seen)
        }

        fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
            self.calls.set(self.calls.get() + 1);
            self.files.list(path)
        }

        fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
            self.calls.set(self.calls.get() + 1);
            self.files.entry(path)
        }
    }

    let calls = Rc::new(Cell::new(0));
    let tree = LocalTree {
        files: MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname='local'\nversion='0.1.0'\nedition='2021'\n",
            ),
            ("src/lib.rs", "pub fn entry() {}\n"),
        ]),
        calls: Rc::clone(&calls),
    };
    let object: &dyn Tree = &tree;
    let workspace = raf_source::Workspace::load(object, &["."]).expect("caller-local Tree");
    workspace
        .resolve("local::entry")
        .expect("actual public item");
    assert!(calls.get() > 0, "the caller-owned backend was used");
    assert!(workspace.resolve("local::missing").is_err());
}

#[test]
fn a_key_that_is_both_a_file_and_a_directory_is_refused() {
    let error = MemoryTree::try_from_iter([("a", "x"), ("a/b", "y")]).expect_err("conflict");
    assert!(matches!(error, TreeError::NotRegular { .. }));
    assert_eq!(error.path(), "a");
    assert!(MemoryTree::try_from_iter([("a", "x"), ("ab/b", "y")]).is_ok());
}

#[test]
fn collecting_pairs_builds_the_same_tree_as_the_array_conversion() {
    let collected: MemoryTree = [("a.rs", "x".to_owned())].into_iter().collect();
    assert_eq!(collected, MemoryTree::from([("a.rs", "x")]));
}

#[test]
fn duplicate_keys_are_refused_literally_and_after_normalisation_beside_distinct_keys() {
    for (first, second) in [
        ("a.rs", "a.rs"),
        ("src/../a.rs", "a.rs"),
        ("./a.rs", "a.rs"),
    ] {
        let error = MemoryTree::try_from_iter([(first, "1"), (second, "2")]).expect_err(second);
        assert!(
            matches!(error, TreeError::DuplicatePath { .. }),
            "{error:?}"
        );
        assert_eq!(error.path(), second);

        let tree = MemoryTree::from([(first, "1"), (second, "2")]);
        let read = tree.read(&path("a.rs"), 1024).expect_err("poisoned read");
        assert!(matches!(read, TreeError::DuplicatePath { .. }), "{read:?}");
        assert!(matches!(
            tree.list(&RelPath::root()),
            Err(TreeError::DuplicatePath { .. })
        ));
        assert!(matches!(
            tree.entry(&path("a.rs")),
            Err(TreeError::DuplicatePath { .. })
        ));
    }
    let tree = MemoryTree::try_from_iter([("a.rs", "1"), ("b.rs", "2")]).expect("distinct");
    assert_eq!(tree.read(&path("b.rs"), 1024).expect("read"), "2");
}

#[test]
fn the_identity_of_a_file_is_its_normalised_key_and_refuses_what_read_refuses() {
    let tree = sample();
    let lib = tree.identity(&path("src/lib.rs")).expect("a file");
    assert_eq!(
        tree.identity(&path("src/a/../lib.rs")).expect("same file"),
        lib
    );
    assert_eq!(lib, FileId::from_key("src/lib.rs"));
    let other = tree.identity(&path("src/z.rs")).expect("another file");
    assert_ne!(other, lib);
    assert!(matches!(
        tree.identity(&path("src")),
        Err(TreeError::NotRegular { .. })
    ));
    assert!(matches!(
        tree.identity(&path("src/none.rs")),
        Err(TreeError::NotFound { .. })
    ));
}

#[test]
fn one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen() {
    let tree = sample();
    let lib = FileId::from_key("src/lib.rs");
    assert_eq!(
        tree.read_unseen(&path("src/a/../lib.rs"), 1024, &|_| false)
            .expect("read"),
        Opened::Read {
            id: lib.clone(),
            text: "mod a;\n".to_owned()
        }
    );
    // A seen file is not read, so even a cap it passes is no refusal.
    assert_eq!(
        tree.read_unseen(&path("src/lib.rs"), 0, &|id| *id == lib)
            .expect("seen"),
        Opened::Seen(lib)
    );
    assert!(matches!(
        tree.read_unseen(&path("src/lib.rs"), 0, &|_| false),
        Err(TreeError::TooLarge { .. })
    ));
}
