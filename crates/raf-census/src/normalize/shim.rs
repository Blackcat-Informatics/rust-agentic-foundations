// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The shim candidate predicate.

use super::form::{Form, Kind, Node};

/// Whether the body is a thin forwarder: one expression (no `let`, no
/// statement boundary before the tail), at least one call, and no `self`, since
/// a delegation on `self` is bound to its own type and cannot collapse with
/// another type's.
///
/// The predicate reads the verbatim form. Whether the forwarder is also long
/// enough, and whether its signature admits a shim key, is decided elsewhere.
pub(crate) fn candidate(form: &Form) -> bool {
    let nodes = &form.nodes;
    let mut statements = 0;
    for (index, node) in nodes.iter().enumerate() {
        if let Node::Leaf(token) = node {
            match token.kind {
                Kind::Punct if token.text == ";" && index + 1 < nodes.len() => statements += 1,
                Kind::Ident if token.text == "let" => return false,
                _ => {}
            }
        }
    }
    statements == 0 && has_call(nodes) && !mentions_self(nodes)
}

/// Whether a name or a macro bang is directly followed by a parenthesised
/// group, anywhere in `nodes`. A turbofish belongs to its name: in
/// `f::<T>(x)` and `x.m::<T>()` the call follows the closing `>`.
fn has_call(nodes: &[Node]) -> bool {
    let mut previous_is_name = false;
    let mut index = 0;
    while index < nodes.len() {
        match &nodes[index] {
            Node::Group(group) => {
                if previous_is_name && group.delim == super::form::Delim::Paren {
                    return true;
                }
                if has_call(&group.children) {
                    return true;
                }
                previous_is_name = false;
            }
            Node::Leaf(token) => {
                if previous_is_name && let Some(end) = turbofish_end(nodes, index) {
                    if nodes[index..end].iter().any(|node| {
                        node.as_group()
                            .is_some_and(|group| has_call(&group.children))
                    }) {
                        return true;
                    }
                    index = end;
                    continue;
                }
                previous_is_name = match token.kind {
                    Kind::Ident => true,
                    Kind::Punct => token.text == "!",
                    Kind::Literal => false,
                };
            }
        }
        index += 1;
    }
    false
}

/// When `nodes[start..]` begins a turbofish, `::<...>` with balanced angle
/// brackets, the index just past its closing `>`.
fn turbofish_end(nodes: &[Node], start: usize) -> Option<usize> {
    let punct = |index: usize, wanted: &str| {
        nodes
            .get(index)
            .and_then(Node::as_leaf)
            .is_some_and(|leaf| leaf.kind == Kind::Punct && leaf.text == wanted)
    };
    if !(punct(start, ":") && punct(start + 1, ":") && punct(start + 2, "<")) {
        return None;
    }
    let mut depth = 0_usize;
    for index in start + 2..nodes.len() {
        if punct(index, "<") {
            depth += 1;
        } else if punct(index, ">") {
            // The `>` of an `->` inside the arguments closes nothing.
            let arrow = index > 0
                && nodes
                    .get(index - 1)
                    .and_then(Node::as_leaf)
                    .is_some_and(|leaf| leaf.text == "-" && leaf.joint);
            if !arrow {
                depth -= 1;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
        }
    }
    None
}

fn mentions_self(nodes: &[Node]) -> bool {
    nodes.iter().any(|node| match node {
        Node::Group(group) => mentions_self(&group.children),
        Node::Leaf(token) => token.kind == Kind::Ident && token.text == "self",
    })
}
