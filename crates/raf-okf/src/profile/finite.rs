// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Typed vocabularies and findings for the header's closed scalar values.

use super::{
    Actor, AssessmentVerdict, ContractEffect, DocType, Entry, EvidenceVerdict, HeaderShapeError,
    ReviewKind, ReviewVerdict, Rule, State, ValueKind, scalar_text, strings,
};

/// The refusal text for a `review_required` list outside its closed, distinct kinds.
pub(super) const REVIEW_KINDS_SHAPE: &str =
    "a list of distinct security, correctness, design, performance or documentation kinds";

const FLAGS: [(bool, &str); 2] = [(true, "true"), (false, "false")];

#[derive(Clone, Copy)]
pub(super) enum Vocabulary {
    State,
    ContractEffect,
    Flag,
    Actor,
    Verdict,
    ReviewKind,
    ReviewKinds,
}

impl Vocabulary {
    pub(super) const fn for_kind(kind: ValueKind) -> Option<Self> {
        match kind {
            ValueKind::State => Some(Self::State),
            ValueKind::ContractEffect => Some(Self::ContractEffect),
            ValueKind::Flag => Some(Self::Flag),
            ValueKind::Actor => Some(Self::Actor),
            ValueKind::Verdict => Some(Self::Verdict),
            ValueKind::ReviewKind => Some(Self::ReviewKind),
            ValueKind::ReviewKinds => Some(Self::ReviewKinds),
            _ => None,
        }
    }

    /// The authored texts the shape judges: one scalar, or every member of an
    /// all-scalar list. Any other value has no texts.
    pub(super) fn texts(self, entry: &Entry) -> Option<Vec<&str>> {
        if self.is_list() {
            strings(entry.value())
        } else {
            scalar_text(entry).map(|text| vec![text])
        }
    }

    /// Whether the key holds a list whose every member is drawn from the vocabulary.
    pub(super) const fn is_list(self) -> bool {
        matches!(self, Self::ReviewKinds)
    }

    pub(super) fn names(self, doc_type: DocType) -> Result<Vec<&'static str>, HeaderShapeError> {
        let names = match self {
            Self::State => State::ALL.map(State::name).into(),
            Self::ContractEffect => ContractEffect::ALL.map(ContractEffect::name).into(),
            Self::Flag => FLAGS.map(|(_, name)| name).into(),
            Self::Actor => Actor::ALL.map(Actor::name).into(),
            Self::Verdict => match doc_type {
                DocType::Evidence => EvidenceVerdict::ALL.map(EvidenceVerdict::name).into(),
                DocType::Assessment => AssessmentVerdict::ALL.map(AssessmentVerdict::name).into(),
                DocType::Review => ReviewVerdict::ALL.map(ReviewVerdict::name).into(),
                _ => return Err(HeaderShapeError::UnexpectedReport),
            },
            Self::ReviewKind | Self::ReviewKinds => ReviewKind::ALL.map(ReviewKind::name).into(),
        };
        Ok(names)
    }

    pub(super) fn rejection(self, entry: &Entry, doc_type: DocType) -> Rule {
        match (self, scalar_text(entry)) {
            (Self::State, Some(text)) => Rule::BadState { found: text.into() },
            (Self::ContractEffect, text) => Rule::BadContractEffect {
                found: text.unwrap_or("<not a scalar>").into(),
            },
            (Self::Flag, _) => Rule::WrongShape {
                expected: "true or false",
            },
            (Self::Actor, Some(text)) => Rule::BadActor { found: text.into() },
            (Self::Verdict, Some(text)) => Rule::BadVerdict {
                doc_type,
                found: text.into(),
            },
            (Self::ReviewKind, _) => Rule::WrongShape {
                expected: "security, correctness, design, performance or documentation",
            },
            (Self::ReviewKinds, _) => Rule::WrongShape {
                expected: REVIEW_KINDS_SHAPE,
            },
            (Self::State | Self::Actor | Self::Verdict, None) => Rule::WrongShape {
                expected: "a scalar",
            },
        }
    }
}

pub(super) fn flag(text: &str) -> Option<bool> {
    FLAGS
        .into_iter()
        .find_map(|(value, name)| (name == text).then_some(value))
}

pub(super) fn decode<T>(
    entry: &Entry,
    read: impl FnOnce(&str) -> Option<T>,
) -> Result<T, HeaderShapeError> {
    // At this point the native report admitted the authored value. A failed
    // conversion means the projection and typed vocabulary disagree.
    scalar_text(entry)
        .and_then(read)
        .ok_or(HeaderShapeError::UnexpectedReport)
}
