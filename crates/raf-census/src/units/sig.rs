// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The two things a unit needs `syn` for: the names a function's parameters
//! bind, and where the initialiser of a constant starts and ends.

use std::ops::Range;

use raf_source::SymbolKind;
use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::Error;

/// A parse failure of `text`, whose first byte is on `first_line` of `file`.
fn refusal(file: &str, first_line: usize, error: &syn::Error) -> Error {
    Error::Source {
        file: file.to_owned(),
        line: first_line + error.span().start().line.saturating_sub(1),
        reason: format!("the item does not parse: {error}"),
    }
}

/// Every name a parameter pattern binds, in source order. Types are not
/// visited, so a path in a type is not a binding.
#[derive(Default)]
struct Bindings(Vec<String>);

impl<'ast> Visit<'ast> for Bindings {
    fn visit_pat_ident(&mut self, pat: &'ast syn::PatIdent) {
        self.0.push(pat.ident.to_string());
        syn::visit::visit_pat_ident(self, pat);
    }

    fn visit_type(&mut self, _: &'ast syn::Type) {}
}

/// The names the parameters of a function bind, in signature order, from the
/// text of its signature: `mut x`, `ref x`, the names inside tuple and struct
/// patterns. `self` is a receiver and binds nothing the body renames.
///
/// The signature is parsed as the kind of item it belongs to, so a modifier
/// that only one kind admits, such as `default` on a method of an
/// implementation, is not a failure.
///
/// # Errors
///
/// [`Error::Source`] when the signature does not parse.
pub(super) fn parameters(
    file: &str,
    first_line: usize,
    kind: SymbolKind,
    signature: &str,
) -> Result<Vec<String>, Error> {
    let text = format!("{signature} {{}}");
    let refused = |error: &syn::Error| refusal(file, first_line, error);
    let inputs = match kind {
        SymbolKind::InherentMethod | SymbolKind::TraitImplMethod => {
            syn::parse_str::<syn::ImplItemFn>(&text)
                .map_err(|error| refused(&error))?
                .sig
                .inputs
        }
        SymbolKind::TraitDefaultMethod => {
            syn::parse_str::<syn::TraitItemFn>(&text)
                .map_err(|error| refused(&error))?
                .sig
                .inputs
        }
        _ => {
            syn::parse_str::<syn::ItemFn>(&text)
                .map_err(|error| refused(&error))?
                .sig
                .inputs
        }
    };
    let mut bindings = Bindings::default();
    for input in &inputs {
        if let syn::FnArg::Typed(typed) = input {
            bindings.visit_pat(&typed.pat);
        }
    }
    Ok(bindings.0)
}

/// Where the initialiser of a constant or static is inside `item`, the text of
/// the whole item: the byte range between the `=` and the `;`, never the type
/// and never an attribute. `None` for an associated constant with no value.
///
/// # Errors
///
/// [`Error::Source`] when the item does not parse.
pub(super) fn initializer(
    file: &str,
    first_line: usize,
    kind: SymbolKind,
    item: &str,
) -> Result<Option<Range<usize>>, Error> {
    let between = |eq: &dyn Spanned, semi: &dyn Spanned| {
        Some(eq.span().byte_range().end..semi.span().byte_range().start)
    };
    let range = match kind {
        SymbolKind::Static => {
            let parsed = syn::parse_str::<syn::ItemStatic>(item)
                .map_err(|error| refusal(file, first_line, &error))?;
            between(&parsed.eq_token, &parsed.semi_token)
        }
        SymbolKind::Const => {
            let parsed = syn::parse_str::<syn::ItemConst>(item)
                .map_err(|error| refusal(file, first_line, &error))?;
            between(&parsed.eq_token, &parsed.semi_token)
        }
        _ => {
            if let Ok(parsed) = syn::parse_str::<syn::ImplItemConst>(item) {
                between(&parsed.eq_token, &parsed.semi_token)
            } else {
                let parsed = syn::parse_str::<syn::TraitItemConst>(item)
                    .map_err(|error| refusal(file, first_line, &error))?;
                parsed
                    .default
                    .as_ref()
                    .and_then(|(eq, _)| between(eq, &parsed.semi_token))
            }
        }
    };
    Ok(range)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameters_are_every_bound_name_in_signature_order() {
        let names = parameters(
            "a.rs",
            1,
            SymbolKind::InherentMethod,
            "pub fn go(&mut self, (a, mut b): (u8, u8), ref c: u8, P { x, y: z }: P, d: Vec<u8>) -> u8",
        )
        .expect("a valid signature");
        assert_eq!(names, ["a", "b", "c", "x", "z", "d"]);
    }

    #[test]
    fn a_default_method_of_an_implementation_is_a_signature() {
        let names = parameters(
            "a.rs",
            1,
            SymbolKind::TraitImplMethod,
            "default fn go(x: u8)",
        )
        .expect("a valid signature");
        assert_eq!(names, ["x"]);
    }

    #[test]
    fn an_unparsable_signature_is_a_source_error_beside_a_valid_one() {
        let valid = parameters("a.rs", 4, SymbolKind::Fn, "fn go(x: u8)");
        assert_eq!(valid.expect("the neighbour parses"), ["x"]);
        let refused = parameters("a.rs", 4, SymbolKind::Fn, "fn go(x: u8");
        match refused {
            Err(Error::Source { file, line, .. }) => assert_eq!((file.as_str(), line), ("a.rs", 4)),
            other => panic!("expected a source error, found {other:?}"),
        }
    }

    #[test]
    fn the_initializer_is_between_the_equals_and_the_semicolon() {
        let item = "#[allow(dead_code)]\npub const N: [u8; 0x10000] = [0; 4];";
        let range = initializer("a.rs", 1, SymbolKind::Const, item)
            .expect("a valid item")
            .expect("a constant with a value");
        assert_eq!(item.get(range).map(str::trim), Some("[0; 4]"));
        let with_value = initializer("a.rs", 1, SymbolKind::AssocConst, "const N: u8 = 3;");
        assert!(with_value.expect("a valid item").is_some());
        let required = initializer("a.rs", 1, SymbolKind::AssocConst, "const N: u8;");
        assert!(required.expect("a valid item").is_none());
    }

    #[test]
    fn an_unparsable_initializer_is_a_source_error_beside_a_valid_one() {
        assert!(initializer("a.rs", 9, SymbolKind::Static, "static N: u8 = 3;").is_ok());
        let refused = initializer("a.rs", 9, SymbolKind::Static, "static N: u8 = ;");
        match refused {
            Err(Error::Source { file, line, .. }) => assert_eq!((file.as_str(), line), ("a.rs", 9)),
            other => panic!("expected a source error, found {other:?}"),
        }
    }
}
