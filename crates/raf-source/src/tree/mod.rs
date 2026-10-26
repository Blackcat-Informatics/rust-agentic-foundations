// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The `Tree` seam: every file the crate looks at goes through it.

mod disk;
mod git;
mod memory;
mod sources;

pub use disk::DiskTree;
pub use git::CommittedGitTree;
pub use memory::MemoryTree;
pub use sources::SourceTrees;

use crate::error::TreeError;
use crate::path::{RelPath, SourceAuthorityId};
use std::sync::Arc;

/// Standalone backends cannot silently interpret another source's spelling.
fn require_first_party(path: &RelPath) -> Result<(), TreeError> {
    if path.authority() == SourceAuthorityId::FIRST_PARTY {
        Ok(())
    } else {
        Err(TreeError::WrongAuthority {
            path: path.to_string(),
            authority: path.authority(),
        })
    }
}

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

/// The identity of a file: two paths whose identities are equal name one file.
///
/// [`DiskTree`] takes it from the verified handle: on Unix the device and
/// inode numbers, on Windows the path the system gives for the handle, below
/// the root. [`MemoryTree`]
/// uses the UTF-8 bytes of its normalised key. Another backend builds one with
/// [`FileId::from_device`] or [`FileId::from_key`].
/// A composite tree scopes that same backend identity with [`FileId::in_source`];
/// equal backend keys in different authorities are different files.
///
/// A device and inode pair names one file only where the file system keeps
/// inode numbers unique and stable while the tree is read, as local file
/// systems do; a file system that reuses or invents them (some network and
/// user-space mounts) can make two files look like one.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId {
    authority: SourceAuthorityId,
    identity: Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Identity {
    /// Only the checked Git backend creates snapshot/path identities.
    Git { commit: [u8; 20], path: RelPath },
    /// A device and an inode number.
    Device { dev: u64, ino: u64 },
    /// An opaque byte key that names the file in its backend.
    Key(Arc<[u8]>),
}

impl FileId {
    pub(crate) fn git(commit: [u8; 20], path: &RelPath) -> Self {
        Self {
            authority: SourceAuthorityId::FIRST_PARTY,
            identity: Identity::Git {
                commit,
                path: path.clone(),
            },
        }
    }
    /// The identity of a file given by its device and inode numbers.
    #[must_use]
    pub fn from_device(dev: u64, ino: u64) -> Self {
        Self {
            authority: SourceAuthorityId::FIRST_PARTY,
            identity: Identity::Device { dev, ino },
        }
    }

    /// The identity of a file given by a key that names exactly one file of
    /// the backend, such as its normalised path or native path encoding.
    ///
    /// Keys are compared byte for byte without a text conversion. Textual
    /// callers supply their UTF-8 bytes; native backends can preserve keys
    /// that are not valid UTF-8. The copied payload is shared by clones.
    #[must_use]
    pub fn from_key(key: impl AsRef<[u8]>) -> Self {
        Self {
            authority: SourceAuthorityId::FIRST_PARTY,
            identity: Identity::Key(key.as_ref().into()),
        }
    }

    /// Explicitly scope the same backend identity to a selected source.
    ///
    /// The underlying device/key identity stays intact, including aliases to one
    /// physical file. Cloning a byte-key identity shares its existing payload.
    #[must_use]
    pub fn in_source(mut self, authority: SourceAuthorityId) -> Self {
        self.authority = authority;
        self
    }

    /// The source authority in which this backend identity is interpreted.
    #[must_use]
    pub const fn authority(&self) -> SourceAuthorityId {
        self.authority
    }

    /// The bytes the identity holds on the heap beyond its own size.
    pub(crate) fn heap_bytes(&self) -> usize {
        match &self.identity {
            Identity::Git { path, .. } => crate::cost::shared_text(path.as_str().len()),
            Identity::Device { .. } => 0,
            Identity::Key(key) => crate::cost::shared_text(key.len()),
        }
    }
}

/// What [`Tree::read_unseen`] found at a path.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Opened {
    /// A file whose identity the caller already holds; it was not read.
    Seen(FileId),
    /// A file read in full, with its identity from the same open.
    Read {
        /// The identity of the file.
        id: FileId,
        /// Its text.
        text: String,
    },
}

/// What [`DiskTree::read_unseen_bytes`] found at a path.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum OpenedBytes {
    /// An already-seen file; its payload was not read.
    Seen(FileId),
    /// The complete bytes and identity from the same verified file handle.
    Read {
        /// The identity of the file.
        id: FileId,
        /// Its arbitrary bytes, without UTF-8 validation or conversion.
        bytes: Vec<u8>,
    },
}

/// One entry of a directory listing.
///
/// The struct is `#[non_exhaustive]`: adding a field is not a breaking change,
/// so a pattern names the fields it reads and ends in `..`, and a [`Tree`]
/// backend outside the crate builds an entry with [`Entry::new`].
///
/// ```compile_fail,E0638
/// # fn names(entry: &raf_source::Entry) {
/// let raf_source::Entry { name, kind } = entry;
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Entry {
    /// The entry name: one segment, without a directory part.
    pub name: String,
    /// What the entry is.
    pub kind: EntryKind,
}

impl Entry {
    /// An entry named `name` of kind `kind`.
    ///
    /// ```rust
    /// use raf_source::{Entry, EntryKind};
    ///
    /// let entry = Entry::new("lib.rs", EntryKind::File);
    /// assert_eq!((entry.name.as_str(), entry.kind), ("lib.rs", EntryKind::File));
    /// ```
    #[must_use]
    pub fn new(name: impl Into<String>, kind: EntryKind) -> Self {
        Self {
            name: name.into(),
            kind,
        }
    }
}

/// A read-only tree of UTF-8 text files below one root.
///
/// The trait is object-safe and has no `Send` or `Sync` bound: loading is
/// serial, and requiring `Tree: Sync` for a parallel load would be a breaking
/// change to [`Workspace::load`](crate::Workspace::load). Paths are
/// [`RelPath`] values, so no call can name a place above the root by text. A
/// backend may still refuse a path that its contents lead out of.
///
/// Standalone backends accept first-party paths and refuse other authorities
/// with [`TreeError::WrongAuthority`], rather than discarding their tags.
/// [`SourceTrees`] selects a backend by authority and deliberately passes it an
/// unqualified path, then scopes returned file identities to the selected source.
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

    /// Open the file at `path` once, take its identity from that open, and
    /// read its text from the same open unless `seen` says the identity is
    /// one the caller already holds.
    ///
    /// Two paths that name one file, such as a path and the same path through
    /// a symlink to a directory, have equal identities, and two different
    /// files never do. Because the identity and the text come from one open,
    /// no change to the tree between two calls can pair one file's identity
    /// with another file's text.
    ///
    /// # Errors
    ///
    /// As [`Tree::read`]; a file that `seen` accepts is not read, so it is
    /// never [`TreeError::TooLarge`] or [`TreeError::NotUtf8`].
    fn read_unseen(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError>;

    /// The identity of the file at `path`, without reading it: the identity
    /// [`Tree::read_unseen`] gives when every identity counts as seen.
    ///
    /// # Errors
    ///
    /// As [`Tree::read`], without [`TreeError::TooLarge`] and
    /// [`TreeError::NotUtf8`].
    fn identity(&self, path: &RelPath) -> Result<FileId, TreeError> {
        match self.read_unseen(path, 0, &|_| true)? {
            Opened::Seen(id) | Opened::Read { id, .. } => Ok(id),
        }
    }

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
