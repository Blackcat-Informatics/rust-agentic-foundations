// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Written receiver proofs retained in the binding's lexical frame.

use std::cell::RefCell;

use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::allocation::{Memory, Resident};
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
    // Source may refuse an alias. Report the refusal only at an admitted
    // method token on this exact reach.
    proof: Result<ItemId, RefCell<Option<Box<CodeResolveError>>>>,
}

#[derive(Clone, Copy)]
pub(super) enum ReceiverPath<'a> {
    Written(&'a syn::Path),
    Braced(&'a syn::Path),
    Tuple(&'a syn::Path),
    Unit(&'a syn::Path),
}

impl<'a> ReceiverPath<'a> {
    fn path(self) -> &'a syn::Path {
        match self {
            Self::Written(path) | Self::Braced(path) | Self::Tuple(path) | Self::Unit(path) => path,
        }
    }

    pub(super) fn namespace(self) -> Namespace {
        match self {
            Self::Written(_) | Self::Braced(_) => Namespace::Type,
            Self::Tuple(_) | Self::Unit(_) => Namespace::Value,
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
            Self::Unit(_) => FieldShape::Unit,
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
        let work = span
            .byte_range()
            .len()
            .saturating_add(identifier.to_string().len());
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
            let proof = match scopes
                .with_receiver_lookup(|| nominal(&lookup, scopes, receiver, workspace, limits))
            {
                Ok(Some(id)) => Ok(id),
                Ok(None) => continue,
                Err(Error::CodePath { source, .. }) => Err(RefCell::new(Some(source))),
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
                || reach.proof.as_ref().is_ok_and(|id| !types.contains(id))
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
                Ok(_) => matched.push(reach.owner),
                Err(source) => {
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
) -> Option<(&'a syn::Ident, ReceiverPath<'a>)> {
    let (pattern, ty) = match (pattern, ty) {
        (syn::Pat::Type(typed), None) => (&*typed.pat, Some(&*typed.ty)),
        (pattern, ty) => (pattern, ty),
    };
    let syn::Pat::Ident(identifier) = pattern else {
        return None;
    };
    if identifier.by_ref.is_some() || identifier.subpat.is_some() {
        return None;
    }
    let receiver = match ty {
        Some(syn::Type::Path(ty)) if ty.qself.is_none() => ReceiverPath::Written(&ty.path),
        None => constructor(initializer?)?,
        _ => return None,
    };
    receiver
        .path()
        .segments
        .iter()
        .all(|segment| segment.arguments.is_empty())
        .then_some((&identifier.ident, receiver))
}

fn constructor(initializer: &syn::Expr) -> Option<ReceiverPath<'_>> {
    use crate::syntax::inert_attributes;

    let (path, proof) = match initializer {
        syn::Expr::Struct(expr) if expr.qself.is_none() && inert_attributes(&expr.attrs) => {
            return Some(ReceiverPath::Braced(&expr.path));
        }
        syn::Expr::Call(call) if inert_attributes(&call.attrs) => {
            let syn::Expr::Path(callee) = call.func.as_ref() else {
                return None;
            };
            (callee, ReceiverPath::Tuple(&callee.path))
        }
        syn::Expr::Path(path) => (path, ReceiverPath::Unit(&path.path)),
        _ => return None,
    };
    (path.qself.is_none() && inert_attributes(&path.attrs)).then_some(proof)
}

pub(super) fn local(receiver: &syn::Expr) -> Option<String> {
    let syn::Expr::Path(receiver) = receiver else {
        return None;
    };
    if receiver.qself.is_some() {
        return None;
    }
    receiver
        .path
        .get_ident()
        .map(|ident| ident.unraw().to_string())
}
