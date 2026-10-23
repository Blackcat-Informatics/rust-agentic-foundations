// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The validated root-relative path every [`Tree`](crate::Tree) call takes.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use purrdf_lex::walk::WorkList;

/// A source authority within one load; this handle is not durable source proof.
///
/// The first-party tree is authority zero. A catalog assigns other authorities
/// and records their exact source identities separately.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceAuthorityId(pub(crate) usize);

impl SourceAuthorityId {
    /// The ordinary first-party tree used by [`RelPath::new`].
    pub const FIRST_PARTY: Self = Self(0);

    /// The load-local authority number.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// A path below a tree root: `/`-separated, UTF-8, never absolute and never
/// leaving the root.
///
/// The authority is part of identity, equality and ordering. Its relative text
/// stays unchanged: [`RelPath::as_str`] alone does not identify a source.
/// Ordinary constructors select the first-party source; [`RelPath::in_source`]
/// explicitly selects another independently rooted authority.
///
/// The empty path is the root itself. Empty segments and `.` segments are
/// dropped. A `..` segment is kept, because on a disk it means the parent of
/// the directory the earlier segments really reach, which differs from the
/// lexical parent when one of them is a symlink; it is refused only when the
/// segments before it do not leave a level to climb out of.
///
/// The text is shared, so a clone does not copy it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RelPath {
    text: Arc<str>,
    /// The independently rooted source that interprets this relative path.
    authority: SourceAuthorityId,
}

/// Why an input is not a [`RelPath`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PathError {
    /// The input starts with `/`.
    Absolute {
        /// The refused input.
        input: String,
    },
    /// A `..` segment climbs above the root.
    AboveRoot {
        /// The refused input.
        input: String,
    },
    /// The input holds a backslash, which separates segments on some platforms.
    Backslash {
        /// The refused input.
        input: String,
    },
    /// The input holds a NUL character.
    Nul {
        /// The refused input.
        input: String,
    },
    /// A segment starts with a drive letter and a colon.
    DriveLetter {
        /// The refused input.
        input: String,
    },
}

impl PathError {
    /// The input that was refused.
    #[must_use]
    pub fn input(&self) -> &str {
        match self {
            Self::Absolute { input }
            | Self::AboveRoot { input }
            | Self::Backslash { input }
            | Self::Nul { input }
            | Self::DriveLetter { input } => input,
        }
    }
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let input = crate::error::Esc(self.input());
        match self {
            Self::Absolute { .. } => write!(
                f,
                "`{input}` is absolute; a path must be relative to the root"
            ),
            Self::AboveRoot { .. } => write!(f, "`{input}` climbs above the root with `..`"),
            Self::Backslash { .. } => write!(
                f,
                "`{input}` holds a backslash; segments are separated by `/`"
            ),
            Self::Nul { .. } => write!(f, "`{input}` holds a NUL character"),
            Self::DriveLetter { .. } => write!(
                f,
                "`{input}` holds a segment that starts with a drive letter"
            ),
        }
    }
}

impl Error for PathError {}

fn starts_with_drive_letter(segment: &str) -> bool {
    let mut chars = segment.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic()) && chars.next() == Some(':')
}

impl RelPath {
    /// The root itself.
    #[must_use]
    pub fn root() -> Self {
        Self {
            text: Arc::from(""),
            authority: SourceAuthorityId::FIRST_PARTY,
        }
    }

    /// Parse `input` as a path below the root.
    ///
    /// # Errors
    ///
    /// Returns a [`PathError`] when the input is absolute, climbs above the
    /// root, or holds a backslash, a NUL or a drive-letter segment.
    pub fn new(input: &str) -> Result<Self, PathError> {
        Self::root().join(input)
    }

    /// Append `input`, which is read as a relative path.
    ///
    /// `.` and empty segments are dropped; `..` segments are kept so that the
    /// disk, not the text, decides where they lead.
    ///
    /// # Errors
    ///
    /// Returns a [`PathError`] when the input is absolute, climbs above this
    /// path's root, or holds a backslash, a NUL or a drive-letter segment.
    pub fn join(&self, input: &str) -> Result<Self, PathError> {
        let owned = || input.to_owned();
        if input.starts_with('/') {
            return Err(PathError::Absolute { input: owned() });
        }
        if input.contains('\\') {
            return Err(PathError::Backslash { input: owned() });
        }
        if input.contains('\0') {
            return Err(PathError::Nul { input: owned() });
        }
        let mut segments = WorkList::<&str, 16>::new();
        let mut depth = 0_usize;
        for segment in self.segments() {
            // Every prefix of this already validated path stays inside its root.
            if segment == ".." {
                depth -= 1;
            } else {
                depth += 1;
            }
            segments.push(segment);
        }
        for segment in input.split('/') {
            match segment {
                "" | "." => {}
                ".." => {
                    depth = depth
                        .checked_sub(1)
                        .ok_or_else(|| PathError::AboveRoot { input: owned() })?;
                    segments.push(segment);
                }
                _ => {
                    if starts_with_drive_letter(segment) {
                        return Err(PathError::DriveLetter { input: owned() });
                    }
                    depth += 1;
                    segments.push(segment);
                }
            }
        }
        let mut text = String::new();
        for segment in segments.iter() {
            if !text.is_empty() {
                text.push('/');
            }
            text.push_str(segment);
        }
        Ok(Self {
            text: Arc::from(text),
            authority: self.authority,
        })
    }

    /// The independently rooted source of this path.
    #[must_use]
    pub const fn authority(&self) -> SourceAuthorityId {
        self.authority
    }

    /// Explicitly select an authority for this same relative spelling.
    ///
    /// A load-local handle selects a source; it does not authenticate its bytes.
    /// Module traversal never changes authority this way.
    #[must_use]
    pub fn in_source(&self, authority: SourceAuthorityId) -> Self {
        Self {
            text: self.text.clone(),
            authority,
        }
    }

    /// The root of this path's own source authority.
    #[must_use]
    pub fn source_root(&self) -> Self {
        Self::root().in_source(self.authority)
    }

    /// True for the root itself.
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.text.is_empty()
    }

    /// The path as `/`-separated text; the root is the empty string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The segments in order, `..` included.
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.text.split('/').filter(|segment| !segment.is_empty())
    }

    /// The last segment, or `None` for the root.
    #[must_use]
    pub fn file_name(&self) -> Option<&str> {
        self.segments().last()
    }

    /// This path without its last segment, or `None` for the root.
    ///
    /// The result is lexical: when the last segment is `..` it is not the
    /// directory the path leads out of.
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        if self.is_root() {
            return None;
        }
        let text = self.text.rfind('/').map_or("", |index| &self.text[..index]);
        Some(Self {
            text: Arc::from(text),
            authority: self.authority,
        })
    }
}

impl fmt::Display for RelPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_root() {
            f.write_str(".")
        } else {
            f.write_str(&self.text)
        }
    }
}

#[cfg(test)]
mod display_tests {
    use super::RelPath;

    #[test]
    fn control_characters_in_a_refused_path_are_escaped() {
        let refused = RelPath::new("/a\u{1b}[31m\u{202e}b").expect_err("absolute");
        let text = refused.to_string();
        assert!(!text.contains('\u{1b}'), "{text}");
        assert!(!text.contains('\u{202e}'), "{text}");
        assert!(text.contains("\\u{1b}"), "{text}");
    }
}
