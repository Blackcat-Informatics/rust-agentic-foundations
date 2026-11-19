// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The identity of one ratcheted site: a file, a rule and a key that survives
//! line movement.

use purrdf_hash::{Domain, blake3, frame::frame_le_into, hex::Digest32};
use raf_source::RelPath;

use super::ReportError as E;
use crate::{Error, RuleId};

/// The domain of counted debt site keys.
const DOMAIN: Domain = Domain::new(b"raf-census/debt-site/v1");

/// A canonical BLAKE3-256 key for a counted site or an exact numeric bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SiteKey(Digest32);

impl SiteKey {
    /// The key of a counted site with an optional string witness.
    ///
    /// The stable owner is a symbol for pattern and job-rule debt, or a canonical
    /// file path for numeric debt.
    ///
    /// The preimage holds the domain, the symbol framed by `purrdf-hash`, one
    /// presence byte and the framed literal when present. Positions are excluded,
    /// so inserting lines above the site preserves the key.
    #[must_use]
    pub fn digest(symbol: &str, literal: Option<&str>) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(DOMAIN.as_bytes());
        frame_le_into(&mut hasher, symbol.as_bytes());
        match literal {
            None => {
                hasher.update(&[0]);
            }
            Some(literal) => {
                hasher.update(&[1]);
                frame_le_into(&mut hasher, literal.as_bytes());
            }
        }
        Self(Digest32::new(*hasher.finalize().as_bytes()))
    }

    /// The exact ledger locator for one written numeric bound.
    #[must_use]
    pub(crate) fn numeric_bound(byte: usize, text: &str) -> Self {
        let domain = Domain::new(b"raf-census/numeric-bound/v1");
        let mut hasher = blake3::Hasher::new();
        hasher.update(domain.as_bytes());
        frame_le_into(&mut hasher, &(byte as u128).to_le_bytes());
        frame_le_into(&mut hasher, text.as_bytes());
        Self(Digest32::new(*hasher.finalize().as_bytes()))
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
