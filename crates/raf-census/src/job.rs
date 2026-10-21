// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Jobs: a job names the one place a piece of work lives and what would be a
//! second copy of it; matching finds the units that are one.

use crate::Fingerprint;
use crate::census::Census;
use crate::units::Unit;

/// What a job forbids outside its home.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JobForbidden {
    /// The fingerprints that name a copy: `body`, `shim` and `table` ids. A
    /// `shim` id names the units that group under it, so a method of a trait
    /// implementation and a forwarder that calls through `Self::` never match
    /// it. A `rule` id names no unit in the census and matches none.
    pub fingerprints: Vec<Fingerprint>,
    /// Integer constants, by value, that only the home may spell.
    pub constants: Vec<u128>,
}

/// One job, held in memory. The ledger builds these from its `[[job]]` tables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
    /// The job's id.
    pub id: String,
    /// The symbol of the unit that is the job's home.
    pub home: String,
    /// Symbols of further units that count as the home: the public ways in.
    pub entry_points: Vec<String>,
    /// Symbols of units that may differ on purpose.
    pub variants: Vec<String>,
    /// What the job forbids elsewhere.
    pub forbidden: JobForbidden,
}

impl Job {
    /// A job with no entry points, no variants and nothing forbidden.
    #[must_use]
    pub fn new(id: impl Into<String>, home: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            home: home.into(),
            entry_points: Vec::new(),
            variants: Vec::new(),
            forbidden: JobForbidden::default(),
        }
    }

    /// The same job with `symbols` as entry points.
    #[must_use]
    pub fn entry_points<I, S>(mut self, symbols: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.entry_points = symbols.into_iter().map(Into::into).collect();
        self
    }

    /// The same job with `symbols` as variants.
    #[must_use]
    pub fn variants<I, S>(mut self, symbols: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.variants = symbols.into_iter().map(Into::into).collect();
        self
    }

    /// The same job forbidding `forbidden`.
    #[must_use]
    pub fn forbidding(mut self, forbidden: JobForbidden) -> Self {
        self.forbidden = forbidden;
        self
    }

    fn matches(&self, unit: &Unit) -> bool {
        let forbids = |fingerprint: &Fingerprint| self.forbidden.fingerprints.contains(fingerprint);
        let print = unit.print.as_ref();
        print
            .and_then(|print| print.body.as_ref())
            .is_some_and(forbids)
            || unit.shim_key.as_ref().is_some_and(forbids)
            || unit
                .tables
                .iter()
                .any(|&table| forbids(&Fingerprint::Table(table)))
            || self
                .forbidden
                .constants
                .iter()
                .any(|value| unit.constants.contains(value))
    }

    fn classify(&self, symbol: &str) -> MatchClass {
        if symbol == self.home || self.entry_points.iter().any(|entry| entry == symbol) {
            MatchClass::Home
        } else if self.variants.iter().any(|variant| variant == symbol) {
            MatchClass::Variant
        } else {
            MatchClass::Copy
        }
    }
}

/// How a matching unit stands to its job.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum MatchClass {
    /// The home or one of its entry points.
    Home,
    /// A listed variant.
    Variant,
    /// Anything else: a second copy of the job's work.
    Copy,
}

/// One unit that a job matched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobMatch {
    /// The symbol of the unit.
    pub symbol: String,
    /// The file, relative to the workspace root.
    pub file: String,
    /// The 1-based line of the unit's name.
    pub line: usize,
    /// The byte offset where the source item starts.
    pub start_byte: usize,
    /// How the unit stands to the job.
    pub class: MatchClass,
}

/// What one job found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobReport {
    /// The job's id.
    pub id: String,
    /// The job's home symbol.
    pub home: String,
    /// The matching units, ordered by symbol, then file, then line.
    pub matches: Vec<JobMatch>,
    /// The number of matches classified [`MatchClass::Copy`].
    pub copies: usize,
}

impl Census {
    /// Matches every job against the units: a unit matches when its `body`
    /// fingerprint, its shim key, or the `table:` id of a table it holds, is
    /// among the job's forbidden fingerprints, or one of its integer constants
    /// equals a forbidden constant by value. A shim id matches the units that
    /// group under it: a method of a trait implementation and a forwarder
    /// that calls through `Self::` have no shim key and never match one. One report per job, in the order given.
    #[must_use]
    pub fn match_jobs(&self, jobs: &[Job]) -> Vec<JobReport> {
        jobs.iter()
            .map(|job| {
                let mut matches: Vec<JobMatch> = self
                    .units()
                    .iter()
                    .filter(|unit| job.matches(unit))
                    .map(|unit| JobMatch {
                        symbol: unit.symbol.clone(),
                        file: unit.file.clone(),
                        line: unit.line,
                        start_byte: unit.start_byte,
                        class: job.classify(&unit.symbol),
                    })
                    .collect();
                matches.sort_by(|a, b| {
                    (&a.symbol, &a.file, a.line).cmp(&(&b.symbol, &b.file, b.line))
                });
                let copies = matches
                    .iter()
                    .filter(|found| found.class == MatchClass::Copy)
                    .count();
                JobReport {
                    id: job.id.clone(),
                    home: job.home.clone(),
                    matches,
                    copies,
                }
            })
            .collect()
    }
}
