// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! RDF 1.2 structure written by hand: reifier rows built for the default graph
//! and collection traversals.
//!
//! Both detectors judge a function against its job's declared entry points,
//! which the syntax pass cannot know. The pass therefore reports every
//! candidate with the callees it observed, and the gate keeps the candidates
//! that stand in the wrong relation to the entry points of each job.

use syn::ext::IdentExt as _;
use syn::visit::Visit;

use super::RawHit;
use super::names::{ScopedNames, visit_scopes};
use crate::syntax::Names;

pub(super) const REIFIER: &str = "reifier-default-graph";
pub(super) const LIST_WALK: &str = "rdf-list-walk";
pub(super) const RULES: [&str; 2] = [REIFIER, LIST_WALK];

const RDF_NAMESPACE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
/// Names under which a triple term is written as a constructor or a type.
const TRIPLE_TERMS: [&str; 6] = [
    "Triple",
    "TripleTerm",
    "triple",
    "triple_term",
    "new_triple",
    "from_triple",
];
/// Names under which the default graph is written.
const DEFAULT_GRAPH: [&str; 5] = [
    "None",
    "DefaultGraph",
    "DEFAULT_GRAPH",
    "default_graph",
    "default",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Term {
    First,
    Rest,
    Reifies,
}

impl Term {
    fn local(self) -> &'static str {
        match self {
            Self::First => "first",
            Self::Rest => "rest",
            Self::Reifies => "reifies",
        }
    }

    fn of_iri(text: &str) -> Option<Self> {
        [Self::First, Self::Rest, Self::Reifies]
            .into_iter()
            .find(|term| text.strip_prefix(RDF_NAMESPACE) == Some(term.local()))
    }

    /// A known constant, written after `use` resolution: `RDF_FIRST`, or
    /// `FIRST` under a segment named `rdf`; `REIFIES` also under `vocab`, the
    /// spelling of the report vocabulary.
    fn of_segments(path: &[String]) -> Option<Self> {
        let (last, parents) = path.split_last()?;
        let parent = parents.last().map(|part| part.to_ascii_lowercase());
        [Self::First, Self::Rest, Self::Reifies]
            .into_iter()
            .find(|term| {
                let upper = term.local().to_ascii_uppercase();
                *last == format!("RDF_{upper}")
                    || (*last == upper
                        && (parent.as_deref() == Some("rdf")
                            || (*term == Self::Reifies && parent.as_deref() == Some("vocab"))))
            })
    }
}

/// What one function body shows.
struct Frame {
    name: String,
    span: proc_macro2::Span,
    calls: Vec<String>,
    iterates: bool,
    named_graphs: bool,
    /// Depth of enclosing calls that name the graph of what they store.
    explicit: usize,
    terms: Vec<Term>,
    candidates: Vec<(proc_macro2::Span, String)>,
}

#[derive(Default)]
struct Structure {
    names: Names,
    frames: Vec<Frame>,
    hits: Vec<RawHit>,
}

impl ScopedNames for Structure {
    fn names(&mut self) -> &mut Names {
        &mut self.names
    }
}

impl Structure {
    fn function(&mut self, ident: &syn::Ident, walk: impl FnOnce(&mut Self)) {
        self.frames.push(Frame {
            name: ident.unraw().to_string(),
            span: ident.span(),
            calls: Vec::new(),
            iterates: false,
            named_graphs: false,
            explicit: 0,
            terms: Vec::new(),
            candidates: Vec::new(),
        });
        walk(self);
        let Some(frame) = self.frames.pop() else {
            return;
        };
        if frame.iterates && frame.terms.contains(&Term::First) && frame.terms.contains(&Term::Rest)
        {
            let mut hit = RawHit::new(
                LIST_WALK,
                frame.span,
                "walk RDF collections through the approved list traversal instead of rdf:first and rdf:rest by hand",
            );
            hit.calls.clone_from(&frame.calls);
            self.hits.push(hit);
        }
        if frame.named_graphs {
            for (span, callee) in frame.candidates {
                let mut hit = RawHit::new(
                    REIFIER,
                    span,
                    "a reifier row is built for the default graph in code that handles named graphs; pass the graph of the reifier",
                );
                hit.calls = vec![callee];
                self.hits.push(hit);
            }
        }
    }

    fn resolve(&self, path: &syn::Path) -> Vec<String> {
        let written = path
            .segments
            .iter()
            .map(|part| part.ident.unraw().to_string())
            .collect();
        self.names
            .resolve_written(written, path.leading_colon.is_some())
    }

    fn reifies(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Path(path) => {
                Term::of_segments(&self.resolve(&path.path)) == Some(Term::Reifies)
            }
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(text),
                ..
            }) => Term::of_iri(&text.value()) == Some(Term::Reifies),
            other => unwrap(other).is_some_and(|inner| self.reifies(inner)),
        }
    }

    /// Record a call; `selects` is true when the call itself names the graph
    /// of what it stores. Returns whether its arguments are stored under an
    /// explicit graph, so that a row built inside them is not a default row.
    fn call(
        &mut self,
        (callee, span): (&[String], proc_macro2::Span),
        arguments: &[&syn::Expr],
        selects: bool,
    ) -> bool {
        let text = callee.join("::");
        let reifier = arguments
            .iter()
            .any(|argument| self.reifies(argument) || is_triple_term(argument));
        let Some(frame) = self.frames.last_mut() else {
            return false;
        };
        let local = callee.last().is_some_and(|last| *last == frame.name)
            && (callee.len() == 1
                || matches!(callee[0].as_str(), "Self" | "self" | "crate" | "super"));
        frame.iterates |= local;
        frame.calls.push(text.clone());
        let stores = selects || arguments.iter().any(|argument| is_some_graph(argument));
        let default_graph = match arguments {
            [_, _, _] => !stores && frame.explicit == 0,
            [_, _, _, graph] => is_default_graph(graph),
            _ => false,
        };
        if reifier && default_graph {
            frame.candidates.push((span, text));
        }
        stores
    }

    fn within(&mut self, stores: bool, walk: impl FnOnce(&mut Self)) {
        if stores && let Some(frame) = self.frames.last_mut() {
            frame.explicit += 1;
        }
        walk(self);
        if stores && let Some(frame) = self.frames.last_mut() {
            frame.explicit -= 1;
        }
    }
}

impl<'ast> Visit<'ast> for Structure {
    visit_scopes!(without_functions);

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.generics(&node.sig.generics, |this| {
            this.function(&node.sig.ident, |this| {
                syn::visit::visit_item_fn(this, node);
            });
        });
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.generics(&node.sig.generics, |this| {
            this.function(&node.sig.ident, |this| {
                syn::visit::visit_impl_item_fn(this, node);
            });
        });
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.generics(&node.sig.generics, |this| {
            this.function(&node.sig.ident, |this| {
                syn::visit::visit_trait_item_fn(this, node);
            });
        });
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        let mut stores = false;
        if let syn::Expr::Path(callee) = &*node.func
            && let Some(first) = callee.path.segments.first()
        {
            let resolved = self.resolve(&callee.path);
            stores = self.call(
                (&resolved, first.ident.span()),
                &node.args.iter().collect::<Vec<_>>(),
                false,
            );
        }
        self.within(stores, |this| syn::visit::visit_expr_call(this, node));
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let name = node.method.unraw().to_string();
        let own = matches!(&*node.receiver, syn::Expr::Path(path) if path.path.is_ident("self"));
        let call = if own {
            vec!["self".to_owned(), name]
        } else {
            vec![name]
        };
        let stores = self.call(
            (&call, node.method.span()),
            &node.args.iter().collect::<Vec<_>>(),
            selects_graph(&node.receiver),
        );
        self.within(stores, |this| {
            syn::visit::visit_expr_method_call(this, node);
        });
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.looped();
        syn::visit::visit_expr_for_loop(self, node);
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.looped();
        syn::visit::visit_expr_while(self, node);
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        self.looped();
        syn::visit::visit_expr_loop(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        self.term(Term::of_segments(&self.resolve(node)));
        syn::visit::visit_path(self, node);
    }

    fn visit_lit_str(&mut self, node: &'ast syn::LitStr) {
        self.term(Term::of_iri(&node.value()));
    }

    fn visit_ident(&mut self, node: &'ast proc_macro2::Ident) {
        if let Some(frame) = self.frames.last_mut()
            && names_graph(&words(&node.unraw().to_string()))
        {
            frame.named_graphs = true;
        }
    }

    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if let Some(frame) = self.frames.last_mut()
            && node
                .path
                .segments
                .iter()
                .any(|part| names_dataset(&words(&part.ident.unraw().to_string())))
        {
            frame.named_graphs = true;
        }
        syn::visit::visit_type_path(self, node);
    }
}

impl Structure {
    fn looped(&mut self) {
        if let Some(frame) = self.frames.last_mut() {
            frame.iterates = true;
        }
    }

    fn term(&mut self, term: Option<Term>) {
        if let (Some(frame), Some(term)) = (self.frames.last_mut(), term) {
            frame.terms.push(term);
        }
    }
}

/// One wrapper layer an argument can sit in without changing which term it is.
fn unwrap(expression: &syn::Expr) -> Option<&syn::Expr> {
    match expression {
        syn::Expr::Paren(inner) => Some(&inner.expr),
        syn::Expr::Group(inner) => Some(&inner.expr),
        syn::Expr::Reference(inner) => Some(&inner.expr),
        syn::Expr::Cast(inner) => Some(&inner.expr),
        syn::Expr::MethodCall(call) => Some(&call.receiver),
        syn::Expr::Call(call) if call.args.len() == 1 => call.args.first(),
        _ => None,
    }
}

fn last_name(path: &syn::Path) -> Option<String> {
    path.segments
        .last()
        .map(|part| part.ident.unraw().to_string())
}

/// The words of an identifier, lowercased: `GraphName` and `graph_name` are
/// `graph` and `name`, and `paragraph` is one word.
fn words(identifier: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut previous_lower = false;
    for character in identifier.chars() {
        if character == '_' {
            previous_lower = false;
            words.push(String::new());
            continue;
        }
        if character.is_uppercase() && previous_lower || words.is_empty() {
            words.push(String::new());
        }
        if let Some(word) = words.last_mut() {
            word.extend(character.to_lowercase());
        }
        previous_lower = character.is_lowercase() || character.is_ascii_digit();
    }
    words.retain(|word| !word.is_empty());
    words
}

/// Whether a type names a dataset of quads.
fn names_dataset(words: &[String]) -> bool {
    words
        .iter()
        .any(|word| matches!(word.as_str(), "dataset" | "datasets" | "quad" | "quads"))
}

/// Whether an identifier names a named graph: `named_graph` or `graph_name`.
/// A plain `graph`, or a word that merely contains it, does not.
fn names_graph(words: &[String]) -> bool {
    words.windows(2).any(|pair| {
        matches!(
            (pair[0].as_str(), pair[1].as_str()),
            ("named", "graph" | "graphs") | ("graph", "name" | "names")
        )
    })
}

/// `Some(graph)`: an optional graph argument that is present.
fn is_some_graph(expression: &syn::Expr) -> bool {
    matches!(expression, syn::Expr::Call(call)
        if call.args.len() == 1
            && matches!(&*call.func, syn::Expr::Path(path) if path.path.is_ident("Some")))
}

/// A receiver such as `dataset.graph_mut(name)`: a call named for a graph that
/// is given a graph, so what is stored through it has an explicit graph.
fn selects_graph(receiver: &syn::Expr) -> bool {
    match receiver {
        syn::Expr::MethodCall(call) => {
            words(&call.method.unraw().to_string())
                .iter()
                .any(|word| word == "graph")
                && call.args.iter().any(|argument| !is_default_graph(argument))
        }
        syn::Expr::Paren(inner) => selects_graph(&inner.expr),
        _ => false,
    }
}

/// An object written as a triple term.
fn is_triple_term(expression: &syn::Expr) -> bool {
    let named = |path: &syn::Path| {
        last_name(path).is_some_and(|name| TRIPLE_TERMS.contains(&name.as_str()))
    };
    match expression {
        syn::Expr::Call(call) if matches!(&*call.func, syn::Expr::Path(path) if named(&path.path)) => {
            true
        }
        syn::Expr::Struct(built) if named(&built.path) => true,
        other => unwrap(other).is_some_and(is_triple_term),
    }
}

/// A graph argument that names the default graph.
fn is_default_graph(expression: &syn::Expr) -> bool {
    let default = |path: &syn::Path| {
        last_name(path).is_some_and(|name| DEFAULT_GRAPH.contains(&name.as_str()))
    };
    match expression {
        syn::Expr::Path(path) => default(&path.path),
        syn::Expr::Call(call) if call.args.is_empty() => {
            matches!(&*call.func, syn::Expr::Path(path) if default(&path.path))
        }
        syn::Expr::Paren(_) | syn::Expr::Group(_) | syn::Expr::Reference(_) => {
            unwrap(expression).is_some_and(is_default_graph)
        }
        _ => false,
    }
}

pub(super) fn collect(file: &syn::File) -> Vec<RawHit> {
    let mut structure = Structure::default();
    structure.visit_file(file);
    structure.hits
}
