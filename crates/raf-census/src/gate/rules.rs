// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Fixture admission and ownership-checked execution of registered rules.

use std::collections::{BTreeMap, BTreeSet};

use raf_source::{Workspace, parse_source};

use super::owners::{self, Owners};
use crate::{Census, Error, Ledger, Rule, RuleHit};

pub(super) type UnitKey = (String, String, usize);
pub(super) type Hits = BTreeMap<String, BTreeMap<UnitKey, Vec<RuleHit>>>;
/// The byte and line of every recorded hit, by rule and owning unit. The public
/// `RuleHit` states a line only; the ratchet needs the byte.
pub(super) type Offsets = BTreeMap<String, BTreeMap<UnitKey, Vec<(usize, usize)>>>;

pub(super) struct Hit<'a> {
    pub(super) rule: &'a str,
    pub(super) byte: usize,
    pub(super) line: usize,
    pub(super) message: String,
}

pub(super) struct Subject {
    pub(super) sites: BTreeSet<raf_source::ItemId>,
    pub(super) source_file: raf_source::RelPath,
    pub(super) symbol: String,
    pub(super) file: String,
    pub(super) start_byte: usize,
    pub(super) line: usize,
    pub(super) package: String,
}

pub(super) struct Collected {
    pub(super) hits: Hits,
    pub(super) subjects: BTreeMap<UnitKey, Subject>,
    pub(super) offsets: Offsets,
}

pub(super) struct JobHits<'a> {
    pub(super) shared: &'a Hits,
    pub(super) contextual: Hits,
    pub(super) names: BTreeSet<UnitKey>,
}

#[derive(Clone, Copy)]
pub(super) struct Selected<'a> {
    pub(super) registered: &'a [Box<dyn Rule>],
    pub(super) required: &'a [String],
}

impl Selected<'_> {
    pub(super) fn findings(&self, hits: &Hits) -> Vec<String> {
        let mut findings = Vec::new();
        for id in self.required {
            if let Some(owners) = hits.get(id) {
                for ((_, file, _), rows) in owners {
                    for hit in rows {
                        findings.push(format!(
                            "required rule {id}: {file}:{}: {}: {}",
                            hit.line, hit.symbol, hit.message
                        ));
                    }
                }
            }
        }
        findings
    }
}

impl JobHits<'_> {
    pub(super) fn get(&self, id: &str) -> Option<&BTreeMap<UnitKey, Vec<RuleHit>>> {
        self.contextual.get(id).or_else(|| self.shared.get(id))
    }
}

pub(super) fn collect(
    workspace: &Workspace,
    census: &Census,
    selected: &Selected<'_>,
    owners: &Owners<'_>,
    ledger: &Ledger,
) -> Result<Collected, Error> {
    let rules = selected.registered;
    let admitted = admit(rules)?;
    let mut ids: BTreeSet<_> = admitted.iter().map(|(id, _)| *id).collect();
    let requested = requested(ledger, selected.required, &ids)?;
    let builtin = requested
        .iter()
        .any(|id| crate::detectors::needs_syntax(id));
    ids.extend(crate::detectors::ids());
    let mut collected = Collected {
        hits: ids
            .into_iter()
            .map(|id| (id.to_owned(), BTreeMap::new()))
            .collect(),
        subjects: BTreeMap::new(),
        offsets: BTreeMap::new(),
    };
    fact_hits(census, &requested, &mut collected.hits);
    if rules.is_empty() && !builtin {
        return Ok(collected);
    }
    for source in workspace.files() {
        let Some(ctx) = census.source_context(&source.path) else {
            continue;
        };
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
                crate::detectors::collect(syntax, &requested)
            } else {
                Vec::new()
            };
            (registered, builtins)
        })?;
        builtin_hits(workspace, &source.path, owners, &mut collected, builtins)?;
        for (id, hits) in found {
            for mut hit in hits {
                let resolved = workspace.resolve(&hit.symbol).map_err(|error| {
                    refused(id, &format!("hit owner {:?}: {error}", hit.symbol))
                })?;
                let sites: BTreeSet<_> = resolved
                    .sites
                    .iter()
                    .filter_map(|site| workspace.item(*site))
                    .filter(|item| !workspace.same_source(item).is_empty())
                    .filter(|item| item.file == source.path && item.span.lines.contains(&hit.line))
                    .map(|item| item.span.start_byte)
                    .collect();
                let owners: Vec<_> = sites
                    .iter()
                    .filter_map(|start| owners.get(&(source.path.clone(), *start)))
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
                collected
                    .hits
                    .get_mut(id)
                    .expect("admitted rule")
                    .entry(owners::key(owner))
                    .or_default()
                    .push(hit);
            }
        }
    }
    Ok(collected)
}

fn requested<'a>(
    ledger: &'a Ledger,
    required: &'a [String],
    registered: &BTreeSet<&str>,
) -> Result<BTreeSet<&'a str>, Error> {
    let mut requested: BTreeSet<_> = ledger
        .jobs()
        .iter()
        .flat_map(|row| {
            row.referenced()
                .chain(row.variants.iter().map(|variant| variant.detector.as_str()))
        })
        .collect();
    let mut required_ids = BTreeSet::new();
    for id in required {
        if !required_ids.insert(id.as_str()) {
            return Err(refused(id, "required rule is declared more than once"));
        }
        if !registered.contains(id.as_str()) && !crate::detectors::implements(id) {
            return Err(refused(
                id,
                "required rule has no registered implementation",
            ));
        }
        if matches!(
            id.as_str(),
            "home-literal" | "vocabulary-literal" | "sole-name"
        ) {
            return Err(refused(id, "this detector requires a ledger job and home"));
        }
        requested.insert(id.as_str());
    }
    Ok(requested)
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
    collected: &mut Collected,
    hits: Vec<crate::detectors::RawHit>,
) -> Result<(), Error> {
    for hit in hits {
        record_hit(
            workspace,
            file,
            owners,
            collected,
            Hit {
                rule: hit.rule,
                byte: hit.byte,
                line: hit.line,
                message: hit.message,
            },
        )?;
    }
    Ok(())
}

pub(super) fn record_hit(
    workspace: &Workspace,
    file: &raf_source::RelPath,
    owners: &Owners<'_>,
    collected: &mut Collected,
    hit: Hit<'_>,
) -> Result<(), Error> {
    let physical: BTreeMap<_, _> = workspace
        .item_at(file, hit.byte)
        .into_iter()
        .filter(|item| !workspace.same_source(item).is_empty())
        .filter_map(|item| owners.get(&(file.clone(), item.span.start_byte)))
        .map(|unit| (owners::key(unit), *unit))
        .collect();
    if physical.len() != 1 {
        if physical.is_empty() {
            source_hit(workspace, file, collected, hit)?;
            return Ok(());
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
        collected
            .offsets
            .entry(hit.rule.to_owned())
            .or_default()
            .entry(key.clone())
            .or_default()
            .push((hit.byte, hit.line));
        collected
            .hits
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
    Ok(())
}

fn source_hit(
    workspace: &Workspace,
    file: &raf_source::RelPath,
    collected: &mut Collected,
    hit: Hit<'_>,
) -> Result<(), Error> {
    let mut sites = workspace.item_at(file, hit.byte);
    if sites
        .iter()
        .any(|item| !workspace.same_source(item).is_empty())
    {
        sites.retain(|item| !workspace.same_source(item).is_empty());
    }
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
    collected
        .subjects
        .entry(key.clone())
        .or_insert_with(|| Subject {
            sites: sites.iter().map(|item| item.id).collect(),
            source_file: file.clone(),
            symbol: item.path.to_string(),
            file: file.to_string(),
            start_byte: item.span.start_byte,
            line: item.name_line,
            package: target.package.clone(),
        });
    collected
        .offsets
        .entry(hit.rule.to_owned())
        .or_default()
        .entry(key.clone())
        .or_default()
        .push((hit.byte, hit.line));
    collected
        .hits
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
