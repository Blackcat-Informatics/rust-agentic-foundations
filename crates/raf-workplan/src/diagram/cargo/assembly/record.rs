// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed assembly policy records over the native RDF carrier.

use super::{DiagramError, refused};
use raf_rdf::{Builder, Graph, TermValue, Triple, vocab};

const NS: &str = "urn:raf:rustdoc-assembly:";

pub(super) fn put(
    builder: &mut Builder,
    subject: &str,
    key: &str,
    value: &str,
) -> Result<(), DiagramError> {
    builder
        .insert(Triple {
            subject: TermValue::iri(subject),
            predicate: TermValue::iri(format!("{NS}{key}")),
            object: TermValue::simple_literal(value),
        })
        .map_err(DiagramError::data)
}

pub(super) fn text(graph: &Graph, subject: &str, key: &str) -> Result<String, DiagramError> {
    let node = graph
        .get(subject)
        .ok_or_else(|| refused("assembly record subject is absent"))?;
    let predicate = format!("{NS}{key}");
    let mut values = node.values(&predicate);
    let value = values
        .next()
        .ok_or_else(|| refused("assembly record field is absent"))?;
    if values.next().is_some() {
        return Err(refused("assembly record scalar is repeated"));
    }
    let (text, datatype) = value
        .literal()
        .ok_or_else(|| refused("assembly field must be a string"))?;
    if datatype != vocab::STRING || text.len() > super::MAX_BYTES {
        return Err(refused("assembly field exceeds its closed string contract"));
    }
    Ok(text.to_owned())
}

pub(super) fn count(graph: &Graph, subject: &str, key: &str) -> Result<usize, DiagramError> {
    let lexical = text(graph, subject, key)?;
    let integer = purrdf_xsd::numeric::parse_integer(&lexical).map_err(DiagramError::data)?;
    if integer.is_negative() || integer.canonical_lexical() != lexical {
        return Err(refused("assembly count must be canonical and nonnegative"));
    }
    let value = integer
        .into_bigint()
        .unsigned_abs_u128()
        .and_then(|value| usize::try_from(value).ok())
        .filter(|value| *value <= super::MAX_ITEMS)
        .ok_or_else(|| refused("assembly count exceeds its bound"))?;
    Ok(value)
}

fn word_subject(subject: &str, key: &str, index: usize) -> String {
    if subject.is_empty() {
        format!("#{key}/{index}")
    } else {
        format!("{subject}/{key}/{index}")
    }
}

pub(super) fn words(
    builder: &mut Builder,
    subject: &str,
    key: &str,
    words: &[String],
) -> Result<(), DiagramError> {
    if words.len() > super::MAX_ITEMS {
        return Err(refused("assembly list exceeds its item bound"));
    }
    put(
        builder,
        subject,
        &format!("{key}Count"),
        &words.len().to_string(),
    )?;
    for (index, value) in words.iter().enumerate() {
        put(builder, &word_subject(subject, key, index), "value", value)?;
    }
    Ok(())
}

pub(super) fn read_words(
    graph: &Graph,
    subject: &str,
    key: &str,
) -> Result<Vec<String>, DiagramError> {
    (0..count(graph, subject, &format!("{key}Count"))?)
        .map(|index| text(graph, &word_subject(subject, key, index), "value"))
        .collect()
}

pub(super) fn closed(expected: &Graph, actual: &Graph) -> Result<(), DiagramError> {
    if expected.snapshot().map_err(DiagramError::data)?
        != actual.snapshot().map_err(DiagramError::data)?
    {
        return Err(refused(
            "assembly record contains unknown, orphaned or inconsistent assertions",
        ));
    }
    Ok(())
}
