// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The crate reads and writes text only: its sources name neither the
//! file-system module nor the process module of the standard library.

use std::fs;
use std::path::{Path, PathBuf};

/// The standard-library modules the crate must not use.
const FORBIDDEN: [&str; 2] = ["fs", "process"];

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The 1-based lines of `source` that name a forbidden `std` module, in any
/// spelling: `std::fs`, `std :: process`, `use std::{io, fs}` or a mention in
/// a comment. Comments and strings get no exemption.
fn forbidden_lines(source: &str) -> Vec<usize> {
    // Whitespace is dropped so that spacing and line breaks cannot hide a
    // path, except where it separates two identifier characters. Each kept
    // character remembers its line.
    let mut compact: Vec<(char, usize)> = Vec::new();
    let mut gap = false;
    for (index, line) in source.lines().enumerate() {
        for c in line.chars() {
            if c.is_whitespace() {
                gap = true;
                continue;
            }
            let joins_words = compact.last().is_some_and(|(previous, _)| {
                is_identifier_char(*previous) && is_identifier_char(c)
            });
            if gap && joins_words {
                compact.push((' ', index + 1));
            }
            gap = false;
            compact.push((c, index + 1));
        }
        gap = true;
    }
    let text: String = compact.iter().map(|(c, _)| *c).collect();
    let line_at = |char_index: usize| compact[char_index].1;
    let chars: Vec<char> = text.chars().collect();
    let mut hits = Vec::new();

    let starts_with_at = |at: usize, pattern: &str| -> bool {
        let pattern: Vec<char> = pattern.chars().collect();
        chars
            .get(at..at + pattern.len())
            .is_some_and(|window| window == pattern)
    };
    let boundary_after = |end: usize| !chars.get(end).copied().is_some_and(is_identifier_char);
    let boundary_before = |at: usize| at == 0 || !is_identifier_char(chars[at - 1]);

    for at in 0..chars.len() {
        if !boundary_before(at) {
            continue;
        }
        for module in FORBIDDEN {
            let direct = format!("std::{module}");
            if starts_with_at(at, &direct) && boundary_after(at + direct.chars().count()) {
                hits.push(line_at(at));
            }
        }
        if starts_with_at(at, "std::{") {
            // Walk the group: the head of each top-level item is a module.
            let mut depth = 0usize;
            let mut item_start = at + "std::{".chars().count();
            let mut position = item_start;
            depth += 1;
            while position < chars.len() && depth > 0 {
                let c = chars[position];
                let at_item_end = (c == '}' || c == ',') && depth == 1;
                if at_item_end {
                    let head: String = chars[item_start..position]
                        .iter()
                        .take_while(|c| is_identifier_char(**c))
                        .collect();
                    if FORBIDDEN.contains(&head.as_str()) {
                        hits.push(line_at(item_start));
                    }
                    item_start = position + 1;
                }
                if c == '{' {
                    depth += 1;
                } else if c == '}' {
                    depth -= 1;
                }
                position += 1;
            }
        }
    }
    hits.sort_unstable();
    hits.dedup();
    hits
}

fn rust_sources(directory: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("source directory is readable") {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            rust_sources(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn the_crate_sources_name_neither_the_file_system_nor_the_process_module() {
    let mut files = Vec::new();
    rust_sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert!(files.len() >= 10, "found only {} sources", files.len());
    for file in files {
        let source = fs::read_to_string(&file).expect("source is UTF-8");
        assert_eq!(
            forbidden_lines(&source),
            Vec::<usize>::new(),
            "{} names a forbidden standard-library module",
            file.display()
        );
    }
}

#[test]
fn the_scanner_finds_every_spelling_of_a_forbidden_module() {
    for (source, line) in [
        ("use std::fs;\n", 1),
        ("\nuse std::process::Command;\n", 2),
        ("let a = std :: fs :: read(p);\n", 1),
        ("x\n/// Reads with std::fs::read.\ny\n", 2),
        ("// std::process is not used\n", 1),
        ("use std::{io, fs};\n", 1),
        ("use std::{\n    io,\n    process::Command,\n};\n", 3),
        ("use std::{collections::{HashMap, HashSet}, fs::File};\n", 1),
        ("let s = \"std::fs\";\n", 1),
        ("use ::std::fs::File;\n", 1),
        ("std::\nfs::read(p);\n", 1),
    ] {
        assert_eq!(forbidden_lines(source), [line], "{source:?}");
    }
}

#[test]
fn the_scanner_leaves_neighbouring_paths_alone() {
    for source in [
        "use std::collections::HashMap;\n",
        "use std::{io, fmt, collections::{fs_like, HashMap}};\n",
        "use std::fmt;\nlet offs = 1;\n",
        "use std::fsx::Thing;\n",
        "use std::processor::Thing;\n",
        "use mystd::fs;\n",
        "use crate::frontmatter::fs;\n",
        "let process = 1; let fs = 2;\n",
        "use std::{io, process_id};\n",
    ] {
        assert_eq!(forbidden_lines(source), Vec::<usize>::new(), "{source:?}");
    }
}
