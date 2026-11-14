// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Fixture files written to disk.

use std::path::Path;

/// Writes `files` below `dir`, creating each file and its directories in
/// the order given.
pub fn write<'a>(dir: &Path, files: impl IntoIterator<Item = &'a (&'a str, &'a str)>) {
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().expect("a fixture file has a parent"))
            .expect("the fixture directory is created");
        std::fs::write(&path, text).expect("the fixture file is written");
    }
}
