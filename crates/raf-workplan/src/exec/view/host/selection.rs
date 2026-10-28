// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Exclusions require installed-Git or Cargo observations, not names or ignores.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use purrdf_lex::json::{Limits as JsonLimits, Reader};
use purrdf_lex::walk::try_sort_unstable_by;
use raf_source::{DiskTree, RelPath, Tree, TreeError};

use super::super::git_context::{GitContext, InGit};
use super::{command, query, text};
use crate::exec::view::capture::{Capture, Node, Selection, output_target, relative};
use crate::exec::{
    ExecError, Invocation, Limits, Output, Program, Runner, SystemRunner, ViewError, ViewRequest,
};

pub(super) struct Observation {
    pub selection: Selection,
    pub excluded: Vec<PathBuf>,
    pub flattened_index: Option<Arc<[u8]>>,
}

pub(super) fn observe(
    runner: &dyn Runner,
    request: &ViewRequest,
    own: Option<&Path>,
    filesystem: Option<&super::super::FilesystemView>,
    workspaces: &[RelPath],
) -> Result<Observation, ExecError> {
    let root = request.root();
    let authority = Authority {
        top: git_path(runner, root, "--show-toplevel")?,
        common: git_path(runner, root, "--git-common-dir")?,
        admin: git_path(runner, root, "--absolute-git-dir")?,
        index_path: index_path(runner, root)?,
        objects: super::objects::observe(runner, root)?.into(),
    };
    if filesystem.is_some() {
        let selected = InGit {
            runner,
            context: GitContext::Discover,
        };
        observe_selected(
            &selected, runner, request, own, filesystem, workspaces, &authority,
        )
    } else {
        super::index::route(
            runner,
            &authority.top,
            &authority.admin,
            &authority.common,
            &authority.index_path,
            &authority.objects,
            |selected| {
                observe_selected(
                    selected, runner, request, own, filesystem, workspaces, &authority,
                )
            },
        )
    }
}

struct Authority {
    top: PathBuf,
    common: PathBuf,
    admin: PathBuf,
    index_path: PathBuf,
    objects: Arc<[PathBuf]>,
}

fn observe_selected(
    selected: &dyn Runner,
    original: &dyn Runner,
    request: &ViewRequest,
    own: Option<&Path>,
    filesystem: Option<&super::super::FilesystemView>,
    workspaces: &[RelPath],
    authority: &Authority,
) -> Result<Observation, ExecError> {
    let root = request.root();
    let Authority {
        top,
        common,
        admin,
        index_path,
        objects,
    } = authority;
    let scoped = Scoped {
        runner: selected,
        directory: root,
    };
    let head = crate::git::head(&scoped)
        .map_err(|e| ViewError::new("capture original HEAD", e))?
        .as_str()
        .to_string();
    let default_ref = crate::git::default_branch_ref(&scoped)
        .map_err(|e| ViewError::new("capture default reference", e))?;
    let default_commit = crate::git::resolve_commit(&scoped, &default_ref)
        .map_err(|e| ViewError::new("capture default commit", e))?;
    let merge_base = crate::git::merge_base(&scoped, default_commit.commit())
        .map_err(|e| ViewError::new("capture merge base", e))?
        .as_str()
        .to_string();
    let snapshot = if filesystem.is_some() {
        super::index::IndexSnapshot {
            tree: query(selected, root, Program::Git, &["write-tree"])?,
            original: None,
            bytes: None,
        }
    } else {
        super::index::tree(original, top, admin, common, index_path, objects)?
    };
    let index = super::index::identity(snapshot.tree)?;
    let names = query(
        selected,
        top,
        Program::Git,
        &["ls-files", "--full-name", "-z"],
    )?;
    let tracked = paths(&names)?;
    let worktree_bytes = query(
        selected,
        top,
        Program::Git,
        &["worktree", "list", "--porcelain", "-z"],
    )?;
    let worktrees = registrations(&worktree_bytes, own)?;
    if !worktrees.iter().any(|value| value == top) {
        return Err(ViewError::refusal(
            "capture worktree registration",
            "source top is absent from Git's registered worktrees",
        )
        .into());
    }
    let (outputs, workspaces) = outputs(original, request, top, filesystem, workspaces)?;
    let cargo_inputs = cargo_inputs(request, top, &workspaces)?;
    let mut excluded = vec![
        top.join(".git"),
        common.clone(),
        admin.clone(),
        index_path.clone(),
    ];
    excluded.extend(objects.iter().cloned());
    excluded.extend(
        worktrees
            .iter()
            .filter(|value| *value != top && value.starts_with(top))
            .cloned(),
    );
    excluded.extend(outputs.iter().cloned());
    try_sort_unstable_by(&mut excluded, |a, b| Ok::<_, ViewError>(a.cmp(b)))?;
    Ok(Observation {
        selection: Selection {
            top: top.clone(),
            common: common.clone(),
            admin: admin.clone(),
            index_path: index_path.clone(),
            index,
            original_index: snapshot.original,
            object_directories: objects.clone(),
            head,
            default_ref: default_ref.to_string(),
            default_commit: default_commit.as_str().to_string(),
            merge_base,
            tracked,
            worktrees,
            outputs,
            workspaces,
            cargo_inputs,
        },
        excluded,
        flattened_index: snapshot.bytes,
    })
}

/// Only a captured external link can request discovery beyond loaded manifests.
/// Native workspace ownership and exact Cargo output-root equality must both
/// agree; an arbitrary external asset link is never an output exclusion.
pub(super) fn capture(
    runner: &SystemRunner,
    request: &ViewRequest,
    mut selection: Selection,
    mut excluded: Vec<PathBuf>,
) -> Result<Capture, ExecError> {
    let captured = Capture::read(request, &excluded, Some(selection.clone()))?;
    let tree = DiskTree::new(&selection.top)
        .map_err(|e| ViewError::new("open output-locator authority", e))?;
    let mut admission = Resident;
    let mut memory = Memory::new(&mut admission);
    let mut observed = OrderedTable::<RelPath, PathBuf>::default();
    for (path, node) in &captured.rows {
        let Node::Link(link) = node else { continue };
        let Some(target) = output_target(&captured.root, path, &link.target) else {
            continue;
        };
        if target.starts_with(&captured.root) {
            continue;
        }
        let parent = path.parent().ok_or_else(|| {
            ViewError::refusal("admit external output locator", "link has no parent")
        })?;
        let workspace = raf_source::discover_root(&tree, &parent).map_err(|e| {
            ViewError::new(format!("discover Cargo owner of external link {path}"), e)
        })?;
        let output = if let Some(output) = observed.get(&workspace) {
            output.clone()
        } else {
            let output = workspace_output(
                runner,
                &selection.top.join(workspace.as_str()),
                &selection.top,
                &workspace,
                None,
            )?;
            observed
                .insert(workspace.clone(), output.clone(), &mut memory)
                .map_err(|e| ViewError::new("retain observed output-locator owner", e))?;
            output
        };
        if output != target {
            return Err(ViewError::refusal(
                "admit external output locator",
                format!("{path} does not name its owning Cargo workspace's exact output root: link reaches {}, metadata for {} returns {} (Cargo {}, inherited CARGO_TARGET_DIR {:?})", target.display(), workspace, output.display(), runner.programs().cargo.display(), std::env::var_os("CARGO_TARGET_DIR")),
            )
            .into());
        }
        selection.workspaces.push(workspace);
        selection.outputs.push(output.clone());
        excluded.push(output);
    }
    if observed.is_empty() {
        return Ok(captured);
    }
    try_sort_unstable_by(&mut selection.workspaces, |a, b| {
        Ok::<_, ViewError>(a.cmp(b))
    })?;
    selection.workspaces.dedup();
    try_sort_unstable_by(&mut selection.outputs, |a, b| Ok::<_, ViewError>(a.cmp(b)))?;
    selection.outputs.dedup();
    selection.cargo_inputs = cargo_inputs(request, &selection.top, &selection.workspaces)?;
    drop(captured);
    Capture::read(request, &excluded, Some(selection)).map_err(Into::into)
}

fn index_path(runner: &dyn Runner, root: &Path) -> Result<PathBuf, ExecError> {
    let bytes = query(
        runner,
        root,
        Program::Git,
        &["rev-parse", "--git-path", "index"],
    )?;
    let value = text(bytes, "read effective Git index path")?;
    let value = value.strip_suffix('\n').ok_or_else(|| {
        ViewError::refusal("read effective Git index path", "missing final newline")
    })?;
    let path = root.join(value);
    match std::fs::canonicalize(&path) {
        Ok(path) => Ok(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let parent = path.parent().ok_or_else(|| {
                ViewError::refusal("resolve effective Git index", "index has no parent")
            })?;
            let name = path.file_name().ok_or_else(|| {
                ViewError::refusal("resolve effective Git index", "index has no file name")
            })?;
            Ok(std::fs::canonicalize(parent)
                .map_err(|e| ViewError::new("resolve effective Git index parent", e))?
                .join(name))
        }
        Err(error) => Err(ViewError::new("resolve effective Git index", error).into()),
    }
}

fn registrations(bytes: &[u8], own: Option<&Path>) -> Result<Vec<PathBuf>, ViewError> {
    if !bytes.is_empty() && bytes.last() != Some(&0) {
        return Err(ViewError::refusal(
            "read worktree registration",
            "unterminated Git protocol",
        ));
    }
    let mut worktrees = Vec::new();
    for field in bytes.split(|byte| *byte == 0) {
        if let Some(raw) = field.strip_prefix(b"worktree ") {
            let value = std::str::from_utf8(raw)
                .map_err(|e| ViewError::new("read registered worktree", e))?;
            let value = std::fs::canonicalize(value).map_err(|e| {
                ViewError::new(
                    format!(
                        "resolve registered worktree {value}; restore access or prune its stale registration"
                    ),
                    e,
                )
            })?;
            if Some(value.as_path()) != own {
                worktrees.push(value);
            }
        }
    }
    try_sort_unstable_by(&mut worktrees, |a, b| Ok::<_, ViewError>(a.cmp(b)))?;
    Ok(worktrees)
}

pub(super) fn git_path(runner: &dyn Runner, root: &Path, flag: &str) -> Result<PathBuf, ExecError> {
    let bytes = query(runner, root, Program::Git, &["rev-parse", flag])?;
    let value = text(bytes, "read Git path")?;
    let value = value
        .strip_suffix('\n')
        .ok_or_else(|| ViewError::refusal("read Git path", "missing final newline"))?;
    std::fs::canonicalize(root.join(value))
        .map_err(|e| ViewError::new("resolve Git path", e).into())
}
fn paths(bytes: &[u8]) -> Result<Vec<RelPath>, ExecError> {
    if !bytes.is_empty() && bytes.last() != Some(&0) {
        return Err(
            ViewError::refusal("read tracked paths", "unterminated Git path protocol").into(),
        );
    }
    bytes
        .split(|byte| *byte == 0)
        .filter(|value| !value.is_empty())
        .map(|value| {
            let text =
                std::str::from_utf8(value).map_err(|e| ViewError::new("read tracked path", e))?;
            RelPath::new(text).map_err(|e| ViewError::new("read tracked path", e).into())
        })
        .collect()
}
fn outputs(
    runner: &dyn Runner,
    request: &ViewRequest,
    top: &Path,
    filesystem: Option<&super::super::FilesystemView>,
    additional: &[RelPath],
) -> Result<(Vec<PathBuf>, Vec<RelPath>), ExecError> {
    let tree = DiskTree::new(top).map_err(|e| ViewError::new("load metadata roots", e))?;
    let prefix = relative(
        request
            .root()
            .strip_prefix(top)
            .map_err(|e| ViewError::new("Cargo metadata outside capture root", e))?,
    )?;
    let mut roots = Vec::new();
    for manifest in request
        .required
        .iter()
        .filter(|path| path.file_name() == Some("Cargo.toml"))
    {
        let parent = manifest.parent().ok_or_else(|| {
            ViewError::refusal("discover Cargo workspace", "manifest has no parent")
        })?;
        let parent = prefix
            .join(parent.as_str())
            .map_err(|e| ViewError::new("Cargo discovery path", e))?;
        let workspace = raf_source::discover_root(&tree, &parent)
            .map_err(|e| ViewError::new("discover Cargo workspace", e))?;
        if !roots.contains(&workspace) {
            roots.push(workspace);
        }
    }
    let mut found = Vec::new();
    let loaded = roots.clone();
    roots.extend_from_slice(additional);
    try_sort_unstable_by(&mut roots, |a, b| Ok::<_, ViewError>(a.cmp(b)))?;
    roots.dedup();
    for workspace in &roots {
        let directory = top.join(workspace.as_str());
        let directory = if loaded.contains(workspace) {
            request.root()
        } else {
            &directory
        };
        let output = workspace_output(runner, directory, top, workspace, filesystem)?;
        if !found.contains(&output) {
            found.push(output);
        }
    }
    try_sort_unstable_by(&mut found, |a, b| Ok::<_, ViewError>(a.cmp(b)))?;
    Ok((found, roots))
}

fn workspace_output(
    runner: &dyn Runner,
    directory: &Path,
    top: &Path,
    workspace: &RelPath,
    filesystem: Option<&super::super::FilesystemView>,
) -> Result<PathBuf, ExecError> {
    let expected = std::fs::canonicalize(top.join(workspace.as_str()))
        .map_err(|e| ViewError::new("metadata workspace", e))?;
    let manifest = expected.join("Cargo.toml");
    let manifest = manifest.strip_prefix(directory).unwrap_or(&manifest);
    let args = vec![
        "metadata".into(),
        "--format-version=1".into(),
        "--no-deps".into(),
        "--frozen".into(),
        "--manifest-path".into(),
        manifest
            .to_str()
            .ok_or_else(|| ViewError::refusal("metadata manifest", "non-UTF8 path"))?
            .to_owned(),
    ];
    let bytes = command(runner, directory, Program::Cargo, &args)?;
    let (root, output) = metadata(&bytes)?;
    let process_root =
        filesystem.map_or_else(|| Ok(expected.clone()), |view| view.logical(&expected))?;
    if root != process_root {
        return Err(ViewError::refusal(
            "admit Cargo metadata",
            "workspace_root differs from native Source discovery",
        )
        .into());
    }
    Ok(output)
}

fn cargo_inputs(
    request: &ViewRequest,
    top: &Path,
    workspaces: &[RelPath],
) -> Result<Vec<RelPath>, ExecError> {
    let tree = DiskTree::new(top).map_err(|e| ViewError::new("open Cargo input authority", e))?;
    let root = std::fs::canonicalize(request.root())
        .map_err(|e| ViewError::new("resolve Cargo configuration root", e))?;
    let mut candidates: Vec<_> = workspaces
        .iter()
        .flat_map(|workspace| {
            let root = top.join(workspace.as_str());
            [root.join("Cargo.lock"), root.join("Cargo.toml")]
        })
        .collect();
    for root in std::iter::once(root).chain(workspaces.iter().map(|path| top.join(path.as_str()))) {
        for ancestor in root
            .ancestors()
            .take_while(|ancestor| ancestor.starts_with(top))
        {
            candidates.push(ancestor.join(".cargo/config"));
            candidates.push(ancestor.join(".cargo/config.toml"));
        }
    }
    let mut inputs = Vec::new();
    for candidate in candidates {
        let path = relative(
            candidate
                .strip_prefix(top)
                .map_err(|e| ViewError::new("Cargo input outside captured authority", e))?,
        )?;
        match tree.entry(&path) {
            Ok(None) | Err(TreeError::NotFound { .. }) => {}
            Ok(Some(_)) => {
                tree.identity(&path)
                    .map_err(|e| ViewError::new(format!("admit Cargo input {path}"), e))?;
                inputs.push(path);
            }
            Err(error) => {
                return Err(ViewError::new(format!("inspect Cargo input {path}"), error).into());
            }
        }
    }
    try_sort_unstable_by(&mut inputs, |a, b| Ok::<_, ViewError>(a.cmp(b)))?;
    inputs.dedup();
    Ok(inputs)
}
fn metadata(bytes: &[u8]) -> Result<(PathBuf, PathBuf), ExecError> {
    let limits = JsonLimits {
        max_depth: 128,
        max_values: 262_144,
        max_string_bytes: 1024 * 1024,
        unique_members: true,
    };
    let mut reader =
        Reader::from_slice(bytes, limits).map_err(|e| ViewError::new("parse Cargo metadata", e))?;
    let value = reader
        .read_value()
        .map_err(|e| ViewError::new("parse Cargo metadata", e))?;
    reader
        .finish()
        .map_err(|e| ViewError::new("finish Cargo metadata", e))?;
    let object = value
        .as_object()
        .ok_or_else(|| ViewError::refusal("Cargo metadata", "expected an object"))?;
    if object
        .get("version")
        .and_then(purrdf_lex::json::Value::as_u64)
        != Some(1)
    {
        return Err(ViewError::refusal("Cargo metadata", "expected metadata version 1").into());
    }
    let get = |name| -> Result<PathBuf, ExecError> {
        let value = object
            .get(name)
            .and_then(|value| value.as_str())
            .ok_or_else(|| {
                ViewError::refusal("Cargo metadata", format!("missing string {name}"))
            })?;
        let path = PathBuf::from(value);
        if !path.is_absolute()
            || path
                .components()
                .any(|part| part == std::path::Component::ParentDir)
        {
            return Err(ViewError::refusal(
                "Cargo metadata",
                format!("{name} must be absolute without parent components"),
            )
            .into());
        }
        Ok(path)
    };
    Ok((get("workspace_root")?, get("target_directory")?))
}

struct Scoped<'a> {
    runner: &'a dyn Runner,
    directory: &'a Path,
}
impl Runner for Scoped<'_> {
    fn run(&self, invocation: &Invocation, limits: &Limits) -> Result<Output, ExecError> {
        self.runner
            .run(&invocation.clone().at_directory(self.directory), limits)
    }
}
