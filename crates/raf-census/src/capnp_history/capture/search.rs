// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Signed compiler import directories and lookup in a pinned tree.

use std::collections::{BTreeMap, BTreeSet};

use raf_rdf::{Graph, Node};
use raf_source::{EntryKind, RelPath, Tree};

use crate::{ReportError, report::shape};

use super::{CaptureError, canonical_path, fields, join, mismatch, owned_node};

const MAX_DIRECTORIES: usize = 16;
const CAPTURE_FIELDS: [&str; 7] = [
    "compilerSha256",
    "compilerVersion",
    "descriptorFile",
    "descriptorSha256",
    "schemaRoot",
    "schemaSource",
    "importDirectory",
];

pub(super) enum ImportSearch {
    SchemaRoot,
    Ordered(Vec<RelPath>),
}

impl ImportSearch {
    pub(super) fn read(graph: &Graph, root: &Node<'_>) -> Result<Self, CaptureError> {
        match shape::class(root)? {
            "CapnpCaptureV1" => {
                fields(root, "CapnpCaptureV1", &CAPTURE_FIELDS[..6])?;
                Ok(Self::SchemaRoot)
            }
            "CapnpCaptureV2" => {
                fields(root, "CapnpCaptureV2", &CAPTURE_FIELDS)?;
                ordered(graph, root).map(Self::Ordered)
            }
            found => Err(ReportError::Kind {
                expected: "CapnpCaptureV1 or CapnpCaptureV2".into(),
                found: found.into(),
            }
            .into()),
        }
    }

    pub(super) fn directories(&self) -> &[RelPath] {
        match self {
            Self::SchemaRoot => &[],
            Self::Ordered(directories) => directories,
        }
    }

    pub(super) fn verify_directories(
        &self,
        tree: &dyn Tree,
        root: &RelPath,
    ) -> Result<(), CaptureError> {
        for directory in self.directories() {
            let path = join(root, directory.as_str())?;
            if tree.entry(&path).map_err(CaptureError::Read)? != Some(EntryKind::Dir) {
                return Err(mismatch(format!(
                    "capture import directory {path} must be a committed directory"
                )));
            }
        }
        Ok(())
    }
}

fn ordered(graph: &Graph, root: &Node<'_>) -> Result<Vec<RelPath>, CaptureError> {
    let mut positions = BTreeMap::new();
    let mut directories = BTreeSet::new();
    for reference in root.values(&shape::property("importDirectory")) {
        if positions.len() >= MAX_DIRECTORIES {
            return Err(mismatch("capture admits at most 16 import directories"));
        }
        let row = owned_node(graph, reference, "importDirectory")?;
        fields(&row, "CapnpImportDirectory", &["path", "position"])?;
        let path = canonical_path(&shape::string(&row, "path")?, true)?;
        let position = shape::count(&row, "position")?;
        if !directories.insert(path.clone()) || positions.insert(position, path).is_some() {
            return Err(mismatch("capture repeats an import directory or position"));
        }
    }
    if positions.is_empty() || positions.keys().copied().ne(0..positions.len()) {
        return Err(mismatch(
            "capture import directories require contiguous positions starting at zero",
        ));
    }
    Ok(positions.into_values().collect())
}

pub(super) fn first_match(
    tree: &dyn Tree,
    root: &RelPath,
    directories: &[RelPath],
    suffix: &str,
) -> Result<Option<RelPath>, CaptureError> {
    let suffix = canonical_path(suffix, false)?;
    for directory in directories {
        let candidate = join(directory, suffix.as_str())?;
        let path = join(root, candidate.as_str())?;
        match tree.entry(&path).map_err(CaptureError::Read)? {
            None => {}
            Some(EntryKind::File) => return Ok(Some(candidate)),
            Some(_) => {
                return Err(mismatch(format!(
                    "capture import candidate {path} must be a committed regular file"
                )));
            }
        }
    }
    Ok(None)
}
