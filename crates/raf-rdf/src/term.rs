// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Report roles and ordering over `PurRDF`'s native term representation.

use crate::{Error, TermValue, vocab};

/// A report assertion with native RDF terms. Insertion validates its closed roles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Triple {
    /// Snapshot-owned root or fragment IRI.
    pub subject: TermValue,
    /// Absolute vocabulary IRI.
    pub predicate: TermValue,
    /// Atomic value or depth-one triple term.
    pub object: TermValue,
}

pub(crate) fn local(text: &str) -> bool {
    text.is_empty() || text.starts_with('#')
}

pub(crate) fn owned_local<'a>(text: &'a str, owner: &str) -> &'a str {
    match text.strip_prefix(owner) {
        Some("") => "",
        Some(tail) if tail.starts_with('#') => tail,
        _ => text,
    }
}

pub(crate) fn normalize(value: &mut TermValue, base: &str) {
    let mut resident = purrdf_lex::allocation::Resident;
    normalize_with_memory(
        value,
        base,
        &mut purrdf_lex::allocation::Memory::new(&mut resident),
    )
    .expect("resident in-place owner normalization allocates no storage");
}

pub(crate) fn normalize_with_memory<S: purrdf_lex::allocation::Admission + ?Sized>(
    value: &mut TermValue,
    base: &str,
    memory: &mut purrdf_lex::allocation::Memory<'_, S>,
) -> Result<(), purrdf_lex::allocation::StorageError> {
    let mut atom = |value: &mut TermValue| -> Result<(), purrdf_lex::allocation::StorageError> {
        if let TermValue::Iri(iri) = value {
            memory
                .admission_mut()
                .checkpoint(iri.len().min(base.len()).saturating_add(1) as u64)?;
            let remove = iri.len() - owned_local(iri, base).len();
            if remove != 0 {
                memory.admission_mut().checkpoint(iri.len() as u64)?;
                drop(iri.drain(..remove));
            }
        }
        Ok(())
    };
    if let TermValue::Triple { s, o, .. } = value {
        atom(s)?;
        atom(o)?;
    } else {
        atom(value)?;
    }
    Ok(())
}

pub(crate) fn expand(value: &mut TermValue, owner: &str, constructing: bool) -> Result<(), Error> {
    let mut resident = purrdf_lex::allocation::Resident;
    let mut memory = purrdf_lex::allocation::Memory::new(&mut resident);
    // This legacy resident caller already owns its incoming value. Admitted
    // callers enter below with a value produced by their original Memory.
    memory.add_bytes(
        value
            .owned_heap_bytes()
            .ok_or(purrdf_lex::allocation::StorageError::SizeOverflow)?,
    )?;
    expand_with_memory(value, owner, constructing, &mut memory)
}

pub(crate) fn expand_with_memory<S: purrdf_lex::allocation::Admission + ?Sized>(
    value: &mut TermValue,
    owner: &str,
    constructing: bool,
    memory: &mut purrdf_lex::allocation::Memory<'_, S>,
) -> Result<(), Error> {
    let mut atom = |value: &mut TermValue| -> Result<(), Error> {
        if let TermValue::Iri(iri) = value {
            memory
                .admission_mut()
                .checkpoint(iri.len().min(owner.len()).saturating_add(1) as u64)?;
            if local(iri) {
                let mut expanded = memory.string_with_progress(owner)?;
                memory.push_str_with_progress(&mut expanded, iri)?;
                memory.release_string(core::mem::replace(iri, expanded))?;
            } else if constructing && owned_local(iri, owner).len() != iri.len() {
                return Err(Error::Value(memory.string_with_progress(
                    "external IRI aliases the reserved construction owner",
                )?));
            }
        }
        if let TermValue::Literal { datatype, .. } = value
            && constructing
        {
            memory
                .admission_mut()
                .checkpoint(datatype.len().min(owner.len()).saturating_add(1) as u64)?;
            if owned_local(datatype, owner).len() != datatype.len() {
                return Err(Error::Value(memory.string_with_progress(
                    "datatype aliases the reserved construction owner",
                )?));
            }
        }
        Ok(())
    };
    if let TermValue::Triple { s, p, o } = value {
        atom(s)?;
        atom(p)?;
        atom(o)?;
    } else {
        atom(value)?;
    }
    Ok(())
}

pub(crate) fn iri_with_memory<S: purrdf_lex::allocation::Admission + ?Sized>(
    text: &str,
    relative: bool,
    memory: &mut purrdf_lex::allocation::Memory<'_, S>,
) -> Result<(), Error> {
    memory.admission_mut().checkpoint(text.len() as u64)?;
    if relative && local(text) {
        if text.is_empty()
            || text.len() > 1
                && text.as_bytes()[1..]
                    .iter()
                    .all(|b| b.is_ascii_alphanumeric() || b"-._~!$&'()*+,;=:@/?%".contains(b))
        {
            let mut absolute = memory.string_with_progress(vocab::BASE)?;
            memory.push_str_with_progress(&mut absolute, text)?;
            let valid = purrdf_iri::absolute_verdict_with_memory(&absolute, memory)? == Some(true);
            memory.release_string(absolute)?;
            if valid {
                return Ok(());
            }
            return Err(Error::Value(
                memory.format_with_progress(&format_args!("owned IRI {text}"))?,
            ));
        }
    } else if purrdf_iri::absolute_verdict_with_memory(text, memory)? == Some(true) {
        return Ok(());
    }
    Err(Error::Value(memory.format_with_progress(&format_args!(
        "{} IRI {text}",
        if relative {
            "absolute or owned"
        } else {
            "absolute"
        }
    ))?))
}

fn atom_with_memory<S: purrdf_lex::allocation::Admission + ?Sized>(
    term: &TermValue,
    memory: &mut purrdf_lex::allocation::Memory<'_, S>,
) -> Result<(), Error> {
    match term {
        TermValue::Iri(text) => iri_with_memory(text, true, memory),
        TermValue::Literal {
            datatype,
            language,
            direction,
            ..
        } => {
            iri_with_memory(datatype, false, memory)?;
            if language.is_some()
                || direction.is_some()
                || [
                    purrdf_iri::vocab::rdf::LANG_STRING,
                    purrdf_iri::vocab::rdf::DIR_LANG_STRING,
                ]
                .contains(&datatype.as_str())
            {
                return Err(Error::Value(memory.string_with_progress(
                    "language metadata is outside the report profile",
                )?));
            }
            Ok(())
        }
        TermValue::Blank { .. } => Err(Error::Value(
            memory.string_with_progress("blank nodes are outside the report profile")?,
        )),
        TermValue::Triple { .. } => Err(Error::Value(
            memory.string_with_progress("nested triple terms are outside the report profile")?,
        )),
    }
}

impl Triple {
    pub(crate) fn validate(&self) -> Result<(), Error> {
        let subject = self
            .subject
            .as_iri()
            .ok_or_else(|| Error::Value("report subject must be an IRI".into()))?;
        let predicate = self
            .predicate
            .as_iri()
            .ok_or_else(|| Error::Value("predicate must be an IRI".into()))?;
        validate(subject, predicate, &self.object)
    }
}

pub(crate) fn validate(subject: &str, predicate: &str, object: &TermValue) -> Result<(), Error> {
    let mut resident = purrdf_lex::allocation::Resident;
    validate_with_memory(
        subject,
        predicate,
        object,
        &mut purrdf_lex::allocation::Memory::new(&mut resident),
    )
}

pub(crate) fn validate_with_memory<S: purrdf_lex::allocation::Admission + ?Sized>(
    subject: &str,
    predicate: &str,
    object: &TermValue,
    memory: &mut purrdf_lex::allocation::Memory<'_, S>,
) -> Result<(), Error> {
    iri_with_memory(subject, true, memory)?;
    if !local(subject) {
        return Err(Error::Value(
            memory.string_with_progress("report subject must be snapshot-owned")?,
        ));
    }
    iri_with_memory(predicate, false, memory)?;
    if let TermValue::Triple { s, p, o } = object {
        let Some(s) = s.as_iri() else {
            return Err(Error::Value(
                memory.string_with_progress("quoted subject must be an IRI")?,
            ));
        };
        iri_with_memory(s, true, memory)?;
        let Some(p) = p.as_iri() else {
            return Err(Error::Value(
                memory.string_with_progress("quoted predicate must be an IRI")?,
            ));
        };
        iri_with_memory(p, false, memory)?;
        atom_with_memory(o, memory)
    } else {
        atom_with_memory(object, memory)
    }
}
