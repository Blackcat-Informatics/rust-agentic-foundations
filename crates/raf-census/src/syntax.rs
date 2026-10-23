// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Shared written import observations for syntax-only census policies.

use std::collections::BTreeMap;
use syn::ext::IdentExt;

#[derive(Clone, Default)]
pub(crate) struct Names {
    pub(crate) aliases: BTreeMap<String, Vec<Alias>>,
}

/// One written binding; several imports can share a spelling across namespaces.
#[derive(Clone)]
pub(crate) struct Alias {
    pub(crate) path: Vec<String>,
    pub(crate) absolute: bool,
    pub(crate) source_byte: Option<usize>,
}

pub(crate) fn path_text(path: &[String], absolute: bool) -> String {
    format!("{}{}", if absolute { "::" } else { "" }, path.join("::"))
}

impl Names {
    fn alias(
        &mut self,
        local: String,
        path: Vec<String>,
        absolute: bool,
        source_byte: Option<usize>,
    ) {
        self.aliases.entry(local).or_default().push(Alias {
            path,
            absolute,
            source_byte,
        });
    }

    pub(crate) fn shadow(&mut self, name: String) {
        self.alias(name, Vec::new(), false, None);
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
                            path.first().map(|ident| ident.span().byte_range().start),
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
