// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Rust formatting-field order for the framing syntax detector.

/// Select the first two rendered arguments, respecting escaped braces and
/// positional/named selectors. This recognizes source syntax, not a formatter.
pub(super) fn first_pair<'a>(
    name: &str,
    arguments: &'a [syn::Expr],
) -> Option<(&'a syn::Expr, &'a syn::Expr)> {
    let format_at = usize::from(matches!(name, "write" | "writeln"));
    let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(format),
        ..
    }) = arguments.get(format_at)?
    else {
        return None;
    };
    let text = format.value();
    let mut rest = text.as_str();
    let mut next = 0;
    let mut selected = Vec::new();
    while let Some((field, tail)) = next_field(rest) {
        rest = tail;
        let selector = field.split(':').next()?.trim();
        // Rust consumes an implicit precision argument before selecting the
        // rendered value, including when that value has an explicit index.
        if field
            .split_once(':')
            .is_some_and(|(_, spec)| spec.contains(".*"))
        {
            next += 1;
        }
        let supplied = arguments.get(format_at + 1..)?;
        let argument = if selector.is_empty() {
            let selected = supplied.get(next)?;
            next += 1;
            selected
        } else if selector.bytes().all(|byte| byte.is_ascii_digit()) {
            let index = crate::normalize::parse_integer(selector)?.value?;
            supplied.get(usize::try_from(index).ok()?)?
        } else {
            supplied.iter().find_map(|argument| match argument {
                syn::Expr::Assign(named) if matches!(named.left.as_ref(), syn::Expr::Path(path) if path.path.is_ident(selector)) => Some(named.right.as_ref()),
                _ => None,
            })?
        };
        selected.push(argument);
        if selected.len() == 2 {
            return Some((selected[0], selected[1]));
        }
    }
    None
}

fn next_field(mut rest: &str) -> Option<(&str, &str)> {
    while let Some(open) = rest.find('{') {
        rest = &rest[open + 1..];
        if let Some(escaped) = rest.strip_prefix('{') {
            rest = escaped;
            continue;
        }
        return rest.split_once('}');
    }
    None
}

pub(super) fn has_hex_pair(mut text: &str) -> bool {
    while let Some((field, rest)) = next_field(text) {
        if field
            .split_once(':')
            .is_some_and(|(_, spec)| matches!(spec, "02x" | "02X"))
        {
            return true;
        }
        text = rest;
    }
    false
}
