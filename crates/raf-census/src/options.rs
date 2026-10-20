// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The limits of one census run.

use crate::Error;

/// The default smallest body, in value-form tokens, that has a `body`
/// fingerprint.
pub const DEFAULT_MIN_TOKENS: usize = 20;

/// The default cap on one source file: 8 MiB.
pub const DEFAULT_MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;

/// The options of one census run.
///
/// The options are `#[non_exhaustive]`: build them from [`Default`] and the
/// setters, which refuse a value the census cannot honour.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CensusOptions {
    /// A body with fewer value-form tokens has no `body` fingerprint.
    pub min_tokens: usize,
    /// The most bytes one source file may hold; a larger file is a load error.
    pub max_file_bytes: u64,
}

impl Default for CensusOptions {
    fn default() -> Self {
        Self {
            min_tokens: DEFAULT_MIN_TOKENS,
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
        }
    }
}

impl CensusOptions {
    /// Sets the smallest body that has a `body` fingerprint.
    ///
    /// # Errors
    ///
    /// [`Error::Options`] when `min_tokens` is 0, which would fingerprint
    /// every empty body.
    pub fn min_tokens(mut self, min_tokens: usize) -> Result<Self, Error> {
        if min_tokens == 0 {
            return Err(Error::Options(
                "min_tokens is 0; it must be at least 1".to_owned(),
            ));
        }
        self.min_tokens = min_tokens;
        Ok(self)
    }

    /// Sets the cap on the size of one source file.
    #[must_use]
    pub fn max_file_bytes(mut self, max_file_bytes: u64) -> Self {
        self.max_file_bytes = max_file_bytes;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_twenty_tokens_and_eight_mebibytes() {
        let options = CensusOptions::default();
        assert_eq!(options.min_tokens, 20);
        assert_eq!(options.max_file_bytes, 8 * 1024 * 1024);
    }

    #[test]
    fn zero_min_tokens_is_refused_and_one_is_accepted() {
        let refused = CensusOptions::default()
            .min_tokens(0)
            .expect_err("0 is refused");
        assert!(matches!(refused, Error::Options(_)));
        let accepted = CensusOptions::default()
            .min_tokens(1)
            .expect("1 is accepted");
        assert_eq!(accepted.min_tokens, 1);
    }

    #[test]
    fn max_file_bytes_is_set() {
        assert_eq!(
            CensusOptions::default().max_file_bytes(10).max_file_bytes,
            10
        );
    }
}
