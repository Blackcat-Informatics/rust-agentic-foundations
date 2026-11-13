// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Original pattern endpoints without wrapping or cloning aliased expressions.

use proc_macro2::Span;

use super::{Coordinates, Ends, Refusal, expression, outer, ty};

#[cfg(test)]
pub(super) mod fixture;

enum Step<'a> {
    Token(Span),
    Child(&'a syn::Pat),
    Empty,
}

pub(in crate::embedded) fn ends(
    node: &syn::Pat,
    source: &Coordinates<'_, '_>,
) -> Result<Option<Ends>, Refusal> {
    if let syn::Pat::Verbatim(tokens) = node {
        return source.opaque_ends(tokens);
    }
    Ok(match (start(node, source)?, end(node, source)?) {
        (Some(first), Some(last)) => Some((first, last)),
        (None, None) => None,
        _ => {
            return Err(Refusal::Invariant(
                "pattern endpoints disagree on empty emission",
            ));
        }
    })
}

pub(in crate::embedded) fn start(
    mut node: &syn::Pat,
    source: &Coordinates<'_, '_>,
) -> Result<Option<Span>, Refusal> {
    let mut following = None;
    loop {
        source.work(1)?;
        if let Some(span) = outer(attrs(node)?, source.meter())? {
            return Ok(Some(span));
        }
        match first(node, &mut following, source)? {
            Step::Token(span) => return Ok(Some(span)),
            Step::Child(child) => node = child,
            Step::Empty => return Ok(following),
        }
    }
}

fn end(mut node: &syn::Pat, source: &Coordinates<'_, '_>) -> Result<Option<Span>, Refusal> {
    let mut preceding = None;
    loop {
        source.work(1)?;
        match last(node, &mut preceding, source)? {
            Step::Token(span) => return Ok(Some(span)),
            Step::Child(child) => node = child,
            Step::Empty => return Ok(preceding),
        }
    }
}

pub(in crate::embedded) fn arm_start(
    arm: &syn::Arm,
    source: &Coordinates<'_, '_>,
) -> Result<Option<Span>, Refusal> {
    source.work(1)?;
    if let Some(span) = outer(&arm.attrs, source.meter())? {
        return Ok(Some(span));
    }
    start(&arm.pat, source)
}

fn attrs(node: &syn::Pat) -> Result<&[syn::Attribute], Refusal> {
    Ok(match node {
        syn::Pat::Const(n) => &n.attrs,
        syn::Pat::Guard(n) => &n.attrs,
        syn::Pat::Ident(n) => &n.attrs,
        syn::Pat::Lit(n) => &n.attrs,
        syn::Pat::Macro(n) => &n.attrs,
        syn::Pat::Or(n) => &n.attrs,
        syn::Pat::Paren(n) => &n.attrs,
        syn::Pat::Path(n) => &n.attrs,
        syn::Pat::Range(n) => &n.attrs,
        syn::Pat::Reference(n) => &n.attrs,
        syn::Pat::Rest(n) => &n.attrs,
        syn::Pat::Slice(n) => &n.attrs,
        syn::Pat::Struct(n) => &n.attrs,
        syn::Pat::Tuple(n) => &n.attrs,
        syn::Pat::TupleStruct(n) => &n.attrs,
        syn::Pat::Type(n) => &n.attrs,
        syn::Pat::Verbatim(_) => &[],
        syn::Pat::Wild(n) => &n.attrs,
        _ => return Err(Refusal::Invariant("unknown pinned pattern variant")),
    })
}

fn first<'a>(
    node: &'a syn::Pat,
    following: &mut Option<Span>,
    source: &Coordinates<'_, '_>,
) -> Result<Step<'a>, Refusal> {
    Ok(match node {
        syn::Pat::Const(n) => Step::Token(n.const_token.span),
        syn::Pat::Guard(n) => {
            *following = Some(n.if_token.span);
            Step::Child(&n.pat)
        }
        syn::Pat::Ident(n) => Step::Token(
            n.by_ref
                .as_ref()
                .map(|t| t.span)
                .or_else(|| n.mutability.as_ref().map(|t| t.span))
                .unwrap_or_else(|| n.ident.span()),
        ),
        syn::Pat::Lit(n) => Step::Token(n.lit.span()),
        syn::Pat::Macro(n) => token(ty::path_start(&n.mac.path, None, source)?),
        syn::Pat::Or(n) => {
            if let Some(bar) = &n.leading_vert {
                Step::Token(bar.spans[0])
            } else {
                first_case(n, source)?
            }
        }
        syn::Pat::Paren(n) => Step::Token(n.paren_token.span.open()),
        syn::Pat::Path(n) => token(ty::path_start(&n.path, n.qself.as_ref(), source)?),
        syn::Pat::Range(n) => match &n.start {
            Some(expr) => Step::Token(expression::start(expr, source)?),
            None => Step::Token(expression::limits(&n.limits).0),
        },
        syn::Pat::Reference(n) => Step::Token(n.and_token.spans[0]),
        syn::Pat::Rest(n) => Step::Token(n.dot2_token.spans[0]),
        syn::Pat::Slice(n) => Step::Token(n.bracket_token.span.open()),
        syn::Pat::Struct(n) => token(ty::path_start(&n.path, n.qself.as_ref(), source)?),
        syn::Pat::Tuple(n) => Step::Token(n.paren_token.span.open()),
        syn::Pat::TupleStruct(n) => token(ty::path_start(&n.path, n.qself.as_ref(), source)?),
        syn::Pat::Type(n) => {
            *following = Some(n.colon_token.spans[0]);
            Step::Child(&n.pat)
        }
        syn::Pat::Verbatim(tokens) => token(source.opaque_ends(tokens)?.map(|ends| ends.0)),
        syn::Pat::Wild(n) => Step::Token(n.underscore_token.span),
        _ => return Err(Refusal::Invariant("unknown pinned pattern variant")),
    })
}

fn last<'a>(
    node: &'a syn::Pat,
    preceding: &mut Option<Span>,
    source: &Coordinates<'_, '_>,
) -> Result<Step<'a>, Refusal> {
    Ok(match node {
        syn::Pat::Const(n) => Step::Token(n.block.brace_token.span.close()),
        syn::Pat::Guard(n) => Step::Token(expression::end(&n.guard, source)?),
        syn::Pat::Ident(n) => match &n.subpat {
            Some((at, child)) => {
                *preceding = Some(at.spans[0]);
                Step::Child(child)
            }
            None => Step::Token(n.ident.span()),
        },
        syn::Pat::Lit(n) => Step::Token(n.lit.span()),
        syn::Pat::Macro(n) => Step::Token(expression::delimiter_end(&n.mac.delimiter)),
        syn::Pat::Or(n) => {
            if let Some(bar) = &n.leading_vert {
                *preceding = Some(bar.spans[0]);
            }
            last_case(n, source)?
        }
        syn::Pat::Paren(n) => Step::Token(n.paren_token.span.close()),
        syn::Pat::Path(n) => token(ty::path_end(&n.path, n.qself.as_ref(), source)?),
        syn::Pat::Range(n) => match &n.end {
            Some(expr) => Step::Token(expression::end(expr, source)?),
            None => Step::Token(expression::limits(&n.limits).1),
        },
        syn::Pat::Reference(n) => {
            *preceding = Some(
                n.mutability
                    .as_ref()
                    .map_or(n.and_token.spans[0], |t| t.span),
            );
            Step::Child(&n.pat)
        }
        syn::Pat::Rest(n) => Step::Token(n.dot2_token.spans[1]),
        syn::Pat::Slice(n) => Step::Token(n.bracket_token.span.close()),
        syn::Pat::Struct(n) => Step::Token(n.brace_token.span.close()),
        syn::Pat::Tuple(n) => Step::Token(n.paren_token.span.close()),
        syn::Pat::TupleStruct(n) => Step::Token(n.paren_token.span.close()),
        syn::Pat::Type(n) => Step::Token(ty::end(&n.ty, source)?.unwrap_or(n.colon_token.spans[0])),
        syn::Pat::Verbatim(tokens) => token(source.opaque_ends(tokens)?.map(|ends| ends.1)),
        syn::Pat::Wild(n) => Step::Token(n.underscore_token.span),
        _ => return Err(Refusal::Invariant("unknown pinned pattern variant")),
    })
}

fn token(span: Option<Span>) -> Step<'static> {
    span.map_or(Step::Empty, Step::Token)
}

fn first_case<'a>(node: &'a syn::PatOr, source: &Coordinates<'_, '_>) -> Result<Step<'a>, Refusal> {
    for pair in node.cases.pairs() {
        source.work(1)?;
        if matches!(pair.value(), syn::Pat::Verbatim(tokens) if tokens.is_empty()) {
            if let Some(bar) = pair.punct() {
                return Ok(Step::Token(bar.spans[0]));
            }
            continue;
        }
        return Ok(Step::Child(pair.into_value()));
    }
    Ok(Step::Empty)
}

fn last_case<'a>(node: &'a syn::PatOr, source: &Coordinates<'_, '_>) -> Result<Step<'a>, Refusal> {
    for pair in node.cases.pairs().rev() {
        source.work(1)?;
        if let Some(bar) = pair.punct() {
            return Ok(Step::Token(bar.spans[0]));
        }
        if matches!(pair.value(), syn::Pat::Verbatim(tokens) if tokens.is_empty()) {
            continue;
        }
        return Ok(Step::Child(pair.into_value()));
    }
    Ok(Step::Empty)
}
