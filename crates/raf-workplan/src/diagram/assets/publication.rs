// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Asset-specific admission around the existing native sync staging writer.

use raf_source::{DiskTree, EntryKind, RelPath, Tree};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use super::{MANIFEST, read};
use crate::diagram::semantic::hash;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Guard {
    pub path: RelPath,
    pub before: Option<String>,
    pub after: Option<String>,
}

/// Opaque ownership admission produced by the shared diagram computation.
///
/// The public sync report may carry this value but callers cannot forge its
/// admitted paths. The ordinary sync writer checks every byte and destination
/// before publishing the ownership intent and generated files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Publication {
    pub(crate) original_manifest: Option<String>,
    pub(crate) intent: String,
    pub(crate) manifest: String,
    pub(crate) guards: Vec<Guard>,
    pub(crate) removals: Vec<Guard>,
}

pub(crate) fn reserved(path: &RelPath) -> bool {
    path.as_str().starts_with("docs/diagrams/")
}

fn refused(path: &RelPath, reason: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("diagram publication refuses {path}: {reason}"),
    )
}

impl Publication {
    pub(crate) fn removed(&self) -> impl Iterator<Item = &RelPath> {
        self.removals.iter().map(|guard| &guard.path)
    }
    pub(crate) fn admits(&self, path: &RelPath) -> bool {
        path.as_str() == MANIFEST || self.guards.iter().any(|guard| guard.path == *path)
    }

    pub(crate) fn validate(&self, root: &Path, edits: &[(RelPath, String)]) -> io::Result<()> {
        let tree = DiskTree::new(root).map_err(io::Error::other)?;
        let manifest = RelPath::new(MANIFEST).map_err(io::Error::other)?;
        if read(&tree, &manifest)
            .map_err(io::Error::other)?
            .as_deref()
            .map(hash)
            != self.original_manifest
        {
            return Err(refused(
                &manifest,
                "ownership manifest changed since synchronization was computed",
            ));
        }
        for (path, text) in edits.iter().filter(|(path, _)| reserved(path)) {
            let expected = if path.as_str() == MANIFEST {
                Some(hash(&self.manifest))
            } else {
                self.guards
                    .iter()
                    .find(|guard| guard.path == *path)
                    .and_then(|guard| guard.after.clone())
            };
            if expected != Some(hash(text)) {
                return Err(refused(
                    path,
                    "bytes or destination are not in the computed ownership admission",
                ));
            }
        }
        if !edits
            .iter()
            .any(|(path, text)| path.as_str() == MANIFEST && *text == self.manifest)
        {
            return Err(refused(
                &manifest,
                "the final ownership manifest is absent from the report",
            ));
        }
        for guard in self.guards.iter().chain(&self.removals) {
            if read(&tree, &guard.path)
                .map_err(io::Error::other)?
                .as_deref()
                .map(hash)
                != guard.before
            {
                return Err(refused(
                    &guard.path,
                    "asset changed since synchronization was computed",
                ));
            }
            if guard.after.is_some() && !edits.iter().any(|(path, _)| *path == guard.path) {
                return Err(refused(
                    &guard.path,
                    "a required owned edit was removed from the report",
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn directories(
        root: &Path,
        edits: &[(RelPath, String)],
    ) -> io::Result<Vec<(RelPath, fs::Permissions)>> {
        let mut created = Vec::new();
        let tree = DiskTree::new(root).map_err(io::Error::other)?;
        for (path, _) in edits.iter().filter(|(path, _)| reserved(path)) {
            let parent = path
                .parent()
                .ok_or_else(|| refused(path, "missing parent"))?;
            for part in parent
                .as_str()
                .split('/')
                .scan(String::new(), |prefix, part| {
                    if !prefix.is_empty() {
                        prefix.push('/');
                    }
                    prefix.push_str(part);
                    Some(prefix.clone())
                })
            {
                let relative = RelPath::new(&part).map_err(io::Error::other)?;
                match tree.entry(&relative).map_err(io::Error::other)? {
                    Some(EntryKind::Dir) => {}
                    None if part.starts_with("docs/diagrams") => {
                        match fs::create_dir(root.join(&part)) {
                            Ok(()) => {
                                let observed = tree
                                    .capture_directory_bounded(&relative, 0)
                                    .map_err(io::Error::other)?;
                                created.push((relative.clone(), observed.permissions));
                            }
                            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                                if tree.entry(&relative).map_err(io::Error::other)?
                                    != Some(EntryKind::Dir)
                                {
                                    return Err(refused(
                                        path,
                                        "a generated directory was replaced by a non-directory",
                                    ));
                                }
                            }
                            Err(error) => return Err(error),
                        }
                    }
                    _ => return Err(refused(path, "destination crosses a link or non-directory")),
                }
            }
        }
        Ok(created)
    }

    pub(crate) fn stage_intent(&self, root: &Path) -> io::Result<PathBuf> {
        crate::sync::stage(&root.join(MANIFEST), &self.intent)
    }

    pub(crate) fn retire(&self, root: &Path) -> io::Result<()> {
        let tree = DiskTree::new(root).map_err(io::Error::other)?;
        let manifest = RelPath::new(MANIFEST).map_err(io::Error::other)?;
        if read(&tree, &manifest)
            .map_err(io::Error::other)?
            .as_deref()
            .map(hash)
            != Some(hash(&self.intent))
        {
            return Err(refused(
                &manifest,
                "ownership intent changed before retired assets could be removed",
            ));
        }
        for guard in &self.removals {
            if read(&tree, &guard.path)
                .map_err(io::Error::other)?
                .as_deref()
                .map(hash)
                != guard.before
            {
                return Err(refused(&guard.path, "retired asset changed before removal"));
            }
            fs::remove_file(root.join(guard.path.as_str()))?;
        }
        Ok(())
    }
}
