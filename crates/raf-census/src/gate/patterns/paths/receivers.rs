// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Written receiver proofs retained in the binding's lexical frame.

use std::cell::RefCell;

use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::allocation::{Memory, Resident, Shared, StorageError};
use raf_source::{
    CodeResolveError, FieldShape, Item, ItemId, Namespace, SourceAuthorityId, SymbolKind, Workspace,
};
use syn::ext::IdentExt;
use syn::spanned::Spanned;

use super::{
    Compiled, Lookup, OwnerCache, PathOwner, Role, Scopes, Site, charge, first_party_nominal,
    indexed_definitions,
};
use crate::Error;

#[derive(Default)]
pub(super) struct ReceiverBinding(Vec<ReceiverReach>);

struct ReceiverReach {
    owner: ItemId,
    proof: ReceiverProof,
}

#[derive(Clone)]
enum ReceiverProof {
    Nominal(ItemId),
    // A propagated refusal remains shared with the source binding until an
    // admitted method token reports it on this exact reach.
    Refused(Shared<RefCell<Option<Box<CodeResolveError>>>>),
}

#[derive(Clone, Copy)]
pub(super) enum ReceiverPath<'a> {
    Written(&'a syn::Path),
    Braced(&'a syn::Path),
    Tuple(&'a syn::Path),
    Value(&'a syn::Path),
}

impl<'a> ReceiverPath<'a> {
    fn path(self) -> &'a syn::Path {
        match self {
            Self::Written(path) | Self::Braced(path) | Self::Tuple(path) | Self::Value(path) => {
                path
            }
        }
    }

    pub(super) fn namespace(self) -> Namespace {
        match self {
            Self::Written(_) | Self::Braced(_) => Namespace::Type,
            Self::Tuple(_) | Self::Value(_) => Namespace::Value,
        }
    }

    fn admits(self, item: &Item) -> bool {
        if !first_party_nominal(item) {
            return false;
        }
        let shape = match self {
            Self::Written(_) => return true,
            Self::Braced(_) => FieldShape::Named,
            Self::Tuple(_) => FieldShape::Tuple,
            Self::Value(_) => FieldShape::Unit,
        };
        item.kind == SymbolKind::Struct && item.shape == Some(shape)
    }
}

impl ReceiverBinding {
    pub(super) fn states(&self) -> usize {
        self.0.capacity() + 1
    }

    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(super) fn prove<'w>(
        identifier: &syn::Ident,
        receiver: ReceiverPath<'_>,
        label: &str,
        context: (&'w Workspace, &raf_source::CodeResolver<'_>, &Scopes),
        location: (&raf_source::SourceFile, &mut OwnerCache<'w>),
        limits: &mut MatchLimits,
    ) -> Result<Self, Error> {
        let (workspace, code, scopes) = context;
        let (file, cache) = location;
        let span = receiver.path().span();
        let owners = cache.get(workspace, file, span.byte_range().start, limits)?;
        scopes.admit_receiver(label, owners.len().saturating_add(1), limits)?;
        let mut reaches = Vec::new();
        Memory::new(&mut Resident)
            .reserve(&mut reaches, owners.len())
            .map_err(|source| Error::PatternIndexStorage { source })?;
        scopes.admit_receiver(label, reaches.capacity().saturating_add(1), limits)?;
        let site = Site::at(file, span, Role::ReceiverType, receiver.namespace(), false);
        let local = if let ReceiverPath::Value(path) = receiver
            && let Some(name) = path.get_ident()
        {
            let name = name.unraw().to_string();
            scopes
                .written_receiver(&name, label, limits)?
                .map(|(frame, binding)| ([name], frame, binding))
        } else {
            None
        };
        let work = span
            .byte_range()
            .len()
            .saturating_add(identifier.to_string().len())
            .saturating_add(local.as_ref().map_or(0, |(_, _, binding)| binding.0.len()));
        for owner in owners {
            charge(label, limits, work.try_into().unwrap_or(u64::MAX))?;
            let lookup = Lookup {
                code,
                owner: PathOwner::Item(owner.id),
                site: &site,
                label,
                visibility: None,
                receiver_workspace: Some(workspace),
                enum_impl: None,
            };
            let proof = match scopes.with_receiver_lookup(|| {
                if let Some((name, frame, binding)) = &local
                    && let Some(reach) = binding.0.iter().find(|reach| reach.owner == owner.id)
                    && lookup.receiver_reaches_binding(scopes, name, *frame, limits)?
                {
                    return Ok(Some(reach.proof.clone()));
                }
                // A nearer import can resolve to a unit constructor on this reach.
                nominal(&lookup, scopes, receiver, workspace, limits)
                    .map(|id| id.map(ReceiverProof::Nominal))
            }) {
                Ok(Some(proof)) => proof,
                Ok(None) => continue,
                Err(Error::CodePath { source, .. }) => {
                    ReceiverProof::Refused(Shared::try_new(RefCell::new(Some(source))).map_err(
                        |_| Error::PatternIndexStorage {
                            source: StorageError::AllocationFailed,
                        },
                    )?)
                }
                Err(error) => return Err(error),
            };
            reaches.push(ReceiverReach {
                owner: owner.id,
                proof,
            });
        }
        Ok(Self(reaches))
    }

    pub(super) fn matching_owners(
        &self,
        pattern: &Compiled<'_>,
        context: (
            &Workspace,
            &raf_source::CodeResolver<'_>,
            Option<&crate::FileCtx>,
        ),
        binding: (&Scopes, usize, &[String]),
        site: &Site<'_>,
        owners: &[&Item],
        limits: &mut MatchLimits,
    ) -> Result<Vec<ItemId>, Error> {
        let Some(types) = pattern.sites.get(&Namespace::Type) else {
            return Ok(Vec::new());
        };
        let label = pattern.pattern.id.as_str();
        let work = self.0.len().saturating_mul(
            owners
                .len()
                .saturating_add(types.len().bit_width() as usize + 1),
        );
        charge(label, limits, work.try_into().unwrap_or(u64::MAX))?;
        let (workspace, code, file_context) = context;
        let (scopes, binding_frame, name) = binding;
        let mut matched = Vec::new();
        for reach in &self.0 {
            if !owners.iter().any(|owner| owner.id == reach.owner)
                || matches!(reach.proof, ReceiverProof::Nominal(id) if !types.contains(&id))
            {
                continue;
            }
            let lookup = Lookup {
                code,
                owner: PathOwner::Item(reach.owner),
                site,
                label,
                visibility: None,
                receiver_workspace: Some(workspace),
                enum_impl: None,
            };
            if !lookup.receiver_reaches_binding(scopes, name, binding_frame, limits)? {
                continue;
            }
            match &reach.proof {
                ReceiverProof::Nominal(_) => matched.push(reach.owner),
                ReceiverProof::Refused(source) => {
                    if crate::gate::evaluate::in_scope(
                        pattern.pattern,
                        (&site.file.path, site.byte, site.line),
                        Some(std::slice::from_ref(&reach.owner)),
                        (workspace, code),
                        file_context,
                        limits,
                    )? {
                        let source = source
                            .borrow_mut()
                            .take()
                            .expect("INVARIANT: the first admitted receiver refusal ends the scan");
                        return Err(Error::CodePath {
                            pattern: label.to_owned(),
                            file: site.file.path.to_string(),
                            line: site.line,
                            source,
                        });
                    }
                }
            }
        }
        Ok(matched)
    }
}

fn nominal(
    lookup: &Lookup<'_, '_>,
    scopes: &Scopes,
    receiver: ReceiverPath<'_>,
    workspace: &Workspace,
    limits: &mut MatchLimits,
) -> Result<Option<ItemId>, Error> {
    let sites = indexed_definitions(lookup, scopes, receiver.path(), limits)?;
    let [id] = sites.as_slice() else {
        return Ok(None);
    };
    let Some(item) = workspace.item(*id) else {
        return Ok(None);
    };
    if receiver.admits(item) {
        return Ok(Some(*id));
    }
    if receiver.namespace() == Namespace::Type
        && item.kind == SymbolKind::TypeAlias
        && item.file.authority() == SourceAuthorityId::FIRST_PARTY
    {
        let target = lookup.nominal_alias(*id, limits)?;
        return Ok(
            target.filter(|&id| workspace.item(id).is_some_and(|item| receiver.admits(item)))
        );
    }
    Ok(None)
}

pub(super) fn binding<'a>(
    pattern: &'a syn::Pat,
    ty: Option<&'a syn::Type>,
    initializer: Option<&'a syn::Expr>,
    label: &str,
    limits: &mut MatchLimits,
) -> Result<Option<(&'a syn::Ident, ReceiverPath<'a>)>, Error> {
    let (pattern, ty) = match (pattern, ty) {
        (syn::Pat::Type(typed), None) => (&*typed.pat, Some(&*typed.ty)),
        (pattern, ty) => (pattern, ty),
    };
    let syn::Pat::Ident(identifier) = pattern else {
        return Ok(None);
    };
    if identifier.by_ref.is_some() || identifier.subpat.is_some() {
        return Ok(None);
    }
    let receiver = match ty {
        Some(syn::Type::Path(ty)) if ty.qself.is_none() => ReceiverPath::Written(&ty.path),
        None => {
            let Some(initializer) = initializer else {
                return Ok(None);
            };
            let Some(receiver) = inferred(initializer, label, limits)? else {
                return Ok(None);
            };
            receiver
        }
        _ => return Ok(None),
    };
    Ok(receiver
        .path()
        .segments
        .iter()
        .all(|segment| segment.arguments.is_empty())
        .then_some((&identifier.ident, receiver)))
}

fn inferred<'a>(
    initializer: &'a syn::Expr,
    label: &str,
    limits: &mut MatchLimits,
) -> Result<Option<ReceiverPath<'a>>, Error> {
    use crate::syntax::inert_attributes;

    let Some(initializer) = unparenthesized(initializer, label, limits)? else {
        return Ok(None);
    };
    let (path, proof) = match initializer {
        syn::Expr::Struct(expr) if expr.qself.is_none() && inert_attributes(&expr.attrs) => {
            return Ok(Some(ReceiverPath::Braced(&expr.path)));
        }
        syn::Expr::Call(call) if inert_attributes(&call.attrs) => {
            let Some(syn::Expr::Path(callee)) = unparenthesized(&call.func, label, limits)? else {
                return Ok(None);
            };
            (callee, ReceiverPath::Tuple(&callee.path))
        }
        syn::Expr::Path(path) => (path, ReceiverPath::Value(&path.path)),
        _ => return Ok(None),
    };
    Ok((path.qself.is_none() && inert_attributes(&path.attrs)).then_some(proof))
}

pub(super) fn local(
    receiver: &syn::Expr,
    label: &str,
    limits: &mut MatchLimits,
) -> Result<Option<String>, Error> {
    let Some(syn::Expr::Path(path)) = unparenthesized(receiver, label, limits)? else {
        return Ok(None);
    };
    if path.qself.is_some()
        || (matches!(receiver, syn::Expr::Paren(_))
            && !crate::syntax::inert_attributes(&path.attrs))
    {
        return Ok(None);
    }
    Ok(path.path.get_ident().map(|ident| ident.unraw().to_string()))
}

fn unparenthesized<'a>(
    mut expression: &'a syn::Expr,
    label: &str,
    limits: &mut MatchLimits,
) -> Result<Option<&'a syn::Expr>, Error> {
    while let syn::Expr::Paren(wrapper) = expression {
        charge(label, limits, 1)?;
        if !crate::syntax::inert_attributes(&wrapper.attrs) {
            return Ok(None);
        }
        expression = &wrapper.expr;
    }
    Ok(Some(expression))
}
