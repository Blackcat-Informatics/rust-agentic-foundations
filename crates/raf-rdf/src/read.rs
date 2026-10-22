// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native RDF parsing followed by the closed report's ownership and byte laws.

use crate::{Error, Format, Graph, vocab, write};
use purrdf_rdf::{ParseOptions, RdfDataset, TermRef, parse_dataset_with};

fn checked_base(text: &str) -> Result<&str, Error> {
    if text
        .strip_prefix(vocab::BASE)
        .is_none_or(|hex| purrdf_hash::hex::decode_32_canonical(hex).is_none())
    {
        return Err(Error::Value(
            "snapshot BASE requires its namespace and 64 lowercase hexadecimal digits".into(),
        ));
    }
    Ok(text)
}

fn root_base(dataset: &RdfDataset) -> Result<String, Error> {
    let subjects = dataset
        .quads()
        .map(|row| row.s)
        .chain(dataset.reifiers().map(|(subject, _)| subject))
        .chain(dataset.annotations().map(|(subject, _, _)| subject));
    let mut root = None;
    for subject in subjects {
        if let TermRef::Iri(iri) = dataset.resolve(subject)
            && checked_base(iri).is_ok()
        {
            if root.is_some_and(|before| before != iri) {
                return Err(Error::Value("more than one snapshot root".into()));
            }
            root = Some(iri);
        }
    }
    root.map(str::to_owned)
        .ok_or_else(|| Error::Value("absolute snapshot root is absent".into()))
}

pub(crate) fn parse(text: &str, format: Format) -> Result<Graph, Error> {
    let media = match format {
        Format::Turtle => "text/turtle",
        Format::NTriples => "application/n-triples",
    };
    let parsed = parse_dataset_with(text.as_bytes(), media, None, &ParseOptions::default())
        .map_err(|error| {
            let (line, column) = error.location.as_deref().map_or((1, 1), |location| {
                (
                    location
                        .line
                        .and_then(|line| usize::try_from(line).ok())
                        .unwrap_or(1),
                    location.column.map_or(1, |column| column as usize),
                )
            });
            Error::Syntax {
                line,
                column,
                reason: error.to_string(),
            }
        })?;
    let base = match format {
        Format::Turtle => {
            let base = parsed
                .document_base
                .as_ref()
                .filter(|base| matches!(base.origin(), purrdf_iri::BaseOrigin::Directive { .. }))
                .ok_or_else(|| Error::Value("snapshot BASE directive is absent".into()))?;
            checked_base(base.iri().as_str())?.to_owned()
        }
        Format::NTriples => root_base(&parsed.dataset)?,
    };
    if parsed.dataset.named_graphs().next().is_some() {
        return Err(Error::Value(
            "named graphs are outside the report profile".into(),
        ));
    }
    let graph = Graph {
        dataset: parsed.dataset,
        owner: base.clone(),
    };
    graph.validate()?;
    let (computed, canonical) = write::render(&graph, format)?;
    if computed != base {
        return Err(Error::Digest {
            declared: base,
            computed,
        });
    }
    if canonical != text {
        let offset = purrdf_deflate::common_prefix_len(canonical.as_bytes(), text.as_bytes());
        let position = purrdf_iri::LineIndex::new(text)
            .try_locate(text, offset)
            .map_err(|e| Error::Value(e.to_string()))?;
        let line = usize::try_from(position.line)
            .map_err(|_| Error::Value("line exceeds usize".into()))?;
        return Err(Error::NotCanonical { line });
    }
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ownership_normalization_checks_the_selected_root_boundary() {
        let graph = Graph::from_text(
            include_str!("../tests/fixtures/profile.ttl"),
            Format::Turtle,
        )
        .unwrap();
        let base = &graph.owner;
        assert_eq!(graph.local(base), "");
        let fragment = format!("{base}#node");
        assert_eq!(graph.local(&fragment), "#node");
        for other in [
            format!("{base}suffix"),
            format!("{}{}#node", vocab::BASE, "b".repeat(64)),
        ] {
            assert_eq!(graph.local(&other), other);
        }
    }
}
