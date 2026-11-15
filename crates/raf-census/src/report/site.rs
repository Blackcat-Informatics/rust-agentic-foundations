// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The identity of one ratcheted site: a file, a rule and a key that survives
//! line movement.

use purrdf_hash::{blake3, hex::Digest32};
use raf_source::RelPath;

use super::ReportError as E;
use crate::{Error, RuleId};

/// The domain prefix of every site-key preimage.
const DOMAIN: &[u8] = b"raf-census/debt-site/v1";

/// A stable per-site key: the BLAKE3-256 digest of the enclosing item's symbol
/// and the first string literal at or after the site inside that item.
///
/// The preimage is the domain bytes, the symbol as a little-endian `u32` length
/// and its bytes, one presence byte, and when the literal is present its length
/// and bytes in the same encoding. Line numbers and byte offsets are never
/// hashed, so inserting lines above a site leaves its key unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SiteKey(Digest32);

fn length_prefixed(hasher: &mut blake3::Hasher, text: &str) -> Result<(), Error> {
    let length = u32::try_from(text.len())
        .map_err(|_| E::Value("site key: a symbol or literal exceeds u32::MAX bytes".into()))?;
    hasher.update(&length.to_le_bytes());
    hasher.update(text.as_bytes());
    Ok(())
}

impl SiteKey {
    /// The key of a site in the item `symbol` whose first string literal is `literal`.
    ///
    /// # Errors
    /// Refuses a symbol or literal longer than `u32::MAX` bytes.
    pub fn digest(symbol: &str, literal: Option<&str>) -> Result<Self, Error> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(DOMAIN);
        length_prefixed(&mut hasher, symbol)?;
        match literal {
            None => {
                hasher.update(&[0]);
            }
            Some(literal) => {
                hasher.update(&[1]);
                length_prefixed(&mut hasher, literal)?;
            }
        }
        Ok(Self(Digest32::new(*hasher.finalize().as_bytes())))
    }

    /// The canonical rendering: 64 lowercase hexadecimal digits.
    #[must_use]
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    /// Read the canonical rendering; `None` for anything but 64 lowercase digits.
    #[must_use]
    pub fn from_hex(text: &str) -> Option<Self> {
        Digest32::from_hex(text).map(Self)
    }
}

/// One counted site of a ratcheted rule: the file, the rule id and the site key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DebtSite {
    file: RelPath,
    rule: RuleId,
    key: SiteKey,
}

impl DebtSite {
    /// Name a site.
    ///
    /// # Errors
    /// Refuses a file that is not a valid relative path and an invalid rule id.
    pub fn new(text: &str, rule: &str, key: SiteKey) -> Result<Self, Error> {
        let file = RelPath::new(text)
            .map_err(|error| E::Value(format!("debt site file {text:?}: {error}")))?;
        if text.is_empty() || file.to_string() != text {
            return Err(E::Value(format!(
                "debt site file {text:?} is not a canonical nonempty relative path"
            ))
            .into());
        }
        let rule = RuleId::new(rule).map_err(|error| E::Value(error.to_string()))?;
        Ok(Self { file, rule, key })
    }

    /// The file, relative to the workspace root.
    #[must_use]
    pub const fn file(&self) -> &RelPath {
        &self.file
    }

    /// The id of the ratcheted rule.
    #[must_use]
    pub const fn rule(&self) -> &RuleId {
        &self.rule
    }

    /// The stable per-site key.
    #[must_use]
    pub const fn key(&self) -> SiteKey {
        self.key
    }
}
