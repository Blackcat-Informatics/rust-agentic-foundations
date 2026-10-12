// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Document-contract test for this repository.
//!
//! It enforces the RAF profile of OKF documents, stated in
//! `docs/architecture/okf-profile.md`, over the committed Markdown. The checker
//! is a pure function over an in-memory repository (relative path to contents)
//! so every rule has a negative test that breaks exactly one thing in a valid
//! fixture. Only the standard library is used.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Component, Path, PathBuf};

mod support;

use support::{banned_in_line, decode_text, gate_root, read_tracked, tracked_paths};

/// A repository held in memory: relative path to file contents.
type Repo = BTreeMap<PathBuf, String>;

const SECTIONS: [&str; 7] = [
    "Purpose",
    "Contract",
    "Usage",
    "Examples",
    "Constraints",
    "Explanation",
    "Verification",
];
const STATES: [&str; 4] = ["considered", "designed", "implementing", "delivered"];
const CAPABILITY_KEYS: [&str; 7] = [
    "type",
    "id",
    "title",
    "description",
    "state",
    "bindings",
    "depends",
];
const CHANGE_KEYS: [&str; 8] = [
    "type",
    "id",
    "title",
    "description",
    "changes",
    "affects",
    "contract_effect",
    "items",
];
const ARCHITECTURE_KEYS: [&str; 4] = ["type", "id", "title", "description"];
const BINDING_KEYS: [&str; 3] = ["code", "tests", "examples"];
const OUTCOMES: [&str; 2] = ["implemented", "dropped"];
const PROPOSED_OPEN: &str = "> [!PROPOSED change:";
const GENERATED_OPEN: &str = "<!-- raf:generated:";
const GENERATED_END: &str = "end";

// ---------------------------------------------------------------------------
// Frontmatter model and parser
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Scalar(String),
    List(Vec<String>),
    Map(BTreeMap<String, Vec<String>>),
    Items(Vec<BTreeMap<String, String>>),
}

#[derive(Debug)]
struct Document {
    fields: BTreeMap<String, Value>,
    body: String,
}

/// Split `key: rest` where the key is lowercase ASCII with underscores.
fn split_key(line: &str) -> Option<(&str, &str)> {
    let (key, rest) = line.split_once(':')?;
    let valid = !key.is_empty() && key.chars().all(|c| c.is_ascii_lowercase() || c == '_');
    (valid && (rest.is_empty() || rest.starts_with(' '))).then_some((key, rest))
}

fn unquote(raw: &str) -> String {
    let raw = raw.trim();
    if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
        return raw[1..raw.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\");
    }
    if raw.len() >= 2 && raw.starts_with('\'') && raw.ends_with('\'') {
        return raw[1..raw.len() - 1].replace("''", "'");
    }
    raw.to_string()
}

fn parse_inline_list(raw: &str) -> Result<Vec<String>, String> {
    let inner = raw
        .trim()
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .ok_or_else(|| format!("unterminated inline list `{raw}`"))?;
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for c in inner.chars() {
        match (quote, c) {
            (Some(q), _) if c == q => {
                quote = None;
                current.push(c);
            }
            (None, '"' | '\'') => {
                quote = Some(c);
                current.push(c);
            }
            (None, ',') => parts.push(std::mem::take(&mut current)),
            _ => current.push(c),
        }
    }
    parts.push(current);
    parts
        .into_iter()
        .map(|part| {
            let part = part.trim();
            if part.is_empty() {
                Err(format!("empty entry in inline list `{raw}`"))
            } else {
                Ok(unquote(part))
            }
        })
        .collect()
}

fn parse_sequence(block: &[&str]) -> Result<Value, String> {
    let mut scalars = Vec::new();
    let mut maps: Vec<BTreeMap<String, String>> = Vec::new();
    let mut in_map = false;
    for line in block {
        if let Some(item) = line.strip_prefix("  - ") {
            let item = item.trim();
            if let Some((key, rest)) = split_key(item) {
                maps.push(BTreeMap::from([(key.to_string(), unquote(rest))]));
                in_map = true;
            } else {
                scalars.push(unquote(item));
                in_map = false;
            }
        } else if let Some(continuation) = line.strip_prefix("    ") {
            let (key, rest) = split_key(continuation.trim())
                .ok_or_else(|| format!("cannot parse list entry line `{line}`"))?;
            let entry = maps
                .last_mut()
                .filter(|_| in_map)
                .ok_or_else(|| format!("continuation without a map entry `{line}`"))?;
            if entry.insert(key.to_string(), unquote(rest)).is_some() {
                return Err(format!("duplicate entry key `{key}`"));
            }
        } else {
            return Err(format!("cannot parse list line `{line}`"));
        }
    }
    match (scalars.is_empty(), maps.is_empty()) {
        (false, false) => Err("list mixes scalar and map entries".to_string()),
        (_, false) => Ok(Value::Items(maps)),
        _ => Ok(Value::List(scalars)),
    }
}

fn parse_map(block: &[&str]) -> Result<Value, String> {
    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in block {
        if let Some(item) = line.strip_prefix("    - ") {
            let key = current
                .as_ref()
                .ok_or_else(|| format!("list item without a key `{line}`"))?;
            map.entry(key.clone()).or_default().push(unquote(item));
        } else if let Some(entry) = line.strip_prefix("  ").filter(|e| !e.starts_with(' ')) {
            let (key, rest) =
                split_key(entry).ok_or_else(|| format!("cannot parse map line `{line}`"))?;
            if map.contains_key(key) {
                return Err(format!("duplicate map key `{key}`"));
            }
            let rest = rest.trim();
            if rest.is_empty() {
                map.insert(key.to_string(), Vec::new());
                current = Some(key.to_string());
            } else if rest.starts_with('[') {
                map.insert(key.to_string(), parse_inline_list(rest)?);
                current = None;
            } else {
                return Err(format!("map value of `{key}` must be a list"));
            }
        } else {
            return Err(format!("cannot parse map line `{line}`"));
        }
    }
    Ok(Value::Map(map))
}

fn parse_fields(front: &[&str]) -> Result<BTreeMap<String, Value>, String> {
    let mut fields = BTreeMap::new();
    let mut index = 0;
    while let Some(line) = front.get(index) {
        index += 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (key, rest) =
            split_key(line).ok_or_else(|| format!("cannot parse frontmatter line `{line}`"))?;
        if fields.contains_key(key) {
            return Err(format!("duplicate key `{key}`"));
        }
        let rest = rest.trim();
        let value = if rest.is_empty() {
            let mut block = Vec::new();
            while let Some(next) = front.get(index) {
                let t = next.trim();
                if t.is_empty() || t.starts_with('#') {
                    index += 1;
                } else if next.starts_with(' ') {
                    block.push(*next);
                    index += 1;
                } else {
                    break;
                }
            }
            match block.first() {
                None => Value::Scalar(String::new()),
                Some(first) if first.trim_start().starts_with("- ") => parse_sequence(&block)?,
                Some(_) => parse_map(&block)?,
            }
        } else if rest.starts_with('[') {
            Value::List(parse_inline_list(rest)?)
        } else {
            Value::Scalar(unquote(rest))
        };
        fields.insert(key.to_string(), value);
    }
    Ok(fields)
}

/// `None` when the text has no frontmatter, otherwise the parsed document.
fn parse_document(text: &str) -> Option<Result<Document, String>> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.first().map(|l| l.trim_end()) != Some("---") {
        return None;
    }
    let Some(close) = lines
        .iter()
        .skip(1)
        .position(|l| l.trim_end() == "---")
        .map(|p| p + 1)
    else {
        return Some(Err("frontmatter is not closed by a `---` line".to_string()));
    };
    let body = lines.get(close + 1..).unwrap_or_default().join("\n");
    let front = lines.get(1..close).unwrap_or_default();
    Some(parse_fields(front).map(|fields| Document { fields, body }))
}

// ---------------------------------------------------------------------------
// Markdown scanning helpers
// ---------------------------------------------------------------------------

/// Lines with their 1-based number and whether they are fence lines or sit
/// inside a fenced code block.
fn fenced_lines(text: &str) -> Vec<(usize, &str, bool)> {
    let mut in_fence = false;
    let mut out = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            out.push((index + 1, line, true));
        } else {
            out.push((index + 1, line, in_fence));
        }
    }
    out
}

fn heading_text(line: &str) -> Option<(usize, &str)> {
    let hashes = line.chars().take_while(|c| *c == '#').count();
    let rest = line.get(hashes..)?;
    ((1..=6).contains(&hashes) && rest.starts_with(' ')).then(|| (hashes, rest.trim()))
}

fn github_anchor(heading: &str) -> String {
    heading
        .trim_end_matches('#')
        .trim()
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            '-' | '_' => Some(c),
            c if c.is_alphanumeric() => Some(c.to_lowercase().next().unwrap_or(c)),
            _ => None,
        })
        .collect()
}

fn anchors(text: &str) -> BTreeSet<String> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut out = BTreeSet::new();
    for (_, line, fenced) in fenced_lines(text) {
        if fenced {
            continue;
        }
        if let Some((_, heading)) = heading_text(line) {
            let base = github_anchor(heading);
            let count = seen.entry(base.clone()).or_insert(0);
            out.insert(if *count == 0 {
                base
            } else {
                format!("{base}-{count}")
            });
            *count += 1;
        }
    }
    out
}

/// H2 sections in order: name and the raw text under each.
fn sections(body: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for (_, line, fenced) in fenced_lines(body) {
        let h2 = (!fenced)
            .then(|| heading_text(line))
            .flatten()
            .filter(|(level, _)| *level == 2);
        if let Some((_, name)) = h2 {
            out.push((name.to_string(), String::new()));
        } else if let Some((_, content)) = out.last_mut() {
            content.push_str(line);
            content.push('\n');
        }
    }
    out
}

fn is_kebab(slug: &str) -> bool {
    !slug.is_empty()
        && slug.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

// ---------------------------------------------------------------------------
// Repository checker
// ---------------------------------------------------------------------------

fn violation(path: &Path, message: impl std::fmt::Display) -> String {
    format!("{}: {message}", path.display())
}

fn is_scanned(path: &Path) -> bool {
    let markdown = path.extension().is_some_and(|e| e == "md");
    let mut components = path.components();
    let first = components.next();
    let top_level = components.next().is_none();
    markdown
        && (top_level || first.is_some_and(|c| c.as_os_str() == "docs" || c.as_os_str() == "work"))
}

/// The OKF type a document must declare because of where it lives.
fn expected_type(path: &Path) -> Option<&'static str> {
    let parts: Vec<&str> = path.iter().filter_map(|p| p.to_str()).collect();
    match parts.as_slice() {
        ["docs", "capabilities", _] => Some("Capability"),
        ["docs", "architecture", _] => Some("Architecture"),
        ["work", "changes", _] => Some("Change"),
        _ => None,
    }
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string()
}

fn scalar_field<'a>(doc: &'a Document, key: &str) -> Option<&'a str> {
    match doc.fields.get(key) {
        Some(Value::Scalar(s)) => Some(s.as_str()),
        _ => None,
    }
}

fn list_field<'a>(doc: &'a Document, key: &str) -> Option<&'a [String]> {
    match doc.fields.get(key) {
        Some(Value::List(l)) => Some(l.as_slice()),
        _ => None,
    }
}

struct Capability {
    path: PathBuf,
    id: String,
    state: String,
    depends: Vec<String>,
}

struct Change {
    path: PathBuf,
    slug: String,
    changes: Vec<String>,
}

#[derive(Default)]
struct Found {
    capabilities: Vec<Capability>,
    changes: Vec<Change>,
}

fn check_scalars(path: &Path, doc: &Document, keys: &[&str], out: &mut Vec<String>) {
    for key in ["title", "description"] {
        if keys.contains(&key) && doc.fields.contains_key(key) {
            match scalar_field(doc, key) {
                Some(s) if !s.trim().is_empty() => {}
                _ => out.push(violation(
                    path,
                    format!("`{key}` must be a non-empty string"),
                )),
            }
        }
    }
}

fn check_keys(path: &Path, doc: &Document, keys: &[&str], out: &mut Vec<String>) {
    for key in doc.fields.keys() {
        if !keys.contains(&key.as_str()) {
            out.push(violation(path, format!("unknown key `{key}`")));
        }
    }
    for key in keys {
        if !doc.fields.contains_key(*key) {
            out.push(violation(path, format!("missing required key `{key}`")));
        }
    }
}

/// Validate `id`, returning the slug when the id has the right form.
fn check_id(path: &Path, doc: &Document, prefix: &str, out: &mut Vec<String>) -> Option<String> {
    let id = scalar_field(doc, "id")?;
    let Some(slug) = id.strip_prefix(prefix).filter(|s| is_kebab(s)) else {
        out.push(violation(
            path,
            format!("id `{id}` must be `{prefix}<slug>` with a lowercase kebab slug"),
        ));
        return None;
    };
    if slug != file_stem(path) {
        out.push(violation(
            path,
            format!("id `{id}` does not match the file name"),
        ));
    }
    Some(slug.to_string())
}

fn check_section_layout(path: &Path, doc: &Document, out: &mut Vec<String>) {
    let found = sections(&doc.body);
    let names: Vec<&str> = found.iter().map(|(n, _)| n.as_str()).collect();
    for expected in SECTIONS {
        if !names.contains(&expected) {
            out.push(violation(path, format!("missing section `{expected}`")));
        }
    }
    for name in &names {
        if !SECTIONS.contains(name) {
            out.push(violation(path, format!("unexpected section `{name}`")));
        }
    }
    let known: Vec<&str> = names
        .iter()
        .copied()
        .filter(|n| SECTIONS.contains(n))
        .collect();
    let ordered: Vec<&str> = SECTIONS
        .iter()
        .copied()
        .filter(|s| known.contains(s))
        .collect();
    let duplicated = known.len() != ordered.len();
    if duplicated {
        out.push(violation(path, "a section appears more than once"));
    } else if known != ordered {
        out.push(violation(
            path,
            format!("sections out of order: found {}", known.join(", ")),
        ));
    }
    for (name, content) in &found {
        if SECTIONS.contains(&name.as_str()) && content.trim().is_empty() {
            out.push(violation(path, format!("section `{name}` is empty")));
        }
    }
}

fn check_capability(path: &Path, doc: &Document, found: &mut Found, out: &mut Vec<String>) {
    check_keys(path, doc, &CAPABILITY_KEYS, out);
    check_scalars(path, doc, &CAPABILITY_KEYS, out);
    let slug = check_id(path, doc, "cap:", out);
    let state = scalar_field(doc, "state");
    if let Some(state) = state.filter(|s| !STATES.contains(s)) {
        out.push(violation(path, format!("bad state `{state}`")));
    } else if doc.fields.contains_key("state") && state.is_none() {
        out.push(violation(path, "`state` must be a scalar"));
    }
    if let Some(value) = doc.fields.get("bindings") {
        match value {
            Value::Map(map) => {
                let keys: Vec<&str> = map.keys().map(String::as_str).collect();
                let mut wanted = BINDING_KEYS;
                wanted.sort_unstable();
                if keys != wanted {
                    out.push(violation(
                        path,
                        "bindings must have exactly the keys code, tests and examples",
                    ));
                }
            }
            _ => out.push(violation(
                path,
                "bindings must be a map of code, tests and examples lists",
            )),
        }
    }
    let mut depends = Vec::new();
    if doc.fields.contains_key("depends") {
        if let Some(list) = list_field(doc, "depends") {
            for id in list {
                if id.strip_prefix("cap:").is_some_and(is_kebab) {
                    depends.push(id.clone());
                } else {
                    out.push(violation(
                        path,
                        format!("depends entry `{id}` is not a cap id"),
                    ));
                }
            }
        } else {
            out.push(violation(path, "`depends` must be a list"));
        }
    }
    check_section_layout(path, doc, out);
    if let (Some(slug), Some(state)) = (slug, state) {
        found.capabilities.push(Capability {
            path: path.to_path_buf(),
            id: format!("cap:{slug}"),
            state: state.to_string(),
            depends,
        });
    }
}

fn check_change(path: &Path, doc: &Document, found: &mut Found, out: &mut Vec<String>) {
    check_keys(path, doc, &CHANGE_KEYS, out);
    check_scalars(path, doc, &CHANGE_KEYS, out);
    let slug = check_id(path, doc, "change:", out);
    let mut changes = Vec::new();
    if doc.fields.contains_key("changes") {
        match list_field(doc, "changes") {
            Some(list) if !list.is_empty() => {
                for id in list {
                    if id.strip_prefix("cap:").is_some_and(is_kebab) {
                        changes.push(id.clone());
                    } else {
                        out.push(violation(
                            path,
                            format!("changes entry `{id}` is not a cap id"),
                        ));
                    }
                }
            }
            _ => out.push(violation(
                path,
                "`changes` must be a non-empty list of cap ids",
            )),
        }
    }
    if doc.fields.contains_key("affects") {
        match list_field(doc, "affects") {
            Some(list) => {
                for entry in list {
                    check_affects(path, entry, &changes, out);
                }
            }
            None => out.push(violation(path, "`affects` must be a list")),
        }
    }
    if doc.fields.contains_key("contract_effect") {
        match scalar_field(doc, "contract_effect") {
            Some("none" | "changes") => {}
            other => out.push(violation(
                path,
                format!(
                    "bad contract_effect `{}`",
                    other.unwrap_or("<not a scalar>")
                ),
            )),
        }
    }
    check_items(path, doc, out);
    let summary = sections(&doc.body)
        .into_iter()
        .find(|(name, _)| name == "Summary");
    match summary {
        Some((_, content)) if !content.trim().is_empty() => {}
        Some(_) => out.push(violation(path, "section `Summary` is empty")),
        None => out.push(violation(path, "missing section `Summary`")),
    }
    if let Some(slug) = slug {
        found.changes.push(Change {
            path: path.to_path_buf(),
            slug,
            changes,
        });
    }
}

fn check_affects(path: &Path, entry: &str, changes: &[String], out: &mut Vec<String>) {
    let parsed = entry
        .split_once('#')
        .filter(|(cap, _)| cap.strip_prefix("cap:").is_some_and(is_kebab));
    let Some((cap, section)) = parsed else {
        out.push(violation(
            path,
            format!("affects entry `{entry}` must be `cap:<slug>#<section>`"),
        ));
        return;
    };
    if !SECTIONS.iter().any(|s| s.to_lowercase() == section) {
        out.push(violation(
            path,
            format!("affects entry `{entry}` has a bad section `{section}`"),
        ));
    }
    if !changes.iter().any(|c| c == cap) {
        out.push(violation(
            path,
            format!("affects entry `{entry}` names a capability missing from `changes`"),
        ));
    }
}

fn check_items(path: &Path, doc: &Document, out: &mut Vec<String>) {
    match doc.fields.get("items") {
        None => {}
        Some(Value::List(list)) if list.is_empty() => {}
        Some(Value::Items(items)) => {
            for item in items {
                for key in item.keys() {
                    if key != "claim" && key != "outcome" {
                        out.push(violation(path, format!("unknown item key `{key}`")));
                    }
                }
                if item.get("claim").is_none_or(|c| c.trim().is_empty()) {
                    out.push(violation(path, "every item needs a non-empty `claim`"));
                }
                if let Some(outcome) = item.get("outcome") {
                    let moved = outcome.strip_prefix("moved:change:").is_some_and(is_kebab);
                    if !OUTCOMES.contains(&outcome.as_str()) && !moved {
                        out.push(violation(path, format!("bad outcome `{outcome}`")));
                    }
                }
            }
        }
        Some(_) => out.push(violation(
            path,
            "`items` must be a list of maps with `claim` and optional `outcome`",
        )),
    }
}

fn check_architecture(path: &Path, doc: &Document, out: &mut Vec<String>) {
    check_keys(path, doc, &ARCHITECTURE_KEYS, out);
    check_scalars(path, doc, &ARCHITECTURE_KEYS, out);
    check_id(path, doc, "arch:", out);
    if doc.body.trim().is_empty() {
        out.push(violation(path, "architecture body is empty"));
    }
}

fn check_document(path: &Path, text: &str, found: &mut Found, out: &mut Vec<String>) {
    let expected = expected_type(path);
    let doc = match parse_document(text) {
        None => {
            if let Some(expected) = expected {
                out.push(violation(
                    path,
                    format!("missing frontmatter; expected a {expected} document"),
                ));
            }
            return;
        }
        Some(Err(error)) => {
            out.push(violation(path, format!("unparseable frontmatter: {error}")));
            return;
        }
        Some(Ok(doc)) => doc,
    };
    let Some(declared) = scalar_field(&doc, "type") else {
        out.push(violation(path, "missing required key `type`"));
        return;
    };
    if let Some(expected) = expected.filter(|e| *e != declared) {
        out.push(violation(
            path,
            format!("type `{declared}` is not allowed here; expected `{expected}`"),
        ));
    }
    match declared {
        "Capability" => check_capability(path, &doc, found, out),
        "Change" => check_change(path, &doc, found, out),
        "Architecture" => check_architecture(path, &doc, out),
        other => out.push(violation(path, format!("bad type `{other}`"))),
    }
}

/// A well-formed or malformed PROPOSED marker found outside code fences.
struct Marker {
    path: PathBuf,
    slug: String,
}

fn scan_markers(path: &Path, text: &str, markers: &mut Vec<Marker>, out: &mut Vec<String>) {
    let mut open_generated: Option<usize> = None;
    for (number, line, fenced) in fenced_lines(text) {
        if fenced {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.starts_with("> [!PROPOSED") {
            let slug = trimmed
                .strip_prefix(PROPOSED_OPEN)
                .and_then(|rest| rest.strip_suffix(']'))
                .filter(|s| is_kebab(s));
            match slug {
                Some(slug) => markers.push(Marker {
                    path: path.to_path_buf(),
                    slug: slug.to_string(),
                }),
                None => out.push(violation(
                    path,
                    format!("line {number}: malformed PROPOSED marker"),
                )),
            }
        }
        if trimmed.starts_with(GENERATED_OPEN) {
            let kind = trimmed
                .strip_prefix(GENERATED_OPEN)
                .and_then(|rest| rest.strip_suffix(" -->"));
            match (kind, open_generated) {
                (Some(GENERATED_END), Some(_)) => open_generated = None,
                (Some(GENERATED_END), None) => out.push(violation(
                    path,
                    format!("line {number}: generated end marker without a start"),
                )),
                (Some(kind), None) if is_kebab(kind) => open_generated = Some(number),
                (Some(_), Some(_)) => out.push(violation(
                    path,
                    format!("line {number}: generated block inside another generated block"),
                )),
                _ => out.push(violation(
                    path,
                    format!("line {number}: malformed generated marker"),
                )),
            }
        }
    }
    if let Some(number) = open_generated {
        out.push(violation(
            path,
            format!("line {number}: generated block is never closed"),
        ));
    }
}

fn scan_banned(path: &Path, text: &str, out: &mut Vec<String>) {
    for (index, line) in text.lines().enumerate() {
        if let Some(kind) = banned_in_line(line) {
            out.push(violation(
                path,
                format!("line {}: {kind} is banned", index + 1),
            ));
        }
    }
}

fn link_targets(line: &str) -> Vec<String> {
    let outside: String = line
        .split('`')
        .enumerate()
        .filter(|(index, _)| index % 2 == 0)
        .map(|(_, part)| part)
        .collect::<Vec<_>>()
        .join(" ");
    let mut targets = Vec::new();
    let mut rest = outside.as_str();
    while let Some(start) = rest.find("](") {
        let after = rest.get(start + 2..).unwrap_or_default();
        let Some(end) = after.find(')') else { break };
        let raw = after.get(..end).unwrap_or_default().trim();
        let target = raw.split_once(' ').map_or(raw, |(t, _)| t);
        targets.push(target.trim_matches(['<', '>']).to_string());
        rest = after.get(end..).unwrap_or_default();
    }
    targets
}

fn normalize(base: &Path, target: &str) -> Option<PathBuf> {
    let mut parts: Vec<std::ffi::OsString> = base
        .parent()
        .map(|p| p.iter().map(ToOwned::to_owned).collect())
        .unwrap_or_default();
    for component in Path::new(target).components() {
        match component {
            Component::Normal(name) => parts.push(name.to_os_string()),
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(parts.into_iter().collect())
}

fn check_links(repo: &Repo, path: &Path, text: &str, out: &mut Vec<String>) {
    for (number, line, fenced) in fenced_lines(text) {
        if fenced {
            continue;
        }
        for target in link_targets(line) {
            if target.is_empty()
                || target.contains("://")
                || target.starts_with("mailto:")
                || target.starts_with("tel:")
            {
                continue;
            }
            let (file_part, anchor) = match target.split_once('#') {
                Some((f, a)) => (f, Some(a)),
                None => (target.as_str(), None),
            };
            let resolved = if file_part.is_empty() {
                Some(path.to_path_buf())
            } else {
                normalize(path, file_part)
            };
            let Some(resolved) = resolved else {
                out.push(violation(
                    path,
                    format!("line {number}: link `{target}` leaves the repository"),
                ));
                continue;
            };
            let exists = repo.contains_key(&resolved)
                || repo
                    .keys()
                    .any(|k| k.starts_with(&resolved) && k != &resolved);
            if !exists {
                out.push(violation(
                    path,
                    format!("line {number}: broken link `{target}`"),
                ));
                continue;
            }
            let is_markdown = resolved.extension().is_some_and(|e| e == "md");
            if let (Some(anchor), Some(content), true) = (
                anchor.filter(|a| !a.is_empty()),
                repo.get(&resolved),
                is_markdown,
            ) && !anchors(content).contains(anchor)
            {
                out.push(violation(
                    path,
                    format!("line {number}: broken anchor `{target}`"),
                ));
            }
        }
    }
}

fn find_cycles(capabilities: &[Capability], out: &mut Vec<String>) {
    let graph: BTreeMap<&str, (&Path, &[String])> = capabilities
        .iter()
        .map(|c| (c.id.as_str(), (c.path.as_path(), c.depends.as_slice())))
        .collect();
    let mut reported: BTreeSet<Vec<&str>> = BTreeSet::new();
    let mut done: BTreeSet<&str> = BTreeSet::new();
    for start in graph.keys() {
        let mut stack: Vec<(&str, usize)> = vec![(start, 0)];
        let mut on_path: Vec<&str> = vec![start];
        while let Some((node, next)) = stack.pop() {
            let deps = graph.get(node).map_or(&[][..], |(_, d)| *d);
            let Some(dep) = deps.get(next) else {
                on_path.pop();
                done.insert(node);
                continue;
            };
            stack.push((node, next + 1));
            let dep = dep.as_str();
            if let Some(position) = on_path.iter().position(|n| *n == dep) {
                let mut cycle: Vec<&str> = on_path.iter().skip(position).copied().collect();
                if let Some(min) = cycle
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, n)| **n)
                    .map(|(i, _)| i)
                {
                    cycle.rotate_left(min);
                }
                if reported.insert(cycle.clone()) {
                    let path = graph.get(cycle[0]).map_or(Path::new(""), |(p, _)| p);
                    out.push(violation(
                        path,
                        format!("depends cycle: {} -> {}", cycle.join(" -> "), cycle[0]),
                    ));
                }
            } else if graph.contains_key(dep) && !done.contains(dep) {
                on_path.push(dep);
                stack.push((dep, 0));
            }
        }
    }
}

fn check_cross_references(repo: &Repo, found: &Found, markers: &[Marker], out: &mut Vec<String>) {
    let ids: BTreeSet<&str> = found.capabilities.iter().map(|c| c.id.as_str()).collect();
    for cap in &found.capabilities {
        for dep in &cap.depends {
            if !ids.contains(dep.as_str()) {
                out.push(violation(&cap.path, format!("dangling depends `{dep}`")));
            }
        }
    }
    find_cycles(&found.capabilities, out);
    for change in &found.changes {
        for id in &change.changes {
            if !ids.contains(id.as_str()) {
                out.push(violation(
                    &change.path,
                    format!("changes names unknown `{id}`"),
                ));
            }
        }
    }
    for marker in markers {
        let record = PathBuf::from(format!("work/changes/{}.md", marker.slug));
        if !repo.contains_key(&record) {
            out.push(violation(
                &marker.path,
                format!(
                    "orphan PROPOSED marker: {} does not exist",
                    record.display()
                ),
            ));
        }
        if expected_type(&marker.path) != Some("Capability") {
            out.push(violation(
                &marker.path,
                "PROPOSED marker outside a capability",
            ));
        }
    }
    for cap in &found.capabilities {
        let proposed: Vec<&Marker> = markers.iter().filter(|m| m.path == cap.path).collect();
        let listing: Vec<&Change> = found
            .changes
            .iter()
            .filter(|c| c.changes.contains(&cap.id))
            .collect();
        for marker in &proposed {
            if !listing.iter().any(|c| c.slug == marker.slug) {
                out.push(violation(
                    &cap.path,
                    format!("PROPOSED change `{}` does not list {}", marker.slug, cap.id),
                ));
            }
        }
        match cap.state.as_str() {
            "considered" if proposed.is_empty() => out.push(violation(
                &cap.path,
                "state considered requires its candidate contract in PROPOSED blocks",
            )),
            "delivered" => {
                if !proposed.is_empty() {
                    out.push(violation(
                        &cap.path,
                        "state delivered must have no PROPOSED blocks",
                    ));
                }
                for change in &listing {
                    out.push(violation(
                        &cap.path,
                        format!(
                            "state delivered must not be listed by open change record {}",
                            change.path.display()
                        ),
                    ));
                }
            }
            _ if listing.is_empty() => out.push(violation(
                &cap.path,
                format!(
                    "state {} requires an open change record listing {}",
                    cap.state, cap.id
                ),
            )),
            _ => {}
        }
    }
}

fn check_directories(repo: &Repo, out: &mut Vec<String>) {
    for path in repo.keys().filter(|p| is_scanned(p)) {
        let parts: Vec<&str> = path.iter().filter_map(|p| p.to_str()).collect();
        let nested = matches!(
            parts.as_slice(),
            ["docs", "capabilities" | "architecture", _, ..]
        ) && parts.len() > 3
            || matches!(parts.as_slice(), ["work", "changes", _, ..]) && parts.len() > 3;
        if nested {
            out.push(violation(
                path,
                "documents must sit directly in their directory",
            ));
        }
        let hand_index = parts.as_slice() == ["docs", "capabilities", "index.md"]
            && !repo.get(path).is_some_and(|t| t.contains(GENERATED_OPEN));
        if hand_index {
            out.push(violation(
                path,
                "a hand-written capability index is not allowed",
            ));
        }
    }
}

/// Check every rule of the profile over an in-memory repository.
fn check(repo: &Repo) -> Vec<String> {
    let mut out = Vec::new();
    let mut found = Found::default();
    let mut markers = Vec::new();
    check_directories(repo, &mut out);
    for (path, text) in repo.iter().filter(|(p, _)| is_scanned(p)) {
        scan_banned(path, text, &mut out);
        scan_markers(path, text, &mut markers, &mut out);
        check_links(repo, path, text, &mut out);
        check_document(path, text, &mut found, &mut out);
    }
    check_cross_references(repo, &found, &markers, &mut out);
    out.sort();
    out.dedup();
    out
}

/// Count the valid-looking OKF documents by declared type.
fn document_types(repo: &Repo) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for (path, text) in repo.iter().filter(|(p, _)| is_scanned(p)) {
        if let Some(Ok(doc)) = parse_document(text)
            && let Some(kind) = scalar_field(&doc, "type")
        {
            *counts.entry(kind.to_string()).or_insert(0) += 1;
            let _ = path;
        }
    }
    counts
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

/// Load the real tree from the git index, read under the gate root: every
/// tracked path (for link targets) and the text of every Markdown file.
fn load_repo(root: &Path) -> Repo {
    tracked_paths()
        .into_iter()
        .map(|path| {
            let text = if path.extension().is_some_and(|e| e == "md") {
                let bytes = read_tracked(root, &path);
                decode_text(&bytes)
                    .unwrap_or_else(|| panic!("{} is not UTF-8 text", path.display()))
                    .to_string()
            } else {
                String::new()
            };
            (path, text)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn repository_documents_satisfy_the_profile() {
    let repo = load_repo(&gate_root());
    let violations = check(&repo);
    let mut report = String::new();
    for v in &violations {
        let _ = writeln!(report, "  {v}");
    }
    assert!(
        violations.is_empty(),
        "{} profile violations:\n{report}",
        violations.len()
    );
    let types = document_types(&repo);
    assert!(
        types.get("Architecture").copied().unwrap_or(0) >= 1,
        "no Architecture document found; found types: {types:?}"
    );
    if repo.keys().any(|p| p.starts_with("docs/capabilities")) {
        assert!(
            types.get("Capability").copied().unwrap_or(0) >= 1,
            "docs/capabilities exists but holds no Capability document; found types: {types:?}"
        );
    }
}

mod fixtures {
    use super::{Path, PathBuf, Repo, check};

    pub(super) fn capability(slug: &str, state: &str, depends: &str, body: &str) -> String {
        let sections = [
            "Purpose",
            "Contract",
            "Usage",
            "Examples",
            "Constraints",
            "Explanation",
            "Verification",
        ]
        .iter()
        .map(|name| format!("## {name}\n\nText for {name}.\n"))
        .collect::<Vec<_>>()
        .join("\n");
        format!(
            "---\n# SPDX-FileCopyrightText: 2026 Example\n# SPDX-License-Identifier: CC-BY-4.0\n\
             type: Capability\nid: cap:{slug}\ntitle: \"Title of {slug}\"\n\
             description: A description of {slug}.\nstate: {state}\n\
             bindings:\n  code: []\n  tests: []\n  examples: []\n\
             depends: {depends}\n---\n\n# {slug}\n\n{body}{sections}"
        )
    }

    const PROPOSED: &str = "> [!PROPOSED change:add-alpha]\n> The candidate contract.\n\n";

    pub(super) fn valid() -> Repo {
        let mut repo = Repo::new();
        let mut put = |path: &str, text: String| {
            repo.insert(PathBuf::from(path), text);
        };
        put(
            "docs/capabilities/alpha.md",
            capability("alpha", "designed", "[cap:beta]", ""),
        );
        put(
            "docs/capabilities/beta.md",
            capability("beta", "considered", "[]", PROPOSED),
        );
        put(
            "docs/capabilities/gamma.md",
            capability("gamma", "delivered", "[cap:beta, cap:alpha]", ""),
        );
        put(
            "docs/architecture/overview.md",
            "---\ntype: Architecture\nid: arch:overview\ntitle: Overview\n\
             description: How it fits.\n---\n\n# Overview\n\nBody text.\n"
                .to_string(),
        );
        put(
            "work/changes/add-alpha.md",
            "---\ntype: Change\nid: change:add-alpha\ntitle: Add alpha\n\
             description: Adds alpha and beta.\n\
             changes: [cap:alpha, cap:beta]\naffects:\n  - cap:alpha#contract\n  - cap:beta#purpose\n\
             contract_effect: changes\nitems:\n  - claim: \"alpha reads input\"\n    outcome: implemented\n  \
             - claim: \"beta is sketched\"\n---\n\n## Summary\n\nAdds both.\n"
                .to_string(),
        );
        put(
            "README.md",
            "# Readme\n\nSee [alpha](docs/capabilities/alpha.md#purpose) and [the overview](./docs/architecture/overview.md).\n"
                .to_string(),
        );
        repo
    }

    pub(super) fn mutated(path: &str, from: &str, to: &str) -> Vec<String> {
        let mut repo = valid();
        let text = repo
            .get_mut(Path::new(path))
            .unwrap_or_else(|| panic!("fixture has no {path}"));
        assert!(text.contains(from), "fixture {path} has no `{from}`");
        *text = text.replacen(from, to, 1);
        check(&repo)
    }
}

use fixtures::{capability, mutated, valid};

fn assert_hit(violations: &[String], needle: &str) {
    assert!(
        violations.iter().any(|v| v.contains(needle)),
        "expected a violation containing `{needle}`, got: {violations:#?}"
    );
}

#[test]
fn accepts_valid_fixture() {
    let violations = check(&valid());
    assert!(violations.is_empty(), "{violations:#?}");
    let types = document_types(&valid());
    assert_eq!(types.get("Capability"), Some(&3));
    assert_eq!(types.get("Change"), Some(&1));
    assert_eq!(types.get("Architecture"), Some(&1));
}

#[test]
fn parses_quoted_and_block_forms() {
    let doc = parse_document("---\n# note\ntype: X\ntitle: 'It''s'\nitems:\n  - claim: \"a: b\"\n    outcome: dropped\nlist:\n  - one\n  - two\n---\nbody")
        .expect("has frontmatter")
        .expect("parses");
    assert_eq!(doc.fields.get("title"), Some(&Value::Scalar("It's".into())));
    assert_eq!(
        doc.fields.get("list"),
        Some(&Value::List(vec!["one".into(), "two".into()]))
    );
    let Some(Value::Items(items)) = doc.fields.get("items") else {
        panic!("items did not parse");
    };
    assert_eq!(items[0].get("claim").map(String::as_str), Some("a: b"));
}

#[test]
fn rejects_unparseable_frontmatter() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "title: Overview",
        "title Overview",
    );
    assert_hit(&violations, "unparseable frontmatter");
}

#[test]
fn rejects_unknown_key() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "title: Overview",
        "title: Overview\nextra: 1",
    );
    assert_hit(&violations, "unknown key `extra`");
}

#[test]
fn rejects_missing_required_key() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "description: How it fits.\n",
        "",
    );
    assert_hit(&violations, "missing required key `description`");
}

#[test]
fn rejects_bad_type() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "type: Architecture",
        "type: Notes",
    );
    assert_hit(&violations, "bad type `Notes`");
}

#[test]
fn rejects_type_in_the_wrong_directory() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "type: Architecture",
        "type: Change",
    );
    assert_hit(&violations, "not allowed here");
}

#[test]
fn rejects_id_filename_mismatch() {
    let violations = mutated(
        "docs/capabilities/alpha.md",
        "id: cap:alpha",
        "id: cap:other",
    );
    assert_hit(&violations, "does not match the file name");
}

#[test]
fn rejects_bad_state() {
    let violations = mutated(
        "docs/capabilities/alpha.md",
        "state: designed",
        "state: done",
    );
    assert_hit(&violations, "bad state `done`");
}

#[test]
fn rejects_wrong_bindings_keys() {
    let violations = mutated(
        "docs/capabilities/alpha.md",
        "  examples: []\n",
        "  docs: []\n",
    );
    assert_hit(&violations, "bindings must have exactly");
}

#[test]
fn rejects_missing_section() {
    let violations = mutated(
        "docs/capabilities/alpha.md",
        "## Usage\n\nText for Usage.\n\n",
        "",
    );
    assert_hit(&violations, "missing section `Usage`");
}

#[test]
fn rejects_sections_out_of_order() {
    let violations = mutated("docs/capabilities/alpha.md", "## Purpose", "## Swap");
    assert_hit(&violations, "unexpected section `Swap`");
    let mut repo = valid();
    let text = repo
        .get_mut(Path::new("docs/capabilities/alpha.md"))
        .expect("fixture capability");
    *text = text
        .replace("## Contract", "## TMP")
        .replace("## Usage", "## Contract")
        .replace("## TMP", "## Usage");
    assert_hit(&check(&repo), "sections out of order");
}

#[test]
fn rejects_empty_section() {
    let violations = mutated("docs/capabilities/alpha.md", "Text for Examples.\n", "");
    assert_hit(&violations, "section `Examples` is empty");
}

#[test]
fn rejects_dangling_depends() {
    let violations = mutated("docs/capabilities/alpha.md", "[cap:beta]", "[cap:missing]");
    assert_hit(&violations, "dangling depends `cap:missing`");
}

#[test]
fn rejects_depends_two_cycle() {
    let violations = mutated(
        "docs/capabilities/beta.md",
        "depends: []",
        "depends: [cap:alpha]",
    );
    assert_hit(
        &violations,
        "depends cycle: cap:alpha -> cap:beta -> cap:alpha",
    );
}

#[test]
fn rejects_broken_relative_link() {
    let violations = mutated(
        "README.md",
        "docs/capabilities/alpha.md#purpose",
        "docs/capabilities/nope.md",
    );
    assert_hit(&violations, "broken link `docs/capabilities/nope.md`");
}

#[test]
fn rejects_broken_anchor() {
    let violations = mutated("README.md", "alpha.md#purpose", "alpha.md#no-such-heading");
    assert_hit(&violations, "broken anchor");
}

#[test]
fn rejects_link_leaving_the_repository() {
    let violations = mutated(
        "README.md",
        "./docs/architecture/overview.md",
        "../../outside.md",
    );
    assert_hit(&violations, "leaves the repository");
}

#[test]
fn anchors_follow_github_rules() {
    assert_eq!(github_anchor("Why `x`: it's (fine)"), "why-x-its-fine");
    let dup = anchors("## Same\n\n## Same\n");
    assert!(dup.contains("same") && dup.contains("same-1"));
}

#[test]
fn rejects_orphan_proposed_marker() {
    let violations = mutated(
        "docs/capabilities/beta.md",
        "change:add-alpha",
        "change:ghost",
    );
    assert_hit(&violations, "orphan PROPOSED marker: work/changes/ghost.md");
}

#[test]
fn rejects_malformed_proposed_marker() {
    let violations = mutated(
        "docs/capabilities/beta.md",
        "> [!PROPOSED change:add-alpha]",
        "> [!PROPOSED add-alpha]",
    );
    assert_hit(&violations, "malformed PROPOSED marker");
}

#[test]
fn rejects_considered_without_proposed_block() {
    let violations = mutated(
        "docs/capabilities/beta.md",
        "> [!PROPOSED change:add-alpha]\n> The candidate contract.\n",
        "",
    );
    assert_hit(
        &violations,
        "requires its candidate contract in PROPOSED blocks",
    );
}

#[test]
fn rejects_open_capability_without_change_record() {
    let violations = mutated(
        "work/changes/add-alpha.md",
        "changes: [cap:alpha, cap:beta]",
        "changes: [cap:beta]",
    );
    assert_hit(
        &violations,
        "requires an open change record listing cap:alpha",
    );
}

#[test]
fn rejects_delivered_with_proposed_block() {
    let violations = mutated(
        "docs/capabilities/gamma.md",
        "# gamma\n",
        "# gamma\n\n> [!PROPOSED change:add-alpha]\n> Left over.\n",
    );
    assert_hit(&violations, "delivered must have no PROPOSED blocks");
}

#[test]
fn rejects_delivered_with_open_change_record() {
    let violations = mutated(
        "work/changes/add-alpha.md",
        "changes: [cap:alpha, cap:beta]",
        "changes: [cap:alpha, cap:beta, cap:gamma]",
    );
    assert_hit(
        &violations,
        "delivered must not be listed by open change record",
    );
}

#[test]
fn rejects_affects_with_bad_section() {
    let violations = mutated(
        "work/changes/add-alpha.md",
        "cap:alpha#contract",
        "cap:alpha#nonsense",
    );
    assert_hit(&violations, "bad section `nonsense`");
}

#[test]
fn rejects_affects_slug_not_in_changes() {
    let violations = mutated(
        "work/changes/add-alpha.md",
        "cap:alpha#contract",
        "cap:gamma#contract",
    );
    assert_hit(&violations, "missing from `changes`");
}

#[test]
fn rejects_bad_contract_effect() {
    let violations = mutated(
        "work/changes/add-alpha.md",
        "contract_effect: changes",
        "contract_effect: maybe",
    );
    assert_hit(&violations, "bad contract_effect `maybe`");
}

#[test]
fn rejects_bad_outcome() {
    let violations = mutated(
        "work/changes/add-alpha.md",
        "outcome: implemented",
        "outcome: finished",
    );
    assert_hit(&violations, "bad outcome `finished`");
}

#[test]
fn accepts_moved_outcome() {
    let violations = mutated(
        "work/changes/add-alpha.md",
        "outcome: implemented",
        "outcome: moved:change:other-work",
    );
    assert!(violations.is_empty(), "{violations:#?}");
}

#[test]
fn rejects_change_without_summary() {
    let violations = mutated("work/changes/add-alpha.md", "## Summary", "## Notes");
    assert_hit(&violations, "missing section `Summary`");
}

#[test]
fn rejects_unclosed_generated_block() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "Body text.",
        "Body text.\n\n<!-- raf:generated:matrix -->\n",
    );
    assert_hit(&violations, "generated block is never closed");
}

#[test]
fn rejects_hand_written_capability_index() {
    let mut repo = valid();
    repo.insert(
        PathBuf::from("docs/capabilities/index.md"),
        "# Index\n".to_string(),
    );
    assert_hit(&check(&repo), "hand-written capability index");
}

#[test]
fn rejects_issue_number_token() {
    let token = concat!("Per ", "#", "12");
    let violations = mutated(
        "docs/architecture/overview.md",
        "Body text.",
        &format!("Body text. {token} said so."),
    );
    assert_hit(&violations, "issue or pull request number is banned");
    let url = concat!("https://example.invalid/", "issues/", "7");
    let violations = mutated("README.md", "# Readme", &format!("# Readme\n\n{url}"));
    assert_hit(&violations, "issue or pull request number is banned");
}

#[test]
fn issue_token_scan_spares_hex_colours() {
    assert!(banned_in_line("fill #111214 and #fbbc05 and #1f2").is_none());
    assert!(banned_in_line(concat!("see PR", "#", "9")).is_some());
    assert!(banned_in_line(concat!("CHANGE", "-", "17")).is_some());
}

#[test]
fn rejects_process_wording() {
    let phrase = concat!("As agreed in ", "this", " branch");
    let violations = mutated(
        "docs/architecture/overview.md",
        "Body text.",
        &format!("Body text. {phrase}."),
    );
    assert_hit(&violations, "development-process wording is banned");
    let label = concat!("Task", " 4 is done");
    let violations = mutated("README.md", "# Readme", &format!("# Readme\n\n{label}"));
    assert_hit(&violations, "development-process wording is banned");
}

#[test]
fn fixture_builder_is_self_consistent() {
    let doc = capability("delta", "implementing", "[]", "");
    assert!(doc.contains("id: cap:delta"));
}

#[test]
fn rejects_capability_without_frontmatter() {
    let mut repo = valid();
    repo.insert(
        PathBuf::from("docs/capabilities/plain.md"),
        "# Plain\n\nNo frontmatter here.\n".to_string(),
    );
    assert_hit(
        &check(&repo),
        "docs/capabilities/plain.md: missing frontmatter",
    );
}

#[test]
fn rejects_unclosed_frontmatter() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "---\n\n# Overview",
        "# Overview",
    );
    assert_hit(&violations, "frontmatter is not closed");
}

#[test]
fn rejects_proposed_marker_in_architecture() {
    let marker = "> [!PROPOSED change:add-alpha]\n> Not allowed here.";
    let violations = mutated("docs/architecture/overview.md", "Body text.", marker);
    assert_hit(
        &violations,
        "overview.md: PROPOSED marker outside a capability",
    );
}

#[test]
fn rejects_proposed_marker_in_change() {
    let marker = "Adds both.\n\n> [!PROPOSED change:add-alpha]\n> Not allowed here.";
    let violations = mutated("work/changes/add-alpha.md", "Adds both.", marker);
    assert_hit(
        &violations,
        "add-alpha.md: PROPOSED marker outside a capability",
    );
}

#[test]
fn rejects_change_naming_unknown_capability() {
    let violations = mutated(
        "work/changes/add-alpha.md",
        "changes: [cap:alpha, cap:beta]",
        "changes: [cap:alpha, cap:beta, cap:ghost]",
    );
    assert_hit(&violations, "changes names unknown `cap:ghost`");
}

/// Nested documents are rejected, not validated: every document sits directly
/// in its directory.
#[test]
fn rejects_nested_documents() {
    for nested in ["docs/capabilities/sub/x.md", "work/changes/sub/y.md"] {
        let mut repo = valid();
        repo.insert(PathBuf::from(nested), "# Nested\n".to_string());
        assert_hit(
            &check(&repo),
            &format!("{nested}: documents must sit directly in their directory"),
        );
    }
}

#[test]
fn rejects_acceptance_label() {
    let label = concat!("Criterion ", "AC", "3", " is met");
    let violations = mutated("README.md", "# Readme", &format!("# Readme\n\n{label}"));
    assert_hit(&violations, "development-process wording is banned");
}

#[test]
fn rejects_resource_key() {
    // `resource` is not in any closed key set, so the unknown-key path rejects it.
    let violations = mutated(
        "docs/architecture/overview.md",
        "title: Overview",
        "title: Overview\nresource: https://example.invalid/overview",
    );
    assert_hit(&violations, "unknown key `resource`");
    let violations = mutated(
        "docs/capabilities/alpha.md",
        "state: designed",
        "state: designed\nresource: https://example.invalid/alpha",
    );
    assert_hit(&violations, "unknown key `resource`");
}
