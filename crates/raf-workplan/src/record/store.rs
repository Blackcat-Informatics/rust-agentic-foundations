// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Checked record publication over the existing native readers.

use std::io;
use std::path::{Path, PathBuf};

use raf_source::{DiskTree, EntryKind, MemoryTree, RelPath, SourceAuthorityId, Tree, TreeError};

use super::{RecordError, publish_replacement};
use crate::files::{self, CreateError};

/// The publication boundary used by every record-writing workflow.
///
/// Inspection refuses symbolic-link or non-directory ancestors. Creation is
/// exclusive and replacement requires a regular file. Each publication checks
/// its destination again; refusal preserves existing files. Implementations
/// must retain their reader's authority rules and original failure causes.
/// The disk adapter retains its documented path-based name-swap limitation.
pub trait RecordStore {
    /// The existing native reader, opened only when needed.
    ///
    /// # Errors
    /// Returns the original reader-opening failure without substituting a reader.
    fn tree(&mut self) -> Result<&dyn Tree, RecordError>;

    /// Inspect the destination and every ancestor without publishing anything.
    ///
    /// # Errors
    /// Refuses foreign authorities, links, non-directory ancestors and reader failures.
    fn inspect(&mut self, path: &RelPath) -> Result<Option<EntryKind>, RecordError> {
        first_party(path)?;
        let directory = path.parent();
        Ok(files::target_in(
            self.tree()?,
            directory.as_ref().map_or("", RelPath::as_str),
            path,
        )?
        .kind)
    }

    /// Exclusively publish the complete text, rechecking the destination.
    ///
    /// # Errors
    /// Returns `Exists` for an occupied destination, or the original write refusal.
    fn create_new(&mut self, path: &RelPath, text: &str) -> Result<(), RecordError>;

    /// Replace one existing regular file, rechecking before publication.
    ///
    /// # Errors
    /// Returns `Missing` for a non-file destination, or the original write refusal.
    fn replace(&mut self, path: &RelPath, text: &str) -> Result<(), RecordError>;
}

/// Record publication through the existing disk reader and filesystem writer.
///
/// Construction performs no I/O, so model and text validation keep their
/// precedence over destination inspection. The native reader opens on first use.
/// Inspection retains no directory handles against subsequent name swaps.
#[derive(Debug)]
pub struct DiskRecordStore {
    root: PathBuf,
    tree: Option<DiskTree>,
}

impl DiskRecordStore {
    /// Select the repository root; no file is opened or written.
    #[must_use]
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            tree: None,
        }
    }
}

fn tree_error(path: &RelPath, source: TreeError) -> RecordError {
    RecordError::Write {
        path: path.to_string(),
        source: io::Error::new(io::ErrorKind::InvalidInput, source),
    }
}

fn first_party(path: &RelPath) -> Result<(), RecordError> {
    if path.authority() != SourceAuthorityId::FIRST_PARTY {
        return Err(tree_error(
            path,
            TreeError::WrongAuthority {
                path: path.to_string(),
                authority: path.authority(),
            },
        ));
    }
    Ok(())
}

impl RecordStore for DiskRecordStore {
    fn inspect(&mut self, path: &RelPath) -> Result<Option<EntryKind>, RecordError> {
        first_party(path)?;
        let directory = path.parent();
        Ok(files::target(
            &self.root,
            directory.as_ref().map_or("", RelPath::as_str),
            path,
        )?
        .kind)
    }

    fn tree(&mut self) -> Result<&dyn Tree, RecordError> {
        if self.tree.is_none() {
            self.tree = Some(
                DiskTree::new(&self.root).map_err(|source| RecordError::Write {
                    path: self.root.display().to_string(),
                    source: io::Error::new(io::ErrorKind::InvalidInput, source),
                })?,
            );
        }
        Ok(self
            .tree
            .as_ref()
            .expect("the checked native reader is open"))
    }

    fn create_new(&mut self, path: &RelPath, text: &str) -> Result<(), RecordError> {
        self.inspect(path)?;
        let directory = path.parent();
        files::create_new(
            &self.root,
            directory.as_ref().map_or("", RelPath::as_str),
            path,
            text,
        )
        .map_err(|error| match error {
            CreateError::Exists => RecordError::Exists { path: path.clone() },
            CreateError::Write { path, source } => RecordError::Write { path, source },
        })
    }

    fn replace(&mut self, path: &RelPath, text: &str) -> Result<(), RecordError> {
        if self.inspect(path)? != Some(EntryKind::File) {
            return Err(RecordError::Missing { path: path.clone() });
        }
        publish_replacement(&self.root, path, text).map_err(|source| RecordError::Write {
            path: path.to_string(),
            source,
        })
    }
}

impl RecordStore for MemoryTree {
    fn tree(&mut self) -> Result<&dyn Tree, RecordError> {
        Ok(self)
    }

    fn create_new(&mut self, path: &RelPath, text: &str) -> Result<(), RecordError> {
        if self.inspect(path)?.is_some() {
            return Err(RecordError::Exists { path: path.clone() });
        }
        MemoryTree::create_new(self, path, text.to_owned())
            .map_err(|source| tree_error(path, source))
    }

    fn replace(&mut self, path: &RelPath, text: &str) -> Result<(), RecordError> {
        if self.inspect(path)? != Some(EntryKind::File) {
            return Err(RecordError::Missing { path: path.clone() });
        }
        MemoryTree::replace(self, path, text.to_owned()).map_err(|source| tree_error(path, source))
    }
}
