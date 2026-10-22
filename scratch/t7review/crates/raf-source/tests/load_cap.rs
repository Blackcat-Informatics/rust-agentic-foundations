// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The byte cap on a file, judged through `Workspace::load` rather than a bare
//! read: a module file of exactly the default cap loads, one byte more is
//! refused naming the file and the limit, and a cap lowered in the options
//! applies to a file the module walk reaches. The files are padded with
//! comments, so they are valid Rust that costs the parser almost nothing.

use raf_source::{
    DEFAULT_MAX_FILE_BYTES, LoadError, LoadErrorKind, LoadOptions, MemoryTree, Tree, TreeError,
    Workspace,
};

const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

/// `head` followed by comment lines, `len` bytes in all.
fn padded_to(head: &str, len: usize) -> String {
    let mut text = String::with_capacity(len);
    text.push_str(head);
    while text.len() + 80 < len {
        text.push_str("//");
        text.push_str(&"x".repeat(77));
        text.push('\n');
    }
    // The rest is one comment line of the exact remaining length, without
    // its final newline when only one byte is left.
    let rest = len - text.len();
    match rest {
        0 => {}
        1 => text.push('\n'),
        _ => {
            text.push_str("//");
            text.push_str(&"y".repeat(rest - 3));
            text.push('\n');
        }
    }
    assert_eq!(text.len(), len);
    text
}

fn cap() -> usize {
    usize::try_from(DEFAULT_MAX_FILE_BYTES).expect("the cap fits")
}

fn load(files: &[(&str, &str)], options: &LoadOptions) -> Result<Workspace, LoadError> {
    let mut all = vec![("Cargo.toml", DEMO)];
    all.extend_from_slice(files);
    let tree: MemoryTree = all.into_iter().collect();
    Workspace::load_with(&tree, &["."], options)
}

fn too_large(error: &LoadError) -> Option<u64> {
    match &error.kind {
        LoadErrorKind::Tree(inner) => match **inner {
            TreeError::TooLarge { limit, .. } => Some(limit),
            _ => None,
        },
        _ => None,
    }
}

#[test]
fn a_library_root_of_exactly_the_cap_loads_and_one_byte_more_is_refused_naming_it() {
    let at = padded_to("pub fn at_cap() {}\n", cap());
    let ws = load(&[("src/lib.rs", &at)], &LoadOptions::default()).expect("at the cap");
    assert!(ws.resolve("demo::at_cap").is_ok());

    let over = padded_to("pub fn over_cap() {}\n", cap() + 1);
    let error = load(&[("src/lib.rs", &over)], &LoadOptions::default()).expect_err("one over");
    assert_eq!(too_large(&error), Some(DEFAULT_MAX_FILE_BYTES), "{error}");
    assert_eq!(error.file.as_str(), "src/lib.rs");
    assert!(error.to_string().contains("src/lib.rs"), "{error}");
}

#[test]
fn a_module_file_of_exactly_the_cap_loads_and_one_byte_more_is_refused_naming_it() {
    let lib = "pub mod child;\n";
    let at = padded_to("pub fn at_cap() {}\n", cap());
    let ws = load(
        &[("src/lib.rs", lib), ("src/child.rs", &at)],
        &LoadOptions::default(),
    )
    .expect("at the cap");
    assert!(ws.resolve("demo::child::at_cap").is_ok());

    let over = padded_to("pub fn over_cap() {}\n", cap() + 1);
    let error = load(
        &[("src/lib.rs", lib), ("src/child.rs", &over)],
        &LoadOptions::default(),
    )
    .expect_err("one over");
    assert_eq!(too_large(&error), Some(DEFAULT_MAX_FILE_BYTES), "{error}");
    assert_eq!(
        error.file.as_str(),
        "src/lib.rs",
        "the declaring file: {error}"
    );
    assert_eq!(error.line, Some(1), "{error}");
    assert!(error.to_string().contains("src/child.rs"), "{error}");
}

#[test]
fn a_lowered_cap_applies_to_a_child_file_and_spares_a_root_within_it() {
    let lib = "pub mod child;\n";
    let child = padded_to("pub fn small() {}\n", 1000);
    let mut options = LoadOptions::default();
    options.max_file_bytes = 1000;
    let ws = load(&[("src/lib.rs", lib), ("src/child.rs", &child)], &options)
        .expect("the child is exactly the lowered cap");
    assert!(ws.resolve("demo::child::small").is_ok());

    let child = padded_to("pub fn small() {}\n", 1001);
    let error = load(&[("src/lib.rs", lib), ("src/child.rs", &child)], &options)
        .expect_err("one over the lowered cap");
    assert_eq!(too_large(&error), Some(1000), "{error}");
    assert_eq!(
        error.file.as_str(),
        "src/lib.rs",
        "the declaring file: {error}"
    );
    assert_eq!(error.line, Some(1), "{error}");
    assert!(error.to_string().contains("src/child.rs"), "{error}");
}

#[test]
fn a_cap_larger_than_the_default_admits_a_file_the_default_refuses() {
    let big = padded_to("pub fn big() {}\n", cap() + 1);
    let mut options = LoadOptions::default();
    options.max_file_bytes = DEFAULT_MAX_FILE_BYTES + 1;
    let ws = load(&[("src/lib.rs", &big)], &options).expect("within the raised cap");
    assert!(ws.resolve("demo::big").is_ok());
}

#[test]
fn the_memory_tree_used_here_reads_the_padded_file_at_its_exact_size() {
    let text = padded_to("", cap());
    let tree: MemoryTree = [("a.rs", text.as_str())].into_iter().collect();
    let read = tree
        .read(
            &raf_source::RelPath::new("a.rs").expect("path"),
            DEFAULT_MAX_FILE_BYTES,
        )
        .expect("reads");
    assert_eq!(read.len(), cap());
}
