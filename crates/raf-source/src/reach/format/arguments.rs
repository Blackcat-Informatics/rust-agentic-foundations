// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Literal-leading format arguments, admitted before recursive expression parsing.

use std::collections::BTreeSet;

use proc_macro2::{Group, TokenStream, TokenTree};
use syn::ext::IdentExt as _;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, LitStr, Token};

use super::{Arguments, Refused};
use crate::error::LoadErrorKind;
use crate::{Budget, RelPath};

struct Parsed {
    literal: LitStr,
    named: Vec<(String, usize)>,
}

impl Parse for Parsed {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let literal = input.parse()?;
        let mut named = Vec::new();
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            if input.peek(Ident::peek_any) && input.peek2(Token![=]) && !input.peek3(Token![=]) {
                let name = input.call(Ident::parse_any)?;
                input.parse::<Token![=]>()?;
                named.push((
                    name.to_string().trim_start_matches("r#").to_owned(),
                    name.span().byte_range().start,
                ));
            }
            input.parse::<Expr>()?;
        }
        Ok(Self { literal, named })
    }
}

pub(super) fn read(
    group: &Group,
    source: &str,
    base: usize,
    bytes: &mut usize,
) -> Option<Result<Arguments, Refused>> {
    let stream = group.stream();
    let first = stream.clone().into_iter().next()?;
    if !matches!(&first, TokenTree::Literal(_))
        || syn::parse2::<LitStr>(TokenStream::from(first)).is_err()
    {
        return None;
    }
    let start = base + group.span_open().byte_range().end;
    let end = base + group.span_close().byte_range().start;
    let line =
        group.span_open().end().line + source[..base].bytes().filter(|&b| b == b'\n').count();
    let Some(text) = source.get(start..end) else {
        return Some(Err(Refused::Syntax {
            message: "format argument spans are outside their source".into(),
            line,
            column: 1,
        }));
    };
    *bytes += text.len();
    let file = RelPath::new("format-arguments.rs").expect("a fixed relative path");
    if let Err(error) = crate::prescan::prescan(&file, text, Budget::FileTokens.limit()) {
        let line = line + error.line.unwrap_or(1) - 1;
        return Some(Err(match error.kind {
            LoadErrorKind::Budget { budget, limit } => Refused::Budget {
                budget,
                limit,
                line,
            },
            other => Refused::Syntax {
                message: format!("format argument admission failed: {other:?}"),
                line,
                column: 1,
            },
        }));
    }
    let parsed = match syn::parse2::<Parsed>(stream) {
        Ok(parsed) => parsed,
        Err(error) => {
            return Some(Err(Refused::Syntax {
                message: format!("cannot read format arguments: {error}"),
                line: error.span().start().line
                    + source[..base].bytes().filter(|&b| b == b'\n').count(),
                column: error.span().start().column + 1,
            }));
        }
    };
    let value = parsed.literal.value();
    *bytes += value.len();
    let Some(mut captures) = captures(&value) else {
        return Some(Err(Refused::Syntax {
            message: "cannot read the literal's format fields".into(),
            line,
            column: 1,
        }));
    };
    let mut bindings = Vec::with_capacity(parsed.named.len());
    for (name, start) in parsed.named {
        captures.remove(name.as_str());
        bindings.push(base + start);
    }
    Some(Ok(Arguments {
        captures: captures.into_iter().map(String::into_boxed_str).collect(),
        bindings,
    }))
}

fn identifier(name: &str) -> bool {
    !name.is_empty()
        && name != "_"
        && !name.starts_with("r#")
        && syn::parse::Parser::parse_str(Ident::parse_any, name).is_ok()
}

/// Rust's fmt grammar permits identifier-or-keyword captures and named counts.
/// Numeric arguments and positional star precision are not captured values.
fn captures(text: &str) -> Option<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    let mut chars = text.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        if ch == '{' || ch == '}' {
            if chars.peek().is_some_and(|(_, next)| *next == ch) {
                chars.next();
                continue;
            }
            if ch == '}' {
                return None;
            }
            let start = chars.peek()?.0;
            let end = loop {
                let (at, ch) = chars.next()?;
                if ch == '{' {
                    return None;
                }
                if ch == '}' {
                    break at;
                }
            };
            let field = &text[start..end];
            let (argument, spec) = field
                .split_once(':')
                .map_or((field, None), |(a, b)| (a, Some(b)));
            let argument = argument.trim_end();
            if identifier(argument) {
                names.insert(argument.to_owned());
            } else if !argument.is_empty() && !argument.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            if let Some(spec) = spec {
                counts(spec, &mut names)?;
            }
        }
    }
    Some(names)
}

fn counts(spec: &str, names: &mut BTreeSet<String>) -> Option<()> {
    let mut rest = spec;
    let align = |ch| matches!(ch, '<' | '^' | '>');
    let mut preview = rest.chars();
    if let Some(first) = preview.next()
        && (align(first) || preview.next().is_some_and(align))
    {
        rest = preview.as_str();
    }
    if rest.starts_with(['+', '-']) {
        rest = &rest[1..];
    }
    if rest.starts_with('#') {
        rest = &rest[1..];
    }
    if rest.starts_with('0') {
        rest = &rest[1..];
    }
    if let Some(end) = rest.find('$') {
        let count = &rest[..end];
        if identifier(count) || (!count.is_empty() && count.bytes().all(|b| b.is_ascii_digit())) {
            if identifier(count) {
                names.insert(count.to_owned());
            }
            rest = &rest[end + 1..];
        }
    }
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    rest = &rest[digits..];
    if let Some(precision) = rest.strip_prefix('.') {
        rest = precision;
        if let Some(tail) = rest.strip_prefix('*') {
            rest = tail;
        } else if let Some(end) = rest.find('$') {
            let count = &rest[..end];
            if identifier(count) {
                names.insert(count.to_owned());
            } else if count.is_empty() || !count.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            rest = &rest[end + 1..];
        } else {
            let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
            if digits == 0 {
                return None;
            }
            rest = &rest[digits..];
        }
    }
    let kind = rest.trim_end();
    (kind.is_empty() || matches!(kind, "?" | "x?" | "X?") || identifier(kind)).then_some(())
}
