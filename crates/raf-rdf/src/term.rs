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

pub(crate) fn normalize(value: &mut TermValue, base: &str) {
    let atom = |value: &mut TermValue| {
        if let TermValue::Iri(iri) = value {
            if iri == base {
                iri.clear();
            } else if let Some(fragment) =
                iri.strip_prefix(base).filter(|tail| tail.starts_with('#'))
            {
                *iri = fragment.to_owned();
            }
        }
    };
    if let TermValue::Triple { s, o, .. } = value {
        atom(s);
        atom(o);
    } else {
        atom(value);
    }
}

pub(crate) fn expand(value: &mut TermValue, owner: &str, constructing: bool) -> Result<(), Error> {
    let atom = |value: &mut TermValue| -> Result<(), Error> {
        if let TermValue::Iri(iri) = value {
            if local(iri) {
                *iri = format!("{owner}{iri}");
            } else if constructing
                && (iri == owner
                    || iri
                        .strip_prefix(owner)
                        .is_some_and(|tail| tail.starts_with('#')))
            {
                return Err(Error::Value(
                    "external IRI aliases the reserved construction owner".into(),
                ));
            }
        }
        if let TermValue::Literal { datatype, .. } = value
            && constructing
            && (datatype == owner
                || datatype
                    .strip_prefix(owner)
                    .is_some_and(|tail| tail.starts_with('#')))
        {
            return Err(Error::Value(
                "datatype aliases the reserved construction owner".into(),
            ));
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

pub(crate) fn iri(text: &str, relative: bool) -> Result<(), Error> {
    if relative && local(text) {
        if text.is_empty()
            || (text.len() > 1
                && text.as_bytes()[1..]
                    .iter()
                    .all(|b| b.is_ascii_alphanumeric() || b"-._~!$&'()*+,;=:@/?%".contains(b)))
        {
            // The native parser admits absolute IRIs. Validate the owned
            // reference in its actual absolute snapshot namespace.
            purrdf_iri::parse(&format!("{}{text}", vocab::BASE))
                .map_err(|_| Error::Value(format!("owned IRI {text}")))?;
            return Ok(());
        }
    } else if purrdf_iri::parse(text).is_ok_and(|iri| iri.scheme().is_some()) {
        return Ok(());
    }
    Err(Error::Value(format!(
        "{} IRI {text}",
        if relative {
            "absolute or owned"
        } else {
            "absolute"
        }
    )))
}

fn atom(term: &TermValue) -> Result<(), Error> {
    match term {
        TermValue::Iri(text) => iri(text, true),
        TermValue::Literal {
            datatype,
            language,
            direction,
            ..
        } => {
            iri(datatype, false)?;
            if language.is_some()
                || direction.is_some()
                || [
                    purrdf_iri::vocab::rdf::LANG_STRING,
                    purrdf_iri::vocab::rdf::DIR_LANG_STRING,
                ]
                .contains(&datatype.as_str())
            {
                return Err(Error::Value(
                    "language metadata is outside the report profile".into(),
                ));
            }
            Ok(())
        }
        TermValue::Blank { .. } => Err(Error::Value(
            "blank nodes are outside the report profile".into(),
        )),
        TermValue::Triple { .. } => Err(Error::Value(
            "nested triple terms are outside the report profile".into(),
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
    iri(subject, true)?;
    if !local(subject) {
        return Err(Error::Value("report subject must be snapshot-owned".into()));
    }
    iri(predicate, false)?;
    if let TermValue::Triple { s, p, o } = object {
        iri(
            s.as_iri()
                .ok_or_else(|| Error::Value("quoted subject must be an IRI".into()))?,
            true,
        )?;
        iri(
            p.as_iri()
                .ok_or_else(|| Error::Value("quoted predicate must be an IRI".into()))?,
            false,
        )?;
        atom(o)
    } else {
        atom(object)
    }
}
