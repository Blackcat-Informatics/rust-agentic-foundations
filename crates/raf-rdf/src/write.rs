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
    failure: Option<StorageError>,
}

impl<'o, 'a, O: Admission + ?Sized> Output<'o, 'a, O> {
    fn new(memory: &'o mut Memory<'a, O>) -> Self {
        Self {
            text: String::new(),
            memory,
            failure: None,
        }
    }
    fn status(&self) -> Result<(), StorageError> {
        self.failure.map_or(Ok(()), Err)
    }
    fn finish(self) -> Result<String, Error> {
        self.status()?;
        Ok(self.text)
    }
}

impl<O: Admission + ?Sized> fmt::Write for Output<'_, '_, O> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.push_str(text);
        self.status().map_err(|_| fmt::Error)
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
            self.failure = Some(error);
        }
        !self.failed()
    }
    fn push_str(&mut self, text: &str) {
        if self.failed() {
            return;
        }
        if let Err(error) = self.memory.push_str_with_progress(&mut self.text, text) {
            self.failure = Some(error);
        }
    }
    fn push(&mut self, character: char) {
        self.push_str(character.encode_utf8(&mut [0; 4]));
    }
    fn failed(&self) -> bool {
        self.failure.is_some()
    }
}

fn owned_iri<O: Admission + ?Sized>(base: &str, text: &str, output: &mut Output<'_, '_, O>) {
    output.push('<');
    purrdf_lex::iri_escape::push_escaped(base, output);
    purrdf_lex::iri_escape::push_escaped(text, output);
    output.push('>');
}

fn object<S: Admission + ?Sized, O: Admission + ?Sized>(
    value: &TermValue,
    base: Option<&str>,
    output: &mut Output<'_, '_, O>,
    scratch: &mut Memory<'_, S>,
) -> Result<(), Error> {
    let result = if let Some(base) = base {
        value
            .try_write_nested_with_memory(
                output,
                ["<<( ", " ", " )>>"],
                |output, leaf, memory| {
                    if let TermValue::Iri(iri) = leaf
                        && term::local(iri)
                    {
                        owned_iri(base, iri, output);
                        output.status()
                    } else {
                        purrdf_core::write_term_value_with_memory(leaf, output, memory)
                    }
                },
                |output, text, _| {
                    output.push_str(text);
                    output.status()
                },
                scratch,
            )
            .map_err(|error| match error {
                OwnedTermFoldError::Visitor(cause) | OwnedTermFoldError::Storage(cause) => cause,
            })
    } else {
        purrdf_core::write_term_value_with_memory(value, output, scratch)
    };
    // The destination may have failed before native scratch cleanup did.
    output.status()?;
    result?;
    Ok(())
}

fn body<S: Admission + ?Sized, O: Admission + ?Sized>(
    selection: &Selection<'_>,
    base: Option<&str>,
    output: &mut Output<'_, '_, O>,
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
        object(key(&selection.keys, row.o, scratch)?, base, output, scratch)?;
        output.push_str(" .\n");
        output.status()?;
    }
    Ok(())
}

fn normalized<S: Admission + ?Sized, O: Admission + ?Sized>(
    selection: &Selection<'_>,
    scratch: &mut Memory<'_, S>,
    output: &mut Memory<'_, O>,
) -> Result<String, Error> {
    let mut destination = Output::new(output);
    destination.push_str(VERSION);
    destination.status()?;
    body(selection, None, &mut destination, scratch)?;
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
    memory.admission_mut().checkpoint(1)?;
    Ok(memory.format_with_progress(&format_args!(
        "{}{}",
        vocab::BASE,
        purrdf_hash::hex::Lower(hasher.finalize().as_bytes())
    ))?)
}

fn selected<S: Admission + ?Sized, O: Admission + ?Sized>(
    selection: &Selection<'_>,
    format: Format,
    scratch: &mut Memory<'_, S>,
    output: &mut Memory<'_, O>,
) -> Result<(String, String), Error> {
    let mut normalized = normalized(selection, scratch, output)?;
    let base = identity(&normalized, scratch)?;
    let canonical = match format {
        Format::Turtle => {
            let directive = scratch.format_with_progress(&format_args!("BASE <{base}>\n"))?;
            let required = normalized
                .len()
                .checked_add(directive.len())
                .ok_or(StorageError::SizeOverflow)?;
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
            let mut destination = Output::new(output);
            body(selection, Some(&base), &mut destination, scratch)?;
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
    scratch.try_scope(|scratch| {
        output.try_scope(|output| {
            let selection = Selection::new(graph, scratch, output)?;
            let result = selected(&selection, format, scratch, output);
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
    let (mut working, mut destination) = (Resident, Resident);
    render_with_memory(
        graph.view(),
        format,
        &mut Memory::new(&mut working),
        &mut Memory::new(&mut destination),
    )
}

pub(crate) fn base(graph: &Graph) -> Result<String, Error> {
    render(graph, Format::Turtle).map(|(base, _)| base)
}
