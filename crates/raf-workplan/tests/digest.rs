// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The digest of a record input: the canonical encoding pinned against an
//! independent SHA-256, determinism, line endings, generated blocks, and the
//! definitions of a symbol.

use std::collections::BTreeSet;

use raf_okf::{Body, CapabilitySection};
use raf_source::{MemoryTree, ResolveError, Workspace};
use raf_workplan::digest::{Digest, SymbolDigestError};

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";

fn workspace(files: &[(&str, &str)]) -> Workspace {
    let mut tree: Vec<(&str, &str)> = vec![("Cargo.toml", MANIFEST)];
    tree.extend_from_slice(files);
    Workspace::load(&tree.into_iter().collect::<MemoryTree>(), &["."]).expect("the workspace loads")
}

fn body(text: &str) -> Body {
    Body::parse(text).unwrap_or_else(|error| panic!("{error}: {text}"))
}

fn contract(text: &str) -> Digest {
    Digest::section(&body(text), CapabilitySection::Contract).expect("the body has a Contract")
}

/// The digests below were computed with `sha256sum` over the encoding the
/// module documents, `<kind> NUL <name> NUL <length> NUL <text>`, so they pin
/// the encoding and the hash independently of this crate:
///
/// ```text
/// printf 'section\0contract\0%s\0Contract\n\nIt holds.\n' 20 | sha256sum
/// printf 'symbol\0demo::f\0%s\0/// Doc.\npub fn f() {}' 22 | sha256sum
/// printf 'record\0\0%s\0pub fn f() {}' 13 | sha256sum
/// ```
#[test]
fn each_kind_hashes_its_documented_encoding() {
    assert_eq!(
        contract("# T\n\n## Contract\n\nIt holds.\n").to_string(),
        "9e6c5157c556404b5ebdca48a18aaa806c8c266ae0fc2a8bed5ec1c282ea6dd9"
    );
    let source = workspace(&[("src/lib.rs", "/// Doc.\npub fn f() {}\n")]);
    assert_eq!(
        Digest::symbol(&source, "demo::f")
            .expect("resolves")
            .to_string(),
        "abe2e068b58299fce4063eb1a6062a21958bf0b1bbbea258d199943015b37b21"
    );
    assert_eq!(
        Digest::record("pub fn f() {}").to_string(),
        "4225d769b57370cd80e0d3c8b288194aa184a7858182a02f1f7bd71f463df117"
    );
}

#[test]
fn a_symbol_and_a_record_with_equal_bytes_differ() {
    // The span text of `demo::f` is exactly `pub fn f() {}`, the text of the
    // record: only the kind and the name separate the two.
    let source = workspace(&[("src/lib.rs", "pub fn f() {}\n")]);
    let symbol = Digest::symbol(&source, "demo::f").expect("resolves");
    let record = Digest::record("pub fn f() {}");
    assert_ne!(symbol, record);
    // A section whose authored text equals a record text differs from it too.
    let text = "# T\n\n## Contract\n\nIt holds.\n";
    assert_ne!(contract(text), Digest::record("Contract\n\nIt holds.\n"));
}

#[test]
fn a_digest_is_the_same_on_every_run() {
    let text = "# T\n\n## Contract\n\nIt holds.\n\n## Usage\n\nRun it.\n";
    assert_eq!(contract(text), contract(text));
    let files = [("src/lib.rs", "pub fn f() {}\npub mod m { pub fn g() {} }\n")];
    let first = workspace(&files);
    let second = workspace(&files);
    for path in ["demo::f", "demo::m::g", "demo::m"] {
        assert_eq!(
            Digest::symbol(&first, path).expect("resolves"),
            Digest::symbol(&second, path).expect("resolves"),
            "{path}"
        );
    }
    assert_eq!(Digest::record(text), Digest::record(text));
}

#[test]
fn crlf_and_lf_give_the_same_digest() {
    assert_eq!(
        contract("# T\r\n\r\n## Contract\r\n\r\nIt holds.\r\n"),
        contract("# T\n\n## Contract\n\nIt holds.\n")
    );
    let lf = workspace(&[(
        "src/lib.rs",
        "/// Doc.\n/// More.\npub fn f() {\n    run();\n}\n",
    )]);
    let crlf = workspace(&[(
        "src/lib.rs",
        "/// Doc.\r\n/// More.\r\npub fn f() {\r\n    run();\r\n}\r\n",
    )]);
    assert_eq!(
        Digest::symbol(&lf, "demo::f").expect("resolves"),
        Digest::symbol(&crlf, "demo::f").expect("resolves")
    );
    assert_eq!(
        Digest::record("---\ntype: Coverage\n---\n"),
        Digest::record("---\r\ntype: Coverage\r\n---\r\n")
    );
    // A lone carriage return is text, not a line ending, and CRLF is replaced
    // once, in each kind alike: `x\r\r\ny` keeps one CR before its LF, so it
    // differs from `x\r\ny` (which reads as `x\ny`) and from `x\ny`.
    assert_ne!(Digest::record("a\rb"), Digest::record("a\nb"));
    assert_eq!(Digest::record("x\r\ny"), Digest::record("x\ny"));
    assert_ne!(Digest::record("x\r\r\ny"), Digest::record("x\r\ny"));
    assert_ne!(Digest::record("x\r\r\ny"), Digest::record("x\ny"));
    let section = |text: &str| contract(&format!("# T\n\n## Contract\n\n{text}"));
    assert_eq!(section("x\r\ny\n"), section("x\ny\n"));
    assert_ne!(section("x\r\r\ny\n"), section("x\r\ny\n"));
}

#[test]
fn a_regenerated_block_leaves_the_section_digest_and_an_edit_moves_it() {
    let base = "# T\n\n## Contract\n\nIt reads.\n<!-- raf:generated:matrix -->\nold\n<!-- raf:generated:end -->\n\n## Usage\n\nRun it.\n";
    let synced = base.replace("\nold\n", "\nnew rows\nand more\n");
    let edited = base.replace("It reads.", "It reads and writes.");
    let usage = base.replace("Run it.", "Run it twice.");
    let removed = base.replace(
        "<!-- raf:generated:matrix -->\nold\n<!-- raf:generated:end -->\n",
        "",
    );
    assert_eq!(contract(base), contract(&synced));
    assert_ne!(contract(base), contract(&edited));
    // Another section is another input.
    assert_eq!(contract(base), contract(&usage));
    // The markers are authored: removing the block moves the digest.
    assert_ne!(contract(base), contract(&removed));
    assert_eq!(
        Digest::section(&body(base), CapabilitySection::Examples),
        None
    );
    // Two sections of one body have different digests.
    assert_ne!(
        Digest::section(&body(base), CapabilitySection::Usage),
        Some(contract(base))
    );
}

#[test]
fn a_symbol_digest_reads_the_definitions_of_the_path() {
    let base = "/// Doc.\npub fn f() {}\npub fn other() {}\npub use f as alias;\n";
    let source = workspace(&[("src/lib.rs", base)]);
    let f = Digest::symbol(&source, "demo::f").expect("resolves");
    // An edit of another item leaves it; an edit of its documentation or its
    // body moves it.
    let other = workspace(&[("src/lib.rs", &base.replace("other() {}", "other() { 1; }"))]);
    assert_eq!(Digest::symbol(&other, "demo::f").expect("resolves"), f);
    let doc = workspace(&[("src/lib.rs", &base.replace("/// Doc.", "/// Docs."))]);
    assert_ne!(Digest::symbol(&doc, "demo::f").expect("resolves"), f);
    // Moving the item down the file is no edit of its text.
    let moved = workspace(&[(
        "src/lib.rs",
        "pub fn other() {}\n/// Doc.\npub fn f() {}\npub use f as alias;\n",
    )]);
    assert_eq!(Digest::symbol(&moved, "demo::f").expect("resolves"), f);
    // A re-export is another name of the same definition: the text is the
    // same and the name is part of the digest.
    assert_ne!(Digest::symbol(&source, "demo::alias").expect("resolves"), f);
}

#[test]
fn every_definition_of_a_path_counts_once_in_file_order() {
    let lib = "#[cfg(unix)]\npub fn g() {}\n#[cfg(not(unix))]\npub fn g() {}\n";
    let source = workspace(&[("src/lib.rs", lib)]);
    assert_eq!(source.resolve("demo::g").expect("resolves").sites.len(), 2);
    let both = Digest::symbol(&source, "demo::g").expect("resolves");
    let single = workspace(&[("src/lib.rs", "#[cfg(unix)]\npub fn g() {}\n")]);
    assert_ne!(Digest::symbol(&single, "demo::g").expect("resolves"), both);
    // Each definition is part of it: editing the second moves the digest.
    let second = workspace(&[(
        "src/lib.rs",
        &lib.replace("(unix))]\npub fn g() {}", "(unix))]\npub fn g() { 2; }"),
    )]);
    assert_ne!(Digest::symbol(&second, "demo::g").expect("resolves"), both);
    // A file the index reads once for two `cfg` declarations of one module
    // gives one definition.
    let twice = workspace(&[
        (
            "src/lib.rs",
            "#[cfg(unix)]\n#[path = \"x.rs\"]\npub mod a;\n#[cfg(not(unix))]\n#[path = \"x.rs\"]\npub mod a;\n",
        ),
        ("src/x.rs", "pub fn g() {}\n"),
    ]);
    let once = workspace(&[
        ("src/lib.rs", "#[path = \"x.rs\"]\npub mod a;\n"),
        ("src/x.rs", "pub fn g() {}\n"),
    ]);
    assert_eq!(
        Digest::symbol(&twice, "demo::a::g").expect("resolves"),
        Digest::symbol(&once, "demo::a::g").expect("resolves")
    );
}

#[test]
fn a_path_that_names_no_single_symbol_has_no_digest() {
    let source = workspace(&[(
        "src/lib.rs",
        "pub fn f() {}\nmod a { pub fn x() {} }\nmod b { pub fn x() {} }\npub use a::*;\npub use b::*;\n",
    )]);
    let error = Digest::symbol(&source, "demo::missing").expect_err("no such symbol");
    assert!(matches!(
        &error,
        SymbolDigestError::Resolve {
            error: ResolveError::NotFound(_),
            ..
        }
    ));
    assert!(error.to_string().contains("demo::missing"), "{error}");
    assert!(matches!(
        Digest::symbol(&source, "demo::x"),
        Err(SymbolDigestError::Resolve {
            error: ResolveError::Ambiguous(_),
            ..
        })
    ));
    // The neighbour resolves.
    assert!(Digest::symbol(&source, "demo::f").is_ok());
}

#[test]
fn a_digest_is_written_and_read_as_sixty_four_lowercase_hex_digits() {
    let digest = Digest::record("x");
    let text = digest.to_string();
    assert_eq!(text.len(), 64);
    assert!(
        text.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    );
    assert_eq!(text.parse::<Digest>().expect("reads back"), digest);
    assert_eq!(format!("{digest:?}"), format!("Digest({text})"));
    assert_eq!(digest.bytes().len(), 32);
    for bad in [
        String::new(),
        text[..63].to_string(),
        format!("{text}0"),
        text.to_uppercase(),
        format!("g{}", &text[1..]),
        format!(" {}", &text[1..]),
    ] {
        let error = bad.parse::<Digest>().expect_err("refused");
        assert!(error.to_string().contains("64 lowercase"), "{error}");
    }
    // Digests order and hash as values.
    let set: BTreeSet<Digest> = [
        Digest::record("b"),
        Digest::record("a"),
        Digest::record("b"),
    ]
    .into_iter()
    .collect();
    assert_eq!(set.len(), 2);
}
