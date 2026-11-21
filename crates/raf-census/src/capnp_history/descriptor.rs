// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Admitted schema nodes, file declarations, and projected data layouts.

mod declarations;
mod files;
mod rpc;

use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    ops::Range,
    rc::Rc,
};

use capnp::{
    message::ReaderOptions,
    schema_capnp::{code_generator_request, field, node, type_, value},
    serialize,
};
use raf_source::{RelPath, Tree};

use super::{
    DescriptorError as Error, TypeId,
    capture::Capture,
    occupancy::{Occupancy, POINTER_BASE},
    shape::{self, Brand, DefaultValue, Type, TypeScope},
};

pub(super) const MAX_BYTES: u64 = 2 * 1024 * 1024;
pub(super) const MAX_WORDS: usize = 1024 * 1024;
const MAX_DEPTH: u8 = 64;
const MAX_NAME_BYTES: usize = 256;

pub(super) type GroupPath = Rc<[Rc<str>]>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Kind {
    Struct,
    Enum,
    Interface,
}

#[derive(Debug)]
pub(super) struct Schema {
    pub(super) roots: BTreeSet<TypeId>,
    pub(super) nodes: BTreeMap<TypeId, Node>,
    pub(super) files: BTreeMap<TypeId, RelPath>,
    pub(super) declarations: BTreeMap<TypeId, Declaration>,
}

use declarations::AnnotationTargets;
pub use declarations::{AnnotationTarget, DeclarationKind};

#[derive(Debug)]
pub(super) struct Declaration {
    pub(super) name: String,
    pub(super) scope: TypeId,
    pub(super) body: DeclarationBody,
}

#[derive(Debug)]
pub(super) enum DeclarationBody {
    Struct,
    Enum,
    Interface,
    Constant(Type),
    Annotation {
        declared_type: Type,
        targets: AnnotationTargets,
    },
}

impl DeclarationBody {
    pub(super) const fn kind(&self) -> DeclarationKind {
        match self {
            Self::Struct => DeclarationKind::Struct,
            Self::Enum => DeclarationKind::Enum,
            Self::Interface => DeclarationKind::Interface,
            Self::Constant(_) => DeclarationKind::Constant,
            Self::Annotation { .. } => DeclarationKind::Annotation,
        }
    }
}

#[derive(Debug)]
pub(super) enum Node {
    Struct(Struct),
    Enum(Vec<String>),
    Interface(Interface),
}

pub(super) type Superclasses = BTreeMap<TypeId, BTreeSet<Brand>>;

#[derive(Debug)]
pub(super) struct Interface {
    pub(super) parameters: u32,
    pub(super) enclosing_parameters: BTreeMap<TypeId, u32>,
    pub(super) superclasses: Superclasses,
    pub(super) methods: Vec<Method>,
}

#[derive(Debug)]
pub(super) struct Method {
    pub(super) name: String,
    pub(super) implicit_arity: u32,
    pub(super) params: Endpoint,
    pub(super) results: Endpoint,
}

#[derive(Debug)]
pub(super) enum Endpoint {
    Named(Type),
    Inline { id: TypeId, structure: Struct },
}

#[derive(Debug)]
pub(super) struct Struct {
    pub(super) parameters: u32,
    pub(super) slots: BTreeMap<u16, Slot>,
    pub(super) groups: BTreeMap<GroupPath, Group>,
    pub(super) pairs: BTreeSet<(u16, u16)>,
}

#[derive(Debug)]
pub(super) struct Group {
    pub(super) ordinal: Option<u16>,
    pub(super) discriminant: Option<u16>,
    pub(super) tag_offset: Option<u32>,
}

#[derive(Debug)]
pub(super) struct Slot {
    pub(super) owner: GroupPath,
    pub(super) name: String,
    pub(super) offset: u32,
    pub(super) discriminant: Option<u16>,
    pub(super) ty: Type,
    pub(super) default: DefaultValue,
    pub(super) explicit_default: bool,
    pub(super) required: bool,
}

pub(super) struct Nodes<'a>(BTreeMap<TypeId, node::Reader<'a>>);

type DeclarationScopes = BTreeMap<TypeId, Vec<TypeId>>;

#[derive(Clone, Copy, Default)]
pub(super) struct Annotations {
    pub(super) required: Option<TypeId>,
    pub(super) paired_presence: Option<TypeId>,
}

#[derive(Clone, Copy)]
enum AnnotationKind {
    Required,
    PairedPresence,
}

#[derive(Clone, Copy)]
pub(super) enum Roots<'a> {
    /// Every selected root and file must exist in the base.
    Require(&'a BTreeSet<TypeId>),
    /// An absent selected root or file is a finding in the current revision.
    AllowMissing(&'a BTreeSet<TypeId>),
    /// Admit all pinned declarations and project only the retained roots.
    Declared(&'a BTreeSet<TypeId>),
}

pub(super) fn read(
    bytes: &[u8],
    roots: &BTreeSet<TypeId>,
    annotations: Annotations,
    mode: Roots<'_>,
    capture: Option<(&Capture, &dyn Tree)>,
) -> Result<Schema, Error> {
    let mut limits = ReaderOptions::new();
    limits.traversal_limit_in_words(Some(MAX_WORDS));
    limits.nesting_limit(i32::from(MAX_DEPTH));
    let mut remainder = bytes;
    let message = serialize::read_message_from_flat_slice(&mut remainder, limits)?;
    if !remainder.is_empty() {
        return Err(layout(
            "expected exactly one unpacked compiler request with no trailing bytes",
        ));
    }
    let pointer = message.get_root::<capnp::any_pointer::Reader<'_>>()?;
    if pointer.is_null() {
        return Err(layout("compiler request has a null root"));
    }
    let request = pointer.get_as::<code_generator_request::Reader<'_>>()?;
    let mut nodes = BTreeMap::new();
    for node in request.get_nodes()? {
        let id = TypeId(node.get_id());
        if id.0 & (1 << 63) == 0 || nodes.insert(id, node).is_some() {
            return Err(layout(format!("invalid or repeated node ID {id}")));
        }
    }
    if let Some((capture, tree)) = capture {
        capture.verify_request(tree, request, &nodes)?;
    }
    let nodes = Nodes(nodes);
    let mut scopes = DeclarationScopes::new();
    for (id, kind) in [
        (annotations.required, AnnotationKind::Required),
        (annotations.paired_presence, AnnotationKind::PairedPresence),
    ] {
        if let Some(id) = id {
            nodes.validate_field_annotation(id, kind, &mut scopes)?;
        }
    }
    if let Roots::Declared(declared) = mode {
        for &id in declared {
            nodes.validate_persisted_root(id, &mut scopes)?;
        }
    }
    let require_roots = !matches!(mode, Roots::AllowMissing(_));
    let mut projected = nodes.project(roots, annotations, require_roots, &mut scopes)?;
    match mode {
        Roots::Declared(declared) => {
            nodes.validate_other_layouts(declared, &projected, annotations, &mut scopes)?;
        }
        Roots::Require(selected) | Roots::AllowMissing(selected) => {
            projected.files = files::project(request, &nodes, selected, require_roots)?;
            projected.declarations = declarations::project(&nodes, selected)?;
        }
    }
    Ok(projected)
}

pub(super) fn layout(message: impl Into<String>) -> Error {
    Error::Layout(message.into())
}

impl<'a> Nodes<'a> {
    fn validate_field_annotation(
        &self,
        id: TypeId,
        kind: AnnotationKind,
        scopes: &mut DeclarationScopes,
    ) -> Result<(), Error> {
        let (label, value_type) = match kind {
            AnnotationKind::Required => ("required", "Void"),
            AnnotationKind::PairedPresence => ("paired presence", "UInt16"),
        };
        let invalid = || {
            layout(format!(
                "{label} annotation {id} must declare a {value_type} field target"
            ))
        };
        let declaration = self
            .0
            .get(&id)
            .ok_or_else(|| layout(format!("missing {label} annotation {id}")))?;
        let node::Annotation(annotation) = declaration.which()? else {
            return Err(invalid());
        };
        if !annotation.get_targets_field()
            || !annotation.has_type()
            || !matches!(
                (kind, annotation.get_type()?.which()?),
                (AnnotationKind::Required, type_::Void(()))
                    | (AnnotationKind::PairedPresence, type_::Uint16(()))
            )
        {
            return Err(invalid());
        }
        if !self.is_declared(id, scopes)? {
            return Err(layout(format!(
                "{label} annotation {id} must belong to a declared schema scope"
            )));
        }
        Ok(())
    }

    fn node(&self, id: TypeId) -> Result<node::Reader<'a>, Error> {
        self.0
            .get(&id)
            .copied()
            .ok_or_else(|| layout(format!("missing schema node {id}")))
    }

    fn validate_persisted_root(
        &self,
        id: TypeId,
        scopes: &mut DeclarationScopes,
    ) -> Result<(), Error> {
        let datatype = match self.node(id)?.which()? {
            node::Struct(data) => !data.get_is_group(),
            node::Enum(_) => true,
            _ => false,
        };
        if !datatype || !self.is_declared(id, scopes)? {
            return Err(layout(format!(
                "persisted root {id} must be a declared schema struct or enum"
            )));
        }
        Ok(())
    }

    fn is_declared(&self, id: TypeId, scopes: &mut DeclarationScopes) -> Result<bool, Error> {
        self.declaration_chain(id, scopes, |_| Ok(true))
    }

    fn declaration_chain(
        &self,
        id: TypeId,
        scopes: &mut DeclarationScopes,
        mut admit: impl FnMut(node::Reader<'_>) -> Result<bool, Error>,
    ) -> Result<bool, Error> {
        // Compiler-generated RPC parameter and result structs have no declaration.
        let mut child = self.node(id)?;
        if !admit(child)? {
            return Ok(false);
        }
        for _ in 0..MAX_DEPTH {
            let parent_id = TypeId(child.get_scope_id());
            let Some(&parent) = self.0.get(&parent_id) else {
                return Ok(false);
            };
            if !admit(parent)? {
                return Ok(false);
            }
            let entries = match scopes.entry(parent_id) {
                Entry::Occupied(entries) => entries.into_mut(),
                Entry::Vacant(entry) => {
                    let mut ids: Vec<_> = parent
                        .get_nested_nodes()?
                        .iter()
                        .map(|node| TypeId(node.get_id()))
                        .collect();
                    ids.sort_unstable();
                    entry.insert(ids)
                }
            };
            let child_id = TypeId(child.get_id());
            let index = entries.partition_point(|entry| *entry < child_id);
            if entries.get(index) != Some(&child_id) || entries.get(index + 1) == Some(&child_id) {
                return Ok(false);
            }
            match parent.which()? {
                node::File(()) => return Ok(parent.get_scope_id() == 0),
                node::Interface(_) => {}
                node::Struct(data) if !data.get_is_group() => {}
                _ => return Ok(false),
            }
            child = parent;
        }
        Ok(false)
    }

    pub(super) fn require_kind(&self, id: TypeId, expected: Kind) -> Result<(), Error> {
        let actual = self.node(id)?.which()?;
        if match (actual, expected) {
            (node::Struct(value), Kind::Struct) => !value.get_is_group(),
            (node::Enum(_), Kind::Enum) | (node::Interface(_), Kind::Interface) => true,
            _ => false,
        } {
            Ok(())
        } else {
            Err(layout(format!(
                "node {id} is not a referenced {expected:?}"
            )))
        }
    }

    pub(super) fn contains_scope(&self, mut owner: TypeId, scope: TypeId) -> Result<bool, Error> {
        for _ in 0..MAX_DEPTH {
            if owner == scope {
                return Ok(true);
            }
            owner = TypeId(self.node(owner)?.get_scope_id());
            if owner.0 == 0 {
                return Ok(false);
            }
        }
        Err(layout("schema scope chain exceeds the nesting bound"))
    }

    pub(super) fn parameters(&self, scope: TypeId) -> Result<u32, Error> {
        Ok(self.node(scope)?.get_parameters()?.len())
    }

    pub(super) fn enumerant_count(&self, id: TypeId) -> Result<u32, Error> {
        let node::Enum(data) = self.node(id)?.which()? else {
            return Err(layout(format!("node {id} is not an enum")));
        };
        Ok(data.get_enumerants()?.len())
    }

    fn project(
        &self,
        roots: &BTreeSet<TypeId>,
        annotations: Annotations,
        require_roots: bool,
        scopes: &mut DeclarationScopes,
    ) -> Result<Schema, Error> {
        let mut needed = BTreeSet::new();
        let mut present = BTreeSet::new();
        for &root in roots {
            if require_roots || self.0.contains_key(&root) {
                self.node(root)?;
                needed.insert(root);
                present.insert(root);
            }
        }
        let mut nodes = BTreeMap::new();
        let mut rpc = None;
        while let Some(id) = needed.pop_first() {
            if nodes.contains_key(&id) {
                continue;
            }
            nodes.insert(
                id,
                self.project_node(id, &mut needed, annotations, scopes, &mut rpc)?,
            );
        }
        if let Some(projection) = rpc {
            projection.check_inheritance(&nodes)?;
        }
        Ok(Schema {
            roots: present,
            nodes,
            files: BTreeMap::new(),
            declarations: BTreeMap::new(),
        })
    }

    fn validate_other_layouts(
        &self,
        declared: &BTreeSet<TypeId>,
        retained: &Schema,
        annotations: Annotations,
        scopes: &mut DeclarationScopes,
    ) -> Result<(), Error> {
        let mut needed = declared.clone();
        let mut checked = BTreeSet::new();
        let mut rpc = None;
        while let Some(id) = needed.pop_first() {
            if !retained.nodes.contains_key(&id) && checked.insert(id) {
                self.project_node(id, &mut needed, annotations, scopes, &mut rpc)?;
            }
        }
        Ok(())
    }

    fn project_node(
        &self,
        id: TypeId,
        needed: &mut BTreeSet<TypeId>,
        annotations: Annotations,
        scopes: &mut DeclarationScopes,
        rpc: &mut Option<rpc::Projection>,
    ) -> Result<Node, Error> {
        let node = self.node(id)?;
        match node.which()? {
            node::Struct(data) if !data.get_is_group() => {
                if node.get_scope_id() == 0 && self.rpc_projection(rpc)?.owns_inline(id) {
                    return Err(layout(format!(
                        "detached RPC node {id} must be selected through its owning method"
                    )));
                }
                Ok(Node::Struct(self.structure(
                    id,
                    data,
                    needed,
                    annotations,
                    scopes,
                    None,
                )?))
            }
            node::Enum(data) => Ok(Node::Enum(enumerants(id, data)?)),
            node::Interface(data) => {
                let interface = self.rpc_projection(rpc)?.interface(
                    self,
                    node,
                    data,
                    needed,
                    annotations,
                    scopes,
                )?;
                Ok(Node::Interface(interface))
            }
            _ => Err(layout(format!(
                "selected node {id} must be a data struct, enum or supported RPC interface"
            ))),
        }
    }

    fn rpc_projection<'b>(
        &self,
        cache: &'b mut Option<rpc::Projection>,
    ) -> Result<&'b mut rpc::Projection, Error> {
        if let Some(projection) = cache {
            return Ok(projection);
        }
        Ok(cache.insert(rpc::Projection::read(self)?))
    }

    fn structure(
        &self,
        id: TypeId,
        reader: node::struct_::Reader<'a>,
        needed: &mut BTreeSet<TypeId>,
        annotations: Annotations,
        scopes: &mut DeclarationScopes,
        inline_type_scope: Option<TypeScope>,
    ) -> Result<Struct, Error> {
        let mut collector = Fields {
            nodes: self,
            needed,
            annotations,
            slots: BTreeMap::new(),
            pairs: BTreeSet::new(),
            groups: BTreeMap::new(),
            used_groups: BTreeSet::new(),
            ordinals: BTreeSet::new(),
            sizes: (reader.get_data_word_count(), reader.get_pointer_count()),
            inline_type_scope,
        };
        collector.group(id, reader, Rc::from([]), (None, None), false)?;
        collector.validate_peers(id)?;
        if collector
            .ordinals
            .last()
            .is_some_and(|&last| usize::from(last) + 1 != collector.ordinals.len())
        {
            return Err(layout(format!(
                "ordinals of struct {id} are not consecutive from zero"
            )));
        }
        let declared_constraint = if !collector.pairs.is_empty() {
            Some("paired presence fields")
        } else if annotations.required.is_some()
            && collector.slots.values().any(|slot| slot.required)
        {
            Some("required fields")
        } else {
            None
        };
        if let Some(constraint) = declared_constraint
            && !self.is_declared(id, scopes)?
        {
            return Err(layout(format!(
                "{constraint} must belong to a declared struct; check node {id} and its enclosing scopes"
            )));
        }
        Ok(Struct {
            parameters: self.parameters(id)?,
            slots: collector.slots,
            groups: collector.groups,
            pairs: collector.pairs,
        })
    }
}

fn enumerants(id: TypeId, data: node::enum_::Reader<'_>) -> Result<Vec<String>, Error> {
    let values = data.get_enumerants()?;
    if values.len() > u32::from(u16::MAX) + 1 {
        return Err(layout(format!("enum {id} exceeds the ordinal range")));
    }
    let mut names = Vec::new();
    let mut unique = BTreeSet::new();
    for value in values {
        let name = member_name(value.get_name()?)?;
        if !unique.insert(name) {
            return Err(layout(format!("enum {id} repeats enumerant {name}")));
        }
        names.push(name.to_owned());
    }
    Ok(names)
}

fn member_name(name: capnp::text::Reader<'_>) -> Result<&str, Error> {
    let name = name
        .to_str()
        .map_err(|error| layout(format!("invalid UTF-8 member name: {error}")))?;
    if name.len() > MAX_NAME_BYTES {
        return Err(layout("Cap'n Proto member name exceeds the 256-byte bound"));
    }
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(layout(format!("invalid Cap'n Proto member name {name:?}")));
    }
    Ok(name)
}

struct Fields<'a, 'b> {
    nodes: &'b Nodes<'a>,
    needed: &'b mut BTreeSet<TypeId>,
    annotations: Annotations,
    slots: BTreeMap<u16, Slot>,
    pairs: BTreeSet<(u16, u16)>,
    groups: BTreeMap<GroupPath, Group>,
    used_groups: BTreeSet<TypeId>,
    ordinals: BTreeSet<u16>,
    sizes: (u16, u16),
    inline_type_scope: Option<TypeScope>,
}

impl<'a> Fields<'a, '_> {
    fn field_annotations(
        &self,
        (owner, name): (TypeId, &str),
        field: field::Reader<'_>,
    ) -> Result<(bool, Option<u16>), Error> {
        let mut required = false;
        let mut peer = None;
        if self.annotations.required.is_none() && self.annotations.paired_presence.is_none() {
            return Ok((required, peer));
        }
        for annotation in field.get_annotations()? {
            let marker = TypeId(annotation.get_id());
            if self.annotations.required == Some(marker) {
                if required
                    || !annotation.has_value()
                    || !matches!(annotation.get_value()?.which()?, value::Void(()))
                {
                    return Err(layout(format!(
                        "expected one Void application of required annotation {marker} at {owner}.{name}"
                    )));
                }
                required = true;
            } else if self.annotations.paired_presence == Some(marker) {
                let invalid = || {
                    layout(format!(
                        "expected one UInt16 application of paired presence annotation {marker} at {owner}.{name}"
                    ))
                };
                if peer.is_some() || !annotation.has_value() {
                    return Err(invalid());
                }
                let value::Uint16(ordinal) = annotation.get_value()?.which()? else {
                    return Err(invalid());
                };
                peer = Some(ordinal);
            } else {
                continue;
            }
            // Annotation brands do not add persisted data types to the closure.
            shape::read_brand(
                annotation.get_brand()?,
                self.nodes,
                (
                    TypeScope::Lexical(self.inline_type_scope.map_or(owner, TypeScope::owner)),
                    marker,
                ),
                &mut BTreeSet::new(),
            )?;
        }
        Ok((required, peer))
    }

    fn validate_peers(&self, owner: TypeId) -> Result<(), Error> {
        for &(source, peer) in &self.pairs {
            let slot = &self.slots[&source];
            if !matches!(slot.ty.storage(), Some((true, _)))
                || !self.slots.get(&peer).is_some_and(|slot| {
                    slot.owner.is_empty()
                        && slot.discriminant.is_none()
                        && matches!(slot.ty.storage(), Some((true, _)))
                })
            {
                return Err(invalid_pair(owner, &slot.name, peer));
            }
        }
        Ok(())
    }

    fn group(
        &mut self,
        id: TypeId,
        reader: node::struct_::Reader<'a>,
        path: GroupPath,
        (ordinal, discriminant): (Option<u16>, Option<u16>),
        inherited_activation: bool,
    ) -> Result<Occupancy, Error> {
        if path.len() >= usize::from(MAX_DEPTH) {
            return Err(layout("schema groups exceed the nesting bound"));
        }
        let count = reader.get_discriminant_count();
        let offset = reader.get_discriminant_offset();
        if count == 1 || (count != 0 && offset >= u32::from(self.sizes.0) * 4) {
            return Err(layout(format!("invalid union tag layout in node {id}")));
        }
        let group = Group {
            ordinal,
            discriminant,
            tag_offset: (count != 0).then_some(offset),
        };
        if !self.used_groups.insert(id) {
            return Err(layout(format!("repeated group node {id}")));
        }
        let collision = |_| {
            layout(format!(
                "overlapping active fields or union tag in node {id}"
            ))
        };
        let mut occupied = Occupancy::default();
        let mut arms = Occupancy::default();
        if count != 0 {
            occupied
                .insert(offset * 16..(offset + 1) * 16, false)
                .map_err(collision)?;
        }
        let mut discriminants = BTreeSet::new();
        let mut names = BTreeSet::new();
        let conditional = inherited_activation || discriminant.is_some();
        for field in reader.get_fields()? {
            let name = member_name(field.get_name()?)?;
            if !names.insert(name) {
                return Err(layout(format!("node {id} repeats field {name}")));
            }
            let value = field.get_discriminant_value();
            let discriminant = if value == field::NO_DISCRIMINANT {
                None
            } else if value < count && discriminants.insert(value) {
                Some(value)
            } else {
                return Err(layout(format!(
                    "invalid or repeated union tag value for {id}.{name}"
                )));
            };
            let destination = if discriminant.is_some() {
                &mut arms
            } else {
                &mut occupied
            };
            self.field(
                (id, &path),
                name,
                field,
                discriminant,
                conditional,
                destination,
            )?;
        }
        if discriminants.len() != usize::from(count) {
            return Err(layout(format!("incomplete union tag values in node {id}")));
        }
        occupied.merge(arms, false).map_err(collision)?;
        self.groups.insert(path, group);
        Ok(occupied)
    }

    fn field(
        &mut self,
        (owner, path): (TypeId, &GroupPath),
        name: &str,
        field: field::Reader<'a>,
        discriminant: Option<u16>,
        inherited_activation: bool,
        occupied: &mut Occupancy,
    ) -> Result<(), Error> {
        let ordinal = match field.get_ordinal().which()? {
            field::ordinal::Explicit(value) => Some(value),
            field::ordinal::Implicit(()) => None,
        };
        let collision = |_| {
            layout(format!(
                "slot or group at {owner}.{name} overlaps active storage"
            ))
        };
        let exclusive = discriminant.is_some();
        let conditional = inherited_activation || exclusive;
        let (required, peer) = self.field_annotations((owner, name), field)?;
        match field.which()? {
            field::Slot(slot) => {
                if required && conditional {
                    return Err(invalid_required_field(owner, name));
                }
                let ordinal =
                    ordinal.ok_or_else(|| layout(format!("{owner}.{name} has no ordinal")))?;
                if let Some(peer) = peer {
                    if !path.is_empty() || conditional || ordinal >= peer {
                        return Err(invalid_pair(owner, name, peer));
                    }
                    self.pairs.insert((ordinal, peer));
                }
                self.record_ordinal(owner, name, ordinal)?;
                if let Some(span) =
                    self.slot((owner, path), (name, ordinal), slot, discriminant, required)?
                {
                    occupied.insert(span, exclusive).map_err(collision)?;
                }
            }
            field::Group(group) => {
                if let Some(peer) = peer {
                    return Err(invalid_pair(owner, name, peer));
                }
                if required {
                    return Err(invalid_required_field(owner, name));
                }
                let id = TypeId(group.get_type_id());
                let node = self.nodes.node(id)?;
                let node::Struct(overlay) = node.which()? else {
                    return Err(layout(format!(
                        "{owner}.{name} refers to non-struct group {id}"
                    )));
                };
                if !overlay.get_is_group()
                    || node.get_scope_id() != owner.0
                    || (overlay.get_data_word_count(), overlay.get_pointer_count()) != self.sizes
                {
                    return Err(layout(format!(
                        "group {id} does not share the layout of {owner}.{name}"
                    )));
                }
                if let Some(ordinal) = ordinal {
                    let count = u32::from(overlay.get_discriminant_count());
                    if count == 0 || overlay.get_fields()?.len() != count {
                        return Err(layout(format!(
                            "ordinary group {owner}.{name} cannot declare ordinal @{ordinal}"
                        )));
                    }
                    self.record_ordinal(owner, name, ordinal)?;
                }
                let mut path = path.to_vec();
                path.push(Rc::from(name));
                let footprint = self.group(
                    id,
                    overlay,
                    path.into(),
                    (ordinal, discriminant),
                    inherited_activation,
                )?;
                occupied.merge(footprint, exclusive).map_err(collision)?;
            }
        }
        Ok(())
    }

    fn record_ordinal(&mut self, owner: TypeId, name: &str, ordinal: u16) -> Result<(), Error> {
        if !self.ordinals.insert(ordinal) {
            return Err(layout(format!(
                "repeated ordinal @{ordinal} at {owner}.{name}"
            )));
        }
        Ok(())
    }

    fn slot(
        &mut self,
        (owner, path): (TypeId, &GroupPath),
        (name, ordinal): (&str, u16),
        slot: field::slot::Reader<'_>,
        discriminant: Option<u16>,
        required: bool,
    ) -> Result<Option<Range<u32>>, Error> {
        if !slot.has_type() || !slot.has_default_value() {
            return Err(layout(format!(
                "missing type or default at {owner}.{name} @{ordinal}"
            )));
        }
        let type_scope = self.inline_type_scope.unwrap_or(TypeScope::Lexical(owner));
        let ty = Type::read(slot.get_type()?, self.nodes, type_scope, self.needed)?;
        let default = shape::default(&ty, slot.get_default_value()?, self.nodes)?;
        let offset = slot.get_offset();
        let storage = ty.storage();
        if required && !matches!(storage, Some((true, _))) {
            return Err(invalid_required_field(owner, name));
        }
        let limit = match storage {
            None => 1,
            Some((true, _)) => u32::from(self.sizes.1),
            Some((false, bits)) => u32::from(self.sizes.0) * 64 / bits,
        };
        if offset >= limit {
            return Err(layout(format!(
                "slot offset out of bounds at {owner}.{name} @{ordinal}"
            )));
        }
        let slot = Slot {
            owner: Rc::clone(path),
            name: name.to_owned(),
            offset,
            discriminant,
            ty,
            default,
            explicit_default: slot.get_had_explicit_default(),
            required,
        };
        self.slots.insert(ordinal, slot);
        Ok(storage.map(|(pointer, width)| {
            let start = if pointer {
                POINTER_BASE + offset
            } else {
                offset * width
            };
            start..start + width
        }))
    }
}

fn invalid_pair(owner: TypeId, name: &str, peer: u16) -> Error {
    layout(format!(
        "paired presence at {owner}.{name} with peer @{peer} must connect two direct unconditional pointer slots in ascending ordinal order"
    ))
}

fn invalid_required_field(owner: TypeId, name: &str) -> Error {
    layout(format!(
        "required annotation at {owner}.{name} must mark an unconditional pointer slot in a declared struct"
    ))
}
