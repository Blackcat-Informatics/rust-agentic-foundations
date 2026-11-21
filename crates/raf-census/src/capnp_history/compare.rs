// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Strict history findings for declarations, members, and compiled layouts.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use raf_source::RelPath;

use super::{
    Error, TypeId,
    descriptor::{
        AnnotationTarget, Declaration, DeclarationBody, DeclarationKind, Endpoint, Interface,
        Method, Node, Schema, Struct,
    },
};

const MAX_FINDING_BYTES: usize = 2 * 1024 * 1024;

/// A change forbidden by the strict schema history policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    /// Nominal schema node that owns the history.
    pub node: TypeId,
    /// Local declaration name or qualified path of a field, enumerant or RPC method.
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
    /// A selected file has no request row or node in the current descriptor.
    RemovedFile {
        /// The canonical requested filename in the base descriptor.
        previous: RelPath,
    },
    /// A retained file ID has a different requested filename.
    RenamedFile {
        /// The canonical requested filename in the base descriptor.
        previous: RelPath,
        /// The canonical requested filename in the current descriptor.
        current: RelPath,
    },
    /// An old declaration ID left the selected files.
    RemovedDeclaration,
    /// A retained declaration ID has a different local name.
    RenamedDeclaration {
        /// The unqualified name in the current descriptor.
        current: String,
    },
    /// A retained declaration ID has a different immediate lexical owner.
    MovedDeclaration {
        /// The immediate owner in the base descriptor.
        previous: TypeId,
        /// The immediate owner in the current descriptor.
        current: TypeId,
    },
    /// A retained declaration ID has a different declared kind.
    ChangedDeclarationKind {
        /// The kind in the base descriptor.
        previous: DeclarationKind,
        /// The kind in the current descriptor.
        current: DeclarationKind,
    },
    /// A retained constant has a different declared type.
    ChangedConstantType,
    /// A retained annotation has a different declared type.
    ChangedAnnotationType,
    /// An annotation can no longer apply to a former target kind.
    RemovedAnnotationTarget {
        /// The kind no longer accepted by the annotation.
        target: AnnotationTarget,
    },
    /// A type changed between a struct, an enum and an RPC interface.
    ChangedKind,
    /// A direct superclass was removed from an RPC interface.
    RemovedSuperclass {
        /// ID of the former direct superclass.
        superclass: TypeId,
    },
    /// The count of a type's own parameters changed.
    ChangedParameters,
    /// The arity of a lexical parameter scope available to an RPC interface changed.
    ChangedEnclosingParameters {
        /// ID of the enclosing declaration.
        scope: TypeId,
    },
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
    /// A compiled default changed, or an RPC input gained or lost an explicit default.
    ChangedDefault,
    /// An appended input parameter lacks an explicit default.
    MissingParameterDefault,
    /// A pointer slot became required in an existing struct.
    RequiredField,
    /// Two pointer slots became subject to the same presence requirement.
    PairedPresence {
        /// The second field in ascending ordinal order.
        peer: String,
    },
}

impl fmt::Display for ChangeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::RemovedRoot => "selected root removed",
            Self::RemovedFile { previous } => return write!(f, "selected file {previous} removed"),
            Self::RenamedFile { previous, current } => {
                return write!(f, "selected file renamed from {previous} to {current}");
            }
            Self::RemovedDeclaration => "declaration removed from selected files",
            Self::RenamedDeclaration { current } => {
                return write!(f, "declaration renamed to {current}");
            }
            Self::MovedDeclaration { previous, current } => {
                return write!(f, "declaration moved from scope {previous} to {current}");
            }
            Self::ChangedDeclarationKind { previous, current } => {
                return write!(f, "declaration kind changed from {previous} to {current}");
            }
            Self::ChangedConstantType => "constant declared type changed",
            Self::ChangedAnnotationType => "annotation declared type changed",
            Self::RemovedAnnotationTarget { target } => {
                return write!(f, "annotation target {target} removed");
            }
            Self::ChangedKind => "schema kind changed",
            Self::RemovedSuperclass { superclass } => {
                return write!(f, "direct superclass {superclass} removed");
            }
            Self::ChangedParameters => "type parameter count changed",
            Self::ChangedEnclosingParameters { scope } => {
                return write!(f, "enclosing scope {scope} type parameter count changed");
            }
            Self::RemovedMember => "member removed",
            Self::RenamedMember { current } => return write!(f, "member renamed to {current}"),
            Self::ChangedOrdinal => "ordinal changed",
            Self::ChangedGroup => "owning group changed",
            Self::ChangedSlot => "slot offset changed",
            Self::ChangedDiscriminant => "union discriminant changed",
            Self::ChangedType => "type changed",
            Self::ChangedDefault => "default changed",
            Self::MissingParameterDefault => "appended input parameter needs an explicit default",
            Self::RequiredField => "pointer field became required",
            Self::PairedPresence { peer } => {
                return write!(f, "pointer field became paired with {peer}");
            }
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
            ChangeKind::RenamedMember { current: name }
            | ChangeKind::RenamedDeclaration { current: name }
            | ChangeKind::PairedPresence { peer: name } => name.len(),
            ChangeKind::RemovedFile { previous } => previous.as_str().len(),
            ChangeKind::RenamedFile { previous, current } => {
                previous.as_str().len() + current.as_str().len()
            }
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
    for (&id, previous) in &base.files {
        let kind = match current.files.get(&id) {
            None => ChangeKind::RemovedFile {
                previous: previous.clone(),
            },
            Some(current) if previous != current => ChangeKind::RenamedFile {
                previous: previous.clone(),
                current: current.clone(),
            },
            Some(_) => continue,
        };
        push(id, None, kind)?;
    }
    declaration_changes(&base.declarations, &current.declarations, &mut push)?;
    for (&node, before) in &base.nodes {
        let Some(after) = current.nodes.get(&node) else {
            continue;
        };
        let mut emit = |member, kind| push(node, member, kind);
        match (before, after) {
            (Node::Struct(before), Node::Struct(after)) => {
                structure(before, after, false, &mut emit)
            }
            (Node::Enum(before), Node::Enum(after)) => {
                ordinals(before, after, String::as_str, &mut emit, |_, _, _| Ok(()))
            }
            (Node::Interface(before), Node::Interface(after)) => {
                interface(before, after, &mut emit)
            }
            _ => emit(None, ChangeKind::ChangedKind),
        }?;
    }
    Ok(findings)
}

fn declaration_changes(
    base: &BTreeMap<TypeId, Declaration>,
    current: &BTreeMap<TypeId, Declaration>,
    push: &mut impl FnMut(TypeId, Option<String>, ChangeKind) -> Result<(), Error>,
) -> Result<(), Error> {
    for (&id, before) in base {
        let mut emit = |kind| push(id, Some(before.name.clone()), kind);
        let Some(after) = current.get(&id) else {
            emit(ChangeKind::RemovedDeclaration)?;
            continue;
        };
        if before.name != after.name {
            emit(ChangeKind::RenamedDeclaration {
                current: after.name.clone(),
            })?;
        }
        if before.scope != after.scope {
            emit(ChangeKind::MovedDeclaration {
                previous: before.scope,
                current: after.scope,
            })?;
        }
        let previous = before.body.kind();
        let current = after.body.kind();
        if previous != current {
            emit(ChangeKind::ChangedDeclarationKind { previous, current })?;
            continue;
        }
        match (&before.body, &after.body) {
            (DeclarationBody::Constant(before), DeclarationBody::Constant(after))
                if before != after =>
            {
                emit(ChangeKind::ChangedConstantType)?;
            }
            (
                DeclarationBody::Annotation {
                    declared_type: before_type,
                    targets: before_targets,
                },
                DeclarationBody::Annotation {
                    declared_type: after_type,
                    targets: after_targets,
                },
            ) => {
                if before_type != after_type {
                    emit(ChangeKind::ChangedAnnotationType)?;
                }
                for target in before_targets.removed_in(*after_targets) {
                    emit(ChangeKind::RemovedAnnotationTarget { target })?;
                }
            }
            _ => {}
        }
    }
    Ok(())
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
    input_defaults: bool,
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
    let previous_names: BTreeSet<_> = before
        .slots
        .values()
        .map(|slot| (slot.owner.as_ref(), slot.name.as_str()))
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
        if (slot.owner != current.owner || slot.name != current.name)
            && previous_names.contains(&(current.owner.as_ref(), current.name.as_str()))
        {
            emit(name(), ChangeKind::RemovedMember)?;
            continue;
        }
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
        if slot.default != current.default
            || (input_defaults && slot.explicit_default != current.explicit_default)
        {
            emit(name(), ChangeKind::ChangedDefault)?;
        }
    }
    for (&ordinal, slot) in &after.slots {
        if slot.required && !before.slots.get(&ordinal).is_some_and(|old| old.required) {
            emit(
                Some(member_name(&slot.owner, &slot.name)),
                ChangeKind::RequiredField,
            )?;
        }
    }
    for &(source, peer) in after.pairs.difference(&before.pairs) {
        emit(
            Some(after.slots[&source].name.clone()),
            ChangeKind::PairedPresence {
                peer: after.slots[&peer].name.clone(),
            },
        )?;
    }
    Ok(())
}

fn ordinals<T, Emit>(
    before: &[T],
    after: &[T],
    name: impl Fn(&T) -> &str,
    emit: &mut Emit,
    mut retained: impl FnMut(&T, &T, &mut Emit) -> Result<(), Error>,
) -> Result<(), Error>
where
    Emit: FnMut(Option<String>, ChangeKind) -> Result<(), Error>,
{
    let previous: BTreeSet<_> = before.iter().map(&name).collect();
    let current_ordinals: BTreeMap<_, _> = after
        .iter()
        .enumerate()
        .map(|(ordinal, member)| (name(member), ordinal))
        .collect();
    for (ordinal, member) in before.iter().enumerate() {
        let label = name(member);
        if let Some(&current) = current_ordinals.get(label) {
            if current != ordinal {
                emit(Some(label.to_owned()), ChangeKind::ChangedOrdinal)?;
            }
            retained(member, &after[current], emit)?;
            continue;
        }
        if let Some(replacement) = after
            .get(ordinal)
            .filter(|candidate| !previous.contains(name(candidate)))
        {
            emit(
                Some(label.to_owned()),
                ChangeKind::RenamedMember {
                    current: name(replacement).to_owned(),
                },
            )?;
            retained(member, replacement, emit)?;
        } else {
            emit(Some(label.to_owned()), ChangeKind::RemovedMember)?;
        }
    }
    Ok(())
}

fn interface(
    before: &Interface,
    after: &Interface,
    emit: &mut impl FnMut(Option<String>, ChangeKind) -> Result<(), Error>,
) -> Result<(), Error> {
    if before.parameters != after.parameters {
        emit(None, ChangeKind::ChangedParameters)?;
    }
    for (&scope, &count) in &before.enclosing_parameters {
        if after.enclosing_parameters.get(&scope) != Some(&count) {
            emit(None, ChangeKind::ChangedEnclosingParameters { scope })?;
        }
    }
    for &scope in after.enclosing_parameters.keys() {
        if !before.enclosing_parameters.contains_key(&scope) {
            emit(None, ChangeKind::ChangedEnclosingParameters { scope })?;
        }
    }
    for &superclass in before.superclasses.difference(&after.superclasses) {
        emit(None, ChangeKind::RemovedSuperclass { superclass })?;
    }
    ordinals(
        &before.methods,
        &after.methods,
        |method| method.name.as_str(),
        emit,
        method,
    )
}

fn method(
    old: &Method,
    new: &Method,
    emit: &mut impl FnMut(Option<String>, ChangeKind) -> Result<(), Error>,
) -> Result<(), Error> {
    for (before, after, role, input) in [
        (&old.params, &new.params, "parameters", true),
        (&old.results, &new.results, "results", false),
    ] {
        let mut emit = |member: Option<String>, kind| {
            let name = &old.name;
            let path = match member {
                Some(member) => format!("{name}.{role}.{member}"),
                None => format!("{name}.{role}"),
            };
            emit(Some(path), kind)
        };
        match (before, after) {
            (Endpoint::Named(previous), Endpoint::Named(current)) if previous == current => {}
            (
                Endpoint::Inline {
                    id: old_id,
                    structure: previous,
                },
                Endpoint::Inline {
                    id: new_id,
                    structure: current,
                },
            ) => {
                if old_id != new_id {
                    emit(None, ChangeKind::ChangedType)?;
                }
                inline_endpoint(previous, current, input, &mut emit)?;
            }
            _ => emit(None, ChangeKind::ChangedType)?,
        }
    }
    Ok(())
}

fn inline_endpoint(
    before: &Struct,
    after: &Struct,
    input: bool,
    emit: &mut impl FnMut(Option<String>, ChangeKind) -> Result<(), Error>,
) -> Result<(), Error> {
    structure(before, after, input, emit)?;
    if input {
        // Inline lists have only consecutive slots, so every ordinal
        // at or beyond the old length is an appended parameter.
        for slot in after.slots.values().skip(before.slots.len()) {
            if !slot.explicit_default {
                emit(Some(slot.name.clone()), ChangeKind::MissingParameterDefault)?;
            }
        }
    }
    Ok(())
}
