// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Shared written import observations for syntax-only census policies.

use std::collections::{BTreeMap, BTreeSet};
use syn::ext::IdentExt;

#[derive(Clone, Default)]
pub(crate) struct Names {
    pub(crate) aliases: BTreeMap<String, Vec<String>>,
    pub(crate) absolute: BTreeSet<String>,
}

impl Names {
    fn alias(&mut self, local: String, path: Vec<String>, absolute: bool) {
        if absolute {
            self.absolute.insert(local.clone());
        } else {
            self.absolute.remove(&local);
        }
        self.aliases.insert(local, path);
    }

    pub(crate) fn shadow(&mut self, name: String) {
        self.alias(name, Vec::new(), false);
    }

    pub(crate) fn item(&mut self, item: &syn::Item) {
        match item {
            syn::Item::Use(item) => {
                visit_import_paths(&item.tree, |path, alias| {
                    if let Some(alias) = alias {
                        self.alias(
                            alias.unraw().to_string(),
                            path.iter().map(|ident| ident.unraw().to_string()).collect(),
                            item.leading_colon.is_some(),
                        );
                    }
                });
            }
            syn::Item::Struct(item) => self.shadow(item.ident.unraw().to_string()),
            syn::Item::Enum(item) => self.shadow(item.ident.unraw().to_string()),
            syn::Item::Type(item) => self.shadow(item.ident.unraw().to_string()),
            syn::Item::Mod(item) => self.shadow(item.ident.unraw().to_string()),
            _ => {}
        }
    }
}

/// Observe each written import path, retaining original identifier spans.
/// A glob has no bound alias; `self` names the prefix it imports.
pub(crate) fn visit_import_paths(
    tree: &syn::UseTree,
    mut emit: impl FnMut(&[syn::Ident], Option<&syn::Ident>),
) {
    walk_import(tree, &mut Vec::new(), &mut emit);
}

fn walk_import(
    tree: &syn::UseTree,
    prefix: &mut Vec<syn::Ident>,
    emit: &mut impl FnMut(&[syn::Ident], Option<&syn::Ident>),
) {
    match tree {
        syn::UseTree::Path(path) => {
            prefix.push(path.ident.clone());
            walk_import(&path.tree, prefix, emit);
            prefix.pop();
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                walk_import(item, prefix, emit);
            }
        }
        syn::UseTree::Name(name) if name.ident == "self" => emit(prefix, prefix.last()),
        syn::UseTree::Name(name) => {
            prefix.push(name.ident.clone());
            emit(prefix, Some(&name.ident));
            prefix.pop();
        }
        syn::UseTree::Rename(rename) => {
            if rename.ident != "self" {
                prefix.push(rename.ident.clone());
            }
            emit(prefix, Some(&rename.rename));
            if rename.ident != "self" {
                prefix.pop();
            }
        }
        syn::UseTree::Glob(_) => emit(prefix, None),
    }
}
