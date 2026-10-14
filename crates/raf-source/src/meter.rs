// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! What one load spends against its work and memory budgets.

use std::cell::Cell;

use crate::budget::Budget;
use crate::error::{LoadError, LoadErrorKind};
use crate::options::LoadOptions;
use crate::path::RelPath;
use crate::prescan::Scan;

/// What a load spent against its budgets, as [`Workspace::usage`] reports it.
///
/// [`Workspace::usage`]: crate::Workspace::usage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Usage {
    /// Bytes read: every source file and manifest ([`Budget::TotalBytes`]).
    pub total_bytes: usize,
    /// Bytes charged to the index ([`Budget::ItemBytes`]).
    pub item_bytes: usize,
    /// Bytes charged to the facts of the files ([`Budget::FactBytes`]).
    pub fact_bytes: usize,
    /// The most tokens of one file ([`Budget::FileTokens`]).
    pub max_file_tokens: usize,
    /// The highest value the recursion counter of the pre-scan reached in one
    /// file ([`Budget::Recursion`]).
    pub max_recursion: usize,
    /// The deepest directory that discovery or a `workspace.members` pattern
    /// entered, in levels below its root ([`Budget::DirectoryDepth`]).
    pub max_directory_depth: usize,
}

/// The running totals of one load, checked against its limits.
pub(crate) struct Meter {
    options: LoadOptions,
    total_bytes: Cell<usize>,
    item_bytes: Cell<usize>,
    fact_bytes: Cell<usize>,
    max_file_tokens: Cell<usize>,
    max_recursion: Cell<usize>,
    max_directory_depth: Cell<usize>,
}

/// A refusal by one budget, naming the file and the line.
pub(crate) fn over_budget(
    budget: Budget,
    limit: usize,
    file: &RelPath,
    line: Option<usize>,
) -> LoadError {
    LoadError::new(file, line, LoadErrorKind::Budget { budget, limit })
}

/// Add `amount` to `total`, refusing a sum above `limit`.
fn add(total: &Cell<usize>, amount: usize, limit: usize) -> Result<(), ()> {
    let sum = total.get().saturating_add(amount);
    total.set(sum);
    if sum > limit { Err(()) } else { Ok(()) }
}

impl Meter {
    pub(crate) fn new(options: &LoadOptions) -> Self {
        Self {
            options: *options,
            total_bytes: Cell::new(0),
            item_bytes: Cell::new(0),
            fact_bytes: Cell::new(0),
            max_file_tokens: Cell::new(0),
            max_recursion: Cell::new(0),
            max_directory_depth: Cell::new(0),
        }
    }

    pub(crate) fn options(&self) -> &LoadOptions {
        &self.options
    }

    /// Charge the text of `file`, just read, to the bytes of the load.
    pub(crate) fn read(&self, file: &RelPath, text: &str) -> Result<(), LoadError> {
        let limit = self.options.max_total_bytes;
        add(&self.total_bytes, text.len(), limit)
            .map_err(|()| over_budget(Budget::TotalBytes, limit, file, None))
    }

    /// Charge `bytes` of index records built for `file` at `line`.
    ///
    /// Every charge is an upper bound of the allocations it covers, derived
    /// from the layout of the containers that hold them (see `crate::cost`),
    /// never an estimate measured on typical input, so the budget bounds what
    /// the index and the walk allocate whatever the shape of the input.
    pub(crate) fn items(
        &self,
        bytes: usize,
        file: &RelPath,
        line: Option<usize>,
    ) -> Result<(), LoadError> {
        let limit = self.options.max_item_bytes;
        add(&self.item_bytes, bytes, limit)
            .map_err(|()| over_budget(Budget::ItemBytes, limit, file, line))
    }

    /// What the facts of the next file may still take: the bytes charged so
    /// far and the limit.
    pub(crate) fn fact_allowance(&self) -> FactAllowance {
        FactAllowance {
            spent: self.fact_bytes.get(),
            limit: self.options.max_fact_bytes,
        }
    }

    /// Record the facts of a file, charged on its parse thread.
    pub(crate) fn facts(&self, bytes: usize) {
        self.fact_bytes
            .set(self.fact_bytes.get().saturating_add(bytes));
    }

    /// Record what the pre-scan of a parsed file counted.
    pub(crate) fn scanned(&self, scan: Scan) {
        self.max_file_tokens
            .set(self.max_file_tokens.get().max(scan.tokens));
        self.max_recursion
            .set(self.max_recursion.get().max(scan.recursion));
    }

    /// Check a directory `depth` levels below its root and record it.
    pub(crate) fn directory(&self, depth: usize, dir: &RelPath) -> Result<(), LoadError> {
        if depth > Budget::DirectoryDepth.limit() {
            return Err(over_budget(
                Budget::DirectoryDepth,
                Budget::DirectoryDepth.limit(),
                dir,
                None,
            ));
        }
        self.max_directory_depth
            .set(self.max_directory_depth.get().max(depth));
        Ok(())
    }

    pub(crate) fn usage(&self) -> Usage {
        Usage {
            total_bytes: self.total_bytes.get(),
            item_bytes: self.item_bytes.get(),
            fact_bytes: self.fact_bytes.get(),
            max_file_tokens: self.max_file_tokens.get(),
            max_recursion: self.max_recursion.get(),
            max_directory_depth: self.max_directory_depth.get(),
        }
    }
}

/// The fact bytes one file may take, handed to its parse thread.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FactAllowance {
    /// Bytes the earlier files of the load took.
    pub(crate) spent: usize,
    /// The limit of the load.
    pub(crate) limit: usize,
}
