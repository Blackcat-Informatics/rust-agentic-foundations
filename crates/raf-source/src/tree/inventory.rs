// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Common path traversal over sealed immutable inventories.

use super::EntryKind;
use crate::meter::{ItemAdmission, Meter};
use crate::{LoadError, LoadErrorKind, RelPath};
use purrdf_lex::allocation::Memory;
use purrdf_lex::walk::WorkList;

pub(super) fn locate(
    path: &RelPath,
    meter: &Meter,
    mut record: impl FnMut(&str) -> Result<Option<EntryKind>, LoadError>,
) -> Result<Option<(EntryKind, String)>, LoadError> {
    let mut kind = EntryKind::Dir;
    let mut canonical = String::new();
    let mut parents = WorkList::<_, 16>::new();
    let mut admission = ItemAdmission::new(meter, path);
    let mut memory = Memory::new(&mut admission);
    let refused = |message: &str| {
        LoadError::new(
            path,
            None,
            LoadErrorKind::Locked {
                field: "source-path",
                message: message.to_owned(),
                source: None,
            },
        )
    };
    let native = |source| LoadError::new(path, None, LoadErrorKind::Storage(source));
    let result = (|| {
        for segment in path.segments() {
            meter.work(segment.len().saturating_add(1), path, None)?;
            if kind != EntryKind::Dir {
                return Err(refused(
                    "sealed source path traverses a non-directory or symlink",
                ));
            }
            if segment == ".." {
                canonical.truncate(
                    parents
                        .pop()
                        .ok_or_else(|| refused("sealed source path leaves its source root"))?,
                );
                kind = EntryKind::Dir;
                continue;
            }
            meter.directory(parents.len(), path)?;
            parents
                .try_push_admitted(canonical.len(), &mut memory)
                .map_err(native)?;
            if !canonical.is_empty() {
                memory.push_str(&mut canonical, "/").map_err(native)?;
            }
            memory.push_str(&mut canonical, segment).map_err(native)?;
            let Some(found) = record(&canonical)? else {
                return Ok(None);
            };
            kind = found;
        }
        Ok(Some((kind, canonical)))
    })();
    result.map_err(|error| match error.kind {
        LoadErrorKind::Storage(source) => admission.error(source),
        _ => error,
    })
}
