// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Lexical scope traversal shared by written-name policies.

use super::{Call, CallPath};
use crate::syntax::{AliasAttributes, Names, path_text};
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

    fn block(&mut self, node: &syn::Block, walk: impl FnOnce(&mut Self)) {
        let items = node.stmts.iter().filter_map(|statement| match statement {
            syn::Stmt::Item(item) => Some(item),
            _ => None,
        });
        self.scope(items, true, walk);
    }

    fn module(&mut self, node: &syn::ItemMod, walk: impl FnOnce(&mut Self)) {
        let items = node.content.iter().flat_map(|(_, items)| items);
        self.scope(items, false, walk);
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
        $crate::detectors::names::visit_scopes!(@file_modules);
        fn visit_block(&mut self, node: &'ast syn::Block) {
            self.block(node, |this| syn::visit::visit_block(this, node));
        }
    };
    (without_functions_blocks_or_impls) => {
        $crate::detectors::names::visit_generic_items! {
            visit_item_struct(ItemStruct, generics),
            visit_item_union(ItemUnion, generics),
            visit_item_type(ItemType, generics),
            visit_item_trait(ItemTrait, generics),
        }
    };
    (@file_modules) => {
        fn visit_file(&mut self, node: &'ast syn::File) {
            self.file(node, |this| syn::visit::visit_file(this, node));
        }

        fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
            self.module(node, |this| syn::visit::visit_item_mod(this, node));
        }

    };
}

pub(super) use {visit_generic_items, visit_scopes};

/// An explicit block import can be handed to Source at its declaration. If
/// that import depends on another block binding, module-level lookup alone
/// cannot prove its target.
pub(super) fn call_path(
    names: &Names,
    path: &[String],
    absolute: bool,
    byte: usize,
    block_glob: bool,
) -> Option<Call> {
    let (head, tail) = path.split_first()?;
    let written = || Call {
        byte,
        path: CallPath::Written(path_text(path, absolute)),
        receiver: None,
    };
    if absolute || matches!(head.as_str(), "crate" | "self" | "super") {
        return Some(written());
    }
    if head == "Self" && tail.len() <= 1 {
        return Some(Call {
            byte,
            path: CallPath::SelfType(tail.join("::")),
            receiver: None,
        });
    }
    let Some(aliases) = names.aliases.get(head) else {
        return (!block_glob).then(written);
    };
    let mut bindings = aliases
        .iter()
        .rev()
        .filter(|alias| alias.attributes != AliasAttributes::Excluded);
    let Some(alias) = bindings.next() else {
        return (!block_glob && !aliases.iter().any(|alias| alias.frame == 0)).then(written);
    };
    if alias.attributes != AliasAttributes::Inert {
        return None;
    }
    let target = alias.path.first();
    let rooted = alias.absolute
        || target.is_some_and(|target| matches!(target.as_str(), "crate" | "self" | "super"));
    if alias.frame == 0 {
        if block_glob
            || bindings
                .clone()
                .any(|other| other.attributes != AliasAttributes::Inert)
        {
            return None;
        }
        if !aliases
            .iter()
            .any(|other| other.frame == 0 && other.attributes == AliasAttributes::Excluded)
        {
            return Some(written());
        }
        // Source's structural index can retain the excluded competitor. A
        // single rooted import can be checked from the same reading module.
        if !rooted || alias.external || bindings.next().is_some() {
            return None;
        }
    } else if bindings
        .next()
        .is_some_and(|other| other.frame == alias.frame)
    {
        return None;
    }
    let target = target?;
    if !rooted
        && (block_glob
            || names
                .aliases
                .get(target)
                .into_iter()
                .flatten()
                .any(|other| {
                    other.frame > 0
                        && other.frame <= alias.frame
                        && other.attributes != AliasAttributes::Excluded
                        && !std::ptr::eq(alias, other)
                }))
    {
        return None;
    }
    let mut expanded = alias.path.clone();
    expanded.extend_from_slice(tail);
    Some(Call {
        byte: if alias.frame == 0 {
            byte
        } else {
            alias.source_byte?
        },
        path: CallPath::Import(path_text(&expanded, alias.absolute)),
        receiver: None,
    })
}
