// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Shared canonical parsing followed by the complete census domain validator.

use super::{Baseline, Index, ReportError, shape};
use crate::Error;

pub(super) fn parse(text: &str, format: raf_rdf::Format) -> Result<raf_rdf::Graph, Error> {
    let graph = raf_rdf::Graph::from_text(text, format).map_err(ReportError::Ground)?;
    Ok(graph)
}
pub(super) fn baseline(text: &str, format: raf_rdf::Format) -> Result<Baseline, Error> {
    Ok(shape::baseline(&parse(text, format)?)?)
}
pub(super) fn index(text: &str, format: raf_rdf::Format) -> Result<Index, Error> {
    Ok(shape::index(&parse(text, format)?)?)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_graph_of_each_example_is_its_pinned_triple_set() {
        for (text, native, expected) in [
            (
                include_str!("../../tests/fixtures/reports/baseline.ttl"),
                include_str!("../../tests/fixtures/reports/baseline.nt"),
                include_str!("../../tests/fixtures/reports/baseline.oracle.nt"),
            ),
            (
                include_str!("../../tests/fixtures/reports/index.ttl"),
                include_str!("../../tests/fixtures/reports/index.nt"),
                include_str!("../../tests/fixtures/reports/index.oracle.nt"),
            ),
            (
                include_str!("../../tests/fixtures/reports/awkward.ttl"),
                include_str!("../../tests/fixtures/reports/awkward.nt"),
                include_str!("../../tests/fixtures/reports/awkward.oracle.nt"),
            ),
        ] {
            let graph = raf_rdf::Graph::from_text(text, raf_rdf::Format::Turtle).unwrap();
            let nt = graph.to_text(raf_rdf::Format::NTriples).unwrap();
            assert_eq!(nt, native);
            let mut triples: Vec<_> = nt.lines().collect();
            triples.sort_unstable();
            // Pins are sorted ground triples from registry purrdf-rdf 3.0.1,
            // a distinct revision of the same native parser family.
            assert_eq!(format!("{}\n", triples.join("\n")), expected);
            assert_eq!(
                raf_rdf::Graph::from_text(&nt, raf_rdf::Format::NTriples)
                    .unwrap()
                    .snapshot()
                    .unwrap(),
                graph.snapshot().unwrap()
            );
        }
    }
}
