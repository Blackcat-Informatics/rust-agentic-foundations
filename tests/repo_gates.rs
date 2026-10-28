// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Repository gates that used to be shell recipes.
//!
//! The gates run over the git index, read under `RAF_GATE_ROOT` (default: the
//! package directory):
//!
//! * the Mulan licence texts match their pinned digest, and the Chinese
//!   licensing explanation matches the fingerprint in its back-translation
//!   record;
//! * no tracked text file carries an issue or pull request number or
//!   development-process wording;
//! * no tracked file has a Python extension or Python interpreter shebang;
//! * `CITATION.cff` and `[workspace.package]` carry the same version;
//! * the fuzz harness workspace carries the same `[workspace.lints]` tables as
//!   the root, and every workspace manifest inherits them;
//! * the fuzz harness `deny.toml` is the root `deny.toml` plus its one
//!   crate-scoped licence exception;
//! * every tracked first-party Cargo manifest rejects the obsolete JSON result
//!   dependency, including aliases, target tables and workspace declarations;
//! * no tracked Rust file under `src/`, `tests/` or `examples/` of any crate in
//!   `crates/`, or under the root `tests/`, names a clock or a wait on one, so
//!   no test asserts time; a name with a suffix, such as `sleep_ms` or
//!   `wait_timeout_while`, counts as the name. `raf-source` also refuses
//!   `Duration` and `elapsed`; the other crates may name them, because the
//!   workplan runner's limits and its scripted data are made of them. The
//!   examples of `raf-source` are judged by the same stricter rule as its
//!   sources and tests; clippy's disallowed types and methods also cover them.
//!   Exactly two files are exempt: the runner's one clock reading and this
//!   file. Native source identities and bounded Rust syntax separately admit
//!   the standard production clock and its runner selection, while preserving
//!   inherited test-only implementations.
//!
//! Each gate is a pure function over text or bytes, and each has a named
//! negative fixture below that breaks exactly one thing. Test names start with
//! `licenses_`, `issue_refs_`, `citation_`, `lints_`, `deny_` or
//! `time_in_crates_`, `python_` or `rdf_`, which names the gate a test belongs to;
//! `make licenses` and `make issue-refs` run the whole binary. The dependency
//! policy parses TOML, production clock admission uses the native Rust source
//! model, and the remaining helpers operate directly on text or bytes.

mod support;

#[path = "repo_gates/dependencies.rs"]
mod dependencies;

#[path = "repo_gates/python.rs"]
mod python;

#[path = "repo_gates/ci.rs"]
mod ci;

use std::path::{Path, PathBuf};

use raf_source::syn::{self, ext::IdentExt as _, spanned::Spanned as _, visit::Visit as _};

use support::{banned_in_line, decode_text, gate_root, read_tracked, tracked_paths};

/// Digest both Mulan texts must match.
const MULAN_SHA256: &str = "eb7a1d713eb919b146787629e22e4c975cb701f529a65d4d7e0fcd417558bf1c";
const MULAN_FILES: [&str; 2] = ["LICENSE-MULAN", "LICENSES/MulanPSL-2.0.txt"];
const BACKTRANSLATION: &str = "docs/LICENSING-BACKTRANSLATION.md";
const ZH_HANS: &str = "docs/LICENSING.zh-Hans.md";
const FINGERPRINT_LABEL: &str = "Source SHA-256";

// ---------------------------------------------------------------------------
// SHA-256 (FIPS 180-4)
// ---------------------------------------------------------------------------

/// The lowercase hexadecimal SHA-256 digest of `data`.
fn sha256_hex(data: &[u8]) -> String {
    purrdf_core::content_store::ContentDigest::of(data).to_hex()
}

// ---------------------------------------------------------------------------
// Gate functions
// ---------------------------------------------------------------------------

/// A problem with one text, or `None` when its digest is the `pinned` one.
fn digest_problem(name: &str, bytes: &[u8], pinned: &str) -> Option<String> {
    let got = sha256_hex(bytes);
    (got != pinned).then(|| format!("{name}: sha256 {got} != pinned {pinned}"))
}

/// The 64-digit hash on the first line of `record` that names the source.
fn recorded_fingerprint(record: &str) -> Option<String> {
    let line = record.lines().find(|l| l.contains(FINGERPRINT_LABEL))?;
    let bytes = line.as_bytes();
    (0..bytes.len().saturating_sub(63)).find_map(|start| {
        let window = bytes.get(start..start + 64)?;
        let window = std::str::from_utf8(window).ok()?;
        let hex = purrdf_hash::hex::decode_32_canonical(window).is_some();
        let before_ok = start == 0 || purrdf_hash::hex::nibble(bytes[start - 1]).is_none();
        let after_ok = bytes
            .get(start + 64)
            .is_none_or(|byte| purrdf_hash::hex::nibble(*byte).is_none());
        (hex && before_ok && after_ok).then(|| window.to_owned())
    })
}

/// A problem with the zh-Hans fingerprint, or `None` when it matches.
fn fingerprint_problem(record: &str, document: &[u8]) -> Option<String> {
    let got = sha256_hex(document);
    match recorded_fingerprint(record) {
        None => Some(format!(
            "{BACKTRANSLATION}: no 64-digit hash on a `{FINGERPRINT_LABEL}` line"
        )),
        Some(want) if want != got => Some(format!(
            "{ZH_HANS}: sha256 {got} != recorded {want} in {BACKTRANSLATION}"
        )),
        Some(_) => None,
    }
}

/// One `path:line: kind` entry per banned line in `text`.
fn scan_text(path: &Path, text: &str) -> Vec<String> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            banned_in_line(line).map(|kind| format!("{}:{}: {kind}", path.display(), index + 1))
        })
        .collect()
}

/// Scan every tracked text file under `root`. Returns the hits and the number
/// of text files scanned; binary files (a NUL byte) are recognised by content
/// and left out of the count. Invalid UTF-8 without a NUL byte is scanned
/// lossily.
fn scan_tree(root: &Path, paths: &[PathBuf]) -> (Vec<String>, usize) {
    let mut hits = Vec::new();
    let mut scanned = 0;
    for path in paths {
        let bytes = read_tracked(root, path);
        if let Some(text) = decode_text(&bytes) {
            scanned += 1;
            hits.extend(scan_text(path, &text));
        }
    }
    (hits, scanned)
}

/// One clock rule: the Rust files it judges may not name any of `words`.
struct ClockRule {
    words: &'static [&'static str],
}

/// The spellings that read a clock or wait on one, each matched as a word (see
/// [`has_word`]): the `std::time` and `core::time` modules, the timer modules of
/// async runtimes, the types that carry a reading or a span, a sleep, and the
/// waits and socket options that carry a timeout.
const SOURCE_CLOCK_WORDS: &[&str] = &[
    "std::time",
    "core::time",
    "tokio::time",
    "Instant",
    "Duration",
    "SystemTime",
    "sleep",
    "recv_timeout",
    "wait_timeout",
    "park_timeout",
    "set_read_timeout",
    "set_write_timeout",
    "elapsed",
];

/// The spellings the other crates may not name. `Duration`, `elapsed`,
/// `timeout`, `with_timeout` and `wall` are absent on purpose: the workplan
/// runner's limits and its scripted data are made of them.
const CRATE_CLOCK_WORDS: &[&str] = &[
    "Instant",
    "SystemTime",
    "UNIX_EPOCH",
    "tokio::time",
    "core::time",
    "sleep",
    "recv_timeout",
    "wait_timeout",
    "park_timeout",
    "set_read_timeout",
    "set_write_timeout",
];

/// The rule of the sources and tests of `raf-source`.
const SOURCE_RULE: ClockRule = ClockRule {
    words: SOURCE_CLOCK_WORDS,
};

/// The rule of every other crate and of the root `tests/`.
const CRATE_RULE: ClockRule = ClockRule {
    words: CRATE_CLOCK_WORDS,
};

/// The crate whose sources, tests and examples follow [`SOURCE_RULE`].
const SOURCE_CRATE: &str = "raf-source";

/// The directory that holds the crates, and the directories of each crate that
/// a rule judges, so a crate added under it is judged without a change here.
const CRATES_DIR: &str = "crates";
const CRATE_DIRS: [&str; 3] = ["src", "tests", "examples"];

/// The root directory judged by [`CRATE_RULE`].
const ROOT_TESTS_DIR: &str = "tests";

/// The two files no rule judges: the one place the runner reads the clock, and
/// the gate's own file, which defines the word tables and the fixtures.
const CLOCK_CLOCK_FILE: &str = "crates/raf-workplan/src/exec/clock.rs";
const CLOCK_GATE_FILE: &str = "tests/repo_gates.rs";
const CLOCK_EXEMPT: [&str; 2] = [CLOCK_CLOCK_FILE, CLOCK_GATE_FILE];

const CLOCK_TRAIT: &str = "raf_workplan::exec::clock::Clock";
const WALL_CLOCK: &str = "raf_workplan::exec::clock::WallClock";
const CLOCK_SYSTEM_FILE: &str = "crates/raf-workplan/src/exec/system.rs";
const CLOCK_PATTERN_FILE: &str = "crates/raf-workplan/src/exec/pattern.rs";
const CLOCK_MACROS: &[&str] = &[
    "format",
    "write",
    "writeln",
    "vec",
    "matches",
    "cfg",
    "assert",
    "assert_eq",
    "debug_assert",
];
const CLOCK_DERIVES: &[&str] = &[
    "Debug",
    "Clone",
    "Copy",
    "Default",
    "PartialEq",
    "Eq",
    "PartialOrd",
    "Ord",
    "Hash",
];

/// Inspect real token groups, not literal contents or comments. Even a standard
/// expression macro must not hide items or another opaque macro invocation.
fn clock_macro_tokens(mut cursor: syn::buffer::Cursor<'_>) -> bool {
    let mut qualified = false;
    while !cursor.eof() {
        if let Some((name, next)) = cursor.ident() {
            if [
                "impl",
                "use",
                "extern",
                "macro_rules",
                "mod",
                "trait",
                "struct",
                "enum",
                "union",
                "type",
                "fn",
            ]
            .iter()
            .any(|word| name == *word)
            {
                return false;
            }
            if let Some((bang, after)) = next.punct()
                && bang.as_char() == '!'
                && after.any_group().is_some()
                && (qualified || !CLOCK_MACROS.iter().any(|word| name == *word))
            {
                return false;
            }
            cursor = next;
            qualified = false;
        } else if let Some((inside, _, _, next)) = cursor.any_group() {
            if !clock_macro_tokens(inside) {
                return false;
            }
            cursor = next;
            qualified = false;
        } else if let Some((punctuation, next)) = cursor.punct() {
            if punctuation.as_char() == '#' {
                return false;
            }
            qualified = punctuation.as_char() == ':';
            cursor = next;
        } else if let Some((_, next)) = cursor.token_tree() {
            cursor = next;
            qualified = false;
        } else {
            return false;
        }
    }
    true
}

fn reserved_clock_use(tree: &syn::UseTree) -> bool {
    fn binding(tree: &syn::UseTree, parent: Option<&syn::Ident>) -> bool {
        let reserved = |name: &syn::Ident| {
            let name = name.unraw();
            CLOCK_MACROS
                .iter()
                .chain(CLOCK_DERIVES)
                .chain(["validated", "fmt", "std"].iter())
                .any(|word| name == *word)
        };
        match tree {
            syn::UseTree::Path(path) => binding(&path.tree, Some(&path.ident)),
            syn::UseTree::Name(name) => reserved(if name.ident == "self" {
                parent.unwrap_or(&name.ident)
            } else {
                &name.ident
            }),
            syn::UseTree::Rename(name) => reserved(&name.rename),
            syn::UseTree::Group(group) => group.items.iter().any(|item| binding(item, parent)),
            syn::UseTree::Glob(_) => true,
        }
    }
    let mut direct = Vec::new();
    if use_names(tree, "", &mut direct) && direct == ["std::fmt"] {
        return false;
    }
    binding(tree, None)
}

fn token_name<'a>(
    cursor: syn::buffer::Cursor<'a>,
    expected: &str,
) -> Option<syn::buffer::Cursor<'a>> {
    cursor
        .ident()
        .and_then(|(name, next)| (name == expected).then_some(next))
}

fn token_punctuation(
    cursor: syn::buffer::Cursor<'_>,
    expected: char,
) -> Option<syn::buffer::Cursor<'_>> {
    cursor
        .punct()
        .and_then(|(punctuation, next)| (punctuation.as_char() == expected).then_some(next))
}

fn token_variable<'a>(
    cursor: syn::buffer::Cursor<'a>,
    expected: &str,
) -> Option<syn::buffer::Cursor<'a>> {
    token_punctuation(cursor, '$').and_then(|next| token_name(next, expected))
}

fn standard_derives(list: &syn::MetaList) -> bool {
    list.parse_args_with(syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
        .is_ok_and(|paths| {
            paths
                .iter()
                .all(|path| CLOCK_DERIVES.iter().any(|name| source_path(path, &[name])))
        })
}

fn validated_impl(cursor: syn::buffer::Cursor<'_>) -> bool {
    let inherent = token_variable(cursor, "name");
    let display = token_name(cursor, "fmt")
        .and_then(|next| token_punctuation(next, ':'))
        .and_then(|next| token_punctuation(next, ':'))
        .and_then(|next| token_name(next, "Display"))
        .and_then(|next| token_name(next, "for"))
        .and_then(|next| token_variable(next, "name"));
    inherent
        .or(display)
        .is_some_and(|next| next.any_group().is_some())
}

/// Admit only documented metadata and standard derives in the fixed producer.
/// Doc comments become doc attributes in native tokens and remain nonsemantic.
fn validated_attribute(cursor: syn::buffer::Cursor<'_>) -> bool {
    if token_variable(cursor, "meta").is_some_and(syn::buffer::Cursor::eof) {
        return true;
    }
    syn::parse2::<syn::Meta>(cursor.token_stream()).is_ok_and(|meta| {
        source_path(meta.path(), &["doc"]) || source_path(meta.path(), &["must_use"])
            || matches!(&meta, syn::Meta::List(list) if source_path(&list.path, &["derive"]) && standard_derives(list))
    })
}

fn validated_tokens(mut cursor: syn::buffer::Cursor<'_>) -> bool {
    while !cursor.eof() {
        if let Some((name, next)) = cursor.ident() {
            if name == "impl" && !validated_impl(next) {
                return false;
            }
            if ["use", "extern", "macro_rules", "mod", "trait", "type"]
                .iter()
                .any(|word| name == *word)
                || token_punctuation(next, '!').is_some_and(|after| after.any_group().is_some())
            {
                return false;
            }
            cursor = next;
        } else if let Some(after) = token_punctuation(cursor, '#') {
            let Some((attribute, _, _, next)) = after.any_group() else {
                return false;
            };
            if !validated_attribute(attribute) {
                return false;
            }
            cursor = next;
        } else if let Some((inside, _, _, next)) = cursor.any_group() {
            if !validated_tokens(inside) {
                return false;
            }
            cursor = next;
        } else if let Some((_, next)) = cursor.token_tree() {
            cursor = next;
        } else {
            return false;
        }
    }
    true
}

/// The existing producer's single matcher admits only doc attributes, a name
/// and a Pattern path. Its body may emit inherent and standard Display impls.
fn validated_definition(value: &syn::Macro) -> bool {
    let buffer = syn::buffer::TokenBuffer::new2(value.tokens.clone());
    let Some((matcher, _, _, after)) = buffer.begin().any_group() else {
        return false;
    };
    let expected = syn::parse_str::<syn::ItemMacro>(
        "macro_rules! admitted { ($(#[$meta:meta])* $name:ident, $pattern:expr) => {}; }",
    )
    .expect("fixed native matcher syntax");
    let expected = syn::buffer::TokenBuffer::new2(expected.mac.tokens);
    let expected = expected.begin().any_group().expect("fixed matcher group").0;
    if matcher.token_stream().to_string() != expected.token_stream().to_string() {
        return false;
    }
    let Some(after) = token_punctuation(after, '=').and_then(|next| token_punctuation(next, '>'))
    else {
        return false;
    };
    let Some((body, _, _, after)) = after.any_group() else {
        return false;
    };
    validated_tokens(body) && token_punctuation(after, ';').unwrap_or(after).eof()
}

fn validated_call(value: &syn::Macro) -> bool {
    value
        .parse_body_with(|input: syn::parse::ParseStream<'_>| {
            let attributes = input.call(syn::Attribute::parse_outer)?;
            if attributes
                .iter()
                .any(|attribute| !source_path(attribute.path(), &["doc"]))
            {
                return Err(input.error("validated producer accepts only forwarded doc attributes"));
            }
            let _: syn::Ident = input.parse()?;
            let _: syn::Token![,] = input.parse()?;
            let path: syn::Path = input.parse()?;
            if path.segments.len() != 2
                || path.segments[0].ident != "Pattern"
                || path
                    .segments
                    .iter()
                    .any(|segment| !matches!(segment.arguments, syn::PathArguments::None))
            {
                return Err(input.error("validated producer requires a direct Pattern variant"));
            }
            if !input.is_empty() {
                let _: syn::Token![,] = input.parse()?;
            }
            Ok(())
        })
        .is_ok()
}

fn clock_macro_definitions(workspace: &raf_source::Workspace) -> Vec<String> {
    workspace.items().filter(|item| item.test == raf_source::TestRole::None
        && item.kind == raf_source::SymbolKind::MacroRules
        && item.path.starts_with("raf_workplan::")
        && (CLOCK_MACROS.iter().chain(CLOCK_DERIVES).any(|name| item.name == *name)
            || (item.name == "validated" && item.file.as_str() != CLOCK_PATTERN_FILE)))
        .map(|item| format!("{}:{}: production macro must not shadow an admitted standard macro, derive or validated producer", item.file, item.name_line)).collect()
}

/// Admit the production clock through native identities and a deliberately
/// closed source vocabulary. This does not expand macros or evaluate host time.
fn clock_implementation_problems(workspace: &raf_source::Workspace) -> Result<Vec<String>, String> {
    let clock = workspace.resolve(CLOCK_TRAIT).map_err(|e| e.to_string())?;
    let wall = workspace.resolve(WALL_CLOCK).map_err(|e| e.to_string())?;
    let runner = workspace
        .resolve("raf_workplan::exec::Runner")
        .map_err(|e| e.to_string())?;
    let system = workspace
        .resolve("raf_workplan::exec::system::SystemRunner")
        .map_err(|e| e.to_string())?;
    let mut problems = Vec::new();
    let mut implementations = Vec::new();
    let mut selections = Vec::new();
    if clock.sites.len() != 1 || wall.sites.len() != 1 || system.sites.len() != 1 {
        problems.push(
            "Clock, WallClock and SystemRunner must each have one authoritative definition".into(),
        );
    }
    for block in workspace
        .impls()
        .filter(|block| block.test != raf_source::TestRole::CfgTest)
    {
        let Some(path) = &block.trait_path else {
            continue;
        };
        let resolved = workspace.resolve_from(&block.module, path);
        if block.self_items == system.sites
            && resolved
                .as_ref()
                .is_ok_and(|found| found.sites == runner.sites)
        {
            let site = (block.file.clone(), block.line);
            if !selections.contains(&site) {
                selections.push(site);
            }
            if block.file.as_str() != CLOCK_SYSTEM_FILE || path.as_str() != "Runner" {
                problems.push(format!("{}:{}: SystemRunner's production Runner implementation requires its admitted direct trait path", block.file, block.line));
            }
        }
        if resolved
            .as_ref()
            .is_ok_and(|found| found.sites == clock.sites)
        {
            let site = (block.file.clone(), block.line);
            if !implementations.contains(&site) {
                implementations.push(site);
            }
            if block.file.as_str() != CLOCK_CLOCK_FILE || block.self_items != wall.sites {
                problems.push(format!(
                    "{}:{}: production Clock implementation must be the authoritative WallClock",
                    block.file, block.line
                ));
            }
        } else if path.split("::").any(|name| name == "Clock") && resolved.is_err() {
            problems.push(format!(
                "{}:{}: cannot resolve clock trait {path}: {}",
                block.file,
                block.line,
                resolved.unwrap_err()
            ));
        }
    }
    if implementations.len() != 1 {
        problems.push(format!(
            "expected one production Clock implementation, found {}",
            implementations.len()
        ));
    }
    if selections.len() != 1 {
        problems.push(format!(
            "expected one production SystemRunner selection, found {}",
            selections.len()
        ));
    }
    for name in ["Clock", "WallClock"] {
        let expected = if name == "Clock" {
            &clock.sites
        } else {
            &wall.sites
        };
        if workspace
            .resolve_from("raf_workplan::exec::system", name)
            .map_err(|e| e.to_string())?
            .sites
            != *expected
        {
            problems.push(format!(
                "SystemRunner {name} import must name the authoritative clock"
            ));
        }
    }
    Ok(problems)
}

fn production_clock_problems(workspace: &raf_source::Workspace) -> Result<Vec<String>, String> {
    let mut problems = clock_implementation_problems(workspace)?;
    problems.extend(clock_macro_definitions(workspace));
    let mut clock_seen = false;
    let mut system_seen = false;
    for file in workspace.files().filter(|file| {
        !file.test
            && file.reached.iter().any(|reach| {
                reach.module_path == "raf_workplan"
                    || reach.module_path == "raf_workplan::exec"
                    || reach.module_path.starts_with("raf_workplan::exec::")
            })
    }) {
        let mut tests: Vec<_> = workspace
            .items_by_source()
            .filter(|(path, _, sites)| {
                **path == file.path
                    && sites.iter().all(|id| {
                        workspace
                            .item(*id)
                            .is_some_and(|item| item.test != raf_source::TestRole::None)
                    })
            })
            .map(|(_, _, sites)| {
                let span = &workspace
                    .item(sites[0])
                    .expect("native source group item")
                    .span;
                span.start_byte..span.end_byte
            })
            .collect();
        tests.extend(
            workspace
                .impls()
                .filter(|block| {
                    block.file == file.path
                        && block.test == raf_source::TestRole::CfgTest
                        && workspace
                            .impls()
                            .filter(|other| {
                                other.file == block.file
                                    && other.span.start_byte == block.span.start_byte
                            })
                            .all(|other| other.test == raf_source::TestRole::CfgTest)
                })
                .map(|block| block.span.start_byte..block.span.end_byte),
        );
        let blocks: Vec<_> = workspace
            .impls()
            .filter(|block| block.file == file.path)
            .map(|block| block.span.start_byte..block.span.end_byte)
            .collect();
        let path = file.path.as_str();
        clock_seen |= path == CLOCK_CLOCK_FILE;
        system_seen |= path == CLOCK_SYSTEM_FILE;
        let found = raf_source::parse_source(&file.path, &file.text, |syntax| {
            let mut admission = ClockAdmission {
                path,
                tests,
                blocks,
                problems: Vec::new(),
            };
            if path == CLOCK_CLOCK_FILE {
                admission.clock(syntax);
            }
            if path == CLOCK_SYSTEM_FILE {
                admission.runner(syntax);
            }
            admission.visit_file(syntax);
            admission.problems
        })
        .map_err(|e| e.to_string())?;
        problems.extend(found);
    }
    if !clock_seen || !system_seen {
        problems
            .push("authoritative production clock and runner source must both be loaded".into());
    }
    Ok(problems)
}

struct ClockAdmission<'a> {
    path: &'a str,
    tests: Vec<std::ops::Range<usize>>,
    blocks: Vec<std::ops::Range<usize>>,
    problems: Vec<String>,
}

fn source_path(path: &syn::Path, expected: &[&str]) -> bool {
    path.segments.len() == expected.len()
        && path.segments.iter().zip(expected).all(|(segment, name)| {
            segment.ident == *name && matches!(segment.arguments, syn::PathArguments::None)
        })
}

fn expression_path(expression: &syn::Expr, expected: &[&str]) -> bool {
    matches!(expression, syn::Expr::Path(path) if path.qself.is_none() && source_path(&path.path, expected))
}

fn returned_expression(block: &syn::Block) -> Option<&syn::Expr> {
    match block.stmts.as_slice() {
        [syn::Stmt::Expr(syn::Expr::Return(value), _)] => value.expr.as_deref(),
        [syn::Stmt::Expr(value, None)] => Some(value),
        _ => None,
    }
}

fn use_names(tree: &syn::UseTree, prefix: &str, names: &mut Vec<String>) -> bool {
    match tree {
        syn::UseTree::Path(path) => {
            use_names(&path.tree, &format!("{prefix}{}::", path.ident), names)
        }
        syn::UseTree::Name(name) => {
            names.push(format!("{prefix}{}", name.ident));
            true
        }
        syn::UseTree::Group(group) => group
            .items
            .iter()
            .all(|item| use_names(item, prefix, names)),
        syn::UseTree::Rename(_) | syn::UseTree::Glob(_) => false,
    }
}

fn clock_use(tree: &syn::UseTree) -> bool {
    let related = |name: &syn::Ident| {
        let name = name.unraw();
        ["Clock", "WallClock", "clock"]
            .iter()
            .any(|word| name == *word)
    };
    match tree {
        syn::UseTree::Path(path) => related(&path.ident) || clock_use(&path.tree),
        syn::UseTree::Name(name) => related(&name.ident),
        syn::UseTree::Rename(name) => related(&name.ident) || related(&name.rename),
        syn::UseTree::Group(group) => group.items.iter().any(clock_use),
        syn::UseTree::Glob(_) => false,
    }
}

impl ClockAdmission<'_> {
    fn test(&self, span: std::ops::Range<usize>) -> bool {
        self.tests
            .iter()
            .any(|range| range.start <= span.start && span.end <= range.end)
    }

    fn refuse(&mut self, line: usize, message: &str) {
        self.problems
            .push(format!("{}:{line}: {message}", self.path));
    }

    fn clock(&mut self, file: &syn::File) {
        let mut imports = Vec::new();
        let (mut state, mut start, mut elapsed) = (false, false, false);
        let (mut starts, mut readings) = (0, 0);
        for item in &file.items {
            if self.test(item.span().byte_range()) {
                continue;
            }
            let admitted = match item {
                syn::Item::Use(value) => use_names(&value.tree, "", &mut imports),
                syn::Item::Trait(value) => value.ident == "Clock",
                syn::Item::Struct(value) if value.ident == "WallClock" => {
                    state = matches!(&value.fields, syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 && matches!(&fields.unnamed[0].ty, syn::Type::Path(path) if path.qself.is_none() && source_path(&path.path, &["Instant"])));
                    state
                }
                syn::Item::Impl(value) if matches!(value.self_ty.as_ref(), syn::Type::Path(path) if source_path(&path.path, &["WallClock"])) =>
                {
                    for member in &value.items {
                        if let syn::ImplItem::Fn(method) = member {
                            if self.test(method.span().byte_range()) {
                                continue;
                            }
                            if value.trait_.is_none() && method.sig.ident == "start" {
                                starts += 1;
                                start = matches!(returned_expression(&method.block), Some(syn::Expr::Call(call)) if expression_path(&call.func, &["Self"]) && call.args.len() == 1 && matches!(&call.args[0], syn::Expr::Call(now) if expression_path(&now.func, &["Instant", "now"]) && now.args.is_empty()));
                            }
                            if value
                                .trait_
                                .as_ref()
                                .is_some_and(|(path, _)| source_path(path, &["Clock"]))
                                && method.sig.ident == "elapsed"
                            {
                                readings += 1;
                                elapsed = matches!(returned_expression(&method.block), Some(syn::Expr::MethodCall(call)) if call.method == "elapsed" && call.args.is_empty() && matches!(call.receiver.as_ref(), syn::Expr::Field(field) if expression_path(&field.base, &["self"]) && matches!(&field.member, syn::Member::Unnamed(index) if index.index == 0)));
                            }
                        }
                    }
                    true
                }
                _ => false,
            };
            if !admitted {
                self.refuse(
                    item.span().start().line,
                    "unsupported production item in the authoritative clock module",
                );
            }
        }
        imports.sort();
        if imports != ["std::thread", "std::time::Duration", "std::time::Instant"] {
            self.refuse(1, "clock imports must directly name standard thread, Duration and Instant without aliases");
        }
        if !state || !start || !elapsed || starts != 1 || readings != 1 {
            self.refuse(1, "WallClock must store standard Instant, start with Instant::now(), and return self.0.elapsed() directly");
        }
    }

    fn runner(&mut self, file: &syn::File) {
        let mut selected = false;
        let mut imported = false;
        let mut runs = 0;
        for item in &file.items {
            if self.test(item.span().byte_range()) {
                continue;
            }
            if let syn::Item::Use(value) = item {
                let mut names = Vec::new();
                if use_names(&value.tree, "", &mut names)
                    && names.iter().any(|name| name == "super::clock::Clock")
                {
                    names.sort();
                    imported = names == ["super::clock::Clock", "super::clock::WallClock"];
                }
            }
            if let syn::Item::Impl(value) = item
                && value
                    .trait_
                    .as_ref()
                    .is_some_and(|(path, _)| source_path(path, &["Runner"]))
                && matches!(value.self_ty.as_ref(), syn::Type::Path(path) if source_path(&path.path, &["SystemRunner"]))
            {
                for member in &value.items {
                    if let syn::ImplItem::Fn(method) = member
                        && method.sig.ident == "run"
                    {
                        if self.test(method.span().byte_range()) {
                            continue;
                        }
                        runs += 1;
                        selected = matches!(returned_expression(&method.block), Some(syn::Expr::MethodCall(call)) if expression_path(&call.receiver, &["self"]) && call.method == "run_with" && call.args.len() == 3 && expression_path(&call.args[0], &["invocation"]) && expression_path(&call.args[1], &["limits"]) && expression_path(&call.args[2], &["WallClock", "start"]));
                    }
                }
            }
        }
        if !selected || !imported || runs != 1 {
            self.refuse(
                1,
                "SystemRunner::run must select the canonical WallClock::start through run_with",
            );
        }
    }
}

impl<'ast> syn::visit::Visit<'ast> for ClockAdmission<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        if self.test(item.span().byte_range()) {
            return;
        }
        syn::visit::visit_item(self, item);
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if !self.blocks.contains(&item.span().byte_range()) {
            self.refuse(
                item.span().start().line,
                "block-local impl is outside the native production clock inventory",
            );
        }
        syn::visit::visit_item_impl(self, item);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        if reserved_clock_use(&item.tree) {
            self.refuse(item.span().start().line, "imports must not shadow admitted standard macros, derives or clock producer bindings");
        }
        if clock_use(&item.tree) {
            let mut names = Vec::new();
            if !use_names(&item.tree, "", &mut names) {
                self.refuse(
                    item.span().start().line,
                    "clock-related import aliases and globs are unsupported; use direct names",
                );
            }
        }
    }

    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        let validated = self.path == CLOCK_PATTERN_FILE
            && ((item.ident.as_ref().is_some_and(|name| name == "validated")
                && source_path(&item.mac.path, &["macro_rules"])
                && validated_definition(&item.mac))
                || (item.ident.is_none()
                    && source_path(&item.mac.path, &["validated"])
                    && validated_call(&item.mac)));
        if !validated {
            self.refuse(
                item.span().start().line,
                "unsupported production item generation in the clock's execution scope",
            );
        }
    }

    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        self.refuse(
            item.span().start().line,
            "extern crate imports are outside the admitted clock macro provenance",
        );
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if item.ident.unraw() == "std" || item.ident.unraw() == "fmt" {
            self.refuse(
                item.span().start().line,
                "modules must not shadow standard clock or producer namespaces",
            );
        }
        syn::visit::visit_item_mod(self, item);
    }

    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        if ![
            "doc",
            "cfg",
            "allow",
            "must_use",
            "derive",
            "deprecated",
            "inline",
            "cold",
            "track_caller",
            "non_exhaustive",
        ]
        .iter()
        .any(|name| source_path(attribute.path(), &[name]))
        {
            self.refuse(
                attribute.span().start().line,
                "unsupported production attribute generation in the clock's execution scope",
            );
        }
        if source_path(attribute.path(), &["derive"]) {
            let admitted =
                matches!(&attribute.meta, syn::Meta::List(list) if standard_derives(list));
            if !admitted {
                self.refuse(
                    attribute.span().start().line,
                    "only standard derives are admitted in the clock's execution scope",
                );
            }
        }
    }

    fn visit_macro(&mut self, value: &'ast syn::Macro) {
        if !CLOCK_MACROS
            .iter()
            .any(|name| source_path(&value.path, &[name]))
            || !clock_macro_tokens(syn::buffer::TokenBuffer::new2(value.tokens.clone()).begin())
        {
            self.refuse(
                value.path.span().start().line,
                "unsupported production macro or item-bearing token input in the clock's execution scope",
            );
        }
    }
}

/// The rule that judges `path`, when it is a Rust file under the root `tests/`
/// or under `crates/<name>/{src,tests,examples}`, and not an exempt file. The
/// crate directories come from the path, so every crate is judged.
fn clock_rule(path: &Path) -> Option<&'static ClockRule> {
    if path.extension().is_none_or(|ext| ext != "rs")
        || CLOCK_EXEMPT.iter().any(|exempt| path == Path::new(exempt))
    {
        return None;
    }
    if path.starts_with(ROOT_TESTS_DIR) {
        return Some(&CRATE_RULE);
    }
    let mut parts = path.iter();
    if parts.next()? != CRATES_DIR {
        return None;
    }
    let name = parts.next()?;
    let dir = parts.next()?;
    // A file directly in `crates/<name>` names no directory.
    parts.next()?;
    if !CRATE_DIRS.iter().any(|ruled| dir == *ruled) {
        return None;
    }
    if name == SOURCE_CRATE {
        Some(&SOURCE_RULE)
    } else {
        Some(&CRATE_RULE)
    }
}

/// True when `word` occurs in `line` with no identifier character before it,
/// and after it either none or an `_` that begins a suffix: `sleep_ms`,
/// `sleep_until`, `wait_timeout_while` and `park_timeout_ms` count as `sleep`,
/// `wait_timeout` and `park_timeout`, while `sleeping` and `Instantiate` do not
/// count as `sleep` and `Instant`.
fn has_word(line: &str, word: &str) -> bool {
    let identifier = |c: char| c.is_alphanumeric() || c == '_';
    line.match_indices(word).any(|(start, _)| {
        let before = line[..start].chars().next_back();
        let after = line[start + word.len()..].chars().next();
        !before.is_some_and(identifier) && !after.is_some_and(|c| identifier(c) && c != '_')
    })
}

/// One `path:line: word` entry per clock spelling in `text`, when a rule
/// judges `path`; nothing for any other path.
fn clock_hits(path: &Path, text: &str) -> Vec<String> {
    let Some(rule) = clock_rule(path) else {
        return Vec::new();
    };
    let mut hits = Vec::new();
    for (index, line) in text.lines().enumerate() {
        for word in rule.words {
            if has_word(line, word) {
                hits.push(format!("{}:{}: {word}", path.display(), index + 1));
            }
        }
    }
    hits
}

/// The `version` under `[workspace.package]` in a `Cargo.toml` text.
fn workspace_version(manifest: &str) -> Option<String> {
    let mut in_section = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_section = line == "[workspace.package]";
        } else if in_section && let Some(rest) = line.strip_prefix("version") {
            let value = rest.trim_start().strip_prefix('=')?.trim();
            return Some(value.trim_matches('"').to_string());
        }
    }
    None
}

/// The top-level `version:` value in a `CITATION.cff` text.
fn citation_version(citation: &str) -> Option<String> {
    let value = citation.lines().find_map(|l| l.strip_prefix("version:"))?;
    Some(value.trim().trim_matches(['"', '\'']).to_string())
}

/// A problem with version agreement, or `None` when the two agree.
fn version_problem(manifest: &str, citation: &str) -> Option<String> {
    match (workspace_version(manifest), citation_version(citation)) {
        (None, _) => Some("Cargo.toml: no version under [workspace.package]".to_string()),
        (_, None) => Some("CITATION.cff: no top-level version".to_string()),
        (Some(cargo), Some(cff)) if cargo != cff => Some(format!(
            "CITATION.cff: version {cff} != [workspace.package] version {cargo} in Cargo.toml"
        )),
        _ => None,
    }
}

/// The lines of every table whose header starts with `prefix`, headers
/// included, with blank lines and comments dropped.
fn tables_with_prefix(toml: &str, prefix: &str) -> Vec<String> {
    let mut in_section = false;
    let mut lines = Vec::new();
    for line in toml.lines().map(str::trim) {
        if line.starts_with('[') {
            in_section = line.starts_with(prefix);
        }
        if in_section && !line.is_empty() && !line.starts_with('#') {
            lines.push(line.to_string());
        }
    }
    lines
}

/// A problem with the lint tables of the fuzz workspace, or `None` when they
/// equal the root tables.
fn lint_tables_problem(root: &str, fuzz: &str) -> Option<String> {
    let want = tables_with_prefix(root, "[workspace.lints");
    let got = tables_with_prefix(fuzz, "[workspace.lints");
    if want.is_empty() {
        Some("Cargo.toml: no [workspace.lints] tables".to_string())
    } else if want != got {
        Some(format!(
            "fuzz/Cargo.toml: [workspace.lints] tables differ from Cargo.toml: root has {want:?}, fuzz has {got:?}"
        ))
    } else {
        None
    }
}

/// The level of lint `name` in the `[workspace.lints.rust]` table, as written
/// between its quotes, or `None` when the table does not set it as a string.
fn rust_lint_level(manifest: &str, name: &str) -> Option<String> {
    let mut in_table = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_table = line == "[workspace.lints.rust]";
        } else if in_table {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if key.trim() == name {
                return value
                    .trim()
                    .strip_prefix('"')
                    .and_then(|rest| rest.strip_suffix('"'))
                    .map(str::to_string);
            }
        }
    }
    None
}

/// The quoted entries of the `members` array of a workspace manifest.
fn workspace_members(manifest: &str) -> Vec<String> {
    let mut in_workspace = false;
    let mut collecting = false;
    let mut members = Vec::new();
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_workspace = line == "[workspace]";
        }
        if in_workspace && line.starts_with("members") {
            collecting = true;
        }
        if collecting {
            members.extend(line.split('"').skip(1).step_by(2).map(str::to_string));
            if line.contains(']') {
                collecting = false;
            }
        }
    }
    members
}

/// True when the manifest has a `[lints]` table holding `workspace = true`.
fn inherits_workspace_lints(manifest: &str) -> bool {
    let mut in_lints = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_lints = line == "[lints]";
        } else if in_lints && line.replace(' ', "") == "workspace=true" {
            return true;
        }
    }
    false
}

/// The manifests in `names` that do not inherit the workspace lints.
fn without_workspace_lints(names: &[String], read: impl Fn(&str) -> String) -> Vec<String> {
    names
        .iter()
        .filter(|name| !inherits_workspace_lints(&read(name)))
        .cloned()
        .collect()
}

/// The tables of a `deny.toml` as lines, comments and blank lines dropped,
/// split at each header.
fn deny_tables(deny: &str) -> Vec<Vec<String>> {
    let mut tables: Vec<Vec<String>> = Vec::new();
    for line in deny.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            tables.push(Vec::new());
        }
        if let Some(table) = tables.last_mut() {
            table.push(line.to_string());
        }
    }
    tables
}

/// The crate-scoped exception the fuzz configuration adds to the root one.
const FUZZ_EXCEPTION: [&str; 3] = [
    "[[licenses.exceptions]]",
    "crate = \"libfuzzer-sys@=0.4.13\"",
    "allow = [\"NCSA\"]",
];

/// A problem with the fuzz `deny.toml`, or `None` when it is the root
/// configuration plus exactly the crate-scoped exception.
fn deny_problem(root: &str, fuzz: &str) -> Option<String> {
    let want = deny_tables(root);
    let (extra, shared): (Vec<_>, Vec<_>) = deny_tables(fuzz)
        .into_iter()
        .partition(|table| table.iter().map(String::as_str).eq(FUZZ_EXCEPTION));
    if extra.len() != 1 {
        Some(format!(
            "fuzz/deny.toml: expected exactly one exception for libfuzzer-sys, found {}",
            extra.len()
        ))
    } else if shared != want {
        Some("fuzz/deny.toml: the shared sections differ from deny.toml".to_string())
    } else {
        None
    }
}

fn read_text(root: &Path, name: &str) -> String {
    let bytes = read_tracked(root, Path::new(name));
    decode_text(&bytes)
        .unwrap_or_else(|| panic!("{name} is not UTF-8 text"))
        .to_string()
}

// ---------------------------------------------------------------------------
// Gates on the real tree
// ---------------------------------------------------------------------------

#[test]
fn licenses_mulan_texts_match_the_pinned_digest() {
    let root = gate_root();
    let problems: Vec<String> = MULAN_FILES
        .iter()
        .filter_map(|name| {
            digest_problem(name, &read_tracked(&root, Path::new(name)), MULAN_SHA256)
        })
        .collect();
    assert!(
        problems.is_empty(),
        "licenses:\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn licenses_zh_hans_matches_the_recorded_fingerprint() {
    let root = gate_root();
    let record = read_text(&root, BACKTRANSLATION);
    let document = read_tracked(&root, Path::new(ZH_HANS));
    if let Some(problem) = fingerprint_problem(&record, &document) {
        panic!("licenses: {problem}");
    }
}

#[test]
fn issue_refs_tracked_files_are_clean() {
    let root = gate_root();
    let paths = tracked_paths();
    let (hits, scanned) = scan_tree(&root, &paths);
    assert!(scanned > 0, "no tracked text file was scanned");
    assert!(
        hits.is_empty(),
        "issue-refs: {} hits; remove issue/PR tokens and process wording:\n  {}",
        hits.len(),
        hits.join("\n  ")
    );
}

#[test]
fn time_in_crates_tracked_files_name_no_clock() {
    let root = gate_root();
    let tracked = tracked_paths();
    for exempt in CLOCK_EXEMPT {
        assert!(
            tracked.iter().any(|path| path == Path::new(exempt)),
            "time-in-crates: the exempt file {exempt} is not tracked; drop the exemption"
        );
    }
    let workspace = raf_source::Workspace::load(
        &raf_source::DiskTree::new(&root).expect("gate source tree"),
        &["."],
    )
    .expect("load the source judged by the clock gate");
    let problems =
        production_clock_problems(&workspace).expect("resolve and parse production clock sources");
    assert!(
        problems.is_empty(),
        "production clock contract:\n  {}",
        problems.join("\n  ")
    );
    let gate = read_tracked(&root, Path::new(CLOCK_GATE_FILE));
    let gate = decode_text(&gate).expect("the gate file is text");
    assert!(
        gate.contains(&format!("const {}_CLOCK_WORDS", "CRATE")),
        "time-in-crates: {CLOCK_GATE_FILE} no longer defines the word tables; drop the exemption"
    );
    for path in &tracked {
        let in_crate_source = path.starts_with(CRATES_DIR)
            && path.iter().nth(2).is_some_and(|dir| dir == "src")
            && path.extension().is_some_and(|ext| ext == "rs");
        assert!(
            !in_crate_source || clock_rule(path).is_some() || path == Path::new(CLOCK_CLOCK_FILE),
            "time-in-crates: no rule judges {}",
            path.display()
        );
    }
    let mut hits = Vec::new();
    let mut scanned = 0;
    for path in tracked {
        if clock_rule(&path).is_some() {
            scanned += 1;
            let bytes = read_tracked(&root, &path);
            if let Some(text) = decode_text(&bytes) {
                hits.extend(clock_hits(&path, &text));
            }
        }
    }
    assert!(scanned > 0, "no ruled file was scanned");
    assert!(
        hits.is_empty(),
        "time-in-crates: {} hits; count work or script a clock instead of reading one:\n  {}",
        hits.len(),
        hits.join("\n  ")
    );
}

#[test]
fn citation_version_matches_the_workspace() {
    let root = gate_root();
    let problem = version_problem(
        &read_text(&root, "Cargo.toml"),
        &read_text(&root, "CITATION.cff"),
    );
    if let Some(problem) = problem {
        panic!("citation: {problem}");
    }
}

// ---------------------------------------------------------------------------
// Fixtures: SHA-256 and the licence gate
// ---------------------------------------------------------------------------

#[test]
fn licenses_sha256_matches_known_vectors() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    // Two blocks, so the padding path that spills into a second block runs.
    assert_eq!(
        sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
    // Exactly one block of data forces a block holding only padding.
    assert_eq!(
        sha256_hex(&[b'a'; 64]),
        "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb"
    );
}

#[test]
fn licenses_rejects_an_altered_mulan_text() {
    let altered = digest_problem("LICENSE-MULAN", b"not the licence", MULAN_SHA256);
    assert!(altered.is_some_and(|p| p.starts_with("LICENSE-MULAN: sha256 ")));
    assert!(digest_problem("LICENSE-MULAN", b"", MULAN_SHA256).is_some());
}

#[test]
fn licenses_accepts_bytes_that_hash_to_the_pin() {
    let pin = sha256_hex(b"the text");
    assert!(digest_problem("x", b"the text", &pin).is_none());
    assert!(digest_problem("x", b"the text.", &pin).is_some());
    assert_eq!(MULAN_SHA256.len(), 64);
}

#[test]
fn licenses_rejects_a_fingerprint_mismatch() {
    let record = format!("- {FINGERPRINT_LABEL}: `{}`.", sha256_hex(b"other"));
    assert!(fingerprint_problem(&record, b"changed text").is_some());
}

#[test]
fn licenses_accepts_a_matching_fingerprint() {
    let record = format!("- {FINGERPRINT_LABEL}: `{}`.", sha256_hex(b"the text"));
    assert!(fingerprint_problem(&record, b"the text").is_none());
}

#[test]
fn licenses_require_a_canonical_fingerprint_with_non_hex_boundaries() {
    let digest = sha256_hex(b"the text");
    let record = |text: &str| format!("- {FINGERPRINT_LABEL}: {text}");
    for invalid in [
        digest.to_uppercase(),
        format!("a{digest}"),
        format!("A{digest}"),
        format!("{digest}f"),
        format!("{digest}F"),
        digest[..63].to_owned(),
    ] {
        assert_eq!(recorded_fingerprint(&record(&invalid)), None, "{invalid}");
    }
    for valid in [
        format!("`{digest}`"),
        format!("g{digest}G"),
        format!("é{digest}!"),
    ] {
        assert_eq!(recorded_fingerprint(&record(&valid)), Some(digest.clone()));
    }
}

#[test]
fn licenses_rejects_a_record_without_a_fingerprint_line() {
    let record = format!("- {FINGERPRINT_LABEL}: none recorded.\n- other: `abc`.");
    assert!(fingerprint_problem(&record, b"text").is_some());
    assert!(fingerprint_problem("no label here", b"text").is_some());
}

// ---------------------------------------------------------------------------
// Fixtures: the issue-reference and process-wording gate
// ---------------------------------------------------------------------------

/// Fixtures spell `#` as `@` so the test source does not trip its own gate.
fn hashed(fixture: &str) -> String {
    fixture.replace('@', "#")
}

fn assert_banned(line: &str, kind: &str) {
    assert_eq!(banned_in_line(line), Some(kind), "expected a hit: {line:?}");
}

fn assert_clean(line: &str) {
    assert_eq!(banned_in_line(line), None, "expected clean: {line:?}");
}

const NUMBER: &str = "issue or pull request number";
const PROCESS: &str = "development-process wording";

#[test]
fn issue_refs_rejects_a_hash_number_after_a_space() {
    assert_banned(&hashed("see @123"), NUMBER);
    assert_banned(&hashed("Per @12"), NUMBER);
    assert_banned(&hashed("@7 first"), NUMBER);
}

#[test]
fn issue_refs_rejects_a_hash_number_after_a_word_character() {
    assert_banned(&hashed("org/repo@12"), NUMBER);
    assert_banned(&hashed("repo@12"), NUMBER);
    assert_banned(&hashed("Fixes@12"), NUMBER);
    assert_banned(&hashed("x@1"), NUMBER);
    assert_banned(
        &hashed("see Blackcat-Informatics/rust-agentic-foundations@12 for it"),
        NUMBER,
    );
}

#[test]
fn issue_refs_rejects_a_parenthesised_hash_number() {
    assert_banned(&hashed("(@1234)"), NUMBER);
    assert_banned(&hashed("fixed in (@12345)."), NUMBER);
}

#[test]
fn issue_refs_spares_a_quoted_three_digit_colour() {
    assert_clean(&hashed(r#"fill="@000""#));
}

#[test]
fn issue_refs_spares_a_hex_colour_with_letters() {
    assert_clean(&hashed("stroke='@abc'"));
    assert_clean(&hashed("color: @1f2"));
    assert_clean(&hashed("fill @111214 and @fbbc05"));
}

#[test]
fn issue_refs_spares_an_xml_character_reference() {
    assert_clean(&hashed("&@38;"));
}

#[test]
fn issue_refs_spares_a_six_digit_colour() {
    assert_clean(&hashed("@123456"));
}

#[test]
fn issue_refs_rejects_a_colon_prefixed_number() {
    // Only a quoted three- or four-digit token is a colour; after a colon an
    // all-digit token is a reference.
    assert_banned(&hashed("id:@12"), NUMBER);
    assert_banned(&hashed("id:@12345"), NUMBER);
    assert_banned(&hashed("Fixes:@123"), NUMBER);
    assert_banned(&hashed("id:@1234"), NUMBER);
}

#[test]
fn issue_refs_rejects_an_all_digit_css_colour() {
    // An all-digit CSS colour fails closed; write it with letters or as
    // `@112233`.
    assert_banned(&hashed("color: @123"), NUMBER);
}

#[test]
fn issue_refs_rejects_an_issue_url() {
    assert_banned(concat!("https://example.invalid/", "issues/", "7"), NUMBER);
}

#[test]
fn issue_refs_rejects_a_pull_url() {
    assert_banned(concat!("https://example.invalid/", "pull/", "7"), NUMBER);
}

#[test]
fn issue_refs_rejects_a_pull_request_label() {
    assert_banned(concat!("see PR", " ", "9"), NUMBER);
    assert_banned(concat!("see PR", "9"), NUMBER);
    assert_clean("the PRESS release");
    // The label is case-sensitive, as the original search was: a lowercase
    // `pr` followed by a digit is common in identifiers and must not hit.
    assert_clean(concat!("pr", " 12"));
    assert_clean(concat!("pr", "12 = 3"));
}

#[test]
fn issue_refs_rejects_a_change_number() {
    assert_banned(concat!("CHANGE", "-", "17"), NUMBER);
    assert_clean("CHANGE-me");
}

#[test]
fn issue_refs_rejects_the_word_issue_with_a_number() {
    assert_banned(concat!("Closes issue", " 12"), NUMBER);
    assert_banned(concat!("Issue", " 3 is open"), NUMBER);
    assert_clean(concat!("reissue", " 12 copies"));
}

#[test]
fn issue_refs_rejects_a_github_style_number() {
    assert_banned(concat!("see GH", "-12"), NUMBER);
    assert_clean(concat!("HIGH", "-12 grade"));
}

#[test]
fn issue_refs_rejects_the_words_pull_request_with_a_number() {
    assert_banned(concat!("merged in pull", " request 11"), NUMBER);
    assert_banned(concat!("Pull", " Request 11"), NUMBER);
}

#[test]
fn issue_refs_spares_near_miss_prose() {
    assert_clean("an issue-reference is a token");
    assert_clean("the issue tracker is external");
    assert_clean("pull request templates live in .github");
    assert_clean("issue numbers are not recorded here");
}

#[test]
fn issue_refs_rejects_this_issue() {
    assert_banned(concat!("As noted in this", " issue"), PROCESS);
}

#[test]
fn issue_refs_rejects_a_plan_possessive() {
    assert_banned(concat!("the plan", "'s step"), PROCESS);
    assert_clean("the planet");
}

#[test]
fn issue_refs_rejects_per_the_plan() {
    assert_banned(concat!("Per the", " plan we skip it"), PROCESS);
}

#[test]
fn issue_refs_rejects_this_branch() {
    assert_banned(concat!("As agreed in this", " branch"), PROCESS);
}

#[test]
fn issue_refs_rejects_a_task_number() {
    assert_banned(concat!("Task", " 4 is done"), PROCESS);
    assert_clean("the task list");
}

#[test]
fn issue_refs_rejects_an_acceptance_label() {
    assert_banned(concat!("Criterion AC", "3 is met"), PROCESS);
    assert_clean("the SPAC3 board and AC/DC");
}

#[test]
fn issue_refs_reports_every_hit_as_file_and_line() {
    let text = hashed("clean line\nsee @12\nalso clean\n");
    let hits = scan_text(Path::new("docs/a.md"), &text);
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(hits[0], format!("docs/a.md:2: {NUMBER}"));
    let both = scan_text(
        Path::new("b.txt"),
        &format!("{}\n{}\n", hashed("x @9"), concat!("Task", " 1")),
    );
    assert_eq!(
        both,
        vec![format!("b.txt:1: {NUMBER}"), format!("b.txt:2: {PROCESS}")]
    );
}

#[test]
fn issue_refs_treats_only_a_nul_byte_as_binary() {
    assert!(decode_text(b"plain text").is_some());
    assert!(decode_text(b"\x89PNG\r\n\x1a\n\x00\x00\x00").is_none());
    assert!(decode_text(b"text\0text").is_none());
}

#[test]
fn issue_refs_flags_a_token_in_a_latin_1_file() {
    // Invalid UTF-8 is not binary: a Latin-1 file is still scanned.
    let mut bytes = b"caf\xe9 ".to_vec();
    bytes.extend_from_slice(hashed("see @12\n").as_bytes());
    assert!(std::str::from_utf8(&bytes).is_err());
    let text = decode_text(&bytes).expect("a file without NUL is text");
    assert_eq!(
        scan_text(Path::new("latin1.txt"), &text),
        vec![format!("latin1.txt:1: {NUMBER}")]
    );
}

#[test]
fn issue_refs_skips_a_nul_containing_file_holding_a_token() {
    let bytes = hashed("see @12\0\n").into_bytes();
    assert!(decode_text(&bytes).is_none());
}

#[test]
fn issue_refs_scans_text_files_and_skips_binary_ones() {
    let scratch =
        purrdf_testkit::TempDir::new_in(std::env::temp_dir()).expect("create scratch directory");
    let dir = scratch.path();
    let bad = hashed("see @12\n");
    std::fs::write(dir.join("text.md"), bad).expect("write text fixture");
    std::fs::write(dir.join("image.bin"), [0x89, b'P', 0, b'@', b'1', b'2']).expect("write binary");
    let paths = [PathBuf::from("text.md"), PathBuf::from("image.bin")];
    let (hits, scanned) = scan_tree(dir, &paths);
    assert_eq!(scanned, 1);
    assert_eq!(hits, vec![format!("text.md:1: {NUMBER}")]);
}

// ---------------------------------------------------------------------------
// Fixtures: clocks in the crates
// ---------------------------------------------------------------------------

/// A planted reading of the clock, assembled so that this file stays clean.
fn planted() -> String {
    format!("let started = std::{}::{}::now();\n", "time", "Instant")
}

/// A planted reading that both word tables refuse.
fn planted_instant() -> String {
    format!("let started = {}::now();\n", "Instant")
}

/// Handwritten parsed-source data, never a rewrite of production code.
fn clock_fixture(start: &str, elapsed: &str, run: &str, extra: &str) -> raf_source::Workspace {
    clock_fixture_sources(start, elapsed, run, extra, "pub mod exec;", "")
}

fn clock_fixture_sources(
    start: &str,
    elapsed: &str,
    run: &str,
    extra: &str,
    root: &str,
    producer: &str,
) -> raf_source::Workspace {
    let clock = format!(
        r"
use std::thread;
use std::time::{{Duration, Instant}};
pub(super) trait Clock {{ fn elapsed(&self) -> Duration; fn pause(&mut self, pause: Duration); }}
pub(super) struct WallClock(Instant);
impl WallClock {{ pub(super) fn start() -> Self {{ {start} }} }}
impl Clock for WallClock {{
    fn elapsed(&self) -> Duration {{ {elapsed} }}
    fn pause(&mut self, pause: Duration) {{ thread::sleep(pause); }}
}}
"
    );
    let system = format!(
        r"
use super::clock::{{Clock, WallClock}};
use super::Runner;
struct SystemRunner;
impl Runner for SystemRunner {{ fn run(&self, invocation: (), limits: ()) {{ {run} }} }}
{extra}
"
    );
    let entries: [(&str, String); 8] = [
        (
            "Cargo.toml",
            "[workspace]\nmembers=['crates/raf-workplan']\nresolver='3'\n".into(),
        ),
        (
            "crates/raf-workplan/Cargo.toml",
            "[package]\nname='raf-workplan'\nversion='0.1.0'\nedition='2024'\n".into(),
        ),
        ("crates/raf-workplan/src/lib.rs", root.into()),
        (
            "crates/raf-workplan/src/exec.rs",
            "mod clock; mod system; mod pattern; pub trait Runner {} #[cfg(all(test, unix))] mod scripted;"
                .into(),
        ),
        (CLOCK_CLOCK_FILE, clock),
        (CLOCK_SYSTEM_FILE, system),
        (CLOCK_PATTERN_FILE, producer.into()),
        (
            "crates/raf-workplan/src/exec/scripted.rs",
            "use super::clock::Clock; struct Scripted; impl Clock for Scripted {}".into(),
        ),
    ];
    let tree = raf_source::MemoryTree::try_from_iter(entries).expect("fixture paths are valid");
    raf_source::Workspace::load(&tree, &["."]).expect("the handwritten clock fixture parses")
}

fn clock_producer_fixture(generation: &str, attributes: &str) -> String {
    format!(
        r"
use std::fmt;
macro_rules! validated {{
    ($(#[$meta:meta])* $name:ident, $pattern:expr) => {{
        $(#[$meta])*
        /// A doc comment mentioning impl Clock remains data.
        struct $name(String);
        impl $name {{
            /// Another doc comment.
            fn value(&self) -> &str {{ &self.0 }}
        }}
        {generation}
    }};
}}
validated!({attributes} Sample, Pattern::Name);
"
    )
}

fn production_clock_fixtures() {
    const START: &str = "Self(Instant::now())";
    const ELAPSED: &str = "self.0.elapsed()";
    const RUN: &str = "self.run_with(invocation, limits, WallClock::start)";
    for (start, elapsed, run, extra, cause) in [
        (START, "Duration::ZERO", RUN, "", "return self.0.elapsed()"),
        (
            "Self(other())",
            ELAPSED,
            RUN,
            "",
            "start with Instant::now()",
        ),
        (
            START,
            ELAPSED,
            "self.run_with(invocation, limits, Other::start)",
            "",
            "SystemRunner::run",
        ),
        (
            START,
            ELAPSED,
            RUN,
            "struct Other; impl Clock for Other {}",
            "production Clock implementation",
        ),
        (
            START,
            ELAPSED,
            RUN,
            "use super::clock::Clock as Renamed; struct Other; impl Renamed for Other {}",
            "production Clock implementation",
        ),
        (
            START,
            ELAPSED,
            RUN,
            "struct Other; #[cfg(any(test, feature=\"alternate\"))] impl Clock for Other {}",
            "production Clock implementation",
        ),
        (
            START,
            ELAPSED,
            RUN,
            "macro_rules! generated { () => { impl Clock for Other {} }; } generated!();",
            "production item generation",
        ),
        (
            START,
            ELAPSED,
            RUN,
            "struct Other; impl Other {} fn local() { impl Clock for Other {} }",
            "block-local impl",
        ),
        (
            START,
            ELAPSED,
            RUN,
            "fn hidden() { let _ = format!(\"{}\", { struct Other; impl Clock for Other { fn elapsed(&self) -> std::time::Duration { std::time::Duration::ZERO } fn pause(&mut self, _: std::time::Duration) {} } 0 }); }",
            "item-bearing token input",
        ),
        (
            START,
            ELAPSED,
            RUN,
            "use custom::Injected as r#Debug; #[derive(Debug)] struct Derived;",
            "must not shadow",
        ),
    ] {
        let workspace = clock_fixture(start, elapsed, run, extra);
        let problems = production_clock_problems(&workspace).expect("fixture identities resolve");
        assert!(
            problems.iter().any(|problem| problem.contains(cause)),
            "expected {cause}: {problems:?}"
        );
    }
    let valid = clock_fixture(
        "/* monotonic constructor */ Self ( Instant :: now () )",
        "return self . 0 . elapsed ();",
        RUN,
        "mod unrelated { pub trait Clock {} } struct Other; impl unrelated::Clock for Other {} #[cfg(all(test, unix))] mod scripts { struct Other; impl super::Clock for Other {} } fn literal() { let _ = format!(/* impl Clock is a comment */ \"impl Clock\"); }",
    );
    assert_eq!(
        production_clock_problems(&valid).unwrap(),
        Vec::<String>::new()
    );
    producer_clock_fixtures();
}

fn producer_clock_fixtures() {
    const START: &str = "Self(Instant::now())";
    const ELAPSED: &str = "self.0.elapsed()";
    const RUN: &str = "self.run_with(invocation, limits, WallClock::start)";
    for (root, generation, attributes, cause) in [
        (
            "macro_rules! format { ($($input:tt)*) => { 0 }; } pub mod exec;",
            "",
            "",
            "must not shadow",
        ),
        (
            "pub mod exec;",
            "",
            "#[derive(Injected)]",
            "production item generation",
        ),
        (
            "pub mod exec;",
            "opaque!();",
            "",
            "production item generation",
        ),
    ] {
        let workspace = clock_fixture_sources(
            START,
            ELAPSED,
            RUN,
            "",
            root,
            &clock_producer_fixture(generation, attributes),
        );
        let problems = production_clock_problems(&workspace).expect("producer fixture resolves");
        assert!(
            problems.iter().any(|problem| problem.contains(cause)),
            "expected {cause}: {problems:?}"
        );
    }
    let valid = clock_fixture_sources(
        START,
        ELAPSED,
        RUN,
        "",
        "pub mod exec;",
        &clock_producer_fixture(
            "/* impl Clock is only a comment */",
            "/// Forwarded documentation mentioning impl Clock.\n",
        ),
    );
    assert_eq!(
        production_clock_problems(&valid).unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn time_in_crates_rejects_a_planted_clock_reading_with_file_and_line() {
    production_clock_fixtures();
    let text = format!("fn clean() {{}}\n{}", planted());
    let hits = clock_hits(Path::new("crates/raf-source/tests/a.rs"), &text);
    assert_eq!(
        hits,
        vec![
            "crates/raf-source/tests/a.rs:2: std::time".to_owned(),
            "crates/raf-source/tests/a.rs:2: Instant".to_owned(),
        ]
    );
}

/// The directories the crate rule judges: each of [`CRATE_DIRS`] of the
/// crates that are tracked today but `raf-source`, of a crate that does not
/// exist yet, and the root `tests/`.
fn crate_rule_dirs() -> Vec<String> {
    let mut dirs = Vec::new();
    for name in [
        "raf-okf",
        "raf-workplan",
        "cargo-workplan",
        "raf-not-yet-written",
    ] {
        for dir in CRATE_DIRS {
            dirs.push(format!("{CRATES_DIR}/{name}/{dir}"));
        }
    }
    dirs.push(ROOT_TESTS_DIR.to_owned());
    dirs
}

/// The directories of `raf-source`, each judged by [`SOURCE_RULE`].
const SOURCE_RULE_DIRS: [&str; 3] = [
    "crates/raf-source/src",
    "crates/raf-source/tests",
    "crates/raf-source/examples",
];

#[test]
fn time_in_crates_rejects_a_planted_reading_in_every_ruled_directory() {
    let text = format!("fn clean() {{}}\n{}", planted_instant());
    let source = SOURCE_RULE_DIRS.map(str::to_owned);
    for dir in crate_rule_dirs().iter().chain(&source) {
        let path = format!("{dir}/nested/x.rs");
        assert_eq!(
            clock_hits(Path::new(&path), &text),
            vec![format!("{path}:2: Instant")],
            "{path}"
        );
    }
}

#[test]
fn time_in_crates_judges_a_crate_added_under_the_crates_directory() {
    let hits = |path: &str, text: &str| clock_hits(Path::new(path), text);
    for dir in CRATE_DIRS {
        let path = format!("crates/raf-census/{dir}/a.rs");
        assert_eq!(
            hits(&path, "let t = SystemTime::now();\n"),
            vec![format!("{path}:1: SystemTime")],
            "{path}"
        );
        // The new crate follows the crate rule, not the stricter one.
        assert_eq!(
            hits(&path, "let limit = Duration::from_secs(5);\n"),
            Vec::<String>::new()
        );
    }
    for path in [
        "crates/raf-census/benches/a.rs",
        "crates/raf-census/build.rs",
        "crates/raf-census/src/notes.md",
    ] {
        assert_eq!(
            hits(path, &planted_instant()),
            Vec::<String>::new(),
            "{path}"
        );
    }
}

#[test]
fn time_in_crates_rejects_each_source_spelling_in_the_library_and_the_tests() {
    for (word, line) in [
        ("Duration", "let d = Duration::from_secs(1);"),
        ("SystemTime", "let t = SystemTime::now();"),
        ("sleep", "std::thread::sleep(pause);"),
        ("recv_timeout", "receiver.recv_timeout(limit)"),
        ("core::time", "use core::time::Duration as D;"),
        ("tokio::time", "tokio::time::sleep(pause).await;"),
        ("wait_timeout", "condvar.wait_timeout(guard, limit)"),
        ("park_timeout", "std::thread::park_timeout(limit);"),
        ("set_read_timeout", "socket.set_read_timeout(limit)"),
        ("set_write_timeout", "socket.set_write_timeout(limit)"),
        ("elapsed", "let took = started.elapsed();"),
        ("sleep", "std::thread::sleep_ms(5);"),
        (
            "wait_timeout",
            "condvar.wait_timeout_while(guard, limit, f)",
        ),
        ("elapsed", "let took = elapsed_ms;"),
    ] {
        for dir in SOURCE_RULE_DIRS {
            let path = format!("{dir}/nested/x.rs");
            let hits = clock_hits(Path::new(&path), line);
            // A line can carry two spellings (`core::time::Duration`): the
            // spelling under test is reported among the hits, at line 1.
            assert!(
                hits.contains(&format!("{path}:1: {word}")),
                "{line}: {hits:?}"
            );
        }
    }
}

#[test]
fn time_in_crates_rejects_each_crate_spelling_in_every_other_directory() {
    for (word, line) in [
        ("Instant", "let t = Instant::now();"),
        ("SystemTime", "let t = SystemTime::now();"),
        ("UNIX_EPOCH", "let t = UNIX_EPOCH;"),
        ("tokio::time", "tokio::time::pause();"),
        ("core::time", "use core::time::Span;"),
        ("sleep", "std::thread::sleep(pause);"),
        ("recv_timeout", "receiver.recv_timeout(limit)"),
        ("wait_timeout", "condvar.wait_timeout(guard, limit)"),
        ("park_timeout", "std::thread::park_timeout(limit);"),
        ("set_read_timeout", "socket.set_read_timeout(limit)"),
        ("set_write_timeout", "socket.set_write_timeout(limit)"),
        ("sleep", "std::thread::sleep_ms(5);"),
        ("sleep", "std::thread::sleep_until(deadline);"),
        ("park_timeout", "std::thread::park_timeout_ms(5);"),
        (
            "wait_timeout",
            "condvar.wait_timeout_while(guard, limit, f)",
        ),
        ("wait_timeout", "condvar.wait_timeout_ms(guard, 5)"),
    ] {
        for dir in crate_rule_dirs() {
            let path = format!("{dir}/x.rs");
            let hits = clock_hits(Path::new(&path), line);
            assert!(
                hits.contains(&format!("{path}:1: {word}")),
                "{path}: {line}: {hits:?}"
            );
        }
    }
}

#[test]
fn time_in_crates_lets_the_runner_name_durations_and_limits() {
    let text = "let limit = Duration::from_secs(5);\n\
                let took = report.elapsed;\n\
                let b = Budget::with_timeout(limit);\n\
                let wall = 3; let timeout = 1;\n";
    for path in [
        "crates/raf-okf/src/a.rs",
        "crates/raf-workplan/tests/a.rs",
        "crates/cargo-workplan/src/a.rs",
        "tests/a.rs",
    ] {
        assert_eq!(
            clock_hits(Path::new(path), text),
            Vec::<String>::new(),
            "{path}"
        );
    }
    for dir in SOURCE_RULE_DIRS {
        let path = format!("{dir}/a.rs");
        let hits = clock_hits(Path::new(&path), text);
        assert!(hits.contains(&format!("{path}:1: Duration")), "{hits:?}");
        assert!(hits.contains(&format!("{path}:2: elapsed")), "{hits:?}");
    }
}

#[test]
fn time_in_crates_exempts_the_clock_file_but_not_its_neighbour() {
    let text = planted_instant();
    assert_eq!(
        clock_hits(Path::new(CLOCK_CLOCK_FILE), &text),
        Vec::<String>::new()
    );
    assert_eq!(
        clock_hits(Path::new("crates/raf-workplan/src/exec/system.rs"), &text),
        vec!["crates/raf-workplan/src/exec/system.rs:1: Instant".to_owned()]
    );
}

#[test]
fn time_in_crates_exempts_the_gate_file_but_not_its_neighbours() {
    let text = planted_instant();
    assert_eq!(
        clock_hits(Path::new(CLOCK_GATE_FILE), &text),
        Vec::<String>::new()
    );
    for path in ["tests/support/mod.rs", "tests/docs_contract.rs"] {
        assert_eq!(
            clock_hits(Path::new(path), &text),
            vec![format!("{path}:1: Instant")],
            "{path}"
        );
    }
}

#[test]
fn time_in_crates_passes_the_clean_neighbour() {
    let text = "fn count(tree: &Counting) -> usize { tree.reads() }\n\
                // an instantaneous step, a duration-free count, sleepless\n\
                fn Instantiate() {}\n\
                let state = \"sleeping\";\n\
                let unslept = oversleep + sleepy;\n";
    for path in [
        "crates/raf-source/tests/a.rs",
        "crates/raf-source/examples/a.rs",
        "crates/raf-okf/src/a.rs",
        "crates/cargo-workplan/tests/a.rs",
    ] {
        assert_eq!(clock_hits(Path::new(path), text), Vec::<String>::new());
    }
}

#[test]
fn time_in_crates_exempts_the_paths_outside_every_rule() {
    let text = planted();
    for path in [
        "crates/raf-source/README.md",
        "crates/raf-source/src_notes/a.rs",
        "crates/raf-okf/README.md",
        "crates/raf-okf/benches/a.rs",
        "crates/raf-okf/build.rs",
        "fuzz/fuzz_targets/a.rs",
        "docs/a.rs",
        "docs/CLOCK.md",
        "README.md",
        "build.rs",
    ] {
        assert!(clock_hits(Path::new(path), &text).is_empty(), "{path}");
    }
}

// ---------------------------------------------------------------------------
// Fixtures: version agreement
// ---------------------------------------------------------------------------

const MANIFEST: &str = "[package]\nversion.workspace = true\n\n[workspace.package]\nversion = \"0.1.0\"\nedition = \"2024\"\n";

#[test]
fn citation_accepts_matching_versions() {
    assert!(version_problem(MANIFEST, "title: x\nversion: \"0.1.0\"\n").is_none());
    assert!(version_problem(MANIFEST, "version: 0.1.0\n").is_none());
}

#[test]
fn citation_rejects_a_version_bump_without_a_matching_manifest() {
    let problem = version_problem(MANIFEST, "version: \"0.2.0\"\n");
    assert!(problem.is_some_and(|p| p.contains("0.2.0") && p.contains("0.1.0")));
}

#[test]
fn citation_rejects_a_manifest_bump_without_a_matching_citation() {
    let bumped = MANIFEST.replace("0.1.0", "0.3.0");
    assert!(version_problem(&bumped, "version: \"0.1.0\"\n").is_some());
}

#[test]
fn citation_rejects_a_missing_version() {
    assert!(version_problem(MANIFEST, "title: x\n").is_some());
    assert!(version_problem("[package]\nversion = \"0.1.0\"\n", "version: \"0.1.0\"\n").is_some());
}

// ---------------------------------------------------------------------------
// Gates on the real tree: lints and licence configuration
// ---------------------------------------------------------------------------

#[test]
fn lints_fuzz_workspace_tables_equal_the_root_tables() {
    let root = gate_root();
    let problem = lint_tables_problem(
        &read_text(&root, "Cargo.toml"),
        &read_text(&root, "fuzz/Cargo.toml"),
    );
    assert_eq!(problem, None);
}

#[test]
fn lints_the_root_denies_missing_docs_unsafe_code_and_unreachable_pub() {
    let manifest = read_text(&gate_root(), "Cargo.toml");
    for lint in ["missing_docs", "unsafe_code", "unreachable_pub"] {
        assert_eq!(
            rust_lint_level(&manifest, lint).as_deref(),
            Some("deny"),
            "[workspace.lints.rust] must set {lint} = \"deny\""
        );
    }
}

#[test]
fn lints_every_workspace_manifest_inherits_the_workspace_lints() {
    let root = gate_root();
    let manifest = read_text(&root, "Cargo.toml");
    let mut names: Vec<String> = workspace_members(&manifest)
        .iter()
        .map(|member| format!("{member}/Cargo.toml"))
        .collect();
    assert!(!names.is_empty(), "the root manifest lists no members");
    names.push("Cargo.toml".to_string());
    names.push("fuzz/Cargo.toml".to_string());
    let missing = without_workspace_lints(&names, |name| read_text(&root, name));
    assert_eq!(missing, Vec::<String>::new());
}

#[test]
fn deny_fuzz_configuration_is_the_root_configuration_plus_one_exception() {
    let root = gate_root();
    let problem = deny_problem(
        &read_text(&root, "deny.toml"),
        &read_text(&root, "fuzz/deny.toml"),
    );
    assert_eq!(problem, None);
}

// ---------------------------------------------------------------------------
// Fixtures: lints and licence configuration
// ---------------------------------------------------------------------------

const LINT_ROOT: &str = "[workspace]\nmembers = [\"a\"]\n\n[workspace.lints.rust]\n# why\nmissing_docs = \"deny\"\n\n[workspace.lints.clippy]\nall = \"warn\"\n";

#[test]
fn lints_reads_the_level_of_a_lint_and_sees_a_weakened_one() {
    assert_eq!(
        rust_lint_level(LINT_ROOT, "missing_docs").as_deref(),
        Some("deny")
    );
    let weakened = LINT_ROOT.replace("missing_docs = \"deny\"", "missing_docs = \"warn\"");
    assert_eq!(
        rust_lint_level(&weakened, "missing_docs").as_deref(),
        Some("warn")
    );
    // A lint of another table, or one that is not set, has no level here.
    assert_eq!(rust_lint_level(LINT_ROOT, "all"), None);
    assert_eq!(rust_lint_level(LINT_ROOT, "unsafe_code"), None);
}

#[test]
fn lints_accepts_equal_tables_whatever_the_comments() {
    let fuzz = "[package]\nname = \"f\"\n\n[workspace.lints.rust]\nmissing_docs = \"deny\"\n# other note\n\n[workspace.lints.clippy]\nall = \"warn\"\n";
    assert_eq!(lint_tables_problem(LINT_ROOT, fuzz), None);
}

#[test]
fn lints_rejects_a_weakened_level_in_the_fuzz_tables() {
    let fuzz = LINT_ROOT.replace("\"deny\"", "\"warn\"");
    assert!(lint_tables_problem(LINT_ROOT, &fuzz).is_some());
}

#[test]
fn lints_rejects_an_added_or_removed_lint_in_the_fuzz_tables() {
    let added = format!("{LINT_ROOT}pedantic = \"warn\"\n");
    assert!(lint_tables_problem(LINT_ROOT, &added).is_some());
    let removed = LINT_ROOT.replace("missing_docs = \"deny\"\n", "");
    assert!(lint_tables_problem(LINT_ROOT, &removed).is_some());
    assert!(lint_tables_problem(LINT_ROOT, "[package]\nname = \"f\"\n").is_some());
}

#[test]
fn lints_rejects_a_root_without_lint_tables() {
    assert!(lint_tables_problem("[workspace]\n", "[workspace]\n").is_some());
}

#[test]
fn lints_reads_single_and_multi_line_member_lists() {
    assert_eq!(workspace_members(LINT_ROOT), ["a"]);
    let spread = "[workspace]\nresolver = \"3\"\nmembers = [\n  \"crates/a\",\n  \"crates/b\",\n]\nexclude = [\"fuzz\"]\n";
    assert_eq!(workspace_members(spread), ["crates/a", "crates/b"]);
}

#[test]
fn lints_rejects_a_member_manifest_that_does_not_inherit() {
    let names = [
        "ok/Cargo.toml",
        "plain/Cargo.toml",
        "off/Cargo.toml",
        "other/Cargo.toml",
    ]
    .map(str::to_string);
    let read = |name: &str| match name {
        "ok/Cargo.toml" => "[package]\nname = \"ok\"\n\n[lints]\nworkspace = true\n".to_string(),
        "plain/Cargo.toml" => "[package]\nname = \"plain\"\n".to_string(),
        "off/Cargo.toml" => "[lints]\nworkspace = false\n".to_string(),
        // `workspace = true` under another table is not the lint switch.
        _ => "[package]\nworkspace = true\n\n[lints]\nrust = {}\n".to_string(),
    };
    assert_eq!(
        without_workspace_lints(&names, read),
        ["plain/Cargo.toml", "off/Cargo.toml", "other/Cargo.toml"]
    );
}

const DENY_ROOT: &str = "# note\n[licenses]\nallow = [\"MIT\"]\n\n[[licenses.exceptions]]\ncrate = \"unicode-ident\"\nallow = [\"Unicode-3.0\"]\n\n[bans]\nwildcards = \"deny\"\n";

fn fuzz_deny() -> String {
    DENY_ROOT.replace(
        "\n[bans]",
        "\n# why\n[[licenses.exceptions]]\ncrate = \"libfuzzer-sys@=0.4.13\"\nallow = [\"NCSA\"]\n\n[bans]",
    )
}

#[test]
fn deny_accepts_the_root_configuration_plus_the_one_exception() {
    assert_eq!(deny_problem(DENY_ROOT, &fuzz_deny()), None);
}

#[test]
fn deny_rejects_a_shared_section_that_differs() {
    let changed = fuzz_deny().replace("wildcards = \"deny\"", "wildcards = \"warn\"");
    assert!(deny_problem(DENY_ROOT, &changed).is_some());
    let widened = fuzz_deny().replace("allow = [\"MIT\"]", "allow = [\"MIT\", \"GPL-3.0\"]");
    assert!(deny_problem(DENY_ROOT, &widened).is_some());
}

#[test]
fn deny_rejects_a_missing_or_widened_or_repeated_exception() {
    assert!(deny_problem(DENY_ROOT, DENY_ROOT).is_some());
    let widened = fuzz_deny().replace("[\"NCSA\"]", "[\"NCSA\", \"GPL-3.0\"]");
    assert!(deny_problem(DENY_ROOT, &widened).is_some());
    let other_crate = fuzz_deny().replace("libfuzzer-sys", "other");
    assert!(deny_problem(DENY_ROOT, &other_crate).is_some());
    let twice = fuzz_deny().replace(
        "\n[bans]",
        "\n[[licenses.exceptions]]\ncrate = \"libfuzzer-sys@=0.4.13\"\nallow = [\"NCSA\"]\n\n[bans]",
    );
    assert!(deny_problem(DENY_ROOT, &twice).is_some());
}

#[test]
fn deny_rejects_a_dropped_root_exception() {
    let dropped = fuzz_deny().replace(
        "[[licenses.exceptions]]\ncrate = \"unicode-ident\"\nallow = [\"Unicode-3.0\"]\n",
        "",
    );
    assert!(deny_problem(DENY_ROOT, &dropped).is_some());
}
