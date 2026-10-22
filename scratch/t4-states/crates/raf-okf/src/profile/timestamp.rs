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

/// Read `count` ASCII digits from the front of `text`.
fn digits(text: &str, count: usize) -> Result<(u32, &str), TimestampError> {
    let (head, rest) = text
        .split_at_checked(count)
        .ok_or(TimestampError::Malformed)?;
    if !head.bytes().all(|b| b.is_ascii_digit()) {
        return Err(TimestampError::Malformed);
    }
    let value = head.parse().map_err(|_| TimestampError::Malformed)?;
    Ok((value, rest))
}

/// Read `separator` from the front of `text`.
fn separator(text: &str, separator: char) -> Result<&str, TimestampError> {
    text.strip_prefix(separator)
        .ok_or(TimestampError::Malformed)
}

fn in_range(value: u32, max: u32, field: &'static str) -> Result<(), TimestampError> {
    if value <= max {
        Ok(())
    } else {
        Err(TimestampError::OutOfRange { field })
    }
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
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
    let (year, rest) = digits(text, 4)?;
    let rest = separator(rest, '-')?;
    let (month, rest) = digits(rest, 2)?;
    let rest = separator(rest, '-')?;
    let (day, rest) = digits(rest, 2)?;
    let rest = separator(rest, 'T')?;
    let (hour, rest) = digits(rest, 2)?;
    let rest = separator(rest, ':')?;
    let (minute, rest) = digits(rest, 2)?;
    let rest = separator(rest, ':')?;
    let (second, mut rest) = digits(rest, 2)?;
    if let Some(fraction) = rest.strip_prefix('.') {
        let count = fraction.bytes().take_while(u8::is_ascii_digit).count();
        if count == 0 {
            return Err(TimestampError::Malformed);
        }
        rest = &fraction[count..];
    }
    if month == 0 || month > 12 {
        return Err(TimestampError::OutOfRange { field: "month" });
    }
    if day == 0 || day > days_in_month(year, month) {
        return Err(TimestampError::OutOfRange { field: "day" });
    }
    in_range(hour, 23, "hour")?;
    in_range(minute, 59, "minute")?;
    in_range(second, 59, "second")?;
    if rest == "Z" {
        return Ok(());
    }
    let sign = rest.chars().next();
    if !matches!(sign, Some('+' | '-')) {
        return Err(TimestampError::Malformed);
    }
    let (offset_hour, rest) = digits(&rest[1..], 2)?;
    let rest = separator(rest, ':')?;
    let (offset_minute, rest) = digits(rest, 2)?;
    if !rest.is_empty() {
        return Err(TimestampError::Malformed);
    }
    in_range(offset_hour, 23, "offset hour")?;
    in_range(offset_minute, 59, "offset minute")
}
