// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Links: inline links and images, reference definitions, and the `href` and
//! `src` attributes of inline HTML.
//!
//! The inline scanner follows `CommonMark`: a link text is followed
//! immediately by `(`, the parts inside the parentheses are separated by
//! spaces, tabs and line endings only, and a bare destination ends at a
//! space or an ASCII control character, not at any Unicode white space. A
//! link whose text or parentheses span a line break is not read as a link; it
//! is recorded as a split link instead.

use std::error::Error;
use std::fmt;

/// The longest link destination or title the scanner reads, in bytes,
/// not counting the delimiters of a title or of an angle-bracket destination.
pub(crate) const MAX_LINK_PART: usize = 4096;

/// The syntax a link was written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LinkForm {
    /// `[text](target)`.
    Inline,
    /// `![alt](target)`.
    Image,
    /// `[label]: target`.
    ReferenceDefinition,
    /// An `href="target"` attribute of inline HTML.
    Href,
    /// A `src="target"` attribute of inline HTML.
    Src,
}

/// A link outside fenced code, code spans and HTML comments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    target: String,
    form: LinkForm,
    line: usize,
}

/// A link or image whose text or destination spans a line break. It is not
/// a link; it is recorded so the profile can report it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitLink {
    form: LinkForm,
    line: usize,
}

impl SplitLink {
    /// [`LinkForm::Inline`] or [`LinkForm::Image`].
    #[must_use]
    pub fn form(&self) -> LinkForm {
        self.form
    }

    /// The 1-based line, in the whole text, where the link opens.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }
}

/// What a link target names.
///
/// The three forms are every place a target can lead, so the enum is
/// exhaustive: a consumer matches every variant, and a new one is a breaking
/// change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkTarget {
    /// A target with a scheme (`https:`, `mailto:`) or a network path (`//`).
    External(String),
    /// A heading anchor in the same document; the text after `#`.
    Fragment(String),
    /// A path inside the repository, relative to the root, with an anchor.
    Relative {
        /// The normalised path, `/`-separated, with no `.` or `..`.
        path: String,
        /// The text after `#`, when the target has one.
        anchor: Option<String>,
    },
}

/// Why a link target cannot be resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LinkError {
    /// The target is empty.
    Empty,
    /// The target is an absolute path.
    Absolute {
        /// The target.
        target: String,
    },
    /// The target climbs above the repository root.
    EscapesRoot {
        /// The target.
        target: String,
    },
}

impl fmt::Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("the link has an empty target"),
            Self::Absolute { target } => write!(f, "link `{target}` is an absolute path"),
            Self::EscapesRoot { target } => {
                write!(f, "link `{target}` leaves the repository")
            }
        }
    }
}

impl Error for LinkError {}

impl Link {
    /// The target as written, with backslash escapes removed.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    /// The syntax the link was written in.
    #[must_use]
    pub fn form(&self) -> LinkForm {
        self.form
    }

    /// The 1-based line, in the whole text, of the link.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }

    /// The text after the first `#` of the target, when there is one.
    #[must_use]
    pub fn anchor(&self) -> Option<&str> {
        self.target.split_once('#').map(|(_, anchor)| anchor)
    }

    /// What the target names, resolved against the repository-relative path
    /// of the document that holds the link.
    ///
    /// # Errors
    ///
    /// [`LinkError::Empty`] for an empty target, [`LinkError::Absolute`] for
    /// a path that starts with `/`, and [`LinkError::EscapesRoot`] when `..`
    /// segments climb above the root of `doc_path`. A target that leaves the
    /// root is never clamped to it.
    pub fn target_in(&self, doc_path: &str) -> Result<LinkTarget, LinkError> {
        resolve(&self.target, doc_path)
    }
}

fn has_scheme(target: &str) -> bool {
    let mut chars = target.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    for c in chars {
        if c == ':' {
            return true;
        }
        if !(c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-')) {
            return false;
        }
    }
    false
}

fn resolve(target: &str, doc_path: &str) -> Result<LinkTarget, LinkError> {
    if target.is_empty() {
        return Err(LinkError::Empty);
    }
    if has_scheme(target) || target.starts_with("//") {
        return Ok(LinkTarget::External(target.to_string()));
    }
    if let Some(anchor) = target.strip_prefix('#') {
        return Ok(LinkTarget::Fragment(anchor.to_string()));
    }
    if target.starts_with('/') {
        return Err(LinkError::Absolute {
            target: target.to_string(),
        });
    }
    let (file, anchor) = match target.split_once('#') {
        Some((file, anchor)) => (file, Some(anchor.to_string())),
        None => (target, None),
    };
    let escapes = || LinkError::EscapesRoot {
        target: target.to_string(),
    };
    let mut parts: Vec<&str> = doc_path.split('/').collect();
    parts.pop();
    parts.retain(|part| !part.is_empty() && *part != ".");
    for part in file.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    return Err(escapes());
                }
            }
            name => parts.push(name),
        }
    }
    Ok(LinkTarget::Relative {
        path: parts.join("/"),
        anchor,
    })
}

/// Project native authoring occurrences into repository link policy.
pub(super) fn project(
    native: &purrdf_markdown::AuthoringDocument<'_>,
    lines: &[super::blocks::Line],
    origin: super::Origin,
) -> (Vec<Link>, Vec<SplitLink>) {
    use purrdf_markdown::syntax::AuthoringLinkForm;
    let form = |value| match value {
        AuthoringLinkForm::Inline => LinkForm::Inline,
        AuthoringLinkForm::Image => LinkForm::Image,
        AuthoringLinkForm::ReferenceDefinition => LinkForm::ReferenceDefinition,
        AuthoringLinkForm::Href => LinkForm::Href,
        AuthoringLinkForm::Src => LinkForm::Src,
    };
    let mut ordered: Vec<(u8, u64, Link)> = native
        .links()
        .iter()
        .map(|link| {
            let at = if matches!(link.form, AuthoringLinkForm::Href | AuthoringLinkForm::Src) {
                link.destination.source.start
            } else {
                link.source.start
            };
            let group = match link.form {
                AuthoringLinkForm::Href | AuthoringLinkForm::Src => 0,
                AuthoringLinkForm::ReferenceDefinition => 1,
                AuthoringLinkForm::Inline | AuthoringLinkForm::Image => 2,
            };
            (
                group,
                link.source.end,
                Link {
                    target: link.destination.decoded.to_string(),
                    form: form(link.form),
                    line: origin.line + super::blocks::line_of(lines, super::blocks::offset(at)),
                },
            )
        })
        .collect();
    // Within one line, HTML attributes precede definitions and inline
    // closures; an inner image closes before its enclosing link.
    ordered.sort_by_key(|(group, end, link)| (link.line, *group, *end));
    let occurrences = ordered.into_iter().map(|(_, _, link)| link).collect();
    let split = native
        .split_links()
        .iter()
        .map(|link| SplitLink {
            form: form(link.form),
            line: origin.line
                + super::blocks::line_of(lines, super::blocks::offset(link.source.start)),
        })
        .collect();
    (occurrences, split)
}
