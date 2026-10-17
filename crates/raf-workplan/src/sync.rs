// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Writing the generated blocks: [`Workplan::sync`] computes the new text of
//! every stale or missing file, and [`Workplan::write`] puts it on disk.
//!
//! The two are apart on purpose. `sync` is a pure function of the loaded
//! documents and the source index: it reads nothing more, writes nothing, and
//! returns the whole new text of each file that has to change, so every file is
//! known good before the first byte is written. `write` is the only part that
//! touches the disk.

use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use raf_okf::{Index, ParseError};
use raf_source::{EntryKind, RelPath};

use crate::Finding;
use crate::Workplan;
use crate::files;
use crate::generated::{self, INDEX_PATH};

/// What a sync needs beyond the documents: the SPDX text of the index it
/// creates.
///
/// The index `docs/capabilities/index.md` opens with two HTML comment lines,
/// the copyright text and the licence identifier. Both are required to create
/// the index and are refused when unset; an existing index keeps its own
/// header and these fields are not read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct SyncOptions {
    /// The copyright text of the first SPDX comment line of a new index.
    pub copyright: Option<String>,
    /// The licence identifier of the second SPDX comment line of a new index.
    pub licence: Option<String>,
}

impl SyncOptions {
    /// Options with the SPDX text of a new index set.
    #[must_use]
    pub fn new(copyright: impl Into<String>, licence: impl Into<String>) -> Self {
        Self {
            copyright: Some(copyright.into()),
            licence: Some(licence.into()),
        }
    }

    /// The SPDX values of a new index, each on one line and without `-->`.
    fn spdx(&self) -> Result<(&str, &str), SyncError> {
        Ok((
            one_line("copyright", self.copyright.as_deref())?,
            one_line("licence", self.licence.as_deref())?,
        ))
    }
}

/// A value of the SPDX header: set, not empty, on one line, without `-->`.
fn one_line<'a>(field: &'static str, value: Option<&'a str>) -> Result<&'a str, SyncError> {
    match value {
        None => Err(SyncError::SpdxUnset { field }),
        Some(text) if text.trim().is_empty() => Err(SyncError::SpdxInvalid {
            field,
            reason: "it is empty",
        }),
        Some(text) if text.contains(['\n', '\r']) => Err(SyncError::SpdxInvalid {
            field,
            reason: "it spans more than one line",
        }),
        Some(text) if text.contains("-->") => Err(SyncError::SpdxInvalid {
            field,
            reason: "it contains `-->`, which would end the comment",
        }),
        Some(text) => Ok(text.trim()),
    }
}

/// The files a sync would write: the whole new text of each, sorted by path.
///
/// A report with no edit means the repository is in sync. The report holds no
/// timestamp and is equal for equal inputs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncReport {
    /// The repository-relative path and the new text of every stale or missing
    /// file.
    pub edits: Vec<(RelPath, String)>,
}

impl SyncReport {
    /// Whether nothing has to be written.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }
}

/// Why a sync cannot compute its edits. Nothing is written in any case.
#[derive(Debug)]
#[non_exhaustive]
pub enum SyncError {
    /// Files that sync refuses: a block of an unknown kind or in the wrong
    /// place, a body that mixes line endings, or an index that is not a
    /// generated index. Each finding is one `check` also reports.
    Refused(Vec<Finding>),
    /// A capability or change document is there and did not load, so the
    /// tables that read it would be wrong. Its findings are in
    /// [`Workplan::check`]; fix them first.
    Unloaded(Vec<String>),
    /// The index has to be created and an SPDX field of the options is unset.
    SpdxUnset {
        /// `copyright` or `licence`.
        field: &'static str,
    },
    /// The index has to be created and an SPDX field cannot be written into a
    /// one-line comment.
    SpdxInvalid {
        /// `copyright` or `licence`.
        field: &'static str,
        /// Why the value is refused.
        reason: &'static str,
    },
    /// The rewritten text of a file does not read back as a document, for
    /// instance because it would pass the size bound.
    Rewrite {
        /// The file.
        file: String,
        /// Why.
        reason: String,
    },
    /// The index the sync would create does not read as an index.
    NewIndex(ParseError),
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(findings) => {
                write!(f, "sync refuses {} file(s):", findings.len())?;
                for finding in findings {
                    write!(f, "\n  {finding}")?;
                }
                Ok(())
            }
            Self::Unloaded(ids) => write!(
                f,
                "sync would write tables that miss {}, which {} but did not load; fix the findings of `check` for {} first",
                ids.join(", "),
                if ids.len() == 1 { "exists" } else { "exist" },
                if ids.len() == 1 { "it" } else { "them" },
            ),
            Self::SpdxUnset { field } => write!(
                f,
                "{INDEX_PATH} does not exist and the {field} of its SPDX header is not set; set `SyncOptions::{field}`"
            ),
            Self::SpdxInvalid { field, reason } => write!(
                f,
                "the {field} of the SPDX header of {INDEX_PATH} is refused: {reason}"
            ),
            Self::Rewrite { file, reason } => {
                write!(f, "the rewritten {file} does not read back: {reason}")
            }
            Self::NewIndex(error) => {
                write!(f, "the new {INDEX_PATH} does not read as an index: {error}")
            }
        }
    }
}

impl Error for SyncError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NewIndex(error) => Some(error),
            _ => None,
        }
    }
}

impl Workplan<'_> {
    /// The edits that bring every generated block up to date.
    ///
    /// For each stale block of a capability document and of the capability
    /// index, and for a missing index, the report holds the whole new text of
    /// the file. The text outside a block is kept byte for byte, each block is
    /// written in the line ending of the body it sits in, and a file with no
    /// generated block is not in the report. The result is a pure function of
    /// the loaded documents and the source index: no timestamp, stable order.
    /// Staleness is judged by the renderers that write, the same as in
    /// [`Workplan::check`], so syncing the result leaves nothing to write.
    ///
    /// A missing `docs/capabilities/index.md` is created with the two SPDX
    /// comment lines from `options`, a blank line and the `catalogue` block.
    ///
    /// ```rust
    /// use raf_source::{MemoryTree, Workspace};
    /// use raf_workplan::{SyncOptions, Workplan};
    ///
    /// let source = Workspace::load(
    ///     &MemoryTree::from([
    ///         ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ///         ("src/lib.rs", ""),
    ///     ]),
    ///     &["."],
    /// )?;
    /// let docs = MemoryTree::from([(
    ///     "docs/architecture/overview.md",
    ///     "---\ntype: Architecture\nid: arch:overview\ntitle: Overview\n\
    ///      description: How it fits.\n---\n\n# Overview\n\nBody text.\n",
    /// )]);
    /// let plan = Workplan::load(&docs, &source);
    /// // No capability, no index to create, no block to rewrite.
    /// let report = plan.sync(&SyncOptions::default())?;
    /// assert!(report.is_empty());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`SyncError::Unloaded`] when a capability or change document is there
    /// and did not load; [`SyncError::Refused`] when a file cannot be rewritten (an unknown or
    /// misplaced block, mixed line endings, an index that is hand-written or
    /// unreadable); [`SyncError::SpdxUnset`] or [`SyncError::SpdxInvalid`] when
    /// the index has to be created and `options` do not allow it;
    /// [`SyncError::Rewrite`] and [`SyncError::NewIndex`] when a result does
    /// not read back. No edit is returned with an error.
    pub fn sync(&self, options: &SyncOptions) -> Result<SyncReport, SyncError> {
        let unloaded = self.unloaded();
        if !unloaded.is_empty() {
            return Err(SyncError::Unloaded(unloaded));
        }
        let analysis = generated::analyze(self);
        let mut refused: Vec<Finding> = analysis.invalid_index.into_iter().collect();
        refused.extend(analysis.refusals);
        if !refused.is_empty() {
            return Err(SyncError::Refused(refused));
        }
        if let Some((file, reason)) = analysis.failures.into_iter().next() {
            return Err(SyncError::Rewrite { file, reason });
        }
        let mut edits: Vec<(String, String)> = analysis.edits;
        if let Some(content) = analysis.new_index {
            let (copyright, licence) = options.spdx()?;
            let text = generated::new_index_text(copyright, licence, &content);
            Index::parse(&text).map_err(SyncError::NewIndex)?;
            edits.push((INDEX_PATH.to_string(), text));
        }
        edits.sort_by(|a, b| a.0.cmp(&b.0));
        let edits = edits
            .into_iter()
            .map(|(path, text)| {
                RelPath::new(&path)
                    .map(|path| (path, text))
                    .map_err(|error| SyncError::Rewrite {
                        file: path,
                        reason: error.to_string(),
                    })
            })
            .collect::<Result<_, _>>()?;
        Ok(SyncReport { edits })
    }

    /// Write the edits of a report under `root`.
    ///
    /// Every target is judged before the first byte is written. A target must
    /// be a regular file that already exists, or the capability index
    /// `docs/capabilities/index.md` in a directory that exists, and each
    /// directory on the way to it must be a directory and not a symbolic link,
    /// inside the root or out of it; a symbolic link, a directory, a path that
    /// leaves `root`, a path below a linked directory and any other target are
    /// refused, and so is a report that names a path twice. Every new text is
    /// first written to a temporary file in the directory of its target, and
    /// only when all of them are written are the files renamed into place, so
    /// an I/O failure while writing leaves no target changed. A file keeps its
    /// permissions.
    ///
    /// # Errors
    ///
    /// An [`io::Error`] naming the refused target, or the operating system's
    /// error. What remains possible is a rename that fails after earlier
    /// renames succeeded: those files hold the new text of a sync, the
    /// remaining temporary files are removed, and syncing again completes the
    /// rest. A temporary file is created exclusively under a unique name and
    /// only the ones this call created are removed.
    pub fn write(report: &SyncReport, root: &Path) -> io::Result<()> {
        let mut seen = std::collections::BTreeSet::new();
        for (path, _) in &report.edits {
            if !seen.insert(path.as_str()) {
                return Err(refused(path, "it is listed twice"));
            }
            judge(root, path)?;
        }
        let root = fs::canonicalize(root)?;
        let mut staged: Vec<(PathBuf, PathBuf)> = Vec::new();
        for (path, text) in &report.edits {
            let target = root.join(path.as_str());
            match stage(&target, text) {
                Ok(temporary) => staged.push((temporary, target)),
                Err(error) => {
                    discard(&staged);
                    return Err(error);
                }
            }
        }
        for (done, (temporary, target)) in staged.iter().enumerate() {
            if let Err(error) = fs::rename(temporary, target) {
                discard(&staged[done..]);
                return Err(error);
            }
        }
        Ok(())
    }
}

/// Remove the temporary files this call created.
fn discard(staged: &[(PathBuf, PathBuf)]) {
    for (temporary, _) in staged {
        // The call is failing already; a temporary file that cannot be
        // removed is only litter and must not hide the first error.
        let _ = fs::remove_file(temporary);
    }
}

/// An error that names a refused target.
fn refused(path: &RelPath, reason: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("refusing to write {path}: {reason}"),
    )
}

/// Require that `path` is a regular file, or the new capability index, below
/// directories that are not links: the way to it is judged as every write of
/// the crate judges it ([`files::target`]), so a directory link inside the
/// root never lets an edit land in another directory.
fn judge(root: &Path, path: &RelPath) -> io::Result<()> {
    let directory = path
        .parent()
        .ok_or_else(|| refused(path, "it has no directory"))?;
    let found = files::target(root, directory.as_str(), path).map_err(|error| {
        refused(
            path,
            &format!("the directory {} is refused: {}", error.path, error.source),
        )
    })?;
    if !found.directories {
        return Err(refused(path, "its directory does not exist"));
    }
    match found.kind {
        Some(EntryKind::File) => Ok(()),
        Some(_) => Err(refused(path, "it is not a regular file")),
        None if path.as_str() == INDEX_PATH => Ok(()),
        None => Err(refused(
            path,
            "it does not exist; only the capability index is created",
        )),
    }
}

/// Write `text` to a new temporary file beside `target`, with the permissions
/// of `target`, and return its path. The file is created exclusively under a
/// name no other file has, so a file that is already there is never touched.
pub(crate) fn stage(target: &Path, text: &str) -> io::Result<PathBuf> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "the target has no file name")
        })?;
    for _ in 0..1000 {
        let temporary = target.with_file_name(format!(
            ".{name}.{}.{}.sync.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        let written = (|| {
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
            if let Ok(existing) = fs::metadata(target) {
                fs::set_permissions(&temporary, existing.permissions())?;
            }
            Ok(())
        })();
        return match written {
            Ok(()) => Ok(temporary),
            Err(error) => {
                // Only the file created just above is removed.
                let _ = fs::remove_file(&temporary);
                Err(error)
            }
        };
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "no unused temporary file name beside the target",
    ))
}
