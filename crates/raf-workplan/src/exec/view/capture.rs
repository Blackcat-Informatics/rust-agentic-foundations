// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! RAF selection over checked native host observations; never an ignore-rule walk.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use purrdf_hash::frame::frame_le_into as part;
use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use purrdf_lex::walk::{WorkList, try_equal_range_by, try_sort_unstable_by};
use raf_source::{
    CapturedDirectory, CapturedFile, CapturedLink, DiskTree, EntryKind, RelPath, Tree,
};

use super::{ViewError, ViewRequest};

pub(super) const MAX_ENTRIES: usize = 262_144;
pub(super) const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_BYTES: usize = 512 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Node {
    File(Arc<CapturedFile>),
    Dir(Arc<CapturedDirectory>),
    Link(Arc<CapturedLink>),
}

impl Node {
    fn kind(&self) -> &'static str {
        match self {
            Self::File(_) => "file",
            Self::Dir(_) => "directory",
            Self::Link(_) => "link",
        }
    }

    fn difference(&self, actual: &Self) -> Option<&'static str> {
        match (self, actual) {
            (Self::File(expected), Self::File(actual)) => [
                (expected.path != actual.path, "logical path"),
                (expected.id != actual.id, "identity"),
                (expected.bytes != actual.bytes, "bytes"),
                (expected.permissions != actual.permissions, "permissions"),
            ]
            .into_iter()
            .find_map(|(changed, field)| changed.then_some(field)),
            (Self::Dir(expected), Self::Dir(actual)) => [
                (expected.path != actual.path, "logical path"),
                (expected.id != actual.id, "identity"),
                (expected.entries != actual.entries, "membership"),
                (expected.permissions != actual.permissions, "permissions"),
            ]
            .into_iter()
            .find_map(|(changed, field)| changed.then_some(field)),
            (Self::Link(expected), Self::Link(actual)) => [
                (expected.path != actual.path, "logical path"),
                (expected.id != actual.id, "identity"),
                (expected.target != actual.target, "target"),
            ]
            .into_iter()
            .find_map(|(changed, field)| changed.then_some(field)),
            _ => Some("entry kind"),
        }
    }
}

/// The original installed-Git and Cargo observations admitting exclusions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Selection {
    pub top: PathBuf,
    pub common: PathBuf,
    pub admin: PathBuf,
    pub index_path: PathBuf,
    pub index: String,
    pub original_index: Option<Arc<CapturedFile>>,
    pub object_directories: Arc<[PathBuf]>,
    pub head: String,
    pub default_ref: String,
    pub default_commit: String,
    pub merge_base: String,
    pub tracked: Vec<RelPath>,
    pub worktrees: Vec<PathBuf>,
    pub outputs: Vec<PathBuf>,
    pub workspaces: Vec<RelPath>,
    pub cargo_inputs: Vec<RelPath>,
}

impl Selection {
    fn difference(&self, actual: &Self) -> Option<&'static str> {
        let Self {
            top,
            common,
            admin,
            index_path,
            index,
            original_index,
            object_directories,
            head,
            default_ref,
            default_commit,
            merge_base,
            tracked,
            worktrees,
            outputs,
            workspaces,
            cargo_inputs,
        } = self;
        [
            (top != &actual.top, "selection.top"),
            (common != &actual.common, "selection.common"),
            (admin != &actual.admin, "selection.admin"),
            (index_path != &actual.index_path, "selection.index_path"),
            (index != &actual.index, "selection.index"),
            (
                original_index != &actual.original_index,
                "selection.original_index",
            ),
            (
                object_directories != &actual.object_directories,
                "selection.object_directories",
            ),
            (head != &actual.head, "selection.head"),
            (default_ref != &actual.default_ref, "selection.default_ref"),
            (
                default_commit != &actual.default_commit,
                "selection.default_commit",
            ),
            (merge_base != &actual.merge_base, "selection.merge_base"),
            (tracked != &actual.tracked, "selection.tracked"),
            (worktrees != &actual.worktrees, "selection.worktrees"),
            (outputs != &actual.outputs, "selection.outputs"),
            (workspaces != &actual.workspaces, "selection.workspaces"),
            (
                cargo_inputs != &actual.cargo_inputs,
                "selection.cargo_inputs",
            ),
        ]
        .into_iter()
        .find_map(|(changed, field)| changed.then_some(field))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Capture {
    pub root: PathBuf,
    pub prefix: RelPath,
    pub(super) rows: Vec<(RelPath, Node)>,
    pub excluded: Vec<PathBuf>,
    pub(super) selection: Option<Selection>,
    pub digest: String,
    pub(super) output_aliases: Vec<(RelPath, PathBuf)>,
}

impl Capture {
    pub(super) fn changed(&self, actual: &Self, operation: &str) -> ViewError {
        match self.difference(actual) {
            Some(reason) => ViewError::changed(operation, reason),
            None => ViewError::refusal(
                operation,
                "capture equality reported a mismatch without a differing field; inspect capture comparison diagnostics",
            ),
        }
    }

    fn difference(&self, actual: &Self) -> Option<String> {
        let Self {
            root,
            prefix,
            rows,
            excluded,
            selection,
            digest,
            output_aliases,
        } = self;
        for (changed, field) in [
            (root != &actual.root, "root"),
            (prefix != &actual.prefix, "prefix"),
        ] {
            if changed {
                return Some(format!("capture.{field} changed"));
            }
        }
        match (selection, &actual.selection) {
            (Some(expected), Some(actual)) => {
                if let Some(field) = expected.difference(actual) {
                    return Some(format!("{field} changed"));
                }
            }
            (None, None) => {}
            _ => return Some("capture.selection presence changed".into()),
        }
        for ((path, expected), (actual_path, observed)) in rows.iter().zip(&actual.rows) {
            if path != actual_path {
                return Some(format!(
                    "captured row membership changed between {path} and {actual_path}"
                ));
            }
            if let Some(field) = expected.difference(observed) {
                return Some(format!(
                    "captured {} {field} changed at {path}",
                    expected.kind()
                ));
            }
        }
        if rows.len() != actual.rows.len() {
            let first = rows.len().min(actual.rows.len());
            if let Some((path, node)) = rows.get(first).or_else(|| actual.rows.get(first)) {
                return Some(format!(
                    "captured {} membership changed at {path}",
                    node.kind()
                ));
            }
        }
        [
            (excluded != &actual.excluded, "excluded"),
            (output_aliases != &actual.output_aliases, "output_aliases"),
            (digest != &actual.digest, "digest"),
        ]
        .into_iter()
        .find_map(|(changed, field)| changed.then(|| format!("capture.{field} changed")))
    }

    pub(super) fn read(
        request: &ViewRequest,
        excluded: &[PathBuf],
        selection: Option<Selection>,
    ) -> Result<Self, ViewError> {
        let root = selection
            .as_ref()
            .map_or_else(|| request.root.clone(), |value| value.top.clone());
        let root = std::fs::canonicalize(root)
            .map_err(|e| ViewError::new("resolve repository capture root", e))?;
        let workspace = std::fs::canonicalize(&request.root)
            .map_err(|e| ViewError::new("resolve capture root", e))?;
        let prefix = relative(
            workspace
                .strip_prefix(&root)
                .map_err(|e| ViewError::new("workspace outside captured repository", e))?,
        )?;
        let tree = DiskTree::new(&root).map_err(|e| ViewError::new("open capture", e))?;
        let mut admission = Resident;
        let mut memory = Memory::new(&mut admission);
        let mut exclusions = OrderedTable::<PathBuf, ()>::default();
        for path in excluded {
            if exclusions.get(path).is_none() {
                exclusions
                    .insert(path.clone(), (), &mut memory)
                    .map_err(|e| ViewError::new("index capture exclusions", e))?;
            }
        }
        let outputs = selection
            .as_ref()
            .map_or(&[][..], |value| value.outputs.as_slice());
        let mut walk = Walk {
            tree: &tree,
            root: &root,
            outputs,
            exclusions,
            rows: Vec::new(),
            aliases: Vec::new(),
            bytes: 0,
        };
        walk.read(&mut memory)?;
        let excluded: Vec<PathBuf> = walk
            .exclusions
            .ordered()
            .map(|(path, ())| path.clone())
            .collect();
        for (path, node) in &mut walk.rows {
            if let Node::Dir(directory) = node {
                let directory = Arc::get_mut(directory).ok_or_else(|| {
                    ViewError::refusal(
                        "complete selected membership",
                        "directory observation unexpectedly shared before admission",
                    )
                })?;
                directory.entries.retain(|entry| {
                    !excluded_by(
                        &walk.exclusions,
                        &root.join(path.as_str()).join(&entry.name),
                    )
                });
            }
        }
        let Walk { rows, aliases, .. } = walk;
        let mut captured = Self {
            root,
            prefix,
            rows,
            excluded,
            selection,
            digest: String::new(),
            output_aliases: aliases,
        };
        try_sort_unstable_by(&mut captured.rows, |a, b| Ok::<_, ViewError>(a.0.cmp(&b.0)))?;
        if captured.rows.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(ViewError::refusal(
                "capture repository",
                "directory traversal produced a repeated path",
            ));
        }
        captured.admit(request)?;
        captured.digest = captured.hash(request);
        Ok(captured)
    }

    pub(super) fn get(&self, path: &RelPath) -> Option<&Node> {
        let range = try_equal_range_by(&self.rows, |(candidate, _)| {
            Ok::<_, std::convert::Infallible>(candidate.cmp(path))
        });
        match range {
            Ok(range) => self
                .rows
                .get(range.start)
                .filter(|(candidate, _)| candidate == path)
                .map(|(_, node)| node),
            Err(never) => match never {},
        }
    }

    pub(super) fn excludes(&self, path: &Path) -> bool {
        path.ancestors().any(|ancestor| {
            match try_equal_range_by(&self.excluded, |candidate| {
                Ok::<_, std::convert::Infallible>(candidate.as_path().cmp(ancestor))
            }) {
                Ok(range) => !range.is_empty(),
                Err(never) => match never {},
            }
        })
    }

    fn admit(&self, request: &ViewRequest) -> Result<(), ViewError> {
        let tracked: &[RelPath] = if let Some(selection) = &self.selection {
            &selection.tracked
        } else {
            request.tracked.as_deref().unwrap_or(&[])
        };
        for path in tracked {
            let absolute = if self.selection.is_some() {
                self.root.join(path.as_str())
            } else {
                self.root.join(self.prefix.as_str()).join(path.as_str())
            };
            if self.excludes(&absolute) {
                return Err(ViewError::refusal(
                    "admit capture exclusions",
                    format!(
                        "tracked input {} overlaps an excluded root",
                        absolute.display()
                    ),
                ));
            }
        }
        for path in &request.required {
            let absolute = self.root.join(self.prefix.as_str()).join(path.as_str());
            if self.excludes(&absolute) {
                return Err(ViewError::refusal(
                    "admit capture exclusions",
                    format!("loaded input {path} overlaps an excluded root"),
                ));
            }
        }
        for path in self.selection.iter().flat_map(|value| &value.cargo_inputs) {
            if self.excludes(&self.root.join(path.as_str())) {
                return Err(ViewError::refusal(
                    "admit Cargo inputs",
                    format!("Cargo input {path} overlaps an excluded root or is not retained"),
                ));
            }
            self.retained_input(path).map_err(|e| {
                ViewError::new(
                    format!("Cargo input {path} is not retained by the admitted selection"),
                    e,
                )
            })?;
        }
        for path in &request.required {
            self.identity(path).map_err(|e| ViewError::new(format!("loaded input {path} is not retained by the admitted selection (including output aliases)"), e))?;
        }
        for output in self.selection.iter().flat_map(|value| &value.outputs) {
            if self.root.starts_with(output) {
                return Err(ViewError::refusal(
                    "admit Cargo output",
                    "output contains the repository root",
                ));
            }
        }
        let mut admission = Resident;
        let mut memory = Memory::new(&mut admission);
        let mut aliases: OrderedTable<raf_source::FileId, Arc<CapturedFile>> =
            OrderedTable::default();
        for (_, node) in &self.rows {
            if let Node::File(file) = node {
                if let Some(previous) = aliases.get(&file.id) {
                    if previous.bytes != file.bytes || previous.permissions != file.permissions {
                        return Err(ViewError::refusal(
                            "capture hardlink aliases",
                            "one physical file changed between its captured aliases",
                        ));
                    }
                } else if aliases
                    .insert(file.id.clone(), file.clone(), &mut memory)
                    .map_err(|e| ViewError::new("index captured file aliases", e))?
                    .is_some()
                {
                    return Err(ViewError::refusal(
                        "capture aliases",
                        "duplicate identity admission",
                    ));
                }
            }
        }
        Ok(())
    }

    fn hash(&self, request: &ViewRequest) -> String {
        let mut hash = purrdf_hash::blake3::Hasher::new();
        for (path, node) in &self.rows {
            part(&mut hash, path.as_str().as_bytes());
            match node {
                Node::File(file) => {
                    part(&mut hash, b"file");
                    part(&mut hash, format!("{:?}", file.id).as_bytes());
                    part(&mut hash, &file.bytes);
                    part(&mut hash, &mode(&file.permissions));
                }
                Node::Dir(dir) => {
                    part(&mut hash, b"dir");
                    part(&mut hash, format!("{:?}", dir.id).as_bytes());
                    part(&mut hash, &mode(&dir.permissions));
                    for entry in &dir.entries {
                        part(&mut hash, entry.name.as_bytes());
                        part(&mut hash, format!("{:?}", entry.kind).as_bytes());
                    }
                }
                Node::Link(link) => {
                    part(&mut hash, b"link");
                    part(&mut hash, format!("{:?}", link.id).as_bytes());
                    part(&mut hash, link.target.as_os_str().as_encoded_bytes());
                }
            }
        }
        if let Some(selection) = &self.selection {
            part(&mut hash, selection.index.as_bytes());
            if let Some(file) = &selection.original_index {
                part(&mut hash, b"present-index");
                part(&mut hash, format!("{:?}", file.id).as_bytes());
                part(&mut hash, format!("{:?}", file.permissions).as_bytes());
                part(&mut hash, &file.bytes);
            } else {
                part(&mut hash, b"missing-index");
            }
            for path in selection.object_directories.iter() {
                part(&mut hash, path.as_os_str().as_encoded_bytes());
            }
            for value in [
                &selection.head,
                &selection.default_ref,
                &selection.default_commit,
                &selection.merge_base,
            ] {
                part(&mut hash, value.as_bytes());
            }
            for path in [
                &selection.top,
                &selection.common,
                &selection.admin,
                &selection.index_path,
            ] {
                part(&mut hash, path.as_os_str().as_encoded_bytes());
            }
            for path in &selection.cargo_inputs {
                part(&mut hash, path.as_str().as_bytes());
            }
            for path in &selection.workspaces {
                part(&mut hash, path.as_str().as_bytes());
            }
            for path in &selection.worktrees {
                part(&mut hash, path.as_os_str().as_encoded_bytes());
            }
            for path in &selection.outputs {
                part(&mut hash, path.as_os_str().as_encoded_bytes());
            }
        }
        for path in &self.excluded {
            part(&mut hash, path.as_os_str().as_encoded_bytes());
        }
        for (path, target) in &self.output_aliases {
            part(&mut hash, path.as_str().as_bytes());
            part(&mut hash, target.as_os_str().as_encoded_bytes());
        }
        part(&mut hash, request.hidden.as_str().as_bytes());
        for path in &request.removed {
            part(&mut hash, path.as_str().as_bytes());
        }
        for (path, text) in &request.rewritten {
            part(&mut hash, path.as_str().as_bytes());
            part(&mut hash, text.as_bytes());
        }
        if let Some(paths) = &request.tracked {
            part(&mut hash, b"explicit-tracked-oracle");
            for path in paths {
                part(&mut hash, path.as_str().as_bytes());
            }
        } else {
            part(&mut hash, b"all-selected-files-oracle");
        }
        purrdf_hash::hex::encode(hash.finalize().as_bytes())
    }
}

#[cfg(unix)]
fn mode(value: &std::fs::Permissions) -> Vec<u8> {
    use std::os::unix::fs::PermissionsExt as _;
    value.mode().to_le_bytes().to_vec()
}
#[cfg(not(unix))]
fn mode(value: &std::fs::Permissions) -> Vec<u8> {
    vec![u8::from(value.readonly())]
}

pub(super) fn relative(path: &Path) -> Result<RelPath, ViewError> {
    let text = path
        .to_str()
        .ok_or_else(|| ViewError::refusal("capture path", "non-UTF-8 entry name"))?;
    RelPath::new(text).map_err(|e| ViewError::new("capture path", e))
}

struct Walk<'a> {
    tree: &'a DiskTree,
    root: &'a Path,
    outputs: &'a [PathBuf],
    exclusions: OrderedTable<PathBuf, ()>,
    rows: Vec<(RelPath, Node)>,
    aliases: Vec<(RelPath, PathBuf)>,
    bytes: usize,
}
impl Walk<'_> {
    fn read(&mut self, memory: &mut Memory<'_, Resident>) -> Result<(), ViewError> {
        let mut pending: WorkList<RelPath, 16> = WorkList::new();
        let mut visited = 0;
        pending.push(RelPath::root());
        while let Some(path) = pending.pop() {
            visited += 1;
            if visited > MAX_ENTRIES {
                return Err(ViewError::refusal(
                    "capture repository",
                    "observed entry limit exceeded",
                ));
            }
            let absolute = self.root.join(path.as_str());
            if excluded_by(&self.exclusions, &absolute) {
                if self.outputs.contains(&absolute)
                    && self
                        .tree
                        .entry(&path)
                        .map_err(|e| ViewError::new("inspect Cargo output locator", e))?
                        == Some(EntryKind::Symlink)
                {
                    let link = self
                        .tree
                        .capture_link(&path)
                        .map_err(|e| ViewError::new("capture Cargo output locator", e))?;
                    memory
                        .push(&mut self.aliases, (path, link.target))
                        .map_err(|e| ViewError::new("retain output locator", e))?;
                }
                continue;
            }
            let kind = self
                .tree
                .entry(&path)
                .map_err(|e| ViewError::new(format!("inspect {path}"), e))?;
            let node = match kind {
                Some(EntryKind::File) => {
                    let remaining = u64::try_from(MAX_BYTES - self.bytes)
                        .map_err(|e| ViewError::new("remaining capture bytes", e))?;
                    let file = self
                        .tree
                        .capture_file(&path, MAX_FILE_BYTES.min(remaining))
                        .map_err(|e| {
                            ViewError::new(
                                format!("capture {path} within file/aggregate budget"),
                                e,
                            )
                        })?;
                    self.bytes = self.bytes.checked_add(file.bytes.len()).ok_or_else(|| {
                        ViewError::refusal("capture repository", "byte count overflow")
                    })?;
                    Node::File(Arc::new(file))
                }
                Some(EntryKind::Dir) => {
                    let directory = self
                        .tree
                        .capture_directory_bounded(&path, MAX_ENTRIES - visited - pending.len())
                        .map_err(|e| ViewError::new(format!("capture {path}"), e))?;
                    for entry in &directory.entries {
                        pending.push(
                            path.join(&entry.name)
                                .map_err(|e| ViewError::new("capture child path", e))?,
                        );
                    }
                    Node::Dir(Arc::new(directory))
                }
                Some(EntryKind::Symlink) => {
                    let link = self
                        .tree
                        .capture_link(&path)
                        .map_err(|e| ViewError::new(format!("capture {path}"), e))?;
                    if output_target(self.root, &path, &link.target)
                        .is_some_and(|target| self.outputs.contains(&target))
                    {
                        self.exclusions
                            .insert(absolute, (), memory)
                            .map_err(|e| ViewError::new("admit evidenced Cargo output alias", e))?;
                        memory
                            .push(&mut self.aliases, (path, link.target))
                            .map_err(|e| ViewError::new("retain Cargo output alias", e))?;
                        continue;
                    }
                    Node::Link(Arc::new(link))
                }
                _ => {
                    return Err(ViewError::refusal(
                        "capture repository",
                        format!("{path} is missing or an unsupported special entry"),
                    ));
                }
            };
            memory
                .push(&mut self.rows, (path, node))
                .map_err(|e| ViewError::new("retain captured entry", e))?;
        }
        try_sort_unstable_by(&mut self.aliases, |a, b| Ok::<_, ViewError>(a.0.cmp(&b.0)))?;
        Ok(())
    }
}
fn excluded_by(excluded: &OrderedTable<PathBuf, ()>, path: &Path) -> bool {
    path.ancestors().any(|ancestor| {
        excluded
            .get_by(|stored| ancestor.cmp(stored.as_path()))
            .is_some()
    })
}
pub(super) fn output_target(root: &Path, path: &RelPath, target: &Path) -> Option<PathBuf> {
    if target.is_absolute() {
        if target
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return None;
        }
        return Some(target.to_path_buf());
    }
    let parent = path.parent()?;
    let mut absolute = root.join(parent.as_str());
    let mut normal = false;
    for part in target.components() {
        match part {
            std::path::Component::ParentDir if !normal => {
                if !absolute.pop() {
                    return None;
                }
            }
            std::path::Component::CurDir => {}
            std::path::Component::Normal(name) => {
                normal = true;
                absolute.push(name);
            }
            _ => return None,
        }
    }
    Some(absolute)
}
