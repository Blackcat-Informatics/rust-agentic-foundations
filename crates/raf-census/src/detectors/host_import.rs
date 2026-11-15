// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Host function imports declared in `extern` blocks, by attribute identity.
//!
//! A hit's message is the import's identity: the namespace attributes of the
//! block and of the item, then the imported name. The gate compares those
//! messages to keep the imports of a job's home only.

use syn::ext::IdentExt as _;
use syn::visit::Visit;

use super::RawHit;

pub(super) const RULE: &str = "foreign-host-import";

/// Attribute keys that name the namespace a host function lives in.
const NAMESPACE_KEYS: [&str; 3] = ["module", "wasm_import_module", "js_namespace"];
/// Attribute keys that rename the imported function.
const NAME_KEYS: [&str; 2] = ["js_name", "link_name"];

/// The written text of an attribute value: a string, a path or a list of paths.
fn value_text(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(text),
            ..
        }) => Some(text.value()),
        syn::Expr::Path(path) => Some(
            path.path
                .segments
                .iter()
                .map(|segment| segment.ident.unraw().to_string())
                .collect::<Vec<_>>()
                .join("::"),
        ),
        syn::Expr::Array(array) => array
            .elems
            .iter()
            .map(value_text)
            .collect::<Option<Vec<_>>>()
            .map(|parts| parts.join("::")),
        _ => None,
    }
}

/// Every `key = value` pair of the attributes that carry host binding
/// arguments: `wasm_bindgen(..)`, `link(..)`, `link_name = ".."` and
/// `wasm_import_module = ".."`.
fn pairs(attrs: &[syn::Attribute]) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for attribute in attrs {
        match &attribute.meta {
            syn::Meta::NameValue(pair) => {
                if let Some(key) = pair.path.get_ident().map(ToString::to_string)
                    && (key == "link_name" || key == "wasm_import_module")
                    && let Some(value) = value_text(&pair.value)
                {
                    found.push((key, value));
                }
            }
            syn::Meta::List(list)
                if list.path.is_ident("wasm_bindgen") || list.path.is_ident("link") =>
            {
                // An argument this reader does not recognise carries no identity.
                let _ = attribute.parse_nested_meta(|meta| {
                    let key = meta.path.get_ident().map(ToString::to_string);
                    if meta.input.peek(syn::Token![=]) {
                        let value: syn::Expr = meta.value()?.parse()?;
                        if let (Some(key), Some(value)) = (key, value_text(&value)) {
                            found.push((key, value));
                        }
                    } else if meta.input.peek(syn::token::Paren) {
                        let content;
                        syn::parenthesized!(content in meta.input);
                        content.parse::<proc_macro2::TokenStream>()?;
                    }
                    Ok(())
                });
            }
            _ => {}
        }
    }
    found
}

struct Imports {
    block: Vec<(String, String)>,
    hits: Vec<RawHit>,
}

impl<'ast> Visit<'ast> for Imports {
    fn visit_item_foreign_mod(&mut self, node: &'ast syn::ItemForeignMod) {
        let outer = std::mem::replace(&mut self.block, pairs(&node.attrs));
        syn::visit::visit_item_foreign_mod(self, node);
        self.block = outer;
    }

    fn visit_foreign_item_fn(&mut self, node: &'ast syn::ForeignItemFn) {
        let item = pairs(&node.attrs);
        let mut identity: Vec<String> = Vec::new();
        for (key, value) in self.block.iter().chain(&item) {
            if NAMESPACE_KEYS.contains(&key.as_str()) {
                identity.push(value.clone());
            }
        }
        let renamed = item
            .iter()
            .rev()
            .find(|(key, _)| NAME_KEYS.contains(&key.as_str()))
            .map(|(_, value)| value.clone());
        identity.push(renamed.unwrap_or_else(|| node.sig.ident.unraw().to_string()));
        RawHit::push(
            &mut self.hits,
            RULE,
            node.sig.ident.span(),
            &format!(
                "host function `{}` is imported by an extern block",
                identity.join("::")
            ),
        );
    }
}

pub(super) fn collect(file: &syn::File) -> Vec<RawHit> {
    let mut visitor = Imports {
        block: Vec::new(),
        hits: Vec::new(),
    };
    visitor.visit_file(file);
    visitor.hits
}
