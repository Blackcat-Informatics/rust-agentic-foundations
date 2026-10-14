// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The text of an error a load returns carries no raw control character, even
//! when the input chose the characters.

use raf_source::{MemoryTree, ResolveError, Workspace};

#[test]
fn a_path_attribute_with_terminal_escapes_is_shown_escaped() {
    let lib = "#[path = \"../../../\\u{1b}]0;pwned\\u{7}\\u{1b}[2J\\u{1b}[31mfake.rs\"] mod m;\n";
    let tree: MemoryTree = [
        (
            "p/Cargo.toml",
            "[package]\nname = \"p\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
        ),
        ("p/src/lib.rs", lib),
    ]
    .into_iter()
    .collect();
    let text = Workspace::load(&tree, &["p"])
        .expect_err("the path leaves the root")
        .to_string();
    assert!(!text.chars().any(char::is_control), "{text:?}");
    assert!(text.contains(r"\u{1b}]0;pwned\u{7}\u{1b}[2J"), "{text}");
}

#[test]
fn a_manifest_with_control_characters_in_a_value_is_shown_escaped() {
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"a\\u001b[2Jb\\\\\"]\n[workspace.package]\nedition = \"2021\"\n",
        ),
        (
            "p/Cargo.toml",
            "[package]\nname = \"p\"\nversion = \"0.0.0\"\nedition.workspace = true\n",
        ),
        ("p/src/lib.rs", ""),
    ]
    .into_iter()
    .collect();
    let text = Workspace::load(&tree, &["p"])
        .expect_err("the pattern is outside the syntax")
        .to_string();
    assert!(!text.chars().any(char::is_control), "{text:?}");
    assert!(text.contains(r"a\u{1b}[2Jb"), "{text}");
}

#[test]
fn a_resolved_path_or_package_with_control_characters_is_shown_escaped() {
    let tree: MemoryTree = [
        (
            "p/Cargo.toml",
            "[package]\nname = \"p\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
        ),
        ("p/src/lib.rs", "pub fn f() {}\n"),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["p"]).unwrap_or_else(|e| panic!("{e}"));
    let path = "p::\u{1b}]0;pwned\u{7}\u{202e}f";
    let invalid = ws.resolve(path);
    assert_eq!(invalid, Err(ResolveError::Invalid(path.to_owned())));
    let unknown = ws.resolve_in_package(path, "p::f");
    assert_eq!(unknown, Err(ResolveError::UnknownPackage(path.to_owned())));
    for error in [invalid, unknown] {
        let text = error.expect_err("refused").to_string();
        assert!(
            !text.chars().any(|c| c.is_control() || c == '\u{202e}'),
            "{text:?}"
        );
        assert!(text.contains(r"p::\u{1b}]0;pwned\u{7}\u{202e}f"), "{text}");
    }
    assert!(ws.resolve("p::f").is_ok());
}
