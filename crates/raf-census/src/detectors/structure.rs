// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! RDF 1.2 structure written by hand: reifier rows built for the default graph
//! and collection traversals.
//!
//! Both detectors judge a function against its job's declared entry points,
//! which the syntax pass cannot know. The pass therefore reports every
//! candidate with the callees it observed, and the gate keeps the candidates
//! that stand in the wrong relation to the entry points of each job.

use purrdf_iri::vocab::rdf;
use syn::ext::IdentExt as _;
use syn::spanned::Spanned as _;
use syn::visit::Visit;

use super::names::{ScopedNames, call_path, visit_scopes};
use super::{Call, CallEvidence, CallPath, RawHit, ReceiverMode};
use crate::syntax::{
    Names, definitely_excluded, expression_attributes,
    inert_attributes as inert_receiver_attributes, visit_import_paths,
};

pub(super) const REIFIER: &str = "reifier-default-graph";
pub(super) const LIST_WALK: &str = "rdf-list-walk";
pub(super) const RULES: [&str; 2] = [REIFIER, LIST_WALK];

/// Names under which a triple term is written as a constructor or a type.
const TRIPLE_TERMS: [&str; 7] = [
    "Triple",
    "TripleTerm",
    "triple",
    "triple_term",
    "new_triple",
    "from_triple",
    "intern_triple",
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
        match text {
            rdf::FIRST => Some(Self::First),
            rdf::REST => Some(Self::Rest),
            rdf::REIFIES => Some(Self::Reifies),
            _ => None,
        }
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
    span: proc_macro2::Span,
    receiver: Option<ReceiverMode>,
    list: Option<ListBody>,
    named_graphs: bool,
    block_glob: bool,
    bindings: Vec<Binding>,
    candidates: Vec<(proc_macro2::Span, Call)>,
}

#[derive(Default)]
struct ListBody {
    calls: Vec<Call>,
    terms: Vec<Term>,
    looped: bool,
}

struct Binding {
    name: syn::Ident,
    graph: GraphBinding,
    nominal: Option<Call>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GraphBinding {
    Handle,
    Value,
    Unknown,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum WriteProbe {
    #[default]
    Inactive,
    Header,
    Nested,
}

/// The one row operand whose enclosing call supplies its graph.
#[derive(Clone, Copy)]
enum ExplicitRow<'ast> {
    Call(&'ast syn::ExprCall),
    Method(&'ast syn::ExprMethodCall),
}

impl<'ast> ExplicitRow<'ast> {
    fn of(expression: &'ast syn::Expr) -> Option<Self> {
        match graph_operand(expression) {
            syn::Expr::Call(call) if call.args.len() == 3 => Some(Self::Call(call)),
            syn::Expr::MethodCall(call) if call.args.len() == 3 => Some(Self::Method(call)),
            _ => None,
        }
    }
}

#[derive(Default)]
struct Structure<'ast> {
    names: Names,
    frames: Vec<Frame>,
    hits: Vec<RawHit>,
    explicit_row: Option<ExplicitRow<'ast>>,
    impl_graph: bool,
    uncertain: bool,
    write_probe: WriteProbe,
    tail_local: Option<&'ast syn::Local>,
}

impl ScopedNames for Structure<'_> {
    fn names(&mut self) -> &mut Names {
        &mut self.names
    }
}

impl<'ast> Structure<'ast> {
    fn recording(&self) -> bool {
        !self.uncertain && !self.probing_writes()
    }

    fn probing_writes(&self) -> bool {
        self.write_probe != WriteProbe::Inactive
    }

    fn function(&mut self, signature: &syn::Signature, walk: impl FnOnce(&mut Self)) {
        if !self.recording() {
            return;
        }
        let ident = &signature.ident;
        self.frames.push(Frame {
            span: ident.span(),
            receiver: receiver_mode(signature),
            list: Some(ListBody::default()),
            named_graphs: false,
            block_glob: false,
            bindings: Vec::new(),
            candidates: Vec::new(),
        });
        for input in &signature.inputs {
            if let syn::FnArg::Typed(input) = input {
                self.bind(
                    &input.pat,
                    Some(&input.ty),
                    GraphBinding::Unknown,
                    &input.attrs,
                );
            }
        }
        walk(self);
        let frame = self
            .frames
            .pop()
            .expect("INVARIANT: entered a detector frame");
        if let Some(list) = frame.list
            && list.terms.contains(&Term::First)
            && list.terms.contains(&Term::Rest)
        {
            let mut hit = RawHit::new(
                LIST_WALK,
                frame.span,
                "walk RDF collections through the approved list traversal instead of rdf:first and rdf:rest by hand",
            );
            hit.call_evidence = Some(CallEvidence::List {
                calls: list.calls,
                looped: list.looped,
            });
            self.hits.push(hit);
        }
        if frame.named_graphs {
            for (span, call) in frame.candidates {
                let mut hit = RawHit::new(
                    REIFIER,
                    span,
                    "a reifier row is built for the default graph in code that handles named graphs; pass the graph of the reifier",
                );
                hit.call_evidence = Some(CallEvidence::Reifier(call));
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
            .resolve_inert_written(written, path.leading_colon.is_some())
    }

    fn reifies(&self, expression: &syn::Expr) -> bool {
        match graph_operand(expression) {
            syn::Expr::Path(path) => {
                Term::of_segments(&self.resolve(&path.path)) == Some(Term::Reifies)
            }
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(text),
                ..
            }) => Term::of_iri(&text.value()) == Some(Term::Reifies),
            syn::Expr::Call(call) if matches!(call.func.as_ref(), syn::Expr::Path(path) if iri_constructor(&path.path)) => {
                call.args.len() == 1 && call.args.first().is_some_and(|value| self.reifies(value))
            }
            syn::Expr::MethodCall(call) if call.method == "intern_iri" => {
                call.args.len() == 1 && call.args.first().is_some_and(|value| self.reifies(value))
            }
            _ => false,
        }
    }

    /// Record a call, with the explicit graph of a triple supplied by its
    /// receiver or by the call that directly stores that triple.
    fn call(
        &mut self,
        observed: Option<Call>,
        span: proc_macro2::Span,
        arguments: &[&syn::Expr],
        explicit: bool,
    ) {
        if !self.recording() {
            return;
        }
        let Some(observed) = observed else {
            return;
        };
        let reifier = matches!(arguments, [_, predicate, object, ..]
            if self.reifies(predicate) || is_triple_term(object));
        let Some(frame) = self.frames.last_mut() else {
            return;
        };
        let default_graph = match arguments {
            [_, _, _] => !explicit,
            [_, _, _, graph] => is_default_graph(graph),
            _ => false,
        };
        if reifier && default_graph {
            frame.candidates.push((span, observed.clone()));
        }
        if let Some(list) = &mut frame.list {
            list.calls.push(observed);
        }
    }

    fn observed_path(&self, path: &syn::Path, member: Option<&syn::Ident>) -> Option<Call> {
        let mut spelling: Vec<_> = path
            .segments
            .iter()
            .map(|part| part.ident.unraw().to_string())
            .collect();
        let first = path.segments.first()?;
        let frame = self.frames.last()?;
        if member.is_none()
            && let Some(local) = path.get_ident()
            && self.binding(local).is_some()
        {
            return None;
        }
        if let Some(member) = member {
            spelling.push(member.unraw().to_string());
        }
        call_path(
            &self.names,
            &spelling,
            path.leading_colon.is_some(),
            first.ident.span().byte_range().start,
            frame.block_glob,
        )
    }

    fn method(&self, node: &syn::ExprMethodCall) -> Option<Call> {
        let local = receiver_ident(&node.receiver)?;
        let method = node.method.unraw().to_string();
        if local == "self" {
            return Some(Call {
                byte: node.method.span().byte_range().start,
                path: CallPath::SelfType(method),
                receiver: Some(self.frames.last()?.receiver?),
            });
        }
        let mut nominal = self.binding(local)?.nominal.clone()?;
        match &mut nominal.path {
            CallPath::Written(path) | CallPath::Import(path) => {
                path.push_str("::");
                path.push_str(&method);
            }
            CallPath::SelfType(member) => *member = method,
        }
        Some(nominal)
    }

    /// Whether a receiver selects a graph, inline or through a local handle.
    fn selects(&self, receiver: &syn::Expr) -> bool {
        if !written_expression(receiver) {
            return false;
        }
        match graph_operand(receiver) {
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|name| self.binding(name))
                .is_some_and(|binding| binding.graph == GraphBinding::Handle),
            syn::Expr::MethodCall(call) => {
                words(&call.method.unraw().to_string())
                    .iter()
                    .any(|word| word == "graph")
                    && call.args.len() == 1
                    && call
                        .args
                        .first()
                        .is_some_and(|argument| !is_default_graph(graph_operand(argument)))
            }
            _ => false,
        }
    }

    fn with_row(&mut self, row: Option<ExplicitRow<'ast>>, walk: impl FnOnce(&mut Self)) {
        let row = row.or(self.explicit_row);
        let outer = std::mem::replace(&mut self.explicit_row, row);
        walk(self);
        self.explicit_row = outer;
    }

    fn written_scope(&mut self, certain: bool, walk: impl FnOnce(&mut Self)) {
        let outer = self.uncertain;
        self.uncertain |= !certain;
        walk(self);
        self.uncertain = outer;
    }

    fn attributed_scope(&mut self, attributes: &[syn::Attribute], walk: impl FnOnce(&mut Self)) {
        if !definitely_excluded(attributes) {
            self.written_scope(inert_receiver_attributes(attributes), walk);
        }
    }

    fn expression_scope(&mut self, expression: &syn::Expr, walk: impl FnOnce(&mut Self)) {
        if let Some(attributes) = expression_attributes(expression) {
            self.attributed_scope(attributes, walk);
        } else {
            self.written_scope(false, walk);
        }
    }

    fn deferred(&mut self, walk: impl FnOnce(&mut Self)) {
        let outer = self.frames.last_mut().and_then(|frame| frame.list.take());
        walk(self);
        if let Some(frame) = self.frames.last_mut() {
            frame.list = outer;
        }
    }

    /// Assignments can remove an incoming graph proof before another iteration.
    /// This pass shares lexical binding scopes with the ordinary visit and
    /// records no detector evidence or nested prepasses.
    fn repeated(
        &mut self,
        body: &'ast syn::Block,
        condition: Option<&'ast syn::Expr>,
        walk: impl Fn(&mut Self),
    ) {
        if self.probing_writes() {
            self.nested_repeat(walk);
            return;
        }
        if self.recording()
            && self.frames.last().is_some_and(|frame| {
                frame
                    .bindings
                    .iter()
                    .any(|binding| binding.graph == GraphBinding::Handle)
            })
            && !single_pass(body, condition)
        {
            self.write_probe = WriteProbe::Header;
            self.graph_scope(&walk);
            self.write_probe = WriteProbe::Inactive;
        }
        walk(self);
    }

    fn nested_repeat(&mut self, walk: impl FnOnce(&mut Self)) {
        let outer = self.write_probe;
        if self.probing_writes() {
            self.write_probe = WriteProbe::Nested;
        }
        walk(self);
        self.write_probe = outer;
    }

    fn graph_scope(&mut self, walk: impl FnOnce(&mut Self)) {
        let Some((len, block_glob)) = self
            .frames
            .last()
            .map(|frame| (frame.bindings.len(), frame.block_glob))
        else {
            walk(self);
            return;
        };
        walk(self);
        let frame = self
            .frames
            .last_mut()
            .expect("INVARIANT: scope kept its function");
        frame.bindings.truncate(len);
        frame.block_glob = block_glob;
    }

    fn binding(&self, name: &syn::Ident) -> Option<&Binding> {
        let name = name.unraw();
        self.frames
            .last()?
            .bindings
            .iter()
            .rev()
            .find(|binding| binding.name == name)
    }

    fn bind(
        &mut self,
        pattern: &syn::Pat,
        ty: Option<&syn::Type>,
        kind: GraphBinding,
        attributes: &[syn::Attribute],
    ) {
        let mut known = !self.uncertain && inert_receiver_attributes(attributes);
        let (pattern, ty) = match pattern {
            syn::Pat::Type(typed) => {
                known &= inert_receiver_attributes(&typed.attrs);
                (typed.pat.as_ref(), Some(typed.ty.as_ref()))
            }
            other => (other, ty),
        };
        if let syn::Pat::Ident(binding) = pattern {
            known &= inert_receiver_attributes(&binding.attrs);
        }
        let nominal = match pattern {
            syn::Pat::Ident(binding)
                if known
                    && !self.probing_writes()
                    && binding.by_ref.is_none()
                    && binding.subpat.is_none() =>
            {
                ty.and_then(receiver_annotation).and_then(|(path, mode)| {
                    let parts: Vec<_> = path
                        .segments
                        .iter()
                        .map(|part| part.ident.unraw().to_string())
                        .collect();
                    let first = path.segments.first()?;
                    let block_glob = self.frames.last()?.block_glob;
                    let mut nominal = call_path(
                        &self.names,
                        &parts,
                        path.leading_colon.is_some(),
                        first.ident.span().byte_range().start,
                        block_glob,
                    )?;
                    nominal.receiver = Some(mode);
                    Some(nominal)
                })
            }
            _ => None,
        };
        let kind = if !known {
            GraphBinding::Unknown
        } else if kind == GraphBinding::Handle {
            kind
        } else {
            ty.map_or(GraphBinding::Unknown, graph_type)
        };
        let Some(frame) = self.frames.last_mut() else {
            return;
        };
        let selected = match pattern {
            syn::Pat::Ident(binding) => Some(binding.ident.unraw()),
            _ => None,
        };
        raf_source::visit_pattern_bindings(pattern, |binding| {
            let name = binding.unraw();
            let (graph, nominal) = if selected.as_ref() == Some(&name) {
                (kind, nominal.clone())
            } else {
                (GraphBinding::Unknown, None)
            };
            frame.bindings.push(Binding {
                name,
                graph,
                nominal,
            });
        });
    }

    fn block_bindings(&mut self, block: &syn::Block) {
        let Some(frame) = self.frames.last_mut() else {
            return;
        };
        for statement in &block.stmts {
            let syn::Stmt::Item(item) = statement else {
                continue;
            };
            let name = match item {
                syn::Item::Fn(item) if !definitely_excluded(&item.attrs) => &item.sig.ident,
                syn::Item::Const(item) if !definitely_excluded(&item.attrs) => &item.ident,
                syn::Item::Static(item) if !definitely_excluded(&item.attrs) => &item.ident,
                syn::Item::Use(item) if !definitely_excluded(&item.attrs) => {
                    visit_import_paths(&item.tree, |_, alias| frame.block_glob |= alias.is_none());
                    continue;
                }
                _ => continue,
            };
            frame.bindings.push(Binding {
                name: name.unraw(),
                graph: GraphBinding::Unknown,
                nominal: None,
            });
        }
    }

    fn initializer(&self, expression: &syn::Expr) -> GraphBinding {
        if self.selects(expression) {
            GraphBinding::Handle
        } else {
            GraphBinding::Unknown
        }
    }

    /// A storage call either takes a row alongside its graph term or takes
    /// the row on a receiver that already selects its graph. Only that direct
    /// operand gets the graph, since nested calls may build other rows.
    fn stored_row(
        &self,
        callee: &str,
        arguments: &[&'ast syn::Expr],
        selects: bool,
    ) -> Option<ExplicitRow<'ast>> {
        if !words(callee)
            .iter()
            .any(|word| matches!(word.as_str(), "add" | "insert" | "push" | "store" | "write"))
        {
            return None;
        }
        match arguments {
            [row] if selects => ExplicitRow::of(row),
            [first, second] => ExplicitRow::of(first)
                .filter(|_| self.supplies_graph(second))
                .or_else(|| ExplicitRow::of(second).filter(|_| self.supplies_graph(first))),
            _ => None,
        }
    }

    fn supplies_graph(&self, expression: &syn::Expr) -> bool {
        if !written_expression(expression) {
            return false;
        }
        let expression = graph_operand(expression);
        (matches!(expression, syn::Expr::Path(path) if path.path.get_ident().and_then(|name|
            self.binding(name)).is_some_and(|binding| binding.graph == GraphBinding::Value))
            || names_graph_value(expression))
            && !is_default_graph(expression)
    }

    /// Visit a conditional expression in order, with bindings from each
    /// successful let operand visible to the following operand and body.
    fn condition(&mut self, expression: &'ast syn::Expr) {
        self.expression_scope(expression, |this| match expression {
            syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
                this.condition(&binary.left);
                this.condition(&binary.right);
            }
            syn::Expr::Let(binding) => {
                let kind = this.initializer(&binding.expr);
                this.visit_expr_let(binding);
                this.bind(&binding.pat, None, kind, &binding.attrs);
            }
            _ => this.visit_expr(expression),
        });
    }

    fn conditional_body(&mut self, condition: &'ast syn::Expr, body: &'ast syn::Block) {
        self.graph_scope(|this| {
            this.condition(condition);
            this.visit_block(body);
        });
    }
}

impl<'ast> Visit<'ast> for Structure<'ast> {
    visit_scopes!(without_functions_blocks_or_impls);

    fn visit_file(&mut self, node: &'ast syn::File) {
        self.written_scope(inert_receiver_attributes(&node.attrs), |this| {
            this.file(node, |this| syn::visit::visit_file(this, node));
        });
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        self.written_scope(inert_receiver_attributes(&node.attrs), |this| {
            this.module(node, |this| syn::visit::visit_item_mod(this, node));
        });
    }

    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        self.expression_scope(node, |this| {
            syn::visit::visit_expr(this, node);
        });
    }

    fn visit_block(&mut self, node: &'ast syn::Block) {
        // A break here targets the probed loop only outside nested loops.
        // The ordinary visit still carries these writes out of the loop.
        if self.write_probe == WriteProbe::Header
            && !self.uncertain
            && terminal_break(node).is_some_and(|exit| exit.label.is_none())
            && !can_continue(node, None)
        {
            return;
        }
        let tail = match node.stmts.last() {
            Some(syn::Stmt::Local(local)) => Some(local),
            _ => None,
        };
        let outer = std::mem::replace(&mut self.tail_local, tail);
        self.graph_scope(|this| {
            this.block_bindings(node);
            this.block(node, |this| syn::visit::visit_block(this, node));
        });
        self.tail_local = outer;
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let named = nominal_path(&node.self_ty).is_some_and(|path| {
            path.segments.iter().any(|part| {
                let words = words(&part.ident.unraw().to_string());
                names_dataset(&words) || names_graph(&words)
            })
        });
        let outer = std::mem::replace(&mut self.impl_graph, named);
        self.written_scope(inert_receiver_attributes(&node.attrs), |this| {
            this.generics(&node.generics, |this| {
                syn::visit::visit_item_impl(this, node);
            });
        });
        self.impl_graph = outer;
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.written_scope(inert_receiver_attributes(&node.attrs), |this| {
            this.generics(&node.sig.generics, |this| {
                this.function(&node.sig, |this| {
                    syn::visit::visit_item_fn(this, node);
                });
            });
        });
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.written_scope(inert_receiver_attributes(&node.attrs), |this| {
            this.generics(&node.sig.generics, |this| {
                this.function(&node.sig, |this| {
                    this.frames
                        .last_mut()
                        .expect("INVARIANT: entered the method")
                        .named_graphs = this.impl_graph;
                    syn::visit::visit_impl_item_fn(this, node);
                });
            });
        });
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.written_scope(inert_receiver_attributes(&node.attrs), |this| {
            this.generics(&node.sig.generics, |this| {
                this.function(&node.sig, |this| {
                    syn::visit::visit_trait_item_fn(this, node);
                });
            });
        });
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if self.probing_writes() {
            syn::visit::visit_expr_call(self, node);
            return;
        }
        let mut row = None;
        if let syn::Expr::Path(callee) = graph_operand(&node.func)
            && let Some(first) = callee.path.segments.first()
        {
            let resolved = self.resolve(&callee.path);
            let arguments: Vec<_> = node.args.iter().collect();
            let explicit = matches!(self.explicit_row, Some(ExplicitRow::Call(call))
                if std::ptr::eq(call, node));
            let observed = match &callee.qself {
                None => self.observed_path(&callee.path, None),
                Some(qualified)
                    if qualified.position == 0
                        && callee.path.segments.len() == 1
                        && let syn::Type::Path(path) = bare_type(&qualified.ty)
                        && path.qself.is_none() =>
                {
                    self.observed_path(&path.path, Some(&first.ident))
                }
                _ => None,
            };
            self.call(observed, first.ident.span(), &arguments, explicit);
            if let Some(name) = resolved.last() {
                row = self.stored_row(name, &arguments, false);
            }
        }
        self.with_row(row, |this| syn::visit::visit_expr_call(this, node));
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if self.probing_writes() {
            syn::visit::visit_expr_method_call(self, node);
            return;
        }
        let name = node.method.unraw().to_string();
        let selects = self.selects(&node.receiver);
        let arguments: Vec<_> = node.args.iter().collect();
        let row = self.stored_row(&name, &arguments, selects);
        let explicit = matches!(self.explicit_row, Some(ExplicitRow::Method(call))
            if std::ptr::eq(call, node));
        self.call(
            self.method(node),
            node.method.span(),
            &arguments,
            selects || explicit,
        );
        self.with_row(row, |this| {
            syn::visit::visit_expr_method_call(this, node);
        });
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.looped();
        self.nested_repeat(|this| this.visit_expr(&node.expr));
        self.repeated(&node.body, None, |this| {
            this.graph_scope(|this| {
                this.visit_pat(&node.pat);
                this.bind(&node.pat, None, GraphBinding::Unknown, &node.attrs);
                this.visit_block(&node.body);
            });
        });
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.looped();
        self.repeated(&node.body, Some(&node.cond), |this| {
            this.conditional_body(&node.cond, &node.body);
        });
    }

    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.conditional_body(&node.cond, &node.then_branch);
        if let Some((_, branch)) = &node.else_branch {
            self.visit_expr(branch);
        }
    }

    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        self.graph_scope(|this| {
            for pattern in &node.inputs {
                this.bind(pattern, None, GraphBinding::Unknown, &node.attrs);
            }
            this.deferred(|this| syn::visit::visit_expr_closure(this, node));
        });
    }

    fn visit_expr_async(&mut self, node: &'ast syn::ExprAsync) {
        self.deferred(|this| syn::visit::visit_expr_async(this, node));
    }

    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        self.attributed_scope(&node.attrs, |this| {
            this.graph_scope(|this| {
                this.bind(&node.pat, None, GraphBinding::Unknown, &node.attrs);
                syn::visit::visit_arm(this, node);
            });
        });
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        self.looped();
        self.repeated(&node.body, None, |this| {
            syn::visit::visit_expr_loop(this, node);
        });
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        if self.recording() {
            self.term(Term::of_segments(&self.resolve(node)));
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_lit_str(&mut self, node: &'ast syn::LitStr) {
        self.term(Term::of_iri(&node.value()));
    }

    fn visit_ident(&mut self, node: &'ast proc_macro2::Ident) {
        if self.recording()
            && let Some(frame) = self.frames.last_mut()
            && names_graph(&words(&node.unraw().to_string()))
        {
            frame.named_graphs = true;
        }
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        let kind = node
            .init
            .as_ref()
            .map_or(GraphBinding::Unknown, |init| self.initializer(&init.expr));
        self.attributed_scope(&node.attrs, |this| {
            let discarded = !this.uncertain
                && (matches!(node.pat, syn::Pat::Wild(_))
                    || this.tail_local.is_some_and(|tail| std::ptr::eq(tail, node)))
                && node.init.as_ref().is_some_and(|init| {
                    matches!(*init.expr, syn::Expr::Async(_) | syn::Expr::Closure(_))
                        && written_expression(&init.expr)
                });
            let mut handles = Vec::new();
            if discarded && let Some(frame) = this.frames.last() {
                handles.extend(
                    frame
                        .bindings
                        .iter()
                        .enumerate()
                        .filter_map(|(index, binding)| {
                            (binding.graph == GraphBinding::Handle).then_some(index)
                        }),
                );
            }
            syn::visit::visit_local(this, node);
            if let Some(frame) = this.frames.last_mut() {
                for index in handles {
                    frame.bindings[index].graph = GraphBinding::Handle;
                }
            }
            this.bind(&node.pat, None, kind, &node.attrs);
        });
    }

    fn visit_pat_type(&mut self, node: &'ast syn::PatType) {
        self.written_scope(inert_receiver_attributes(&node.attrs), |this| {
            syn::visit::visit_pat_type(this, node);
        });
    }

    fn visit_receiver(&mut self, node: &'ast syn::Receiver) {
        self.written_scope(inert_receiver_attributes(&node.attrs), |this| {
            syn::visit::visit_receiver(this, node);
        });
    }

    fn visit_expr_assign(&mut self, node: &'ast syn::ExprAssign) {
        syn::visit::visit_expr_assign(self, node);
        if let Some(frame) = self.frames.last_mut() {
            assignee_bindings(&node.left, &mut |name| {
                if let Some(binding) = frame
                    .bindings
                    .iter_mut()
                    .rev()
                    .find(|binding| name.unraw() == binding.name)
                    && binding.graph == GraphBinding::Handle
                {
                    binding.graph = GraphBinding::Unknown;
                }
            });
        }
    }

    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if self.recording()
            && let Some(frame) = self.frames.last_mut()
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

impl Structure<'_> {
    fn looped(&mut self) {
        if self.recording()
            && let Some(list) = self.frames.last_mut().and_then(|frame| frame.list.as_mut())
        {
            list.looped = true;
        }
    }

    fn term(&mut self, term: Option<Term>) {
        if self.recording()
            && let (Some(list), Some(term)) = (
                self.frames.last_mut().and_then(|frame| frame.list.as_mut()),
                term,
            )
        {
            list.terms.push(term);
        }
    }
}

fn written_expression(expression: &syn::Expr) -> bool {
    expression_attributes(expression).is_some_and(inert_receiver_attributes)
}

/// A direct terminal break rules out a backedge if neither the condition nor
/// the body can continue. Macros and uncertain attributes can introduce one.
fn single_pass(body: &syn::Block, condition: Option<&syn::Expr>) -> bool {
    terminal_break(body).is_some() && !can_continue(body, condition)
}

fn terminal_break(body: &syn::Block) -> Option<&syn::ExprBreak> {
    match body.stmts.last()? {
        syn::Stmt::Expr(syn::Expr::Break(exit), _) if inert_receiver_attributes(&exit.attrs) => {
            Some(exit)
        }
        _ => None,
    }
}

fn can_continue(body: &syn::Block, condition: Option<&syn::Expr>) -> bool {
    #[derive(Default)]
    struct CanContinue(bool);

    impl<'ast> Visit<'ast> for CanContinue {
        fn visit_expr(&mut self, node: &'ast syn::Expr) {
            if self.0 {
                return;
            }
            let Some(attributes) = expression_attributes(node) else {
                self.0 = true;
                return;
            };
            if !definitely_excluded(attributes) {
                self.0 = matches!(node, syn::Expr::Verbatim(_));
                syn::visit::visit_expr(self, node);
            }
        }

        fn visit_local(&mut self, node: &'ast syn::Local) {
            if !definitely_excluded(&node.attrs) {
                syn::visit::visit_local(self, node);
            }
        }

        fn visit_arm(&mut self, node: &'ast syn::Arm) {
            if !definitely_excluded(&node.attrs) {
                syn::visit::visit_arm(self, node);
            }
        }

        fn visit_stmt_macro(&mut self, node: &'ast syn::StmtMacro) {
            if !definitely_excluded(&node.attrs) {
                syn::visit::visit_stmt_macro(self, node);
            }
        }

        fn visit_expr_continue(&mut self, _: &'ast syn::ExprContinue) {
            self.0 = true;
        }

        fn visit_macro(&mut self, _: &'ast syn::Macro) {
            self.0 = true;
        }

        fn visit_attribute(&mut self, node: &'ast syn::Attribute) {
            self.0 |= !inert_receiver_attributes(std::slice::from_ref(node));
        }
    }

    let mut can_continue = CanContinue::default();
    if let Some(condition) = condition {
        can_continue.visit_expr(condition);
    }
    can_continue.visit_block(body);
    can_continue.0
}

/// Visit the local bindings that an assignment replaces. Receivers,
/// subscripts and tuple-struct names are reads, so their identifiers are skipped.
fn assignee_bindings(expression: &syn::Expr, visit: &mut impl FnMut(&syn::Ident)) {
    if expression_attributes(expression).is_some_and(definitely_excluded) {
        return;
    }
    let targets = match expression {
        syn::Expr::Path(path) if path.qself.is_none() => {
            if let Some(name) = path.path.get_ident() {
                visit(name);
            }
            return;
        }
        syn::Expr::Paren(inner) => return assignee_bindings(&inner.expr, visit),
        syn::Expr::Group(inner) => return assignee_bindings(&inner.expr, visit),
        syn::Expr::Tuple(tuple) => &tuple.elems,
        syn::Expr::Array(array) => &array.elems,
        syn::Expr::Call(call) if matches!(call.func.as_ref(), syn::Expr::Path(_)) => &call.args,
        syn::Expr::Struct(structure) => {
            for field in &structure.fields {
                assignee_bindings(&field.expr, visit);
            }
            return;
        }
        _ => return,
    };
    for target in targets {
        assignee_bindings(target, visit);
    }
}

fn nominal_path(ty: &syn::Type) -> Option<&syn::Path> {
    match ty {
        syn::Type::Path(path) if path.qself.is_none() => Some(&path.path),
        syn::Type::Reference(inner) => nominal_path(&inner.elem),
        syn::Type::Paren(inner) => nominal_path(&inner.elem),
        syn::Type::Group(inner) => nominal_path(&inner.elem),
        _ => None,
    }
}

fn receiver_ident(expression: &syn::Expr) -> Option<&syn::Ident> {
    match expression {
        syn::Expr::Path(path) if path.qself.is_none() => path.path.get_ident(),
        syn::Expr::Paren(inner) => receiver_ident(&inner.expr),
        syn::Expr::Group(inner) => receiver_ident(&inner.expr),
        _ => None,
    }
}

fn bare_type(ty: &syn::Type) -> &syn::Type {
    match ty {
        syn::Type::Paren(inner) => bare_type(&inner.elem),
        syn::Type::Group(inner) => bare_type(&inner.elem),
        _ => ty,
    }
}

fn borrow_mode(mutable: bool) -> ReceiverMode {
    if mutable {
        ReceiverMode::Mutable
    } else {
        ReceiverMode::Shared
    }
}

fn receiver_annotation(ty: &syn::Type) -> Option<(&syn::Path, ReceiverMode)> {
    let (nominal, mode) = match bare_type(ty) {
        syn::Type::Reference(reference) => (
            bare_type(&reference.elem),
            borrow_mode(reference.mutability.is_some()),
        ),
        nominal => (nominal, ReceiverMode::Value),
    };
    let syn::Type::Path(path) = nominal else {
        return None;
    };
    (path.qself.is_none()
        && path
            .path
            .segments
            .iter()
            .all(|part| part.arguments.is_empty()))
    .then_some((&path.path, mode))
}

fn receiver_mode(signature: &syn::Signature) -> Option<ReceiverMode> {
    let syn::FnArg::Receiver(receiver) = signature.inputs.first()? else {
        return None;
    };
    if !receiver.attrs.is_empty() {
        return None;
    }
    Some(match &receiver.kind {
        syn::ReceiverKind::Reference(_, _, mutable) => borrow_mode(mutable.is_some()),
        syn::ReceiverKind::Value => ReceiverMode::Value,
        _ => return None,
    })
}

fn iri_constructor(path: &syn::Path) -> bool {
    let mut names = path.segments.iter().rev().map(|part| &part.ident);
    match (names.next(), names.next()) {
        (Some(name), Some(owner)) => {
            (name == "iri" && (owner == "Term" || owner == "TermValue"))
                || ((name == "new" || name == "parse") && (owner == "Iri" || owner == "IriRef"))
        }
        _ => false,
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

/// Unwrap syntax that carries the same row or graph value on success.
fn graph_operand(expression: &syn::Expr) -> &syn::Expr {
    match expression {
        syn::Expr::Paren(inner) => graph_operand(&inner.expr),
        syn::Expr::Group(inner) => graph_operand(&inner.expr),
        syn::Expr::Reference(inner) => graph_operand(&inner.expr),
        syn::Expr::Try(inner) => graph_operand(&inner.expr),
        syn::Expr::MethodCall(call)
            if (call.method == "unwrap" && call.args.is_empty())
                || (call.method == "expect" && call.args.len() == 1) =>
        {
            graph_operand(&call.receiver)
        }
        _ => expression,
    }
}

fn graph_value_name(name: &str) -> bool {
    matches!(name, "g" | "name") || words(name).iter().any(|word| word == "graph")
}

fn graph_type(ty: &syn::Type) -> GraphBinding {
    if nominal_path(ty)
        .and_then(last_name)
        .is_some_and(|name| names_graph(&words(&name)))
    {
        GraphBinding::Value
    } else {
        GraphBinding::Unknown
    }
}

/// Written evidence that the sibling of a row is the supplied graph.
fn names_graph_value(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Path(path) => last_name(&path.path).is_some_and(|name| graph_value_name(&name)),
        syn::Expr::Field(field) => matches!(&field.member, syn::Member::Named(name)
            if graph_value_name(&name.unraw().to_string())),
        syn::Expr::Call(call) => match &*call.func {
            syn::Expr::Path(path)
                if path
                    .path
                    .segments
                    .last()
                    .is_some_and(|part| part.ident == "Some") =>
            {
                call.args.len() == 1
                    && call
                        .args
                        .first()
                        .is_some_and(|argument| !is_default_graph(graph_operand(argument)))
            }
            syn::Expr::Path(path) => path.path.segments.iter().any(|part| {
                words(&part.ident.unraw().to_string())
                    .iter()
                    .any(|word| word == "graph")
            }),
            _ => false,
        },
        _ => false,
    }
}

/// An object written as a triple term.
fn is_triple_term(expression: &syn::Expr) -> bool {
    let named = |path: &syn::Path| {
        last_name(path).is_some_and(|name| TRIPLE_TERMS.contains(&name.as_str()))
    };
    match graph_operand(expression) {
        syn::Expr::Call(call) if matches!(&*call.func, syn::Expr::Path(path) if named(&path.path)) => {
            true
        }
        syn::Expr::Struct(built) if named(&built.path) => true,
        syn::Expr::MethodCall(call) => {
            TRIPLE_TERMS.contains(&call.method.unraw().to_string().as_str())
        }
        _ => false,
    }
}

/// A graph argument that names the default graph.
fn is_default_graph(expression: &syn::Expr) -> bool {
    let default = |path: &syn::Path| {
        last_name(path).is_some_and(|name| DEFAULT_GRAPH.contains(&name.as_str()))
    };
    match graph_operand(expression) {
        syn::Expr::Path(path) => default(&path.path),
        syn::Expr::Call(call) if call.args.is_empty() => {
            matches!(&*call.func, syn::Expr::Path(path) if default(&path.path))
        }
        _ => false,
    }
}

pub(super) fn collect(file: &syn::File) -> Vec<RawHit> {
    let mut structure = Structure::default();
    structure.visit_file(file);
    structure.hits
}

/// Crate-root methods whose declarations neither change shape nor disappear
/// through attributes. Source verifies the root and inherent definition.
pub(super) fn receivers(file: &syn::File) -> Vec<(usize, ReceiverMode)> {
    if !inert_receiver_attributes(&file.attrs) {
        return Vec::new();
    }
    file.items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Impl(block)
                if block.trait_.is_none() && inert_receiver_attributes(&block.attrs) =>
            {
                Some(block)
            }
            _ => None,
        })
        .flat_map(|block| &block.items)
        .filter_map(|member| match member {
            syn::ImplItem::Fn(method) if inert_receiver_attributes(&method.attrs) => {
                receiver_mode(&method.sig).map(|mode| (member.span().byte_range().start, mode))
            }
            _ => None,
        })
        .collect()
}
