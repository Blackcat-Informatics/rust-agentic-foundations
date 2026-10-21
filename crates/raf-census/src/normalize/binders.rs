// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The names a body binds: the only names `rename_locals` renames.
//!
//! The body is a token tree, not a syntax tree, so a binder is found by its
//! keyword and its place: the pattern after `let`, `for` and in a `match` arm;
//! the parameters of a closure and of a local `fn`; the name a local type or
//! `fn` declares; a label or a lifetime; a macro metavariable; and the name of
//! a named macro argument. A name is bound for the whole body, not for a scope.

use std::collections::BTreeSet;

use super::form::{Delim, Kind, Node};
use super::passes::{
    before, ends_operand, is_alone, is_arrow, is_attribute, is_ident, is_joint, is_plain_assign,
    is_punct, leaf,
};
use super::words::{PRIMITIVES, is_keyword};

/// The bound names of `nodes`. A lifetime or a label is stored as `'name`.
pub(crate) fn collect(nodes: &[Node]) -> BTreeSet<String> {
    let mut bound = BTreeSet::new();
    scan(nodes, false, false, &mut bound);
    bound
}

fn name_of(node: Option<&Node>) -> Option<&str> {
    leaf(node)
        .filter(|token| token.kind == Kind::Ident)
        .map(|token| token.text.as_str())
}

/// Scans one delimiter level. `arms` is a `match` block, `macro_args` the
/// arguments of a macro call.
fn scan(nodes: &[Node], arms: bool, macro_args: bool, bound: &mut BTreeSet<String>) {
    if arms {
        scan_arms(nodes, bound);
        return;
    }
    if macro_args {
        named_arguments(nodes, bound);
    }
    let mut match_blocks = Vec::new();
    let mut index = 0;
    while index < nodes.len() {
        match &nodes[index] {
            Node::Group(group) => {
                if !is_attribute(nodes, index) {
                    let call = is_punct(before(nodes, index, 1), '!')
                        && name_of(before(nodes, index, 2)).is_some();
                    scan(&group.children, match_blocks.contains(&index), call, bound);
                }
            }
            Node::Leaf(token) if token.kind == Kind::Ident => {
                keyword(nodes, index, &token.text, &mut match_blocks, bound);
            }
            Node::Leaf(token) if token.kind == Kind::Punct => match token.text.as_str() {
                "|" => {
                    index = pipe(nodes, index, bound);
                    continue;
                }
                "$" => {
                    if let Some(name) = name_of(nodes.get(index + 1))
                        && !is_keyword(name)
                    {
                        bound.insert(name.to_owned());
                    }
                }
                _ => {}
            },
            Node::Leaf(_) => {}
        }
        index += 1;
    }
}

/// What an identifier at `index` binds, by its text.
fn keyword(
    nodes: &[Node],
    index: usize,
    text: &str,
    match_blocks: &mut Vec<usize>,
    bound: &mut BTreeSet<String>,
) {
    if is_joint(before(nodes, index, 1), '\'') {
        if text != "_" && !is_keyword(text) {
            bound.insert(format!("'{text}"));
        }
        return;
    }
    match text {
        "let" => pattern(&nodes[index + 1..pattern_end(nodes, index + 1)], bound),
        "for" if !is_punct(nodes.get(index + 1), '<') => {
            let head = nodes[index + 1..].iter().position(|node| {
                is_ident(Some(node), "in")
                    || is_punct(Some(node), ';')
                    || node
                        .as_group()
                        .is_some_and(|group| group.delim == Delim::Brace)
            });
            if let Some(at) = head
                && is_ident(nodes.get(index + 1 + at), "in")
            {
                pattern(&nodes[index + 1..index + 1 + at], bound);
            }
        }
        "match" => {
            let block = nodes[index + 1..].iter().position(|node| {
                node.as_group()
                    .is_some_and(|group| group.delim == Delim::Brace)
            });
            if let Some(at) = block {
                match_blocks.push(index + 1 + at);
            }
        }
        "fn" => local_fn(nodes, index, bound),
        "struct" | "enum" | "union" | "trait" | "type" => {
            if let Some(name) = name_of(nodes.get(index + 1))
                && name.starts_with(|first: char| first.is_ascii_uppercase())
            {
                bound.insert(name.to_owned());
            }
        }
        _ => {}
    }
}

/// The `fn name(parameters)` at `index`: the name and the parameter patterns.
fn local_fn(nodes: &[Node], index: usize, bound: &mut BTreeSet<String>) {
    let Some(name) = name_of(nodes.get(index + 1)).filter(|name| !is_keyword(name)) else {
        return;
    };
    bound.insert(name.to_owned());
    let mut depth = 0_usize;
    for (at, node) in nodes.iter().enumerate().skip(index + 2) {
        if is_punct(Some(node), '<') {
            depth += 1;
        } else if is_punct(Some(node), '>') && !is_joint(before(nodes, at, 1), '-') {
            depth = depth.saturating_sub(1);
        } else if is_punct(Some(node), ';') {
            return;
        } else if depth == 0
            && let Some(group) = node.as_group()
        {
            if group.delim == Delim::Paren {
                parameters(&group.children, bound);
            }
            return;
        }
    }
}

/// A `|` at `index`: a closure's parameters, or a binary or. Returns the index
/// to continue from.
fn pipe(nodes: &[Node], index: usize, bound: &mut BTreeSet<String>) -> usize {
    let operator = ends_operand(before(nodes, index, 1));
    if is_joint(nodes.get(index), '|') && is_punct(nodes.get(index + 1), '|') {
        // `||`: a closure with no parameter, or the logical or.
        return index + 2;
    }
    if operator {
        return index + 1;
    }
    let close = nodes[index + 1..]
        .iter()
        .position(|node| is_punct(Some(node), '|'));
    match close {
        Some(at) => {
            parameters(&nodes[index + 1..index + 1 + at], bound);
            index + 2 + at
        }
        None => index + 1,
    }
}

/// The parameter list `pattern: Type, pattern: Type` of a closure or a `fn`.
fn parameters(nodes: &[Node], bound: &mut BTreeSet<String>) {
    let mut start = 0;
    let mut depth = 0_isize;
    let mut typed = false;
    for (index, node) in nodes.iter().enumerate() {
        if is_annotation_colon(nodes, index) {
            typed = true;
            pattern(&nodes[start..index], bound);
        } else if typed && is_punct(Some(node), '<') {
            depth += 1;
        } else if typed
            && is_punct(Some(node), '>')
            && !is_joint(before(nodes, index, 1), '-')
            && !is_joint(before(nodes, index, 1), '=')
        {
            depth -= 1;
        } else if is_punct(Some(node), ',') && depth <= 0 {
            if !typed {
                pattern(&nodes[start..index], bound);
            }
            start = index + 1;
            depth = 0;
            typed = false;
        }
    }
    if !typed {
        pattern(&nodes[start..], bound);
    }
}

/// Whether `nodes[index]` is the `:` that starts a type annotation, not half of
/// a `::`.
fn is_annotation_colon(nodes: &[Node], index: usize) -> bool {
    is_alone(nodes.get(index), ':') && !is_joint(before(nodes, index, 1), ':')
}

/// The end of the pattern that starts at `from`: the `=`, the `;` or the `:`
/// that closes it.
fn pattern_end(nodes: &[Node], from: usize) -> usize {
    (from..nodes.len())
        .find(|index| {
            is_punct(nodes.get(*index), ';')
                || is_plain_assign(nodes, *index)
                || is_annotation_colon(nodes, *index)
        })
        .unwrap_or(nodes.len())
}

/// The names bound by `, name = value` arguments of a macro call.
fn named_arguments(nodes: &[Node], bound: &mut BTreeSet<String>) {
    for index in 1..nodes.len() {
        if let Some(name) = name_of(nodes.get(index))
            && !is_keyword(name)
            && is_punct(before(nodes, index, 1), ',')
            && is_plain_assign(nodes, index + 1)
        {
            bound.insert(name.to_owned());
        }
    }
}

/// The arms of a `match` block: each pattern binds, and each guard and body is
/// scanned as an expression.
fn scan_arms(nodes: &[Node], bound: &mut BTreeSet<String>) {
    let mut at = 0;
    while at < nodes.len() {
        let stop = (at..nodes.len())
            .find(|index| is_arrow(nodes, *index) || is_ident(nodes.get(*index), "if"))
            .unwrap_or(nodes.len());
        pattern(&nodes[at..stop], bound);
        let mut arrow = stop;
        if is_ident(nodes.get(stop), "if") {
            arrow = (stop..nodes.len())
                .find(|index| is_arrow(nodes, *index))
                .unwrap_or(nodes.len());
            scan(&nodes[stop + 1..arrow], false, false, bound);
        }
        if arrow >= nodes.len() {
            return;
        }
        let body = arrow + 2;
        let block = nodes
            .get(body)
            .and_then(Node::as_group)
            .is_some_and(|group| group.delim == Delim::Brace);
        let end = if block {
            body + 1
        } else {
            (body..nodes.len())
                .find(|index| is_punct(nodes.get(*index), ','))
                .unwrap_or(nodes.len())
        };
        scan(
            &nodes[body.min(nodes.len())..end.min(nodes.len())],
            false,
            false,
            bound,
        );
        at = if is_punct(nodes.get(end), ',') {
            end + 1
        } else {
            end
        };
    }
}

/// The names a pattern binds.
fn pattern(nodes: &[Node], bound: &mut BTreeSet<String>) {
    pattern_level(nodes, false, bound);
}

fn pattern_level(nodes: &[Node], braces: bool, bound: &mut BTreeSet<String>) {
    for (index, node) in nodes.iter().enumerate() {
        match node {
            Node::Group(group) => {
                if !is_attribute(nodes, index) {
                    pattern_level(&group.children, group.delim == Delim::Brace, bound);
                }
            }
            Node::Leaf(token) if token.kind == Kind::Ident => {
                let text = token.text.as_str();
                let path = (is_punct(before(nodes, index, 1), ':')
                    && is_punct(before(nodes, index, 2), ':'))
                    || (is_punct(nodes.get(index + 1), ':') && is_punct(nodes.get(index + 2), ':'));
                let constructor = nodes
                    .get(index + 1)
                    .is_some_and(|next| next.as_group().is_some() || is_punct(Some(next), '!'));
                let field = braces && is_alone(nodes.get(index + 1), ':');
                let item = text == "_"
                    || text.starts_with(|first: char| first.is_ascii_uppercase() || first == '$')
                    || is_keyword(text)
                    || PRIMITIVES.contains(&text)
                    || is_joint(before(nodes, index, 1), '\'');
                if !(path || constructor || field || item) {
                    bound.insert(text.to_owned());
                }
            }
            Node::Leaf(_) => {}
        }
    }
}
