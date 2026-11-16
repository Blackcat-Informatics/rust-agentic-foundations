// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The artifact inputs of evidence records: files cited by repository-relative
//! path and the digest of their bytes.
//!
//! The load reads each cited file once, through the verified handle of the
//! tree, and keeps its digest or the reason it has none. A file outside the
//! workspace root, a missing file, a file that is not regular and a file over
//! [`ARTIFACT_LIMIT`] have no digest. Nothing reads a revision or Git, and a
//! file under an ignored directory is read as any other: whether it is
//! tracked is not judged.

use std::collections::BTreeMap;

use raf_okf::{EvidenceRecord, InputKind};
use raf_source::{RelPath, Tree};

use crate::Loaded;
use crate::digest::Digest;
use crate::obligation::InputMoved;

/// The most bytes an artifact may hold; a larger file is refused, never read in part.
pub(crate) const ARTIFACT_LIMIT: u64 = 256 * 1024 * 1024;

/// The digest of each cited artifact now, or why it has none, by path.
#[derive(Debug, Default)]
pub(crate) struct Artifacts(BTreeMap<String, Result<Digest, String>>);

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

    /// Whether the artifact `path` still digests as `recorded`.
    pub(crate) fn moved(&self, path: &str, recorded: &str) -> Result<(), InputMoved> {
        match self.0.get(path) {
            Some(Ok(digest)) if digest.to_string() == recorded => Ok(()),
            Some(Ok(_)) => Err(InputMoved::Moved),
            Some(Err(reason)) => Err(InputMoved::Unresolved(reason.clone())),
            // The load read every artifact a loaded record cites.
            None => Err(InputMoved::Unresolved(
                "the load did not read this artifact".to_string(),
            )),
        }
    }
}

fn read(tree: &dyn Tree, path: &str) -> Result<Digest, String> {
    let relative = RelPath::new(path).map_err(|error| error.to_string())?;
    let bytes = tree
        .read_bytes(&relative, ARTIFACT_LIMIT)
        .map_err(|error| error.to_string())?;
    Ok(Digest::artifact(path, &bytes))
}
