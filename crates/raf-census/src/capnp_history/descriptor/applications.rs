// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Annotation applications in strict Cap'n Proto history.

use std::collections::{BTreeMap, BTreeSet};

use capnp::{
    schema_capnp::{annotation, node},
    struct_list,
};

use super::{
    AnnotationTarget, Annotations, ApplicationNodeKind, DeclarationKind, DeclarationScopes, Error,
    NodeApplications, Nodes, Schema, TypeId, layout,
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
    Node(ApplicationNodeKind),
}

impl ApplicationNodeKind {
    fn is_nominal(self) -> bool {
        matches!(
            self,
            Self::Declaration(
                DeclarationKind::Struct | DeclarationKind::Enum | DeclarationKind::Interface
            )
        )
    }

    fn policy(self) -> (AnnotationTarget, &'static str) {
        match self {
            Self::File => (AnnotationTarget::File, "files"),
            Self::Declaration(kind) => match kind {
                DeclarationKind::Struct => (AnnotationTarget::Struct, "struct declarations"),
                DeclarationKind::Enum => (AnnotationTarget::Enum, "enum declarations"),
                DeclarationKind::Interface => {
                    (AnnotationTarget::Interface, "interface declarations")
                }
                DeclarationKind::Constant => (AnnotationTarget::Const, "constant declarations"),
                DeclarationKind::Annotation => {
                    (AnnotationTarget::Annotation, "annotation declarations")
                }
            },
        }
    }

    fn application_error(self, owner: TypeId, annotation: Option<TypeId>, source: Error) -> Error {
        if self.is_nominal() {
            Error::NominalApplication {
                owner,
                annotation,
                source: Box::new(source),
            }
        } else {
            Error::NodeApplication {
                owner,
                annotation,
                source: Box::new(source),
            }
        }
    }
}

impl ApplicationCarrier<'_> {
    fn policy(self) -> (AnnotationTarget, &'static str, Annotations) {
        let (target, name) = match self {
            Self::DataField { presence, .. } => {
                return (AnnotationTarget::Field, "data fields", presence);
            }
            Self::Enumerant(_) => (AnnotationTarget::Enumerant, "enumerants"),
            Self::Node(kind) => kind.policy(),
        };
        (target, name, Annotations::default())
    }

    fn location(self, owner: TypeId) -> String {
        match self {
            Self::DataField { name, .. } | Self::Enumerant(name) => format!("{owner}.{name}"),
            Self::Node(_) => owner.to_string(),
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
            Self::Node(kind) => kind.application_error(owner, Some(annotation), source),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(in crate::capnp_history) struct Application {
    brand: Brand,
    value: DefaultValue,
}

impl Nodes<'_> {
    pub(super) fn node_applications(
        &self,
        schema: &Schema,
        scopes: &mut DeclarationScopes,
    ) -> Result<BTreeMap<TypeId, NodeApplications>, Error> {
        let selected: BTreeSet<_> = schema
            .nodes
            .keys()
            .chain(schema.files.keys())
            .chain(schema.declarations.keys())
            .copied()
            .collect();
        selected
            .into_iter()
            .map(|id| self.node_application_row(id, scopes).map(|row| (id, row)))
            .collect()
    }

    fn node_application_row(
        &self,
        owner: TypeId,
        scopes: &mut DeclarationScopes,
    ) -> Result<NodeApplications, Error> {
        let node = self.node(owner)?;
        let kind = match node.which()? {
            node::File(()) => ApplicationNodeKind::File,
            node::Struct(data) if !data.get_is_group() => {
                ApplicationNodeKind::Declaration(DeclarationKind::Struct)
            }
            node::Enum(_) => ApplicationNodeKind::Declaration(DeclarationKind::Enum),
            node::Interface(_) => ApplicationNodeKind::Declaration(DeclarationKind::Interface),
            node::Const(_) => ApplicationNodeKind::Declaration(DeclarationKind::Constant),
            node::Annotation(_) => ApplicationNodeKind::Declaration(DeclarationKind::Annotation),
            node::Struct(_) => {
                return Err(layout(format!(
                    "node {owner} is not a written schema declaration"
                )));
            }
        };
        let at_owner = |source| kind.application_error(owner, None, source);
        let source = node
            .get_annotations()
            .map_err(Error::from)
            .map_err(at_owner)?;
        if !source.is_empty() {
            let declared = match kind {
                ApplicationNodeKind::File => {
                    no_own_parameters(node).map_err(at_owner)?;
                    true
                }
                kind if kind.is_nominal() => self.is_declared(owner, scopes).map_err(at_owner)?,
                ApplicationNodeKind::Declaration(_) => self
                    .application_declaration_chain(owner, scopes)
                    .map_err(at_owner)?,
            };
            if !declared {
                return Err(layout(format!(
                    "annotated declaration {owner} must have a reciprocal declaration chain"
                )));
            }
        }
        let applications =
            self.applications(owner, ApplicationCarrier::Node(kind), source, scopes)?;
        Ok(NodeApplications { kind, applications })
    }

    fn application_declaration_chain(
        &self,
        id: TypeId,
        scopes: &mut DeclarationScopes,
    ) -> Result<bool, Error> {
        self.declaration_chain(id, scopes, |node| {
            if matches!(
                node.which()?,
                node::File(()) | node::Const(_) | node::Annotation(_)
            ) {
                no_own_parameters(node)?;
            }
            Ok(true)
        })
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
            let mut is_declared = || match carrier {
                ApplicationCarrier::Node(kind) if !kind.is_nominal() => {
                    self.application_declaration_chain(id, scopes)
                }
                _ => self.is_declared(id, scopes),
            };
            if !target.enabled(declaration) || !is_declared().map_err(at_application)? {
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

fn no_own_parameters(node: node::Reader<'_>) -> Result<(), Error> {
    if !node.get_parameters()?.is_empty() {
        let id = TypeId(node.get_id());
        return Err(layout(format!(
            "file or scalar declaration {id} cannot declare type parameters"
        )));
    }
    Ok(())
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
