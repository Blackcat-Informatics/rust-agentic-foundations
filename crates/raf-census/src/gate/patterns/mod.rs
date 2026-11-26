// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Ledger pattern execution on an owned lexical session.

pub(super) mod paths;
mod source;
mod tokens;

use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::walk::WorkList;
use raf_source::{RelPath, Workspace};

use crate::normalize::{Kind, Node, leading_forms, literal_kind};
use crate::{Census, Error, Form, LedgerPattern, PatternMatcher, Slice, StringKind};

pub(super) struct Found<'p, 'w> {
    pub(super) pattern: &'p LedgerPattern,
    pub(super) file: &'w RelPath,
    pub(super) byte: usize,
    pub(super) line: usize,
    /// The items whose resolution matched the hit, when the matcher resolves through owners.
    pub(super) owners: Option<Vec<raf_source::ItemId>>,
}

pub(super) fn collect<'p, 'w>(
    patterns: &'p [LedgerPattern],
    workspace: &'w Workspace,
    census: &Census,
    code: &raf_source::CodeResolver<'_>,
    owners: &super::owners::Owners<'_>,
    references: &[String],
    limits: &mut MatchLimits,
) -> Result<Vec<Found<'p, 'w>>, Error> {
    if patterns.is_empty() {
        return Ok(Vec::new());
    }
    // Only plain owned forms/positions leave this existing lexical worker.
    // A caller's proc-macro spans never enter it or get invalidated by it.
    crate::units::on_thread(|| {
        run(
            patterns, workspace, census, code, owners, references, limits,
        )
    })?
}

fn run<'p, 'w>(
    patterns: &'p [LedgerPattern],
    workspace: &'w Workspace,
    census: &Census,
    code: &raf_source::CodeResolver<'_>,
    owners: &super::owners::Owners<'_>,
    references: &[String],
    limits: &mut MatchLimits,
) -> Result<Vec<Found<'p, 'w>>, Error> {
    let mut compiled = Vec::new();
    let mut regular = Vec::new();
    let mut path_patterns = Vec::new();
    for pattern in patterns {
        match &pattern.matcher {
            PatternMatcher::Tokens(text) => {
                let file = format!("pattern:{}", pattern.id);
                let [_, _, form] = leading_forms(&Slice::new(&file, 1, text))?;
                if form.is_empty() {
                    return Err(failure(&pattern.id, "a tokens matcher must not be empty"));
                }
                compiled.push((pattern, form));
            }
            PatternMatcher::LiteralRegex(regex) => regular.push((pattern, regex)),
            _ => path_patterns.push(pattern),
        }
    }
    let mut found = paths::collect(&path_patterns, workspace, census, code, references, limits)?;
    if !compiled.is_empty() {
        for file in workspace.files() {
            let Some(context) = census.source_context(&file.path) else {
                continue;
            };
            // A pattern whose scope excludes this file can never report a hit in
            // it (the same file context decides every hit), so its matching is
            // not run. A file no pattern admits is never normalized.
            let admitted: Vec<_> = compiled
                .iter()
                .filter(|(pattern, _)| {
                    pattern.scope.is_unscoped() || pattern.scope.admits_file(context, &file.path)
                })
                .collect();
            if admitted.is_empty() {
                continue;
            }
            let normalized = source::normalized(workspace, file, owners, limits)
                .map_err(|error| stopped_at(error, &file.path))?;
            for (pattern, form) in admitted {
                let hits = tokens::find(&pattern.id, form, &normalized, limits)
                    .map_err(|error| stopped_at(error, &file.path))?;
                for (byte, line) in hits {
                    retain(&mut found, pattern, &file.path, byte, line, None, limits)?;
                }
            }
        }
    }
    if regular.is_empty() {
        return Ok(found);
    }
    for file in workspace.files() {
        if census.source_context(&file.path).is_none() {
            continue;
        }
        let label = file.path.to_string();
        for literal in string_literals("literalRegex", &label, &file.text, limits)? {
            for (pattern, regex) in &regular {
                charge(&pattern.id, limits, 1)?;
                if regex.is_match(&literal.value, limits).map_err(|error| {
                    failure(&pattern.id, &format!("native ECMA matcher: {error}"))
                })? {
                    retain(
                        &mut found,
                        pattern,
                        &file.path,
                        literal.start,
                        literal.line,
                        None,
                        limits,
                    )?;
                }
            }
        }
    }
    Ok(found)
}

/// One written ordinary or raw string literal of a source file.
pub(super) struct Literal {
    /// The byte offset of the literal's first character.
    pub(super) start: usize,
    /// The 1-based line the literal starts on.
    pub(super) line: usize,
    /// The decoded value.
    pub(super) value: String,
}

/// Every ordinary or raw string literal of `text` that is written in the source
/// (not a byte string, a C string or a desugared documentation comment), in
/// document order. The lexing and the walk are charged to `limits`; exceeding
/// a bound refuses, never truncates. Must run on the lexical worker thread.
pub(super) fn string_literals(
    id: &str,
    label: &str,
    text: &str,
    limits: &mut MatchLimits,
) -> Result<Vec<Literal>, Error> {
    let verbatim = Form::lex(label, 1, text)?;
    let mut pending = WorkList::<_, 16>::new();
    push(id, &mut pending, verbatim.nodes.as_slice(), 0, limits)?;
    let mut literals = Vec::new();
    while let Some(nodes) = pending.pop() {
        for node in nodes {
            charge(id, limits, 1)?;
            match node {
                Node::Group(group) => {
                    push(id, &mut pending, group.children.as_slice(), 0, limits)?;
                }
                Node::Leaf(leaf)
                    if leaf.kind == Kind::Literal && literal_kind(&leaf.text) == "$str" =>
                {
                    if written_literal(text, leaf.place.at.clone())? {
                        literals.push(Literal {
                            start: leaf.place.at.start,
                            line: leaf.place.line,
                            value: ordinary_string(label, leaf.place.line, &leaf.text)?,
                        });
                    }
                }
                Node::Leaf(_) => {}
            }
        }
    }
    literals.sort_by_key(|literal| literal.start);
    Ok(literals)
}

fn written_literal(text: &str, span: std::ops::Range<usize>) -> Result<bool, Error> {
    // proc-macro2 desugars documentation comments into strings, but their
    // spans retain the comment text. Explicit doc attributes keep their quotes.
    text.get(span)
        .map(|written| !written.starts_with('/'))
        .ok_or_else(|| failure("literalRegex", "literal span is outside its source text"))
}

fn ordinary_string(label: &str, line: usize, text: &str) -> Result<String, Error> {
    let (kind, value, _) = crate::units::decode_string(text).map_err(|reason| Error::Source {
        file: label.to_owned(),
        line,
        reason,
    })?;
    if kind != StringKind::Str {
        return Err(failure("literalRegex", "ordinary string kind changed"));
    }
    Ok(value)
}

fn retain<'p, 'w>(
    found: &mut Vec<Found<'p, 'w>>,
    pattern: &'p LedgerPattern,
    file: &'w RelPath,
    byte: usize,
    line: usize,
    owners: Option<Vec<raf_source::ItemId>>,
    limits: &MatchLimits,
) -> Result<(), Error> {
    if found.len() >= limits.states {
        return Err(failure(&pattern.id, "matching hit-state limit exhausted"));
    }
    found.push(Found {
        pattern,
        file,
        byte,
        line,
        owners,
    });
    Ok(())
}

/// Names the file where a step-limit refusal stopped matching. Every admitted
/// file after it is unfinished, so the refusal states that the matching stopped
/// there. Other errors pass through unchanged.
fn stopped_at(error: Error, file: &RelPath) -> Error {
    match error {
        Error::Source {
            file: scope,
            line,
            reason,
        } if scope.starts_with("rule:") && reason.contains("step limit exhausted") => {
            Error::Source {
                file: scope,
                line,
                reason: format!(
                    "{reason}; matching stopped at {file}, and the admitted files after it were not finished"
                ),
            }
        }
        other => other,
    }
}

pub(in crate::gate) fn charge(id: &str, limits: &mut MatchLimits, work: u64) -> Result<(), Error> {
    limits.steps = limits.steps.checked_sub(work).ok_or_else(|| {
        failure(id, "matching step limit exhausted; narrow the pattern/roots or raise the explicit matching grant")
    })?;
    Ok(())
}

pub(super) fn push<T, const N: usize>(
    id: &str,
    pending: &mut WorkList<T, N>,
    value: T,
    occupied: usize,
    limits: &MatchLimits,
) -> Result<(), Error> {
    if pending.len().saturating_add(occupied) >= limits.states {
        return Err(failure(id, "matching frontier-state limit exhausted"));
    }
    pending
        .try_push(value)
        .map_err(|source| Error::PatternStorage {
            pattern: id.to_owned(),
            source,
        })
}

pub(super) fn failure(id: &str, reason: &str) -> Error {
    super::rules::refused(id, reason)
}
