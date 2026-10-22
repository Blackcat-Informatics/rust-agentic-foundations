// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! What a unit holds besides its print: integer constants by value, string
//! literals with their lines, hex-digit tables and calls through `Self::`.
//! Everything is read from the verbatim form of the slice, so a literal keeps
//! the line of the file it is written on.

use std::collections::BTreeSet;

use super::{StringKind, StringLiteral};
use crate::normalize::{Delim, Group, Kind, Node, TableKind, literal_kind, parse_integer};
use crate::{Error, Form};

/// The literals of one slice.
#[derive(Debug, Default)]
pub(super) struct Facts {
    pub(super) constants: BTreeSet<u128>,
    pub(super) strings: Vec<StringLiteral>,
    pub(super) tables: BTreeSet<TableKind>,
}

/// Reads the literals of `form`, a slice of `file`.
///
/// # Errors
///
/// [`Error::Source`] when a string literal does not decode, which a lexed
/// literal never fails to do.
pub(super) fn facts(file: &str, form: &Form) -> Result<Facts, Error> {
    let mut facts = Facts::default();
    walk(file, &form.nodes, &mut facts)?;
    Ok(facts)
}

fn walk(file: &str, nodes: &[Node], facts: &mut Facts) -> Result<(), Error> {
    for node in nodes {
        match node {
            Node::Leaf(leaf) if leaf.kind == Kind::Literal => {
                if let Some(value) = parse_integer(&leaf.text).and_then(|integer| integer.value) {
                    facts.constants.insert(value);
                }
                if matches!(literal_kind(&leaf.text), "$str" | "$bytes" | "$cstr") {
                    let (kind, value, exact) =
                        decode(&leaf.text).map_err(|reason| Error::Source {
                            file: file.to_owned(),
                            line: leaf.place.line,
                            reason,
                        })?;
                    if exact && let Some(table) = classify_digits(&value) {
                        facts.tables.insert(table);
                    }
                    facts.strings.push(StringLiteral {
                        value,
                        line: leaf.place.line,
                        kind,
                    });
                }
            }
            Node::Leaf(_) => {}
            Node::Group(group) => {
                if group.delim == Delim::Bracket
                    && let Some(table) = digit_array(group)
                {
                    facts.tables.insert(table);
                }
                walk(file, &group.children, facts)?;
            }
        }
    }
    Ok(())
}

/// The kind, the text and whether the text is exact, of a string, byte-string
/// or C-string literal. A byte string or C string that is not UTF-8 is
/// decoded lossily and is not exact, so it never names a table.
fn decode(text: &str) -> Result<(StringKind, String, bool), String> {
    let exact = |kind, bytes: Vec<u8>| match String::from_utf8(bytes) {
        Ok(value) => (kind, value, true),
        Err(error) => (
            kind,
            String::from_utf8_lossy(error.as_bytes()).into_owned(),
            false,
        ),
    };
    match syn::parse_str::<syn::Lit>(text) {
        Ok(syn::Lit::Str(value)) => Ok((StringKind::Str, value.value(), true)),
        Ok(syn::Lit::ByteStr(value)) => Ok(exact(StringKind::Bytes, value.value())),
        Ok(syn::Lit::CStr(value)) => Ok(exact(StringKind::CStr, value.value().into_bytes())),
        Ok(_) => Err(format!("`{text}` is not a string literal")),
        Err(error) => Err(format!("the literal `{text}` does not decode: {error}")),
    }
}

/// Which hex-digit table `digits` spells, if any: the sixteen digits in
/// order, all lowercase or all uppercase.
pub(super) fn classify_digits(digits: &str) -> Option<TableKind> {
    if digits.len() != 16 {
        return None;
    }
    let bytes = [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
    let mut encoded = [0; 16];
    let lower = purrdf_hash::hex::encode_to_slice(&bytes, &mut encoded)
        .expect("sixteen bytes hold the encoding of eight bytes");
    if digits == lower {
        Some(TableKind::HexLower)
    } else if digits
        == purrdf_hash::hex::encode_upper_to_slice(&bytes, &mut encoded)
            .expect("sixteen bytes hold the encoding of eight bytes")
    {
        Some(TableKind::HexUpper)
    } else {
        None
    }
}

/// The table a bracketed group of character or byte literals spells.
fn digit_array(group: &Group) -> Option<TableKind> {
    let mut digits = String::new();
    for node in &group.children {
        let leaf = node.as_leaf()?;
        match leaf.kind {
            Kind::Punct if leaf.text == "," => {}
            Kind::Literal => {
                let unprefixed = leaf.text.strip_prefix('b').unwrap_or(&leaf.text);
                let inner = unprefixed.strip_prefix('\'')?.strip_suffix('\'')?;
                let mut chars = inner.chars();
                let (Some(single), None) = (chars.next(), chars.next()) else {
                    return None;
                };
                digits.push(single);
            }
            _ => return None,
        }
    }
    classify_digits(&digits)
}
