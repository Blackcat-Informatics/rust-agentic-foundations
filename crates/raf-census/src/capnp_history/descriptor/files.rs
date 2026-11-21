// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Admit the requested filenames and file nodes of selected identities.

use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

use capnp::schema_capnp::{code_generator_request, node};
use raf_source::RelPath;

use super::{Error, Nodes, TypeId, layout};
use crate::capnp_history::capture::canonical_path;

pub(super) fn project(
    request: code_generator_request::Reader<'_>,
    nodes: &Nodes<'_>,
    selected: &BTreeSet<TypeId>,
    required: bool,
) -> Result<BTreeMap<TypeId, RelPath>, Error> {
    let mut paths = BTreeMap::new();
    if selected.is_empty() {
        return Ok(paths);
    }

    let rows = request.get_requested_files()?;
    for row in rows {
        let id = TypeId(row.get_id());
        if !selected.contains(&id) {
            continue;
        }
        let Entry::Vacant(entry) = paths.entry(id) else {
            return Err(layout(format!(
                "selected file {id} has repeated request rows"
            )));
        };
        let filename = row.get_filename()?;
        let filename = filename
            .to_str()
            .map_err(|_| layout(format!("selected file {id} requires a UTF-8 filename")))?;
        let path = canonical_path(filename, false).map_err(|reason| {
            layout(format!(
                "selected file {id} has an invalid requested filename: {reason}"
            ))
        })?;
        entry.insert(path);
    }

    for id in selected {
        match (paths.get(id), nodes.by_id.get(id)) {
            (None, None) if !required => {}
            (None, _) => {
                return Err(layout(format!(
                    "selected file {id} is missing its request row"
                )));
            }
            (Some(path), None) => {
                return Err(layout(format!(
                    "requested file {id} at {path} is missing its file node"
                )));
            }
            (Some(path), Some(node)) => {
                if !matches!(node.which()?, node::File(())) {
                    return Err(layout(format!(
                        "requested file {id} at {path} requires a file node"
                    )));
                }
                if node.get_display_name()?.as_bytes() != path.as_str().as_bytes() {
                    return Err(layout(format!(
                        "file node name differs from requested filename {path} ({id})"
                    )));
                }
            }
        }
    }
    verify_path_owners(rows, &paths)?;
    Ok(paths)
}

fn verify_path_owners(
    rows: capnp::struct_list::Reader<'_, code_generator_request::requested_file::Owned>,
    paths: &BTreeMap<TypeId, RelPath>,
) -> Result<(), Error> {
    let mut claimed = BTreeMap::new();
    for (&id, path) in paths {
        if let Some(previous) = claimed.insert(path.as_str().as_bytes(), (id, path)) {
            return Err(repeated_path(path, previous.0, id));
        }
    }
    if claimed.is_empty() {
        return Ok(());
    }
    for row in rows {
        let id = TypeId(row.get_id());
        if !paths.contains_key(&id)
            && let Some(&(owner, path)) = claimed.get(row.get_filename()?.as_bytes())
        {
            return Err(repeated_path(path, owner, id));
        }
    }
    Ok(())
}

fn repeated_path(path: &RelPath, first: TypeId, second: TypeId) -> Error {
    layout(format!(
        "requested filename {path} has competing file IDs {first} and {second}"
    ))
}
