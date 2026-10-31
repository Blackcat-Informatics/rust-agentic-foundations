// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Shared written import observations for syntax-only census policies.

use std::collections::BTreeMap;
use syn::ext::IdentExt;

#[derive(Clone, Default)]
pub(crate) struct Names {
    pub(crate) aliases: BTreeMap<String, Vec<Alias>>,
    prelude: BTreeMap<String, Alias>,
    frame: usize,
}

/// One written binding; several imports can share a spelling across namespaces.
#[derive(Clone)]
pub(crate) struct Alias {
    pub(crate) path: Vec<String>,
    pub(crate) absolute: bool,
    pub(crate) source_byte: Option<usize>,
    pub(crate) external: bool,
    /// The lexical declaration frame used by written-name observations.
    pub(crate) frame: usize,
}

pub(crate) fn path_text(path: &[String], absolute: bool) -> String {
    format!("{}{}", if absolute { "::" } else { "" }, path.join("::"))
}

impl Names {
    pub(crate) fn file(items: &[syn::Item]) -> Self {
        let mut names = Self::default();
        for item in items {
            names.item(item);
            if let syn::Item::ExternCrate(item) = item {
                let name = item.rename.as_ref().map_or(&item.ident, |(_, name)| name);
                let name = name.unraw().to_string();
                if let Some(alias) = names.aliases.get(&name).and_then(|aliases| aliases.last()) {
                    names.prelude.insert(name, alias.clone());
                }
            }
        }
        names
    }

    pub(crate) fn module(&self) -> Self {
        Self {
            aliases: BTreeMap::new(),
            prelude: self.prelude.clone(),
            frame: 0,
        }
    }

    pub(crate) fn enter_frame(&mut self) {
        self.frame += 1;
    }

    pub(crate) fn resolve_written(&self, path: Vec<String>, absolute: bool) -> Vec<String> {
        self.resolve(path, absolute, self.frame, None)
    }

    pub(crate) fn resolve_import(
        &self,
        path: Vec<String>,
        absolute: bool,
        local: Option<&syn::Ident>,
        source_byte: Option<usize>,
    ) -> Vec<String> {
        let own = local
            .and_then(|name| self.aliases.get(&name.unraw().to_string()))
            .and_then(|aliases| {
                aliases
                    .iter()
                    .rev()
                    .find(|alias| alias.frame == self.frame && alias.source_byte == source_byte)
            });
        self.resolve(path, absolute, self.frame, own)
    }

    fn resolve<'a>(
        &'a self,
        mut path: Vec<String>,
        absolute: bool,
        mut frame: usize,
        mut excluded: Option<&'a Alias>,
    ) -> Vec<String> {
        let original = path.clone();
        let mut absolute = absolute;
        let mut active: Vec<&Alias> = Vec::new();
        loop {
            let Some(alias) = path.first().and_then(|name| {
                if absolute {
                    self.prelude.get(name)
                } else {
                    self.aliases
                        .get(name)
                        .and_then(|aliases| {
                            aliases.iter().rev().find(|alias| {
                                alias.frame <= frame
                                    && !excluded.is_some_and(|own| std::ptr::eq(own, *alias))
                            })
                        })
                        .or_else(|| self.prelude.get(name))
                }
            }) else {
                break;
            };
            if active.iter().any(|known| std::ptr::eq(*known, alias)) {
                // Cyclic imports cannot establish a native identity. Retain
                // the written observation instead of a partial expansion.
                return original;
            }
            if alias.path.is_empty() {
                return Vec::new();
            }
            active.push(alias);
            let mut expanded = alias.path.clone();
            expanded.extend(path.iter().skip(1).cloned());
            path = expanded;
            if alias.external {
                break;
            }
            // An import target reads its declaration frame, without its own
            // binding. Younger lexical shadows cannot rewrite that target.
            absolute = alias.absolute;
            frame = alias.frame;
            excluded = Some(alias);
        }
        path
    }

    fn alias(
        &mut self,
        local: String,
        path: Vec<String>,
        absolute: bool,
        source_byte: Option<usize>,
        external: bool,
    ) {
        self.aliases.entry(local).or_default().push(Alias {
            path,
            absolute,
            source_byte,
            external,
            frame: self.frame,
        });
    }

    pub(crate) fn shadow(&mut self, name: String) {
        self.alias(name, Vec::new(), false, None, false);
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
                            false,
                        );
                    }
                });
            }
            syn::Item::ExternCrate(item) => {
                let name = item.rename.as_ref().map_or(&item.ident, |(_, name)| name);
                self.alias(
                    name.unraw().to_string(),
                    vec![item.ident.unraw().to_string()],
                    true,
                    Some(item.ident.span().byte_range().start),
                    true,
                );
            }
            syn::Item::Struct(item) => self.shadow(item.ident.unraw().to_string()),
            syn::Item::Enum(item) => self.shadow(item.ident.unraw().to_string()),
            syn::Item::Type(item) => self.shadow(item.ident.unraw().to_string()),
            syn::Item::Mod(item) => self.shadow(item.ident.unraw().to_string()),
            syn::Item::Trait(item) => self.shadow(item.ident.unraw().to_string()),
            syn::Item::TraitAlias(item) => self.shadow(item.ident.unraw().to_string()),
            syn::Item::Union(item) => self.shadow(item.ident.unraw().to_string()),
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
