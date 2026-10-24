// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Provider error context around native ordered allocation.

use purrdf_lex::allocation::{Memory, OrderedTable, Resident, StorageError};

use crate::Error;

pub(super) fn error(job: &str, source: StorageError) -> Error {
    Error::ProviderStorage {
        job: job.to_owned(),
        source,
    }
}

pub(super) fn insert<K: Ord, V>(
    table: &mut OrderedTable<K, V>,
    key: K,
    value: V,
    job: &str,
) -> Result<Option<V>, Error> {
    // Callers admit their logical state before insertion. Matching has no
    // separate byte grant: Resident is the arena's fallible physical producer,
    // not Source's load admission. Resume this same arena's actual live bytes.
    let bytes = table.storage_bytes().map_err(|source| error(job, source))?;
    let mut resident = Resident;
    let mut memory = Memory::resume(&mut resident, bytes);
    table
        .insert(key, value, &mut memory)
        .map_err(|source| error(job, source))
}
