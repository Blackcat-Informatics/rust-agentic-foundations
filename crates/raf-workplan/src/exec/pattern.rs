// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The patterns that decide which text may reach a command line, and the
//! validated newtypes that can only be built through them.
//!
//! The four patterns of the completion gate are written by hand, byte by byte,
//! over ASCII only:
//!
//! * `<name>`: `[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*`;
//! * `<pkg>`, `<bin>`, `<test>` and `<example>`: `[A-Za-z0-9_][A-Za-z0-9_-]*`;
//! * `<item-path>`: `<name>` with an optional generic argument list
//!   `<[A-Za-z0-9_,':&]*>` after each segment;
//! * `<relative-source-path>`: `[A-Za-z0-9_][A-Za-z0-9_./-]*`.
//!
//! Five further validators serve the Git commands: a full reference name, a
//! commit id, a tree path (a file or directory of the workspace as the
//! history command names it), an identifier (the text of a `-S` pickaxe) and a
//! date (the value of `--since`, which the history compares with the author
//! date of each commit and gives to no process). Each newtype keeps its text private, so a
//! value of the type is proof that the text matched.

use std::error::Error;
use std::fmt;

/// The longest value shown in an error message, in characters.
const SHOWN: usize = 64;

/// The longest reference name accepted, in bytes.
const REF_NAME_LIMIT: usize = 1024;

/// The longest tree path accepted, in bytes.
const TREE_PATH_LIMIT: usize = 4096;

/// A pattern a value must match before it reaches a command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Pattern {
    /// `<name>`: a path of identifiers within a crate, such as `a::b::c`.
    Name,
    /// `<pkg>`, `<bin>`, `<test>` and `<example>`: a package or target name,
    /// which may hold hyphens and never begins with one.
    TargetName,
    /// `<item-path>`: a `<name>` whose segments may carry a generic argument
    /// list, such as `Wrapper<T>::get`.
    ItemPath,
    /// `<relative-source-path>`: a relative path of the list output, with no
    /// empty, `.` or `..` segment.
    SourcePath,
    /// A full Git reference name, such as `refs/remotes/origin/main`.
    RefName,
    /// A Git commit id: 40 or 64 lowercase hexadecimal digits.
    Commit,
    /// `<tree-path>`: a path relative to the workspace root as the history
    /// command gives it to `git`, only after `--` or after the colon of one
    /// `-L` argument, so it may begin with `-`. Its segments are separated by
    /// `/` and none is empty, `.` or `..`; it holds no control character and
    /// no `\`, `*`, `?` or `[`, does not begin with `/` or `:` (so it is never
    /// a pathspec pattern or magic), and is at most 4096 bytes.
    TreePath,
    /// `<identifier>`: a Rust identifier, the text of a `-S` argument.
    Identifier,
    /// `<date>`: `YYYY-MM-DD`, a day of the Gregorian calendar from
    /// 1970-01-01 to 2099-12-31, the value of `--since`. It reaches no
    /// process: the history compares the start of that day in UTC with the
    /// author date of each commit, in process.
    Date,
}

impl Pattern {
    /// Every pattern.
    pub const ALL: [Self; 9] = [
        Self::Name,
        Self::TargetName,
        Self::ItemPath,
        Self::SourcePath,
        Self::RefName,
        Self::Commit,
        Self::TreePath,
        Self::Identifier,
        Self::Date,
    ];

    /// The pattern as the capability documents write it.
    #[must_use]
    pub const fn expression(self) -> &'static str {
        match self {
            Self::Name => "[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*",
            Self::TargetName => "[A-Za-z0-9_][A-Za-z0-9_-]*",
            Self::ItemPath => {
                "[A-Za-z_][A-Za-z0-9_]*(<[A-Za-z0-9_,':&]*>)?(::[A-Za-z_][A-Za-z0-9_]*(<[A-Za-z0-9_,':&]*>)?)*"
            }
            Self::SourcePath => "[A-Za-z0-9_][A-Za-z0-9_./-]*",
            Self::RefName => "refs/<segment>(/<segment>)*",
            Self::Commit => "[0-9a-f]{40}|[0-9a-f]{64}",
            Self::TreePath => "[^/:][^/]*(/[^/]+)*",
            Self::Identifier => "[A-Za-z_][A-Za-z0-9_]*",
            Self::Date => "(19[7-9][0-9]|20[0-9][0-9])-[0-9]{2}-[0-9]{2}",
        }
    }

    /// What a value of the pattern names.
    const fn what(self) -> &'static str {
        match self {
            Self::Name => "item path within a crate",
            Self::TargetName => "package or target name",
            Self::ItemPath => "documented item path",
            Self::SourcePath => "relative source path",
            Self::RefName => "full reference name",
            Self::Commit => "commit id",
            Self::TreePath => "tree path",
            Self::Identifier => "identifier",
            Self::Date => "date of the form YYYY-MM-DD from 1970-01-01 to 2099-12-31",
        }
    }

    /// Whether the whole text matches the pattern.
    ///
    /// The relative source path and the tree path are narrower than their
    /// expressions: a segment that is empty, `.` or `..` does not match, so a
    /// path stays inside the tree it is read from, and a tree path holds no
    /// control character, no `\`, `*`, `?` or `[` and at most 4096 bytes. A
    /// date names a real day: `2026-02-29` does not match and `2028-02-29`
    /// does.
    #[must_use]
    pub fn matches(self, text: &str) -> bool {
        match self {
            Self::Name => is_path(text, false),
            Self::TargetName => is_target_name(text),
            Self::ItemPath => is_path(text, true),
            Self::SourcePath => is_source_path(text),
            Self::RefName => is_ref_name(text),
            Self::Commit => is_commit(text),
            Self::TreePath => is_tree_path(text),
            Self::Identifier => is_identifier(text),
            Self::Date => is_date(text),
        }
    }

    /// `text` unchanged when it matches.
    fn check(self, text: &str) -> Result<String, PatternError> {
        if self.matches(text) {
            Ok(text.to_string())
        } else {
            Err(PatternError {
                pattern: self,
                value: shown(text),
            })
        }
    }
}

/// A value that does not match its pattern. The value is kept for the message
/// only; it is never placed on a command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternError {
    pattern: Pattern,
    value: String,
}

impl PatternError {
    /// The pattern the value had to match.
    #[must_use]
    pub fn pattern(&self) -> Pattern {
        self.pattern
    }

    /// The refused value, cut to 64 characters.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "`{}` is not a {}: it must match `{}`; the value is not passed to a process",
            self.value.escape_debug(),
            self.pattern.what(),
            self.pattern.expression()
        )
    }
}

impl Error for PatternError {}

/// The first `SHOWN` characters of a value, marked when cut.
fn shown(text: &str) -> String {
    let mut cut: String = text.chars().take(SHOWN).collect();
    if text.chars().nth(SHOWN).is_some() {
        cut.push_str("...");
    }
    cut
}

/// `<name>` when `generics` is false, `<item-path>` when it is true.
fn is_path(text: &str, generics: bool) -> bool {
    let bytes = text.as_bytes();
    let mut at = 0;
    loop {
        if !bytes.get(at).is_some_and(|b| is_ident_start(*b)) {
            return false;
        }
        at += 1;
        while bytes.get(at).is_some_and(|b| is_ident_continue(*b)) {
            at += 1;
        }
        if generics && bytes.get(at) == Some(&b'<') {
            at += 1;
            while bytes.get(at).is_some_and(|b| is_generic(*b)) {
                at += 1;
            }
            if bytes.get(at) != Some(&b'>') {
                return false;
            }
            at += 1;
        }
        if at == bytes.len() {
            return true;
        }
        if bytes[at..].starts_with(b"::") {
            at += 2;
        } else {
            return false;
        }
    }
}

fn is_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_ident_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn is_generic(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b',' | b'\'' | b':' | b'&')
}

/// `[A-Za-z0-9_][A-Za-z0-9_-]*`.
fn is_target_name(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes
        .first()
        .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
        && bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || matches!(*b, b'_' | b'-'))
}

/// `[A-Za-z0-9_][A-Za-z0-9_./-]*`, with no empty, `.` or `..` segment.
fn is_source_path(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes
        .first()
        .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
        && bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || matches!(*b, b'_' | b'.' | b'/' | b'-'))
        && text
            .split('/')
            .all(|segment| !matches!(segment, "" | "." | ".."))
}

/// A full reference name: `refs/` and then segments of letters, digits and
/// `. _ - = + @ , #` (or any non-ASCII letter or digit), none empty, none
/// beginning with `.` or ending with `.` or `.lock`, and no `..`.
fn is_ref_name(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("refs/") else {
        return false;
    };
    if text.len() > REF_NAME_LIMIT || text.contains("..") {
        return false;
    }
    rest.split('/').all(|segment| {
        !segment.is_empty()
            && !segment.starts_with('.')
            && !segment.ends_with('.')
            && segment.strip_suffix(".lock").is_none()
            && segment.chars().all(|c| {
                if c.is_ascii() {
                    c.is_ascii_alphanumeric()
                        || matches!(c, '.' | '_' | '-' | '=' | '+' | '@' | ',' | '#')
                } else {
                    c.is_alphanumeric()
                }
            })
    })
}

/// 40 (SHA-1) or 64 (SHA-256) lowercase hexadecimal digits.
fn is_commit(text: &str) -> bool {
    matches!(text.len(), 40 | 64)
        && text
            .bytes()
            .all(|byte| purrdf_hash::hex::nibble_canonical(byte).is_some())
}

/// A tree path: see [`Pattern::TreePath`].
fn is_tree_path(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= TREE_PATH_LIMIT
        && !text.starts_with([':', '/'])
        && !text
            .chars()
            .any(|c| c.is_control() || matches!(c, '\\' | '*' | '?' | '['))
        && text
            .split('/')
            .all(|segment| !matches!(segment, "" | "." | ".."))
}

/// `[A-Za-z_][A-Za-z0-9_]*`.
fn is_identifier(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.first().is_some_and(|b| is_ident_start(*b)) && bytes.iter().all(|b| is_ident_continue(*b))
}

/// `YYYY-MM-DD`, a day of the Gregorian calendar of the years 1970 to 2099.
fn is_date(text: &str) -> bool {
    purrdf_xsd::rfc3339::parse_date(text).is_ok_and(|(year, _, _)| (1970..=2099).contains(&year))
}

macro_rules! validated {
    ($(#[$meta:meta])* $name:ident, $pattern:expr) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl $name {
            /// The value, when it matches the pattern.
            ///
            /// # Errors
            ///
            /// A [`PatternError`] when the text does not match.
            pub fn new(text: &str) -> Result<Self, PatternError> {
                $pattern.check(text).map(Self)
            }

            /// The validated text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

validated!(
    /// A package name: `--package=<pkg>`.
    Package,
    Pattern::TargetName
);
validated!(
    /// A binary, test or example target name.
    TargetName,
    Pattern::TargetName
);
validated!(
    /// An example target name: `--example=<example>`.
    ExampleName,
    Pattern::TargetName
);
validated!(
    /// The resolved path of a test within its crate, placed after `--` as the
    /// libtest filter.
    Filter,
    Pattern::Name
);
validated!(
    /// The path of a documented item within its crate, placed after `--` as
    /// the rustdoc filter. Impl items such as `Wrapper<T>::get` are allowed.
    DocFilter,
    Pattern::ItemPath
);
validated!(
    /// A repository-relative path, placed after `<rev>:` inside one argument.
    SourcePath,
    Pattern::SourcePath
);
validated!(
    /// A commit id as Git prints it.
    Commit,
    Pattern::Commit
);
validated!(
    /// The full name of a reference, as `git symbolic-ref` prints it.
    RefName,
    Pattern::RefName
);
validated!(
    /// A file or directory of the workspace, relative to its root, as the
    /// history command gives it to `git`: after `--`, or after the colon of
    /// one `-L<start>,<end>:<file>` argument. It may begin with `-`, because
    /// neither place reads an option.
    TreePath,
    Pattern::TreePath
);
validated!(
    /// A Rust identifier: the text of one `-S<identifier>` argument.
    Identifier,
    Pattern::Identifier
);
validated!(
    /// A day, `YYYY-MM-DD`: the value of `--since`, which the history compares,
    /// as the start of the day in UTC, with the author date of each commit.
    Date,
    Pattern::Date
);

impl TreePath {
    /// The top of the workspace, written `.`: the directory of a package
    /// whose manifest is at the root. The tree-path pattern refuses `.`, so
    /// this is the one way to name it.
    #[must_use]
    pub fn top() -> Self {
        Self(".".to_string())
    }
}

/// A range of lines of a file, `<start>,<end>`, 1-based and inclusive, as
/// `-L<start>,<end>:<file>` reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineRange {
    start: usize,
    end: usize,
}

impl LineRange {
    /// The lines `start` to `end`, when `1 <= start <= end`.
    #[must_use]
    pub fn new(start: usize, end: usize) -> Option<Self> {
        (1 <= start && start <= end).then_some(Self { start, end })
    }

    /// The first line.
    #[must_use]
    pub fn start(&self) -> usize {
        self.start
    }

    /// The last line.
    #[must_use]
    pub fn end(&self) -> usize {
        self.end
    }
}

impl fmt::Display for LineRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.start, self.end)
    }
}

/// The target of a test: `--lib`, `--bin=<bin>` or `--test=<test>`, so that the
/// same name in another target is never run.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Target {
    /// The library of the package.
    Lib,
    /// A binary target.
    Bin(TargetName),
    /// An integration test target.
    Test(TargetName),
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lib => f.write_str("--lib"),
            Self::Bin(name) => write!(f, "--bin={name}"),
            Self::Test(name) => write!(f, "--test={name}"),
        }
    }
}
