// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Static change creation, impact, reconciliation, verification and completion.
//!
//! [`Lifecycle::open`] reads a workspace and its native Git history. Reconciliation
//! combines local document obligations with merge-base comparisons; verification
//! reports declared source bindings and static findings. Neither operation executes
//! tests, examples, documentation tests or other programs.
//!
//! Completion validates outcomes, proposed text, generated documentation, source
//! bindings and current assessments against a prospective document overlay. It
//! reopens static inputs before publication, then writes owned generated documents
//! and removes the completed change record. No execution worktree or process
//! supervisor is involved.

mod changed;
mod complete;
mod completion;
mod create;
mod impact;
mod session;
mod touched;
pub(crate) use touched::BaseTree;
mod untouched;
mod verification;

pub use completion::{Completion, Condition, Failure};
pub use create::{
    AffectsProblem, CapabilityGuidance, NewChange, NewChangeError, NewChangeRequest, StateAdvice,
};
pub use impact::{Dependant, Impact, LinkingSection};
pub use session::{Lifecycle, LifecycleError, Side};
pub use untouched::{SectionDigest, section_digest};
pub use verification::{VerifyReport, verify};

#[cfg(test)]
mod tests;
