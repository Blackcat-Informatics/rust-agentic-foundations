// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Caller-set limits for loading a workspace.

use crate::budget::Budget;

/// The default cap on one file: 8 MiB.
pub const DEFAULT_MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;

/// Limits a caller sets for one load.
///
/// The token budget, the byte budgets and the work budgets of the load and
/// of each query after it default to the limits of [`Budget::FileTokens`],
/// [`Budget::TotalBytes`], [`Budget::ItemBytes`], [`Budget::FactBytes`],
/// [`Budget::GlobBytes`] and [`Budget::Work`]. A load that passes one is
/// refused with
/// [`LoadErrorKind::Budget`](crate::LoadErrorKind::Budget), naming the budget,
/// the limit that applied and the file.
///
/// The options are `Clone` and not `Copy`, so adding an option that is not
/// `Copy`, such as a per-file hook, is not a breaking change.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// The cumulative bytes of index records, entries built for each module
    /// path, native load traversal spills and immutable derived glob facts,
    /// including their shared allocation header. Defaults to the limit of
    /// [`Budget::ItemBytes`].
    pub max_item_bytes: usize,
    /// The most bytes the facts of the files of one load may hold. Defaults to
    /// the limit of [`Budget::FactBytes`].
    pub max_fact_bytes: usize,
    /// The most bytes import settling may hold at once: glob scope tables,
    /// retained reads, queues, subscriptions, graph scratch and impl self-type
    /// and trait-path traversal scratch, plus derived glob-fact construction.
    /// Also bounds
    /// transient selected-route determinacy proofs of each query after the
    /// load, counted from zero and released before it returns. Defaults to
    /// the limit of [`Budget::GlobBytes`].
    pub max_glob_bytes: usize,
    /// The most import-settling and impl self-type and trait-path traversal
    /// steps one load may make, including retained path segments and bytes, counted as they
    /// are made and never given back (see [`Budget::Work`]). Defaults to the
    /// limit of [`Budget::Work`].
    pub max_work: usize,
    /// The most lookup and selected-route proof steps one query after the
    /// load may make: a call of
    /// [`Workspace::resolve`](crate::Workspace::resolve), its variants or
    /// [`Workspace::exported_items_with`](crate::Workspace::exported_items_with),
    /// each counted from zero. A query that passes it fails with
    /// [`ResolveError::Budget`](crate::ResolveError::Budget) or
    /// [`ExportError::Budget`](crate::ExportError::Budget), naming
    /// [`Budget::Work`] and this limit. Defaults to the limit of
    /// [`Budget::Work`].
    pub max_query_work: usize,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
            max_file_tokens: Budget::FileTokens.limit(),
            max_total_bytes: Budget::TotalBytes.limit(),
            max_item_bytes: Budget::ItemBytes.limit(),
            max_fact_bytes: Budget::FactBytes.limit(),
            max_glob_bytes: Budget::GlobBytes.limit(),
            max_work: Budget::Work.limit(),
            max_query_work: Budget::Work.limit(),
        }
    }
}
