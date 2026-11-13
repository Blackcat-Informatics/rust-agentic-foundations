// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Fragment correspondence, separate from the frozen whole-body normal form.
//!
//! Static source inspection never executes consumer programs or changes source.
//! Report, evidence and debt RDF operations charge the original run account.
//! Source lexing, parsing and span storage remain incompletely admitted; public
//! limits do not establish a total native resource guarantee.

mod analysis;
mod attributes;
mod collect;
mod context;
mod coordinates;
mod debt;
mod discovery;
mod error;
mod identity;
mod names;
mod native_literals;
mod native_spans;
mod options;
mod output;
mod pairing;
mod policy;
mod prelude;
mod proof;
#[cfg(target_os = "linux")]
mod publication;
mod regions;
mod report;
mod resource;
mod run;
mod schema;
mod scopes;
mod tokens;
mod views;
mod workspace;

pub use error::{Error, InspectionError, RefusalKind};
pub use options::Options;
pub use output::{DebtRefusal, OutputFailure, OutputFailureKind};
#[cfg(target_os = "linux")]
pub use publication::{DebtAction, DebtWrite, write_debt};
pub use run::{Comparison, Debt, Evidence, Run, Serialized};
pub use views::{Candidate, Function, Region, Seed, Witness};

/// The independent normative embedded allowance record in the workspace root.
pub const DEBT_FILE: &str = "census-embedded-debt.ttl";

#[cfg(test)]
mod boundaries_tests;
