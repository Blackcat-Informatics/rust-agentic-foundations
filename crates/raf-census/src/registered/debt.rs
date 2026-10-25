// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A receipt for an already attempted explicit debt publication.

use super::{Builder, E, Error, Format, Graph, Term, shape, string_value, vocab};
use crate::report::write::put;
use crate::{DebtWrite, MAX_CHECK_RESPONSE_BYTES};

/// The distinct explicit-publication response protocol.
pub const DEBT_WRITE_PROTOCOL_VERSION: &str = "urn:raf:census:registered-debt-write:v1";

/// A complete debt writer result retained in native RDF storage.
///
/// This acknowledges trusted execution; it never authorizes a write or supplies
/// a destination. Publication may precede a later transport failure.
#[derive(Clone, Debug)]
pub struct RegisteredDebtWrite {
    graph: Graph,
    counts: Option<(usize, usize)>,
}

impl RegisteredDebtWrite {
    /// Project the result of the actual Gate writer, without publishing anything.
    ///
    /// # Errors
    /// Refuses native storage/profile failures and oversized responses.
    pub fn from_write(result: &Result<DebtWrite, Error>) -> Result<Self, E> {
        let mut graph = Builder::new();
        put(&mut graph, "", "type", shape::iri("RegisteredDebtWrite"))?;
        put(
            &mut graph,
            "",
            "protocolVersion",
            Term::iri(DEBT_WRITE_PROTOCOL_VERSION),
        )?;
        let counts = match result {
            Ok(receipt) => {
                put(&mut graph, "", "status", Term::simple_literal("written"))?;
                put(
                    &mut graph,
                    "",
                    "groupCount",
                    shape::integer(receipt.groups as u128),
                )?;
                put(
                    &mut graph,
                    "",
                    "jobCount",
                    shape::integer(receipt.jobs as u128),
                )?;
                Some((receipt.groups, receipt.jobs))
            }
            Err(error) => {
                let message = error.to_string();
                if message.len() > MAX_CHECK_RESPONSE_BYTES {
                    return Err(oversized());
                }
                put(&mut graph, "", "status", Term::simple_literal("error"))?;
                put(
                    &mut graph,
                    "",
                    "errorMessage",
                    Term::simple_literal(message),
                )?;
                None
            }
        };
        let receipt = Self {
            graph: graph.finish().map_err(E::Ground)?,
            counts,
        };
        receipt.to_ntriples()?;
        Ok(receipt)
    }

    /// Read a complete canonical N-Triples 1.2 write receipt.
    ///
    /// # Errors
    /// Refuses check responses, unknown versions/fields, inconsistent status,
    /// missing/repeated scalars, extra subjects, noncanonical bytes and size.
    pub fn from_ntriples(text: &str) -> Result<Self, E> {
        if text.len() > MAX_CHECK_RESPONSE_BYTES {
            return Err(oversized());
        }
        let graph = Graph::from_text(text, Format::NTriples).map_err(E::Ground)?;
        let root = graph
            .get("")
            .ok_or_else(|| E::Value("debt receipt requires its ground root".into()))?;
        if shape::class(&root)? != "RegisteredDebtWrite" {
            return Err(E::Kind {
                expected: "RegisteredDebtWrite".into(),
                found: shape::class(&root)?.into(),
            });
        }
        if shape::one(&root, "protocolVersion")?.iri() != Some(DEBT_WRITE_PROTOCOL_VERSION) {
            return Err(E::Value(format!(
                "debt receipt requires protocol {DEBT_WRITE_PROTOCOL_VERSION}"
            )));
        }
        let counts = match string_value(root, "status")? {
            "written" => Some((
                shape::count(&root, "groupCount")?,
                shape::count(&root, "jobCount")?,
            )),
            "error" => {
                string_value(root, "errorMessage")?;
                None
            }
            other => return Err(E::Value(format!("debt receipt status {other}"))),
        };
        let fields: &[&str] = if counts.is_some() {
            &["protocolVersion", "status", "groupCount", "jobCount"]
        } else {
            &["protocolVersion", "status", "errorMessage"]
        };
        for property in root.properties() {
            if property != raf_rdf::vocab::TYPE
                && !property
                    .strip_prefix(vocab::CENSUS)
                    .is_some_and(|key| fields.contains(&key))
            {
                return Err(E::UnknownPredicate(property.into()));
            }
        }
        if graph.nodes().count() != 1 {
            return Err(E::Orphan("debt receipt contains extra subjects".into()));
        }
        Ok(Self { graph, counts })
    }

    /// Emit the canonical native carrier.
    ///
    /// # Errors
    /// Refuses native output failure or an oversized response.
    pub fn to_ntriples(&self) -> Result<String, E> {
        let text = self.graph.to_text(Format::NTriples).map_err(E::Ground)?;
        if text.len() > MAX_CHECK_RESPONSE_BYTES {
            return Err(oversized());
        }
        Ok(text)
    }

    /// Counts of groups and jobs with copies, present only after successful publication.
    #[must_use]
    pub const fn counts(&self) -> Option<(usize, usize)> {
        self.counts
    }

    /// Zero acknowledges publication; one reports the backend error.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        if self.counts.is_some() { 0 } else { 1 }
    }

    /// The original writer error, without treating it as debt or a finding.
    ///
    /// # Panics
    /// Panics if the private constructor-validated graph loses its shape.
    #[must_use]
    pub fn error_message(&self) -> Option<&str> {
        if self.counts.is_some() {
            return None;
        }
        let root = self.graph.get("").expect("validated receipt root");
        Some(string_value(root, "errorMessage").expect("validated receipt error"))
    }
}

fn oversized() -> E {
    E::Value(format!(
        "registered debt response exceeds {MAX_CHECK_RESPONSE_BYTES} bytes"
    ))
}
