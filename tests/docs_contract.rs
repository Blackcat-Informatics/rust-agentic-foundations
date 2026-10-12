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

use support::{banned_in_line, decode_text, gate_root, read_regular, read_tracked, tracked_paths};

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

/// Unfinished-work tokens that may not appear outside code, lowercase and
/// matched without regard to case, spacing or dots between letters. The text
/// is assembled with `concat!` so this list never contains a token as one
/// literal.
const PLACEHOLDER_TOKENS: [&str; 5] = [
    concat!("t", "bd"),
    concat!("to", "do"),
    concat!("fix", "me"),
    concat!("x", "xx"),
    concat!("to be ", "written"),
];

/// The capability inventory.
///
/// This table is the capability inventory: the real tree must hold exactly
/// these capability documents, each with the stated `state`, exactly these
/// architecture documents, and an open change record named after every
/// capability whose state is not `delivered`. Changing a state or adding a
/// capability edits this table in the same commit. `raf_workplan::Gate`
/// replaces it.
struct Inventory {
    /// Capability slug and the state its document declares.
    capabilities: &'static [(&'static str, &'static str)],
    /// Architecture slugs.
    architecture: &'static [&'static str],
}

const INVENTORY: Inventory = Inventory {
    capabilities: &[
        ("census-cli", "designed"),
        ("census-debt-ratchet", "designed"),
        ("census-detectors", "designed"),
        ("census-ledger", "designed"),
        ("census-migration", "designed"),
        ("census-normal-form", "designed"),
        ("census-patterns", "designed"),
        ("census-reference-scan", "designed"),
        ("census-rule-api", "designed"),
        ("okf-profile", "designed"),
        ("source-index", "designed"),
        ("workplan-cli", "designed"),
        ("workplan-complete-gate", "designed"),
        ("workplan-current-rationale", "considered"),
        ("workplan-document-model", "designed"),
        ("workplan-entity-history", "considered"),
        ("workplan-evidence-claims", "considered"),
        ("workplan-generated-docs", "designed"),
        ("workplan-prose-assessment", "considered"),
        ("workplan-reconcile", "designed"),
        ("workplan-two-way-coverage", "considered"),
    ],
    architecture: &["normal-form", "okf-profile"],
};

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

/// Cut a trailing ` # comment` from a value, leaving quoted text intact.
fn strip_comment(raw: &str) -> &str {
    let raw = raw.trim();
    let in_list = raw.starts_with('[');
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut just_closed_single = false;
    let mut prev = ' ';
    for (index, c) in raw.char_indices() {
        let reopened = just_closed_single && c == '\'';
        just_closed_single = false;
        if let Some(q) = quote {
            if q == '"' && c == '\\' && !escaped {
                escaped = true;
                prev = c;
                continue;
            }
            if c == q && !escaped {
                quote = None;
                just_closed_single = q == '\'';
            }
            escaped = false;
        } else if (c == '"' || c == '\'')
            && (index == 0 || reopened || (in_list && matches!(prev, '[' | ',' | ' ')))
        {
            quote = Some(c);
        } else if c == '#' && (index == 0 || prev.is_whitespace()) {
            return raw.get(..index).unwrap_or_default().trim_end();
        }
        prev = c;
    }
    raw
}

/// The text of a quoted scalar; the closing quote must end the value.
fn quoted_scalar(raw: &str, quote: char) -> Result<String, String> {
    let mut out = String::new();
    let mut chars = raw.char_indices().skip(1).peekable();
    while let Some((index, c)) = chars.next() {
        if quote == '"' && c == '\\' {
            match chars.next() {
                Some((_, escaped @ ('"' | '\\'))) => out.push(escaped),
                Some((_, other)) => {
                    out.push('\\');
                    out.push(other);
                }
                None => break,
            }
        } else if c == quote {
            if quote == '\'' && chars.next_if(|(_, next)| *next == '\'').is_some() {
                out.push('\'');
                continue;
            }
            return if raw.get(index + 1..).unwrap_or_default().trim().is_empty() {
                Ok(out)
            } else {
                Err(format!("text after the closing quote in `{raw}`"))
            };
        } else {
            out.push(c);
        }
    }
    Err(format!("unterminated quoted value `{raw}`"))
}

/// True for a YAML block scalar header such as `>`, `|`, `>-` or `|2`.
fn is_block_indicator(raw: &str) -> bool {
    raw.strip_prefix(['>', '|'])
        .is_some_and(|rest| rest.chars().all(|c| matches!(c, '+' | '-' | '0'..='9')))
}

/// An unquoted scalar: no block indicator, no leading indicator character and
/// no `: ` that real YAML would read as a nested mapping.
fn plain_scalar(raw: &str) -> Result<String, String> {
    if is_block_indicator(raw) {
        return Err(format!("block scalar indicator `{raw}` used as a value"));
    }
    if raw.starts_with(['>', '|', '&', '*', '!', '%', '@', '`']) {
        return Err(format!("`{raw}` starts with a YAML indicator character"));
    }
    if raw.contains(": ") || raw.ends_with(':') {
        return Err(format!("unquoted value `{raw}` contains `: `; quote it"));
    }
    Ok(raw.to_string())
}

/// Parse one scalar value: quoted or plain, with a trailing comment removed.
fn scalar(raw: &str) -> Result<String, String> {
    let raw = strip_comment(raw);
    match raw.chars().next() {
        Some(quote @ ('"' | '\'')) => quoted_scalar(raw, quote),
        _ => plain_scalar(raw),
    }
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
                scalar(part)
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
                maps.push(BTreeMap::from([(key.to_string(), scalar(rest)?)]));
                in_map = true;
            } else {
                scalars.push(scalar(item)?);
                in_map = false;
            }
        } else if let Some(continuation) = line.strip_prefix("    ") {
            let (key, rest) = split_key(continuation.trim())
                .ok_or_else(|| format!("cannot parse list entry line `{line}`"))?;
            let entry = maps
                .last_mut()
                .filter(|_| in_map)
                .ok_or_else(|| format!("continuation without a map entry `{line}`"))?;
            if entry.insert(key.to_string(), scalar(rest)?).is_some() {
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
            map.entry(key.clone()).or_default().push(scalar(item)?);
        } else if let Some(entry) = line.strip_prefix("  ").filter(|e| !e.starts_with(' ')) {
            let (key, rest) =
                split_key(entry).ok_or_else(|| format!("cannot parse map line `{line}`"))?;
            if map.contains_key(key) {
                return Err(format!("duplicate map key `{key}`"));
            }
            let rest = strip_comment(rest);
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
        let rest = strip_comment(rest);
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
            Value::Scalar(scalar(rest)?)
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

/// Remove HTML comments, including multi-line ones, from Markdown text.
fn strip_html_comments(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("<!--") {
        out.push_str(rest.get(..start).unwrap_or_default());
        let after = rest.get(start + 4..).unwrap_or_default();
        match after.find("-->") {
            Some(end) => rest = after.get(end + 3..).unwrap_or_default(),
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// True when the text has no visible content once comments are removed.
fn is_blank(text: &str) -> bool {
    strip_html_comments(text).trim().is_empty()
}

/// The line with inline code spans removed.
fn outside_code(line: &str) -> String {
    line.split('`')
        .enumerate()
        .filter(|(index, _)| index % 2 == 0)
        .map(|(_, part)| part)
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// True when `token` occurs in `line` as a whole word.
fn has_word(line: &str, token: &str) -> bool {
    line.match_indices(token).any(|(index, _)| {
        let before = line.get(..index).and_then(|s| s.chars().next_back());
        let after = line
            .get(index + token.len()..)
            .and_then(|s| s.chars().next());
        !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
    })
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
    let markdown = is_markdown(path);
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

/// The directory that holds documents of a RAF type.
fn type_directory(declared: &str) -> Option<&'static str> {
    match declared {
        "Capability" => Some("docs/capabilities"),
        "Change" => Some("work/changes"),
        "Architecture" => Some("docs/architecture"),
        _ => None,
    }
}

/// The generated catalogue, which has no frontmatter.
fn is_capability_index(path: &Path) -> bool {
    path == Path::new("docs/capabilities/index.md")
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
    affects: Vec<String>,
    moved: Vec<String>,
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
        if SECTIONS.contains(&name.as_str()) && is_blank(content) {
            out.push(violation(path, format!("section `{name}` is empty")));
        }
    }
}

/// Binding lists by state: nothing is bound before delivery, and a delivered
/// capability binds at least one code path and one test.
fn check_binding_contents(
    path: &Path,
    state: Option<&str>,
    map: &BTreeMap<String, Vec<String>>,
    out: &mut Vec<String>,
) {
    let bound = |key: &str| map.get(key).map_or(0, Vec::len);
    match state {
        Some(state @ ("designed" | "considered")) => {
            for key in BINDING_KEYS {
                if bound(key) > 0 {
                    out.push(violation(
                        path,
                        format!("state {state} must have empty bindings, but `{key}` is not empty"),
                    ));
                }
            }
        }
        Some("delivered") => {
            if bound("code") == 0 {
                out.push(violation(
                    path,
                    "state delivered must bind at least one code path",
                ));
            }
            if bound("tests") == 0 {
                out.push(violation(
                    path,
                    "state delivered must bind at least one test",
                ));
            }
        }
        _ => {}
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
                check_binding_contents(path, state, map, out);
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
    let mut affects = Vec::new();
    if doc.fields.contains_key("affects") {
        match list_field(doc, "affects") {
            Some(list) => {
                affects = list.to_vec();
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
    let moved = check_items(path, doc, out);
    let summary = sections(&doc.body)
        .into_iter()
        .find(|(name, _)| name == "Summary");
    match summary {
        Some((_, content)) if !is_blank(&content) => {}
        Some(_) => out.push(violation(path, "section `Summary` is empty")),
        None => out.push(violation(path, "missing section `Summary`")),
    }
    if let Some(slug) = slug {
        found.changes.push(Change {
            path: path.to_path_buf(),
            slug,
            changes,
            affects,
            moved,
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

/// Validate `items`, returning the slugs that `moved:change:<slug>` outcomes name.
fn check_items(path: &Path, doc: &Document, out: &mut Vec<String>) -> Vec<String> {
    let mut moved_to = Vec::new();
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
                    let moved = outcome
                        .strip_prefix("moved:change:")
                        .filter(|s| is_kebab(s));
                    if let Some(slug) = moved {
                        moved_to.push(slug.to_string());
                    } else if !OUTCOMES.contains(&outcome.as_str()) {
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
    moved_to
}

fn check_architecture(path: &Path, doc: &Document, out: &mut Vec<String>) {
    check_keys(path, doc, &ARCHITECTURE_KEYS, out);
    check_scalars(path, doc, &ARCHITECTURE_KEYS, out);
    check_id(path, doc, "arch:", out);
    if is_blank(&doc.body) {
        out.push(violation(path, "architecture body is empty"));
    }
}

fn check_document(path: &Path, text: &str, found: &mut Found, out: &mut Vec<String>) {
    let expected = expected_type(path);
    let doc = match parse_document(text) {
        None => {
            if let Some(expected) = expected.filter(|_| !is_capability_index(path)) {
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
    if expected.is_none()
        && let Some(home) = type_directory(declared)
    {
        out.push(violation(
            path,
            format!("a {declared} document belongs directly in {home}"),
        ));
    }
    match declared {
        "Capability" => check_capability(path, &doc, found, out),
        "Change" => check_change(path, &doc, found, out),
        "Architecture" => check_architecture(path, &doc, out),
        other => out.push(violation(path, format!("bad type `{other}`"))),
    }
}

/// A well-formed PROPOSED marker found outside code fences.
struct Marker {
    path: PathBuf,
    slug: String,
    /// The lowercase name of the second-level section that holds the marker.
    section: Option<String>,
}

/// True when the line reads as a PROPOSED marker, however it is spelled: the
/// quote marks, list bullets and spaces before `[!` are skipped and the word
/// is compared without regard to case.
fn looks_like_proposed(trimmed: &str) -> bool {
    trimmed
        .trim_start_matches(['>', ' ', '\t', '-', '*'])
        .to_ascii_lowercase()
        .starts_with("[!proposed")
}

fn scan_markers(path: &Path, text: &str, markers: &mut Vec<Marker>, out: &mut Vec<String>) {
    let mut open_generated: Option<usize> = None;
    let mut section: Option<String> = None;
    for (number, line, fenced) in fenced_lines(text) {
        if fenced {
            continue;
        }
        if let Some((2, name)) = heading_text(line) {
            section = Some(name.to_lowercase());
        }
        let trimmed = line.trim();
        if looks_like_proposed(trimmed) {
            let slug = trimmed
                .strip_prefix(PROPOSED_OPEN)
                .and_then(|rest| rest.strip_suffix(']'))
                .filter(|s| is_kebab(s));
            match slug {
                Some(slug) => markers.push(Marker {
                    path: path.to_path_buf(),
                    slug: slug.to_string(),
                    section: section.clone(),
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

/// Lowercase text with whitespace runs collapsed to one space and the dots
/// between letters removed, so `T.B.D.` and `to  be\twritten` normalise to the
/// plain token.
fn normalise_prose(line: &str) -> String {
    let collapsed = line
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let chars: Vec<char> = collapsed.chars().collect();
    chars
        .iter()
        .enumerate()
        .filter(|(index, c)| {
            let letter_before = index
                .checked_sub(1)
                .and_then(|i| chars.get(i))
                .is_some_and(|p| p.is_alphabetic());
            let letter_after = chars.get(index + 1).is_some_and(|n| n.is_alphabetic());
            !(**c == '.' && letter_before && letter_after)
        })
        .map(|(_, c)| *c)
        .collect()
}

/// Reject unfinished-work tokens in prose, outside code fences and code spans.
fn scan_placeholders(path: &Path, text: &str, out: &mut Vec<String>) {
    for (number, line, fenced) in fenced_lines(text) {
        if fenced {
            continue;
        }
        let prose = normalise_prose(&outside_code(line));
        for token in PLACEHOLDER_TOKENS {
            if has_word(&prose, token) {
                out.push(violation(
                    path,
                    format!("line {number}: placeholder `{token}` is not resolved text"),
                ));
            }
        }
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

/// The target of a reference definition such as `[label]: target "title"`.
fn definition_target(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix('[')?;
    let (label, after) = rest.split_once("]:")?;
    if label.trim().is_empty() || label.starts_with('^') {
        return None;
    }
    let after = after.trim_start();
    let target = match after.strip_prefix('<') {
        Some(inner) => inner.split('>').next().unwrap_or_default(),
        None => after.split_whitespace().next().unwrap_or_default(),
    };
    (!target.is_empty()).then(|| target.to_string())
}

/// The values of `href="..."` and `src="..."` attributes in inline HTML.
/// True for a character that can belong to an HTML attribute name.
fn is_attribute_name_char(c: char) -> bool {
    is_word_char(c) || matches!(c, '-' | ':' | '.')
}

/// Where the HTML scan stands between lines: inside a tag, and inside a
/// quoted attribute value.
#[derive(Default)]
struct TagState {
    in_tag: bool,
    quote: Option<char>,
    /// An `href=` or `src=` ended the previous line, so its value is next.
    awaiting_value: bool,
}

/// Index just past the run of characters from `start` that satisfy `keep`.
fn skip_while(chars: &[char], start: usize, keep: impl Fn(char) -> bool) -> usize {
    let mut index = start;
    while chars.get(index).is_some_and(|c| keep(*c)) {
        index += 1;
    }
    index
}

/// Read one attribute at `start` (the first character of its name) and return
/// the index after it, pushing the value of an `href` or `src` attribute.
fn read_attribute(
    chars: &[char],
    start: usize,
    state: &mut TagState,
    targets: &mut Vec<String>,
) -> usize {
    let name_end = skip_while(chars, start, is_attribute_name_char);
    let name: String = chars
        .get(start..name_end)
        .unwrap_or_default()
        .iter()
        .collect::<String>()
        .to_ascii_lowercase();
    let after_name = skip_while(chars, name_end, char::is_whitespace);
    if chars.get(after_name) != Some(&'=') {
        return name_end;
    }
    let value_start = skip_while(chars, after_name + 1, char::is_whitespace);
    read_value(
        chars,
        value_start,
        name == "href" || name == "src",
        state,
        targets,
    )
}

/// Read the attribute value that starts at `start`, returning the index after
/// it. A value that is not on this line is read from the next one when the
/// attribute is `href` or `src`.
fn read_value(
    chars: &[char],
    start: usize,
    wanted: bool,
    state: &mut TagState,
    targets: &mut Vec<String>,
) -> usize {
    match chars.get(start) {
        Some(&quote @ ('"' | '\'')) => {
            let inner = start + 1;
            let close = chars
                .get(inner..)
                .and_then(|rest| rest.iter().position(|c| *c == quote));
            if let Some(offset) = close {
                if wanted {
                    let value = chars.get(inner..inner + offset).unwrap_or_default();
                    targets.push(value.iter().collect());
                }
                inner + offset + 1
            } else {
                state.quote = Some(quote);
                chars.len()
            }
        }
        Some(_) => {
            let end = skip_while(chars, start, |c| !c.is_whitespace() && c != '>');
            if wanted {
                let value: String = chars.get(start..end).unwrap_or_default().iter().collect();
                targets.push(value.trim_end_matches('/').to_string());
            }
            end
        }
        None => {
            state.awaiting_value = wanted;
            chars.len()
        }
    }
}

/// The `href` and `src` values found on one line, tracking tag state across
/// lines. A tag opens at `<` before a letter or `/` and runs to the next `>`
/// that is not inside quotes. Autolinks such as `<https://x>` are not tags.
fn html_attribute_targets(line: &str, state: &mut TagState) -> Vec<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut targets = Vec::new();
    let mut index = 0;
    if state.awaiting_value && state.in_tag {
        state.awaiting_value = false;
        let start = skip_while(&chars, 0, char::is_whitespace);
        index = read_value(&chars, start, true, state, &mut targets);
    }
    state.awaiting_value = false;
    while let Some(&c) = chars.get(index) {
        if let Some(quote) = state.quote {
            match chars
                .get(index..)
                .and_then(|r| r.iter().position(|x| *x == quote))
            {
                Some(offset) => {
                    state.quote = None;
                    index += offset + 1;
                }
                None => break,
            }
        } else if !state.in_tag {
            let opens = c == '<'
                && chars
                    .get(index + 1)
                    .is_some_and(|n| n.is_alphabetic() || *n == '/');
            index += 1;
            if opens {
                let name_end = skip_while(&chars, index, |x| is_attribute_name_char(x) || x == '/');
                let autolink = chars
                    .get(index..name_end)
                    .unwrap_or_default()
                    .contains(&':');
                state.in_tag = !autolink;
                index = name_end;
            }
        } else if c == '>' {
            state.in_tag = false;
            index += 1;
        } else if is_attribute_name_char(c) {
            index = read_attribute(&chars, index, state, &mut targets);
        } else if matches!(c, '"' | '\'') {
            state.quote = Some(c);
            index += 1;
        } else {
            index += 1;
        }
    }
    targets
}

/// Every link target on one line: inline links and images, reference
/// definitions. Links are single-line. HTML attributes are read separately,
/// because a tag may span lines.
fn link_targets(line: &str) -> Vec<String> {
    let outside = outside_code(line);
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
    targets.extend(definition_target(&outside));
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
    let mut tags = TagState::default();
    for (number, line, fenced) in fenced_lines(text) {
        if fenced || line.trim().is_empty() {
            // Inline HTML cannot span a blank line or a fence.
            tags = TagState::default();
            continue;
        }
        let attributes = html_attribute_targets(&outside_code(line), &mut tags);
        for target in link_targets(line).into_iter().chain(attributes) {
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

/// State rules for one capability: PROPOSED blocks, their `affects` entries
/// and the open change record.
fn check_capability_state(
    cap: &Capability,
    found: &Found,
    markers: &[Marker],
    out: &mut Vec<String>,
) {
    let proposed: Vec<&Marker> = markers.iter().filter(|m| m.path == cap.path).collect();
    let listing: Vec<&Change> = found
        .changes
        .iter()
        .filter(|c| c.changes.contains(&cap.id))
        .collect();
    for marker in &proposed {
        let Some(change) = listing.iter().find(|c| c.slug == marker.slug) else {
            out.push(violation(
                &cap.path,
                format!("PROPOSED change `{}` does not list {}", marker.slug, cap.id),
            ));
            continue;
        };
        match &marker.section {
            Some(section) => {
                let entry = format!("{}#{section}", cap.id);
                if !change.affects.contains(&entry) {
                    out.push(violation(
                        &cap.path,
                        format!(
                            "PROPOSED block in section `{section}` needs `{entry}` in the \
                             affects of change `{}`",
                            change.slug
                        ),
                    ));
                }
            }
            None => out.push(violation(
                &cap.path,
                "a PROPOSED block must sit inside one of the seven sections",
            )),
        }
    }
    match cap.state.as_str() {
        "considered" if proposed.is_empty() => out.push(violation(
            &cap.path,
            "state considered requires its candidate contract in PROPOSED blocks",
        )),
        "designed" if !proposed.is_empty() => out.push(violation(
            &cap.path,
            "state designed must have no PROPOSED blocks",
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
        _ => {}
    }
    if cap.state != "delivered" && listing.is_empty() {
        out.push(violation(
            &cap.path,
            format!(
                "state {} requires an open change record listing {}",
                cap.state, cap.id
            ),
        ));
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
        check_capability_state(cap, found, markers, out);
    }
    let change_slugs: BTreeSet<&str> = found.changes.iter().map(|c| c.slug.as_str()).collect();
    for change in &found.changes {
        for target in &change.moved {
            if !change_slugs.contains(target.as_str()) {
                out.push(violation(
                    &change.path,
                    format!("item moved to change `{target}`, but work/changes/{target}.md does not exist"),
                ));
            }
        }
    }
}

/// True for a line that opens with a real SPDX tag.
fn is_spdx_tag(text: &str) -> bool {
    ["SPDX-FileCopyrightText:", "SPDX-License-Identifier:"]
        .iter()
        .any(|tag| text.strip_prefix(tag).is_some_and(|v| !v.trim().is_empty()))
}

/// Why the text is not a generated capability index, or `None` when it is.
///
/// Apart from a leading HTML comment of SPDX lines, every non-blank line lies
/// inside a generated block, and each block marker starts its own line.
fn index_problem(text: &str) -> Option<String> {
    let mut in_header = false;
    let mut header_done = false;
    let mut open = false;
    let mut blocks = 0_usize;
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        let trimmed = line.trim();
        if in_header {
            if trimmed == "-->" {
                in_header = false;
            } else if !is_spdx_tag(trimmed) {
                return Some(format!(
                    "line {number}: the header comment holds more than SPDX tags"
                ));
            }
            continue;
        }
        let spdx_line = trimmed
            .strip_prefix("<!--")
            .and_then(|rest| rest.strip_suffix("-->"))
            .is_some_and(|inner| is_spdx_tag(inner.trim()));
        if trimmed.is_empty() || (!open && blocks == 0 && spdx_line) {
            continue;
        }
        if !open && blocks == 0 && !header_done && line == "<!--" {
            in_header = true;
            header_done = true;
            continue;
        }
        if line.starts_with(GENERATED_OPEN) {
            let kind = line
                .strip_prefix(GENERATED_OPEN)
                .and_then(|rest| rest.strip_suffix(" -->"));
            match kind {
                Some(GENERATED_END) if open => open = false,
                Some(kind) if kind != GENERATED_END && is_kebab(kind) && !open => {
                    open = true;
                    blocks += 1;
                }
                _ => return Some(format!("line {number}: malformed or misplaced marker")),
            }
        } else if !open {
            return Some(format!("line {number}: text outside a generated block"));
        }
    }
    if open {
        Some("a generated block is never closed".to_string())
    } else if blocks == 0 {
        Some("it holds no generated block".to_string())
    } else {
        None
    }
}

fn is_markdown(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "md")
}

fn check_directories(repo: &Repo, out: &mut Vec<String>) {
    for path in repo.keys() {
        let parts: Vec<&str> = path.iter().filter_map(|p| p.to_str()).collect();
        let typed_directory = matches!(
            parts.as_slice(),
            ["docs", "capabilities" | "architecture", _, ..] | ["work", "changes", _, ..]
        );
        if typed_directory && !is_markdown(path) {
            out.push(violation(
                path,
                "only Markdown documents belong in a document directory",
            ));
        }
        if !is_scanned(path) {
            continue;
        }
        if typed_directory && parts.len() > 3 {
            out.push(violation(
                path,
                "documents must sit directly in their directory",
            ));
        }
        if is_capability_index(path)
            && let Some(problem) = repo.get(path).and_then(|t| index_problem(t))
        {
            out.push(violation(
                path,
                format!("a hand-written capability index is not allowed: {problem}"),
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
    for (path, text) in repo.iter().filter(|(p, _)| is_markdown(p)) {
        check_links(repo, path, text, &mut out);
    }
    for (path, text) in repo.iter().filter(|(p, _)| is_scanned(p)) {
        scan_banned(path, text, &mut out);
        scan_placeholders(path, text, &mut out);
        scan_markers(path, text, &mut markers, &mut out);
        check_document(path, text, &mut found, &mut out);
    }
    check_cross_references(repo, &found, &markers, &mut out);
    out.sort();
    out.dedup();
    out
}

/// The `id` and `state` of the document at `path`, or why it cannot be read.
fn identity(repo: &Repo, path: &str) -> Result<(String, Option<String>), String> {
    let text = repo.get(Path::new(path)).ok_or("the document is missing")?;
    let doc = parse_document(text)
        .ok_or("the document has no frontmatter")?
        .map_err(|e| format!("unparseable frontmatter: {e}"))?;
    let id = scalar_field(&doc, "id").unwrap_or_default().to_string();
    Ok((id, scalar_field(&doc, "state").map(str::to_string)))
}

/// Compare the tree with the inventory: every listed document exists with the
/// stated `id` and `state`, every open capability has its change record, and
/// no document exists that the inventory does not list.
fn check_inventory(repo: &Repo, inventory: &Inventory) -> Vec<String> {
    let mut out = Vec::new();
    let mut listed: BTreeSet<String> = BTreeSet::new();
    let mut expect = |path: String, id: String, state: Option<&str>| {
        match identity(repo, &path) {
            Err(reason) => out.push(violation(Path::new(&path), reason)),
            Ok((found_id, found_state)) => {
                if found_id != id {
                    out.push(violation(
                        Path::new(&path),
                        format!("inventory expects id `{id}`, found `{found_id}`"),
                    ));
                }
                if let Some(state) = state
                    && found_state.as_deref() != Some(state)
                {
                    out.push(violation(
                        Path::new(&path),
                        format!(
                            "inventory expects state `{state}`, found `{}`",
                            found_state.unwrap_or_default()
                        ),
                    ));
                }
            }
        }
        listed.insert(path);
    };
    for (slug, state) in inventory.capabilities {
        expect(
            format!("docs/capabilities/{slug}.md"),
            format!("cap:{slug}"),
            Some(state),
        );
        if *state != "delivered" {
            expect(
                format!("work/changes/{slug}.md"),
                format!("change:{slug}"),
                None,
            );
        }
    }
    for slug in inventory.architecture {
        expect(
            format!("docs/architecture/{slug}.md"),
            format!("arch:{slug}"),
            None,
        );
    }
    for path in repo.keys().filter(|p| is_markdown(p)) {
        let known = path.to_str().is_some_and(|p| listed.contains(p));
        if expected_type(path).is_some() && !known && !is_capability_index(path) {
            out.push(violation(
                path,
                "the document is not in the inventory table",
            ));
        }
    }
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
    let mut violations = check(&repo);
    violations.extend(check_inventory(&repo, &INVENTORY));
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
    for kind in ["Architecture", "Capability", "Change"] {
        assert!(
            types.get(kind).copied().unwrap_or(0) >= 1,
            "no {kind} document found; found types: {types:?}"
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
        .map(|name| {
            let extra = if *name == "Purpose" { body } else { "" };
            format!("## {name}\n\n{extra}Text for {name}.\n")
        })
        .collect::<Vec<_>>()
        .join("\n");
        let (code, tests) = if state == "delivered" {
            ("[src/lib.rs]", "[tests/it.rs]")
        } else {
            ("[]", "[]")
        };
        format!(
            "---\n# SPDX-FileCopyrightText: 2026 Example\n# SPDX-License-Identifier: CC-BY-4.0\n\
             type: Capability\nid: cap:{slug}\ntitle: \"Title of {slug}\"\n\
             description: A description of {slug}.\nstate: {state}\n\
             bindings:\n  code: {code}\n  tests: {tests}\n  examples: []\n\
             depends: {depends}\n---\n\n# {slug}\n\n{sections}"
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

    /// A change record with the given `changes` list, `affects` lines and
    /// `items` block (the text after `items:`).
    pub(super) fn change_record(slug: &str, changes: &str, affects: &str, items: &str) -> String {
        let items = if items.is_empty() {
            "items: []\n".to_string()
        } else {
            format!("items:\n{items}")
        };
        format!(
            "---\ntype: Change\nid: change:{slug}\ntitle: Change {slug}\n\
             description: Changes {slug}.\nchanges: {changes}\naffects:\n{affects}\
             contract_effect: changes\n{items}---\n\n## Summary\n\nChanges it.\n"
        )
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
    let mut repo = valid();
    repo.insert(
        PathBuf::from("work/changes/other-work.md"),
        fixtures::change_record("other-work", "[cap:alpha]", "  - cap:alpha#usage\n", ""),
    );
    let record = repo
        .get_mut(Path::new("work/changes/add-alpha.md"))
        .expect("fixture change");
    *record = record.replacen(
        "outcome: implemented",
        "outcome: moved:change:other-work",
        1,
    );
    let violations = check(&repo);
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

// ---------------------------------------------------------------------------
// Fixtures for the hardened rules
// ---------------------------------------------------------------------------

/// Apply each `(from, to, needle)` mutation to `path` and require the needle.
fn assert_cases(path: &str, cases: &[(&str, &str, &str)]) {
    for (from, to, needle) in cases {
        let violations = mutated(path, from, to);
        assert!(
            violations.iter().any(|v| v.contains(needle)),
            "replacing `{from}` with `{to}` should report `{needle}`, got: {violations:#?}"
        );
    }
}

fn assert_clean(violations: &[String]) {
    assert!(violations.is_empty(), "{violations:#?}");
}

const OVERVIEW: &str = "docs/architecture/overview.md";
const ALPHA: &str = "docs/capabilities/alpha.md";
const BETA: &str = "docs/capabilities/beta.md";
const GAMMA: &str = "docs/capabilities/gamma.md";
const CHANGE: &str = "work/changes/add-alpha.md";

// --- capability inventory ---------------------------------------------------

const FIXTURE_INVENTORY: Inventory = Inventory {
    capabilities: &[
        ("alpha", "designed"),
        ("beta", "considered"),
        ("gamma", "delivered"),
    ],
    architecture: &["overview"],
};

/// A valid tree whose documents are exactly `FIXTURE_INVENTORY`.
fn inventory_repo() -> Repo {
    let mut repo = valid();
    repo.remove(Path::new(CHANGE));
    let beta = repo.get_mut(Path::new(BETA)).expect("fixture beta");
    *beta = beta.replace("change:add-alpha", "change:beta");
    for (slug, section) in [("alpha", "contract"), ("beta", "purpose")] {
        repo.insert(
            PathBuf::from(format!("work/changes/{slug}.md")),
            fixtures::change_record(
                slug,
                &format!("[cap:{slug}]"),
                &format!("  - cap:{slug}#{section}\n"),
                "",
            ),
        );
    }
    repo
}

#[test]
fn inventory_accepts_a_matching_tree() {
    let repo = inventory_repo();
    assert_clean(&check(&repo));
    assert_clean(&check_inventory(&repo, &FIXTURE_INVENTORY));
}

#[test]
fn inventory_table_has_the_documented_shape() {
    let slugs: BTreeSet<&str> = INVENTORY.capabilities.iter().map(|(s, _)| *s).collect();
    assert_eq!(slugs.len(), 21, "21 capability slugs, each listed once");
    assert!(slugs.iter().all(|s| is_kebab(s)));
    let count = |wanted: &str| {
        INVENTORY
            .capabilities
            .iter()
            .filter(|(_, state)| *state == wanted)
            .count()
    };
    assert_eq!(count("designed"), 16);
    assert_eq!(count("considered"), 5);
    assert_eq!(INVENTORY.architecture.len(), 2);
}

#[test]
fn inventory_rejects_a_missing_capability() {
    let mut repo = inventory_repo();
    repo.remove(Path::new(BETA));
    let found = check_inventory(&repo, &FIXTURE_INVENTORY);
    assert_hit(&found, "docs/capabilities/beta.md: the document is missing");
}

#[test]
fn inventory_rejects_a_missing_change_record() {
    let mut repo = inventory_repo();
    repo.remove(Path::new("work/changes/alpha.md"));
    let found = check_inventory(&repo, &FIXTURE_INVENTORY);
    assert_hit(&found, "work/changes/alpha.md: the document is missing");
}

#[test]
fn inventory_rejects_a_missing_architecture_document() {
    let mut repo = inventory_repo();
    repo.remove(Path::new(OVERVIEW));
    let found = check_inventory(&repo, &FIXTURE_INVENTORY);
    assert_hit(
        &found,
        "docs/architecture/overview.md: the document is missing",
    );
}

#[test]
fn inventory_rejects_a_state_change() {
    let mut repo = inventory_repo();
    let alpha = repo.get_mut(Path::new(ALPHA)).expect("fixture alpha");
    *alpha = alpha.replace("state: designed", "state: implementing");
    let found = check_inventory(&repo, &FIXTURE_INVENTORY);
    assert_hit(
        &found,
        "inventory expects state `designed`, found `implementing`",
    );
}

#[test]
fn inventory_rejects_an_id_change() {
    let mut repo = inventory_repo();
    let overview = repo.get_mut(Path::new(OVERVIEW)).expect("fixture overview");
    *overview = overview.replace("arch:overview", "arch:other");
    let found = check_inventory(&repo, &FIXTURE_INVENTORY);
    assert_hit(
        &found,
        "inventory expects id `arch:overview`, found `arch:other`",
    );
}

#[test]
fn inventory_rejects_a_document_the_table_does_not_list() {
    let mut repo = inventory_repo();
    repo.insert(
        PathBuf::from("docs/capabilities/extra.md"),
        capability("extra", "designed", "[]", ""),
    );
    let found = check_inventory(&repo, &FIXTURE_INVENTORY);
    assert_hit(
        &found,
        "docs/capabilities/extra.md: the document is not in the inventory table",
    );
}

// --- placeholders and empty sections ----------------------------------------

#[test]
fn rejects_placeholder_tokens() {
    for token in PLACEHOLDER_TOKENS {
        let violations = mutated(
            OVERVIEW,
            "Body text.",
            &format!("Body text. Still {token} here."),
        );
        assert_hit(&violations, "is not resolved text");
    }
    let shouted = concat!("Left TO", "DO: finish");
    let violations = mutated(ALPHA, "Text for Usage.", shouted);
    assert_hit(&violations, "is not resolved text");
    let mixed_case = concat!("It is To Be ", "Written.");
    let violations = mutated(ALPHA, "Text for Usage.", mixed_case);
    assert_hit(&violations, "is not resolved text");
}

#[test]
fn placeholder_scan_spares_code_and_longer_words() {
    let token = concat!("T", "BD");
    let fenced = format!("Body text.\n\n```text\n{token}\n```\n\nUse `{token}` and {token}x.\n");
    assert_clean(&mutated(OVERVIEW, "Body text.\n", &fenced));
}

#[test]
fn rejects_sections_left_empty_by_comments() {
    for comment in [
        "<!-- later -->\n",
        "<!--\nlater\n-->\n",
        "<!-- a --> <!-- b -->\n",
    ] {
        let violations = mutated(ALPHA, "Text for Examples.\n", comment);
        assert_hit(&violations, "section `Examples` is empty");
    }
    let violations = mutated(CHANGE, "Adds both.\n", "<!-- soon -->\n");
    assert_hit(&violations, "section `Summary` is empty");
    let violations = mutated(
        OVERVIEW,
        "# Overview\n\nBody text.\n",
        "<!-- nothing yet -->\n",
    );
    assert_hit(&violations, "architecture body is empty");
}

#[test]
fn comment_stripping_keeps_visible_text() {
    assert_eq!(strip_html_comments("a <!-- x --> b"), "a  b");
    assert_eq!(strip_html_comments("a <!-- never closed"), "a ");
    assert!(!is_blank("<!-- x -->\ntext"));
    assert!(is_blank("  <!-- x -->\n  "));
}

// --- frontmatter scalars ----------------------------------------------------

#[test]
fn rejects_unterminated_quote() {
    assert_cases(
        OVERVIEW,
        &[
            (
                "title: Overview",
                "title: \"Overview",
                "unterminated quoted value",
            ),
            (
                "title: Overview",
                "title: 'Overview",
                "unterminated quoted value",
            ),
            (
                "title: Overview",
                "title: \"Over\" view",
                "text after the closing quote",
            ),
        ],
    );
}

#[test]
fn rejects_unquoted_colon_space() {
    assert_cases(
        OVERVIEW,
        &[
            (
                "title: Overview",
                "title: Built-in: census",
                "contains `: `",
            ),
            ("title: Overview", "title: Overview:", "contains `: `"),
        ],
    );
}

#[test]
fn rejects_leading_indicator_character() {
    for indicator in ['>', '|', '&', '*', '!', '%', '@', '`'] {
        let violations = mutated(
            OVERVIEW,
            "title: Overview",
            &format!("title: {indicator}Overview"),
        );
        assert_hit(&violations, "starts with a YAML indicator character");
    }
}

#[test]
fn rejects_block_scalar_indicator_value() {
    for indicator in [">", "|", ">-", "|+", ">2", "|-"] {
        let violations = mutated(
            OVERVIEW,
            "description: How it fits.",
            &format!("description: {indicator}"),
        );
        assert_hit(&violations, "block scalar indicator");
    }
}

#[test]
fn accepts_comments_after_scalars() {
    let doc = parse_document(
        "---\ntitle: Overview # a note\nquoted: \"a # b\" # c\nsingle: 'It''s # x'  # c\n\
         hash: a#b\nlist: [one, \"two # 2\"] # c\nempty: # nothing\nitems:\n  - claim: x # c\n---\n",
    )
    .expect("has frontmatter")
    .expect("parses");
    let scalar_of = |key: &str| scalar_field(&doc, key).map(str::to_string);
    assert_eq!(scalar_of("title").as_deref(), Some("Overview"));
    assert_eq!(scalar_of("quoted").as_deref(), Some("a # b"));
    assert_eq!(scalar_of("single").as_deref(), Some("It's # x"));
    assert_eq!(scalar_of("hash").as_deref(), Some("a#b"));
    assert_eq!(scalar_of("empty").as_deref(), Some(""));
    assert_eq!(
        list_field(&doc, "list"),
        Some(&["one".to_string(), "two # 2".to_string()][..])
    );
    let violations = mutated(ALPHA, "state: designed", "state: designed # still planned");
    assert_clean(&violations);
    let violations = mutated(
        ALPHA,
        "depends: [cap:beta]",
        "depends: [cap:beta] # one edge",
    );
    assert_clean(&violations);
}

#[test]
fn rejects_bad_scalars_in_every_value_position() {
    assert_cases(
        ALPHA,
        &[
            (
                "depends: [cap:beta]",
                "depends: [cap:beta, \"x]",
                "unterminated quoted value",
            ),
            (
                "depends: [cap:beta]",
                "depends: [cap:beta, a: b]",
                "contains `: `",
            ),
            (
                "depends: [cap:beta]",
                "depends:\n  - \"cap:beta",
                "unterminated quoted value",
            ),
            (
                "depends: [cap:beta]",
                "depends:\n  - >",
                "block scalar indicator",
            ),
            (
                "  code: []\n",
                "  code:\n    - 'src/lib.rs\n",
                "unterminated quoted value",
            ),
            (
                "  code: []\n",
                "  code: [*anchor]",
                "starts with a YAML indicator",
            ),
        ],
    );
    let items = "  - claim: \"unterminated\n";
    let mut repo = valid();
    repo.insert(
        PathBuf::from(CHANGE),
        fixtures::change_record(
            "add-alpha",
            "[cap:alpha, cap:beta]",
            "  - cap:alpha#contract\n  - cap:beta#purpose\n",
            items,
        ),
    );
    assert_hit(&check(&repo), "unterminated quoted value");
    let items = "  - claim: fine\n    outcome: > folded\n";
    repo.insert(
        PathBuf::from(CHANGE),
        fixtures::change_record(
            "add-alpha",
            "[cap:alpha, cap:beta]",
            "  - cap:alpha#contract\n  - cap:beta#purpose\n",
            items,
        ),
    );
    assert_hit(&check(&repo), "starts with a YAML indicator");
}

// --- PROPOSED markers -------------------------------------------------------

#[test]
fn rejects_lowercase_proposed_marker() {
    assert_cases(
        BETA,
        &[(
            "> [!PROPOSED change:add-alpha]",
            "> [!proposed change:add-alpha]",
            "malformed PROPOSED marker",
        )],
    );
    let violations = mutated(
        GAMMA,
        "# gamma\n",
        "# gamma\n\n> [!Proposed change:add-alpha]\n> Left over.\n",
    );
    assert_hit(&violations, "malformed PROPOSED marker");
}

#[test]
fn rejects_proposed_marker_without_space() {
    assert_cases(
        BETA,
        &[(
            "> [!PROPOSED change:add-alpha]",
            ">[!PROPOSED change:add-alpha]",
            "malformed PROPOSED marker",
        )],
    );
}

#[test]
fn rejects_proposed_marker_with_extra_spacing_or_nesting() {
    for marker in [
        ">  [!PROPOSED change:add-alpha]",
        "> > [!PROPOSED change:add-alpha]",
        "- > [!PROPOSED change:add-alpha]",
        "[!PROPOSED change:add-alpha]",
    ] {
        let violations = mutated(BETA, "> [!PROPOSED change:add-alpha]", marker);
        assert_hit(&violations, "malformed PROPOSED marker");
    }
}

#[test]
fn rejects_designed_with_proposed_block() {
    let violations = mutated(
        ALPHA,
        "Text for Contract.\n",
        "> [!PROPOSED change:add-alpha]\n> Not yet.\n\nText for Contract.\n",
    );
    assert_hit(&violations, "state designed must have no PROPOSED blocks");
}

#[test]
fn rejects_proposed_section_missing_from_affects() {
    let violations = mutated(CHANGE, "  - cap:beta#purpose\n", "  - cap:beta#usage\n");
    assert_hit(
        &violations,
        "PROPOSED block in section `purpose` needs `cap:beta#purpose` in the affects of change `add-alpha`",
    );
}

#[test]
fn rejects_proposed_block_outside_a_section() {
    let violations = mutated(
        BETA,
        "# beta\n",
        "# beta\n\n> [!PROPOSED change:add-alpha]\n> Early.\n",
    );
    assert_hit(
        &violations,
        "a PROPOSED block must sit inside one of the seven sections",
    );
}

// --- bindings by state ------------------------------------------------------

#[test]
fn rejects_bindings_before_delivery() {
    for (path, state) in [(ALPHA, "designed"), (BETA, "considered")] {
        for key in BINDING_KEYS {
            let violations = mutated(
                path,
                &format!("  {key}: []"),
                &format!("  {key}: [src/lib.rs]"),
            );
            assert_hit(
                &violations,
                &format!("state {state} must have empty bindings, but `{key}` is not empty"),
            );
        }
    }
}

#[test]
fn rejects_delivered_without_code_or_test_binding() {
    let violations = mutated(GAMMA, "  code: [src/lib.rs]", "  code: []");
    assert_hit(
        &violations,
        "state delivered must bind at least one code path",
    );
    let violations = mutated(GAMMA, "  tests: [tests/it.rs]", "  tests: []");
    assert_hit(&violations, "state delivered must bind at least one test");
}

// --- links ------------------------------------------------------------------

#[test]
fn rejects_broken_reference_definition() {
    assert_cases(
        "README.md",
        &[
            (
                "# Readme",
                "# Readme\n\n[ref]: ./missing.md",
                "broken link `./missing.md`",
            ),
            (
                "# Readme",
                "# Readme\n\n  [ref]: <./missing.md> \"Title\"",
                "broken link `./missing.md`",
            ),
            (
                "# Readme",
                "# Readme\n\n[ref]: docs/capabilities/alpha.md#no-such",
                "broken anchor",
            ),
        ],
    );
}

#[test]
fn rejects_broken_href_and_src_attributes() {
    assert_cases(
        "README.md",
        &[
            (
                "# Readme",
                "# Readme\n\n<a href=\"docs/missing.md\">gone</a>",
                "broken link `docs/missing.md`",
            ),
            (
                "# Readme",
                "# Readme\n\n<img src='docs/missing.svg' width=\"2\">",
                "broken link `docs/missing.svg`",
            ),
            (
                "# Readme",
                "# Readme\n\n<a HREF=\"../../outside.md\">out</a>",
                "leaves the repository",
            ),
        ],
    );
}

#[test]
fn rejects_broken_link_outside_the_document_directories() {
    for path in [
        ".github/PULL_REQUEST_TEMPLATE.md",
        ".github/ISSUE_TEMPLATE/bug.md",
        "crates/tool/README.md",
    ] {
        let mut repo = valid();
        repo.insert(
            PathBuf::from(path),
            "See [gone](docs/missing.md).\n".to_string(),
        );
        assert_hit(&check(&repo), &format!("{path}: line 1: broken link"));
    }
}

#[test]
fn accepts_resolving_reference_and_attribute_links() {
    let mut repo = valid();
    repo.insert(
        PathBuf::from(".github/PULL_REQUEST_TEMPLATE.md"),
        "# Template\n\n[docs]: ../docs/architecture/overview.md\n\
         <img src=\"../docs/architecture/overview.md\" alt=\"x\">\n\
         <a href=\"https://example.invalid/a\">site</a> <a href=\"#template\">top</a>\n\
         A footnote[^1] and `[code]: ./missing.md` stay quiet.\n\n[^1]: A note.\n"
            .to_string(),
    );
    assert_clean(&check(&repo));
}

// --- moved outcomes, typed documents, directories ---------------------------

#[test]
fn rejects_moved_outcome_naming_a_missing_change() {
    let violations = mutated(
        CHANGE,
        "outcome: implemented",
        "outcome: moved:change:ghost",
    );
    assert_hit(
        &violations,
        "item moved to change `ghost`, but work/changes/ghost.md does not exist",
    );
}

#[test]
fn rejects_typed_document_outside_its_directory() {
    let capability_text = capability("stray", "designed", "[]", "");
    let architecture_text = valid()
        .remove(Path::new(OVERVIEW))
        .expect("fixture overview")
        .replace("overview", "stray");
    let change_text = fixtures::change_record("stray", "[cap:alpha]", "  - cap:alpha#usage\n", "");
    for (path, text, kind, home) in [
        (
            "docs/other/stray.md",
            &capability_text,
            "Capability",
            "docs/capabilities",
        ),
        ("docs/stray.md", &change_text, "Change", "work/changes"),
        (
            "work/stray.md",
            &architecture_text,
            "Architecture",
            "docs/architecture",
        ),
        (
            "stray.md",
            &capability_text,
            "Capability",
            "docs/capabilities",
        ),
    ] {
        let mut repo = valid();
        repo.insert(PathBuf::from(path), text.clone());
        assert_hit(
            &check(&repo),
            &format!("{path}: a {kind} document belongs directly in {home}"),
        );
    }
}

#[test]
fn rejects_non_markdown_file_in_a_document_directory() {
    for path in [
        "docs/capabilities/notes.txt",
        "docs/capabilities/ALPHA.MD",
        "docs/architecture/diagram.svg",
        "work/changes/draft.yaml",
        "docs/capabilities/sub/data.json",
    ] {
        let mut repo = valid();
        repo.insert(PathBuf::from(path), String::new());
        assert_hit(
            &check(&repo),
            &format!("{path}: only Markdown documents belong in a document directory"),
        );
    }
}

#[test]
fn accepts_a_generated_capability_index() {
    let mut repo = valid();
    repo.insert(
        PathBuf::from("docs/capabilities/index.md"),
        "<!-- raf:generated:catalogue -->\n| alpha |\n<!-- raf:generated:end -->\n".to_string(),
    );
    assert_clean(&check(&repo));
}

// --- audit: one fixture for every other implemented rule --------------------

#[test]
fn rejects_empty_or_non_scalar_title() {
    assert_cases(
        ALPHA,
        &[
            (
                "title: \"Title of alpha\"",
                "title: \"\"",
                "`title` must be a non-empty string",
            ),
            (
                "title: \"Title of alpha\"",
                "title: [a, b]",
                "`title` must be a non-empty string",
            ),
            (
                "description: A description of alpha.",
                "description:",
                "`description` must be a non-empty string",
            ),
        ],
    );
}

#[test]
fn rejects_bad_id_prefix_or_slug() {
    assert_cases(
        ALPHA,
        &[
            ("id: cap:alpha", "id: change:alpha", "must be `cap:<slug>`"),
            ("id: cap:alpha", "id: cap:Alpha", "must be `cap:<slug>`"),
            ("id: cap:alpha", "id: cap:a--b", "must be `cap:<slug>`"),
            ("id: cap:alpha", "id: cap:al_pha", "must be `cap:<slug>`"),
        ],
    );
    assert_cases(
        CHANGE,
        &[(
            "id: change:add-alpha",
            "id: cap:add-alpha",
            "must be `change:<slug>`",
        )],
    );
    assert_cases(
        OVERVIEW,
        &[("id: arch:overview", "id: overview", "must be `arch:<slug>`")],
    );
}

#[test]
fn rejects_non_scalar_state_and_scalar_bindings() {
    assert_cases(
        ALPHA,
        &[
            (
                "state: designed",
                "state: [designed]",
                "`state` must be a scalar",
            ),
            (
                "bindings:\n  code: []\n  tests: []\n  examples: []\n",
                "bindings: none\n",
                "bindings must be a map",
            ),
        ],
    );
}

#[test]
fn rejects_bad_depends_shapes() {
    assert_cases(
        ALPHA,
        &[
            (
                "depends: [cap:beta]",
                "depends: [beta]",
                "depends entry `beta` is not a cap id",
            ),
            (
                "depends: [cap:beta]",
                "depends: cap:beta",
                "`depends` must be a list",
            ),
            (
                "depends: [cap:beta]",
                "depends: [cap:Beta]",
                "is not a cap id",
            ),
        ],
    );
}

#[test]
fn rejects_self_and_three_node_dependency_cycles() {
    assert_cases(
        ALPHA,
        &[(
            "depends: [cap:beta]",
            "depends: [cap:alpha]",
            "depends cycle: cap:alpha -> cap:alpha",
        )],
    );
    assert_cases(
        BETA,
        &[(
            "depends: []",
            "depends: [cap:gamma]",
            "depends cycle: cap:alpha -> cap:beta -> cap:gamma -> cap:alpha",
        )],
    );
}

#[test]
fn rejects_proposed_change_that_does_not_list_the_capability() {
    let violations = mutated(
        CHANGE,
        "changes: [cap:alpha, cap:beta]",
        "changes: [cap:alpha]",
    );
    assert_hit(
        &violations,
        "PROPOSED change `add-alpha` does not list cap:beta",
    );
}

#[test]
fn rejects_bad_changes_list() {
    assert_cases(
        CHANGE,
        &[
            (
                "changes: [cap:alpha, cap:beta]",
                "changes: []",
                "`changes` must be a non-empty list",
            ),
            (
                "changes: [cap:alpha, cap:beta]",
                "changes: cap:alpha",
                "`changes` must be a non-empty list",
            ),
            (
                "changes: [cap:alpha, cap:beta]",
                "changes: [cap:alpha, beta]",
                "changes entry `beta` is not a cap id",
            ),
        ],
    );
}

#[test]
fn rejects_bad_affects_shapes() {
    assert_cases(
        CHANGE,
        &[
            (
                "affects:\n  - cap:alpha#contract\n  - cap:beta#purpose\n",
                "affects: cap:alpha#contract\n",
                "`affects` must be a list",
            ),
            (
                "cap:alpha#contract",
                "alpha-contract",
                "must be `cap:<slug>#<section>`",
            ),
            (
                "cap:alpha#contract",
                "cap:alpha",
                "must be `cap:<slug>#<section>`",
            ),
        ],
    );
}

#[test]
fn rejects_non_scalar_contract_effect() {
    assert_cases(
        CHANGE,
        &[(
            "contract_effect: changes",
            "contract_effect: [none]",
            "bad contract_effect `<not a scalar>`",
        )],
    );
}

/// Replace the `add-alpha` change record with one whose `items` block is given.
fn with_items(items: &str) -> Vec<String> {
    let mut repo = valid();
    repo.insert(
        PathBuf::from(CHANGE),
        fixtures::change_record(
            "add-alpha",
            "[cap:alpha, cap:beta]",
            "  - cap:alpha#contract\n  - cap:beta#purpose\n",
            items,
        ),
    );
    check(&repo)
}

#[test]
fn rejects_item_shape_errors() {
    for (items, needle) in [
        (
            "  - claim: \"x\"\n    owner: me\n",
            "unknown item key `owner`",
        ),
        (
            "  - outcome: dropped\n",
            "every item needs a non-empty `claim`",
        ),
        ("  - claim: \"\"\n", "every item needs a non-empty `claim`"),
        (
            "  - claim: \"x\"\n    claim: \"y\"\n",
            "duplicate entry key `claim`",
        ),
        (
            "  - claim: \"x\"\n  - plain\n",
            "list mixes scalar and map entries",
        ),
        (
            "  - plain\n    outcome: dropped\n",
            "continuation without a map entry",
        ),
        (
            "  - claim: \"x\"\n    !!!\n",
            "cannot parse list entry line",
        ),
        ("  - claim: \"x\"\n  junk\n", "cannot parse list line"),
        ("  - one\n  - two\n", "`items` must be a list of maps"),
    ] {
        let violations = with_items(items);
        assert!(
            violations.iter().any(|v| v.contains(needle)),
            "items {items:?} should report `{needle}`, got: {violations:#?}"
        );
    }
    assert_hit(
        &mutated(CHANGE, "items:\n", "items: nothing\nignored_by_test:\n"),
        "`items` must be a list of maps",
    );
}

#[test]
fn rejects_frontmatter_shape_errors() {
    assert_cases(
        OVERVIEW,
        &[
            (
                "title: Overview",
                "title: Overview\ntitle: Again",
                "duplicate key `title`",
            ),
            (
                "title: Overview",
                "title: Overview\nBad Key: 1",
                "cannot parse frontmatter line",
            ),
        ],
    );
    assert_cases(
        ALPHA,
        &[
            (
                "  code: []\n",
                "  code: []\n  code: []\n",
                "duplicate map key `code`",
            ),
            (
                "  code: []\n",
                "  code: foo\n",
                "map value of `code` must be a list",
            ),
            ("  code: []\n", "  code: [a\n", "unterminated inline list"),
            (
                "  code: []\n",
                "  code: [a,,b]\n",
                "empty entry in inline list",
            ),
            (
                "  code: []\n",
                "  code: []\n    - stray\n",
                "list item without a key",
            ),
            (
                "  code: []\n",
                "  code: []\n      deep\n",
                "cannot parse map line",
            ),
        ],
    );
    assert_cases(
        ALPHA,
        &[(
            "type: Capability",
            "title: again\ntype: Capability",
            "duplicate key",
        )],
    );
}

#[test]
fn rejects_missing_type_key() {
    assert_cases(
        OVERVIEW,
        &[("type: Architecture\n", "", "missing required key `type`")],
    );
}

#[test]
fn rejects_duplicate_section() {
    assert_cases(
        ALPHA,
        &[(
            "## Usage\n\nText for Usage.\n",
            "## Usage\n\nText for Usage.\n\n## Usage\n\nAgain.\n",
            "a section appears more than once",
        )],
    );
}

#[test]
fn rejects_generated_marker_errors() {
    assert_cases(
        OVERVIEW,
        &[
            (
                "Body text.",
                "Body text.\n\n<!-- raf:generated:end -->\n",
                "generated end marker without a start",
            ),
            (
                "Body text.",
                "Body text.\n\n<!-- raf:generated:a -->\n<!-- raf:generated:b -->\n<!-- raf:generated:end -->\n",
                "generated block inside another generated block",
            ),
            (
                "Body text.",
                "Body text.\n\n<!-- raf:generated: -->\n",
                "malformed generated marker",
            ),
            (
                "Body text.",
                "Body text.\n\n<!-- raf:generated:Bad Kind -->\n",
                "malformed generated marker",
            ),
            (
                "Body text.",
                "Body text.\n\n<!-- raf:generated:matrix\n",
                "malformed generated marker",
            ),
        ],
    );
}

// --- reading files ----------------------------------------------------------

/// A scratch directory removed when dropped, named by process id, a
/// nanosecond timestamp and a counter so no two runs share a path.
#[cfg(unix)]
struct Scratch(PathBuf);

#[cfg(unix)]
impl Scratch {
    fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!(
            "raf-docs-contract-{}-{nanos}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&dir).expect("scratch directory");
        Self(dir)
    }
}

#[cfg(unix)]
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(unix)]
#[test]
fn refuses_non_regular_files_instead_of_blocking() {
    use std::process::Command;
    use std::sync::mpsc;
    use std::time::Duration;

    let scratch = Scratch::new();
    let dir = scratch.0.clone();
    let made = Command::new("mkfifo")
        .arg(dir.join("trap.md"))
        .status()
        .expect("mkfifo runs");
    assert!(made.success(), "mkfifo failed");
    std::fs::write(dir.join("plain.md"), "text").expect("plain file");
    std::os::unix::fs::symlink(dir.join("plain.md"), dir.join("link.md")).expect("symlink");

    let (sender, receiver) = mpsc::channel();
    let root = dir.clone();
    std::thread::spawn(move || {
        let _ = sender.send(read_regular(&root, Path::new("trap.md")));
    });
    let outcome = receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("reading a FIFO must not block");
    assert!(
        outcome
            .as_ref()
            .is_err_and(|e| e.contains("not a regular file")),
        "{outcome:?}"
    );
    let link = read_regular(&dir, Path::new("link.md"));
    assert!(
        link.as_ref().is_err_and(
            |e| e.contains("link.md") && e.contains("symlinks are not allowed in the tree")
        ),
        "{link:?}"
    );
    assert_eq!(
        read_regular(&dir, Path::new("plain.md")),
        Ok(b"text".to_vec())
    );
}

#[cfg(unix)]
#[test]
fn refuses_a_file_swapped_in_after_inspection() {
    let scratch = Scratch::new();
    let dir = &scratch.0;
    std::fs::write(dir.join("first.md"), "one").expect("first file");
    std::fs::write(dir.join("second.md"), "two").expect("second file");
    let inspected = std::fs::symlink_metadata(dir.join("first.md")).expect("stat first");
    let opened = std::fs::File::open(dir.join("second.md"))
        .and_then(|f| f.metadata())
        .expect("stat second");
    let verdict = support::check_handle("first.md", &inspected, &opened);
    assert!(
        verdict.is_err_and(|e| e.contains("changed while it was being opened")),
        "a different inode must be refused"
    );
    let same = std::fs::File::open(dir.join("first.md"))
        .and_then(|f| f.metadata())
        .expect("stat first handle");
    assert_eq!(support::check_handle("first.md", &inspected, &same), Ok(()));
}

// --- generated index, placeholder spellings, unquoted attributes ------------

/// Assembled in pieces so that this file carries no SPDX tag of its own here.
const SPDX_ID: &str = concat!("SPDX-License-", "Identifier: CC-BY-4.0");

fn index_repo(text: &str) -> Vec<String> {
    let mut repo = valid();
    repo.insert(
        PathBuf::from("docs/capabilities/index.md"),
        text.to_string(),
    );
    check(&repo)
}

#[test]
fn accepts_a_generated_index_with_an_spdx_header() {
    let header = format!("<!--\nSPDX-FileCopyrightText: 2026 Example\n{SPDX_ID}\n-->\n\n");
    let body = "<!-- raf:generated:catalogue -->\n| alpha |\n\n<!-- raf:generated:end -->\n";
    assert_clean(&index_repo(&format!("{header}{body}")));
    assert_clean(&index_repo(&format!("<!-- {SPDX_ID} -->\n{body}")));
}

#[test]
fn rejects_an_index_with_prose_outside_the_blocks() {
    let block = "<!-- raf:generated:catalogue -->\n| alpha |\n<!-- raf:generated:end -->\n";
    for text in [
        format!("# Index\n\n{block}"),
        format!("{block}\nHand-written note.\n"),
        format!("<!--\n{SPDX_ID}\nprose\n-->\n{block}"),
    ] {
        assert_hit(
            &index_repo(&text),
            "hand-written capability index is not allowed",
        );
    }
}

#[test]
fn rejects_an_index_whose_marker_is_only_in_a_code_span() {
    let text = "Generated by `<!-- raf:generated:catalogue -->` honestly.\n";
    assert_hit(&index_repo(text), "text outside a generated block");
    let fenced_in_span =
        "`<!-- raf:generated:catalogue -->`\n| alpha |\n`<!-- raf:generated:end -->`\n";
    assert_hit(
        &index_repo(fenced_in_span),
        "hand-written capability index is not allowed",
    );
}

#[test]
fn rejects_an_index_with_a_mid_line_marker() {
    for text in [
        "prose <!-- raf:generated:catalogue --> | alpha |\n<!-- raf:generated:end -->\n",
        "<!-- raf:generated:catalogue -->\n| alpha |\ntrailing <!-- raf:generated:end -->\n x\n",
        "  <!-- raf:generated:catalogue -->\n| alpha |\n<!-- raf:generated:end -->\n",
        "<!-- raf:generated:catalogue -->\n",
        "\n",
    ] {
        assert_hit(
            &index_repo(text),
            "hand-written capability index is not allowed",
        );
    }
}

#[test]
fn rejects_placeholder_spellings() {
    for spelling in [
        concat!("t", "bd"),
        concat!("T.B", ".D."),
        concat!("t.b", ".d"),
        concat!("To", "Do"),
        concat!("fix", "Me"),
        concat!("Fix", "ME:"),
        concat!("x", "xX"),
        concat!("to  be", "\twritten"),
        concat!("TO BE ", "WRITTEN"),
    ] {
        let violations = mutated(
            OVERVIEW,
            "Body text.",
            &format!("Body text. Still {spelling} soon."),
        );
        assert_hit(&violations, "is not resolved text");
    }
}

#[test]
fn placeholder_scan_spares_longer_words_and_dotted_prose() {
    let text = format!(
        "Body text. {} and {} and e.g. a.b.c, {}s.\n",
        concat!("t", "bdx"),
        concat!("to", "dos"),
        concat!("x", "xxx"),
    );
    assert_clean(&mutated(OVERVIEW, "Body text.\n", &text));
}

#[test]
fn rejects_broken_unquoted_attribute_values() {
    assert_cases(
        "README.md",
        &[
            (
                "# Readme",
                "# Readme\n\n<a href=docs/missing.md>x</a>",
                "broken link `docs/missing.md`",
            ),
            (
                "# Readme",
                "# Readme\n\n<img src=docs/missing.svg />",
                "broken link `docs/missing.svg`",
            ),
            (
                "# Readme",
                "# Readme\n\n<a href=../../outside.md>x</a>",
                "leaves the repository",
            ),
        ],
    );
    let mut repo = valid();
    repo.insert(
        PathBuf::from(".github/note.md"),
        "# Note\n\n<a href=../docs/architecture/overview.md>ok</a>\n".to_string(),
    );
    assert_clean(&check(&repo));
}

#[test]
fn rejects_an_index_header_with_unknown_tags_or_in_the_wrong_place() {
    let block = "<!-- raf:generated:catalogue -->\n| alpha |\n<!-- raf:generated:end -->\n";
    for text in [
        format!("<!-- SPDX-Anything: x -->\n{block}"),
        format!("<!--\nSPDX-Anything: x\n-->\n{block}"),
        format!("{block}<!-- {SPDX_ID} -->\n"),
        format!("{block}\n<!--\n{SPDX_ID}\n-->\n{block}"),
    ] {
        assert_hit(
            &index_repo(&text),
            "hand-written capability index is not allowed",
        );
    }
}

#[test]
fn rejects_an_index_with_an_indented_marker() {
    for indent in [" ", "    ", "\t"] {
        let text = format!(
            "{indent}<!-- raf:generated:catalogue -->\n| alpha |\n<!-- raf:generated:end -->\n"
        );
        assert_hit(&index_repo(&text), "text outside a generated block");
        let text = format!(
            "<!-- raf:generated:catalogue -->\n| alpha |\n{indent}<!-- raf:generated:end -->\n"
        );
        assert_hit(
            &index_repo(&text),
            "hand-written capability index is not allowed",
        );
    }
}

#[test]
fn reads_attributes_with_spaces_around_the_equals_sign() {
    assert_cases(
        "README.md",
        &[
            (
                "# Readme",
                "# Readme\n\n<a href = \"docs/missing.md\">x</a>",
                "broken link `docs/missing.md`",
            ),
            (
                "# Readme",
                "# Readme\n\n<img src\t=  docs/missing.svg>",
                "broken link `docs/missing.svg`",
            ),
        ],
    );
}

#[test]
fn does_not_read_attributes_that_only_end_in_href_or_src() {
    let text = "# Readme\n\n<a data-href=\"docs/missing.md\" x-src='docs/none.svg' data_src=docs/n.svg>x</a>\n\
                Use the href attribute and src = nothing here.\n";
    assert_clean(&mutated("README.md", "# Readme\n", text));
}

#[test]
fn rejects_broken_attributes_in_tags_split_across_lines() {
    for (text, needle) in [
        (
            "<img\n  src=\"docs/missing.svg\"\n  alt=\"x\">\n",
            "README.md: line 4: broken link `docs/missing.svg`",
        ),
        (
            "<img\n  alt=\"x\"\n  src=docs/missing.svg />\n",
            "README.md: line 5: broken link `docs/missing.svg`",
        ),
        (
            "<img\nalt=\"x\"\nsrc = 'docs/missing.svg'\nwidth=\"2\">\n",
            "README.md: line 5: broken link `docs/missing.svg`",
        ),
        (
            "<a\n  href=\"docs/missing.md\">gone</a>\n",
            "README.md: line 4: broken link `docs/missing.md`",
        ),
        (
            "<a title=\"x>y\" href=\"./nope.md\">gone</a>\n",
            "README.md: line 3: broken link `./nope.md`",
        ),
        (
            "<a title=\"two\nlines>\" href=\"./nope.md\">gone</a>\n",
            "README.md: line 4: broken link `./nope.md`",
        ),
    ] {
        let violations = mutated("README.md", "# Readme\n", &format!("# Readme\n\n{text}"));
        assert_hit(&violations, needle);
    }
}

#[test]
fn tag_scan_spares_prose_autolinks_and_closed_tags() {
    let text = "# Readme\n\nWrite src = nothing or href = nothing; if a < b then x > y.\n\
                <https://example.invalid/?src=a&href=b>\n\
                <p align=\"center\">\n  <img src=\"docs/architecture/overview.md\" alt=\"ok\">\n</p>\n\
                After the tag, src = nothing is prose again.\n";
    assert_clean(&mutated("README.md", "# Readme\n", text));
}

#[test]
fn rejects_an_index_header_tag_without_a_value() {
    let block = "<!-- raf:generated:catalogue -->\n| alpha |\n<!-- raf:generated:end -->\n";
    for header in [
        "<!-- SPDX-FileCopyrightText: -->\n".to_string(),
        "<!--\nSPDX-FileCopyrightText:   \n-->\n".to_string(),
        format!("<!--\n{SPDX_ID}\nSPDX-FileCopyrightText:\n-->\n"),
    ] {
        assert_hit(
            &index_repo(&format!("{header}{block}")),
            "hand-written capability index is not allowed",
        );
    }
}

#[test]
fn tag_state_does_not_survive_a_blank_line_or_a_fence() {
    let broken = "<img src=\"./nope.png\">\n";
    for prose in [
        "a<b it's fine\n\n",
        "a<b \"quoted\n\n",
        "a<b it's fine\n\n```text\ncode\n```\n",
    ] {
        let violations = mutated(
            "README.md",
            "# Readme\n",
            &format!("# Readme\n\n{prose}{broken}"),
        );
        assert!(
            violations
                .iter()
                .any(|v| v.contains("broken link `./nope.png`")),
            "{prose:?}: {violations:#?}"
        );
    }
}

#[test]
fn reads_an_attribute_value_that_starts_on_the_next_line() {
    for (text, needle) in [
        (
            "<img src=\n  \"docs/missing.svg\" alt=\"x\">\n",
            "README.md: line 4: broken link `docs/missing.svg`",
        ),
        (
            "<a href =\n  docs/missing.md>gone</a>\n",
            "README.md: line 4: broken link `docs/missing.md`",
        ),
    ] {
        let violations = mutated("README.md", "# Readme\n", &format!("# Readme\n\n{text}"));
        assert_hit(&violations, needle);
    }
    let fine = "<img src=\n  \"docs/architecture/overview.md\">\n";
    assert_clean(&mutated(
        "README.md",
        "# Readme\n",
        &format!("# Readme\n\n{fine}"),
    ));
}
