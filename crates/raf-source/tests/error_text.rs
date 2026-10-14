// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The text of an error a load returns carries no raw control character, even
//! when the input chose the characters.

use raf_source::{MemoryTree, Workspace};

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
