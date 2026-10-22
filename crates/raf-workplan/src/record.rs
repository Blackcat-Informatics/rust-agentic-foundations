// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Writing the evidence, assessment, coverage and debt records.
//!
//! A [`Record`] is written as the text the `raf-okf` writer renders for its
//! type ([`raf_okf::EvidenceRecord::to_document`] and its siblings), opened
//! with the two SPDX comment lines when [`RecordOptions`] gives them. Before
//! any byte is written the text is read back through the profile and the
//! reference scan at the path of the record, so a written record loads with
//! no finding of its own. [`create`] writes a new record and never replaces
//! a file; [`replace`] puts new text in place of a record that exists, through
//! a temporary file and a rename.

use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use raf_okf::{
    AssessmentRecord, ConstructError, CoverageRecord, DebtRecord, DocType, Document,
    EvidenceRecord, Profile, RecordHeader, is_kebab,
};
use raf_source::{EntryKind, RelPath};

use crate::files::{self, CreateError, Refused, Target};
use crate::spdx::{self, SpdxError};
use crate::{Finding, refs, sync};

/// A record to write.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Record {
    /// An evidence record, written to `work/evidence/<slug>.md`.
    Evidence(EvidenceRecord),
    /// An assessment record, written to `work/assessments/<slug>.md`.
    Assessment(AssessmentRecord),
    /// The coverage record, written to `work/coverage/default.md`.
    Coverage(CoverageRecord),
    /// The debt record, written to `work/debt/default.md`.
    Debt(DebtRecord),
}

impl Record {
    /// The type of the record.
    #[must_use]
    pub fn doc_type(&self) -> DocType {
        match self {
            Self::Evidence(_) => DocType::Evidence,
            Self::Assessment(_) => DocType::Assessment,
            Self::Coverage(_) => DocType::Coverage,
            Self::Debt(_) => DocType::Debt,
        }
    }

    /// The id, slug, tags and timestamp of the record.
    #[must_use]
    pub fn header(&self) -> &RecordHeader {
        match self {
            Self::Evidence(record) => &record.header,
            Self::Assessment(record) => &record.header,
            Self::Coverage(record) => &record.header,
            Self::Debt(record) => &record.header,
        }
    }

    /// The repository-relative path of the record: the directory of its type
    /// and its slug.
    ///
    /// # Errors
    ///
    /// [`RecordError::BadSlug`] when the slug is not a lowercase kebab-case
    /// slug, so it names no file of the directory.
    pub fn path(&self) -> Result<RelPath, RecordError> {
        let slug = &self.header().slug;
        let bad = || RecordError::BadSlug { slug: slug.clone() };
        if !is_kebab(slug) {
            return Err(bad());
        }
        RelPath::new(&format!("{}/{slug}.md", self.doc_type().directory())).map_err(|_| bad())
    }

    /// The text of the record file: the rendering of the `raf-okf` writer,
    /// with the SPDX comment lines of `options` after the opening fence, read
    /// back through the profile and the reference scan at [`Record::path`].
    ///
    /// ```rust
    /// use raf_okf::{CoverageRecord, Exclusion, ExclusionItem};
    /// use raf_workplan::record::{Record, RecordOptions};
    ///
    /// let record = Record::Coverage(CoverageRecord::new(
    ///     vec!["demo".to_string()],
    ///     vec![Exclusion::new(
    ///         ExclusionItem::Prefix("demo::internal".to_string()),
    ///         "Helpers with no contract.",
    ///     )],
    /// ));
    /// let options = RecordOptions::new()
    ///     .copyright("2026 Example Org <dev@example.org>")
    ///     .licence("CC-BY-4.0");
    /// let text = record.text(&options)?;
    /// assert!(text.starts_with("---\n# SPDX-"));
    /// assert!(text.contains("\nscope: [demo]\n"));
    /// assert_eq!(record.path()?.as_str(), "work/coverage/default.md");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`RecordError::BadSlug`]; [`RecordError::SpdxUnset`] or
    /// [`RecordError::SpdxInvalid`] when only one SPDX field is given or a
    /// field is not one line of text; [`RecordError::Build`] when a value
    /// cannot be written in the frontmatter subset; and
    /// [`RecordError::Invalid`] when the text does not read back as a record
    /// with no finding, such as a value the profile refuses or an issue
    /// reference in a command.
    pub fn text(&self, options: &RecordOptions) -> Result<String, RecordError> {
        let path = self.path()?;
        let spdx = spdx::pair(options.copyright.as_deref(), options.licence.as_deref())?;
        let document = match self {
            Self::Evidence(record) => record.to_document(),
            Self::Assessment(record) => record.to_document(),
            Self::Coverage(record) => record.to_document(),
            Self::Debt(record) => record.to_document(),
        }?;
        let text = spdx::with_spdx(document.render(), spdx);
        let mut findings = Vec::new();
        refs::scan(path.as_str(), &text, &mut findings);
        match Document::parse(&text) {
            Ok(read) => {
                if let Err(violations) = Profile.validate(&read, path.as_str()) {
                    findings.extend(violations.iter().map(Finding::from_violation));
                }
            }
            Err(error) => findings.push(Finding::from_parse_error(path.as_str(), &error)),
        }
        if findings.is_empty() {
            Ok(text)
        } else {
            Err(RecordError::Invalid { path, findings })
        }
    }
}

/// The SPDX comment lines a written record opens with.
///
/// A repository that checks the licence of every file (REUSE) needs each
/// record to open with two SPDX comment lines; `copyright` and `licence` give
/// their text, and are written together or not at all.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct RecordOptions {
    /// The text of the `SPDX-FileCopyrightText` comment line. Required when
    /// `licence` is set.
    pub copyright: Option<String>,
    /// The identifier of the `SPDX-License-Identifier` comment line. Required
    /// when `copyright` is set.
    pub licence: Option<String>,
}

impl RecordOptions {
    /// Options with no SPDX lines.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the record with the SPDX copyright comment line `text`.
    #[must_use]
    pub fn copyright(mut self, text: impl Into<String>) -> Self {
        self.copyright = Some(text.into());
        self
    }

    /// Open the record with the SPDX licence comment line `id`.
    #[must_use]
    pub fn licence(mut self, id: impl Into<String>) -> Self {
        self.licence = Some(id.into());
        self
    }
}

/// Why a record was not written. Nothing is written in any case.
#[derive(Debug)]
#[non_exhaustive]
pub enum RecordError {
    /// The slug is not a lowercase kebab-case slug.
    BadSlug {
        /// The slug as given.
        slug: String,
    },
    /// One of `copyright` and `licence` is given and the other is not.
    SpdxUnset {
        /// The field that is missing: `copyright` or `licence`.
        field: &'static str,
    },
    /// An SPDX field cannot be written as one comment line.
    SpdxInvalid {
        /// The field: `copyright` or `licence`.
        field: &'static str,
        /// Why.
        reason: &'static str,
    },
    /// A value cannot be written in the frontmatter subset.
    Build(ConstructError),
    /// The text does not read back as a record with no finding.
    Invalid {
        /// The path of the record.
        path: RelPath,
        /// The findings the text would have.
        findings: Vec<Finding>,
    },
    /// [`create`]: a file, or a link, is already at the path. It is untouched.
    Exists {
        /// The path of the record.
        path: RelPath,
    },
    /// [`replace`]: no regular file is at the path to replace.
    Missing {
        /// The path of the record.
        path: RelPath,
    },
    /// The file could not be written, or a directory it needs is refused.
    Write {
        /// The path that was being written or judged.
        path: String,
        /// Why.
        source: io::Error,
    },
}

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadSlug { slug } => write!(
                f,
                "`{slug}` is not a record slug; use lowercase letters and digits joined by single `-`"
            ),
            Self::SpdxUnset { field } => write!(
                f,
                "the record needs both SPDX lines or neither, and the {field} is not given; pass --copyright <text> and --licence <id> together"
            ),
            Self::SpdxInvalid { field, reason } => write!(
                f,
                "the {field} of the SPDX header of the record is refused: {reason}"
            ),
            Self::Build(error) => write!(f, "the record cannot be written: {error}"),
            Self::Invalid { path, findings } => {
                write!(
                    f,
                    "the record {path} would not load clean, so nothing is written"
                )?;
                crate::lookup::write_findings(f, findings)
            }
            Self::Exists { path } => write!(
                f,
                "{path} already exists and is left as it is; a new record never replaces a file"
            ),
            Self::Missing { path } => write!(
                f,
                "{path} is not a record file that exists, so there is nothing to replace"
            ),
            Self::Write { path, source } => write!(f, "cannot write {path}: {source}"),
        }
    }
}

impl Error for RecordError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Build(error) => Some(error),
            Self::Write { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<ConstructError> for RecordError {
    fn from(error: ConstructError) -> Self {
        Self::Build(error)
    }
}

impl From<Refused> for RecordError {
    fn from(refused: Refused) -> Self {
        Self::Write {
            path: refused.path,
            source: refused.source,
        }
    }
}

impl From<SpdxError> for RecordError {
    fn from(error: SpdxError) -> Self {
        match error {
            SpdxError::Unset { field } => Self::SpdxUnset { field },
            SpdxError::Invalid { field, reason } => Self::SpdxInvalid { field, reason },
        }
    }
}

/// Write the new record `record` below `root`, the directory that holds
/// `work/`, and return its path.
///
/// The text is [`Record::text`]. The file is created exclusively: a file or a
/// link already at the path is [`RecordError::Exists`] and is not touched.
/// The directory of the record type is created when it is missing; it and
/// `work` must each be a directory or not exist, and a symbolic link, inside
/// the root or out of it, or a file there is refused during preflight. This
/// path-based inspection retains no directory handles against a later name
/// swap. A failure after the file was
/// created removes it again.
///
/// # Errors
///
/// Every error of [`Record::text`], before anything is written;
/// [`RecordError::Exists`]; and [`RecordError::Write`] when the file or its
/// directory cannot be created, in which case no file is left.
pub fn create(
    root: &Path,
    record: &Record,
    options: &RecordOptions,
) -> Result<RelPath, RecordError> {
    let text = record.text(options)?;
    let path = record.path()?;
    files::create_new(root, record.doc_type().directory(), &path, &text).map_err(|error| {
        match error {
            CreateError::Exists => RecordError::Exists { path: path.clone() },
            CreateError::Write { path, source } => RecordError::Write { path, source },
        }
    })?;
    Ok(path)
}

/// Put the text of `record` in place of the record file that exists at its
/// path below `root`, and return the path.
///
/// `work` and the directory of the record type must each be a directory, and
/// a symbolic link there, inside the root or out of it, or a file is refused,
/// as [`create`] refuses it during preflight. Publication uses those paths
/// without retaining directory handles against a later name swap. The target
/// must then be a regular file; a missing file, a
/// directory and a symbolic link are refused.
/// The new text is written to a temporary file beside the target, created
/// exclusively, and renamed over it, so the target holds either its old text
/// or the new one, never part of either. The file keeps its permissions.
///
/// # Errors
///
/// Every error of [`Record::text`], before anything is written;
/// [`RecordError::Missing`] when no regular file is at the path;
/// [`RecordError::Write`] naming the directory refused on the way, before
/// anything is written; and [`RecordError::Write`] when the temporary file
/// cannot be written or renamed, in which case the temporary file is removed
/// and the target is unchanged.
pub fn replace(
    root: &Path,
    record: &Record,
    options: &RecordOptions,
) -> Result<RelPath, RecordError> {
    let text = record.text(options)?;
    let path = record.path()?;
    let write = |source: io::Error| RecordError::Write {
        path: path.to_string(),
        source,
    };
    match files::target(root, record.doc_type().directory(), &path) {
        Ok(Target {
            kind: Some(EntryKind::File),
            ..
        }) => {}
        Ok(_) => return Err(RecordError::Missing { path }),
        Err(error) => return Err(error.into()),
    }
    publish_replacement(root, &path, &text).map_err(write)?;
    Ok(path)
}

/// Publish the replacement after the regular-file and directory inspection.
/// The path inspection retains no handles or later-name comparison.
fn publish_replacement(root: &Path, path: &RelPath, text: &str) -> io::Result<()> {
    let target = fs::canonicalize(root)?.join(path.as_str());
    let temporary = sync::stage(&target, text)?;
    if let Err(error) = fs::rename(&temporary, &target) {
        // Only the temporary file this call created is removed.
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::{
        Record, RecordError, RecordOptions, Target, files, fs, publish_replacement, replace,
    };
    use raf_okf::CoverageRecord;
    use raf_source::EntryKind;

    #[test]
    fn a_directory_swap_after_preflight_is_not_rechecked_by_replacement_publication() {
        let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir())
            .expect("create replacement fixture");
        let root = scratch.path();
        let record = Record::Coverage(CoverageRecord::new(Vec::new(), Vec::new()));
        let options = RecordOptions::new();
        let path = record.path().unwrap();
        let text = record.text(&options).unwrap();
        fs::create_dir_all(root.join("work/coverage")).unwrap();
        fs::create_dir(root.join("alternate")).unwrap();
        fs::write(root.join(path.as_str()), "original\n").unwrap();
        fs::write(root.join("alternate/default.md"), "alternate\n").unwrap();
        assert!(matches!(
            files::target(root, "work/coverage", &path).unwrap(),
            Target {
                kind: Some(EntryKind::File),
                directories: true
            }
        ));

        fs::rename(root.join("work/coverage"), root.join("original")).unwrap();
        std::os::unix::fs::symlink("../alternate", root.join("work/coverage")).unwrap();
        publish_replacement(root, &path, &text)
            .expect("replacement follows the names after inspection");
        assert_eq!(
            fs::read_to_string(root.join("alternate/default.md")).unwrap(),
            text
        );
        assert_eq!(
            fs::read_to_string(root.join("original/default.md")).unwrap(),
            "original\n"
        );
        assert!(matches!(
            replace(root, &record, &options),
            Err(RecordError::Write { path, .. }) if path == "work/coverage"
        ));

        fs::remove_file(root.join("work/coverage")).unwrap();
        fs::rename(root.join("original"), root.join("work/coverage")).unwrap();
        assert_eq!(replace(root, &record, &options).unwrap(), path);
        assert_eq!(fs::read_to_string(root.join(path.as_str())).unwrap(), text);
    }
}
