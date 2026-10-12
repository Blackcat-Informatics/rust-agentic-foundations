// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The `Tree` seam: every file the crate looks at goes through it.

mod disk;
mod memory;

pub use disk::DiskTree;
pub use memory::MemoryTree;

use crate::error::TreeError;
use crate::path::RelPath;

/// What a directory entry is, judged without following a final symlink.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum EntryKind {
    /// A regular file.
    File,
    /// A directory.
    Dir,
    /// A symbolic link, whatever it points at.
    Symlink,
    /// A FIFO, a socket, a device or anything else.
    Other,
}

/// One entry of a directory listing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entry {
    /// The entry name: one segment, without a directory part.
    pub name: String,
    /// What the entry is.
    pub kind: EntryKind,
}

/// A read-only tree of UTF-8 text files below one root.
///
/// The trait is object-safe and has no `Send` or `Sync` bound. Paths are
/// [`RelPath`] values, so no call can name a place above the root by text. A
/// backend may still refuse a path that its contents lead out of.
pub trait Tree {
    /// The text of the file at `path`.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::NotFound`] when nothing is there,
    /// [`TreeError::OutsideRoot`] when the path leads out of the root,
    /// [`TreeError::NotRegular`] when it is not a regular file,
    /// [`TreeError::TooLarge`] when the file holds more than `max_bytes`
    /// bytes, [`TreeError::NotUtf8`] when it is not UTF-8, and
    /// [`TreeError::SymlinkLoop`] or [`TreeError::Io`] when the operating
    /// system fails.
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError>;

    /// The entries of the directory at `path`, sorted by name.
    ///
    /// A symlink is listed as [`EntryKind::Symlink`] and is never followed:
    /// listing a symlink itself is refused.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::NotFound`] when the directory is missing,
    /// [`TreeError::NotRegular`] when `path` is not a real directory,
    /// [`TreeError::OutsideRoot`] when it leads out of the root,
    /// [`TreeError::NonUtf8Name`] when an entry name is not UTF-8 (the entry
    /// is never dropped), and [`TreeError::SymlinkLoop`] or
    /// [`TreeError::Io`] when the operating system fails.
    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError>;

    /// What is at `path`, or `None` when nothing is.
    ///
    /// A final symlink is reported as [`EntryKind::Symlink`] and not followed.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::OutsideRoot`] when the directory holding `path`
    /// leads out of the root, and [`TreeError::SymlinkLoop`] or
    /// [`TreeError::Io`] when the operating system fails. A missing entry is
    /// `Ok(None)`, not an error.
    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError>;
}
