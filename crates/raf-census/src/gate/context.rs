// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Job-scoped detector evidence and one shared forbidden-name evaluation.

use purrdf_jsonschema::ecma::MatchLimits;
use std::collections::{BTreeMap, BTreeSet};

use super::{
    evaluate::{ResolvedJob, policy},
    owners,
    rules::{Hits, JobHits},
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
    shared: &'a Hits,
    census: &Census,
    row: &LedgerJob,
    scope: &ResolvedJob,
    limits: &mut MatchLimits,
) -> Result<JobHits<'a>, Error> {
    let mut result = JobHits {
        shared,
        contextual: BTreeMap::new(),
        names: BTreeSet::new(),
    };
    let home_literal = enabled(row, "home-literal");
    let vocabulary = enabled(row, "vocabulary-literal");
    let sole = enabled(row, "sole-name");
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
