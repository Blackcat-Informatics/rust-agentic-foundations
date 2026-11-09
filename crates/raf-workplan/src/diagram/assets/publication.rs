// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Asset-specific admission around the existing native sync staging writer.

use raf_source::{EntryKind, RelPath, Tree};
use std::{collections::BTreeMap, fs, io, rc::Rc};

use super::{MANIFEST, read};
use crate::diagram::semantic::hash;
use crate::publication_io::{Root, Staged, Target};

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

    pub(crate) fn validate(
        &self,
        root: &Rc<Root>,
        edits: &[(RelPath, String)],
    ) -> io::Result<BTreeMap<RelPath, Target>> {
        let tree = root.tree();
        let manifest = RelPath::new(MANIFEST).map_err(io::Error::other)?;
        if read(tree, &manifest)
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
            if read(tree, &guard.path)
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
        let mut targets = BTreeMap::new();
        for (path, expected) in std::iter::once((&manifest, self.original_manifest.as_deref()))
            .chain(
                self.guards
                    .iter()
                    .chain(&self.removals)
                    .map(|guard| (&guard.path, guard.before.as_deref())),
            )
        {
            let parent = path
                .parent()
                .ok_or_else(|| refused(path, "missing parent"))?;
            let found = crate::files::target_in(tree, parent.as_str(), path)
                .map_err(|error| error.source)?;
            root.admit_parent(path)?;
            if found.directories {
                let target = root.capture(path)?;
                target.verify_content(expected)?;
                targets.insert(path.clone(), target);
            }
        }
        Ok(targets)
    }

    pub(crate) fn directories(
        root: &Rc<Root>,
        edits: &[(RelPath, String)],
    ) -> io::Result<Vec<(RelPath, fs::Permissions)>> {
        let mut created = Vec::new();
        let tree = root.tree();
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
                        match root.create_directory(&relative) {
                            Ok(true) => {
                                let observed = tree
                                    .capture_directory_bounded(&relative, 0)
                                    .map_err(io::Error::other)?;
                                created.push((relative.clone(), observed.permissions));
                            }
                            Ok(false) => {}
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

    pub(crate) fn verify_target(&self, path: &RelPath, target: &Target) -> io::Result<()> {
        if path.as_str() == MANIFEST {
            return target.verify_content(self.original_manifest.as_deref());
        }
        if let Some(guard) = self.guards.iter().find(|guard| guard.path == *path) {
            target.verify_content(guard.before.as_deref())?;
        }
        Ok(())
    }

    pub(crate) fn stage_intent(
        &self,
        root: &Rc<Root>,
        grant: &mut crate::publication_io::StagingGrant,
    ) -> io::Result<Staged> {
        let manifest = RelPath::new(MANIFEST).map_err(io::Error::other)?;
        let target = root.capture(&manifest)?;
        target.verify_content(self.original_manifest.as_deref())?;
        target.stage_reserved(&self.intent, grant)
    }

    pub(crate) fn retire(
        &self,
        root: &Rc<Root>,
        targets: &mut BTreeMap<RelPath, Target>,
    ) -> io::Result<()> {
        let manifest = RelPath::new(MANIFEST).map_err(io::Error::other)?;
        root.capture(&manifest)?
            .verify_content(Some(&hash(&self.intent)))?;
        for guard in &self.removals {
            let target = targets.remove(&guard.path).ok_or_else(|| {
                refused(
                    &guard.path,
                    "retirement has no retained destination admission",
                )
            })?;
            target.verify_content(guard.before.as_deref())?;
            target.remove()?;
        }
        Ok(())
    }
}
