// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! File tokens with each implementation normalized in its own body context.
//! Module-level declaration names remain literal; only body-local declarations
//! follow the existing local-name pass. Matcher text has the same body context
//! as an independent normal-form slice.

use std::ops::Range;

use purrdf_jsonschema::ecma::MatchLimits;
use raf_source::{SourceFile, Workspace};

use crate::normalize::{Node, leading_forms};
use crate::{Error, Form, Slice};

use super::{charge, failure};

pub(super) fn normalized(
    workspace: &Workspace,
    file: &SourceFile,
    owners: &super::super::owners::Owners<'_>,
    limits: &mut MatchLimits,
) -> Result<Form, Error> {
    let label = file.path.to_string();
    let mut form = Form::lex(&label, 1, &file.text)?;
    let mut regions = Vec::new();
    for (path, offset, ids) in workspace.items_by_source() {
        if path != &file.path {
            continue;
        }
        let Some(unit) = owners.get(&(path.clone(), offset)) else {
            continue;
        };
        let item = ids
            .first()
            .and_then(|id| workspace.item(*id))
            .ok_or_else(|| failure("tokens", "a physical census item has no source definition"))?;
        let source = crate::units::slice_source(workspace, item, unit.kind)?.ok_or_else(|| {
            failure(
                "tokens",
                "a physical census unit has no body or initializer",
            )
        })?;
        regions.push((source.offset..source.offset + source.text.len(), source));
    }
    // An outer implementation's normal form already includes its nested items.
    regions.sort_by(|(a, _), (b, _)| a.start.cmp(&b.start).then(b.end.cmp(&a.end)));
    let mut covered = 0;
    let mut replacements = Vec::new();
    for (range, source) in regions {
        if range.start < covered {
            continue;
        }
        covered = range.end;
        let slice = Slice::new(&label, source.first_line, source.text).params(source.params);
        let [_, _, mut normalized] = leading_forms(&slice)?;
        shift(&mut normalized.nodes, source.offset, limits)?;
        replacements.push((range, normalized.nodes));
    }
    let mut replacements = replacements.into_iter().peekable();
    overlay(&mut form.nodes, &mut replacements, limits)?;
    if replacements.any(|(_, nodes)| !nodes.is_empty()) {
        return Err(failure(
            "tokens",
            "a normalized body has no matching file token range",
        ));
    }
    Ok(form)
}

// Both walks are bounded by the lexer's MAX_NESTING before they start. They
// operate on the existing Rust token representation, not another parser.
fn shift(nodes: &mut [Node], offset: usize, limits: &mut MatchLimits) -> Result<(), Error> {
    for node in nodes {
        charge("tokens", limits, 1)?;
        match node {
            Node::Leaf(leaf) => {
                leaf.place.at.start += offset;
                leaf.place.at.end += offset;
            }
            Node::Group(group) => {
                group.open.at.start += offset;
                group.open.at.end += offset;
                group.close.at.start += offset;
                group.close.at.end += offset;
                shift(&mut group.children, offset, limits)?;
            }
        }
    }
    Ok(())
}

fn extent(node: &Node) -> Range<usize> {
    match node {
        Node::Leaf(leaf) => leaf.place.at.clone(),
        Node::Group(group) => group.open.at.start..group.close.at.end,
    }
}

type Replacements = std::iter::Peekable<std::vec::IntoIter<(Range<usize>, Vec<Node>)>>;

fn overlay(
    nodes: &mut Vec<Node>,
    replacements: &mut Replacements,
    limits: &mut MatchLimits,
) -> Result<(), Error> {
    let mut input = std::mem::take(nodes).into_iter().peekable();
    while let Some(mut node) = input.next() {
        charge("tokens", limits, 1)?;
        let span = extent(&node);
        while replacements
            .peek()
            .is_some_and(|(range, _)| range.end <= span.start)
        {
            let (_, replacement) = replacements.next().expect("peeked replacement");
            if !replacement.is_empty() {
                return Err(failure(
                    "tokens",
                    "a normalized body has no matching file token range",
                ));
            }
        }
        let Some((range, _)) = replacements.peek() else {
            nodes.push(node);
            nodes.extend(input);
            break;
        };
        if span.start >= range.start && span.end <= range.end {
            let (range, replacement) = replacements.next().expect("peeked replacement");
            while input.peek().is_some_and(|node| {
                let span = extent(node);
                span.start >= range.start && span.end <= range.end
            }) {
                charge("tokens", limits, 1)?;
                input.next();
            }
            nodes.extend(replacement);
        } else if span.start < range.start && span.end > range.end {
            if let Node::Group(group) = &mut node {
                overlay(&mut group.children, replacements, limits)?;
            } else {
                return Err(failure(
                    "tokens",
                    "a normalized body cuts through a source token",
                ));
            }
            nodes.push(node);
        } else if span.end <= range.start {
            nodes.push(node);
        } else {
            return Err(failure(
                "tokens",
                "a normalized body partially overlaps a source token",
            ));
        }
    }
    Ok(())
}
