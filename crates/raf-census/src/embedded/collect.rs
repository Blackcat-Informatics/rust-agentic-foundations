// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Lexical occurrences and syntax boundaries from the bounded Source AST.
//!
//! Returned records borrow only original source, never parser spans or tokens.
//! Their buffers retain the owning analysis's original admission. The complete
//! analysis must retain these records together with that meter until it ends.

use syn::punctuated::Pair;
use syn::visit::Visit;

use super::attributes;
use super::proof::{Declaration, Event, Family, Occurrence};
use super::resource::{Meter, Refusal, Scratch};
use super::scopes::{PatternMeaning, Scopes};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Span {
    pub(super) start: usize,
    pub(super) end: usize,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Annotation<'source> {
    pub(super) span: Span,
    /// Original syntax that owns this event, including zero-width markers.
    pub(super) anchor: Span,
    /// Zero-width control markers supplement, never replace, fixed syntax.
    pub(super) marker: bool,
    pub(super) occurrence: Occurrence<'source>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Statement {
    pub(super) block: usize,
    pub(super) span: Span,
}

/// Owned flat buffers; the outer analysis owns their unreleased original grant.
pub(super) struct Facts<'source> {
    pub(super) body: Span,
    pub(super) declarations: Vec<Declaration>,
    pub(super) annotations: Vec<Annotation<'source>>,
    pub(super) expressions: Vec<Span>,
    pub(super) statements: Vec<Statement>,
    pub(super) opaque: Vec<Span>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Failure {
    pub(super) byte: usize,
    pub(super) cause: Refusal,
}

#[derive(Clone, Copy)]
struct Pending<'source> {
    name: &'source str,
    id: usize,
    mode: u8,
}

#[derive(Clone, Copy)]
struct Control<'source> {
    id: usize,
    family: Family,
    label: Option<&'source str>,
}

#[derive(Clone, Copy)]
enum Transfer {
    Return,
    Try,
    Await,
    Break { continuing: bool },
}

/// Collect one actual parsed function, under its real enclosing lexical frames.
pub(super) fn function<'source, 'meter>(
    source: &'source str,
    signature: &syn::Signature,
    body: &syn::Block,
    scopes: &mut Scopes<'source, 'meter>,
    meter: &'meter Meter,
) -> Result<Facts<'source>, Failure> {
    let range = scopes
        .range(body.brace_token.span.join())
        .map_err(|cause| Failure { byte: 0, cause })?;
    let body_span = Span {
        start: range.start,
        end: range.end,
    };
    let mut collector = Collector {
        source,
        scopes,
        meter,
        facts: Facts {
            body: body_span,
            declarations: Vec::new(),
            annotations: Vec::new(),
            expressions: Vec::new(),
            statements: Vec::new(),
            opaque: Vec::new(),
        },
        controls: Scratch::new(meter),
        failure: None,
        condition: false,
        next_block: 0,
    };
    let result = collector.start(signature, body);
    if let Some(failure) = collector.failure {
        return Err(failure);
    }
    result.map_err(|cause| Failure {
        byte: body_span.start,
        cause,
    })?;
    Ok(collector.facts)
}

struct Collector<'source, 'borrow, 'meter> {
    source: &'source str,
    scopes: &'borrow mut Scopes<'source, 'meter>,
    meter: &'meter Meter,
    facts: Facts<'source>,
    controls: Scratch<'meter, Control<'source>>,
    failure: Option<Failure>,
    condition: bool,
    next_block: usize,
}

impl<'source> Collector<'source, '_, '_> {
    fn checked(&self, span: proc_macro2::Span) -> Result<Span, Refusal> {
        self.checked_ends((span, span))
    }

    fn checked_ends(&self, ends: super::native_spans::Ends) -> Result<Span, Refusal> {
        let range = self.scopes.coordinates().between(ends.0, ends.1)?;
        self.meter.work(1)?;
        if range.start >= range.end || self.source.get(range.clone()).is_none() {
            return Err(Refusal::Invariant(
                "a syntax region has no nonempty UTF-8 source span",
            ));
        }
        Ok(Span {
            start: range.start,
            end: range.end,
        })
    }

    fn expression_span(&self, expression: &syn::Expr) -> Result<Span, Refusal> {
        self.checked_ends(super::native_spans::expression::ends(
            expression,
            self.scopes.coordinates(),
        )?)
    }

    fn statement_span(&self, statement: &syn::Stmt) -> Result<Span, Refusal> {
        let ends = super::native_spans::statement::ends(statement, self.scopes.coordinates())?
            .unwrap_or_else(|| {
                let empty = proc_macro2::Span::call_site();
                (empty, empty)
            });
        self.checked_ends(ends)
    }

    fn type_span(&self, ty: &syn::Type) -> Result<Span, Refusal> {
        let ends =
            super::native_spans::ty::ends(ty, self.scopes.coordinates())?.unwrap_or_else(|| {
                // A complete empty type has the native printer's call-site span.
                // Empty children never use this value as a parent endpoint.
                let empty = proc_macro2::Span::call_site();
                (empty, empty)
            });
        self.checked_ends(ends)
    }

    fn pattern_span(&self, pattern: &syn::Pat) -> Result<Span, Refusal> {
        let ends = super::native_spans::pattern::ends(pattern, self.scopes.coordinates())?
            .unwrap_or_else(|| {
                let empty = proc_macro2::Span::call_site();
                (empty, empty)
            });
        self.checked_ends(ends)
    }

    fn inside(&self, span: Span) -> bool {
        self.facts.body.start <= span.start && span.end <= self.facts.body.end
    }

    fn start(&mut self, signature: &syn::Signature, body: &syn::Block) -> Result<(), Refusal> {
        let source_body = self
            .source
            .get(self.facts.body.start..self.facts.body.end)
            .ok_or(Refusal::Invariant("function body is outside its source"))?;
        if !source_body.starts_with('{') || !source_body.ends_with('}') {
            return Err(Refusal::Invariant(
                "function body is not an original braced block",
            ));
        }
        if signature.asyncness.is_some() && signature.constness.is_some() {
            let byte = self
                .scopes
                .range(super::native_spans::signature_start(signature))?
                .start;
            let cause =
                Refusal::Unsupported("a function cannot combine async and const boundary profiles");
            self.record(byte, cause);
            return Err(cause);
        }
        self.scopes.enter_values()?;
        for parameter in signature.generics.params.pairs().map(Pair::into_value) {
            let first = super::native_spans::generic_start(parameter, self.meter)?;
            let byte = self.scopes.range(first)?.start;
            self.meter.work(1)?;
            let attrs = match parameter {
                syn::GenericParam::Const(parameter) => &parameter.attrs,
                syn::GenericParam::Type(parameter) => &parameter.attrs,
                syn::GenericParam::Lifetime(parameter) => &parameter.attrs,
            };
            attributes::list(attrs, self.meter, self.scopes.coordinates()).inspect_err(
                |&cause| {
                    self.record(byte, cause);
                },
            )?;
            if let syn::GenericParam::Const(parameter) = parameter {
                let name = self.scopes.ident(&parameter.ident)?;
                self.scopes.constant(name)?;
            }
        }
        let family = if signature.asyncness.is_some() {
            Family::AsyncFunction
        } else if signature.constness.is_some() {
            Family::ConstFunction
        } else {
            Family::Function
        };
        if family.asynchronous() {
            self.scopes.asynchronous_profile()?;
        }
        let target = self.declaration(signature.ident.span(), family)?;
        self.controls.push(Control {
            id: target,
            family,
            label: None,
        })?;
        let mut pending = Scratch::new(self.meter);
        for argument in signature.inputs.pairs().map(Pair::into_value) {
            self.meter.work(1)?;
            match argument {
                syn::FnArg::Typed(argument) => {
                    attributes::list(&argument.attrs, self.meter, self.scopes.coordinates())?;
                    self.pattern(&argument.pat, &mut pending, None, None)?;
                }
                syn::FnArg::Receiver(receiver) => {
                    attributes::list(&receiver.attrs, self.meter, self.scopes.coordinates())?;
                    let span = self.checked(receiver.self_token.span)?;
                    let name = self
                        .source
                        .get(span.start..span.end)
                        .filter(|name| *name == "self")
                        .ok_or(Refusal::Invariant(
                            "method receiver has no original self token",
                        ))?;
                    let id = self.declaration(receiver.self_token.span, Family::Receiver)?;
                    pending.push(Pending { name, id, mode: 0 })?;
                }
            }
        }
        self.activate(&pending)?;
        self.visit_block(body);
        self.check_failure()?;
        let _ = self.controls.pop()?;
        self.scopes.leave()
    }

    fn declaration(&mut self, span: proc_macro2::Span, family: Family) -> Result<usize, Refusal> {
        let span = self.checked(span)?;
        self.declaration_at(span, family)
    }

    fn declaration_at(&mut self, span: Span, family: Family) -> Result<usize, Refusal> {
        let id = self.facts.declarations.len();
        self.meter.push_retained(
            &mut self.facts.declarations,
            Declaration {
                byte: span.start,
                family,
            },
        )?;
        Ok(id)
    }

    fn annotate(
        &mut self,
        span: proc_macro2::Span,
        occurrence: Occurrence<'source>,
    ) -> Result<(), Refusal> {
        let span = self.checked(span)?;
        if self.inside(span) {
            self.meter.push_retained(
                &mut self.facts.annotations,
                Annotation {
                    span,
                    anchor: span,
                    marker: false,
                    occurrence,
                },
            )?;
        }
        Ok(())
    }

    fn marker(
        &mut self,
        span: proc_macro2::Span,
        id: usize,
        event: Event,
        opcode: &'static str,
    ) -> Result<(), Refusal> {
        let token = self.checked(span)?;
        if !self.inside(token) {
            return Err(Refusal::Invariant(
                "a control marker is outside its function body",
            ));
        }
        self.meter.push_retained(
            &mut self.facts.annotations,
            Annotation {
                span: Span {
                    start: token.end,
                    end: token.end,
                },
                anchor: token,
                marker: true,
                occurrence: Occurrence {
                    binding: id,
                    event,
                    fixed: Some(opcode),
                },
            },
        )
    }

    fn entry(&mut self, origin: Span, id: usize, opcode: &'static str) -> Result<(), Refusal> {
        if !self.inside(origin) {
            return Err(Refusal::Invariant(
                "boundary entry is outside its source owner",
            ));
        }
        self.meter.push_retained(
            &mut self.facts.annotations,
            Annotation {
                span: Span {
                    start: origin.start,
                    end: origin.start,
                },
                anchor: origin,
                marker: true,
                occurrence: Occurrence {
                    binding: id,
                    event: Event::Declaration,
                    fixed: Some(opcode),
                },
            },
        )
    }

    fn activate(&mut self, pending: &Scratch<'_, Pending<'source>>) -> Result<(), Refusal> {
        for binding in pending.iter() {
            self.meter.work(1)?;
            self.scopes.bind(binding.name, binding.id)?;
        }
        Ok(())
    }

    fn check_failure(&self) -> Result<(), Refusal> {
        self.failure.map_or(Ok(()), |failure| Err(failure.cause))
    }

    fn record(&mut self, byte: usize, cause: Refusal) {
        if self.failure.is_none() {
            self.failure = Some(Failure { byte, cause });
        }
    }

    fn anchor(&mut self, span: proc_macro2::Span) -> Option<usize> {
        self.anchor_result(Ok(span))
    }

    fn anchor_result(&mut self, span: Result<proc_macro2::Span, Refusal>) -> Option<usize> {
        match span.and_then(|span| self.scopes.range(span)) {
            Ok(range) => Some(range.start),
            Err(cause) => {
                self.record(self.facts.body.start, cause);
                None
            }
        }
    }

    fn opaque_at(&mut self, span: Span) -> Result<(), Refusal> {
        if !self.inside(span) {
            return Err(Refusal::Invariant(
                "an opaque source region is outside its owner",
            ));
        }
        self.meter.push_retained(&mut self.facts.opaque, span)
    }

    fn opaque_ends(&mut self, ends: Result<Option<super::native_spans::Ends>, Refusal>) {
        let ends = match ends {
            Ok(Some(ends)) => ends,
            Ok(None) => {
                let empty = proc_macro2::Span::call_site();
                (empty, empty)
            }
            Err(cause) => {
                self.record(self.facts.body.start, cause);
                return;
            }
        };
        let Some(byte) = self.anchor(ends.0) else {
            return;
        };
        let result = self
            .checked_ends(ends)
            .and_then(|span| self.opaque_at(span));
        if let Err(cause) = result {
            self.record(byte, cause);
        }
    }

    fn pattern(
        &mut self,
        pattern: &syn::Pat,
        pending: &mut Scratch<'_, Pending<'source>>,
        expected: Option<&[Pending<'source>]>,
        field: Option<&'source str>,
    ) -> Result<(), Refusal> {
        self.meter.work(1)?;
        attributes::pattern(pattern, self.meter, self.scopes.coordinates())?;
        // Parameter binders belong to this function, but signature-owned
        // const/type syntax is outside the body analyzed by fragment proof.
        let in_body = self.inside(self.pattern_span(pattern)?);
        match pattern {
            syn::Pat::Ident(pattern) => {
                self.pattern_ident(pattern, pending, expected, field)?;
                if let Some((_, subpattern)) = &pattern.subpat {
                    self.pattern(subpattern, pending, expected, None)?;
                }
            }
            syn::Pat::Or(pattern) => self.alternatives(pattern, pending, expected)?,
            syn::Pat::Paren(pattern) => self.pattern(&pattern.pat, pending, expected, field)?,
            syn::Pat::Reference(pattern) => self.pattern(&pattern.pat, pending, expected, field)?,
            syn::Pat::Type(pattern) => {
                self.pattern(&pattern.pat, pending, expected, field)?;
                if in_body {
                    self.visit_type(&pattern.ty);
                    self.check_failure()?;
                }
            }
            syn::Pat::Tuple(pattern) => {
                for element in pattern.elems.pairs().map(Pair::into_value) {
                    self.pattern(element, pending, expected, None)?;
                }
            }
            syn::Pat::TupleStruct(pattern) => {
                if in_body {
                    if let Some(qself) = &pattern.qself {
                        self.visit_type(&qself.ty);
                    }
                    self.visit_path(&pattern.path);
                    self.check_failure()?;
                }
                for element in pattern.elems.pairs().map(Pair::into_value) {
                    self.pattern(element, pending, expected, None)?;
                }
            }
            syn::Pat::Slice(pattern) => {
                for element in pattern.elems.pairs().map(Pair::into_value) {
                    self.pattern(element, pending, expected, None)?;
                }
            }
            syn::Pat::Struct(pattern) => {
                if in_body {
                    if let Some(qself) = &pattern.qself {
                        self.visit_type(&qself.ty);
                    }
                    self.visit_path(&pattern.path);
                    self.check_failure()?;
                }
                for member in pattern.fields.pairs().map(Pair::into_value) {
                    let fixed = self.shorthand(member)?;
                    self.pattern(&member.pat, pending, expected, fixed)?;
                }
            }
            syn::Pat::Path(pattern) if in_body => {
                if let Some(qself) = &pattern.qself {
                    self.visit_type(&qself.ty);
                }
                self.visit_path(&pattern.path);
                self.check_failure()?;
            }
            syn::Pat::Range(pattern) if in_body => {
                if let Some(start) = &pattern.start {
                    self.const_expression(start, "pattern-bound")?;
                }
                if let Some(end) = &pattern.end {
                    self.const_expression(end, "pattern-bound")?;
                }
            }
            syn::Pat::Const(pattern) if in_body => {
                pattern.modifiers.require_empty().map_err(|_| {
                    Refusal::Unsupported("const pattern modifiers have no admitted profile")
                })?;
                self.boundary_block(
                    pattern.const_token.span,
                    &pattern.block,
                    Family::ConstContext,
                    "const-pattern",
                )?;
            }
            syn::Pat::Lit(_)
            | syn::Pat::Rest(_)
            | syn::Pat::Wild(_)
            | syn::Pat::Path(_)
            | syn::Pat::Range(_)
            | syn::Pat::Const(_) => {}
            _ => {
                return Err(Refusal::Unsupported(
                    "pattern binding meaning is not modeled for this syntax",
                ));
            }
        }
        Ok(())
    }

    fn shorthand(&self, member: &syn::FieldPat) -> Result<Option<&'source str>, Refusal> {
        if member.colon_token.is_some() {
            return Ok(None);
        }
        let syn::Member::Named(name) = &member.member else {
            return Err(Refusal::Invariant("a shorthand pattern has no named field"));
        };
        self.scopes.ident(name).map(Some)
    }

    fn pattern_ident(
        &mut self,
        pattern: &syn::PatIdent,
        pending: &mut Scratch<'_, Pending<'source>>,
        expected: Option<&[Pending<'source>]>,
        fixed: Option<&'source str>,
    ) -> Result<(), Refusal> {
        attributes::list(&pattern.attrs, self.meter, self.scopes.coordinates())?;
        let name = self.scopes.ident(&pattern.ident)?;
        let mode =
            u8::from(pattern.by_ref.is_some()) | (u8::from(pattern.mutability.is_some()) << 1);
        match self.scopes.pattern(name)? {
            PatternMeaning::Fixed if mode == 0 && pattern.subpat.is_none() => return Ok(()),
            PatternMeaning::Fixed => {
                return Err(Refusal::Unsupported(
                    "an explicit binding conflicts with a fixed pattern name",
                ));
            }
            PatternMeaning::Unsupported(reason) => return Err(Refusal::Unsupported(reason)),
            PatternMeaning::Binding => {}
        }
        for prior in pending.iter() {
            self.meter.work(1)?;
            if self.scopes.same_name(prior.name, name)? {
                return Err(Refusal::Unsupported(
                    "one pattern binds the same name more than once",
                ));
            }
        }
        let id = if let Some(expected) = expected {
            let mut found = None;
            for prior in expected {
                self.meter.work(1)?;
                if self.scopes.same_name(prior.name, name)? {
                    if prior.mode != mode {
                        return Err(Refusal::Unsupported(
                            "or-pattern alternatives change a binding mode",
                        ));
                    }
                    found = Some(prior.id);
                }
            }
            found.ok_or(Refusal::Unsupported(
                "or-pattern alternatives bind different names",
            ))?
        } else {
            self.declaration(pattern.ident.span(), Family::Value)?
        };
        pending.push(Pending { name, id, mode })?;
        self.annotate(
            pattern.ident.span(),
            Occurrence {
                binding: id,
                event: Event::Declaration,
                fixed,
            },
        )
    }

    fn alternatives(
        &mut self,
        pattern: &syn::PatOr,
        pending: &mut Scratch<'_, Pending<'source>>,
        expected: Option<&[Pending<'source>]>,
    ) -> Result<(), Refusal> {
        let mut cases = pattern.cases.pairs().map(Pair::into_value);
        let first = cases
            .next()
            .ok_or(Refusal::Invariant("an or-pattern has no alternative"))?;
        let mut names = Scratch::new(self.meter);
        self.pattern(first, &mut names, expected, None)?;
        for case in cases {
            let mut alternative = Scratch::new(self.meter);
            self.pattern(case, &mut alternative, Some(names.as_slice()), None)?;
            if names.len() != alternative.len() {
                return Err(Refusal::Unsupported(
                    "or-pattern alternatives bind different names",
                ));
            }
        }
        for name in names.iter() {
            for prior in pending.iter() {
                self.meter.work(1)?;
                if self.scopes.same_name(prior.name, name.name)? {
                    return Err(Refusal::Unsupported(
                        "one pattern binds the same name more than once",
                    ));
                }
            }
            pending.push(*name)?;
        }
        Ok(())
    }

    fn expression(&mut self, expression: &syn::Expr) -> Result<(), Refusal> {
        self.meter.work(1)?;
        attributes::expression(expression, self.meter, self.scopes.coordinates())?;
        let span = self.expression_span(expression)?;
        if !self.inside(span) {
            return Err(Refusal::Invariant(
                "an expression region is outside its source owner",
            ));
        }
        self.meter
            .push_retained(&mut self.facts.expressions, span)?;
        self.expression_contents(expression)?;
        self.check_failure()
    }

    fn expression_contents(&mut self, expression: &syn::Expr) -> Result<(), Refusal> {
        match expression {
            syn::Expr::Path(path) => self.path(path)?,
            syn::Expr::If(node) => self.if_expression(node)?,
            syn::Expr::Closure(node) => self.closure(node)?,
            syn::Expr::Async(_) | syn::Expr::Const(_) | syn::Expr::TryBlock(_) => {
                self.expression_boundary(expression)?;
            }
            syn::Expr::Await(node) => {
                self.visit_expr(&node.base);
                self.check_failure()?;
                let target = self.target(None, Transfer::Await)?;
                self.marker(node.await_token.span, target, Event::Use, "await")?;
            }
            syn::Expr::Repeat(node) => {
                self.visit_expr(&node.expr);
                self.check_failure()?;
                self.const_expression(&node.len, "repeat-length")?;
            }
            syn::Expr::ForLoop(node) => self.for_loop(node)?,
            syn::Expr::While(node) => self.while_loop(node)?,
            syn::Expr::Loop(node) => self.loop_expression(node)?,
            syn::Expr::Block(node) if node.label.is_some() => self.labelled_block(node)?,
            syn::Expr::Return(node) => {
                let target = self.target(None, Transfer::Return)?;
                self.marker(node.return_token.span, target, Event::Use, "return")?;
                if let Some(value) = &node.expr {
                    self.visit_expr(value);
                }
            }
            syn::Expr::Break(node) => {
                let target =
                    self.target(node.label.as_ref(), Transfer::Break { continuing: false })?;
                self.marker(node.break_token.span, target, Event::Use, "break")?;
                self.label_use(node.label.as_ref(), target)?;
                if let Some(value) = &node.expr {
                    self.visit_expr(value);
                }
            }
            syn::Expr::Continue(node) => {
                let target =
                    self.target(node.label.as_ref(), Transfer::Break { continuing: true })?;
                self.marker(node.continue_token.span, target, Event::Use, "continue")?;
                self.label_use(node.label.as_ref(), target)?;
            }
            syn::Expr::Try(node) => {
                let target = self.target(None, Transfer::Try)?;
                self.marker(node.question_token.span, target, Event::Use, "try")?;
                self.visit_expr(&node.expr);
            }
            syn::Expr::Let(node) if self.condition => {
                self.visit_expr(&node.expr);
                self.check_failure()?;
                let mut pending = Scratch::new(self.meter);
                self.pattern(&node.pat, &mut pending, None, None)?;
                self.activate(&pending)?;
            }
            syn::Expr::Macro(_) => {
                let span = self.expression_span(expression)?;
                self.opaque_at(span)?;
            }
            syn::Expr::Yield(_) | syn::Expr::Verbatim(_) | syn::Expr::Let(_) => {
                return Err(Refusal::Unsupported(
                    "this expression needs an additional modeled lexical or control boundary",
                ));
            }
            syn::Expr::Array(_)
            | syn::Expr::Assign(_)
            | syn::Expr::Binary(_)
            | syn::Expr::Block(_)
            | syn::Expr::Call(_)
            | syn::Expr::Cast(_)
            | syn::Expr::Field(_)
            | syn::Expr::Group(_)
            | syn::Expr::Index(_)
            | syn::Expr::Infer(_)
            | syn::Expr::Lit(_)
            | syn::Expr::Match(_)
            | syn::Expr::MethodCall(_)
            | syn::Expr::Paren(_)
            | syn::Expr::Range(_)
            | syn::Expr::RawAddr(_)
            | syn::Expr::Reference(_)
            | syn::Expr::Struct(_)
            | syn::Expr::Tuple(_)
            | syn::Expr::Unary(_)
            | syn::Expr::Unsafe(_) => syn::visit::visit_expr(self, expression),
            _ => {
                return Err(Refusal::Unsupported(
                    "expression syntax has no admitted lexical profile",
                ));
            }
        }
        Ok(())
    }

    fn expression_boundary(&mut self, expression: &syn::Expr) -> Result<(), Refusal> {
        let (modifiers, span, block, family, name, refusal) = match expression {
            syn::Expr::Async(node) => {
                self.scopes.asynchronous_profile()?;
                (
                    &node.modifiers,
                    node.async_token.span,
                    &node.block,
                    Family::AsyncBlock,
                    "async",
                    "async block modifiers have no admitted profile",
                )
            }
            syn::Expr::Const(node) => (
                &node.modifiers,
                node.const_token.span,
                &node.block,
                Family::ConstContext,
                "const",
                "const block modifiers have no admitted profile",
            ),
            syn::Expr::TryBlock(node) => (
                &node.modifiers,
                node.try_token.span,
                &node.block,
                Family::Try,
                "try-block",
                "try block modifiers have no admitted profile",
            ),
            _ => return Err(Refusal::Invariant("expression is not a block boundary")),
        };
        modifiers
            .require_empty()
            .map_err(|_| Refusal::Unsupported(refusal))?;
        self.boundary_block(span, block, family, name)
    }

    fn path(&mut self, path: &syn::ExprPath) -> Result<(), Refusal> {
        if path.qself.is_none()
            && path.path.leading_colon.is_none()
            && path.path.segments.len() == 1
        {
            let name = &path.path.segments[0].ident;
            if let Some(id) = self.scopes.value(self.scopes.ident(name)?)? {
                self.annotate(
                    name.span(),
                    Occurrence {
                        binding: id,
                        event: Event::Use,
                        fixed: None,
                    },
                )?;
            }
        }
        // Type names stay fixed. Explicit const expressions in generic
        // arguments still need their own no-capture/control context.
        if let Some(qself) = &path.qself {
            self.visit_type(&qself.ty);
        }
        for segment in path.path.segments.pairs().map(Pair::into_value) {
            self.visit_path_arguments(&segment.arguments);
        }
        self.check_failure()
    }

    fn if_expression(&mut self, node: &syn::ExprIf) -> Result<(), Refusal> {
        self.scopes.enter_values()?;
        let prior = self.condition;
        self.condition = true;
        self.visit_expr(&node.cond);
        self.condition = prior;
        self.check_failure()?;
        self.visit_block(&node.then_branch);
        self.check_failure()?;
        self.scopes.leave()?;
        if let Some((_, other)) = &node.else_branch {
            self.visit_expr(other);
        }
        self.check_failure()
    }

    fn closure(&mut self, node: &syn::ExprClosure) -> Result<(), Refusal> {
        if node.asyncness.is_some() {
            self.scopes.asynchronous_profile()?;
        }
        if node.constness.is_some() {
            return Err(Refusal::Unsupported(
                "const closures have no admitted capture profile",
            ));
        }
        node.modifiers
            .require_empty()
            .map_err(|_| Refusal::Unsupported("closure modifiers have no admitted profile"))?;
        self.scopes.enter_values()?;
        let family = if node.asyncness.is_some() {
            Family::AsyncClosure
        } else {
            Family::Closure
        };
        let start =
            super::native_spans::expression::closure_start(node, self.scopes.coordinates())?;
        let end = super::native_spans::expression::end(&node.body, self.scopes.coordinates())?;
        let origin = self.checked_ends((start, end))?;
        let id = self.declaration_at(origin, family)?;
        self.controls.push(Control {
            id,
            family,
            label: None,
        })?;
        self.entry(
            origin,
            id,
            if node.asyncness.is_some() {
                "async-closure"
            } else {
                "closure"
            },
        )?;
        let mut pending = Scratch::new(self.meter);
        for parameter in node.inputs.pairs().map(Pair::into_value) {
            self.pattern(parameter, &mut pending, None, None)?;
        }
        self.activate(&pending)?;
        self.visit_return_type(&node.output);
        self.check_failure()?;
        let prior = self.condition;
        self.condition = false;
        self.visit_expr(&node.body);
        self.condition = prior;
        self.check_failure()?;
        let _ = self.controls.pop()?;
        self.scopes.leave()
    }

    fn boundary_block(
        &mut self,
        token: proc_macro2::Span,
        body: &syn::Block,
        family: Family,
        opcode: &'static str,
    ) -> Result<(), Refusal> {
        let id = self.declaration(token, family)?;
        self.controls.push(Control {
            id,
            family,
            label: None,
        })?;
        self.marker(token, id, Event::Declaration, opcode)?;
        if family == Family::ConstContext {
            self.scopes.enter_const()?;
        }
        let prior = self.condition;
        self.condition = false;
        self.visit_block(body);
        self.condition = prior;
        self.check_failure()?;
        if family == Family::ConstContext {
            self.scopes.leave()?;
        }
        let _ = self.controls.pop()?;
        Ok(())
    }

    fn const_expression(
        &mut self,
        expression: &syn::Expr,
        opcode: &'static str,
    ) -> Result<(), Refusal> {
        let origin = self.expression_span(expression)?;
        let id = self.declaration_at(origin, Family::ConstContext)?;
        self.controls.push(Control {
            id,
            family: Family::ConstContext,
            label: None,
        })?;
        self.entry(origin, id, opcode)?;
        self.scopes.enter_const()?;
        let prior = self.condition;
        self.condition = false;
        self.visit_expr(expression);
        self.condition = prior;
        self.check_failure()?;
        self.scopes.leave()?;
        let _ = self.controls.pop()?;
        Ok(())
    }

    fn control(
        &mut self,
        token: proc_macro2::Span,
        label: Option<&syn::Label>,
        opcode: &'static str,
    ) -> Result<(), Refusal> {
        let primary = label.map_or(token, |label| label.name.ident.span());
        let id = self.declaration(primary, Family::Loop)?;
        let label_name = label
            .map(|label| self.scopes.ident(&label.name.ident))
            .transpose()?;
        self.controls.push(Control {
            id,
            family: Family::Loop,
            label: label_name,
        })?;
        if let Some(label) = label {
            self.annotate(
                label.name.ident.span(),
                Occurrence {
                    binding: id,
                    event: Event::Declaration,
                    fixed: Some("label"),
                },
            )?;
        }
        self.marker(token, id, Event::Declaration, opcode)
    }

    fn for_loop(&mut self, node: &syn::ExprForLoop) -> Result<(), Refusal> {
        self.visit_expr(&node.expr);
        self.check_failure()?;
        self.scopes.enter_values()?;
        let mut pending = Scratch::new(self.meter);
        self.pattern(&node.pat, &mut pending, None, None)?;
        self.activate(&pending)?;
        self.control(node.for_token.span, node.label.as_ref(), "for")?;
        self.visit_block(&node.body);
        self.check_failure()?;
        let _ = self.controls.pop()?;
        self.scopes.leave()
    }

    fn while_loop(&mut self, node: &syn::ExprWhile) -> Result<(), Refusal> {
        self.scopes.enter_values()?;
        self.control(node.while_token.span, node.label.as_ref(), "while")?;
        let prior = self.condition;
        self.condition = true;
        self.visit_expr(&node.cond);
        self.condition = prior;
        self.check_failure()?;
        self.visit_block(&node.body);
        self.check_failure()?;
        let _ = self.controls.pop()?;
        self.scopes.leave()
    }

    fn loop_expression(&mut self, node: &syn::ExprLoop) -> Result<(), Refusal> {
        self.control(node.loop_token.span, node.label.as_ref(), "loop")?;
        self.visit_block(&node.body);
        self.check_failure()?;
        let _ = self.controls.pop()?;
        Ok(())
    }

    fn labelled_block(&mut self, node: &syn::ExprBlock) -> Result<(), Refusal> {
        let label = node
            .label
            .as_ref()
            .ok_or(Refusal::Invariant("labelled block has no label"))?;
        let id = self.declaration(label.name.ident.span(), Family::Label)?;
        let name = self.scopes.ident(&label.name.ident)?;
        self.controls.push(Control {
            id,
            family: Family::Label,
            label: Some(name),
        })?;
        self.annotate(
            label.name.ident.span(),
            Occurrence {
                binding: id,
                event: Event::Declaration,
                fixed: Some("label"),
            },
        )?;
        self.marker(
            node.block.brace_token.span.open(),
            id,
            Event::Declaration,
            "block",
        )?;
        self.visit_block(&node.block);
        self.check_failure()?;
        let _ = self.controls.pop()?;
        Ok(())
    }

    fn label_use(&mut self, label: Option<&syn::Lifetime>, id: usize) -> Result<(), Refusal> {
        if let Some(label) = label {
            self.annotate(
                label.ident.span(),
                Occurrence {
                    binding: id,
                    event: Event::Use,
                    fixed: Some("label"),
                },
            )?;
        }
        Ok(())
    }

    fn arm_pattern(&mut self, pattern: &syn::Pat) -> Result<(), Refusal> {
        attributes::pattern(pattern, self.meter, self.scopes.coordinates())?;
        if let syn::Pat::Guard(guard) = pattern {
            self.arm_pattern(&guard.pat)?;
            self.condition = true;
            self.visit_expr(&guard.guard);
            self.check_failure()
        } else {
            let mut pending = Scratch::new(self.meter);
            self.pattern(pattern, &mut pending, None, None)?;
            self.activate(&pending)
        }
    }

    fn target(&self, label: Option<&syn::Lifetime>, transfer: Transfer) -> Result<usize, Refusal> {
        let label_name = label
            .map(|label| self.scopes.ident(&label.ident))
            .transpose()?;
        for control in self.controls.iter().rev() {
            self.meter.work(1)?;
            if control.family == Family::ConstContext {
                break;
            }
            match transfer {
                Transfer::Return if control.family.function_like() => return Ok(control.id),
                Transfer::Try
                    if control.family == Family::Try || control.family.function_like() =>
                {
                    return Ok(control.id);
                }
                Transfer::Await if control.family.asynchronous() => return Ok(control.id),
                Transfer::Await if control.family.function_like() => break,
                Transfer::Break { continuing } => {
                    let matched = match (label_name, control.label) {
                        (Some(written), Some(known)) => self.scopes.same_name(written, known)?,
                        (None, _) => control.family == Family::Loop,
                        _ => false,
                    };
                    if matched {
                        if continuing && control.family != Family::Loop {
                            return Err(Refusal::Unsupported(
                                "continue names a labelled block rather than a loop",
                            ));
                        }
                        return Ok(control.id);
                    }
                    if control.family.function_like() {
                        break;
                    }
                }
                _ => {}
            }
        }
        Err(Refusal::Unsupported(
            "control transfer has no admitted lexical destination",
        ))
    }
}

impl<'ast> Visit<'ast> for Collector<'_, '_, '_> {
    fn visit_expr(&mut self, expression: &'ast syn::Expr) {
        if self.failure.is_some() {
            return;
        }
        let start = super::native_spans::expression::start(expression, self.scopes.coordinates());
        let Some(byte) = self.anchor_result(start) else {
            return;
        };
        if let Err(cause) = self.expression(expression) {
            self.record(byte, cause);
        }
    }

    fn visit_type(&mut self, ty: &'ast syn::Type) {
        if self.failure.is_some() {
            return;
        }
        let start = super::native_spans::ty::start(ty, self.scopes.coordinates())
            .map(|span| span.unwrap_or_else(proc_macro2::Span::call_site));
        let Some(byte) = self.anchor_result(start) else {
            return;
        };
        let result = (|| {
            self.meter.work(1)?;
            attributes::ty(ty, self.meter, self.scopes.coordinates())?;
            match ty {
                syn::Type::Macro(_) | syn::Type::Verbatim(_) => {
                    let span = self.type_span(ty)?;
                    self.opaque_at(span)?;
                }
                syn::Type::Array(_)
                | syn::Type::FnPtr(_)
                | syn::Type::Group(_)
                | syn::Type::ImplTrait(_)
                | syn::Type::Infer(_)
                | syn::Type::Never(_)
                | syn::Type::Paren(_)
                | syn::Type::Path(_)
                | syn::Type::Ptr(_)
                | syn::Type::Reference(_)
                | syn::Type::Slice(_)
                | syn::Type::TraitObject(_)
                | syn::Type::Tuple(_) => syn::visit::visit_type(self, ty),
                _ => {
                    return Err(Refusal::Unsupported(
                        "type syntax has no admitted lexical profile",
                    ));
                }
            }
            self.check_failure()
        })();
        if let Err(cause) = result {
            self.record(byte, cause);
        }
    }

    fn visit_type_array(&mut self, ty: &'ast syn::TypeArray) {
        if self.failure.is_some() {
            return;
        }
        let start = super::native_spans::expression::start(&ty.len, self.scopes.coordinates());
        let Some(byte) = self.anchor_result(start) else {
            return;
        };
        self.visit_type(&ty.elem);
        if self.failure.is_none()
            && let Err(cause) = self.const_expression(&ty.len, "array-length")
        {
            self.record(byte, cause);
        }
    }

    fn visit_generic_argument(&mut self, argument: &'ast syn::GenericArgument) {
        if self.failure.is_some() {
            return;
        }
        let start = super::native_spans::argument_start(argument, self.scopes.coordinates());
        let Some(byte) = self.anchor_result(start) else {
            return;
        };
        if let syn::GenericArgument::Const(expression) = argument {
            if let Err(cause) = self.const_expression(expression, "const-argument") {
                self.record(byte, cause);
            }
        } else if matches!(
            argument,
            syn::GenericArgument::Lifetime(_)
                | syn::GenericArgument::Type(_)
                | syn::GenericArgument::AssocConst(_)
                | syn::GenericArgument::AssocType(_)
                | syn::GenericArgument::Constraint(_)
        ) {
            syn::visit::visit_generic_argument(self, argument);
        } else {
            self.record(
                byte,
                Refusal::Unsupported("generic argument syntax has no admitted lexical profile"),
            );
        }
    }

    fn visit_assoc_const(&mut self, argument: &'ast syn::AssocConst) {
        if self.failure.is_some() {
            return;
        }
        let start =
            super::native_spans::expression::start(&argument.value, self.scopes.coordinates());
        let Some(byte) = self.anchor_result(start) else {
            return;
        };
        if let Some(generics) = &argument.generics {
            self.visit_angle_bracketed_generic_arguments(generics);
        }
        if self.failure.is_none()
            && let Err(cause) = self.const_expression(&argument.value, "associated-const")
        {
            self.record(byte, cause);
        }
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        if self.failure.is_some() {
            return;
        }
        let Some(byte) = self.anchor(block.brace_token.span.join()) else {
            return;
        };
        let prior = self.condition;
        self.condition = false;
        let result = (|| {
            self.scopes.enter_block(block)?;
            let id = self.next_block;
            self.next_block = self
                .next_block
                .checked_add(1)
                .ok_or(Refusal::Invariant("block identity overflow"))?;
            for statement in &block.stmts {
                self.meter.work(1)?;
                let span = self.statement_span(statement)?;
                if !self.inside(span) {
                    return Err(Refusal::Invariant("statement is outside its source owner"));
                }
                self.meter
                    .push_retained(&mut self.facts.statements, Statement { block: id, span })?;
                // A lone semicolon is a complete statement whose expression is
                // empty verbatim syntax; it has no content to classify.
                if !matches!(
                    statement,
                    syn::Stmt::Expr(syn::Expr::Verbatim(tokens), Some(_)) if tokens.is_empty()
                ) {
                    self.visit_stmt(statement);
                }
                self.check_failure()?;
            }
            self.scopes.leave()
        })();
        self.condition = prior;
        if let Err(cause) = result {
            self.record(byte, cause);
        }
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if self.failure.is_some() {
            return;
        }
        let first = super::native_spans::local_start(local, self.meter);
        let Some(byte) = self.anchor_result(first) else {
            return;
        };
        let result = (|| {
            attributes::list(&local.attrs, self.meter, self.scopes.coordinates())?;
            if let Some(init) = &local.init {
                self.visit_expr(&init.expr);
                if let Some((_, diverge)) = &init.diverge {
                    self.visit_expr(diverge);
                }
                self.check_failure()?;
            }
            let mut pending = Scratch::new(self.meter);
            self.pattern(&local.pat, &mut pending, None, None)?;
            self.activate(&pending)
        })();
        if let Err(cause) = result {
            self.record(byte, cause);
        }
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        if self.failure.is_some() {
            return;
        }
        let start = super::native_spans::pattern::arm_start(arm, self.scopes.coordinates())
            .map(|span| span.unwrap_or_else(proc_macro2::Span::call_site));
        let Some(byte) = self.anchor_result(start) else {
            return;
        };
        let result = (|| {
            attributes::list(&arm.attrs, self.meter, self.scopes.coordinates())?;
            self.scopes.enter_values()?;
            let prior = self.condition;
            self.arm_pattern(&arm.pat)?;
            self.condition = false;
            self.visit_expr(&arm.body);
            self.condition = prior;
            self.check_failure()?;
            self.scopes.leave()
        })();
        if let Err(cause) = result {
            self.record(byte, cause);
        }
    }

    fn visit_item(&mut self, item: &'ast syn::Item) {
        if self.failure.is_some() {
            return;
        }
        let ends = super::native_spans::statement::item_ends(item, self.scopes.coordinates());
        self.opaque_ends(ends);
    }

    fn visit_field_value(&mut self, field: &'ast syn::FieldValue) {
        if self.failure.is_some() {
            return;
        }
        let first = super::native_spans::field_start(field, self.meter);
        let Some(byte) = self.anchor_result(first) else {
            return;
        };
        if field.colon_token.is_some() {
            self.visit_expr(&field.expr);
            return;
        }
        let result = (|| {
            let syn::Member::Named(member) = &field.member else {
                return Err(Refusal::Invariant("a shorthand value has no named field"));
            };
            let syn::Expr::Path(path) = &field.expr else {
                return Err(Refusal::Invariant(
                    "a shorthand value is not an identifier path",
                ));
            };
            if path.qself.is_some()
                || path.path.leading_colon.is_some()
                || path.path.segments.len() != 1
            {
                return Err(Refusal::Invariant(
                    "a shorthand value is not one unqualified name",
                ));
            }
            let name = self.scopes.ident(&path.path.segments[0].ident)?;
            let fixed = self.scopes.ident(member)?;
            if !self.scopes.same_name(name, fixed)? {
                return Err(Refusal::Invariant(
                    "shorthand source and field names disagree",
                ));
            }
            if let Some(id) = self.scopes.value(name)? {
                self.annotate(
                    path.path.segments[0].ident.span(),
                    Occurrence {
                        binding: id,
                        event: Event::Use,
                        fixed: Some(fixed),
                    },
                )?;
            }
            Ok(())
        })();
        if let Err(cause) = result {
            self.record(byte, cause);
        }
    }

    fn visit_stmt_macro(&mut self, node: &'ast syn::StmtMacro) {
        if self.failure.is_some() {
            return;
        }
        let ends = super::native_spans::statement::stmt_macro_ends(node, self.scopes.coordinates())
            .map(Some);
        self.opaque_ends(ends);
    }
}

#[cfg(test)]
#[path = "collect_tests.rs"]
mod tests;
