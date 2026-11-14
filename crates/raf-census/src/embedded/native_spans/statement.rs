// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Original composite endpoints from stored delimiters and semicolons.

use super::{Coordinates, Ends, Refusal, expression, item_start, local_start, outer, ty};

pub(in crate::embedded) fn ends(
    node: &syn::Stmt,
    source: &Coordinates<'_, '_>,
) -> Result<Option<Ends>, Refusal> {
    source.work(1)?;
    match node {
        syn::Stmt::Local(local) => Ok(Some((
            local_start(local, source.meter())?,
            local.semi_token.spans[0],
        ))),
        syn::Stmt::Item(item) => item_ends(item, source),
        syn::Stmt::Macro(node) => stmt_macro_ends(node, source).map(Some),
        syn::Stmt::Expr(node, semi) => {
            // A lone semicolon has an empty verbatim expression. The actual
            // punctuation is its entire emitted statement, not call-site data.
            if matches!(node, syn::Expr::Verbatim(tokens) if tokens.is_empty()) {
                return Ok(semi.as_ref().map(|semi| (semi.spans[0], semi.spans[0])));
            }
            let (first, mut last) = expression::ends(node, source)?;
            if let Some(semi) = semi {
                last = semi.spans[0];
            }
            Ok(Some((first, last)))
        }
    }
}

pub(in crate::embedded) fn stmt_macro_ends(
    node: &syn::StmtMacro,
    source: &Coordinates<'_, '_>,
) -> Result<Ends, Refusal> {
    source.work(1)?;
    let (first, last) = macro_ends(&node.mac, source)?;
    Ok((
        outer(&node.attrs, source.meter())?.unwrap_or(first),
        node.semi_token.as_ref().map_or(last, |semi| semi.spans[0]),
    ))
}

fn macro_ends(node: &syn::Macro, source: &Coordinates<'_, '_>) -> Result<Ends, Refusal> {
    source.work(1)?;
    Ok((
        ty::path_start(&node.path, None, source)?.unwrap_or(node.bang_token.spans[0]),
        expression::delimiter_end(&node.delimiter),
    ))
}

pub(in crate::embedded) fn item_ends(
    node: &syn::Item,
    source: &Coordinates<'_, '_>,
) -> Result<Option<Ends>, Refusal> {
    if let syn::Item::Verbatim(tokens) = node {
        return source.opaque_ends(tokens);
    }
    let first = item_start(node, source.meter())?.ok_or(Refusal::Invariant(
        "a concrete item lost its first source token",
    ))?;
    source.work(1)?;
    let last = match node {
        syn::Item::Const(n) => n.semi_token.spans[0],
        syn::Item::Enum(n) => n.brace_token.span.close(),
        syn::Item::ExternCrate(n) => n.semi_token.spans[0],
        syn::Item::Fn(n) => n.block.brace_token.span.close(),
        syn::Item::ForeignMod(n) => n.brace_token.span.close(),
        syn::Item::Impl(n) => n.brace_token.span.close(),
        syn::Item::Macro(n) => n.semi_token.as_ref().map_or_else(
            || expression::delimiter_end(&n.mac.delimiter),
            |semi| semi.spans[0],
        ),
        syn::Item::Mod(n) => match &n.content {
            Some((brace, _)) => brace.span.close(),
            None => {
                n.semi
                    .as_ref()
                    .ok_or(Refusal::Invariant(
                        "parsed outline module lost its semicolon",
                    ))?
                    .spans[0]
            }
        },
        syn::Item::Static(n) => n.semi_token.spans[0],
        syn::Item::Struct(n) => match &n.fields {
            syn::Fields::Named(fields) => fields.brace_token.span.close(),
            syn::Fields::Unnamed(_) | syn::Fields::Unit => {
                n.semi_token
                    .as_ref()
                    .ok_or(Refusal::Invariant(
                        "parsed tuple or unit struct lost its semicolon",
                    ))?
                    .spans[0]
            }
        },
        syn::Item::Trait(n) => n.brace_token.span.close(),
        syn::Item::TraitAlias(n) => n.semi_token.spans[0],
        syn::Item::Type(n) => n.semi_token.spans[0],
        syn::Item::Union(n) => n.fields.brace_token.span.close(),
        syn::Item::Use(n) => n.semi_token.spans[0],
        _ => return Err(Refusal::Invariant("unknown concrete pinned item variant")),
    };
    Ok(Some((first, last)))
}
