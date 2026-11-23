// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One admitted normalized body and native RDF term emitter supply both formats.

use crate::view::GraphView;
use crate::{Error, Format, Graph, TermValue, term, vocab};
use purrdf_core::{DatasetView, OwnedTermFoldError, QuadIds, TermId};
use purrdf_lex::{
    allocation::{Admission, Memory, Resident, StorageError},
    control::{ControlledError, compare_bytes},
    text_out::TextOut,
};
use std::{cmp::Ordering, fmt};

const VERSION: &str = "VERSION \"1.2\"\n";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Spelling {
    Current,
    PurRdf4,
}

struct Selection<'a> {
    graph: GraphView<'a>,
    rows: Vec<QuadIds>,
    keys: Vec<(TermId, Option<TermValue>)>,
}

impl<'a> Selection<'a> {
    fn new<S: Admission + ?Sized, O: Admission + ?Sized>(
        graph: GraphView<'a>,
        scratch: &mut Memory<'_, S>,
        output: &mut Memory<'_, O>,
    ) -> Result<Self, Error> {
        let mut rows = Vec::new();
        let mut keys = Vec::new();
        for row in graph.rows() {
            scratch.admission_mut().checkpoint(1)?;
            scratch.push_copy_with_progress(&mut rows, row)?;
            if keys.len() == keys.capacity() {
                scratch.admission_mut().checkpoint(keys.len() as u64)?;
            }
            scratch.push(&mut keys, (row.o, None))?;
        }
        purrdf_lex::walk::try_sort_unstable_by(&mut keys, |a, b| {
            scratch.admission_mut().checkpoint(1)?;
            Ok::<_, StorageError>(a.0.cmp(&b.0))
        })?;
        scratch.admission_mut().checkpoint(keys.len() as u64)?;
        purrdf_lex::walk::try_dedup_by(&mut keys, |a, b| {
            scratch.admission_mut().checkpoint(1)?;
            Ok::<_, StorageError>(a.0 == b.0)
        })?;
        for (id, value) in &mut keys {
            scratch.admission_mut().checkpoint(1)?;
            let mut owned =
                graph
                    .dataset
                    .term_value_with_memory(*id, scratch, Error::Lookup, |error, _| {
                        Error::Storage(error)
                    })?;
            term::normalize_with_memory(&mut owned, graph.owner, scratch)?;
            *value = Some(owned);
        }
        purrdf_lex::walk::try_sort_unstable_by(&mut rows, |a, b| {
            compare_rows(graph, &keys, a, b, scratch, output)
        })?;
        Ok(Self { graph, rows, keys })
    }

    fn release<S: Admission + ?Sized>(mut self, memory: &mut Memory<'_, S>) -> Result<(), Error> {
        while let Some((_, value)) = self.keys.pop() {
            if let Some(value) = value
                && let Err((remaining, error)) = value.release_with_memory(memory)
            {
                drop(remaining);
                return Err(error.into());
            }
        }
        memory.release_vec(self.keys)?;
        memory.release_vec(self.rows)?;
        Ok(())
    }
}

fn iri(graph: GraphView<'_>, id: TermId) -> Result<&str, Error> {
    graph.iri(id)
}

fn local<'a, S: Admission + ?Sized>(
    graph: GraphView<'_>,
    text: &'a str,
    memory: &mut Memory<'_, S>,
) -> Result<&'a str, Error> {
    memory
        .admission_mut()
        .checkpoint(text.len().min(graph.owner.len()).saturating_add(1) as u64)?;
    Ok(term::owned_local(text, graph.owner))
}

fn key<'a, S: Admission + ?Sized>(
    keys: &'a [(TermId, Option<TermValue>)],
    id: TermId,
    memory: &mut Memory<'_, S>,
) -> Result<&'a TermValue, Error> {
    let (mut first, mut last) = (0, keys.len());
    while first < last {
        memory.admission_mut().checkpoint(1)?;
        let middle = first + (last - first) / 2;
        match id.cmp(&keys[middle].0) {
            Ordering::Less => last = middle,
            Ordering::Greater => first = middle + 1,
            Ordering::Equal => {
                return keys[middle].1.as_ref().ok_or(Error::Invariant(
                    "rendering key has no admitted native value",
                ));
            }
        }
    }
    Err(Error::Invariant("rendered object has no admitted key"))
}

fn compare_rows<S: Admission + ?Sized, O: Admission + ?Sized>(
    graph: GraphView<'_>,
    keys: &[(TermId, Option<TermValue>)],
    left: &QuadIds,
    right: &QuadIds,
    scratch: &mut Memory<'_, S>,
    output: &mut Memory<'_, O>,
) -> Result<Ordering, Error> {
    scratch.admission_mut().checkpoint(1)?;
    let a = local(graph, iri(graph, left.s)?, scratch)?;
    let b = local(graph, iri(graph, right.s)?, scratch)?;
    let mut progress = |units| output.admission_mut().checkpoint(units);
    let order = compare_bytes(a.as_bytes(), b.as_bytes(), &mut progress)?;
    if !order.is_eq() {
        return Ok(order);
    }
    let order = compare_bytes(
        iri(graph, left.p)?.as_bytes(),
        iri(graph, right.p)?.as_bytes(),
        &mut progress,
    )?;
    if !order.is_eq() {
        return Ok(order);
    }
    key(keys, left.o, scratch)?
        .try_cmp_with_control(key(keys, right.o, scratch)?, scratch, &mut progress)
        .map_err(|error| match error {
            ControlledError::Native(cause) | ControlledError::Control(cause) => {
                Error::Storage(cause)
            }
        })
}

/// Native emitters retain their exact spelling; this adapter owns only growth
/// and the first refusal of their original destination account.
struct Output<'o, 'a, O: Admission + ?Sized> {
    text: String,
    memory: &'o mut Memory<'a, O>,
    failure: Option<Error>,
    limit: Option<usize>,
}

impl<'o, 'a, O: Admission + ?Sized> Output<'o, 'a, O> {
    fn new(memory: &'o mut Memory<'a, O>, limit: Option<usize>) -> Self {
        Self {
            text: String::new(),
            memory,
            failure: None,
            limit,
        }
    }
    fn finish(self) -> Result<String, Error> {
        self.status()?;
        Ok(self.text)
    }
}

trait Destination: TextOut {
    fn status(&self) -> Result<(), Error>;
}

impl<O: Admission + ?Sized> Destination for Output<'_, '_, O> {
    fn status(&self) -> Result<(), Error> {
        self.failure.clone().map_or(Ok(()), Err)
    }
}

impl<O: Admission + ?Sized> fmt::Write for Output<'_, '_, O> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.push_str(text);
        if self.failed() {
            Err(fmt::Error)
        } else {
            Ok(())
        }
    }
}

impl<O: Admission + ?Sized> TextOut for Output<'_, '_, O> {
    fn scan_limit(&self) -> usize {
        if self.memory.admission().progress_enabled() {
            256
        } else {
            usize::MAX
        }
    }
    fn checkpoint(&mut self, bytes: usize) -> bool {
        if self.failed() {
            return false;
        }
        if let Err(error) = self.memory.admission_mut().checkpoint(bytes as u64) {
            self.failure = Some(error.into());
        }
        !self.failed()
    }
    fn push_str(&mut self, text: &str) {
        if self.failed() {
            return;
        }
        if let Some(max_bytes) = self.limit
            && text.len() > max_bytes.saturating_sub(self.text.len())
        {
            self.failure = Some(Error::OutputLimit { max_bytes });
            return;
        }
        if let Err(error) = self.memory.push_str_with_progress(&mut self.text, text) {
            self.failure = Some(error.into());
        }
    }
    fn push(&mut self, character: char) {
        self.push_str(character.encode_utf8(&mut [0; 4]));
    }
    fn failed(&self) -> bool {
        self.failure.is_some()
    }
}

struct DigestOutput(purrdf_hash::blake3::Hasher);

impl fmt::Write for DigestOutput {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.push_str(text);
        Ok(())
    }
}

impl TextOut for DigestOutput {
    fn push_str(&mut self, text: &str) {
        self.0.update(text.as_bytes());
    }

    fn push(&mut self, character: char) {
        self.push_str(character.encode_utf8(&mut [0; 4]));
    }
}

impl Destination for DigestOutput {
    fn status(&self) -> Result<(), Error> {
        Ok(())
    }
}

fn owned_iri(base: &str, text: &str, output: &mut impl Destination) {
    output.push('<');
    purrdf_lex::iri_escape::push_escaped(base, output);
    purrdf_lex::iri_escape::push_escaped(text, output);
    output.push('>');
}

fn object<S: Admission + ?Sized>(
    value: &TermValue,
    base: Option<&str>,
    spelling: Spelling,
    output: &mut impl Destination,
    scratch: &mut Memory<'_, S>,
) -> Result<(), Error> {
    let result = if base.is_some() || spelling == Spelling::PurRdf4 {
        value
            .try_write_nested_with_memory(
                output,
                ["<<( ", " ", " )>>"],
                |output, leaf, memory| match (leaf, base, spelling) {
                    (TermValue::Iri(iri), Some(base), _) if term::local(iri) => {
                        owned_iri(base, iri, output);
                        output.status()
                    }
                    (
                        TermValue::Literal {
                            lexical_form,
                            datatype,
                            language,
                            direction,
                        },
                        _,
                        Spelling::PurRdf4,
                    ) => {
                        purrdf_lex::term_syntax::write_literal_rdfc12_v3(
                            lexical_form,
                            datatype,
                            language.as_deref(),
                            direction.map(purrdf_core::RdfTextDirection::as_str),
                            output,
                        );
                        output.status()
                    }
                    _ => purrdf_core::write_term_value_with_memory(leaf, output, memory)
                        .map_err(Error::from),
                },
                |output, text, _| {
                    output.push_str(text);
                    output.status()
                },
                scratch,
            )
            .map_err(|error| match error {
                OwnedTermFoldError::Visitor(cause) => cause,
                OwnedTermFoldError::Storage(cause) => cause.into(),
            })
    } else {
        purrdf_core::write_term_value_with_memory(value, output, scratch).map_err(Error::from)
    };
    // The destination may have failed before native scratch cleanup did.
    output.status()?;
    result?;
    Ok(())
}

fn body<S: Admission + ?Sized>(
    selection: &Selection<'_>,
    base: Option<&str>,
    spelling: Spelling,
    output: &mut impl Destination,
    scratch: &mut Memory<'_, S>,
) -> Result<(), Error> {
    for row in &selection.rows {
        scratch.admission_mut().checkpoint(1)?;
        let subject = local(selection.graph, iri(selection.graph, row.s)?, scratch)?;
        if let Some(base) = base {
            owned_iri(base, subject, output);
        } else {
            purrdf_lex::term_syntax::write_iri(subject, output);
        }
        output.push(' ');
        purrdf_lex::term_syntax::write_iri(iri(selection.graph, row.p)?, output);
        output.push(' ');
        output.status()?;
        object(
            key(&selection.keys, row.o, scratch)?,
            base,
            spelling,
            output,
            scratch,
        )?;
        output.push_str(" .\n");
        output.status()?;
    }
    Ok(())
}

fn normalized<S: Admission + ?Sized, O: Admission + ?Sized>(
    selection: &Selection<'_>,
    spelling: Spelling,
    scratch: &mut Memory<'_, S>,
    output: &mut Memory<'_, O>,
    limit: Option<usize>,
) -> Result<String, Error> {
    let mut destination = Output::new(output, limit);
    destination.push_str(VERSION);
    destination.status()?;
    body(selection, None, spelling, &mut destination, scratch)?;
    destination.finish()
}

fn identity<S: Admission + ?Sized>(
    text: &str,
    memory: &mut Memory<'_, S>,
) -> Result<String, Error> {
    let mut hasher = purrdf_hash::blake3::Hasher::new();
    for extent in text.as_bytes().chunks(256) {
        memory.admission_mut().checkpoint(extent.len() as u64)?;
        hasher.update(extent);
    }
    digest_iri(&hasher, memory)
}

fn digest_iri<S: Admission + ?Sized>(
    hasher: &purrdf_hash::blake3::Hasher,
    memory: &mut Memory<'_, S>,
) -> Result<String, Error> {
    memory.admission_mut().checkpoint(1)?;
    let digest = hasher.finalize();
    Ok(memory.format_with_progress(&format_args!(
        "{}{}",
        vocab::BASE,
        purrdf_hash::hex::Lower(digest.as_bytes())
    ))?)
}

fn selected<S: Admission + ?Sized, O: Admission + ?Sized>(
    selection: &Selection<'_>,
    format: Format,
    spelling: Spelling,
    scratch: &mut Memory<'_, S>,
    output: &mut Memory<'_, O>,
    limit: Option<usize>,
) -> Result<(String, String), Error> {
    if let (Format::NTriples, Some(max_bytes)) = (format, limit) {
        let mut digest = DigestOutput(purrdf_hash::blake3::Hasher::new());
        digest.push_str(VERSION);
        body(selection, None, spelling, &mut digest, scratch)?;
        let base = digest_iri(&digest.0, scratch)?;
        let mut destination = Output::new(output, Some(max_bytes));
        body(selection, Some(&base), spelling, &mut destination, scratch)?;
        return Ok((base, destination.finish()?));
    }
    let mut normalized = normalized(selection, spelling, scratch, output, limit)?;
    let base = identity(&normalized, scratch)?;
    let canonical = match format {
        Format::Turtle => {
            let directive = scratch.format_with_progress(&format_args!("BASE <{base}>\n"))?;
            let required = normalized
                .len()
                .checked_add(directive.len())
                .ok_or(StorageError::SizeOverflow)?;
            if let Some(max_bytes) = limit
                && required > max_bytes
            {
                return Err(Error::OutputLimit { max_bytes });
            }
            if required > normalized.capacity() {
                output.admission_mut().checkpoint(normalized.len() as u64)?;
            }
            output.reserve_string(&mut normalized, required)?;
            output.admission_mut().checkpoint(required as u64)?;
            normalized.insert_str(VERSION.len(), &directive);
            scratch.release_string(directive)?;
            normalized
        }
        Format::NTriples => {
            output.release_string(normalized)?;
            let mut destination = Output::new(output, None);
            body(selection, Some(&base), spelling, &mut destination, scratch)?;
            destination.finish()?
        }
    };
    Ok((base, canonical))
}

pub(crate) fn render_with_memory<S: Admission + ?Sized, O: Admission + ?Sized>(
    graph: GraphView<'_>,
    format: Format,
    scratch: &mut Memory<'_, S>,
    output: &mut Memory<'_, O>,
) -> Result<(String, String), Error> {
    render_admitted(graph, format, Spelling::Current, scratch, output, None)
}

fn render_admitted<S: Admission + ?Sized, O: Admission + ?Sized>(
    graph: GraphView<'_>,
    format: Format,
    spelling: Spelling,
    scratch: &mut Memory<'_, S>,
    output: &mut Memory<'_, O>,
    limit: Option<usize>,
) -> Result<(String, String), Error> {
    scratch.try_scope(|scratch| {
        output.try_scope(|output| {
            let selection = Selection::new(graph, scratch, output)?;
            let result = selected(&selection, format, spelling, scratch, output, limit);
            match result {
                Err(error) => {
                    // Destroy failed owners without polling more work or
                    // replacing their first cause. The enclosing original
                    // scratch scope refunds their delta after destruction.
                    drop(selection);
                    Err(error)
                }
                Ok(value) => {
                    selection.release(scratch)?;
                    Ok(value)
                }
            }
        })
    })
}

pub(crate) fn text_with_memory<S: Admission + ?Sized, O: Admission + ?Sized>(
    graph: &Graph,
    format: Format,
    scratch: &mut Memory<'_, S>,
    output: &mut Memory<'_, O>,
) -> Result<String, Error> {
    text_with_view(graph.view(), format, scratch, output)
}

pub(crate) fn text_with_view<S: Admission + ?Sized, O: Admission + ?Sized>(
    graph: GraphView<'_>,
    format: Format,
    scratch: &mut Memory<'_, S>,
    output: &mut Memory<'_, O>,
) -> Result<String, Error> {
    scratch.try_scope(|scratch| {
        output.try_scope(|output| {
            let (base, text) = render_with_memory(graph, format, scratch, output)?;
            scratch.release_string(base)?;
            Ok(text)
        })
    })
}

pub(crate) fn render(graph: &Graph, format: Format) -> Result<(String, String), Error> {
    render_spelling(graph, format, Spelling::Current)
}

pub(crate) fn render_spelling(
    graph: &Graph,
    format: Format,
    spelling: Spelling,
) -> Result<(String, String), Error> {
    let (mut working, mut destination) = (Resident, Resident);
    render_admitted(
        graph.view(),
        format,
        spelling,
        &mut Memory::new(&mut working),
        &mut Memory::new(&mut destination),
        None,
    )
}

pub(crate) fn render_bounded(
    graph: &Graph,
    format: Format,
    max_bytes: usize,
) -> Result<String, Error> {
    let (mut scratch, mut output) = (Resident, Resident);
    render_admitted(
        graph.view(),
        format,
        Spelling::Current,
        &mut Memory::new(&mut scratch),
        &mut Memory::new(&mut output),
        Some(max_bytes),
    )
    .map(|(_, text)| text)
}

pub(crate) fn base(graph: &Graph) -> Result<String, Error> {
    render(graph, Format::Turtle).map(|(base, _)| base)
}
