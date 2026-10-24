// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Audit otherwise erased generic arguments for executable computation.

use syn::visit::{self, Visit};

use super::{Proof, charge};
use crate::Error;

pub(super) fn check(proof: &mut Proof<'_>, arguments: &syn::PathArguments) -> Result<(), Error> {
    let mut visitor = Arguments { proof, error: None };
    visitor.visit_path_arguments(arguments);
    visitor.error.map_or(Ok(()), Err)
}

struct Arguments<'a, 'b> {
    proof: &'a mut Proof<'b>,
    error: Option<Error>,
}

impl Arguments<'_, '_> {
    fn admitted(&mut self) -> bool {
        if self.error.is_some() {
            return false;
        }
        if let Err(error) = charge(&self.proof.expected.row.job.id, self.proof.limits, 1) {
            self.error = Some(error);
            return false;
        }
        true
    }

    fn refuse(&mut self) {
        self.error = Some(
            self.proof
                .refused("generic arguments contain unsupported computation or expansion"),
        );
    }
}

impl<'ast> Visit<'ast> for Arguments<'_, '_> {
    fn visit_expr(&mut self, expression: &'ast syn::Expr) {
        if !self.admitted() {
            return;
        }
        let plain = match self.proof.plain(expression) {
            Ok(plain) => plain,
            Err(error) => {
                self.error = Some(error);
                return;
            }
        };
        let supported = match plain {
            syn::Expr::Lit(node) => node.attrs.is_empty(),
            syn::Expr::Path(node) => node.qself.is_none(),
            syn::Expr::Unary(node) => {
                node.attrs.is_empty() && matches!(node.op, syn::UnOp::Neg(_) | syn::UnOp::Not(_))
            }
            syn::Expr::Block(node) => {
                node.attrs.is_empty() && node.label.is_none() && single_expression(&node.block)
            }
            syn::Expr::Const(node) => node.attrs.is_empty() && single_expression(&node.block),
            _ => false,
        };
        if supported {
            visit::visit_expr(self, plain);
        } else {
            self.refuse();
        }
    }

    fn visit_type(&mut self, ty: &'ast syn::Type) {
        if !self.admitted() {
            return;
        }
        if matches!(ty, syn::Type::Verbatim(_) | syn::Type::Macro(_)) {
            self.refuse();
        } else {
            visit::visit_type(self, ty);
        }
    }

    fn visit_macro(&mut self, _: &'ast syn::Macro) {
        self.refuse();
    }
}

fn single_expression(block: &syn::Block) -> bool {
    matches!(block.stmts.as_slice(), [syn::Stmt::Expr(_, None)])
}
