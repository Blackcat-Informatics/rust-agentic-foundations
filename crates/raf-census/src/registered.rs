// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Explicit repository operations and responses over the native RDF profile.

use raf_rdf::{Builder, Format, Graph, Node, TermValue as Term};

use crate::report::{shape, vocab, write::put};
use crate::{Error, GateReport, ReportError as E};

mod debt;
pub use debt::{DEBT_WRITE_PROTOCOL_VERSION, RegisteredDebtWrite};

/// The operation explicitly selected by a repository target's arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegisteredOperation {
    /// Read-only evaluation; selected by an empty argument sequence.
    Check,
    /// Read the complete index from the repository's registered evaluation.
    Index,
    /// Publish debt through the repository's registered Gate.
    WriteDebt,
}

impl RegisteredOperation {
    /// Decode the target arguments after its program name.
    ///
    /// # Errors
    /// Refuses unknown, repeated or additional arguments before any evaluation.
    pub fn from_args(args: impl IntoIterator<Item = std::ffi::OsString>) -> Result<Self, Error> {
        let mut args = args.into_iter();
        match (args.next(), args.next()) {
            (None, None) => Ok(Self::Check),
            (Some(token), None) if token == "--census-debt-write-v1" => Ok(Self::WriteDebt),
            (Some(token), None) if token == "--census-index-v1" => Ok(Self::Index),
            _ => Err(Error::Options(
                "registered target expects no arguments for check, exactly --census-index-v1 for index, or exactly --census-debt-write-v1 for debt publication".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests;

/// The fixed, read-only registered-check response protocol.
pub const CHECK_PROTOCOL_VERSION: &str = "urn:raf:census:registered-check:v1";
/// Maximum canonical response size, matching the bounded host stream policy.
pub const MAX_CHECK_RESPONSE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Outcome {
    Clean,
    Findings,
    Error,
}

impl Outcome {
    const fn text(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Findings => "findings",
            Self::Error => "error",
        }
    }
}

/// One complete registered Gate outcome, retained in native RDF storage.
///
/// Finding ordinals retain order and multiplicity, including equal messages.
/// This response never authorizes or performs a debt write.
#[derive(Clone, Debug)]
pub struct RegisteredCheck {
    graph: Graph,
    outcome: Outcome,
    count: usize,
}

impl RegisteredCheck {
    /// Project an actual Gate evaluation, preserving findings or its error.
    ///
    /// # Errors
    /// Refuses native storage/profile failures and oversized responses.
    pub fn from_check(result: &Result<GateReport, Error>) -> Result<Self, E> {
        match result {
            Ok(report) => Self::build(
                if report.is_clean() {
                    Outcome::Clean
                } else {
                    Outcome::Findings
                },
                report.findings().iter().map(String::as_str),
                None,
            ),
            Err(error) => Self::build(Outcome::Error, std::iter::empty(), Some(&error.to_string())),
        }
    }

    fn build<'a>(
        outcome: Outcome,
        messages: impl ExactSizeIterator<Item = &'a str>,
        error: Option<&str>,
    ) -> Result<Self, E> {
        let mut graph = Builder::new();
        let count = messages.len();
        put(&mut graph, "", "type", shape::iri("RegisteredCheck"))?;
        put(
            &mut graph,
            "",
            "protocolVersion",
            Term::iri(CHECK_PROTOCOL_VERSION),
        )?;
        put(
            &mut graph,
            "",
            "status",
            Term::simple_literal(outcome.text()),
        )?;
        put(
            &mut graph,
            "",
            "findingCount",
            shape::integer(count as u128),
        )?;
        let mut bytes = 0usize;
        for (ordinal, message) in messages.enumerate() {
            bytes = bytes.checked_add(message.len()).ok_or_else(oversized)?;
            if bytes > MAX_CHECK_RESPONSE_BYTES {
                return Err(oversized());
            }
            let id = finding_id(ordinal);
            put(&mut graph, "", "finding", Term::iri(&id))?;
            put(&mut graph, &id, "type", shape::iri("CensusFinding"))?;
            put(&mut graph, &id, "ordinal", shape::integer(ordinal as u128))?;
            put(&mut graph, &id, "message", Term::simple_literal(message))?;
        }
        if let Some(error) = error {
            if error.len() > MAX_CHECK_RESPONSE_BYTES {
                return Err(oversized());
            }
            put(&mut graph, "", "errorMessage", Term::simple_literal(error))?;
        }
        let graph = graph.finish().map_err(E::Ground)?;
        let result = Self {
            graph,
            outcome,
            count,
        };
        result.to_ntriples()?;
        Ok(result)
    }

    /// Read one exact canonical N-Triples1.2 response and validate its closed shape.
    ///
    /// # Errors
    /// Refuses size, canonical bytes, protocol version, fields, ordinals and
    /// inconsistent outcomes, including unreferenced or incomplete findings.
    pub fn from_ntriples(text: &str) -> Result<Self, E> {
        if text.len() > MAX_CHECK_RESPONSE_BYTES {
            return Err(oversized());
        }
        let graph = Graph::from_text(text, Format::NTriples).map_err(E::Ground)?;
        let root = graph
            .get("")
            .ok_or_else(|| E::Value("registered response requires its ground root".into()))?;
        if shape::class(&root)? != "RegisteredCheck" {
            return Err(E::Kind {
                expected: "RegisteredCheck".into(),
                found: shape::class(&root)?.into(),
            });
        }
        if shape::one(&root, "protocolVersion")?.iri() != Some(CHECK_PROTOCOL_VERSION) {
            return Err(E::Value(format!(
                "registered check requires protocol {CHECK_PROTOCOL_VERSION}"
            )));
        }
        let outcome = match shape::string(&root, "status")?.as_str() {
            "clean" => Outcome::Clean,
            "findings" => Outcome::Findings,
            "error" => Outcome::Error,
            other => return Err(E::Value(format!("registered check status {other}"))),
        };
        let count = shape::count(&root, "findingCount")?;
        validate_outcome(root, outcome, count)?;
        validate_members(&graph, root, count)?;
        Ok(Self {
            graph,
            outcome,
            count,
        })
    }

    /// Emit the native canonical N-Triples1.2 carrier.
    ///
    /// # Errors
    /// Refuses native output failure or a response exceeding the stream limit.
    pub fn to_ntriples(&self) -> Result<String, E> {
        let text = self.graph.to_text(Format::NTriples).map_err(E::Ground)?;
        if text.len() > MAX_CHECK_RESPONSE_BYTES {
            return Err(oversized());
        }
        Ok(text)
    }

    /// The target exit code required by this complete outcome: zero or one.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        if matches!(self.outcome, Outcome::Clean) {
            0
        } else {
            1
        }
    }

    /// Finding messages in their original order, without a parallel assertion map.
    ///
    /// # Panics
    /// Panics if the private graph loses its constructor-validated finding shape.
    pub fn findings(&self) -> impl ExactSizeIterator<Item = &str> {
        (0..self.count).map(|ordinal| {
            let node = self
                .graph
                .get(&finding_id(ordinal))
                .expect("validated finding sequence");
            shape::one(&node, "message")
                .expect("validated message")
                .literal()
                .expect("validated string")
                .0
        })
    }

    /// The original evaluation error, distinct from ordinary findings.
    ///
    /// # Panics
    /// Panics if the private graph loses its constructor-validated error shape.
    #[must_use]
    pub fn error_message(&self) -> Option<&str> {
        if self.outcome != Outcome::Error {
            return None;
        }
        let root = self.graph.get("").expect("validated root");
        Some(
            shape::one(&root, "errorMessage")
                .expect("validated error")
                .literal()
                .expect("validated string")
                .0,
        )
    }
}

fn finding_id(ordinal: usize) -> String {
    format!("#finding/{ordinal}")
}

fn oversized() -> E {
    E::Value(format!(
        "registered check response exceeds {MAX_CHECK_RESPONSE_BYTES} bytes"
    ))
}

fn validate_outcome(root: Node<'_>, outcome: Outcome, count: usize) -> Result<(), E> {
    let errors = root.values(&shape::property("errorMessage")).count();
    if (outcome == Outcome::Findings) != (count > 0) || (outcome == Outcome::Error && count != 0) {
        return Err(E::Inconsistent(
            "registered check status and finding count".into(),
        ));
    }
    if outcome == Outcome::Error {
        shape::string(&root, "errorMessage")?;
    } else if errors != 0 {
        return Err(E::Inconsistent(
            "non-error response contains errorMessage".into(),
        ));
    }
    Ok(())
}

fn validate_members(graph: &Graph, root: Node<'_>, count: usize) -> Result<(), E> {
    let links = root.values(&shape::property("finding"));
    if links.count() != count {
        return Err(E::Inconsistent(
            "registered findingCount disagrees with membership".into(),
        ));
    }
    for value in root.values(&shape::property("finding")) {
        let id = value
            .iri()
            .ok_or_else(|| E::Value("registered finding reference must be an IRI".into()))?;
        let node = graph.get(id).ok_or_else(|| E::Dangling(id.into()))?;
        let ordinal = shape::count(&node, "ordinal")?;
        if ordinal >= count || id != finding_id(ordinal) {
            return Err(E::Name(id.into()));
        }
        string_value(node, "message")?;
    }
    let mut subjects = 0usize;
    for (id, node) in graph.nodes() {
        subjects += 1;
        let class = shape::class(&node)?;
        let fields: &[&str] = match (id.is_empty(), class) {
            (true, "RegisteredCheck") => &[
                "protocolVersion",
                "status",
                "findingCount",
                "finding",
                "errorMessage",
            ],
            (false, "CensusFinding") => &["ordinal", "message"],
            _ => return Err(E::Class(format!("{id}: {class}"))),
        };
        for property in node.properties() {
            if property != raf_rdf::vocab::TYPE
                && !property
                    .strip_prefix(vocab::CENSUS)
                    .is_some_and(|key| fields.contains(&key))
            {
                return Err(E::UnknownPredicate(property.into()));
            }
        }
    }
    if subjects.checked_sub(1) != Some(count) {
        return Err(E::Orphan(
            "registered response has unreferenced subjects".into(),
        ));
    }
    Ok(())
}

fn string_value<'a>(node: Node<'a>, key: &str) -> Result<&'a str, E> {
    shape::one(&node, key)?
        .literal()
        .filter(|(_, datatype)| *datatype == raf_rdf::vocab::STRING)
        .map(|(text, _)| text)
        .ok_or_else(|| E::Value(format!("registered {key}: string expected")))
}
