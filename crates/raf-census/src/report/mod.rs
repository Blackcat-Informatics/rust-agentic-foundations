// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Canonical ground RDF 1.2 reports with a strict closed-shape reader.

mod debt;
mod error;
mod ledger;
pub(crate) mod mint;
mod query;
mod read;
pub(crate) mod reference;
pub(crate) mod shape;
mod upgrade;
/// The report vocabulary and closed shapes.
pub mod vocab;
pub(crate) mod write;

pub use debt::{DebtChange, DebtGroup, DebtRecord, MAX_DEBT_BYTES};
pub use error::ReportError;
pub use ledger::{
    Ledger, LedgerDistinct, LedgerExtensionGraph, LedgerJob, LedgerPattern, LedgerVariant,
    MAX_LEDGER_BYTES, NamePattern, PatternMatcher, PreferredProvider,
};
pub use query::{Query, QueryReport};
pub use reference::{ReferenceSource, ReferenceWitness};
pub use upgrade::{ReportKind, upgrade_report_form};
pub use vocab::REPORT_FORM;

use crate::census::Census;
use crate::group::{Group, GroupKind, Member};
use crate::job::{JobReport, MatchClass};
use crate::{Error, NORMAL_FORM};

/// Where a unit is, as a report row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberRow {
    /// The symbol of the unit.
    pub symbol: String,
    /// The file, relative to the workspace root.
    pub file: String,
    /// The 1-based line of the unit's name.
    pub line: usize,
    /// The byte offset of the source item.
    pub start_byte: usize,
}

impl From<&Member> for MemberRow {
    fn from(member: &Member) -> Self {
        Self {
            symbol: member.symbol.clone(),
            file: member.file.clone(),
            line: member.line,
            start_byte: member.start_byte,
        }
    }
}

/// A group of the baseline report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaselineGroup {
    /// The shared fingerprint.
    pub fingerprint: crate::Fingerprint,
    /// The tokens of the value form of the shared body.
    pub tokens: usize,
    /// The units, ordered by symbol, then file, then line.
    pub members: Vec<MemberRow>,
}

/// A unit that holds a hexadecimal digit table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableRow {
    /// The symbol of the unit.
    pub symbol: String,
    /// The file, relative to the workspace root.
    pub file: String,
    /// The 1-based line of the unit's name.
    pub line: usize,
    /// The byte offset of the source item.
    pub start_byte: usize,
    /// The table's name after `table:`.
    pub table: crate::TableKind,
}

/// An integer constant above `0xffff` held by units of two or more packages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstantRow {
    /// The integer value.
    pub value: u128,
    /// The number of packages that hold it.
    pub packages: usize,
    /// The units that hold it.
    pub members: Vec<MemberRow>,
}

/// The baseline report: what the workspace repeats.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Baseline {
    /// The normal form the fingerprints belong to.
    pub normal_form: u32,
    /// The value-token floor of a body fingerprint.
    pub min_tokens: usize,
    /// The number of units.
    pub units: usize,
    /// The body groups.
    pub body_groups: Vec<BaselineGroup>,
    /// The shim groups.
    pub shim_groups: Vec<BaselineGroup>,
    /// The hexadecimal digit tables.
    pub hex_tables: Vec<TableRow>,
    /// The large constants shared across packages.
    pub shared_constants: Vec<ConstantRow>,
}

/// One unit of the index report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexUnit {
    /// Reference receipt for an external comparison unit; `None` denotes an
    /// ordinary first-party row in this index, without external source proof.
    pub reference_source: Option<String>,
    /// The symbol of the unit.
    pub symbol: String,
    /// The package of the unit.
    pub package: String,
    /// The file, relative to the workspace root.
    pub file: String,
    /// The 1-based line of the unit's name.
    pub line: usize,
    /// The byte offset of the source item.
    pub start_byte: usize,
    /// Whether the unit ships.
    pub shipping: bool,
    /// The tokens of the value form of the body; 0 for a unit that is not a function.
    pub tokens: usize,
    /// The `body` fingerprint, when the body reaches `min_tokens`.
    pub body: Option<crate::Fingerprint>,
    /// The `shim` fingerprint the unit groups under; none for a unit that is
    /// not an eligible forwarder.
    pub shim: Option<crate::Fingerprint>,
}

/// A group of the index report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexGroup {
    /// The value-form token count.
    pub tokens: usize,
    /// Whether the units share a body or a shim.
    pub kind: GroupKind,
    /// The shared fingerprint.
    pub fingerprint: crate::Fingerprint,
    /// The units, ordered by symbol, then file, then line.
    pub members: Vec<MemberRow>,
}

/// One match of a job.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchRow {
    /// The symbol of the unit.
    pub symbol: String,
    /// How the unit stands to the job.
    pub class: MatchClass,
    /// The source file.
    pub file: String,
    /// The one-based source line.
    pub line: usize,
    /// The byte offset of the source item.
    pub start_byte: usize,
}

/// One job of the index report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobRow {
    /// The job's id.
    pub id: String,
    /// The job's home symbol.
    pub home: String,
    /// The matching units.
    pub matches: Vec<MatchRow>,
    /// The number of matches that are copies.
    pub copies: usize,
}

/// The index report: every unit with its fingerprints, every group, every job.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Index {
    /// The normal form the fingerprints belong to.
    pub normal_form: u32,
    /// Exact dependency identity and observation kind for reference units.
    pub reference_sources: Vec<ReferenceSource>,
    /// Every full-fingerprint cross-boundary match, independently of job counts.
    pub reference_witnesses: Vec<ReferenceWitness>,
    /// Every unit, ordered by symbol, then file, then line.
    pub units: Vec<IndexUnit>,
    /// Every group, in the order of [`Census::groups`].
    pub groups: Vec<IndexGroup>,
    /// The jobs the caller matched, sorted by id.
    pub jobs: Vec<JobRow>,
}

impl Baseline {
    /// The canonical N-Triples 1.2 spelling, sharing the Turtle snapshot identity.
    ///
    /// # Errors
    /// Refuses values outside the closed census shape.
    pub fn to_ntriples(&self) -> Result<String, Error> {
        write::baseline(self, raf_rdf::Format::NTriples)
    }
    /// Read exactly the closed canonical N-Triples report profile.
    ///
    /// # Errors
    /// Refuses invalid terms, shapes, identities and noncanonical bytes.
    pub fn from_ntriples(text: &str) -> Result<Self, Error> {
        read::baseline(text, raf_rdf::Format::NTriples)
    }
    /// The report's one canonical Turtle 1.2 spelling.
    ///
    /// # Errors
    /// Returns a typed report error for a value outside the closed shape.
    pub fn to_turtle(&self) -> Result<String, Error> {
        write::baseline(self, raf_rdf::Format::Turtle)
    }
    /// Reads exactly the canonical report, preserving its declared normal form.
    ///
    /// # Errors
    /// Refuses unknown terms, invalid shapes, noncanonical spelling and digest edits.
    pub fn from_turtle(text: &str) -> Result<Self, Error> {
        read::baseline(text, raf_rdf::Format::Turtle)
    }
}
impl Index {
    /// The canonical N-Triples 1.2 spelling, sharing the Turtle snapshot identity.
    ///
    /// # Errors
    /// Refuses values outside the closed census shape.
    pub fn to_ntriples(&self) -> Result<String, Error> {
        write::index(self, raf_rdf::Format::NTriples)
    }
    /// Read exactly the closed canonical N-Triples report profile.
    ///
    /// # Errors
    /// Refuses invalid terms, shapes, identities and noncanonical bytes.
    pub fn from_ntriples(text: &str) -> Result<Self, Error> {
        read::index(text, raf_rdf::Format::NTriples)
    }
    /// The report's one canonical Turtle 1.2 spelling.
    ///
    /// # Errors
    /// Returns a typed report error for a value outside the closed shape.
    pub fn to_turtle(&self) -> Result<String, Error> {
        write::index(self, raf_rdf::Format::Turtle)
    }
    /// Reads exactly the canonical report, preserving its declared normal form.
    ///
    /// # Errors
    /// Refuses unknown terms, invalid shapes, noncanonical spelling and digest edits.
    pub fn from_turtle(text: &str) -> Result<Self, Error> {
        read::index(text, raf_rdf::Format::Turtle)
    }
}

fn rows(members: &[Member]) -> Vec<MemberRow> {
    members.iter().map(MemberRow::from).collect()
}

fn baseline_group(group: &Group) -> BaselineGroup {
    BaselineGroup {
        fingerprint: group.fingerprint.clone(),
        tokens: group.tokens,
        members: rows(&group.members),
    }
}

impl Census {
    /// The baseline report of the loaded workspace.
    #[must_use]
    pub fn baseline(&self) -> Baseline {
        Baseline {
            normal_form: NORMAL_FORM,
            min_tokens: self.options().min_tokens,
            units: self.units().len(),
            body_groups: self.body_groups().map(baseline_group).collect(),
            shim_groups: self.shim_groups().map(baseline_group).collect(),
            hex_tables: self
                .hex_tables()
                .iter()
                .map(|t| TableRow {
                    symbol: t.member.symbol.clone(),
                    file: t.member.file.clone(),
                    line: t.member.line,
                    table: t.table,
                    start_byte: t.member.start_byte,
                })
                .collect(),
            shared_constants: self
                .shared_constants()
                .iter()
                .map(|c| ConstantRow {
                    value: c.value,
                    packages: c.packages,
                    members: rows(&c.members),
                })
                .collect(),
        }
    }

    /// The index report of the loaded workspace, with the jobs the caller
    /// matched through [`Census::match_jobs`].
    ///
    /// # Errors
    /// Refuses duplicate job ids.
    pub fn index(&self, jobs: &[JobReport]) -> Result<Index, Error> {
        let mut units: Vec<IndexUnit> = self
            .units()
            .iter()
            .map(|unit| IndexUnit {
                reference_source: None,
                symbol: unit.symbol.clone(),
                package: unit.package.clone(),
                file: unit.file.clone(),
                line: unit.line,
                start_byte: unit.start_byte,
                shipping: unit.shipping,
                tokens: unit.print.as_ref().map_or(0, |print| print.tokens),
                body: unit
                    .print
                    .as_ref()
                    .and_then(|print| print.body.as_ref())
                    .cloned(),
                shim: unit.shim_key.clone(),
            })
            .collect();
        units.extend(self.reference_units.iter().cloned());
        units.sort_by(|a, b| (&a.symbol, &a.file, a.line).cmp(&(&b.symbol, &b.file, b.line)));
        let mut report = Index {
            normal_form: NORMAL_FORM,
            reference_sources: self.reference_sources.clone(),
            reference_witnesses: self.reference_witnesses.clone(),
            units,
            groups: self
                .groups()
                .iter()
                .map(|group| IndexGroup {
                    tokens: group.tokens,
                    kind: group.kind,
                    fingerprint: group.fingerprint.clone(),
                    members: rows(&group.members),
                })
                .collect(),
            jobs: jobs
                .iter()
                .map(|job| JobRow {
                    id: job.id.clone(),
                    home: job.home.clone(),
                    matches: job
                        .matches
                        .iter()
                        .map(|found| MatchRow {
                            symbol: found.symbol.clone(),
                            file: found.file.clone(),
                            line: found.line,
                            start_byte: found.start_byte,
                            class: found.class,
                        })
                        .collect(),
                    copies: job.copies,
                })
                .collect(),
        };
        report.jobs.sort_by(|a, b| a.id.cmp(&b.id));
        if let Some(pair) = report.jobs.windows(2).find(|pair| pair[0].id == pair[1].id) {
            return Err(ReportError::DuplicateJob(pair[0].id.clone()).into());
        }
        Ok(report)
    }
}
