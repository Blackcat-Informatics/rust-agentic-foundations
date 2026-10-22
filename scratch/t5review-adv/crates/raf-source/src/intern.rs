// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One copy of each string the records of a load share.

use std::collections::HashSet;

use crate::cost::{hash_entry, shared_text};
use crate::item::SharedStr;

/// What the first copy of a string costs beyond its bytes: its entry in the
/// table and the header of the shared allocation.
pub(crate) const INTERN_ENTRY_BYTES: usize = hash_entry::<SharedStr>() + shared_text(0);

/// A table of shared strings.
#[derive(Debug, Default)]
pub(crate) struct Interner {
    strings: HashSet<SharedStr>,
}

impl Interner {
    /// The shared copy of `text`, and what it cost: its length and
    /// [`INTERN_ENTRY_BYTES`] the first time, nothing after.
    pub(crate) fn intern(&mut self, text: &str) -> (SharedStr, usize) {
        if let Some(known) = self.strings.get(text) {
            return (known.clone(), 0);
        }
        let shared = SharedStr::from(text);
        self.strings.insert(shared.clone());
        (shared, text.len() + INTERN_ENTRY_BYTES)
    }
}
