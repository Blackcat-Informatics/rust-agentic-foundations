// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The RAF profile: the schema table, the typed form of a valid document and
//! the validator that reports every violation.

mod body_rules;
#[cfg(test)]
mod cost_tests;
mod diagrams;
mod records;
mod review;
mod schema;
mod timestamp;
mod typed;
mod violation;
mod write;

use std::collections::HashSet;

use crate::document::Document;
use crate::frontmatter::value::{Entry, ItemValue, Map, Scalar, Value};
use crate::work::{Work, add};

pub(crate) use body_rules::placeholders;
pub use diagrams::{DiagramDeclaration, DiagramGrouping, DiagramKind, DiagramKindError};
pub use review::{
    ReviewDisposition, ReviewFinding, ReviewKind, ReviewLocation, ReviewRecord, ReviewScope,
    ReviewSeverity, ReviewVerdict,
};
pub use schema::{DocType, KeySpec, ValueKind};
pub use timestamp::{TimestampError, validate_timestamp};
pub use typed::{
    Actor, Affects, Architecture, AssessmentRecord, AssessmentVerdict, Bindings, Capability,
    CapabilitySection, Ceiling, CeilingClass, CeilingLimit, Change, ChangeItem, ContractEffect,
    CoverageRecord, DebtFinding, DebtLine, DebtRecord, EvidenceRecord, EvidenceSubject,
    EvidenceVerdict, Exception, ExceptionTarget, Exclusion, ExclusionItem, Header, Input,
    InputKind, Measure, Number, Outcome, RafDocument, RecordHeader, SectionRef, State,
};
pub use violation::{Location, Rule, RuleTag, Violation};

/// The binding keys of a capability.
const BINDING_KEYS: [&str; 3] = ["code", "tests", "examples"];

/// The RAF profile of OKF.
///
/// The profile is a unit: the closed key sets, the value rules and the body
/// rules are fixed, and [`Profile::validate`] applies them to one document.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Profile;

/// A lowercase kebab-case slug.
fn is_kebab(slug: &str) -> bool {
    crate::body::is_kebab(slug)
}

/// The scalar text of an entry.
fn scalar_text(entry: &Entry) -> Option<&str> {
    entry.value().as_scalar().map(Scalar::text)
}

/// The texts of a list whose items are all scalars.
fn strings(value: &Value) -> Option<Vec<&str>> {
    let list = value.as_list()?;
    list.items()
        .iter()
        .map(|item| item.value().as_scalar().map(Scalar::text))
        .collect()
}

/// `cap:<kebab-slug>`.
fn is_capability_id(entry: &str) -> bool {
    entry.strip_prefix("cap:").is_some_and(is_kebab)
}

/// The file name of `path` without `.md`.
fn file_stem(path: &str) -> &str {
    let name = match path.rsplit_once('/') {
        Some((_, name)) => name,
        None => path,
    };
    match name.strip_suffix(".md") {
        Some(stem) => stem,
        None => name,
    }
}

/// Whether `path` is a Markdown file directly in `directory`.
fn is_directly_in(path: &str, directory: &str) -> bool {
    let path = match path.strip_prefix("./") {
        Some(rest) => rest,
        None => path,
    };
    path.strip_prefix(directory)
        .and_then(|rest| rest.strip_prefix('/'))
        .is_some_and(|name| {
            !name.contains('/')
                && name
                    .strip_suffix(".md")
                    .is_some_and(|stem| !stem.is_empty())
        })
}

struct Check<'a> {
    file: &'a str,
    out: Vec<Violation>,
}

impl Check<'_> {
    fn add(&mut self, at: Location, rule: Rule) {
        self.out.push(Violation {
            file: self.file.to_string(),
            at,
            rule,
        });
    }

    fn key(&mut self, key: &str, rule: Rule) {
        self.add(Location::Key(key.to_string()), rule);
    }
}

/// The result of checking an optional key.
#[derive(Default)]
enum Slot<T> {
    /// The key is absent, which is valid.
    #[default]
    Absent,
    /// The key is present and valid.
    Valid(T),
    /// The key is present and a violation was added.
    Invalid,
}

impl<T> Slot<T> {
    fn from_check(value: Option<T>) -> Self {
        match value {
            Some(value) => Self::Valid(value),
            None => Self::Invalid,
        }
    }

    /// The value of a present key, nothing for an absent key, and `Invalid`
    /// for a key that failed its check.
    fn settle(self) -> Result<Option<T>, Invalid> {
        match self {
            Self::Absent => Ok(None),
            Self::Valid(value) => Ok(Some(value)),
            Self::Invalid => Err(Invalid),
        }
    }
}

/// A key that failed its check.
struct Invalid;

/// What the key checks collected.
#[derive(Default)]
struct Collected {
    id: Option<(String, String)>,
    title: Option<String>,
    description: Option<String>,
    tags: Slot<Vec<String>>,
    timestamp: Slot<String>,
    state: Option<State>,
    bindings: Option<Bindings>,
    depends: Option<Vec<String>>,
    review_required: Slot<Vec<ReviewKind>>,
    diagrams: Slot<Vec<DiagramDeclaration>>,
    changes: Option<Vec<String>>,
    /// The valid capability ids of `changes`, even when another entry is not.
    change_ids: Vec<String>,
    affects: Option<Vec<Affects>>,
    effect: Option<ContractEffect>,
    items: Option<Vec<ChangeItem>>,
    subject: Option<records::Subject>,
    inputs: Option<Vec<Input>>,
    verdict: Option<records::Verdict>,
    by: Option<Actor>,
    covers: Option<Vec<String>>,
    command: Slot<String>,
    revision: Slot<String>,
    values: Slot<Vec<Measure>>,
    limits: Slot<Vec<Measure>>,
    scope: Option<Vec<String>>,
    exclusions: Option<Vec<Exclusion>>,
    ceilings: Slot<Vec<Ceiling>>,
    exceptions: Slot<Vec<Exception>>,
    debt_findings: Option<Vec<DebtFinding>>,
    debt_lines: Option<Vec<DebtLine>>,
    review_scope: Option<ReviewScope>,
    review_kind: Option<ReviewKind>,
    review_findings: Option<Vec<ReviewFinding>>,
    review_reference: Slot<String>,
    /// Whether a rule between the keys of a record was broken.
    between_broken: bool,
}

impl Profile {
    /// Check one document against the profile.
    ///
    /// `path` is the repository-relative path of the document, with `/`
    /// separators. The result is the typed form of the document, or every
    /// violation found: a frontmatter rule names the key, a body rule names
    /// the section or the line. The validator looks at one document: whether
    /// the capabilities an id names exist is not its concern.
    ///
    /// # Errors
    ///
    /// The list of [`Violation`]s, never empty.
    pub fn validate(&self, document: &Document, path: &str) -> Result<RafDocument, Vec<Violation>> {
        let mut check = Check {
            file: path,
            out: Vec::new(),
        };
        let map = document.frontmatter().map();
        let Some(type_entry) = map.get("type") else {
            check.key(
                "type",
                Rule::MissingKey {
                    key: "type".to_string(),
                },
            );
            return Err(check.out);
        };
        let Some(type_text) = scalar_text(type_entry) else {
            check.key(
                "type",
                Rule::WrongShape {
                    expected: "a scalar",
                },
            );
            return Err(check.out);
        };
        let Some(doc_type) = DocType::from_name(type_text) else {
            check.key(
                "type",
                Rule::UnknownType {
                    found: type_text.to_string(),
                },
            );
            return Err(check.out);
        };
        if !is_directly_in(path, doc_type.directory()) {
            check.key("type", Rule::WrongDirectory { declared: doc_type });
        }
        check_key_set(&mut check, map, doc_type);
        let collected = check_values(&mut check, map, doc_type, file_stem(path));
        check_frontmatter_placeholders(&mut check, map);
        let body = document.body();
        body_rules::check_sections(doc_type, body, path, &mut check.out);
        body_rules::check_blocks(doc_type, body, path, &mut check.out);
        body_rules::check_placeholders(body, path, &mut check.out);
        // A value is missing from `collected` only after a violation was
        // added, so an empty list of violations always has a typed document.
        match build(doc_type, collected) {
            Some(typed) if check.out.is_empty() => Ok(typed),
            _ => Err(check.out),
        }
    }
}

fn check_key_set(check: &mut Check<'_>, map: &Map, doc_type: DocType) {
    let table = doc_type.keys();
    for entry in map.entries() {
        let key = entry.key();
        if table.iter().any(|spec| spec.key == key) {
            continue;
        }
        let rule = if key == "resource" {
            Rule::ResourceKey { doc_type }
        } else {
            Rule::UnknownKey {
                key: key.to_string(),
            }
        };
        check.key(key, rule);
    }
    for spec in table {
        if spec.required && map.get(spec.key).is_none() {
            check.key(
                spec.key,
                Rule::MissingKey {
                    key: spec.key.to_string(),
                },
            );
        }
    }
}

fn check_values(check: &mut Check<'_>, map: &Map, doc_type: DocType, stem: &str) -> Collected {
    let mut found = Collected::default();
    for spec in doc_type.keys() {
        let Some(entry) = map.get(spec.key) else {
            // An absent optional key is valid, and `Collected` starts absent.
            continue;
        };
        let key = spec.key;
        match spec.value {
            ValueKind::Diagrams => found.diagrams = Slot::from_check(diagrams::check(check, entry)),
            ValueKind::Id => found.id = check_id(check, entry, doc_type, stem),
            ValueKind::Text => {
                let text = check_text(check, entry);
                if key == "title" {
                    found.title = text;
                } else {
                    found.description = text;
                }
            }
            ValueKind::Tags => {
                found.tags = Slot::from_check(check_tags(check, entry));
            }
            ValueKind::Timestamp => {
                found.timestamp = Slot::from_check(check_timestamp(check, entry));
            }
            ValueKind::State => found.state = check_state(check, entry),
            ValueKind::ReviewKinds => {
                found.review_required = Slot::from_check(review::check_required(check, entry));
            }
            ValueKind::Bindings => {
                found.bindings = check_bindings(check, entry, found.state);
            }
            ValueKind::CapabilityIds => {
                let ids = check_capability_ids(check, entry, key == "changes");
                match key {
                    "changes" => {
                        if let Some(texts) = strings(entry.value()) {
                            found.change_ids = texts
                                .into_iter()
                                .filter(|id| is_capability_id(id))
                                .map(str::to_string)
                                .collect();
                        }
                        found.changes = ids;
                    }
                    "for" => {
                        found.covers = ids.filter(|ids| records::check_covers(check, ids));
                    }
                    _ => found.depends = ids,
                }
            }
            ValueKind::Affects => {
                found.affects = check_affects(check, entry, &found.change_ids);
            }
            ValueKind::ContractEffect => found.effect = check_effect(check, entry),
            ValueKind::Items => found.items = check_items(check, entry),
            ValueKind::Subject => {
                found.subject = records::check_subject(check, entry, doc_type);
            }
            ValueKind::Inputs => found.inputs = records::check_inputs(check, entry, doc_type),
            ValueKind::Verdict => {
                found.verdict = records::check_verdict(check, entry, doc_type);
            }
            ValueKind::Actor => found.by = records::check_actor(check, entry),
            ValueKind::Command => {
                found.command = Slot::from_check(records::check_command(check, entry));
            }
            ValueKind::Revision => {
                found.revision = Slot::from_check(records::check_revision(check, entry));
            }
            ValueKind::Measures => {
                let measures = Slot::from_check(records::check_measures(check, entry));
                if key == "limits" {
                    found.limits = measures;
                } else {
                    found.values = measures;
                }
            }
            ValueKind::Scope => found.scope = records::check_scope(check, entry),
            ValueKind::Exclusions => found.exclusions = records::check_exclusions(check, entry),
            ValueKind::Ceilings => {
                found.ceilings = Slot::from_check(records::check_ceilings(check, entry));
            }
            ValueKind::Exceptions => {
                found.exceptions = Slot::from_check(records::check_exceptions(check, entry));
            }
            ValueKind::DebtFindings | ValueKind::DebtLines => debt_rows(check, entry, &mut found),
            ValueKind::ReviewScope => found.review_scope = review::check_scope(check, entry),
            ValueKind::ReviewKind => {
                found.review_kind = review::check_kind(check, entry);
            }
            ValueKind::ReviewFindings => {
                found.review_findings = review::check_findings(check, entry);
            }
            ValueKind::ReviewReference => {
                found.review_reference = Slot::from_check(check_text(check, entry));
            }
            // `type` was read first.
            ValueKind::DocType => {}
        }
    }
    check_record_consistency(check, map, doc_type, &mut found);
    found
}

fn check_record_consistency(
    check: &mut Check<'_>,
    map: &Map,
    doc_type: DocType,
    found: &mut Collected,
) {
    found.between_broken = !records::check_between(
        check,
        &records::Keys {
            subject: found.subject.as_ref(),
            inputs: found.inputs.as_deref(),
            verdict: found.verdict,
            has_command: map.get("command").is_some(),
            values: measures(&found.values),
            limits: measures(&found.limits),
        },
    );
    if doc_type == DocType::Review {
        found.between_broken |= !review::check_consistency(
            check,
            found.verdict,
            found.review_findings.as_deref(),
            !matches!(&found.review_reference, Slot::Absent),
        );
    }
}

/// Check a list of the debt record and keep the rows that are valid.
fn debt_rows(check: &mut Check<'_>, entry: &Entry, found: &mut Collected) {
    if entry.key() == "findings" {
        found.debt_findings = records::check_debt_findings(check, entry);
    } else {
        found.debt_lines = records::check_debt_lines(check, entry);
    }
}

/// The measures of an optional key: none when it is absent, `None` when it
/// failed its check.
fn measures(slot: &Slot<Vec<Measure>>) -> Option<&[Measure]> {
    match slot {
        Slot::Absent => Some(&[]),
        Slot::Valid(measures) => Some(measures),
        Slot::Invalid => None,
    }
}

pub(crate) fn id_slug(doc_type: DocType, id: &str) -> Result<&str, Rule> {
    let prefix = doc_type.id_prefix();
    let slug = id
        .strip_prefix(prefix)
        .filter(|slug| is_kebab(slug))
        .ok_or(Rule::InvalidId { prefix })?;
    if doc_type.singleton().is_some_and(|only| only != slug) {
        return Err(Rule::SingletonId { doc_type });
    }
    Ok(slug)
}

fn check_id(
    check: &mut Check<'_>,
    entry: &Entry,
    doc_type: DocType,
    stem: &str,
) -> Option<(String, String)> {
    let Some(id) = scalar_text(entry) else {
        check.key(
            "id",
            Rule::WrongShape {
                expected: "a scalar",
            },
        );
        return None;
    };
    let slug = match id_slug(doc_type, id) {
        Ok(slug) => slug,
        Err(rule) => {
            check.key("id", rule);
            return None;
        }
    };
    if slug != stem {
        check.key(
            "id",
            Rule::IdFileName {
                id: id.to_string(),
                stem: stem.to_string(),
            },
        );
        return None;
    }
    Some((id.to_string(), slug.to_string()))
}

fn check_text(check: &mut Check<'_>, entry: &Entry) -> Option<String> {
    match scalar_text(entry) {
        Some(text) if !text.trim().is_empty() => Some(text.to_string()),
        _ => {
            check.key(entry.key(), Rule::EmptyText);
            None
        }
    }
}

fn check_tags(check: &mut Check<'_>, entry: &Entry) -> Option<Vec<String>> {
    let Some(tags) = strings(entry.value()) else {
        check.key(
            "tags",
            Rule::WrongShape {
                expected: "a list of strings",
            },
        );
        return None;
    };
    let mut seen = HashSet::new();
    let mut valid = true;
    for tag in &tags {
        if tag.trim().is_empty() {
            check.key("tags", Rule::EmptyTag);
            valid = false;
        } else if !seen.insert(*tag) {
            check.key(
                "tags",
                Rule::DuplicateTag {
                    tag: (*tag).to_string(),
                },
            );
            valid = false;
        }
    }
    valid.then(|| tags.into_iter().map(str::to_string).collect())
}

fn check_timestamp(check: &mut Check<'_>, entry: &Entry) -> Option<String> {
    let Some(text) = scalar_text(entry) else {
        check.key(
            "timestamp",
            Rule::WrongShape {
                expected: "a scalar",
            },
        );
        return None;
    };
    match validate_timestamp(text) {
        Ok(()) => Some(text.to_string()),
        Err(reason) => {
            check.key("timestamp", Rule::BadTimestamp { reason });
            None
        }
    }
}

fn check_state(check: &mut Check<'_>, entry: &Entry) -> Option<State> {
    let Some(text) = scalar_text(entry) else {
        check.key(
            "state",
            Rule::WrongShape {
                expected: "a scalar",
            },
        );
        return None;
    };
    let state = State::from_name(text);
    if state.is_none() {
        check.key(
            "state",
            Rule::BadState {
                found: text.to_string(),
            },
        );
    }
    state
}

fn check_bindings(check: &mut Check<'_>, entry: &Entry, state: Option<State>) -> Option<Bindings> {
    let Some(map) = entry.value().as_map() else {
        check.key(
            "bindings",
            Rule::WrongShape {
                expected: "a map of code, tests and examples lists",
            },
        );
        return None;
    };
    let exact = map.entries().len() == BINDING_KEYS.len()
        && BINDING_KEYS.iter().all(|key| map.get(key).is_some());
    if !exact {
        check.key("bindings", Rule::BindingsKeys);
    }
    let mut lists: Vec<Option<Vec<String>>> = Vec::new();
    for key in BINDING_KEYS {
        let Some(binding) = map.get(key) else {
            lists.push(None);
            continue;
        };
        if let Some(texts) = strings(binding.value()) {
            lists.push(Some(texts.into_iter().map(str::to_string).collect()));
        } else {
            check.key(
                "bindings",
                Rule::BindingNotAList {
                    binding: key.to_string(),
                },
            );
            lists.push(None);
        }
    }
    let bound = |index: usize| lists[index].as_ref().map_or(0, Vec::len);
    match state {
        Some(state @ (State::Designed | State::Considered)) => {
            for (index, key) in BINDING_KEYS.iter().enumerate() {
                if bound(index) > 0 {
                    check.key(
                        "bindings",
                        Rule::BindingNotEmpty {
                            state,
                            binding: (*key).to_string(),
                        },
                    );
                }
            }
        }
        Some(State::Delivered) => {
            for (index, key) in BINDING_KEYS.iter().enumerate().take(2) {
                if bound(index) == 0 && lists[index].is_some() {
                    check.key(
                        "bindings",
                        Rule::BindingRequired {
                            binding: (*key).to_string(),
                        },
                    );
                }
            }
        }
        Some(State::Implementing) | None => {}
    }
    let mut lists = lists.into_iter();
    match (exact, lists.next(), lists.next(), lists.next()) {
        (true, Some(Some(code)), Some(Some(tests)), Some(Some(examples))) => Some(Bindings {
            code,
            tests,
            examples,
        }),
        _ => None,
    }
}

fn check_capability_ids(
    check: &mut Check<'_>,
    entry: &Entry,
    non_empty: bool,
) -> Option<Vec<String>> {
    let key = entry.key();
    let Some(ids) = strings(entry.value()) else {
        check.key(
            key,
            Rule::WrongShape {
                expected: "a list of cap ids",
            },
        );
        return None;
    };
    let mut valid = true;
    if non_empty && ids.is_empty() {
        check.key(key, Rule::EmptyChanges);
        valid = false;
    }
    for id in &ids {
        if !is_capability_id(id) {
            check.key(
                key,
                Rule::BadCapabilityId {
                    entry: (*id).to_string(),
                },
            );
            valid = false;
        }
    }
    valid.then(|| ids.into_iter().map(str::to_string).collect())
}

fn check_affects(check: &mut Check<'_>, entry: &Entry, changes: &[String]) -> Option<Vec<Affects>> {
    let listed: HashSet<&str> = changes
        .iter()
        .inspect(|_| add(Work::ChangeIdsSet, 1))
        .map(String::as_str)
        .collect();
    let Some(texts) = strings(entry.value()) else {
        check.key(
            "affects",
            Rule::WrongShape {
                expected: "a list of strings",
            },
        );
        return None;
    };
    let mut parsed = Vec::new();
    let mut valid = true;
    for text in texts {
        add(Work::AffectsEntries, 1);
        let split = text
            .split_once('#')
            .filter(|(capability, _)| is_capability_id(capability));
        let Some((capability, section_slug)) = split else {
            check.key(
                "affects",
                Rule::BadAffects {
                    entry: text.to_string(),
                },
            );
            valid = false;
            continue;
        };
        let section = CapabilitySection::from_slug(section_slug);
        if section.is_none() {
            check.key(
                "affects",
                Rule::BadAffectsSection {
                    entry: text.to_string(),
                    section: section_slug.to_string(),
                },
            );
            valid = false;
        }
        add(Work::ListedLookups, 1);
        if !listed.contains(capability) {
            check.key(
                "affects",
                Rule::AffectsNotInChanges {
                    entry: text.to_string(),
                },
            );
            valid = false;
        }
        if let Some(section) = section {
            parsed.push(Affects {
                capability: capability.to_string(),
                section,
            });
        }
    }
    valid.then_some(parsed)
}

fn check_effect(check: &mut Check<'_>, entry: &Entry) -> Option<ContractEffect> {
    let effect = match scalar_text(entry) {
        Some("none") => Some(ContractEffect::None),
        Some("changes") => Some(ContractEffect::Changes),
        _ => None,
    };
    if effect.is_none() {
        let found = scalar_text(entry).map_or("<not a scalar>", |text| text);
        check.key(
            "contract_effect",
            Rule::BadContractEffect {
                found: found.to_string(),
            },
        );
    }
    effect
}

fn check_items(check: &mut Check<'_>, entry: &Entry) -> Option<Vec<ChangeItem>> {
    let shape = Rule::WrongShape {
        expected: "a list of maps with `claim` and optional `outcome`",
    };
    let Some(list) = entry.value().as_list() else {
        check.key("items", shape);
        return None;
    };
    let mut items = Vec::new();
    let mut valid = true;
    for item in list.items() {
        let ItemValue::Map(map) = item.value() else {
            check.key("items", shape.clone());
            return None;
        };
        let mut claim = None;
        for field in map.entries() {
            if !matches!(field.key(), "claim" | "outcome" | "reason") {
                check.key(
                    "items",
                    Rule::ItemUnknownKey {
                        key: field.key().to_string(),
                    },
                );
                valid = false;
            }
        }
        match map.get("claim").and_then(scalar_text) {
            Some(text) if !text.trim().is_empty() => claim = Some(text.to_string()),
            _ => {
                check.key("items", Rule::ItemClaimEmpty);
                valid = false;
            }
        }
        let mut outcome = None;
        if let Some(entry) = map.get("outcome") {
            let found = scalar_text(entry).map_or("<not a scalar>", |text| text);
            outcome = match found {
                "implemented" => Some(Outcome::Implemented),
                "dropped" => Some(Outcome::Dropped),
                other => other
                    .strip_prefix("moved:change:")
                    .filter(|slug| is_kebab(slug))
                    .map(|slug| Outcome::Moved(slug.to_string())),
            };
            if outcome.is_none() {
                check.key(
                    "items",
                    Rule::BadOutcome {
                        found: found.to_string(),
                    },
                );
                valid = false;
            }
        }
        // A reason is accepted beside any outcome or none: an author may write
        // it before the outcome. Whether an outcome needs one is a rule of
        // strict mode, not of the profile.
        let mut reason = None;
        if let Some(entry) = map.get("reason") {
            match scalar_text(entry) {
                Some(text) if !text.trim().is_empty() => reason = Some(text.to_string()),
                _ => {
                    check.key("items", Rule::ItemReasonEmpty);
                    valid = false;
                }
            }
        }
        if let Some(claim) = claim {
            items.push(ChangeItem {
                claim,
                outcome,
                reason,
            });
        }
    }
    valid.then_some(items)
}

/// The placeholder rule over the frontmatter values a reader sees as text.
fn check_frontmatter_placeholders(check: &mut Check<'_>, map: &Map) {
    let mut texts: Vec<(&str, &str)> = Vec::new();
    for key in ["title", "description"] {
        if let Some(text) = map.get(key).and_then(scalar_text) {
            texts.push((key, text));
        }
    }
    // The prose of a list of maps: the claim and the reason of each change
    // item and the reason of each coverage exclusion.
    for (key, field) in [
        ("items", "claim"),
        ("items", "reason"),
        ("exclusions", "reason"),
        ("exceptions", "reason"),
    ] {
        if let Some(list) = map.get(key).and_then(|entry| entry.value().as_list()) {
            for item in list.items() {
                if let Some(text) = item
                    .value()
                    .as_map()
                    .and_then(|item| item.get(field))
                    .and_then(scalar_text)
                {
                    texts.push((key, text));
                }
            }
        }
    }
    for (key, text) in texts {
        // Code spans are spared, as in the prose scan.
        let prose = crate::body::remove_code_spans(text);
        for token in placeholders(&prose) {
            check.key(
                key,
                Rule::Placeholder {
                    token: token.to_string(),
                },
            );
        }
    }
}

fn build(doc_type: DocType, found: Collected) -> Option<RafDocument> {
    if found.between_broken {
        return None;
    }
    if doc_type.is_record() {
        return build_record(doc_type, found);
    }
    let Collected {
        id,
        title,
        description,
        tags,
        timestamp,
        state,
        bindings,
        depends,
        review_required,
        changes,
        affects,
        effect,
        items,
        diagrams,
        ..
    } = found;
    let (id, slug) = id?;
    let (Ok(tags), Ok(timestamp)) = (tags.settle(), timestamp.settle()) else {
        return None;
    };
    let header = Header {
        id,
        slug,
        title: title?,
        description: description?,
        tags,
        timestamp,
    };
    let diagrams = diagrams.settle().ok()?.unwrap_or_default();
    match doc_type {
        DocType::Capability => Some(RafDocument::Capability(Capability {
            header,
            state: state?,
            bindings: bindings?,
            depends: depends?,
            review_required: review_required.settle().ok()?.unwrap_or_default(),
            diagrams,
        })),
        DocType::Change => Some(RafDocument::Change(Change {
            header,
            changes: changes?,
            affects: affects?,
            contract_effect: effect?,
            items: items?,
        })),
        DocType::Architecture => Some(RafDocument::Architecture(Architecture { header, diagrams })),
        // A record was built above.
        DocType::Evidence
        | DocType::Assessment
        | DocType::Coverage
        | DocType::Debt
        | DocType::Review => None,
    }
}

fn build_record(doc_type: DocType, found: Collected) -> Option<RafDocument> {
    let Collected {
        id,
        tags,
        timestamp,
        subject,
        inputs,
        verdict,
        by,
        covers,
        command,
        revision,
        values,
        limits,
        scope,
        exclusions,
        ceilings,
        exceptions,
        debt_findings,
        debt_lines,
        review_scope,
        review_kind,
        review_findings,
        review_reference,
        ..
    } = found;
    let header = record_header(id, tags, timestamp)?;
    match (doc_type, subject, verdict) {
        (
            DocType::Evidence,
            Some(records::Subject::Evidence(subject)),
            Some(records::Verdict::Evidence(verdict)),
        ) => {
            let (Ok(command), Ok(revision), Ok(values), Ok(limits)) = (
                command.settle(),
                revision.settle(),
                values.settle(),
                limits.settle(),
            ) else {
                return None;
            };
            Some(RafDocument::Evidence(EvidenceRecord {
                header,
                subject,
                inputs: inputs?,
                verdict,
                by: by?,
                covers: covers?,
                command,
                revision,
                values: values.unwrap_or_default(),
                limits: limits.unwrap_or_default(),
            }))
        }
        (
            DocType::Assessment,
            Some(records::Subject::Section(subject)),
            Some(records::Verdict::Assessment(verdict)),
        ) => Some(RafDocument::Assessment(AssessmentRecord {
            header,
            subject,
            inputs: inputs?,
            verdict,
            by: by?,
        })),
        (DocType::Coverage, _, _) => Some(RafDocument::Coverage(CoverageRecord {
            header,
            scope: scope?,
            exclusions: exclusions?,
            ceilings: ceilings.settle().ok()?.unwrap_or_default(),
            exceptions: exceptions.settle().ok()?.unwrap_or_default(),
        })),
        (DocType::Debt, _, _) => Some(RafDocument::Debt(DebtRecord {
            header,
            findings: debt_findings?,
            lines: debt_lines?,
        })),
        (DocType::Review, _, Some(records::Verdict::Review(verdict))) => {
            header.timestamp.as_ref()?;
            Some(RafDocument::Review(ReviewRecord {
                header,
                scope: review_scope?,
                revision: revision.settle().ok()??,
                kind: review_kind?,
                verdict,
                by: by?,
                inputs: inputs?,
                findings: review_findings?,
                findings_ref: review_reference.settle().ok()?,
            }))
        }
        // A value that failed its check, or a document type with no record.
        _ => None,
    }
}

fn record_header(
    id: Option<(String, String)>,
    tags: Slot<Vec<String>>,
    timestamp: Slot<String>,
) -> Option<RecordHeader> {
    let (id, slug) = id?;
    Some(RecordHeader {
        id,
        slug,
        tags: tags.settle().ok()?,
        timestamp: timestamp.settle().ok()?,
    })
}
