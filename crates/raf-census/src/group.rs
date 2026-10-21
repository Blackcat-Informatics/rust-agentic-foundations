// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Grouping: the units that share a fingerprint, the units that hold a
//! hexadecimal digit table, and the large integer constants that more than one
//! package spells.
//!
//! A unit with a shim form is never body-grouped: a forwarder is not compared
//! by structure. It joins the shim group of its `shim_key`, when it has one.

use std::collections::{BTreeMap, BTreeSet};

use crate::normalize::TableKind;
use crate::units::Unit;
use crate::{Error, Fingerprint};

/// The smallest integer constant that is a large one: values above `0xffff`.
pub(crate) const LARGE_CONSTANT: u128 = 0xffff;

/// Where a unit is, as a group, a table or a constant reports it.
///
/// Members order by symbol, then file, then line.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Member {
    /// The symbol of the unit.
    pub symbol: String,
    /// The file, relative to the workspace root.
    pub file: String,
    /// The 1-based line of the unit's name.
    pub line: usize,
    /// The byte offset where the source item starts.
    pub start_byte: usize,
    /// The package of the unit.
    pub package: String,
}

impl Member {
    fn of(unit: &Unit) -> Self {
        Self {
            symbol: unit.symbol.clone(),
            file: unit.file.clone(),
            line: unit.line,
            start_byte: unit.start_byte,
            package: unit.package.clone(),
        }
    }
}

/// What a group is keyed by.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum GroupKind {
    /// Bodies with one value form.
    Body,
    /// Forwarders with one shim form.
    Shim,
}

/// Two or more units that share one fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Group {
    /// Whether the units share a body or a shim.
    pub kind: GroupKind,
    /// The fingerprint they share.
    pub fingerprint: Fingerprint,
    /// The tokens of the value form of the shared body.
    pub tokens: usize,
    /// The units, ordered by symbol, then file, then line.
    pub members: Vec<Member>,
}

impl Group {
    /// The symbols of the members, in member order.
    pub fn member_symbols(&self) -> impl Iterator<Item = &str> {
        self.members.iter().map(|member| member.symbol.as_str())
    }
}

/// A unit that holds a hexadecimal digit table.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct HexTable {
    /// The unit.
    pub member: Member,
    /// The case of the table.
    pub table: TableKind,
}

/// An integer constant above `0xffff` that units of two or more packages hold.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SharedConstant {
    /// The value.
    pub value: u128,
    /// The number of packages that hold it.
    pub packages: usize,
    /// The units that hold it, ordered by symbol, then file, then line.
    pub members: Vec<Member>,
}

/// A unit offered to the group builder.
pub(crate) struct Candidate<'a> {
    pub(crate) kind: GroupKind,
    pub(crate) member: Member,
    pub(crate) tokens: usize,
    pub(crate) fingerprint: &'a Fingerprint,
    pub(crate) digest: &'a [u8; 32],
}

/// Every group of two or more candidates, ordered by member count descending,
/// then tokens descending, then fingerprint ascending.
///
/// The full digest is kept for every candidate, singletons included.
///
/// # Errors
///
/// [`Error::Collision`] when two candidates have one fingerprint and different
/// digests.
pub(crate) fn build_groups<'a>(
    candidates: impl IntoIterator<Item = Candidate<'a>>,
) -> Result<Vec<Group>, Error> {
    let mut by_print: BTreeMap<(GroupKind, Fingerprint), (&[u8; 32], Group)> = BTreeMap::new();
    for candidate in candidates {
        let key = (candidate.kind, candidate.fingerprint.clone());
        if let Some((holder, group)) = by_print.get_mut(&key) {
            if *holder != candidate.digest {
                return Err(Error::Collision {
                    fingerprint: key.1.to_string(),
                    first: group.members[0].symbol.clone(),
                    second: candidate.member.symbol,
                });
            }
            group.tokens = group.tokens.max(candidate.tokens);
            group.members.push(candidate.member);
            continue;
        }
        let group = Group {
            kind: candidate.kind,
            fingerprint: key.1.clone(),
            tokens: candidate.tokens,
            members: vec![candidate.member],
        };
        by_print.insert(key, (candidate.digest, group));
    }
    let mut groups: Vec<Group> = by_print
        .into_values()
        .map(|(_, group)| group)
        .filter(|group| group.members.len() >= 2)
        .collect();
    for group in &mut groups {
        group.members.sort();
    }
    groups.sort_by(|a, b| {
        b.members
            .len()
            .cmp(&a.members.len())
            .then_with(|| b.tokens.cmp(&a.tokens))
            .then_with(|| a.fingerprint.cmp(&b.fingerprint))
            .then_with(|| a.kind.cmp(&b.kind))
    });
    Ok(groups)
}

/// Everything the census derives from its units by comparing them.
#[derive(Debug)]
pub(crate) struct Grouping {
    pub(crate) groups: Vec<Group>,
    pub(crate) hex_tables: Vec<HexTable>,
    pub(crate) shared_constants: Vec<SharedConstant>,
}

/// Groups `units`, finds their hex tables and their shared large constants.
///
/// # Errors
///
/// [`Error::Collision`], see [`build_groups`].
pub(crate) fn group(units: &[Unit]) -> Result<Grouping, Error> {
    let mut candidates = Vec::new();
    for unit in units {
        let Some(print) = &unit.print else { continue };
        if print.shim.is_some() {
            // A forwarder is never compared by structure; it groups by its
            // shim, and only when its shim key allows.
            if let (Some(key), Some(hashed)) = (&unit.shim_key, &print.shim) {
                candidates.push(Candidate {
                    kind: GroupKind::Shim,
                    member: Member::of(unit),
                    tokens: print.tokens,
                    fingerprint: key,
                    digest: &hashed.digest,
                });
            }
        } else if let Some(hashed) = &print.body {
            candidates.push(Candidate {
                kind: GroupKind::Body,
                member: Member::of(unit),
                tokens: print.tokens,
                fingerprint: &hashed.fingerprint,
                digest: &hashed.digest,
            });
        }
    }
    let groups = build_groups(candidates)?;

    let mut hex_tables: Vec<HexTable> = units
        .iter()
        .flat_map(|unit| {
            unit.tables.iter().map(|&table| HexTable {
                member: Member::of(unit),
                table,
            })
        })
        .collect();
    hex_tables.sort();

    let mut by_value: BTreeMap<u128, Vec<&Unit>> = BTreeMap::new();
    for unit in units {
        for &value in unit.constants.range(LARGE_CONSTANT + 1..) {
            by_value.entry(value).or_default().push(unit);
        }
    }
    let shared_constants = by_value
        .into_iter()
        .filter_map(|(value, holders)| {
            let packages: BTreeSet<&str> =
                holders.iter().map(|unit| unit.package.as_str()).collect();
            (packages.len() >= 2).then(|| {
                let mut members: Vec<Member> = holders.into_iter().map(Member::of).collect();
                members.sort();
                SharedConstant {
                    value,
                    packages: packages.len(),
                    members,
                }
            })
        })
        .collect();
    Ok(Grouping {
        groups,
        hex_tables,
        shared_constants,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::Hashed;

    fn hashed(id: u64, tail: u8) -> Hashed {
        let mut digest = [0_u8; 32];
        digest[..8].copy_from_slice(&id.to_be_bytes());
        digest[31] = tail;
        Hashed {
            fingerprint: Fingerprint::Body(id),
            digest,
        }
    }

    fn candidate<'a>(symbol: &str, hashed: &'a Hashed) -> Candidate<'a> {
        Candidate {
            kind: GroupKind::Body,
            member: Member {
                start_byte: 0,
                symbol: symbol.to_owned(),
                file: "src/lib.rs".to_owned(),
                line: 1,
                package: "demo".to_owned(),
            },
            tokens: 30,
            fingerprint: &hashed.fingerprint,
            digest: &hashed.digest,
        }
    }

    #[test]
    fn one_fingerprint_with_two_digests_is_a_collision_beside_equal_digests() {
        let a = hashed(7, 0);
        let same = hashed(7, 0);
        let forged = hashed(7, 1);
        let groups = build_groups([candidate("a::f", &a), candidate("b::f", &same)])
            .expect("equal digests group");
        assert_eq!(groups.len(), 1);
        match build_groups([candidate("a::f", &a), candidate("b::g", &forged)]) {
            Err(Error::Collision {
                fingerprint,
                first,
                second,
            }) => {
                assert_eq!(fingerprint, Fingerprint::Body(7).to_string());
                assert_eq!((first.as_str(), second.as_str()), ("a::f", "b::g"));
            }
            other => panic!("expected a collision, found {other:?}"),
        }
    }

    #[test]
    fn a_collision_is_found_even_when_a_third_unit_stands_alone() {
        let a = hashed(9, 0);
        let b = hashed(9, 2);
        let c = hashed(10, 0);
        let result = build_groups([candidate("a", &a), candidate("c", &c), candidate("b", &b)]);
        assert!(matches!(result, Err(Error::Collision { .. })), "{result:?}");
    }

    #[test]
    fn members_order_by_symbol_then_file_then_line() {
        let h = hashed(5, 0);
        let at = |symbol: &str, file: &str, line: usize| {
            let mut c = candidate(symbol, &h);
            c.member.file = file.to_owned();
            c.member.line = line;
            c
        };
        let groups = build_groups([
            at("b", "x.rs", 1),
            at("a", "z.rs", 1),
            at("a", "y.rs", 9),
            at("a", "y.rs", 2),
        ])
        .expect("one digest");
        let order: Vec<(&str, &str, usize)> = groups[0]
            .members
            .iter()
            .map(|m| (m.symbol.as_str(), m.file.as_str(), m.line))
            .collect();
        assert_eq!(
            order,
            [
                ("a", "y.rs", 2),
                ("a", "y.rs", 9),
                ("a", "z.rs", 1),
                ("b", "x.rs", 1)
            ]
        );
    }
}
