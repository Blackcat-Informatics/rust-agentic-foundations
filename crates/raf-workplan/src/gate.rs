// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The dev-dependency gate: [`Gate::workspace`] loads the documents and the
//! source of a checkout from disk, runs [`Workplan::check`] and, in
//! [`Gate::assert_clean`], panics with every finding.

use std::error::Error;
use std::fmt::{self, Write as _};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use raf_okf::{DocType, Location};
use raf_source::{DiskTree, EntryKind, LoadError, RelPath, Tree, TreeError, Workspace};

use crate::{Finding, LinkContext, Workplan};

/// The name of the navigation file of the capability directory, which is not a
/// capability document.
const INDEX_NAME: &str = "index.md";

/// Why a [`Gate`] cannot give a verdict. A gate that cannot look at the
/// repository does not pass it: every case is an error and none is a skip.
#[derive(Debug)]
#[non_exhaustive]
pub enum GateError {
    /// The directory cannot be opened as a tree.
    Root {
        /// The directory the gate was given.
        dir: PathBuf,
        /// Why it cannot be opened.
        source: TreeError,
    },
    /// `docs/capabilities` does not exist, cannot be listed, or holds no
    /// capability document. A repository with nothing to check must not pass.
    NoCapabilities {
        /// The directory the gate was given.
        dir: PathBuf,
        /// What is wrong with `docs/capabilities`.
        reason: String,
    },
    /// The source workspace does not load.
    Source {
        /// The directory the gate was given.
        dir: PathBuf,
        /// The source roots the load was given.
        roots: Vec<String>,
        /// Why the load failed.
        source: Box<LoadError>,
    },
    /// The tracked paths of a git checkout cannot be listed.
    Tracked {
        /// The directory the gate was given.
        dir: PathBuf,
        /// Why.
        reason: String,
    },
}

impl fmt::Display for GateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Root { dir, source } => write!(
                f,
                "cannot open {} as a repository tree: {source}; pass the directory that holds `docs/` and `work/`",
                dir.display()
            ),
            Self::NoCapabilities { dir, reason } => write!(
                f,
                "{}/{} {reason}; a repository with no capability document has nothing to check, so the gate refuses it: create the capability documents, or pass the repository root",
                dir.display(),
                DocType::Capability.directory(),
            ),
            Self::Source { dir, roots, source } => write!(
                f,
                "the source workspace under {} (roots {roots:?}) does not load: {source}; name the directories that hold the packages with `Gate::source_roots`",
                dir.display()
            ),
            Self::Tracked { dir, reason } => write!(
                f,
                "the tracked paths of {} cannot be listed: {reason}; pass them with `Gate::tracked_paths`",
                dir.display()
            ),
        }
    }
}

impl Error for GateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Root { source, .. } => Some(source),
            Self::Source { source, .. } => Some(source),
            Self::NoCapabilities { .. } | Self::Tracked { .. } => None,
        }
    }
}

/// The checks of a repository on disk, for a test of the repository itself.
///
/// ```rust,no_run
/// // tests/workplan.rs
/// #[test]
/// fn workplan() {
///     raf_workplan::Gate::workspace(env!("CARGO_MANIFEST_DIR")).assert_clean();
/// }
/// ```
///
/// [`Gate::workspace`] names the directory. The gate reads `docs/capabilities`,
/// `docs/architecture` and `work/changes` below it, loads the source
/// [`Workspace`] with the roots `["."]` (change them with
/// [`Gate::source_roots`]), and runs [`Workplan::check`].
///
/// A repository with no `docs/capabilities` directory, or with a directory
/// that holds no capability document, is an error and not a clean verdict: a
/// gate pointed at the wrong directory must not pass. A missing `work/changes`
/// is the empty set of changes.
///
/// Links are judged against the tracked paths of the checkout. They are taken
/// from `git ls-files` when the directory is inside a git work tree. If `git`
/// cannot run, fails for another reason, is told by `GIT_DIR` or
/// `GIT_WORK_TREE` about a repository it cannot use, or cannot list the index,
/// the gate is an error ([`GateError::Tracked`]).
///
/// There is one fallback, taken only when `git` itself answers that the
/// directory is not inside a git work tree, as for a copy of the files without
/// `.git` such as the snapshot of a commit hook. Then every file under the
/// directory stands for a tracked path (see [`LinkContext::of_tree`]). That
/// weakens the rule that a link to an untracked file is broken, since a file
/// that is on disk and untracked then passes. [`Gate::tracked_paths`] replaces
/// both with a list the caller owns. A path that is a symlink on disk is a
/// tracked symlink, and a link to it is `link-symlink`.
#[derive(Debug, Clone)]
pub struct Gate {
    dir: PathBuf,
    roots: Vec<String>,
    tracked: Option<Vec<RelPath>>,
}

impl Gate {
    /// A gate over the checkout in `dir`: the directory that holds `docs/`
    /// and `work/`, usually the manifest directory of the package.
    #[must_use]
    pub fn workspace(dir: impl AsRef<Path>) -> Self {
        Self {
            dir: dir.as_ref().to_path_buf(),
            roots: vec![".".to_string()],
            tracked: None,
        }
    }

    /// Load the source workspace from these roots, relative to the directory,
    /// instead of `["."]`.
    #[must_use]
    pub fn source_roots(mut self, roots: &[&str]) -> Self {
        self.roots = roots.iter().map(|root| (*root).to_string()).collect();
        self
    }

    /// Judge links against exactly these tracked paths, instead of the ones
    /// `git ls-files` lists or the files of the directory.
    #[must_use]
    pub fn tracked_paths(mut self, paths: impl IntoIterator<Item = RelPath>) -> Self {
        self.tracked = Some(paths.into_iter().collect());
        self
    }

    /// Every finding of the checks over the repository.
    ///
    /// # Errors
    ///
    /// A [`GateError`] when the gate cannot look at the repository: the
    /// directory does not open, `docs/capabilities` is missing or holds no
    /// capability document, the source workspace does not load, or the
    /// tracked paths of a git checkout cannot be listed.
    pub fn try_check(&self) -> Result<Vec<Finding>, GateError> {
        let tree = DiskTree::new(&self.dir).map_err(|source| GateError::Root {
            dir: self.dir.clone(),
            source,
        })?;
        self.require_capabilities(&tree)?;
        let roots: Vec<&str> = self.roots.iter().map(String::as_str).collect();
        let source = Workspace::load(&tree, &roots).map_err(|source| GateError::Source {
            dir: self.dir.clone(),
            roots: self.roots.clone(),
            source: Box::new(source),
        })?;
        let context = match self.tracked_list()? {
            Some(paths) => {
                let symlinks: Vec<RelPath> = paths
                    .iter()
                    .filter(|path| self.is_symlink(path))
                    .cloned()
                    .collect();
                LinkContext::new(&tree, paths).with_symlinks(symlinks)
            }
            None => LinkContext::of_tree(&tree),
        };
        Ok(Workplan::load_with(&tree, &source, &context).check())
    }

    /// Every finding of the checks over the repository.
    ///
    /// # Panics
    ///
    /// Panics with the [`GateError`] when the gate cannot look at the
    /// repository; see [`Gate::try_check`].
    #[must_use]
    pub fn check(&self) -> Vec<Finding> {
        match self.try_check() {
            Ok(findings) => findings,
            Err(error) => panic!("workplan gate: {error}"),
        }
    }

    /// Panic unless the repository has no finding. The message lists every
    /// finding as `kind file:line message (fix)`.
    ///
    /// # Panics
    ///
    /// Panics with every finding, or with the [`GateError`] when the gate
    /// cannot look at the repository.
    pub fn assert_clean(&self) {
        let findings = self.check();
        assert!(findings.is_empty(), "{}", report(&self.dir, &findings));
    }

    /// Require a `docs/capabilities` directory that holds a capability
    /// document.
    fn require_capabilities(&self, tree: &DiskTree) -> Result<(), GateError> {
        let missing = |reason: String| GateError::NoCapabilities {
            dir: self.dir.clone(),
            reason,
        };
        let directory = RelPath::new(DocType::Capability.directory())
            .map_err(|error| missing(format!("is not a path: {error}")))?;
        let entries = match tree.list(&directory) {
            Ok(entries) => entries,
            Err(TreeError::NotFound { .. }) => return Err(missing("does not exist".to_string())),
            Err(error) => return Err(missing(format!("cannot be listed: {error}"))),
        };
        let has_document = entries
            .iter()
            .any(|entry| entry.kind != EntryKind::Dir && is_document_name(&entry.name));
        if has_document {
            Ok(())
        } else {
            Err(missing("holds no capability document".to_string()))
        }
    }

    /// The tracked paths the links are judged against, or `None` when every
    /// file of the directory stands for one.
    fn tracked_list(&self) -> Result<Option<Vec<RelPath>>, GateError> {
        if let Some(paths) = &self.tracked {
            return Ok(Some(paths.clone()));
        }
        git_tracked(&self.dir)
    }

    /// Whether the path is a symlink on disk. A path that is not there is not
    /// a symlink: the link rules report a link to it as they do for any
    /// tracked path.
    fn is_symlink(&self, path: &RelPath) -> bool {
        fs::symlink_metadata(self.dir.join(path.as_str()))
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
    }
}

/// Whether a file name in the capability directory is a capability document:
/// Markdown other than the index.
#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "the profile defines a document file name by the lowercase suffix `.md`"
)]
fn is_document_name(name: &str) -> bool {
    name.ends_with(".md") && name != INDEX_NAME
}

/// The tracked paths of the git work tree that holds `dir`, relative to `dir`.
///
/// `None` is returned in exactly one case: `git` runs and answers that `dir` is
/// not inside a git work tree (it is outside any repository, or inside a
/// `.git` directory). Every other failure is an error: `git` cannot run, `git`
/// fails for another reason, `GIT_DIR` or `GIT_WORK_TREE` names a repository
/// that cannot be used, or `git ls-files` fails inside a work tree.
fn git_tracked(dir: &Path) -> Result<Option<Vec<RelPath>>, GateError> {
    let tracked = |reason: String| GateError::Tracked {
        dir: dir.to_path_buf(),
        reason,
    };
    let inside = Command::new("git")
        .env("LC_ALL", "C")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map_err(|error| tracked(format!("cannot run `git`: {error}")))?;
    if inside.status.success() {
        match inside.stdout.trim_ascii() {
            b"true" => {}
            b"false" => return Ok(None),
            other => {
                return Err(tracked(format!(
                    "`git rev-parse --is-inside-work-tree` answered `{}`",
                    String::from_utf8_lossy(other)
                )));
            }
        }
    } else {
        let message = String::from_utf8_lossy(&inside.stderr);
        let environment = ["GIT_DIR", "GIT_WORK_TREE"]
            .into_iter()
            .find(|name| std::env::var_os(name).is_some());
        return match environment {
            Some(name) => Err(tracked(format!(
                "{name} is set and `git` cannot use it: {}",
                message.trim()
            ))),
            None if message.contains("not a git repository") => Ok(None),
            None => Err(tracked(format!(
                "`git rev-parse --is-inside-work-tree` failed: {}",
                message.trim()
            ))),
        };
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|error| tracked(format!("cannot run `git ls-files`: {error}")))?;
    if !output.status.success() {
        return Err(tracked(format!(
            "`git ls-files` failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let mut paths = Vec::new();
    for name in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
    {
        let text = std::str::from_utf8(name)
            .map_err(|error| tracked(format!("a tracked path is not UTF-8: {error}")))?;
        paths.push(
            RelPath::new(text)
                .map_err(|error| tracked(format!("`{text}` is not a path: {error}")))?,
        );
    }
    Ok(Some(paths))
}

/// The panic message of a gate with findings: one line for each, as
/// `kind file:line message (fix)`.
fn report(dir: &Path, findings: &[Finding]) -> String {
    let mut text = format!(
        "workplan gate: {} finding(s) in {}:\n",
        findings.len(),
        dir.display()
    );
    for finding in findings {
        let _ = writeln!(
            text,
            "  {} {} {} ({})",
            finding.kind,
            place(finding),
            finding.message,
            finding.fix
        );
    }
    text
}

/// `file:line` for a finding at a line, and the file with the key or the
/// section otherwise.
fn place(finding: &Finding) -> String {
    match &finding.location {
        Location::Line(line) => format!("{}:{line}", finding.file),
        Location::Key(key) => format!("{} (key `{key}`)", finding.file),
        Location::Section(name) => format!("{} (section `{name}`)", finding.file),
        _ => finding.file.clone(),
    }
}
