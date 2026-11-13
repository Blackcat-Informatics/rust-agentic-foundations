// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Retained native parsing, admitted physical replay and canonical acceptance.

use crate::{AdmittedGraph, BuildCause, BuildFailure, Error, Format, vocab, write};
use purrdf_core::{DatasetView, RdfDatasetBuilder, TermGuard, TermId, TermRef};
use purrdf_lex::{
    allocation::{Admission, Memory, OrderedTable, OwnedAdmission},
    control::{ControlledError, Progress, compare_bytes},
};
use purrdf_rdf::{BorrowedReadError, NativeRdfFormat, ParseRefusal};
use std::{
    cell::{Cell, RefCell},
    fmt,
};

/// A borrowed refusal observed while every native payload and grant still lives.
pub enum ImportFailure<'f, 'input, A: Admission, P: Progress, F: Copy> {
    /// The unchanged native parser refusal, including its original source owner.
    Parse {
        /// The original native refusal; no partial dataset is exposed.
        cause: &'f ParseRefusal<'input, A, P>,
        /// Original caller admission refusal, captured before observation.
        captured: Option<F>,
    },
    /// The unchanged owned construction refusal under its original account.
    Build(&'f BuildFailure<A, F>),
    /// A closed-profile, identity or canonical-byte refusal under live grants.
    Profile {
        /// The original structured refusal.
        cause: &'f BuildCause,
        /// Original caller admission refusal, captured before observation.
        captured: Option<F>,
    },
}

impl<A: Admission, P: Progress, F: Copy> ImportFailure<'_, '_, A, P, F> {
    /// Borrow the first original caller refusal without replacing native causes.
    #[must_use]
    pub fn captured(&self) -> Option<F> {
        match self {
            Self::Parse { captured, .. } | Self::Profile { captured, .. } => *captured,
            Self::Build(cause) => cause.captured(),
        }
    }
}

impl<A: Admission, P: Progress, F: Copy> fmt::Display for ImportFailure<'_, '_, A, P, F>
where
    P::Error: fmt::Display,
{
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse { cause, .. } => cause.fmt(output),
            Self::Build(cause) => cause.fmt(output),
            Self::Profile { cause, .. } => cause.fmt(output),
        }
    }
}

impl<A: Admission> AdmittedGraph<A> {
    /// Parse and observe a canonical report under fresh original subaccounts.
    /// All supplied subaccounts and progress must charge the same authority.
    /// The observer receives failures only while their original owners live;
    /// a successful graph owns its independent construction subaccount and does
    /// not retain the input's lifetime.
    ///
    /// Canonical absolute N-Triples uses the native controlled Turtle grammar,
    /// followed by its own root law and exact N-Triples canonical-byte equality.
    /// This does not admit general N-Triples as equivalent Turtle input.
    ///
    /// # Panics
    /// Panics if a successful native parse omits its promised dataset.
    pub fn import_captured<'input, P: Progress, F: Copy, R>(
        text: &'input str,
        format: Format,
        mut fresh: impl FnMut() -> A,
        progress: P,
        capture: &Cell<Option<F>>,
        observe: impl for<'f> FnOnce(Result<Self, ImportFailure<'f, 'input, A, P, F>>) -> R,
    ) -> R {
        let parsed = match purrdf_rdf::parse_dataset_controlled(
            text.as_bytes(),
            NativeRdfFormat::Turtle,
            None,
            fresh(),
            progress,
        ) {
            Ok(parsed) => parsed,
            Err(cause) => {
                let captured = capture.get();
                return observe(Err(ImportFailure::Parse {
                    cause: &cause,
                    captured,
                }));
            }
        };
        let mut scratch_account = OwnedAdmission::new(fresh());
        let mut output_account = OwnedAdmission::new(fresh());
        let mut scratch = Memory::new(&mut scratch_account);
        let mut output = Memory::new(&mut output_account);
        let dataset = parsed
            .dataset()
            .expect("successful native parse retains its dataset");
        let base = match selected_base(&dataset, parsed.document_base_iri(), format, &mut scratch) {
            Ok(base) => base,
            Err(cause) => {
                let captured = capture.get();
                return observe(Err(ImportFailure::Profile {
                    cause: &cause,
                    captured,
                }));
            }
        };
        let graph = match Self::build_owned_captured(fresh(), &base, capture, |construction| {
            construction.replay(&dataset)
        }) {
            Ok(graph) => graph,
            Err(cause) => return observe(Err(ImportFailure::Build(&cause))),
        };
        let validation = (|| {
            let (computed, canonical) =
                write::render_with_memory(graph.view(), format, &mut scratch, &mut output)?;
            if !compare_bytes(base.as_bytes(), computed.as_bytes(), &mut |units| {
                scratch.admission_mut().checkpoint(units)
            })?
            .is_eq()
            {
                return Err(BuildCause::Profile(Error::Digest {
                    declared: base,
                    computed,
                }));
            }
            let mut offset = 0;
            let common = canonical.len().min(text.len());
            while offset < common {
                let end = offset.saturating_add(256).min(common);
                scratch.admission_mut().checkpoint((end - offset) as u64)?;
                let matched = purrdf_deflate::common_prefix_len(
                    &canonical.as_bytes()[offset..end],
                    &text.as_bytes()[offset..end],
                );
                let complete = matched == end - offset;
                offset += matched;
                if !complete {
                    break;
                }
            }
            if offset != canonical.len() || offset != text.len() {
                let position = purrdf_iri::PositionCursor::new(text)
                    .advance(offset, &mut scratch)
                    .map_err(|cause| match cause {
                        ControlledError::Native(cause) => BuildCause::Position(cause),
                        ControlledError::Control(cause) => cause.into(),
                    })?;
                let line = usize::try_from(position.line).map_err(|_| {
                    BuildCause::Profile(Error::Invariant("native line exceeds usize"))
                })?;
                return Err(BuildCause::Profile(Error::NotCanonical { line }));
            }
            Ok::<(), BuildCause>(())
        })();
        match validation {
            Ok(()) => observe(Ok(graph)),
            Err(cause) => {
                let captured = capture.get();
                observe(Err(ImportFailure::Profile {
                    cause: &cause,
                    captured,
                }))
            }
        }
    }
}

fn snapshot<S: Admission + ?Sized>(text: &str, memory: &mut Memory<'_, S>) -> Result<bool, Error> {
    memory
        .admission_mut()
        .checkpoint(text.len().min(vocab::BASE.len()).saturating_add(1) as u64)?;
    let Some(hex) = text.strip_prefix(vocab::BASE) else {
        return Ok(false);
    };
    memory.admission_mut().checkpoint(1)?;
    if hex.len() != 64 {
        return Ok(false);
    }
    memory.admission_mut().checkpoint(hex.len() as u64)?;
    Ok(purrdf_hash::hex::decode_32_canonical(hex).is_some())
}

fn selected_base<
    D: DatasetView<Id = TermId, ReadError = BorrowedReadError>,
    S: Admission + ?Sized,
>(
    dataset: &D,
    directive: Option<&str>,
    format: Format,
    memory: &mut Memory<'_, S>,
) -> Result<String, BuildCause> {
    if format == Format::Turtle {
        let Some(base) = directive else {
            return Err(Error::Value(
                memory.string_with_progress("snapshot BASE directive is absent")?,
            )
            .into());
        };
        if !snapshot(base, memory)? {
            return Err(Error::Value(memory.string_with_progress(
                "snapshot BASE requires its namespace and 64 lowercase hexadecimal digits",
            )?)
            .into());
        }
        return Ok(memory.string_with_progress(base)?);
    }
    let mut root: Option<String> = None;
    for row in dataset
        .quads()
        .chain(dataset.reifier_quads())
        .chain(dataset.annotation_quads())
    {
        memory.admission_mut().checkpoint(1)?;
        let guard = dataset.resolve(row.s).map_err(BuildCause::Read)?;
        let TermRef::Iri(iri) = guard.term() else {
            continue;
        };
        if !snapshot(iri, memory)? {
            continue;
        }
        if let Some(root) = &root {
            if !compare_bytes(root.as_bytes(), iri.as_bytes(), &mut |units| {
                memory.admission_mut().checkpoint(units)
            })?
            .is_eq()
            {
                return Err(Error::Value(
                    memory.string_with_progress("more than one snapshot root")?,
                )
                .into());
            }
        } else {
            root = Some(memory.string_with_progress(iri)?);
        }
    }
    root.map_or_else(
        || {
            Err(
                Error::Value(memory.string_with_progress("absolute snapshot root is absent")?)
                    .into(),
            )
        },
        Ok,
    )
}

pub(crate) fn replay<
    D: DatasetView<Id = TermId, ReadError = BorrowedReadError>,
    S: Admission + ?Sized,
>(
    dataset: &D,
    native: &mut RdfDatasetBuilder,
    memory: &mut Memory<'_, S>,
) -> Result<(), BuildCause> {
    memory.admission_mut().checkpoint(1)?;
    if dataset.named_graphs().next().is_some() {
        return Err(Error::Value(
            memory.string_with_progress("named graphs are outside the report profile")?,
        )
        .into());
    }
    let mut memo = OrderedTable::default();
    for (kind, row) in dataset
        .quads()
        .map(|row| (0, row))
        .chain(dataset.reifier_quads().map(|row| (1, row)))
        .chain(dataset.annotation_quads().map(|row| (2, row)))
    {
        memory.admission_mut().checkpoint(1)?;
        let s = replay_term(dataset, row.s, native, &mut memo, memory)?;
        let p = replay_term(dataset, row.p, native, &mut memo, memory)?;
        let o = replay_term(dataset, row.o, native, &mut memo, memory)?;
        match kind {
            0 => {
                native.push_quad_with_memory(s, p, o, None, memory)?;
            }
            1 => {
                native.push_reifier_in_graph_with_memory(s, o, None, memory)?;
            }
            _ => native.push_annotation_in_graph_with_memory(s, p, o, None, memory)?,
        }
    }
    memo.release(memory)?;
    Ok(())
}

fn replay_term<
    D: DatasetView<Id = TermId, ReadError = BorrowedReadError>,
    S: Admission + ?Sized,
>(
    dataset: &D,
    id: TermId,
    native: &mut RdfDatasetBuilder,
    memo: &mut OrderedTable<TermId, TermId>,
    memory: &mut Memory<'_, S>,
) -> Result<TermId, BuildCause> {
    let found = memo
        .try_get_by(|key| {
            memory.admission_mut().checkpoint(1)?;
            Ok::<_, BuildCause>(id.cmp(key))
        })?
        .copied();
    if let Some(found) = found {
        return Ok(found);
    }
    let native = RefCell::new(native);
    let result = purrdf_core::ir::try_fold_term_with_memory(
        dataset,
        id,
        |_, value, memory| {
            let native = &mut **native.borrow_mut();
            let mapped = match value {
                TermRef::Iri(text) => native.intern_iri_with_memory(text, memory)?,
                TermRef::Blank { label, scope } => {
                    native.intern_blank_with_memory(label, scope, memory)?
                }
                TermRef::Literal {
                    lexical,
                    datatype,
                    language,
                    direction,
                } => {
                    let guard = dataset.resolve(datatype).map_err(BuildCause::Read)?;
                    let TermRef::Iri(datatype) = guard.term() else {
                        return Err(
                            Error::Invariant("native literal datatype is not an IRI").into()
                        );
                    };
                    native.intern_literal_parts_with_memory(
                        lexical,
                        Some(datatype),
                        language,
                        direction,
                        memory,
                    )?
                }
                TermRef::Triple { .. } => {
                    return Err(Error::Invariant("native fold exposed triple as leaf").into());
                }
            };
            Ok(mapped)
        },
        |_, s, p, o, memory| {
            Ok(native
                .borrow_mut()
                .intern_triple_with_memory(s, p, o, memory)?)
        },
        BuildCause::Read,
        |_, _| {},
        |error, _| BuildCause::from(error),
        memory,
    )?;
    let entry = memo.prepare_insert_with_memory(id, memory, |left, right, memory| {
        memory.admission_mut().checkpoint(1)?;
        Ok::<_, BuildCause>(left.cmp(right))
    })?;
    entry.publish(result);
    Ok(result)
}
