// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Registry `.crate` archives and a Cargo archive cache laid out on disk.

use std::path::Path;

use purrdf_gts::files::{FileEntry, pack_entries_v2};

/// The `.crate` bytes (a gzip-compressed tar) of `name` at `version`, holding
/// `files` under `name-version/`.
pub fn crate_archive(name: &str, version: &str, files: &[(&str, &str)]) -> Vec<u8> {
    let entries: Vec<_> = files
        .iter()
        .map(|(path, text)| FileEntry {
            path: format!("{name}-{version}/{path}"),
            data: Some(text.as_bytes().to_vec()),
            ..FileEntry::default()
        })
        .collect();
    let encoded = pack_entries_v2(&entries).expect("the fixture files pack");
    let graph = purrdf_gts::reader::read(&encoded, false, None);
    let tar = purrdf_gts::tar::to_tar_vec(&graph, &purrdf_gts::tar::ToTarOptions::default())
        .expect("the fixture tar is written");
    purrdf_deflate::gzip::compress(&tar, purrdf_deflate::Level::DEFAULT)
}

/// The lock file checksum of an archive.
pub fn checksum(archive: &[u8]) -> String {
    purrdf_core::ContentDigest::of(archive).to_string()
}

/// Stores `archive` where Cargo keeps it: `cache/INDEX/NAME-VERSION.crate`.
pub fn place(cache: &Path, index: &str, name: &str, version: &str, archive: &[u8]) {
    let directory = cache.join(index);
    std::fs::create_dir_all(&directory).expect("the index directory is created");
    std::fs::write(directory.join(format!("{name}-{version}.crate")), archive)
        .expect("the archive is stored");
}
