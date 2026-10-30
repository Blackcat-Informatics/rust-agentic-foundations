// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Immutable documentation directories beside cumulative host output.

use super::{DiagramBundle, DiagramError, documentation};
use purrdf_core::ContentDigest;
use raf_rdf::{Builder, Format, TermValue, Triple};
use raf_source::{DiskTree, EntryKind, RelPath, Tree};
use std::{
    fs,
    path::{Path, PathBuf},
};

const MANIFEST: &str = "manifest.ttl";
const VOCAB: &str = "https://blackcatinformatics.ca/raf/workplan/documentation#";

/// Publish a complete, immutable documentation bundle beside existing host output.
///
/// The existing root may contain arbitrary Cargo/Rustdoc or static-site files.
/// Only `raf-workplan/<digest>/` is created. Its name is native content SHA-256
/// over a canonical RDF 1.2 ownership manifest containing the format version,
/// artifact paths and exact content digests. Changed presentations receive a
/// new directory, preserving cumulative output and versioned explorer links.
///
/// An existing directory must have the exact intended manifest and no foreign
/// entries. Present artifacts must match exactly; absent owned artifacts are
/// filled only after validating every existing entry. The shared native sync
/// staging writer publishes each complete file. A destination is returned only
/// after verifying the complete inventory and all bytes.
///
/// This uses the sync writer's by-name publication semantics: path checks do
/// not prevent concurrent swaps between inspection and rename. It does not
/// provide an atomic directory transaction or concurrent-publisher isolation.
///
/// # Errors
/// Refuses symlinks, changed owned files, unexpected entries, invalid ownership
/// and native filesystem/RDF failures. An interrupted clean owned directory can
/// be completed. An orphan temporary file or a directory lacking its manifest
/// instead fails with its path for inspection; unknown files are never removed.
pub fn publish(bundle: &DiagramBundle, root: &Path) -> Result<PathBuf, DiagramError> {
    let mut files = documentation::build(bundle)?;
    let manifest = manifest(&files)?;
    let digest = ContentDigest::of(manifest.as_bytes());
    files.push((relative(MANIFEST)?, manifest));
    let root = fs::canonicalize(root).map_err(DiagramError::data)?;
    let tree = DiskTree::new(&root).map_err(DiagramError::data)?;
    let namespace = relative("raf-workplan")?;
    directory(&tree, &root, &namespace)?;
    let destination = relative(&format!("raf-workplan/{digest}"))?;
    let created = directory(&tree, &root, &destination)?;
    if created {
        // Ownership intent is itself complete before any payload is published.
        // A crash before this point leaves an unowned directory and refuses.
        let (path, text) = files.last().ok_or_else(|| {
            refused("ownership manifest is absent from the assembled inventory".to_owned())
        })?;
        let target = root.join(destination.as_str()).join(path.as_str());
        let staged = crate::sync::stage(&target, text).map_err(DiagramError::data)?;
        if let Err(error) = fs::rename(&staged, &target) {
            return Err(publication_failure(&error, &[staged]));
        }
    }
    validate(&tree, &destination, &files, false)?;
    let mut missing = Vec::new();
    for (path, text) in &files {
        let full = relative(&format!("{destination}/{path}"))?;
        if tree.entry(&full).map_err(DiagramError::data)?.is_none() {
            missing.push((full, text));
        }
    }
    let mut staged = Vec::new();
    for (full, text) in missing {
        let target = root.join(full.as_str());
        match crate::sync::stage(&target, text) {
            Ok(temporary) => staged.push((temporary, target, full)),
            Err(error) => {
                let remaining: Vec<_> = staged.iter().map(|(path, _, _)| path.clone()).collect();
                return Err(publication_failure(&error, &remaining));
            }
        }
    }
    // Detect every late destination collision before publishing any payload.
    let admission = staged
        .iter()
        .try_for_each(|(_, _, full)| match tree.entry(full) {
            Ok(None) => Ok(()),
            Ok(Some(_)) => Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("documentation destination {full} appeared during staging"),
            )),
            Err(error) => Err(std::io::Error::other(error)),
        });
    if let Err(error) = admission {
        let remaining: Vec<_> = staged.iter().map(|(path, _, _)| path.clone()).collect();
        return Err(publication_failure(&error, &remaining));
    }
    for (index, (temporary, target, _)) in staged.iter().enumerate() {
        if let Err(error) = fs::rename(temporary, target) {
            let remaining: Vec<_> = staged[index..]
                .iter()
                .map(|(path, _, _)| path.clone())
                .collect();
            return Err(publication_failure(&error, &remaining));
        }
    }
    validate(&tree, &destination, &files, true)?;
    Ok(root.join(destination.as_str()))
}

fn publication_failure(error: &std::io::Error, temporaries: &[PathBuf]) -> DiagramError {
    let mut reason = format!("documentation publication failed: {error}");
    for path in temporaries {
        if let Err(cleanup) = fs::remove_file(path) {
            use std::fmt::Write as _;
            write!(
                reason,
                "; cannot remove owned temporary {}: {cleanup}",
                path.display()
            )
            .expect("writing to String cannot fail");
        }
    }
    refused(reason)
}

fn directory(tree: &DiskTree, root: &Path, path: &RelPath) -> Result<bool, DiagramError> {
    match tree.entry(path).map_err(DiagramError::data)? {
        Some(EntryKind::Dir) => Ok(false),
        None => {
            fs::create_dir(root.join(path.as_str())).map_err(DiagramError::data)?;
            if tree
                .capture_directory_bounded(path, 0)
                .map_err(DiagramError::data)?
                .entries
                .is_empty()
            {
                Ok(true)
            } else {
                Err(refused(format!(
                    "new documentation directory {path} is not empty"
                )))
            }
        }
        _ => Err(refused(format!(
            "documentation directory {path} is a link or non-directory"
        ))),
    }
}

fn validate(
    tree: &DiskTree,
    destination: &RelPath,
    files: &[(RelPath, String)],
    complete: bool,
) -> Result<(), DiagramError> {
    let entries = tree
        .capture_directory_bounded(destination, files.len())
        .map_err(DiagramError::data)?;
    for entry in &entries.entries {
        if entry.kind != EntryKind::File
            || !files.iter().any(|(path, _)| path.as_str() == entry.name)
        {
            return Err(refused(format!(
                "unexpected documentation entry {destination}/{}; inspect it before retrying",
                entry.name
            )));
        }
    }
    for (path, text) in files {
        let full = relative(&format!("{destination}/{path}"))?;
        match tree.entry(&full).map_err(DiagramError::data)? {
            None if !complete && path.as_str() != MANIFEST => {}
            Some(EntryKind::File) => {
                let actual = tree
                    .capture_file(&full, text.len() as u64)
                    .map_err(DiagramError::data)?;
                if actual.bytes != text.as_bytes() {
                    return Err(refused(format!(
                        "documentation file {full} changed; inspect it before retrying"
                    )));
                }
            }
            _ => {
                return Err(refused(format!(
                    "required owned documentation file {full} is absent or not regular; inspect its ownership before retrying"
                )));
            }
        }
    }
    Ok(())
}

fn manifest(files: &[(RelPath, String)]) -> Result<String, DiagramError> {
    let mut builder = Builder::new();
    for (subject, key, value) in
        std::iter::once((String::new(), "schema", "raf-documentation-1".to_owned())).chain(
            files.iter().enumerate().flat_map(|(index, (path, text))| {
                let subject = format!("#file-{index}");
                [
                    (subject.clone(), "path", path.as_str().to_owned()),
                    (
                        subject,
                        "sha256",
                        ContentDigest::of(text.as_bytes()).to_hex(),
                    ),
                ]
            }),
        )
    {
        builder
            .insert(Triple {
                subject: TermValue::Iri(subject),
                predicate: TermValue::Iri(format!("{VOCAB}{key}")),
                object: TermValue::typed_literal(value, "http://www.w3.org/2001/XMLSchema#string"),
            })
            .map_err(DiagramError::data)?;
    }
    builder
        .finish()
        .map_err(DiagramError::data)?
        .to_text(Format::Turtle)
        .map_err(DiagramError::data)
}

fn relative(path: &str) -> Result<RelPath, DiagramError> {
    RelPath::new(path).map_err(DiagramError::data)
}

fn refused(reason: String) -> DiagramError {
    DiagramError::Refused { reason }
}
