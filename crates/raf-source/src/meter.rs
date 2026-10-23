// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! What one load spends against its work and memory budgets.

use std::cell::Cell;

use purrdf_lex::allocation::{Admission, StorageError};

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
    /// The most bytes charged to the glob scope tables at once
    /// ([`Budget::GlobBytes`]): what the tables of one settle and the changes
    /// waiting to be passed on held at their peak, as charged.
    pub glob_bytes: usize,
    /// The most tokens of one file ([`Budget::FileTokens`]).
    pub max_file_tokens: usize,
    /// The highest value the recursion counter of the pre-scan reached in one
    /// file ([`Budget::Recursion`]).
    pub max_recursion: usize,
    /// The deepest directory that discovery or a `workspace.members` pattern
    /// entered, in levels below its root ([`Budget::DirectoryDepth`]).
    pub max_directory_depth: usize,
    /// The steps of glob table work the load made ([`Budget::Work`]), over
    /// every settle of its tables: zero for a load without glob imports.
    pub work: usize,
}

/// The running totals of one load, checked against its limits.
pub(crate) struct Meter {
    options: LoadOptions,
    total_bytes: Cell<usize>,
    item_bytes: Cell<usize>,
    fact_bytes: Cell<usize>,
    glob_bytes: Cell<usize>,
    glob_peak: Cell<usize>,
    max_file_tokens: Cell<usize>,
    max_recursion: Cell<usize>,
    max_directory_depth: Cell<usize>,
    work: Cell<usize>,
}

/// Admit native physical storage to the original cumulative item-byte grant.
pub(crate) struct ItemAdmission<'m> {
    meter: &'m Meter,
    file: &'m RelPath,
    live: usize,
    failure: Option<LoadError>,
}

impl<'m> ItemAdmission<'m> {
    pub(crate) const fn new(meter: &'m Meter, file: &'m RelPath) -> Self {
        Self::resume(meter, file, 0)
    }

    /// Continue an account whose native payloads are still retained.
    pub(crate) const fn resume(meter: &'m Meter, file: &'m RelPath, live: usize) -> Self {
        Self {
            meter,
            file,
            live,
            failure: None,
        }
    }

    pub(crate) fn error(&mut self, error: StorageError) -> LoadError {
        self.failure
            .take()
            .unwrap_or_else(|| LoadError::new(self.file, None, LoadErrorKind::Storage(error)))
    }
}

impl Admission for ItemAdmission<'_> {
    fn resize(&mut self, live: usize) -> Result<(), StorageError> {
        if self.failure.is_some() {
            return Err(StorageError::AdmissionFailed);
        }
        if live > self.live
            && let Err(error) = self.meter.items(live - self.live, self.file, None)
        {
            self.failure = Some(error);
            return Err(StorageError::AdmissionFailed);
        }
        // ItemBytes is cumulative: freeing storage never refunds this grant.
        self.live = live;
        Ok(())
    }
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
    /// A scoped parser callback spends the same original grant on its own thread.
    pub(crate) fn resume(options: &LoadOptions, usage: Usage) -> Self {
        let meter = Self::new(options);
        meter.total_bytes.set(usage.total_bytes);
        meter.item_bytes.set(usage.item_bytes);
        meter.fact_bytes.set(usage.fact_bytes);
        meter.glob_peak.set(usage.glob_bytes);
        meter.max_file_tokens.set(usage.max_file_tokens);
        meter.max_recursion.set(usage.max_recursion);
        meter.max_directory_depth.set(usage.max_directory_depth);
        meter.work.set(usage.work);
        meter
    }

    /// Commit only actual configuration scratch/work spent by a completed callback.
    pub(crate) fn configuration_spent(
        &self,
        before: Usage,
        after: Usage,
        file: &RelPath,
    ) -> Result<(), LoadError> {
        let backwards = || {
            LoadError::new(
                file,
                None,
                LoadErrorKind::Internal {
                    what: "configuration callback usage moved backwards".to_owned(),
                },
            )
        };
        self.items(
            after
                .item_bytes
                .checked_sub(before.item_bytes)
                .ok_or_else(backwards)?,
            file,
            None,
        )?;
        self.work(
            after.work.checked_sub(before.work).ok_or_else(backwards)?,
            file,
            None,
        )
    }
    pub(crate) fn new(options: &LoadOptions) -> Self {
        Self {
            options: options.clone(),
            total_bytes: Cell::new(0),
            item_bytes: Cell::new(0),
            fact_bytes: Cell::new(0),
            glob_bytes: Cell::new(0),
            glob_peak: Cell::new(0),
            max_file_tokens: Cell::new(0),
            max_recursion: Cell::new(0),
            max_directory_depth: Cell::new(0),
            work: Cell::new(0),
        }
    }

    pub(crate) fn options(&self) -> &LoadOptions {
        &self.options
    }

    /// Charge the text of `file`, just read, to the bytes of the load.
    pub(crate) fn read(&self, file: &RelPath, text: &str) -> Result<(), LoadError> {
        self.read_bytes(file, text.len())
    }

    /// Charge binary input before a bounded decoder or hash processes it.
    pub(crate) fn read_bytes(&self, file: &RelPath, bytes: usize) -> Result<(), LoadError> {
        let limit = self.options.max_total_bytes;
        add(&self.total_bytes, bytes, limit)
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

    /// Charge `bytes` of glob scope tables built for the glob import at
    /// `line` of `file`.
    ///
    /// Like [`Meter::items`], every charge is an upper bound of the
    /// allocations it covers, derived from the layout of the containers that
    /// hold them (see `crate::cost`). Unlike the other byte budgets, this one
    /// bounds what is held at once: what is freed is released
    /// ([`Meter::release_globs`]), and the budget refuses the charge that
    /// would make what is held pass the limit.
    pub(crate) fn globs(
        &self,
        bytes: usize,
        file: &RelPath,
        line: Option<usize>,
    ) -> Result<(), LoadError> {
        let limit = self.options.max_glob_bytes;
        let held = add(&self.glob_bytes, bytes, limit);
        self.glob_peak
            .set(self.glob_peak.get().max(self.glob_bytes.get()));
        held.map_err(|()| over_budget(Budget::GlobBytes, limit, file, line))
    }

    /// Charge `units` steps of glob table work made for the glob import at
    /// `line` of `file`, before they are made.
    ///
    /// Unlike the glob bytes, the work is never released: the budget bounds
    /// every step one load makes, and refuses the charge that makes the count
    /// pass the limit.
    pub(crate) fn work(
        &self,
        units: usize,
        file: &RelPath,
        line: Option<usize>,
    ) -> Result<(), LoadError> {
        let limit = self.options.max_work;
        #[cfg(test)]
        charges::record(units);
        add(&self.work, units, limit).map_err(|()| over_budget(Budget::Work, limit, file, line))
    }

    /// Release `bytes` of glob scope tables charged before and now freed.
    pub(crate) fn release_globs(&self, bytes: usize) {
        let held = self
            .glob_bytes
            .get()
            .checked_sub(bytes)
            .expect("only what was charged is released");
        self.glob_bytes.set(held);
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
            glob_bytes: self.glob_peak.get(),
            max_file_tokens: self.max_file_tokens.get(),
            max_recursion: self.max_recursion.get(),
            max_directory_depth: self.max_directory_depth.get(),
            work: self.work.get(),
        }
    }
}

/// The charges of glob table work made on this thread, for the tests that
/// check where a load is refused.
#[cfg(test)]
pub(crate) mod charges {
    use std::cell::RefCell;

    thread_local! {
        /// Each charge of glob table work since the last [`take`].
        static CHARGES: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
    }

    /// Record one charge.
    pub(super) fn record(units: usize) {
        CHARGES.with_borrow_mut(|charges| charges.push(units));
    }

    /// The charges recorded since the last call, in the order they were made.
    pub(crate) fn take() -> Vec<usize> {
        CHARGES.take()
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

#[cfg(test)]
mod tests {
    use std::error::Error;

    use purrdf_lex::allocation::{Admission, Memory, StorageError};

    use super::{ItemAdmission, Meter, charges};
    use crate::budget::Budget;
    use crate::error::LoadErrorKind;
    use crate::options::LoadOptions;
    use crate::path::RelPath;

    fn meter(max_work: usize) -> Meter {
        Meter::new(&LoadOptions {
            max_work,
            ..LoadOptions::default()
        })
    }

    #[test]
    fn native_storage_preserves_the_item_limit_and_cumulative_charges() {
        let file = RelPath::new("Cargo.toml").expect("path");
        let meter = Meter::new(&LoadOptions {
            max_item_bytes: 16,
            ..LoadOptions::default()
        });
        let mut admission = ItemAdmission::new(&meter, &file);
        admission.resize(16).expect("exact limit");
        admission.resize(8).expect("released storage");
        assert_eq!(meter.usage().item_bytes, 16);
        let native = admission.resize(9).expect_err("growth remains cumulative");
        assert_eq!(native, StorageError::AdmissionFailed);
        assert_eq!(admission.resize(8), Err(StorageError::AdmissionFailed));
        let error = admission.error(native);
        assert!(matches!(
            error.kind,
            LoadErrorKind::Budget {
                budget: Budget::ItemBytes,
                limit: 16,
            }
        ));
        assert_eq!(error.file, file);
        assert_eq!(meter.usage().item_bytes, 17);
    }

    #[test]
    fn native_storage_capacity_failure_has_its_original_error_source() {
        let file = RelPath::new("Cargo.toml").expect("path");
        let meter = Meter::new(&LoadOptions::default());
        let mut admission = ItemAdmission::new(&meter, &file);
        let mut values = Vec::<u64>::new();
        let native = {
            let mut memory = Memory::new(&mut admission);
            memory
                .reserve(&mut values, 1)
                .expect("small native allocation");
            let native = memory
                .reserve(&mut values, usize::MAX)
                .expect_err("impossible native capacity");
            memory
                .reserve(&mut values, 2)
                .expect("capacity failure did not poison admission");
            native
        };
        let error = admission.error(native);
        assert!(matches!(error.kind, LoadErrorKind::Storage(found) if found == native));
        assert_eq!(error.file, file);
        assert_eq!(
            error.source().expect("native source").to_string(),
            native.to_string()
        );
        assert_eq!(values.len(), 0);
    }

    #[test]
    fn work_up_to_the_limit_is_accepted_and_the_charge_that_passes_it_is_refused() {
        let file = RelPath::new("src/lib.rs").expect("path");
        let meter = meter(10);
        meter.work(4, &file, Some(3)).expect("below the limit");
        meter.work(6, &file, Some(3)).expect("at the limit");
        assert_eq!(meter.usage().work, 10);
        let error = meter.work(5, &file, Some(7)).expect_err("over the limit");
        assert!(
            matches!(
                error.kind,
                LoadErrorKind::Budget {
                    budget: Budget::Work,
                    limit: 10
                }
            ),
            "{error}"
        );
        assert_eq!((error.file.as_str(), error.line), ("src/lib.rs", Some(7)));
        // The count at the refusal is the limit and the charge that passed it.
        assert_eq!(meter.usage().work, 15);
        assert_eq!(charges::take(), vec![4, 6, 5]);
    }

    #[test]
    fn work_is_never_released_and_each_meter_counts_from_zero() {
        let file = RelPath::new("src/lib.rs").expect("path");
        let first = meter(100);
        first.globs(64, &file, None).expect("glob bytes");
        first.work(40, &file, None).expect("work");
        first.release_globs(64);
        first.work(40, &file, None).expect("work");
        assert_eq!(first.usage().work, 80);
        assert_eq!(first.usage().glob_bytes, 64);
        let second = meter(100);
        assert_eq!(second.usage().work, 0);
        second.work(1, &file, None).expect("work");
        assert_eq!(second.usage().work, 1);
        assert_eq!(first.usage().work, 80);
    }
}
