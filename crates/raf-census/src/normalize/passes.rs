// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The four passes of the tower. Each is a pure function from one form to the
//! next and keeps the source place of every token it keeps.

use std::collections::BTreeMap;

use super::form::{Delim, Form, Group, Kind, Leaf, Node};
use super::literal;
use super::words::{KEYWORDS, PRIMITIVES, VALUE_ADAPTERS, is_keyword};

/// The ids of the passes, in the order the tower applies them.
pub const PASSES: [&str; 4] = [
    "rename_params",
    "rename_locals",
    "kind_literals",
    "erase_held",
];

fn leaf(node: Option<&Node>) -> Option<&Leaf> {
    node.and_then(Node::as_leaf)
}

fn is_punct(node: Option<&Node>, wanted: char) -> bool {
    leaf(node).is_some_and(|leaf| leaf.kind == Kind::Punct && leaf.text.starts_with(wanted))
}

fn is_joint(node: Option<&Node>, wanted: char) -> bool {
    is_punct(node, wanted) && leaf(node).is_some_and(|leaf| leaf.joint)
}

fn is_alone(node: Option<&Node>, wanted: char) -> bool {
    is_punct(node, wanted) && leaf(node).is_some_and(|leaf| !leaf.joint)
}

fn is_ident(node: Option<&Node>, wanted: &str) -> bool {
    leaf(node).is_some_and(|leaf| leaf.kind == Kind::Ident && leaf.text == wanted)
}

fn is_paren(node: Option<&Node>) -> bool {
    node.and_then(Node::as_group)
        .is_some_and(|group| group.delim == Delim::Paren)
}

fn before(nodes: &[Node], index: usize, distance: usize) -> Option<&Node> {
    index.checked_sub(distance).and_then(|at| nodes.get(at))
}

/// Whether the group at `index` is the bracket of an attribute, `#[..]` or
/// `#![..]`; a doc comment in a body lexes to one. Its names are the
/// attribute's own, never locals.
fn is_attribute(nodes: &[Node], index: usize) -> bool {
    let Some(group) = nodes.get(index).and_then(Node::as_group) else {
        return false;
    };
    if group.delim != Delim::Bracket {
        return false;
    }
    let bang = is_punct(before(nodes, index, 1), '!');
    is_punct(before(nodes, index, if bang { 2 } else { 1 }), '#')
}

/// Whether the token at `index` follows a single `.`: a field or a method,
/// not the end of a `..` range.
fn after_dot(nodes: &[Node], index: usize) -> bool {
    is_alone(before(nodes, index, 1), '.') && !is_joint(before(nodes, index, 2), '.')
}

/// Whether the identifier at `index` names an item rather than a local binding:
/// a path segment (`a::b`), a call (`f(…)`, `.m(…)`), a macro (`m!`), a
/// capitalised name (a type, variant or constant) or a primitive type.
fn names_an_item(nodes: &[Node], index: usize, text: &str) -> bool {
    let path_before =
        index >= 2 && is_punct(nodes.get(index - 1), ':') && is_punct(nodes.get(index - 2), ':');
    let path_after = is_punct(nodes.get(index + 1), ':') && is_punct(nodes.get(index + 2), ':');
    path_before
        || path_after
        || is_paren(nodes.get(index + 1))
        || is_punct(nodes.get(index + 1), '!')
        || text.starts_with(|first: char| first.is_ascii_uppercase())
        || PRIMITIVES.contains(&text)
}

/// Whether the identifier at `index` names a field: `.field` that is not a
/// method call, or, inside a struct literal or pattern (`fields`), a name in
/// field position.
fn names_a_field(nodes: &[Node], index: usize, fields: bool) -> bool {
    let previous = before(nodes, index, 1);
    let next = nodes.get(index + 1);
    let accessed = after_dot(nodes, index) && !is_paren(next) && !is_joint(next, ':');
    let in_position = fields
        && (index == 0 || is_punct(previous, ','))
        && (next.is_none() || is_punct(next, ',') || is_alone(next, ':'));
    accessed || in_position
}

/// Whether the braces after `previous` hold a struct literal or pattern.
fn opens_fields(previous: Option<&Node>) -> bool {
    leaf(previous).is_some_and(|leaf| {
        leaf.kind == Kind::Ident
            && (leaf.text == "Self"
                || leaf
                    .text
                    .starts_with(|first: char| first.is_ascii_uppercase()))
    })
}

/// Whether `node` ends an operand, so a `&` or `*` after it is the binary
/// operator rather than a borrow or a deref. A braced block ends a statement,
/// not an operand.
fn ends_operand(node: Option<&Node>) -> bool {
    match node {
        None => false,
        Some(Node::Leaf(leaf)) => match leaf.kind {
            Kind::Ident => {
                matches!(leaf.text.as_str(), "self" | "Self" | "true" | "false")
                    || !KEYWORDS.contains(&leaf.text.as_str())
            }
            Kind::Literal => true,
            Kind::Punct => leaf.text == "?",
        },
        Some(Node::Group(group)) => group.delim != Delim::Brace,
    }
}

fn renamed(leaf: &Leaf, text: String) -> Node {
    Node::Leaf(Leaf {
        text,
        ..leaf.clone()
    })
}

/// Rewrites the `{name}` and `{name:spec}` placeholders of the string literals
/// that are direct arguments of a macro call, through `lookup`. `{}`, `{0}`
/// and `{{` are kept, and so is a name `lookup` does not know.
fn rewrite_formats(nodes: &mut [Node], lookup: &dyn Fn(&str) -> Option<String>) {
    for index in 0..nodes.len() {
        let call = index >= 2
            && is_punct(nodes.get(index - 1), '!')
            && leaf(nodes.get(index - 2)).is_some_and(|name| name.kind == Kind::Ident);
        if let Node::Group(group) = &mut nodes[index] {
            if call {
                for child in &mut group.children {
                    if let Node::Leaf(literal) = child
                        && literal.kind == Kind::Literal
                        && let Some(text) = rewrite_placeholders(&literal.text, lookup)
                    {
                        literal.text = text;
                    }
                }
            }
            rewrite_formats(&mut group.children, lookup);
        }
    }
}

/// The literal with its placeholders rewritten, or `None` when it is not a
/// string or nothing changed.
fn rewrite_placeholders(text: &str, lookup: &dyn Fn(&str) -> Option<String>) -> Option<String> {
    let raw = text.starts_with('r');
    if !(raw || text.starts_with('"')) {
        return None;
    }
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut changed = false;
    let mut at = 0;
    while at < chars.len() {
        let character = chars[at];
        if !raw && character == '\\' {
            // An escape: `\u{…}` holds hex digits, never a placeholder.
            let unicode = chars.get(at + 1) == Some(&'u') && chars.get(at + 2) == Some(&'{');
            let width = if unicode {
                chars[at..]
                    .iter()
                    .position(|c| *c == '}')
                    .map_or(chars.len() - at, |end| end + 1)
            } else {
                2.min(chars.len() - at)
            };
            out.extend(&chars[at..at + width]);
            at += width;
            continue;
        }
        if character == '{' {
            if chars.get(at + 1) == Some(&'{') {
                out.push_str("{{");
                at += 2;
                continue;
            }
            let start = at + 1;
            let end = chars[start..]
                .iter()
                .position(|c| !(c.is_alphanumeric() || *c == '_'))
                .map_or(chars.len(), |len| start + len);
            let name: String = chars[start..end].iter().collect();
            let ident = name
                .chars()
                .next()
                .is_some_and(|first| first.is_alphabetic() || first == '_');
            if ident
                && matches!(chars.get(end), Some('}' | ':'))
                && let Some(replacement) = lookup(&name)
            {
                out.push('{');
                out.push_str(&replacement);
                at = end;
                changed = true;
                continue;
            }
        }
        out.push(character);
        at += 1;
    }
    changed.then_some(out)
}

// ---------------------------------------------------------------------------
// rename_params
// ---------------------------------------------------------------------------

/// Renames the function's own parameters to `$0`, `$1`, … in signature order.
/// A name after a `.` and a field name are not parameters.
pub(crate) fn rename_params(form: &Form, params: &[String]) -> Form {
    let map: BTreeMap<&str, usize> = params
        .iter()
        .enumerate()
        .map(|(position, name)| (name.as_str(), position))
        .collect();
    let mut nodes = params_level(&form.nodes, &map, false);
    rewrite_formats(&mut nodes, &|name| {
        map.get(name).map(|position| format!("${position}"))
    });
    Form { nodes }
}

fn params_level(nodes: &[Node], map: &BTreeMap<&str, usize>, fields: bool) -> Vec<Node> {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| match node {
            Node::Group(_) if is_attribute(nodes, index) => node.clone(),
            Node::Group(group) => {
                let opens = group.delim == Delim::Brace && opens_fields(before(nodes, index, 1));
                Node::Group(Group {
                    children: params_level(&group.children, map, opens),
                    ..group.clone()
                })
            }
            Node::Leaf(token) if token.kind == Kind::Ident => match map.get(token.text.as_str()) {
                Some(position)
                    if !after_dot(nodes, index) && !names_a_field(nodes, index, fields) =>
                {
                    renamed(token, format!("${position}"))
                }
                _ => node.clone(),
            },
            Node::Leaf(_) => node.clone(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// rename_locals
// ---------------------------------------------------------------------------

/// Renames every local name (bindings, closure parameters, lifetimes, labels,
/// types declared in the body) to its position, numbered after the highest
/// parameter number the body uses, by first appearance. Names inside tokens
/// the value form erases are not numbered: they become `$_`.
pub(crate) fn rename_locals(form: &Form) -> Form {
    let mut state = Namer {
        start: next_number(&form.nodes),
        names: BTreeMap::new(),
    };
    let mut nodes = state.level(&form.nodes, false, false);
    let assigned = state.names;
    rewrite_formats(&mut nodes, &|name| {
        assigned.get(name).map(|number| format!("${number}"))
    });
    Form { nodes }
}

/// One past the highest parameter number in `nodes`, or 0: the `$n` names, and
/// the `{$n}` placeholders `rename_params` wrote into format strings.
fn next_number(nodes: &[Node]) -> usize {
    nodes
        .iter()
        .map(|node| match node {
            Node::Leaf(token) if token.kind == Kind::Ident => token
                .text
                .strip_prefix('$')
                .and_then(|digits| digits.parse::<usize>().ok())
                .map_or(0, |number| number + 1),
            Node::Leaf(token) if token.kind == Kind::Literal => placeholder_numbers(&token.text),
            Node::Leaf(_) => 0,
            Node::Group(group) => next_number(&group.children),
        })
        .max()
        .unwrap_or(0)
}

/// One past the highest `n` of a `{$n` in a literal, or 0.
fn placeholder_numbers(text: &str) -> usize {
    text.split("{$")
        .skip(1)
        .filter_map(|rest| {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            digits.parse::<usize>().ok()
        })
        .map(|number| number + 1)
        .max()
        .unwrap_or(0)
}

struct Namer {
    start: usize,
    names: BTreeMap<String, usize>,
}

impl Namer {
    fn number(&mut self, text: &str) -> usize {
        let next = self.start + self.names.len();
        *self.names.entry(text.to_owned()).or_insert(next)
    }

    /// Registers the types declared at this level (`struct S`, `enum E`,
    /// `union U`, `trait T`, `type A`) as local names: a type declared inside a
    /// body is a private detail of that body.
    fn declare_local_types(&mut self, nodes: &[Node]) {
        for pair in nodes.windows(2) {
            if let [Node::Leaf(keyword), Node::Leaf(name)] = pair
                && keyword.kind == Kind::Ident
                && matches!(
                    keyword.text.as_str(),
                    "struct" | "enum" | "union" | "trait" | "type"
                )
                && name.kind == Kind::Ident
                && name
                    .text
                    .starts_with(|first: char| first.is_ascii_uppercase())
            {
                self.number(&name.text);
            }
        }
    }

    fn level(&mut self, nodes: &[Node], fields: bool, ghost: bool) -> Vec<Node> {
        if !ghost {
            self.declare_local_types(nodes);
        }
        let erased = erasure_mask(nodes);
        nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                let ghost = ghost || erased[index];
                match node {
                    Node::Group(_) if is_attribute(nodes, index) => node.clone(),
                    Node::Group(group) => {
                        let opens =
                            group.delim == Delim::Brace && opens_fields(before(nodes, index, 1));
                        Node::Group(Group {
                            children: self.level(&group.children, opens, ghost),
                            ..group.clone()
                        })
                    }
                    Node::Leaf(token) if token.kind == Kind::Ident => {
                        self.ident(nodes, index, token, fields, ghost)
                    }
                    Node::Leaf(_) => node.clone(),
                }
            })
            .collect()
    }

    fn ident(
        &mut self,
        nodes: &[Node],
        index: usize,
        token: &Leaf,
        fields: bool,
        ghost: bool,
    ) -> Node {
        let text = token.text.as_str();
        let local_type = text.starts_with(|first: char| first.is_ascii_uppercase())
            && self.names.contains_key(text);
        let kept = text.starts_with('$')
            || text == "_"
            || (!local_type
                && (is_keyword(text)
                    || names_an_item(nodes, index, text)
                    || names_a_field(nodes, index, fields)));
        if kept {
            Node::Leaf(token.clone())
        } else if ghost {
            renamed(token, "$_".to_owned())
        } else {
            let number = self.number(text);
            renamed(token, format!("${number}"))
        }
    }
}

// ---------------------------------------------------------------------------
// kind_literals
// ---------------------------------------------------------------------------

/// Replaces every literal with its kind (`$int`, `$str`, …). With `keep` the
/// pass is the identity: a body below the literal tier keeps its literals. A
/// tuple index (`pair.0`) is a field, not a constant, and is always kept.
pub(crate) fn kind_literals(form: &Form, keep: bool) -> Form {
    if keep {
        return form.clone();
    }
    Form {
        nodes: kinds_level(&form.nodes),
    }
}

fn kinds_level(nodes: &[Node]) -> Vec<Node> {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| match node {
            Node::Group(group) => Node::Group(Group {
                children: kinds_level(&group.children),
                ..group.clone()
            }),
            Node::Leaf(token) if token.kind == Kind::Literal && !after_dot(nodes, index) => {
                renamed(token, literal::kind(&token.text).to_owned())
            }
            Node::Leaf(_) => node.clone(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// erase_held
// ---------------------------------------------------------------------------

/// Drops the spellings that only change how a value is held: a borrow
/// (`&x`, `&mut x`, `&&x`), a deref (`*x`), `ref`, the value adapters called
/// with no arguments, a full-range reindex (`x[..]`), a `let` type annotation
/// and a turbofish that names types only. The reference inside an index
/// (`m[&k]`) goes with the borrow. The binary `&`, `&&`, `*` and their
/// compound assignments are kept.
pub(crate) fn erase_held(form: &Form) -> Form {
    Form {
        nodes: erase_level(&form.nodes),
    }
}

fn erase_level(nodes: &[Node]) -> Vec<Node> {
    let erased = erasure_mask(nodes);
    nodes
        .iter()
        .zip(erased)
        .filter(|(_, erased)| !erased)
        .map(|(node, _)| match node {
            Node::Group(group) => Node::Group(Group {
                children: erase_level(&group.children),
                ..group.clone()
            }),
            Node::Leaf(_) => node.clone(),
        })
        .collect()
}

/// Whether `node` is exactly `..`, the full range of `x[..]`.
fn is_full_range(group: &Group) -> bool {
    group.children.len() == 2
        && is_joint(group.children.first(), '.')
        && is_punct(group.children.get(1), '.')
}

/// Whether the `>` at `index` is the head of `->` or `=>`, not a closing angle.
fn is_arrow_head(nodes: &[Node], index: usize) -> bool {
    let previous = before(nodes, index, 1);
    is_joint(previous, '-') || is_joint(previous, '=')
}

/// The end (exclusive) of the generic argument list whose `<` is at `open`.
fn angle_end(nodes: &[Node], open: usize) -> Option<usize> {
    let mut depth = 0_usize;
    for (index, node) in nodes.iter().enumerate().skip(open) {
        let Some(token) = node.as_leaf().filter(|token| token.kind == Kind::Punct) else {
            continue;
        };
        match token.text.as_str() {
            "<" => depth += 1,
            ">" if !is_arrow_head(nodes, index) => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            ";" => return None,
            _ => {}
        }
    }
    None
}

/// Whether generic arguments name types only: no literal, `true`, `false` or
/// braced const block, whose value would change what the call does.
fn names_only_types(arguments: &[Node]) -> bool {
    arguments.iter().all(|node| match node {
        Node::Leaf(token) => match token.kind {
            Kind::Literal => false,
            Kind::Ident => token.text != "true" && token.text != "false",
            Kind::Punct => true,
        },
        Node::Group(group) => group.delim != Delim::Brace,
    })
}

/// The span of a `let` binding's type annotation, `: Type` up to the `=` or `;`
/// that closes it, for the `let` at `at`.
fn let_annotation(nodes: &[Node], at: usize) -> Option<(usize, usize)> {
    let mut index = at + 1;
    let colon = loop {
        let node = nodes.get(index)?;
        if is_punct(Some(node), ';') || is_punct(Some(node), '=') {
            return None;
        }
        if is_alone(Some(node), ':') && !is_joint(before(nodes, index, 1), ':') {
            break index;
        }
        index += 1;
    };
    let mut depth = 0_isize;
    for (index, node) in nodes.iter().enumerate().skip(colon + 1) {
        let Some(token) = node.as_leaf().filter(|token| token.kind == Kind::Punct) else {
            continue;
        };
        match token.text.as_str() {
            "<" => depth += 1,
            ">" if !is_arrow_head(nodes, index) => depth -= 1,
            ";" => return Some((colon, index)),
            "=" if depth <= 0 && !token.joint => return Some((colon, index)),
            _ => {}
        }
    }
    Some((colon, nodes.len()))
}

/// Which of `nodes` (one delimiter level) the value form drops.
fn erasure_mask(nodes: &[Node]) -> Vec<bool> {
    let mut erased = vec![false; nodes.len()];
    let mut index = 0;
    while index < nodes.len() {
        let previous = before(nodes, index, 1);
        match &nodes[index] {
            Node::Leaf(token)
                if token.kind == Kind::Punct && matches!(token.text.as_str(), "&" | "*") =>
            {
                if ends_operand(previous) {
                    // A binary operator; step over `&&`, `&=` and `*=` whole.
                    index += if token.joint { 2 } else { 1 };
                    continue;
                }
                erased[index] = true;
                if token.text == "&" && is_ident(nodes.get(index + 1), "mut") {
                    erased[index + 1] = true;
                    index += 1;
                }
            }
            Node::Leaf(token)
                if token.kind == Kind::Punct
                    && token.text == "."
                    && !token.joint
                    && !is_joint(previous, '.') =>
            {
                let adapter = leaf(nodes.get(index + 1)).is_some_and(|name| {
                    name.kind == Kind::Ident && VALUE_ADAPTERS.contains(&name.text.as_str())
                }) && nodes
                    .get(index + 2)
                    .and_then(Node::as_group)
                    .is_some_and(|group| group.delim == Delim::Paren && group.children.is_empty());
                if adapter {
                    erased[index..index + 3].fill(true);
                    index += 3;
                    continue;
                }
            }
            Node::Leaf(token)
                if token.kind == Kind::Punct
                    && token.text == ":"
                    && token.joint
                    && is_punct(nodes.get(index + 1), ':')
                    && is_punct(nodes.get(index + 2), '<') =>
            {
                if let Some(end) = angle_end(nodes, index + 2)
                    && names_only_types(&nodes[index + 3..end - 1])
                {
                    erased[index..end].fill(true);
                    index = end;
                    continue;
                }
            }
            Node::Group(group)
                if group.delim == Delim::Bracket
                    && ends_operand(previous)
                    && is_full_range(group) =>
            {
                erased[index] = true;
            }
            Node::Leaf(token) if token.kind == Kind::Ident && token.text == "ref" => {
                erased[index] = true;
                if is_ident(nodes.get(index + 1), "mut") {
                    erased[index + 1] = true;
                    index += 1;
                }
            }
            Node::Leaf(token) if token.kind == Kind::Ident && token.text == "let" => {
                if let Some((start, end)) = let_annotation(nodes, index) {
                    erased[start..end].fill(true);
                }
            }
            _ => {}
        }
        index += 1;
    }
    erased
}
