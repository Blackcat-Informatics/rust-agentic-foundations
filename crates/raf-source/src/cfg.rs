// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `cfg` predicates, read only to learn whether they imply `test`.
//!
//! Index construction evaluates no predicate. The crate keeps every item and every module
//! whatever its `cfg`; it only tags what a `test` predicate guards.

use std::fmt;

use syn::parse::Parser as _;
use syn::punctuated::Punctuated;
use syn::{Expr, ExprLit, Lit, Meta, Token};

/// A parsed `cfg` predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CfgExpr {
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
            Self::Not(_) | Self::Other(_) => false,
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
        match meta {
            Meta::Path(path) => {
                if path.is_ident("test") {
                    Ok(Self::Test)
                } else {
                    Ok(Self::Other(path_text(path)))
                }
            }
            Meta::NameValue(pair) => match &pair.value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(value),
                    ..
                }) => Ok(Self::Other(format!(
                    "{} = \"{}\"",
                    path_text(&pair.path),
                    value.value()
                ))),
                _ => Err(format!(
                    "the value of `{}` is not a string literal",
                    path_text(&pair.path)
                )),
            },
            Meta::List(list) => {
                let operands = parse_list(&list.tokens)?
                    .iter()
                    .map(Self::from_meta)
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
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// The comma-separated meta items of a list's tokens.
///
/// # Errors
///
/// Returns the parser's message when the tokens are not meta items.
pub(crate) fn parse_list(tokens: &proc_macro2::TokenStream) -> Result<Vec<Meta>, String> {
    Punctuated::<Meta, Token![,]>::parse_terminated
        .parse2(tokens.clone())
        .map(|list| list.into_iter().collect())
        .map_err(|error| error.to_string())
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
            Self::Test => f.write_str("test"),
            Self::Not(inner) => write!(f, "not({inner})"),
            Self::All(operands) => list(f, "all", operands),
            Self::Any(operands) => list(f, "any", operands),
            Self::Other(text) => f.write_str(text),
        }
    }
}
