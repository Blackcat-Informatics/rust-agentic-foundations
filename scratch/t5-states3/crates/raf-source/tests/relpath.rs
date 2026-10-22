// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A relative path is normalised on the way in and never leaves the root.

use raf_source::{PathError, RelPath};

fn text(input: &str) -> String {
    RelPath::new(input).expect("accepted").as_str().to_owned()
}

#[test]
fn dot_and_empty_segments_are_dropped() {
    assert_eq!(text("a/./b"), "a/b");
    assert_eq!(text("./a"), "a");
    assert_eq!(text("a//b/"), "a/b");
    assert_eq!(text(""), "");
    assert!(RelPath::new("").expect("root").is_root());
    assert!(RelPath::new(".").expect("root").is_root());
}

#[test]
fn a_parent_segment_below_the_root_is_kept_for_the_disk_to_resolve() {
    assert_eq!(text("a/../b"), "a/../b");
    assert_eq!(text("a/b/../.."), "a/b/../..");
    assert!(!RelPath::new("a/..").expect("accepted").is_root());
}

#[test]
fn a_parent_segment_above_the_root_is_refused_beside_the_one_that_stays_inside() {
    assert!(matches!(
        RelPath::new("../a"),
        Err(PathError::AboveRoot { .. })
    ));
    assert!(matches!(
        RelPath::new("a/../.."),
        Err(PathError::AboveRoot { .. })
    ));
    assert!(matches!(
        RelPath::new("a/../../b"),
        Err(PathError::AboveRoot { .. })
    ));
    assert!(RelPath::new("a/../b").is_ok());
}

#[test]
fn an_absolute_input_is_refused_beside_its_relative_form() {
    let refused = RelPath::new("/a").expect_err("absolute");
    assert!(matches!(refused, PathError::Absolute { .. }));
    assert_eq!(refused.input(), "/a");
    assert_eq!(text("a"), "a");
}

#[test]
fn backslash_nul_and_drive_letter_inputs_are_refused_beside_plain_names() {
    assert!(matches!(
        RelPath::new("a\\b"),
        Err(PathError::Backslash { .. })
    ));
    assert!(matches!(RelPath::new("a\0b"), Err(PathError::Nul { .. })));
    assert!(matches!(
        RelPath::new("C:x"),
        Err(PathError::DriveLetter { .. })
    ));
    assert!(matches!(
        RelPath::new("a/c:x"),
        Err(PathError::DriveLetter { .. })
    ));
    assert_eq!(text("cc/xy:z"), "cc/xy:z");
}

#[test]
fn join_extends_a_path_and_checks_the_climb_against_the_whole() {
    let base = RelPath::new("a/b").expect("path");
    assert_eq!(base.join("c/./d").expect("joined").as_str(), "a/b/c/d");
    assert_eq!(base.join("../c").expect("joined").as_str(), "a/b/../c");
    assert!(base.join("../..").is_ok());
    assert!(matches!(
        base.join("../../.."),
        Err(PathError::AboveRoot { .. })
    ));
    assert!(matches!(base.join("/c"), Err(PathError::Absolute { .. })));
    // A climb already spent by an earlier `..` cannot be spent again.
    let spent = RelPath::new("a/..").expect("path");
    assert!(matches!(spent.join(".."), Err(PathError::AboveRoot { .. })));
}

#[test]
fn parent_and_file_name_are_lexical() {
    let path = RelPath::new("a/b/c.rs").expect("path");
    assert_eq!(path.file_name(), Some("c.rs"));
    assert_eq!(path.parent().expect("parent").as_str(), "a/b");
    let single = RelPath::new("a").expect("path");
    assert!(single.parent().expect("parent").is_root());
    assert_eq!(RelPath::root().file_name(), None);
    assert_eq!(RelPath::root().parent(), None);
    // The parent keeps its own climb budget: `a/..` has nothing left to climb.
    let parent = RelPath::new("a/../b")
        .expect("path")
        .parent()
        .expect("parent");
    assert_eq!(parent.as_str(), "a/..");
    assert!(matches!(
        parent.join(".."),
        Err(PathError::AboveRoot { .. })
    ));
    assert!(parent.join("c").is_ok());
}

#[test]
fn display_names_the_root_as_a_dot() {
    assert_eq!(RelPath::root().to_string(), ".");
    assert_eq!(RelPath::new("a/b").expect("path").to_string(), "a/b");
}
