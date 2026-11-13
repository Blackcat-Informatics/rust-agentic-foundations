// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Independent admitted RDF evidence. Reading claims never constructs source proof.

mod read;
mod write;

use super::{
    output::{Failure, OutputFailure},
    resource::{Meter, Text},
    schema::Graph,
};
use crate::report::{ReportError as E, mint};

pub(super) const VOCAB: &str = "https://blackcatinformatics.ca/raf/census/embedded/1#";
const FORM: usize = 1;
pub(super) const PRELUDE_PROFILE: &str = "rust-declared-prelude-values-v1";
const MAX_INPUT_BYTES: usize = 16 * 1024 * 1024;

/// Imported shape-checked claims, not an Analysis, Matches or binding proof.
pub(super) struct Claims<'m> {
    graph: Graph<'m>,
}
impl<'m> Claims<'m> {
    pub(super) fn produce<R>(
        matches: &'m super::pairing::Matches<'_>,
        observe: impl for<'f> FnOnce(Result<Self, OutputFailure<'f>>) -> R,
    ) -> R {
        write::graph(matches, |outcome| match outcome {
            Ok(graph) => Self::from_graph(graph, matches.analysis.meter(), observe),
            Err(cause) => observe(Err(cause)),
        })
    }
    pub(super) fn from_graph<R>(
        graph: Graph<'m>,
        meter: &'m Meter,
        observe: impl for<'f> FnOnce(Result<Self, OutputFailure<'f>>) -> R,
    ) -> R {
        match read::validate(&graph, meter) {
            Ok(()) => observe(Ok(Self { graph })),
            Err(cause) => observe(Err(OutputFailure::failure(&cause))),
        }
    }
    pub(super) fn from_text<R>(
        text: &str,
        format: raf_rdf::Format,
        meter: &'m Meter,
        observe: impl for<'f> FnOnce(Result<Self, OutputFailure<'f>>) -> R,
    ) -> R {
        if text.len() > MAX_INPUT_BYTES {
            let cause = Failure::report(
                meter,
                E::Value,
                &"embedded report exceeds its input byte limit",
            );
            return observe(Err(OutputFailure::failure(&cause)));
        }
        raf_rdf::AdmittedGraph::import_captured(
            text,
            format,
            || meter.native(),
            meter.progress(),
            meter.capture(),
            |outcome| match outcome {
                Ok(graph) => Self::from_graph(graph, meter, observe),
                Err(cause) => observe(Err(OutputFailure::import(&cause))),
            },
        )
    }
    pub(super) fn to_text_admitted<R>(
        &self,
        format: raf_rdf::Format,
        meter: &'m Meter,
        observe: impl for<'f> FnOnce(Result<Text<'m>, OutputFailure<'f>>) -> R,
    ) -> R {
        Text::render_admitted(meter, &self.graph, format, observe)
    }
}

pub(super) fn key_with_memory<'a, S: purrdf_lex::allocation::Admission + ?Sized>(
    kind: &str,
    parts: impl IntoIterator<Item = &'a str>,
    memory: &mut purrdf_lex::allocation::Memory<'_, S>,
) -> Result<String, purrdf_lex::allocation::StorageError> {
    let mut name = memory.string_with_progress("#")?;
    memory.push_str_with_progress(&mut name, kind)?;
    for part in parts {
        memory.push_str_with_progress(&mut name, "/")?;
        let encoded = mint::part_with_memory(part, memory)?;
        let result = memory.push_str_with_progress(&mut name, &encoded);
        if let std::borrow::Cow::Owned(encoded) = encoded {
            memory.release_string(encoded)?;
        }
        result?;
    }
    Ok(name)
}

const OWNER_TEXT: &[&str] = &[
    "manifest",
    "package",
    "targetRoot",
    "targetKind",
    "targetName",
    "cratePath",
    "symbol",
];
const OWNER_NUMBERS: &[&str] = &[
    "manifestAuthority",
    "targetRootAuthority",
    "itemStartByte",
    "itemEndByte",
];

#[cfg(test)]
pub(in crate::embedded) mod tests;
