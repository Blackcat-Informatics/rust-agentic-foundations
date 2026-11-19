// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The source syntax that can establish an inventory observation.

use proc_macro2::{Ident, Span};
use purrdf_jsonschema::ecma::MatchLimits;
use raf_source::SourceFile;
use syn::ext::IdentExt;
use syn::{BinOp, Expr, FnArg, Lit, Pat, Signature, Type, spanned::Spanned, visit::Visit};

use super::{BoundRole, Scan, UNCLASSIFIED, patterns};
use crate::Error;
use crate::detectors::{ReceiverMode, receiver_mode};

pub(super) struct Slots {
    size: Vec<bool>,
    receiver: Option<ReceiverMode>,
}

impl Slots {
    pub(super) fn accepts(
        &self,
        receiver: Option<ReceiverMode>,
        arity: usize,
        index: usize,
    ) -> bool {
        let offset = usize::from(receiver.is_some());
        (receiver.is_none() || self.receiver == receiver)
            && arity.saturating_add(offset) == self.size.len()
            && self.size.get(index + offset) == Some(&true)
    }

    pub(super) fn states(&self) -> usize {
        1usize.saturating_add(self.size.len())
    }
}

pub(super) fn collect(
    scan: &mut Scan,
    file: &SourceFile,
    syntax: &syn::File,
    limits: &mut MatchLimits,
) -> Result<(), Error> {
    patterns::charge(
        UNCLASSIFIED,
        limits,
        file.text.len().try_into().unwrap_or(u64::MAX),
    )?;
    let mut visitor = Syntax {
        scan,
        file,
        limits,
        error: None,
    };
    visitor.visit_file(syntax);
    visitor.error.map_or(Ok(()), Err)
}

struct Syntax<'a> {
    scan: &'a mut Scan,
    file: &'a SourceFile,
    limits: &'a mut MatchLimits,
    error: Option<Error>,
}

impl Syntax<'_> {
    fn record(&mut self, token: Span, context: Span, role: BoundRole) {
        if self.error.is_none() {
            self.error = self
                .scan
                .site(
                    &self.file.path,
                    token.into(),
                    context.byte_range(),
                    role,
                    self.limits,
                )
                .err();
        }
    }

    fn named(&mut self, name: &Ident, ty: &Type, value: Option<&Expr>, context: Span) {
        if has_word(name, LIMIT_WORDS) && (number_type(ty) || value.is_some_and(number)) {
            self.record(name.span(), context, BoundRole::Named);
            if self.error.is_none()
                && let Err(error) = self.scan.retain(1, self.limits)
            {
                self.error = Some(error);
            }
            if self.error.is_none() {
                self.scan.declarations.insert(
                    (self.file.path.clone(), context.byte_range().start),
                    name.span().byte_range().start,
                );
            }
        }
    }

    fn signature(&mut self, item: Span, signature: &Signature) {
        if self.error.is_some() || signature.variadic.is_some() {
            return;
        }
        let mut size = Vec::with_capacity(signature.inputs.len());
        for argument in &signature.inputs {
            let (attributes, named) = match argument {
                FnArg::Receiver(receiver) => (&receiver.attrs, false),
                FnArg::Typed(typed) => (
                    &typed.attrs,
                    matches!(typed.pat.as_ref(), Pat::Ident(binding) if binding.subpat.is_none() && size_name(&binding.ident)),
                ),
            };
            if attributes.iter().any(conditional) {
                return;
            }
            size.push(named);
        }
        if !size.contains(&true) {
            return;
        }
        let slots = Slots {
            size,
            receiver: receiver_mode(signature),
        };
        if let Err(error) = self.scan.retain(slots.states(), self.limits) {
            self.error = Some(error);
            return;
        }
        self.scan
            .signatures
            .insert((self.file.path.clone(), item.byte_range().start), slots);
    }
}

impl<'ast> Visit<'ast> for Syntax<'_> {
    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        self.named(&node.ident, &node.ty, Some(&node.expr), node.span());
        syn::visit::visit_item_const(self, node);
    }

    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
        self.named(&node.ident, &node.ty, Some(&node.expr), node.span());
        syn::visit::visit_item_static(self, node);
    }

    fn visit_impl_item_const(&mut self, node: &'ast syn::ImplItemConst) {
        self.named(&node.ident, &node.ty, Some(&node.expr), node.span());
        syn::visit::visit_impl_item_const(self, node);
    }

    fn visit_trait_item_const(&mut self, node: &'ast syn::TraitItemConst) {
        self.named(
            &node.ident,
            &node.ty,
            node.default.as_ref().map(|(_, expr)| expr),
            node.span(),
        );
        syn::visit::visit_trait_item_const(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.signature(node.span(), &node.sig);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.signature(node.span(), &node.sig);
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.signature(node.span(), &node.sig);
        syn::visit::visit_trait_item_fn(self, node);
    }

    fn visit_foreign_item_fn(&mut self, node: &'ast syn::ForeignItemFn) {
        self.signature(node.span(), &node.sig);
        syn::visit::visit_foreign_item_fn(self, node);
    }
}

pub(in crate::gate) fn comparisons(node: &syn::ExprBinary) -> impl Iterator<Item = &Expr> {
    let compares = matches!(
        node.op,
        BinOp::Eq(_) | BinOp::Ne(_) | BinOp::Lt(_) | BinOp::Le(_) | BinOp::Gt(_) | BinOp::Ge(_)
    );
    [(&*node.left, &*node.right), (&*node.right, &*node.left)]
        .into_iter()
        .filter_map(move |(measure, bound)| (compares && length_or_count(measure)).then_some(bound))
}

pub(in crate::gate) fn limiting_method(node: &syn::ExprMethodCall) -> impl Iterator<Item = &Expr> {
    let name = node.method.unraw().to_string();
    let supported = matches!(
        (name.as_str(), node.args.len()),
        ("truncate" | "take" | "min", 1) | ("clamp", 2)
    );
    node.args.iter().filter(move |_| supported)
}

pub(in crate::gate) fn limiting_call(node: &syn::ExprCall) -> impl Iterator<Item = &Expr> {
    let first = match ungroup(&node.func) {
        Expr::Path(path) => path.path.segments.last().map_or(node.args.len(), |member| {
            match (member.ident.unraw().to_string().as_str(), node.args.len()) {
                ("min", 2) | ("truncate" | "take", 1) => 0,
                ("clamp", 3) => 1,
                ("truncate" | "take", 2)
                    if path.path.segments.len() > 1 || path.qself.is_some() =>
                {
                    1
                }
                _ => node.args.len(),
            }
        }),
        _ => node.args.len(),
    };
    node.args.iter().skip(first)
}

const LIMIT_WORDS: &[&str] = &["max", "limit", "cap", "budget", "bound", "capacity"];
const SIZE_WORDS: &[&str] = &[
    "size", "len", "length", "count", "capacity", "limit", "max", "cap", "budget", "bound",
];
const MEASURE_WORDS: &[&str] = &["len", "length", "count"];

fn has_word(name: &Ident, words: &[&str]) -> bool {
    let name = name.to_string();
    name.strip_prefix("r#")
        .unwrap_or(&name)
        .split('_')
        .any(|part| words.iter().any(|word| part.eq_ignore_ascii_case(word)))
}

fn size_name(name: &Ident) -> bool {
    has_word(name, SIZE_WORDS)
}

fn conditional(attribute: &syn::Attribute) -> bool {
    attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr")
}

fn number_type(ty: &Type) -> bool {
    const NUMBERS: &[&str] = &[
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
        "f32", "f64",
    ];
    match ty {
        Type::Group(group) => number_type(&group.elem),
        Type::Paren(paren) => number_type(&paren.elem),
        Type::Path(path) if path.qself.is_none() => {
            let mut segments = path.path.segments.iter();
            match (
                segments.next(),
                segments.next(),
                segments.next(),
                segments.next(),
            ) {
                (Some(last), None, None, None) | (Some(_), Some(_), Some(last), None)
                    if (path.path.segments.len() == 1
                        || (path.path.segments[1].ident == "primitive"
                            && (path.path.segments[0].ident == "core"
                                || path.path.segments[0].ident == "std"))) =>
                {
                    NUMBERS.iter().any(|name| last.ident == *name)
                }
                _ => false,
            }
        }
        _ => false,
    }
}

fn number(expr: &Expr) -> bool {
    integer(expr).is_some()
        || matches!(ungroup(expr), Expr::Lit(literal) if matches!(literal.lit, Lit::Float(_)))
}

pub(in crate::gate) fn integer(expr: &Expr) -> Option<Span> {
    let expr = ungroup(expr);
    match expr {
        Expr::Lit(literal) if matches!(literal.lit, Lit::Int(_)) => Some(expr.span()),
        Expr::Unary(unary)
            if matches!(unary.op, syn::UnOp::Neg(_))
                && matches!(ungroup(&unary.expr), Expr::Lit(literal) if matches!(literal.lit, Lit::Int(_))) =>
        {
            Some(expr.span())
        }
        _ => None,
    }
}

pub(in crate::gate) fn ungroup(mut expr: &Expr) -> &Expr {
    loop {
        expr = match expr {
            Expr::Paren(paren) => &paren.expr,
            Expr::Group(group) => &group.expr,
            _ => return expr,
        };
    }
}

fn length_or_count(expr: &Expr) -> bool {
    match ungroup(expr) {
        Expr::MethodCall(call) => call.args.is_empty() && has_word(&call.method, MEASURE_WORDS),
        Expr::Field(field) => {
            matches!(&field.member, syn::Member::Named(name) if has_word(name, MEASURE_WORDS))
        }
        Expr::Path(path) => path
            .path
            .segments
            .last()
            .is_some_and(|part| has_word(&part.ident, MEASURE_WORDS)),
        Expr::Call(call) => {
            matches!(ungroup(&call.func), Expr::Path(path) if path.path.segments.last().is_some_and(|part| has_word(&part.ident, MEASURE_WORDS)))
        }
        _ => false,
    }
}
