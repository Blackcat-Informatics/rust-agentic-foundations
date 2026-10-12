// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The RAF profile: the schema table, the typed form of a valid document and
//! the validator that reports every violation.

mod body_rules;
mod schema;
mod timestamp;
mod typed;
mod violation;

use std::collections::HashSet;

use crate::document::Document;
use crate::frontmatter::value::{Entry, ItemValue, Map, Scalar, Value};

pub(crate) use body_rules::placeholders;
pub use schema::{DocType, KeySpec, ValueKind};
pub use timestamp::{TimestampError, validate_timestamp};
pub use typed::{
    Affects, Architecture, Bindings, Capability, CapabilitySection, Change, ChangeItem,
    ContractEffect, Header, Outcome, RafDocument, State,
};
pub use violation::{Location, Rule, Violation};

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
    changes: Option<Vec<String>>,
    /// The valid capability ids of `changes`, even when another entry is not.
    change_ids: Vec<String>,
    affects: Option<Vec<Affects>>,
    effect: Option<ContractEffect>,
    items: Option<Vec<ChangeItem>>,
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
            ValueKind::Bindings => {
                found.bindings = check_bindings(check, entry, found.state);
            }
            ValueKind::CapabilityIds => {
                let ids = check_capability_ids(check, entry, key == "changes");
                if key == "changes" {
                    if let Some(texts) = strings(entry.value()) {
                        found.change_ids = texts
                            .into_iter()
                            .filter(|id| is_capability_id(id))
                            .map(str::to_string)
                            .collect();
                    }
                    found.changes = ids;
                } else {
                    found.depends = ids;
                }
            }
            ValueKind::Affects => {
                found.affects = check_affects(check, entry, &found.change_ids);
            }
            ValueKind::ContractEffect => found.effect = check_effect(check, entry),
            ValueKind::Items => found.items = check_items(check, entry),
            // `type` was read first.
            ValueKind::DocType => {}
        }
    }
    found
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
    let prefix = doc_type.id_prefix();
    let Some(slug) = id.strip_prefix(prefix).filter(|slug| is_kebab(slug)) else {
        check.key("id", Rule::InvalidId { prefix });
        return None;
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
    let listed: HashSet<&str> = changes.iter().map(String::as_str).collect();
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
            if field.key() != "claim" && field.key() != "outcome" {
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
        if let Some(claim) = claim {
            items.push(ChangeItem { claim, outcome });
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
    if let Some(list) = map.get("items").and_then(|entry| entry.value().as_list()) {
        for item in list.items() {
            if let Some(text) = item
                .value()
                .as_map()
                .and_then(|item| item.get("claim"))
                .and_then(scalar_text)
            {
                texts.push(("items", text));
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
    let Collected {
        id,
        title,
        description,
        tags,
        timestamp,
        state,
        bindings,
        depends,
        changes,
        change_ids: _,
        affects,
        effect,
        items,
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
    match doc_type {
        DocType::Capability => Some(RafDocument::Capability(Capability {
            header,
            state: state?,
            bindings: bindings?,
            depends: depends?,
        })),
        DocType::Change => Some(RafDocument::Change(Change {
            header,
            changes: changes?,
            affects: affects?,
            contract_effect: effect?,
            items: items?,
        })),
        DocType::Architecture => Some(RafDocument::Architecture(Architecture { header })),
    }
}
