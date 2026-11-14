// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Original work and live-storage grants of one fragment analysis.

use std::cell::Cell;
use std::cmp::Ordering;

use purrdf_lex::allocation::{Admission, Memory, StorageError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Refusal {
    Count {
        resource: &'static str,
        limit: usize,
    },
    Work {
        limit: usize,
    },
    Memory {
        limit: usize,
    },
    SyntaxExcess {
        nodes: usize,
        prepaid: usize,
        held: usize,
        kind: &'static str,
    },
    Storage(StorageError),
    Invariant(&'static str),
    Unsupported(&'static str),
}

impl From<StorageError> for Refusal {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&super::Error::from(*self), output)
    }
}

pub(super) struct Work<'m> {
    meter: &'m Meter,
}

impl purrdf_lex::control::Progress for Work<'_> {
    type Error = Refusal;

    fn charge(&mut self, units: u64) -> Result<(), Refusal> {
        let units = usize::try_from(units).map_err(|_| Refusal::Work {
            limit: self.meter.work_limit,
        })?;
        self.meter.work(units)
    }
}

pub(super) struct Meter {
    work: Cell<usize>,
    live: Cell<usize>,
    peak: Cell<usize>,
    work_limit: usize,
    memory_limit: usize,
    capture: Cell<Option<Refusal>>,
}

/// The original account after admission ends, held until its diagnostic drops.
/// No reset, clone or new admission can reconstruct authority from these values.
#[derive(Debug)]
pub(super) struct TerminalAccount {
    _work: usize,
    _live: usize,
    _peak: usize,
    _work_limit: usize,
    _memory_limit: usize,
    _capture: Option<Refusal>,
}

/// Already admitted storage filled infallibly when the account becomes terminal.
pub(super) type TerminalSlot = Box<std::mem::MaybeUninit<TerminalAccount>>;

impl Meter {
    pub(super) const fn new(work_limit: usize, memory_limit: usize) -> Self {
        Self {
            work: Cell::new(0),
            live: Cell::new(0),
            peak: Cell::new(0),
            work_limit,
            memory_limit,
            capture: Cell::new(None),
        }
    }

    /// Consume the original account without allocating or resetting its state.
    /// The owner destroys analysis buffers before making this terminal transfer.
    pub(super) fn into_terminal(self) -> TerminalAccount {
        TerminalAccount {
            _work: self.work.into_inner(),
            _live: self.live.into_inner(),
            _peak: self.peak.into_inner(),
            _work_limit: self.work_limit,
            _memory_limit: self.memory_limit,
            _capture: self.capture.into_inner(),
        }
    }

    /// Fund the terminal record's reads/writes and allocate its slot fallibly.
    /// Its bytes remain in this original account until explicit deallocation.
    pub(super) fn terminal_slot(&self) -> Result<TerminalSlot, Refusal> {
        let work = size_of::<TerminalAccount>()
            .checked_mul(4)
            .ok_or(Refusal::Storage(StorageError::SizeOverflow))?;
        self.work(work)?;
        let mut native = self.native();
        let result = Memory::new(&mut native).boxed(std::mem::MaybeUninit::uninit());
        result.map_err(|cause| native.failure.unwrap_or(Refusal::Storage(cause)))
    }

    pub(super) fn work(&self, additional: usize) -> Result<(), Refusal> {
        let next = self
            .work
            .get()
            .checked_add(additional)
            .filter(|next| *next <= self.work_limit)
            .ok_or(Refusal::Work {
                limit: self.work_limit,
            })?;
        self.work.set(next);
        Ok(())
    }

    pub(super) fn usage(&self) -> (usize, usize) {
        (self.work.get(), self.peak.get())
    }

    /// Prepay native payload without borrowing the caller-owned analysis.
    /// The caller refunds this exact amount only after its worker has joined.
    pub(super) fn prepay_native(&self, bytes: usize) -> Result<(), Refusal> {
        let mut native = self.native();
        native
            .resize(bytes)
            .map_err(|cause| native.failure.unwrap_or(Refusal::Storage(cause)))
    }

    /// Refund a distinct prepaid envelope after its native payload has died.
    pub(super) fn refund_native(&self, bytes: usize) {
        let mut native = Native {
            meter: self,
            live: bytes,
            failure: None,
        };
        // The paired prepayment remains in the original total until this join.
        native
            .resize(0)
            .expect("a joined native envelope refunds its original admitted amount");
    }

    /// Start a fresh subaccount without resetting the original run's budgets.
    pub(super) fn native(&self) -> Native<'_> {
        self.capture.set(None);
        Native {
            meter: self,
            live: 0,
            failure: None,
        }
    }

    pub(super) const fn capture(&self) -> &Cell<Option<Refusal>> {
        &self.capture
    }

    pub(super) const fn progress(&self) -> Work<'_> {
        Work { meter: self }
    }

    pub(super) fn compare(&self, left: &str, right: &str) -> Result<Ordering, Refusal> {
        self.work(left.len().min(right.len()).saturating_add(1))?;
        Ok(left.cmp(right))
    }

    /// Grow a retained analysis buffer through its original native account.
    /// Retained buffers must stay with the owning analysis until it ends.
    pub(super) fn push_retained<T>(&self, values: &mut Vec<T>, value: T) -> Result<(), Refusal> {
        let relocation = if values.len() == values.capacity() {
            values.len()
        } else {
            0
        };
        self.work(relocation.checked_add(1).ok_or(Refusal::Work {
            limit: self.work_limit,
        })?)?;
        let bytes = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(Refusal::Storage(StorageError::SizeOverflow))?;
        let mut admission = Native {
            meter: self,
            live: bytes,
            failure: None,
        };
        Memory::resume(&mut admission, bytes)
            .push(values, value)
            .map_err(|error| admission.failure.unwrap_or(Refusal::Storage(error)))
    }

    /// Reserve only the buffer layout; owned payloads require their own grants.
    pub(super) fn reserve_retained<T>(
        &self,
        values: &mut Vec<T>,
        required: usize,
    ) -> Result<(), Refusal> {
        if required > values.capacity() {
            self.work(values.len())?;
        }
        let bytes = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(Refusal::Storage(StorageError::SizeOverflow))?;
        let mut admission = Native {
            meter: self,
            live: bytes,
            failure: None,
        };
        Memory::resume(&mut admission, bytes)
            .reserve(values, required)
            .map_err(|error| admission.failure.unwrap_or(Refusal::Storage(error)))
    }

    pub(super) fn release_buffer<T>(&self, values: Vec<T>) {
        let bytes = values
            .capacity()
            .checked_mul(size_of::<T>())
            .expect("an admitted vector retains its checked layout");
        let mut admission = Native {
            meter: self,
            live: bytes,
            failure: None,
        };
        Memory::resume(&mut admission, bytes)
            .release_vec(values)
            .expect("an admitted vector has a valid layout and infallible refund");
    }
}

pub(super) struct Native<'m> {
    meter: &'m Meter,
    live: usize,
    failure: Option<Refusal>,
}

impl Admission for Native<'_> {
    fn progress_enabled(&self) -> bool {
        true
    }

    fn checkpoint(&mut self, units: u64) -> Result<(), StorageError> {
        let result = usize::try_from(units)
            .map_err(|_| Refusal::Work {
                limit: self.meter.work_limit,
            })
            .and_then(|units| self.meter.work(units));
        result.map_err(|error| {
            let first = *self.failure.get_or_insert(error);
            self.meter.capture().set(Some(first));
            StorageError::AdmissionFailed
        })
    }

    fn resize(&mut self, live: usize) -> Result<(), StorageError> {
        if live > self.live {
            let next = self
                .meter
                .live
                .get()
                .checked_add(live - self.live)
                .filter(|next| *next <= self.meter.memory_limit);
            let Some(next) = next else {
                let first = *self.failure.get_or_insert(Refusal::Memory {
                    limit: self.meter.memory_limit,
                });
                self.meter.capture().set(Some(first));
                return Err(StorageError::AdmissionFailed);
            };
            self.meter.live.set(next);
            self.meter.peak.set(self.meter.peak.get().max(next));
        } else {
            self.meter
                .live
                .set(self.meter.live.get() - (self.live - live));
        }
        self.live = live;
        Ok(())
    }
}

/// One original subgrant. Its payload must be declared before this owner.
pub(super) struct Grant<'m> {
    native: Native<'m>,
}

impl<'m> Grant<'m> {
    pub(super) fn new(meter: &'m Meter) -> Self {
        Self {
            native: meter.native(),
        }
    }

    pub(super) const fn meter(&self) -> &'m Meter {
        self.native.meter
    }

    pub(super) const fn captured(&self) -> Option<Refusal> {
        self.native.failure
    }

    /// Admit a native operation's requested-storage envelope before it starts.
    /// The operation's payload must die before this reservation is refunded.
    pub(super) fn reserve_native(&mut self, bytes: usize) -> Result<(), Refusal> {
        self.native
            .resize(bytes)
            .map_err(|error| self.native.failure.unwrap_or(Refusal::Storage(error)))
    }

    /// Keep an owned diagnostic unchanged; its caller retains this grant.
    pub(super) fn raw_memory<T, E>(
        &mut self,
        operation: impl FnOnce(&mut Memory<'_, Native<'m>>) -> Result<T, E>,
    ) -> Result<T, E> {
        let bytes = self.native.live;
        operation(&mut Memory::resume(&mut self.native, bytes))
    }

    pub(super) fn memory<T>(
        &mut self,
        operation: impl FnOnce(&mut Memory<'_, Native<'m>>) -> Result<T, Refusal>,
    ) -> Result<T, Refusal> {
        let bytes = self.native.live;
        let result = operation(&mut Memory::resume(&mut self.native, bytes));
        result.map_err(|error| self.native.failure.unwrap_or(error))
    }

    pub(super) fn try_memory<T, E: From<Refusal>>(
        &mut self,
        operation: impl FnOnce(&mut Memory<'_, Native<'m>>) -> Result<T, E>,
    ) -> Result<T, E> {
        let bytes = self.native.live;
        let result = operation(&mut Memory::resume(&mut self.native, bytes));
        result.map_err(|error| self.native.failure.map_or(error, E::from))
    }
}

impl Drop for Grant<'_> {
    fn drop(&mut self) {
        // Every payload precedes its grant, so the refund follows destruction.
        self.native
            .resize(0)
            .expect("original grant refunds cannot fail");
    }
}

/// Owned UTF-8 whose capacity cannot escape its original borrowed grant.
pub(super) struct Text<'m> {
    text: String,
    _grant: Grant<'m>,
}

impl<'m> Text<'m> {
    pub(super) fn render_admitted<R>(
        meter: &'m Meter,
        graph: &raf_rdf::AdmittedGraph<Native<'m>>,
        format: raf_rdf::Format,
        observe: impl for<'f> FnOnce(Result<Self, super::OutputFailure<'f>>) -> R,
    ) -> R {
        let mut scratch = Grant::new(meter);
        let mut output = Grant::new(meter);
        let result = output.raw_memory(|destination| {
            scratch.raw_memory(|working| graph.to_text_with_memory(format, working, destination))
        });
        match result {
            Ok(text) => observe(Ok(Self {
                text,
                _grant: output,
            })),
            Err(cause) => {
                let captured = output.captured().or_else(|| scratch.captured());
                observe(Err(super::output::OutputFailure::rdf(&cause, captured)))
            }
        }
    }

    pub(super) fn new(meter: &'m Meter, text: &str) -> Result<Self, Refusal> {
        let mut grant = Grant::new(meter);
        meter.work(text.len())?;
        let text = grant.memory(|memory| Ok(memory.string(text)?))?;
        Ok(Self {
            text,
            _grant: grant,
        })
    }

    pub(super) fn build(
        meter: &'m Meter,
        operation: impl FnOnce(&mut Memory<'_, Native<'m>>) -> Result<String, Refusal>,
    ) -> Result<Self, Refusal> {
        let mut grant = Grant::new(meter);
        let text = grant.memory(operation)?;
        Ok(Self {
            text,
            _grant: grant,
        })
    }

    pub(super) fn display(
        meter: &'m Meter,
        value: &impl std::fmt::Display,
    ) -> Result<Self, Refusal> {
        Self::build(meter, |memory| Ok(memory.format_with_progress(value)?))
    }

    pub(super) fn try_build<E: From<Refusal>>(
        meter: &'m Meter,
        operation: impl FnOnce(&mut Memory<'_, Native<'m>>) -> Result<String, E>,
    ) -> Result<Self, E> {
        let mut grant = Grant::new(meter);
        let text = grant.try_memory(operation)?;
        Ok(Self {
            text,
            _grant: grant,
        })
    }
}

impl std::ops::Deref for Text<'_> {
    type Target = str;
    fn deref(&self) -> &str {
        &self.text
    }
}

impl std::fmt::Debug for Text<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.text, formatter)
    }
}

#[cfg(test)]
impl PartialEq for Text<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

#[cfg(test)]
impl Eq for Text<'_> {}

impl AsRef<str> for Text<'_> {
    fn as_ref(&self) -> &str {
        &self.text
    }
}

/// A native allocation and its original grant end together, in that order.
pub(super) struct Scratch<'m, T> {
    values: Vec<T>,
    meter: &'m Meter,
}

impl<'m, T> Scratch<'m, T> {
    pub(super) const fn new(meter: &'m Meter) -> Self {
        Self {
            values: Vec::new(),
            meter,
        }
    }

    pub(super) fn push(&mut self, value: T) -> Result<(), Refusal> {
        self.meter.push_retained(&mut self.values, value)
    }

    pub(super) fn iter(&self) -> std::slice::Iter<'_, T> {
        self.values.iter()
    }

    pub(super) fn len(&self) -> usize {
        self.values.len()
    }

    pub(super) fn as_slice(&self) -> &[T] {
        &self.values
    }

    /// Mutation does not itself charge work; the owning algorithm must do so
    /// before each comparison, scan or move. This never changes storage.
    pub(super) fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.values
    }

    pub(super) fn dedup_by(
        &mut self,
        mut equal: impl FnMut(&T, &T) -> bool,
    ) -> Result<(), Refusal> {
        self.meter.work(self.values.len())?;
        purrdf_lex::walk::try_dedup_by(&mut self.values, |left, right| {
            self.meter.work(1)?;
            Ok(equal(left, right))
        })
    }

    pub(super) fn pop(&mut self) -> Result<Option<T>, Refusal> {
        self.meter.work(1)?;
        Ok(self.values.pop())
    }

    pub(super) fn truncate(&mut self, len: usize) -> Result<(), Refusal> {
        self.meter.work(self.values.len().saturating_sub(len))?;
        self.values.truncate(len);
        Ok(())
    }
}

impl<T> Drop for Scratch<'_, T> {
    fn drop(&mut self) {
        self.meter.release_buffer(std::mem::take(&mut self.values));
    }
}

impl<T> std::ops::Deref for Scratch<'_, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simultaneous_proof_scratch_shares_the_original_grant_until_deallocation() {
        let probe = Meter::new(10, usize::MAX);
        let mut measured = Scratch::new(&probe);
        measured.push(0_usize).unwrap();
        let admitted = probe.live.get();
        drop(measured);
        let meter = Meter::new(10, admitted);
        let mut first = Scratch::new(&meter);
        first.push(1_usize).unwrap();
        let mut second = Scratch::new(&meter);
        assert_eq!(
            second.push(2_usize),
            Err(Refusal::Memory { limit: admitted })
        );
        assert_eq!(meter.live.get(), admitted);
        drop(first);
        assert_eq!(meter.live.get(), 0);
        second.push(2_usize).unwrap();
        drop(second);
        assert_eq!(meter.live.get(), 0);
        // Three push attempts and two native slot writes share this account.
        assert_eq!(meter.work.get(), 5);
        meter.work(meter.work_limit - meter.work.get()).unwrap();
        assert_eq!(meter.work(1), Err(Refusal::Work { limit: 10 }));
    }
}
