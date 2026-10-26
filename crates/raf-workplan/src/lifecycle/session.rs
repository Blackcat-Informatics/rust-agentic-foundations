// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! [`Lifecycle`]: an opened repository and a runner, the entry point of the
//! commands that start a process.

use std::error::Error;
use std::fmt;
use std::io;

use raf_okf::ContractEffect;
use raf_source::LoadError;

use crate::bound::Executor;
use crate::exec::{ExecError, Programs, Runner, SystemRunner};
use crate::git::{self, GitError, Prefix};
use crate::lookup::{ChangeLookupError, write_findings};
use crate::obligation::{Obligation, ReconcileError, Reconciled};
use crate::{Finding, Gate, GateError, Opened, SyncError, Workplan};

use super::verification::{VerifyReport, verify};
use super::{Impact, changed, examples, touched, untouched};

/// Which version of a document a [`LifecycleError::Document`] is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Side {
    /// The file as it was at the merge base.
    MergeBase,
    /// The file as it is in the working tree.
    WorkingTree,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::MergeBase => "at the merge base",
            Self::WorkingTree => "in the working tree",
        })
    }
}

/// Why a command that starts a process cannot give its answer. Every case is
/// an error and none is a pass: the tool could not look, which is not the
/// same as finding nothing. A command-line wrapper exits with 2 for each.
#[derive(Debug)]
#[non_exhaustive]
pub enum LifecycleError {
    /// A `git` question has no answer: the Git is older than 2.30, the
    /// repository has no default branch (`git remote set-head origin --auto`
    /// sets it), the history has no merge base, a file at the merge base is
    /// over the bound, or `git` could not run.
    Git(GitError),
    /// The repository cannot be opened.
    Gate(GateError),
    /// The slug names no loaded change record.
    Change(ChangeLookupError),
    /// The change cannot be reconciled: a capability it lists did not load.
    Reconcile(ReconcileError),
    /// A process of a bound test or example could not be started or read to
    /// its end, so the run has no verdict.
    Exec(ExecError),
    /// The source index cannot say which functions are tests.
    Index(LoadError),
    /// The public surface of a library or binary target that a `code` binding
    /// reaches cannot be listed, so which bound public items a change with
    /// `contract_effect: none` touches is unknown (the complete policy).
    Surface {
        /// The package of the target.
        package: String,
        /// The crate path of the target.
        target: String,
        /// Which source index: the working tree's, or the one loaded at the
        /// merge base.
        side: Side,
        /// Why the source index cannot list it.
        reason: String,
    },
    /// A capability document cannot be compared: it is not readable, it is
    /// not UTF-8, it does not parse as a document body, or it has no such
    /// section.
    Document {
        /// The repository-relative path of the file.
        path: String,
        /// Which version of the file.
        side: Side,
        /// Why it cannot be compared.
        reason: String,
    },
    /// A command over every capability found capability documents that did
    /// not load, so what they bind is unknown.
    Unloaded {
        /// The ids of the capabilities, sorted.
        capabilities: Vec<String>,
        /// The findings about their files.
        findings: Vec<Finding>,
    },
    /// The generated blocks that the removal of a record stales cannot be
    /// computed, although `check` of the repository without the record is
    /// clean. Nothing was written.
    Sync(SyncError),
    /// The rewrite of the generated blocks failed, so the record was not
    /// removed. The files are written one at a time through a temporary file
    /// and a rename, so some of them may already hold their new text.
    Rewrite {
        /// Why the write failed.
        source: io::Error,
    },
    /// The record could not be removed, after the generated blocks were
    /// rewritten. The blocks no longer list the record.
    Remove {
        /// The repository-relative path of the record.
        path: String,
        /// The files whose generated blocks were rewritten.
        rewritten: Vec<String>,
        /// Why the removal failed.
        source: io::Error,
    },
    /// The way to the record is refused: `work` or `work/changes` is a
    /// symbolic link or not a directory, or the record is not a regular file
    /// there, so removing it could remove another file. Judged with each
    /// judgement of the static conditions: the first before anything runs,
    /// the second after the bound tests and examples ran and before the
    /// writes. Nothing was written in either case.
    RecordRefused {
        /// The refused directory, or the path of the record.
        path: String,
        /// Whether the bound tests and examples had run: `false` when the
        /// first judgement refused, `true` when the second did.
        ran: bool,
        /// Why it is refused.
        source: io::Error,
    },
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Git(error) => error.fmt(f),
            Self::Gate(error) => error.fmt(f),
            Self::Change(error) => error.fmt(f),
            Self::Reconcile(error) => error.fmt(f),
            Self::Exec(error) => error.fmt(f),
            Self::Index(error) => write!(f, "the source index cannot list the tests: {error}"),
            Self::Surface {
                package,
                target,
                side,
                reason,
            } => write!(
                f,
                "the public surface of `{target}` in the package `{package}` {side} cannot be listed, so the bound public items the change touches are unknown: {reason}"
            ),
            Self::Document { path, side, reason } => write!(
                f,
                "`{path}` {side} cannot be compared: {reason}; a section that cannot be read is not reported as touched or untouched"
            ),
            Self::Unloaded {
                capabilities,
                findings,
            } => {
                write!(
                    f,
                    "{} did not load, so what {} bind is unknown; fix the findings first",
                    capabilities.join(", "),
                    if capabilities.len() == 1 {
                        "it"
                    } else {
                        "they"
                    }
                )?;
                write_findings(f, findings)
            }
            Self::Sync(error) => write!(
                f,
                "the generated blocks that removing the record stales cannot be computed: {error}; nothing was written"
            ),
            Self::Rewrite { source } => write!(
                f,
                "the generated blocks could not be rewritten: {source}; the record was not removed; some files may already hold their new text, which `docs sync` finishes or restores"
            ),
            Self::Remove {
                path,
                rewritten,
                source,
            } => write!(
                f,
                "`{path}` could not be removed: {source}; the generated blocks of {} were already rewritten and no longer list it, so remove the file yourself, or run `docs sync` and then `change complete` again",
                if rewritten.is_empty() {
                    "no file".to_string()
                } else {
                    rewritten.join(", ")
                }
            ),
            Self::RecordRefused { path, ran, source } => write!(
                f,
                "`{path}` is refused on the way to the change record at the {}: {source}; nothing was written, so make `work` and `work/changes` plain directories that hold the record",
                if *ran {
                    "second judgement, after the bound tests and examples ran"
                } else {
                    "first judgement, before anything ran"
                }
            ),
        }
    }
}

impl Error for LifecycleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Git(error) => Some(error),
            Self::Gate(error) => Some(error),
            Self::Change(error) => Some(error),
            Self::Reconcile(error) => Some(error),
            Self::Exec(error) => Some(error),
            Self::Index(error) => Some(error),
            Self::Sync(error) => Some(error),
            Self::Rewrite { source }
            | Self::Remove { source, .. }
            | Self::RecordRefused { source, .. } => Some(source),
            Self::Document { .. } | Self::Unloaded { .. } | Self::Surface { .. } => None,
        }
    }
}

impl From<GitError> for LifecycleError {
    fn from(error: GitError) -> Self {
        Self::Git(error)
    }
}

impl From<GateError> for LifecycleError {
    fn from(error: GateError) -> Self {
        Self::Gate(error)
    }
}

impl From<ChangeLookupError> for LifecycleError {
    fn from(error: ChangeLookupError) -> Self {
        Self::Change(error)
    }
}

impl From<ReconcileError> for LifecycleError {
    fn from(error: ReconcileError) -> Self {
        Self::Reconcile(error)
    }
}

impl From<ExecError> for LifecycleError {
    fn from(error: ExecError) -> Self {
        Self::Exec(error)
    }
}

impl From<SyncError> for LifecycleError {
    fn from(error: SyncError) -> Self {
        Self::Sync(error)
    }
}

impl From<LoadError> for LifecycleError {
    fn from(error: LoadError) -> Self {
        Self::Index(error)
    }
}

/// A repository opened for the commands that start a process: `reconcile`,
/// `complete` and, in the same entry point, `verify`.
///
/// [`Lifecycle::open`] asks `git --version` first and before any other
/// process, and refuses a Git older than 2.30 (the first release whose
/// `rev-parse` handles `--end-of-options`); only then does it open the
/// repository ([`Gate::open`]). The version gate belongs to this entry point
/// and not to [`Gate::open`], and the commands that need no Git (`change new`,
/// `change impact` and [`verify`](super::verify)) do not go through it.
///
/// Every `git` and `cargo` process is started by the [`Runner`] with the
/// working directory of its [`Programs`], which is the root of the repository.
/// `git` names a file at a commit from the top of the work tree, so the open also
/// asks `git rev-parse --show-prefix` once: a workspace below the top of its
/// repository (a monorepo) reads its documents at that prefix, and a prefix that
/// the source-path pattern cannot carry is an error. A linked work tree is its
/// own top. The root must be inside a git work tree.
///
/// [`Lifecycle::complete`] opens the repository a second time, with the
/// [`Gate`] it was opened with, to judge it again before it writes.
///
/// The complete repository policy judges both reconciliation and the final
/// completion check after the change record is removed.
///
/// ```rust,no_run
/// use raf_workplan::exec::Programs;
/// use raf_workplan::lifecycle::Lifecycle;
///
/// let lifecycle = Lifecycle::open(Programs::new("."))?;
/// let owed = lifecycle.reconcile("incremental-parsing")?;
/// print!("{owed}");
/// std::process::exit(i32::from(!owed.is_empty()));
/// # Ok::<(), raf_workplan::lifecycle::LifecycleError>(())
/// ```
#[derive(Debug)]
pub struct Lifecycle<R = SystemRunner> {
    pub(super) gate: Gate,
    pub(super) opened: Opened,
    pub(super) runner: R,
    pub(super) prefix: Prefix,
}

impl Lifecycle<SystemRunner> {
    /// Open the repository in the working directory of `programs`, starting
    /// its `git` and `cargo`.
    ///
    /// # Errors
    ///
    /// [`LifecycleError::Git`] when the Git is older than 2.30, cannot be
    /// started or does not state a version (nothing else has run), or when the
    /// root is not inside a git work tree or below a directory the source-path
    /// pattern cannot carry;
    /// [`LifecycleError::Gate`] when the repository cannot be opened.
    pub fn open(programs: Programs) -> Result<Self, LifecycleError> {
        let gate = Gate::workspace(&programs.cwd).programs(programs.clone());
        Self::open_with(&gate, SystemRunner::new(programs))
    }
}

impl<R: Runner> Lifecycle<R> {
    /// Open the repository `gate` names, asking `runner` for every process of
    /// the commands, including tracked-path queries during opening and the
    /// refresh after bound runs. Gate-configured programs do not replace the
    /// supplied runner. The runner's working directory must be the gate's, which
    /// [`Lifecycle::open`] guarantees.
    ///
    /// # Errors
    ///
    /// As [`Lifecycle::open`].
    pub fn open_with(gate: &Gate, runner: R) -> Result<Self, LifecycleError> {
        git::require_version(&runner)?;
        let prefix = git::show_prefix(&runner)?;
        let opened = gate.open_with_runner(&runner)?;
        Ok(Self {
            gate: gate.clone(),
            opened,
            runner,
            prefix,
        })
    }

    /// The repository, as it was found at the open.
    #[must_use]
    pub fn opened(&self) -> &Opened {
        &self.opened
    }

    /// Where the root is below the top of its git work tree.
    #[must_use]
    pub fn prefix(&self) -> &Prefix {
        &self.prefix
    }

    /// The runner every process goes through.
    #[must_use]
    pub fn runner(&self) -> &R {
        &self.runner
    }

    /// The full obligation list of the change `change`: the static kinds of
    /// [`Workplan::reconcile`](crate::Workplan::reconcile) and the three that
    /// need a process.
    ///
    /// * `section-untouched`: for each section in `affects`, the section at
    ///   the merge base of `HEAD` and the default branch is compared with the
    ///   working tree ([`section_digest`](super::section_digest)); a section
    ///   with the same authored text is owed. `contract_effect: none` leaves
    ///   out `contract`, `usage` and `examples`. A repository with no default
    ///   branch is an error, so is a merge-base file that does not parse.
    /// * `evidence-changed`: each evidence record a claim of a listed
    ///   capability names is read at the same merge base; one that passed
    ///   there and now fails, holds a value over a limit (its own or the merge
    ///   base's), has lost a limit of the merge base, or no longer loads is
    ///   owed at each claim that names it. A record there that does not read
    ///   as an evidence record is an error.
    /// * `example-failing`: the example targets and documentation tests that
    ///   the capabilities of the change bind and that resolve are built and
    ///   run ([`Executor`]); one that fails, runs no test or proves nothing is
    ///   owed. Test bindings are not run here, and the examples of a binding
    ///   that does not resolve are not either (it is `binding-missing`).
    ///
    /// The merge base is asked for once, and only when a section or a claim
    /// needs it. The documents are loaded as they are on disk at the call.
    /// Nothing in the repository is written.
    ///
    /// # Errors
    ///
    /// [`LifecycleError::Change`] and [`LifecycleError::Reconcile`] as for
    /// [`Workplan::reconcile`](crate::Workplan::reconcile);
    /// [`LifecycleError::Git`] and [`LifecycleError::Document`] when a section
    /// or a record cannot be compared; [`LifecycleError::Exec`] when a process
    /// of an example cannot be started or read to its end.
    pub fn reconcile(&self, change: &str) -> Result<Reconciled, LifecycleError> {
        let plan = self.opened.workplan();
        let fixed = plan.reconcile(change)?;
        let impact = plan.impact(change)?;
        let effect = plan.change(change)?.value().contract_effect;
        let mut owed = fixed.obligations().to_vec();
        owed.extend(self.process_obligations(
            &self.opened,
            &plan,
            &impact,
            effect,
            true,
            &mut None,
        )?);
        Ok(Reconciled::new(fixed.change().to_string(), owed))
    }

    /// The obligations of a change that need a process, over `opened` and
    /// `plan`, its documents: `section-untouched`, `evidence-changed` and,
    /// under the complete policy for a change with `contract_effect: none`,
    /// `contract-effect-understated` from `git` at one merge base, and, when
    /// `examples` is set, `example-failing` from the executor. It is the one
    /// place [`Lifecycle::reconcile`] and
    /// [`Lifecycle::complete`](Lifecycle::complete) (which judges the examples
    /// by running them under condition 5) assemble them.
    pub(super) fn process_obligations(
        &self,
        opened: &Opened,
        plan: &Workplan<'_>,
        impact: &Impact,
        effect: ContractEffect,
        examples: bool,
        kept: &mut Option<touched::BaseIndex>,
    ) -> Result<Vec<Obligation>, LifecycleError> {
        let mut base = None;
        let mut owed = untouched::raise(
            opened,
            &self.runner,
            &self.prefix,
            impact,
            effect,
            &mut base,
        )?;
        owed.extend(changed::raise(
            opened,
            &self.runner,
            &self.prefix,
            impact,
            &mut base,
        )?);
        if effect == ContractEffect::None {
            owed.extend(touched::raise(
                plan,
                &self.runner,
                &self.prefix,
                &impact.change,
                &mut base,
                kept,
            )?);
        }
        if examples {
            let executor = Executor::new(opened.workspace(), &self.runner)?;
            owed.extend(examples::raise(&executor, impact)?);
        }
        Ok(owed)
    }

    /// Run the bound tests and examples of the change `change`, or of every
    /// capability when `change` is `None`. See [`verify`](super::verify).
    ///
    /// # Errors
    ///
    /// As [`verify`](super::verify).
    pub fn verify(&self, change: Option<&str>) -> Result<VerifyReport, LifecycleError> {
        verify(&self.opened, &self.runner, change)
    }
}
