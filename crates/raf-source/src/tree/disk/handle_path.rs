// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Windows: the canonical path is opened as a handle, held without delete
//! sharing (a directory with backup semantics), and the path the system gives
//! for that handle (`GetFinalPathNameByHandleW`) must lie under the root,
//! after the open and again after each use. A file is read through the
//! handle, so its contents are those of the verified file; a listing and an
//! entry lookup go by the handle's path, between the two checks. An outside
//! file can be opened in a race and is then refused before any read.
//!
//! A Linux test build runs the same strategy with `/proc/self/fd/<fd>` as the
//! handle's path and a non-blocking open, so it is exercised there.

use std::fs::File;
#[cfg(unix)]
use std::fs::Metadata;
use std::path::{Path, PathBuf};

use super::{
    DiskTree, Entry, EntryKind, Want, link_read_error, lookup, open_error, read_capped,
    read_entries, refuse_unless_directory, verify_handle_path,
};
use crate::error::TreeError;
use crate::path::RelPath;
use crate::tree::FileId;

/// A handle with the verified path the system gives for it.
pub(super) struct Handle {
    file: File,
    path: PathBuf,
    #[cfg(unix)]
    metadata: Metadata,
}

/// Contain `path`, open the canonical path as a handle, and verify the path
/// the system gives for the handle against the root, and the handle against
/// `want`.
pub(super) fn open(
    tree: &DiskTree,
    path: &RelPath,
    want: Want,
    shown: &str,
) -> Result<Handle, TreeError> {
    let candidate = tree.candidate(path);
    let canonical = tree.contain(&candidate, shown)?;
    if want == Want::Listing {
        refuse_unless_directory(&candidate, shown)?;
    }
    hook!(tree, before_open);
    let file = open_handle(tree, &canonical, want).map_err(|e| open_error(shown, e))?;
    hook!(tree, after_open);
    let path = verified_path(&tree.root, &file, shown)?;
    let metadata = file.metadata().map_err(|e| TreeError::from_io(shown, e))?;
    want.require(&metadata, shown)?;
    Ok(Handle {
        file,
        path,
        #[cfg(unix)]
        metadata,
    })
}

/// The path the system gives for `file`, which must lie under `root`.
fn verified_path(root: &Path, file: &File, shown: &str) -> Result<PathBuf, TreeError> {
    let path = path_of(file).map_err(|e| link_read_error(shown, e))?;
    verify_handle_path(root, &path, shown)?;
    Ok(path)
}

/// Windows: the normalised path of the handle, with its drive letter.
#[cfg(windows)]
fn path_of(file: &File) -> std::io::Result<PathBuf> {
    super::final_path::of(file)
}

/// A Linux test build: the kernel's path for the handle.
#[cfg(unix)]
fn path_of(file: &File) -> std::io::Result<PathBuf> {
    std::fs::read_link(super::descriptor_path(file))
}

/// `FILE_SHARE_READ`: others may read the file while the handle is held.
#[cfg(windows)]
const FILE_SHARE_READ: u32 = 0x0000_0001;
/// `FILE_SHARE_WRITE`: others may write the file while the handle is held.
#[cfg(windows)]
const FILE_SHARE_WRITE: u32 = 0x0000_0002;
/// `FILE_FLAG_BACKUP_SEMANTICS`: lets a directory be opened as a handle.
#[cfg(windows)]
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;

/// Windows: open `path` as a handle, a directory as well as a file. Delete
/// sharing is withheld, so while the handle is held the file or directory
/// cannot be renamed or deleted under it.
#[cfg(windows)]
fn open_handle(_tree: &DiskTree, path: &Path, _want: Want) -> std::io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt as _;
    std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)
}

/// A Linux test build: open `path` without blocking and without following a
/// final symlink; a directory is opened as one.
#[cfg(unix)]
fn open_handle(tree: &DiskTree, path: &Path, want: Want) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt as _;
    let directory = if want == Want::File {
        0
    } else {
        libc::O_DIRECTORY
    };
    let flags = libc::O_NONBLOCK | libc::O_NOFOLLOW | directory;
    #[cfg(test)]
    tree.note_open(flags.cast_unsigned());
    std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(flags)
        .open(path)
}

impl Handle {
    /// Unix: the device and inode of the handle.
    #[cfg(unix)]
    #[expect(
        clippy::unnecessary_wraps,
        reason = "the signature is shared with Windows, where the identity can fail"
    )]
    pub(super) fn identity(&self, _root: &Path, _shown: &str) -> Result<FileId, TreeError> {
        Ok(super::device_identity(&self.metadata))
    }

    /// Windows: the verified path of the handle, below the root.
    #[cfg(windows)]
    pub(super) fn identity(&self, root: &Path, shown: &str) -> Result<FileId, TreeError> {
        let below = self
            .path
            .strip_prefix(root)
            .map_err(|_| super::outside(shown))?;
        Ok(FileId::from_native(below.as_os_str().to_owned()))
    }

    /// The path the system gives for the handle must still lie under `root`.
    pub(super) fn recheck(&self, root: &Path, shown: &str) -> Result<(), TreeError> {
        verified_path(root, &self.file, shown).map(drop)
    }

    /// Read at most `max_bytes` bytes of text from the verified handle.
    pub(super) fn read(&self, max_bytes: u64, shown: &str) -> Result<String, TreeError> {
        read_capped(&self.file, max_bytes, shown)
    }

    /// List the directory at the handle's verified path.
    pub(super) fn entries(&self, shown: &str) -> Result<Vec<Entry>, TreeError> {
        read_entries(&self.path, shown)
    }

    /// Look `name` up below the handle's verified path.
    pub(super) fn lookup(&self, name: &str, shown: &str) -> Result<Option<EntryKind>, TreeError> {
        lookup(&self.path.join(name), shown)
    }
}
