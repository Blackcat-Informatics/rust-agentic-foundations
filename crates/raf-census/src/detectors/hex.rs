// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Base16 syntax recognition over the gate's existing bounded parse.

use proc_macro2::Span;
use syn::visit::Visit;

use super::RawHit;

pub(super) const RULES: [&str; 3] = ["hex-format-loop", "hex-pair-radix", "hex-digit-radix"];

struct Hex {
    repeated: usize,
    hits: Vec<RawHit>,
}

impl Hex {
    fn hit(&mut self, rule: &'static str, span: Span, message: &str) {
        self.hits.push(RawHit {
            rule,
            byte: span.byte_range().start,
            line: span.start().line,
            message: message.into(),
        });
    }

    fn repeated(&mut self, walk: impl FnOnce(&mut Self)) {
        self.repeated += 1;
        walk(self);
        self.repeated -= 1;
    }

    fn function(&mut self, walk: impl FnOnce(&mut Self)) {
        let previous = std::mem::replace(&mut self.repeated, 0);
        walk(self);
        self.repeated = previous;
    }
}

fn sixteen(expr: &syn::Expr) -> bool {
    matches!(expr, syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(value), .. })
        if crate::normalize::parse_integer(&value.to_string()).and_then(|value| value.value) == Some(16))
}

impl<'ast> Visit<'ast> for Hex {
    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.visit_expr(&node.expr);
        self.repeated(|this| this.visit_block(&node.body));
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        // Conditions are evaluated on every iteration too.
        self.repeated(|this| syn::visit::visit_expr_while(this, node));
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        self.repeated(|this| this.visit_block(&node.body));
    }

    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        self.repeated(|this| syn::visit::visit_expr_closure(this, node));
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(function) = node.func.as_ref()
            && function
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "from_str_radix")
            && node.args.len() == 2
            && node.args.iter().nth(1).is_some_and(sixteen)
        {
            let segments = &function.path.segments;
            let u8_type = segments.iter().rev().nth(1).is_some_and(|segment| segment.ident == "u8")
                || function.qself.as_ref().is_some_and(|qualified| matches!(qualified.ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("u8")));
            self.hit(
                if u8_type {
                    "hex-pair-radix"
                } else {
                    "hex-digit-radix"
                },
                segments.last().expect("radix path").ident.span(),
                "radix-16 decoding written through a general integer conversion",
            );
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "to_digit"
            && node.args.len() == 1
            && node.args.first().is_some_and(sixteen)
        {
            self.hit(
                "hex-digit-radix",
                node.method.span(),
                "a single hex digit decoded through a general radix conversion",
            );
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if let Ok(arguments) = node.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            let name = node
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string());
            if let Some(name) = name.as_deref()
                && self.repeated > 0
                && matches!(
                    name,
                    "format"
                        | "format_args"
                        | "write"
                        | "writeln"
                        | "print"
                        | "println"
                        | "eprint"
                        | "eprintln"
                )
            {
                let format_at = usize::from(matches!(name, "write" | "writeln"));
                if let Some(syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(format),
                    ..
                })) = arguments.iter().nth(format_at)
                    && super::format::has_hex_pair(&format.value())
                {
                    self.hit(
                        "hex-format-loop",
                        format.span(),
                        "two-digit hexadecimal formatting inside a loop or closure",
                    );
                }
            }
            for argument in &arguments {
                self.visit_expr(argument);
            }
        } else if let Ok(statements) = node.parse_body_with(syn::Block::parse_within) {
            for statement in &statements {
                self.visit_stmt(statement);
            }
        }
        syn::visit::visit_macro(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.function(|this| syn::visit::visit_item_fn(this, node));
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.function(|this| syn::visit::visit_impl_item_fn(this, node));
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.function(|this| syn::visit::visit_trait_item_fn(this, node));
    }
}

pub(super) fn collect(file: &syn::File) -> Vec<RawHit> {
    let mut visitor = Hex {
        repeated: 0,
        hits: Vec::new(),
    };
    visitor.visit_file(file);
    visitor.hits
}
