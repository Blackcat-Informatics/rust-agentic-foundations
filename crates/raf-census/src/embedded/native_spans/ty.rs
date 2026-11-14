// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Borrowed type endpoints. Empty opaque components emit no token.

use proc_macro2::Span;
use syn::punctuated::Punctuated;

use super::{Coordinates, Ends, Refusal};

#[cfg(test)]
pub(super) mod fixture;

enum Tail<'a> {
    Type(&'a syn::Type),
    Path(&'a syn::Path, Option<&'a syn::QSelf>),
    Bound(&'a syn::TypeParamBound),
    Return(&'a syn::ReturnType),
    Token(Span),
    Empty,
}

pub(in crate::embedded) fn ends(
    node: &syn::Type,
    source: &Coordinates<'_, '_>,
) -> Result<Option<Ends>, Refusal> {
    if let syn::Type::Verbatim(tokens) = node {
        return source.opaque_ends(tokens);
    }
    Ok(match (start(node, source)?, end(node, source)?) {
        (Some(first), Some(last)) => Some((first, last)),
        (None, None) => None,
        _ => {
            return Err(Refusal::Invariant(
                "type endpoints disagree on empty emission",
            ));
        }
    })
}

pub(in crate::embedded) fn start(
    node: &syn::Type,
    source: &Coordinates<'_, '_>,
) -> Result<Option<Span>, Refusal> {
    source.work(1)?;
    Ok(Some(match node {
        syn::Type::Array(n) => n.bracket_token.span.open(),
        syn::Type::Slice(n) => n.bracket_token.span.open(),
        syn::Type::Tuple(n) => n.paren_token.span.open(),
        syn::Type::Paren(n) => n.paren_token.span.open(),
        syn::Type::Group(n) => n.group_token.span,
        syn::Type::Ptr(n) => n.star_token.spans[0],
        syn::Type::Reference(n) => n.and_token.spans[0],
        syn::Type::FnPtr(n) => n
            .lifetimes
            .as_ref()
            .map(|b| b.for_token.span)
            .or_else(|| n.unsafety.as_ref().map(|t| t.span))
            .or_else(|| n.abi.as_ref().map(|a| a.extern_token.span))
            .unwrap_or(n.fn_token.span),
        syn::Type::Infer(n) => n.underscore_token.span,
        syn::Type::Never(n) => n.bang_token.spans[0],
        syn::Type::Path(n) => return path_start(&n.path, n.qself.as_ref(), source),
        syn::Type::Macro(n) => return path_start(&n.mac.path, None, source),
        syn::Type::ImplTrait(n) => n.impl_token.span,
        syn::Type::TraitObject(n) => match &n.dyn_token {
            Some(token) => token.span,
            None => return bounds_start(&n.bounds, source),
        },
        syn::Type::Verbatim(tokens) => {
            return source.opaque_ends(tokens).map(|ends| ends.map(|e| e.0));
        }
        _ => return Err(Refusal::Invariant("unknown pinned type variant")),
    }))
}

pub(in crate::embedded) fn end(
    node: &syn::Type,
    source: &Coordinates<'_, '_>,
) -> Result<Option<Span>, Refusal> {
    walk(Tail::Type(node), source)
}

pub(in crate::embedded) fn path_start(
    path: &syn::Path,
    qself: Option<&syn::QSelf>,
    source: &Coordinates<'_, '_>,
) -> Result<Option<Span>, Refusal> {
    source.work(1)?;
    Ok(qself
        .map(|q| q.lt_token.spans[0])
        .or_else(|| path.leading_colon.as_ref().map(|p| p.spans[0]))
        .or_else(|| path.segments.get(0).map(|s| s.ident.span())))
}

pub(in crate::embedded) fn path_end(
    path: &syn::Path,
    qself: Option<&syn::QSelf>,
    source: &Coordinates<'_, '_>,
) -> Result<Option<Span>, Refusal> {
    walk(Tail::Path(path, qself), source)
}

fn bounds_start(
    bounds: &Punctuated<syn::TypeParamBound, syn::Token![+]>,
    source: &Coordinates<'_, '_>,
) -> Result<Option<Span>, Refusal> {
    for pair in bounds.pairs() {
        source.work(1)?;
        if let Some(first) = bound_start(pair.value(), source)? {
            return Ok(Some(first));
        }
        if let Some(punct) = pair.punct() {
            return Ok(Some(punct.spans[0]));
        }
    }
    Ok(None)
}

fn bound_start(
    bound: &syn::TypeParamBound,
    source: &Coordinates<'_, '_>,
) -> Result<Option<Span>, Refusal> {
    source.work(1)?;
    match bound {
        syn::TypeParamBound::Trait(n) => {
            if let Some(paren) = &n.paren_token {
                return Ok(Some(paren.span.open()));
            }
            if let Some(lifetimes) = &n.lifetimes {
                return Ok(Some(lifetimes.for_token.span));
            }
            if let Some(maybe) = &n.maybe {
                return Ok(Some(maybe.spans[0]));
            }
            path_start(&n.path, None, source)
        }
        syn::TypeParamBound::Lifetime(lifetime) => Ok(Some(lifetime.apostrophe)),
        syn::TypeParamBound::PreciseCapture(capture) => Ok(Some(capture.use_token.span)),
        syn::TypeParamBound::Verbatim(tokens) => {
            source.opaque_ends(tokens).map(|ends| ends.map(|e| e.0))
        }
        _ => Err(Refusal::Invariant("unknown pinned type bound")),
    }
}

fn bounds_tail<'a>(
    bounds: &'a Punctuated<syn::TypeParamBound, syn::Token![+]>,
    source: &Coordinates<'_, '_>,
) -> Result<Tail<'a>, Refusal> {
    for pair in bounds.pairs().rev() {
        source.work(1)?;
        if let Some(punct) = pair.punct() {
            return Ok(Tail::Token(punct.spans[0]));
        }
        let bound = pair.into_value();
        if matches!(bound, syn::TypeParamBound::Verbatim(tokens) if tokens.is_empty()) {
            continue;
        }
        return Ok(Tail::Bound(bound));
    }
    Ok(Tail::Empty)
}

fn walk(mut tail: Tail<'_>, source: &Coordinates<'_, '_>) -> Result<Option<Span>, Refusal> {
    let mut prefix = None;
    loop {
        source.work(1)?;
        tail = match tail {
            Tail::Token(span) => return Ok(Some(span)),
            Tail::Empty => return Ok(prefix),
            Tail::Type(node) => type_tail(node, &mut prefix, source)?,
            Tail::Path(path, qself) => path_tail(path, qself, &mut prefix),
            Tail::Bound(bound) => bound_tail(bound, &mut prefix, source)?,
            Tail::Return(output) => match output {
                syn::ReturnType::Default => Tail::Empty,
                syn::ReturnType::Type(arrow, node) => {
                    prefix = Some(arrow.spans[1]);
                    Tail::Type(node)
                }
            },
        };
    }
}

fn type_tail<'a>(
    node: &'a syn::Type,
    prefix: &mut Option<Span>,
    source: &Coordinates<'_, '_>,
) -> Result<Tail<'a>, Refusal> {
    Ok(match node {
        syn::Type::Array(n) => Tail::Token(n.bracket_token.span.close()),
        syn::Type::Slice(n) => Tail::Token(n.bracket_token.span.close()),
        syn::Type::Tuple(n) => Tail::Token(n.paren_token.span.close()),
        syn::Type::Paren(n) => Tail::Token(n.paren_token.span.close()),
        syn::Type::Group(n) => Tail::Token(n.group_token.span),
        syn::Type::Ptr(n) => {
            *prefix = Some(match &n.mutability {
                syn::PointerMutability::Const(token) => token.span,
                syn::PointerMutability::Mut(token) => token.span,
            });
            Tail::Type(&n.elem)
        }
        syn::Type::Reference(n) => {
            *prefix = Some(
                n.mutability
                    .as_ref()
                    .map(|t| t.span)
                    .or_else(|| n.lifetime.as_ref().map(|l| l.ident.span()))
                    .unwrap_or(n.and_token.spans[0]),
            );
            Tail::Type(&n.elem)
        }
        syn::Type::FnPtr(n) => {
            *prefix = Some(n.paren_token.span.close());
            Tail::Return(&n.output)
        }
        syn::Type::Infer(n) => Tail::Token(n.underscore_token.span),
        syn::Type::Never(n) => Tail::Token(n.bang_token.spans[0]),
        syn::Type::Path(n) => Tail::Path(&n.path, n.qself.as_ref()),
        syn::Type::Macro(n) => Tail::Token(super::expression::delimiter_end(&n.mac.delimiter)),
        syn::Type::ImplTrait(n) => {
            *prefix = Some(n.impl_token.span);
            bounds_tail(&n.bounds, source)?
        }
        syn::Type::TraitObject(n) => {
            if let Some(token) = &n.dyn_token {
                *prefix = Some(token.span);
            }
            bounds_tail(&n.bounds, source)?
        }
        syn::Type::Verbatim(tokens) => match source.opaque_ends(tokens)? {
            Some((_, last)) => Tail::Token(last),
            None => Tail::Empty,
        },
        _ => return Err(Refusal::Invariant("unknown pinned type variant")),
    })
}

fn path_tail<'a>(
    path: &'a syn::Path,
    qself: Option<&syn::QSelf>,
    prefix: &mut Option<Span>,
) -> Tail<'a> {
    if let Some(qself) = qself {
        *prefix = Some(qself.gt_token.spans[0]);
    }
    if let Some(colon) = &path.leading_colon {
        *prefix = Some(colon.spans[1]);
    }
    let Some(pair) = path.segments.pairs().next_back() else {
        return Tail::Empty;
    };
    if let Some(punct) = pair.punct() {
        return Tail::Token(punct.spans[1]);
    }
    if let Some(qself) = qself
        && qself.position >= path.segments.len()
    {
        return Tail::Token(qself.gt_token.spans[0]);
    }
    let segment = pair.into_value();
    *prefix = Some(segment.ident.span());
    match &segment.arguments {
        syn::PathArguments::None => Tail::Empty,
        syn::PathArguments::AngleBracketed(args) => Tail::Token(args.gt_token.spans[0]),
        syn::PathArguments::Parenthesized(args) => {
            *prefix = Some(args.paren_token.span.close());
            Tail::Return(&args.output)
        }
    }
}

fn bound_tail<'a>(
    bound: &'a syn::TypeParamBound,
    prefix: &mut Option<Span>,
    source: &Coordinates<'_, '_>,
) -> Result<Tail<'a>, Refusal> {
    Ok(match bound {
        syn::TypeParamBound::Trait(n) => {
            if let Some(paren) = &n.paren_token {
                return Ok(Tail::Token(paren.span.close()));
            }
            if let Some(lifetimes) = &n.lifetimes {
                *prefix = Some(lifetimes.gt_token.spans[0]);
            }
            if let Some(maybe) = &n.maybe {
                *prefix = Some(maybe.spans[0]);
            }
            Tail::Path(&n.path, None)
        }
        syn::TypeParamBound::Lifetime(lifetime) => Tail::Token(lifetime.ident.span()),
        syn::TypeParamBound::PreciseCapture(capture) => Tail::Token(capture.gt_token.spans[0]),
        syn::TypeParamBound::Verbatim(tokens) => match source.opaque_ends(tokens)? {
            Some((_, last)) => Tail::Token(last),
            None => Tail::Empty,
        },
        _ => return Err(Refusal::Invariant("unknown pinned type bound")),
    })
}
