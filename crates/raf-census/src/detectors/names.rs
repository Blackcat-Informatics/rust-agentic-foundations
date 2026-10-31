// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Lexical scope traversal shared by written-name policies.

use crate::syntax::Names;
use syn::ext::IdentExt as _;

pub(super) trait ScopedNames: Sized {
    fn names(&mut self) -> &mut Names;

    fn file(&mut self, node: &syn::File, walk: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(self.names(), Names::file(&node.items));
        walk(self);
        *self.names() = outer;
    }

    fn generics(&mut self, generics: &syn::Generics, walk: impl FnOnce(&mut Self)) {
        if generics.type_params().next().is_none() {
            walk(self);
            return;
        }
        let outer = self.names().clone();
        self.names().enter_frame();
        for parameter in generics.type_params() {
            self.names().shadow(parameter.ident.unraw().to_string());
        }
        walk(self);
        *self.names() = outer;
    }

    fn scope<'a>(
        &mut self,
        items: impl IntoIterator<Item = &'a syn::Item>,
        inherit: bool,
        walk: impl FnOnce(&mut Self),
    ) {
        let mut items = items.into_iter().peekable();
        if inherit && items.peek().is_none() {
            walk(self);
            return;
        }
        let outer = self.names().clone();
        if inherit {
            self.names().enter_frame();
        } else {
            *self.names() = outer.module();
        }
        for item in items {
            self.names().item(item);
        }
        walk(self);
        *self.names() = outer;
    }
}

macro_rules! visit_generic_items {
    ($( $visit:ident($node:ident, $($field:ident).+) ),* $(,)?) => {
        $(
            fn $visit(&mut self, node: &'ast syn::$node) {
                self.generics(&node.$($field).+, |this| syn::visit::$visit(this, node));
            }
        )*
    };
}

macro_rules! visit_scopes {
    () => {
        $crate::detectors::names::visit_generic_items! {
            visit_item_fn(ItemFn, sig.generics),
            visit_impl_item_fn(ImplItemFn, sig.generics),
            visit_trait_item_fn(TraitItemFn, sig.generics),
            visit_item_impl(ItemImpl, generics),
            visit_item_struct(ItemStruct, generics),
            visit_item_union(ItemUnion, generics),
            visit_item_type(ItemType, generics),
            visit_item_trait(ItemTrait, generics),
        }

        fn visit_file(&mut self, node: &'ast syn::File) {
            self.file(node, |this| syn::visit::visit_file(this, node));
        }

        fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
            let items = node.content.iter().flat_map(|(_, items)| items);
            self.scope(items, false, |this| syn::visit::visit_item_mod(this, node));
        }

        fn visit_block(&mut self, node: &'ast syn::Block) {
            let items = node.stmts.iter().filter_map(|statement| match statement {
                syn::Stmt::Item(item) => Some(item),
                _ => None,
            });
            self.scope(items, true, |this| syn::visit::visit_block(this, node));
        }
    };
}

pub(super) use {visit_generic_items, visit_scopes};
