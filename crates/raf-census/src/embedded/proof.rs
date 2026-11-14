// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Ordered fragment proof after lexical classification has succeeded.
//!
//! No hash or similarity score enters this comparison. The collector must
//! establish each declaration and use before constructing an occurrence;
//! unresolved pattern names cannot be represented as fresh declarations here.

use std::ops::Range;

use super::resource::{Meter, Refusal, Scratch};
use super::{names, policy};
use purrdf_hash::blake3;

/// Separate lexical namespaces and kinds of control-flow destinations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Family {
    Value,
    Receiver,
    Label,
    Function,
    Closure,
    Loop,
    Try,
    AsyncFunction,
    AsyncClosure,
    AsyncBlock,
    ConstFunction,
    ConstContext,
}

impl Family {
    pub(super) const fn function_like(self) -> bool {
        matches!(
            self,
            Self::Function
                | Self::Closure
                | Self::AsyncFunction
                | Self::AsyncClosure
                | Self::AsyncBlock
                | Self::ConstFunction
        )
    }

    pub(super) const fn asynchronous(self) -> bool {
        matches!(
            self,
            Self::AsyncFunction | Self::AsyncClosure | Self::AsyncBlock
        )
    }
}

/// A declaration and a use remain different even when both spell one name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Event {
    Declaration,
    Use,
}

/// One immutable declaration record shared by every occurrence of that binding.
#[derive(Clone, Copy, Debug)]
pub(super) struct Declaration {
    pub(super) byte: usize,
    pub(super) family: Family,
}

/// A proven lexical occurrence, not an identifier guessed from its spelling.
#[derive(Clone, Copy, Debug)]
pub(super) struct Occurrence<'a> {
    /// Index into the region's complete declaration table, captures included.
    pub(super) binding: usize,
    pub(super) event: Event,
    /// A shorthand field retains its field name; a control marker its opcode.
    pub(super) fixed: Option<&'a str>,
}

/// One ordered token or explicit lexical/control-boundary marker.
/// Only renamable identifiers may replace their spelling with an occurrence.
/// Structural tokens stay fixed; control markers must retain their opcode.
#[derive(Clone, Copy, Debug)]
pub(super) struct Atom<'a> {
    pub(super) spelling: &'a str,
    pub(super) occurrence: Option<Occurrence<'a>>,
    /// Native punctuation spacing is structural (`+=` differs from `+ =`).
    pub(super) joint: bool,
}

/// Complete syntax-aligned region admitted by the source collector.
pub(super) struct Region<'a> {
    pub(super) bytes: Range<usize>,
    pub(super) atoms: &'a [Atom<'a>],
    pub(super) declarations: &'a [Declaration],
}

#[derive(Clone, Copy)]
struct Pair {
    left: usize,
    right: usize,
}

/// Compare actual order and a bijection of declaration identities.
///
/// The same grant pays for every comparison, including failed candidates.
/// Scratch remains admitted until its backing allocation has been dropped.
pub(super) fn same(left: &Region<'_>, right: &Region<'_>, meter: &Meter) -> Result<bool, Refusal> {
    meter.work(1)?;
    if left.atoms.len() != right.atoms.len() {
        return Ok(false);
    }
    let mut pairs = Scratch::new(meter);
    for (a, b) in left.atoms.iter().zip(right.atoms) {
        meter.work(1)?;
        if a.joint != b.joint {
            return Ok(false);
        }
        match (a.occurrence, b.occurrence) {
            (None, None) => {
                meter.work(a.spelling.len().max(b.spelling.len()))?;
                if a.spelling != b.spelling {
                    return Ok(false);
                }
            }
            (Some(a), Some(b)) => {
                if !roles(left, right, a, b, meter)?
                    || !corresponds(&mut pairs, a.binding, b.binding, meter)?
                {
                    return Ok(false);
                }
            }
            (None, Some(_)) | (Some(_), None) => return Ok(false),
        }
    }
    Ok(true)
}

fn roles(
    left: &Region<'_>,
    right: &Region<'_>,
    a: Occurrence<'_>,
    b: Occurrence<'_>,
    meter: &Meter,
) -> Result<bool, Refusal> {
    let first = declared(left, a)?;
    let second = declared(right, b)?;
    meter.work(a.fixed.map_or(0, str::len).max(b.fixed.map_or(0, str::len)))?;
    Ok(first.family == second.family
        && a.event == b.event
        && a.fixed == b.fixed
        && left.bytes.contains(&first.byte) == right.bytes.contains(&second.byte))
}

fn declared<'r>(
    region: &'r Region<'_>,
    occurrence: Occurrence<'_>,
) -> Result<&'r Declaration, Refusal> {
    let declaration = region
        .declarations
        .get(occurrence.binding)
        .ok_or(Refusal::Invariant("fragment occurrence has no declaration"))?;
    if occurrence.event == Event::Declaration && !region.bytes.contains(&declaration.byte) {
        return Err(Refusal::Invariant(
            "fragment declaration event is outside its region",
        ));
    }
    if !matches!(declaration.family, Family::Value | Family::Receiver) && occurrence.fixed.is_none()
    {
        return Err(Refusal::Invariant(
            "fragment control marker has no fixed opcode",
        ));
    }
    Ok(declaration)
}

fn corresponds(
    pairs: &mut Scratch<'_, Pair>,
    left: usize,
    right: usize,
    meter: &Meter,
) -> Result<bool, Refusal> {
    for pair in pairs.iter() {
        meter.work(1)?;
        if pair.left == left || pair.right == right {
            return Ok(pair.left == left && pair.right == right);
        }
    }
    pairs.push(Pair { left, right })?;
    Ok(true)
}

/// Canonical ordered proof identity, separate from body/shim fingerprints.
/// Declaration IDs become first-occurrence ranks, retaining family, event,
/// capture/internal status and fixed roles. No coarse digest enters this key.
pub(super) fn digest(region: &Region<'_>, meter: &Meter) -> Result<[u8; 32], Refusal> {
    let mut hash = blake3::Hasher::new();
    encode(&mut hash, b"raf:embedded:verified:v1\0", meter)?;
    framed(&mut hash, policy::PROFILE, meter)?;
    framed(&mut hash, names::PROFILE, meter)?;
    let mut bindings = Scratch::<Pair>::new(meter);
    for atom in region.atoms {
        meter.work(1)?;
        encode(&mut hash, &[u8::from(atom.joint)], meter)?;
        let Some(occurrence) = atom.occurrence else {
            encode(&mut hash, &[0], meter)?;
            framed(&mut hash, atom.spelling, meter)?;
            continue;
        };
        let declaration = declared(region, occurrence)?;
        let mut rank = None;
        for pair in bindings.iter() {
            meter.work(1)?;
            if pair.left == occurrence.binding {
                rank = Some(pair.right);
                break;
            }
        }
        let rank = if let Some(rank) = rank {
            rank
        } else {
            let rank = bindings.len();
            bindings.push(Pair {
                left: occurrence.binding,
                right: rank,
            })?;
            rank
        };
        let rank = u32::try_from(rank)
            .map_err(|_| Refusal::Invariant("fragment binding rank exceeds u32"))?;
        let family = match declaration.family {
            Family::Value => 0,
            Family::Receiver => 1,
            Family::Label => 2,
            Family::Function => 3,
            Family::Closure => 4,
            Family::Loop => 5,
            Family::Try => 6,
            Family::AsyncFunction => 7,
            Family::AsyncClosure => 8,
            Family::AsyncBlock => 9,
            Family::ConstFunction => 10,
            Family::ConstContext => 11,
        };
        let event = match occurrence.event {
            Event::Declaration => 1,
            Event::Use => 2,
        };
        encode(
            &mut hash,
            &[
                event,
                family,
                u8::from(region.bytes.contains(&declaration.byte)),
            ],
            meter,
        )?;
        encode(&mut hash, &rank.to_le_bytes(), meter)?;
        match occurrence.fixed {
            Some(fixed) => {
                encode(&mut hash, &[1], meter)?;
                framed(&mut hash, fixed, meter)?;
            }
            None => encode(&mut hash, &[0], meter)?,
        }
    }
    Ok(*hash.finalize().as_bytes())
}

fn framed(hash: &mut blake3::Hasher, text: &str, meter: &Meter) -> Result<(), Refusal> {
    let len = u32::try_from(text.len())
        .map_err(|_| Refusal::Invariant("fragment key spelling exceeds u32"))?;
    encode(hash, &len.to_le_bytes(), meter)?;
    encode(hash, text.as_bytes(), meter)
}

pub(super) fn encode(
    hash: &mut blake3::Hasher,
    bytes: &[u8],
    meter: &Meter,
) -> Result<(), Refusal> {
    meter.work(bytes.len())?;
    hash.update(bytes);
    Ok(())
}

#[cfg(test)]
#[path = "proof_tests.rs"]
mod tests;
