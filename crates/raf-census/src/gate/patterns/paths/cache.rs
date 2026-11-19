// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! RAF ownership policy over native fallible ordered storage.

use super::super::{charge, failure};
use crate::Error;
use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::allocation::{Admission, Memory, OrderedTable, Resident, StorageError};
use raf_source::{Item, SourceFile, Workspace};

#[derive(Default)]
pub(super) struct OwnerCache<'w> {
    spans: Option<Vec<OwnerSpan<'w>>>,
    entries: OrderedTable<usize, Vec<&'w Item>>,
    // Includes spare reference cells in the native query vectors.
    retained: usize,
    // Exact physical admission covers the ordered slot arena only.
    bytes: usize,
    unit: usize,
    latest: Option<(usize, Vec<&'w Item>)>,
    latest_only: bool,
}

struct OwnerSpan<'w> {
    item: &'w Item,
    synthetic: bool,
    max_end: usize,
}

impl OwnerSpan<'_> {
    fn rank(&self) -> (usize, bool) {
        let span = &self.item.span;
        (span.end_byte - span.start_byte, self.synthetic)
    }

    // The source index provides start order, including the order of equal spans.
    // Each midpoint stores the latest end of its balanced subtree.
    fn index(spans: &mut [Self]) -> usize {
        let middle = spans.len() / 2;
        let (left, right) = spans.split_at_mut(middle);
        let Some((node, right)) = right.split_first_mut() else {
            return 0;
        };
        node.max_end = node
            .item
            .span
            .end_byte
            .max(Self::index(left))
            .max(Self::index(right));
        node.max_end
    }

    fn visit_at<'w>(
        spans: &[OwnerSpan<'w>],
        byte: usize,
        limits: &mut MatchLimits,
        visit: &mut impl FnMut(&OwnerSpan<'w>),
    ) -> Result<(), Error> {
        let (left, right) = spans.split_at(spans.len() / 2);
        let Some((node, right)) = right.split_first() else {
            return Ok(());
        };
        charge("paths", limits, 1)?;
        if node.max_end <= byte {
            return Ok(());
        }
        Self::visit_at(left, byte, limits, visit)?;
        if node.item.span.start_byte <= byte {
            if byte < node.item.span.end_byte {
                visit(node);
            }
            Self::visit_at(right, byte, limits, visit)?;
        }
        Ok(())
    }
}

// An AVL tree has height below twice the key count's bit length. The extra
// two levels cover empty/root operations without a special uncharged path.
fn height_bound(entries: usize) -> u64 {
    u64::from(entries.bit_width()) * 2 + 2
}

impl<'w> OwnerCache<'w> {
    pub(super) fn one_occurrence() -> Self {
        Self {
            latest_only: true,
            ..Self::default()
        }
    }

    pub(super) fn get(
        &mut self,
        workspace: &'w Workspace,
        file: &SourceFile,
        byte: usize,
        limits: &mut MatchLimits,
    ) -> Result<&[&'w Item], Error> {
        if self.latest_only {
            charge("paths", limits, 1)?;
            if self.latest.as_ref().is_none_or(|(known, _)| *known != byte) {
                self.latest = None;
                let owners = self.read(workspace, file, byte, limits)?;
                self.latest = Some((byte, owners));
            }
            return Ok(&self
                .latest
                .as_ref()
                .expect("INVARIANT: the current owner lookup stores an entry before returning")
                .1);
        }
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
        let owners = self.read(workspace, file, byte, limits)?;
        let retained = owners.capacity();
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

    fn read(
        &mut self,
        workspace: &'w Workspace,
        file: &SourceFile,
        byte: usize,
        limits: &mut MatchLimits,
    ) -> Result<Vec<&'w Item>, Error> {
        if self.spans.is_none() {
            let candidates = workspace.item_count_in(&file.path);
            self.admit(candidates, limits)?;
            let work = candidates.saturating_mul(
                file.path
                    .as_str()
                    .len()
                    .saturating_add(size_of::<OwnerSpan<'_>>() + 4),
            );
            charge("paths", limits, work.try_into().unwrap_or(u64::MAX))?;
            let mut spans = Vec::new();
            Memory::new(&mut Resident)
                .reserve(&mut spans, candidates)
                .map_err(|source| Error::PatternIndexStorage { source })?;
            self.admit(spans.capacity(), limits)?;
            spans.extend(workspace.items_in(&file.path).map(|item| OwnerSpan {
                item,
                synthetic: workspace.same_source(item).is_empty(),
                max_end: 0,
            }));
            OwnerSpan::index(&mut spans);
            self.spans = Some(spans);
        }
        let spans = self
            .spans
            .as_deref()
            .expect("INVARIANT: an owner lookup installs the file span index first");
        let mut rank = (usize::MAX, true);
        let mut count = 0;
        OwnerSpan::visit_at(spans, byte, limits, &mut |span| {
            let candidate = span.rank();
            if candidate < rank {
                rank = candidate;
                count = 0;
            }
            if candidate == rank {
                count += 1;
            }
        })?;
        self.admit(count, limits)?;
        let mut owners = Vec::new();
        Memory::new(&mut Resident)
            .reserve(&mut owners, count)
            .map_err(|source| Error::PatternIndexStorage { source })?;
        self.admit(owners.capacity(), limits)?;
        if count != 0 {
            OwnerSpan::visit_at(spans, byte, limits, &mut |span| {
                if span.rank() == rank {
                    owners.push(span.item);
                }
            })?;
        }
        Ok(owners)
    }

    fn admit(&self, references: usize, limits: &MatchLimits) -> Result<(), Error> {
        if limits
            .states
            .checked_sub(self.entries.len())
            .and_then(|remaining| remaining.checked_sub(self.retained))
            .and_then(|remaining| {
                remaining.checked_sub(self.spans.as_ref().map_or(0, Vec::capacity))
            })
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
