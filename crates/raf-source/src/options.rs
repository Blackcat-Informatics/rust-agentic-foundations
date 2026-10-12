// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Caller-set limits for loading a workspace.

/// The default cap on one file: 8 MiB.
pub const DEFAULT_MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;

/// Limits a caller sets for one load.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct LoadOptions {
    /// The most bytes one file may hold. A larger file is refused with
    /// [`TreeError::TooLarge`](crate::TreeError::TooLarge). Defaults to
    /// [`DEFAULT_MAX_FILE_BYTES`].
    pub max_file_bytes: u64,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
        }
    }
}
