// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Stable namespace laws; domain registries enumerate their closed shapes.

/// The IRI-versioned shared report vocabulary.
pub const REPORT: &str = "https://blackcatinformatics.ca/raf/report/1#";
/// Snapshot namespace, followed by a complete lowercase BLAKE3 digest.
pub const BASE: &str = "https://blackcatinformatics.ca/.well-known/genid/raf-report/";
/// RDF structural vocabulary.
pub use purrdf_iri::vocab::rdf::NS as RDF;
/// Named statement provenance over a depth-one triple term.
pub use purrdf_iri::vocab::rdf::REIFIES;
/// RDF class membership.
pub use purrdf_iri::vocab::rdf::TYPE;
/// XSD boolean datatype.
pub use purrdf_xsd::datatype::XSD_BOOLEAN as BOOLEAN;
/// XSD integer datatype.
pub use purrdf_xsd::datatype::XSD_INTEGER as INTEGER;
/// XML Schema datatype vocabulary.
pub use purrdf_xsd::datatype::XSD_NS as XSD;
/// XSD string datatype.
pub use purrdf_xsd::datatype::XSD_STRING as STRING;
