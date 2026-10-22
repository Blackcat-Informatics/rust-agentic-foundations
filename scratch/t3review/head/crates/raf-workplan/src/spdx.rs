// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The two SPDX comment lines a written record opens with, in a repository
//! that checks the licence of every file.

/// The tag of the copyright line of an SPDX header. It is written in two pieces
/// so that this file does not carry the tag itself, which a licence checker
/// reads in any file.
const COPYRIGHT_TAG: &str = concat!("SPDX-", "FileCopyrightText: ");

/// The tag of the licence line of an SPDX header, in two pieces for the same
/// reason.
const LICENCE_TAG: &str = concat!("SPDX-", "License-Identifier: ");

/// Why the SPDX lines cannot be written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpdxError {
    /// One of the two is given and the other, `field`, is not.
    Unset { field: &'static str },
    /// `field` cannot be written as one comment line.
    Invalid {
        field: &'static str,
        reason: &'static str,
    },
}

/// The SPDX lines: both, or neither.
pub(crate) fn pair<'a>(
    copyright: Option<&'a str>,
    licence: Option<&'a str>,
) -> Result<Option<(&'a str, &'a str)>, SpdxError> {
    match (copyright, licence) {
        (None, None) => Ok(None),
        (Some(_), None) => Err(SpdxError::Unset { field: "licence" }),
        (None, Some(_)) => Err(SpdxError::Unset { field: "copyright" }),
        (Some(copyright), Some(licence)) => Ok(Some((
            line("copyright", copyright)?,
            line("licence", licence)?,
        ))),
    }
}

/// A value of the SPDX header: not empty and one line of text.
fn line<'a>(field: &'static str, value: &'a str) -> Result<&'a str, SpdxError> {
    // A control character, and the two separators Unicode adds to the line
    // breaks (U+2028 and U+2029), would end or split the comment line. The
    // value is judged as given, so a line ending at its edge is refused too and
    // not trimmed away.
    if value
        .chars()
        .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
    {
        return Err(SpdxError::Invalid {
            field,
            reason: "it must be one line of text, with no control character and no line or paragraph separator",
        });
    }
    let value = value.trim();
    if value.is_empty() {
        return Err(SpdxError::Invalid {
            field,
            reason: "it is empty",
        });
    }
    Ok(value)
}

/// `rendered` with the two SPDX comment lines directly after the opening
/// fence of the frontmatter, where the documents and records of a repository
/// that checks licences carry them.
pub(crate) fn with_spdx(rendered: String, spdx: Option<(&str, &str)>) -> String {
    let Some((copyright, licence)) = spdx else {
        return rendered;
    };
    let fence = rendered.find('\n').map_or(rendered.len(), |end| end + 1);
    let (head, tail) = rendered.split_at(fence);
    format!("{head}# {COPYRIGHT_TAG}{copyright}\n# {LICENCE_TAG}{licence}\n{tail}")
}
