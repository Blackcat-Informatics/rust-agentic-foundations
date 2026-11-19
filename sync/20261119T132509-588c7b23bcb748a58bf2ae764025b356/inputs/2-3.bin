// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Fixture admission and ownership-checked execution of registered rules.

use std::collections::{BTreeMap, BTreeSet};

use purrdf_jsonschema::ecma::MatchLimits;
use raf_source::{
    CodeResolveError, CodeResolver, Item, ItemId, Namespace, ResolveError, SelfType, SymbolKind,
    Workspace, parse_source,
};

use super::owners::{self, Owners};
use crate::detectors::{ReceiverMode, SyntaxHits};
use crate::{Census, Error, Ledger, Rule, RuleHit};

pub(super) type UnitKey = (String, String, usize);
pub(super) type Hits = BTreeMap<String, BTreeMap<UnitKey, Vec<RuleHit>>>;
/// The byte and line of every recorded hit, by rule and owning unit. The public
/// `RuleHit` states a line only; the ratchet needs the byte.
pub(super) type Offsets = BTreeMap<String, BTreeMap<UnitKey, Vec<Placed>>>;
/// For one actual shipping reach, each call's complete set of possible callees.
type ResolvedCalls = Vec<Vec<ItemId>>;
type Receivers = BTreeMap<raf_source::RelPath, BTreeMap<usize, ReceiverMode>>;

/// One recorded hit with the position the site key needs.
pub(super) struct Placed {
    pub(super) byte: usize,
    pub(super) line: usize,
    pub(super) message: String,
    pub(super) call_reaches: Vec<ResolvedCalls>,
}

/// The first byte of the one-based `line` of `text`; the end of the text for a line past it.
fn line_start(text: &str, line: usize) -> usize {
    line.checked_sub(2).map_or(0, |skip| {
        text.match_indices('\n')
            .nth(skip)
            .map_or(text.len(), |(at, _)| at + 1)
    })
}

/// Record the position of a line-only hit of the unit `owner`: the start of its
/// line, no earlier than the start of the owning item.
pub(super) fn record_line(
    collected: &mut Collected,
    (rule, key): (&str, UnitKey),
    text: &str,
    hit: &RuleHit,
) {
    let byte = line_start(text, hit.line).max(key.2);
    collected
        .offsets
        .entry(rule.to_owned())
        .or_default()
        .entry(key)
        .or_default()
        .push(Placed {
            byte,
            line: hit.line,
            message: hit.message.clone(),
            call_reaches: Vec::new(),
        });
}

pub(super) struct Hit<'a> {
    pub(super) rule: &'a str,
    pub(super) byte: usize,
    pub(super) line: usize,
    pub(super) message: String,
    pub(super) call_reaches: Vec<ResolvedCalls>,
}

pub(super) struct Subject {
    pub(super) sites: BTreeSet<ItemId>,
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
    pub(super) bounds: super::bounds::Scan,
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
    (workspace, code): (&Workspace, &CodeResolver<'_>),
    census: &Census,
    selected: &Selected<'_>,
    owners: &Owners<'_>,
    ledger: &Ledger,
    limits: &mut MatchLimits,
) -> Result<Collected, Error> {
    let rules = selected.registered;
    let tracked = ledger.ratcheted_rules();
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
        bounds: super::bounds::Scan::default(),
    };
    fact_hits(workspace, census, (&requested, &tracked), &mut collected)?;
    if rules.is_empty() && !builtin && ledger.numeric_bounds.is_none() {
        return Ok(collected);
    }
    let mut receivers = Receivers::new();
    let mut pending = Vec::new();
    for source in workspace.files() {
        let Some(ctx) = census.source_context(&source.path) else {
            continue;
        };
        let (found, builtins) = parse_source(&source.path, &source.text, |syntax| {
            if ledger.numeric_bounds.is_some() {
                collected
                    .bounds
                    .file((workspace, code), source, syntax, limits)?;
            }
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
                Ok(SyntaxHits::default())
            };
            Ok::<_, Error>((registered, builtins))
        })??;
        let SyntaxHits {
            hits,
            receivers: declared,
        } = builtins.map_err(|refusal| {
            refused(
                refusal.rule,
                &format!("{}:{}: {}", source.path, refusal.line, refusal.message),
            )
        })?;
        if !declared.is_empty() {
            receivers.insert(source.path.clone(), declared.into_iter().collect());
        }
        for hit in hits {
            if hit.call_evidence.is_some() {
                pending.push((&source.path, hit));
            } else {
                builtin_hit(
                    workspace,
                    &source.path,
                    owners,
                    &mut collected,
                    hit,
                    Vec::new(),
                )?;
            }
        }
        registered_hits(workspace, source, owners, &tracked, &mut collected, found)?;
    }
    for (file, hit) in pending {
        let reaches = source_calls((workspace, code, &receivers), file, &hit)?;
        builtin_hit(workspace, file, owners, &mut collected, hit, reaches)?;
    }
    Ok(collected)
}

fn registered_hits(
    workspace: &Workspace,
    source: &raf_source::SourceFile,
    owners: &Owners<'_>,
    tracked: &BTreeSet<&str>,
    collected: &mut Collected,
    found: Vec<(&'static str, Vec<RuleHit>)>,
) -> Result<(), Error> {
    for (id, hits) in found {
        for mut hit in hits {
            let resolved = workspace
                .resolve(&hit.symbol)
                .map_err(|error| refused(id, &format!("hit owner {:?}: {error}", hit.symbol)))?;
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
            if tracked.contains(id) {
                record_line(collected, (id, owners::key(owner)), &source.text, &hit);
            }
            collected
                .hits
                .get_mut(id)
                .expect("admitted rule")
                .entry(owners::key(owner))
                .or_default()
                .push(hit);
        }
    }
    Ok(())
}

pub(super) fn requested<'a>(
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
    if let Some(id) = requested
        .iter()
        .copied()
        .chain(required.iter().map(String::as_str))
        .find(|id| crate::report::bounds::IDS.contains(id))
    {
        return Err(refused(
            id,
            "numeric bounds guard ids are reserved; configure numericBounds on the ledger",
        ));
    }
    let ratcheted = ledger.ratcheted_rules();
    let mut required_ids = BTreeSet::new();
    for id in required {
        if ratcheted.contains(id.as_str()) {
            return Err(refused(
                id,
                "a rule the Gate requires cannot also be ratcheted by a job; drop it from ratchetedRule or from the required rules",
            ));
        }
        if !required_ids.insert(id.as_str()) {
            return Err(refused(id, "required rule is declared more than once"));
        }
        if !registered.contains(id.as_str()) && !crate::detectors::implements(id) {
            return Err(refused(
                id,
                "required rule has no registered implementation",
            ));
        }
        if crate::detectors::job_scoped(id) {
            return Err(refused(id, "this detector requires a ledger job and home"));
        }
        requested.insert(id.as_str());
    }
    Ok(requested)
}

pub(super) fn admit(rules: &[Box<dyn Rule>]) -> Result<Vec<(&'static str, &dyn Rule)>, Error> {
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
            || crate::report::bounds::IDS.contains(&id)
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

fn fact_hits(
    workspace: &Workspace,
    census: &Census,
    (requested, tracked): (&BTreeSet<&str>, &BTreeSet<&str>),
    collected: &mut Collected,
) -> Result<(), Error> {
    if !requested.contains("hex-table") && !requested.contains("json-pointer-escape") {
        return Ok(());
    }
    for unit in census.units() {
        for hit in crate::detectors::facts(unit) {
            if !requested.contains(hit.rule) {
                continue;
            }
            let found = RuleHit {
                line: hit.line,
                symbol: unit.symbol.clone(),
                message: hit.message,
            };
            if tracked.contains(hit.rule) {
                let path = raf_source::RelPath::new(&unit.file)
                    .map_err(|error| refused(hit.rule, &error.to_string()))?;
                let text = workspace.source(&path).ok_or_else(|| {
                    refused(hit.rule, &format!("{}: source text is absent", unit.file))
                })?;
                record_line(collected, (hit.rule, owners::key(unit)), text, &found);
            }
            collected
                .hits
                .get_mut(hit.rule)
                .expect("implemented fact detector")
                .entry(owners::key(unit))
                .or_default()
                .push(found);
        }
    }
    Ok(())
}

fn builtin_hit(
    workspace: &Workspace,
    file: &raf_source::RelPath,
    owners: &Owners<'_>,
    collected: &mut Collected,
    hit: crate::detectors::RawHit,
    call_reaches: Vec<ResolvedCalls>,
) -> Result<(), Error> {
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
            call_reaches,
        },
    )
}

fn source_calls(
    source: (&Workspace, &CodeResolver<'_>, &Receivers),
    file: &raf_source::RelPath,
    hit: &crate::detectors::RawHit,
) -> Result<Vec<ResolvedCalls>, Error> {
    use crate::detectors::CallEvidence;

    let Some(evidence) = &hit.call_evidence else {
        return Ok(Vec::new());
    };
    let (calls, needs_recursion) = match evidence {
        CallEvidence::Reifier(call) => (std::slice::from_ref(call), false),
        CallEvidence::List { calls, looped } => (calls.as_slice(), !looped),
    };
    let mut reaches = Vec::new();
    for owner in source.0.item_at(file, hit.byte) {
        if !source.0.is_shipping(owner) {
            continue;
        }
        let mut proven = Vec::new();
        for call in calls {
            let sites = resolve_call(source, owner, call).map_err(|error| {
                let path = match &call.path {
                    crate::detectors::CallPath::Written(path)
                    | crate::detectors::CallPath::Import(path)
                    | crate::detectors::CallPath::SelfType(path) => path,
                };
                let candidates = match &error {
                    CodeResolveError::Resolve(ResolveError::Ambiguous(ids)) => ids
                        .iter()
                        .filter_map(|id| source.0.item(*id))
                        .map(|item| format!("{} ({:?}, {})", item.path, item.kind, item.file))
                        .collect::<Vec<_>>()
                        .join(", "),
                    _ => String::new(),
                };
                refused(
                    hit.rule,
                    &format!(
                        "{file}:{}: call {path:?} from {}: {error}; candidates {candidates}",
                        hit.line, owner.path
                    ),
                )
            })?;
            if !sites.is_empty() {
                proven.push(sites);
            }
        }
        if !needs_recursion
            || proven
                .iter()
                .any(|sites| sites.iter().all(|id| *id == owner.id))
        {
            reaches.push(proven);
        }
    }
    Ok(reaches)
}

pub(super) fn self_member(workspace: &Workspace, owner: &Item, member: &str) -> Option<String> {
    if member.is_empty()
        || owner.kind != SymbolKind::InherentMethod
        || owner.self_key != Some(SelfType::Local)
    {
        return None;
    }
    let (self_path, _) = owner.path.as_str().rsplit_once("::")?;
    let root = &workspace.target(owner.target)?.crate_path;
    let nominal = self_path.strip_prefix(root)?.strip_prefix("::")?;
    Some(format!("crate::{nominal}::{member}"))
}

fn resolve_call(
    (workspace, code, receivers): (&Workspace, &CodeResolver<'_>, &Receivers),
    owner: &Item,
    call: &crate::detectors::Call,
) -> Result<Vec<ItemId>, CodeResolveError> {
    use crate::detectors::CallPath;

    // A block import declared in an enclosing function has a different
    // reading owner. Only origins within this exact indexed item are admitted.
    if !(owner.span.start_byte..owner.span.end_byte).contains(&call.byte) {
        return Ok(Vec::new());
    }
    let self_path;
    let (path, import) = match &call.path {
        CallPath::Written(path) => (path.as_str(), false),
        CallPath::Import(path) => (path.as_str(), true),
        CallPath::SelfType(member) => {
            let Some(path) = self_member(workspace, owner, member) else {
                return Ok(Vec::new());
            };
            self_path = path;
            (self_path.as_str(), false)
        }
    };
    let lookup = |path: &str, namespace| {
        if import {
            code.resolve_import_from_item_at(owner.id, call.byte, path, namespace)
        } else {
            code.resolve_from_item_at(owner.id, call.byte, path, namespace)
        }
    };
    let sites = match lookup(path, Namespace::Value) {
        Ok(resolved) if call.receiver.is_none() || resolved.type_aliases.is_empty() => {
            resolved.sites
        }
        Ok(_)
        | Err(
            CodeResolveError::Resolve(ResolveError::NotFound(_) | ResolveError::UnresolvedSelf(_))
            | CodeResolveError::AliasMember { .. },
        ) => return Ok(Vec::new()),
        Err(CodeResolveError::Resolve(ResolveError::Ambiguous(ids))) if call.receiver.is_some() => {
            let Some(inherent) = inherent_candidate(workspace, path, &ids, |nominal| {
                lookup(nominal, Namespace::Type)
            })?
            else {
                return Err(CodeResolveError::Resolve(ResolveError::Ambiguous(ids)));
            };
            vec![inherent]
        }
        Err(error) => return Err(error),
    };
    for id in &sites {
        let item = workspace
            .item(*id)
            .ok_or(CodeResolveError::UnknownItem(*id))?;
        match item.kind {
            SymbolKind::Fn if call.receiver.is_none() => {}
            SymbolKind::InherentMethod
                if call
                    .receiver
                    .is_none_or(|mode| exact_receiver(workspace, receivers, item, mode)) =>
            {
                match code.require_unconditional_inherent_method(*id) {
                    Ok(()) => {}
                    Err(CodeResolveError::MethodApplicability(_)) => return Ok(Vec::new()),
                    Err(error) => return Err(error),
                }
            }
            _ => return Ok(Vec::new()),
        }
    }
    Ok(sites)
}

/// An inherent candidate can take precedence only when the native type lookup
/// identifies one nominal definition and every candidate belongs to its member.
pub(super) fn inherent_candidate(
    workspace: &Workspace,
    path: &str,
    candidates: &[ItemId],
    lookup_type: impl FnOnce(&str) -> Result<raf_source::Resolution, CodeResolveError>,
) -> Result<Option<ItemId>, CodeResolveError> {
    let Some((prefix, member)) = path.rsplit_once("::") else {
        return Ok(None);
    };
    let resolved = match lookup_type(prefix) {
        Ok(resolved) => resolved,
        Err(CodeResolveError::Resolve(ResolveError::NotFound(_) | ResolveError::Ambiguous(_))) => {
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    let [id] = resolved.sites.as_slice() else {
        return Ok(None);
    };
    let nominal = workspace
        .item(*id)
        .ok_or(CodeResolveError::UnknownItem(*id))?;
    if !matches!(
        nominal.kind,
        SymbolKind::Struct | SymbolKind::Enum | SymbolKind::Union
    ) {
        return Ok(None);
    }
    let path = format!("{}::{member}", nominal.path);
    let mut inherent = None;
    for id in candidates {
        let item = workspace
            .item(*id)
            .ok_or(CodeResolveError::UnknownItem(*id))?;
        if item.path.as_str() != path || item.self_key != Some(SelfType::Local) {
            return Ok(None);
        }
        match item.kind {
            SymbolKind::InherentMethod if item.target == nominal.target && inherent.is_none() => {
                inherent = Some(*id);
            }
            SymbolKind::TraitImplMethod => {}
            _ => return Ok(None),
        }
    }
    Ok(inherent)
}

fn exact_receiver(
    workspace: &Workspace,
    receivers: &Receivers,
    item: &Item,
    mode: ReceiverMode,
) -> bool {
    workspace
        .target(item.target)
        .is_some_and(|target| target.root == item.file)
        && receivers
            .get(&item.file)
            .and_then(|definitions| definitions.get(&item.span.start_byte))
            == Some(&mode)
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
            .push(Placed {
                byte: hit.byte,
                line: hit.line,
                message: hit.message.clone(),
                call_reaches: hit.call_reaches,
            });
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
        .push(Placed {
            byte: hit.byte,
            line: hit.line,
            message: hit.message.clone(),
            call_reaches: hit.call_reaches,
        });
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
