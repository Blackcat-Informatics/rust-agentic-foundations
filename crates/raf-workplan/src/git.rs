// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native read-only repository queries. No Git executable is discovered or started.

use std::error::Error;
use std::fmt;

use crate::identifiers::{Commit, PatternError, RefName, SourcePath};
use raf_okf::MAX_TEXT_BYTES;
use raf_source::RelPath;

pub use raf_source::git::Repository;

/// A locally present commit admitted through immutable native object reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCommit(Commit);
impl ResolvedCommit {
    /// The admitted full commit identifier.
    #[must_use]
    pub const fn commit(&self) -> &Commit {
        &self.0
    }
    /// The admitted identifier as lowercase text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
impl fmt::Display for ResolvedCommit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// The selected common ancestor of HEAD and a default branch commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeBase(ResolvedCommit);
impl MergeBase {
    /// The admitted ancestor for historical reads.
    #[must_use]
    pub const fn resolved(&self) -> &ResolvedCommit {
        &self.0
    }
    /// The full ancestor identifier.
    #[must_use]
    pub const fn commit(&self) -> &Commit {
        self.0.commit()
    }
    /// The full ancestor identifier as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
impl fmt::Display for MergeBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// The selected directory's path below its worktree root, ending with `/`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Prefix(String);
impl Prefix {
    /// Empty at the checkout root; otherwise a relative directory and `/`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Whether selection is the checkout root.
    #[must_use]
    pub fn is_top(&self) -> bool {
        self.0.is_empty()
    }
    /// Join one validated workspace path to the repository prefix.
    ///
    /// # Errors
    /// Refuses a joined path outside the source-path profile.
    pub fn join(&self, path: &SourcePath) -> Result<SourcePath, PatternError> {
        SourcePath::new(&format!("{}{path}", self.0))
    }
}

/// A hard refusal of a native Git read; no failure becomes partial success.
#[derive(Debug)]
#[non_exhaustive]
pub enum GitError {
    /// Original native repository, object or decoding failure.
    Native(raf_source::git::Error),
    /// Native data does not satisfy the selected closed profile.
    Answer {
        /// The operation whose data was refused.
        command: String,
        /// The required profile.
        expected: &'static str,
        /// The refused value, bounded for display.
        answer: String,
    },
    /// An acquired path is not UTF-8.
    NotUtf8 {
        /// Original conversion reason.
        reason: String,
    },
    /// An acquired path is not a relative tree path.
    NotAPath {
        /// The refused path.
        path: String,
        /// Original path-profile reason.
        reason: String,
    },
    /// Origin HEAD is absent or does not select a symbolic default branch.
    NoDefaultBranch {
        /// Original native diagnostic.
        stderr: String,
    },
    /// The selected histories have no common ancestor.
    NoMergeBase {
        /// The selected default branch commit.
        default: Commit,
    },
    /// A complete history result exceeds the supported byte bound.
    LogTooLarge {
        /// The selected history operation.
        command: String,
        /// The supported cumulative byte limit.
        limit: usize,
    },
    /// A historical document exceeds the document byte bound.
    BlobTooLarge {
        /// Repository-relative source path.
        path: SourcePath,
        /// The maximum admitted bytes.
        limit: usize,
    },
}
impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native(error) => error.fmt(f),
            Self::Answer {
                command,
                expected,
                answer,
            } => write!(
                f,
                "native Git {command}: expected {expected}, found {}",
                answer.escape_debug()
            ),
            Self::NotUtf8 { reason } => write!(f, "Git path is not UTF-8: {reason}"),
            Self::NotAPath { path, reason } => {
                write!(f, "Git path {}: {reason}", path.escape_debug())
            }
            Self::NoDefaultBranch { stderr } => write!(
                f,
                "the repository has no symbolic origin default branch: {stderr}; git remote set-head origin --auto sets refs/remotes/origin/HEAD to the remote default"
            ),
            Self::NoMergeBase { default } => write!(
                f,
                "HEAD and default commit {default} have no common ancestor"
            ),
            Self::LogTooLarge { command, limit } => write!(
                f,
                "native Git {command} exceeds {limit} bytes; no partial history is returned"
            ),
            Self::BlobTooLarge { path, limit } => write!(
                f,
                "historical document {path} exceeds its {limit} byte bound"
            ),
        }
    }
}
impl Error for GitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Native(error) => Some(error),
            _ => None,
        }
    }
}
impl From<raf_source::git::Error> for GitError {
    fn from(error: raf_source::git::Error) -> Self {
        Self::Native(error)
    }
}

pub(super) const QUERY_BYTES: usize = 16 * 1024 * 1024;

fn answer(operation: &str, expected: &'static str, value: &str) -> GitError {
    GitError::Answer {
        command: operation.into(),
        expected,
        answer: value.chars().take(80).collect(),
    }
}
fn admitted(text: &str) -> Result<ResolvedCommit, GitError> {
    Commit::new(text)
        .map(ResolvedCommit)
        .map_err(|_| answer("commit", "a full lowercase commit identifier", text))
}

/// Read the selected directory's prefix below its worktree root.
///
/// # Errors
/// Refuses bare or outside-worktree selections and unsupported path spellings.
pub fn show_prefix(repository: &Repository) -> Result<Prefix, GitError> {
    let root = repository
        .work_dir()
        .ok_or_else(|| answer("prefix", "a worktree", "bare repository"))?;
    let relative = repository
        .directory()
        .strip_prefix(root)
        .map_err(|e| answer("prefix", "a directory inside the worktree", &e.to_string()))?;
    if relative.as_os_str().is_empty() {
        return Ok(Prefix::default());
    }
    let text = relative.to_str().ok_or_else(|| GitError::NotUtf8 {
        reason: "selected directory prefix".into(),
    })?;
    let checked = SourcePath::new(text).map_err(|e| GitError::NotAPath {
        path: text.into(),
        reason: e.to_string(),
    })?;
    Ok(Prefix(format!("{checked}/")))
}

/// Read the selected index, never substituting checkout files for a failed index.
///
/// # Errors
/// Refuses malformed, oversized or inaccessible indexes and invalid paths.
pub fn tracked_paths(repository: &Repository) -> Result<Option<Vec<RelPath>>, GitError> {
    repository
        .tracked_paths(QUERY_BYTES)?
        .map(|names| {
            names
                .into_iter()
                .map(|name| {
                    RelPath::new(&name).map_err(|error| GitError::NotAPath {
                        path: name,
                        reason: error.to_string(),
                    })
                })
                .collect()
        })
        .transpose()
}

/// Read origin HEAD's exact full symbolic reference.
///
/// # Errors
/// Refuses an absent/direct default reference or an invalid reference profile.
pub fn default_branch_ref(repository: &Repository) -> Result<RefName, GitError> {
    let reference = repository
        .default_branch_ref()
        .map_err(|error| GitError::NoDefaultBranch {
            stderr: error.to_string(),
        })?;
    RefName::new(&reference).map_err(|_| answer("origin HEAD", "a full reference name", &reference))
}

/// Admit the selected checkout's HEAD as an existing commit.
///
/// # Errors
/// Refuses unborn, missing, corrupt or non-commit HEAD objects.
pub fn head(repository: &Repository) -> Result<ResolvedCommit, GitError> {
    admitted(&repository.resolve_commit("HEAD")?)
}

/// Resolve one full reference to an existing commit.
///
/// # Errors
/// Refuses missing or incorrectly typed reference targets.
pub fn resolve_commit(
    repository: &Repository,
    reference: &RefName,
) -> Result<ResolvedCommit, GitError> {
    admitted(&repository.resolve_commit(reference.as_str())?)
}

/// Admit one exact full identifier as a locally present commit.
///
/// # Errors
/// Refuses missing, corrupt, incorrectly typed or incompatible-hash objects.
pub fn resolve_commit_id(
    repository: &Repository,
    commit: &Commit,
) -> Result<ResolvedCommit, GitError> {
    repository.resolve_commit(commit.as_str())?;
    Ok(ResolvedCommit(commit.clone()))
}

/// Find the best common ancestor of HEAD and one admitted default commit.
///
/// # Errors
/// Refuses unrelated histories and native traversal failures.
pub fn merge_base(repository: &Repository, default: &Commit) -> Result<MergeBase, GitError> {
    let current = head(repository)?;
    repository
        .merge_base(current.as_str(), default.as_str())?
        .ok_or_else(|| GitError::NoMergeBase {
            default: default.clone(),
        })
        .and_then(|id| admitted(&id).map(MergeBase))
}

/// Select origin's symbolic default, resolve it once, then find its merge base.
///
/// # Errors
/// As [`default_branch_ref`], [`resolve_commit`] and [`merge_base`].
pub fn default_merge_base(repository: &Repository) -> Result<MergeBase, GitError> {
    merge_base(
        repository,
        resolve_commit(repository, &default_branch_ref(repository)?)?.commit(),
    )
}

/// Read a bounded historical document; only a genuinely absent path is `None`.
///
/// # Errors
/// Refuses directories, submodules, missing objects, corruption and oversized blobs.
pub fn blob_at(
    repository: &Repository,
    revision: &ResolvedCommit,
    path: &SourcePath,
) -> Result<Option<Vec<u8>>, GitError> {
    let Some(entry) = repository.entry_at(revision.as_str(), path.as_str())? else {
        return Ok(None);
    };
    if !matches!(
        entry.kind,
        raf_source::git::EntryKind::Blob | raf_source::git::EntryKind::Symlink
    ) {
        return Err(answer("blob", "a blob path", path.as_str()));
    }
    if entry.size > MAX_TEXT_BYTES {
        return Err(GitError::BlobTooLarge {
            path: path.clone(),
            limit: MAX_TEXT_BYTES,
        });
    }
    Ok(Some(repository.blob(&entry.id, MAX_TEXT_BYTES)?))
}

/// A committed leaf's mode; directories are represented by descendant paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TreeEntry {
    /// A regular file.
    File,
    /// A symbolic-link blob.
    Symlink,
    /// A submodule locator.
    Submodule,
}

/// Read a complete bounded tree below one selected repository prefix.
///
/// # Errors
/// Refuses missing/corrupt objects, unsupported paths and cumulative bounds.
pub(crate) fn tree_at(
    repository: &Repository,
    revision: &ResolvedCommit,
    prefix: &Prefix,
) -> Result<Vec<(RelPath, TreeEntry)>, GitError> {
    let mut output = Vec::new();
    for entry in repository.tree(revision.as_str(), QUERY_BYTES)? {
        let kind = match entry.kind {
            raf_source::git::EntryKind::Tree => continue,
            raf_source::git::EntryKind::Blob => TreeEntry::File,
            raf_source::git::EntryKind::Symlink => TreeEntry::Symlink,
            raf_source::git::EntryKind::Submodule => TreeEntry::Submodule,
        };
        let Some(path) = entry.path.strip_prefix(prefix.as_str()) else {
            continue;
        };
        let path = RelPath::new(path).map_err(|error| GitError::NotAPath {
            path: entry.path.clone(),
            reason: error.to_string(),
        })?;
        output.push((path, kind));
    }
    Ok(output)
}

mod log;
pub(crate) use log::start_of_day;
pub use log::{
    AuthorDate, LastChange, LogEntry, document_log, is_shallow, last_change, symbol_log,
};
