// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Bounded candidate evidence and independently verified syntax-region pairs.

use super::analysis::{Analysis, Function};
use super::collect::Span;
use super::discovery::{self, GRAM};
use super::policy::{self, Limits};
use super::proof::{self, Region};
use super::regions;
use super::resource::{Meter, Refusal, Scratch};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Anchor {
    pub(super) function: usize,
    pub(super) atom: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Posting {
    digest: [u8; 32],
    anchor: Anchor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Alignment {
    pub(super) left: Anchor,
    pub(super) right: Anchor,
    pub(super) len: usize,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Candidate {
    pub(super) left: Anchor,
    pub(super) right: Anchor,
    pub(super) seed_digest: [u8; 32],
    /// None means the matching hashes failed exact coarse comparison.
    pub(super) alignment: Option<Alignment>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Site {
    pub(super) function: usize,
    pub(super) bytes: Span,
    pub(super) first_line: usize,
    pub(super) last_line: usize,
    pub(super) atom_start: usize,
    pub(super) atom_end: usize,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Witness {
    pub(super) left: Site,
    pub(super) right: Site,
    pub(super) candidate: usize,
    pub(super) digest: [u8; 32],
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Counts {
    pub(super) atoms: usize,
    pub(super) postings: usize,
    pub(super) seed_pairs: usize,
    pub(super) regions: usize,
    pub(super) verified: usize,
}

/// Output buffers and input facts all precede their one original account.
pub(super) struct Matches<'source> {
    pub(super) candidates: Vec<Candidate>,
    pub(super) witnesses: Vec<Witness>,
    pub(super) counts: Counts,
    pub(super) analysis: Analysis<'source>,
}

pub(super) fn find(analysis: Analysis<'_>, limits: Limits) -> Result<Matches<'_>, Refusal> {
    let mut output = Matches {
        candidates: Vec::new(),
        witnesses: Vec::new(),
        counts: Counts::default(),
        analysis,
    };
    {
        let meter = &output.analysis.meter;
        let mut postings = Scratch::new(meter);
        let mut atoms = 0;
        for (function, input) in output.analysis.functions.iter().enumerate() {
            meter.work(1)?;
            atoms = policy::add(
                atoms,
                input.tokens.atoms.len(),
                limits.atoms,
                "indexed atoms",
            )?;
            // Eligibility is monotone under taking subregions: a function
            // failing the complete floor cannot contain an eligible fragment.
            if !policy::eligible(&input.tokens.atoms, &input.facts.declarations, meter)? {
                continue;
            }
            let seeds = discovery::discover(&input.tokens.atoms, meter)?;
            for seed in seeds.iter() {
                meter.work(1)?;
                policy::add(postings.len(), 1, limits.postings, "seed postings")?;
                postings.push(Posting {
                    digest: seed.digest,
                    anchor: Anchor {
                        function,
                        atom: seed.at,
                    },
                })?;
            }
        }
        output.counts.atoms = atoms;
        output.counts.postings = postings.len();
        run(
            &output.analysis,
            postings.as_mut_slice(),
            limits,
            &mut output.candidates,
            &mut output.witnesses,
            &mut output.counts,
        )?;
    }
    Ok(output)
}

fn run(
    analysis: &Analysis<'_>,
    postings: &mut [Posting],
    limits: Limits,
    candidates: &mut Vec<Candidate>,
    witnesses: &mut Vec<Witness>,
    counts: &mut Counts,
) -> Result<(), Refusal> {
    let meter = &analysis.meter;
    sort(postings, meter, |a, b| {
        (a.digest, a.anchor).cmp(&(b.digest, b.anchor))
    })?;
    let mut engine = Engine {
        functions: &analysis.functions,
        meter,
        limits,
        candidates,
        witnesses,
        counts,
    };
    let mut start = 0;
    while start < postings.len() {
        meter.work(1)?;
        let mut end = start + 1;
        while end < postings.len() {
            meter.work(32)?;
            if postings[end].digest != postings[start].digest {
                break;
            }
            end += 1;
        }
        let len = end - start;
        let pairs = len
            .checked_mul(len - 1)
            .map(|pairs| pairs / 2)
            .ok_or(Refusal::Count {
                resource: "matching-hash seed pairs",
                limit: limits.seed_pairs,
            })?;
        engine.counts.seed_pairs = policy::add(
            engine.counts.seed_pairs,
            pairs,
            limits.seed_pairs,
            "matching-hash seed pairs",
        )?;
        for left in start..end {
            for right in left + 1..end {
                meter.work(1)?;
                if postings[left].anchor == postings[right].anchor {
                    return Err(Refusal::Invariant(
                        "a seed posting was indexed more than once",
                    ));
                }
                engine.seed(postings[left], postings[right])?;
            }
        }
        start = end;
    }
    engine.maximal()
}

struct Engine<'source, 'borrow> {
    functions: &'borrow [Function<'source>],
    meter: &'borrow Meter,
    limits: Limits,
    candidates: &'borrow mut Vec<Candidate>,
    witnesses: &'borrow mut Vec<Witness>,
    counts: &'borrow mut Counts,
}

impl Engine<'_, '_> {
    fn seed(&mut self, left: Posting, right: Posting) -> Result<(), Refusal> {
        let alignment = expand(self.functions, left.anchor, right.anchor, self.meter)?;
        if let Some(alignment) = alignment {
            for prior in self.candidates.iter() {
                self.meter.work(1)?;
                if prior.alignment == Some(alignment) {
                    return Ok(());
                }
            }
        }
        policy::add(
            self.candidates.len(),
            1,
            self.limits.candidates,
            "discovery candidates",
        )?;
        let candidate = self.candidates.len();
        self.meter.push_retained(
            self.candidates,
            Candidate {
                left: left.anchor,
                right: right.anchor,
                seed_digest: left.digest,
                alignment,
            },
        )?;
        let Some(alignment) = alignment else {
            return Ok(());
        };
        let functions = self.functions;
        let function = &functions[alignment.left.function];
        let bounds = lexical_span(
            function,
            alignment.left.atom,
            alignment.left.atom + alignment.len,
            self.meter,
        )?;
        let mut seen = Scratch::new(self.meter);
        self.propose(alignment, candidate, function.facts.body, bounds, &mut seen)?;
        for expression in &function.facts.expressions {
            self.meter.work(1)?;
            self.propose(alignment, candidate, *expression, bounds, &mut seen)?;
        }
        // The flat collector's nested traversal interleaves blocks. Source
        // order within one block, not adjacent Vec slots, defines sequences.
        for (at, first) in function.facts.statements.iter().enumerate() {
            self.meter.work(1)?;
            if first.span.start < bounds.start || first.span.start >= bounds.end {
                continue;
            }
            for last in &function.facts.statements[at..] {
                self.meter.work(1)?;
                if last.block != first.block {
                    continue;
                }
                if last.span.end > bounds.end {
                    break;
                }
                self.propose(
                    alignment,
                    candidate,
                    Span {
                        start: first.span.start,
                        end: last.span.end,
                    },
                    bounds,
                    &mut seen,
                )?;
            }
        }
        Ok(())
    }

    fn propose(
        &mut self,
        alignment: Alignment,
        candidate: usize,
        span: Span,
        bounds: Span,
        seen: &mut Scratch<'_, Span>,
    ) -> Result<(), Refusal> {
        self.meter.work(1)?;
        if span.start < bounds.start || span.end > bounds.end {
            return Ok(());
        }
        for prior in seen.iter() {
            self.meter.work(1)?;
            if *prior == span {
                return Ok(());
            }
        }
        self.counts.regions = policy::add(
            self.counts.regions,
            1,
            self.limits.regions,
            "syntax-region proposals",
        )?;
        seen.push(span)?;
        let functions = self.functions;
        let a = &functions[alignment.left.function];
        let b = &functions[alignment.right.function];
        let Some(left) = regions::admit(a.source, &a.facts, &a.tokens, span, self.meter)? else {
            return Ok(());
        };
        if left.atoms.start < alignment.left.atom
            || left.atoms.end > alignment.left.atom + alignment.len
        {
            return Ok(());
        }
        let start = alignment.right.atom + (left.atoms.start - alignment.left.atom);
        let end = start + left.atoms.len();
        let right_span = lexical_span(b, start, end, self.meter)?;
        let Some(right) = regions::admit(b.source, &b.facts, &b.tokens, right_span, self.meter)?
        else {
            return Ok(());
        };
        if right.atoms != (start..end) {
            return Ok(());
        }
        if same_file(a, b)? && overlap(left.bytes, right.bytes) {
            return Ok(());
        }
        let left_region = Region {
            bytes: left.bytes.start..left.bytes.end,
            atoms: &a.tokens.atoms[left.atoms.clone()],
            declarations: &a.facts.declarations,
        };
        let right_region = Region {
            bytes: right.bytes.start..right.bytes.end,
            atoms: &b.tokens.atoms[right.atoms.clone()],
            declarations: &b.facts.declarations,
        };
        if !policy::eligible(left_region.atoms, left_region.declarations, self.meter)?
            || !policy::eligible(right_region.atoms, right_region.declarations, self.meter)?
            || !proof::same(&left_region, &right_region, self.meter)?
        {
            return Ok(());
        }
        let left = site(alignment.left.function, &left);
        let right = site(alignment.right.function, &right);
        for prior in self.witnesses.iter() {
            self.meter.work(1)?;
            if prior.left == left && prior.right == right {
                return Ok(());
            }
        }
        self.counts.verified = policy::add(
            self.counts.verified,
            1,
            self.limits.verified,
            "verified region pairs before containment",
        )?;
        let digest = proof::digest(&left_region, self.meter)?;
        self.meter.push_retained(
            self.witnesses,
            Witness {
                left,
                right,
                candidate,
                digest,
            },
        )
    }

    fn maximal(&mut self) -> Result<(), Refusal> {
        let mut remove = Scratch::new(self.meter);
        for (at, smaller) in self.witnesses.iter().enumerate() {
            let mut contained = false;
            for (other, larger) in self.witnesses.iter().enumerate() {
                self.meter.work(1)?;
                if at != other
                    && contains(larger.left, smaller.left)
                    && contains(larger.right, smaller.right)
                    && smaller.left.atom_start - larger.left.atom_start
                        == smaller.right.atom_start - larger.right.atom_start
                {
                    contained = true;
                    break;
                }
            }
            remove.push(contained)?;
        }
        let mut written = 0;
        for (at, remove) in remove.iter().enumerate() {
            self.meter.work(1)?;
            if !*remove {
                self.witnesses[written] = self.witnesses[at];
                written += 1;
            }
        }
        self.meter.work(self.witnesses.len() - written)?;
        self.witnesses.truncate(written);
        sort(self.witnesses, self.meter, |a, b| {
            (a.left, a.right).cmp(&(b.left, b.right))
        })
    }
}

fn expand(
    functions: &[Function<'_>],
    left: Anchor,
    right: Anchor,
    meter: &Meter,
) -> Result<Option<Alignment>, Refusal> {
    let a = &functions
        .get(left.function)
        .ok_or(Refusal::Invariant("left seed has no actual function"))?
        .tokens
        .atoms;
    let b = &functions
        .get(right.function)
        .ok_or(Refusal::Invariant("right seed has no actual function"))?
        .tokens
        .atoms;
    let left_end = left
        .atom
        .checked_add(GRAM)
        .ok_or(Refusal::Invariant("left seed range overflows"))?;
    let right_end = right
        .atom
        .checked_add(GRAM)
        .ok_or(Refusal::Invariant("right seed range overflows"))?;
    let a_seed = a
        .get(left.atom..left_end)
        .ok_or(Refusal::Invariant("left seed is outside original atoms"))?;
    let b_seed = b
        .get(right.atom..right_end)
        .ok_or(Refusal::Invariant("right seed is outside original atoms"))?;
    for (a, b) in a_seed.iter().zip(b_seed) {
        if !discovery::same_atom(*a, *b, meter)? {
            return Ok(None);
        }
    }
    let mut left_start = left.atom;
    let mut right_start = right.atom;
    while left_start > 0 && right_start > 0 {
        if !discovery::same_atom(a[left_start - 1], b[right_start - 1], meter)? {
            break;
        }
        left_start -= 1;
        right_start -= 1;
    }
    let mut len = left_end - left_start;
    while left_start + len < a.len() && right_start + len < b.len() {
        if !discovery::same_atom(a[left_start + len], b[right_start + len], meter)? {
            break;
        }
        len += 1;
    }
    Ok(Some(Alignment {
        left: Anchor {
            atom: left_start,
            ..left
        },
        right: Anchor {
            atom: right_start,
            ..right
        },
        len,
    }))
}

fn lexical_span(
    function: &Function<'_>,
    start: usize,
    end: usize,
    meter: &Meter,
) -> Result<Span, Refusal> {
    let atoms = function
        .tokens
        .atoms
        .get(start..end)
        .ok_or(Refusal::Invariant(
            "aligned region is outside original atoms",
        ))?;
    let spans = function
        .tokens
        .spans
        .get(start..end)
        .ok_or(Refusal::Invariant("aligned region lost its source spans"))?;
    let mut span: Option<Span> = None;
    for (atom, position) in atoms.iter().zip(spans) {
        meter.work(1)?;
        if atom.spelling.is_empty() {
            continue;
        }
        if let Some(span) = &mut span {
            span.end = position.end;
        } else {
            span = Some(*position);
        }
    }
    span.ok_or(Refusal::Invariant(
        "aligned syntax has no original lexical token",
    ))
}

fn same_file(a: &Function<'_>, b: &Function<'_>) -> Result<bool, Refusal> {
    let (a, _) = a.origin.source().ok_or(Refusal::Invariant(
        "matching requires actual Source provenance",
    ))?;
    let (b, _) = b.origin.source().ok_or(Refusal::Invariant(
        "matching requires actual Source provenance",
    ))?;
    Ok(std::ptr::eq(a, b))
}

fn site(function: usize, region: &regions::Admitted) -> Site {
    Site {
        function,
        bytes: region.bytes,
        first_line: region.first_line,
        last_line: region.last_line,
        atom_start: region.atoms.start,
        atom_end: region.atoms.end,
    }
}

fn overlap(a: Span, b: Span) -> bool {
    a.start < b.end && b.start < a.end
}

fn contains(a: Site, b: Site) -> bool {
    a.function == b.function
        && a.bytes.start <= b.bytes.start
        && b.bytes.end <= a.bytes.end
        && a.atom_start <= b.atom_start
        && b.atom_end <= a.atom_end
}

fn sort<T>(
    values: &mut [T],
    meter: &Meter,
    mut compare: impl FnMut(&T, &T) -> std::cmp::Ordering,
) -> Result<(), Refusal> {
    // One initial movement allowance per slot plus comparisons and their
    // bounded swaps in the allocation-free native partition/heap algorithm.
    meter.work(values.len())?;
    purrdf_lex::walk::try_sort_unstable_by(values, |a, b| {
        meter.work(40)?;
        Ok(compare(a, b))
    })
}

#[cfg(test)]
#[path = "pairing_tests.rs"]
mod tests;
