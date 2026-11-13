// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Admit complete source syntax, never arbitrary windows, for lexical proof.

use std::ops::Range;

use super::collect::{Facts, Span};
use super::resource::{Meter, Refusal};
use super::tokens::Tokens;

pub(super) struct Admitted {
    pub(super) bytes: Span,
    pub(super) first_line: usize,
    pub(super) last_line: usize,
    pub(super) atoms: Range<usize>,
}

pub(super) fn admit(
    source: &str,
    facts: &Facts<'_>,
    tokens: &Tokens<'_>,
    candidate: Span,
    meter: &Meter,
) -> Result<Option<Admitted>, Refusal> {
    meter.work(1)?;
    if candidate.start >= candidate.end
        || candidate.start < facts.body.start
        || candidate.end > facts.body.end
        || source.get(candidate.start..candidate.end).is_none()
    {
        return Err(Refusal::Invariant(
            "candidate is not a nonempty UTF-8 region of its owner",
        ));
    }
    if !syntax(facts, candidate, meter)? {
        return Ok(None);
    }
    for opaque in &facts.opaque {
        meter.work(1)?;
        if candidate.start < opaque.end && opaque.start < candidate.end {
            return Ok(None);
        }
    }
    if tokens.atoms.len() != tokens.spans.len() {
        return Err(Refusal::Invariant(
            "lexical atoms lost their original source spans",
        ));
    }
    let mut first = None;
    let mut end = 0;
    let mut gap = false;
    for (at, (atom, anchor)) in tokens.atoms.iter().zip(&tokens.spans).enumerate() {
        meter.work(1)?;
        let inside = candidate.start <= anchor.start && anchor.end <= candidate.end;
        if inside {
            if gap {
                return Err(Refusal::Invariant(
                    "admitted syntax has a discontinuous lexical view",
                ));
            }
            first.get_or_insert(at);
            end = at + 1;
        } else {
            if !atom.spelling.is_empty()
                && candidate.start < anchor.end
                && anchor.start < candidate.end
            {
                return Err(Refusal::Invariant(
                    "syntax region cuts an original lexical token",
                ));
            }
            gap |= first.is_some();
        }
    }
    let Some(first) = first else {
        return Ok(None);
    };
    let mut first_line = 1;
    let mut last_line = 1;
    for (at, byte) in source.bytes().take(candidate.end).enumerate() {
        meter.work(1)?;
        if byte == b'\n' {
            first_line += usize::from(at < candidate.start);
            last_line += usize::from(at + 1 < candidate.end);
        }
    }
    Ok(Some(Admitted {
        bytes: candidate,
        first_line,
        last_line,
        atoms: first..end,
    }))
}

fn syntax(facts: &Facts<'_>, candidate: Span, meter: &Meter) -> Result<bool, Refusal> {
    if candidate == facts.body {
        return Ok(true);
    }
    for expression in &facts.expressions {
        meter.work(1)?;
        if *expression == candidate {
            return Ok(true);
        }
    }
    for (at, first) in facts.statements.iter().enumerate() {
        meter.work(1)?;
        if first.span.start != candidate.start {
            continue;
        }
        let mut previous = candidate.start;
        for statement in &facts.statements[at..] {
            meter.work(1)?;
            if statement.block != first.block {
                continue;
            }
            if statement.span.start < previous || statement.span.end > candidate.end {
                break;
            }
            if statement.span.end == candidate.end {
                return Ok(true);
            }
            previous = statement.span.end;
        }
    }
    Ok(false)
}
