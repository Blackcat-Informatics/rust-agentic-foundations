// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Strict history findings for old members and their compiled layout.

use std::{collections::BTreeMap, fmt};

use super::{
    Error, TypeId,
    descriptor::{Node, Schema, Struct},
};

const MAX_FINDING_BYTES: usize = 2 * 1024 * 1024;

/// A change forbidden by the strict schema history policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    /// Nominal schema node that owns the history.
    pub node: TypeId,
    /// Group-qualified member, or no member for a type-level change.
    pub member: Option<String>,
    /// The history rule that failed.
    pub kind: ChangeKind,
}

/// The rule that failed for a previously committed layout.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChangeKind {
    /// A selected root disappeared.
    RemovedRoot,
    /// A struct became an enum or an enum became a struct.
    ChangedKind,
    /// The count of a struct's own type parameters changed.
    ChangedParameters,
    /// An existing member disappeared.
    RemovedMember,
    /// The old ordinal now has a different name.
    RenamedMember {
        /// The name at the original ordinal in the current descriptor.
        current: String,
    },
    /// An originally declared ordinal no longer identifies the same member.
    ChangedOrdinal,
    /// The field moved to a different group.
    ChangedGroup,
    /// The compiled data or pointer slot moved.
    ChangedSlot,
    /// The group tag's offset or the member's tag value changed.
    ChangedDiscriminant,
    /// The type expression changed, including type arguments.
    ChangedType,
    /// The default changed in the compiled descriptor.
    ChangedDefault,
}

impl fmt::Display for ChangeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::RemovedRoot => "selected root removed",
            Self::ChangedKind => "schema kind changed",
            Self::ChangedParameters => "type parameter count changed",
            Self::RemovedMember => "member removed",
            Self::RenamedMember { current } => return write!(f, "member renamed to {current}"),
            Self::ChangedOrdinal => "ordinal changed",
            Self::ChangedGroup => "owning group changed",
            Self::ChangedSlot => "slot offset changed",
            Self::ChangedDiscriminant => "union discriminant changed",
            Self::ChangedType => "type changed",
            Self::ChangedDefault => "default changed",
        })
    }
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cap'n Proto history for {}", self.node)?;
        if let Some(member) = &self.member {
            write!(f, ".{member}")?;
        }
        write!(f, ": {}", self.kind)
    }
}

pub(super) fn changes(base: &Schema, current: &Schema) -> Result<Vec<Change>, Error> {
    let mut findings = Vec::new();
    let mut available = MAX_FINDING_BYTES;
    let mut push = |node, member: Option<String>, kind| {
        let extra = match &kind {
            ChangeKind::RenamedMember { current } => current.len(),
            _ => 0,
        };
        let bytes = size_of::<Change>() + member.as_ref().map_or(0, String::len) + extra;
        let remaining = available.checked_sub(bytes).ok_or(Error::FindingLimit)?;
        available = remaining;
        findings.push(Change { node, member, kind });
        Ok(())
    };
    for &node in base.roots.difference(&current.roots) {
        push(node, None, ChangeKind::RemovedRoot)?;
    }
    for (&node, before) in &base.nodes {
        let Some(after) = current.nodes.get(&node) else {
            continue;
        };
        let mut emit = |member, kind| push(node, member, kind);
        match (before, after) {
            (Node::Struct(before), Node::Struct(after)) => structure(before, after, &mut emit),
            (Node::Enum(before), Node::Enum(after)) => enumeration(before, after, &mut emit),
            _ => emit(None, ChangeKind::ChangedKind),
        }?;
    }
    Ok(findings)
}

fn member_name(group: &[std::rc::Rc<str>], name: &str) -> String {
    if group.is_empty() {
        return name.into();
    }
    format!("{}.{name}", group.join("."))
}

fn structure(
    before: &Struct,
    after: &Struct,
    emit: &mut impl FnMut(Option<String>, ChangeKind) -> Result<(), Error>,
) -> Result<(), Error> {
    if before.parameters != after.parameters {
        emit(None, ChangeKind::ChangedParameters)?;
    }
    for (path, group) in &before.groups {
        let member = || (!path.is_empty()).then(|| path.join("."));
        let Some(current) = after.groups.get(path) else {
            emit(member(), ChangeKind::RemovedMember)?;
            continue;
        };
        if group.ordinal != current.ordinal {
            emit(member(), ChangeKind::ChangedOrdinal)?;
        }
        if group.discriminant != current.discriminant
            || group
                .tag_offset
                .is_some_and(|offset| current.tag_offset != Some(offset))
        {
            emit(member(), ChangeKind::ChangedDiscriminant)?;
        }
    }
    let current_names: BTreeMap<_, _> = after
        .slots
        .iter()
        .map(|(&ordinal, slot)| ((slot.owner.as_ref(), slot.name.as_str()), ordinal))
        .collect();
    for (&ordinal, slot) in &before.slots {
        let name = || Some(member_name(&slot.owner, &slot.name));
        let current_ordinal = current_names
            .get(&(slot.owner.as_ref(), slot.name.as_str()))
            .copied()
            .unwrap_or(ordinal);
        if current_ordinal != ordinal {
            emit(name(), ChangeKind::ChangedOrdinal)?;
        }
        let Some(current) = after.slots.get(&current_ordinal) else {
            emit(name(), ChangeKind::RemovedMember)?;
            continue;
        };
        if slot.owner != current.owner {
            emit(name(), ChangeKind::ChangedGroup)?;
        }
        if slot.name != current.name {
            emit(
                name(),
                ChangeKind::RenamedMember {
                    current: current.name.clone(),
                },
            )?;
        }
        if slot.ty != current.ty {
            emit(name(), ChangeKind::ChangedType)?;
        }
        if slot.offset != current.offset {
            emit(name(), ChangeKind::ChangedSlot)?;
        }
        if slot.discriminant != current.discriminant {
            emit(name(), ChangeKind::ChangedDiscriminant)?;
        }
        if slot.default != current.default {
            emit(name(), ChangeKind::ChangedDefault)?;
        }
    }
    Ok(())
}

fn enumeration(
    before: &[String],
    after: &[String],
    emit: &mut impl FnMut(Option<String>, ChangeKind) -> Result<(), Error>,
) -> Result<(), Error> {
    let previous: std::collections::BTreeSet<_> = before.iter().collect();
    let current_ordinals: BTreeMap<_, _> = after
        .iter()
        .enumerate()
        .map(|(ordinal, name)| (name, ordinal))
        .collect();
    for (ordinal, name) in before.iter().enumerate() {
        let kind = match current_ordinals.get(name) {
            Some(&current) if current == ordinal => continue,
            Some(_) => ChangeKind::ChangedOrdinal,
            None => match after
                .get(ordinal)
                .filter(|current| !previous.contains(current))
            {
                Some(current) => ChangeKind::RenamedMember {
                    current: current.clone(),
                },
                None => ChangeKind::RemovedMember,
            },
        };
        emit(Some(name.clone()), kind)?;
    }
    Ok(())
}
