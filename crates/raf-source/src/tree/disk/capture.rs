// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Checked host captures retain the logical path and actual host identity.

use std::fs::Permissions;
use std::path::PathBuf;

use purrdf_lex::walk::try_sort_unstable_by;

use super::{DiskTree, Entry, FileId, RelPath, TreeError, Want};

/// A regular file's bytes and permissions taken through one verified handle.
///
/// This is an observation, not an immutable filesystem snapshot. A caller
/// preparing a later publication must compare the source again before publishing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedFile {
    /// The requested logical path, including its source authority.
    pub path: RelPath,
    /// The original host identity; a copied file must use its own identity.
    pub id: FileId,
    /// Exact bounded bytes, including non-UTF-8 contents.
    pub bytes: Vec<u8>,
    /// Native permissions, including Unix executable bits.
    pub permissions: Permissions,
}

/// A directory's sorted entries and identity from one verified directory handle.
///
/// Retaining this observation lets a later comparison detect added or removed
/// names as well as changes to the files that were originally present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedDirectory {
    /// The requested logical path, including its source authority.
    pub path: RelPath,
    /// The original directory's host identity.
    pub id: FileId,
    /// All entries, including links and unsupported special files.
    pub entries: Vec<Entry>,
    /// Native directory permissions needed to preserve access semantics.
    pub permissions: Permissions,
}

/// A lexical symbolic link, captured without opening or reading its target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedLink {
    /// The requested link path, including its source authority.
    pub path: RelPath,
    /// The link's identity, rather than its destination's identity.
    pub id: FileId,
    /// The native target spelling, without UTF-8 conversion or normalization.
    pub target: PathBuf,
}

impl DiskTree {
    /// Capture bytes, identity and permissions through one verified regular handle.
    ///
    /// The type and permissions are checked again after reading. Symlinks are
    /// followed under the same containment rules as [`Self::read_bytes`]. Ownership,
    /// ACLs and extended attributes are not captured by this operation.
    ///
    /// # Errors
    /// The errors of [`Self::read_bytes`], or an interrupted observation when
    /// the file's type or permissions change during the read.
    pub fn capture_file(&self, path: &RelPath, max_bytes: u64) -> Result<CapturedFile, TreeError> {
        super::super::require_first_party(path)?;
        let shown = path.to_string();
        let handle = self.open(path, Want::File, &shown)?;
        self.used(&handle, &shown, |handle| {
            let before = handle.metadata(&shown)?;
            let id = handle.identity(&self.root, &shown)?;
            let bytes = handle.read(max_bytes, &shown)?;
            #[cfg(test)]
            self.observed();
            let after = handle.metadata(&shown)?;
            if !after.is_file() || before.permissions() != after.permissions() {
                return Err(changed(&shown));
            }
            Ok(CapturedFile {
                path: path.clone(),
                id,
                bytes,
                permissions: after.permissions(),
            })
        })
    }

    /// Capture a complete sorted listing and the verified directory's identity.
    ///
    /// # Errors
    /// The errors of [`super::Tree::list`]. A directory symlink is refused.
    pub fn capture_directory(&self, path: &RelPath) -> Result<CapturedDirectory, TreeError> {
        super::super::require_first_party(path)?;
        let shown = path.to_string();
        let handle = self.open(path, Want::Listing, &shown)?;
        self.used(&handle, &shown, |handle| {
            let before = handle.metadata(&shown)?;
            let id = handle.identity(&self.root, &shown)?;
            let mut entries = handle.entries(&shown)?;
            try_sort_unstable_by(&mut entries, |left, right| {
                Ok::<_, TreeError>(left.cmp(right))
            })?;
            #[cfg(test)]
            self.observed();
            let mut repeated = handle.entries(&shown)?;
            try_sort_unstable_by(&mut repeated, |left, right| {
                Ok::<_, TreeError>(left.cmp(right))
            })?;
            let after = handle.metadata(&shown)?;
            if !after.is_dir() || before.permissions() != after.permissions() || entries != repeated
            {
                return Err(changed(&shown));
            }
            Ok(CapturedDirectory {
                path: path.clone(),
                id,
                entries,
                permissions: after.permissions(),
            })
        })
    }

    /// Capture a link through its verified containing-directory handle.
    ///
    /// The target is not followed, so dangling and outside-pointing links can
    /// be described safely. This does not authorize materializing or following
    /// them elsewhere. Native Unix captures compare the link identity and exact
    /// target spelling before and after reading; other hosts refuse explicitly.
    ///
    /// # Errors
    /// Foreign authorities, outside parents, a non-link, a changed link, an
    /// unsupported host or a failed host operation refuse without a capture.
    pub fn capture_link(&self, path: &RelPath) -> Result<CapturedLink, TreeError> {
        super::super::require_first_party(path)?;
        let shown = path.to_string();
        let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
            return Err(super::not_regular(&shown));
        };
        if name == ".." {
            return Err(super::not_regular(&shown));
        }
        super::refuse_windows_names(path, &shown)?;
        let handle = self.open(&parent, Want::Dir, &shown)?;
        let (id, target) = self.used(&handle, &shown, |handle| {
            handle.link(name, &shown, || {
                #[cfg(test)]
                self.observed();
            })
        })?;
        Ok(CapturedLink {
            path: path.clone(),
            id,
            target,
        })
    }
}

pub(super) fn changed(shown: &str) -> TreeError {
    TreeError::from_io(
        shown,
        std::io::Error::new(
            std::io::ErrorKind::Interrupted,
            "entry changed during capture; retry from a stable source",
        ),
    )
}
