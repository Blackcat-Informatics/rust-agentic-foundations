// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Fingerprints: the short names of normal forms.

use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

use super::form::Form;

/// The id of the byte encoding a form is hashed under: the tokens in order,
/// each as its UTF-8 length (u32, little endian) followed by its bytes; the
/// BLAKE3 digest of that concatenation; the identifier is the first 8 bytes
/// as one big-endian u64 in 16 lowercase hexadecimal digits.
pub const ENCODING: &str = "tokens-len-u32le-utf8/blake3/first8-be-hex16";

/// The BLAKE3 digest of a form under [`ENCODING`].
pub(crate) fn digest(form: &Form) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for token in form.tokens() {
        let length = u32::try_from(token.text.len())
            .expect("the lexer refuses a token longer than u32::MAX bytes");
        hasher.update(&length.to_le_bytes());
        hasher.update(token.text.as_bytes());
    }
    *hasher.finalize().as_bytes()
}

/// The first 8 bytes of a digest as the big-endian number a fingerprint holds.
pub(crate) fn short(digest: &[u8; 32]) -> u64 {
    let mut head = [0_u8; 8];
    head.copy_from_slice(&digest[..8]);
    u64::from_be_bytes(head)
}

/// A recognised table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TableKind {
    /// The sixteen hexadecimal digits in lowercase.
    HexLower,
    /// The sixteen hexadecimal digits in uppercase.
    HexUpper,
}

impl TableKind {
    /// The name after `table:`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::HexLower => "hex-lower",
            Self::HexUpper => "hex-upper",
        }
    }
}

/// The id of a detector, pattern or rule: the name after `rule:`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleId(String);

impl RuleId {
    /// A rule id: not empty, and ASCII letters, digits, `-`, `_` and `.` only.
    ///
    /// # Errors
    ///
    /// [`ParseFingerprintError::Rule`] for an empty id or any other character.
    pub fn new(id: &str) -> Result<Self, ParseFingerprintError> {
        if id.is_empty()
            || !id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        {
            return Err(ParseFingerprintError::Rule(id.to_owned()));
        }
        Ok(Self(id.to_owned()))
    }

    /// The id as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The name of a form, a table or a rule: `<kind>:<identifier>`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Fingerprint {
    /// A function body in its value form.
    Body(u64),
    /// A forwarder in its shim form.
    Shim(u64),
    /// A recognised table.
    Table(TableKind),
    /// A hit of a detector, a declarative pattern or a rule.
    Rule(RuleId),
}

impl Fingerprint {
    /// The `body` fingerprint of a digest.
    #[must_use]
    pub fn body(digest: &[u8; 32]) -> Self {
        Self::Body(short(digest))
    }

    /// The `shim` fingerprint of a digest.
    #[must_use]
    pub fn shim(digest: &[u8; 32]) -> Self {
        Self::Shim(short(digest))
    }

    /// The kind prefix.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Body(_) => "body",
            Self::Shim(_) => "shim",
            Self::Table(_) => "table",
            Self::Rule(_) => "rule",
        }
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Body(hash) | Self::Shim(hash) => write!(f, "{}:{hash:016x}", self.kind()),
            Self::Table(table) => write!(f, "table:{}", table.name()),
            Self::Rule(rule) => write!(f, "rule:{}", rule.as_str()),
        }
    }
}

/// Why a string is not a fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseFingerprintError {
    /// There is no `:` between the kind and the identifier.
    NoKind(String),
    /// The kind is not `body`, `shim`, `table` or `rule`.
    Kind(String),
    /// A `body` or `shim` identifier is not 16 characters.
    Width(usize),
    /// A `body` or `shim` identifier has a character other than `0-9a-f`.
    Hex(String),
    /// A `table` name is not `hex-lower` or `hex-upper`.
    Table(String),
    /// A `rule` id is empty or has a character outside the id alphabet.
    Rule(String),
}

impl fmt::Display for ParseFingerprintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoKind(text) => write!(f, "`{text}` has no `<kind>:` prefix"),
            Self::Kind(kind) => write!(
                f,
                "unknown fingerprint kind `{kind}`; the kinds are body, shim, table and rule"
            ),
            Self::Width(width) => write!(
                f,
                "a body or shim identifier has 16 hexadecimal digits, not {width}"
            ),
            Self::Hex(id) => write!(f, "`{id}` is not 16 lowercase hexadecimal digits"),
            Self::Table(name) => write!(
                f,
                "unknown table `{name}`; the tables are hex-lower and hex-upper"
            ),
            Self::Rule(id) => write!(f, "`{id}` is not a rule id"),
        }
    }
}

impl StdError for ParseFingerprintError {}

impl FromStr for Fingerprint {
    type Err = ParseFingerprintError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (kind, id) = text
            .split_once(':')
            .ok_or_else(|| ParseFingerprintError::NoKind(text.to_owned()))?;
        match kind {
            "body" | "shim" => {
                if id.len() != 16 {
                    return Err(ParseFingerprintError::Width(id.len()));
                }
                if !id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
                    return Err(ParseFingerprintError::Hex(id.to_owned()));
                }
                let hash = u64::from_str_radix(id, 16)
                    .map_err(|_| ParseFingerprintError::Hex(id.to_owned()))?;
                Ok(if kind == "body" {
                    Self::Body(hash)
                } else {
                    Self::Shim(hash)
                })
            }
            "table" => match id {
                "hex-lower" => Ok(Self::Table(TableKind::HexLower)),
                "hex-upper" => Ok(Self::Table(TableKind::HexUpper)),
                other => Err(ParseFingerprintError::Table(other.to_owned())),
            },
            "rule" => RuleId::new(id).map(Self::Rule),
            other => Err(ParseFingerprintError::Kind(other.to_owned())),
        }
    }
}
