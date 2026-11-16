// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Host function imports declared in `extern` blocks, by attribute identity.
//!
//! A hit's message is the import's identity: the namespace attributes of the
//! block and of the item, each with its kind, then the imported name. The gate
//! compares those messages to keep the imports of a job's home only. An
//! attribute argument the reader cannot place is refused, never skipped.

use std::collections::BTreeSet;

use syn::ext::IdentExt as _;
use syn::spanned::Spanned as _;
use syn::visit::Visit;

use super::{RawHit, Refusal};

pub(super) const RULE: &str = "foreign-host-import";

/// Arguments that select where a host function lives, by kind.
const NAMESPACES: [&str; 5] = [
    "module",
    "raw_module",
    "inline_js",
    "js_class",
    "js_namespace",
];
/// Arguments of `wasm_bindgen(..)` that do not change which host function is
/// imported.
const NEUTRAL: [&str; 14] = [
    "catch",
    "extends",
    "final",
    "param_description",
    "return_description",
    "skip_jsdoc",
    "skip_typescript",
    "structural",
    "typescript_type",
    "unchecked_param_type",
    "unchecked_return_type",
    "variadic",
    "vendor_prefix",
    "wasm_bindgen",
];

/// Which way a property import goes; a read and a write are different imports.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Access {
    Read,
    Write,
}

/// What identifies one import: where it lives, and what it is called.
#[derive(Default)]
struct Identity {
    namespaces: BTreeSet<(&'static str, String)>,
    name: Option<String>,
    /// A property access: the access and the written property, when it is written.
    property: Option<(Access, Option<String>)>,
    method: bool,
    constructor: bool,
    /// Whether a `js_namespace` argument was written on this block or item itself.
    namespace_written: bool,
}

fn refuse(span: proc_macro2::Span, message: impl Into<String>) -> Refusal {
    Refusal {
        rule: RULE,
        line: span.start().line,
        message: message.into(),
    }
}

/// Whether a path ends in `name`, so a re-exported `wasm_bindgen::prelude::wasm_bindgen` reads as
/// `wasm_bindgen`.
fn last_is(path: &syn::Path, name: &str) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == name)
}

/// A namespace entry, which needs the argument's value.
fn place(
    kind: &'static str,
    value: Option<String>,
    span: proc_macro2::Span,
) -> Result<(&'static str, String), Refusal> {
    value.map(|value| (kind, value)).ok_or_else(|| {
        refuse(
            span,
            format!("the host binding argument `{kind}` requires a value"),
        )
    })
}

/// The written text of an attribute value: a string, a path or a list of paths.
fn value_text(expr: &syn::Expr) -> Result<String, Refusal> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(text),
            ..
        }) => Ok(text.value()),
        syn::Expr::Path(path) => Ok(path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.unraw().to_string())
            .collect::<Vec<_>>()
            .join("::")),
        syn::Expr::Array(array) => Ok(array
            .elems
            .iter()
            .map(value_text)
            .collect::<Result<Vec<_>, _>>()?
            .join("::")),
        other => Err(refuse(
            other.span(),
            "a host binding argument value must be a string, a path or a list of paths",
        )),
    }
}

impl Identity {
    /// Read the attributes that carry host binding arguments:
    /// `wasm_bindgen(..)`, `link(..)`, `link_name = ".."` and the same inside
    /// `cfg_attr(..)`.
    fn attributes(&mut self, attrs: &[syn::Attribute]) -> Result<(), Refusal> {
        attrs
            .iter()
            .try_for_each(|attribute| self.meta(&attribute.meta))
    }

    fn meta(&mut self, meta: &syn::Meta) -> Result<(), Refusal> {
        match meta {
            syn::Meta::NameValue(pair) if pair.path.is_ident("link_name") => {
                self.name = Some(value_text(&pair.value)?);
            }
            syn::Meta::List(list) if list.path.is_ident("cfg_attr") => {
                let metas = list
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                    )
                    .map_err(|error| refuse(error.span(), error.to_string()))?;
                // The first entry is the predicate.
                for inner in metas.iter().skip(1) {
                    self.meta(inner)?;
                }
            }
            syn::Meta::List(list) if last_is(&list.path, "wasm_bindgen") => {
                self.arguments(list, true)?;
            }
            syn::Meta::List(list) if list.path.is_ident("link") => self.arguments(list, false)?,
            _ => {}
        }
        Ok(())
    }

    fn arguments(&mut self, list: &syn::MetaList, bindgen: bool) -> Result<(), Refusal> {
        let mut failure = None;
        let parsed = list.parse_nested_meta(|meta| {
            let Some(key) = meta.path.get_ident().map(|ident| ident.unraw().to_string()) else {
                failure = Some(refuse(
                    meta.path.span(),
                    "a host binding argument must be a plain name",
                ));
                return Ok(());
            };
            if meta.input.peek(syn::token::Paren) {
                if key == "cfg" && !bindgen {
                    // The predicate of `link(.., cfg(..))` decides whether the link applies and
                    // not which host function is imported, so it is read past.
                    let predicate;
                    syn::parenthesized!(predicate in meta.input);
                    predicate.parse::<proc_macro2::TokenStream>()?;
                    return Ok(());
                }
                failure = Some(refuse(
                    meta.path.span(),
                    format!("the host binding argument `{key}` does not take a parenthesized list"),
                ));
                return Ok(());
            }
            let value = if meta.input.peek(syn::Token![=]) {
                let expr: syn::Expr = meta.value()?.parse()?;
                Some(value_text(&expr))
            } else {
                None
            };
            let value = match value {
                Some(Ok(text)) => Some(text),
                Some(Err(error)) => {
                    failure = Some(error);
                    return Ok(());
                }
                None => None,
            };
            if let Err(error) = self.argument(&key, value, meta.path.span(), bindgen) {
                failure = Some(error);
            }
            Ok(())
        });
        if let Some(failure) = failure {
            return Err(failure);
        }
        parsed.map_err(|error| refuse(error.span(), error.to_string()))
    }

    fn argument(
        &mut self,
        key: &str,
        value: Option<String>,
        span: proc_macro2::Span,
        bindgen: bool,
    ) -> Result<(), Refusal> {
        let namespace = |kind: &'static str, value: Option<String>| place(kind, value, span);
        if bindgen && let Some(kind) = NAMESPACES.iter().copied().find(|known| *known == key) {
            self.namespace_written |= kind == "js_namespace";
            self.namespaces.insert(namespace(kind, value)?);
            return Ok(());
        }
        match (key, bindgen) {
            ("wasm_import_module", false) => {
                self.namespaces
                    .insert(namespace("wasm_import_module", value)?);
            }
            ("name", false) => {
                self.namespaces.insert(namespace("library", value)?);
            }
            ("kind" | "modifiers" | "import_name_type", false) => {}
            ("static_method_of", true) => {
                self.namespaces.insert(namespace("js_namespace", value)?);
            }
            ("js_name", true) => {
                self.name = Some(value.ok_or_else(|| {
                    refuse(span, "the host binding argument `js_name` requires a value")
                })?);
            }
            ("getter" | "setter", true) => {
                let access = if key == "getter" {
                    Access::Read
                } else {
                    Access::Write
                };
                if let Some((written, _)) = self.property {
                    let message = if written == access {
                        format!("the host binding argument `{key}` is written twice on one import")
                    } else {
                        "an import cannot be both a getter and a setter".to_owned()
                    };
                    return Err(refuse(span, message));
                }
                self.property = Some((access, value));
            }
            ("method", true) => self.method = true,
            ("constructor", true) => self.constructor = true,
            ("indexing_getter" | "indexing_setter" | "indexing_deleter", true) => {
                self.namespaces.insert(("indexing", key.to_owned()));
            }
            (neutral, true) if NEUTRAL.contains(&neutral) => {}
            _ => {
                return Err(refuse(
                    span,
                    format!(
                        "the host binding argument `{key}` is not one this detector can place; it reads module, raw_module, inline_js, js_class, js_namespace, static_method_of, method, js_name, getter, setter and indexing_* arguments of `wasm_bindgen`, and name and wasm_import_module of `link`, so name the host function with one of them"
                    ),
                ));
            }
        }
        Ok(())
    }

    /// The identity text, once the block and the item have been read.
    fn describe(&self, function: &syn::ForeignItemFn) -> Result<String, Refusal> {
        let declared = function.sig.ident.unraw().to_string();
        let mut namespaces = self.namespaces.clone();
        let has_class = namespaces.iter().any(|(kind, _)| *kind == "js_class");
        if self.method && !has_class {
            let receiver = match function.sig.inputs.first() {
                Some(syn::FnArg::Typed(argument)) => match argument.ty.as_ref() {
                    syn::Type::Reference(reference) => reference.elem.as_ref(),
                    other => other,
                },
                _ => {
                    return Err(refuse(
                        function.sig.ident.span(),
                        "a `method` import needs a typed first parameter naming its receiver",
                    ));
                }
            };
            let syn::Type::Path(path) = receiver else {
                return Err(refuse(
                    receiver.span(),
                    "a `method` import's receiver must be a named type",
                ));
            };
            let text = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.unraw().to_string())
                .collect::<Vec<_>>()
                .join("::");
            namespaces.insert(("method_of", text));
        }
        if self.constructor && !has_class {
            namespaces.insert(("constructs", constructed(function)?));
        }
        let written = |property: &Option<String>| property.clone().or_else(|| self.name.clone());
        let name = match &self.property {
            Some((Access::Read, property)) => {
                format!("get {}", written(property).unwrap_or(declared))
            }
            Some((Access::Write, property)) => format!(
                "set {}",
                written(property).unwrap_or_else(|| declared
                    .strip_prefix("set_")
                    .unwrap_or(&declared)
                    .to_owned())
            ),
            None if self.constructor => self
                .name
                .clone()
                .unwrap_or_else(|| "constructor".to_owned()),
            None => self.name.clone().unwrap_or(declared),
        };
        let place = namespaces
            .iter()
            .map(|(kind, value)| format!("{kind} {value}"))
            .collect::<Vec<_>>()
            .join(", ");
        Ok(if place.is_empty() {
            format!("host function `{name}` is imported by an extern block")
        } else {
            format!("host function `{name}` ({place}) is imported by an extern block")
        })
    }
}

/// The class a constructor import builds: its returned type, or the first type of a
/// returned `Result`.
fn constructed(function: &syn::ForeignItemFn) -> Result<String, Refusal> {
    let syn::ReturnType::Type(_, returned) = &function.sig.output else {
        return Err(refuse(
            function.sig.ident.span(),
            "a `constructor` import needs a return type naming its class",
        ));
    };
    let syn::Type::Path(path) = returned.as_ref() else {
        return Err(refuse(
            returned.span(),
            "a `constructor` import's return type must be a named type",
        ));
    };
    let last = path.path.segments.last();
    let inner = last.and_then(|segment| match &segment.arguments {
        syn::PathArguments::AngleBracketed(arguments) if segment.ident == "Result" => {
            arguments.args.iter().find_map(|argument| match argument {
                syn::GenericArgument::Type(syn::Type::Path(inner)) => Some(inner),
                _ => None,
            })
        }
        _ => None,
    });
    let named = inner.unwrap_or(path);
    Ok(named
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.unraw().to_string())
        .collect::<Vec<_>>()
        .join("::"))
}

struct Imports {
    block: Identity,
    hits: Vec<RawHit>,
    refusal: Option<Refusal>,
}

impl Imports {
    fn import(&mut self, function: &syn::ForeignItemFn) -> Result<(), Refusal> {
        let mut identity = Identity {
            namespaces: self.block.namespaces.clone(),
            name: self.block.name.clone(),
            ..Identity::default()
        };
        identity.attributes(&function.attrs)?;
        // The binding attribute reads one namespace path from the block or the item,
        // and which wins differs between releases, so both is refused.
        if identity.namespace_written && self.block.namespace_written {
            return Err(refuse(
                function.sig.ident.span(),
                "`js_namespace` is written on both an extern block and an item in it; write the full path on one of them",
            ));
        }
        let message = identity.describe(function)?;
        let span = function.sig.ident.span();
        self.hits.push(RawHit {
            rule: RULE,
            byte: span.byte_range().start,
            line: span.start().line,
            message,
        });
        Ok(())
    }
}

impl<'ast> Visit<'ast> for Imports {
    fn visit_item_foreign_mod(&mut self, node: &'ast syn::ItemForeignMod) {
        if self.refusal.is_some() {
            return;
        }
        let mut block = Identity::default();
        if let Err(refusal) = block.attributes(&node.attrs) {
            self.refusal = Some(refusal);
            return;
        }
        let outer = std::mem::replace(&mut self.block, block);
        syn::visit::visit_item_foreign_mod(self, node);
        self.block = outer;
    }

    fn visit_foreign_item_macro(&mut self, node: &'ast syn::ForeignItemMacro) {
        if self.refusal.is_none() {
            self.refusal = Some(refuse(
                node.mac.span(),
                "a macro invocation inside an extern block may declare host imports this detector cannot read; write the imports out",
            ));
        }
    }

    fn visit_foreign_item_fn(&mut self, node: &'ast syn::ForeignItemFn) {
        if self.refusal.is_none()
            && let Err(refusal) = self.import(node)
        {
            self.refusal = Some(refusal);
        }
    }
}

pub(super) fn collect(file: &syn::File) -> Result<Vec<RawHit>, Refusal> {
    let mut visitor = Imports {
        block: Identity::default(),
        hits: Vec::new(),
        refusal: None,
    };
    visitor.visit_file(file);
    visitor.refusal.map_or(Ok(visitor.hits), Err)
}
