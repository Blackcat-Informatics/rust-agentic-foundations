// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Syntactic byte-layout detectors, shared by the production Census gate.

use syn::visit::Visit;

use super::RawHit;
use proc_macro2::Span;
use std::collections::{BTreeMap, BTreeSet};

/// A field written after its own length.
pub(crate) const LENGTH_PREFIX_FRAME: &str = "length-prefix-frame";
/// A fixed-width integer read or written through a range-indexed sub-slice.
pub(crate) const LE_SLICE_INT: &str = "le-slice-int";
/// Rounding up to a power of two by mask.
pub(crate) const ALIGN_UP_MASK: &str = "align-up-mask";
/// Ceiling division by hand.
pub(crate) const DIV_CEIL_BY_HAND: &str = "div-ceil-by-hand";
/// The first-mismatch index by hand: an XOR of two loaded words and a zero
/// count.
pub(crate) const XOR_FIRST_MISMATCH: &str = "xor-first-mismatch";

/// Every rule this module computes.
pub(crate) const RULES: [&str; 5] = [
    LENGTH_PREFIX_FRAME,
    LE_SLICE_INT,
    ALIGN_UP_MASK,
    DIV_CEIL_BY_HAND,
    XOR_FIRST_MISMATCH,
];

/// The word loads `xor-first-mismatch` reads.
const WORD_LOADS: [&str; 3] = ["from_le_bytes", "from_ne_bytes", "from_be_bytes"];

/// What `xor-first-mismatch` has seen of one function body.
#[derive(Clone, Default)]
struct MismatchFrame {
    /// The locals bound to a word load.
    loads: Vec<String>,
    /// Local values that retain an XOR of two loaded words.
    xors: BTreeMap<String, Span>,
    /// Assignments invalidate a previously tracked value across nested blocks.
    writes: BTreeSet<String>,
}

/// The formatting macros whose arguments `length-prefix-frame` reads.
const FORMAT_MACROS: [&str; 4] = ["format", "format_args", "write", "writeln"];

/// A stable spelling of the simple place expressions a frame writes: a path, a
/// field, a no-argument method call on one (`s.as_bytes()`), a literal index,
/// through parentheses. `None` for anything else.
fn place(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Path(path) if path.qself.is_none() => Some(
            path.path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::"),
        ),
        syn::Expr::Field(field) => {
            let base = place(&field.base)?;
            let member = match &field.member {
                syn::Member::Named(name) => name.to_string(),
                syn::Member::Unnamed(index) => index.index.to_string(),
            };
            Some(format!("{base}.{member}"))
        }
        syn::Expr::MethodCall(call) if call.args.is_empty() && call.turbofish.is_none() => {
            Some(format!("{}.{}()", place(&call.receiver)?, call.method))
        }
        syn::Expr::Index(index) => match index.index.as_ref() {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(literal),
                ..
            }) => Some(format!(
                "{}[{}]",
                place(&index.expr)?,
                literal.base10_digits()
            )),
            _ => None,
        },
        syn::Expr::Paren(inner) => place(&inner.expr),
        syn::Expr::Group(inner) => place(&inner.expr),
        _ => None,
    }
}

/// The place an argument writes as bytes: `E`, `&E`, `E.as_bytes()`,
/// `E.as_ref()` or `&E[..]`, spelt as [`place`] spells `E`.
fn written(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Reference(reference) => written(&reference.expr),
        syn::Expr::Paren(inner) => written(&inner.expr),
        syn::Expr::MethodCall(call)
            if call.args.is_empty() && (call.method == "as_bytes" || call.method == "as_ref") =>
        {
            place(&call.receiver)
        }
        syn::Expr::Index(index) if matches!(index.index.as_ref(), syn::Expr::Range(range) if range.start.is_none() && range.end.is_none()) => {
            place(&index.expr)
        }
        other => place(other),
    }
}

/// The place `expr` is the bare length of: `E.len()`.
fn bare_length(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::MethodCall(call) if call.method == "len" && call.args.is_empty() => {
            place(&call.receiver)
        }
        syn::Expr::Paren(inner) => bare_length(&inner.expr),
        _ => None,
    }
}

/// The wrappers a length passes through on its way into a writer: byte
/// conversion, a fallible widening's unwrapping, a reference, parentheses.
const LENGTH_WRAPPERS: [&str; 9] = [
    "to_le_bytes",
    "to_be_bytes",
    "expect",
    "unwrap",
    "unwrap_or",
    "unwrap_or_else",
    "map_err",
    "ok_or",
    "ok_or_else",
];

/// The place `expr` is a 64-bit length of — `E.len()` widened by `as u64`,
/// `u64::try_from(E.len())` or `u64::from(E.len())`, under any of
/// [`LENGTH_WRAPPERS`] — or the place a local bound to such a length (in
/// `bound`) stands for. A bare `E.len()` handed to a writer is not read: the
/// writer's width and encoding are out of sight (a capacity check, a varint).
fn length_of(expr: &syn::Expr, bound: &[(String, String)]) -> Option<String> {
    match expr {
        syn::Expr::Reference(inner) => length_of(&inner.expr, bound),
        syn::Expr::Paren(inner) => length_of(&inner.expr, bound),
        syn::Expr::Try(inner) => length_of(&inner.expr, bound),
        syn::Expr::MethodCall(call)
            if LENGTH_WRAPPERS.contains(&call.method.to_string().as_str()) =>
        {
            length_of(&call.receiver, bound)
        }
        syn::Expr::Cast(cast) => match cast.ty.as_ref() {
            syn::Type::Path(ty) if ty.path.is_ident("u64") => bare_length(&cast.expr),
            _ => None,
        },
        syn::Expr::Call(call) => {
            let syn::Expr::Path(function) = call.func.as_ref() else {
                return None;
            };
            let segments: Vec<String> = function
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();
            match segments.as_slice() {
                [.., ty, method] if ty == "u64" && (method == "try_from" || method == "from") => {
                    bare_length(call.args.first()?)
                }
                _ => None,
            }
        }
        syn::Expr::Path(path) => {
            let ident = path.path.get_ident()?.to_string();
            bound
                .iter()
                .rev()
                .find(|(name, _)| *name == ident)
                .map(|(_, place)| place.clone())
        }
        _ => None,
    }
}

/// Whether a writer's name says it writes a self-delimiting integer encoding
/// (a varint, LEB128, a CBOR head) or a hasher's native integer, not a
/// fixed-width length prefix.
fn is_other_encoding(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    ["varint", "leb", "cbor"]
        .iter()
        .any(|word| name.contains(word))
        || (name.starts_with("write_") && name != "write_all" && name != "write")
}

/// A write statement: the sink it writes to, the writer's name and its
/// arguments.
struct Write<'a> {
    sink: String,
    writer: String,
    arguments: Vec<&'a syn::Expr>,
}

/// The write a statement makes: a method call whose receiver is a place
/// (arguments are the call's), or a free call whose first argument is a place
/// (arguments are the rest). The statement may end in `;` or `?`.
fn write_of(statement: &syn::Stmt) -> Option<Write<'_>> {
    let syn::Stmt::Expr(expr, _) = statement else {
        return None;
    };
    let expr = match expr {
        syn::Expr::Try(inner) => inner.expr.as_ref(),
        other => other,
    };
    match expr {
        syn::Expr::MethodCall(call) => Some(Write {
            sink: place(&call.receiver)?,
            writer: call.method.to_string(),
            arguments: call.args.iter().collect(),
        }),
        syn::Expr::Call(call) => {
            let syn::Expr::Path(function) = call.func.as_ref() else {
                return None;
            };
            let mut arguments = call.args.iter();
            let first = arguments.next()?;
            Some(Write {
                sink: written(first)?,
                writer: function.path.segments.last()?.ident.to_string(),
                arguments: arguments.collect(),
            })
        }
        _ => None,
    }
}

/// Whether two writes reach one sink: the same place, or a place and a field
/// of it (`self.u64(len)` then `self.buf.extend_from_slice(bytes)`).
fn same_sink(first: &str, second: &str) -> bool {
    let nested = |outer: &str, inner: &str| {
        inner
            .strip_prefix(outer)
            .is_some_and(|rest| rest.starts_with('.'))
    };
    first == second || nested(first, second) || nested(second, first)
}

/// A `let` binding one ident to a length of a place (see [`length_of`]): its
/// ident and the place.
fn length_binding(statement: &syn::Stmt, bound: &[(String, String)]) -> Option<(String, String)> {
    let syn::Stmt::Local(local) = statement else {
        return None;
    };
    let syn::Pat::Ident(ident) = &local.pat else {
        return None;
    };
    let place = length_of(&local.init.as_ref()?.expr, bound)?;
    Some((ident.ident.to_string(), place))
}

/// The visitor for the byte-layout rules.
struct Layout {
    hits: Vec<RawHit>,
    mismatch: Vec<MismatchFrame>,
}

impl Layout {
    fn hit(&mut self, rule: &'static str, span: Span, message: String) {
        self.hits.push(RawHit {
            rule,
            byte: span.byte_range().start,
            line: span.start().line,
            message,
        });
    }

    /// Keep load and XOR bindings local to their function.
    fn function(&mut self, walk: impl FnOnce(&mut Self)) {
        self.mismatch.push(MismatchFrame::default());
        walk(self);
        self.mismatch.pop();
    }

    /// Whether `expr` is a word load: `uN::from_le_bytes(…)` (or `ne`/`be`),
    /// through parentheses, or a local of this function bound to one.
    fn is_word_load(&self, expr: &syn::Expr) -> bool {
        match expr {
            syn::Expr::Paren(inner) => self.is_word_load(&inner.expr),
            syn::Expr::Call(call) => is_word_load_call(call),
            syn::Expr::Path(path) => path.path.get_ident().is_some_and(|ident| {
                self.mismatch
                    .last()
                    .is_some_and(|frame| frame.loads.contains(&ident.to_string()))
            }),
            _ => false,
        }
    }

    fn word_xor(&self, expr: &syn::Expr) -> Option<Span> {
        match expr {
            syn::Expr::Paren(inner) => self.word_xor(&inner.expr),
            syn::Expr::Binary(binary)
                if matches!(binary.op, syn::BinOp::BitXor(_))
                    && self.is_word_load(&binary.left)
                    && self.is_word_load(&binary.right) =>
            {
                use syn::spanned::Spanned as _;
                Some(binary.op.span())
            }
            syn::Expr::Path(path) => self
                .mismatch
                .last()?
                .xors
                .get(&path.path.get_ident()?.to_string())
                .copied(),
            _ => None,
        }
    }

    fn invalidate(&mut self, expr: &syn::Expr) {
        if let Some(name) = place(expr)
            && let Some(frame) = self.mismatch.last_mut()
        {
            frame.loads.retain(|bound| *bound != name);
            frame.xors.remove(&name);
            frame.writes.insert(name);
        }
    }

    /// `length-prefix-frame` over one block's statements.
    fn frames(&mut self, statements: &[syn::Stmt]) {
        let mut bound: Vec<(String, String)> = Vec::new();
        for (index, statement) in statements.iter().enumerate() {
            if let Some(binding) = length_binding(statement, &bound) {
                bound.push(binding);
                continue;
            }
            let Some(first) = write_of(statement) else {
                continue;
            };
            let Some(second) = statements.get(index + 1).and_then(write_of) else {
                continue;
            };
            if !same_sink(&first.sink, &second.sink) || is_other_encoding(&first.writer) {
                continue;
            }
            let prefixed: Vec<String> = first
                .arguments
                .iter()
                .filter_map(|argument| length_of(argument, &bound))
                .collect();
            if let Some(field) = second
                .arguments
                .iter()
                .filter_map(|argument| written(argument))
                .find(|field| prefixed.contains(field))
            {
                let line = statement_line(statement);
                self.hit(
                    LENGTH_PREFIX_FRAME,
                    line,
                    format!("`{field}` is written after its own length: a hand-written frame"),
                );
            }
        }
    }
}

/// The first line of a statement.
fn statement_line(statement: &syn::Stmt) -> Span {
    use syn::spanned::Spanned as _;
    statement.span()
}

/// Whether `expr` is a range-indexed sub-slice `x[a..b]` (either bound
/// present).
fn is_range_slice(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Index(index) => matches!(
            index.index.as_ref(),
            syn::Expr::Range(range) if range.start.is_some() || range.end.is_some()
        ),
        syn::Expr::Paren(inner) => is_range_slice(&inner.expr),
        syn::Expr::Reference(reference) => is_range_slice(&reference.expr),
        _ => false,
    }
}

/// Whether `expr` converts a range-indexed sub-slice to an array:
/// `x[a..b].try_into()` under any `expect`/`unwrap`/`?`/`map_err` wrapping.
fn slice_array(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::MethodCall(call) if call.method == "try_into" => is_range_slice(&call.receiver),
        syn::Expr::MethodCall(call) => slice_array(&call.receiver),
        syn::Expr::Try(inner) => slice_array(&inner.expr),
        syn::Expr::Paren(inner) => slice_array(&inner.expr),
        syn::Expr::Call(call) => {
            // `<[u8; N]>::try_from(&x[a..b])` spelt as a call.
            let names_try_from = matches!(call.func.as_ref(), syn::Expr::Path(path)
                if path.path.segments.last().is_some_and(|segment| segment.ident == "try_from"));
            names_try_from && call.args.first().is_some_and(is_range_slice)
        }
        _ => false,
    }
}

/// The integer type names a `uN::from_le_bytes`/`to_le_bytes` rule accepts.
const INTEGERS: [&str; 8] = ["u16", "u32", "u64", "u128", "i16", "i32", "i64", "i128"];

impl<'ast> Visit<'ast> for Layout {
    fn visit_block(&mut self, node: &'ast syn::Block) {
        let bindings = self.mismatch.last().cloned();
        self.frames(&node.stmts);
        syn::visit::visit_block(self, node);
        if let (Some(mut previous), Some(current)) = (bindings, self.mismatch.last_mut()) {
            for name in &current.writes {
                previous.loads.retain(|bound| bound != name);
                previous.xors.remove(name);
            }
            previous.writes.extend(current.writes.iter().cloned());
            *current = previous;
        }
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let name = node
            .path
            .segments
            .last()
            .map_or_default(|segment| segment.ident.to_string());
        if FORMAT_MACROS.contains(&name.as_str())
            && let Ok(arguments) = node.parse_body_with(
                syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
            )
        {
            let arguments: Vec<_> = arguments.into_iter().collect();
            if let Some((length, data)) = super::format::first_pair(&name, &arguments)
                && let Some(field) = bare_length(length)
                && written(data).as_ref() == Some(&field)
            {
                self.hit(
                    LENGTH_PREFIX_FRAME,
                    node.path.segments[0].ident.span(),
                    format!("`{field}` is formatted after its own length: a decimal length prefix"),
                );
            }
        }
        syn::visit::visit_macro(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(function) = node.func.as_ref() {
            let segments: Vec<String> = function
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();
            if let [.., integer, method] = segments.as_slice()
                && method == "from_le_bytes"
                && INTEGERS.contains(&integer.as_str())
                && node.args.first().is_some_and(slice_array)
            {
                self.hit(
                    LE_SLICE_INT,
                    integer_line(function),
                    format!(
                        "`{integer}::from_le_bytes` over a range-indexed sub-slice repeats the integer width"
                    ),
                );
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if (node.method == "trailing_zeros" || node.method == "leading_zeros")
            && node.args.is_empty()
            && let Some(span) = self.word_xor(&node.receiver)
        {
            self.hit(
                XOR_FIRST_MISMATCH,
                span,
                "a zero count on an XOR of loaded words computes the first mismatch by hand".into(),
            );
        }
        if node.method == "copy_from_slice"
            && is_range_slice(&node.receiver)
            && node.args.first().is_some_and(|argument| {
                let argument = match argument {
                    syn::Expr::Reference(reference) => reference.expr.as_ref(),
                    other => other,
                };
                matches!(argument, syn::Expr::MethodCall(call) if call.method == "to_le_bytes")
            })
        {
            self.hit(
                LE_SLICE_INT,
                node.method.span(),
                "a little-endian integer written through a range-indexed sub-slice repeats the integer width"
                    .to_owned(),
            );
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        if matches!(
            node.op,
            syn::BinOp::AddAssign(_)
                | syn::BinOp::SubAssign(_)
                | syn::BinOp::MulAssign(_)
                | syn::BinOp::DivAssign(_)
                | syn::BinOp::RemAssign(_)
                | syn::BinOp::BitXorAssign(_)
                | syn::BinOp::BitAndAssign(_)
                | syn::BinOp::BitOrAssign(_)
                | syn::BinOp::ShlAssign(_)
                | syn::BinOp::ShrAssign(_)
        ) {
            self.invalidate(&node.left);
        }
        match &node.op {
            syn::BinOp::BitAnd(token) if alignment_rounding(&node.left, &node.right) => {
                self.hit(
                    ALIGN_UP_MASK,
                    token.span,
                    "alignment rounding written as an add-and-mask".to_owned(),
                );
            }
            syn::BinOp::Div(token) if is_biased_by(&node.left, &node.right) => {
                self.hit(
                    DIV_CEIL_BY_HAND,
                    token.span,
                    "ceiling division written as an add-and-divide".to_owned(),
                );
            }
            _ => {}
        }
        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        let pattern = match &node.pat {
            syn::Pat::Type(typed) => typed.pat.as_ref(),
            other => other,
        };
        if let syn::Pat::Ident(binding) = pattern {
            let loaded = node
                .init
                .as_ref()
                .is_some_and(|init| self.is_word_load(&init.expr));
            let xor = node
                .init
                .as_ref()
                .and_then(|init| self.word_xor(&init.expr));
            if let Some(frame) = self.mismatch.last_mut() {
                let name = binding.ident.to_string();
                frame.writes.remove(&name);
                frame.loads.retain(|bound| *bound != name);
                frame.xors.remove(&name);
                if loaded {
                    frame.loads.push(name.clone());
                }
                if let Some(span) = xor {
                    frame.xors.insert(name, span);
                }
            }
        }
        syn::visit::visit_local(self, node);
    }

    fn visit_expr_assign(&mut self, node: &'ast syn::ExprAssign) {
        self.invalidate(&node.left);
        syn::visit::visit_expr_assign(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.function(|this| syn::visit::visit_item_fn(this, node));
    }
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.function(|this| syn::visit::visit_impl_item_fn(this, node));
    }
    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.function(|this| syn::visit::visit_trait_item_fn(this, node));
    }
}

/// Whether `call` is `uN::from_le_bytes(…)`, `from_ne_bytes` or
/// `from_be_bytes` on an integer type.
fn is_word_load_call(call: &syn::ExprCall) -> bool {
    let syn::Expr::Path(function) = call.func.as_ref() else {
        return false;
    };
    let segments: Vec<String> = function
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    matches!(
        segments.as_slice(),
        [.., integer, method]
            if WORD_LOADS.contains(&method.as_str()) && INTEGERS.contains(&integer.as_str())
    )
}

/// The line of a call's path.
fn integer_line(function: &syn::ExprPath) -> Span {
    function.path.segments[0].ident.span()
}

fn unparen(mut expr: &syn::Expr) -> &syn::Expr {
    while let syn::Expr::Paren(inner) = expr {
        expr = &inner.expr;
    }
    expr
}

#[derive(PartialEq, Eq)]
enum Mask {
    Literal(u128),
    Alignment(String),
}

fn mask(expr: &syn::Expr) -> Option<Mask> {
    match unparen(expr) {
        syn::Expr::Binary(binary)
            if matches!(binary.op, syn::BinOp::Sub(_)) && is_one(&binary.right) =>
        {
            place(&binary.left).map(Mask::Alignment)
        }
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(value),
            ..
        }) => {
            let value = crate::normalize::parse_integer(&value.to_string())?.value?;
            (value != 0 && value.checked_add(1).is_some_and(u128::is_power_of_two))
                .then_some(Mask::Literal(value))
        }
        _ => None,
    }
}

/// An add-and-mask must add the same low bits that the complement removes.
fn alignment_rounding(numerator: &syn::Expr, complement: &syn::Expr) -> bool {
    let syn::Expr::Unary(complement) = unparen(complement) else {
        return false;
    };
    if !matches!(complement.op, syn::UnOp::Not(_)) {
        return false;
    }
    let Some(expected) = mask(&complement.expr) else {
        return false;
    };
    let syn::Expr::Binary(numerator) = unparen(numerator) else {
        return false;
    };
    match numerator.op {
        syn::BinOp::Add(_) => mask(&numerator.right).is_some_and(|bias| bias == expected),
        syn::BinOp::Sub(_) if is_one(&numerator.right) => {
            let syn::Expr::Binary(add) = unparen(&numerator.left) else {
                return false;
            };
            matches!(add.op, syn::BinOp::Add(_))
                && place(&add.right)
                    .map(Mask::Alignment)
                    .is_some_and(|bias| bias == expected)
        }
        _ => false,
    }
}

/// Whether `numerator` is `(n + d - 1)` for the divisor `divisor` = `d`.
fn is_biased_by(numerator: &syn::Expr, divisor: &syn::Expr) -> bool {
    let syn::Expr::Paren(numerator) = numerator else {
        return false;
    };
    let syn::Expr::Binary(minus) = numerator.expr.as_ref() else {
        return false;
    };
    if !matches!(minus.op, syn::BinOp::Sub(_)) || !is_one(&minus.right) {
        return false;
    }
    let syn::Expr::Binary(plus) = minus.left.as_ref() else {
        return false;
    };
    matches!(plus.op, syn::BinOp::Add(_))
        && place(divisor).is_some()
        && place(&plus.right) == place(divisor)
}

/// Whether `expr` is the integer literal `1`.
fn is_one(expr: &syn::Expr) -> bool {
    matches!(expr, syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(value), .. }) if crate::normalize::parse_integer(&value.to_string()).and_then(|value| value.value) == Some(1))
}

/// Collect every syntactic hit; source ownership and shipping status belong to the gate.
pub(super) fn collect(parsed: &syn::File) -> Vec<RawHit> {
    let mut visitor = Layout {
        hits: Vec::new(),
        mismatch: Vec::new(),
    };
    visitor.visit_file(parsed);
    visitor.hits
}
