// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Split only mount-exception ancestors; all other copied subtrees stay intact.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use purrdf_lex::allocation::{Memory, OrderedTable, Resident};

use super::super::capture::{MAX_ENTRIES, Node, relative};
use super::{CapturedView, ViewError};
use raf_source::DiskTree;

#[cfg(unix)]
pub(super) fn copied(
    view: &CapturedView,
    destination: &Path,
    exceptions: &[PathBuf],
) -> Result<Vec<OsString>, ViewError> {
    let mut admission = Resident;
    let mut memory = Memory::new(&mut admission);
    let mut split = OrderedTable::<PathBuf, ()>::default();
    split
        .insert(view.capture.root.clone(), (), &mut memory)
        .map_err(|e| ViewError::new("index process root", e))?;
    let absolute_links = view
        .capture
        .rows
        .iter()
        .filter(|(_, node)| matches!(node, Node::Link(link) if link.target.is_absolute()))
        .map(|(path, _)| view.capture.root.join(path.as_str()));
    for exception in exceptions.iter().cloned().chain(absolute_links) {
        for parent in exception
            .ancestors()
            .skip(1)
            .take_while(|path| path.starts_with(&view.capture.root))
        {
            if split
                .get_by(|stored| parent.cmp(stored.as_path()))
                .is_none()
            {
                split
                    .insert(parent.to_owned(), (), &mut memory)
                    .map_err(|e| ViewError::new("index process mount ancestors", e))?;
            }
        }
    }
    let hidden = view
        .capture
        .prefix
        .join(view.request.hidden.as_str())
        .map_err(|e| ViewError::new("process record removal", e))?;
    let mut args = Vec::new();
    create_ancestors(view, destination, &split, &mut args, &mut memory)?;
    let mut covered = OrderedTable::<PathBuf, ()>::default();
    for (path, node) in &view.capture.rows {
        let logical = view.capture.root.join(path.as_str());
        if *path == hidden
            || logical.ancestors().skip(1).any(|parent| {
                covered
                    .get_by(|stored| parent.cmp(stored.as_path()))
                    .is_some()
            })
        {
            continue;
        }
        match node {
            Node::Dir(_) if split.get(&logical).is_some() => {}
            Node::Dir(_) => {
                mount(
                    &mut args,
                    "--bind",
                    &destination.join(path.as_str()),
                    &logical,
                    &mut memory,
                )?;
                covered
                    .insert(logical, (), &mut memory)
                    .map_err(|e| ViewError::new("index covered process subtree", e))?;
            }
            Node::File(_) => mount(
                &mut args,
                "--bind",
                &destination.join(path.as_str()),
                &logical,
                &mut memory,
            )?,
            Node::Link(link) => mount(&mut args, "--symlink", &link.target, &logical, &mut memory)?,
        }
    }
    for (path, _) in &view.request.rewritten {
        let path = view
            .capture
            .prefix
            .join(path.as_str())
            .map_err(|e| ViewError::new("process generated path", e))?;
        if view.capture.get(&path).is_none() {
            let parent = path
                .parent()
                .ok_or_else(|| ViewError::refusal("process generated path", "missing parent"))?;
            if split
                .get(&view.capture.root.join(parent.as_str()))
                .is_some()
            {
                mount(
                    &mut args,
                    "--bind",
                    &destination.join(path.as_str()),
                    &view.capture.root.join(path.as_str()),
                    &mut memory,
                )?;
            }
        }
    }
    Ok(args)
}

#[cfg(unix)]
fn create_ancestors(
    view: &CapturedView,
    destination: &Path,
    split: &OrderedTable<PathBuf, ()>,
    args: &mut Vec<OsString>,
    memory: &mut Memory<'_, Resident>,
) -> Result<(), ViewError> {
    let tree = DiskTree::new(&view.capture.root)
        .map_err(|e| ViewError::new("observe owned process ancestors", e))?;
    for (path, ()) in split.ordered() {
        let relative = relative(
            path.strip_prefix(&view.capture.root)
                .map_err(|e| ViewError::new("process ancestor outside authority", e))?,
        )?;
        let permissions = match view.capture.get(&relative) {
            Some(Node::Dir(directory)) => directory.permissions.clone(),
            None if Some(path.as_path()) == destination.parent() && view.capture.excludes(path) => {
                tree.capture_directory_bounded(&relative, MAX_ENTRIES)
                    .map_err(|e| ViewError::new("observe reserved process parent", e))?
                    .permissions
            }
            _ => {
                return Err(ViewError::refusal(
                    "process ancestor mode",
                    format!(
                        "{} has no captured or owned directory observation",
                        path.display()
                    ),
                ));
            }
        };
        memory
            .extend(
                args,
                [
                    OsString::from("--perms"),
                    mode(&permissions),
                    OsString::from(if relative.is_root() {
                        "--tmpfs"
                    } else {
                        "--dir"
                    }),
                    path.as_os_str().to_owned(),
                ],
            )
            .map_err(|e| ViewError::new("allocate process mount argv", e))?;
    }
    Ok(())
}

fn mount(
    args: &mut Vec<OsString>,
    operation: &'static str,
    source: &Path,
    target: &Path,
    memory: &mut Memory<'_, Resident>,
) -> Result<(), ViewError> {
    memory
        .extend(
            args,
            [
                OsString::from(operation),
                source.as_os_str().to_owned(),
                target.as_os_str().to_owned(),
            ],
        )
        .map_err(|e| ViewError::new("allocate process bind argv", e))
}

#[cfg(unix)]
fn mode(permissions: &std::fs::Permissions) -> OsString {
    use std::os::unix::fs::PermissionsExt as _;
    OsString::from(format!("{:o}", permissions.mode() & 0o7777))
}

#[cfg(not(unix))]
pub(super) fn copied(
    _: &CapturedView,
    _: &Path,
    _: &[PathBuf],
) -> Result<Vec<OsString>, ViewError> {
    Err(ViewError::refusal(
        "prepare native process layout",
        "this platform has no admitted native mount-view backend",
    ))
}
