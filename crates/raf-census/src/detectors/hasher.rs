// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Written standard collection paths and lexical import aliases.

use crate::syntax::Names;
use syn::ext::IdentExt;
use syn::visit::Visit;

use super::RawHit;
use super::names::{ScopedNames, visit_scopes};

impl Names {
    fn required(&self, path: &syn::Path, index: usize) -> Option<usize> {
        let written: Vec<_> = path
            .segments
            .iter()
            .take(index + 1)
            .map(|segment| segment.ident.unraw().to_string())
            .collect();
        let written = self.resolve_written(written, path.leading_colon.is_some());
        let names: Vec<_> = written.iter().map(String::as_str).collect();
        match names.as_slice() {
            ["std", "collections", "HashMap"] | ["std", "collections", "hash_map", "HashMap"] => {
                Some(3)
            }
            ["std", "collections", "HashSet"] | ["std", "collections", "hash_set", "HashSet"] => {
                Some(2)
            }
            _ => None,
        }
    }

    fn explicit(&self, path: &syn::Path) -> bool {
        path.segments.len().checked_sub(1).is_some_and(|last| {
            self.required(path, last)
                .is_some_and(|required| arguments(&path.segments[last]) >= required)
        })
    }
}

fn arguments(segment: &syn::PathSegment) -> usize {
    match &segment.arguments {
        syn::PathArguments::AngleBracketed(arguments) => arguments
            .args
            .iter()
            .filter(|argument| matches!(argument, syn::GenericArgument::Type(_)))
            .count(),
        _ => 0,
    }
}

struct Hasher {
    names: Names,
    vouched: Option<*const syn::ExprPath>,
    hits: Vec<RawHit>,
}

impl Hasher {
    fn path(&mut self, path: &syn::Path, in_type: bool) {
        for (index, segment) in path.segments.iter().enumerate() {
            let Some(required) = self.names.required(path, index) else {
                continue;
            };
            let last = index + 1 == path.segments.len();
            let constructor = path.segments.iter().nth(index + 1).is_some_and(|next| {
                matches!(
                    next.ident.unraw().to_string().as_str(),
                    "new" | "default" | "from" | "from_iter" | "with_capacity"
                )
            });
            if arguments(segment) < required && ((last && in_type) || (!last && constructor)) {
                let span = segment.ident.span();
                self.hits.push(RawHit {
                    rule: "std-default-hasher",
                    byte: span.byte_range().start,
                    line: span.start().line,
                    message: "a standard hash collection leaves its hasher unspecified".into(),
                });
            }
        }
    }
}

impl ScopedNames for Hasher {
    fn names(&mut self) -> &mut Names {
        &mut self.names
    }
}

impl<'ast> Visit<'ast> for Hasher {
    visit_scopes!();

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.generics(&node.generics, |this| {
            syn::visit::visit_item_enum(this, node);
        });
    }

    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if node.qself.is_none() {
            self.path(&node.path, true);
        }
        syn::visit::visit_type_path(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if self.vouched != Some(std::ptr::from_ref(node)) && node.qself.is_none() {
            self.path(&node.path, false);
        }
        syn::visit::visit_expr_path(self, node);
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        let previous = self.vouched.take();
        if let syn::Pat::Type(annotation) = &node.pat
            && let syn::Type::Path(ty) = annotation.ty.as_ref()
            && self.names.explicit(&ty.path)
            && let Some(initializer) = &node.init
            && let syn::Expr::Call(call) = initializer.expr.as_ref()
            && let syn::Expr::Path(path) = call.func.as_ref()
        {
            self.vouched = Some(std::ptr::from_ref(path));
        }
        syn::visit::visit_local(self, node);
        self.vouched = previous;
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if let Ok(expressions) = node.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for expression in &expressions {
                self.visit_expr(expression);
            }
        }
        syn::visit::visit_macro(self, node);
    }
}

pub(super) fn collect(file: &syn::File) -> Vec<RawHit> {
    let mut visitor = Hasher {
        names: Names::default(),
        vouched: None,
        hits: Vec::new(),
    };
    visitor.visit_file(file);
    visitor.hits
}
