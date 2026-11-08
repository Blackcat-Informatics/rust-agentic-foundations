// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Checked record publication over the existing native readers.

use std::io;
use std::path::{Path, PathBuf};
use std::{collections::BTreeMap, rc::Rc};

use raf_source::{EntryKind, MemoryTree, RelPath, SourceAuthorityId, Tree, TreeError};

use super::RecordError;
use crate::files;
use crate::publication_io::{Root, Target, cleanup_after, with_cleanup};

/// The publication boundary used by every record-writing workflow.
///
/// Inspection refuses symbolic-link or non-directory ancestors. Creation is
/// exclusive and replacement requires a regular file. Each publication checks
/// its destination again; refusal preserves existing files. Implementations
/// must retain their reader's authority rules and original failure causes.
/// The disk adapter retains admitted parent bindings and file snapshots through
/// publication. Checks and renames are separate operations, so callers must not
/// assume isolation from adversarial concurrent publishers.
/// Implementations are supplied explicitly by the caller, never loaded from
/// inspected repository data. RAF's concrete disk adapter is private; filesystem
/// callers publish through the typed record APIs.
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
pub(crate) struct DiskRecordStore {
    root: PathBuf,
    authority: Option<Rc<Root>>,
    targets: BTreeMap<RelPath, Target>,
}

impl std::fmt::Debug for DiskRecordStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiskRecordStore")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

impl DiskRecordStore {
    /// Select the repository root; no file is opened or written.
    #[must_use]
    pub(crate) fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            authority: None,
            targets: BTreeMap::new(),
        }
    }

    fn authority(&mut self) -> io::Result<Rc<Root>> {
        if self.authority.is_none() {
            self.authority = Some(Root::open(&self.root)?);
        }
        Ok(Rc::clone(
            self.authority
                .as_ref()
                .expect("the checked native authority is open"),
        ))
    }

    fn target(&mut self, path: &RelPath) -> Result<Target, RecordError> {
        if let Some(target) = self.targets.remove(path) {
            return Ok(target);
        }
        self.authority()
            .and_then(|root| root.capture(path))
            .map_err(|source| write_error(path, source))
    }
}

fn write_error(path: &RelPath, source: io::Error) -> RecordError {
    RecordError::Write {
        path: path.to_string(),
        source,
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
        let root = self
            .authority()
            .map_err(|source| write_error(path, source))?;
        let directory = path.parent();
        let found = files::target_in(
            root.tree(),
            directory.as_ref().map_or("", RelPath::as_str),
            path,
        )?;
        root.admit_parent(path)
            .map_err(|source| write_error(path, source))?;
        if found.directories
            && matches!(found.kind, None | Some(EntryKind::File))
            && !self.targets.contains_key(path)
        {
            self.targets.insert(
                path.clone(),
                root.capture(path)
                    .map_err(|source| write_error(path, source))?,
            );
        }
        Ok(found.kind)
    }

    fn tree(&mut self) -> Result<&dyn Tree, RecordError> {
        self.authority().map_err(|source| RecordError::Write {
            path: self.root.display().to_string(),
            source,
        })?;
        Ok(self
            .authority
            .as_ref()
            .expect("the checked native reader is open")
            .tree())
    }

    fn create_new(&mut self, path: &RelPath, text: &str) -> Result<(), RecordError> {
        if self.inspect(path)?.is_some() {
            return Err(RecordError::Exists { path: path.clone() });
        }
        let root = self
            .authority()
            .map_err(|source| write_error(path, source))?;
        if let Some(parent) = path.parent() {
            let mut prefix = String::new();
            for component in parent.as_str().split('/').filter(|part| !part.is_empty()) {
                if !prefix.is_empty() {
                    prefix.push('/');
                }
                prefix.push_str(component);
                let relative = RelPath::new(&prefix)
                    .map_err(|source| write_error(path, io::Error::other(source)))?;
                root.create_directory(&relative)
                    .map_err(|source| write_error(path, source))?;
            }
        }
        let target = self.target(path)?;
        if target.exists() {
            return Err(RecordError::Exists { path: path.clone() });
        }
        let staged = target
            .stage(text)
            .map_err(|source| write_error(path, source))?;
        staged.publish().map_err(|source| {
            let cleanup = staged.discard();
            if source.kind() == io::ErrorKind::AlreadyExists && cleanup.is_ok() {
                RecordError::Exists { path: path.clone() }
            } else {
                write_error(path, with_cleanup(source, cleanup))
            }
        })
    }

    fn replace(&mut self, path: &RelPath, text: &str) -> Result<(), RecordError> {
        if self.inspect(path)? != Some(EntryKind::File) {
            return Err(RecordError::Missing { path: path.clone() });
        }
        let staged = self
            .target(path)?
            .stage(text)
            .map_err(|source| write_error(path, source))?;
        staged
            .publish()
            .map_err(|source| write_error(path, cleanup_after(source, &[staged])))
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
