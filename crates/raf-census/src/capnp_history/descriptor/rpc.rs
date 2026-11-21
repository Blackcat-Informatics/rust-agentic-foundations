// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Admit RPC inheritance and assign endpoint layouts to their owning roles.

use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

use capnp::{
    schema_capnp::{brand, field, method, node},
    struct_list,
};

use super::{
    Annotations, DeclarationScopes, Endpoint, Error, Interface, Kind, Method, Node, Nodes, TypeId,
    layout, member_name,
};
use crate::capnp_history::shape::{self, Arguments, Pointer, Type, TypeScope};

// A present entry with no owner records an invalid or competing declaration.
type Owners = BTreeMap<TypeId, Option<TypeId>>;
type Parameters<'a> = struct_list::Reader<'a, node::parameter::Owned>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ScopePolicy {
    Nongeneric,
    LexicalGenerics,
}

#[derive(Clone, Copy)]
struct MethodOwner<'a> {
    id: TypeId,
    parameter_scopes: &'a BTreeMap<TypeId, u32>,
    implicit_arity: u32,
}

pub(super) struct Projection {
    inline_owners: BTreeMap<TypeId, u8>,
    declared_inline: Option<BTreeSet<TypeId>>,
    named: BTreeSet<TypeId>,
    owners: Option<Owners>,
    proved_declarations: BTreeSet<(TypeId, ScopePolicy)>,
}

impl Projection {
    pub(super) fn read(nodes: &Nodes<'_>) -> Result<Self, Error> {
        let mut counts = BTreeMap::<TypeId, u8>::new();
        for reader in nodes.0.values() {
            if let Ok(node::Interface(interface)) = reader.which() {
                for method in interface.get_methods()? {
                    for id in [
                        method.get_param_struct_type(),
                        method.get_result_struct_type(),
                    ] {
                        let id = TypeId(id);
                        if nodes
                            .0
                            .get(&id)
                            .is_some_and(|endpoint| endpoint.get_scope_id() == 0)
                        {
                            let count = counts.entry(id).or_default();
                            *count = count.saturating_add(1);
                        }
                    }
                }
            }
        }
        Ok(Self {
            inline_owners: counts,
            declared_inline: None,
            named: BTreeSet::new(),
            owners: None,
            proved_declarations: BTreeSet::new(),
        })
    }

    pub(super) fn owns_inline(&self, id: TypeId) -> bool {
        self.inline_owners.contains_key(&id)
    }

    pub(super) fn check_inheritance(&self, nodes: &BTreeMap<TypeId, Node>) -> Result<(), Error> {
        if self.owners.is_some() {
            check_cycles(nodes)?;
        }
        Ok(())
    }

    pub(super) fn interface<'a>(
        &mut self,
        nodes: &Nodes<'a>,
        reader: node::Reader<'a>,
        data: node::interface::Reader<'a>,
        needed: &mut BTreeSet<TypeId>,
        annotations: Annotations,
        scopes: &mut DeclarationScopes,
    ) -> Result<Interface, Error> {
        let id = TypeId(reader.get_id());
        let mut parameter_scopes = BTreeMap::new();
        let mut has_generic_flag = false;
        if !nodes.declaration_chain(id, scopes, |declaration| {
            let count = declaration.get_parameters()?.len();
            if count != 0 {
                parameter_scopes.insert(TypeId(declaration.get_id()), count);
            }
            has_generic_flag |= declaration.get_is_generic();
            Ok(true)
        })? {
            return Err(layout(format!(
                "RPC interface {id} must be declared in the schema"
            )));
        }
        if (has_generic_flag || !parameter_scopes.is_empty())
            && !self.reciprocal_declaration(nodes, id, ScopePolicy::LexicalGenerics, scopes)?
        {
            return Err(layout(format!(
                "generic RPC interface {id} requires a unique reciprocal declaration chain with consistent generic flags"
            )));
        }
        let superclasses = self.superclasses(nodes, id, data, scopes)?;
        needed.extend(superclasses.iter().copied());
        let methods = self.methods(
            nodes,
            (id, &parameter_scopes),
            data,
            needed,
            annotations,
            scopes,
        )?;
        let parameters = parameter_scopes.remove(&id).unwrap_or(0);
        Ok(Interface {
            parameters,
            enclosing_parameters: parameter_scopes,
            superclasses,
            methods,
        })
    }

    fn superclasses(
        &mut self,
        nodes: &Nodes<'_>,
        derived: TypeId,
        interface: node::interface::Reader<'_>,
        scopes: &mut DeclarationScopes,
    ) -> Result<BTreeSet<TypeId>, Error> {
        let mut bases = BTreeSet::new();
        for superclass in interface.get_superclasses()? {
            let id = TypeId(superclass.get_id());
            if !bases.insert(id) {
                return Err(layout(format!(
                    "RPC interface {derived} repeats direct superclass {id}"
                )));
            }
            if !superclass.get_brand()?.get_scopes()?.is_empty() {
                return Err(layout(format!(
                    "RPC interface {derived} superclass {id} must have an empty brand"
                )));
            }
            nodes.require_kind(id, Kind::Interface)?;
        }
        if !bases.is_empty() {
            self.require_inheritance_declaration(nodes, derived, scopes)?;
            for &id in &bases {
                self.require_inheritance_declaration(nodes, id, scopes)?;
            }
        }
        Ok(bases)
    }

    fn require_inheritance_declaration(
        &mut self,
        nodes: &Nodes<'_>,
        id: TypeId,
        scopes: &mut DeclarationScopes,
    ) -> Result<(), Error> {
        if !self.reciprocal_declaration(nodes, id, ScopePolicy::Nongeneric, scopes)? {
            return Err(layout(format!(
                "RPC inheritance interface {id} requires a reciprocal nongeneric declaration chain"
            )));
        }
        Ok(())
    }

    fn reciprocal_declaration(
        &mut self,
        nodes: &Nodes<'_>,
        id: TypeId,
        policy: ScopePolicy,
        scopes: &mut DeclarationScopes,
    ) -> Result<bool, Error> {
        if self.proved_declarations.contains(&(id, policy)) {
            return Ok(true);
        }
        let owners = match self.owners {
            Some(ref owners) => owners,
            None => self.owners.insert(lexical_owners(nodes)?),
        };
        let mut expected_parent_generic: Option<bool> = None;
        if !nodes.declaration_chain(id, scopes, |reader| {
            let parent = TypeId(reader.get_scope_id());
            let owner = owners.get(&TypeId(reader.get_id()));
            let reciprocal = if parent.0 == 0 {
                owner.is_none()
            } else {
                owner == Some(&Some(parent))
            };
            if !reciprocal {
                return Ok(false);
            }
            if policy == ScopePolicy::Nongeneric || parent.0 == 0 {
                return Ok(
                    nongeneric(reader)? && expected_parent_generic.is_none_or(|expected| !expected)
                );
            }
            let generic = reader.get_is_generic();
            let declares_parameters = !reader.get_parameters()?.is_empty();
            let agrees = expected_parent_generic.is_none_or(|expected| expected == generic);
            // An inner declaration without its own parameters inherits its parent's flag.
            expected_parent_generic = (!declares_parameters).then_some(generic);
            Ok(agrees && (!declares_parameters || generic))
        })? {
            return Ok(false);
        }
        self.proved_declarations.insert((id, policy));
        Ok(true)
    }

    fn methods<'a>(
        &mut self,
        nodes: &Nodes<'a>,
        (id, parameter_scopes): (TypeId, &BTreeMap<TypeId, u32>),
        interface: node::interface::Reader<'a>,
        needed: &mut BTreeSet<TypeId>,
        annotations: Annotations,
        scopes: &mut DeclarationScopes,
    ) -> Result<Vec<Method>, Error> {
        let list = interface.get_methods()?;
        if list.len() > u32::from(u16::MAX) + 1 {
            return Err(layout(format!(
                "RPC interface {id} exceeds the method ordinal range"
            )));
        }
        let mut names = BTreeSet::new();
        let mut methods = Vec::new();
        for (ordinal, method) in list.into_iter().enumerate() {
            let name = member_name(method.get_name()?)?;
            if !names.insert(name) {
                return Err(layout(format!("repeated RPC method {id}.{name}")));
            }
            let location = || format!("RPC interface {id} method @{ordinal} {name}");
            let implicit = implicit_parameters(method)?;
            let owner = MethodOwner {
                id,
                parameter_scopes,
                implicit_arity: implicit.len(),
            };
            let mut endpoint = |target,
                                brand: brand::Reader<'_>,
                                role|
             -> Result<Endpoint, Error> {
                let reader = nodes.node(target)?;
                let error =
                    |reason: &str| layout(format!("{} {role} node {target} {reason}", location()));
                let node::Struct(data) = reader.which()? else {
                    return Err(error("must be a nongroup struct"));
                };
                if data.get_is_group() {
                    return Err(error("must be a nongroup struct"));
                }
                if reader.get_scope_id() == 0 {
                    self.inline(
                        reader,
                        data,
                        (owner.parameter_scopes, brand),
                        implicit,
                        error,
                    )?;
                    if self.has_declaring_scope(nodes, target)? {
                        return Err(error(
                            "has a lexical declaration claim despite its zero scope",
                        ));
                    }
                    let structure = nodes.structure(
                        target,
                        data,
                        needed,
                        annotations,
                        scopes,
                        (!owner.parameter_scopes.is_empty() || owner.implicit_arity != 0)
                            .then_some(TypeScope::Inline {
                                owner: owner.id,
                                wrapper: target,
                                arity: owner.implicit_arity,
                            }),
                    )?;
                    return Ok(Endpoint::Inline {
                        id: target,
                        structure,
                    });
                }
                let endpoint = if brand.get_scopes()?.is_empty() {
                    if !nongeneric(reader)?
                        || (!self.named.contains(&target)
                            && !nodes.declaration_chain(target, scopes, nongeneric)?)
                    {
                        return Err(error("requires a reciprocal nongeneric declaration chain"));
                    }
                    Type::Named(Kind::Struct, target, BTreeMap::new())
                } else {
                    self.branded_endpoint(nodes, (owner, reader), brand, needed, scopes, error)?
                };
                if self.named.insert(target) {
                    needed.insert(target);
                }
                Ok(Endpoint::Named(endpoint))
            };
            let params = endpoint(
                TypeId(method.get_param_struct_type()),
                method.get_param_brand()?,
                "parameters",
            )?;
            let results = endpoint(
                TypeId(method.get_result_struct_type()),
                method.get_result_brand()?,
                "results",
            )?;
            methods.push(Method {
                name: name.to_owned(),
                implicit_arity: owner.implicit_arity,
                params,
                results,
            });
        }
        Ok(methods)
    }

    fn branded_endpoint(
        &mut self,
        nodes: &Nodes<'_>,
        (owner, endpoint): (MethodOwner<'_>, node::Reader<'_>),
        brand: brand::Reader<'_>,
        needed: &mut BTreeSet<TypeId>,
        scopes: &mut DeclarationScopes,
        error: impl Fn(&str) -> Error,
    ) -> Result<Type, Error> {
        let id = TypeId(endpoint.get_id());
        if !endpoint.get_is_generic() {
            return Err(error(
                "must have an empty brand for a nongeneric declaration",
            ));
        }
        if owner.parameter_scopes.is_empty()
            && !self.reciprocal_declaration(nodes, owner.id, ScopePolicy::Nongeneric, scopes)?
        {
            return Err(error(
                "requires unique reciprocal declarations in the nongeneric method owner's scopes",
            ));
        }
        if !self.reciprocal_declaration(nodes, id, ScopePolicy::LexicalGenerics, scopes)? {
            return Err(error(
                "requires unique reciprocal declarations and consistent generic flags in the endpoint's scopes",
            ));
        }
        let scope_count = brand.get_scopes()?.len();
        let brand = shape::read_brand(
            brand,
            nodes,
            (
                TypeScope::MethodBrand {
                    owner: owner.id,
                    arity: owner.implicit_arity,
                },
                id,
            ),
            needed,
        )?;
        // Canonicalization drops empty scopes, which method brands cannot carry.
        if brand.is_empty()
            || u32::try_from(brand.len()) != Ok(scope_count)
            || !complete_brand(nodes, id, &brand, owner.parameter_scopes)?
        {
            return Err(error(
                if owner.parameter_scopes.is_empty() && owner.implicit_arity == 0 {
                    "requires a complete concrete endpoint brand"
                } else {
                    "requires a complete endpoint brand in the method's parameter scopes"
                },
            ));
        }
        Ok(Type::Named(Kind::Struct, id, brand))
    }

    fn has_declaring_scope(&mut self, nodes: &Nodes<'_>, id: TypeId) -> Result<bool, Error> {
        if let Some(owners) = &self.owners {
            return Ok(owners.contains_key(&id));
        }
        if let Some(claimed) = &self.declared_inline {
            return Ok(claimed.contains(&id));
        }
        let mut claimed = BTreeSet::new();
        for reader in nodes.0.values() {
            for child in reader.get_nested_nodes()? {
                let child_id = TypeId(child.get_id());
                if self.owns_inline(child_id) {
                    claimed.insert(child_id);
                }
            }
        }
        Ok(self.declared_inline.insert(claimed).contains(&id))
    }

    fn inline(
        &self,
        reader: node::Reader<'_>,
        data: node::struct_::Reader<'_>,
        (parameter_scopes, brand): (&BTreeMap<TypeId, u32>, brand::Reader<'_>),
        implicit: Parameters<'_>,
        error: impl FnOnce(&str) -> Error,
    ) -> Result<(), Error> {
        let scopes = brand.get_scopes()?;
        let generic = !parameter_scopes.is_empty();
        if generic {
            let reason = "must inherit exactly the generic method owner's scopes";
            if usize::try_from(scopes.len()) != Ok(parameter_scopes.len()) {
                return Err(error(reason));
            }
            let mut inherited = BTreeSet::new();
            for scope in scopes {
                let id = TypeId(scope.get_scope_id());
                if !parameter_scopes.contains_key(&id)
                    || !matches!(scope.which()?, brand::scope::Inherit(()))
                    || !inherited.insert(id)
                {
                    return Err(error(reason));
                }
            }
        } else if !scopes.is_empty() {
            return Err(error("must have an empty inline endpoint brand"));
        }
        if reader.get_is_generic() != (generic || !implicit.is_empty())
            || !reader.get_nested_nodes()?.is_empty()
        {
            return Err(error(
                "must match the method's generic state and have no declarations",
            ));
        }
        let parameters = reader.get_parameters()?;
        let reason = "must declare its method's implicit parameters in order";
        if parameters.len() != implicit.len() {
            return Err(error(reason));
        }
        for (parameter, expected) in parameters.into_iter().zip(implicit) {
            if member_name(parameter.get_name()?)? != member_name(expected.get_name()?)? {
                return Err(error(reason));
            }
        }
        let id = TypeId(reader.get_id());
        if self.inline_owners.get(&id) != Some(&1) {
            return Err(error("must have exactly one owning method role"));
        }
        if data.get_discriminant_count() != 0 {
            return Err(error("must contain only inline slots"));
        }
        for (index, slot) in data.get_fields()?.into_iter().enumerate() {
            if !matches!(slot.which()?, field::Slot(_))
                || slot.get_discriminant_value() != field::NO_DISCRIMINANT
            {
                return Err(error("must contain only inline slots"));
            }
            if !matches!(slot.get_ordinal().which()?, field::ordinal::Explicit(ordinal)
                if usize::from(ordinal) == index)
            {
                return Err(error(
                    "must list inline fields in order with explicit ordinals from zero",
                ));
            }
        }
        Ok(())
    }
}

fn implicit_parameters(method: method::Reader<'_>) -> Result<Parameters<'_>, Error> {
    let parameters = method.get_implicit_parameters()?;
    for parameter in parameters {
        member_name(parameter.get_name()?)?;
    }
    Ok(parameters)
}

fn nongeneric(reader: node::Reader<'_>) -> Result<bool, Error> {
    Ok(!reader.get_is_generic() && reader.get_parameters()?.is_empty())
}

fn complete_brand(
    nodes: &Nodes<'_>,
    mut id: TypeId,
    brand: &BTreeMap<TypeId, Arguments>,
    parameter_scopes: &BTreeMap<TypeId, u32>,
) -> Result<bool, Error> {
    for _ in 0..=super::MAX_DEPTH {
        let node = nodes.node(id)?;
        let count = node.get_parameters()?.len();
        if count != 0 {
            match brand.get(&id) {
                Some(Arguments::Inherited(arity))
                    if parameter_scopes.get(&id) == Some(arity) && *arity == count => {}
                Some(Arguments::Bound(values)) if u32::try_from(values.len()) == Ok(count) => {
                    for value in values {
                        if !complete_type(nodes, value, parameter_scopes)? {
                            return Ok(false);
                        }
                    }
                }
                _ => return Ok(false),
            }
        }
        id = TypeId(node.get_scope_id());
        if id.0 == 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

fn complete_type(
    nodes: &Nodes<'_>,
    ty: &Type,
    parameter_scopes: &BTreeMap<TypeId, u32>,
) -> Result<bool, Error> {
    match ty {
        Type::Named(_, id, brand) => complete_brand(nodes, *id, brand, parameter_scopes),
        Type::List(element) => complete_type(nodes, element, parameter_scopes),
        Type::Pointer(Pointer::Any) => Ok(false),
        Type::Pointer(Pointer::Parameter(scope, _)) => Ok(parameter_scopes.contains_key(scope)),
        Type::Atom(_) | Type::Pointer(_) => Ok(true),
    }
}

fn lexical_owners(nodes: &Nodes<'_>) -> Result<Owners, Error> {
    let mut owners = Owners::new();
    for (&scope, reader) in &nodes.0 {
        let mut names = BTreeMap::new();
        for child in reader.get_nested_nodes()? {
            let id = TypeId(child.get_id());
            let Ok(name) = member_name(child.get_name()?) else {
                owners.insert(id, None);
                continue;
            };
            // Mark ambiguous claims so they refuse if a selected chain uses them.
            let previous = names.insert(name, id);
            if let Some(previous) = previous {
                owners.insert(previous, None);
            }
            match owners.entry(id) {
                Entry::Vacant(entry) => {
                    entry.insert(previous.is_none().then_some(scope));
                }
                Entry::Occupied(mut entry) => {
                    entry.insert(None);
                }
            }
        }
    }
    Ok(owners)
}

#[derive(PartialEq, Eq)]
enum Visit {
    Active,
    Complete,
}

/// The projected graph shares bases by ID. Only an edge to an active path is a cycle.
fn check_cycles(nodes: &BTreeMap<TypeId, Node>) -> Result<(), Error> {
    let mut visits = BTreeMap::new();
    let mut stack = Vec::new();
    for (&root, node) in nodes {
        let Node::Interface(interface) = node else {
            continue;
        };
        if interface.superclasses.is_empty() || visits.contains_key(&root) {
            continue;
        }
        visits.insert(root, Visit::Active);
        stack.push((root, interface.superclasses.iter()));
        while let Some((id, bases)) = stack.last_mut() {
            let Some(&base) = bases.next() else {
                visits.insert(*id, Visit::Complete);
                stack.pop();
                continue;
            };
            match visits.entry(base) {
                Entry::Occupied(entry) if *entry.get() == Visit::Active => {
                    return Err(layout(format!("RPC inheritance cycle from {id} to {base}")));
                }
                Entry::Occupied(_) => {}
                Entry::Vacant(entry) => {
                    let Some(Node::Interface(interface)) = nodes.get(&base) else {
                        return Err(layout(format!("missing projected RPC superclass {base}")));
                    };
                    entry.insert(Visit::Active);
                    stack.push((base, interface.superclasses.iter()));
                }
            }
        }
    }
    Ok(())
}
