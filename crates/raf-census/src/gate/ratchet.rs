// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The counted sites of ratcheted patterns: hits are diverted from findings,
//! keyed by their enclosing symbol and first string literal, and counted.

use std::collections::{BTreeMap, BTreeSet};

use purrdf_jsonschema::ecma::MatchLimits;
use raf_source::{RelPath, Workspace};

use super::patterns::string_literals;
use super::rules::{Offsets, UnitKey, refused};
use crate::{DebtSite, Error, Ledger, SiteKey};

/// The (rule, owning unit, byte) of every diverted hit, with its line. The set
/// deduplicates a hit that two jobs reach through one shared pattern.
type Hits = BTreeMap<(String, UnitKey, usize), usize>;

/// The ratcheted pattern ids of a ledger and the hits diverted from findings.
pub(super) struct Sink<'a> {
    rules: BTreeSet<&'a str>,
    offsets: &'a Offsets,
    hits: Hits,
}

impl<'a> Sink<'a> {
    pub(super) fn new(ledger: &'a Ledger, offsets: &'a Offsets) -> Self {
        Self {
            rules: ledger
                .patterns()
                .iter()
                .filter(|pattern| pattern.ratcheted)
                .map(|pattern| pattern.id.as_str())
                .collect(),
            offsets,
            hits: Hits::new(),
        }
    }

    pub(super) fn is_ratcheted(&self, id: &str) -> bool {
        self.rules.contains(id)
    }

    pub(super) fn rules(&self) -> impl Iterator<Item = &str> {
        self.rules.iter().copied()
    }

    /// Divert every hit of the ratcheted rule `id` in the unit `unit`.
    pub(super) fn divert(&mut self, id: &str, unit: &UnitKey) {
        for (byte, line) in self
            .offsets
            .get(id)
            .and_then(|units| units.get(unit))
            .into_iter()
            .flatten()
        {
            self.hits
                .insert((id.to_owned(), unit.clone(), *byte), *line);
        }
    }

    /// Count the diverted hits by (file, rule, site key).
    pub(super) fn count(
        &self,
        workspace: &Workspace,
        ledger: &Ledger,
        limits: &mut MatchLimits,
    ) -> Result<Counted, Error> {
        if self.hits.is_empty() {
            return Ok(Counted::default());
        }
        let summaries: BTreeMap<&str, &str> = ledger
            .patterns()
            .iter()
            .map(|pattern| (pattern.id.as_str(), pattern.summary.as_str()))
            .collect();
        crate::units::on_thread(|| count_sites(workspace, &self.hits, &summaries, limits))?
    }
}

/// Counted sites and, for each, the locations of its hits.
#[derive(Default)]
pub(super) struct Counted {
    pub(super) counts: BTreeMap<DebtSite, usize>,
    pub(super) locations: BTreeMap<DebtSite, Vec<String>>,
}

fn count_sites(
    workspace: &Workspace,
    hits: &Hits,
    summaries: &BTreeMap<&str, &str>,
    limits: &mut MatchLimits,
) -> Result<Counted, Error> {
    let mut by_file: BTreeMap<&str, Vec<(&str, &str, usize, usize)>> = BTreeMap::new();
    for ((rule, (symbol, file, _), byte), line) in hits {
        by_file
            .entry(file)
            .or_default()
            .push((rule, symbol, *byte, *line));
    }
    let mut counted = Counted::default();
    for (file, rows) in by_file {
        let path = RelPath::new(file).map_err(|error| refused("ratchet", &error.to_string()))?;
        let text = workspace
            .source(&path)
            .ok_or_else(|| refused("ratchet", &format!("{file}: source text is absent")))?;
        let literals = string_literals("ratchet", file, text, limits)?;
        for (rule, symbol, byte, line) in rows {
            let end = workspace
                .item_at(&path, byte)
                .iter()
                .map(|item| item.span.end_byte)
                .max()
                .ok_or_else(|| {
                    refused(rule, &format!("{file}:{line}: hit lies in no source item"))
                })?;
            let first = literals.partition_point(|literal| literal.start < byte);
            let literal = literals
                .get(first)
                .filter(|literal| literal.start < end)
                .map(|literal| literal.value.as_str());
            let site = DebtSite::new(file, rule, SiteKey::digest(symbol, literal)?)?;
            *counted.counts.entry(site.clone()).or_default() += 1;
            counted.locations.entry(site).or_default().push(format!(
                "{file}:{line}: {symbol}: {}",
                summaries.get(rule).copied().unwrap_or_default()
            ));
        }
    }
    Ok(counted)
}
