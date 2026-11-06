// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The units of a side, the token streams they are compared by, the headers
//! of impl blocks and the tree of the merge base, against memory trees and
//! scripted `git` answers.

use std::collections::BTreeSet;

use raf_source::{MemoryTree, RelPath, Workspace};

use super::{Reader, Surface, impl_headers};
use crate::lifecycle::Side;
use crate::obligation::Touch;

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

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

#[test]
fn static_surface_comparison_classifies_changes_without_executing_source() {
    let then = workspace(
        "pub mod api { pub fn steady() {} pub fn changed() {} pub fn hidden() {} pub fn removed() {} pub fn left() {} pub fn right() {} pub use left as selected; } #[test] fn check() {}",
    );
    let now = workspace(
        "pub mod api { pub fn steady() {} pub fn changed() { panic!(\"inert source\"); } fn hidden() {} pub fn added() {} pub fn left() {} pub fn right() {} pub use right as selected; } #[test] fn check() {}",
    );
    let docs = MemoryTree::from([(
        "docs/capabilities/api.md",
        "---\ntype: Capability\nid: cap:api\ntitle: Static surface\ndescription: Classify source changes as data.\nstate: implementing\nbindings:\n  code: [demo::api]\n  tests: [demo::check]\n  examples: []\ndepends: []\n---\n\n## Purpose\n\nDescribe changes.\n\n## Contract\n\nClassify changes.\n\n## Usage\n\nInspect data.\n\n## Examples\n\nCompare declarations.\n\n## Constraints\n\nNo execution.\n\n## Explanation\n\nSource is data.\n\n## Verification\n\nCheck classifications.\n",
    )]);
    let plan = crate::Workplan::load(&docs, &now);
    assert_eq!(plan.capabilities().len(), 1);
    let packages = BTreeSet::from(["demo"]);
    let old = Surface::of(&then, &packages, Side::MergeBase).unwrap();
    let new = Surface::of(&now, &packages, Side::WorkingTree).unwrap();
    let touched = super::compare(&plan, &new, Some(&then), &old);
    let changes: std::collections::BTreeMap<_, _> = touched
        .iter()
        .map(|item| (item.item.as_str(), item.how))
        .collect();
    for (path, kind) in [
        ("demo::api::changed", Touch::Tokens),
        ("demo::api::added", Touch::New),
        ("demo::api::hidden", Touch::Hidden),
        ("demo::api::removed", Touch::Removed),
        ("demo::api::selected", Touch::Retargeted),
    ] {
        assert_eq!(changes.get(path), Some(&kind), "{changes:?}");
    }
    assert!(!changes.contains_key("demo::api::steady"));
}
