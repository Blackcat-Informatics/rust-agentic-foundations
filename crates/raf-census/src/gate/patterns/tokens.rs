// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Balanced token matching over the existing normal-form nodes.

use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::walk::WorkList;

use crate::normalize::{Kind, Node};
use crate::{Error, Form};

use super::{charge, failure, push};

/// Search each source position once; a wildcard consumes one node, never a run.
pub(super) fn find(
    id: &str,
    pattern: &Form,
    source: &Form,
    limits: &mut MatchLimits,
) -> Result<Vec<(usize, usize)>, Error> {
    if pattern.is_empty() {
        return Err(failure(id, "a tokens matcher must not be empty"));
    }
    let mut found = Vec::new();
    let mut pending = WorkList::<_, 16>::new();
    push(id, &mut pending, (source.nodes.as_slice(), 0), 0, limits)?;
    while let Some((nodes, at)) = pending.pop() {
        let Some(node) = nodes.get(at) else {
            continue;
        };
        charge(id, limits, 1)?;
        if matches(id, &pattern.nodes, &nodes[at..], pending.len(), limits)? {
            let place = match node {
                Node::Leaf(leaf) => &leaf.place,
                Node::Group(group) => &group.open,
            };
            if found.len() >= limits.states {
                return Err(failure(id, "matching hit-state limit exhausted"));
            }
            found.push((place.at.start, place.line));
        }
        push(id, &mut pending, (nodes, at + 1), 0, limits)?;
        if let Node::Group(group) = node {
            push(id, &mut pending, (&group.children, 0), 0, limits)?;
        }
    }
    found.sort_unstable();
    found.dedup();
    Ok(found)
}

fn matches(
    id: &str,
    pattern: &[Node],
    source: &[Node],
    occupied: usize,
    limits: &mut MatchLimits,
) -> Result<bool, Error> {
    let mut pending = WorkList::<_, 16>::new();
    push(id, &mut pending, (pattern, source, false), occupied, limits)?;
    while let Some((pattern, source, whole)) = pending.pop() {
        let mut left = 0;
        let mut right = 0;
        while left < pattern.len() {
            charge(id, limits, 1)?;
            let Some(actual) = source.get(right) else {
                return Ok(false);
            };
            if wildcard(&pattern[left..]) {
                left += 2;
            } else {
                match (&pattern[left], actual) {
                    (Node::Leaf(expected), Node::Leaf(actual)) => {
                        charge(
                            id,
                            limits,
                            u64::try_from(expected.text.len().max(actual.text.len()))
                                .unwrap_or(u64::MAX),
                        )?;
                        if expected.kind != actual.kind
                            || expected.text != actual.text
                            || expected.joint != actual.joint
                        {
                            return Ok(false);
                        }
                    }
                    (Node::Group(expected), Node::Group(actual))
                        if expected.delim == actual.delim =>
                    {
                        push(
                            id,
                            &mut pending,
                            (
                                expected.children.as_slice(),
                                actual.children.as_slice(),
                                true,
                            ),
                            occupied,
                            limits,
                        )?;
                    }
                    _ => return Ok(false),
                }
                left += 1;
            }
            right += 1;
        }
        if whole && right != source.len() {
            return Ok(false);
        }
    }
    Ok(true)
}

fn wildcard(nodes: &[Node]) -> bool {
    matches!(nodes, [Node::Leaf(dollar), Node::Leaf(name), ..]
        if dollar.kind == Kind::Punct && dollar.text == "$"
            && name.kind == Kind::Ident && name.text == "_")
}
