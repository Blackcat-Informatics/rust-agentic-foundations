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
//! [`Census::baseline`] and [`Census::index`] render the two reports; both
//! are ground RDF 1.2 graphs with a closed shape. `to_turtle` and `to_ntriples`
//! write their two canonical spellings; `from_turtle` and `from_ntriples`
//! validate the selected spelling, shared snapshot digest and census semantics.
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
mod detectors;
mod error;
mod gate;
mod group;
mod job;
mod normalize;
mod options;
pub mod report;
mod rule;
mod syntax;
mod units;
mod version;

pub use census::{Census, FileCtx};
pub use error::Error;
pub use gate::{
    DEBT_FILE, DebtWrite, Gate, GateReport, LEDGER_FILE, LedgerExtension, MAX_INDEX_BYTES,
    Migration, MigrationInput, SYMBOL_ANCHOR_PREFIX, UnmappedRow, migrate_index, migrate_legacy,
};
pub use group::{Group, GroupKind, HexTable, Member, SharedConstant};
pub use job::{Job, JobForbidden, JobMatch, JobReport, MatchClass};
pub use normalize::{
    Divergence, ENCODING, Fingerprint, Form, LITERAL_TIER, Level, MAX_NESTING, MIN_SHIM_TOKENS,
    Normalizer, PASSES, ParseFingerprintError, Print, RuleId, SHIM_MAX_TOKENS, Slice, TableKind,
    Token, TokenDiff, Tower, VALUE_ADAPTERS, WEAK_KEYWORDS, manifest,
};
pub use options::{CensusOptions, DEFAULT_MAX_FILE_BYTES, DEFAULT_MIN_TOKENS};
pub use report::{
    Baseline, BaselineGroup, ConstantRow, DebtChange, DebtGroup, DebtRecord, Index, IndexGroup,
    IndexUnit, JobRow, MAX_DEBT_BYTES, MatchRow, MemberRow, REPORT_FORM, ReportError, TableRow,
};
pub use report::{
    Ledger, LedgerDistinct, LedgerJob, LedgerPattern, LedgerVariant, MAX_LEDGER_BYTES, NamePattern,
    PatternMatcher, PreferredProvider,
};
pub use rule::{Rule, RuleFixtures, RuleHit};
pub use units::{StringKind, StringLiteral, Unit, UnitKind};
pub use version::{NORMAL_FORM, check_normal_form};
