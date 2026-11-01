// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Complete witness retention under the Gate's original matching grant.

use crate::{Error, Fingerprint, MemberRow, ReferenceWitness, Unit};
use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::allocation::{Admission, Memory, OrderedTable, StorageError};

type Key<'a> = (
    &'a str,
    &'a str,
    usize,
    usize,
    &'a str,
    &'a str,
    usize,
    usize,
    &'a Fingerprint,
);

struct WitnessAdmission<'a> {
    limits: &'a mut MatchLimits,
    live: usize,
    failure: Option<Error>,
}

impl WitnessAdmission<'_> {
    fn charge(&mut self, id: &str, amount: u64) -> Result<(), StorageError> {
        super::super::patterns::charge(id, self.limits, amount).map_err(|error| {
            self.failure = Some(error);
            StorageError::AdmissionFailed
        })
    }

    fn refuse(&mut self, reason: &str) -> StorageError {
        self.failure = Some(super::refusal(reason));
        StorageError::AdmissionFailed
    }

    fn retain_state(&mut self) -> Result<(), StorageError> {
        self.limits.states = self.limits.states.checked_sub(1).ok_or_else(|| {
            self.refuse("matching witness-state limit exhausted; narrow reference sources or raise the explicit matching grant")
        })?;
        Ok(())
    }
}

impl Admission for WitnessAdmission<'_> {
    fn resize(&mut self, live: usize) -> Result<(), StorageError> {
        if live > self.live {
            self.charge(
                "reference-witness-storage",
                (live - self.live).try_into().unwrap_or(u64::MAX),
            )?;
        }
        self.live = live;
        Ok(())
    }
}

fn candidates<'a>(
    consumers: &'a [Unit],
    by_print: &OrderedTable<&Fingerprint, Vec<&'a Unit>>,
    memory: &mut Memory<'_, WitnessAdmission<'_>>,
) -> Result<OrderedTable<Key<'a>, ()>, StorageError> {
    let mut witnessed = OrderedTable::default();
    for consumer in consumers {
        for fingerprint in super::prints(consumer) {
            memory.admission_mut().charge("reference-witness", 1)?;
            if let Some(references) = by_print.get(&fingerprint) {
                for reference in references {
                    let work = [
                        &consumer.symbol,
                        &consumer.file,
                        &reference.symbol,
                        &reference.file,
                    ]
                    .into_iter()
                    .fold(1_u64, |sum, text| {
                        sum.saturating_add(text.len().try_into().unwrap_or(u64::MAX))
                    });
                    memory.admission_mut().charge("reference-witness", work)?;
                    let key = (
                        consumer.symbol.as_str(),
                        consumer.file.as_str(),
                        consumer.line,
                        consumer.start_byte,
                        reference.symbol.as_str(),
                        reference.file.as_str(),
                        reference.line,
                        reference.start_byte,
                        fingerprint,
                    );
                    if witnessed.get(&key).is_none() {
                        memory.admission_mut().retain_state()?;
                        witnessed.insert(key, (), memory)?;
                    }
                }
            }
        }
    }
    Ok(witnessed)
}

fn rows(
    witnessed: &OrderedTable<Key<'_>, ()>,
    memory: &mut Memory<'_, WitnessAdmission<'_>>,
) -> Result<Vec<ReferenceWitness>, StorageError> {
    let mut rows = Vec::new();
    for ((symbol, file, line, start_byte, rsymbol, rfile, rline, rstart, fingerprint), ()) in
        witnessed.ordered()
    {
        let fingerprint = match fingerprint {
            Fingerprint::Body(bytes) => Fingerprint::Body(*bytes),
            Fingerprint::Shim(bytes) => Fingerprint::Shim(*bytes),
            _ => {
                return Err(memory
                    .admission_mut()
                    .refuse("reference witnesses require body or shim fingerprints"));
            }
        };
        let row = ReferenceWitness {
            consumer: MemberRow {
                symbol: memory.string(symbol)?,
                file: memory.string(file)?,
                line: *line,
                start_byte: *start_byte,
            },
            reference: MemberRow {
                symbol: memory.string(rsymbol)?,
                file: memory.string(rfile)?,
                line: *rline,
                start_byte: *rstart,
            },
            fingerprint,
        };
        memory.push(&mut rows, row)?;
    }
    Ok(rows)
}

pub(super) fn collect(
    consumers: &[Unit],
    by_print: &OrderedTable<&Fingerprint, Vec<&Unit>>,
    limits: &mut MatchLimits,
) -> Result<Vec<ReferenceWitness>, Error> {
    if by_print.is_empty() {
        return Ok(Vec::new());
    }
    let mut admission = WitnessAdmission {
        limits,
        live: 0,
        failure: None,
    };
    let result = (|| {
        let mut memory = Memory::new(&mut admission);
        let witnessed = candidates(consumers, by_print, &mut memory)?;
        rows(&witnessed, &mut memory)
    })();
    result.map_err(|source| {
        admission
            .failure
            .take()
            .unwrap_or(Error::SourceIndexStorage { source })
    })
}
