// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Written JSON dependencies, RDF text emission and incomplete local term enums.

use syn::ext::IdentExt as _;
use syn::visit::Visit;

use super::RawHit;
use super::names::{ScopedNames, visit_scopes};
use crate::syntax::{Names, visit_import_paths};

pub(super) const RULES: [&str; 3] = ["serde-json", "rdf-text-writer", "rdf-term-model"];

#[derive(Default)]
struct Policy {
    names: Names,
    hits: Vec<RawHit>,
}

impl Policy {
    fn json_path(&mut self, path: &syn::Path) {
        let Some(first) = path.segments.first() else {
            return;
        };
        let written: Vec<_> = path
            .segments
            .iter()
            .map(|part| part.ident.unraw().to_string())
            .collect();
        let written = self
            .names
            .resolve_written(written, path.leading_colon.is_some());
        if written.first().is_some_and(|name| name == "serde_json") {
            RawHit::push(
                &mut self.hits,
                "serde-json",
                first.ident.span(),
                "use native PurRDF JSON parsing/projection instead of serde_json",
            );
        }
    }

    fn macro_paths(&mut self, body: proc_macro2::TokenStream) {
        let mut pending = vec![body];
        while let Some(body) = pending.pop() {
            let mut tokens = body.into_iter();
            let mut variable = false;
            let mut member = false;
            let mut colons = 0;
            while let Some(token) = tokens.next() {
                if path_separator(&token, tokens.clone().next()) {
                    tokens.next();
                    variable = false;
                    colons = if member { 0 } else { 2 };
                    continue;
                }
                let next_variable = matches!(&token,
                    proc_macro2::TokenTree::Punct(mark) if mark.as_char() == '$');
                let next_member = if let proc_macro2::TokenTree::Ident(ident) = &token {
                    let mut following = tokens.clone();
                    following
                        .next()
                        .is_some_and(|first| path_separator(&first, following.next()))
                        && syn::parse2::<syn::Path>(
                            proc_macro2::TokenTree::Ident(ident.clone()).into(),
                        )
                        .is_ok()
                } else {
                    false
                };
                match token {
                    proc_macro2::TokenTree::Group(group) => pending.push(group.stream()),
                    proc_macro2::TokenTree::Ident(ident) if !variable => {
                        if let Some(node) = parsed_format(&ident, tokens.clone()) {
                            self.macro_format(&node, &ident.unraw().to_string());
                        }
                        if ident == "enum"
                            && !member
                            && let Some((item, consumed)) = parsed_enum(&ident, tokens.clone())
                        {
                            for _ in 0..consumed {
                                tokens.next();
                            }
                            self.visit_item_enum(&item);
                            variable = false;
                            member = false;
                            colons = 0;
                            continue;
                        }
                        if next_member && !member {
                            let written = self
                                .names
                                .resolve_written(vec![ident.unraw().to_string()], colons >= 2);
                            if written.first().is_some_and(|name| name == "serde_json") {
                                RawHit::push(
                                    &mut self.hits,
                                    "serde-json",
                                    ident.span(),
                                    "use native PurRDF JSON parsing/projection instead of serde_json",
                                );
                            }
                        }
                    }
                    _ => {}
                }
                variable = next_variable;
                member = next_member;
                colons = 0;
            }
        }
    }

    fn macro_format(&mut self, node: &syn::Macro, name: &str) {
        let Some(index) = format_index(name) else {
            return;
        };
        let mut tokens = node.tokens.clone().into_iter();
        if index == 1 {
            for token in tokens.by_ref() {
                if matches!(token, proc_macro2::TokenTree::Punct(mark) if mark.as_char() == ',') {
                    break;
                }
            }
        }
        if let Some(proc_macro2::TokenTree::Literal(literal)) = tokens.next()
            && let Ok(text) = syn::parse2::<syn::LitStr>(proc_macro2::TokenStream::from(
                proc_macro2::TokenTree::Literal(literal),
            ))
            && rdf_format(&text.value())
        {
            RawHit::push(
                &mut self.hits,
                "rdf-text-writer",
                text.span(),
                "emit RDF through native PurRDF writers instead of formatting RDF syntax",
            );
        }
    }
}

impl ScopedNames for Policy {
    fn names(&mut self) -> &mut Names {
        &mut self.names
    }
}

impl<'ast> Visit<'ast> for Policy {
    visit_scopes!();

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        visit_import_paths(&node.tree, |path, local| {
            let written = self.names.resolve_import(
                path.iter().map(|part| part.unraw().to_string()).collect(),
                node.leading_colon.is_some(),
                local,
                path.first().map(|part| part.span().byte_range().start),
            );
            if let Some(first) = path.first()
                && written.first().is_some_and(|name| name == "serde_json")
            {
                RawHit::push(
                    &mut self.hits,
                    "serde-json",
                    first.span(),
                    "use native PurRDF JSON parsing/projection instead of serde_json",
                );
            }
        });
    }

    fn visit_item_extern_crate(&mut self, node: &'ast syn::ItemExternCrate) {
        if node.ident.unraw() == "serde_json" {
            RawHit::push(
                &mut self.hits,
                "serde-json",
                node.ident.span(),
                "use native PurRDF JSON parsing/projection instead of serde_json",
            );
        }
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        self.json_path(node);
        syn::visit::visit_path(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        self.json_path(&node.path);
        let name = node
            .path
            .segments
            .last()
            .map(|part| part.ident.unraw().to_string());
        if let Ok(arguments) = node.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            if let Some(index) = name.as_deref().and_then(format_index)
                && let Some(syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(text),
                    ..
                })) = arguments.iter().nth(index)
                && rdf_format(&text.value())
            {
                RawHit::push(
                    &mut self.hits,
                    "rdf-text-writer",
                    text.span(),
                    "emit RDF through native PurRDF writers instead of formatting RDF syntax",
                );
            }
            for argument in &arguments {
                self.visit_expr(argument);
            }
        } else {
            if let Some(name) = name.as_deref() {
                self.macro_format(node, name);
            }
            self.macro_paths(node.tokens.clone());
        }
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        let variants: Vec<_> = node
            .variants
            .iter()
            .map(|variant| variant.ident.unraw().to_string())
            .collect();
        let has = |names: &[&str]| {
            variants
                .iter()
                .any(|variant| names.contains(&variant.as_str()))
        };
        if has(&["Iri", "IRI", "NamedNode", "Uri", "URI"])
            && has(&["Literal", "TypedLiteral", "SimpleLiteral"])
            && has(&["Blank", "BlankNode", "BNode"])
            && !has(&["Triple", "TripleTerm", "Statement"])
        {
            RawHit::push(
                &mut self.hits,
                "rdf-term-model",
                node.ident.span(),
                "use native PurRDF RDF 1.2 terms; this local RDF enum omits triple terms",
            );
        }
        self.generics(&node.generics, |this| {
            syn::visit::visit_item_enum(this, node);
        });
    }
}

fn path_separator(first: &proc_macro2::TokenTree, second: Option<proc_macro2::TokenTree>) -> bool {
    matches!(first, proc_macro2::TokenTree::Punct(mark)
        if mark.as_char() == ':' && mark.spacing() == proc_macro2::Spacing::Joint)
        && matches!(second, Some(proc_macro2::TokenTree::Punct(mark)) if mark.as_char() == ':')
}

fn format_index(name: &str) -> Option<usize> {
    match name {
        "write" | "writeln" => Some(1),
        "format" | "format_args" | "print" | "println" | "eprint" | "eprintln" => Some(0),
        _ => None,
    }
}

fn parsed_format(
    ident: &syn::Ident,
    mut following: impl Iterator<Item = proc_macro2::TokenTree>,
) -> Option<syn::Macro> {
    let proc_macro2::TokenTree::Punct(bang) = following.next()? else {
        return None;
    };
    if bang.as_char() != '!' {
        return None;
    }
    let proc_macro2::TokenTree::Group(arguments) = following.next()? else {
        return None;
    };
    format_index(&ident.unraw().to_string())?;
    syn::parse2(
        [
            proc_macro2::TokenTree::Ident(ident.clone()),
            proc_macro2::TokenTree::Punct(bang),
            proc_macro2::TokenTree::Group(arguments),
        ]
        .into_iter()
        .collect(),
    )
    .ok()
}

fn parsed_enum(
    ident: &syn::Ident,
    following: impl Iterator<Item = proc_macro2::TokenTree>,
) -> Option<(syn::ItemEnum, usize)> {
    let mut candidate = vec![proc_macro2::TokenTree::Ident(ident.clone())];
    let mut angles = 0usize;
    let mut arrow = false;
    for (index, token) in following.enumerate() {
        if matches!(&token, proc_macro2::TokenTree::Ident(word) if word == "enum")
            || matches!(&token, proc_macro2::TokenTree::Punct(mark) if mark.as_char() == ';' && angles == 0)
        {
            return None;
        }
        let body = matches!(&token, proc_macro2::TokenTree::Group(group)
            if group.delimiter() == proc_macro2::Delimiter::Brace && angles == 0);
        if let proc_macro2::TokenTree::Punct(mark) = &token {
            match mark.as_char() {
                '<' => angles += 1,
                '>' if !arrow => angles = angles.saturating_sub(1),
                _ => {}
            }
        }
        arrow = matches!(&token, proc_macro2::TokenTree::Punct(mark) if mark.as_char() == '-');
        candidate.push(token);
        if body {
            return syn::parse2(candidate.into_iter().collect())
                .ok()
                .map(|item| (item, index + 1));
        }
    }
    None
}

fn rdf_format(text: &str) -> bool {
    let text = text.trim();
    if !text.ends_with('.') || !text.starts_with('<') {
        return false;
    }
    // Fixed subject and predicate IRIs are still handwritten RDF emission.
    // Requiring separated term boundaries avoids treating ordinary markup as
    // an RDF row merely because it contains several angle-bracket pairs.
    let fixed = text.contains("> <") && (text.matches("> <").count() >= 2 || text.contains("> \""));
    let iri = text.contains("<{") && text.contains("}>");
    let triple = text.matches("<{").count() >= 2;
    let literal = text.contains("\"{") || text.contains("^^") || text.contains("@{");
    fixed || (iri && (triple || literal))
}

pub(super) fn collect(file: &syn::File) -> Vec<RawHit> {
    let mut policy = Policy::default();
    policy.visit_file(file);
    policy.hits
}
