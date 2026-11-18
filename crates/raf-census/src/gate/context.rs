// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Job-scoped detector evidence and one shared forbidden-name evaluation.

use purrdf_jsonschema::ecma::MatchLimits;
use std::collections::{BTreeMap, BTreeSet};

use super::{
    evaluate::{ResolvedJob, policy},
    owners,
    rules::{Collected, Hits, JobHits, Subject, UnitKey},
};
use crate::{Census, Error, Fingerprint, LedgerJob, RuleHit, StringKind, Unit, UnitKind};

fn enabled(row: &LedgerJob, id: &str) -> bool {
    row.job
        .forbidden
        .fingerprints
        .iter()
        .any(|fingerprint| matches!(fingerprint, Fingerprint::Rule(rule) if rule.as_str() == id))
}

fn literals(unit: &Unit) -> impl Iterator<Item = &crate::StringLiteral> {
    unit.strings
        .iter()
        .filter(|literal| literal.kind == StringKind::Str)
}

pub(super) fn collect<'a>(
    collected: &'a Collected,
    census: &Census,
    row: &LedgerJob,
    scope: &ResolvedJob,
    limits: &mut MatchLimits,
) -> Result<JobHits<'a>, Error> {
    let mut result = structures(collected, row, scope)?;
    let home_literal = enabled(row, "home-literal");
    let vocabulary = enabled(row, "vocabulary-literal");
    let sole = enabled(row, "sole-name");
    if enabled(row, "foreign-host-import") {
        foreign_imports(&mut result, row, scope, &collected.subjects)?;
    }
    if row.names.is_empty() && !home_literal && !vocabulary && !sole {
        return Ok(result);
    }
    let tokens: BTreeSet<_> = if home_literal {
        census
            .units()
            .iter()
            .chain(&census.reference_raw_units)
            .filter(|unit| scope.home_units.contains(&owners::key(unit)))
            .flat_map(literals)
            .map(|literal| literal.value.as_str())
            .collect()
    } else {
        BTreeSet::new()
    };
    let namespaces = if vocabulary {
        namespaces(census, row, scope)?
    } else {
        BTreeSet::new()
    };
    if home_literal && tokens.is_empty() {
        return Err(policy(&format!(
            "job {:?}: home-literal requires a home containing string literals",
            row.job.id
        )));
    }
    if vocabulary && namespaces.is_empty() {
        return Err(policy(&format!(
            "job {:?}: vocabulary-literal requires namespace constants under its home or entry points",
            row.job.id
        )));
    }
    for unit in census.units() {
        let key = owners::key(unit);
        let named = matches!(unit.kind, UnitKind::Fn | UnitKind::Method | UnitKind::Const)
            && name_matches(row, &unit.name, limits)?;
        if named {
            result.names.insert(key.clone());
        }
        let at_home = scope.homes.contains(&key);
        let in_package = scope.home_packages.contains(&unit.package);
        if sole
            && named
            && in_package
            && !at_home
            && matches!(unit.kind, UnitKind::Fn | UnitKind::Method)
        {
            push(
                &mut result.contextual,
                "sole-name",
                unit,
                unit.line,
                "a second function has the forbidden home name".into(),
            );
        }
        if !unit.shipping || (in_package && row.preferred_provider.is_none()) || at_home {
            continue;
        }
        for literal in literals(unit) {
            if home_literal && tokens.contains(literal.value.as_str()) {
                push(
                    &mut result.contextual,
                    "home-literal",
                    unit,
                    literal.line,
                    format!("literal {:?} repeats a token of the home", literal.value),
                );
            }
            if vocabulary
                && !embedded_document(&literal.value)
                && let Some(namespace) = namespaces
                    .iter()
                    .filter(|namespace| literal.value.starts_with(**namespace))
                    .max_by_key(|namespace| namespace.len())
            {
                push(
                    &mut result.contextual,
                    "vocabulary-literal",
                    unit,
                    literal.line,
                    format!(
                        "literal {:?} spells a term of namespace {namespace:?}",
                        literal.value
                    ),
                );
            }
        }
    }
    Ok(result)
}

/// Turtle, `TriG` and SPARQL keywords that mark a literal as an embedded document.
const EMBEDDED_DOCUMENT_KEYWORDS: [&str; 9] = [
    "PREFIX",
    "BASE",
    "SELECT",
    "WHERE",
    "CONSTRUCT",
    "ASK",
    "DESCRIBE",
    "INSERT",
    "DELETE",
];

/// Whether a literal is an embedded query or data document rather than a
/// hand-spelled term: it spans more than one line and contains a listed
/// keyword as a whole word, in any ASCII letter case. A word is a maximal run
/// of alphanumeric characters and underscores.
fn embedded_document(value: &str) -> bool {
    value.contains('\n')
        && value
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .any(|word| {
                EMBEDDED_DOCUMENT_KEYWORDS
                    .iter()
                    .any(|keyword| word.eq_ignore_ascii_case(keyword))
            })
}

fn namespaces<'a>(
    census: &'a Census,
    row: &LedgerJob,
    scope: &ResolvedJob,
) -> Result<BTreeSet<&'a str>, Error> {
    census
        .units()
        .iter()
        .chain(&census.reference_raw_units)
        .filter(|unit| {
            scope.homes.contains(&owners::key(unit))
                && unit.kind == UnitKind::Const
                && (unit.name == "NS" || unit.name.ends_with("_NS"))
        })
        .map(|unit| {
            unit.string_initializer().filter(|value| !value.is_empty()).ok_or_else(|| policy(&format!(
                "job {:?}: namespace {} requires a nonempty literal string initializer; computed namespace evidence is unavailable",
                row.job.id, unit.symbol
            )))
        })
        .collect()
}

fn name_matches(row: &LedgerJob, name: &str, limits: &mut MatchLimits) -> Result<bool, Error> {
    for pattern in &row.names {
        if pattern.is_match(name, limits).map_err(|error| {
            policy(&format!(
                "job {:?}, name pattern {:?}: {error}",
                row.job.id,
                pattern.source()
            ))
        })? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn push(hits: &mut Hits, id: &'static str, unit: &Unit, line: usize, message: String) {
    hits.entry(id.to_owned())
        .or_default()
        .entry(owners::key(unit))
        .or_default()
        .push(RuleHit {
            line,
            message,
            symbol: unit.symbol.clone(),
        });
}

/// Keep the imports of a host function that the job's home also imports, at
/// the sites outside the home.
///
/// A built-in hit's message states the imported identity, so equal messages
/// are the same host function.
fn foreign_imports(
    result: &mut JobHits<'_>,
    row: &LedgerJob,
    scope: &ResolvedJob,
    subjects: &BTreeMap<UnitKey, Subject>,
) -> Result<(), Error> {
    const ID: &str = "foreign-host-import";
    let at_home = |key: &UnitKey| {
        scope.homes.contains(key)
            || subjects
                .get(key)
                .is_some_and(|subject| !subject.sites.is_disjoint(&scope.home_sources))
    };
    let imports = result.shared.get(ID);
    let imported: BTreeSet<&str> = imports
        .into_iter()
        .flatten()
        .filter(|(key, _)| at_home(key))
        .flat_map(|(_, rows)| rows.iter().map(|hit| hit.message.as_str()))
        .collect();
    if imported.is_empty() {
        return Err(policy(&format!(
            "job {:?}: foreign-host-import requires a home that declares a host function import in an extern block",
            row.job.id
        )));
    }
    let mut kept = BTreeMap::new();
    for (key, rows) in imports.into_iter().flatten() {
        if at_home(key) {
            continue;
        }
        let rows: Vec<_> = rows
            .iter()
            .filter(|hit| imported.contains(hit.message.as_str()))
            .cloned()
            .collect();
        if !rows.is_empty() {
            kept.insert(key.clone(), rows);
        }
    }
    result.contextual.insert(ID.to_owned(), kept);
    Ok(())
}

const STRUCTURE_RULES: [(&str, bool); 2] =
    [("reifier-default-graph", true), ("rdf-list-walk", false)];

pub(super) fn uses_structure(row: &LedgerJob) -> bool {
    STRUCTURE_RULES
        .iter()
        .any(|(id, _)| uses_structure_rule(row, id))
}

fn uses_structure_rule(row: &LedgerJob, id: &str) -> bool {
    enabled(row, id)
        || row
            .variants
            .iter()
            .any(|variant| variant.detector.as_str() == id)
}

/// The job's hits before its context detectors run, with the structure
/// detectors already judged against its home and entry points.
fn structures<'a>(
    collected: &'a Collected,
    row: &LedgerJob,
    scope: &ResolvedJob,
) -> Result<JobHits<'a>, Error> {
    let mut result = JobHits {
        shared: &collected.hits,
        contextual: BTreeMap::new(),
        names: BTreeSet::new(),
    };
    for (id, delegated) in STRUCTURE_RULES {
        if uses_structure_rule(row, id) {
            structure(&mut result, collected, scope, (id, delegated))?;
        }
    }
    Ok(result)
}

/// Keep the candidates of a structure detector that stand in the right relation
/// to the job's home and entry points: with `delegated`, a candidate that calls
/// one of them is a hit (the reifier row is built through a declared
/// constructor); without it, a candidate that calls none of them is a hit.
fn structure(
    result: &mut JobHits<'_>,
    collected: &Collected,
    scope: &ResolvedJob,
    (id, delegated): (&'static str, bool),
) -> Result<(), Error> {
    let mut kept = BTreeMap::new();
    for (key, rows) in collected.hits.get(id).into_iter().flatten() {
        let placed = collected
            .offsets
            .get(id)
            .and_then(|offsets| offsets.get(key))
            .map_or(&[][..], Vec::as_slice);
        if placed.len() != rows.len() {
            return Err(super::rules::refused(
                id,
                "a structure hit has no unique recorded placement",
            ));
        }
        let rows: Vec<_> = rows
            .iter()
            .zip(placed)
            .filter(|(_, placed)| {
                placed.call_reaches.iter().any(|calls| {
                    calls
                        .iter()
                        .any(|sites| sites.iter().all(|id| scope.home_sources.contains(id)))
                        == delegated
                })
            })
            .map(|(hit, _)| hit.clone())
            .collect();
        if !rows.is_empty() {
            kept.insert(key.clone(), rows);
        }
    }
    result.contextual.insert(id.to_owned(), kept);
    Ok(())
}
