// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native carriers for authoring-codec refusal fixtures.

/// Require native RDF construction to succeed before judging an authoring shape.
pub(super) fn dataset_rows(
    rows: &[purrdf_core::RdfQuad],
) -> std::sync::Arc<purrdf_core::RdfDataset> {
    let mut builder = purrdf_core::RdfDatasetBuilder::new();
    for row in rows {
        builder.push_owned_quad(row).unwrap();
    }
    builder.freeze().unwrap()
}
