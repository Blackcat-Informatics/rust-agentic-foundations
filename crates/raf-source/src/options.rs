// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Caller-set limits for loading a workspace.

use crate::budget::Budget;

/// The default cap on one file: 8 MiB.
pub const DEFAULT_MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;

/// Limits a caller sets for one load.
///
/// The token budget and the byte budgets default to the limits of
/// [`Budget::FileTokens`], [`Budget::TotalBytes`], [`Budget::ItemBytes`] and
/// [`Budget::FactBytes`]. A load that passes one is refused with
/// [`LoadErrorKind::Budget`](crate::LoadErrorKind::Budget), naming the budget,
/// the limit that applied and the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct LoadOptions {
    /// The most bytes one file may hold. A larger file is refused with
    /// [`TreeError::TooLarge`](crate::TreeError::TooLarge). Defaults to
    /// [`DEFAULT_MAX_FILE_BYTES`].
    pub max_file_bytes: u64,
    /// The most tokens one file may hold, counted by the pre-scan before the
    /// file is parsed. Parsing a file holds at most
    /// [`PARSE_BYTES_PER_TOKEN`](crate::PARSE_BYTES_PER_TOKEN) bytes per token
    /// at its peak, so this bounds that peak: about 3.1 GiB at the default of
    /// [`Budget::FileTokens`], and below 512 MiB at 320,000 tokens, which a
    /// caller on a host with little memory sets.
    pub max_file_tokens: usize,
    /// The most bytes one load may read, source files and manifests together.
    /// Defaults to the limit of [`Budget::TotalBytes`].
    pub max_total_bytes: usize,
    /// The most bytes the symbol index may keep in item records and in the
    /// entries built for each module path. Defaults to the limit of
    /// [`Budget::ItemBytes`].
    pub max_item_bytes: usize,
    /// The most bytes the facts of the files of one load may hold. Defaults to
    /// the limit of [`Budget::FactBytes`].
    pub max_fact_bytes: usize,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
            max_file_tokens: Budget::FileTokens.limit(),
            max_total_bytes: Budget::TotalBytes.limit(),
            max_item_bytes: Budget::ItemBytes.limit(),
            max_fact_bytes: Budget::FactBytes.limit(),
        }
    }
}
