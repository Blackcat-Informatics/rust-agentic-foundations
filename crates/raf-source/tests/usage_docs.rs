// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The Usage and Examples blocks of the capability document are the first two
//! code blocks of the crate documentation, which rustdoc compiles.

use std::path::{Path, PathBuf};

fn capability_document() -> String {
    let root = std::env::var_os("RAF_GATE_ROOT").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        PathBuf::from,
    );
    std::fs::read_to_string(root.join("docs/capabilities/source-index.md")).expect("the document")
}

/// The code of the fenced blocks whose info string starts with `rust`, in
/// order, from lines that may carry a prefix such as `//! `.
fn rust_blocks(lines: &[String]) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let mut open: Option<(String, Vec<&str>)> = None;
    for line in lines {
        if let Some((info, code)) = &mut open {
            if line == "```" {
                let mut text = code.join("\n");
                text.push('\n');
                blocks.push((info.clone(), text));
                open = None;
            } else {
                code.push(line);
            }
        } else if let Some(info) = line.strip_prefix("```rust") {
            open = Some((format!("rust{info}"), Vec::new()));
        }
    }
    blocks
}

fn section(text: &str, heading: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("## ") {
            inside = name == heading;
        } else if inside {
            lines.push(line.to_owned());
        }
    }
    lines
}

fn crate_docs() -> Vec<String> {
    include_str!("../src/lib.rs")
        .lines()
        .filter_map(|line| line.strip_prefix("//!"))
        .map(|line| line.strip_prefix(' ').map_or(line, |rest| rest).to_owned())
        .collect()
}

#[test]
fn the_usage_block_is_the_first_crate_doctest_and_is_compiled_without_running() {
    let usage = rust_blocks(&section(&capability_document(), "Usage"));
    let doctests = rust_blocks(&crate_docs());
    assert_eq!(usage.len(), 1);
    assert_eq!(usage[0].0, "rust,no_run", "Usage needs a real tree");
    assert_eq!(doctests[0], usage[0]);
    assert!(
        capability_document().contains("compiled and not run"),
        "the document says why Usage is not run"
    );
}

#[test]
fn the_examples_block_is_the_second_crate_doctest() {
    let examples = rust_blocks(&section(&capability_document(), "Examples"));
    let doctests = rust_blocks(&crate_docs());
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0].0, "rust");
    assert_eq!(doctests[1], examples[0]);
}

#[test]
fn the_comparison_fails_for_a_block_that_differs_by_one_line() {
    let usage = rust_blocks(&section(&capability_document(), "Usage"));
    let mut changed = usage[0].1.clone();
    changed = changed.replacen("crates", "crate", 1);
    assert_ne!(changed, usage[0].1);
    let doctests = rust_blocks(&crate_docs());
    assert_ne!(doctests[0].1, changed);
}
