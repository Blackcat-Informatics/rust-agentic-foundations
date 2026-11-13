// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Stored source endpoints with explicit native opaque-stream boundaries.

use proc_macro2::Span;

use super::coordinates::Coordinates;
use super::resource::{Meter, Refusal};

pub(super) mod expression;
pub(super) mod pattern;
pub(super) mod statement;
pub(super) mod ty;

#[cfg(test)]
pub(super) mod fixture;

pub(super) type Ends = (Span, Span);

pub(super) fn argument_start(
    argument: &syn::GenericArgument,
    source: &Coordinates<'_, '_>,
) -> Result<Span, Refusal> {
    source.work(1)?;
    match argument {
        syn::GenericArgument::Lifetime(lifetime) => Ok(lifetime.apostrophe),
        syn::GenericArgument::Type(node) => {
            ty::start(node, source).map(|span| span.unwrap_or_else(Span::call_site))
        }
        syn::GenericArgument::Const(node) => expression::start(node, source),
        syn::GenericArgument::AssocType(node) => Ok(node.ident.span()),
        syn::GenericArgument::AssocConst(node) => Ok(node.ident.span()),
        syn::GenericArgument::Constraint(node) => Ok(node.ident.span()),
        _ => Err(Refusal::Invariant("unknown pinned generic argument")),
    }
}

pub(super) fn generic_start(parameter: &syn::GenericParam, meter: &Meter) -> Result<Span, Refusal> {
    meter.work(1)?;
    let (attrs, first) = match parameter {
        syn::GenericParam::Const(parameter) => (&parameter.attrs, parameter.const_token.span),
        syn::GenericParam::Type(parameter) => (&parameter.attrs, parameter.ident.span()),
        syn::GenericParam::Lifetime(parameter) => (&parameter.attrs, parameter.lifetime.apostrophe),
    };
    Ok(outer(attrs, meter)?.unwrap_or(first))
}

pub(super) fn local_start(local: &syn::Local, meter: &Meter) -> Result<Span, Refusal> {
    meter.work(1)?;
    Ok(outer(&local.attrs, meter)?.unwrap_or(local.let_token.span))
}

pub(super) fn field_start(field: &syn::FieldValue, meter: &Meter) -> Result<Span, Refusal> {
    meter.work(1)?;
    let member = match &field.member {
        syn::Member::Named(ident) => ident.span(),
        syn::Member::Unnamed(index) => index.span,
    };
    Ok(outer(&field.attrs, meter)?.unwrap_or(member))
}

/// A typed item's first printed token. Verbatim streams retain their native
/// lookup path: the public stream API has no borrowed first-token access.
pub(super) fn item_start(item: &syn::Item, meter: &Meter) -> Result<Option<Span>, Refusal> {
    meter.work(1)?;
    let (attrs, vis, keyword) = match item {
        syn::Item::Const(item) => (&item.attrs, Some(&item.vis), item.const_token.span),
        syn::Item::Enum(item) => (&item.attrs, Some(&item.vis), item.enum_token.span),
        syn::Item::ExternCrate(item) => (&item.attrs, Some(&item.vis), item.extern_token.span),
        syn::Item::Fn(item) => (&item.attrs, Some(&item.vis), signature_start(&item.sig)),
        syn::Item::ForeignMod(item) => (
            &item.attrs,
            None,
            item.unsafety
                .as_ref()
                .map_or(item.abi.extern_token.span, |token| token.span),
        ),
        syn::Item::Impl(item) => (
            &item.attrs,
            None,
            item.modifiers
                .defaultness
                .as_ref()
                .map(|token| token.span)
                .or_else(|| item.unsafety.as_ref().map(|token| token.span))
                .unwrap_or(item.impl_token.span),
        ),
        syn::Item::Macro(item) => {
            let first = if let Some(colon) = &item.mac.path.leading_colon {
                colon.spans[0]
            } else {
                item.mac
                    .path
                    .segments
                    .get(0)
                    .ok_or(Refusal::Invariant("parsed item macro lost its path"))?
                    .ident
                    .span()
            };
            (&item.attrs, None, first)
        }
        syn::Item::Mod(item) => (
            &item.attrs,
            Some(&item.vis),
            item.unsafety
                .as_ref()
                .map_or(item.mod_token.span, |token| token.span),
        ),
        syn::Item::Static(item) => (&item.attrs, Some(&item.vis), item.static_token.span),
        syn::Item::Struct(item) => (&item.attrs, Some(&item.vis), item.struct_token.span),
        syn::Item::Trait(item) => (
            &item.attrs,
            Some(&item.vis),
            item.unsafety
                .as_ref()
                .map(|token| token.span)
                .or_else(|| item.modifiers.auto_token.as_ref().map(|token| token.span))
                .unwrap_or(item.trait_token.span),
        ),
        syn::Item::TraitAlias(item) => (&item.attrs, Some(&item.vis), item.trait_token.span),
        syn::Item::Type(item) => (&item.attrs, Some(&item.vis), item.type_token.span),
        syn::Item::Union(item) => (&item.attrs, Some(&item.vis), item.union_token.span),
        syn::Item::Use(item) => (&item.attrs, Some(&item.vis), item.use_token.span),
        syn::Item::Verbatim(_) => return Ok(None),
        _ => {
            return Err(Refusal::Invariant(
                "the pinned parser produced an unknown item variant",
            ));
        }
    };
    Ok(Some(
        outer(attrs, meter)?
            .or_else(|| vis.and_then(visibility))
            .unwrap_or(keyword),
    ))
}

fn outer(attrs: &[syn::Attribute], meter: &Meter) -> Result<Option<Span>, Refusal> {
    for attribute in attrs {
        meter.work(1)?;
        if matches!(attribute.style, syn::AttrStyle::Outer) {
            return Ok(Some(attribute.pound_token.spans[0]));
        }
    }
    Ok(None)
}

fn visibility(visibility: &syn::Visibility) -> Option<Span> {
    match visibility {
        syn::Visibility::Public(token) => Some(token.span),
        syn::Visibility::Restricted(visibility) => Some(visibility.pub_token.span),
        syn::Visibility::Inherited => None,
    }
}

/// Follow the pinned signature printer's first-token order only. No signature
/// end is inferred from the body, return type, where clause or source trivia.
pub(super) fn signature_start(signature: &syn::Signature) -> Span {
    if let Some(token) = &signature.constness {
        return token.span;
    }
    if let Some(token) = &signature.asyncness {
        return token.span;
    }
    match &signature.safety {
        syn::Safety::Safe(token) => return token.span,
        syn::Safety::Unsafe(token) => return token.span,
        syn::Safety::Default => {}
    }
    signature
        .abi
        .as_ref()
        .map_or(signature.fn_token.span, |abi| abi.extern_token.span)
}

pub(super) fn function(function: &syn::ItemFn, meter: &Meter) -> Result<Ends, Refusal> {
    meter.work(1)?;
    let first = outer(&function.attrs, meter)?
        .or_else(|| visibility(&function.vis))
        .unwrap_or_else(|| signature_start(&function.sig));
    // Free-function printing does not emit FnModifiers::defaultness.
    Ok((first, function.block.brace_token.span.join()))
}

pub(super) fn method(function: &syn::ImplItemFn, meter: &Meter) -> Result<Ends, Refusal> {
    meter.work(1)?;
    let first = outer(&function.attrs, meter)?
        .or_else(|| visibility(&function.vis))
        .or_else(|| {
            function
                .modifiers
                .defaultness
                .as_ref()
                .map(|token| token.span)
        })
        .unwrap_or_else(|| signature_start(&function.sig));
    Ok((first, function.block.brace_token.span.join()))
}

pub(super) fn trait_function(function: &syn::TraitItemFn, meter: &Meter) -> Result<Ends, Refusal> {
    meter.work(1)?;
    let first = outer(&function.attrs, meter)?.unwrap_or_else(|| signature_start(&function.sig));
    let last = if let Some(body) = &function.default {
        body.brace_token.span.join()
    } else {
        function
            .semi_token
            .as_ref()
            .ok_or(Refusal::Invariant(
                "parsed trait function lost its semicolon",
            ))?
            .spans[0]
    };
    Ok((first, last))
}

pub(super) fn module(module: &syn::ItemMod, meter: &Meter) -> Result<Ends, Refusal> {
    meter.work(1)?;
    let first = outer(&module.attrs, meter)?
        .or_else(|| visibility(&module.vis))
        .or_else(|| module.unsafety.as_ref().map(|token| token.span))
        .unwrap_or(module.mod_token.span);
    let last = if let Some((brace, _)) = &module.content {
        brace.span.join()
    } else {
        module
            .semi
            .as_ref()
            .ok_or(Refusal::Invariant(
                "parsed outline module lost its semicolon",
            ))?
            .spans[0]
    };
    Ok((first, last))
}
