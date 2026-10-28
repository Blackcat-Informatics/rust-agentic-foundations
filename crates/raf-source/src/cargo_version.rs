// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Borrowed Cargo package versions and dependency constraints used by the loader.
//!
//! This adapter owns Cargo's grammar and matching policy. Native storage holds
//! at most 32 comparators inline; parsing never allocates or recurses.

use core::{cmp::Ordering, fmt};
use purrdf_lex::{
    scan::{ByteClass, byte_run_count, needle_table},
    walk::WorkList,
};

const MAX_COMPARATORS: usize = 32;

// Cargo owns these ASCII classes; the native kernel owns every span scan.
const DIGIT_STOPS: [u8; 256] = cargo_stops(b"0123456789");
const IDENTIFIER_STOPS: [u8; 256] =
    cargo_stops(b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-");
const SPACE_STOPS: [u8; 256] = cargo_stops(b" ");
const NON_DIGIT: ByteClass<{ byte_run_count(&DIGIT_STOPS) }> = ByteClass::from_table(DIGIT_STOPS);
const NON_IDENTIFIER: ByteClass<{ byte_run_count(&IDENTIFIER_STOPS) }> =
    ByteClass::from_table(IDENTIFIER_STOPS);
const NON_SPACE: ByteClass<{ byte_run_count(&SPACE_STOPS) }> = ByteClass::from_table(SPACE_STOPS);

const fn cargo_stops(allowed: &[u8]) -> [u8; 256] {
    let mut table = needle_table(allowed);
    let mut index = 0;
    while index < table.len() {
        table[index] ^= 1;
        index += 1;
    }
    table
}

#[derive(Debug)]
pub(crate) struct Error {
    offset: usize,
    reason: &'static str,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.reason, self.offset)
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub(crate) struct Version<'a> {
    text: &'a str,
    numbers: [u64; 3],
    pre: &'a str,
}

impl<'a> Version<'a> {
    pub(crate) fn parse(text: &'a str) -> Result<Self, Error> {
        let mut cursor = Cursor { text, offset: 0 };
        let major = cursor.number()?;
        cursor.require(b'.')?;
        let minor = cursor.number()?;
        cursor.require(b'.')?;
        let patch = cursor.number()?;
        let pre = cursor.qualifiers()?;
        cursor.finish()?;
        Ok(Self {
            text,
            numbers: [major, minor, patch],
            pre,
        })
    }
}

impl fmt::Display for Version<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.text)
    }
}

#[derive(Clone, Copy, Debug)]
enum Operator {
    Exact,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Tilde,
    Caret,
    Wildcard,
}

#[derive(Debug)]
struct Comparator<'a> {
    operator: Operator,
    numbers: [u64; 3],
    precision: usize,
    pre: &'a str,
}

impl Comparator<'_> {
    fn matches(&self, version: &Version<'_>) -> bool {
        let numeric = version.numbers[..self.precision].cmp(&self.numbers[..self.precision]);
        match self.operator {
            Operator::Exact | Operator::Wildcard => {
                numeric == Ordering::Equal && version.pre == self.pre
            }
            Operator::Greater | Operator::GreaterEqual | Operator::Less | Operator::LessEqual => {
                // Missing components describe an entire numeric prefix, so a
                // strict inequality cannot compare its omitted components.
                let ordered = if numeric == Ordering::Equal && self.precision == 3 {
                    prerelease_order(version.pre, self.pre)
                } else {
                    numeric
                };
                let strict = if matches!(self.operator, Operator::Greater | Operator::GreaterEqual)
                {
                    Ordering::Greater
                } else {
                    Ordering::Less
                };
                ordered == strict
                    || (matches!(self.operator, Operator::GreaterEqual | Operator::LessEqual)
                        && numeric == Ordering::Equal
                        && version.pre == self.pre)
            }
            Operator::Tilde => {
                let fixed = self.precision.min(2);
                version.numbers[..fixed] == self.numbers[..fixed]
                    && if self.precision == 3 && version.numbers[2] != self.numbers[2] {
                        version.numbers[2] > self.numbers[2]
                    } else {
                        prerelease_order(version.pre, self.pre) != Ordering::Less
                    }
            }
            Operator::Caret => {
                // Fix the prefix through the first nonzero component. Partial
                // forms fix only the available zero prefix. No upper-bound
                // increment is needed, including at the maximum integer.
                let fixed = self.numbers[..self.precision]
                    .iter()
                    .position(|number| *number != 0)
                    .map_or(self.precision, |index| index + 1);
                version.numbers[..fixed] == self.numbers[..fixed]
                    && numeric != Ordering::Less
                    && (self.precision < 3
                        || numeric != Ordering::Equal
                        || prerelease_order(version.pre, self.pre) != Ordering::Less)
            }
        }
    }

    fn admits_prerelease(&self, version: &Version<'_>) -> bool {
        self.precision == 3 && self.numbers == version.numbers && !self.pre.is_empty()
    }
}

#[derive(Debug)]
pub(crate) struct Requirement<'a> {
    comparators: WorkList<Comparator<'a>, MAX_COMPARATORS>,
}

impl<'a> Requirement<'a> {
    pub(crate) fn parse(text: &'a str) -> Result<Self, Error> {
        let mut cursor = Cursor { text, offset: 0 };
        let mut comparators = WorkList::new();
        cursor.spaces();
        if cursor.wildcard() {
            cursor.spaces();
            cursor.finish()?;
            return Ok(Self { comparators });
        }
        loop {
            if comparators.len() == MAX_COMPARATORS {
                return Err(cursor.error("at most 32 version comparators are allowed"));
            }
            comparators.push(cursor.comparator()?);
            cursor.spaces();
            if cursor.peek().is_none() {
                return Ok(Self { comparators });
            }
            cursor.require(b',')?;
            cursor.spaces();
        }
    }

    pub(crate) fn matches(&self, version: &Version<'_>) -> bool {
        self.comparators
            .iter()
            .all(|comparator| comparator.matches(version))
            && (version.pre.is_empty()
                || self
                    .comparators
                    .iter()
                    .any(|comparator| comparator.admits_prerelease(version)))
    }

    /// Bound qualifier scans, identifier steps and numeric comparisons before matching.
    pub(crate) fn comparison_work(&self, version: &Version<'_>) -> usize {
        self.comparators.iter().fold(0usize, |work, comparator| {
            work.saturating_add(
                version
                    .pre
                    .len()
                    .saturating_add(comparator.pre.len())
                    // Equality, splitting, numeric classification and comparison
                    // visit qualifier bytes; identifier steps also cost work.
                    .saturating_mul(6)
                    .saturating_add(8),
            )
        })
    }
}

/// The empty qualifier denotes a release, which sorts after every prerelease.
fn prerelease_order(left: &str, right: &str) -> Ordering {
    match (left.is_empty(), right.is_empty()) {
        (true, false) => return Ordering::Greater,
        (false, true) => return Ordering::Less,
        _ => {}
    }
    let mut left_parts = left.split('.');
    let mut right_parts = right.split('.');
    loop {
        let (left, right) = match (left_parts.next(), right_parts.next()) {
            (Some(left), Some(right)) => (left, right),
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
        };
        let numeric_left = NON_DIGIT.find_first(left.as_bytes()).is_none();
        let numeric_right = NON_DIGIT.find_first(right.as_bytes()).is_none();
        let order = match (numeric_left, numeric_right) {
            (true, true) => left.len().cmp(&right.len()).then_with(|| left.cmp(right)),
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => left.cmp(right),
        };
        if order != Ordering::Equal {
            return order;
        }
    }
}

struct Cursor<'a> {
    text: &'a str,
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn error(&self, reason: &'static str) -> Error {
        Error {
            offset: self.offset,
            reason,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.offset).copied()
    }

    fn take(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.offset += 1;
            true
        } else {
            false
        }
    }

    fn require(&mut self, byte: u8) -> Result<(), Error> {
        if self.take(byte) {
            Ok(())
        } else {
            Err(self.error("expected version separator"))
        }
    }

    fn finish(&self) -> Result<(), Error> {
        if self.peek().is_none() {
            Ok(())
        } else {
            Err(self.error("unexpected version input"))
        }
    }

    fn spaces(&mut self) {
        self.scan(&NON_SPACE);
    }

    // Every Cargo class permits only ASCII, so its first stop is a UTF-8 boundary.
    fn scan<const RUNS: usize>(&mut self, stops: &ByteClass<RUNS>) {
        let remaining = &self.text.as_bytes()[self.offset..];
        self.offset += stops.find_first(remaining).unwrap_or(remaining.len());
    }

    fn wildcard(&mut self) -> bool {
        if matches!(self.peek(), Some(b'*' | b'x' | b'X')) {
            self.offset += 1;
            true
        } else {
            false
        }
    }

    fn number(&mut self) -> Result<u64, Error> {
        let start = self.offset;
        self.scan(&NON_DIGIT);
        let digits = &self.text[start..self.offset];
        if digits.is_empty() {
            return Err(self.error("expected unsigned version component"));
        }
        if digits.len() > 1 && digits.starts_with('0') {
            return Err(self.error("version component has a leading zero"));
        }
        digits
            .parse()
            .map_err(|_| self.error("version component exceeds the maximum unsigned integer"))
    }

    fn identifier(&mut self, prerelease: bool) -> Result<&'a str, Error> {
        let start = self.offset;
        loop {
            let segment = self.offset;
            self.scan(&NON_IDENTIFIER);
            let identifier = &self.text[segment..self.offset];
            if identifier.is_empty() {
                return Err(self.error("expected nonempty version identifier"));
            }
            if prerelease
                && identifier.len() > 1
                && identifier.starts_with('0')
                && NON_DIGIT.find_first(identifier.as_bytes()).is_none()
            {
                return Err(self.error("numeric prerelease identifier has a leading zero"));
            }
            if !self.take(b'.') {
                return Ok(&self.text[start..self.offset]);
            }
        }
    }

    fn qualifiers(&mut self) -> Result<&'a str, Error> {
        let pre = if self.take(b'-') {
            self.identifier(true)?
        } else {
            ""
        };
        if self.take(b'+') {
            self.identifier(false)?;
        }
        Ok(pre)
    }

    fn comparator(&mut self) -> Result<Comparator<'a>, Error> {
        let explicit = matches!(self.peek(), Some(b'=' | b'>' | b'<' | b'~' | b'^'));
        let mut operator = match self.peek() {
            Some(b'=') => {
                self.offset += 1;
                Operator::Exact
            }
            Some(b'>') => {
                self.offset += 1;
                if self.take(b'=') {
                    Operator::GreaterEqual
                } else {
                    Operator::Greater
                }
            }
            Some(b'<') => {
                self.offset += 1;
                if self.take(b'=') {
                    Operator::LessEqual
                } else {
                    Operator::Less
                }
            }
            Some(b'~') => {
                self.offset += 1;
                Operator::Tilde
            }
            Some(b'^') => {
                self.offset += 1;
                Operator::Caret
            }
            _ => Operator::Caret,
        };
        self.spaces();
        let mut numbers = [self.number()?, 0, 0];
        let mut precision = 1;
        let mut wildcard = false;
        for number in numbers.iter_mut().skip(1) {
            if !self.take(b'.') {
                break;
            }
            if self.wildcard() {
                wildcard = true;
            } else if wildcard {
                return Err(self.error("numeric component cannot follow a wildcard"));
            } else {
                *number = self.number()?;
                precision += 1;
            }
        }
        if wildcard && !explicit {
            operator = Operator::Wildcard;
        }
        let pre = if precision == 3 {
            self.qualifiers()?
        } else {
            ""
        };
        Ok(Comparator {
            operator,
            numbers,
            precision,
            pre,
        })
    }
}
