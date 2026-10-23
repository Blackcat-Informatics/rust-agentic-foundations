// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Ledger pattern execution on an owned lexical session.

mod paths;
mod source;
mod tokens;

use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::walk::WorkList;
use raf_source::{RelPath, Workspace};

use crate::normalize::{Kind, Node, leading_forms, literal_kind};
use crate::{Error, Form, LedgerPattern, PatternMatcher, Slice, StringKind};

pub(super) struct Found<'p, 'w> {
    pub(super) pattern: &'p LedgerPattern,
    pub(super) file: &'w RelPath,
    pub(super) byte: usize,
    pub(super) line: usize,
}

pub(super) fn collect<'p, 'w>(
    patterns: &'p [LedgerPattern],
    workspace: &'w Workspace,
    code: &raf_source::CodeResolver<'_>,
    owners: &super::owners::Owners<'_>,
    limits: &mut MatchLimits,
) -> Result<Vec<Found<'p, 'w>>, Error> {
    if patterns.is_empty() {
        return Ok(Vec::new());
    }
    // Only plain owned forms/positions leave this existing lexical worker.
    // A caller's proc-macro spans never enter it or get invalidated by it.
    crate::units::on_thread(|| run(patterns, workspace, code, owners, limits))?
}

fn run<'p, 'w>(
    patterns: &'p [LedgerPattern],
    workspace: &'w Workspace,
    code: &raf_source::CodeResolver<'_>,
    owners: &super::owners::Owners<'_>,
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
    let mut found = paths::collect(&path_patterns, workspace, code, limits)?;
    if !compiled.is_empty() {
        for file in workspace.files() {
            let normalized = source::normalized(workspace, file, owners, limits)?;
            for (pattern, form) in &compiled {
                for (byte, line) in tokens::find(&pattern.id, form, &normalized, limits)? {
                    retain(&mut found, pattern, &file.path, byte, line, limits)?;
                }
            }
        }
    }
    if regular.is_empty() {
        return Ok(found);
    }
    for file in workspace.files() {
        let label = file.path.to_string();
        let verbatim = Form::lex(&label, 1, &file.text)?;
        if !regular.is_empty() {
            let mut pending = WorkList::<_, 16>::new();
            push(
                "literalRegex",
                &mut pending,
                verbatim.nodes.as_slice(),
                0,
                limits,
            )?;
            while let Some(nodes) = pending.pop() {
                for node in nodes {
                    charge("literalRegex", limits, 1)?;
                    match node {
                        Node::Group(group) => {
                            push(
                                "literalRegex",
                                &mut pending,
                                group.children.as_slice(),
                                0,
                                limits,
                            )?;
                        }
                        Node::Leaf(leaf)
                            if leaf.kind == Kind::Literal && literal_kind(&leaf.text) == "$str" =>
                        {
                            let (kind, value, _) = crate::units::decode_string(&leaf.text)
                                .map_err(|reason| Error::Source {
                                    file: label.clone(),
                                    line: leaf.place.line,
                                    reason,
                                })?;
                            if kind != StringKind::Str {
                                return Err(failure(
                                    "literalRegex",
                                    "ordinary string kind changed",
                                ));
                            }
                            for (pattern, regex) in &regular {
                                charge(&pattern.id, limits, 1)?;
                                if regex.is_match(&value, limits).map_err(|error| {
                                    failure(&pattern.id, &format!("native ECMA matcher: {error}"))
                                })? {
                                    retain(
                                        &mut found,
                                        pattern,
                                        &file.path,
                                        leaf.place.at.start,
                                        leaf.place.line,
                                        limits,
                                    )?;
                                }
                            }
                        }
                        Node::Leaf(_) => {}
                    }
                }
            }
        }
    }
    Ok(found)
}

fn retain<'p, 'w>(
    found: &mut Vec<Found<'p, 'w>>,
    pattern: &'p LedgerPattern,
    file: &'w RelPath,
    byte: usize,
    line: usize,
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
    });
    Ok(())
}

pub(super) fn charge(id: &str, limits: &mut MatchLimits, work: u64) -> Result<(), Error> {
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
