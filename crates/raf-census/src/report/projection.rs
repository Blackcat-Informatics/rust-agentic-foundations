// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Borrowed evidence shared by live and decoded report projections.

use purrdf_lex::allocation::{Memory, OrderedTable, Resident};

use super::{Index, IndexUnit, JobRow, MatchRow, MemberRow, ReferenceSource, ReferenceWitness};
use super::{ReportError as E, mint};
use crate::{Census, Fingerprint, JobMatch, JobReport, MatchClass, NORMAL_FORM, Unit};

pub(super) type Site<'a> = (&'a str, &'a str, usize, usize);
pub(super) type Sites<'a> = OrderedTable<Site<'a>, Option<UnitRef<'a>>>;

pub(super) fn site_id((symbol, file, line, start): Site<'_>) -> String {
    mint::unit(symbol, file, line, start)
}

pub(super) fn member_site(member: &MemberRow) -> Site<'_> {
    (&member.symbol, &member.file, member.line, member.start_byte)
}

#[derive(Clone, Copy)]
pub(super) enum UnitRef<'a> {
    Live(&'a Unit),
    Indexed(&'a IndexUnit),
}

impl<'a> UnitRef<'a> {
    pub(super) fn site(self) -> Site<'a> {
        match self {
            Self::Live(unit) => (&unit.symbol, &unit.file, unit.line, unit.start_byte),
            Self::Indexed(unit) => (&unit.symbol, &unit.file, unit.line, unit.start_byte),
        }
    }

    pub(super) fn source(self) -> Option<&'a str> {
        match self {
            Self::Live(_) => None,
            Self::Indexed(unit) => unit.reference_source.as_deref(),
        }
    }

    pub(super) fn fields(
        self,
    ) -> (
        &'a str,
        bool,
        usize,
        Option<&'a Fingerprint>,
        Option<&'a Fingerprint>,
    ) {
        match self {
            Self::Live(unit) => (
                &unit.package,
                unit.shipping,
                unit.print.as_ref().map_or(0, |print| print.tokens),
                unit.print.as_ref().and_then(|print| print.body.as_ref()),
                unit.shim_key.as_ref(),
            ),
            Self::Indexed(unit) => (
                &unit.package,
                unit.shipping,
                unit.tokens,
                unit.body.as_ref(),
                unit.shim.as_ref(),
            ),
        }
    }
}

pub(super) struct Evidence<'a> {
    pub(super) normal_form: u32,
    pub(super) sources: &'a [ReferenceSource],
    pub(super) witnesses: &'a [ReferenceWitness],
    live: &'a [Unit],
    indexed: &'a [IndexUnit],
}

impl<'a> From<&'a Census> for Evidence<'a> {
    fn from(census: &'a Census) -> Self {
        Self {
            normal_form: NORMAL_FORM,
            sources: &census.reference_sources,
            witnesses: &census.reference_witnesses,
            live: census.units(),
            indexed: &census.reference_units,
        }
    }
}

impl<'a> From<&'a Index> for Evidence<'a> {
    fn from(index: &'a Index) -> Self {
        Self {
            normal_form: index.normal_form,
            sources: &index.reference_sources,
            witnesses: &index.reference_witnesses,
            live: &[],
            indexed: &index.units,
        }
    }
}

impl<'a> Evidence<'a> {
    pub(super) fn units(&self) -> impl Iterator<Item = UnitRef<'a>> {
        self.live
            .iter()
            .map(UnitRef::Live)
            .chain(self.indexed.iter().map(UnitRef::Indexed))
    }

    pub(super) fn locate(&self, sites: &mut Sites<'a>) -> Result<(), E> {
        if sites.is_empty() {
            return Ok(());
        }
        for unit in self.units() {
            let site = unit.site();
            if let Some(selected) = sites.get_mut_by(|known| site.cmp(known))
                && selected.replace(unit).is_some()
            {
                return Err(E::Repeated(site_id(site)));
            }
        }
        for (site, unit) in sites.iter() {
            required_unit(*site, *unit)?;
        }
        Ok(())
    }
}

pub(super) fn required_unit<'a>(
    site: Site<'a>,
    unit: Option<UnitRef<'a>>,
) -> Result<UnitRef<'a>, E> {
    unit.ok_or_else(|| E::Dangling(site_id(site)))
}

pub(super) struct JobRef<'a> {
    pub(super) id: &'a str,
    pub(super) home: &'a str,
    pub(super) copies: usize,
    live: &'a [JobMatch],
    indexed: &'a [MatchRow],
}

impl<'a> From<&'a JobReport> for JobRef<'a> {
    fn from(job: &'a JobReport) -> Self {
        Self {
            id: &job.id,
            home: &job.home,
            copies: job.copies,
            live: &job.matches,
            indexed: &[],
        }
    }
}

impl<'a> From<&'a JobRow> for JobRef<'a> {
    fn from(job: &'a JobRow) -> Self {
        Self {
            id: &job.id,
            home: &job.home,
            copies: job.copies,
            live: &[],
            indexed: &job.matches,
        }
    }
}

impl<'a> JobRef<'a> {
    pub(super) fn matches(&self) -> impl Iterator<Item = (Site<'a>, MatchClass)> {
        self.live
            .iter()
            .map(|row| {
                (
                    (
                        row.symbol.as_str(),
                        row.file.as_str(),
                        row.line,
                        row.start_byte,
                    ),
                    row.class,
                )
            })
            .chain(self.indexed.iter().map(|row| {
                (
                    (
                        row.symbol.as_str(),
                        row.file.as_str(),
                        row.line,
                        row.start_byte,
                    ),
                    row.class,
                )
            }))
    }

    pub(super) fn selected(&self, evidence: &Evidence<'a>) -> Result<Sites<'a>, crate::Error> {
        let mut sites = Sites::default();
        let mut resident = Resident;
        let mut memory = Memory::new(&mut resident);
        let mut copies = 0;
        for (site, class) in self.matches() {
            copies += usize::from(class == MatchClass::Copy);
            if sites
                .insert(site, None, &mut memory)
                .map_err(|source| crate::Error::QueryStorage { source })?
                .is_some()
            {
                return Err(E::Repeated(site_id(site)).into());
            }
        }
        evidence.locate(&mut sites)?;
        if self.copies != copies {
            return Err(E::Inconsistent(format!("job {}: copies", self.id)).into());
        }
        Ok(sites)
    }
}
