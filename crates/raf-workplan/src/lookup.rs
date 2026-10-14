// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! [`Workplan::change`]: the change record a slug names, and the two ways a
//! slug can name none.

use std::error::Error;
use std::fmt;

use raf_okf::Change;

use crate::{Finding, Loaded, Workplan};

/// The prefix a change id carries.
pub(crate) const CHANGE_PREFIX: &str = "change:";

/// A change slug without the `change:` prefix a caller may have written.
pub(crate) fn bare_slug(change: &str) -> &str {
    change.strip_prefix(CHANGE_PREFIX).unwrap_or(change)
}

/// Why a slug names no loaded change record.
#[derive(Debug)]
#[non_exhaustive]
pub enum ChangeLookupError {
    /// No change record has the slug.
    Unknown {
        /// The slug as asked for, without a `change:` prefix.
        slug: String,
        /// The slugs of the change records that loaded, sorted.
        known: Vec<String>,
    },
    /// A change record exists and did not load, so what it lists is unknown.
    NotLoaded {
        /// The slug.
        slug: String,
        /// The findings about the record.
        findings: Vec<Finding>,
    },
}

/// `no change record `<slug>`; ...`, with the open changes named.
pub(crate) fn write_unknown(
    f: &mut fmt::Formatter<'_>,
    slug: &str,
    known: &[String],
) -> fmt::Result {
    write!(f, "no change record `{slug}`; ")?;
    if known.is_empty() {
        write!(f, "there is no open change")
    } else {
        write!(f, "the open changes are {}", known.join(", "))
    }
}

/// The record did not load, then its findings one to a line.
pub(crate) fn write_not_loaded(
    f: &mut fmt::Formatter<'_>,
    slug: &str,
    findings: &[Finding],
) -> fmt::Result {
    write!(
        f,
        "the record of change `{slug}` did not load, so what it lists is unknown; fix its findings first"
    )?;
    write_findings(f, findings)
}

/// The findings, one per line.
pub(crate) fn write_findings(f: &mut fmt::Formatter<'_>, findings: &[Finding]) -> fmt::Result {
    for finding in findings {
        write!(f, "\n  {finding}")?;
    }
    Ok(())
}

impl fmt::Display for ChangeLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown { slug, known } => write_unknown(f, slug, known),
            Self::NotLoaded { slug, findings } => write_not_loaded(f, slug, findings),
        }
    }
}

impl Error for ChangeLookupError {}

impl Workplan<'_> {
    /// The change record whose slug is `change`, with or without the `change:`
    /// prefix.
    ///
    /// ```rust
    /// use raf_source::{MemoryTree, Workspace};
    /// use raf_workplan::{ChangeLookupError, Workplan};
    ///
    /// let source = Workspace::load(
    ///     &MemoryTree::from([
    ///         ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ///         ("src/lib.rs", ""),
    ///     ]),
    ///     &["."],
    /// )?;
    /// let plan = Workplan::load(&MemoryTree::from([("docs/architecture/.keep", "")]), &source);
    /// // No record has the slug, and none is open.
    /// assert!(matches!(
    ///     plan.change("grow"),
    ///     Err(ChangeLookupError::Unknown { ref known, .. }) if known.is_empty()
    /// ));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`ChangeLookupError::Unknown`] when no change record has the slug, with
    /// the slugs that loaded; [`ChangeLookupError::NotLoaded`] when a record
    /// has it and did not load, with the findings about the record.
    pub fn change(&self, change: &str) -> Result<&Loaded<Change>, ChangeLookupError> {
        let slug = bare_slug(change);
        let id = format!("{CHANGE_PREFIX}{slug}");
        if let Some(record) = self
            .changes
            .iter()
            .find(|record| record.value().header.id == id)
        {
            return Ok(record);
        }
        if self.unloaded().contains(&id) {
            return Err(ChangeLookupError::NotLoaded {
                slug: slug.to_string(),
                findings: self.findings_of(&format!("work/changes/{slug}.md")),
            });
        }
        let mut known: Vec<String> = self
            .changes
            .iter()
            .filter_map(|record| record.value().header.id.strip_prefix(CHANGE_PREFIX))
            .map(str::to_string)
            .collect();
        known.sort();
        Err(ChangeLookupError::Unknown {
            slug: slug.to_string(),
            known,
        })
    }

    /// The findings of the load about the file `path`.
    pub(crate) fn findings_of(&self, path: &str) -> Vec<Finding> {
        self.findings
            .iter()
            .filter(|finding| finding.file == path)
            .cloned()
            .collect()
    }
}
