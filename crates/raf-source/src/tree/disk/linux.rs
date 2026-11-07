// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Linux: a candidate is first taken as an `O_PATH` reference, which performs
//! no I/O: it runs no device's open handler, completes no FIFO rendezvous and
//! reads nothing. The kernel's own path for the reference must lie under the
//! root and its type must be the one wanted. Only then is it opened for
//! reading or listed, through its descriptor path, which names the inode the
//! reference holds and is never resolved by name again. An outside candidate
//! is therefore never opened for reading or listing.

use std::ffi::OsString;
use std::fs::{File, Metadata, OpenOptions};
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::{Path, PathBuf};

use super::{
    DiskTree, Entry, EntryKind, Want, descriptor_path, device_identity, lookup, open_error,
    read_capped, read_entries, refuse_unless_directory, verify_linux_link,
};
use crate::error::TreeError;
use crate::path::RelPath;
use crate::tree::FileId;
use rustix::fs::{AtFlags, fstat, statat};

/// A directory capability is exposed only while it names the admitted scope.
pub(super) fn verify_root(tree: &DiskTree, shown: &str) -> Result<(), TreeError> {
    let actual = std::fs::read_link(descriptor_path(&tree.root_directory.dir))
        .map_err(|e| super::link_read_error(shown, e))?;
    if actual == tree.root {
        Ok(())
    } else {
        Err(super::outside(shown))
    }
}

/// A verified `O_PATH` reference.
pub(super) struct Handle {
    reference: File,
    metadata: Metadata,
    parent: Option<(File, OsString)>,
}

/// Contain `path`, take an `O_PATH` reference to the canonical path without
/// following a final symlink, and verify it against the root and `want`.
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
    let flags = libc::O_PATH | libc::O_NOFOLLOW;
    #[cfg(test)]
    tree.note_open(flags.cast_unsigned());
    let reference = OpenOptions::new()
        .read(true)
        .custom_flags(flags)
        .open(&canonical)
        .map_err(|e| open_error(shown, e))?;
    hook!(tree, after_open);
    verify_linux_link(&tree.root, &descriptor_path(&reference), shown)?;
    let metadata = reference
        .metadata()
        .map_err(|e| TreeError::from_io(shown, e))?;
    want.require(&metadata, shown)?;
    let parent = if metadata.is_file() {
        let directory = canonical
            .parent()
            .ok_or_else(|| super::not_regular(shown))?;
        let name = canonical
            .file_name()
            .ok_or_else(|| super::not_regular(shown))?;
        let flags = libc::O_PATH | libc::O_NOFOLLOW | libc::O_DIRECTORY;
        #[cfg(test)]
        tree.note_open(flags.cast_unsigned());
        let parent = OpenOptions::new()
            .read(true)
            .custom_flags(flags)
            .open(directory)
            .map_err(|e| open_error(shown, e))?;
        Some((parent, name.to_owned()))
    } else {
        None
    };
    let handle = Handle {
        reference,
        metadata,
        parent,
    };
    handle.recheck(tree, shown)?;
    Ok(handle)
}

impl Handle {
    /// Fresh metadata from the already verified reference.
    pub(super) fn metadata(&self, shown: &str) -> Result<Metadata, TreeError> {
        self.reference
            .metadata()
            .map_err(|e| TreeError::from_io(shown, e))
    }

    /// Read the lexical link below the pinned directory, never its destination.
    pub(super) fn link(
        &self,
        name: &str,
        shown: &str,
        between: impl FnOnce(),
    ) -> Result<(FileId, PathBuf), TreeError> {
        let path = descriptor_path(&self.reference).join(name);
        let before = std::fs::symlink_metadata(&path).map_err(|e| TreeError::from_io(shown, e))?;
        if !before.file_type().is_symlink() {
            return Err(super::not_regular(shown));
        }
        let target = std::fs::read_link(&path).map_err(|e| TreeError::from_io(shown, e))?;
        between();
        let repeated = std::fs::read_link(&path).map_err(|e| TreeError::from_io(shown, e))?;
        let after = std::fs::symlink_metadata(&path).map_err(|e| TreeError::from_io(shown, e))?;
        if !after.file_type().is_symlink()
            || device_identity(&before) != device_identity(&after)
            || before.permissions() != after.permissions()
            || target != repeated
        {
            return Err(super::capture::changed(shown));
        }
        Ok((device_identity(&after), target))
    }

    /// The device and inode of the reference.
    #[expect(
        clippy::unnecessary_wraps,
        reason = "the signature is shared with Windows, where the identity can fail"
    )]
    pub(super) fn identity(&self, _root: &Path, _shown: &str) -> Result<FileId, TreeError> {
        Ok(device_identity(&self.metadata))
    }

    /// The pinned object must retain its name and original-root ancestry.
    pub(super) fn recheck(&self, tree: &DiskTree, shown: &str) -> Result<(), TreeError> {
        verify_linux_link(&tree.root, &descriptor_path(&self.reference), shown)?;
        let start = match &self.parent {
            Some((parent, name)) => {
                let found = statat(parent, name.as_os_str(), AtFlags::SYMLINK_NOFOLLOW)
                    .map_err(|_| super::outside(shown))?;
                let held = fstat(&self.reference).map_err(|_| super::outside(shown))?;
                if !super::unix_root::same(&found, &held) {
                    return Err(super::outside(shown));
                }
                parent.try_clone()
            }
            None => self.reference.try_clone(),
        }
        .map_err(|e| TreeError::from_io(shown, e))?;
        super::unix_root::climb(start.into(), &tree.root_directory, shown)
    }

    /// Open the verified file through its descriptor path and read at most
    /// `max_bytes` bytes.
    pub(super) fn read(&self, max_bytes: u64, shown: &str) -> Result<Vec<u8>, TreeError> {
        let file = File::open(descriptor_path(&self.reference))
            .map_err(|e| TreeError::from_io(shown, e))?;
        read_capped(file, max_bytes, shown)
    }

    /// List the verified directory through its descriptor path.
    pub(super) fn entries(&self, shown: &str, max_entries: usize) -> Result<Vec<Entry>, TreeError> {
        read_entries(&descriptor_path(&self.reference), shown, max_entries)
    }

    /// Look `name` up below the verified directory's descriptor path.
    pub(super) fn lookup(&self, name: &str, shown: &str) -> Result<Option<EntryKind>, TreeError> {
        lookup(&descriptor_path(&self.reference).join(name), shown)
    }
}
