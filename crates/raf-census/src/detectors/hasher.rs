// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Written standard collection paths and lexical import aliases.

use std::collections::BTreeMap;
use syn::visit::Visit;

use super::RawHit;

#[derive(Clone, Default)]
struct Names(BTreeMap<String, Vec<String>>);

impl Names {
    fn imports(&mut self, tree: &syn::UseTree, prefix: &mut Vec<String>) {
        match tree {
            syn::UseTree::Path(path) => {
                prefix.push(path.ident.to_string());
                self.imports(&path.tree, prefix);
                prefix.pop();
            }
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    self.imports(item, prefix);
                }
            }
            syn::UseTree::Name(name) => {
                let mut path = prefix.clone();
                let local = if name.ident == "self" {
                    prefix.last().cloned().unwrap_or_default()
                } else {
                    path.push(name.ident.to_string());
                    name.ident.to_string()
                };
                self.0.insert(local, path);
            }
            syn::UseTree::Rename(rename) => {
                let mut path = prefix.clone();
                if rename.ident != "self" {
                    path.push(rename.ident.to_string());
                }
                self.0.insert(rename.rename.to_string(), path);
            }
            syn::UseTree::Glob(_) => {}
        }
    }

    fn item(&mut self, item: &syn::Item) {
        match item {
            syn::Item::Use(item) => self.imports(&item.tree, &mut Vec::new()),
            syn::Item::Struct(item) => {
                self.0.insert(item.ident.to_string(), Vec::new());
            }
            syn::Item::Enum(item) => {
                self.0.insert(item.ident.to_string(), Vec::new());
            }
            syn::Item::Type(item) => {
                self.0.insert(item.ident.to_string(), Vec::new());
            }
            syn::Item::Mod(item) => {
                self.0.insert(item.ident.to_string(), Vec::new());
            }
            _ => {}
        }
    }

    fn required(&self, path: &syn::Path, index: usize) -> Option<usize> {
        let mut written: Vec<_> = path
            .segments
            .iter()
            .take(index + 1)
            .map(|segment| segment.ident.to_string())
            .collect();
        if path.leading_colon.is_none() {
            for _ in 0..=self.0.len() {
                let Some(alias) = written.first().and_then(|first| self.0.get(first)) else {
                    break;
                };
                let mut resolved = alias.clone();
                resolved.extend(written.iter().skip(1).cloned());
                if resolved == written {
                    break;
                }
                written = resolved;
            }
        }
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
    fn generics(&mut self, generics: &syn::Generics, walk: impl FnOnce(&mut Self)) {
        let outer = self.names.clone();
        for parameter in generics.type_params() {
            self.names.0.insert(parameter.ident.to_string(), Vec::new());
        }
        walk(self);
        self.names = outer;
    }

    fn path(&mut self, path: &syn::Path, in_type: bool) {
        for (index, segment) in path.segments.iter().enumerate() {
            let Some(required) = self.names.required(path, index) else {
                continue;
            };
            let last = index + 1 == path.segments.len();
            let constructor = path.segments.iter().nth(index + 1).is_some_and(|next| {
                matches!(
                    next.ident.to_string().as_str(),
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

impl<'ast> Visit<'ast> for Hasher {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.generics(&node.sig.generics, |this| {
            syn::visit::visit_item_fn(this, node);
        });
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.generics(&node.sig.generics, |this| {
            syn::visit::visit_impl_item_fn(this, node);
        });
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.generics(&node.sig.generics, |this| {
            syn::visit::visit_trait_item_fn(this, node);
        });
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        self.generics(&node.generics, |this| {
            syn::visit::visit_item_impl(this, node);
        });
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.generics(&node.generics, |this| {
            syn::visit::visit_item_struct(this, node);
        });
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.generics(&node.generics, |this| {
            syn::visit::visit_item_enum(this, node);
        });
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        self.generics(&node.generics, |this| {
            syn::visit::visit_item_type(this, node);
        });
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        self.generics(&node.generics, |this| {
            syn::visit::visit_item_trait(this, node);
        });
    }

    fn visit_file(&mut self, file: &'ast syn::File) {
        for item in &file.items {
            self.names.item(item);
        }
        syn::visit::visit_file(self, file);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let outer = self.names.clone();
        // A module has its own imports, rather than inheriting parent imports.
        self.names = Names::default();
        if let Some((_, items)) = &node.content {
            for item in items {
                self.names.item(item);
            }
        }
        syn::visit::visit_item_mod(self, node);
        self.names = outer;
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let outer = self.names.clone();
        for statement in &block.stmts {
            if let syn::Stmt::Item(item) = statement {
                self.names.item(item);
            }
        }
        syn::visit::visit_block(self, block);
        self.names = outer;
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
