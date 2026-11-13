// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Exact live-source counts with descriptors and tables under their run's account.
//! RDF storage, parsing, validation and output share that same original account.

mod rdf;

use super::{
    identity,
    pairing::Matches,
    resource::{Grant, Meter, Refusal, Scratch, Text},
};
use purrdf_lex::allocation::OrderedTable;
use std::{cell::Cell, cmp::Ordering};

pub(super) use super::output::Failure;

struct Pair<'m> {
    digest: [u8; 32],
    left: Text<'m>,
    right: Text<'m>,
}

fn order(
    meter: &Meter,
    digest: &[u8; 32],
    left: &str,
    right: &str,
    prior: &Pair<'_>,
) -> Result<Ordering, Refusal> {
    meter.work(32)?;
    let order = digest.cmp(&prior.digest);
    if !order.is_eq() {
        return Ok(order);
    }
    let order = meter.compare(left, &prior.left)?;
    if !order.is_eq() {
        return Ok(order);
    }
    meter.compare(right, &prior.right)
}

/// Payloads are destroyed before the table-layout grant is refunded.
struct Entries<'m> {
    table: OrderedTable<Pair<'m>, Cell<usize>>,
    grant: Grant<'m>,
}

impl<'m> Entries<'m> {
    fn new(meter: &'m Meter) -> Self {
        Self {
            table: OrderedTable::default(),
            grant: Grant::new(meter),
        }
    }
    fn meter(&self) -> &'m Meter {
        self.grant.meter()
    }
    fn iter(&self) -> impl Iterator<Item = (&Pair<'m>, &Cell<usize>)> {
        self.table.iter()
    }
    fn get(
        &self,
        digest: &[u8; 32],
        left: &str,
        right: &str,
    ) -> Result<Option<&Cell<usize>>, Refusal> {
        self.table
            .try_get_by(|prior| order(self.meter(), digest, left, right, prior))
    }
    fn insert(
        &mut self,
        digest: [u8; 32],
        left: &str,
        right: &str,
        count: usize,
    ) -> Result<bool, Failure<'m>> {
        if self.get(&digest, left, right)?.is_some() {
            return Ok(false);
        }
        let meter = self.meter();
        let pair = Pair {
            digest,
            left: Text::new(meter, left)?,
            right: Text::new(meter, right)?,
        };
        // Structural publication visits no more levels than retained entries.
        meter.work(self.table.len().checked_add(1).ok_or(Failure::Overflow)?)?;
        self.grant.memory(|memory| {
            self.table.try_insert_with(
                pair,
                memory,
                |a, b| order(meter, &a.digest, &a.left, &a.right, b),
                |_, _, _| Ok(Cell::new(count)),
            )?;
            Ok(())
        })?;
        Ok(true)
    }
}

struct Present<'m> {
    entries: Entries<'m>,
}

impl<'m> Present<'m> {
    fn collect(matches: &'m Matches<'_>) -> Result<Self, Failure<'m>> {
        let meter = matches.analysis.meter();
        let mut entries = Entries::new(meter);
        if matches.witnesses.is_empty() {
            return Ok(Self { entries });
        }
        let mut endpoints = Scratch::new(meter);
        for _ in &matches.analysis.functions {
            endpoints.push(None)?;
        }
        for witness in &matches.witnesses {
            meter.work(1)?;
            for at in [witness.left.function, witness.right.function] {
                meter.work(1)?;
                let cached = endpoints
                    .as_mut_slice()
                    .get_mut(at)
                    .ok_or(Refusal::Invariant("verified pair lost its source function"))?;
                if cached.is_none() {
                    let function = matches
                        .analysis
                        .functions
                        .get(at)
                        .ok_or(Refusal::Invariant("verified pair lost its source function"))?;
                    *cached = Some(identity::endpoint(function, meter)?);
                }
            }
            let endpoint = |at| {
                endpoints
                    .as_slice()
                    .get(at)
                    .and_then(Option::as_ref)
                    .ok_or(Refusal::Invariant(
                        "verified pair lost its admitted endpoint",
                    ))
            };
            let (mut left, mut right) = (
                endpoint(witness.left.function)?,
                endpoint(witness.right.function)?,
            );
            if meter.compare(left, right)?.is_gt() {
                std::mem::swap(&mut left, &mut right);
            }
            if let Some(count) = entries.get(&witness.digest, left, right)? {
                count.set(count.get().checked_add(1).ok_or(Failure::Overflow)?);
            } else {
                entries.insert(witness.digest, left, right, 1)?;
            }
        }
        Ok(Self { entries })
    }
}

/// Normative allowances borrow the one selected run's original account.
pub(super) struct Record<'m> {
    entries: Entries<'m>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Comparison {
    Unchanged,
    Shrunk { removed: usize },
}

impl<'m> Record<'m> {
    pub(super) fn adopt(matches: &'m Matches<'_>) -> Result<Self, Failure<'m>> {
        Ok(Self {
            entries: Present::collect(matches)?.entries,
        })
    }
    fn same_run(&self, matches: &Matches<'_>) -> Result<(), Failure<'m>> {
        if !std::ptr::eq(self.entries.meter(), matches.analysis.meter()) {
            return Err(Failure::AccountMismatch);
        }
        Ok(())
    }
    pub(super) fn compare(&self, matches: &'m Matches<'_>) -> Result<Comparison, Failure<'m>> {
        self.same_run(matches)?;
        self.compare_present(&Present::collect(matches)?)
    }
    fn compare_present(&self, present: &Present<'_>) -> Result<Comparison, Failure<'m>> {
        let meter = self.entries.meter();
        for (pair, count) in present.entries.iter() {
            meter.work(1)?;
            let recorded = self
                .entries
                .get(&pair.digest, &pair.left, &pair.right)?
                .map_or(0, Cell::get);
            if count.get() > recorded {
                meter.work(
                    pair.left
                        .len()
                        .checked_add(pair.right.len())
                        .ok_or(Failure::Overflow)?,
                )?;
                return Err(Failure::Growth {
                    digest: pair.digest,
                    left: *purrdf_hash::blake3::hash(pair.left.as_bytes()).as_bytes(),
                    right: *purrdf_hash::blake3::hash(pair.right.as_bytes()).as_bytes(),
                    recorded,
                    present: count.get(),
                });
            }
        }
        let mut removed = 0usize;
        for (pair, count) in self.entries.iter() {
            meter.work(1)?;
            let remaining = present
                .entries
                .get(&pair.digest, &pair.left, &pair.right)?
                .map_or(0, Cell::get);
            removed = removed
                .checked_add(
                    count
                        .get()
                        .checked_sub(remaining)
                        .ok_or(Failure::Overflow)?,
                )
                .ok_or(Failure::Overflow)?;
        }
        Ok(if removed == 0 {
            Comparison::Unchanged
        } else {
            Comparison::Shrunk { removed }
        })
    }
    pub(super) fn rewrite_existing(&self, matches: &'m Matches<'_>) -> Result<Self, Failure<'m>> {
        self.same_run(matches)?;
        let present = Present::collect(matches)?;
        self.compare_present(&present)?;
        Ok(Self {
            entries: present.entries,
        })
    }
    pub(super) fn to_text<R>(
        &self,
        format: raf_rdf::Format,
        observe: impl for<'f> FnOnce(Result<Text<'m>, super::OutputFailure<'f>>) -> R,
    ) -> R {
        rdf::write(self, format, observe)
    }
    pub(super) fn from_text<R>(
        text: &str,
        format: raf_rdf::Format,
        meter: &'m Meter,
        observe: impl for<'f> FnOnce(Result<Self, super::OutputFailure<'f>>) -> R,
    ) -> R {
        rdf::read(text, format, meter, observe)
    }
}

#[cfg(test)]
mod tests;
