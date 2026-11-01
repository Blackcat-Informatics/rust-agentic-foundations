// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Registry completeness of the inputs exercised through the production Gate.

#[path = "../../tests/support/detector_fixtures.rs"]
mod fixtures;

#[test]
fn every_executable_detector_has_positive_and_negative_gate_fixtures() {
    let coverage: Vec<_> = fixtures::LAYOUT
        .map(|(id, _, _)| (id, 1, 1))
        .into_iter()
        .chain(fixtures::FACTS.map(|(id, _, _, _)| (id, 1, 1)))
        .chain(fixtures::HEX.map(|(id, _, _, _)| (id, 1, 1)))
        .chain(fixtures::CONTEXTUAL.map(|(id, _, _, _)| (id, 1, 1)))
        .chain(fixtures::RDF.map(|(id, _, _)| {
            let nested = fixtures::RDF_NESTED
                .iter()
                .filter(|(rule, _, _)| *rule == id)
                .count();
            (id, 1 + nested, 1 + nested)
        }))
        .chain([
            (
                fixtures::HASHER_ID,
                fixtures::HASHER_POSITIVE.len(),
                fixtures::HASHER_NEGATIVE.len(),
            ),
            (
                fixtures::SOLE_NAME_ID,
                fixtures::SOLE_NAME_POSITIVE.len(),
                fixtures::SOLE_NAME_NEGATIVE.len(),
            ),
        ])
        .collect();
    let registered: Vec<_> = super::ids().collect();
    for (id, _, _) in fixtures::RDF_NESTED {
        assert!(
            fixtures::RDF.iter().any(|(rule, _, _)| *rule == id),
            "unregistered nested fixture family {id}"
        );
    }
    assert_eq!(coverage.len(), registered.len(), "fixture registry differs");
    for (index, id) in registered.iter().enumerate() {
        assert!(!registered[..index].contains(id), "duplicate detector {id}");
        assert_eq!(
            coverage
                .iter()
                .filter(|(covered, _, _)| covered == id)
                .count(),
            1,
            "detector {id} must have exactly one fixture family"
        );
    }
    for (id, positive, negative) in coverage {
        assert!(
            positive > 0 && negative > 0,
            "missing fixture side for {id}"
        );
    }
}
