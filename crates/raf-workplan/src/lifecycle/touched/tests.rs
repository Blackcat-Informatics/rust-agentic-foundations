// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The units of a side, the token streams they are compared by, the headers
//! of impl blocks and the tree of the merge base, against memory trees and
//! scripted `git` answers.

use std::collections::BTreeSet;

use raf_source::{EntryKind, MemoryTree, RelPath, Tree, TreeError, Workspace};

use super::{BaseTree, Reader, Surface, impl_headers, load_at};
use crate::exec::{Call, Commit, FakeRunner, Output, Program};
use crate::git::{self, GitError, MergeBase, Prefix};
use crate::lifecycle::{LifecycleError, Side};

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

fn path(text: &str) -> RelPath {
    RelPath::new(text).expect("a path")
}

/// The workspace of one library whose `src/lib.rs` is `lib`.
fn workspace(lib: &str) -> Workspace {
    Workspace::load(
        &MemoryTree::from([("Cargo.toml", MANIFEST), ("src/lib.rs", lib)]),
        &["."],
    )
    .expect("the library loads")
}

/// The rendered stream of the item `item` of the library `lib`, read as the
/// rule reads it: the bytes of its span in the source index.
fn stream(lib: &str, item: &str) -> String {
    let source = workspace(lib);
    let found = source
        .items()
        .find(|candidate| *candidate.path == *item)
        .unwrap_or_else(|| panic!("no {item}"));
    let text = source.source(&found.file).expect("indexed");
    Reader::new(text).stream_of(found.span.start_byte..found.span.end_byte)
}

/// The rendered stream of `text` as a whole.
fn rendered(text: &str) -> String {
    Reader::new(text).stream_of(0..text.len())
}

#[test]
fn whitespace_and_plain_comments_leave_the_stream_and_a_doc_comment_or_a_token_changes_it() {
    let base = "/// Adds.\npub fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n";
    let same = [
        "/// Adds.\npub fn add(a: u32, b: u32) -> u32 { a + b }\n",
        "/// Adds.\npub fn add(\n    a: u32,\n    b: u32\n) -> u32 {\n    // plain\n    a + b /* block */\n}\n",
        "/// Adds.\r\npub fn add(a: u32, b: u32) -> u32 {\r\n    a + b\r\n}\r\n",
        "/// Adds.\npub fn add(a:u32,b:u32)->u32{\n\ta+b\n}\n",
        "#[doc = \" Adds.\"]\npub fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n",
    ];
    for text in same {
        assert_eq!(
            stream(text, "demo::add"),
            stream(base, "demo::add"),
            "{text}"
        );
    }
    // A trailing comma is a token, so a reflow that adds one (as rustfmt does
    // for one parameter per line) touches the item; so does a trailing space
    // inside a doc comment, which is text of its `#[doc]` attribute.
    let different = [
        "/// Adds two.\npub fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n",
        "/// Adds. \npub fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n",
        "/** Adds. */\npub fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n",
        "/// Adds.\npub fn add(a: u32, b: u32) -> u32 {\n    b + a\n}\n",
        "/// Adds.\n#[must_use]\npub fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n",
        "/// Adds.\npub(crate) fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n",
        "/// Adds.\npub fn add(a: u64, b: u32) -> u32 {\n    a + b\n}\n",
        "/// Adds.\npub fn add(\n    a: u32,\n    b: u32,\n) -> u32 {\n    a + b\n}\n",
    ];
    for text in different {
        assert_ne!(
            stream(text, "demo::add"),
            stream(base, "demo::add"),
            "{text}"
        );
    }
}

#[test]
fn the_surface_holds_the_public_items_impl_members_by_their_header_and_trait_impl_members_of_public_types_only()
 {
    let lib = "pub mod inner {\n    pub fn go() {}\n    fn helper() {}\n}\n#[derive(PartialEq)]\npub struct Frame;\nimpl Frame {\n    pub fn push(&self) {}\n    fn grow(&self) {}\n}\nimpl Clone for Frame {\n    fn clone(&self) -> Self { Frame }\n}\nimpl Eq for Frame {}\npub trait Shape {\n    fn area(&self) -> u32;\n}\nstruct Quiet;\nimpl Clone for Quiet {\n    fn clone(&self) -> Self { Quiet }\n}\n#[doc(hidden)]\npub fn hidden() {}\npub trait Foreign { fn foreign(&self); }\nimpl Foreign for Vec<u8> { fn foreign(&self) {} }\npub(crate) fn narrow() {}\nmacro_rules! make { () => {} }\npub use inner::go as run;\n";
    let source = workspace(lib);
    let packages = BTreeSet::from(["demo"]);
    let surface = Surface::of(&source, &packages, Side::WorkingTree).expect("a surface");
    let keys: Vec<(&str, String)> = surface
        .units
        .keys()
        .map(|key| (key.path.as_str(), key.header.clone()))
        .collect();
    assert_eq!(
        keys,
        [
            ("demo::Foreign", String::new()),
            ("demo::Foreign::foreign", String::new()),
            ("demo::Frame", String::new()),
            ("demo::Frame::clone", rendered("impl Clone for Frame")),
            ("demo::Frame::push", rendered("impl Frame")),
            ("demo::Shape", String::new()),
            ("demo::Shape::area", String::new()),
            ("demo::inner", String::new()),
            ("demo::inner::go", String::new()),
        ]
    );
    // Each path of the surface names its unit: a re-export names the item it
    // exports.
    assert_eq!(
        surface
            .exports
            .get("demo::run")
            .map(|key| key.path.as_str()),
        Some("demo::inner::go")
    );
    assert_eq!(
        surface
            .exports
            .get("demo::inner::go")
            .map(|key| key.path.as_str()),
        Some("demo::inner::go")
    );
    // Removing excluded syntax changes no compared identity or token stream:
    // an empty impl, a hidden function and an impl for a foreign self type.
    let reduced = lib
        .replace("impl Eq for Frame {}", "")
        .replace("#[doc(hidden)]\npub fn hidden() {}", "")
        .replace("impl Foreign for Vec<u8> { fn foreign(&self) {} }", "");
    let reduced_source = workspace(&reduced);
    let reduced_surface =
        Surface::of(&reduced_source, &packages, Side::WorkingTree).expect("a reduced surface");
    let entries = |surface: &Surface<'_>| {
        surface
            .units
            .iter()
            .map(|(key, unit)| (key.clone(), unit.stream.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(entries(&surface), entries(&reduced_surface));
    assert_eq!(surface.exports, reduced_surface.exports);
}

#[test]
fn an_impl_header_is_the_tokens_before_its_brace_at_every_depth_and_a_file_that_does_not_parse_is_an_error()
 {
    let text = "pub struct W<T>(T);\n#[cfg(any())]\nimpl<T: Clone> W<T> where T: Copy {\n    pub fn get(&self) {}\n}\nmod m {\n    impl super::W<u8> {}\n}\n";
    let headers = impl_headers(&path("src/lib.rs"), text).expect("it parses");
    let first = text.find("impl<").expect("the first impl") - "#[cfg(any())]\n".len();
    let open = text.find(" {\n    pub fn").expect("its brace") + 1;
    assert_eq!(
        headers[0].header,
        rendered("#[cfg(any())]\nimpl<T: Clone> W<T> where T: Copy")
    );
    assert_eq!(headers[0].body.start, open);
    assert_eq!(
        &text[first..open],
        "#[cfg(any())]\nimpl<T: Clone> W<T> where T: Copy "
    );
    assert!(
        headers[0]
            .body
            .contains(&text.find("pub fn get").expect("get"))
    );
    assert_eq!(headers[1].header, rendered("impl super::W<u8>"));
    assert_eq!(headers.len(), 2);
    assert!(impl_headers(&path("src/lib.rs"), "pub fn (").is_err());
}

fn said(text: &str) -> Output {
    Output::new(Some(0), text.as_bytes().to_vec(), Vec::new())
}

/// The merge base of the scripted answers.
fn base() -> MergeBase {
    let commit = Commit::new(COMMIT).expect("a commit id");
    let runner = FakeRunner::new().on(
        Program::Git,
        &["merge-base", "HEAD", COMMIT],
        Ok(said(&format!("{COMMIT}\n"))),
    );
    git::merge_base(&runner, &commit).expect("a merge base")
}

/// The prefix `text` as `git rev-parse --show-prefix` prints it.
fn prefix(text: &str) -> Prefix {
    let runner = FakeRunner::new().on(
        Program::Git,
        &["rev-parse", "--show-prefix"],
        Ok(said(&format!("{text}\n"))),
    );
    git::show_prefix(&runner).expect("a prefix")
}

const LISTING: &str = "100644 blob 1111111111111111111111111111111111111111\tCargo.toml\x00100644 blob 2222222222222222222222222222222222222222\tsrc/lib.rs\x00120000 blob 3333333333333333333333333333333333333333\tsrc/link.rs\x00160000 commit 4444444444444444444444444444444444444444\tvendor/sub\x00100644 blob 5555555555555555555555555555555555555555\tother/README.md\x00";

/// A runner that answers the listing below `top` and the blob of each of
/// `files`, named from the top of the work tree.
fn scripted(top: &str, files: &[(&str, &str)]) -> FakeRunner {
    let mut listing = String::new();
    for record in LISTING.split('\0').filter(|record| !record.is_empty()) {
        let (head, name) = record.split_once('\t').expect("a record");
        listing.push_str(head);
        listing.push('\t');
        if !name.starts_with("other/") {
            listing.push_str(top);
        }
        listing.push_str(name);
        listing.push('\0');
    }
    let mut runner = FakeRunner::new().on(
        Program::Git,
        &["ls-tree", "-r", "-z", "--full-tree", COMMIT],
        Ok(said(&listing)),
    );
    for (name, text) in files {
        runner = runner.on(
            Program::Git,
            &["cat-file", "blob", &format!("{COMMIT}:{top}{name}")],
            Ok(said(text)),
        );
    }
    runner
}

fn commands(runner: &FakeRunner) -> Vec<String> {
    runner.calls().iter().map(Call::command).collect()
}

#[test]
fn the_tree_of_the_merge_base_answers_from_one_listing_and_reads_each_file_once_with_the_blob_row()
{
    for top in ["", "services/api/"] {
        let runner = scripted(
            top,
            &[
                ("Cargo.toml", MANIFEST),
                ("src/lib.rs", "pub fn go() {}\n"),
                ("src/link.rs", "lib.rs"),
            ],
        );
        let (base, prefix) = (base(), prefix(top));
        let listing = git::tree_at(&runner, base.resolved(), &prefix).expect("a listing");
        let tree = BaseTree::new(&runner, &base, &prefix, listing);

        // The listing names the files below the prefix and their directories.
        assert_eq!(
            tree.entry(&path("src")).expect("an answer"),
            Some(EntryKind::Dir)
        );
        assert_eq!(
            tree.entry(&path("src/lib.rs")).expect("an answer"),
            Some(EntryKind::File)
        );
        assert_eq!(
            tree.entry(&path("src/link.rs")).expect("an answer"),
            Some(EntryKind::Symlink)
        );
        assert_eq!(
            tree.entry(&path("vendor/sub")).expect("an answer"),
            Some(EntryKind::Other)
        );
        assert_eq!(
            tree.entry(&path("src/missing.rs")).expect("an answer"),
            None
        );
        assert_eq!(
            tree.entry(&path("other")).expect("an answer"),
            top.is_empty().then_some(EntryKind::Dir),
            "{top}"
        );
        let names: Vec<(String, EntryKind)> = tree
            .list(&RelPath::root())
            .expect("the root")
            .into_iter()
            .map(|entry| (entry.name, entry.kind))
            .collect();
        let mut expected = vec![
            ("Cargo.toml".to_string(), EntryKind::File),
            ("src".to_string(), EntryKind::Dir),
            ("vendor".to_string(), EntryKind::Dir),
        ];
        if top.is_empty() {
            expected.insert(1, ("other".to_string(), EntryKind::Dir));
        }
        assert_eq!(names, expected);
        assert!(matches!(
            tree.list(&path("src/lib.rs")),
            Err(TreeError::NotRegular { .. })
        ));
        assert!(matches!(
            tree.read(&path("src"), 1024),
            Err(TreeError::NotRegular { .. })
        ));
        assert!(matches!(
            tree.read(&path("src/missing.rs"), 1024),
            Err(TreeError::NotFound { .. })
        ));
        assert!(matches!(
            tree.read(&path("src/lib.rs"), 3),
            Err(TreeError::TooLarge { .. })
        ));

        // A file is read once however often the load asks, and a link reads
        // the file it names.
        assert_eq!(
            tree.read(&path("src/lib.rs"), 1024).expect("a text"),
            "pub fn go() {}\n"
        );
        assert_eq!(
            tree.read(&path("src/link.rs"), 1024).expect("a text"),
            "pub fn go() {}\n"
        );
        let source = Workspace::load(&tree, &["."]).expect("the merge base loads");
        assert!(source.items().any(|item| *item.path == *"demo::go"));
        let asked = commands(&runner);
        assert_eq!(
            asked,
            [
                format!("git ls-tree -r -z --full-tree {COMMIT}"),
                format!("git cat-file blob {COMMIT}:{top}src/lib.rs"),
                format!("git cat-file blob {COMMIT}:{top}src/link.rs"),
                format!("git cat-file blob {COMMIT}:{top}Cargo.toml"),
            ],
            "{top}"
        );
    }
}

#[test]
fn a_git_failure_while_the_merge_base_loads_is_that_error_and_a_file_that_is_not_utf8_names_the_file()
 {
    // `src/lib.rs` has no scripted answer, so `git` fails on it.
    let runner = scripted("", &[("Cargo.toml", MANIFEST)]);
    let (base, prefix) = (base(), prefix(""));
    let error = load_at(&runner, &base, &prefix, &[".".to_string()]).expect_err("no answer");
    assert!(
        matches!(&error, LifecycleError::Git(GitError::Exec(_))),
        "{error:?}"
    );

    let runner = scripted("", &[("Cargo.toml", MANIFEST)]).on(
        Program::Git,
        &["cat-file", "blob", &format!("{COMMIT}:src/lib.rs")],
        Ok(Output::new(Some(0), vec![0xff, 0xfe], Vec::new())),
    );
    let error = load_at(&runner, &base, &prefix, &[".".to_string()]).expect_err("not UTF-8");
    assert!(
        matches!(
            &error,
            LifecycleError::Document { path, side: Side::MergeBase, .. } if path == "src/lib.rs"
        ),
        "{error:?}"
    );

    // The neighbour: the same tree with a text file loads, and a root that is
    // not a directory at the merge base loads nothing and reads nothing.
    let runner = scripted(
        "",
        &[("Cargo.toml", MANIFEST), ("src/lib.rs", "pub fn go() {}\n")],
    );
    assert!(
        load_at(&runner, &base, &prefix, &[".".to_string()])
            .expect("it loads")
            .is_some()
    );
    let runner = scripted("", &[]);
    assert!(
        load_at(&runner, &base, &prefix, &["crates/gone".to_string()])
            .expect("nothing to load")
            .is_none()
    );
    assert_eq!(
        commands(&runner),
        [format!("git ls-tree -r -z --full-tree {COMMIT}")]
    );
}
