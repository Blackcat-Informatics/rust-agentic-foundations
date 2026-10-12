// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The typed errors of the tree backends.

use std::error::Error;
use std::fmt;
use std::io;

use crate::budget::Budget;
use crate::path::{PathError, RelPath};

/// Why a [`Tree`](crate::Tree) call failed.
///
/// Every variant names the path it concerns. Only [`TreeError::NotFound`]
/// means that nothing is there; every other variant is a refusal or a failure
/// and must never be read as absence.
#[derive(Debug)]
#[non_exhaustive]
pub enum TreeError {
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
            Self::NotFound { path }
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
            Self::NotFound { path } => write!(f, "{path}: no such file or directory"),
            Self::OutsideRoot { path } => write!(f, "{path} leads outside the root"),
            Self::NotRegular { path } => {
                write!(f, "{path} is not a regular file or a real directory")
            }
            Self::SymlinkLoop { path } => write!(f, "{path} runs into a symlink loop"),
            Self::TooLarge { path, limit } => {
                write!(f, "{path} is larger than the limit of {limit} bytes")
            }
            Self::NotUtf8 { path } => write!(f, "{path} is not valid UTF-8"),
            Self::NonUtf8Name { path, name } => write!(
                f,
                "directory {path} holds an entry whose name is not valid UTF-8: {name:?}"
            ),
            Self::DuplicatePath { path } => {
                write!(f, "{path} names a path that an earlier key already holds")
            }
            Self::Io { path, source } => write!(f, "{path}: {source}"),
        }
    }
}

impl Error for TreeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
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
            Self::ModDecl { module } => write!(f, "`mod {module};`"),
            Self::PathAttr {
                module,
                predicate: None,
            } => write!(f, "the `path` attribute of module `{module}`"),
            Self::PathAttr {
                module,
                predicate: Some(predicate),
            } => write!(
                f,
                "the `path` attribute of module `{module}` under `{predicate}`"
            ),
            Self::ManifestKey { key } => write!(f, "manifest key `{key}`"),
            Self::Discovery => f.write_str("package discovery"),
        }
    }
}

/// What went wrong while loading a file.
#[derive(Debug)]
#[non_exhaustive]
pub enum LoadErrorKind {
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
    },
    /// A target is wrong in a way the manifest keys explain.
    Target(Box<TargetProblem>),
    /// A directory entry that discovery would have entered is a symlink.
    DirSymlink,
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
    /// A `cfg` or `cfg_attr` predicate could not be read.
    CfgParse {
        /// What is wrong with it.
        message: String,
    },
    /// The file holds an item that the index does not model.
    UnsupportedItem {
        /// A description of the item.
        what: String,
    },
    /// An inherent `impl` block whose self type does not resolve, through
    /// type aliases, to a struct, enum or union of the workspace.
    InherentImplSelf {
        /// The self type as written.
        self_type: String,
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
    /// Two targets have one crate path, so their items could not be told apart.
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
            Self::MissingFile { path } => write!(f, "the target file {path} does not exist"),
            Self::NoName => f.write_str("the target has no name"),
            Self::SharedRoot { root, other } => {
                write!(f, "the root {root} already belongs to the target `{other}`")
            }
            Self::AmbiguousRoot { first, second } => {
                write!(f, "both {first} and {second} could be the root")
            }
            Self::DuplicateCratePath {
                crate_path,
                other,
                with,
            } => write!(
                f,
                "the crate path `{crate_path}` is already used by the target `{other}` of {with}"
            ),
            Self::DuplicatePackage { name, with } => {
                write!(f, "package `{name}` is also defined by {with}")
            }
        }
    }
}

impl fmt::Display for LoadErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tree(error) => write!(f, "{error}"),
            Self::Budget { budget, limit } => {
                write!(f, "exceeds the budget for {budget} ({limit})")
            }
            Self::Syntax { message, column } => {
                write!(f, "syntax error at column {column}: {message}")
            }
            Self::Spawn(error) => write!(f, "cannot start the parse thread: {error}"),
            Self::Manifest { message } => write!(f, "invalid manifest: {message}"),
            Self::Target(problem) => write!(f, "{problem}"),
            Self::DirSymlink => f.write_str("a symlinked directory is not followed"),
            Self::UnresolvedMod { module } => {
                write!(f, "module `{module}` has no file")
            }
            Self::BothModFiles {
                module,
                first,
                second,
            } => write!(f, "module `{module}` has both {first} and {second}"),
            Self::DuplicatePathAttr { module } => {
                write!(f, "module `{module}` has two `path` attributes")
            }
            Self::ModuleCycle { module, again } => write!(
                f,
                "module `{module}` leads back to {again}, which is still being followed"
            ),
            Self::CfgParse { message } => write!(f, "cannot read a cfg predicate: {message}"),
            Self::UnsupportedItem { what } => write!(f, "unsupported item: {what}"),
            Self::InherentImplSelf { self_type } => write!(
                f,
                "the self type `{self_type}` of an inherent impl is not a local struct, enum or union"
            ),
            Self::UnresolvedTrait { path } => {
                write!(
                    f,
                    "the trait `{path}` of an impl cannot be named by an absolute path"
                )
            }
            Self::Internal { what } => write!(f, "internal error: {what}"),
            Self::SuperAboveRoot { path } => {
                write!(f, "`{path}` climbs above the crate root with `super`")
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
        write!(f, "{}", self.file)?;
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
            LoadErrorKind::Tree(error) => Some(error),
            LoadErrorKind::Spawn(error) => Some(error),
            _ => None,
        }
    }
}
