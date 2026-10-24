// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Rule findings on source items that have no structural fingerprint.

use std::collections::{BTreeMap, BTreeSet};

use super::rules::{JobHits, Subject, UnitKey};
use crate::{JobMatch, JobReport, LedgerJob, MatchClass};

pub(super) fn variant(
    variant: &crate::LedgerVariant,
    workspace: &raf_source::Workspace,
    owners: &super::owners::Owners<'_>,
    hits: &JobHits<'_>,
    subjects: &BTreeMap<UnitKey, Subject>,
) -> Result<BTreeSet<UnitKey>, String> {
    let resolution = workspace
        .resolve(&variant.symbol)
        .map_err(|error| format!("stale ledger path {:?}: {error}", variant.symbol))?;
    let file = raf_source::RelPath::new(&variant.file).map_err(|error| error.to_string())?;
    let mut keys = BTreeSet::new();
    for site in resolution.sites {
        let item = workspace
            .item(site)
            .ok_or("variant source item is absent")?;
        if item.file != file {
            continue;
        }
        let physical = (item.file.clone(), item.span.start_byte);
        let key = owners
            .get(&physical)
            .filter(|_| !workspace.same_source(item).is_empty())
            .map(|unit| super::owners::key(unit))
            .or_else(|| {
                subjects
                    .iter()
                    .find(|(_, subject)| {
                        subject.sites.contains(&site)
                            && subject.source_file == physical.0
                            && subject.start_byte == physical.1
                    })
                    .map(|(key, _)| key.clone())
            })
            .ok_or_else(|| {
                format!(
                    "{} no longer matches detector {:?}",
                    variant.symbol, variant.detector
                )
            })?;
        if !hits
            .get(&variant.detector)
            .is_some_and(|matches| matches.contains_key(&key))
        {
            return Err(format!(
                "{} no longer matches detector {:?}",
                variant.symbol, variant.detector
            ));
        }
        keys.insert(key);
    }
    if keys.is_empty() {
        return Err(format!(
            "{} does not exist in {}",
            variant.symbol, variant.file
        ));
    }
    Ok(keys)
}

pub(super) fn append(
    row: &LedgerJob,
    hits: &JobHits<'_>,
    subjects: &BTreeMap<UnitKey, Subject>,
    home_packages: &BTreeSet<String>,
    home_sources: &BTreeSet<raf_source::ItemId>,
    variants: &BTreeSet<UnitKey>,
    report: &mut JobReport,
) {
    for (key, subject) in subjects {
        let selected = super::rules::referenced(row).any(|id| {
            (!crate::detectors::home_exempt(id) || !home_packages.contains(&subject.package))
                && hits
                    .get(id)
                    .is_some_and(|matches| matches.contains_key(key))
        });
        if selected {
            let class = if !subject.sites.is_disjoint(home_sources) {
                MatchClass::Home
            } else if variants.contains(key) {
                MatchClass::Variant
            } else {
                MatchClass::Copy
            };
            report.copies += usize::from(class == MatchClass::Copy);
            report.matches.push(JobMatch {
                symbol: subject.symbol.clone(),
                file: subject.file.clone(),
                start_byte: subject.start_byte,
                line: subject.line,
                class,
            });
        }
    }
    report
        .matches
        .sort_by(|a, b| (&a.symbol, &a.file, a.line).cmp(&(&b.symbol, &b.file, b.line)));
}
