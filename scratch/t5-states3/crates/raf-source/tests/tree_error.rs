// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Every error names its path, and only `NotFound` means absence.

use std::error::Error;
use std::io;

use raf_source::{PathError, TreeError};

fn every_variant(path: &str) -> Vec<TreeError> {
    vec![
        TreeError::NotFound { path: path.into() },
        TreeError::OutsideRoot { path: path.into() },
        TreeError::NotRegular { path: path.into() },
        TreeError::SymlinkLoop { path: path.into() },
        TreeError::TooLarge {
            path: path.into(),
            limit: 7,
        },
        TreeError::NotUtf8 { path: path.into() },
        TreeError::NonUtf8Name {
            path: path.into(),
            name: "bad\u{fffd}.rs".into(),
        },
        TreeError::DuplicatePath { path: path.into() },
        TreeError::Io {
            path: path.into(),
            source: io::Error::other("disk on fire"),
        },
    ]
}

#[test]
fn every_variant_names_its_path_in_display_and_in_path() {
    for error in every_variant("src/lib.rs") {
        assert_eq!(error.path(), "src/lib.rs", "{error:?}");
        assert!(error.to_string().contains("src/lib.rs"), "{error:?}");
    }
}

#[test]
fn only_io_has_a_source_and_only_not_found_is_absence() {
    for error in every_variant("p") {
        let is_io = matches!(error, TreeError::Io { .. });
        assert_eq!(error.source().is_some(), is_io, "{error:?}");
    }
    assert!(
        TreeError::Io {
            path: "p".into(),
            source: io::Error::other("disk on fire")
        }
        .to_string()
        .contains("disk on fire")
    );
}

#[test]
fn an_unconstructible_path_converts_to_outside_root_naming_the_input() {
    let error: TreeError = PathError::AboveRoot {
        input: "../a".into(),
    }
    .into();
    assert!(matches!(error, TreeError::OutsideRoot { .. }));
    assert_eq!(error.path(), "../a");
}
