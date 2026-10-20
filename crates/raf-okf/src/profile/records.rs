// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The value rules of the record types: evidence, assessments and coverage.

use std::collections::{HashMap, HashSet};

use crate::frontmatter::value::{Entry, ItemValue, Map, Scalar};
use crate::profile::schema::DocType;
use crate::profile::typed::{
    Actor, AssessmentVerdict, CapabilitySection, Ceiling, CeilingClass, CeilingLimit,
    EvidenceSubject, EvidenceVerdict, Exception, ExceptionTarget, Exclusion, ExclusionItem, Input,
    InputKind, Measure, Number, SectionRef,
};
use crate::profile::violation::{Rule, input_kind_allowed};
use crate::profile::{Check, is_capability_id, is_kebab, scalar_text};
use crate::work::{Work, add};

/// The keys of a row of `inputs`, in writing order.
const INPUT_KEYS: [&str; 3] = ["kind", "name", "digest"];

/// The keys of a row of `exclusions`, in writing order.
const EXCLUSION_KEYS: [&str; 2] = ["item", "reason"];

/// The subject of a record, by type.
#[derive(Debug, Clone)]
pub(super) enum Subject {
    Evidence(EvidenceSubject),
    Section(SectionRef),
}

/// The verdict of a record, by type.
#[derive(Debug, Clone, Copy)]
pub(super) enum Verdict {
    Evidence(EvidenceVerdict),
    Assessment(AssessmentVerdict),
}

/// Whether `text` is an identifier of a path segment: an ASCII letter or `_`
/// first, then letters, digits and `_`, and not `_` alone.
fn is_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && text != "_"
}

/// The number of segments of a path of identifiers joined by `::`, or `None`
/// when `text` is not such a path.
fn path_segments(text: &str) -> Option<usize> {
    let mut count = 0;
    for segment in text.split("::") {
        if !is_identifier(segment) {
            return None;
        }
        count += 1;
    }
    Some(count)
}

/// A path below a crate root, such as `raf_okf::Body`: two segments or more.
fn is_item_path(text: &str) -> bool {
    path_segments(text).is_some_and(|count| count >= 2)
}

/// `count` lowercase hexadecimal digits.
fn is_lower_hex(text: &str, count: usize) -> bool {
    text.len() == count
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// A SHA-256 digest: 64 lowercase hexadecimal digits.
fn is_digest(text: &str) -> bool {
    is_lower_hex(text, 64)
}

/// A Git commit id: 40 (SHA-1) or 64 (SHA-256) lowercase hexadecimal digits.
fn is_commit(text: &str) -> bool {
    is_lower_hex(text, 40) || is_lower_hex(text, 64)
}

/// A crate package name: an ASCII letter, then letters, digits, `-` and `_`.
fn is_package_name(text: &str) -> bool {
    let mut chars = text.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// `cap:<slug>#<section>` with one of the seven sections.
pub(super) fn section_ref(text: &str) -> Option<SectionRef> {
    let (capability, slug) = text.split_once('#')?;
    if !is_capability_id(capability) {
        return None;
    }
    Some(SectionRef {
        capability: capability.to_string(),
        section: CapabilitySection::from_slug(slug)?,
    })
}

/// The scalar text of an entry, or a `WrongShape` violation.
fn scalar_of<'e>(check: &mut Check<'_>, entry: &'e Entry) -> Option<&'e str> {
    let text = scalar_text(entry);
    if text.is_none() {
        check.key(
            entry.key(),
            Rule::WrongShape {
                expected: "a scalar",
            },
        );
    }
    text
}

pub(super) fn check_subject(
    check: &mut Check<'_>,
    entry: &Entry,
    doc_type: DocType,
) -> Option<Subject> {
    let text = scalar_of(check, entry)?;
    let subject = match doc_type {
        DocType::Evidence => {
            if text == "command" {
                Some(Subject::Evidence(EvidenceSubject::Command))
            } else {
                text.strip_prefix("binding:")
                    .filter(|path| is_item_path(path))
                    .map(|path| Subject::Evidence(EvidenceSubject::Binding(path.to_string())))
            }
        }
        _ => section_ref(text).map(Subject::Section),
    };
    if subject.is_none() {
        check.key(
            "subject",
            Rule::BadSubject {
                doc_type,
                found: text.to_string(),
            },
        );
    }
    subject
}

/// Whether `name` is of the form an input of `kind` names.
fn input_name_fits(kind: InputKind, name: &str) -> bool {
    match kind {
        InputKind::Section => section_ref(name).is_some(),
        InputKind::Symbol => is_item_path(name),
        InputKind::Record => name.strip_prefix("evidence:").is_some_and(is_kebab),
    }
}

/// The keys of a row outside `keys`, and the keys of `keys` it lacks, as
/// violations; whether the row holds exactly `keys`.
fn row_keys(
    check: &mut Check<'_>,
    key: &str,
    list: &'static str,
    row: &Map,
    keys: &[&'static str],
) -> bool {
    let mut exact = true;
    for field in row.entries() {
        if !keys.contains(&field.key()) {
            check.key(
                key,
                Rule::RowUnknownKey {
                    list,
                    key: field.key().to_string(),
                },
            );
            exact = false;
        }
    }
    for wanted in keys {
        if row.get(wanted).is_none() {
            check.key(key, Rule::RowMissingKey { list, key: wanted });
            exact = false;
        }
    }
    exact
}

/// The rows of a list of maps, or a `WrongShape` violation. An empty flow
/// list `[]` holds no rows.
fn rows<'e>(
    check: &mut Check<'_>,
    entry: &'e Entry,
    expected: &'static str,
) -> Option<Vec<&'e Map>> {
    let Some(list) = entry.value().as_list() else {
        check.key(entry.key(), Rule::WrongShape { expected });
        return None;
    };
    let mut maps = Vec::new();
    for item in list.items() {
        let ItemValue::Map(map) = item.value() else {
            check.key(entry.key(), Rule::WrongShape { expected });
            return None;
        };
        maps.push(map);
    }
    Some(maps)
}

pub(super) fn check_inputs(
    check: &mut Check<'_>,
    entry: &Entry,
    doc_type: DocType,
) -> Option<Vec<Input>> {
    let rows = rows(
        check,
        entry,
        "a list of maps with `kind`, `name` and `digest`",
    )?;
    let mut valid = true;
    if rows.is_empty() {
        check.key("inputs", Rule::EmptyList { list: "inputs" });
        valid = false;
    }
    let mut inputs = Vec::new();
    let mut seen = HashSet::new();
    for row in rows {
        if !row_keys(check, "inputs", "inputs", row, &INPUT_KEYS) {
            valid = false;
            continue;
        }
        let field = |key: &str| row.get(key).and_then(scalar_text);
        let (Some(kind_text), Some(name), Some(digest)) =
            (field("kind"), field("name"), field("digest"))
        else {
            check.key(
                "inputs",
                Rule::WrongShape {
                    expected: "a list of maps of scalars",
                },
            );
            valid = false;
            continue;
        };
        let kind =
            InputKind::from_name(kind_text).filter(|kind| input_kind_allowed(doc_type, *kind));
        let Some(kind) = kind else {
            check.key(
                "inputs",
                Rule::BadInputKind {
                    doc_type,
                    found: kind_text.to_string(),
                },
            );
            valid = false;
            continue;
        };
        if !input_name_fits(kind, name) {
            check.key(
                "inputs",
                Rule::BadInputName {
                    kind,
                    found: name.to_string(),
                },
            );
            valid = false;
        }
        if !is_digest(digest) {
            check.key(
                "inputs",
                Rule::BadDigest {
                    found: digest.to_string(),
                },
            );
            valid = false;
        }
        if !seen.insert((kind, name)) {
            check.key(
                "inputs",
                Rule::DuplicateEntry {
                    list: "inputs",
                    entry: format!("{} {name}", kind.name()),
                },
            );
            valid = false;
        }
        inputs.push(Input {
            kind,
            name: name.to_string(),
            digest: digest.to_string(),
        });
    }
    valid.then_some(inputs)
}

pub(super) fn check_verdict(
    check: &mut Check<'_>,
    entry: &Entry,
    doc_type: DocType,
) -> Option<Verdict> {
    let text = scalar_of(check, entry)?;
    let verdict = match (doc_type, text) {
        (DocType::Evidence, "pass") => Some(Verdict::Evidence(EvidenceVerdict::Pass)),
        (DocType::Evidence, "fail") => Some(Verdict::Evidence(EvidenceVerdict::Fail)),
        (DocType::Assessment, "accepted") => Some(Verdict::Assessment(AssessmentVerdict::Accepted)),
        (DocType::Assessment, "rejected") => Some(Verdict::Assessment(AssessmentVerdict::Rejected)),
        _ => None,
    };
    if verdict.is_none() {
        check.key(
            "verdict",
            Rule::BadVerdict {
                doc_type,
                found: text.to_string(),
            },
        );
    }
    verdict
}

pub(super) fn check_actor(check: &mut Check<'_>, entry: &Entry) -> Option<Actor> {
    let text = scalar_of(check, entry)?;
    let actor = Actor::from_name(text);
    if actor.is_none() {
        check.key(
            "by",
            Rule::BadActor {
                found: text.to_string(),
            },
        );
    }
    actor
}

pub(super) fn check_command(check: &mut Check<'_>, entry: &Entry) -> Option<String> {
    match scalar_text(entry) {
        Some(text) if !text.trim().is_empty() => Some(text.to_string()),
        _ => {
            check.key("command", Rule::EmptyText);
            None
        }
    }
}

pub(super) fn check_revision(check: &mut Check<'_>, entry: &Entry) -> Option<String> {
    let text = scalar_of(check, entry)?;
    if is_commit(text) {
        Some(text.to_string())
    } else {
        check.key(
            "revision",
            Rule::BadRevision {
                found: text.to_string(),
            },
        );
        None
    }
}

pub(super) fn check_measures(check: &mut Check<'_>, entry: &Entry) -> Option<Vec<Measure>> {
    let list: &'static str = if entry.key() == "limits" {
        "limits"
    } else {
        "values"
    };
    let Some(map) = entry.value().as_map() else {
        check.key(
            list,
            Rule::WrongShape {
                expected: "a map of names to numbers",
            },
        );
        return None;
    };
    let mut measures = Vec::new();
    let mut valid = true;
    for field in map.entries() {
        let found = scalar_text(field);
        if let Some(value) = found.and_then(Number::parse) {
            measures.push(Measure {
                name: field.key().to_string(),
                value,
            });
        } else {
            check.key(
                list,
                Rule::BadMeasure {
                    list,
                    name: field.key().to_string(),
                    found: found.map_or("<not a scalar>", |text| text).to_string(),
                },
            );
            valid = false;
        }
    }
    valid.then_some(measures)
}

pub(super) fn check_scope(check: &mut Check<'_>, entry: &Entry) -> Option<Vec<String>> {
    let Some(list) = entry.value().as_list() else {
        check.key(
            "scope",
            Rule::WrongShape {
                expected: "a list of crate package names",
            },
        );
        return None;
    };
    let mut names = Vec::new();
    let mut valid = true;
    let mut seen = HashSet::new();
    for item in list.items() {
        let Some(name) = item.value().as_scalar().map(Scalar::text) else {
            check.key(
                "scope",
                Rule::WrongShape {
                    expected: "a list of crate package names",
                },
            );
            return None;
        };
        if !is_package_name(name) {
            check.key(
                "scope",
                Rule::BadCrateName {
                    entry: name.to_string(),
                },
            );
            valid = false;
        } else if !seen.insert(name) {
            check.key(
                "scope",
                Rule::DuplicateEntry {
                    list: "scope",
                    entry: name.to_string(),
                },
            );
            valid = false;
        }
        names.push(name.to_string());
    }
    valid.then_some(names)
}

/// The item of an exclusion, or the rule it breaks.
fn exclusion_item(text: &str) -> Result<ExclusionItem, Rule> {
    let (path, prefix) = match text.strip_suffix("::*") {
        Some(path) => (path, true),
        None => (text, false),
    };
    match path_segments(path) {
        Some(1) => Err(Rule::CrateRootExclusion {
            found: text.to_string(),
        }),
        Some(_) if prefix => Ok(ExclusionItem::Prefix(path.to_string())),
        Some(_) => Ok(ExclusionItem::Item(path.to_string())),
        None => Err(Rule::BadExclusionItem {
            found: text.to_string(),
        }),
    }
}

pub(super) fn check_exclusions(check: &mut Check<'_>, entry: &Entry) -> Option<Vec<Exclusion>> {
    let rows = rows(check, entry, "a list of maps with `item` and `reason`")?;
    let mut exclusions = Vec::new();
    let mut valid = true;
    let mut seen = HashSet::new();
    for row in rows {
        if !row_keys(check, "exclusions", "exclusions", row, &EXCLUSION_KEYS) {
            valid = false;
            continue;
        }
        let field = |key: &str| row.get(key).and_then(scalar_text);
        let (Some(item_text), Some(reason)) = (field("item"), field("reason")) else {
            check.key(
                "exclusions",
                Rule::WrongShape {
                    expected: "a list of maps of scalars",
                },
            );
            valid = false;
            continue;
        };
        let item = match exclusion_item(item_text) {
            Ok(item) => Some(item),
            Err(rule) => {
                check.key("exclusions", rule);
                valid = false;
                None
            }
        };
        if reason.trim().is_empty() {
            check.key(
                "exclusions",
                Rule::EmptyReason {
                    item: item_text.to_string(),
                },
            );
            valid = false;
        }
        if !seen.insert(item_text) {
            check.key(
                "exclusions",
                Rule::DuplicateEntry {
                    list: "exclusions",
                    entry: item_text.to_string(),
                },
            );
            valid = false;
        }
        if let Some(item) = item {
            exclusions.push(Exclusion {
                item,
                reason: reason.to_string(),
            });
        }
    }
    valid.then_some(exclusions)
}

/// The keys a row of `ceilings` may hold.
const CEILING_KEYS: [&str; 4] = ["crate", "lines", "percent", "of"];

/// The keys a row of `exceptions` may hold.
const EXCEPTION_KEYS: [&str; 3] = ["item", "file", "reason"];

/// The keys of a row outside `allowed`, as violations; whether there are none.
fn known_keys(check: &mut Check<'_>, list: &'static str, row: &Map, allowed: &[&str]) -> bool {
    let mut known = true;
    for field in row.entries() {
        if !allowed.contains(&field.key()) {
            check.key(
                list,
                Rule::RowUnknownKey {
                    list,
                    key: field.key().to_string(),
                },
            );
            known = false;
        }
    }
    known
}

/// The limit of a ceiling row of `package`: exactly one of `lines`, a whole
/// number, and `percent`, a number from 0 to 100. A value that is not a
/// number is `BadMeasure`; any other shape is `BadCeiling`.
fn ceiling_limit(check: &mut Check<'_>, package: &str, row: &Map) -> Option<CeilingLimit> {
    let bad = |problem: String| Rule::BadCeiling {
        package: package.to_string(),
        problem,
    };
    let (key, text) = match (row.get("lines"), row.get("percent")) {
        (Some(_), Some(_)) => {
            check.key(
                "ceilings",
                bad("names both `lines` and `percent`; it names one of them".to_string()),
            );
            return None;
        }
        (None, None) => {
            check.key(
                "ceilings",
                bad("names neither `lines` nor `percent`".to_string()),
            );
            return None;
        }
        (Some(entry), None) => ("lines", scalar_text(entry)),
        (None, Some(entry)) => ("percent", scalar_text(entry)),
    };
    let Some(number) = text.and_then(Number::parse) else {
        check.key(
            "ceilings",
            Rule::BadMeasure {
                list: "ceilings",
                name: key.to_string(),
                found: text.map_or("<not a scalar>", |text| text).to_string(),
            },
        );
        return None;
    };
    let digits = |text: &str| !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit());
    if key == "lines" {
        if digits(number.text()) {
            return Some(CeilingLimit::Lines(number));
        }
        check.key(
            "ceilings",
            bad(format!(
                "has `lines: {}`, which is not a whole number",
                number.text()
            )),
        );
        return None;
    }
    let hundred = Number::parse("100").filter(|_| !number.text().starts_with('-'));
    if hundred.is_some_and(|hundred| number.compare(&hundred) != std::cmp::Ordering::Greater) {
        return Some(CeilingLimit::Percent(number));
    }
    check.key(
        "ceilings",
        bad(format!(
            "has `percent: {}`, which is not from 0 to 100",
            number.text()
        )),
    );
    None
}

pub(super) fn check_ceilings(check: &mut Check<'_>, entry: &Entry) -> Option<Vec<Ceiling>> {
    let rows = rows(
        check,
        entry,
        "a list of maps with `crate`, `lines` or `percent`, and an optional `of`",
    )?;
    let mut ceilings = Vec::new();
    let mut valid = true;
    let mut seen = HashSet::new();
    for row in rows {
        valid &= known_keys(check, "ceilings", row, &CEILING_KEYS);
        let Some(package_entry) = row.get("crate") else {
            check.key(
                "ceilings",
                Rule::RowMissingKey {
                    list: "ceilings",
                    key: "crate",
                },
            );
            valid = false;
            continue;
        };
        let Some(package) = scalar_text(package_entry) else {
            check.key(
                "ceilings",
                Rule::WrongShape {
                    expected: "a list of maps of scalars",
                },
            );
            valid = false;
            continue;
        };
        if !is_package_name(package) {
            check.key(
                "ceilings",
                Rule::BadCrateName {
                    entry: package.to_string(),
                },
            );
            valid = false;
        }
        let of = match row.get("of").map(scalar_text) {
            None => Some(CeilingClass::All),
            Some(text) => {
                let class = text.and_then(CeilingClass::from_name);
                if class.is_none() {
                    check.key(
                        "ceilings",
                        Rule::BadCeiling {
                            package: package.to_string(),
                            problem: format!(
                                "has `of: {}`, which is not `shipping`, `test` or `all`",
                                text.unwrap_or("<not a scalar>")
                            ),
                        },
                    );
                }
                class
            }
        };
        let limit = ceiling_limit(check, package, row);
        let (Some(of), Some(limit)) = (of, limit) else {
            valid = false;
            continue;
        };
        if !seen.insert((package, of)) {
            check.key(
                "ceilings",
                Rule::DuplicateEntry {
                    list: "ceilings",
                    entry: format!("{package} of {}", of.name()),
                },
            );
            valid = false;
        }
        ceilings.push(Ceiling::new(package, limit, of));
    }
    valid.then_some(ceilings)
}

/// The `file` of an exception: a repository-relative path of `/`-separated
/// segments, none empty, `.` or `..`, or such a path and a trailing `/` for a
/// directory.
fn exception_file(text: &str) -> Result<ExceptionTarget, &'static str> {
    if text.starts_with('/') || text.contains('\\') || text.as_bytes().get(1) == Some(&b':') {
        return Err("is an absolute path; it is a path relative to the repository root");
    }
    let (path, directory) = match text.strip_suffix('/') {
        Some(path) => (path, true),
        None => (text, false),
    };
    let mut segments = path.split('/');
    if segments.clone().any(|segment| segment == "..") {
        return Err("holds a `..` segment; it is a path below the repository root");
    }
    if path.is_empty() || segments.any(|segment| segment.is_empty() || segment == ".") {
        return Err("is not a repository-relative path of named segments");
    }
    Ok(if directory {
        ExceptionTarget::Directory(path.to_string())
    } else {
        ExceptionTarget::File(path.to_string())
    })
}

/// The `item` of an exception: an item path or a `<module>::*` prefix below a
/// crate root.
fn exception_item(text: &str) -> Result<ExceptionTarget, &'static str> {
    match exclusion_item(text) {
        Ok(ExclusionItem::Item(path)) => Ok(ExceptionTarget::Item(path)),
        Ok(ExclusionItem::Prefix(path)) => Ok(ExceptionTarget::Prefix(path)),
        Err(Rule::CrateRootExclusion { .. }) => {
            Err("names a whole crate; a crate's lines are bounded by its ceiling")
        }
        Err(_) => Err("is neither an item path nor a `<module>::*` prefix"),
    }
}

pub(super) fn check_exceptions(check: &mut Check<'_>, entry: &Entry) -> Option<Vec<Exception>> {
    let rows = rows(
        check,
        entry,
        "a list of maps with `item` or `file`, and `reason`",
    )?;
    let mut exceptions = Vec::new();
    let mut valid = true;
    let mut seen = HashSet::new();
    for row in rows {
        valid &= known_keys(check, "exceptions", row, &EXCEPTION_KEYS);
        let field = |key: &str| row.get(key).map(scalar_text);
        let Some(reason) = field("reason") else {
            check.key(
                "exceptions",
                Rule::RowMissingKey {
                    list: "exceptions",
                    key: "reason",
                },
            );
            valid = false;
            continue;
        };
        let (named, target) = match (field("item"), field("file")) {
            (Some(Some(item)), None) => (item, exception_item(item)),
            (None, Some(Some(file))) => (file, exception_file(file)),
            (Some(_), Some(_)) => (
                "",
                Err("names both `item` and `file`; it names one of them"),
            ),
            (None, None) => ("", Err("names neither `item` nor `file`")),
            (Some(None), None) | (None, Some(None)) => {
                check.key(
                    "exceptions",
                    Rule::WrongShape {
                        expected: "a list of maps of scalars",
                    },
                );
                valid = false;
                continue;
            }
        };
        let Some(reason) = reason else {
            check.key(
                "exceptions",
                Rule::WrongShape {
                    expected: "a list of maps of scalars",
                },
            );
            valid = false;
            continue;
        };
        if reason.trim().is_empty() {
            check.key(
                "exceptions",
                Rule::EmptyReason {
                    item: named.to_string(),
                },
            );
            valid = false;
        }
        let target = match target {
            Ok(target) => target,
            Err(problem) => {
                check.key(
                    "exceptions",
                    Rule::BadException {
                        found: named.to_string(),
                        problem,
                    },
                );
                valid = false;
                continue;
            }
        };
        if !seen.insert(target.written()) {
            check.key(
                "exceptions",
                Rule::DuplicateEntry {
                    list: "exceptions",
                    entry: target.written(),
                },
            );
            valid = false;
        }
        exceptions.push(Exception::new(target, reason));
    }
    valid.then_some(exceptions)
}

/// The capability ids of `for`: at least one, and none twice. The entries are
/// cap ids already, or `BadCapabilityId` was reported.
pub(super) fn check_covers(check: &mut Check<'_>, ids: &[String]) -> bool {
    let mut valid = true;
    if ids.is_empty() {
        check.key("for", Rule::EmptyList { list: "for" });
        valid = false;
    }
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id.as_str()) {
            check.key(
                "for",
                Rule::DuplicateEntry {
                    list: "for",
                    entry: id.clone(),
                },
            );
            valid = false;
        }
    }
    valid
}

/// What the rules between the keys of a record read.
pub(super) struct Keys<'a> {
    pub(super) subject: Option<&'a Subject>,
    pub(super) inputs: Option<&'a [Input]>,
    pub(super) verdict: Option<Verdict>,
    /// Whether the `command` key is present, valid or not.
    pub(super) has_command: bool,
    /// `None` when the key is present and invalid.
    pub(super) values: Option<&'a [Measure]>,
    /// `None` when the key is present and invalid.
    pub(super) limits: Option<&'a [Measure]>,
}

/// The rules between the keys of a record. Each runs only when the keys it
/// reads are valid, so one fault is reported once.
pub(super) fn check_between(check: &mut Check<'_>, keys: &Keys<'_>) -> bool {
    let mut valid = true;
    match keys.subject {
        Some(Subject::Evidence(subject)) => {
            let wants_command = *subject == EvidenceSubject::Command;
            if wants_command != keys.has_command {
                check.key(
                    "subject",
                    Rule::CommandSubject {
                        has_command: keys.has_command,
                    },
                );
                valid = false;
            }
        }
        Some(Subject::Section(subject)) => {
            let text = subject.text();
            if let Some(inputs) = keys.inputs
                && !inputs
                    .iter()
                    .any(|input| input.kind == InputKind::Section && input.name == text)
            {
                check.key("inputs", Rule::SubjectInputMissing { subject: text });
                valid = false;
            }
        }
        None => {}
    }
    if let (Some(values), Some(limits)) = (keys.values, keys.limits) {
        // One lookup per limit, so a record with many measures is judged in
        // time linear in their number.
        let by_name: HashMap<&str, &Measure> = values
            .iter()
            .inspect(|_| add(Work::MeasuresIndexed, 1))
            .map(|value| (value.name.as_str(), value))
            .collect();
        for limit in limits {
            add(Work::LimitLookups, 1);
            let Some(value) = by_name.get(limit.name.as_str()) else {
                // A failing execution, such as one killed at its time limit,
                // may have printed none of its values.
                if matches!(keys.verdict, Some(Verdict::Evidence(EvidenceVerdict::Fail))) {
                    continue;
                }
                check.key(
                    "limits",
                    Rule::LimitWithoutValue {
                        name: limit.name.clone(),
                    },
                );
                valid = false;
                continue;
            };
            if matches!(keys.verdict, Some(Verdict::Evidence(EvidenceVerdict::Pass)))
                && value.value.exceeds(&limit.value)
            {
                check.key(
                    "verdict",
                    Rule::VerdictOverLimit {
                        name: limit.name.clone(),
                    },
                );
                valid = false;
            }
        }
    }
    valid
}
