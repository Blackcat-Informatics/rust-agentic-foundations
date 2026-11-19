// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Occupied storage across simultaneously active fields.

use std::{collections::BTreeMap, ops::Range};

// The largest data section occupies fewer than 2^22 bits. Placing pointer
// indices above it keeps layouts in one ordered space without aliasing.
pub(super) const POINTER_BASE: u32 = 1 << 22;

#[derive(Default)]
pub(super) struct Occupancy(BTreeMap<u32, u32>);

impl Occupancy {
    pub(super) fn insert(&mut self, span: Range<u32>, exclusive: bool) -> Result<(), Overlap> {
        let Range { mut start, mut end } = span;
        if let Some((&left, &right)) = self.0.range(..=start).next_back()
            && right >= start
        {
            if !exclusive && right > start {
                return Err(Overlap);
            }
            start = left;
            end = end.max(right);
            self.0.remove(&left);
        }
        while let Some((&left, &right)) = self.0.range(start..).next() {
            if left > end {
                break;
            }
            if !exclusive && left < end {
                return Err(Overlap);
            }
            end = end.max(right);
            self.0.remove(&left);
        }
        self.0.insert(start, end);
        Ok(())
    }

    pub(super) fn merge(&mut self, other: Self, exclusive: bool) -> Result<(), Overlap> {
        for (start, end) in other.0 {
            self.insert(start..end, exclusive)?;
        }
        Ok(())
    }
}

pub(super) struct Overlap;
