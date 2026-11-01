// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Built-in syntax recognition; source ownership is assigned by the gate.

mod format;
mod hasher;
mod hex;
mod layout;
mod names;
mod rdf;

#[cfg(test)]
mod tests;

pub(crate) fn ids() -> impl Iterator<Item = &'static str> {
    layout::RULES
        .into_iter()
        .chain(hex::RULES)
        .chain(["hex-table", "json-pointer-escape"])
        .chain(["home-literal", "sole-name", "vocabulary-literal"])
        .chain(["std-default-hasher"])
        .chain(rdf::RULES)
}

pub(crate) fn implements(id: &str) -> bool {
    ids().any(|candidate| candidate == id)
}

pub(crate) fn home_exempt(id: &str) -> bool {
    implements(id) && id != "sole-name" && !rdf::RULES.contains(&id)
}

pub(crate) fn needs_syntax(id: &str) -> bool {
    layout::RULES.contains(&id)
        || hex::RULES.contains(&id)
        || id == "std-default-hasher"
        || rdf::RULES.contains(&id)
}

pub(crate) fn shipping_only(id: &str) -> bool {
    implements(id)
        && !matches!(
            id,
            "std-default-hasher"
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
                    message:
                        "a JSON Pointer escape written by hand; use the approved pointer encoder"
                            .into(),
                });
            }
        }
    }
    hits
}

pub(crate) struct RawHit {
    pub(crate) rule: &'static str,
    pub(crate) byte: usize,
    pub(crate) line: usize,
    pub(crate) message: String,
}

impl RawHit {
    fn push(hits: &mut Vec<Self>, rule: &'static str, span: proc_macro2::Span, message: &str) {
        hits.push(Self {
            rule,
            byte: span.byte_range().start,
            line: span.start().line,
            message: message.into(),
        });
    }
}

pub(crate) fn collect(
    file: &syn::File,
    requested: &std::collections::BTreeSet<&str>,
) -> Vec<RawHit> {
    let mut hits = Vec::new();
    if layout::RULES.iter().any(|id| requested.contains(id)) {
        hits.extend(layout::collect(file));
    }
    if hex::RULES.iter().any(|id| requested.contains(id)) {
        hits.extend(hex::collect(file));
    }
    if requested.contains("std-default-hasher") {
        hits.extend(hasher::collect(file));
    }
    if rdf::RULES.iter().any(|id| requested.contains(id)) {
        hits.extend(rdf::collect(file));
    }
    hits.retain(|hit| requested.contains(hit.rule));
    hits
}
