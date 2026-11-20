// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Project the uniquely owned written declarations of selected files.

use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    fmt,
};

use capnp::schema_capnp::node;

use super::{
    Declaration, DeclarationBody, Error, MAX_DEPTH, Nodes, Type, TypeId, layout, member_name,
};

/// Kind of a written Cap'n Proto declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeclarationKind {
    /// A declared struct with its own identity.
    Struct,
    /// A declared enum.
    Enum,
    /// A declared interface.
    Interface,
    /// A declared constant.
    Constant,
    /// A declared annotation.
    Annotation,
}

impl fmt::Display for DeclarationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Interface => "interface",
            Self::Constant => "constant",
            Self::Annotation => "annotation",
        })
    }
}

pub(super) fn project(
    nodes: &Nodes<'_>,
    selected: &BTreeSet<TypeId>,
) -> Result<BTreeMap<TypeId, Declaration>, Error> {
    let mut declarations = BTreeMap::new();
    if selected.is_empty() {
        return Ok(declarations);
    }

    let mut pending = Vec::new();
    for file in selected {
        if let Some(reader) = nodes.0.get(file) {
            if reader.get_scope_id() != 0 {
                return Err(layout(format!("selected file {file} must have scope zero")));
            }
            pending.push((*file, 0));
        }
    }
    while let Some((owner, depth)) = pending.pop() {
        let children = nodes.node(owner)?.get_nested_nodes()?;
        if children.is_empty() {
            continue;
        }
        if declarations.get(&owner).is_some_and(|decl: &Declaration| {
            !matches!(
                decl.body,
                DeclarationBody::Struct | DeclarationBody::Interface
            )
        }) {
            return Err(layout(format!(
                "declaration {owner} cannot own declarations"
            )));
        }
        if depth == MAX_DEPTH {
            return Err(layout(format!(
                "declaration scope {owner} exceeds the {MAX_DEPTH}-level nesting bound"
            )));
        }
        let mut names = BTreeSet::new();
        for child in children {
            let id = TypeId(child.get_id());
            if selected.contains(&id) {
                return Err(layout(format!(
                    "scope {owner} cannot list selected file {id} as a child"
                )));
            }
            let Entry::Vacant(entry) = declarations.entry(id) else {
                return Err(layout(format!(
                    "scope {owner} lists an already owned declaration {id}"
                )));
            };
            let name = member_name(child.get_name()?)?;
            if !names.insert(name) {
                return Err(layout(format!(
                    "scope {owner} repeats declaration name {name}"
                )));
            }
            entry.insert(admit(nodes, id, name, owner)?);
            pending.push((id, depth + 1));
        }
    }
    verify_other_nodes(nodes, selected, &declarations)?;
    Ok(declarations)
}

fn admit(nodes: &Nodes<'_>, id: TypeId, name: &str, scope: TypeId) -> Result<Declaration, Error> {
    let child = nodes.0.get(&id).ok_or_else(|| {
        layout(format!(
            "scope {scope} lists missing declaration {name} ({id})"
        ))
    })?;
    if child.get_scope_id() != scope.0 {
        return Err(layout(format!(
            "declaration {name} ({id}) disagrees with listing scope {scope}"
        )));
    }
    let body = match child.which()? {
        node::Struct(data) if !data.get_is_group() => DeclarationBody::Struct,
        node::Enum(_) => DeclarationBody::Enum,
        node::Interface(_) => DeclarationBody::Interface,
        node::Const(constant) => {
            if !constant.has_type() {
                return Err(layout(format!(
                    "constant declaration {name} ({id}) has no type"
                )));
            }
            // Type references validate nominal identity without selecting their layouts.
            DeclarationBody::Constant(Type::read(
                constant.get_type()?,
                nodes,
                id,
                &mut BTreeSet::new(),
            )?)
        }
        node::Annotation(_) => DeclarationBody::Annotation,
        _ => {
            return Err(layout(format!(
                "scope {scope} must list a written declaration at {name} ({id})"
            )));
        }
    };
    Ok(Declaration {
        name: name.to_owned(),
        scope,
        body,
    })
}

fn verify_other_nodes(
    nodes: &Nodes<'_>,
    selected: &BTreeSet<TypeId>,
    declarations: &BTreeMap<TypeId, Declaration>,
) -> Result<(), Error> {
    let reached = |id| selected.contains(&id) || declarations.contains_key(&id);
    for (&id, reader) in &nodes.0 {
        if reached(id) {
            continue;
        }
        let scope = TypeId(reader.get_scope_id());
        if reached(scope) {
            let group_owner = declarations
                .get(&scope)
                .is_some_and(|declaration| matches!(declaration.body, DeclarationBody::Struct));
            if !group_owner || !matches!(reader.which()?, node::Struct(data) if data.get_is_group())
            {
                return Err(layout(format!(
                    "unlisted declaration {id} claims selected scope {scope}"
                )));
            }
        }

        // Unrelated namespaces can be partial. Only a competing edge into the
        // selected inventory affects this projection.
        for child in reader.get_nested_nodes()? {
            let child_id = TypeId(child.get_id());
            if reached(child_id) {
                return Err(layout(format!(
                    "scope {id} competes for selected file or declaration {child_id}"
                )));
            }
        }
    }
    Ok(())
}
