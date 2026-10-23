// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Ledger resolution, staleness, extended matching and exact present debt.

use std::collections::{BTreeMap, BTreeSet};

use purrdf_jsonschema::ecma::MatchLimits;
use raf_source::{RelPath, Workspace};

use super::{GateReport, LedgerExtension, SYMBOL_ANCHOR_PREFIX, owners, rules};
use crate::{
    Census, DebtGroup, DebtRecord, Error, Fingerprint, Job, JobReport, Ledger, LedgerJob,
    MatchClass, NORMAL_FORM, Rule, Unit,
};

pub(super) fn evaluate(
    ledger: &Ledger,
    workspace: &Workspace,
    census: Census,
    registered: &[Box<dyn Rule>],
    extensions: &[Box<dyn LedgerExtension>],
) -> Result<GateReport, Error> {
    let owners = owners::index(&census);
    let collected = rules::collect(workspace, &census, registered, &owners, ledger)?;
    let hits = &collected.hits;
    let mut findings = Vec::new();
    let mut reports = Vec::new();
    let mut copies = BTreeMap::new();
    let mut limits = MatchLimits::default();
    for row in ledger.jobs() {
        validate_dependencies(row, hits)?;
        extension(ledger, row, extensions)?;
        stale_fingerprints(&census, row, &mut findings);
        let Some(resolved) = resolve_job(workspace, &owners, row, &mut findings) else {
            continue;
        };
        let hits = super::context::collect(hits, &census, row, &resolved, &mut limits)?;
        let ResolvedJob {
            mut job,
            homes,
            home_packages,
            ..
        } = resolved;
        let variants = resolve_variants(
            workspace,
            &owners,
            row,
            &hits,
            &collected.subjects,
            &mut findings,
        )?;
        // Legacy in-memory jobs classify symbol-wide variants. The ledger's
        // narrower sanction must include the declared source file as well.
        job.variants.clear();
        let mut matched = Vec::new();
        for unit in census.units() {
            let selected = job.matches(unit) || hits.names.contains(&owners::key(unit));
            let rule_match = row.job.forbidden.fingerprints.iter().any(|fingerprint| {
                if let Fingerprint::Rule(id) = fingerprint {
                    has_hit(&hits, id.as_str(), unit)
                        && (!crate::detectors::home_exempt(id.as_str())
                            || !home_packages.contains(unit.package.as_str()))
                } else {
                    false
                }
            });
            if selected || rule_match {
                matched.push(unit);
            }
        }
        let mut report = JobReport::collect(&job, matched, |unit| {
            if homes.contains(&owners::key(unit)) {
                MatchClass::Home
            } else if variants.contains(&owners::key(unit)) {
                MatchClass::Variant
            } else {
                MatchClass::Copy
            }
        });
        super::subjects::append(
            row,
            &hits,
            &collected.subjects,
            &home_packages,
            &variants,
            &mut report,
        );
        report_copies(row, &report, &hits, &owners, &home_packages, &mut findings);
        if !row.enforced && report.copies > 0 {
            copies.insert(job.id.clone(), report.copies);
        }
        reports.push(report);
    }
    let sanctioned = resolve_distinct(ledger, workspace, &owners, &mut findings)?;
    let groups = census
        .groups()
        .iter()
        .map(|group| {
            let mut counts = BTreeMap::new();
            for symbol in group.member_symbols() {
                *counts.entry(symbol.to_owned()).or_default() += 1;
            }
            DebtGroup::with_counts(group.kind, counts)
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    for row in &sanctioned {
        if !groups.contains(row) {
            findings.push(format!(
                "distinct {:?} {:?} equals no current group",
                row.kind(),
                row.members()
            ));
        }
    }
    let debt = DebtRecord::new(NORMAL_FORM, groups.difference(&sanctioned).cloned(), copies)?;
    Ok(GateReport {
        census,
        jobs: reports,
        findings,
        debt,
    })
}

fn resolve_distinct(
    ledger: &Ledger,
    workspace: &Workspace,
    owners: &owners::Owners<'_>,
    findings: &mut Vec<String>,
) -> Result<BTreeSet<DebtGroup>, Error> {
    let mut sanctioned = BTreeSet::new();
    for row in ledger.distinct() {
        documented(workspace, &row.anchor, findings);
        let mut counts = BTreeMap::new();
        let mut physical = BTreeSet::new();
        let mut valid = true;
        for (symbol, expected) in row.group.members() {
            let Some(units) = resolve(workspace, owners, symbol, findings) else {
                valid = false;
                continue;
            };
            if units.len() != *expected {
                findings.push(format!(
                    "distinct member {symbol:?}: declared {expected} source units, found {}",
                    units.len()
                ));
                valid = false;
            }
            for unit in units {
                if !physical.insert(owners::key(unit)) {
                    findings.push(format!("distinct member {symbol:?} repeats one physical source unit through an alias"));
                    valid = false;
                }
                *counts.entry(unit.symbol.clone()).or_default() += 1;
            }
        }
        if valid {
            sanctioned.insert(DebtGroup::with_counts(row.group.kind(), counts)?);
        }
    }
    Ok(sanctioned)
}

fn resolve_variants(
    workspace: &Workspace,
    owners: &owners::Owners<'_>,
    row: &LedgerJob,
    hits: &rules::JobHits<'_>,
    subjects: &BTreeMap<rules::UnitKey, rules::Subject>,
    findings: &mut Vec<String>,
) -> Result<BTreeSet<rules::UnitKey>, Error> {
    let mut variants = BTreeSet::new();
    let mut probe = row.job.clone();
    probe.forbidden.constants.clear();
    probe
        .forbidden
        .fingerprints
        .retain(|fingerprint| matches!(fingerprint, Fingerprint::Body(_) | Fingerprint::Shim(_)));
    for variant in &row.variants {
        documented(workspace, &variant.anchor, findings);
        if variant.detector != "isomorphic" {
            match super::subjects::variant(variant, workspace, owners, hits, subjects) {
                Ok(keys) => variants.extend(keys),
                Err(error) => findings.push(format!("job {:?} variant: {error}", row.job.id)),
            }
            continue;
        }
        let Some(resolved) = resolve(workspace, owners, &variant.symbol, findings) else {
            continue;
        };
        let file = RelPath::new(&variant.file)
            .map_err(|error| policy(&format!("job {:?} variant file: {error}", row.job.id)))?
            .to_string();
        let units: Vec<_> = resolved
            .into_iter()
            .filter(|unit| unit.file == file)
            .collect();
        if units.is_empty() {
            findings.push(format!(
                "job {:?}: variant {} does not exist in {file}",
                row.job.id, variant.symbol
            ));
        }
        for unit in units {
            let selected = if variant.detector == "isomorphic" {
                probe.matches(unit)
            } else {
                has_hit(hits, &variant.detector, unit)
            };
            if selected {
                variants.insert(owners::key(unit));
            } else {
                findings.push(format!(
                    "job {:?}: variant {} in {file} at byte {} no longer matches detector {:?}",
                    row.job.id, variant.symbol, unit.start_byte, variant.detector
                ));
            }
        }
    }
    Ok(variants)
}

fn report_copies(
    row: &LedgerJob,
    report: &JobReport,
    hits: &rules::JobHits<'_>,
    owners: &owners::Owners<'_>,
    home_packages: &BTreeSet<String>,
    findings: &mut Vec<String>,
) {
    let job = &row.job;
    if !row.enforced && report.copies == 0 {
        findings.push(format!(
            "job {:?} has no copies; set enforced to true",
            job.id
        ));
    }
    for found in report
        .matches
        .iter()
        .filter(|found| found.class == MatchClass::Copy)
    {
        let key = (found.symbol.clone(), found.file.clone(), found.start_byte);
        let mut rule_copy = false;
        for fingerprint in &job.forbidden.fingerprints {
            if let Fingerprint::Rule(id) = fingerprint
                && crate::detectors::home_exempt(id.as_str())
                && owners
                    .get(&(found.file.clone(), found.start_byte))
                    .is_some_and(|unit| home_packages.contains(unit.package.as_str()))
            {
                continue;
            }
            if let Fingerprint::Rule(id) = fingerprint
                && let Some(matches) = hits.get(id.as_str()).and_then(|units| units.get(&key))
            {
                rule_copy = true;
                for hit in matches {
                    findings.push(format!(
                        "{}:{}: job {:?}, rule {}: {}; use {}",
                        found.file,
                        hit.line,
                        job.id,
                        id.as_str(),
                        hit.message,
                        job.home
                    ));
                }
            }
        }
        if row.enforced && !rule_copy {
            findings.push(format!(
                "{}:{}: job {:?}: {} is a COPY; use {}",
                found.file, found.line, job.id, found.symbol, job.home
            ));
        }
    }
}

fn resolve<'a>(
    workspace: &Workspace,
    owners: &owners::Owners<'a>,
    symbol: &str,
    findings: &mut Vec<String>,
) -> Option<Vec<&'a Unit>> {
    match owners::resolve(workspace, owners, symbol) {
        Ok(units) => Some(units),
        Err(error) => {
            findings.push(format!("stale ledger path {symbol:?}: {error}"));
            None
        }
    }
}

pub(super) struct ResolvedJob {
    pub(super) job: Job,
    pub(super) homes: BTreeSet<rules::UnitKey>,
    pub(super) home_units: BTreeSet<rules::UnitKey>,
    pub(super) home_packages: BTreeSet<String>,
}

fn resolve_home<'a>(
    workspace: &Workspace,
    owners: &owners::Owners<'a>,
    symbol: &str,
    findings: &mut Vec<String>,
) -> Option<owners::Home<'a>> {
    match owners::home(workspace, owners, symbol) {
        Ok(home) => Some(home),
        Err(error) => {
            findings.push(format!("stale ledger path {symbol:?}: {error}"));
            None
        }
    }
}

fn resolve_job(
    workspace: &Workspace,
    owners: &owners::Owners<'_>,
    row: &LedgerJob,
    findings: &mut Vec<String>,
) -> Option<ResolvedJob> {
    let home = resolve_home(workspace, owners, &row.job.home, findings);
    let entries = row
        .job
        .entry_points
        .iter()
        .map(|entry| resolve_home(workspace, owners, entry, findings))
        .collect::<Vec<_>>()
        .into_iter()
        .collect::<Option<Vec<_>>>();
    match (home, entries) {
        (Some(home), Some(entries)) => {
            let mut job = row.job.clone();
            job.home = home.path;
            job.entry_points = entries.iter().map(|entry| entry.path.clone()).collect();
            let home_units = home.units.iter().copied().map(owners::key).collect();
            let homes = home
                .units
                .into_iter()
                .chain(entries.into_iter().flat_map(|entry| entry.units))
                .map(owners::key)
                .collect();
            Some(ResolvedJob {
                job,
                homes,
                home_packages: home.packages,
                home_units,
            })
        }
        _ => None,
    }
}

fn documented(workspace: &Workspace, anchor: &str, findings: &mut Vec<String>) {
    let Some(encoded) = anchor.strip_prefix(SYMBOL_ANCHOR_PREFIX) else {
        findings.push(format!("anchor {anchor:?}: use {SYMBOL_ANCHOR_PREFIX} followed by an encoded documented source path"));
        return;
    };
    let Ok(symbol) = purrdf_lex::percent::decode(encoded) else {
        findings.push(format!("anchor {anchor:?}: invalid encoded source path"));
        return;
    };
    match workspace.resolve(&symbol) {
        Ok(resolution)
            if resolution
                .sites
                .iter()
                .all(|id| workspace.item(*id).is_some_and(|item| item.documented)) => {}
        _ => findings.push(format!(
            "anchor {anchor:?} does not resolve to a documented source item"
        )),
    }
}

fn stale_fingerprints(census: &Census, row: &LedgerJob, findings: &mut Vec<String>) {
    for fingerprint in &row.job.forbidden.fingerprints {
        if matches!(fingerprint, Fingerprint::Body(_) | Fingerprint::Shim(_)) {
            let mut probe = Job::new("fingerprint", "");
            probe.forbidden.fingerprints.push(fingerprint.clone());
            if !census.units().iter().any(|unit| probe.matches(unit)) {
                findings.push(format!(
                    "job {:?}: fingerprint {fingerprint} matches no unit",
                    row.job.id
                ));
            }
        }
    }
}

fn has_hit(hits: &rules::JobHits<'_>, id: &str, unit: &Unit) -> bool {
    hits.get(id).is_some_and(|units| {
        units.contains_key(&(unit.symbol.clone(), unit.file.clone(), unit.start_byte))
    })
}

fn validate_dependencies(row: &LedgerJob, hits: &rules::Hits) -> Result<(), Error> {
    if let Some(provider) = &row.preferred_provider {
        return Err(policy(&format!(
            "job {:?}: preferred provider {:?} requires grounded locked-source and delegation execution; a declaration is not a clean verdict",
            row.job.id, provider.package
        )));
    }
    if let Some(pattern) = row.patterns.first() {
        return Err(policy(&format!(
            "job {:?}: pattern {pattern:?} has no declared executable pattern",
            row.job.id
        )));
    }
    for fingerprint in &row.job.forbidden.fingerprints {
        if let Fingerprint::Rule(id) = fingerprint
            && !hits.contains_key(id.as_str())
        {
            return Err(Error::RegistrationRequired {
                job: row.job.id.clone(),
                component: format!(
                    "unknown rule id {:?}; register its executable Rule on Gate",
                    id.as_str()
                ),
            });
        }
    }
    for variant in &row.variants {
        if variant.detector != "isomorphic" && !hits.contains_key(variant.detector.as_str()) {
            return Err(Error::RegistrationRequired {
                job: row.job.id.clone(),
                component: format!(
                    "unknown variant detector {:?}; register its executable Rule on Gate",
                    variant.detector
                ),
            });
        }
    }
    Ok(())
}

fn extension(
    ledger: &Ledger,
    job: &LedgerJob,
    extensions: &[Box<dyn LedgerExtension>],
) -> Result<(), Error> {
    let Some(id) = &job.extension else {
        return Ok(());
    };
    let node = ledger.graph().get(id).expect("validated extension node");
    if node
        .properties()
        .all(|property| property == raf_rdf::vocab::TYPE)
    {
        return Ok(());
    }
    let mut claimed = false;
    for extension in extensions {
        if let Some(result) = extension.validate(&job.job.id, node) {
            if claimed {
                return Err(policy(&format!(
                    "job {:?}: more than one LedgerExtension claims {id}",
                    job.job.id
                )));
            }
            result.map_err(|reason| {
                policy(&format!("job {:?}: extension {id}: {reason}", job.job.id))
            })?;
            claimed = true;
        }
    }
    if !claimed {
        return Err(Error::RegistrationRequired {
            job: job.job.id.clone(),
            component: format!("unclaimed extension {id}; register a LedgerExtension on Gate"),
        });
    }
    Ok(())
}

pub(super) fn policy(reason: &str) -> Error {
    Error::Source {
        file: "helpers-ledger.ttl".into(),
        line: 1,
        reason: reason.into(),
    }
}
