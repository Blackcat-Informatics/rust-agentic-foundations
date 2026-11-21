// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Annotation applications in strict history for data fields.

use std::collections::{BTreeMap, BTreeSet};

use capnp::schema_capnp::{field, node};

use super::{Annotations, DeclarationScopes, Error, Nodes, TypeId, layout};
use crate::capnp_history::shape::{self, Arguments, Brand, DefaultValue, Pointer, Type, TypeScope};

pub(in crate::capnp_history) type FieldApplications = BTreeMap<TypeId, Vec<FieldApplication>>;

#[derive(Debug, PartialEq, Eq)]
pub(in crate::capnp_history) struct FieldApplication {
    brand: Brand,
    value: DefaultValue,
}

impl Nodes<'_> {
    pub(super) fn field_applications(
        &self,
        owner: TypeId,
        name: &str,
        field: field::Reader<'_>,
        presence: Annotations,
        scopes: &mut DeclarationScopes,
    ) -> Result<FieldApplications, Error> {
        let mut applications = FieldApplications::new();
        for application in field.get_annotations()? {
            let id = TypeId(application.get_id());
            if presence.required == Some(id) || presence.paired_presence == Some(id) {
                continue;
            }
            let node = self.node(id)?;
            let node::Annotation(declaration) = node.which()? else {
                return Err(layout(format!(
                    "application {id} at {owner}.{name} refers to a non-annotation node"
                )));
            };
            if !declaration.get_targets_field() || !self.is_declared(id, scopes)? {
                return Err(layout(format!(
                    "annotation {id} at {owner}.{name} must be declared for data fields"
                )));
            }
            if !application.has_value() {
                return Err(layout(format!(
                    "annotation {id} at {owner}.{name} has no application value"
                )));
            }
            if !declaration.has_type() {
                return Err(layout(format!(
                    "annotation {id} at {owner}.{name} has no declared type"
                )));
            }
            let mut dependencies = BTreeSet::new();
            let declared = Type::read(
                declaration.get_type()?,
                self,
                TypeScope::Lexical(id),
                &mut dependencies,
            )?;
            let brand = shape::read_brand(
                application.get_brand()?,
                self,
                (TypeScope::Lexical(owner), id),
                &mut dependencies,
            )?;
            let ty = applied_value_type(declared, &brand);
            let value =
                shape::default(&ty, application.get_value()?, self).map_err(
                    |error| match error {
                        Error::Layout(message) => {
                            layout(format!("annotation {id} at {owner}.{name}: {message}"))
                        }
                        error => error,
                    },
                )?;
            applications
                .entry(id)
                .or_default()
                .push(FieldApplication { brand, value });
        }
        Ok(applications)
    }
}

fn applied_value_type(ty: Type, brand: &Brand) -> Type {
    match ty {
        Type::Pointer(Pointer::Parameter(scope, index)) => match brand.get(&scope) {
            Some(Arguments::Inherited(_)) => ty,
            Some(Arguments::Bound(values)) => values
                .get(usize::from(index))
                .cloned()
                .unwrap_or(Type::Pointer(Pointer::Any)),
            None => Type::Pointer(Pointer::Any),
        },
        Type::List(item) => Type::List(Box::new(applied_value_type(*item, brand))),
        // A named value is classified by its nominal kind and ID. The
        // application retains its complete canonical brand separately.
        ty => ty,
    }
}
