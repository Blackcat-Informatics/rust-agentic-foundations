// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Fallible raw physical-row queries without native index construction.

use crate::{AdmittedGraph, Error, TermValue, term, view::GraphView};
use purrdf_core::{DatasetView as _, TermId, TermRef};
use purrdf_lex::{
    allocation::{Admission, Memory},
    control::compare_bytes,
};

/// A borrowed asserted subject of originally admitted immutable storage.
#[derive(Clone, Copy)]
pub struct AdmittedNode<'a> {
    view: GraphView<'a>,
    id: TermId,
}

/// A borrowed term of originally admitted immutable storage.
#[derive(Clone, Copy)]
pub struct AdmittedValue<'a> {
    view: GraphView<'a>,
    id: TermId,
}

impl<A: Admission> AdmittedGraph<A> {
    /// Visit physical records while lending the same original query memory to
    /// each callback. This supports nested fallible queries without an index or
    /// a copied assertion collection. Subjects and properties may recur.
    /// # Errors
    /// Returns the first native, admission or unchanged callback refusal.
    pub fn try_for_each_record_with_memory<'a, S: Admission + ?Sized, E: From<Error>>(
        &'a self,
        memory: &mut Memory<'_, S>,
        mut visit: impl FnMut(
            AdmittedNode<'a>,
            &'a str,
            &'a str,
            AdmittedValue<'a>,
            &mut Memory<'_, S>,
        ) -> Result<(), E>,
    ) -> Result<(), E> {
        let view = self.view();
        for row in view.rows() {
            memory.admission_mut().checkpoint(1).map_err(Error::from)?;
            let subject = view.iri(row.s)?;
            memory
                .admission_mut()
                .checkpoint(subject.len().min(view.owner.len()).saturating_add(1) as u64)
                .map_err(Error::from)?;
            visit(
                AdmittedNode { view, id: row.s },
                view.local(subject),
                view.iri(row.p)?,
                AdmittedValue { view, id: row.o },
                memory,
            )?;
        }
        Ok(())
    }

    /// Find an asserted subject through charged physical-row scanning.
    /// # Errors
    /// Returns work exhaustion immediately, rather than reporting absence.
    pub fn get_with_memory<S: Admission + ?Sized>(
        &self,
        subject: &str,
        memory: &mut Memory<'_, S>,
    ) -> Result<Option<AdmittedNode<'_>>, Error> {
        let view = self.view();
        Ok(view
            .find(subject, memory)?
            .map(|id| AdmittedNode { view, id }))
    }

    /// Visit every physical record in ordinary, reifier and annotation order.
    /// Subjects and properties may recur; no uniqueness index is allocated.
    /// The iterator stops permanently after its first refusal.
    pub fn records_with_memory<'a, 'm, 's, S: Admission + ?Sized>(
        &'a self,
        memory: &'s mut Memory<'m, S>,
    ) -> impl Iterator<Item = Result<(&'a str, &'a str, AdmittedValue<'a>), Error>> + 's
    where
        'a: 's,
    {
        let view = self.view();
        let mut rows = view.rows();
        let mut stopped = false;
        std::iter::from_fn(move || {
            if stopped {
                return None;
            }
            let row = rows.next()?;
            let result = (|| {
                memory.admission_mut().checkpoint(1)?;
                let subject = view.iri(row.s)?;
                memory
                    .admission_mut()
                    .checkpoint(subject.len().min(view.owner.len()).saturating_add(1) as u64)?;
                Ok((
                    view.local(subject),
                    view.iri(row.p)?,
                    AdmittedValue { view, id: row.o },
                ))
            })();
            stopped = result.is_err();
            Some(result)
        })
    }
}

impl<'a> AdmittedNode<'a> {
    /// The operation-local native subject key. It identifies no storage outside
    /// this borrowed graph and does not transfer ownership of any term payload.
    #[must_use]
    pub const fn native_id(self) -> TermId {
        self.id
    }

    /// Borrow the local subject spelling, charging its ownership comparison.
    /// # Errors
    /// Returns the first work refusal or violated subject-role invariant.
    pub fn subject_with_memory<S: Admission + ?Sized>(
        self,
        memory: &mut Memory<'_, S>,
    ) -> Result<&'a str, Error> {
        let text = self.view.iri(self.id)?;
        memory
            .admission_mut()
            .checkpoint(text.len().min(self.view.owner.len()).saturating_add(1) as u64)?;
        Ok(self.view.local(text))
    }

    /// Borrow outgoing property/object pairs across all physical record layers.
    /// Properties recur when a subject has multiple values. The iterator stops
    /// permanently after its first refusal and never creates native indexes.
    pub fn properties_with_memory<'m, 's, S: Admission + ?Sized>(
        self,
        memory: &'s mut Memory<'m, S>,
    ) -> impl Iterator<Item = Result<(&'a str, AdmittedValue<'a>), Error>> + 's
    where
        'a: 's,
    {
        self.property_records_with_memory(memory)
            .map(|record| record.map(|(_, property, value)| (property, value)))
    }

    /// Borrow native property keys, spellings and objects in physical order.
    /// Keys are meaningful only within this graph. Consumers can admit their
    /// own distinct native-key ordering without creating hidden query indexes.
    /// The iterator stops permanently after its first refusal.
    pub fn property_records_with_memory<'m, 's, S: Admission + ?Sized>(
        self,
        memory: &'s mut Memory<'m, S>,
    ) -> impl Iterator<Item = Result<(TermId, &'a str, AdmittedValue<'a>), Error>> + 's
    where
        'a: 's,
    {
        let mut rows = self.view.rows();
        let mut stopped = false;
        std::iter::from_fn(move || {
            if stopped {
                return None;
            }
            loop {
                let row = rows.next()?;
                let result = (|| {
                    memory.admission_mut().checkpoint(1)?;
                    if row.s != self.id {
                        return Ok(None);
                    }
                    Ok(Some((
                        row.p,
                        self.view.iri(row.p)?,
                        AdmittedValue {
                            view: self.view,
                            id: row.o,
                        },
                    )))
                })();
                match result {
                    Ok(None) => {}
                    Ok(Some(value)) => return Some(Ok(value)),
                    Err(error) => {
                        stopped = true;
                        return Some(Err(error));
                    }
                }
            }
        })
    }

    /// Borrow values for an absolute property through charged byte comparisons.
    /// The iterator stops permanently after its first refusal.
    pub fn values_with_memory<'m, 's, S: Admission + ?Sized>(
        self,
        predicate: &'s str,
        memory: &'s mut Memory<'m, S>,
    ) -> impl Iterator<Item = Result<AdmittedValue<'a>, Error>> + 's
    where
        'a: 's,
    {
        let mut rows = self.view.rows();
        let mut stopped = false;
        std::iter::from_fn(move || {
            if stopped {
                return None;
            }
            loop {
                let row = rows.next()?;
                let result = (|| {
                    memory.admission_mut().checkpoint(1)?;
                    if row.s != self.id {
                        return Ok(None);
                    }
                    let property = self.view.iri(row.p)?;
                    let equal =
                        compare_bytes(property.as_bytes(), predicate.as_bytes(), &mut |units| {
                            memory.admission_mut().checkpoint(units)
                        })?
                        .is_eq();
                    Ok(equal.then_some(AdmittedValue {
                        view: self.view,
                        id: row.o,
                    }))
                })();
                match result {
                    Ok(None) => {}
                    Ok(Some(value)) => return Some(Ok(value)),
                    Err(error) => {
                        stopped = true;
                        return Some(Err(error));
                    }
                }
            }
        })
    }
}

impl<'a> AdmittedValue<'a> {
    /// Borrow the original native representation without copying its payload.
    #[must_use]
    pub fn term(self) -> TermRef<'a> {
        self.view.dataset.resolve(self.id)
    }

    /// Borrow an IRI after charging exact owned-root normalization.
    /// # Errors
    /// Returns work exhaustion immediately.
    pub fn iri_with_memory<S: Admission + ?Sized>(
        self,
        memory: &mut Memory<'_, S>,
    ) -> Result<Option<&'a str>, Error> {
        memory.admission_mut().checkpoint(1)?;
        let TermRef::Iri(text) = self.term() else {
            return Ok(None);
        };
        memory
            .admission_mut()
            .checkpoint(text.len().min(self.view.owner.len()).saturating_add(1) as u64)?;
        Ok(Some(self.view.local(text)))
    }

    /// Borrow lexical text and datatype of a non-language literal.
    #[must_use]
    pub fn literal(self) -> Option<(&'a str, &'a str)> {
        if let TermRef::Literal {
            lexical,
            datatype,
            language: None,
            direction: None,
        } = self.term()
            && let TermRef::Iri(datatype) = self.view.dataset.resolve(datatype)
        {
            return Some((lexical, datatype));
        }
        None
    }

    /// Borrow the original native components of a quoted triple term.
    #[must_use]
    pub fn quoted(self) -> Option<(Self, Self, Self)> {
        match self.term() {
            TermRef::Triple { s, p, o } => Some((
                Self {
                    view: self.view,
                    id: s,
                },
                Self {
                    view: self.view,
                    id: p,
                },
                Self {
                    view: self.view,
                    id: o,
                },
            )),
            _ => None,
        }
    }

    /// Copy one native value using the caller's original surviving grant.
    /// The returned payload must be destroyed before that grant is released.
    /// # Errors
    /// Returns the first native lookup, work or storage refusal.
    pub fn to_owned_with_memory<S: Admission + ?Sized>(
        self,
        memory: &mut Memory<'_, S>,
    ) -> Result<TermValue, Error> {
        memory.try_scope(|memory| {
            let mut value = self.view.dataset.term_value_with_memory(
                self.id,
                memory,
                Error::Lookup,
                |error, _| Error::Storage(error),
            )?;
            term::normalize_with_memory(&mut value, self.view.owner, memory)?;
            Ok(value)
        })
    }
}
