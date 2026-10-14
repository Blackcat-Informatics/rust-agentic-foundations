// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Shared support for the repository-gate test binaries.
//!
//! This module is the single definition site for the list of files the gates
//! judge. The issue-reference and process-wording patterns are defined once, in
//! `raf_workplan::refs`, and [`banned_in_line`] calls them.
//!
//! The file list comes from the git index (`git ls-files -z`), so untracked
//! scratch files never count. Each path is read under the gate root, which is
//! `RAF_GATE_ROOT` when set and the package directory otherwise. The commit
//! hook points `RAF_GATE_ROOT` at a checkout of the index, so the gates judge
//! what the commit records.

use std::borrow::Cow;
use std::env;
use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Why a line is banned, or `None` when it is clean. The patterns live in
/// `raf_workplan::refs`, the one place that defines them.
#[allow(
    dead_code,
    reason = "the repository gates call it; the document-contract test judges the same patterns through the model"
)]
pub(crate) fn banned_in_line(line: &str) -> Option<&'static str> {
    raf_workplan::refs::banned_in_line(line).map(raf_workplan::refs::Banned::description)
}

/// The directory the gates read files from.
pub(crate) fn gate_root() -> PathBuf {
    env::var_os("RAF_GATE_ROOT").map_or_else(package_dir, PathBuf::from)
}

fn package_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every path in the git index of the package directory, relative and sorted.
///
/// # Panics
///
/// Panics when `git` cannot run or exits unsuccessfully.
pub(crate) fn tracked_paths() -> Vec<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(package_dir())
        .args(["ls-files", "-z"])
        .output()
        .unwrap_or_else(|e| panic!("cannot run `git ls-files -z`: {e}"));
    assert!(
        output.status.success(),
        "`git ls-files -z` failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut paths: Vec<PathBuf> = output
        .stdout
        .split(|b| *b == 0)
        .filter(|name| !name.is_empty())
        .map(|name| {
            PathBuf::from(
                std::str::from_utf8(name)
                    .unwrap_or_else(|e| panic!("tracked path is not UTF-8: {e}")),
            )
        })
        .collect();
    paths.sort();
    paths.dedup();
    paths
}

/// Compare what `symlink_metadata` saw before `open` with the opened handle.
///
/// The handle must be a regular file and, on Unix, the same device and inode,
/// so a file swapped in between the two steps is refused rather than read.
///
/// # Errors
///
/// Returns a description when the handle is not the file that was inspected.
pub(crate) fn check_handle(
    shown: &str,
    before: &fs::Metadata,
    on_handle: &fs::Metadata,
) -> Result<(), String> {
    if !on_handle.file_type().is_file() {
        return Err(format!("{shown} is not a regular file"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        if before.dev() != on_handle.dev() || before.ino() != on_handle.ino() {
            return Err(format!("{shown} changed while it was being opened"));
        }
    }
    #[cfg(not(unix))]
    let _ = before;
    Ok(())
}

/// The bytes of a regular file, read under the gate root.
///
/// The path is first inspected without following links, so a FIFO or a
/// device named like a document is refused before `open` can block on it, and
/// a symlink is refused rather than followed out of the root. The opened
/// handle is then checked against that inspection by [`check_handle`].
///
/// # Errors
///
/// Returns a description when the path is missing, is a symlink, is not a
/// regular file, changed while it was opened, or cannot be read.
pub(crate) fn read_regular(root: &Path, path: &Path) -> Result<Vec<u8>, String> {
    let full = root.join(path);
    let shown = full.display().to_string();
    let before = fs::symlink_metadata(&full).map_err(|e| format!("cannot stat {shown}: {e}"))?;
    if before.file_type().is_symlink() {
        return Err(format!(
            "{shown} is a symlink; symlinks are not allowed in the tree"
        ));
    }
    if !before.file_type().is_file() {
        return Err(format!("{shown} is not a regular file"));
    }
    let mut file = fs::File::open(&full).map_err(|e| format!("cannot open {shown}: {e}"))?;
    let on_handle = file
        .metadata()
        .map_err(|e| format!("cannot stat the open handle of {shown}: {e}"))?;
    check_handle(&shown, &before, &on_handle)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|e| format!("cannot read {shown}: {e}"))?;
    Ok(bytes)
}

/// The bytes of a tracked path, read under the gate root.
///
/// # Panics
///
/// Panics when the file is missing, is not a regular file, or is unreadable:
/// a path the index records but the gate root cannot supply is a failure,
/// never a skip.
pub(crate) fn read_tracked(root: &Path, path: &Path) -> Vec<u8> {
    read_regular(root, path).unwrap_or_else(|e| {
        panic!(
            "cannot read tracked file {} under {}: {e}",
            path.display(),
            root.display()
        )
    })
}

/// The text of a file, or `None` when the bytes are binary.
///
/// Only a NUL byte marks a file as binary, which matches `git grep -I`. Any
/// other file is scanned: invalid UTF-8 (a Latin-1 file, say) is decoded
/// lossily so a token in it is still found rather than skipped.
pub(crate) fn decode_text(bytes: &[u8]) -> Option<Cow<'_, str>> {
    if bytes.contains(&0) {
        return None;
    }
    Some(String::from_utf8_lossy(bytes))
}
