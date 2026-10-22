// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The file-system module of the standard library is named only by the disk
//! backend: `Tree` is the one seam through which the crate looks at a file.

use std::fs;
use std::path::{Path, PathBuf};

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The 1-based lines of `source` that name `std::fs` in any spelling:
/// `std::fs::read`, `std :: fs`, `use std::{io, fs}` or a mention in a comment
/// or a string. Comments and strings get no exemption.
fn fs_lines(source: &str) -> Vec<usize> {
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
    let chars: Vec<char> = compact.iter().map(|(c, _)| *c).collect();
    let starts_with_at = |at: usize, pattern: &str| -> bool {
        let pattern: Vec<char> = pattern.chars().collect();
        chars
            .get(at..at + pattern.len())
            .is_some_and(|window| window == pattern)
    };
    let boundary_before = |at: usize| at == 0 || !is_identifier_char(chars[at - 1]);
    let boundary_after = |end: usize| !chars.get(end).copied().is_some_and(is_identifier_char);

    let mut hits = Vec::new();
    for at in 0..chars.len() {
        if !boundary_before(at) {
            continue;
        }
        if starts_with_at(at, "std::fs") && boundary_after(at + "std::fs".len()) {
            hits.push(compact[at].1);
        }
        if starts_with_at(at, "std::{") {
            // Walk the group: the head of each top-level item is a module.
            let mut depth = 1usize;
            let mut item_start = at + "std::{".len();
            let mut position = item_start;
            while position < chars.len() && depth > 0 {
                let c = chars[position];
                if (c == '}' || c == ',') && depth == 1 {
                    let head: String = chars[item_start..position]
                        .iter()
                        .take_while(|c| is_identifier_char(**c))
                        .collect();
                    if head == "fs" {
                        hits.push(compact[item_start].1);
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

/// True for the disk backend's files: `tree/disk.rs` and everything below
/// `tree/disk/`.
fn is_disk_backend(relative: &Path) -> bool {
    relative == Path::new("tree/disk.rs") || relative.starts_with("tree/disk")
}

#[test]
fn only_the_disk_backend_names_the_file_system_module() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_sources(&source_root, &mut files);
    assert!(files.len() >= 8, "found only {} sources", files.len());

    let mut disk_files_using_fs = 0;
    for file in files {
        let relative = file.strip_prefix(&source_root).expect("below src");
        let source = fs::read_to_string(&file).expect("source is UTF-8");
        let hits = fs_lines(&source);
        if is_disk_backend(relative) {
            disk_files_using_fs += usize::from(!hits.is_empty());
        } else {
            assert_eq!(
                hits,
                Vec::<usize>::new(),
                "{} names the file-system module outside the disk backend",
                file.display()
            );
        }
    }
    assert!(disk_files_using_fs > 0, "the disk backend must use it");
}

#[test]
fn the_scanner_finds_every_spelling_of_the_file_system_module() {
    for (source, line) in [
        ("use std::fs;\n", 1),
        ("\nlet a = std :: fs :: read(p);\n", 2),
        ("x\n/// Reads with std::fs::read.\ny\n", 2),
        ("// std::fs is not used\n", 1),
        ("use std::{io, fs};\n", 1),
        ("use std::{\n    io,\n    fs::File,\n};\n", 3),
        ("use std::{collections::{HashMap, HashSet}, fs::File};\n", 1),
        ("let s = \"std::fs\";\n", 1),
        ("use ::std::fs::File;\n", 1),
        ("std::\nfs::read(p);\n", 1),
    ] {
        assert_eq!(fs_lines(source), [line], "{source:?}");
    }
}

#[test]
fn the_scanner_leaves_neighbouring_paths_alone() {
    for source in [
        "use std::collections::HashMap;\n",
        "use std::{io, fmt, collections::{fs_like, HashMap}};\n",
        "use std::fsx::Thing;\n",
        "use mystd::fs;\n",
        "use crate::tree::fs;\n",
        "let fs = 2;\n",
        "use std::{io, fs_id};\n",
    ] {
        assert_eq!(fs_lines(source), Vec::<usize>::new(), "{source:?}");
    }
}

#[test]
fn only_files_of_the_disk_backend_are_exempt() {
    for exempt in ["tree/disk.rs", "tree/disk/tests.rs"] {
        assert!(is_disk_backend(Path::new(exempt)), "{exempt}");
    }
    // `diskette.rs` starts with the text `disk` but is a sibling module.
    for covered in [
        "tree/mod.rs",
        "tree/memory.rs",
        "tree/diskette.rs",
        "lib.rs",
        "path.rs",
    ] {
        assert!(!is_disk_backend(Path::new(covered)), "{covered}");
    }
}
