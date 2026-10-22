// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The timestamp rule of the optional `timestamp` key.

use std::error::Error;
use std::fmt;

/// Why a timestamp is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TimestampError {
    /// The text is not `YYYY-MM-DDThh:mm:ss[.fraction]` followed by `Z` or a
    /// `+hh:mm` / `-hh:mm` offset.
    Malformed,
    /// A field is outside its range: the month, the day of that month, the
    /// hour, the minute, the second or the offset.
    OutOfRange {
        /// The name of the field.
        field: &'static str,
    },
}

impl fmt::Display for TimestampError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed => f.write_str(
                "expected `YYYY-MM-DDThh:mm:ss`, optional fraction digits, then `Z` or `+hh:mm`",
            ),
            Self::OutOfRange { field } => write!(f, "the {field} is out of range"),
        }
    }
}

impl Error for TimestampError {}

/// The authored lexical phase precedes calendar checks; offset syntax follows
/// them. Keep that diagnostic order without interpreting any numeric field.
fn clock_suffix(text: &str) -> Option<&str> {
    let prefix = text.as_bytes().get(..19)?;
    let template = b"0000-00-00T00:00:00";
    if !prefix.iter().zip(template).all(|(found, expected)| {
        if *expected == b'0' {
            found.is_ascii_digit()
        } else {
            found == expected
        }
    }) {
        return None;
    }
    let suffix = text.get(19..)?;
    if let Some(fraction) = suffix.strip_prefix('.') {
        let digits = fraction.bytes().take_while(u8::is_ascii_digit).count();
        (digits != 0).then(|| &fraction[digits..])
    } else {
        Some(suffix)
    }
}

/// RAF checks the whole offset spelling before its numeric range.
fn offset_shape(suffix: &str) -> bool {
    matches!(suffix.as_bytes(), [b'+' | b'-', a, b, b':', c, d]
        if [a, b, c, d].into_iter().all(u8::is_ascii_digit))
}

/// Check that `text` is an RFC 3339 date and time that is also a valid
/// `xsd:dateTime`: four-digit year, a capital `T`, seconds from 00 to 59, an
/// optional fraction of one or more digits, and a capital `Z` or a numeric
/// offset. A timestamp without an offset is refused.
///
/// # Errors
///
/// [`TimestampError::Malformed`] when the text does not have the shape, and
/// [`TimestampError::OutOfRange`] when a field is outside its range.
pub fn validate_timestamp(text: &str) -> Result<(), TimestampError> {
    use purrdf_xsd::rfc3339::{self, LeapSecond, ParseErrorKind, Separator};

    let suffix = clock_suffix(text).ok_or(TimestampError::Malformed)?;
    let leap = text.as_bytes().get(17..19) == Some(b"60");
    let parsed = rfc3339::parse(text, Separator::Rfc3339Abnf, LeapSecond::MonthEnd);
    match parsed {
        Ok(_) if leap => Err(TimestampError::OutOfRange { field: "second" }),
        Ok(_) if suffix == "z" => Err(TimestampError::Malformed),
        Ok(_) => Ok(()),
        Err(error) => Err(match error.kind() {
            ParseErrorKind::OutOfRange { field, .. } if leap && field.starts_with("offset") => {
                TimestampError::OutOfRange { field: "second" }
            }
            ParseErrorKind::OutOfRange { field, .. }
                if field.starts_with("offset") && !offset_shape(suffix) =>
            {
                TimestampError::Malformed
            }
            ParseErrorKind::OutOfRange { field, .. } => TimestampError::OutOfRange {
                field: if field == "day of month" {
                    "day"
                } else {
                    field
                },
            },
            ParseErrorKind::LeapSecond(_) => TimestampError::OutOfRange { field: "second" },
            ParseErrorKind::EndOfInput { .. }
            | ParseErrorKind::Unexpected { .. }
            | ParseErrorKind::Trailing { .. }
                if leap =>
            {
                TimestampError::OutOfRange { field: "second" }
            }
            ParseErrorKind::EndOfInput { .. }
            | ParseErrorKind::Unexpected { .. }
            | ParseErrorKind::Trailing { .. } => TimestampError::Malformed,
        }),
    }
}
