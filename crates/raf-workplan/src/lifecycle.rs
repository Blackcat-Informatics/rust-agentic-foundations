// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The first two steps of a change: opening its record and reading what it
//! touches.
//!
//! [`Workplan::new_change`](crate::Workplan::new_change) creates `work/changes/<slug>.md` for a slug, the
//! capabilities it changes and the sections it affects, and says what the author
//! does next: the PROPOSED block to place and the state each listed capability
//! needs while the block is there ([`NewChange`]). It writes exactly one file,
//! created exclusively and never over an existing one, and reads the record
//! back through the loader before it reports success.
//!
//! [`Workplan::impact`](crate::Workplan::impact) returns what a change reaches as one [`Impact`] value:
//! the capabilities it changes, the sections it affects, the bindings of those
//! capabilities, the capabilities that depend on them and the sections that
//! link to the affected sections, because those readers are affected too. It
//! writes nothing and is a pure function of the loaded documents.

mod create;
mod impact;

pub use create::{
    AffectsProblem, CapabilityGuidance, NewChange, NewChangeError, NewChangeRequest, StateAdvice,
};
pub use impact::{Dependant, Impact, LinkingSection};
