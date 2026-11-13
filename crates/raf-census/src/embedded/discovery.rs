// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Deterministic coarse fingerprints; every selected seed still needs proof.
//!
//! Eight-token grams and four-gram windows select the rightmost minimum full
//! BLAKE3 digest. Identical contiguous coarse sequences of eleven atoms share
//! a selected seed. This says nothing about reordered statements, binding
//! correspondence, syntax boundaries or behavior. No digest is a finding.

use std::cmp::Reverse;

use purrdf_hash::blake3;

use super::names;
use super::proof::{Atom, Event, encode as update};
use super::resource::{Meter, Refusal, Scratch};

pub(super) const GRAM: usize = 8;
pub(super) const WINDOW: usize = 4;
const DOMAIN: &[u8] = b"raf:embedded:coarse:v1\0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Seed {
    /// First atom of the selected gram, within its original source sequence.
    pub(super) at: usize,
    pub(super) digest: [u8; 32],
}

/// Coarse names retain occurrence roles, but never whole-body binding numbers.
pub(super) fn discover<'m>(
    atoms: &[Atom<'_>],
    meter: &'m Meter,
) -> Result<Scratch<'m, Seed>, Refusal> {
    select(atoms.windows(GRAM).map(|gram| digest(gram, meter)), meter)
}

/// Exact equality of the coarse encoding, independently of its digest.
/// Binding identities and declaration/capture families still require proof.
pub(super) fn same_atom(a: Atom<'_>, b: Atom<'_>, meter: &Meter) -> Result<bool, Refusal> {
    meter.work(1)?;
    if a.joint != b.joint {
        return Ok(false);
    }
    match (a.occurrence, b.occurrence) {
        (None, None) => {
            meter.work(a.spelling.len().max(b.spelling.len()))?;
            Ok(a.spelling == b.spelling)
        }
        (Some(a), Some(b)) => {
            meter.work(a.fixed.map_or(0, str::len).max(b.fixed.map_or(0, str::len)))?;
            Ok(a.event == b.event && a.fixed == b.fixed)
        }
        _ => Ok(false),
    }
}

fn digest(gram: &[Atom<'_>], meter: &Meter) -> Result<[u8; 32], Refusal> {
    let mut hash = blake3::Hasher::new();
    update(&mut hash, DOMAIN, meter)?;
    text(&mut hash, names::PROFILE, meter)?;
    for atom in gram {
        update(&mut hash, &[u8::from(atom.joint)], meter)?;
        match atom.occurrence {
            None => {
                update(&mut hash, &[0], meter)?;
                text(&mut hash, atom.spelling, meter)?;
            }
            Some(occurrence) => {
                let role = match occurrence.event {
                    Event::Declaration => 1,
                    Event::Use => 2,
                };
                update(&mut hash, &[role], meter)?;
                match occurrence.fixed {
                    Some(fixed) => {
                        update(&mut hash, &[1], meter)?;
                        text(&mut hash, fixed, meter)?;
                    }
                    None => update(&mut hash, &[0], meter)?,
                }
            }
        }
    }
    Ok(*hash.finalize().as_bytes())
}

fn text(hash: &mut blake3::Hasher, value: &str, meter: &Meter) -> Result<(), Refusal> {
    let length = u32::try_from(value.len())
        .map_err(|_| Refusal::Invariant("fragment token exceeds the u32 encoding limit"))?;
    update(hash, &length.to_le_bytes(), meter)?;
    update(hash, value.as_bytes(), meter)
}

fn select(
    digests: impl Iterator<Item = Result<[u8; 32], Refusal>>,
    meter: &Meter,
) -> Result<Scratch<'_, Seed>, Refusal> {
    let mut window = [Seed {
        at: 0,
        digest: [0; 32],
    }; WINDOW];
    let mut previous = None;
    let mut found = Scratch::new(meter);
    for (at, digest) in digests.enumerate() {
        let digest = digest?;
        meter.work(1)?;
        window[at % WINDOW] = Seed { at, digest };
        if at < WINDOW - 1 {
            continue;
        }
        meter.work(WINDOW)?;
        let selected = *window
            .iter()
            .min_by_key(|seed| (seed.digest, Reverse(seed.at)))
            .expect("a complete fixed window is nonempty");
        if previous != Some(selected.at) {
            found.push(selected)?;
            previous = Some(selected.at);
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedded::proof::{Declaration, Family, Occurrence, Region, same};

    #[test]
    fn rightmost_ties_and_persistent_minima_keep_original_positions_without_duplicate_seeds() {
        let meter = Meter::new(10_000, 10_000);
        let equal = select((0..7).map(|_| Ok([7; 32])), &meter).unwrap();
        assert_eq!(
            equal.iter().map(|seed| seed.at).collect::<Vec<_>>(),
            [3, 4, 5, 6]
        );
        let varied = select(
            [9, 4, 2, 3, 7, 1, 5]
                .into_iter()
                .map(|value| Ok([value; 32])),
            &meter,
        )
        .unwrap();
        assert_eq!(
            varied.iter().map(|seed| seed.at).collect::<Vec<_>>(),
            [2, 5]
        );
        let short = select((0..WINDOW - 1).map(|_| Ok([0; 32])), &meter).unwrap();
        assert_eq!(short.iter().count(), 0);
    }

    #[test]
    fn an_embedded_renamed_sequence_keeps_a_shared_seed_after_unrelated_prefix_atoms() {
        let source = (0..32)
            .map(|index| Atom {
                joint: false,
                spelling: "left variable spelling",
                occurrence: Some(Occurrence {
                    binding: index % 3,
                    event: Event::Use,
                    fixed: None,
                }),
            })
            .collect::<Vec<_>>();
        let mut other = vec![
            Atom {
                joint: false,
                spelling: "unrelated",
                occurrence: None
            };
            3
        ];
        other.extend(source.iter().map(|atom| Atom {
            joint: false,
            spelling: "different variable spelling",
            occurrence: Some(Occurrence {
                binding: atom.occurrence.unwrap().binding + 17,
                ..atom.occurrence.unwrap()
            }),
        }));
        let meter = Meter::new(100_000, 100_000);
        let left = discover(&source, &meter).unwrap();
        let right = discover(&other, &meter).unwrap();
        assert!(left.iter().any(|a| {
            right
                .iter()
                .any(|b| a.digest == b.digest && b.at == a.at + 3)
        }));
        assert_eq!(
            discover(&source[..GRAM - 1], &meter)
                .unwrap()
                .iter()
                .count(),
            0
        );
    }

    #[test]
    fn colliding_candidate_hashes_do_not_establish_a_binding_correspondence() {
        let left = [Atom {
            joint: false,
            spelling: "+",
            occurrence: None,
        }; GRAM + WINDOW - 1];
        let right = [Atom {
            joint: false,
            spelling: "-",
            occurrence: None,
        }; GRAM + WINDOW - 1];
        let meter = Meter::new(10_000, 10_000);
        let first = select(left.windows(GRAM).map(|_| Ok([0; 32])), &meter).unwrap();
        let second = select(right.windows(GRAM).map(|_| Ok([0; 32])), &meter).unwrap();
        assert_eq!(first.iter().next(), second.iter().next());
        let declarations = [Declaration {
            byte: 0,
            family: Family::Value,
        }];
        assert!(
            !same(
                &Region {
                    bytes: 0..100,
                    atoms: &left,
                    declarations: &declarations
                },
                &Region {
                    bytes: 0..100,
                    atoms: &right,
                    declarations: &declarations
                },
                &meter,
            )
            .unwrap()
        );
    }

    #[test]
    fn discovery_refuses_exhaustion_and_preserves_the_first_input_cause() {
        let atoms = [Atom {
            joint: false,
            spelling: "fixed",
            occurrence: None,
        }; GRAM + WINDOW - 1];
        let measured = Meter::new(100_000, 100_000);
        drop(discover(&atoms, &measured).unwrap());
        let (work, bytes) = measured.usage();
        let exact = Meter::new(work, bytes);
        assert_eq!(discover(&atoms, &exact).unwrap().iter().count(), 1);
        let short_work = Meter::new(work - 1, bytes);
        assert!(matches!(
            discover(&atoms, &short_work),
            Err(Refusal::Work { .. })
        ));
        let short_memory = Meter::new(work, bytes - 1);
        assert!(matches!(
            discover(&atoms, &short_memory),
            Err(Refusal::Memory { .. })
        ));
        let original = Refusal::Invariant("source classification refused");
        let exhausted = Meter::new(0, 0);
        assert!(
            matches!(select(std::iter::once(Err(original)), &exhausted), Err(cause) if cause == original)
        );
    }
}
