// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Original-account native construction and immutable physical RDF storage.

use crate::{Error, Format, TermValue, graph::CONSTRUCTION_OWNER, term, view::GraphView, write};
use purrdf_core::{
    NativeBuildError, OwnedTermFoldError, RdfDataset, RdfDatasetBuilder, RecordKind, TermId,
};
use purrdf_lex::allocation::{Admission, Memory, OrderedTable, OwnedAdmission, StorageError};
use std::{cell::Cell, fmt};

/// The original native build or closed-profile refusal.
#[derive(Debug)]
pub enum BuildCause {
    /// Native validation, allocation or layout failure, without formatting loss.
    Native(NativeBuildError),
    /// RAF's closed ground-profile refusal.
    Profile(Error),
    /// The unchanged read refusal of a retained native parser view.
    Read(purrdf_rdf::BorrowedReadError),
    /// Native source-position refusal during canonical-byte diagnosis.
    Position(purrdf_iri::PositionError),
}
impl BuildCause {
    fn admission_refused(&self) -> bool {
        matches!(
            self,
            Self::Native(NativeBuildError::Storage(StorageError::AdmissionFailed))
                | Self::Profile(Error::Storage(StorageError::AdmissionFailed))
        )
    }
}

/// Non-owning notification that construction retains its first failure.
/// Ignoring this token never permits a poisoned construction to publish.
#[derive(Clone, Copy, Debug)]
pub struct ConstructionStopped(());

/// A construction body either propagated its retained native stop or supplied
/// a separate caller-owned failure. The first native poison always wins.
pub enum ConstructionExit<E> {
    /// Propagate the construction's non-owning stop token.
    Stopped(ConstructionStopped),
    /// Preserve the caller's original owned failure without a substituted cause.
    Caller(E),
}
impl<E> From<ConstructionStopped> for ConstructionExit<E> {
    fn from(stopped: ConstructionStopped) -> Self {
        Self::Stopped(stopped)
    }
}

/// A caller failure followed by the original construction account.
/// Its original diagnostic ownership must be carried by `E` itself.
pub struct CallerFailure<A: Admission, E> {
    cause: E,
    _account: OwnedAdmission<A>,
}
impl<A: Admission, E> CallerFailure<A, E> {
    /// Borrow the unchanged caller failure while both original owners live.
    #[must_use]
    pub const fn cause(&self) -> &E {
        &self.cause
    }
}

/// An owned construction refusal with no partial graph exposed.
pub enum ConstructionFailure<A: Admission, F: Copy, E> {
    /// The original native or closed-profile failure.
    Native(BuildFailure<A, F>),
    /// The original caller failure and the same construction account.
    Caller(CallerFailure<A, E>),
}
impl From<Error> for BuildCause {
    fn from(error: Error) -> Self {
        Self::Profile(error)
    }
}
impl From<StorageError> for BuildCause {
    fn from(error: StorageError) -> Self {
        Self::Profile(Error::Storage(error))
    }
}
impl From<NativeBuildError> for BuildCause {
    fn from(error: NativeBuildError) -> Self {
        Self::Native(error)
    }
}
impl fmt::Display for BuildCause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native(error) => error.fmt(formatter),
            Self::Profile(error) => error.fmt(formatter),
            Self::Read(error) => error.fmt(formatter),
            Self::Position(error) => error.fmt(formatter),
        }
    }
}
impl std::error::Error for BuildCause {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Native(error) => Some(error),
            Self::Profile(error) => Some(error),
            Self::Read(error) => Some(error),
            Self::Position(error) => Some(error),
        }
    }
}

type Key = (u8, TermId, TermId, TermId);
struct Staged {
    native: Option<RdfDatasetBuilder>,
    frozen: Option<RdfDataset>,
    owner: String,
    entries: Option<OrderedTable<Key, ()>>,
}

/// An opaque failure retaining its owned diagnostic and original account.
/// Partial graph storage is destroyed before this value is returned; its
/// conservative grant remains held until the diagnostic is destroyed.
pub struct BuildFailure<A: Admission, F: Copy = ()> {
    cause: BuildCause,
    original: Option<F>,
    _account: OwnedAdmission<A>,
}
impl<A: Admission, F: Copy> BuildFailure<A, F> {
    /// Borrow the unchanged native or profile failure while its account lives.
    #[must_use]
    pub const fn cause(&self) -> &BuildCause {
        &self.cause
    }
    /// The caller's original refusal, captured before its callback resumed.
    #[must_use]
    pub const fn captured(&self) -> Option<F> {
        self.original
    }
}
impl<A: Admission, F: Copy> fmt::Debug for BuildFailure<A, F> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cause.fmt(formatter)
    }
}
impl<A: Admission, F: Copy> fmt::Display for BuildFailure<A, F> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.cause, formatter)
    }
}
impl<A: Admission, F: Copy> std::error::Error for BuildFailure<A, F> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

/// Owned immutable native graph storage followed by its one original account.
/// It is deliberately not cloneable and exposes no payload-detaching operation.
pub struct AdmittedGraph<A: Admission> {
    dataset: RdfDataset,
    owner: String,
    _live_bytes: usize,
    _account: OwnedAdmission<A>,
}

/// A native physical-record construction scope. Only `AdmittedGraph::build`
/// supplies this value; its original account remains owned by that operation.
pub struct Construction<'s, 'm, A: Admission, F: Copy = ()> {
    staged: &'s mut Staged,
    memory: &'s mut Memory<'m, OwnedAdmission<A>>,
    first: &'s mut Option<BuildCause>,
    original: &'s mut Option<F>,
    captured_at_step: &'s mut bool,
    capture: &'s Cell<Option<F>>,
}

impl<A: Admission> AdmittedGraph<A> {
    /// Construct fresh physical RDF records under one fresh original account.
    ///
    /// This does not edit a base or infer mutable provenance. Existing resident
    /// `Builder::from_graph` edits retain their native mutable implementation.
    ///
    /// # Errors
    /// Retains the first native or profile cause and its same account on failure.
    ///
    /// # Panics
    /// Panics if an internal construction lifecycle invariant is violated.
    pub fn build(
        account: A,
        body: impl FnOnce(&mut Construction<'_, '_, A>) -> Result<(), ConstructionStopped>,
    ) -> Result<Self, BuildFailure<A>> {
        Self::build_captured(account, &Cell::new(None), body)
    }

    /// Construct while snapshotting the caller's typed native-refusal capture.
    /// This reads the capture at refusal; it never resets captures or budgets.
    /// # Errors
    /// Retains the first owned cause and its same original account on failure.
    ///
    /// # Panics
    /// Panics if an internal construction lifecycle invariant is violated.
    pub fn build_captured<F: Copy>(
        account: A,
        capture: &Cell<Option<F>>,
        body: impl FnOnce(&mut Construction<'_, '_, A, F>) -> Result<(), ConstructionStopped>,
    ) -> Result<Self, BuildFailure<A, F>> {
        Self::build_owned_captured(account, CONSTRUCTION_OWNER, capture, body)
    }

    pub(crate) fn build_owned_captured<F: Copy>(
        account: A,
        owner: &str,
        capture: &Cell<Option<F>>,
        body: impl FnOnce(&mut Construction<'_, '_, A, F>) -> Result<(), ConstructionStopped>,
    ) -> Result<Self, BuildFailure<A, F>> {
        Self::build_owned_observed(
            account,
            owner,
            capture,
            |construction| {
                body(construction).map_err(ConstructionExit::<std::convert::Infallible>::Stopped)
            },
            |outcome| match outcome {
                Ok(graph) => Ok(graph),
                Err(ConstructionFailure::Native(failure)) => Err(failure),
                Err(ConstructionFailure::Caller(failure)) => match failure.cause {},
            },
        )
    }

    /// Construct while preserving a caller's separate original failure owner.
    /// The observer may retain an owned failure; its payload cannot be detached
    /// from the original account. A caller diagnostic must retain its own grant
    /// inside `E`; this operation never adopts resident caller allocations.
    ///
    /// # Panics
    /// Panics if an internal construction lifecycle invariant is violated.
    pub fn build_observed<F: Copy, E, R>(
        account: A,
        capture: &Cell<Option<F>>,
        body: impl FnOnce(&mut Construction<'_, '_, A, F>) -> Result<(), ConstructionExit<E>>,
        observe: impl FnOnce(Result<Self, ConstructionFailure<A, F, E>>) -> R,
    ) -> R {
        Self::build_owned_observed(account, CONSTRUCTION_OWNER, capture, body, observe)
    }

    fn build_owned_observed<F: Copy, E, R>(
        account: A,
        owner: &str,
        capture: &Cell<Option<F>>,
        body: impl FnOnce(&mut Construction<'_, '_, A, F>) -> Result<(), ConstructionExit<E>>,
        observe: impl FnOnce(Result<Self, ConstructionFailure<A, F, E>>) -> R,
    ) -> R {
        let mut account = OwnedAdmission::new(account);
        let mut staged = Staged {
            native: Some(RdfDatasetBuilder::new()),
            frozen: None,
            owner: String::new(),
            entries: Some(OrderedTable::default()),
        };
        let (mut first, mut original, mut captured_at_step) = (None, None, false);
        let mut caller = None;
        let (result, live) = {
            let mut memory = Memory::new(&mut account);
            let result = (|| {
                staged.owner = memory.string_with_progress(owner)?;
                let outcome = body(&mut Construction {
                    staged: &mut staged,
                    memory: &mut memory,
                    first: &mut first,
                    original: &mut original,
                    captured_at_step: &mut captured_at_step,
                    capture,
                });
                if let Some(cause) = first.take() {
                    return Err(cause);
                }
                match outcome {
                    Err(ConstructionExit::Caller(cause)) => {
                        caller = Some(cause);
                        return Ok(());
                    }
                    Err(ConstructionExit::Stopped(_)) => {
                        return Err(Error::Invariant(
                            "construction stopped without this operation's failure",
                        )
                        .into());
                    }
                    Ok(()) => {}
                }
                let native = staged
                    .native
                    .take()
                    .expect("a live construction retains its native builder");
                staged.frozen = Some(native.freeze_with_memory(&mut memory)?);
                staged
                    .entries
                    .take()
                    .expect("a live construction retains its exact membership table")
                    .release(&mut memory)?;
                let view = GraphView {
                    dataset: staged
                        .frozen
                        .as_ref()
                        .expect("native freeze supplied the dataset"),
                    owner: &staged.owner,
                };
                view.validate(&mut memory)?;
                Ok::<_, BuildCause>(())
            })();
            (result, memory.admitted_bytes())
        };
        let outcome = match result {
            Err(cause) => {
                if !captured_at_step && cause.admission_refused() {
                    original = capture.get();
                }
                drop(staged);
                Err(ConstructionFailure::Native(BuildFailure {
                    cause,
                    original,
                    _account: account,
                }))
            }
            Ok(()) if caller.is_some() => {
                drop(staged);
                Err(ConstructionFailure::Caller(CallerFailure {
                    cause: caller.take().expect("caller refusal was present"),
                    _account: account,
                }))
            }
            Ok(()) => Ok(Self {
                dataset: staged
                    .frozen
                    .take()
                    .expect("successful construction froze native storage"),
                owner: core::mem::take(&mut staged.owner),
                _live_bytes: live,
                _account: account,
            }),
        };
        observe(outcome)
    }

    pub(crate) fn view(&self) -> GraphView<'_> {
        GraphView {
            dataset: &self.dataset,
            owner: &self.owner,
        }
    }

    /// Serialize through the same admitted renderer as resident graphs.
    /// The caller retains output's surviving grant with the returned string.
    /// # Errors
    /// Returns the first typed lookup, invariant or storage refusal.
    pub fn to_text_with_memory<S: Admission + ?Sized, O: Admission + ?Sized>(
        &self,
        format: Format,
        scratch: &mut Memory<'_, S>,
        output: &mut Memory<'_, O>,
    ) -> Result<String, Error> {
        write::text_with_view(self.view(), format, scratch, output)
    }
}

impl<A: Admission, F: Copy> Construction<'_, '_, A, F> {
    /// Insert a borrowed IRI object without constructing a resident term.
    /// # Errors
    /// Retains invalid roles, physical duplicates or original-account exhaustion.
    pub fn insert_iri(
        &mut self,
        kind: RecordKind,
        subject: &str,
        predicate: &str,
        object: &str,
    ) -> Result<(), ConstructionStopped> {
        self.insert_borrowed(kind, subject, predicate, |memory| {
            Ok(TermValue::Iri(memory.string_with_progress(object)?))
        })
    }

    /// Insert a borrowed non-language literal under the original account.
    /// # Errors
    /// Retains invalid roles, physical duplicates or original-account exhaustion.
    pub fn insert_literal(
        &mut self,
        kind: RecordKind,
        subject: &str,
        predicate: &str,
        lexical: &str,
        datatype: &str,
    ) -> Result<(), ConstructionStopped> {
        self.insert_borrowed(kind, subject, predicate, |memory| {
            Ok(TermValue::Literal {
                lexical_form: memory.string_with_progress(lexical)?,
                datatype: memory.string_with_progress(datatype)?,
                language: None,
                direction: None,
            })
        })
    }

    /// Format a literal through original admission before creating its term.
    /// Canonical lexical policy remains the caller's selected native formatter.
    /// # Errors
    /// Retains invalid roles, physical duplicates or original-account exhaustion.
    pub fn insert_display_literal(
        &mut self,
        kind: RecordKind,
        subject: &str,
        predicate: &str,
        lexical: &impl fmt::Display,
        datatype: &str,
    ) -> Result<(), ConstructionStopped> {
        self.insert_borrowed(kind, subject, predicate, |memory| {
            Ok(TermValue::Literal {
                lexical_form: memory.format_with_progress(lexical)?,
                datatype: memory.string_with_progress(datatype)?,
                language: None,
                direction: None,
            })
        })
    }

    fn insert_borrowed(
        &mut self,
        kind: RecordKind,
        subject: &str,
        predicate: &str,
        object: impl FnOnce(&mut Memory<'_, OwnedAdmission<A>>) -> Result<TermValue, BuildCause>,
    ) -> Result<(), ConstructionStopped> {
        if self.first.is_some() {
            return Err(ConstructionStopped(()));
        }
        let result = (|| {
            let object = object(self.memory)?;
            term::validate_with_memory(subject, predicate, &object, self.memory)?;
            let subject = self.intern_iri(subject)?;
            let predicate = self.intern_iri(predicate)?;
            let object = self.intern_owned(object)?;
            self.push_ids(kind, subject, predicate, object)
        })();
        self.retain_failure(result)
    }

    pub(crate) fn replay(
        &mut self,
        view: &impl purrdf_core::DatasetView<Id = TermId, ReadError = purrdf_rdf::BorrowedReadError>,
    ) -> Result<(), ConstructionStopped> {
        if self.first.is_some() {
            return Err(ConstructionStopped(()));
        }
        let result = crate::import::replay(
            view,
            self.staged
                .native
                .as_mut()
                .expect("construction has not frozen"),
            self.memory,
        );
        self.retain_failure(result)
    }

    fn retain_failure(
        &mut self,
        result: Result<(), BuildCause>,
    ) -> Result<(), ConstructionStopped> {
        if let Err(cause) = result {
            if cause.admission_refused() {
                *self.original = self.capture.get();
            }
            *self.captured_at_step = true;
            *self.first = Some(cause);
            return Err(ConstructionStopped(()));
        }
        Ok(())
    }

    fn intern(&mut self, value: &TermValue) -> Result<TermId, BuildCause> {
        let owned = value
            .try_clone_with_memory(self.memory, |_| Ok::<_, StorageError>(()))
            .map_err(|error| match error {
                OwnedTermFoldError::Visitor(cause) | OwnedTermFoldError::Storage(cause) => {
                    Error::Storage(cause)
                }
            })?;
        self.intern_owned(owned)
    }

    fn intern_iri(&mut self, text: &str) -> Result<TermId, BuildCause> {
        let text = self.memory.string_with_progress(text)?;
        self.intern_owned(TermValue::Iri(text))
    }

    fn intern_owned(&mut self, mut owned: TermValue) -> Result<TermId, BuildCause> {
        term::expand_with_memory(&mut owned, &self.staged.owner, true, self.memory)?;
        let id = self
            .staged
            .native
            .as_mut()
            .expect("construction has not frozen")
            .intern_value_with_memory(&owned, self.memory)?;
        if let Err((remaining, cause)) = owned.release_with_memory(self.memory) {
            drop(remaining);
            return Err(cause.into());
        }
        Ok(id)
    }

    /// Insert one explicitly typed physical record, preserving authored layers.
    ///
    /// # Errors
    /// Refuses invalid profile roles, duplicated physical records and exhaustion.
    pub fn insert_record(
        &mut self,
        kind: RecordKind,
        subject: &str,
        predicate: &str,
        object: &TermValue,
    ) -> Result<(), ConstructionStopped> {
        if self.first.is_some() {
            return Err(ConstructionStopped(()));
        }
        let result = self.insert_record_inner(kind, subject, predicate, object);
        self.retain_failure(result)
    }

    fn insert_record_inner(
        &mut self,
        kind: RecordKind,
        subject: &str,
        predicate: &str,
        object: &TermValue,
    ) -> Result<(), BuildCause> {
        term::validate_with_memory(subject, predicate, object, self.memory)?;
        let subject = self.intern_iri(subject)?;
        let predicate = self.intern_iri(predicate)?;
        let object = self.intern(object)?;
        self.push_ids(kind, subject, predicate, object)
    }

    fn push_ids(
        &mut self,
        kind: RecordKind,
        subject: TermId,
        predicate: TermId,
        object: TermId,
    ) -> Result<(), BuildCause> {
        let tag = match kind {
            RecordKind::Ordinary => 0,
            RecordKind::Reifier => 1,
            RecordKind::Annotation => 2,
            _ => return Err(Error::Invariant("unsupported native record layer").into()),
        };
        let key = (tag, subject, predicate, object);
        let entries = self
            .staged
            .entries
            .as_mut()
            .expect("construction retains its membership table");
        let entry = entries.prepare_insert_with_memory(key, self.memory, |a, b, memory| {
            memory.admission_mut().checkpoint(4)?;
            Ok::<_, BuildCause>(a.cmp(b))
        })?;
        if entry.is_occupied() {
            return Err(Error::Repeated.into());
        }
        entry.publish(());
        let native = self
            .staged
            .native
            .as_mut()
            .expect("construction has not frozen");
        match kind {
            RecordKind::Ordinary => {
                native.push_quad_with_memory(subject, predicate, object, None, self.memory)?;
            }
            RecordKind::Reifier => {
                self.memory
                    .admission_mut()
                    .checkpoint(crate::vocab::REIFIES.len() as u64)?;
                if !matches!(native.resolve(predicate), purrdf_core::TermRef::Iri(iri) if iri == crate::vocab::REIFIES)
                {
                    return Err(Error::Value(
                        self.memory
                            .string_with_progress("a reifier record must use rdf:reifies")?,
                    )
                    .into());
                }
                native.push_reifier_in_graph_with_memory(subject, object, None, self.memory)?;
            }
            RecordKind::Annotation => native.push_annotation_in_graph_with_memory(
                subject,
                predicate,
                object,
                None,
                self.memory,
            )?,
            _ => return Err(Error::Invariant("unsupported native record layer").into()),
        }
        Ok(())
    }
}
