// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The containment cases of `DiskTree`, run through `Workspace::load`. Each
//! fixture sits in a fresh temporary directory beside an outside sentinel that
//! is a FIFO wherever one is needed: reading it would block, and every load
//! runs under a watchdog, so a test that passes never read it.

#![cfg(unix)]

mod support;

use std::cell::RefCell;
use std::collections::BTreeMap;

use raf_source::{
    Budget, DiskTree, Entry, EntryKind, FileId, LoadError, LoadErrorKind, Opened, Origin, RelPath,
    TargetProblem, Tree, TreeError, Workspace,
};
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

/// A disk tree that counts the reads of each file, by the file's identity.
struct CountingReads {
    inner: DiskTree,
    reads: RefCell<BTreeMap<FileId, usize>>,
}

impl CountingReads {
    fn new(scratch: &Scratch) -> Self {
        Self {
            inner: DiskTree::new(scratch.path("root")).expect("tree"),
            reads: RefCell::new(BTreeMap::new()),
        }
    }

    /// How many files were read, after checking that none was read twice.
    fn files_read_once(&self) -> usize {
        let reads = self.reads.borrow();
        assert!(reads.values().all(|count| *count == 1), "{reads:?}");
        reads.len()
    }
}

impl Tree for CountingReads {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        let id = self.inner.identity(path)?;
        *self.reads.borrow_mut().entry(id).or_default() += 1;
        self.inner.read(path, max_bytes)
    }
    fn read_unseen(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        let opened = self.inner.read_unseen(path, max_bytes, seen)?;
        if let Opened::Read { id, .. } = &opened {
            *self.reads.borrow_mut().entry(id.clone()).or_default() += 1;
        }
        Ok(opened)
    }
    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        self.inner.list(path)
    }
    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        self.inner.entry(path)
    }
}

#[test]
fn symlink_aliases_of_one_directory_parse_it_once() {
    // `l` and `m` lead back to `src`, so each level doubles the spellings of
    // the next file: eighteen levels spell the last one 2^17 ways and the walk
    // arrives 2^18 times in all, the most the module-path budget allows.
    const LEVELS: usize = 18;
    let scratch = fixture();
    scratch.link("root/src/l", ".");
    scratch.link("root/src/m", ".");
    scratch.file("root/src/lib.rs", "#[path = \"x0.rs\"]\npub mod a;\n");
    for level in 0..LEVELS {
        let text = if level + 1 == LEVELS {
            "pub fn f() {}\n".to_owned()
        } else {
            let next = level + 1;
            format!(
                "#[path = \"l/x{next}.rs\"]\npub mod a;\n#[path = \"m/x{next}.rs\"]\npub mod b;\npub fn f() {{}}\n"
            )
        };
        scratch.file(&format!("root/src/x{level}.rs"), &text);
    }
    let tree = CountingReads::new(&scratch);
    let result = Workspace::load(&tree, &["."]);
    // The manifest, the root file and each level's file, each read once.
    assert_eq!(tree.files_read_once(), 2 + LEVELS);
    // Every arrival records items, so the index passes its byte budget before
    // it holds a record for each of the 2^18 module paths.
    let error = result.expect_err("over the item budget");
    assert!(
        matches!(
            error.kind,
            LoadErrorKind::Budget {
                budget: Budget::ItemBytes,
                ..
            }
        ),
        "{error}"
    );
    assert!(error.file.as_str().starts_with("src/"), "{error}");
}

#[test]
fn a_file_reached_through_a_directory_symlink_is_one_file_under_the_spelling_that_sorts_first() {
    let scratch = fixture();
    scratch.link("root/src/z", ".");
    scratch.file(
        "root/src/lib.rs",
        "#[path = \"z/b.rs\"]\npub mod first;\n#[path = \"b.rs\"]\npub mod second;\n",
    );
    scratch.file("root/src/b.rs", "pub fn f() {}\n");
    let tree = CountingReads::new(&scratch);
    let ws = Workspace::load(&tree, &["."]).expect("loads");
    assert_eq!(tree.files_read_once(), 3);
    let paths: Vec<&str> = ws.files().map(|file| file.path.as_str()).collect();
    assert_eq!(paths, ["src/b.rs", "src/lib.rs"]);
    let modules: Vec<String> = ws
        .files()
        .next()
        .expect("b.rs")
        .reached
        .iter()
        .map(|reach| reach.module_path.to_string())
        .collect();
    assert_eq!(modules, ["demo::first", "demo::second"]);
}

#[test]
fn a_module_that_reaches_its_own_file_through_a_directory_symlink_is_a_cycle() {
    let scratch = fixture();
    scratch.link("root/src/l", ".");
    scratch.file("root/src/lib.rs", "mod x;\n");
    scratch.file("root/src/x.rs", "#[path = \"l/x.rs\"]\nmod again;\n");
    let error = load(&scratch).expect_err("a cycle");
    assert!(
        matches!(
            &error.kind,
            LoadErrorKind::ModuleCycle { module, again }
                if module == "again" && again.as_str() == "src/l/x.rs"
        ),
        "{error}"
    );
    assert_eq!(error.file.as_str(), "src/x.rs");
    assert_eq!(error.line, Some(2));
}
