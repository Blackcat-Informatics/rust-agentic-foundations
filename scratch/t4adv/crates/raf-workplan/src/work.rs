// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Counts of the work the workplan does, for tests that assert a bound on the
//! work instead of timing it.
//!
//! A place that does a unit of work calls [`add`] with the kind and the amount.
//! Outside `cfg(test)` the call compiles to nothing. In a test, [`measure`]
//! runs a closure against fresh tallies and panics the moment any tally passes
//! its ceiling, so an implementation that does too much work fails at once
//! instead of running to the end. The tallies belong to the calling thread, so
//! only work done on that thread is counted.

use std::borrow::Borrow;
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;

/// One kind of work that is counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Work {
    /// A range of blanked blocks stepped past while a section's prose is read.
    BlankedSteps,
    /// A character of a section read for its prose.
    ProseChars,
    /// A line of a section read for the paths its code spans name.
    ContractLines,
    /// A record put into the sets the claims are judged against, and a slug of a claim judged against them.
    ClaimChecks,
    /// A heading stepped over looking for the end of a conditions list.
    ConditionsScan,
    /// A prose line looked at for a list item under a conditions heading.
    ProseLinesScanned,
    /// A heading judged by the rules of current rationale.
    HeadingsScanned,
    /// A binding path tested against the paths a section names.
    NamingLookups,
    /// A byte of a line walked or searched while its words are found.
    ScanBytes,
    /// A phrase tested against the words at one position.
    PhraseTests,
    /// An item of a change record judged for its outcome.
    OutcomeChecks,
    /// A change put into the set of slugs the moves of a record are judged against.
    SlugsListed,
    /// An exclusion of the coverage record put into the index of the exclusions.
    ExclusionsIndexed,
    /// A path of an entry, or a module it is below, looked up in the index of
    /// the exclusions.
    IndexLookups,
    /// An exclusion that a lookup found marked as matched.
    ExclusionMarks,
    /// A public item, re-export or test judged for coverage.
    EntriesJudged,
    /// A slug looked up in a set the claims are judged against.
    ClaimLookups,
    /// A slug looked up in a set of change slugs.
    SlugLookups,
    /// A path looked up among the paths a section names.
    NamedLookups,
    /// A byte of a paragraph read for its code spans.
    ParagraphBytes,
    /// A line compared while the first prose line of a range is found.
    RangeStarts,
    /// A unit judged by the rules of strict mode: a delivered capability
    /// judged by the content rules, or a package that strict mode alone
    /// enforces.
    StrictUnits,
}

/// A set that can be asked whether it holds a value.
pub(crate) trait Holds<Q: ?Sized> {
    /// Whether the set holds `value`.
    fn holds(&self, value: &Q) -> bool;
}

impl<T: Eq + Hash + Borrow<Q>, Q: Eq + Hash + ?Sized> Holds<Q> for HashSet<T> {
    fn holds(&self, value: &Q) -> bool {
        self.contains(value)
    }
}

impl<T: Ord + Borrow<Q>, Q: Ord + ?Sized> Holds<Q> for BTreeSet<T> {
    fn holds(&self, value: &Q) -> bool {
        self.contains(value)
    }
}

impl<S: Holds<Q>, Q: ?Sized> Holds<Q> for &S {
    fn holds(&self, value: &Q) -> bool {
        (**self).holds(value)
    }
}

/// A set whose every lookup is counted as one unit of a kind of [`Work`]. The
/// count is part of the lookup, so a lookup that does not go through the set
/// is not counted.
pub(crate) struct Counted<S> {
    set: S,
    work: Work,
}

impl<S> Counted<S> {
    /// `set`, counting its lookups as `work`.
    pub(crate) const fn new(work: Work, set: S) -> Self {
        Self { set, work }
    }

    /// Whether the set holds `value`; counts one lookup.
    pub(crate) fn contains<Q: ?Sized>(&self, value: &Q) -> bool
    where
        S: Holds<Q>,
    {
        add(self.work, 1);
        self.set.holds(value)
    }
}

/// The number of kinds of [`Work`].
#[cfg(test)]
const KINDS: usize = 22;

/// Every kind of [`Work`], in tally order.
#[cfg(test)]
const ALL: [Work; KINDS] = [
    Work::BlankedSteps,
    Work::ProseChars,
    Work::ContractLines,
    Work::ClaimChecks,
    Work::ConditionsScan,
    Work::ProseLinesScanned,
    Work::HeadingsScanned,
    Work::NamingLookups,
    Work::ScanBytes,
    Work::PhraseTests,
    Work::OutcomeChecks,
    Work::SlugsListed,
    Work::ExclusionsIndexed,
    Work::IndexLookups,
    Work::ExclusionMarks,
    Work::EntriesJudged,
    Work::ClaimLookups,
    Work::SlugLookups,
    Work::NamedLookups,
    Work::ParagraphBytes,
    Work::RangeStarts,
    Work::StrictUnits,
];

/// What one [`measure`] counted, per kind of [`Work`].
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Tally([usize; KINDS]);

#[cfg(test)]
impl Tally {
    /// The count of one kind of work.
    pub(crate) const fn get(&self, work: Work) -> usize {
        self.0[work as usize]
    }

    /// The count of every kind of work together.
    pub(crate) fn total(&self) -> usize {
        self.0.iter().sum()
    }
}

#[cfg(test)]
mod counting {
    use std::cell::Cell;

    use super::{KINDS, Tally, Work};

    thread_local! {
        /// The counts of this thread since the last reset.
        static TALLY: Cell<Tally> = const { Cell::new(Tally([0; KINDS])) };
        /// The count a kind may not pass while a measure runs: the one kind
        /// it limits, or every kind when that is `None`.
        static CEILING: Cell<Option<(Option<Work>, usize)>> = const { Cell::new(None) };
    }

    /// Count `amount` units of `work`, and panic if that passes the ceiling.
    pub(crate) fn add(work: Work, amount: usize) {
        let mut tally = TALLY.get();
        let slot = &mut tally.0[work as usize];
        *slot = slot.saturating_add(amount);
        let count = *slot;
        TALLY.set(tally);
        if let Some((limited, ceiling)) = CEILING.get()
            && limited.is_none_or(|kind| kind == work)
        {
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
        measured(None, ceiling, run)
    }

    /// Like [`measure`], but only `limited` is held to `ceiling`; the other
    /// kinds are counted without a limit.
    pub(crate) fn measure_kind<T>(
        limited: Work,
        ceiling: usize,
        run: impl FnOnce() -> T,
    ) -> (T, Tally) {
        measured(Some(limited), ceiling, run)
    }

    fn measured<T>(limited: Option<Work>, ceiling: usize, run: impl FnOnce() -> T) -> (T, Tally) {
        TALLY.set(Tally::default());
        CEILING.set(Some((limited, ceiling)));
        let _armed = Armed;
        let result = run();
        (result, TALLY.get())
    }
}

#[cfg(test)]
pub(crate) use counting::{add, measure, measure_kind};

/// Count work: nothing outside tests.
#[cfg(not(test))]
#[inline]
pub(crate) fn add(_work: Work, _amount: usize) {}

#[cfg(test)]
mod tests {
    use super::{ALL, Work, add, measure, measure_kind};

    #[test]
    fn a_counter_increments_by_what_is_added() {
        let ((), tally) = measure(100, || {
            add(Work::ClaimChecks, 1);
            add(Work::ClaimChecks, 4);
            add(Work::ContractLines, 7);
        });
        assert_eq!(tally.get(Work::ClaimChecks), 5);
        assert_eq!(tally.get(Work::ContractLines), 7);
        assert_eq!(tally.get(Work::BlankedSteps), 0);
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
        assert_eq!(tally.total(), ALL.len() * (ALL.len() + 1) / 2);
    }

    #[test]
    fn the_result_of_the_closure_is_returned() {
        let (value, tally) = measure(10, || 42);
        assert_eq!(value, 42);
        assert_eq!(tally, super::Tally::default());
    }

    #[test]
    #[should_panic(expected = "ProseChars work passed the ceiling of 3: 4")]
    fn passing_the_ceiling_panics_at_once() {
        measure(3, || {
            for _ in 0..4 {
                add(Work::ProseChars, 1);
            }
        });
    }

    #[test]
    #[should_panic(expected = "ProseChars work passed the ceiling of 3: 4")]
    fn the_panic_comes_before_the_work_that_follows_it() {
        measure(3, || {
            add(Work::ProseChars, 4);
            unreachable!("the ceiling should have stopped the closure");
        });
    }

    #[test]
    fn reaching_the_ceiling_is_allowed() {
        let ((), tally) = measure(3, || add(Work::ProseChars, 3));
        assert_eq!(tally.get(Work::ProseChars), 3);
    }

    #[test]
    #[should_panic(expected = "BlankedSteps work passed the ceiling of 2: 3")]
    fn a_limited_kind_panics_at_its_ceiling() {
        measure_kind(Work::BlankedSteps, 2, || add(Work::BlankedSteps, 3));
    }

    #[test]
    fn a_limited_measure_counts_the_other_kinds_without_a_ceiling() {
        let ((), tally) = measure_kind(Work::BlankedSteps, 2, || {
            add(Work::BlankedSteps, 2);
            add(Work::ClaimChecks, 500);
        });
        assert_eq!(tally.get(Work::ClaimChecks), 500);
    }

    #[test]
    fn the_tallies_are_reset_between_measures() {
        let ((), first) = measure(10, || add(Work::ProseChars, 5));
        let ((), second) = measure(10, || add(Work::ProseChars, 2));
        assert_eq!(first.get(Work::ProseChars), 5);
        assert_eq!(second.get(Work::ProseChars), 2);
    }

    #[test]
    fn a_measure_that_panicked_leaves_neither_counts_nor_ceiling_behind() {
        let caught = std::panic::catch_unwind(|| measure(1, || add(Work::ClaimChecks, 2)));
        assert!(caught.is_err());
        let ((), tally) = measure(10, || {});
        assert_eq!(tally.get(Work::ClaimChecks), 0);
        // With no measure running there is no ceiling.
        add(Work::ClaimChecks, 1000);
    }
}
