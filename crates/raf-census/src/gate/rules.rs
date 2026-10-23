// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Fixture admission and ownership-checked execution of registered rules.

use std::collections::{BTreeMap, BTreeSet};

use raf_source::{Workspace, parse_source};

use super::owners::{self, Owners};
use crate::{Census, Error, Fingerprint, Ledger, Rule, RuleHit};

pub(super) type UnitKey = (String, String, usize);
pub(super) type Hits = BTreeMap<&'static str, BTreeMap<UnitKey, Vec<RuleHit>>>;

pub(super) struct Subject {
    pub(super) symbol: String,
    pub(super) file: String,
    pub(super) start_byte: usize,
    pub(super) line: usize,
    pub(super) package: String,
}

pub(super) struct Collected {
    pub(super) hits: Hits,
    pub(super) subjects: BTreeMap<UnitKey, Subject>,
}

pub(super) struct JobHits<'a> {
    pub(super) shared: &'a Hits,
    pub(super) contextual: Hits,
    pub(super) names: BTreeSet<UnitKey>,
}

impl JobHits<'_> {
    pub(super) fn get(&self, id: &str) -> Option<&BTreeMap<UnitKey, Vec<RuleHit>>> {
        self.contextual.get(id).or_else(|| self.shared.get(id))
    }
}

pub(super) fn collect(
    workspace: &Workspace,
    census: &Census,
    rules: &[Box<dyn Rule>],
    owners: &Owners<'_>,
    ledger: &Ledger,
) -> Result<Collected, Error> {
    let admitted = admit(rules)?;
    let mut ids: BTreeSet<_> = admitted.iter().map(|(id, _)| *id).collect();
    let requested = requested(ledger);
    let builtin = requested
        .iter()
        .any(|id| crate::detectors::needs_syntax(id));
    ids.extend(crate::detectors::ids());
    let mut matches: Hits = ids.into_iter().map(|id| (id, BTreeMap::new())).collect();
    let mut subjects = BTreeMap::new();
    fact_hits(census, &requested, &mut matches);
    if rules.is_empty() && !builtin {
        return Ok(Collected {
            hits: matches,
            subjects,
        });
    }
    let contexts: BTreeMap<_, _> = census
        .files()
        .iter()
        .map(|ctx| (ctx.path.as_str(), ctx))
        .collect();
    for source in workspace.files() {
        let ctx = contexts
            .get(source.path.as_str())
            .ok_or_else(|| refused("source", "a loaded file has no census context"))?;
        let (found, builtins) = parse_source(&source.path, &source.text, |syntax| {
            let registered = admitted
                .iter()
                .map(|(id, rule)| {
                    let mut hits = Vec::new();
                    rule.check(ctx, syntax, &mut hits);
                    (*id, hits)
                })
                .collect::<Vec<_>>();
            let builtins = if builtin {
                crate::detectors::collect(syntax)
            } else {
                Vec::new()
            };
            (registered, builtins)
        })?;
        builtin_hits(
            workspace,
            &source.path,
            owners,
            &mut matches,
            &mut subjects,
            builtins,
        )?;
        for (id, hits) in found {
            for mut hit in hits {
                let resolved = workspace.resolve(&hit.symbol).map_err(|error| {
                    refused(id, &format!("hit owner {:?}: {error}", hit.symbol))
                })?;
                let sites: BTreeSet<_> = resolved
                    .sites
                    .iter()
                    .filter_map(|site| workspace.item(*site))
                    .filter(|item| item.file == source.path && item.span.lines.contains(&hit.line))
                    .map(|item| item.span.start_byte)
                    .collect();
                let owners: Vec<_> = sites
                    .iter()
                    .filter_map(|start| owners.get(&(source.path.to_string(), *start)))
                    .collect();
                if owners.len() != 1 || hit.message.trim().is_empty() {
                    return Err(refused(
                        id,
                        &format!(
                            "hit at {}:{} must name one containing census unit with a nonempty message",
                            source.path, hit.line
                        ),
                    ));
                }
                let owner = owners[0];
                hit.symbol.clone_from(&owner.symbol);
                matches
                    .get_mut(id)
                    .expect("admitted rule")
                    .entry(owners::key(owner))
                    .or_default()
                    .push(hit);
            }
        }
    }
    Ok(Collected {
        hits: matches,
        subjects,
    })
}

fn requested(ledger: &Ledger) -> BTreeSet<&str> {
    ledger
        .jobs()
        .iter()
        .flat_map(|row| {
            row.job
                .forbidden
                .fingerprints
                .iter()
                .filter_map(|fingerprint| match fingerprint {
                    Fingerprint::Rule(id) => Some(id.as_str()),
                    _ => None,
                })
                .chain(row.variants.iter().map(|variant| variant.detector.as_str()))
        })
        .collect()
}

fn admit(rules: &[Box<dyn Rule>]) -> Result<Vec<(&'static str, &dyn Rule)>, Error> {
    let mut ids = BTreeSet::new();
    let mut admitted = Vec::new();
    for rule in rules {
        let id = rule.id();
        if !id.split('-').all(|part| {
            part.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        }) || !ids.insert(id)
            || crate::rule::BUILTIN_RULE_IDS.contains(&id)
        {
            return Err(refused(
                id,
                "rule ids must be unique lowercase kebab-case names and must not replace a built-in id",
            ));
        }
        rule.fixtures().validate(rule.as_ref())?;
        admitted.push((id, rule.as_ref()));
    }
    Ok(admitted)
}

fn fact_hits(census: &Census, requested: &BTreeSet<&str>, matches: &mut Hits) {
    if !requested.contains("hex-table") && !requested.contains("json-pointer-escape") {
        return;
    }
    for unit in census.units() {
        for hit in crate::detectors::facts(unit) {
            if requested.contains(hit.rule) {
                matches
                    .get_mut(hit.rule)
                    .expect("implemented fact detector")
                    .entry(owners::key(unit))
                    .or_default()
                    .push(RuleHit {
                        line: hit.line,
                        symbol: unit.symbol.clone(),
                        message: hit.message,
                    });
            }
        }
    }
}

fn builtin_hits(
    workspace: &Workspace,
    file: &raf_source::RelPath,
    owners: &Owners<'_>,
    matches: &mut Hits,
    subjects: &mut BTreeMap<UnitKey, Subject>,
    hits: Vec<crate::detectors::RawHit>,
) -> Result<(), Error> {
    for hit in hits {
        let physical: BTreeMap<_, _> = workspace
            .item_at(file, hit.byte)
            .into_iter()
            .filter_map(|item| owners.get(&(file.to_string(), item.span.start_byte)))
            .map(|unit| (owners::key(unit), *unit))
            .collect();
        if physical.len() != 1 {
            if physical.is_empty() {
                source_hit(workspace, file, matches, subjects, hit)?;
                continue;
            }
            return Err(refused(
                hit.rule,
                &format!(
                    "{}:{}: detector hit must have one physical census owner",
                    file, hit.line
                ),
            ));
        }
        let (key, unit) = physical.into_iter().next().expect("one physical owner");
        if unit.shipping || !crate::detectors::shipping_only(hit.rule) {
            matches
                .get_mut(hit.rule)
                .expect("implemented detector")
                .entry(key)
                .or_default()
                .push(RuleHit {
                    line: hit.line,
                    symbol: unit.symbol.clone(),
                    message: hit.message,
                });
        }
    }
    Ok(())
}

fn source_hit(
    workspace: &Workspace,
    file: &raf_source::RelPath,
    matches: &mut Hits,
    subjects: &mut BTreeMap<UnitKey, Subject>,
    hit: crate::detectors::RawHit,
) -> Result<(), Error> {
    let sites = workspace.item_at(file, hit.byte);
    let mut by_location = BTreeMap::new();
    for item in sites {
        by_location
            .entry(item.span.start_byte)
            .or_insert_with(Vec::new)
            .push(item);
    }
    if by_location.len() != 1 {
        return Err(refused(
            hit.rule,
            "detector hit requires one physical source item",
        ));
    }
    let mut sites = by_location.into_values().next().expect("one source item");
    sites.sort_by(|a, b| {
        (!workspace.is_shipping(a), &a.path).cmp(&(!workspace.is_shipping(b), &b.path))
    });
    let item = sites[0];
    if crate::detectors::shipping_only(hit.rule)
        && !sites.iter().any(|site| workspace.is_shipping(site))
    {
        return Ok(());
    }
    let target = workspace
        .target(item.target)
        .ok_or_else(|| refused(hit.rule, "source item has no owning target"))?;
    let key = (
        item.path.to_string(),
        file.to_string(),
        item.span.start_byte,
    );
    subjects.entry(key.clone()).or_insert_with(|| Subject {
        symbol: item.path.to_string(),
        file: file.to_string(),
        start_byte: item.span.start_byte,
        line: item.name_line,
        package: target.package.clone(),
    });
    matches
        .get_mut(hit.rule)
        .expect("implemented detector")
        .entry(key)
        .or_default()
        .push(RuleHit {
            line: hit.line,
            symbol: item.path.to_string(),
            message: hit.message,
        });
    Ok(())
}

pub(super) fn refused(id: &str, reason: &str) -> Error {
    Error::Source {
        file: format!("rule:{id}"),
        line: 1,
        reason: reason.into(),
    }
}
