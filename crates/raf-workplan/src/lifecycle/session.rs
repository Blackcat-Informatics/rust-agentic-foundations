// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Static change operations over an opened workspace and read-only Git data.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use raf_okf::ContractEffect;
use raf_source::LoadError;

use crate::git::{self, GitError, Prefix, Repository};
use crate::lookup::{ChangeLookupError, write_findings};
use crate::obligation::{Obligation, ReconcileError, Reconciled};
use crate::{Finding, Gate, GateError, Opened, SyncError, Workplan};

use super::verification::{VerifyReport, verify};
use super::{Impact, changed, touched, untouched};

/// Which version of a document a comparison reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Side {
    /// The version at the merge base.
    MergeBase,
    /// The version in the working tree.
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

/// Why a static change operation cannot give an answer.
#[derive(Debug)]
#[non_exhaustive]
pub enum LifecycleError {
    /// The native repository could not be opened.
    Repository {
        /// The requested workspace directory.
        path: PathBuf,
        /// The native failure.
        source: raf_source::git::Error,
    },
    /// Required history data is unavailable or invalid.
    Git(GitError),
    /// The workspace could not be opened.
    Gate(GateError),
    /// The change record is missing or invalid.
    Change(ChangeLookupError),
    /// A required capability did not load.
    Reconcile(ReconcileError),
    /// The source index could not be read.
    Index(LoadError),
    /// The relevant public surface cannot be determined.
    Surface {
        /// The owning package.
        package: String,
        /// The target within that package.
        target: String,
        /// The source version.
        side: Side,
        /// The specific failure.
        reason: String,
    },
    /// A document cannot be compared.
    Document {
        /// The repository-relative path.
        path: String,
        /// The document version.
        side: Side,
        /// The specific failure.
        reason: String,
    },
    /// Required capabilities did not load.
    Unloaded {
        /// The missing capability identifiers.
        capabilities: Vec<String>,
        /// Their findings.
        findings: Vec<Finding>,
    },
    /// Prospective generated documentation cannot be computed.
    Sync(SyncError),
    /// Generated publication failed; the record remains.
    Rewrite {
        /// The publication failure.
        source: io::Error,
    },
    /// Generated documentation was published, but record removal failed.
    Remove {
        /// The change record path.
        path: String,
        /// The documents already rewritten.
        rewritten: Vec<String>,
        /// The removal failure.
        source: io::Error,
    },
    /// A record destination or parent is not owned and regular.
    RecordRefused {
        /// The refused path.
        path: String,
        /// The ownership or file-kind failure.
        source: io::Error,
    },
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository { path, source } => write!(
                f,
                "cannot open read-only Git repository at {}: {source}",
                path.display()
            ),
            Self::Git(error) => error.fmt(f),
            Self::Gate(error) => error.fmt(f),
            Self::Change(error) => error.fmt(f),
            Self::Reconcile(error) => error.fmt(f),
            Self::Index(error) => write!(f, "cannot read source index: {error}"),
            Self::Surface {
                package,
                target,
                side,
                reason,
            } => write!(
                f,
                "cannot determine the public surface of `{target}` in `{package}` {side}: {reason}"
            ),
            Self::Document { path, side, reason } => {
                write!(f, "cannot compare `{path}` {side}: {reason}")
            }
            Self::Unloaded {
                capabilities,
                findings,
            } => {
                write!(
                    f,
                    "required capabilities did not load: {}; fix their findings",
                    capabilities.join(", ")
                )?;
                write_findings(f, findings)
            }
            Self::Sync(error) => write!(
                f,
                "cannot compute prospective generated documentation: {error}; nothing was written"
            ),
            Self::Rewrite { source } => write!(
                f,
                "generated documentation could not be rewritten: {source}; the record remains; some documents may already contain their replacements"
            ),
            Self::Remove {
                path,
                rewritten,
                source,
            } => write!(
                f,
                "cannot remove `{path}`: {source}; documentation already rewritten: {}; inspect it before retrying completion",
                rewritten.join(", ")
            ),
            Self::RecordRefused { path, source } => write!(
                f,
                "change-record destination `{path}` is refused: {source}; use regular directories and a regular record file; nothing was written"
            ),
        }
    }
}

impl Error for LifecycleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Repository { source, .. } => Some(source),
            Self::Git(error) => Some(error),
            Self::Gate(error) => Some(error),
            Self::Change(error) => Some(error),
            Self::Reconcile(error) => Some(error),
            Self::Index(error) => Some(error),
            Self::Sync(error) => Some(error),
            Self::Rewrite { source }
            | Self::Remove { source, .. }
            | Self::RecordRefused { source, .. } => Some(source),
            Self::Surface { .. } | Self::Document { .. } | Self::Unloaded { .. } => None,
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

/// An opened workspace with native read-only history for static change checks.
#[derive(Debug)]
pub struct Lifecycle {
    pub(super) gate: Gate,
    pub(super) opened: Opened,
    pub(super) repository: Repository,
    pub(super) prefix: Prefix,
}

impl Lifecycle {
    /// Open a workspace without launching a program.
    ///
    /// # Errors
    /// Refuses an unreadable workspace or unavailable native repository.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, LifecycleError> {
        Self::open_with(&Gate::workspace(root))
    }

    /// Open a configured gate without executing inspected code.
    ///
    /// # Errors
    /// Preserves workspace and native Git failures.
    pub fn open_with(gate: &Gate) -> Result<Self, LifecycleError> {
        let repository =
            Repository::open(gate.directory()).map_err(|source| LifecycleError::Repository {
                path: gate.directory().to_path_buf(),
                source,
            })?;
        let prefix = git::show_prefix(&repository)?;
        Ok(Self {
            gate: gate.clone(),
            opened: gate.open()?,
            repository,
            prefix,
        })
    }

    /// The workspace as opened for this operation.
    #[must_use]
    pub fn opened(&self) -> &Opened {
        &self.opened
    }

    /// The workspace prefix within its Git worktree.
    #[must_use]
    pub fn prefix(&self) -> &Prefix {
        &self.prefix
    }

    /// Static obligations, including comparison with the merge base.
    ///
    /// # Errors
    /// Refuses missing changes or unreadable comparison inputs.
    pub fn reconcile(&self, change: &str) -> Result<Reconciled, LifecycleError> {
        let plan = self.opened.workplan();
        let fixed = plan.reconcile(change)?;
        let impact = plan.impact(change)?;
        let effect = plan.change(change)?.value().contract_effect;
        let mut owed = fixed.obligations().to_vec();
        owed.extend(self.history_obligations(&self.opened, &plan, &impact, effect, &mut None)?);
        Ok(Reconciled::new(fixed.change().to_string(), owed))
    }

    pub(super) fn history_obligations(
        &self,
        opened: &Opened,
        plan: &Workplan<'_>,
        impact: &Impact,
        effect: ContractEffect,
        kept: &mut Option<touched::BaseIndex>,
    ) -> Result<Vec<Obligation>, LifecycleError> {
        let mut base = None;
        let mut owed = untouched::raise(
            opened,
            &self.repository,
            &self.prefix,
            impact,
            effect,
            &mut base,
        )?;
        owed.extend(changed::raise(
            opened,
            &self.repository,
            &self.prefix,
            impact,
            &mut base,
        )?);
        if effect == ContractEffect::None {
            owed.extend(touched::raise(
                plan,
                &self.repository,
                &self.prefix,
                &impact.change,
                &mut base,
                kept,
            )?);
        }
        Ok(owed)
    }

    /// Validate binding structure without running tests or examples.
    ///
    /// # Errors
    /// Refuses missing changes or unloaded capabilities.
    pub fn verify(&self, change: Option<&str>) -> Result<VerifyReport, LifecycleError> {
        verify(&self.opened, change)
    }
}
