// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Judging the way to a file the crate writes, and creating a new file below
//! a root, exclusively.
//!
//! Every write of a record, a change record or a generated document goes
//! through [`target`] first: each directory on the way must be a directory and
//! not a symbolic link, so a link inside the root never lets a write land in
//! another directory, such as a record replacing a document.

use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::Path;

use raf_source::{DiskTree, EntryKind, RelPath, Tree};

/// Why a file was not created.
#[derive(Debug)]
pub(crate) enum CreateError {
    /// A file, or a link, is already at the path. It is untouched.
    Exists,
    /// The file could not be written, or a directory it needs is refused.
    Write {
        /// The path that was being written or judged.
        path: String,
        source: io::Error,
    },
}

fn to_io(error: raf_source::TreeError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, error)
}

fn write_error(path: &str, source: io::Error) -> CreateError {
    CreateError::Write {
        path: path.to_string(),
        source,
    }
}

/// A path on the way to a write that is refused, and why. Nothing is written.
#[derive(Debug)]
pub(crate) struct Refused {
    /// The refused directory, or the path when the tree cannot judge it.
    pub(crate) path: String,
    pub(crate) source: io::Error,
}

impl From<Refused> for CreateError {
    fn from(refused: Refused) -> Self {
        Self::Write {
            path: refused.path,
            source: refused.source,
        }
    }
}

fn refused(path: &str, source: io::Error) -> Refused {
    Refused {
        path: path.to_string(),
        source,
    }
}

/// What is at the path a write is about to use, once every directory on the
/// way to it has been judged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Target {
    /// Whether every directory on the way exists. When one does not, nothing
    /// below it exists either.
    pub(crate) directories: bool,
    /// What is at the path itself, a final symbolic link not followed; `None`
    /// when nothing is there.
    pub(crate) kind: Option<EntryKind>,
}

/// Judge the way to `path`, which lies directly in `directory`, a
/// `/`-separated path below `root`, and say what is at `path`.
///
/// Each directory on the way (for `work/evidence`, `work` and then
/// `work/evidence`) must be a directory or not exist; a symbolic link, wherever
/// it points, a file or anything else there is refused. The tree of `root`
/// judges each of them, so none leads outside the root, and since none is a
/// link, the path names the file it names in `root` and no other.
///
/// # Errors
///
/// [`Refused`] naming the refused directory, or the path when the tree cannot
/// judge it. Nothing is written.
pub(crate) fn target(root: &Path, directory: &str, path: &RelPath) -> Result<Target, Refused> {
    let tree = DiskTree::new(root).map_err(|error| refused(path.as_str(), to_io(error)))?;
    let mut prefix = String::new();
    for part in directory.split('/') {
        if !prefix.is_empty() {
            prefix.push('/');
        }
        prefix.push_str(part);
        let rel = RelPath::new(&prefix).map_err(|error| {
            refused(&prefix, io::Error::new(io::ErrorKind::InvalidInput, error))
        })?;
        match tree.entry(&rel) {
            Ok(Some(EntryKind::Dir)) => {}
            Ok(None) => {
                return Ok(Target {
                    directories: false,
                    kind: None,
                });
            }
            Ok(Some(EntryKind::Symlink)) => {
                return Err(refused(
                    &prefix,
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "it is a symbolic link, and nothing is written through a link",
                    ),
                ));
            }
            Ok(Some(_)) => {
                return Err(refused(
                    &prefix,
                    io::Error::new(io::ErrorKind::InvalidInput, "it is not a directory"),
                ));
            }
            Err(error) => return Err(refused(&prefix, to_io(error))),
        }
    }
    let kind = tree
        .entry(path)
        .map_err(|error| refused(path.as_str(), to_io(error)))?;
    Ok(Target {
        directories: true,
        kind,
    })
}

/// Create `path` below `root` with `text`, exclusively.
///
/// `path` lies directly in `directory`, a `/`-separated path below `root`,
/// and the way to it is judged by [`target`]: a symbolic link or a file on the
/// way is refused, and a missing directory is created. The file is created
/// with `create_new`, so an existing file or link at the path is
/// [`CreateError::Exists`] and is not touched. A failure after the file was
/// created removes it again.
pub(crate) fn create_new(
    root: &Path,
    directory: &str,
    path: &RelPath,
    text: &str,
) -> Result<(), CreateError> {
    let found = target(root, directory, path)?;
    if found.kind.is_some() {
        return Err(CreateError::Exists);
    }
    if !found.directories {
        fs::create_dir_all(root.join(directory)).map_err(|error| write_error(directory, error))?;
    }
    let target = root.join(path.as_str());
    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)
    {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            return Err(CreateError::Exists);
        }
        Err(error) => return Err(write_error(path.as_str(), error)),
    };
    let written = file
        .write_all(text.as_bytes())
        .and_then(|()| file.sync_all());
    if let Err(error) = written {
        drop(file);
        // The file was created by this call just above; nothing else is
        // removed.
        let _ = fs::remove_file(&target);
        return Err(write_error(path.as_str(), error));
    }
    Ok(())
}
