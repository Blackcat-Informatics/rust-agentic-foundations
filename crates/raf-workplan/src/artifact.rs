// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The artifact inputs of evidence records: files cited by repository-relative
//! path and the digest of their bytes.
//!
//! The load reads each cited file once, through the verified handle of the
//! tree, and keeps its digest or the reason it has none. A missing file, a file
//! that is not regular, a symbolic-link loop, a file outside the workspace root
//! and a file over [`ARTIFACT_LIMIT`] have no digest, and the input is stale.
//! Any other failure (an I/O error, a tree that cannot return bytes, a refused
//! source authority) is a refusal: no currency was established, and reporting
//! it as a stale input would send the author to record the evidence again,
//! which cannot clear it. Nothing reads a revision or Git, and a file under an
//! ignored directory is read as any other: whether it is tracked is not judged.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use raf_okf::{EvidenceRecord, InputKind};
use raf_source::{RelPath, Tree, TreeError};

use crate::Loaded;
use crate::digest::Digest;
use crate::obligation::InputMoved;

/// The most bytes an artifact may hold; a larger file is refused, never read in part.
pub(crate) const ARTIFACT_LIMIT: u64 = 256 * 1024 * 1024;

/// What the load found at the path of a cited artifact.
#[derive(Debug)]
enum Found {
    /// The digest of the bytes of the file.
    Digest(Digest),
    /// No file whose bytes could be digested is there; why.
    Absent(String),
    /// The tree refused or failed the read; why.
    Refused(String),
}

/// The tree refused or failed to read a cited artifact, so its currency is unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArtifactRefusal {
    /// The repository-relative path of the artifact.
    pub(crate) path: String,
    /// What the tree said.
    pub(crate) reason: String,
}

impl fmt::Display for ArtifactRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the artifact `{}` could not be read: {}",
            self.path, self.reason
        )
    }
}

impl Error for ArtifactRefusal {}

/// The digest of each cited artifact now, or why it has none, by path.
#[derive(Debug, Default)]
pub(crate) struct Artifacts(BTreeMap<String, Found>);

impl Artifacts {
    /// Read each artifact the `evidence` records cite, once.
    pub(crate) fn load(tree: &dyn Tree, evidence: &[Loaded<EvidenceRecord>]) -> Self {
        let mut now = BTreeMap::new();
        for input in evidence
            .iter()
            .flat_map(|record| &record.value().inputs)
            .filter(|input| input.kind == InputKind::Artifact)
        {
            now.entry(input.name.clone())
                .or_insert_with(|| read(tree, &input.name));
        }
        Self(now)
    }

    /// Whether the artifact `path` still digests as `recorded`: `None` when it
    /// does, and how it moved when it does not.
    ///
    /// # Errors
    ///
    /// [`ArtifactRefusal`] when the tree refused or failed the read.
    pub(crate) fn moved(
        &self,
        path: &str,
        recorded: &str,
    ) -> Result<Option<InputMoved>, ArtifactRefusal> {
        let refused = |reason: String| ArtifactRefusal {
            path: path.to_string(),
            reason,
        };
        match self.0.get(path) {
            Some(Found::Digest(digest)) if digest.to_string() == recorded => Ok(None),
            Some(Found::Digest(_)) => Ok(Some(InputMoved::Moved)),
            Some(Found::Absent(reason)) => Ok(Some(InputMoved::Unresolved(reason.clone()))),
            Some(Found::Refused(reason)) => Err(refused(reason.clone())),
            // The load read every artifact a loaded record cites.
            None => Err(refused("the load did not read this artifact".to_string())),
        }
    }
}

fn read(tree: &dyn Tree, path: &str) -> Found {
    let relative = match RelPath::new(path) {
        Ok(relative) => relative,
        Err(error) => return Found::Absent(error.to_string()),
    };
    match tree.read_bytes(&relative, ARTIFACT_LIMIT) {
        Ok(bytes) => Found::Digest(Digest::artifact(path, &bytes)),
        Err(
            error @ (TreeError::NotFound { .. }
            | TreeError::NotRegular { .. }
            | TreeError::SymlinkLoop { .. }
            | TreeError::OutsideRoot { .. }
            | TreeError::TooLarge { .. }),
        ) => Found::Absent(error.to_string()),
        Err(error) => Found::Refused(error.to_string()),
    }
}
