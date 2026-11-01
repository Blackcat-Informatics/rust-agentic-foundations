// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Same-handle observations of bytes actually read; no atomic-input assertion.

use super::{DiagramError, MAX_BYTES, MAX_ITEMS, refused};
use purrdf_core::ContentDigest;
use raf_source::{DiskTree, Entry, EntryKind, FileId, Opened, RelPath, Tree, TreeError};
use std::collections::BTreeMap;
use std::sync::Mutex;

pub(super) struct Observed {
    disk: DiskTree,
    reads: Mutex<BTreeMap<RelPath, (FileId, ContentDigest, usize)>>,
}

impl Observed {
    pub(super) fn new(root: &std::path::Path) -> Result<Self, DiagramError> {
        Ok(Self {
            disk: DiskTree::new(root).map_err(DiagramError::data)?,
            reads: Mutex::default(),
        })
    }

    fn remember(&self, path: &RelPath, id: &FileId, text: &str) -> Result<(), TreeError> {
        let failure = |message| TreeError::Io {
            path: path.to_string(),
            source: std::io::Error::other(message),
        };
        let mut reads = self
            .reads
            .lock()
            .map_err(|_| failure("assembly source observation lock is poisoned"))?;
        let digest = ContentDigest::of(text.as_bytes());
        if let Some((previous, hash, _)) = reads.get(path) {
            if previous != id || *hash != digest {
                return Err(failure("source changed between assembly reads"));
            }
        } else {
            let bytes = reads
                .values()
                .try_fold(text.len(), |sum, (_, _, length)| sum.checked_add(*length));
            if reads.len() == MAX_ITEMS || bytes.is_none_or(|bytes| bytes > MAX_BYTES) {
                return Err(failure(
                    "assembly source observations exceed their item/byte bound",
                ));
            }
            reads.insert(path.clone(), (id.clone(), digest, text.len()));
        }
        Ok(())
    }

    pub(super) fn verify(&self) -> Result<Vec<(String, String)>, DiagramError> {
        let reads = self
            .reads
            .lock()
            .map_err(|_| refused("assembly source observation lock is poisoned"))?;
        let mut result = Vec::new();
        for (path, (expected_id, expected_digest, _)) in reads.iter() {
            let Opened::Read { id, text } = self
                .disk
                .read_unseen(path, MAX_BYTES as u64, &|_| false)
                .map_err(DiagramError::data)?
            else {
                return Err(refused("required assembly source was not read"));
            };
            if id != *expected_id || ContentDigest::of(text.as_bytes()) != *expected_digest {
                return Err(refused(
                    "source observation changed during checked include generation",
                ));
            }
            result.push((path.to_string(), expected_digest.to_string()));
        }
        Ok(result)
    }
}

impl Tree for Observed {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        match self.read_unseen(path, max_bytes, &|_| false)? {
            Opened::Read { text, .. } => Ok(text),
            Opened::Seen(_) => Err(TreeError::Io {
                path: path.to_string(),
                source: std::io::Error::other("assembly source read was omitted"),
            }),
            _ => Err(TreeError::Io {
                path: path.to_string(),
                source: std::io::Error::other(
                    "unsupported native source observation; update the assembly adapter for this Tree result",
                ),
            }),
        }
    }
    fn read_unseen(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        let opened = self.disk.read_unseen(path, max_bytes, seen)?;
        if let Opened::Read { id, text } = &opened {
            self.remember(path, id, text)?;
        }
        Ok(opened)
    }
    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        self.disk.list(path)
    }
    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        self.disk.entry(path)
    }
}
