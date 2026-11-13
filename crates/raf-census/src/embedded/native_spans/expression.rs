// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Original source extents, independent of synthetic printing groups.

use proc_macro2::Span;

use super::{Coordinates, Ends, Refusal, outer};

enum Step<'a> {
    Token(Span),
    Child(&'a syn::Expr),
}

pub(in crate::embedded) fn ends(
    node: &syn::Expr,
    source: &Coordinates<'_, '_>,
) -> Result<Ends, Refusal> {
    if let syn::Expr::Verbatim(tokens) = node {
        return source.opaque_ends(tokens).map(|ends| {
            ends.unwrap_or_else(|| {
                let empty = Span::call_site();
                (empty, empty)
            })
        });
    }
    Ok((start(node, source)?, end(node, source)?))
}

pub(in crate::embedded) fn start(
    mut node: &syn::Expr,
    source: &Coordinates<'_, '_>,
) -> Result<Span, Refusal> {
    loop {
        source.work(1)?;
        if let Some(span) = outer(attrs(node)?, source.meter())? {
            return Ok(span);
        }
        match first(node, source)? {
            Step::Token(span) => return Ok(span),
            Step::Child(child) => node = child,
        }
    }
}

pub(in crate::embedded) fn end(
    mut node: &syn::Expr,
    source: &Coordinates<'_, '_>,
) -> Result<Span, Refusal> {
    loop {
        source.work(1)?;
        match last(node, source)? {
            Step::Token(span) => return Ok(span),
            Step::Child(child) => node = child,
        }
    }
}

fn attrs(node: &syn::Expr) -> Result<&[syn::Attribute], Refusal> {
    Ok(match node {
        syn::Expr::Array(n) => &n.attrs,
        syn::Expr::Assign(n) => &n.attrs,
        syn::Expr::Async(n) => &n.attrs,
        syn::Expr::Await(n) => &n.attrs,
        syn::Expr::Binary(n) => &n.attrs,
        syn::Expr::Block(n) => &n.attrs,
        syn::Expr::Break(n) => &n.attrs,
        syn::Expr::Call(n) => &n.attrs,
        syn::Expr::Cast(n) => &n.attrs,
        syn::Expr::Closure(n) => &n.attrs,
        syn::Expr::Const(n) => &n.attrs,
        syn::Expr::Continue(n) => &n.attrs,
        syn::Expr::Field(n) => &n.attrs,
        syn::Expr::ForLoop(n) => &n.attrs,
        syn::Expr::Group(n) => &n.attrs,
        syn::Expr::If(n) => &n.attrs,
        syn::Expr::Index(n) => &n.attrs,
        syn::Expr::Infer(n) => &n.attrs,
        syn::Expr::Let(n) => &n.attrs,
        syn::Expr::Lit(n) => &n.attrs,
        syn::Expr::Loop(n) => &n.attrs,
        syn::Expr::Macro(n) => &n.attrs,
        syn::Expr::Match(n) => &n.attrs,
        syn::Expr::MethodCall(n) => &n.attrs,
        syn::Expr::Paren(n) => &n.attrs,
        syn::Expr::Path(n) => &n.attrs,
        syn::Expr::Range(n) => &n.attrs,
        syn::Expr::RawAddr(n) => &n.attrs,
        syn::Expr::Reference(n) => &n.attrs,
        syn::Expr::Repeat(n) => &n.attrs,
        syn::Expr::Return(n) => &n.attrs,
        syn::Expr::Struct(n) => &n.attrs,
        syn::Expr::Try(n) => &n.attrs,
        syn::Expr::TryBlock(n) => &n.attrs,
        syn::Expr::Tuple(n) => &n.attrs,
        syn::Expr::Unary(n) => &n.attrs,
        syn::Expr::Unsafe(n) => &n.attrs,
        syn::Expr::Verbatim(_) => &[],
        syn::Expr::While(n) => &n.attrs,
        syn::Expr::Yield(n) => &n.attrs,
        _ => return Err(Refusal::Invariant("unknown pinned expression variant")),
    })
}

fn first<'a>(node: &'a syn::Expr, source: &Coordinates<'_, '_>) -> Result<Step<'a>, Refusal> {
    use Step::{Child, Token};
    Ok(match node {
        syn::Expr::Array(n) => Token(n.bracket_token.span.open()),
        syn::Expr::Assign(n) => Child(&n.left),
        syn::Expr::Async(n) => Token(n.async_token.span),
        syn::Expr::Await(n) => Child(&n.base),
        syn::Expr::Binary(n) => Child(&n.left),
        syn::Expr::Block(n) => Token(
            n.label
                .as_ref()
                .map_or(n.block.brace_token.span.open(), |l| l.name.apostrophe),
        ),
        syn::Expr::Break(n) => Token(n.break_token.span),
        syn::Expr::Call(n) => Child(&n.func),
        syn::Expr::Cast(n) => Child(&n.expr),
        syn::Expr::Closure(n) => Token(closure_prefix(n)),
        syn::Expr::Const(n) => Token(n.const_token.span),
        syn::Expr::Continue(n) => Token(n.continue_token.span),
        syn::Expr::Field(n) => Child(&n.base),
        syn::Expr::ForLoop(n) => Token(
            n.label
                .as_ref()
                .map_or(n.for_token.span, |l| l.name.apostrophe),
        ),
        syn::Expr::Group(n) => Child(&n.expr),
        syn::Expr::If(n) => Token(n.if_token.span),
        syn::Expr::Index(n) => Child(&n.expr),
        syn::Expr::Infer(n) => Token(n.underscore_token.span),
        syn::Expr::Let(n) => Token(n.let_token.span),
        syn::Expr::Lit(n) => Token(n.lit.span()),
        syn::Expr::Loop(n) => Token(
            n.label
                .as_ref()
                .map_or(n.loop_token.span, |l| l.name.apostrophe),
        ),
        syn::Expr::Macro(n) => Token(path_start(&n.mac.path, None, source)?),
        syn::Expr::Match(n) => Token(n.match_token.span),
        syn::Expr::MethodCall(n) => Child(&n.receiver),
        syn::Expr::Paren(n) => Token(n.paren_token.span.open()),
        syn::Expr::Path(n) => Token(path_start(&n.path, n.qself.as_ref(), source)?),
        syn::Expr::Range(n) => match &n.start {
            Some(start) => Child(start),
            None => Token(limits(&n.limits).0),
        },
        syn::Expr::RawAddr(n) => Token(n.and_token.spans[0]),
        syn::Expr::Reference(n) => Token(n.and_token.spans[0]),
        syn::Expr::Repeat(n) => Token(n.bracket_token.span.open()),
        syn::Expr::Return(n) => Token(n.return_token.span),
        syn::Expr::Struct(n) => Token(path_start(&n.path, n.qself.as_ref(), source)?),
        syn::Expr::Try(n) => Child(&n.expr),
        syn::Expr::TryBlock(n) => Token(n.try_token.span),
        syn::Expr::Tuple(n) => Token(n.paren_token.span.open()),
        syn::Expr::Unary(n) => Token(match &n.op {
            syn::UnOp::Deref(t) => t.spans[0],
            syn::UnOp::Not(t) => t.spans[0],
            syn::UnOp::Neg(t) => t.spans[0],
            _ => return Err(Refusal::Invariant("unknown pinned unary operator")),
        }),
        syn::Expr::Unsafe(n) => Token(n.unsafe_token.span),
        syn::Expr::Verbatim(tokens) => Token(
            source
                .opaque_ends(tokens)?
                .map_or_else(Span::call_site, |ends| ends.0),
        ),
        syn::Expr::While(n) => Token(
            n.label
                .as_ref()
                .map_or(n.while_token.span, |l| l.name.apostrophe),
        ),
        syn::Expr::Yield(n) => Token(n.yield_token.span),
        _ => return Err(Refusal::Invariant("unknown pinned expression variant")),
    })
}

fn last<'a>(node: &'a syn::Expr, source: &Coordinates<'_, '_>) -> Result<Step<'a>, Refusal> {
    use Step::{Child, Token};
    Ok(match node {
        syn::Expr::Array(n) => Token(n.bracket_token.span.close()),
        syn::Expr::Assign(n) => Child(&n.right),
        syn::Expr::Async(n) => Token(n.block.brace_token.span.close()),
        syn::Expr::Await(n) => Token(n.await_token.span),
        syn::Expr::Binary(n) => Child(&n.right),
        syn::Expr::Block(n) => Token(n.block.brace_token.span.close()),
        syn::Expr::Break(n) => match &n.expr {
            Some(expr) => Child(expr),
            None => Token(
                n.label
                    .as_ref()
                    .map_or(n.break_token.span, |l| l.ident.span()),
            ),
        },
        syn::Expr::Call(n) => Token(n.paren_token.span.close()),
        syn::Expr::Cast(n) => Token(super::ty::end(&n.ty, source)?.unwrap_or(n.as_token.span)),
        syn::Expr::Closure(n) => Child(&n.body),
        syn::Expr::Const(n) => Token(n.block.brace_token.span.close()),
        syn::Expr::Continue(n) => Token(
            n.label
                .as_ref()
                .map_or(n.continue_token.span, |l| l.ident.span()),
        ),
        syn::Expr::Field(n) => Token(match &n.member {
            syn::Member::Named(i) => i.span(),
            syn::Member::Unnamed(i) => i.span,
        }),
        syn::Expr::ForLoop(n) => Token(n.body.brace_token.span.close()),
        syn::Expr::Group(n) => Child(&n.expr),
        syn::Expr::If(n) => match &n.else_branch {
            Some((_, expr)) => Child(expr),
            None => Token(n.then_branch.brace_token.span.close()),
        },
        syn::Expr::Index(n) => Token(n.bracket_token.span.close()),
        syn::Expr::Infer(n) => Token(n.underscore_token.span),
        syn::Expr::Let(n) => Child(&n.expr),
        syn::Expr::Lit(n) => Token(n.lit.span()),
        syn::Expr::Loop(n) => Token(n.body.brace_token.span.close()),
        syn::Expr::Macro(n) => Token(delimiter_end(&n.mac.delimiter)),
        syn::Expr::Match(n) => Token(n.brace_token.span.close()),
        syn::Expr::MethodCall(n) => Token(n.paren_token.span.close()),
        syn::Expr::Paren(n) => Token(n.paren_token.span.close()),
        syn::Expr::Path(n) => Token(path_end(&n.path, n.qself.as_ref(), source)?),
        syn::Expr::Range(n) => match &n.end {
            Some(end) => Child(end),
            None => Token(limits(&n.limits).1),
        },
        syn::Expr::RawAddr(n) => Child(&n.expr),
        syn::Expr::Reference(n) => Child(&n.expr),
        syn::Expr::Repeat(n) => Token(n.bracket_token.span.close()),
        syn::Expr::Return(n) => match &n.expr {
            Some(expr) => Child(expr),
            None => Token(n.return_token.span),
        },
        syn::Expr::Struct(n) => Token(n.brace_token.span.close()),
        syn::Expr::Try(n) => Token(n.question_token.spans[0]),
        syn::Expr::TryBlock(n) => Token(n.block.brace_token.span.close()),
        syn::Expr::Tuple(n) => Token(n.paren_token.span.close()),
        syn::Expr::Unary(n) => Child(&n.expr),
        syn::Expr::Unsafe(n) => Token(n.block.brace_token.span.close()),
        syn::Expr::Verbatim(tokens) => Token(
            source
                .opaque_ends(tokens)?
                .map_or_else(Span::call_site, |ends| ends.1),
        ),
        syn::Expr::While(n) => Token(n.body.brace_token.span.close()),
        syn::Expr::Yield(n) => match &n.expr {
            Some(expr) => Child(expr),
            None => Token(n.yield_token.span),
        },
        _ => return Err(Refusal::Invariant("unknown pinned expression variant")),
    })
}

pub(in crate::embedded) fn closure_start(
    node: &syn::ExprClosure,
    source: &Coordinates<'_, '_>,
) -> Result<Span, Refusal> {
    source.work(1)?;
    Ok(outer(&node.attrs, source.meter())?.unwrap_or_else(|| closure_prefix(node)))
}

fn closure_prefix(node: &syn::ExprClosure) -> Span {
    node.lifetimes
        .as_ref()
        .map(|t| t.for_token.span)
        .or_else(|| node.constness.as_ref().map(|t| t.span))
        .or_else(|| node.asyncness.as_ref().map(|t| t.span))
        .or_else(|| node.capture.as_ref().map(|t| t.span))
        .unwrap_or(node.inputs_begin.spans[0])
}

pub(super) fn limits(limits: &syn::RangeLimits) -> Ends {
    match limits {
        syn::RangeLimits::HalfOpen(token) => (token.spans[0], token.spans[1]),
        syn::RangeLimits::Closed(token) => (token.spans[0], token.spans[2]),
    }
}

pub(super) fn delimiter_end(delimiter: &syn::MacroDelimiter) -> Span {
    match delimiter {
        syn::MacroDelimiter::Paren(t) => t.span.close(),
        syn::MacroDelimiter::Brace(t) => t.span.close(),
        syn::MacroDelimiter::Bracket(t) => t.span.close(),
    }
}

fn path_start(
    path: &syn::Path,
    qself: Option<&syn::QSelf>,
    source: &Coordinates<'_, '_>,
) -> Result<Span, Refusal> {
    super::ty::path_start(path, qself, source)?
        .ok_or(Refusal::Invariant("parsed expression path is empty"))
}

fn path_end(
    path: &syn::Path,
    qself: Option<&syn::QSelf>,
    source: &Coordinates<'_, '_>,
) -> Result<Span, Refusal> {
    super::ty::path_end(path, qself, source)?
        .ok_or(Refusal::Invariant("parsed expression path is empty"))
}
