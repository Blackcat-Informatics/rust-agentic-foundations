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
//! [`CensusOptions`] holds the limits of one run and [`Error`] is the one error
//! type of the crate.
//!
//! ```rust
//! use raf_census::{CensusOptions, NORMAL_FORM, check_normal_form};
//!
//! assert!(check_normal_form(NORMAL_FORM).is_ok());
//! let options = CensusOptions::default().min_tokens(10)?;
//! assert_eq!(options.min_tokens, 10);
//! # Ok::<(), raf_census::Error>(())
//! ```

mod error;
mod options;
mod version;

pub use error::Error;
pub use options::{CensusOptions, DEFAULT_MAX_FILE_BYTES, DEFAULT_MIN_TOKENS};
pub use version::{NORMAL_FORM, check_normal_form};
