// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Process-free workspace discovery through `raf-source`.

use raf_source::{DiskTree, LoadError, RelPath, TreeError};
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

/// Why a workspace root cannot be discovered.
#[derive(Debug)]
#[non_exhaustive]
pub enum RootError {
    /// The starting directory cannot be canonicalized.
    Directory(std::io::Error),
    /// The filesystem tree cannot be opened.
    Tree(TreeError),
    /// A manifest or membership finding prevents discovery.
    Source(LoadError),
    /// A filesystem path cannot be represented as a portable tree path.
    Path(String),
}
impl fmt::Display for RootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Directory(error) => write!(f, "cannot open discovery directory: {error}"),
            Self::Tree(error) => write!(f, "{error}"),
            Self::Source(error) => write!(f, "{error}"),
            Self::Path(path) => write!(
                f,
                "cannot represent discovery path `{}` as a tree path",
                path.escape_debug()
            ),
        }
    }
}
impl Error for RootError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Directory(error) => Some(error),
            Self::Tree(error) => Some(error),
            Self::Source(error) => Some(error),
            Self::Path(_) => None,
        }
    }
}
/// Find the workspace containing `directory`, without starting any process.
///
/// # Errors
///
/// Returns [`RootError`] for an inaccessible directory, a malformed manifest,
/// an unlisted package, or conflicting membership. See
/// [`raf_source::discover_root`] for the tree-based discovery contract.
pub fn discover(directory: &Path) -> Result<PathBuf, RootError> {
    let directory = directory.canonicalize().map_err(RootError::Directory)?;
    let boundary = directory
        .ancestors()
        .last()
        .ok_or_else(|| RootError::Path(directory.display().to_string()))?;
    let relative = directory
        .strip_prefix(boundary)
        .map_err(|_| RootError::Path(directory.display().to_string()))?;
    let text = relative
        .to_str()
        .ok_or_else(|| RootError::Path(relative.display().to_string()))?;
    let start = RelPath::new(&text.replace(std::path::MAIN_SEPARATOR, "/"))
        .map_err(|_| RootError::Path(text.to_owned()))?;
    let tree = DiskTree::new(boundary).map_err(RootError::Tree)?;
    let root = raf_source::discover_root(&tree, &start).map_err(RootError::Source)?;
    Ok(boundary.join(root.as_str()))
}
