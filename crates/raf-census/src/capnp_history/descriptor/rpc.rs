// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Admit named RPC endpoints and assign detached layouts to their owning roles.

use std::collections::{BTreeMap, BTreeSet};

use capnp::schema_capnp::{field, node};

use super::{
    Annotations, DeclarationScopes, Endpoint, Error, Method, Nodes, TypeId, layout, member_name,
};

pub(super) struct Endpoints {
    inline_owners: BTreeMap<TypeId, u8>,
    declared_inline: Option<BTreeSet<TypeId>>,
    named: BTreeSet<TypeId>,
}

impl Endpoints {
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
        })
    }

    pub(super) fn owns_inline(&self, id: TypeId) -> bool {
        self.inline_owners.contains_key(&id)
    }

    pub(super) fn methods<'a>(
        &mut self,
        nodes: &Nodes<'a>,
        reader: node::Reader<'a>,
        interface: node::interface::Reader<'a>,
        needed: &mut BTreeSet<TypeId>,
        annotations: Annotations,
        scopes: &mut DeclarationScopes,
    ) -> Result<Vec<Method>, Error> {
        let id = TypeId(reader.get_id());
        if !nongeneric(reader)? {
            return Err(layout(format!("RPC interface {id} must be nongeneric")));
        }
        if !interface.get_superclasses()?.is_empty() {
            return Err(layout(format!(
                "RPC interface {id} must have no superinterfaces"
            )));
        }
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
            if !method.get_implicit_parameters()?.is_empty()
                || !method.get_param_brand()?.get_scopes()?.is_empty()
                || !method.get_result_brand()?.get_scopes()?.is_empty()
            {
                return Err(layout(format!(
                    "{} must use nongeneric endpoints",
                    location()
                )));
            }
            let mut endpoint = |id, role| -> Result<Endpoint, Error> {
                let reader = nodes.node(id)?;
                let error =
                    |reason: &str| layout(format!("{} {role} node {id} {reason}", location()));
                let node::Struct(data) = reader.which()? else {
                    return Err(error("must be a nongroup struct"));
                };
                if data.get_is_group() {
                    return Err(error("must be a nongroup struct"));
                }
                if reader.get_scope_id() == 0 {
                    self.inline(reader, data, error)?;
                    if self.has_declaring_scope(nodes, id)? {
                        return Err(error(
                            "has a lexical declaration claim despite its zero scope",
                        ));
                    }
                    let structure = nodes.structure(id, data, needed, annotations, scopes)?;
                    return Ok(Endpoint::Inline { id, structure });
                }
                if !self.named.contains(&id) {
                    if !nodes.declaration_chain(id, scopes, nongeneric)? {
                        return Err(error("requires a reciprocal nongeneric declaration chain"));
                    }
                    self.named.insert(id);
                    needed.insert(id);
                }
                Ok(Endpoint::Named(id))
            };
            let params = endpoint(TypeId(method.get_param_struct_type()), "parameters")?;
            let results = endpoint(TypeId(method.get_result_struct_type()), "results")?;
            methods.push(Method {
                name: name.to_owned(),
                params,
                results,
            });
        }
        Ok(methods)
    }

    fn has_declaring_scope(&mut self, nodes: &Nodes<'_>, id: TypeId) -> Result<bool, Error> {
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
        error: impl FnOnce(&str) -> Error,
    ) -> Result<(), Error> {
        let id = TypeId(reader.get_id());
        if !nongeneric(reader)? || !reader.get_nested_nodes()?.is_empty() {
            return Err(error(
                "must be a detached nongeneric struct without declarations",
            ));
        }
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

fn nongeneric(reader: node::Reader<'_>) -> Result<bool, Error> {
    Ok(!reader.get_is_generic() && reader.get_parameters()?.is_empty())
}
