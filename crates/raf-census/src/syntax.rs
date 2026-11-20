// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Shared written import observations for syntax-only census policies.

use std::collections::BTreeMap;
use syn::ext::IdentExt;

/// Attributes that preserve the indexed declaration or expression.
pub(crate) fn inert_attributes(attributes: &[syn::Attribute]) -> bool {
    inert_attributes_with(attributes, &[])
}

/// Recognize inert attributes with the additional names valid in a specific syntax position.
pub(crate) fn inert_attributes_with(attributes: &[syn::Attribute], additional: &[&str]) -> bool {
    attributes.iter().all(|attribute| {
        attribute.path().get_ident().is_some_and(|name| {
            let name = name.to_string();
            matches!(
                name.as_str(),
                "doc"
                    | "allow"
                    | "expect"
                    | "warn"
                    | "deny"
                    | "forbid"
                    | "inline"
                    | "cold"
                    | "must_use"
                    | "deprecated"
                    | "track_caller"
            ) || additional.contains(&name.as_str())
        })
    })
}

/// The empty disjunction is false under every configuration.
pub(crate) fn definitely_excluded(attributes: &[syn::Attribute]) -> bool {
    use proc_macro2::{Delimiter, TokenTree};

    attributes.iter().any(|attribute| {
        let syn::Meta::List(cfg) = &attribute.meta else {
            return false;
        };
        if !cfg.path.is_ident("cfg") {
            return false;
        }
        let mut tokens = cfg.tokens.clone().into_iter();
        matches!(
            (tokens.next(), tokens.next(), tokens.next()),
            (Some(TokenTree::Ident(any)), Some(TokenTree::Group(group)), None)
                if any == "any" && group.delimiter() == Delimiter::Parenthesis
                    && group.stream().is_empty()
        )
    })
}

/// Attributes attached to the declarations that participate in lexical lookup.
pub(crate) fn item_attributes(item: &syn::Item) -> &[syn::Attribute] {
    match item {
        syn::Item::Const(item) => &item.attrs,
        syn::Item::Static(item) => &item.attrs,
        syn::Item::Fn(item) => &item.attrs,
        syn::Item::Struct(item) => &item.attrs,
        syn::Item::Enum(item) => &item.attrs,
        syn::Item::Union(item) => &item.attrs,
        syn::Item::Type(item) => &item.attrs,
        syn::Item::Trait(item) => &item.attrs,
        syn::Item::TraitAlias(item) => &item.attrs,
        syn::Item::Impl(item) => &item.attrs,
        syn::Item::Mod(item) => &item.attrs,
        syn::Item::Use(item) => &item.attrs,
        syn::Item::ForeignMod(item) => &item.attrs,
        syn::Item::ExternCrate(item) => &item.attrs,
        syn::Item::Macro(item) => &item.attrs,
        _ => &[],
    }
}

/// Attributes at this expression's own occurrence.
pub(crate) fn expression_attributes(node: &syn::Expr) -> Option<&[syn::Attribute]> {
    Some(match node {
        syn::Expr::Array(n) => &n.attrs,
        syn::Expr::Assign(n) => &n.attrs,
        syn::Expr::Async(n) => &n.attrs,
        syn::Expr::Await(n) => &n.attrs,
        syn::Expr::Binary(n) => &n.attrs,
        syn::Expr::Block(n) => &n.attrs,
        syn::Expr::Break(n) => &n.attrs,
        syn::Expr::Call(n) => &n.attrs,
        syn::Expr::Cast(n) => &n.attrs,
        syn::Expr::Closure(n) => &n.attrs,
        syn::Expr::Const(n) => &n.attrs,
        syn::Expr::Continue(n) => &n.attrs,
        syn::Expr::Field(n) => &n.attrs,
        syn::Expr::ForLoop(n) => &n.attrs,
        syn::Expr::Group(n) => &n.attrs,
        syn::Expr::If(n) => &n.attrs,
        syn::Expr::Index(n) => &n.attrs,
        syn::Expr::Infer(n) => &n.attrs,
        syn::Expr::Let(n) => &n.attrs,
        syn::Expr::Lit(n) => &n.attrs,
        syn::Expr::Loop(n) => &n.attrs,
        syn::Expr::Macro(n) => &n.attrs,
        syn::Expr::Match(n) => &n.attrs,
        syn::Expr::MethodCall(n) => &n.attrs,
        syn::Expr::Paren(n) => &n.attrs,
        syn::Expr::Path(n) => &n.attrs,
        syn::Expr::Range(n) => &n.attrs,
        syn::Expr::RawAddr(n) => &n.attrs,
        syn::Expr::Reference(n) => &n.attrs,
        syn::Expr::Repeat(n) => &n.attrs,
        syn::Expr::Return(n) => &n.attrs,
        syn::Expr::Struct(n) => &n.attrs,
        syn::Expr::Try(n) => &n.attrs,
        syn::Expr::TryBlock(n) => &n.attrs,
        syn::Expr::Tuple(n) => &n.attrs,
        syn::Expr::Unary(n) => &n.attrs,
        syn::Expr::Unsafe(n) => &n.attrs,
        syn::Expr::Verbatim(_) => &[],
        syn::Expr::While(n) => &n.attrs,
        syn::Expr::Yield(n) => &n.attrs,
        _ => return None,
    })
}

#[derive(Clone, Default)]
pub(crate) struct Names {
    pub(crate) aliases: BTreeMap<String, Vec<Alias>>,
    prelude: BTreeMap<String, Vec<Alias>>,
    frame: usize,
}

/// One written binding; several imports can share a spelling across namespaces.
#[derive(Clone)]
pub(crate) struct Alias {
    pub(crate) path: Vec<String>,
    pub(crate) absolute: bool,
    pub(crate) source_byte: Option<usize>,
    pub(crate) external: bool,
    pub(crate) type_only: bool,
    pub(crate) attributes: AliasAttributes,
    /// The lexical declaration frame used by written-name observations.
    pub(crate) frame: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AliasAttributes {
    Inert,
    Uncertain,
    Excluded,
}

impl AliasAttributes {
    pub(crate) fn of(attributes: &[syn::Attribute]) -> Self {
        if inert_attributes(attributes) {
            Self::Inert
        } else if definitely_excluded(attributes) {
            Self::Excluded
        } else {
            Self::Uncertain
        }
    }
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
                    names.prelude.entry(name).or_default().push(alias.clone());
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
        self.resolve(path, absolute, self.frame, None, false)
    }

    /// A spelling supported only by declaration-preserving imports.
    pub(crate) fn resolve_inert_written(&self, path: Vec<String>, absolute: bool) -> Vec<String> {
        self.resolve(path, absolute, self.frame, None, true)
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
        self.resolve(path, absolute, self.frame, own, false)
    }

    fn resolve<'a>(
        &'a self,
        mut path: Vec<String>,
        absolute: bool,
        mut frame: usize,
        mut excluded: Option<&'a Alias>,
        inert_only: bool,
    ) -> Vec<String> {
        let original = path.clone();
        let mut absolute = absolute;
        let mut active: Vec<&Alias> = Vec::new();
        while let Some(name) = path.first() {
            let select = |aliases: &'a [Alias]| {
                aliases
                    .iter()
                    .rev()
                    .find(|alias| {
                        alias.frame <= frame
                            && !excluded.is_some_and(|own| std::ptr::eq(own, *alias))
                            && (!inert_only || alias.attributes != AliasAttributes::Excluded)
                    })
                    .map(|alias| (alias, aliases))
            };
            let Some((alias, alternatives)) = self
                .aliases
                .get(name)
                .filter(|_| !absolute)
                .and_then(|aliases| select(aliases))
                .or_else(|| self.prelude.get(name).and_then(|aliases| select(aliases)))
            else {
                break;
            };
            if inert_only
                && alternatives.iter().any(|other| {
                    other.frame == alias.frame && other.attributes == AliasAttributes::Uncertain
                })
            {
                return Vec::new();
            }
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

    fn alias(&mut self, local: String, alias: Alias) {
        self.aliases.entry(local).or_default().push(alias);
    }

    pub(crate) fn shadow(&mut self, name: String) {
        self.shadow_with(name, AliasAttributes::Inert);
    }

    fn shadow_with(&mut self, name: String, attributes: AliasAttributes) {
        self.alias(
            name,
            Alias {
                path: Vec::new(),
                absolute: false,
                source_byte: None,
                external: false,
                type_only: true,
                attributes,
                frame: self.frame,
            },
        );
    }

    pub(crate) fn import_tree(
        &mut self,
        tree: &syn::UseTree,
        absolute: bool,
        attributes: &[syn::Attribute],
    ) {
        let attributes = AliasAttributes::of(attributes);
        visit_import_paths(tree, |path, leaf| {
            if let Some(alias) = leaf.alias() {
                self.alias(
                    alias.unraw().to_string(),
                    Alias {
                        path: path.iter().map(|ident| ident.unraw().to_string()).collect(),
                        absolute,
                        source_byte: path.first().map(|ident| ident.span().byte_range().start),
                        external: false,
                        type_only: matches!(leaf, ImportLeaf::SelfType(_)),
                        attributes,
                        frame: self.frame,
                    },
                );
            }
        });
    }

    pub(crate) fn item(&mut self, item: &syn::Item) {
        let (shadow, attributes) = match item {
            syn::Item::Use(item) => {
                self.import_tree(&item.tree, item.leading_colon.is_some(), &item.attrs);
                return;
            }
            syn::Item::ExternCrate(item) => {
                let name = item.rename.as_ref().map_or(&item.ident, |(_, name)| name);
                self.alias(
                    name.unraw().to_string(),
                    Alias {
                        path: vec![item.ident.unraw().to_string()],
                        absolute: true,
                        source_byte: Some(item.ident.span().byte_range().start),
                        external: true,
                        type_only: false,
                        attributes: AliasAttributes::of(&item.attrs),
                        frame: self.frame,
                    },
                );
                return;
            }
            syn::Item::Struct(item) => (&item.ident, &item.attrs),
            syn::Item::Enum(item) => (&item.ident, &item.attrs),
            syn::Item::Type(item) => (&item.ident, &item.attrs),
            syn::Item::Mod(item) => (&item.ident, &item.attrs),
            syn::Item::Trait(item) => (&item.ident, &item.attrs),
            syn::Item::TraitAlias(item) => (&item.ident, &item.attrs),
            syn::Item::Union(item) => (&item.ident, &item.attrs),
            _ => return,
        };
        self.shadow_with(shadow.unraw().to_string(), AliasAttributes::of(attributes));
    }
}

/// The terminal leaf of a written import.
pub(crate) enum ImportLeaf<'a> {
    Named(&'a syn::Ident),
    SelfType(Option<&'a syn::Ident>),
    Glob,
}

impl<'a> ImportLeaf<'a> {
    pub(crate) fn alias(&self) -> Option<&'a syn::Ident> {
        match *self {
            Self::Named(alias) => Some(alias),
            Self::SelfType(alias) => alias,
            Self::Glob => None,
        }
    }
}

/// Observe each written import path, retaining original identifier spans.
/// A glob has no bound alias; `self` names the prefix it imports.
pub(crate) fn visit_import_paths(
    tree: &syn::UseTree,
    mut emit: impl FnMut(&[syn::Ident], ImportLeaf<'_>),
) {
    walk_import(tree, &mut Vec::new(), &mut emit);
}

fn walk_import(
    tree: &syn::UseTree,
    prefix: &mut Vec<syn::Ident>,
    emit: &mut impl FnMut(&[syn::Ident], ImportLeaf<'_>),
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
        syn::UseTree::Name(name) if name.ident == "self" => {
            emit(prefix, ImportLeaf::SelfType(prefix.last()));
        }
        syn::UseTree::Name(name) => {
            prefix.push(name.ident.clone());
            emit(prefix, ImportLeaf::Named(&name.ident));
            prefix.pop();
        }
        syn::UseTree::Rename(rename) if rename.ident == "self" => {
            emit(prefix, ImportLeaf::SelfType(Some(&rename.rename)));
        }
        syn::UseTree::Rename(rename) => {
            prefix.push(rename.ident.clone());
            emit(prefix, ImportLeaf::Named(&rename.rename));
            prefix.pop();
        }
        syn::UseTree::Glob(_) => emit(prefix, ImportLeaf::Glob),
    }
}
