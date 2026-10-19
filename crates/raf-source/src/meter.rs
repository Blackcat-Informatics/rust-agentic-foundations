// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! What one load spends against its work and memory budgets.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

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

/// The running totals of one load, or of one query after it, checked
/// against their limits.
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
    /// The most steps of work: `max_work` for a load, `max_query_work` for
    /// a query after it.
    work_limit: usize,
    /// For a query, the module whose lookup first made the work pass the
    /// limit: the query is refused at its next checkpoint, naming it.
    query_passed: Cell<Option<usize>>,
}

/// The lookup steps a load makes while it builds its index: each glob read
/// that a lookup of what the globs of a module bring in under a name looks
/// at, and each view that holds the name looked at to find those reads.
/// They are made by code that answers and cannot refuse, so they are
/// counted here as they are made and drained into the work of the load at
/// its next checkpoint, which can (see `Index::settle_lookups`). An atomic,
/// so that the index stays `Sync`; it clones and compares by its count, so
/// the index keeps its derives. The count is zero once the index is built:
/// a query after the load counts on its own meter (see [`Steps`]), and a
/// debug build checks that none counts here.
#[derive(Debug, Default)]
pub(crate) struct LookupCount {
    /// The steps counted since the last take.
    count: AtomicUsize,
    /// True once the index is built, until its tables are settled again:
    /// a lookup counted here then is a query's, counted in the wrong place.
    built: AtomicBool,
}

impl LookupCount {
    /// Count `steps` more. Only the thread that builds the index counts, so
    /// the count is read and written apart.
    pub(crate) fn add(&self, steps: usize) {
        debug_assert!(
            !self.built.load(Ordering::Relaxed),
            "a lookup after the load counted on the count of the load"
        );
        self.count
            .store(self.get().saturating_add(steps), Ordering::Relaxed);
    }

    /// The steps counted since the last take, and none left.
    pub(crate) fn take(&self) -> usize {
        self.count.swap(0, Ordering::Relaxed)
    }

    /// The steps counted since the last take.
    pub(crate) fn get(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    /// Mark the index built, or settled again when `built` is false.
    pub(crate) fn built(&self, built: bool) {
        self.built.store(built, Ordering::Relaxed);
    }
}

impl Clone for LookupCount {
    fn clone(&self) -> Self {
        Self {
            count: AtomicUsize::new(self.get()),
            built: AtomicBool::new(self.built.load(Ordering::Relaxed)),
        }
    }
}

impl PartialEq for LookupCount {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl Eq for LookupCount {}

/// Where a lookup of what the globs of a module bring in counts its steps.
#[derive(Clone, Copy)]
pub(crate) enum Steps<'m> {
    /// The load: on the index's [`LookupCount`], drained into the work of
    /// the load at its next checkpoint.
    Load,
    /// A query after the load: on the query's own meter, against the limit
    /// of the work of one query.
    Query(&'m Meter),
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
            work_limit: options.max_work,
            query_passed: Cell::new(None),
        }
    }

    /// The meter of one query after a load made with `options`: its work is
    /// the lookup steps of the query, from zero, against `max_query_work`.
    pub(crate) fn query(options: &LoadOptions) -> Self {
        Self {
            work_limit: options.max_query_work,
            ..Self::new(options)
        }
    }

    /// Count `units` lookup steps of a query, made in `module`. The count is
    /// not refused here, where the lookup answers; the first count that
    /// passes the limit records its module, and the query is refused at its
    /// next checkpoint (see [`Meter::query_passed`]).
    pub(crate) fn count_query(&self, units: usize, module: usize) {
        let sum = self.work.get().saturating_add(units);
        self.work.set(sum);
        if sum > self.work_limit && self.query_passed.get().is_none() {
            self.query_passed.set(Some(module));
        }
    }

    /// The module whose lookup made the work of the query pass its limit,
    /// if one did.
    pub(crate) fn query_passed(&self) -> Option<usize> {
        self.query_passed.get()
    }

    /// The most steps of work this meter allows.
    pub(crate) fn work_limit(&self) -> usize {
        self.work_limit
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
        let limit = self.work_limit;
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
    use super::{Meter, charges};
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
