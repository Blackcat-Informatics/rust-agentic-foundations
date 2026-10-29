// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Independent admission of completed bytes, modes, membership and alias topology.

use std::ffi::OsString;
use std::fs::Permissions;
use std::path::{Component, Path, PathBuf};

use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use purrdf_lex::walk::{WorkList, try_equal_range_by, try_sort_unstable_by};
use raf_source::{CapturedFile, EntryKind, FileId, RelPath};

use super::capture::{Capture, Node};
use super::{CapturedView, ViewError};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Alias {
    Original(FileId),
    Rewritten(RelPath),
}
struct Added {
    parent: RelPath,
    path: RelPath,
    kind: EntryKind,
}
struct CopyCheck<'a> {
    view: &'a CapturedView,
    actual: &'a Capture,
    hidden: RelPath,
    removed: Vec<RelPath>,
    modes: OrderedTable<RelPath, Permissions>,
    bodies: OrderedTable<RelPath, &'a str>,
    original: OrderedTable<FileId, ()>,
    groups: OrderedTable<Alias, FileId>,
    identities: OrderedTable<FileId, Alias>,
    added: Vec<Added>,
}

impl CapturedView {
    pub(super) fn verify_copy(
        &self,
        actual: &Capture,
        modes: &[(RelPath, Permissions)],
    ) -> Result<(), ViewError> {
        let mut admission = Resident;
        let mut memory = Memory::new(&mut admission);
        let mut check = CopyCheck::new(self, actual, modes, &mut memory)?;
        let mut count = 0;
        for (path, expected) in &self.capture.rows {
            if *path == check.hidden || check.removed.binary_search(path).is_ok() {
                continue;
            }
            count += 1;
            check.existing(path, expected, &mut memory)?;
        }
        for index in 0..check.added.len() {
            let path = check.added[index].path.clone();
            if check.added[index].kind == EntryKind::Dir {
                let Some(Node::Dir(copy)) = actual.get(&path) else {
                    return Err(mismatch(&path, "missing generated directory"));
                };
                check.detached(&path, &copy.id)?;
                if check.modes.get(&path) != Some(&copy.permissions) {
                    return Err(mismatch(
                        &path,
                        "generated directory mode differs from native creation receipt",
                    ));
                }
                check.generated_directory(&path, copy)?;
                count += 1;
                continue;
            }
            let text = *check
                .bodies
                .get(&path)
                .ok_or_else(|| mismatch(&path, "missing generated body"))?;
            let permissions = check
                .modes
                .get(&path)
                .ok_or_else(|| mismatch(&path, "missing native creation mode"))?
                .clone();
            let Some(Node::File(copy)) = actual.get(&path) else {
                return Err(mismatch(&path, "missing generated addition"));
            };
            count += 1;
            check.new_file(&path, text.as_bytes(), &permissions, copy, &mut memory)?;
        }
        if count != actual.rows.len() {
            return Err(ViewError::refusal(
                "admit completed copy",
                "copied view has unrelated or omitted entries",
            ));
        }
        Ok(())
    }
}
impl<'a> CopyCheck<'a> {
    fn new(
        view: &'a CapturedView,
        actual: &'a Capture,
        receipts: &[(RelPath, Permissions)],
        memory: &mut Memory<'_, Resident>,
    ) -> Result<Self, ViewError> {
        let hidden = view
            .capture
            .prefix
            .join(view.request.hidden.as_str())
            .map_err(|e| ViewError::new("completed removal path", e))?;
        let mut check = Self {
            view,
            actual,
            hidden,
            removed: view
                .request
                .removed
                .iter()
                .map(|path| view.capture.prefix.join(path.as_str()))
                .collect::<Result<_, _>>()
                .map_err(|e| ViewError::new("completed retired assets", e))?,
            modes: OrderedTable::default(),
            bodies: OrderedTable::default(),
            original: OrderedTable::default(),
            groups: OrderedTable::default(),
            identities: OrderedTable::default(),
            added: Vec::new(),
        };
        for (path, permissions) in receipts {
            let path = view
                .capture
                .prefix
                .join(path.as_str())
                .map_err(|e| ViewError::new("rewrite receipt path", e))?;
            if check
                .modes
                .insert(path, permissions.clone(), memory)
                .map_err(|e| ViewError::new("index rewrite modes", e))?
                .is_some()
            {
                return Err(ViewError::refusal(
                    "admit rewrite receipt",
                    "duplicate target",
                ));
            }
        }
        for (path, body) in &view.request.rewritten {
            let path = view
                .capture
                .prefix
                .join(path.as_str())
                .map_err(|e| ViewError::new("completed addition", e))?;
            if check
                .bodies
                .insert(path.clone(), body.as_str(), memory)
                .map_err(|e| ViewError::new("index completed bodies", e))?
                .is_some()
            {
                return Err(mismatch(&path, "duplicate generated request"));
            }
            if check.modes.get(&path).is_none() {
                return Err(mismatch(&path, "requested edit has no native mode receipt"));
            }
            if view.capture.get(&path).is_none() {
                let parent = path
                    .parent()
                    .ok_or_else(|| mismatch(&path, "generated addition has no parent"))?;
                check.added.push(Added {
                    parent,
                    path,
                    kind: EntryKind::File,
                });
            }
        }
        let directories = check.admit_directories(memory)?;
        if receipts.len() != view.request.rewritten.len() + directories {
            return Err(ViewError::refusal(
                "admit rewrite receipts",
                "native publication did not cover exactly requested files and their missing directories",
            ));
        }
        try_sort_unstable_by(&mut check.added, |a, b| {
            Ok::<_, ViewError>((&a.parent, &a.path).cmp(&(&b.parent, &b.path)))
        })?;
        for (_, node) in &view.capture.rows {
            let id = node_id(node);
            if check.original.get(id).is_none() {
                check
                    .original
                    .insert(id.clone(), (), memory)
                    .map_err(|e| ViewError::new("index original physical identities", e))?;
            }
        }
        Ok(check)
    }
    fn admit_directories(&mut self, memory: &mut Memory<'_, Resident>) -> Result<usize, ViewError> {
        // Only missing ancestors of the requested writes may be new directories.
        let mut directories = OrderedTable::default();
        for (path, _) in &self.view.request.rewritten {
            let mut parent = self
                .view
                .capture
                .prefix
                .join(path.as_str())
                .map_err(|e| ViewError::new("generated directory path", e))?
                .parent();
            while let Some(path) = parent {
                if self.view.capture.get(&path).is_some() {
                    break;
                }
                if directories.get(&path).is_none() {
                    directories
                        .insert(path.clone(), (), memory)
                        .map_err(|e| ViewError::new("index generated directories", e))?;
                    if self.modes.get(&path).is_none() {
                        return Err(mismatch(
                            &path,
                            "new directory has no native creation receipt",
                        ));
                    }
                    self.added.push(Added {
                        parent: path
                            .parent()
                            .ok_or_else(|| mismatch(&path, "generated directory has no parent"))?,
                        path: path.clone(),
                        kind: EntryKind::Dir,
                    });
                }
                parent = path.parent();
            }
        }
        Ok(directories.len())
    }
    fn existing(
        &mut self,
        path: &RelPath,
        expected: &Node,
        memory: &mut Memory<'_, Resident>,
    ) -> Result<(), ViewError> {
        let actual = self.actual;
        let copy = actual
            .get(path)
            .ok_or_else(|| mismatch(path, "missing selected entry"))?;
        self.detached(path, node_id(copy))?;
        match (expected, copy) {
            (Node::File(original), Node::File(copy)) => {
                let rewrite = self.bodies.get(path);
                let bytes = rewrite.map_or(original.bytes.as_slice(), |text| text.as_bytes());
                if bytes != copy.bytes || original.permissions != copy.permissions {
                    return Err(mismatch(path, "file bytes or permissions changed"));
                }
                if rewrite.is_some() && self.modes.get(path) != Some(&original.permissions) {
                    return Err(mismatch(path, "existing-file publication mode changed"));
                }
                let group = if rewrite.is_some() {
                    Alias::Rewritten(path.clone())
                } else {
                    Alias::Original(original.id.clone())
                };
                self.alias(path, group, &copy.id, memory)
            }
            (Node::Dir(original), Node::Dir(copy)) => self.directory(path, original, copy),
            (Node::Link(original), Node::Link(copy)) => {
                if self
                    .view
                    .relocated_link(&actual.root, path, &original.target)?
                    != copy.target
                {
                    return Err(mismatch(path, "lexical link target changed"));
                }
                Ok(())
            }
            _ => Err(mismatch(path, "entry kind changed")),
        }
    }
    fn directory(
        &self,
        path: &RelPath,
        original: &raf_source::CapturedDirectory,
        copy: &raf_source::CapturedDirectory,
    ) -> Result<(), ViewError> {
        let mut entries = Vec::new();
        for entry in &original.entries {
            let child = path
                .join(&entry.name)
                .map_err(|e| ViewError::new("completed member path", e))?;
            if child != self.hidden && self.removed.binary_search(&child).is_err() {
                entries.push((entry.name.as_str(), entry.kind));
            }
        }
        let range = try_equal_range_by(&self.added, |entry| {
            Ok::<_, ViewError>(entry.parent.cmp(path))
        })?;
        for entry in &self.added[range] {
            entries.push((
                entry
                    .path
                    .file_name()
                    .ok_or_else(|| mismatch(&entry.path, "addition has no name"))?,
                entry.kind,
            ));
        }
        try_sort_unstable_by(&mut entries, |a, b| Ok::<_, ViewError>(a.cmp(b)))?;
        if original.permissions != copy.permissions
            || entries.len() != copy.entries.len()
            || !entries
                .iter()
                .zip(&copy.entries)
                .all(|((name, kind), entry)| *name == entry.name && *kind == entry.kind)
        {
            return Err(mismatch(
                path,
                "directory mode or completed membership changed",
            ));
        }
        Ok(())
    }
    fn generated_directory(
        &self,
        path: &RelPath,
        copy: &raf_source::CapturedDirectory,
    ) -> Result<(), ViewError> {
        let range = try_equal_range_by(&self.added, |entry| {
            Ok::<_, ViewError>(entry.parent.cmp(path))
        })?;
        let mut expected: Vec<_> = self.added[range]
            .iter()
            .map(|entry| (entry.path.file_name(), entry.kind))
            .collect();
        try_sort_unstable_by(&mut expected, |a, b| Ok::<_, ViewError>(a.cmp(b)))?;
        if expected.len() != copy.entries.len()
            || !expected
                .iter()
                .zip(&copy.entries)
                .all(|((name, kind), entry)| {
                    *name == Some(entry.name.as_str()) && *kind == entry.kind
                })
        {
            return Err(mismatch(
                path,
                "generated directory has unrelated or missing members",
            ));
        }
        Ok(())
    }
    fn new_file(
        &mut self,
        path: &RelPath,
        bytes: &[u8],
        mode: &Permissions,
        copy: &CapturedFile,
        memory: &mut Memory<'_, Resident>,
    ) -> Result<(), ViewError> {
        self.detached(path, &copy.id)?;
        if bytes != copy.bytes || *mode != copy.permissions {
            return Err(mismatch(
                path,
                "generated bytes or native creation mode changed",
            ));
        }
        if self.identities.get(&copy.id).is_some() {
            return Err(mismatch(
                path,
                "new generated file aliases another copied input",
            ));
        }
        self.alias(path, Alias::Rewritten(path.clone()), &copy.id, memory)
    }
    fn detached(&self, path: &RelPath, id: &FileId) -> Result<(), ViewError> {
        if self.original.get(id).is_some() {
            Err(mismatch(path, "copy aliases an original selected object"))
        } else {
            Ok(())
        }
    }
    fn alias(
        &mut self,
        path: &RelPath,
        group: Alias,
        id: &FileId,
        memory: &mut Memory<'_, Resident>,
    ) -> Result<(), ViewError> {
        if self
            .groups
            .get(&group)
            .is_some_and(|previous| previous != id)
            || self
                .identities
                .get(id)
                .is_some_and(|previous| *previous != group)
        {
            return Err(mismatch(path, "hardlink equivalence changed"));
        }
        if self.groups.get(&group).is_none() {
            self.groups
                .insert(group.clone(), id.clone(), memory)
                .map_err(|e| ViewError::new("index completed hardlink classes", e))?;
            self.identities
                .insert(id.clone(), group, memory)
                .map_err(|e| ViewError::new("index completed physical identities", e))?;
        }
        Ok(())
    }
}
fn node_id(node: &Node) -> &FileId {
    match node {
        Node::File(value) => &value.id,
        Node::Dir(value) => &value.id,
        Node::Link(value) => &value.id,
    }
}

impl CapturedView {
    pub(super) fn relocated_link(
        &self,
        destination: &Path,
        path: &RelPath,
        target: &Path,
    ) -> Result<PathBuf, ViewError> {
        self.admit_link(path, target)?;
        if target.is_absolute() {
            let relative = target
                .strip_prefix(&self.capture.root)
                .map_err(|e| ViewError::new("link escapes captured repository", e))?;
            Ok(destination.join(relative))
        } else {
            Ok(target.to_path_buf())
        }
    }

    fn admit_link(&self, path: &RelPath, target: &Path) -> Result<(), ViewError> {
        let mut resolved: Vec<OsString> = path
            .parent()
            .into_iter()
            .flat_map(|path| path.segments().map(OsString::from).collect::<Vec<_>>())
            .collect();
        let mut pending: WorkList<OsString, 16> = WorkList::new();
        self.link_parts(target, &mut resolved, &mut pending)?;
        let mut links = 0;
        while let Some(part) = pending.pop() {
            if part == "." {
                continue;
            }
            if part == ".." {
                if resolved.pop().is_none() {
                    return Err(mismatch(path, "link resolves outside captured repository"));
                }
                continue;
            }
            resolved.push(part);
            let candidate: PathBuf = resolved.iter().collect();
            if self.capture.excludes(&self.capture.root.join(&candidate)) {
                return Err(mismatch(
                    path,
                    "selected link depends on an excluded repository input",
                ));
            }
            let node = if let Some(value) = candidate.to_str() {
                let path =
                    RelPath::new(value).map_err(|e| ViewError::new("resolve captured link", e))?;
                self.capture.get(&path)
            } else {
                None
            };
            if let Some(Node::Link(link)) = node {
                links += 1;
                if links > 40 {
                    return Err(mismatch(path, "symbolic link cycle"));
                }
                resolved.pop();
                self.link_parts(&link.target, &mut resolved, &mut pending)?;
            }
        }
        Ok(())
    }

    fn link_parts(
        &self,
        target: &Path,
        resolved: &mut Vec<OsString>,
        pending: &mut WorkList<OsString, 16>,
    ) -> Result<(), ViewError> {
        let target = if target.is_absolute() {
            resolved.clear();
            target
                .strip_prefix(&self.capture.root)
                .map_err(|e| ViewError::new("link leaves captured root", e))?
        } else {
            target
        };
        let parts: Vec<_> = target
            .components()
            .map(|part| match part {
                Component::Normal(value) => Ok(value.to_os_string()),
                Component::CurDir => Ok(OsString::from(".")),
                Component::ParentDir => Ok(OsString::from("..")),
                _ => Err(ViewError::refusal(
                    "admit link",
                    "unsupported native path component",
                )),
            })
            .collect::<Result<_, _>>()?;
        pending.extend(parts.into_iter().rev());
        Ok(())
    }
}
fn mismatch(path: &RelPath, reason: &str) -> ViewError {
    ViewError::refusal("admit completed copy", format!("{path}: {reason}"))
}
