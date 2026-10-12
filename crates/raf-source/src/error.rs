// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The typed errors of the tree backends.

use std::error::Error;
use std::fmt;
use std::io;

use crate::path::PathError;

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
