// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The RAF census: finds helpers that a workspace implements more than once.
//!
//! A function body is rewritten into the canonical token sequence of the
//! census normal form, and its fingerprint names that sequence, so two copies
//! of one algorithm get one fingerprint. [`NORMAL_FORM`] is the version of that
//! form; a fingerprint is comparable only inside one version, and
//! [`check_normal_form`] refuses any other version, naming both.
//!
//! A [`Normalizer`] takes a [`Slice`], the text of a body with the names of its
//! function's parameters, and returns its [`Print`]; [`Normalizer::explain`]
//! compares two slices level by level.
//!
//! [`Census::load`] loads a workspace through `raf-source` and collects its
//! [`Unit`]s, one for each source item, with the print, constants, strings and
//! shim key of each.
//!
//! [`Census::groups`] lists the units that share a body or a shim, the hex
//! digit tables and the large constants shared across packages; a [`Job`]
//! names a home and what is forbidden elsewhere, and [`Census::match_jobs`]
//! classifies the units it finds.
//!
//! [`CensusOptions`] holds the limits of one run and [`Error`] is the one error
//! type of the crate.
//!
//! ```rust
//! use raf_census::{CensusOptions, Level, NORMAL_FORM, Normalizer, Slice, check_normal_form};
//!
//! assert!(check_normal_form(NORMAL_FORM).is_ok());
//! let normalizer = Normalizer::new(&CensusOptions::default());
//! let a = Slice::new("a.rs", 1, "let n = x.len(); n + 1").params(["x"]);
//! let b = Slice::new("b.rs", 1, "let k = y.len(); k + 1").params(["y"]);
//! let divergence = normalizer.explain(&a, &b)?;
//! assert_eq!(divergence.equal_from, Some(Level::LiteralTier));
//! let options = CensusOptions::default().min_tokens(10)?;
//! assert_eq!(options.min_tokens, 10);
//! # Ok::<(), raf_census::Error>(())
//! ```

mod census;
mod error;
mod group;
mod job;
mod normalize;
mod options;
mod units;
mod version;

pub use census::{Census, FileCtx};
pub use error::Error;
pub use group::{Group, GroupKind, HexTable, Member, SharedConstant};
pub use job::{Job, JobForbidden, JobMatch, JobReport, MatchClass};
pub use normalize::{
    Divergence, ENCODING, Fingerprint, Form, Hashed, LITERAL_TIER, Level, MAX_NESTING,
    MIN_SHIM_TOKENS, Normalizer, PASSES, ParseFingerprintError, Print, RuleId, SHIM_MAX_TOKENS,
    Slice, TableKind, Token, TokenDiff, Tower, VALUE_ADAPTERS, WEAK_KEYWORDS, manifest,
};
pub use options::{CensusOptions, DEFAULT_MAX_FILE_BYTES, DEFAULT_MIN_TOKENS};
pub use units::{StringKind, StringLiteral, Unit, UnitKind};
pub use version::{NORMAL_FORM, check_normal_form};
