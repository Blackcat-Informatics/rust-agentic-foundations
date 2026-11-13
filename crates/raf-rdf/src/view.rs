// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One borrowed semantic view for resident and originally admitted storage.

use crate::{Error, term};
use purrdf_core::{QuadIds, RdfDataset, TermId, TermRef};
use purrdf_lex::{
    allocation::{Admission, Memory, OrderedTable},
    control::compare_bytes,
};

#[derive(Clone, Copy)]
pub(crate) struct GraphView<'a> {
    pub(crate) dataset: &'a RdfDataset,
    pub(crate) owner: &'a str,
}

impl<'a> GraphView<'a> {
    pub(crate) fn rows(self) -> impl Iterator<Item = QuadIds> + 'a {
        self.dataset
            .quads()
            .chain(self.dataset.reifier_quads())
            .chain(self.dataset.annotation_quads())
    }
    pub(crate) fn local(self, text: &'a str) -> &'a str {
        term::owned_local(text, self.owner)
    }
    pub(crate) fn iri(self, id: TermId) -> Result<&'a str, Error> {
        match self.dataset.resolve(id) {
            TermRef::Iri(text) => Ok(text),
            _ => Err(Error::Invariant(
                "asserted subject or predicate is not an IRI",
            )),
        }
    }
    pub(crate) fn find<S: Admission + ?Sized>(
        self,
        subject: &str,
        memory: &mut Memory<'_, S>,
    ) -> Result<Option<TermId>, Error> {
        memory.admission_mut().checkpoint(1)?;
        let relative = term::local(subject);
        for row in self.rows() {
            memory.admission_mut().checkpoint(1)?;
            let TermRef::Iri(iri) = self.dataset.resolve(row.s) else {
                continue;
            };
            let candidate = if relative {
                memory
                    .admission_mut()
                    .checkpoint(iri.len().min(self.owner.len()).saturating_add(1) as u64)?;
                self.local(iri)
            } else {
                iri
            };
            if compare_bytes(subject.as_bytes(), candidate.as_bytes(), &mut |units| {
                memory.admission_mut().checkpoint(units)
            })?
            .is_eq()
            {
                return Ok(Some(row.s));
            }
        }
        Ok(None)
    }
}

impl GraphView<'_> {
    pub(crate) fn validate<S: Admission + ?Sized>(
        self,
        memory: &mut Memory<'_, S>,
    ) -> Result<(), Error> {
        use purrdf_core::DatasetView as _;
        memory.admission_mut().checkpoint(1)?;
        if self.dataset.named_graphs().next().is_some() {
            return Err(Error::Value(memory.string_with_progress(
                "named graphs are outside the report profile",
            )?));
        }
        if self.find("", memory)?.is_none() {
            return Err(Error::Value(
                memory.string_with_progress("the report root is absent")?,
            ));
        }
        let mut subjects = Vec::new();
        for row in self.rows() {
            memory.admission_mut().checkpoint(1)?;
            memory.push_copy_with_progress(&mut subjects, row.s)?;
        }
        purrdf_lex::walk::try_sort_unstable_by(&mut subjects, |a, b| {
            memory.admission_mut().checkpoint(1)?;
            Ok::<_, Error>(a.cmp(b))
        })?;
        let mut assertions = OrderedTable::default();
        for row in self.rows() {
            memory.admission_mut().checkpoint(1)?;
            let entry = assertions.prepare_insert_with_memory(
                (row.s, row.p, row.o, row.g),
                memory,
                |left, right, memory| {
                    memory.admission_mut().checkpoint(4)?;
                    Ok::<_, Error>(left.cmp(right))
                },
            )?;
            if entry.is_occupied() {
                return Err(Error::Repeated);
            }
            entry.publish(());
            let TermRef::Iri(subject) = self.dataset.resolve(row.s) else {
                return Err(Error::Value(
                    memory.string_with_progress("subject must be an IRI")?,
                ));
            };
            memory
                .admission_mut()
                .checkpoint(subject.len().min(self.owner.len()).saturating_add(1) as u64)?;
            let subject = self.local(subject);
            let TermRef::Iri(predicate) = self.dataset.resolve(row.p) else {
                return Err(Error::Value(
                    memory.string_with_progress("predicate must be an IRI")?,
                ));
            };
            let mut value =
                self.dataset
                    .term_value_with_memory(row.o, memory, Error::Lookup, |error, _| {
                        Error::Storage(error)
                    })?;
            term::normalize_with_memory(&mut value, self.owner, memory)?;
            term::validate_with_memory(subject, predicate, &value, memory)?;
            if let Err((remaining, error)) = value.release_with_memory(memory) {
                drop(remaining);
                return Err(error.into());
            }
            match self.dataset.resolve(row.o) {
                TermRef::Triple { s, o, .. } => {
                    self.check_reference(s, &subjects, memory)?;
                    self.check_reference(o, &subjects, memory)?;
                }
                _ => self.check_reference(row.o, &subjects, memory)?,
            }
        }
        memory.release_vec(subjects)?;
        assertions.release(memory)?;
        Ok(())
    }

    fn check_reference<S: Admission + ?Sized>(
        self,
        id: TermId,
        subjects: &[TermId],
        memory: &mut Memory<'_, S>,
    ) -> Result<(), Error> {
        let TermRef::Iri(text) = self.dataset.resolve(id) else {
            return Ok(());
        };
        memory
            .admission_mut()
            .checkpoint(text.len().min(self.owner.len()).saturating_add(1) as u64)?;
        let local = self.local(text);
        if !term::local(local) {
            return Ok(());
        }
        let (mut first, mut last) = (0, subjects.len());
        while first < last {
            memory.admission_mut().checkpoint(1)?;
            let at = first + (last - first) / 2;
            match id.cmp(&subjects[at]) {
                std::cmp::Ordering::Less => last = at,
                std::cmp::Ordering::Greater => first = at + 1,
                std::cmp::Ordering::Equal => return Ok(()),
            }
        }
        Err(Error::Value(memory.format_with_progress(&format_args!(
            "dangling owned node {local}"
        ))?))
    }
}
