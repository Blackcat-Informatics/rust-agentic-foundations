// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The steps of a change: opening its record, reading what it touches,
//! reconciling it with the documents and verifying what it binds.
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
//!
//! [`Lifecycle::open`] is the entry point of the commands that start a process.
//! It asks `git --version` before any other process and refuses a Git older than
//! 2.30, then opens the repository ([`Gate::open`](crate::Gate::open)).
//! [`Lifecycle::reconcile`] returns the whole obligation list of a change: the
//! static kinds of [`Workplan::reconcile`](crate::Workplan::reconcile) and the
//! two that need a process.
//!
//! * `section-untouched` compares each section in `affects` with the file at the
//!   merge base of `HEAD` and the default branch of `origin`
//!   ([`git::default_merge_base`](crate::git::default_merge_base) and
//!   [`git::blob_at`](crate::git::blob_at), the only revision it reads at being
//!   a [`git::MergeBase`](crate::git::MergeBase)). A section is touched when its
//!   authored text differs ([`section_digest`]: the content of generated blocks
//!   is left out on both sides, fences and PROPOSED blocks are not); a file that
//!   is absent at the merge base counts as touched; a merge-base file that
//!   exists and does not parse is an error; a repository with no default branch
//!   is an error that names `git remote set-head origin --auto`. On the default
//!   branch itself the merge base is `HEAD`, so every affected section is owed.
//!   `git` names a file from the top of the work tree, so [`Lifecycle::open`]
//!   asks `git rev-parse --show-prefix` and a workspace below the top of its
//!   repository reads its documents at that prefix; a prefix the source-path
//!   pattern cannot carry is an error, and a linked work tree is its own top.
//! * `example-failing` builds and runs the example targets and documentation
//!   tests of the capabilities the change lists, through the
//!   [`Executor`](crate::bound::Executor); one that fails, runs zero tests or
//!   proves nothing is owed ([`RunProblem`](crate::obligation::RunProblem)).
//!   Test bindings are not run by `reconcile`.
//!
//! [`verify`] runs the bound tests and examples of one change, or of every
//! capability, and returns a [`VerifyReport`] of per-binding
//! [`Evidence`](crate::bound::Evidence). It needs no Git, goes through no
//! version gate and writes nothing.

mod create;
mod examples;
mod impact;
mod session;
mod untouched;
mod verification;

pub use create::{
    AffectsProblem, CapabilityGuidance, NewChange, NewChangeError, NewChangeRequest, StateAdvice,
};
pub use impact::{Dependant, Impact, LinkingSection};
pub use session::{Lifecycle, LifecycleError, Side};
pub use untouched::{SectionDigest, section_digest};
pub use verification::{VerifyReport, verify};

#[cfg(test)]
mod tests;
