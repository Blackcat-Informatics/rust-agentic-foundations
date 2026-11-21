// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Additional first-party inputs consumed by Rust's file-inclusion macros.

use crate::Error;
use proc_macro2::{Ident, TokenStream, TokenTree};
use purrdf_core::ContentDigest;
use raf_source::{LoadOptions, RelPath, Tree, Workspace, prepare_source};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use syn::{parse::Parser as _, visit::Visit as _};

// Only an actual cfg on a parsed declaration proves its contents are inactive.
// Identical tokens inside a macro definition may expand in a shipping context.
fn test_cfg(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg") && attr.parse_args::<Ident>().is_ok_and(|cfg| cfg == "test")
    })
}

#[derive(Default)]
struct TestItems(Vec<Range<usize>>);

impl TestItems {
    fn test_only_file(&mut self, file: &syn::File) -> bool {
        if !crate::syntax::inert_attributes_with(&file.attrs, &["cfg"]) {
            return false;
        }
        if test_cfg(&file.attrs) {
            return true;
        }
        self.visit_file(file);
        false
    }

    fn walk(&mut self, node: &impl syn::spanned::Spanned, attrs: &[syn::Attribute]) -> bool {
        // An attribute macro can rewrite nested declarations before their cfg is applied.
        // Keep all its raw tokens in the inclusion scan.
        if !crate::syntax::inert_attributes_with(attrs, &["cfg"]) {
            return false;
        }
        if test_cfg(attrs) {
            self.0.push(node.span().byte_range());
            false
        } else {
            true
        }
    }
}

// These syntax nodes store their attributes directly. Descending only into
// live parents keeps the excluded spans disjoint.
macro_rules! attributed_nodes {
    ($($visit:ident: $node:ident),+ $(,)?) => {
        $(fn $visit(&mut self, node: &'ast syn::$node) {
            if self.walk(node, &node.attrs) {
                syn::visit::$visit(self, node);
            }
        })+
    }
}

impl<'ast> syn::visit::Visit<'ast> for TestItems {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        if self.walk(item, crate::syntax::item_attributes(item)) {
            syn::visit::visit_item(self, item);
        }
    }

    fn visit_expr(&mut self, expr: &'ast syn::Expr) {
        if self.walk(
            expr,
            crate::syntax::expression_attributes(expr).unwrap_or_default(),
        ) {
            syn::visit::visit_expr(self, expr);
        }
    }

    attributed_nodes! {
        visit_impl_item_fn: ImplItemFn, visit_impl_item_const: ImplItemConst,
        visit_impl_item_type: ImplItemType, visit_impl_item_macro: ImplItemMacro,
        visit_trait_item_fn: TraitItemFn, visit_trait_item_const: TraitItemConst,
        visit_trait_item_type: TraitItemType, visit_trait_item_macro: TraitItemMacro,
        visit_foreign_item_fn: ForeignItemFn, visit_foreign_item_static: ForeignItemStatic,
        visit_foreign_item_type: ForeignItemType, visit_foreign_item_macro: ForeignItemMacro,
        visit_local: Local, visit_stmt_macro: StmtMacro, visit_field: Field,
        visit_variant: Variant, visit_arm: Arm, visit_field_value: FieldValue,
    }
}

fn file_macro(name: &Ident) -> bool {
    [
        "include",
        "r#include",
        "include_str",
        "r#include_str",
        "include_bytes",
        "r#include_bytes",
    ]
    .iter()
    .any(|spelling| name == *spelling)
}

fn paths(path: &RelPath, text: &str, fragment: bool) -> Result<Vec<(RelPath, bool)>, Error> {
    if !(text.contains("include") || fragment && text.contains("mod")) {
        return Ok(Vec::new());
    }
    let parent = path.parent().unwrap_or_else(RelPath::root);
    let scan = |tokens, excluded: &[Range<usize>]| {
        let mut inputs = Vec::new();
        let mut groups = vec![tokens];
        while let Some(group) = groups.pop() {
            collect(
                group,
                &parent,
                path,
                fragment,
                excluded,
                &mut groups,
                &mut inputs,
            )?;
        }
        Ok(inputs)
    };
    let prepared = prepare_source(path, text)?;
    prepared.with_lexed(|lexed| {
        let tokens = lexed.tokens().clone();
        let mut excluded = TestItems::default();
        if text.contains("cfg") {
            if fragment {
                // include! accepts items or an expression at its call site.
                // Other grammars give no proof that any token is test-only.
                if let Ok(items) = syn::parse2::<syn::File>(tokens.clone()) {
                    if excluded.test_only_file(&items) {
                        return Ok(Vec::new());
                    }
                } else if let Ok(expr) = syn::parse2::<syn::Expr>(tokens.clone()) {
                    excluded.visit_expr(&expr);
                }
            } else if lexed.parse(|file| excluded.test_only_file(file))? {
                return Ok(Vec::new());
            }
            excluded.0.sort_unstable_by_key(|range| range.start);
        }
        scan(tokens, &excluded.0)
    })?
}

fn literal_path(input: syn::parse::ParseStream<'_>) -> syn::Result<syn::LitStr> {
    let literal = input.parse()?;
    input.parse::<Option<syn::Token![,]>>()?;
    Ok(literal)
}

fn collect(
    stream: TokenStream,
    parent: &RelPath,
    path: &RelPath,
    fragment: bool,
    excluded: &[Range<usize>],
    groups: &mut Vec<TokenStream>,
    inputs: &mut Vec<(RelPath, bool)>,
) -> Result<(), Error> {
    let mut tokens = stream.into_iter().peekable();
    while let Some(token) = tokens.next() {
        if !excluded.is_empty() {
            let offset = token.span().byte_range().start;
            let next = excluded.partition_point(|range| range.end <= offset);
            if excluded
                .get(next)
                .is_some_and(|range| range.contains(&offset))
            {
                continue;
            }
        }
        match token {
            TokenTree::Group(group) => groups.push(group.stream()),
            TokenTree::Ident(name) if fragment && name == "mod" => {
                if matches!(tokens.peek(), Some(TokenTree::Ident(_))) {
                    tokens.next();
                    if matches!(tokens.peek(), Some(TokenTree::Punct(p)) if p.as_char() == ';') {
                        return Err(Error::Options(format!(
                            "{path}: an included Rust fragment declares a file-backed module whose compiler inputs cannot be established"
                        )));
                    }
                }
            }
            TokenTree::Ident(name) if file_macro(&name) => {
                let colon =
                    |token: &TokenTree| matches!(token, TokenTree::Punct(p) if p.as_char() == ':');
                let grouped_import = tokens.next_if(colon).is_some()
                    && tokens.next_if(colon).is_some()
                    && matches!(tokens.peek(), Some(TokenTree::Group(group)) if group.stream().into_iter().any(|token| matches!(token, TokenTree::Ident(id) if id == "self")));
                if grouped_import
                    || matches!(tokens.peek(), Some(TokenTree::Ident(next)) if next == "as")
                {
                    return Err(Error::Options(format!(
                        "{path}: an aliased {name}! can read files outside the recorded module tree; invoke the file-inclusion macro by its original name"
                    )));
                }
                if !matches!(tokens.peek(), Some(TokenTree::Punct(punct)) if punct.as_char() == '!')
                {
                    continue;
                }
                tokens.next();
                let literal = match tokens.next() {
                    Some(TokenTree::Group(args)) => literal_path.parse2(args.stream()).ok(),
                    _ => None,
                }.ok_or_else(|| Error::Options(format!("{path}: cannot attest the input of dynamic {name}!; the build must identify a literal workspace file")))?;
                let relative = parent
                    .join(&literal.value())
                    .map_err(|e| Error::Options(format!("{path}: {name}! input: {e}")))?;
                inputs.push((relative, name == "include" || name == "r#include"));
            }
            _ => (),
        }
    }
    Ok(())
}

pub(super) fn add(
    tree: &impl Tree,
    workspace: &Workspace,
    options: &LoadOptions,
    files: &mut BTreeMap<RelPath, ContentDigest>,
) -> Result<(), Error> {
    let mut parsed = BTreeSet::new();
    let mut pending = Vec::new();
    for file in workspace.files().filter(|file| file.shipping) {
        parsed.insert(file.path.clone());
        pending.extend(paths(&file.path, &file.text, false)?);
    }
    let mut remaining = options.max_total_bytes;
    while let Some((path, rust)) = pending.pop() {
        if (!rust || parsed.contains(&path)) && files.contains_key(&path) {
            continue;
        }
        let bytes = tree
            .read_bytes(&path, options.max_file_bytes.min(remaining as u64))
            .map_err(|source| super::file_error(&path, source))?;
        remaining -= bytes.len();
        files.insert(path.clone(), ContentDigest::of(&bytes));
        if rust && parsed.insert(path.clone()) {
            let text = std::str::from_utf8(&bytes).map_err(|error| {
                Error::Options(format!("{path}: included Rust is not UTF-8: {error}"))
            })?;
            pending.extend(paths(&path, text, true)?);
        }
    }
    Ok(())
}
