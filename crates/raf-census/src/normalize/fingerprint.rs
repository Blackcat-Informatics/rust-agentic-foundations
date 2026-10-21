// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Fingerprints: the complete names of normal forms.

use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

use super::form::Form;

/// The id of the byte encoding a form is hashed under: the tokens in order,
/// each as its UTF-8 length (u32, little endian) followed by its bytes; the
/// complete BLAKE3 digest of that concatenation in 64 lowercase hexadecimal digits.
pub const ENCODING: &str = "tokens-len-u32le-utf8/blake3/hex64";

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
    Body([u8; 32]),
    /// A forwarder in its shim form.
    Shim([u8; 32]),
    /// A recognised table.
    Table(TableKind),
    /// A hit of a detector, a declarative pattern or a rule.
    Rule(RuleId),
}

impl Fingerprint {
    /// The `body` fingerprint of a digest.
    #[must_use]
    pub fn body(digest: &[u8; 32]) -> Self {
        Self::Body(*digest)
    }

    /// The `shim` fingerprint of a digest.
    #[must_use]
    pub fn shim(digest: &[u8; 32]) -> Self {
        Self::Shim(*digest)
    }

    /// A human-facing abbreviation, separate from canonical display and parsing.
    ///
    /// Hashed names retain their first sixteen hexadecimal digits and an ellipsis;
    /// table and rule names remain complete. Abbreviations are not identities.
    #[must_use]
    pub fn abbreviated(&self) -> String {
        match self {
            Self::Body(hash) | Self::Shim(hash) => {
                let hex = blake3::Hash::from_bytes(*hash).to_hex();
                format!("{}:{}…", self.kind(), &hex.as_str()[..16])
            }
            _ => self.to_string(),
        }
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
            Self::Body(hash) | Self::Shim(hash) => {
                write!(f, "{}:{}", self.kind(), blake3::Hash::from_bytes(*hash))
            }
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
    /// A `body` or `shim` identifier is not 64 characters.
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
            Self::NoKind(text) => write!(f, "`{}` has no `<kind>:` prefix", text.escape_debug()),
            Self::Kind(kind) => write!(
                f,
                "unknown fingerprint kind `{}`; the kinds are body, shim, table and rule",
                kind.escape_debug()
            ),
            Self::Width(width) => write!(
                f,
                "a body or shim identifier has 64 hexadecimal digits, not {width}"
            ),
            Self::Hex(id) => write!(
                f,
                "`{}` is not 64 lowercase hexadecimal digits",
                id.escape_debug()
            ),
            Self::Table(name) => write!(
                f,
                "unknown table `{}`; the tables are hex-lower and hex-upper",
                name.escape_debug()
            ),
            Self::Rule(id) => write!(f, "`{}` is not a rule id", id.escape_debug()),
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
                if id.len() != 64 {
                    return Err(ParseFingerprintError::Width(id.len()));
                }
                if !id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
                    return Err(ParseFingerprintError::Hex(id.to_owned()));
                }
                let hash = blake3::Hash::from_hex(id)
                    .map_err(|_| ParseFingerprintError::Hex(id.to_owned()))?;
                Ok(if kind == "body" {
                    Self::Body(*hash.as_bytes())
                } else {
                    Self::Shim(*hash.as_bytes())
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
