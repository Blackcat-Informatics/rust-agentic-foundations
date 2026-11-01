// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! What one load spends against its work and memory budgets.

use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use purrdf_lex::allocation::{Admission, StorageError};
use purrdf_lex::walk::WorkList;

use crate::budget::Budget;
use crate::error::{LoadError, LoadErrorKind};
use crate::modtree::TargetId;
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
    /// Bytes charged to the index, shared immutable derived glob facts and
    /// native load traversal scratch
    /// ([`Budget::ItemBytes`]).
    pub item_bytes: usize,
    /// Bytes charged to the facts of the files ([`Budget::FactBytes`]).
    pub fact_bytes: usize,
    /// The most bytes charged to import settling at once
    /// ([`Budget::GlobBytes`]): scope tables, retained named and glob reads,
    /// queues, subscriptions, graph scratch, impl self-type and trait-path
    /// traversal scratch, derived glob-fact construction scratch and waiting
    /// changes at their peak. Immutable cached glob facts use `item_bytes`.
    pub glob_bytes: usize,
    /// The most tokens of one file ([`Budget::FileTokens`]).
    pub max_file_tokens: usize,
    /// The highest value the recursion counter of the pre-scan reached in one
    /// file ([`Budget::Recursion`]).
    pub max_recursion: usize,
    /// The deepest directory that discovery or a `workspace.members` pattern
    /// entered, in levels below its root ([`Budget::DirectoryDepth`]).
    pub max_directory_depth: usize,
    /// The import-settling steps the load made ([`Budget::Work`]), including
    /// table reconstruction, retained reads, queues, subscriptions and graph
    /// traversal, plus impl self-type and trait-path traversal and its retained
    /// path segments and bytes. Named imports consume this budget even without glob imports.
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

/// Fallible reservations for temporary resolver storage held at once.
/// Each reservation precedes allocation; dropping the owner releases it.
pub(crate) struct Transient<'m> {
    meter: &'m Meter,
    bytes: Cell<usize>,
}

/// Optional failed-route observations owned by one original query grant.
pub(crate) struct QueryObservation<'m> {
    memory: Transient<'m>,
    targets: RefCell<Vec<TargetId>>,
}

impl<'m> QueryObservation<'m> {
    pub(crate) const fn new(meter: &'m Meter) -> Self {
        Self {
            memory: Transient::new(meter),
            targets: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn record(
        &self,
        target: TargetId,
        file: &RelPath,
        line: Option<usize>,
    ) -> Result<(), LoadError> {
        let mut targets = self.targets.borrow_mut();
        self.memory
            .work(targets.len().saturating_add(1), file, line)?;
        if targets.contains(&target) {
            return Ok(());
        }
        let bytes = targets.capacity() * size_of::<TargetId>();
        let steps = Steps::QueryAt {
            memory: &self.memory,
            file,
            line,
        };
        let mut admission = NativeAdmission::resume(steps, bytes);
        purrdf_lex::allocation::Memory::resume(&mut admission, bytes)
            .push(&mut targets, target)
            .map_err(|error| admission.graph_error(error.into()))
    }

    pub(crate) fn finish(self) -> Vec<TargetId> {
        self.targets.into_inner()
    }
}

impl<'m> Transient<'m> {
    pub(crate) const fn new(meter: &'m Meter) -> Self {
        Self {
            meter,
            bytes: Cell::new(0),
        }
    }

    pub(crate) const fn child(&self) -> Self {
        Self::new(self.meter)
    }

    pub(crate) fn charge(
        &self,
        bytes: usize,
        file: &RelPath,
        line: Option<usize>,
    ) -> Result<(), LoadError> {
        let limit = self.meter.options.max_glob_bytes;
        let next = self
            .bytes
            .get()
            .checked_add(bytes)
            .filter(|_| {
                self.meter
                    .glob_bytes
                    .get()
                    .checked_add(bytes)
                    .is_some_and(|held| held <= limit)
            })
            .ok_or_else(|| over_budget(Budget::GlobBytes, limit, file, line))?;
        self.meter.globs(bytes, file, line)?;
        self.bytes.set(next);
        Ok(())
    }

    pub(crate) fn work(
        &self,
        units: usize,
        file: &RelPath,
        line: Option<usize>,
    ) -> Result<(), LoadError> {
        self.meter.work(units, file, line)
    }

    /// Move an already admitted surviving payload to its enclosing owner.
    /// The global live total stays unchanged; this owner releases only scratch.
    pub(crate) fn transfer_to(
        &self,
        owner: &Self,
        bytes: usize,
        file: &RelPath,
        line: Option<usize>,
    ) -> Result<(), LoadError> {
        let retained = owner.bytes.get().checked_add(bytes);
        if !std::ptr::eq(self.meter, owner.meter)
            || std::ptr::eq(self, owner)
            || bytes > self.bytes.get()
            || retained.is_none()
        {
            return Err(LoadError::new(
                file,
                line,
                LoadErrorKind::Internal {
                    what: "retained import storage exceeds its original allocation grant"
                        .to_owned(),
                },
            ));
        }
        owner
            .bytes
            .set(retained.expect("checked destination grant"));
        self.bytes.set(self.bytes.get() - bytes);
        Ok(())
    }
}

impl Drop for Transient<'_> {
    fn drop(&mut self) {
        self.meter.release_globs(self.bytes.get());
    }
}

/// Lend native physical storage the resolver's original live-byte grant.
/// The parent lease retains successful result storage after this callback dies.
pub(crate) struct NativeAdmission<'m> {
    steps: Steps<'m>,
    items: Option<(&'m Meter, &'m RelPath, Option<usize>)>,
    live: usize,
    failure: Option<LoadError>,
}

impl<'m> NativeAdmission<'m> {
    pub(crate) const fn new(steps: Steps<'m>) -> Self {
        Self {
            steps,
            items: None,
            live: 0,
            failure: None,
        }
    }

    /// Resume a buffer originally constructed under this same parent lease.
    pub(crate) const fn resume(steps: Steps<'m>, live: usize) -> Self {
        Self {
            steps,
            items: None,
            live,
            failure: None,
        }
    }

    /// Admit index-emission scratch to the original cumulative item budget.
    pub(crate) const fn items(
        meter: &'m Meter,
        file: &'m RelPath,
        line: Option<usize>,
        bytes: usize,
    ) -> Self {
        Self {
            steps: Steps::Load,
            items: Some((meter, file, line)),
            live: bytes,
            failure: None,
        }
    }

    pub(crate) fn graph_error(&mut self, error: purrdf_core::graph::GraphError) -> LoadError {
        self.failure.take().unwrap_or_else(|| {
            let (file, line) = if let Some((_, file, line)) = self.items {
                (file.clone(), line)
            } else {
                match self.steps {
                    Steps::LoadAt { file, line, .. } => (file.clone(), Some(line)),
                    Steps::QueryAt { file, line, .. } | Steps::QueryItems { file, line, .. } => {
                        (file.clone(), line)
                    }
                    Steps::Load | Steps::Query(_) | Steps::Observed(_) => (RelPath::root(), None),
                }
            };
            LoadError::new(&file, line, LoadErrorKind::Graph(error))
        })
    }
}

impl Admission for NativeAdmission<'_> {
    fn resize(&mut self, live: usize) -> Result<(), StorageError> {
        if live > self.live {
            let result = if let Some((meter, file, line)) = self.items {
                meter.items(live - self.live, file, line)
            } else {
                self.steps.reserve(live - self.live)
            };
            if let Err(error) = result {
                if self.failure.is_none() {
                    self.failure = Some(error);
                }
                return Err(StorageError::AdmissionFailed);
            }
        } else if let Steps::LoadAt { memory, .. } | Steps::QueryAt { memory, .. } = self.steps {
            let released = self.live - live;
            // Memory invokes shrink only after the native allocation dies.
            memory.bytes.set(memory.bytes.get() - released);
            memory.meter.release_globs(released);
        }
        self.live = live;
        Ok(())
    }
}

/// Lend a native traversal spill the original cumulative item account.
pub(crate) fn push_item_work<T, const N: usize>(
    into: &mut WorkList<T, N>,
    value: T,
    meter: &Meter,
    file: &RelPath,
    line: Option<usize>,
) -> Result<(), LoadError> {
    let bytes = into.heap_capacity() * size_of::<T>();
    let mut admission = NativeAdmission::items(meter, file, line, bytes);
    into.try_push_admitted(
        value,
        &mut purrdf_lex::allocation::Memory::resume(&mut admission, bytes),
    )
    .map_err(|error| admission.graph_error(error.into()))
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
    /// A retained import read: reserve its storage before construction.
    LoadAt {
        memory: &'m Transient<'m>,
        file: &'m RelPath,
        line: usize,
    },
    /// A query after the load: on the query's own meter, against the limit
    /// of the work of one query.
    Query(&'m Meter),
    /// The same query, with opt-in observations of native failed routes.
    Observed(&'m QueryObservation<'m>),
    /// Post-load traversal storage charged cumulatively to `ItemBytes`.
    QueryItems {
        meter: &'m Meter,
        file: &'m RelPath,
        line: Option<usize>,
    },
    /// A query proof with explicit live scratch admission and work charges.
    QueryAt {
        memory: &'m Transient<'m>,
        file: &'m RelPath,
        line: Option<usize>,
    },
}

impl<'m> Steps<'m> {
    pub(crate) fn reserve(self, bytes: usize) -> Result<(), LoadError> {
        match self {
            Self::LoadAt { memory, file, line } => memory.charge(bytes, file, Some(line)),
            Self::QueryAt { memory, file, line } => memory.charge(bytes, file, line),
            Self::QueryItems { meter, file, line } => meter.items(bytes, file, line),
            Self::Load | Self::Query(_) | Self::Observed(_) => Ok(()),
        }
    }

    pub(crate) fn storage_work(self, units: usize) -> Result<(), LoadError> {
        match self {
            Self::LoadAt { memory, file, line } => {
                memory.work(units, file, Some(line))?;
                crate::work::add(crate::work::Work::SettleSteps, units);
                Ok(())
            }
            Self::QueryAt { memory, file, line } => {
                memory.work(units, file, line)?;
                crate::work::add(crate::work::Work::LookupCharges, units);
                Ok(())
            }
            Self::QueryItems { meter, file, line } => {
                meter.work(units, file, line)?;
                crate::work::add(crate::work::Work::LookupCharges, units);
                Ok(())
            }
            Self::Load | Self::Query(_) | Self::Observed(_) => Ok(()),
        }
    }

    pub(crate) fn temporary(self) -> Option<Transient<'m>> {
        match self {
            Self::LoadAt { memory, .. } | Self::QueryAt { memory, .. } => Some(memory.child()),
            Self::Load | Self::Query(_) | Self::Observed(_) | Self::QueryItems { .. } => None,
        }
    }

    pub(crate) fn at(self, memory: &'m Transient<'m>) -> Self {
        match self {
            Self::LoadAt { file, line, .. } => Self::LoadAt { memory, file, line },
            Self::QueryAt { file, line, .. } => Self::QueryAt { memory, file, line },
            Self::Load | Self::Query(_) | Self::Observed(_) | Self::QueryItems { .. } => self,
        }
    }

    pub(crate) fn query_meter(self) -> Option<&'m Meter> {
        match self {
            Self::Query(meter) | Self::QueryItems { meter, .. } => Some(meter),
            Self::Observed(observation) => Some(observation.memory.meter),
            Self::QueryAt { memory, .. } => Some(memory.meter),
            Self::Load | Self::LoadAt { .. } => None,
        }
    }

    /// Attach physical query-stack storage without changing load admission.
    pub(crate) fn with_query_storage(
        self,
        memory: Option<&'m Transient<'m>>,
        site: (&'m RelPath, Option<usize>),
    ) -> Self {
        if matches!(self, Self::QueryItems { .. }) {
            return self;
        }
        memory.map_or(self, |memory| Self::QueryAt {
            memory,
            file: site.0,
            line: site.1,
        })
    }
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

    /// Restore this account's refusal, otherwise preserve an already typed error.
    pub(crate) fn recover(&mut self, error: LoadError) -> LoadError {
        self.failure.take().unwrap_or(error)
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

    /// Continue the same query's work and storage allowance across stages.
    pub(crate) fn resume_query(options: &LoadOptions, usage: Usage) -> Self {
        Self {
            work_limit: options.max_query_work,
            ..Self::resume(options, usage)
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
    use std::error::Error;

    use purrdf_lex::allocation::{Admission, Memory, StorageError};

    use super::{ItemAdmission, Meter, NativeAdmission, Steps, Transient, charges};
    use crate::budget::Budget;
    use crate::error::{LoadError, LoadErrorKind, Origin};
    use crate::options::LoadOptions;
    use crate::path::RelPath;

    fn meter(max_work: usize) -> Meter {
        Meter::new(&LoadOptions {
            max_work,
            ..LoadOptions::default()
        })
    }

    #[test]
    fn native_recovery_preserves_nested_provenance_and_restores_its_own_refusal() {
        let account = RelPath::new("account/Cargo.toml").unwrap();
        let nested = RelPath::new("nested/src/lib.rs").unwrap();
        let meter = Meter::new(&LoadOptions {
            max_item_bytes: 0,
            ..LoadOptions::default()
        });
        let mut admission = ItemAdmission::new(&meter, &account);
        let original = || {
            LoadError::new(
                &nested,
                Some(17),
                LoadErrorKind::Storage(StorageError::AdmissionFailed),
            )
            .reached_by(&Origin::Discovery)
        };
        let preserved = admission.recover(original());
        assert_eq!(preserved.file, nested);
        assert_eq!(preserved.line, Some(17));
        assert!(matches!(
            preserved.origin.as_deref(),
            Some(Origin::Discovery)
        ));
        assert!(matches!(
            preserved.kind,
            LoadErrorKind::Storage(StorageError::AdmissionFailed)
        ));
        assert_eq!(admission.resize(1), Err(StorageError::AdmissionFailed));
        let restored = admission.recover(original());
        assert_eq!(restored.file, account);
        assert_eq!(restored.line, None);
        assert!(matches!(
            restored.kind,
            LoadErrorKind::Budget {
                budget: Budget::ItemBytes,
                limit: 0
            }
        ));
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

    #[test]
    fn transient_reservations_refuse_atomically_and_release_on_every_exit() {
        let file = RelPath::new("src/imports.rs").expect("path");
        for limit in [63, 64] {
            let meter = Meter::new(&LoadOptions {
                max_glob_bytes: limit,
                ..LoadOptions::default()
            });
            meter.globs(16, &file, Some(2)).expect("retained tables");
            {
                let first = Transient::new(&meter);
                first
                    .charge(24, &file, Some(3))
                    .expect("previous generation");
                {
                    let next = first.child();
                    let result = next.charge(24, &file, Some(7));
                    if limit == 64 {
                        result.expect("exact simultaneous limit");
                        assert_eq!(meter.glob_bytes.get(), 64);
                    } else {
                        let error = result.expect_err("one below simultaneous limit");
                        assert_eq!(
                            (error.file.as_str(), error.line),
                            ("src/imports.rs", Some(7))
                        );
                        assert!(matches!(
                            error.kind,
                            LoadErrorKind::Budget {
                                budget: Budget::GlobBytes,
                                limit: 63
                            }
                        ));
                        assert_eq!(next.bytes.get(), 0);
                        assert_eq!(meter.glob_bytes.get(), 40);
                    }
                    assert!(next.charge(usize::MAX, &file, Some(9)).is_err());
                    assert_eq!(meter.glob_bytes.get(), if limit == 64 { 64 } else { 40 });
                }
                assert_eq!(meter.glob_bytes.get(), 40);
                assert!(first.charge(limit, &file, Some(11)).is_err());
                assert_eq!(first.bytes.get(), 24);
                first
                    .work(1, &file, Some(3))
                    .expect("work is nonrefundable");
            }
            assert_eq!(meter.glob_bytes.get(), 16);
            assert_eq!(meter.usage().work, 1);
            let neighbor = Transient::new(&meter);
            neighbor
                .charge(limit - 16, &file, Some(12))
                .expect("failed owners left no phantom reservation");
        }
    }

    #[test]
    fn retained_payload_moves_its_original_grant_without_readmission() {
        let file = RelPath::new("src/imports.rs").expect("path");
        let options = LoadOptions {
            max_glob_bytes: 64,
            ..LoadOptions::default()
        };
        let meter = Meter::new(&options);
        let owner = Transient::new(&meter);
        owner.charge(16, &file, Some(1)).expect("batch storage");
        {
            let scratch = owner.child();
            scratch.charge(48, &file, Some(2)).expect("exact peak");
            scratch
                .transfer_to(&owner, 24, &file, Some(2))
                .expect("retained result");
            assert_eq!(meter.glob_bytes.get(), 64);
            assert_eq!(meter.usage().glob_bytes, 64);
            assert!(scratch.transfer_to(&owner, 25, &file, Some(3)).is_err());
            assert_eq!((scratch.bytes.get(), owner.bytes.get()), (24, 40));
            let other_meter = Meter::new(&options);
            let unrelated = Transient::new(&other_meter);
            assert!(scratch.transfer_to(&unrelated, 1, &file, Some(4)).is_err());
            assert_eq!(other_meter.glob_bytes.get(), 0);
        }
        assert_eq!(meter.glob_bytes.get(), 40);
        owner
            .charge(24, &file, Some(5))
            .expect("released scratch is reusable");
        assert!(owner.charge(1, &file, Some(6)).is_err());
        drop(owner);
        assert_eq!(meter.glob_bytes.get(), 0);
    }

    #[test]
    fn native_emission_spill_preserves_cumulative_item_admission() {
        let file = RelPath::new("src/lib.rs").expect("path");
        let run = |count, limit| {
            let meter = Meter::new(&LoadOptions {
                max_item_bytes: limit,
                max_glob_bytes: 0,
                ..LoadOptions::default()
            });
            let mut pending = purrdf_lex::walk::WorkList::<usize, 1>::new();
            let mut result = Ok(());
            for value in 0..count {
                let live = pending.heap_capacity() * size_of::<usize>();
                let mut admission = NativeAdmission::items(&meter, &file, Some(7), live);
                result = pending
                    .try_push_admitted(value, &mut Memory::resume(&mut admission, live))
                    .map_err(|error| admission.graph_error(error.into()));
                if result.is_err() {
                    break;
                }
            }
            let spent = meter.usage().item_bytes;
            drop(pending);
            assert_eq!(meter.usage().item_bytes, spent, "items are cumulative");
            assert_eq!(meter.usage().glob_bytes, 0);
            (result, spent)
        };
        assert!(
            run(1, 0).0.is_ok(),
            "inline neighbor needs no item allocation"
        );
        let (result, spent) = run(33, usize::MAX);
        result.expect("several native growths");
        assert!(spent > 0);
        assert!(run(33, spent).0.is_ok());
        let (result, _) = run(33, spent - 1);
        let error = result.expect_err("one item byte less");
        assert_eq!((error.file.as_str(), error.line), ("src/lib.rs", Some(7)));
        assert!(matches!(error.kind, LoadErrorKind::Budget {
            budget: Budget::ItemBytes, limit,
        } if limit == spent - 1));
    }

    #[test]
    fn native_graph_storage_preserves_overlapping_grants_and_typed_refusals() {
        let file = RelPath::new("src/imports.rs").expect("path");
        let graph = [vec![1], vec![0, 2], vec![]];
        let run = |limit| {
            let meter = Meter::new(&LoadOptions {
                max_glob_bytes: limit,
                ..LoadOptions::default()
            });
            let outer = Transient::new(&meter);
            outer
                .charge(17, &file, Some(2))
                .expect("unrelated live payload");
            {
                let lease = outer.child();
                let mut admission = NativeAdmission::new(Steps::LoadAt {
                    memory: &lease,
                    file: &file,
                    line: 7,
                });
                let result = purrdf_core::graph::tarjan_scc_with_memory(
                    &graph,
                    &mut Memory::new(&mut admission),
                )
                .map_err(|error| admission.graph_error(error));
                match result {
                    Ok(components) => {
                        assert_eq!(components.len(), 2);
                        // Construction scratch has died; the surviving result
                        // remains covered even after the native callback dies.
                        assert!(lease.bytes.get() > 0);
                        assert_eq!(meter.glob_bytes.get(), 17 + lease.bytes.get());
                        drop(components);
                    }
                    Err(error) => {
                        assert_eq!(
                            (error.file.as_str(), error.line),
                            ("src/imports.rs", Some(7))
                        );
                        assert!(matches!(error.kind, LoadErrorKind::Budget {
                            budget: Budget::GlobBytes, limit: found,
                        } if found == limit));
                        assert_eq!(lease.bytes.get(), 0);
                    }
                }
            }
            assert_eq!(meter.glob_bytes.get(), 17);
            meter.usage().glob_bytes
        };
        let peak = run(usize::MAX);
        assert!(peak > 17);
        assert_eq!(run(peak), peak);
        assert!(run(peak - 1) < peak);
    }
}
