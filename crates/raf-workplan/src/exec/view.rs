// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Explicit captured repository views for completion verification.

mod capture;
mod filesystem;
mod git_context;
pub(crate) use filesystem::FilesystemView;
pub(crate) use git_context::GitContext;
mod host;
mod materialized;
mod tree;
mod virtual_views;
pub use virtual_views::VirtualViews;

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use raf_source::{RelPath, Tree};

use super::ExecError;

pub(crate) use host::HostViews;

/// Whether view preparation failed natively, refused admission, or observed a change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewErrorKind {
    /// An underlying filesystem, JSON or process operation failed.
    Native,
    /// The requested view or execution context is not admitted.
    Refused,
    /// Previously captured source/model facts changed.
    Changed,
}

/// A native refusal while preparing, using or releasing a repository view.
#[derive(Debug, Clone)]
pub struct ViewError {
    kind: ViewErrorKind,
    operation: String,
    source: Arc<dyn Error + Send + Sync>,
}

impl ViewError {
    pub(crate) fn new(
        operation: impl Into<String>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind: ViewErrorKind::Native,
            operation: operation.into(),
            source: Arc::new(source),
        }
    }
    /// Typed distinction between a native failure, admission refusal and source change.
    #[must_use]
    pub fn kind(&self) -> ViewErrorKind {
        self.kind
    }
    pub(crate) fn refusal(operation: &str, reason: impl Into<String>) -> Self {
        let mut error = Self::new(
            operation,
            std::io::Error::new(std::io::ErrorKind::InvalidInput, reason.into()),
        );
        error.kind = ViewErrorKind::Refused;
        error
    }
    pub(crate) fn changed(operation: &str, reason: impl Into<String>) -> Self {
        let mut error = Self::new(
            operation,
            std::io::Error::new(std::io::ErrorKind::Interrupted, reason.into()),
        );
        error.kind = ViewErrorKind::Changed;
        error
    }
}
impl PartialEq for ViewError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.operation == other.operation
            && self.source.to_string() == other.source.to_string()
    }
}
impl Eq for ViewError {}
impl fmt::Display for ViewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.operation, self.source)
    }
}
impl Error for ViewError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}
impl From<ViewError> for ExecError {
    fn from(source: ViewError) -> Self {
        Self::View(source)
    }
}

/// The precise transformation requested by completion, before any publication.
#[derive(Debug, Clone)]
pub struct ViewRequest {
    pub(crate) root: PathBuf,
    pub(crate) roots: Vec<String>,
    pub(crate) tracked: Option<Vec<RelPath>>,
    pub(crate) required: Vec<RelPath>,
    pub(crate) hidden: RelPath,
    pub(crate) rewritten: Vec<(RelPath, String)>,
}
impl ViewRequest {
    /// Original logical workspace root; this is not a copied file's identity.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
    /// Native first-party roots used for the captured source index.
    #[must_use]
    pub fn source_roots(&self) -> &[String] {
        &self.roots
    }
    /// The selected record which the completed view does not contain.
    #[must_use]
    pub fn removed(&self) -> &RelPath {
        &self.hidden
    }
    /// Exact generated replacements requested by the static model.
    #[must_use]
    pub fn rewrites(&self) -> &[(RelPath, String)] {
        &self.rewritten
    }
    /// The caller's original tracked-path oracle.
    #[must_use]
    pub fn tracked(&self) -> Option<&[RelPath]> {
        self.tracked.as_deref()
    }
}

/// An immutable original capture and its completed transformation.
#[derive(Debug)]
pub struct CapturedView {
    pub(crate) request: ViewRequest,
    pub(crate) capture: capture::Capture,
}
impl CapturedView {
    /// The original request and tracked provenance.
    #[must_use]
    pub fn request(&self) -> &ViewRequest {
        &self.request
    }
    /// Captured original files, before record removal and generated replacements.
    #[must_use]
    pub fn original(&self) -> &dyn Tree {
        &self.capture
    }
    /// Completed tracked paths, with the record absent and generated edits present.
    /// # Errors
    /// Native ordering failures refuse the completed oracle.
    pub fn tracked(&self) -> Result<Option<Vec<RelPath>>, ViewError> {
        self.request
            .tracked
            .as_ref()
            .map(|paths| {
                let mut completed: Vec<_> = paths
                    .iter()
                    .filter(|path| **path != self.request.hidden)
                    .cloned()
                    .chain(self.request.rewritten.iter().map(|(path, _)| path.clone()))
                    .collect();
                purrdf_lex::walk::try_sort_unstable_by(&mut completed, |a, b| {
                    Ok::<_, ViewError>(a.cmp(b))
                })?;
                completed.dedup();
                Ok(completed)
            })
            .transpose()
    }
    /// A deterministic digest of selected original bytes, metadata and transformation.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.capture.digest
    }
}

/// Provenance of the original tracked oracle; virtual scripts do not invent Git rows.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IndexProvenance {
    /// A real installed-Git tree of the original index, including staged modes and blobs.
    Native {
        /// Full SHA-1 tree identifier returned by Git.
        tree: String,
        /// Effective original index file, resolved from the caller's Git selectors.
        index_path: PathBuf,
        /// Effective original worktree administration directory.
        git_dir: PathBuf,
        /// Effective original common object and reference directory.
        common_dir: PathBuf,
        /// Original resolved working HEAD.
        head: String,
        /// Original default-branch selector.
        default_ref: String,
        /// Original resolved default commit.
        default_commit: String,
        /// Merge base used by the original source/model judgement.
        merge_base: String,
    },
    /// An explicitly supplied tracked-path oracle, with no claimed physical Git index.
    Supplied,
}

/// Original selection evidence shared by every invocation of one prepared view.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ViewProof {
    pub(crate) index: IndexProvenance,
    pub(crate) tracked: Option<Vec<RelPath>>,
    pub(crate) excluded: Vec<PathBuf>,
}
impl ViewProof {
    /// Where the original tracked/index facts came from.
    #[must_use]
    pub fn index(&self) -> &IndexProvenance {
        &self.index
    }
    /// Exact original logical tracked-path oracle.
    #[must_use]
    pub fn tracked(&self) -> Option<&[RelPath]> {
        self.tracked.as_deref()
    }
    /// Exact evidence-admitted exclusions, including the owned destination.
    #[must_use]
    pub fn excluded(&self) -> &[PathBuf] {
        &self.excluded
    }
    pub(crate) fn of(view: &CapturedView) -> Arc<Self> {
        Arc::new(Self {
            index: view
                .capture
                .selection
                .as_ref()
                .map_or(IndexProvenance::Supplied, |selection| {
                    IndexProvenance::Native {
                        tree: selection.index.clone(),
                        index_path: selection.index_path.clone(),
                        git_dir: selection.admin.clone(),
                        common_dir: selection.common.clone(),
                        head: selection.head.clone(),
                        default_ref: selection.default_ref.clone(),
                        default_commit: selection.default_commit.clone(),
                        merge_base: selection.merge_base.clone(),
                    }
                }),
            tracked: view.request.tracked.clone(),
            excluded: view.capture.excluded.clone(),
        })
    }
}

/// A runner-scoped execution context, retained in every invocation and receipt.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ViewContext {
    pub(crate) owner: u64,
    pub(crate) authority: Arc<u8>,
    pub(crate) slot: u64,
    pub(crate) digest: String,
    pub(crate) logical_root: PathBuf,
    pub(crate) physical_root: Option<PathBuf>,
    pub(crate) proof: Arc<ViewProof>,
    pub(crate) filesystem: Option<Arc<FilesystemView>>,
}
impl ViewContext {
    /// Native process filesystem backend, absent for virtual scripted execution.
    #[must_use]
    pub fn filesystem_backend(&self) -> Option<&'static str> {
        self.filesystem.as_ref().map(|_| "bwrap")
    }
    /// Selected Git top whose original source is replaced by checked copied mounts.
    #[must_use]
    pub fn isolated_root(&self) -> Option<&Path> {
        self.filesystem.as_ref().map(|view| view.original.as_path())
    }
    /// Authentic shared Git administration exposed read-only in the process view.
    #[must_use]
    pub fn common_git_dir(&self) -> Option<&Path> {
        self.filesystem
            .as_ref()
            .map(|view| view.exposed_common.as_path())
    }
    /// Exact owned linked-worktree administration and writable copied index.
    #[must_use]
    pub fn worktree_git_dir(&self) -> Option<&Path> {
        self.filesystem
            .as_ref()
            .map(|view| view.exposed_admin.as_path())
    }
    /// Exact copied index used by native prepared descendants.
    #[must_use]
    pub fn worktree_index(&self) -> Option<PathBuf> {
        self.worktree_git_dir()
            .map(|directory| directory.join("index"))
    }
    /// Original index, tracked-oracle and exclusion provenance.
    #[must_use]
    pub fn proof(&self) -> &ViewProof {
        &self.proof
    }

    /// Digest of the exact captured selection and completed transformation.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
    /// Logical original workspace root.
    #[must_use]
    pub fn logical_root(&self) -> &Path {
        &self.logical_root
    }
    /// Physical backing-copy root, or `None` for virtual scripted execution.
    /// Native processes see the declared logical paths through the filesystem view;
    /// process-local directory identities do not equal this backing directory.
    #[must_use]
    pub fn physical_root(&self) -> Option<&Path> {
        self.physical_root.as_deref()
    }
}

/// A prepared view whose lifetime and cleanup belong to the supplied runner.
#[derive(Debug, Clone)]
pub struct PreparedView {
    pub(crate) context: ViewContext,
    pub(crate) view: Arc<CapturedView>,
}
impl PreparedView {
    /// Context every verification invocation must carry.
    #[must_use]
    pub fn context(&self) -> &ViewContext {
        &self.context
    }
    /// The immutable original and completed models accepted by the runner.
    #[must_use]
    pub fn captured(&self) -> &CapturedView {
        &self.view
    }
    pub(crate) fn virtual_view(
        request: &ViewRequest,
        authority: Arc<u8>,
        slot: u64,
    ) -> Result<Self, ExecError> {
        let capture = capture::Capture::read(request, &[], None)?;
        let view = Arc::new(CapturedView {
            request: request.clone(),
            capture,
        });
        let owner = u64::try_from(Arc::as_ptr(&authority) as usize)
            .map_err(|e| ViewError::new("virtual authority identity", e))?;
        let context = ViewContext {
            owner,
            authority,
            slot,
            digest: view.digest().to_owned(),
            logical_root: view.capture.root.join(view.capture.prefix.as_str()),
            physical_root: None,
            proof: ViewProof::of(&view),
            filesystem: None,
        };
        Ok(Self { context, view })
    }
}

pub(crate) fn unsupported(operation: &str) -> ExecError {
    ViewError::refusal(
        operation,
        "the supplied runner does not support explicit completed repository views",
    )
    .into()
}
