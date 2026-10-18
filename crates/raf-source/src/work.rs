// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Counts of the work the index does, for tests that assert a bound on the
//! work instead of timing it.
//!
//! A place that does a unit of work calls [`add`] with the kind and the amount.
//! Outside `cfg(test)` the call compiles to nothing. In a test, [`measure`]
//! runs a closure against fresh tallies and panics the moment any tally passes
//! its ceiling, so an implementation that does too much work fails at once
//! instead of running to the end. The tallies belong to the calling thread, so
//! only work done on that thread is counted.

#![cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "the counters are read only by tests, and the no-op shim keeps the call sites compiling"
    )
)]

/// One kind of work that is counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Work {
    /// An attribute summarised by the fact extractor.
    Attributes,
    /// A line and column converted to a byte offset.
    Positions,
    /// A character walked converting a column to a byte offset.
    ColumnSteps,
    /// An iteration of the glob segment matcher.
    PatternSteps,
    /// A wildcard pattern tried against a path by the exclude list.
    ExcludeTests,
    /// A path segment appended or visited while resolving a path.
    SegmentReads,
    /// A lookup of a name in the settled tables.
    Lookups,
    /// A name or binding read through a glob scope view.
    ViewReads,
    /// A unit of work charged to the export walk.
    WalkSteps,
}

/// The number of kinds of [`Work`].
const KINDS: usize = 9;

/// Every kind of [`Work`], in tally order.
#[cfg(test)]
const ALL: [Work; KINDS] = [
    Work::Attributes,
    Work::Positions,
    Work::ColumnSteps,
    Work::PatternSteps,
    Work::ExcludeTests,
    Work::SegmentReads,
    Work::Lookups,
    Work::ViewReads,
    Work::WalkSteps,
];

/// What one [`measure`] counted, per kind of [`Work`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Tally([usize; KINDS]);

impl Tally {
    /// The count of one kind of work.
    pub(crate) const fn get(&self, work: Work) -> usize {
        self.0[work as usize]
    }
}

#[cfg(test)]
mod counting {
    use std::cell::Cell;

    use super::{KINDS, Tally, Work};

    thread_local! {
        /// The counts of this thread since the last reset.
        static TALLY: Cell<Tally> = const { Cell::new(Tally([0; KINDS])) };
        /// The count no kind may pass while a measure runs.
        static CEILING: Cell<Option<usize>> = const { Cell::new(None) };
    }

    /// Count `amount` units of `work`, and panic if that passes the ceiling.
    pub(crate) fn add(work: Work, amount: usize) {
        let mut tally = TALLY.get();
        let slot = &mut tally.0[work as usize];
        *slot = slot.saturating_add(amount);
        let count = *slot;
        TALLY.set(tally);
        if let Some(ceiling) = CEILING.get() {
            assert!(
                count <= ceiling,
                "{work:?} work passed the ceiling of {ceiling}: {count}"
            );
        }
    }

    /// Clears the ceiling when a measure ends, even by a panic.
    struct Armed;

    impl Drop for Armed {
        fn drop(&mut self) {
            CEILING.set(None);
        }
    }

    /// Run `run` against fresh tallies and return its result with what it
    /// counted. Panics the moment any kind of work passes `ceiling`.
    pub(crate) fn measure<T>(ceiling: usize, run: impl FnOnce() -> T) -> (T, Tally) {
        TALLY.set(Tally::default());
        CEILING.set(Some(ceiling));
        let _armed = Armed;
        let result = run();
        (result, TALLY.get())
    }
}

#[cfg(test)]
pub(crate) use counting::{add, measure};

/// Count work: nothing outside tests.
#[cfg(not(test))]
#[inline]
pub(crate) fn add(_work: Work, _amount: usize) {}

#[cfg(test)]
mod tests {
    use super::{ALL, Work, add, measure};

    #[test]
    fn a_counter_increments_by_what_is_added() {
        let ((), tally) = measure(100, || {
            add(Work::Lookups, 1);
            add(Work::Lookups, 4);
            add(Work::WalkSteps, 7);
        });
        assert_eq!(tally.get(Work::Lookups), 5);
        assert_eq!(tally.get(Work::WalkSteps), 7);
        assert_eq!(tally.get(Work::Attributes), 0);
    }

    #[test]
    fn every_kind_has_its_own_counter() {
        let ((), tally) = measure(100, || {
            for (index, work) in ALL.into_iter().enumerate() {
                add(work, index + 1);
            }
        });
        for (index, work) in ALL.into_iter().enumerate() {
            assert_eq!(tally.get(work), index + 1, "{work:?}");
        }
    }

    #[test]
    fn the_result_of_the_closure_is_returned() {
        let (value, tally) = measure(10, || 42);
        assert_eq!(value, 42);
        assert_eq!(tally, super::Tally::default());
    }

    #[test]
    #[should_panic(expected = "PatternSteps work passed the ceiling of 3: 4")]
    fn passing_the_ceiling_panics_at_once() {
        measure(3, || {
            for _ in 0..4 {
                add(Work::PatternSteps, 1);
            }
        });
    }

    #[test]
    #[should_panic(expected = "ColumnSteps work passed the ceiling of 3: 4")]
    fn the_panic_comes_before_the_work_that_follows_it() {
        measure(3, || {
            add(Work::ColumnSteps, 4);
            unreachable!("the ceiling should have stopped the closure");
        });
    }

    #[test]
    fn reaching_the_ceiling_is_allowed() {
        let ((), tally) = measure(3, || add(Work::Positions, 3));
        assert_eq!(tally.get(Work::Positions), 3);
    }

    #[test]
    fn the_tallies_are_reset_between_measures() {
        let ((), first) = measure(10, || add(Work::ViewReads, 5));
        let ((), second) = measure(10, || add(Work::ViewReads, 2));
        assert_eq!(first.get(Work::ViewReads), 5);
        assert_eq!(second.get(Work::ViewReads), 2);
    }

    #[test]
    fn a_measure_that_panicked_leaves_neither_counts_nor_ceiling_behind() {
        let caught = std::panic::catch_unwind(|| measure(1, || add(Work::Lookups, 2)));
        assert!(caught.is_err());
        let ((), tally) = measure(10, || {});
        assert_eq!(tally.get(Work::Lookups), 0);
        // With no measure running there is no ceiling.
        add(Work::Lookups, 1000);
    }
}
