// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Versioned fragment eligibility, independent of whole-body normal form.

use super::proof::{Atom, Declaration, Family};
use super::resource::{Meter, Refusal, Scratch};

pub(super) const PROFILE: &str = "embedded-lexical-v1/k8-w4/lex24-info4-fixed2";
pub(super) const MIN_LEXICAL: usize = 24;

#[derive(Clone, Copy)]
pub(super) struct Limits {
    pub(super) atoms: usize,
    pub(super) postings: usize,
    pub(super) seed_pairs: usize,
    pub(super) candidates: usize,
    pub(super) regions: usize,
    pub(super) verified: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            atoms: 2_000_000,
            postings: 250_000,
            seed_pairs: 1_000_000,
            candidates: 100_000,
            regions: 2_000_000,
            verified: 100_000,
        }
    }
}

pub(super) fn add(
    current: usize,
    amount: usize,
    limit: usize,
    resource: &'static str,
) -> Result<usize, Refusal> {
    current
        .checked_add(amount)
        .filter(|next| *next <= limit)
        .ok_or(Refusal::Count { resource, limit })
}

#[derive(Clone, Copy)]
enum Feature<'a> {
    Binding(usize),
    Fixed(&'a str),
}

/// Eligibility is a transparent heuristic, never a semantic verdict. Marker
/// counts, spelling variation of one binding and punctuation padding cannot
/// satisfy its diversity floor. Every proof still retains the complete syntax.
pub(super) fn eligible(
    atoms: &[Atom<'_>],
    declarations: &[Declaration],
    meter: &Meter,
) -> Result<bool, Refusal> {
    let mut lexical = 0;
    for atom in atoms {
        meter.work(1)?;
        lexical += usize::from(!atom.spelling.is_empty());
    }
    if lexical < MIN_LEXICAL {
        return Ok(false);
    }
    let mut features = Scratch::new(meter);
    let mut fixed = 0;
    for atom in atoms {
        meter.work(1)?;
        if atom.spelling.is_empty() {
            continue;
        }
        let feature = if let Some(occurrence) = atom.occurrence {
            let declaration = declarations
                .get(occurrence.binding)
                .ok_or(Refusal::Invariant(
                    "eligibility occurrence has no declaration",
                ))?;
            // Implicit receivers and control/label identities remain in proof,
            // but cannot make repetitive plumbing appear diverse.
            if declaration.family != Family::Value {
                continue;
            }
            Feature::Binding(occurrence.binding)
        } else {
            meter.work(atom.spelling.len())?;
            if !informative(atom.spelling) {
                continue;
            }
            Feature::Fixed(atom.spelling)
        };
        let mut present = false;
        for prior in features.iter() {
            meter.work(1)?;
            present = match (*prior, feature) {
                (Feature::Binding(a), Feature::Binding(b)) => a == b,
                (Feature::Fixed(a), Feature::Fixed(b)) => {
                    meter.work(a.len().max(b.len()))?;
                    a == b
                }
                _ => false,
            };
            if present {
                break;
            }
        }
        if !present {
            // Four existing distinct features already prove the total floor.
            // Only the two fixed features can still be missing; retaining
            // additional binding IDs cannot change either decision.
            if features.len() >= 4 && matches!(feature, Feature::Binding(_)) {
                continue;
            }
            features.push(feature)?;
            fixed += usize::from(matches!(feature, Feature::Fixed(..)));
            if features.len() >= 4 && fixed >= 2 {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn informative(spelling: &str) -> bool {
    if matches!(
        spelling,
        "let"
            | "mut"
            | "ref"
            | "return"
            | "break"
            | "continue"
            | "move"
            | "self"
            | "Self"
            | "as"
            | "true"
            | "false"
            | "fn"
            | "pub"
            | "unsafe"
    ) {
        return false;
    }
    if matches!(
        spelling,
        "+" | "-" | "*" | "/" | "%" | "^" | "&" | "|" | "!" | "<" | ">"
    ) {
        return true;
    }
    // Native lexical tokens distinguish one-character punctuation from word
    // and literal tokens; quoted/raw/numeric literals retain their full text.
    spelling.len() > 1
        || spelling
            .chars()
            .next()
            .is_some_and(|ch| ch == '_' || ch.is_alphanumeric())
}
