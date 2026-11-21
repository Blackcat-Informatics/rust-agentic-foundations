// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Annotation applications in strict Cap'n Proto history.

use std::collections::{BTreeMap, BTreeSet};

use capnp::{
    schema_capnp::{annotation, node},
    struct_list,
};

use super::{AnnotationTarget, Annotations, DeclarationScopes, Error, Nodes, TypeId, layout};
use crate::capnp_history::shape::{self, Arguments, Brand, DefaultValue, Pointer, Type, TypeScope};

pub(in crate::capnp_history) type Applications = BTreeMap<TypeId, Vec<Application>>;

#[derive(Clone, Copy)]
pub(super) enum ApplicationCarrier {
    DataField(Annotations),
    Enumerant,
}

#[derive(Debug, PartialEq, Eq)]
pub(in crate::capnp_history) struct Application {
    brand: Brand,
    value: DefaultValue,
}

impl Nodes<'_> {
    pub(super) fn applications(
        &self,
        (owner, name): (TypeId, &str),
        carrier: ApplicationCarrier,
        source: struct_list::Reader<'_, annotation::Owned>,
        scopes: &mut DeclarationScopes,
    ) -> Result<Applications, Error> {
        let (target, target_name, presence) = match carrier {
            ApplicationCarrier::DataField(presence) => {
                (AnnotationTarget::Field, "data fields", presence)
            }
            ApplicationCarrier::Enumerant => (
                AnnotationTarget::Enumerant,
                "enumerants",
                Annotations::default(),
            ),
        };
        let mut applications = Applications::new();
        for application in source {
            let id = TypeId(application.get_id());
            if presence.required == Some(id) || presence.paired_presence == Some(id) {
                continue;
            }
            let at_application = |error| match error {
                Error::Layout(message) => {
                    layout(format!("annotation {id} at {owner}.{name}: {message}"))
                }
                source if matches!(carrier, ApplicationCarrier::Enumerant) => {
                    Error::EnumerantApplication {
                        owner,
                        member: name.to_owned(),
                        annotation: id,
                        source: Box::new(source),
                    }
                }
                error => error,
            };
            let node = self.node(id).map_err(at_application)?;
            let node::Annotation(declaration) =
                node.which().map_err(Error::from).map_err(at_application)?
            else {
                return Err(layout(format!(
                    "application {id} at {owner}.{name} refers to a non-annotation node"
                )));
            };
            if !target.enabled(declaration)
                || !self.is_declared(id, scopes).map_err(at_application)?
            {
                return Err(layout(format!(
                    "annotation {id} at {owner}.{name} must be declared for {target_name}"
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
                declaration
                    .get_type()
                    .map_err(Error::from)
                    .map_err(at_application)?,
                self,
                TypeScope::Lexical(id),
                &mut dependencies,
            )
            .map_err(at_application)?;
            let brand = shape::read_brand(
                application
                    .get_brand()
                    .map_err(Error::from)
                    .map_err(at_application)?,
                self,
                (TypeScope::Lexical(owner), id),
                &mut dependencies,
            )
            .map_err(at_application)?;
            let ty = applied_value_type(declared, &brand);
            let value = application
                .get_value()
                .map_err(Error::from)
                .map_err(at_application)?;
            let value = shape::default(&ty, value, self).map_err(at_application)?;
            applications
                .entry(id)
                .or_default()
                .push(Application { brand, value });
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
