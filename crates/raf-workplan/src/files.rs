// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Judging the way to a file the crate writes.
//!
//! These path-based helpers inspect explicit metadata destinations and caller-supplied
//! record stores. Existing ancestors must be real directories. Links present
//! during inspection are refused, but a later name swap remains outside this
//! inspection's guarantee. The native disk record store, document sync and
//! diagram publishers retain directory authority through [`crate::publication_io`].

use std::io;
use std::path::Path;

use raf_source::{DiskTree, EntryKind, RelPath, Tree};

fn to_io(error: raf_source::TreeError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, error)
}

/// A path on the way to a write that is refused, and why. Nothing is written.
#[derive(Debug)]
pub(crate) struct Refused {
    /// The refused directory, or the path when the tree cannot judge it.
    pub(crate) path: String,
    pub(crate) source: io::Error,
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
/// link, the path names the file it names in `root` at that inspection.
///
/// # Errors
///
/// [`Refused`] naming the refused directory, or the path when the tree cannot
/// judge it. Nothing is written.
pub(crate) fn target(root: &Path, directory: &str, path: &RelPath) -> Result<Target, Refused> {
    let tree = DiskTree::new(root).map_err(|error| refused(path.as_str(), to_io(error)))?;
    protect_git(root, path)?;
    target_in(&tree, directory, path)
}

/// Explicit data writers must never publish within repository administration.
pub(crate) fn protect_git(root: &Path, path: &RelPath) -> Result<(), Refused> {
    let root = root
        .canonicalize()
        .map_err(|error| refused(path.as_str(), error))?;
    let destination = root.join(path.as_str());
    if destination
        .components()
        .any(|part| part.as_os_str() == ".git")
    {
        return Err(refused(
            path.as_str(),
            io::Error::other("data destinations cannot enter Git administration"),
        ));
    }
    if let Some(repository) = crate::git::Repository::discover(&root)
        .map_err(|error| refused(path.as_str(), io::Error::other(error)))?
        && (destination.starts_with(repository.git_dir())
            || destination.starts_with(repository.common_dir()))
    {
        return Err(refused(
            path.as_str(),
            io::Error::other("data destinations cannot enter Git administration"),
        ));
    }
    Ok(())
}

/// The same destination inspection against a supplied native reader.
pub(crate) fn target_in(
    tree: &dyn Tree,
    directory: &str,
    path: &RelPath,
) -> Result<Target, Refused> {
    let root = path.source_root();
    match tree.entry(&root) {
        Ok(Some(EntryKind::Dir)) => {}
        Ok(_) => {
            return Err(refused(
                &root.to_string(),
                io::Error::new(io::ErrorKind::InvalidInput, "the root is not a directory"),
            ));
        }
        Err(error) => return Err(refused(&root.to_string(), to_io(error))),
    }
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
