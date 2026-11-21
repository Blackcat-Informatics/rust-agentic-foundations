// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Annotation applications in strict Cap'n Proto history.

use std::collections::{BTreeMap, BTreeSet};

use capnp::{
    schema_capnp::{annotation, node},
    struct_list,
};

use super::{
    AnnotationTarget, Annotations, DeclarationBody, DeclarationScopes, Error, Kind, Nodes,
    NominalApplications, Schema, TypeId, layout,
};
use crate::capnp_history::shape::{self, Arguments, Brand, DefaultValue, Pointer, Type, TypeScope};

pub(in crate::capnp_history) type Applications = BTreeMap<TypeId, Vec<Application>>;

#[derive(Clone, Copy)]
pub(super) enum ApplicationCarrier<'a> {
    DataField {
        name: &'a str,
        presence: Annotations,
    },
    Enumerant(&'a str),
    Nominal(Kind),
}

impl ApplicationCarrier<'_> {
    fn policy(self) -> (AnnotationTarget, &'static str, Annotations) {
        let (target, name) = match self {
            Self::DataField { presence, .. } => {
                return (AnnotationTarget::Field, "data fields", presence);
            }
            Self::Enumerant(_) => (AnnotationTarget::Enumerant, "enumerants"),
            Self::Nominal(Kind::Struct) => (AnnotationTarget::Struct, "struct declarations"),
            Self::Nominal(Kind::Enum) => (AnnotationTarget::Enum, "enum declarations"),
            Self::Nominal(Kind::Interface) => {
                (AnnotationTarget::Interface, "interface declarations")
            }
        };
        (target, name, Annotations::default())
    }

    fn location(self, owner: TypeId) -> String {
        match self {
            Self::DataField { name, .. } | Self::Enumerant(name) => format!("{owner}.{name}"),
            Self::Nominal(_) => owner.to_string(),
        }
    }

    fn application_error(self, owner: TypeId, annotation: TypeId, source: Error) -> Error {
        if let Error::Layout(message) = source {
            return layout(format!(
                "annotation {annotation} at {}: {message}",
                self.location(owner)
            ));
        }
        match self {
            Self::DataField { .. } => source,
            Self::Enumerant(member) => Error::EnumerantApplication {
                owner,
                member: member.to_owned(),
                annotation,
                source: Box::new(source),
            },
            Self::Nominal(_) => Error::NominalApplication {
                owner,
                annotation: Some(annotation),
                source: Box::new(source),
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(in crate::capnp_history) struct Application {
    brand: Brand,
    value: DefaultValue,
}

impl Nodes<'_> {
    pub(super) fn nominal_applications(
        &self,
        schema: &Schema,
        scopes: &mut DeclarationScopes,
    ) -> Result<BTreeMap<TypeId, NominalApplications>, Error> {
        let file_declarations = schema.declarations.iter().filter_map(|(&id, declaration)| {
            matches!(
                declaration.body,
                DeclarationBody::Struct | DeclarationBody::Enum | DeclarationBody::Interface,
            )
            .then_some(id)
        });
        let selected: BTreeSet<_> = schema
            .nodes
            .keys()
            .copied()
            .chain(file_declarations)
            .collect();
        selected
            .into_iter()
            .map(|id| {
                self.nominal_application_row(id, scopes)
                    .map(|row| (id, row))
            })
            .collect()
    }

    fn nominal_application_row(
        &self,
        owner: TypeId,
        scopes: &mut DeclarationScopes,
    ) -> Result<NominalApplications, Error> {
        let node = self.node(owner)?;
        let kind = match node.which()? {
            node::Struct(data) if !data.get_is_group() => Kind::Struct,
            node::Enum(_) => Kind::Enum,
            node::Interface(_) => Kind::Interface,
            _ => return Err(layout(format!("node {owner} is not a nominal declaration"))),
        };
        let at_owner = |source| Error::NominalApplication {
            owner,
            annotation: None,
            source: Box::new(source),
        };
        let source = node
            .get_annotations()
            .map_err(Error::from)
            .map_err(at_owner)?;
        if !source.is_empty() && !self.is_declared(owner, scopes).map_err(at_owner)? {
            return Err(layout(format!(
                "annotated declaration {owner} must have a reciprocal declaration chain"
            )));
        }
        let applications =
            self.applications(owner, ApplicationCarrier::Nominal(kind), source, scopes)?;
        Ok(NominalApplications { kind, applications })
    }

    pub(super) fn applications(
        &self,
        owner: TypeId,
        carrier: ApplicationCarrier<'_>,
        source: struct_list::Reader<'_, annotation::Owned>,
        scopes: &mut DeclarationScopes,
    ) -> Result<Applications, Error> {
        let (target, target_name, presence) = carrier.policy();
        let location = || carrier.location(owner);
        let mut applications = Applications::new();
        for application in source {
            let id = TypeId(application.get_id());
            if presence.required == Some(id) || presence.paired_presence == Some(id) {
                continue;
            }
            let at_application = |source| carrier.application_error(owner, id, source);
            let node = self.node(id).map_err(at_application)?;
            let node::Annotation(declaration) =
                node.which().map_err(Error::from).map_err(at_application)?
            else {
                return Err(layout(format!(
                    "application {id} at {} refers to a non-annotation node",
                    location()
                )));
            };
            if !target.enabled(declaration)
                || !self.is_declared(id, scopes).map_err(at_application)?
            {
                return Err(layout(format!(
                    "annotation {id} at {} must be declared for {target_name}",
                    location()
                )));
            }
            if !application.has_value() {
                return Err(layout(format!(
                    "annotation {id} at {} has no application value",
                    location()
                )));
            }
            if !declaration.has_type() {
                return Err(layout(format!(
                    "annotation {id} at {} has no declared type",
                    location()
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
