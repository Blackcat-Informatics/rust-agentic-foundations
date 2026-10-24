// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed signature effects from actual configured native definitions.

use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::walk::WorkList;
use raf_source::{
    CodeResolveError, CodeResolver, ItemId, Namespace, ResolveError, SymbolKind, Workspace,
};
use syn::ext::IdentExt;

use super::{Error, charge};

pub(in super::super) struct Effects {
    pub(in super::super) mutable: WorkList<bool, 8>,
    pub(in super::super) borrowed: bool,
}

impl Effects {
    pub(in super::super) fn empty() -> Self {
        Self {
            mutable: WorkList::new(),
            borrowed: false,
        }
    }
    pub(in super::super) fn push(&mut self, value: bool, job: &str) -> Result<(), Error> {
        self.mutable
            .try_push(value)
            .map_err(|source| Error::PatternStorage {
                pattern: format!("provider:{job}"),
                source,
            })
    }
}

pub(in super::super) fn signature(
    workspace: &Workspace,
    code: &CodeResolver<'_>,
    site: ItemId,
    signature: &syn::Signature,
    job: &str,
    limits: &mut MatchLimits,
) -> Result<Effects, Error> {
    if signature.variadic.is_some() {
        return Err(super::super::failure(
            job,
            "native variadic parameter effects are not modeled",
        ));
    }
    let mut effects = Effects::empty();
    for argument in &signature.inputs {
        charge(job, limits, 1)?;
        let mutable = match argument {
            syn::FnArg::Receiver(receiver) if receiver.attrs.is_empty() => match &receiver.kind {
                syn::ReceiverKind::Value => false,
                syn::ReceiverKind::Reference(_, _, mutable) => mutable.is_some(),
                _ => {
                    return Err(super::super::failure(
                        job,
                        "native typed receiver effects are not modeled",
                    ));
                }
            },
            syn::FnArg::Typed(argument) if argument.attrs.is_empty() => {
                if let syn::Type::Reference(reference) = argument.ty.as_ref() {
                    reference.mutability.is_some()
                } else {
                    owned(workspace, code, site, &argument.ty, job, limits)?;
                    false
                }
            }
            _ => {
                return Err(super::super::failure(
                    job,
                    "native attributed parameter effects are not modeled",
                ));
            }
        };
        effects.push(mutable, job)?;
    }
    if let syn::ReturnType::Type(_, output) = &signature.output {
        effects.borrowed = matches!(output.as_ref(), syn::Type::Reference(_));
        if !effects.borrowed {
            owned(workspace, code, site, output, job, limits)?;
        }
    }
    Ok(effects)
}

fn owned(
    workspace: &Workspace,
    code: &CodeResolver<'_>,
    site: ItemId,
    ty: &syn::Type,
    job: &str,
    limits: &mut MatchLimits,
) -> Result<(), Error> {
    charge(job, limits, 1)?;
    match ty {
        syn::Type::Tuple(tuple) => {
            for element in &tuple.elems {
                owned(workspace, code, site, element, job, limits)?;
            }
            Ok(())
        }
        syn::Type::Paren(node) => owned(workspace, code, site, &node.elem, job, limits),
        syn::Type::Group(node) => owned(workspace, code, site, &node.elem, job, limits),
        syn::Type::Never(_) => Ok(()),
        syn::Type::Path(path) if path.qself.is_none() && path.path.is_ident("Self") => {
            let mut found = false;
            for block in workspace.impls() {
                charge(job, limits, block.members.len() as u64 + 1)?;
                if !block.members.contains(&site) {
                    continue;
                }
                if block.self_items.is_empty()
                    || block.self_items.iter().any(|site| {
                        workspace.item(*site).is_none_or(|item| {
                            !matches!(
                                item.kind,
                                SymbolKind::Struct | SymbolKind::Enum | SymbolKind::Union
                            )
                        })
                    })
                {
                    return Err(super::super::failure(
                        job,
                        "native Self ownership is not an admitted nominal definition",
                    ));
                }
                found = true;
            }
            if found {
                Ok(())
            } else {
                Err(super::super::failure(
                    job,
                    "native Self ownership has no actual impl definition",
                ))
            }
        }
        syn::Type::Path(path)
            if path.qself.is_none()
                && path
                    .path
                    .segments
                    .iter()
                    .all(|part| matches!(part.arguments, syn::PathArguments::None)) =>
        {
            let parts: Vec<_> = path
                .path
                .segments
                .iter()
                .map(|part| part.ident.unraw().to_string())
                .collect();
            let text = crate::syntax::path_text(&parts, path.path.leading_colon.is_some());
            let owner = workspace.item(site).expect("admitted native definition");
            let resolved =
                code.resolve_from_item_at(site, owner.span.start_byte, &text, Namespace::Type);
            match resolved {
                Ok(resolved)
                    if !resolved.sites.is_empty()
                        && resolved.sites.iter().all(|site| {
                            workspace.item(*site).is_some_and(|item| {
                                matches!(
                                    item.kind,
                                    SymbolKind::Struct | SymbolKind::Enum | SymbolKind::Union
                                )
                            })
                        }) =>
                {
                    Ok(())
                }
                Err(CodeResolveError::Resolve(ResolveError::NotFound(_)))
                    if path.path.leading_colon.is_none()
                        && parts.len() == 1
                        && matches!(
                            parts[0].as_str(),
                            "bool"
                                | "char"
                                | "u8"
                                | "u16"
                                | "u32"
                                | "u64"
                                | "u128"
                                | "usize"
                                | "i8"
                                | "i16"
                                | "i32"
                                | "i64"
                                | "i128"
                                | "isize"
                                | "f32"
                                | "f64"
                        ) =>
                {
                    Ok(())
                }
                Err(source) => Err(super::super::path_error(
                    workspace, site, job, &text, source,
                )),
                _ => Err(super::super::failure(
                    job,
                    "native alias or opaque value effects are not modeled; use explicit owned nominal or primitive types",
                )),
            }
        }
        _ => Err(super::super::failure(
            job,
            "native borrowed, generic or opaque value effects are not modeled",
        )),
    }
}
