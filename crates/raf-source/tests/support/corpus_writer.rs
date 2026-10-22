// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Shared shape corpus writer and its safe temporary-directory check.

use crate::support::{self, shapes};
use std::fs;
use std::path::Path;

/// Write the shape table to an explicit maintenance destination.
#[allow(
    dead_code,
    reason = "called by the corpus_generate example, not ordinary integration tests"
)]
pub(crate) fn generate_corpus(dir: &Path) {
    write_shapes(dir, &shapes::all());
}

fn write_shapes(dir: &Path, shapes: &[shapes::Shape]) {
    fs::create_dir_all(dir).expect("create the corpus directory");
    for shape in shapes {
        fs::write(dir.join(format!("{}.rs", shape.name)), &shape.text)
            .expect("write a corpus file");
    }
}

/// Exercise the actual writer in an owned directory without rewriting the corpus.
pub(crate) fn assert_corpus_writer() {
    let scratch = support::Scratch::new();
    let shapes = shapes::all();
    assert!(!shapes.is_empty(), "the writer proof needs actual shapes");
    write_shapes(scratch.base(), &shapes);
    let mut actual: Vec<_> = fs::read_dir(scratch.base())
        .expect("written corpus")
        .map(|entry| entry.expect("entry").file_name())
        .collect();
    actual.sort();
    let mut expected: Vec<std::ffi::OsString> = shapes
        .iter()
        .map(|shape| format!("{}.rs", shape.name).into())
        .collect();
    expected.sort();
    assert_eq!(actual, expected);
    for shape in shapes {
        assert_eq!(
            fs::read(scratch.base().join(format!("{}.rs", shape.name))).expect("written shape"),
            shape.text.as_bytes(),
            "{}",
            shape.name
        );
    }
}
