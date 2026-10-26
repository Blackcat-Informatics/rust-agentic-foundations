// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Linux: a candidate is first taken as an `O_PATH` reference, which performs
//! no I/O: it runs no device's open handler, completes no FIFO rendezvous and
//! reads nothing. The kernel's own path for the reference must lie under the
//! root and its type must be the one wanted. Only then is it opened for
//! reading or listed, through its descriptor path, which names the inode the
//! reference holds and is never resolved by name again. An outside candidate
//! is therefore never opened for reading or listing.

use std::fs::{File, Metadata, OpenOptions};
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::Path;

use super::{
    DiskTree, Entry, EntryKind, Want, descriptor_path, device_identity, lookup, open_error,
    read_capped, read_entries, refuse_unless_directory, verify_linux_link,
};
use crate::error::TreeError;
use crate::path::RelPath;
use crate::tree::FileId;

/// A verified `O_PATH` reference.
pub(super) struct Handle {
    reference: File,
    metadata: Metadata,
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
    Ok(Handle {
        reference,
        metadata,
    })
}

impl Handle {
    /// The device and inode of the reference.
    #[expect(
        clippy::unnecessary_wraps,
        reason = "the signature is shared with Windows, where the identity can fail"
    )]
    pub(super) fn identity(&self, _root: &Path, _shown: &str) -> Result<FileId, TreeError> {
        Ok(device_identity(&self.metadata))
    }

    /// The kernel's path for the reference must still lie under `root`.
    pub(super) fn recheck(&self, root: &Path, shown: &str) -> Result<(), TreeError> {
        verify_linux_link(root, &descriptor_path(&self.reference), shown)
    }

    /// Open the verified file through its descriptor path and read at most
    /// `max_bytes` bytes.
    pub(super) fn read(&self, max_bytes: u64, shown: &str) -> Result<Vec<u8>, TreeError> {
        let file = File::open(descriptor_path(&self.reference))
            .map_err(|e| TreeError::from_io(shown, e))?;
        read_capped(file, max_bytes, shown)
    }

    /// List the verified directory through its descriptor path.
    pub(super) fn entries(&self, shown: &str) -> Result<Vec<Entry>, TreeError> {
        read_entries(&descriptor_path(&self.reference), shown)
    }

    /// Look `name` up below the verified directory's descriptor path.
    pub(super) fn lookup(&self, name: &str, shown: &str) -> Result<Option<EntryKind>, TreeError> {
        lookup(&descriptor_path(&self.reference).join(name), shown)
    }
}
