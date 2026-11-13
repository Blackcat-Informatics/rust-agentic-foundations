// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Fixtures that more than one package's tests use.
//!
//! A locked Census evaluation loads each physical source file under exactly one
//! package, so a file included by `#[path]` into two packages cannot be judged.
//! Each such helper lives here once, and every package depends on this crate
//! for its tests only. The crate is never published and no shipped crate
//! depends on it.

#![allow(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    reason = "these helpers are test fixtures: they panic on a broken environment so the calling test fails with that message"
)]

pub mod assessed;
pub mod files;
pub mod fixture_signing;
pub mod git_repo;
pub mod layout;
pub mod registry;
pub mod strict;
pub mod workspace_input;
