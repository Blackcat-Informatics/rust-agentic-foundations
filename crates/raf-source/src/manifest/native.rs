// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Cargo field selection and scalar policy over the supplied native TOML arena.

use purrdf_toml::{
    Bounds, Document, Float, Kind, Node, Profile, Scalar, SourceDocument, TomlError, Vocabulary,
};

pub(crate) fn read(text: &str) -> Result<Document<'_>, TomlError> {
    let profile = Profile::new(
        "raf-cargo-toml",
        1,
        Vocabulary::standard()?,
        Bounds {
            max_source_bytes: text.len().max(1) as u64,
            max_depth: 128,
            ..Bounds::standard()
        },
    )?;
    let document = purrdf_toml::analyze(
        SourceDocument {
            id: "urn:raf:cargo-toml",
            bytes: text.as_bytes(),
        },
        &profile,
    )?;
    for node in document.nodes() {
        if matches!(
            node.kind,
            Kind::Scalar(Scalar::Float(Float::Infinity { .. }))
        ) && !["inf", "+inf", "-inf"].contains(&&text[node.span.clone()])
        {
            return Err(TomlError::Syntax {
                location: purrdf_toml::Location {
                    byte: node.span.start,
                    line: text[..node.span.start]
                        .bytes()
                        .filter(|byte| *byte == b'\n')
                        .count()
                        + 1,
                    column: text[..node.span.start]
                        .rsplit('\n')
                        .next()
                        .unwrap_or_default()
                        .chars()
                        .count()
                        + 1,
                },
                expected: "Cargo floating-point number must not overflow; explicit inf is required",
            });
        }
    }
    Ok(document)
}

pub(crate) fn field<'a>(document: &'a Document<'_>, table: &Node, key: &str) -> Option<&'a Node> {
    table
        .children
        .iter()
        .map(|id| &document.nodes()[*id])
        .find(|node| node.key.as_ref().is_some_and(|name| name.decoded == key))
}

pub(crate) fn string(node: &Node) -> Option<&str> {
    match &node.kind {
        Kind::Scalar(Scalar::String(text)) => Some(text),
        _ => None,
    }
}

pub(crate) fn boolean(node: &Node) -> Option<bool> {
    match node.kind {
        Kind::Scalar(Scalar::Boolean(value)) => Some(value),
        _ => None,
    }
}

pub(crate) fn table(node: &Node) -> Option<&Node> {
    matches!(node.kind, Kind::Table(_)).then_some(node)
}

pub(crate) fn array(node: &Node) -> Option<&[usize]> {
    matches!(node.kind, Kind::Array | Kind::ArrayOfTables).then_some(node.children.as_slice())
}

pub(crate) fn entries<'a>(document: &'a Document<'_>, table: &Node) -> Vec<(&'a str, &'a Node)> {
    let mut entries: Vec<_> = table
        .children
        .iter()
        .map(|id| {
            let node = &document.nodes()[*id];
            (
                node.key
                    .as_ref()
                    .expect("native table member key")
                    .decoded
                    .as_str(),
                node,
            )
        })
        .collect();
    // Cargo declarations previously used lexical key order. This is a schema
    // selection index, not another TOML value or assertion representation.
    purrdf_lex::walk::try_sort_unstable_by(&mut entries, |left, right| {
        Ok::<_, std::convert::Infallible>(left.0.cmp(right.0))
    })
    .expect("infallible key comparison");
    entries
}
