// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! RAF ownership policy over native fallible ordered storage.

use super::super::{charge, failure};
use crate::Error;
use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::allocation::{Admission, Memory, OrderedTable, StorageError};
use raf_source::{Item, SourceFile, Workspace};

#[derive(Default)]
pub(super) struct OwnerCache<'w> {
    entries: OrderedTable<usize, Vec<&'w Item>>,
    // Includes spare reference cells in the native query vectors.
    retained: usize,
    // Exact physical admission covers the ordered slot arena only.
    bytes: usize,
    unit: usize,
}

// An AVL tree has height below twice the key count's bit length. The extra
// two levels cover empty/root operations without a special uncharged path.
fn height_bound(entries: usize) -> u64 {
    u64::from(entries.bit_width()) * 2 + 2
}

impl<'w> OwnerCache<'w> {
    pub(super) fn get(
        &mut self,
        workspace: &'w Workspace,
        file: &SourceFile,
        byte: usize,
        limits: &mut MatchLimits,
    ) -> Result<&[&'w Item], Error> {
        charge("paths", limits, height_bound(self.entries.len()))?;
        if self.entries.get(&byte).is_none() {
            self.load(workspace, file, byte, limits)?;
        }
        charge("paths", limits, height_bound(self.entries.len()))?;
        Ok(self.entries.get(&byte).expect("cached owner key"))
    }

    fn load(
        &mut self,
        workspace: &'w Workspace,
        file: &SourceFile,
        byte: usize,
        limits: &mut MatchLimits,
    ) -> Result<(), Error> {
        let candidates = workspace.item_count_in(&file.path);
        let work = candidates.saturating_mul(file.path.as_str().len().saturating_add(4));
        charge("paths", limits, work.try_into().unwrap_or(u64::MAX))?;
        self.admit(candidates, limits)?;
        let mut owners = workspace.item_at(&file.path, byte);
        let mut real = 0;
        for index in 0..owners.len() {
            if !workspace.same_source(owners[index]).is_empty() {
                owners.swap(real, index);
                real += 1;
            }
        }
        if real != 0 {
            owners.truncate(real);
        }
        let retained = owners.capacity();
        self.admit(retained, limits)?;
        // Precharge the native search and AVL parent/height/link edits, with
        // 32 primitive operations per possible level. Array growth is charged
        // separately by the native producer's exact layout callback.
        charge(
            "paths",
            limits,
            height_bound(self.entries.len().saturating_add(1)) * 32,
        )?;
        let mut admission = IndexAdmission {
            limits,
            live: self.bytes,
            unit: self.unit,
            failure: None,
        };
        let (result, bytes) = {
            let mut memory = Memory::resume(&mut admission, self.bytes);
            let result = self.entries.insert(byte, owners, &mut memory);
            (result, memory.admitted_bytes())
        };
        self.bytes = bytes;
        self.unit = admission.unit;
        let previous = result.map_err(|source| {
            admission
                .failure
                .unwrap_or(Error::PatternIndexStorage { source })
        })?;
        if previous.is_some() {
            return Err(failure(
                "paths",
                "ownership index unexpectedly replaced a cached byte",
            ));
        }
        self.retained = self.retained.saturating_add(retained);
        Ok(())
    }

    fn admit(&self, references: usize, limits: &MatchLimits) -> Result<(), Error> {
        if limits
            .states
            .checked_sub(self.entries.len())
            .and_then(|remaining| remaining.checked_sub(self.retained))
            .and_then(|remaining| remaining.checked_sub(references))
            .and_then(|remaining| remaining.checked_sub(1))
            .is_none()
        {
            return Err(failure(
                "paths",
                "matching owner-cache state limit exhausted",
            ));
        }
        Ok(())
    }
}

struct IndexAdmission<'a> {
    limits: &'a mut MatchLimits,
    live: usize,
    unit: usize,
    failure: Option<Error>,
}

impl Admission for IndexAdmission<'_> {
    fn resize(&mut self, live: usize) -> Result<(), StorageError> {
        if live > self.live {
            let next = live - self.live;
            if self.unit == 0 {
                self.unit = next;
            }
            // The native arena doubles on growth. Three originally admitted
            // initial layouts per state cover old plus replacement arrays.
            let maximum = self
                .unit
                .saturating_mul(self.limits.states)
                .saturating_mul(3);
            if live > maximum {
                self.failure = Some(failure(
                    "paths",
                    "matching owner-index state limit exhausted",
                ));
                return Err(StorageError::AdmissionFailed);
            }
            if let Err(error) = charge("paths", self.limits, next.try_into().unwrap_or(u64::MAX)) {
                self.failure = Some(error);
                return Err(StorageError::AdmissionFailed);
            }
        }
        self.live = live;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{IndexAdmission, MatchLimits, Memory, StorageError};

    #[test]
    fn native_storage_failures_keep_their_typed_cause() {
        let mut resident = purrdf_lex::allocation::Resident;
        let mut memory = Memory::new(&mut resident);
        let source = memory
            .reserve(&mut Vec::<u8>::new(), usize::MAX)
            .unwrap_err();
        let error = crate::Error::PatternIndexStorage { source };
        assert_eq!(
            std::error::Error::source(&error)
                .unwrap()
                .downcast_ref::<StorageError>(),
            Some(&StorageError::SizeOverflow)
        );
        assert!(error.to_string().contains("ownership index"));
        let mut valid = Vec::<u8>::new();
        memory.reserve(&mut valid, 1).unwrap();
        assert_eq!(valid.capacity(), 1);
    }

    #[test]
    fn index_growth_retains_the_original_grant_and_old_storage_on_refusal() {
        let mut limits = MatchLimits {
            steps: 10_000,
            states: 1,
        };
        let mut admission = IndexAdmission {
            limits: &mut limits,
            live: 0,
            unit: 0,
            failure: None,
        };
        let mut values = Vec::<u8>::new();
        let bytes = {
            let mut memory = Memory::new(&mut admission);
            memory.reserve(&mut values, 64).unwrap();
            values.push(7);
            memory.reserve(&mut values, 128).unwrap();
            assert_eq!(
                memory.reserve(&mut values, 256),
                Err(StorageError::AdmissionFailed)
            );
            assert_eq!(values, [7]);
            assert_eq!(values.capacity(), 128);
            memory.admitted_bytes()
        };
        assert_eq!(bytes, 128);
        assert!(
            admission
                .failure
                .take()
                .unwrap()
                .to_string()
                .contains("state limit")
        );
        admission.limits.states = 2;
        let mut memory = Memory::resume(&mut admission, bytes);
        memory.reserve(&mut values, 256).unwrap();
        assert_eq!(memory.admitted_bytes(), 256);
        assert_eq!(values, [7]);
    }
}
