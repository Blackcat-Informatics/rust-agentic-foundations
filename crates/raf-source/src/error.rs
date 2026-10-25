// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The typed errors of the tree backends.

use std::error::Error;
use std::fmt;
use std::io;

use crate::budget::Budget;
use crate::path::{PathError, RelPath, SourceAuthorityId};

/// Text that came from the input, shown with every control character escaped
/// by [`char::escape_debug`], so that an error message cannot carry a terminal
/// escape sequence, a line break or a NUL.
pub(crate) struct Esc<'a>(pub(crate) &'a str);

impl fmt::Display for Esc<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.chars().try_for_each(|c| match c {
            // Quotes are not control characters and stay readable.
            '\'' | '"' => write!(f, "{c}"),
            _ => write!(f, "{}", c.escape_debug()),
        })
    }
}

/// Why a [`Tree`](crate::Tree) call failed.
///
/// Every variant names the path it concerns. Only [`TreeError::NotFound`]
/// means that nothing is there; every other variant is a refusal or a failure
/// and must never be read as absence.
#[derive(Debug)]
#[non_exhaustive]
pub enum TreeError {
    /// A committed Git host Tree refused source, storage or work evidence.
    GitSource {
        /// The requested relative path.
        path: String,
        /// The original verification or resource failure.
        source: Box<LoadError>,
    },
    /// A source mount received an already-scoped identity from its backend.
    BackendAuthority {
        /// The relative path that was asked for.
        path: String,
        /// The backend identity's unexpected authority.
        authority: SourceAuthorityId,
    },
    /// This tree does not provide the path's selected source authority.
    WrongAuthority {
        /// The relative path that was asked for.
        path: String,
        /// The unavailable authority.
        authority: SourceAuthorityId,
    },
    /// Nothing exists at the path.
    NotFound {
        /// The path that was asked for.
        path: String,
    },
    /// The path leads outside the tree root: by an absolute or `..` input, a
    /// symlink, or a `..` after a directory symlink.
    OutsideRoot {
        /// The path that was asked for.
        path: String,
    },
    /// The path is not a regular file where one is needed, or not a real
    /// directory where one is needed (a FIFO, a device, a directory symlink).
    NotRegular {
        /// The path that was asked for.
        path: String,
    },
    /// Resolving the path ran into a symlink loop.
    SymlinkLoop {
        /// The path that was asked for.
        path: String,
    },
    /// The file holds more bytes than the caller allowed.
    TooLarge {
        /// The path that was asked for.
        path: String,
        /// The limit in bytes that was exceeded.
        limit: u64,
    },
    /// The file is not valid UTF-8.
    NotUtf8 {
        /// The path that was asked for.
        path: String,
    },
    /// A directory entry has a name that is not valid UTF-8.
    NonUtf8Name {
        /// The directory that was listed.
        path: String,
        /// The entry name, with invalid bytes replaced for display.
        name: String,
    },
    /// Two keys of a [`MemoryTree`](crate::MemoryTree) name one path once
    /// normalised.
    DuplicatePath {
        /// The key that repeats an earlier path.
        path: String,
    },
    /// The operating system failed the operation for another reason.
    Io {
        /// The path that was asked for.
        path: String,
        /// The underlying failure.
        source: io::Error,
    },
}

impl TreeError {
    /// The path the error concerns.
    #[must_use]
    pub fn path(&self) -> &str {
        match self {
            Self::GitSource { path, .. }
            | Self::BackendAuthority { path, .. }
            | Self::WrongAuthority { path, .. }
            | Self::NotFound { path }
            | Self::OutsideRoot { path }
            | Self::NotRegular { path }
            | Self::SymlinkLoop { path }
            | Self::TooLarge { path, .. }
            | Self::NotUtf8 { path }
            | Self::NonUtf8Name { path, .. }
            | Self::DuplicatePath { path }
            | Self::Io { path, .. } => path,
        }
    }

    /// Classify an operating-system failure on `path`.
    ///
    /// A missing entry or a missing directory on the way is `NotFound`; a
    /// symlink loop is `SymlinkLoop`; anything else is `Io`.
    pub(crate) fn from_io(path: &str, source: io::Error) -> Self {
        let path = path.to_owned();
        match source.kind() {
            io::ErrorKind::NotFound | io::ErrorKind::NotADirectory => Self::NotFound { path },
            _ if is_symlink_loop(&source) => Self::SymlinkLoop { path },
            _ => Self::Io { path, source },
        }
    }
}

/// True when the operating system reported too many levels of symbolic links.
pub(crate) fn is_symlink_loop(error: &io::Error) -> bool {
    #[cfg(unix)]
    {
        error.raw_os_error() == Some(libc::ELOOP)
    }
    #[cfg(windows)]
    {
        // ERROR_CANT_RESOLVE_FILENAME
        error.raw_os_error() == Some(1921)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = error;
        false
    }
}

impl From<PathError> for TreeError {
    /// A path that is not a [`RelPath`](crate::RelPath) points somewhere the
    /// root does not contain, so it is `OutsideRoot`.
    fn from(error: PathError) -> Self {
        Self::OutsideRoot {
            path: error.input().to_owned(),
        }
    }
}

impl fmt::Display for TreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GitSource { path, source } => write!(f, "{}: {source}", Esc(path)),
            Self::BackendAuthority { path, authority } => write!(
                f,
                "{}: backend returned source authority {}; mounted backend identities must use authority zero",
                Esc(path),
                authority.index()
            ),
            Self::WrongAuthority { path, authority } => write!(
                f,
                "{} belongs to source authority {}, which this tree does not contain",
                Esc(path),
                authority.index()
            ),
            Self::NotFound { path } => write!(f, "{}: no such file or directory", Esc(path)),
            Self::OutsideRoot { path } => write!(f, "{} leads outside the root", Esc(path)),
            Self::NotRegular { path } => {
                write!(f, "{} is not a regular file or a real directory", Esc(path))
            }
            Self::SymlinkLoop { path } => write!(f, "{} runs into a symlink loop", Esc(path)),
            Self::TooLarge { path, limit } => {
                write!(f, "{} is larger than the limit of {limit} bytes", Esc(path))
            }
            Self::NotUtf8 { path } => write!(f, "{} is not valid UTF-8", Esc(path)),
            Self::NonUtf8Name { path, name } => write!(
                f,
                "directory {} holds an entry whose name is not valid UTF-8: \"{}\"",
                Esc(path),
                Esc(name)
            ),
            Self::DuplicatePath { path } => {
                write!(
                    f,
                    "{} names a path that an earlier key already holds",
                    Esc(path)
                )
            }
            Self::Io { path, source } => write!(f, "{}: {source}", Esc(path)),
        }
    }
}

impl Error for TreeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::GitSource { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Where a failing file was reached from.
///
/// A caller that parses a file directly has no origin. A walker that follows
/// `mod` declarations, `#[path]` attributes and manifests sets one so that the
/// message names the module, attribute or key that led to the file.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Origin {
    /// A `mod name;` declaration.
    ModDecl {
        /// The module name as written.
        module: String,
    },
    /// A `#[path = "..."]` attribute on a module.
    PathAttr {
        /// The module name as written.
        module: String,
        /// The `cfg_attr` predicate that guards the attribute, if any.
        predicate: Option<String>,
    },
    /// A key of a `Cargo.toml`.
    ManifestKey {
        /// The key, such as `lib.path`.
        key: String,
    },
    /// Discovery of the packages themselves.
    Discovery,
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModDecl { module } => write!(f, "`mod {};`", Esc(module)),
            Self::PathAttr {
                module,
                predicate: None,
            } => write!(f, "the `path` attribute of module `{}`", Esc(module)),
            Self::PathAttr {
                module,
                predicate: Some(predicate),
            } => write!(
                f,
                "the `path` attribute of module `{}` under `{}`",
                Esc(module),
                Esc(predicate)
            ),
            Self::ManifestKey { key } => write!(f, "manifest key `{}`", Esc(key)),
            Self::Discovery => f.write_str("package discovery"),
        }
    }
}

/// What went wrong while loading a file.
#[derive(Debug)]
#[non_exhaustive]
pub enum LoadErrorKind {
    /// A required Git process or its bounded pipes failed.
    GitIo {
        /// The host operation that failed.
        operation: &'static str,
        /// The original operating-system failure.
        source: io::Error,
    },
    /// Git refused a committed-object request.
    GitExit {
        /// The actual child exit status.
        status: std::process::ExitStatus,
        /// Bounded command output, escaped when displayed.
        diagnostic: String,
    },
    /// Cleanup of an owned Git child failed after a primary refusal.
    GitCleanup {
        /// The original refusal, retained rather than swallowed.
        primary: Box<LoadError>,
        /// The cleanup operation that failed.
        operation: &'static str,
        /// The original cleanup failure.
        source: io::Error,
    },
    /// Native storage refused a physical layout or allocation.
    Storage(purrdf_lex::allocation::StorageError),
    /// The locked package/edge profile is malformed, unsupported or ambiguous.
    Locked {
        /// The field whose evidence was refused.
        field: &'static str,
        /// What is wrong and how to correct it.
        message: String,
        /// Native TOML cause for grammar and parser-resource failures.
        source: Option<Box<purrdf_toml::TomlError>>,
    },
    /// The tree refused or failed the file.
    Tree(Box<TreeError>),
    /// The file exceeds a recursion budget of the pre-scan.
    Budget {
        /// The budget that was exceeded.
        budget: Budget,
        /// The most the budget allows.
        limit: usize,
    },
    /// `syn` could not parse the file.
    Syntax {
        /// The parser's message.
        message: String,
        /// The 1-based column of the error on its line.
        column: usize,
    },
    /// The thread that parses could not be started (the operating system
    /// refused the stack or the thread).
    Spawn(io::Error),
    /// A `Cargo.toml` could not be read as a manifest.
    Manifest {
        /// What is wrong with it.
        message: String,
        /// Native TOML cause; schema errors have no parser failure.
        source: Option<Box<purrdf_toml::TomlError>>,
    },
    /// A target is wrong in a way the manifest keys explain.
    Target(Box<TargetProblem>),
    /// A directory entry that discovery would have entered is a symlink.
    DirSymlink,
    /// The load was given no roots, so there is nothing to discover.
    NoRoots,
    /// A `mod name;` names no file.
    UnresolvedMod {
        /// The module name, without a raw-identifier prefix.
        module: String,
    },
    /// Both `name.rs` and `name/mod.rs` exist for a module.
    BothModFiles {
        /// The module name, without a raw-identifier prefix.
        module: String,
        /// The `name.rs` file.
        first: Box<RelPath>,
        /// The `name/mod.rs` file.
        second: Box<RelPath>,
    },
    /// A module has two `path` attributes that apply unconditionally.
    DuplicatePathAttr {
        /// The module name, without a raw-identifier prefix.
        module: String,
    },
    /// A module leads back to a file that is still being followed.
    ModuleCycle {
        /// The module name, without a raw-identifier prefix.
        module: String,
        /// The file that would be entered a second time.
        again: Box<RelPath>,
    },
    /// A `cfg`, `cfg_attr`, `path` or `test` attribute could not be read: it
    /// has the wrong shape, its predicate cannot be read, or the value of a
    /// `path` is not a string literal.
    CfgParse {
        /// What is wrong with it.
        message: String,
    },
    /// The file holds an item that the index does not model.
    UnsupportedItem {
        /// A description of the item.
        what: String,
    },
    /// The trait of an `impl` block cannot be named by an absolute path.
    UnresolvedTrait {
        /// The trait path as written.
        path: String,
    },
    /// An invariant of the index does not hold. This is a bug in the crate,
    /// reported and not guessed around.
    Internal {
        /// What did not hold.
        what: String,
    },
    /// A path climbs above the crate root with `super`.
    SuperAboveRoot {
        /// The path as written.
        path: String,
    },
}

/// What is wrong with a package target.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TargetProblem {
    /// An explicit target path names no file.
    MissingFile {
        /// The path, relative to the tree root.
        path: RelPath,
    },
    /// A target table has no `name`.
    NoName,
    /// Two targets of a package have one root file.
    SharedRoot {
        /// The shared root file.
        root: RelPath,
        /// The Cargo name of the target that holds it first.
        other: String,
    },
    /// A target name could belong to two root files.
    AmbiguousRoot {
        /// The first candidate.
        first: RelPath,
        /// The second candidate.
        second: RelPath,
    },
    /// Two targets of one package have one crate path, so their items could
    /// not be told apart.
    DuplicateCratePath {
        /// The crate path.
        crate_path: String,
        /// The Cargo name of the target that holds it first.
        other: String,
        /// The manifest of that target's package.
        with: RelPath,
    },
    /// Two packages have one name.
    DuplicatePackage {
        /// The package name.
        name: String,
        /// The manifest of the other package.
        with: RelPath,
    },
}

impl fmt::Display for TargetProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFile { path } => {
                write!(f, "the target file {} does not exist", Esc(path.as_str()))
            }
            Self::NoName => f.write_str("the target has no name"),
            Self::SharedRoot { root, other } => {
                write!(
                    f,
                    "the root {} already belongs to the target `{}`",
                    Esc(root.as_str()),
                    Esc(other)
                )
            }
            Self::AmbiguousRoot { first, second } => {
                write!(
                    f,
                    "both {} and {} could be the root",
                    Esc(first.as_str()),
                    Esc(second.as_str())
                )
            }
            Self::DuplicateCratePath {
                crate_path,
                other,
                with,
            } => write!(
                f,
                "the crate path `{}` is already used by the target `{}` of {}",
                Esc(crate_path),
                Esc(other),
                Esc(with.as_str())
            ),
            Self::DuplicatePackage { name, with } => {
                write!(
                    f,
                    "package `{}` is also defined by {}",
                    Esc(name),
                    Esc(with.as_str())
                )
            }
        }
    }
}

impl fmt::Display for LoadErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GitIo { operation, source } => write!(f, "Git {operation} failed: {source}"),
            Self::GitExit { status, diagnostic } => {
                write!(f, "Git failed ({status}): {}", Esc(diagnostic))
            }
            Self::GitCleanup {
                primary,
                operation,
                source,
            } => write!(f, "{primary}; Git {operation} cleanup failed: {source}"),
            Self::Storage(error) => write!(f, "{error}"),
            Self::Locked { field, message, .. } => {
                write!(
                    f,
                    "invalid locked dependency {}: {}",
                    Esc(field),
                    Esc(message)
                )
            }
            Self::Tree(error) => write!(f, "{error}"),
            Self::Budget { budget, limit } => {
                write!(f, "exceeds the budget for {budget} ({limit})")
            }
            Self::Syntax { message, column } => {
                write!(f, "syntax error at column {column}: {}", Esc(message))
            }
            Self::Spawn(error) => write!(f, "cannot start the parse thread: {error}"),
            Self::Manifest { message, .. } => write!(f, "invalid manifest: {}", Esc(message)),
            Self::Target(problem) => write!(f, "{problem}"),
            Self::DirSymlink => f.write_str("a symlinked directory is not followed"),
            Self::NoRoots => f.write_str("no roots were given; name at least one, such as `.`"),
            Self::UnresolvedMod { module } => {
                write!(f, "module `{}` has no file", Esc(module))
            }
            Self::BothModFiles {
                module,
                first,
                second,
            } => write!(
                f,
                "module `{}` has both {} and {}",
                Esc(module),
                Esc(first.as_str()),
                Esc(second.as_str())
            ),
            Self::DuplicatePathAttr { module } => {
                write!(f, "module `{}` has two `path` attributes", Esc(module))
            }
            Self::ModuleCycle { module, again } => write!(
                f,
                "module `{}` leads back to {}, which is still being followed",
                Esc(module),
                Esc(again.as_str())
            ),
            Self::CfgParse { message } => write!(f, "cannot read an attribute: {}", Esc(message)),
            Self::UnsupportedItem { what } => write!(f, "unsupported item: {}", Esc(what)),
            Self::UnresolvedTrait { path } => {
                write!(
                    f,
                    "the trait `{}` of an impl cannot be named by an absolute path",
                    Esc(path)
                )
            }
            Self::Internal { what } => write!(f, "internal error: {}", Esc(what)),
            Self::SuperAboveRoot { path } => {
                write!(
                    f,
                    "`{}` climbs above the crate root with `super`",
                    Esc(path)
                )
            }
        }
    }
}

/// A failure to load one file, naming the file.
#[derive(Debug)]
#[non_exhaustive]
pub struct LoadError {
    /// The file, relative to the tree root.
    pub file: RelPath,
    /// The 1-based line of the failure, when it has one.
    pub line: Option<usize>,
    /// What led to the file, when a walker reached it.
    pub origin: Option<Box<Origin>>,
    /// What went wrong.
    pub kind: LoadErrorKind,
}

impl LoadError {
    pub(crate) fn new(file: &RelPath, line: Option<usize>, kind: LoadErrorKind) -> Self {
        Self {
            file: file.clone(),
            line,
            origin: None,
            kind,
        }
    }

    /// The same error, reached by `origin`.
    pub(crate) fn reached_by(mut self, origin: &Origin) -> Self {
        if self.origin.is_none() {
            self.origin = Some(Box::new(origin.clone()));
        }
        self
    }

    /// A tree failure on `file`, reached by `origin`.
    pub(crate) fn tree(
        file: &RelPath,
        line: Option<usize>,
        origin: &Origin,
        error: TreeError,
    ) -> Self {
        Self::new(file, line, LoadErrorKind::Tree(Box::new(error))).reached_by(origin)
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Esc(self.file.as_str()))?;
        if let Some(line) = self.line {
            write!(f, ":{line}")?;
        }
        write!(f, ": {}", self.kind)?;
        if let Some(origin) = &self.origin {
            write!(f, " (reached by {origin})")?;
        }
        Ok(())
    }
}

impl Error for LoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            LoadErrorKind::Manifest {
                source: Some(source),
                ..
            }
            | LoadErrorKind::Locked {
                source: Some(source),
                ..
            } => Some(source.as_ref()),
            LoadErrorKind::Storage(error) => Some(error),
            LoadErrorKind::GitIo { source, .. } | LoadErrorKind::GitCleanup { source, .. } => {
                Some(source)
            }
            LoadErrorKind::Tree(error) => Some(error),
            LoadErrorKind::Spawn(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;

    use super::{LoadError, LoadErrorKind, Origin, TargetProblem, TreeError};
    use crate::RelPath;

    /// Every kind of control character, including a NUL.
    const HOSTILE: &str = "a\u{1b}]0;pwned\u{7}\u{1b}[2J\0\r\nb";
    /// The same for a `RelPath`, which refuses a NUL.
    const HOSTILE_PATH: &str = "a\u{1b}]0;pwned\u{7}\u{1b}[2J\r\nb";

    fn clean(text: &str) {
        assert!(
            !text.chars().any(char::is_control),
            "a control character got through: {text:?}"
        );
    }

    #[test]
    fn control_characters_in_a_path_are_escaped() {
        let file = RelPath::new(&format!("src/{HOSTILE_PATH}.rs")).expect("a relative path");
        let text = LoadError::tree(
            &file,
            Some(3),
            &Origin::PathAttr {
                module: HOSTILE.to_owned(),
                predicate: Some(HOSTILE.to_owned()),
            },
            TreeError::OutsideRoot {
                path: HOSTILE.to_owned(),
            },
        )
        .to_string();
        clean(&text);
        assert!(
            text.contains(r"a\u{1b}]0;pwned\u{7}\u{1b}[2J\0\r\nb"),
            "{text}"
        );
        assert!(text.starts_with("src/a\\u{1b}"), "{text}");
    }

    #[test]
    fn every_user_derived_field_of_every_display_is_escaped() {
        let file = RelPath::new("x").expect("a relative path");
        let hostile = || HOSTILE.to_owned();
        let path = || RelPath::new(&format!("d/{HOSTILE_PATH}")).expect("a relative path");
        let tree_errors = [
            TreeError::BackendAuthority {
                path: hostile(),
                authority: crate::SourceAuthorityId(1),
            },
            TreeError::WrongAuthority {
                path: hostile(),
                authority: crate::SourceAuthorityId(1),
            },
            TreeError::NotFound { path: hostile() },
            TreeError::OutsideRoot { path: hostile() },
            TreeError::NotRegular { path: hostile() },
            TreeError::SymlinkLoop { path: hostile() },
            TreeError::TooLarge {
                path: hostile(),
                limit: 1,
            },
            TreeError::NotUtf8 { path: hostile() },
            TreeError::NonUtf8Name {
                path: hostile(),
                name: hostile(),
            },
            TreeError::DuplicatePath { path: hostile() },
            TreeError::Io {
                path: hostile(),
                source: io::Error::other("x"),
            },
        ];
        for error in tree_errors {
            clean(&error.to_string());
        }
        let origins = [
            Origin::ModDecl { module: hostile() },
            Origin::ManifestKey { key: hostile() },
            Origin::PathAttr {
                module: hostile(),
                predicate: None,
            },
        ];
        for origin in origins {
            clean(&origin.to_string());
        }
        let problems = [
            TargetProblem::MissingFile { path: path() },
            TargetProblem::SharedRoot {
                root: path(),
                other: hostile(),
            },
            TargetProblem::AmbiguousRoot {
                first: path(),
                second: path(),
            },
            TargetProblem::DuplicateCratePath {
                crate_path: hostile(),
                other: hostile(),
                with: path(),
            },
            TargetProblem::DuplicatePackage {
                name: hostile(),
                with: path(),
            },
        ];
        for problem in problems {
            clean(&problem.to_string());
        }
        let kinds = vec![
            LoadErrorKind::Syntax {
                message: hostile(),
                column: 1,
            },
            LoadErrorKind::Manifest {
                message: hostile(),
                source: None,
            },
            LoadErrorKind::UnresolvedMod { module: hostile() },
            LoadErrorKind::BothModFiles {
                module: hostile(),
                first: Box::new(path()),
                second: Box::new(path()),
            },
            LoadErrorKind::DuplicatePathAttr { module: hostile() },
            LoadErrorKind::ModuleCycle {
                module: hostile(),
                again: Box::new(path()),
            },
            LoadErrorKind::CfgParse { message: hostile() },
            LoadErrorKind::UnsupportedItem { what: hostile() },
            LoadErrorKind::UnresolvedTrait { path: hostile() },
            LoadErrorKind::Internal { what: hostile() },
            LoadErrorKind::SuperAboveRoot { path: hostile() },
        ];
        for kind in kinds {
            clean(&LoadError::new(&file, None, kind).to_string());
        }
        let hostile_file = RelPath::new(&format!("{HOSTILE_PATH}.rs")).expect("a relative path");
        clean(&LoadError::new(&hostile_file, None, LoadErrorKind::DirSymlink).to_string());
    }

    #[test]
    fn a_spawn_error_displays_the_file_and_carries_its_cause() {
        let file = RelPath::new("src/lib.rs").expect("a plain path");
        let error = LoadError::new(
            &file,
            None,
            LoadErrorKind::Spawn(io::Error::other("no threads left")),
        );
        assert!(matches!(error.kind, LoadErrorKind::Spawn(_)));
        let text = error.to_string();
        assert!(text.starts_with("src/lib.rs: "), "{text}");
        assert!(text.contains("cannot start the parse thread"), "{text}");
        let source = error.source().expect("a spawn error has a cause");
        assert_eq!(source.to_string(), "no threads left");
    }
}
