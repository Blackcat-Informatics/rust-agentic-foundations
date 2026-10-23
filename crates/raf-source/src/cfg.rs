// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Shared Rust cfg predicate parsing and structural test classification.
//!
//! Structural loads keep every alternative and tag what implies test. Explicit
//! configured loads evaluate this same parsed grammar on the bounded parse worker.

use std::fmt;

use syn::ext::IdentExt as _;
use syn::parse::Parser as _;
use syn::punctuated::Punctuated;
use syn::{Expr, ExprLit, Lit, Meta, Token};

/// A parsed `cfg` predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CfgExpr {
    /// An unescaped boolean literal.
    Boolean(bool),
    /// `test`.
    Test,
    /// `not(..)`.
    Not(Box<CfgExpr>),
    /// `all(..)`.
    All(Vec<CfgExpr>),
    /// `any(..)`.
    Any(Vec<CfgExpr>),
    /// Any other name or `key = "value"` pair, kept as written.
    Other(String),
}

impl CfgExpr {
    /// True when the predicate holds only in a test build.
    ///
    /// `test` does; `all(..)` does when any operand does; `any(..)` does when
    /// every operand does; `not(..)` and every other predicate do not.
    pub(crate) fn implies_test(&self) -> bool {
        match self {
            Self::Test => true,
            Self::Boolean(_) | Self::Not(_) | Self::Other(_) => false,
            Self::All(operands) => operands.iter().any(Self::implies_test),
            Self::Any(operands) => !operands.is_empty() && operands.iter().all(Self::implies_test),
        }
    }

    /// Read a predicate from the meta item inside `cfg(..)` or `cfg_attr(..)`.
    ///
    /// # Errors
    ///
    /// Returns a message when the predicate is not a name, a `key = "value"`
    /// pair, or `not`, `all` or `any` over predicates.
    pub(crate) fn from_meta(meta: &Meta) -> Result<Self, String> {
        Self::read_meta(meta, false)
    }

    /// Parse configured predicates, refusing qualified Rust paths.
    pub(crate) fn from_meta_configured(meta: &Meta) -> Result<Self, String> {
        Self::read_meta(meta, true)
    }

    fn read_meta(meta: &Meta, configured: bool) -> Result<Self, String> {
        let path = meta.path();
        if configured
            && (path.leading_colon.is_some()
                || path.segments.len() != 1
                || path
                    .segments
                    .iter()
                    .any(|segment| !segment.arguments.is_none()))
        {
            return Err("cfg names and operators must be unqualified identifiers".to_owned());
        }
        match meta {
            Meta::Path(path) => {
                if let Some(identifier) = path.get_ident() {
                    if identifier == "true" {
                        return Ok(Self::Boolean(true));
                    }
                    if identifier == "false" {
                        return Ok(Self::Boolean(false));
                    }
                }
                let name = path_text(path);
                if name == "test" {
                    Ok(Self::Test)
                } else {
                    Ok(Self::Other(name))
                }
            }
            Meta::NameValue(pair) => match &pair.value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(value),
                    ..
                }) => Ok(Self::Other(format!(
                    "{} = {:?}",
                    path_text(&pair.path),
                    value.value()
                ))),
                _ => Err(format!(
                    "the value of `{}` is not a string literal",
                    path_text(&pair.path)
                )),
            },
            Meta::List(list) => {
                let parser = |input: syn::parse::ParseStream<'_>| {
                    Punctuated::<Meta, Token![,]>::parse_terminated_with(input, parse_meta)
                };
                let operands = parser
                    .parse2(list.tokens.clone())
                    .map_err(|error| error.to_string())?
                    .iter()
                    .map(|meta| Self::read_meta(meta, configured))
                    .collect::<Result<Vec<_>, _>>()?;
                let name = path_text(&list.path);
                match (name.as_str(), operands.len()) {
                    ("not", 1) => {
                        let mut operands = operands;
                        match operands.pop() {
                            Some(only) => Ok(Self::Not(Box::new(only))),
                            None => Err("`not` needs one predicate".to_owned()),
                        }
                    }
                    ("not", _) => Err("`not` needs exactly one predicate".to_owned()),
                    ("all", _) => Ok(Self::All(operands)),
                    ("any", _) => Ok(Self::Any(operands)),
                    (other, _) => Err(format!("`{other}` is not a cfg operator")),
                }
            }
        }
    }
}

fn path_text(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.unraw().to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// The comma-separated meta items of a list's tokens.
///
/// # Errors
///
/// Returns the parser's message when the tokens are not meta items.
pub(crate) fn parse_list(tokens: &proc_macro2::TokenStream) -> Result<Vec<Meta>, String> {
    parse_punctuated(tokens).map(|list| list.into_iter().collect())
}

/// Parse a meta list while retaining its required or optional trailing comma.
pub(crate) fn parse_punctuated(
    tokens: &proc_macro2::TokenStream,
) -> Result<Punctuated<Meta, Token![,]>, String> {
    let parser = |input: syn::parse::ParseStream<'_>| {
        let mut list = Punctuated::new();
        if !input.is_empty() {
            list.push_value(parse_meta(input)?);
        }
        while !input.is_empty() {
            list.push_punct(input.parse::<Token![,]>()?);
            if input.is_empty() {
                break;
            }
            list.push_value(input.parse::<Meta>()?);
        }
        Ok(list)
    };
    parser
        .parse2(tokens.clone())
        .map_err(|error| error.to_string())
}

/// Parse a predicate atom using syn's boolean literal or ordinary meta grammar.
pub(crate) fn parse_meta(input: syn::parse::ParseStream<'_>) -> syn::Result<Meta> {
    if input.peek(syn::LitBool) {
        let literal: syn::LitBool = input.parse()?;
        let ident =
            proc_macro2::Ident::new(if literal.value { "true" } else { "false" }, literal.span());
        Ok(Meta::Path(ident.into()))
    } else {
        input.parse()
    }
}

impl fmt::Display for CfgExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let list = |f: &mut fmt::Formatter<'_>, name: &str, operands: &[Self]| {
            write!(f, "{name}(")?;
            for (index, operand) in operands.iter().enumerate() {
                if index > 0 {
                    f.write_str(", ")?;
                }
                write!(f, "{operand}")?;
            }
            f.write_str(")")
        };
        match self {
            Self::Boolean(value) => write!(f, "{value}"),
            Self::Test => f.write_str("test"),
            Self::Not(inner) => write!(f, "not({inner})"),
            Self::All(operands) => list(f, "all", operands),
            Self::Any(operands) => list(f, "any", operands),
            Self::Other(text) => f.write_str(text),
        }
    }
}
