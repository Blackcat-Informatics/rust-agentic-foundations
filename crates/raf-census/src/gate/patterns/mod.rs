// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Ledger pattern execution on an owned lexical session.

pub(super) mod paths;
mod source;
mod tokens;

use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::walk::WorkList;
use raf_source::{RelPath, SourceFile, Workspace};

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
        match_tokens(&compiled, workspace, census, owners, &mut found, limits)?;
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

/// Matches each compiled tokens pattern in the files its scope admits. Normalization
/// is the shared `tokens` scope: it draws on its own copy of the matching grant,
/// once per file. Each pattern has its own allowance, which grows by the work the
/// admitted file can cost that pattern, so one rule never spends another's steps.
fn match_tokens<'p, 'w>(
    compiled: &[(&'p LedgerPattern, Form)],
    workspace: &'w Workspace,
    census: &Census,
    owners: &super::owners::Owners<'_>,
    found: &mut Vec<Found<'p, 'w>>,
    limits: &MatchLimits,
) -> Result<(), Error> {
    let files: Vec<&SourceFile> = workspace.files().collect();
    let patterns: Vec<&LedgerPattern> = compiled.iter().map(|(pattern, _)| *pattern).collect();
    let mut normalization = MatchLimits {
        steps: limits.steps,
        states: limits.states,
    };
    let mut allowances: Vec<Allowance> = compiled
        .iter()
        .map(|_| Allowance::new(limits.states))
        .collect();
    for (index, file) in files.iter().copied().enumerate() {
        let Some(context) = census.source_context(&file.path) else {
            continue;
        };
        // A pattern whose scope excludes this file can never report a hit in
        // it (the same file context decides every hit), so its matching is
        // not run. A file no pattern admits is never normalized.
        let admitted: Vec<usize> = compiled
            .iter()
            .enumerate()
            .filter(|(_, (pattern, _))| {
                pattern.scope.is_unscoped() || pattern.scope.admits_file(context, &file.path)
            })
            .map(|(position, _)| position)
            .collect();
        if admitted.is_empty() {
            continue;
        }
        let normalized =
            source::normalized(workspace, file, owners, &mut normalization).map_err(|error| {
                let rules = admitted
                    .iter()
                    .map(|&position| compiled[position].0.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                stopped_at(error, &file.path, Stop::Shared(&rules), || {
                    unfinished(&files[index..], census, &patterns)
                })
            })?;
        let file_weight = tokens::weight(&normalized.nodes);
        for position in admitted {
            let (pattern, form) = &compiled[position];
            let allowance = &mut allowances[position];
            allowance.grow(
                tokens::weight(&form.nodes)
                    .saturating_mul(file_weight)
                    .saturating_mul(2),
            );
            let granted = allowance.granted;
            let hits = tokens::find(&pattern.id, form, &normalized, &mut allowance.limits)
                .map_err(|error| {
                    stopped_at(error, &file.path, Stop::Rule(granted), || {
                        unfinished(&files[index..], census, &patterns)
                    })
                })?;
            for (byte, line) in hits {
                retain(found, pattern, &file.path, byte, line, None, limits)?;
            }
        }
    }
    Ok(())
}

/// The matching steps granted to one tokens pattern. Each admitted file adds its
/// share before that file is matched, so the grant follows the admitted tree.
struct Allowance {
    granted: u64,
    limits: MatchLimits,
}

impl Allowance {
    fn new(states: usize) -> Self {
        Self {
            granted: 0,
            limits: MatchLimits { steps: 0, states },
        }
    }

    fn grow(&mut self, steps: u64) {
        self.granted = self.granted.saturating_add(steps);
        self.limits.steps = self.limits.steps.saturating_add(steps);
    }
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

/// Who spent the steps of a refused tokens match.
#[derive(Clone, Copy)]
enum Stop<'a> {
    /// Normalization of a file, charged to the shared `tokens` scope; the refusal
    /// names the patterns that admitted the file.
    Shared(&'a str),
    /// Matching by one pattern, which the refusal names with its allowance.
    Rule(u64),
}

/// Names the file where a step-limit refusal stopped matching and every admitted
/// file that it left unfinished, in workspace order. Other errors pass through
/// unchanged.
fn stopped_at(
    error: Error,
    file: &RelPath,
    stop: Stop<'_>,
    unfinished: impl FnOnce() -> String,
) -> Error {
    match error {
        Error::Source {
            file: scope,
            line,
            reason,
        } if scope.starts_with("rule:") && reason.contains("step limit exhausted") => {
            let (scope, reason) = match stop {
                Stop::Shared(rules) if scope == "rule:tokens" => (format!("rule:{rules}"), reason),
                Stop::Shared(_) => (scope, reason),
                Stop::Rule(granted) => (
                    scope,
                    format!(
                        "matching allowance of {granted} steps exhausted; the allowance is \
                         2 x the admitted tree's weight x the pattern's weight, so narrow the \
                         pattern's scope or the file roots"
                    ),
                ),
            };
            Error::Source {
                file: scope,
                line,
                reason: format!(
                    "{reason}; matching stopped at {file}; unfinished admitted files: {}",
                    unfinished()
                ),
            }
        }
        other => other,
    }
}

/// Lists, in workspace order, the files of `files` that some compiled token
/// pattern admits. The refusal names them, so the caller knows what was not matched.
fn unfinished(files: &[&SourceFile], census: &Census, patterns: &[&LedgerPattern]) -> String {
    files
        .iter()
        .filter(|file| {
            census.source_context(&file.path).is_some_and(|context| {
                patterns.iter().any(|pattern| {
                    pattern.scope.is_unscoped() || pattern.scope.admits_file(context, &file.path)
                })
            })
        })
        .map(|file| file.path.to_string())
        .collect::<Vec<_>>()
        .join(", ")
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
