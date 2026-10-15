// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The steps of a change: opening its record, reading what it touches,
//! reconciling it with the documents, verifying what it binds and completing
//! it.
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
//!
//! # Completing a change
//!
//! [`Lifecycle::complete`] is the transition from an open change to a delivered
//! one, and returns a [`Completion`]: the record was removed, or the change is
//! refused with every [`Failure`] and nothing was written. The order of its
//! steps is part of its contract.
//!
//! 1. The static conditions are judged, all of them, with no early exit: the
//!    obligation list (every kind except `example-failing`, which needs a run),
//!    the outcome of each item (a `moved` outcome names a record that exists),
//!    the PROPOSED blocks of the change in any capability document, loaded or
//!    not, the generated blocks (`docs sync` would write nothing), and `check`
//!    of the repository as it will be once the writes are done. That
//!    repository has no record, so the state rules are judged there: a
//!    `delivered` capability beside its still-open record is exactly what
//!    `complete` resolves and is not a failure, while a capability that is not
//!    `delivered` and is listed by no other open record is `no-open-record`.
//!    The author sets `state: delivered` before running `complete`; it never
//!    edits prose.
//! 2. Only when no static condition failed does it run the bound tests and
//!    examples of the listed capabilities, through the
//!    [`Executor`](crate::bound::Executor). A static failure therefore spends no
//!    process on `cargo`. A capability with empty `bindings.tests` and
//!    `bindings.examples` satisfies this condition only while it is `designed`
//!    or `considered`. A failing example is also an `example-failing`
//!    obligation, so it is reported under conditions 1 and 5.
//! 3. The repository is opened again and the static conditions are judged again,
//!    because the documents and the source can change while `cargo` runs. A
//!    failure, or a different set of bindings to run, refuses.
//! 4. The writes. They are the generated blocks that removing the record stales
//!    (the `Changed by` rows of the relationship tables), rewritten through
//!    [`Workplan::write`](crate::Workplan::write), which stages every file and
//!    then renames; and then, as the very last step, the removal of
//!    `work/changes/<slug>.md`. Nothing else is written. If the rewrite fails the
//!    record is not removed; if the removal fails after the rewrite the
//!    [`LifecycleError::Remove`] names the files already rewritten. A removed
//!    record is therefore never found beside stale blocks.

mod complete;
mod completion;
mod create;
mod examples;
mod impact;
mod session;
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
