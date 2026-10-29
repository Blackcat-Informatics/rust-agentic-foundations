// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! An owned legitimate Git worktree, filled from checked working observations.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use raf_source::{DiskTree, FileId, RelPath};

use super::super::git_context::{GitContext, InGit, OBJECTS};
use super::query;
use crate::exec::view::capture::Node;
use crate::exec::{CapturedView, ExecError, Program, Runner, SystemRunner, ViewError};

pub(super) fn write(
    runner: &SystemRunner,
    view: &CapturedView,
    destination: &Path,
) -> Result<Vec<(RelPath, fs::Permissions)>, ExecError> {
    let selection =
        view.capture.selection.as_ref().ok_or_else(|| {
            ViewError::refusal("materialize view", "missing native Git provenance")
        })?;
    let name = destination
        .to_str()
        .ok_or_else(|| ViewError::refusal("materialize view", "non-UTF-8 owned root"))?;
    super::index::route(
        runner,
        &selection.top,
        &selection.admin,
        &selection.common,
        &selection.index_path,
        &selection.object_directories,
        |original| {
            query(
                original,
                &selection.top,
                Program::Git,
                &[
                    "worktree",
                    "add",
                    "--detach",
                    "--no-checkout",
                    "--",
                    name,
                    &selection.head,
                ],
            )
        },
    )?;
    let discovery = InGit {
        runner,
        context: GitContext::Discover,
    };
    let admin = super::selection::git_path(&discovery, destination, "--absolute-git-dir")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(&admin, fs::Permissions::from_mode(0o700))
            .map_err(|e| ViewError::new("protect owned Git administration", e))?;
    }
    super::objects::create(&admin.join(OBJECTS), &selection.object_directories)?;
    let owned = InGit {
        runner,
        context: GitContext::owned(&admin, &selection.common, destination),
    };
    install_index(view, &admin, &owned, destination)?;
    files(view, destination)?;
    let rewrite = crate::SyncReport {
        diagram_assets: view.request.diagram_assets.clone(),
        edits: view.request.rewritten.clone(),
    };
    let modes =
        crate::Workplan::write_observed(&rewrite, &destination.join(view.capture.prefix.as_str()))
            .map_err(|e| ViewError::new("publish completed generated edits", e))?;
    let removed = view
        .capture
        .prefix
        .join(view.request.hidden.as_str())
        .map_err(|e| ViewError::new("removed snapshot path", e))?;
    query(
        &owned,
        destination,
        Program::Git,
        &["update-index", "--force-remove", "--", removed.as_str()],
    )?;
    for asset in &view.request.removed {
        let path = view
            .capture
            .prefix
            .join(asset.as_str())
            .map_err(|e| ViewError::new("retired snapshot path", e))?;
        query(
            &owned,
            destination,
            Program::Git,
            &["update-index", "--force-remove", "--", path.as_str()],
        )?;
    }
    for (path, _) in &view.request.rewritten {
        let path = view
            .capture
            .prefix
            .join(path.as_str())
            .map_err(|e| ViewError::new("rewritten snapshot path", e))?;
        query(
            &owned,
            destination,
            Program::Git,
            &["add", "--force", "--", path.as_str()],
        )?;
    }
    verify_tracked(&owned, view, destination)?;
    expose(view, destination)?;
    Ok(modes)
}

fn install_index(
    view: &CapturedView,
    admin: &Path,
    owned: &dyn Runner,
    destination: &Path,
) -> Result<(), ExecError> {
    let selection = view
        .capture
        .selection
        .as_ref()
        .ok_or_else(|| ViewError::refusal("install index", "missing selection"))?;
    if let Some(bytes) = &view.flattened_index {
        let mut index = fs::File::create_new(admin.join("index"))
            .map_err(|e| ViewError::new("create flattened owned index", e))?;
        index
            .write_all(bytes)
            .map_err(|e| ViewError::new("write flattened owned index", e))?;
    } else {
        query(&owned, destination, Program::Git, &["read-tree", "--empty"])?;
    }
    let copied = query(&owned, destination, Program::Git, &["write-tree"])?;
    if super::text(copied, "read copied index tree")?.trim() != selection.index {
        return Err(ViewError::refusal(
            "verify copied index",
            "native flattened index differs from captured selection; recapture the repository",
        )
        .into());
    }
    Ok(())
}

/// Restore the root only after its complete captured population is ready.
fn expose(view: &CapturedView, destination: &Path) -> Result<(), ExecError> {
    let root = view
        .capture
        .rows
        .iter()
        .find_map(|(path, node)| {
            if path.as_str().is_empty()
                && let Node::Dir(directory) = node
            {
                Some(directory)
            } else {
                None
            }
        })
        .ok_or_else(|| {
            ViewError::refusal(
                "expose completed copy",
                "captured root permissions are missing; recapture the repository",
            )
        })?;
    fs::set_permissions(destination, root.permissions.clone())
        .map_err(|e| ViewError::new("restore completed root permissions", e))?;
    Ok(())
}

fn verify_tracked(
    runner: &dyn Runner,
    view: &CapturedView,
    destination: &Path,
) -> Result<(), ExecError> {
    // The ordinary Git oracle must agree with the captured top-level index,
    // except for the one selected removal and explicitly generated additions.
    let actual = query(
        runner,
        &destination.join(view.capture.prefix.as_str()),
        Program::Git,
        &["ls-files", "-z"],
    )?;
    let expected = view.tracked()?.ok_or_else(|| {
        ViewError::refusal(
            "materialized Git oracle",
            "physical Git preparation requires an admitted tracked path set",
        )
    })?;
    let actual: Vec<&[u8]> = actual
        .split(|byte| *byte == 0)
        .filter(|value| !value.is_empty())
        .collect();
    let mut expected: Vec<&[u8]> = expected
        .iter()
        .map(|path| path.as_str().as_bytes())
        .collect();
    purrdf_lex::walk::try_sort_unstable_by(&mut expected, |a, b| Ok::<_, ViewError>(a.cmp(b)))?;
    if actual != expected {
        return Err(ViewError::refusal(
            "verify materialized tracked oracle",
            "the native index differs from the completed selection",
        )
        .into());
    }
    Ok(())
}

fn files(view: &CapturedView, destination: &Path) -> Result<(), ExecError> {
    let mut admission = Resident;
    let mut memory = Memory::new(&mut admission);
    let mut hardlinks: OrderedTable<FileId, PathBuf> = OrderedTable::default();
    let hidden = view
        .capture
        .prefix
        .join(view.request.hidden.as_str())
        .map_err(|e| ViewError::new("completed record path", e))?;
    for (path, node) in &view.capture.rows {
        if *path == hidden {
            continue;
        }
        let target = destination.join(path.as_str());
        match node {
            Node::Dir(_) => fs::create_dir_all(&target)
                .map_err(|e| ViewError::new(format!("create {}", target.display()), e))?,
            Node::File(file) => {
                if let Some(original) = hardlinks.get(&file.id) {
                    fs::hard_link(original, &target)
                        .map_err(|e| ViewError::new("preserve captured hardlink", e))?;
                } else {
                    fs::write(&target, &file.bytes)
                        .map_err(|e| ViewError::new("write captured bytes", e))?;
                    if hardlinks
                        .insert(file.id.clone(), target.clone(), &mut memory)
                        .map_err(|e| ViewError::new("index copied hardlink", e))?
                        .is_some()
                    {
                        return Err(ViewError::refusal(
                            "copy hardlink classes",
                            "duplicate identity admission",
                        )
                        .into());
                    }
                }
                fs::set_permissions(&target, file.permissions.clone())
                    .map_err(|e| ViewError::new("restore captured file permissions", e))?;
            }
            Node::Link(link) => make_link(
                &target,
                &view.relocated_link(destination, path, &link.target)?,
            )?,
        }
    }
    for (path, node) in view.capture.rows.iter().rev() {
        if !path.as_str().is_empty()
            && let Node::Dir(dir) = node
        {
            fs::set_permissions(destination.join(path.as_str()), dir.permissions.clone())
                .map_err(|e| ViewError::new("restore captured directory permissions", e))?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn make_link(path: &Path, target: &Path) -> Result<(), ExecError> {
    std::os::unix::fs::symlink(target, path)
        .map_err(|e| ViewError::new("materialize lexical link", e).into())
}
#[cfg(not(unix))]
fn make_link(_path: &Path, _target: &Path) -> Result<(), ExecError> {
    Err(ViewError::refusal(
        "materialize lexical link",
        "this host cannot preserve checked native links",
    )
    .into())
}

pub(super) fn release(
    runner: &SystemRunner,
    root: &Path,
    destination: &Path,
    identity: &FileId,
) -> Result<(), ExecError> {
    let exists = match fs::symlink_metadata(destination) {
        Ok(_) => {
            let actual = DiskTree::new(destination)
                .and_then(|tree| {
                    tree.capture_directory_bounded(
                        &RelPath::root(),
                        crate::exec::view::capture::MAX_ENTRIES,
                    )
                })
                .map_err(|e| ViewError::new("verify owned root before cleanup", e))?;
            if actual.id != *identity {
                return Err(ViewError::refusal(
                    "release owned root",
                    "its physical directory identity changed; refusing removal",
                )
                .into());
            }
            true
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(ViewError::new("inspect owned root before cleanup", error).into()),
    };
    let output = query(
        runner,
        root,
        Program::Git,
        &["worktree", "list", "--porcelain", "-z"],
    )?;
    let registered = output.split(|byte| *byte == 0).any(|field| {
        field.strip_prefix(b"worktree ") == Some(destination.as_os_str().as_encoded_bytes())
    });
    if registered {
        let name = destination.to_str().ok_or_else(|| {
            ViewError::refusal("release owned root", "non-UTF-8 worktree locator")
        })?;
        query(
            runner,
            root,
            Program::Git,
            &["worktree", "remove", "--force", "--", name],
        )?;
    } else if exists {
        fs::remove_dir_all(destination)
            .map_err(|e| ViewError::new("remove unregistered owned root", e))?;
    }
    Ok(())
}
