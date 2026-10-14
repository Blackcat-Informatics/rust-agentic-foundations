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
//! Two further validators serve the Git commands: a full reference name and a
//! commit id. Each newtype keeps its text private, so a value of the type is
//! proof that the text matched.

use std::error::Error;
use std::fmt;

/// The longest value shown in an error message, in characters.
const SHOWN: usize = 64;

/// The longest reference name accepted, in bytes.
const REF_NAME_LIMIT: usize = 1024;

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
}

impl Pattern {
    /// Every pattern.
    pub const ALL: [Self; 6] = [
        Self::Name,
        Self::TargetName,
        Self::ItemPath,
        Self::SourcePath,
        Self::RefName,
        Self::Commit,
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
        }
    }

    /// Whether the whole text matches the pattern.
    ///
    /// The relative source path is narrower than its expression: a segment
    /// that is empty, `.` or `..` does not match, so a path stays inside the
    /// tree it is read from.
    #[must_use]
    pub fn matches(self, text: &str) -> bool {
        match self {
            Self::Name => is_path(text, false),
            Self::TargetName => is_target_name(text),
            Self::ItemPath => is_path(text, true),
            Self::SourcePath => is_source_path(text),
            Self::RefName => is_ref_name(text),
            Self::Commit => is_commit(text),
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
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
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
