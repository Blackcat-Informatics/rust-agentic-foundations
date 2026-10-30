// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Admission of actual Cargo-reported documentation entries.

use super::{DiagramError, Library, Selection, absolute, refused, required, strings, text};
use purrdf_core::ContentDigest;
use purrdf_lex::{
    allocation::{Memory, OrderedTable, Resident},
    json::Value,
};
use raf_source::{DiskTree, EntryKind, FileId, OpenedBytes, RelPath, Tree};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

const MAX_HTML_BYTES: u64 = 16 * 1024 * 1024;

#[cfg(test)]
mod tests;

/// An actual selected-library HTML entry and its observed content identity.
///
/// The native disk reader verifies containment and regular-file reads. Receipt
/// identity comes from Cargo's message, not a reconstructed crate-name path.
/// The bytes are an observation after Cargo; effective compiler configuration,
/// HTML provenance, atomic subprocess inputs and embedded links need their own
/// evidence before a host can claim complete documentation admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryArtifact {
    library: Library,
    reported_path: PathBuf,
    relative_path: RelPath,
    entry_directory: RelPath,
    sha256: ContentDigest,
    file_id: FileId,
    fresh: bool,
}

impl LibraryArtifact {
    /// Exact metadata identity matched by the HTML-bearing Cargo message.
    #[must_use]
    pub fn library(&self) -> &Library {
        &self.library
    }

    /// Absolute filename reported by Cargo itself.
    #[must_use]
    pub fn reported_path(&self) -> &Path {
        &self.reported_path
    }

    /// Admitted filename relative to Cargo's actual target directory.
    #[must_use]
    pub fn relative_path(&self) -> &RelPath {
        &self.relative_path
    }

    /// Actual entry-page directory, the base for that page's relative links.
    #[must_use]
    pub fn entry_directory(&self) -> &RelPath {
        &self.entry_directory
    }

    /// Native unframed SHA-256 of the bounded bytes read through one handle.
    #[must_use]
    pub const fn sha256(&self) -> ContentDigest {
        self.sha256
    }

    /// Physical identity returned by the same native handle as the admitted bytes.
    #[must_use]
    pub fn file_id(&self) -> &FileId {
        &self.file_id
    }

    /// Whether Cargo reports reused cached output (`true`) or a new build (`false`).
    /// This is Cargo's observation, not an authenticated compiler-input receipt.
    #[must_use]
    pub const fn fresh(&self) -> bool {
        self.fresh
    }
}

pub(super) fn admit(
    selection: &Selection,
    messages: &[Value],
) -> Result<Vec<LibraryArtifact>, DiagramError> {
    let disk = DiskTree::new(&selection.target_directory).map_err(DiagramError::data)?;
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let mut selected = OrderedTable::default();
    for (index, library) in selection.libraries.iter().enumerate() {
        if selected.get(&library.package_id.as_str()).is_some() {
            return Err(refused("selected Cargo package identities must be unique"));
        }
        selected
            .insert(library.package_id.as_str(), index, &mut memory)
            .map_err(DiagramError::data)?;
    }
    let mut found = vec![None; selection.libraries.len()];
    let mut identities: OrderedTable<FileId, ()> = OrderedTable::default();
    for message in messages {
        if message.get("reason").and_then(Value::as_str) != Some("compiler-artifact") {
            continue;
        }
        let Some(&index) = selected.get(&text(message, "package_id")?) else {
            continue;
        };
        let Some(reported) = html_entry(message)? else {
            continue;
        };
        let library = &selection.libraries[index];
        if found[index].is_some() {
            return Err(refused(&format!(
                "duplicate HTML receipt for `{}`",
                library.package
            )));
        }
        matching_target(library, message)?;
        let fresh = required(message, "fresh")?
            .as_bool()
            .ok_or_else(|| refused("Cargo artifact `fresh` must be a Boolean"))?;
        let artifact = read_entry(
            &disk,
            selection,
            library,
            reported,
            &mut identities,
            &mut memory,
            fresh,
        )?;
        found[index] = Some(artifact);
    }
    found
        .into_iter()
        .zip(&selection.libraries)
        .map(|(artifact, library)| {
            artifact.ok_or_else(|| {
                refused(&format!(
                    "Cargo reported no HTML entry for selected library `{}`",
                    library.package
                ))
            })
        })
        .collect()
}

fn html_entry(message: &Value) -> Result<Option<PathBuf>, DiagramError> {
    let filenames = strings(required(message, "filenames")?)?;
    let mut html = filenames
        .iter()
        .filter(|name| Path::new(name).extension() == Some(OsStr::new("html")));
    let Some(filename) = html.next() else {
        return Ok(None);
    };
    if html.next().is_some() {
        return Err(refused(
            "one library receipt must identify exactly one HTML entry",
        ));
    }
    let path = PathBuf::from(filename);
    if !path.is_absolute() || filename.chars().any(char::is_control) {
        return Err(refused(
            "reported HTML entry must be an absolute path without controls",
        ));
    }
    Ok(Some(path))
}

fn matching_target(library: &Library, message: &Value) -> Result<(), DiagramError> {
    let target = required(message, "target")?;
    if text(target, "name")? != library.name
        || strings(required(target, "kind")?)? != library.kind
        || strings(required(target, "crate_types")?)? != library.crate_types
        || target.get("doc") != Some(&Value::Bool(true))
        || absolute(message, "manifest_path")? != library.manifest
        || absolute(target, "src_path")? != library.source
        || strings(required(message, "features")?)? != library.features
    {
        return Err(refused(&format!(
            "HTML receipt disagrees with selected Cargo library `{}`",
            library.package
        )));
    }
    Ok(())
}

fn read_entry(
    disk: &DiskTree,
    selection: &Selection,
    library: &Library,
    reported_path: PathBuf,
    identities: &mut OrderedTable<FileId, ()>,
    memory: &mut Memory<'_, Resident>,
    fresh: bool,
) -> Result<LibraryArtifact, DiagramError> {
    let relative = reported_path
        .strip_prefix(&selection.target_directory)
        .map_err(|_| refused("reported HTML entry is outside Cargo's target directory"))?;
    let relative_path = RelPath::new(
        relative
            .to_str()
            .ok_or_else(|| refused("reported HTML entry must be UTF-8"))?,
    )
    .map_err(DiagramError::data)?;
    if disk.entry(&relative_path).map_err(DiagramError::data)? != Some(EntryKind::File) {
        return Err(refused(
            "reported HTML entry must name an existing regular file",
        ));
    }
    let opened = disk
        .read_unseen_bytes(&relative_path, MAX_HTML_BYTES, &|id| {
            identities.get(id).is_some()
        })
        .map_err(DiagramError::data)?;
    let OpenedBytes::Read { id, bytes } = opened else {
        return Err(refused(
            "selected libraries must not share one HTML output file",
        ));
    };
    identities
        .insert(id.clone(), (), memory)
        .map_err(DiagramError::data)?;
    let entry_directory = relative_path
        .parent()
        .ok_or_else(|| refused("reported HTML entry has no relative link directory"))?;
    Ok(LibraryArtifact {
        library: library.clone(),
        reported_path,
        relative_path,
        entry_directory,
        sha256: ContentDigest::of(&bytes),
        file_id: id,
        fresh,
    })
}
