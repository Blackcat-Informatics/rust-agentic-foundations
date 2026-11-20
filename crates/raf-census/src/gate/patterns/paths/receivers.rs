// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Written receiver proofs retained in the binding's lexical frame.

use std::cell::RefCell;

use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::allocation::{Memory, Resident};
use raf_source::{
    CodeResolveError, Item, ItemId, Namespace, SourceAuthorityId, SymbolKind, Workspace,
};
use syn::ext::IdentExt;
use syn::spanned::Spanned;

use super::{
    Compiled, Lookup, PathOwner, Role, Scopes, Site, charge, first_party_nominal,
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

impl ReceiverBinding {
    pub(super) fn states(&self) -> usize {
        self.0.capacity() + 1
    }

    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(super) fn write(
        identifier: &syn::Ident,
        ty: &syn::Type,
        label: &str,
        context: (&Workspace, &raf_source::CodeResolver<'_>, &Scopes),
        location: (&raf_source::SourceFile, &[&Item]),
        limits: &mut MatchLimits,
    ) -> Result<Self, Error> {
        let (workspace, code, scopes) = context;
        let (file, owners) = location;
        let syn::Type::Path(ty) = ty else {
            return Ok(Self::default());
        };
        if ty.qself.is_some()
            || ty
                .path
                .segments
                .iter()
                .any(|segment| !segment.arguments.is_empty())
        {
            return Ok(Self::default());
        }
        scopes.admit_receiver(label, owners.len().saturating_add(1), limits)?;
        let mut reaches = Vec::new();
        Memory::new(&mut Resident)
            .reserve(&mut reaches, owners.len())
            .map_err(|source| Error::PatternIndexStorage { source })?;
        scopes.admit_receiver(label, reaches.capacity().saturating_add(1), limits)?;
        let span = ty.span();
        let site = Site::at(file, span, Role::ReceiverType, Namespace::Type, false);
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
                enum_impl: None,
            };
            let proof = match nominal(&lookup, scopes, &ty.path, workspace, limits) {
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
        let mut matched = Vec::new();
        for reach in &self.0 {
            if !owners.iter().any(|owner| owner.id == reach.owner) {
                continue;
            }
            match &reach.proof {
                Ok(id) if types.contains(id) => matched.push(reach.owner),
                Ok(_) => {}
                Err(source) => {
                    let (workspace, code, file_context) = context;
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
    path: &syn::Path,
    workspace: &Workspace,
    limits: &mut MatchLimits,
) -> Result<Option<ItemId>, Error> {
    let sites = indexed_definitions(lookup, scopes, path, limits)?;
    let [id] = sites.as_slice() else {
        return Ok(None);
    };
    let Some(item) = workspace.item(*id) else {
        return Ok(None);
    };
    if first_party_nominal(item) {
        return Ok(Some(*id));
    }
    if item.kind == SymbolKind::TypeAlias && item.file.authority() == SourceAuthorityId::FIRST_PARTY
    {
        return lookup.nominal_alias(*id, limits);
    }
    Ok(None)
}

pub(super) fn binding<'a>(
    pattern: &'a syn::Pat,
    ty: Option<&'a syn::Type>,
) -> Option<(&'a syn::Ident, &'a syn::Type)> {
    let (pattern, ty) = match (pattern, ty) {
        (syn::Pat::Type(typed), None) => (&*typed.pat, &*typed.ty),
        (pattern, Some(ty)) => (pattern, ty),
        _ => return None,
    };
    let syn::Pat::Ident(identifier) = pattern else {
        return None;
    };
    (identifier.by_ref.is_none() && identifier.subpat.is_none()).then_some((&identifier.ident, ty))
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
