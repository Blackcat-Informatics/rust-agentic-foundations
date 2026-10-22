// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The measured values a command prints and the limits a caller gives: the
//! text `name=number`, read with no guessing.

use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use raf_okf::{Measure, Number};

/// Why a measure or a limit cannot be read, or the limits do not fit what
/// the command printed. Each is an error: nothing is recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MeasureError {
    /// A limit is not of the form `name=number`, with a name of `A-Z a-z 0-9
    /// _ -`.
    NotAMeasure {
        /// The text as given.
        text: String,
    },
    /// The value of a measure or a limit is not a number.
    NotNumber {
        /// The name.
        name: String,
        /// The value as written.
        value: String,
    },
    /// The command printed the value of a name twice, or the same limit is
    /// given twice.
    Twice {
        /// The name.
        name: String,
        /// `value` for a printed value, `limit` for a limit.
        what: &'static str,
    },
    /// A limit names a value the command never printed.
    LimitNotPrinted {
        /// The name of the limit.
        name: String,
    },
    /// The command printed more values than a record keeps.
    TooMany {
        /// The most values a record keeps, [`MAX_VALUES`].
        limit: usize,
    },
}

/// The most measured values one record keeps: a command that prints more
/// `name=number` lines is an error, so a record stays a short result.
pub const MAX_VALUES: usize = 1024;

impl fmt::Display for MeasureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAMeasure { text } => write!(
                f,
                "`{text}` is not a limit; write it `name=number`, with a name of letters, digits, `_` and `-`"
            ),
            Self::NotNumber { name, value } => write!(
                f,
                "the value `{value}` of `{name}` is not a number: a number is an optional `-`, digits, and an optional `.` followed by digits, with no exponent (`1e-3` is refused; write `0.001`), no `+` and no spaces"
            ),
            Self::Twice { name, what } => match *what {
                "limit" => write!(f, "the limit `{name}` is given twice; give each limit once"),
                _ => write!(
                    f,
                    "the command printed `{name}=` twice, so its value is ambiguous; print each measured value once"
                ),
            },
            Self::TooMany { limit } => write!(
                f,
                "the command printed more than {limit} `name=number` values, the most a record keeps; print only the values the evidence states"
            ),
            Self::LimitNotPrinted { name } => write!(
                f,
                "the limit `{name}` names a value the command never printed; the command must print a line `{name}=<number>`, or the limit is left out"
            ),
        }
    }
}

impl Error for MeasureError {}

/// Whether `name` is a measure name: a non-empty run of `A-Z a-z 0-9 _ -`,
/// the characters of a frontmatter key.
fn is_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// The number written as `value` for `name`, or the error that says why it
/// is not one.
fn number(name: &str, value: &str) -> Result<Number, MeasureError> {
    Number::parse(value).ok_or_else(|| MeasureError::NotNumber {
        name: name.to_string(),
        value: value.to_string(),
    })
}

/// Read one limit, `name=number`.
///
/// ```rust
/// use raf_workplan::evidence::{MeasureError, parse_limit};
///
/// let limit = parse_limit("peak=64")?;
/// assert_eq!((limit.name.as_str(), limit.value.text()), ("peak", "64"));
/// assert!(matches!(parse_limit("peak=1e3"), Err(MeasureError::NotNumber { .. })));
/// assert!(matches!(parse_limit("peak"), Err(MeasureError::NotAMeasure { .. })));
/// # Ok::<(), MeasureError>(())
/// ```
///
/// # Errors
///
/// [`MeasureError::NotAMeasure`] when the text has no `=` or its name is not
/// a measure name, and [`MeasureError::NotNumber`] when the value is not a
/// number (an exponent is not accepted).
pub fn parse_limit(text: &str) -> Result<Measure, MeasureError> {
    let Some((name, value)) = text.split_once('=').filter(|(name, _)| is_name(name)) else {
        return Err(MeasureError::NotAMeasure {
            text: text.to_string(),
        });
    };
    Ok(Measure::new(name, number(name, value)?))
}

/// The limits `limits`, refused when a name is given twice.
pub(crate) fn check_limits(limits: &[Measure]) -> Result<(), MeasureError> {
    let mut seen = HashSet::new();
    for limit in limits {
        if !is_name(&limit.name) {
            return Err(MeasureError::NotAMeasure {
                text: format!("{}={}", limit.name, limit.value.text()),
            });
        }
        if !seen.insert(limit.name.as_str()) {
            return Err(MeasureError::Twice {
                name: limit.name.clone(),
                what: "limit",
            });
        }
    }
    Ok(())
}

/// The measured values of the standard output of a command, in the order it
/// printed them.
///
/// A line is read as a measure when the text before its first `=` is a
/// measure name and stands at the start of the line; the rest of the line
/// (without a final CR) is its value, which must be a number. Every other line
/// is not a measure and is discarded. A name printed twice is an error, as is
/// a value that is not a number and more than [`MAX_VALUES`] values.
pub(crate) fn printed(stdout: &[u8]) -> Result<Vec<Measure>, MeasureError> {
    let mut values = Vec::new();
    let mut seen = HashSet::new();
    for line in stdout.split(|byte| *byte == b'\n') {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        let Some(at) = line.iter().position(|byte| *byte == b'=') else {
            continue;
        };
        let (name, value) = (&line[..at], &line[at + 1..]);
        let Ok(name) = std::str::from_utf8(name) else {
            continue;
        };
        if !is_name(name) {
            continue;
        }
        let value = String::from_utf8_lossy(value);
        let measure = Measure::new(name, number(name, &value)?);
        if !seen.insert(name.to_string()) {
            return Err(MeasureError::Twice {
                name: name.to_string(),
                what: "value",
            });
        }
        if values.len() == MAX_VALUES {
            return Err(MeasureError::TooMany { limit: MAX_VALUES });
        }
        values.push(measure);
    }
    Ok(values)
}

/// The names of the values over their limit, in the order of the limits;
/// a limit whose name was not printed is an error.
pub(crate) fn over_limits(
    values: &[Measure],
    limits: &[Measure],
) -> Result<Vec<String>, MeasureError> {
    let mut over = Vec::new();
    for limit in limits {
        let Some(value) = values.iter().find(|value| value.name == limit.name) else {
            return Err(MeasureError::LimitNotPrinted {
                name: limit.name.clone(),
            });
        };
        if value.value.exceeds(&limit.value) {
            over.push(limit.name.clone());
        }
    }
    Ok(over)
}

#[cfg(test)]
mod tests {
    use raf_okf::{Measure, Number};

    use super::{MeasureError, over_limits, parse_limit, printed};

    fn measure(name: &str, value: &str) -> Measure {
        Measure::new(name, Number::parse(value).expect("a number"))
    }

    #[test]
    fn only_lines_of_a_name_an_equals_sign_and_a_number_are_values() {
        let out = b"running 3 benches\npeak=41\n  indented=3\ntime per op = 3\nratio=0.5\r\nx y=1\n=7\n\nlast=-2";
        assert_eq!(
            printed(out),
            Ok(vec![
                measure("peak", "41"),
                measure("ratio", "0.5"),
                measure("last", "-2")
            ])
        );
        assert_eq!(printed(b""), Ok(Vec::new()));
    }

    #[test]
    fn a_value_that_is_not_a_number_is_an_error_and_its_neighbour_is_not() {
        for (line, value) in [
            ("peak=41MiB", "41MiB"),
            ("peak=1e-3", "1e-3"),
            ("peak=1E3", "1E3"),
            ("peak=+1", "+1"),
            ("peak= 1", " 1"),
            ("peak=", ""),
            ("peak=.5", ".5"),
            ("peak=NaN", "NaN"),
        ] {
            let error = printed(line.as_bytes()).expect_err(line);
            assert_eq!(
                error,
                MeasureError::NotNumber {
                    name: "peak".to_string(),
                    value: value.to_string()
                },
                "{line}"
            );
            assert!(error.to_string().contains("no exponent"), "{error}");
        }
        assert!(printed(b"peak=0.001").is_ok());
    }

    #[test]
    fn a_name_printed_twice_is_an_error_and_two_names_are_not() {
        assert_eq!(
            printed(b"peak=1\nother=2\npeak=1\n"),
            Err(MeasureError::Twice {
                name: "peak".to_string(),
                what: "value"
            })
        );
        assert_eq!(printed(b"peak=1\nPeak=2\n").map(|v| v.len()), Ok(2));
    }

    #[test]
    fn a_limit_never_printed_is_an_error_and_a_printed_one_is_judged() {
        let values = [measure("peak", "41"), measure("time", "3")];
        assert_eq!(
            over_limits(&values, &[measure("rss", "64")]),
            Err(MeasureError::LimitNotPrinted {
                name: "rss".to_string()
            })
        );
        assert_eq!(over_limits(&values, &[measure("peak", "64")]), Ok(vec![]));
        assert_eq!(over_limits(&values, &[measure("peak", "41")]), Ok(vec![]));
        assert_eq!(
            over_limits(&values, &[measure("peak", "40.5"), measure("time", "3")]),
            Ok(vec!["peak".to_string()])
        );
    }

    #[test]
    fn a_value_is_over_its_limit_by_its_text_and_not_by_its_nearest_float() {
        let one_e400 = format!("1{}", "0".repeat(400));
        let two_e400 = format!("2{}", "0".repeat(400));
        for (over, limit, under) in [
            ("9007199254740993", "9007199254740992", "9007199254740991"),
            ("0.30000000000000001", "0.3", "0.29999999999999999"),
            ("1.0000000000000000001", "1", "0.9999999999999999999"),
            (two_e400.as_str(), one_e400.as_str(), &one_e400[..400]),
        ] {
            let limits = [measure("peak", limit)];
            assert_eq!(
                over_limits(&[measure("peak", over)], &limits),
                Ok(vec!["peak".to_string()]),
                "{over}"
            );
            assert_eq!(over_limits(&[measure("peak", limit)], &limits), Ok(vec![]));
            assert_eq!(
                over_limits(&[measure("peak", under)], &limits),
                Ok(vec![]),
                "{under}"
            );
        }
    }

    #[test]
    fn a_limit_is_a_name_and_a_number() {
        assert_eq!(parse_limit("peak=64"), Ok(measure("peak", "64")));
        assert_eq!(parse_limit("a-b_c=0.25"), Ok(measure("a-b_c", "0.25")));
        for text in ["peak", "=3", "pe ak=3", "peak:3"] {
            assert!(
                matches!(parse_limit(text), Err(MeasureError::NotAMeasure { .. })),
                "{text}"
            );
        }
        assert!(matches!(
            parse_limit("peak=6.4e1"),
            Err(MeasureError::NotNumber { .. })
        ));
    }
}
