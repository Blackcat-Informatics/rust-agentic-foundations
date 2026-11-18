// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Built-in syntax recognition; source ownership is assigned by the gate.

mod format;
mod hasher;
mod hex;
mod host_import;
mod layout;
mod names;
mod rdf;
mod structure;

#[cfg(test)]
mod tests;

pub(crate) fn ids() -> impl Iterator<Item = &'static str> {
    layout::RULES
        .into_iter()
        .chain(hex::RULES)
        .chain(["hex-table", "json-pointer-escape"])
        .chain(["home-literal", "sole-name", "vocabulary-literal"])
        .chain(["std-default-hasher", host_import::RULE])
        .chain(rdf::RULES)
        .chain(structure::RULES)
}

pub(crate) fn implements(id: &str) -> bool {
    ids().any(|candidate| candidate == id)
}

/// Whether the detector judges one job's home and context, so its hits belong
/// to that job and cannot be counted per file.
pub(crate) fn job_scoped(id: &str) -> bool {
    matches!(
        id,
        "home-literal"
            | "vocabulary-literal"
            | "sole-name"
            | host_import::RULE
            | structure::REIFIER
            | structure::LIST_WALK
    )
}

pub(crate) fn home_exempt(id: &str) -> bool {
    implements(id) && id != "sole-name" && id != host_import::RULE && !rdf::RULES.contains(&id)
}

pub(crate) fn needs_syntax(id: &str) -> bool {
    layout::RULES.contains(&id)
        || hex::RULES.contains(&id)
        || id == "std-default-hasher"
        || id == host_import::RULE
        || rdf::RULES.contains(&id)
        || structure::RULES.contains(&id)
}

pub(crate) fn shipping_only(id: &str) -> bool {
    implements(id)
        && !matches!(
            id,
            "std-default-hasher"
                | host_import::RULE
                | "hex-format-loop"
                | "hex-pair-radix"
                | "hex-table"
                | "sole-name"
                | "serde-json"
                | "rdf-text-writer"
                | "rdf-term-model"
        )
}

/// Reuse facts from the normal form collector instead of lexing these twice.
pub(crate) fn facts(unit: &crate::Unit) -> Vec<RawHit> {
    let mut hits = Vec::new();
    for table in &unit.tables {
        hits.push(RawHit {
            rule: "hex-table",
            byte: unit.start_byte,
            line: unit.line,
            call_evidence: None,
            message: format!(
                "a {} digit table; use the approved base16 implementation",
                table.name()
            ),
        });
    }
    if unit.shipping {
        for literal in &unit.strings {
            if literal.kind == crate::StringKind::Str
                && matches!(literal.value.as_str(), "~0" | "~1")
            {
                hits.push(RawHit {
                    rule: "json-pointer-escape",
                    byte: unit.start_byte,
                    line: literal.line,
                    call_evidence: None,
                    message:
                        "a JSON Pointer escape written by hand; use the approved pointer encoder"
                            .into(),
                });
            }
        }
    }
    hits
}

/// A call spelling at the byte where its lookup context is known.
#[derive(Clone)]
pub(crate) struct Call {
    pub(crate) byte: usize,
    pub(crate) path: CallPath,
    pub(crate) receiver: Option<ReceiverMode>,
}

/// The unchanged receiver type at the first step of a dot-call lookup.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReceiverMode {
    Value,
    Shared,
    Mutable,
}

#[derive(Clone)]
pub(crate) enum CallPath {
    Written(String),
    Import(String),
    SelfType(String),
}

pub(crate) enum CallEvidence {
    Reifier(Call),
    List { calls: Vec<Call>, looped: bool },
}

pub(crate) struct RawHit {
    pub(crate) rule: &'static str,
    pub(crate) byte: usize,
    pub(crate) line: usize,
    pub(crate) message: String,
    /// Unresolved evidence for the job-scoped structure detectors.
    pub(crate) call_evidence: Option<CallEvidence>,
}

impl RawHit {
    fn new(rule: &'static str, span: proc_macro2::Span, message: &str) -> Self {
        Self {
            rule,
            byte: span.byte_range().start,
            line: span.start().line,
            message: message.into(),
            call_evidence: None,
        }
    }

    fn push(hits: &mut Vec<Self>, rule: &'static str, span: proc_macro2::Span, message: &str) {
        hits.push(Self::new(rule, span, message));
    }
}

/// A source construct a detector cannot judge; the gate refuses it with this text.
pub(crate) struct Refusal {
    pub(crate) rule: &'static str,
    pub(crate) line: usize,
    pub(crate) message: String,
}

pub(crate) fn collect(
    file: &syn::File,
    requested: &std::collections::BTreeSet<&str>,
) -> Result<SyntaxHits, Refusal> {
    let mut hits = Vec::new();
    let mut receivers = Vec::new();
    if layout::RULES.iter().any(|id| requested.contains(id)) {
        hits.extend(layout::collect(file));
    }
    if hex::RULES.iter().any(|id| requested.contains(id)) {
        hits.extend(hex::collect(file));
    }
    if requested.contains("std-default-hasher") {
        hits.extend(hasher::collect(file));
    }
    if requested.contains(host_import::RULE) {
        hits.extend(host_import::collect(file)?);
    }
    if rdf::RULES.iter().any(|id| requested.contains(id)) {
        hits.extend(rdf::collect(file));
    }
    if structure::RULES.iter().any(|id| requested.contains(id)) {
        hits.extend(structure::collect(file));
        receivers = structure::receivers(file);
    }
    hits.retain(|hit| requested.contains(hit.rule));
    Ok(SyntaxHits { hits, receivers })
}

#[derive(Default)]
pub(crate) struct SyntaxHits {
    pub(crate) hits: Vec<RawHit>,
    pub(crate) receivers: Vec<(usize, ReceiverMode)>,
}
