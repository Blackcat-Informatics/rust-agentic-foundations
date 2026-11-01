// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Immutable binary resource for the independently indexed native execution helper.
//!
//! Constant evaluation embeds bytes only; it cannot run a clock or generate Rust
//! items. Runtime admission and supervision remain in the execution modules.

/// The helper image compiled by the package build script.
pub(crate) const EXEC_HOST: &[u8] =
    std::include_bytes!(std::concat!(std::env!("OUT_DIR"), "/raf-exec-host"));
