// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The golden registry: every fixture of the normal form, in one place.
//!
//! A fixture is a body, the parameter names of its function and an id. A pair
//! says how two fixtures must compare: equal from some level of the tower, or
//! distinct. The golden test pins the fingerprints of every fixture and fails
//! when a fixture is not pinned or a pin has no fixture, and the form tests
//! check every pair. The `goldens` example prints the table in the pinned
//! form, so a new normal form is a paste.

#![allow(
    dead_code,
    reason = "each binary that includes the registry uses a different part of it"
)]

use std::fmt::Write as _;

use raf_census::{CensusOptions, Level, Normalizer, Slice, VALUE_ADAPTERS};

/// One body with the parameter names of its function.
pub(crate) struct Fixture {
    pub(crate) id: String,
    pub(crate) params: Vec<String>,
    pub(crate) text: String,
    /// The structural tokens the fixture must have, when its name says so.
    pub(crate) tokens: Option<usize>,
}

impl Fixture {
    pub(crate) fn slice(&self) -> Slice<'_> {
        Slice::new("fixture.rs", 1, &self.text).params(self.params.iter().cloned())
    }
}

/// How two fixtures must compare.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Expect {
    /// Equal from this level on (the finest level at which they are equal).
    Equal(Level),
    /// Different even in the value form.
    Distinct,
}

pub(crate) struct Pair {
    pub(crate) id: String,
    pub(crate) a: String,
    pub(crate) b: String,
    pub(crate) expect: Expect,
}

pub(crate) struct Registry {
    pub(crate) fixtures: Vec<Fixture>,
    pub(crate) pairs: Vec<Pair>,
}

impl Registry {
    fn single(&mut self, id: &str, params: &[&str], text: &str, tokens: Option<usize>) {
        assert!(
            self.fixtures.iter().all(|fixture| fixture.id != id),
            "duplicate fixture id {id}"
        );
        self.fixtures.push(Fixture {
            id: id.to_owned(),
            params: params.iter().map(|p| (*p).to_owned()).collect(),
            text: text.to_owned(),
            tokens,
        });
    }

    /// Two fixtures `<name>.a` and `<name>.b` and the pair that relates them.
    fn pair(&mut self, name: &str, a: (&[&str], &str), b: (&[&str], &str), expect: Expect) {
        let (a_id, b_id) = (format!("{name}.a"), format!("{name}.b"));
        self.single(&a_id, a.0, a.1, None);
        self.single(&b_id, b.0, b.1, None);
        self.pairs.push(Pair {
            id: name.to_owned(),
            a: a_id,
            b: b_id,
            expect,
        });
    }

    fn pair_sized(
        &mut self,
        name: &str,
        a: (&[&str], &str, usize),
        b: (&[&str], &str, usize),
        expect: Expect,
    ) {
        let (a_id, b_id) = (format!("{name}.a"), format!("{name}.b"));
        self.single(&a_id, a.0, a.1, Some(a.2));
        self.single(&b_id, b.0, b.1, Some(b.2));
        self.pairs.push(Pair {
            id: name.to_owned(),
            a: a_id,
            b: b_id,
            expect,
        });
    }
}

/// `head` (one statement of `head_tokens` tokens) padded with `a;` and `-a;`
/// statements to exactly `total` tokens.
fn padded(head: &str, head_tokens: usize, total: usize) -> String {
    let rest = total - head_tokens;
    let (threes, twos) = if rest.is_multiple_of(2) {
        (0, rest / 2)
    } else {
        (1, (rest - 3) / 2)
    };
    format!("{head} {}{}", "a; ".repeat(twos), "-a; ".repeat(threes))
}

/// A forwarder `f(a, 1, 2, ...)` of exactly `total` tokens.
fn forwarder(total: usize) -> String {
    let tail = if total.is_multiple_of(2) { "" } else { "?" };
    let args = (total - 3 - tail.len()).div_ceil(2);
    let mut text = String::from("f(a");
    for n in 1..args {
        write!(text, ", {n}").expect("a string takes a write");
    }
    format!("{text}){tail}")
}

/// Structural form: locals, parameters and lifetimes collapse; everything a
/// body names for others stays.
fn structural(r: &mut Registry) {
    use Expect::{Distinct, Equal};
    use Level::{LiteralTier, Shim};
    let walk = |param: &str, total: &str, row: &str, call: &str| {
        format!("let mut {total} = 0; for {row} in {param} {{ {total} += {call}({row}); }} {total}")
    };
    r.pair(
        "structural_renamed_locals",
        (&["items"], &walk("items", "sum", "item", "weigh")),
        (&["rows"], &walk("rows", "acc", "row", "weigh")),
        Equal(LiteralTier),
    );
    r.pair(
        "structural_renamed_params",
        (&["items"], &walk("items", "sum", "item", "weigh")),
        (&["rows"], &walk("rows", "sum", "item", "weigh")),
        Equal(Shim),
    );
    r.pair(
        "structural_renamed_lifetimes",
        (&["v"], "let f = |x: &'a u8| -> &'a u8 { x }; f(v)"),
        (&["v"], "let f = |x: &'b u8| -> &'b u8 { x }; f(v)"),
        Equal(LiteralTier),
    );
    r.pair(
        "structural_different_function",
        (&["items"], &walk("items", "sum", "item", "weigh")),
        (&["items"], &walk("items", "sum", "item", "measure")),
        Distinct,
    );
    r.pair(
        "structural_different_method",
        (&["v"], "let n = v.len(); n + 1"),
        (&["v"], "let n = v.count(); n + 1"),
        Distinct,
    );
    r.pair(
        "structural_different_field",
        (&["p"], "let n = p.width; n + 1"),
        (&["p"], "let n = p.height; n + 1"),
        Distinct,
    );
    r.pair(
        "structural_different_type",
        (&[], "let n = Foo::new(); n.run()"),
        (&[], "let n = Bar::new(); n.run()"),
        Distinct,
    );
    r.pair(
        "structural_local_type_declared",
        (&[], "struct Park { n: u8 } Park { n: 1 }"),
        (&[], "struct ParkWaker { n: u8 } ParkWaker { n: 1 }"),
        Equal(LiteralTier),
    );
    r.pair(
        "structural_tuple_field_kept",
        (&["p"], "p.0 + p.1"),
        (&["p"], "p.1 + p.0"),
        Distinct,
    );
}

/// Binders: one pair per form of binder, whose two sides bind different names
/// and are first equal at the literal tier; and the names no one binds, which
/// are items and stay.
fn binders(r: &mut Registry) {
    use Expect::Equal;
    use Level::LiteralTier;
    r.pair(
        "binder_let",
        (&["x"], "let a = f(x); g(a)"),
        (&["x"], "let b = f(x); g(b)"),
        Equal(LiteralTier),
    );
    r.pair(
        "binder_let_tuple",
        (&["x"], "let (a, b) = pair(x); g(a, b)"),
        (&["x"], "let (c, d) = pair(x); g(c, d)"),
        Equal(LiteralTier),
    );
    r.pair(
        "binder_let_struct",
        (&["x"], "let P { w: a, h: b } = x; g(a, b)"),
        (&["x"], "let P { w: c, h: d } = x; g(c, d)"),
        Equal(LiteralTier),
    );
    r.pair(
        "binder_closure_param",
        (&["x"], "x.map(|a| a + 1)"),
        (&["x"], "x.map(|b| b + 1)"),
        Equal(LiteralTier),
    );
    r.pair(
        "binder_for",
        (&["x"], "for a in x { g(a); }"),
        (&["x"], "for b in x { g(b); }"),
        Equal(LiteralTier),
    );
    r.pair(
        "binder_match_arm",
        (&["x"], "match x { Some(a) => g(a), None => 0 }"),
        (&["x"], "match x { Some(b) => g(b), None => 0 }"),
        Equal(LiteralTier),
    );
    r.pair(
        "binder_if_let",
        (&["x"], "if let Some(a) = x { g(a) } else { 0 }"),
        (&["x"], "if let Some(b) = x { g(b) } else { 0 }"),
        Equal(LiteralTier),
    );
    r.pair(
        "binder_while_let",
        (&["x"], "while let Some(a) = x.next() { g(a); }"),
        (&["x"], "while let Some(b) = x.next() { g(b); }"),
        Equal(LiteralTier),
    );
}

/// Binders that are not patterns.
fn binders_more(r: &mut Registry) {
    use Expect::Equal;
    use Level::LiteralTier;
    r.pair(
        "binder_at_binding_match_arm",
        (&["x"], "match x { n @ 1..=5 => g(n), _ => 0 }"),
        (&["x"], "match x { m @ 1..=5 => g(m), _ => 0 }"),
        Equal(LiteralTier),
    );
    r.pair(
        "binder_at_binding_if_let",
        (&["x"], "if let Some(n @ 1..=5) = x { g(n) } else { 0 }"),
        (&["x"], "if let Some(m @ 1..=5) = x { g(m) } else { 0 }"),
        Equal(LiteralTier),
    );
    r.pair(
        "binder_label",
        (&["x"], "'outer: loop { if f(x) { break 'outer; } }"),
        (&["x"], "'inner: loop { if f(x) { break 'inner; } }"),
        Equal(LiteralTier),
    );
    r.pair(
        "binder_lifetime",
        (&["x"], "fn pick<'a>(v: &'a u8) -> &'a u8 { v } pick(x)"),
        (&["x"], "fn pick<'b>(v: &'b u8) -> &'b u8 { v } pick(x)"),
        Equal(LiteralTier),
    );
    r.pair(
        "binder_local_fn",
        (&["x"], "fn helper(v: u8) -> u8 { v + 1 } helper(x)"),
        (&["x"], "fn assist(w: u8) -> u8 { w + 1 } assist(x)"),
        Equal(LiteralTier),
    );
    r.pair(
        "binder_local_struct",
        (&[], "struct Cell { n: u8 } Cell { n: 1 }"),
        (&[], "struct Slot { n: u8 } Slot { n: 1 }"),
        Equal(LiteralTier),
    );
    r.pair(
        "binder_call_position",
        (&["v"], "let f = |a| a + 1; f(v)"),
        (&["v"], "let g = |a| a + 1; g(v)"),
        Equal(LiteralTier),
    );
    r.pair(
        "binder_shadows_an_item",
        (&["v"], "let double = f(v); map(double)"),
        (&["v"], "let triple = f(v); map(triple)"),
        Equal(LiteralTier),
    );
    r.pair(
        "binder_macro_metavariable",
        (&[], "macro_rules! m { ($x:expr) => { $x }; }"),
        (&[], "macro_rules! m { ($y:expr) => { $y }; }"),
        Equal(LiteralTier),
    );
}

/// Names no one binds are items and stay.
fn items(r: &mut Registry) {
    use Expect::Distinct;
    r.pair(
        "item_function_passed_by_name",
        (&["v"], "v.iter().map(double).collect()"),
        (&["v"], "v.iter().map(triple).collect()"),
        Distinct,
    );
    r.pair(
        "item_predicate_passed_by_name",
        (&["v"], "v.iter().filter(is_small).count()"),
        (&["v"], "v.iter().filter(is_big).count()"),
        Distinct,
    );
    r.pair(
        "item_lowercase_static",
        (&["x"], "x + limit_a"),
        (&["x"], "x + limit_b"),
        Distinct,
    );
    r.pair(
        "item_in_struct_shorthand_field",
        (&["x"], "S { x }"),
        (&["y"], "S { y }"),
        Distinct,
    );
    r.pair(
        "item_field_name_kept_beside_a_local",
        (&[], "let x = f(); S { x }"),
        (&[], "let y = f(); S { y }"),
        Distinct,
    );
}

/// A block after a head that ends in a type name is a block, not a struct
/// literal.
fn head_blocks(r: &mut Registry) {
    use Expect::Equal;
    use Level::LiteralTier;
    let clamp = |next: &str, fallback: &str, head: &str| {
        format!(
            "let {next} = g(); let {fallback} = h(); {head} {next} == MAX {{ {next} }} else {{ {fallback} }}"
        )
    };
    r.pair(
        "head_block_if",
        (&[], &clamp("next", "fallback", "if")),
        (&[], &clamp("later", "other", "if")),
        Equal(LiteralTier),
    );
    r.pair(
        "head_block_else_if",
        (
            &[],
            "let a = g(); if a == MIN { 0 } else if a == MAX { a } else { 1 }",
        ),
        (
            &[],
            "let b = g(); if b == MIN { 0 } else if b == MAX { b } else { 1 }",
        ),
        Equal(LiteralTier),
    );
    r.pair(
        "head_block_while",
        (&[], "let a = g(); while a < MAX { a }"),
        (&[], "let b = g(); while b < MAX { b }"),
        Equal(LiteralTier),
    );
    r.pair(
        "head_block_match",
        (&[], "let a = g(); match a { MAX => a, _ => 0 }"),
        (&[], "let b = g(); match b { MAX => b, _ => 0 }"),
        Equal(LiteralTier),
    );
    r.pair(
        "head_block_for",
        (&[], "let a = g(); for i in 0..MAX { f(i, a); }"),
        (&[], "let b = g(); for j in 0..MAX { f(j, b); }"),
        Equal(LiteralTier),
    );
}

/// Every threshold counts the value form: noise the value form erases never
/// moves a body across a threshold.
fn thresholds(r: &mut Registry) {
    use Expect::Equal;
    use Level::Value;
    let noise = ".clone().as_slice().to_owned()";
    r.pair_sized(
        "threshold_literal_tier_straddle",
        (&["v"], "let n = v.len(); if n > 3 { return 7; } n + 1", 21),
        (
            &["v"],
            &format!("let n = v{noise}.len(); if n > 3 {{ return 7; }} n + 1"),
            21,
        ),
        Equal(Value),
    );
    r.pair_sized(
        "threshold_min_tokens_straddle",
        (&["v"], "if v.len() > 3 { 1 } else { 0 }", 15),
        (
            &["v"],
            &format!("if v{noise}.len() > 3 {{ 1 }} else {{ 0 }}"),
            15,
        ),
        Equal(Value),
    );
}

/// Value form: one pair per erasure.
fn value_form(r: &mut Registry) {
    use Expect::{Distinct, Equal};
    use Level::Value;
    r.pair(
        "value_borrow",
        (&["x"], "g(&x)"),
        (&["x"], "g(x)"),
        Equal(Value),
    );
    r.pair(
        "value_borrow_mut",
        (&["x"], "g(&mut x)"),
        (&["x"], "g(x)"),
        Equal(Value),
    );
    r.pair(
        "value_double_borrow",
        (&["x"], "g(&&x)"),
        (&["x"], "g(x)"),
        Equal(Value),
    );
    r.pair(
        "value_deref",
        (&["x"], "g(*x)"),
        (&["x"], "g(x)"),
        Equal(Value),
    );
    r.pair(
        "value_ref_binding",
        (&["o"], "match o { Some(ref v) => g(v), None => 0 }"),
        (&["o"], "match o { Some(v) => g(v), None => 0 }"),
        Equal(Value),
    );
    r.pair(
        "value_ref_mut_binding",
        (&["o"], "match o { Some(ref mut v) => g(v), None => 0 }"),
        (&["o"], "match o { Some(v) => g(v), None => 0 }"),
        Equal(Value),
    );
    for adapter in VALUE_ADAPTERS {
        r.pair(
            &format!("value_adapter_{adapter}"),
            (&["x"], &format!("g(x.{adapter}())")),
            (&["x"], "g(x)"),
            Equal(Value),
        );
    }
    r.pair(
        "value_adapter_with_argument_kept",
        (&["x", "y"], "g(x.to_owned(y))"),
        (&["x", "y"], "g(x)"),
        Distinct,
    );
}

/// Value form: ranges, annotations, turbofish and index references.
fn value_form_more(r: &mut Registry) {
    use Expect::{Distinct, Equal};
    use Level::Value;
    r.pair(
        "value_full_range",
        (&["v"], "g(&v[..])"),
        (&["v"], "g(&v)"),
        Equal(Value),
    );
    r.pair(
        "value_partial_range_kept",
        (&["v"], "g(&v[1..])"),
        (&["v"], "g(&v)"),
        Distinct,
    );
    r.pair(
        "value_range_to_kept",
        (&["v"], "g(&v[..1])"),
        (&["v"], "g(&v)"),
        Distinct,
    );
    r.pair(
        "value_let_annotation",
        (&["x"], "let n: u64 = f(x); n"),
        (&["x"], "let n = f(x); n"),
        Equal(Value),
    );
    r.pair(
        "value_let_annotation_generic",
        (&["x"], "let n: Vec<&'a str> = f(x); n"),
        (&["x"], "let n = f(x); n"),
        Equal(Value),
    );
    r.pair(
        "value_erased_names_do_not_shift_numbers",
        (&["x"], "let n: &'a str = f(x); let m = g(n); m"),
        (&["x"], "let n = f(x); let m = g(n); m"),
        Equal(Value),
    );
    r.pair(
        "value_type_turbofish",
        (&["x"], "x.collect::<Vec<_>>()"),
        (&["x"], "x.collect()"),
        Equal(Value),
    );
    r.pair(
        "value_const_turbofish_kept",
        (&["x"], "f::<3>(x)"),
        (&["x"], "f(x)"),
        Distinct,
    );
    r.pair(
        "value_bool_turbofish_kept",
        (&["x"], "f::<true>(x)"),
        (&["x"], "f::<false>(x)"),
        Distinct,
    );
    r.pair(
        "value_index_reference",
        (&["m", "k"], "m[&k]"),
        (&["m", "k"], "m[k]"),
        Equal(Value),
    );
    r.pair(
        "value_binary_and_kept",
        (&["a", "b"], "g(a & b)"),
        (&["a", "b"], "g(a b)"),
        Distinct,
    );
    r.pair(
        "value_binary_mul_kept",
        (&["a", "b"], "g(a * b)"),
        (&["a", "b"], "g(a b)"),
        Distinct,
    );
    r.pair(
        "value_logical_and_kept",
        (&["a", "b"], "g(a && b)"),
        (&["a", "b"], "g(a b)"),
        Distinct,
    );
    r.pair(
        "value_compound_assign_kept",
        (&["a", "b"], "a &= b; a *= b;"),
        (&["a", "b"], "a b; a b;"),
        Distinct,
    );
    r.pair(
        "value_binary_after_call_kept",
        (&["a", "b"], "f(a) * b"),
        (&["a", "b"], "f(a) b"),
        Distinct,
    );
}

/// Literal tier: under 30 structural tokens literals stay; from 30 up they
/// are abstracted to their kind.
fn literal_tier(r: &mut Registry) {
    use Expect::{Distinct, Equal};
    use Level::Structural;
    let lit = |literal: &str, total: usize| padded(&format!("f({literal});"), 5, total);
    r.pair_sized(
        "tier_25",
        (&["a"], &lit("\"alpha\"", 25), 25),
        (&["a"], &lit("\"beta\"", 25), 25),
        Distinct,
    );
    r.pair_sized(
        "tier_29",
        (&["a"], &lit("\"alpha\"", 29), 29),
        (&["a"], &lit("\"beta\"", 29), 29),
        Distinct,
    );
    r.pair_sized(
        "tier_30",
        (&["a"], &lit("\"alpha\"", 30), 30),
        (&["a"], &lit("\"beta\"", 30), 30),
        Equal(Structural),
    );
    r.pair_sized(
        "tier_35",
        (&["a"], &lit("\"alpha\"", 35), 35),
        (&["a"], &lit("\"beta\"", 35), 35),
        Equal(Structural),
    );
    r.pair_sized(
        "tier_30_kinds_differ",
        (&["a"], &lit("\"alpha\"", 30), 30),
        (&["a"], &lit("7", 30), 30),
        Distinct,
    );
    r.pair_sized(
        "tier_30_float_is_not_int",
        (&["a"], &lit("1.5", 30), 30),
        (&["a"], &lit("7", 30), 30),
        Distinct,
    );
    r.pair_sized(
        "tier_30_big_int_is_int",
        (
            &["a"],
            &lit("340282366920938463463374607431768211456", 30),
            30,
        ),
        (&["a"], &lit("7", 30), 30),
        Equal(Structural),
    );
    r.pair_sized(
        "tier_29_big_int_kept",
        (
            &["a"],
            &lit("340282366920938463463374607431768211456", 29),
            29,
        ),
        (&["a"], &lit("7", 29), 29),
        Distinct,
    );
    r.pair_sized(
        "tier_30_suffix_is_part_of_the_kind",
        (&["a"], &lit("7u8", 30), 30),
        (&["a"], &lit("7", 30), 30),
        Equal(Structural),
    );
    r.pair_sized(
        "tier_30_tuple_index_kept",
        (&["a"], &padded("a.0;", 4, 30), 30),
        (&["a"], &padded("a.1;", 4, 30), 30),
        Distinct,
    );
    r.pair_sized(
        "tier_30_range_end_is_a_literal",
        (&["a"], &padded("f(..1);", 7, 30), 30),
        (&["a"], &padded("f(..2);", 7, 30), 30),
        Equal(Structural),
    );
}

/// Literal spellings: an integer is its value.
fn literal_spellings(r: &mut Registry) {
    use Expect::{Distinct, Equal};
    use Level::Verbatim;
    r.pair(
        "literal_hex_and_decimal",
        (&[], "f(0xff)"),
        (&[], "f(255)"),
        Equal(Verbatim),
    );
    r.pair(
        "literal_separators",
        (&[], "f(1_000)"),
        (&[], "f(1000)"),
        Equal(Verbatim),
    );
    r.pair(
        "literal_octal_binary",
        (&[], "f(0o17, 0b1010)"),
        (&[], "f(15, 10)"),
        Equal(Verbatim),
    );
    r.pair(
        "literal_suffix_kept",
        (&[], "f(255u8)"),
        (&[], "f(0xffu8)"),
        Equal(Verbatim),
    );
    r.pair(
        "literal_different_value",
        (&[], "f(255)"),
        (&[], "f(256)"),
        Distinct,
    );
    r.pair(
        "literal_suffix_differs",
        (&[], "f(255u8)"),
        (&[], "f(255u16)"),
        Distinct,
    );
    r.pair(
        "literal_float_is_not_int",
        (&[], "f(1.0)"),
        (&[], "f(1)"),
        Distinct,
    );
    r.pair(
        "literal_string_spelling_differs",
        (&[], "f(\"a\")"),
        (&[], "f(r\"a\")"),
        Distinct,
    );
}

/// Shim form.
fn shim_form(r: &mut Registry) {
    use Expect::{Distinct, Equal};
    use Level::{LiteralTier, Shim, Value};
    r.single("shim_5", &["a"], "a.f()", Some(5));
    r.single("shim_6", &["a", "b"], "f(a, b)", Some(6));
    r.single("shim_29", &["a"], &forwarder(29), Some(29));
    r.single("shim_30", &["a"], &forwarder(30), Some(30));
    r.pair(
        "shim_renamed_params",
        (&["x"], "target(x, 1)"),
        (&["y"], "target(y, 1)"),
        Equal(Shim),
    );
    r.pair(
        "shim_only_own_params_renamed",
        (&[], "target(first, 1)"),
        (&[], "target(second, 1)"),
        Distinct,
    );
    r.pair(
        "shim_different_callee",
        (&["x"], "target(x, 1)"),
        (&["x"], "other(x, 1)"),
        Distinct,
    );
    r.pair(
        "shim_different_fixed_argument",
        (&["x"], "target(x, 1)"),
        (&["x"], "target(x, 2)"),
        Distinct,
    );
    // The shim form keeps signature order, so these two forwarders have
    // different shim prints; the body form numbers by first appearance.
    r.pair(
        "shim_parameter_order",
        (&["x", "y"], "target(x, y, 1)"),
        (&["x", "y"], "target(y, x, 1)"),
        Equal(LiteralTier),
    );
    r.single("shim_let_body", &["x"], "let v = target(x, 1, 2); v", None);
    r.single(
        "shim_two_statements",
        &["x"],
        "target(x, 1); other(x, 2)",
        None,
    );
    r.single("shim_trailing_semicolon", &["x"], "target(x, 1, 2);", None);
    r.single(
        "shim_mentions_self",
        &["x"],
        "self.inner.target(x, 1)",
        None,
    );
    r.pair(
        "shim_turbofish_forwarder",
        (&["x"], "target::<u8>(x, 1)"),
        (&["y"], "target::<u8>(y, 1)"),
        Equal(Shim),
    );
    r.pair(
        "shim_closure_body_with_let",
        (&["a"], "g(a, |x| { let y = x; y })"),
        (&["b"], "g(b, |x| { let y = x; y })"),
        Equal(Shim),
    );
    r.pair(
        "doc_comment_renamed_local",
        (&["x"], "/// the total\nlet n = x + 1; n * 2"),
        (&["x"], "/// the total\nlet k = x + 1; k * 2"),
        Equal(LiteralTier),
    );
    r.pair(
        "doc_comment_text_differs",
        (&["x"], "/// the total\nlet n = x + 1; n * 2"),
        (&["x"], "/// the sum\nlet n = x + 1; n * 2"),
        Distinct,
    );
    r.single(
        "shim_method_turbofish",
        &["x"],
        "x.convert::<u8>(1, 2)",
        Some(8),
    );
    r.pair(
        "shim_turbofish_type_differs",
        (&["x"], "target::<u8>(x, 1)"),
        (&["x"], "target::<u16>(x, 1)"),
        Equal(Value),
    );
    r.single(
        "shim_turbofish_without_call",
        &["x"],
        "x.field + Vec::<u8>::LIMIT + 1",
        None,
    );
    r.single("shim_without_call", &["x"], "x.field + x.other + 1", None);
    r.single("shim_macro_call", &["x"], "format!(\"{x}\", x = 1)", None);
}

/// Macro arguments and inline format captures.
fn macro_arguments(r: &mut Registry) {
    use Expect::{Distinct, Equal};
    use Level::{LiteralTier, Shim};
    r.pair(
        "macro_arguments",
        (&["b"], "format!(\"{x:02x}\", x = b)"),
        (&["c"], "format!(\"{y:02x}\", y = c)"),
        Equal(LiteralTier),
    );
    r.pair(
        "macro_inline_capture",
        (&["n", "c"], "format!(\"{n}: {c}\")"),
        (&["a", "b"], "format!(\"{a}: {b}\")"),
        Equal(Shim),
    );
    r.pair(
        "macro_inline_capture_local",
        (&["x"], "let n = f(x); format!(\"{n}: {x:?}\")"),
        (&["y"], "let m = f(y); format!(\"{m}: {y:?}\")"),
        Equal(LiteralTier),
    );
    r.pair(
        "macro_unknown_name_kept",
        (&["x"], "format!(\"{name}: {}\", x)"),
        (&["x"], "format!(\"{other}: {}\", x)"),
        Distinct,
    );
    r.pair(
        "macro_positional_kept",
        (&["x"], "format!(\"{0}\", x)"),
        (&["x"], "format!(\"{1}\", x)"),
        Distinct,
    );
    r.pair(
        "macro_escaped_brace_kept",
        (&["x"], "format!(\"{{x}}\", x = 1)"),
        (&["y"], "format!(\"{{y}}\", y = 1)"),
        Distinct,
    );
    r.pair(
        "macro_unicode_escape_is_not_a_placeholder",
        (&["abc"], "format!(\"\\u{abc}{abc}\")"),
        (&["def"], "format!(\"\\u{abc}{def}\")"),
        Equal(Shim),
    );
    let tail = "h(1); ".repeat(8);
    r.pair(
        "macro_arguments_above_tier",
        (&["b"], &format!("format!(\"{{x:02x}}\", x = b); {tail}")),
        (&["c"], &format!("format!(\"{{y:02x}}\", y = c); {tail}")),
        Equal(LiteralTier),
    );
    r.pair(
        "macro_inline_capture_above_tier",
        (&["n", "c"], &format!("format!(\"{{n}}: {{c}}\"); {tail}")),
        (&["a", "b"], &format!("format!(\"{{a}}: {{b}}\"); {tail}")),
        Equal(Shim),
    );
}

/// Numbering of locals after parameters.
fn numbering(r: &mut Registry) {
    use Expect::{Distinct, Equal};
    use Level::{LiteralTier, Structural, Verbatim};
    r.pair(
        "numbering_unused_trailing_parameter",
        (&["a", "b"], "let x = a; x"),
        (&["a"], "let x = a; x"),
        Equal(Verbatim),
    );
    r.pair(
        "numbering_used_trailing_parameter",
        (&["a", "b"], "let x = b; x"),
        (&["a"], "let x = a; x"),
        Equal(LiteralTier),
    );
    r.pair(
        "numbering_parameter_order",
        (&["a", "b"], "let x = b + a; x"),
        (&["a", "b"], "let x = a + b; x"),
        Equal(LiteralTier),
    );
    r.pair(
        "numbering_placeholder_parameter",
        (&["a"], "let x = 1; println!(\"{a} {x}\")"),
        (&["a"], "let x = 1; println!(\"{x} {x}\")"),
        Distinct,
    );
    let tail = "h(1); ".repeat(8);
    r.pair(
        "numbering_placeholder_parameter_above_tier",
        (
            &["a"],
            &format!("let x = 1; println!(\"{{a}} {{x}}\"); {tail}"),
        ),
        (
            &["a"],
            &format!("let x = 1; println!(\"{{x}} {{x}}\"); {tail}"),
        ),
        Equal(Structural),
    );
    r.pair(
        "numbering_placeholder_first_appearance",
        (&["a", "b"], "println!(\"{b} {a}\"); g(a)"),
        (&["a", "b"], "println!(\"{a} {b}\"); g(b)"),
        Equal(LiteralTier),
    );
    r.pair(
        "numbering_placeholder_versus_argument",
        (&["a"], "let x = 1; println!(\"{a} {x}\")"),
        (&["a"], "let x = 1; println!(\"{} {x}\", a)"),
        Distinct,
    );
}

/// Lexical edges.
fn lexical_edges(r: &mut Registry) {
    use Expect::{Distinct, Equal};
    use Level::{LiteralTier, Shim};
    for word in ["gen", "union", "raw", "safe", "macro_rules"] {
        r.pair(
            &format!("lexical_{word}_kept"),
            (&["x"], &format!("let {word} = f(x); {word}")),
            (&["x"], "let v = f(x); v"),
            Distinct,
        );
    }
    r.pair(
        "lexical_macro_rules_definition",
        (&[], "macro_rules! m { ($x:expr) => { $x }; }"),
        (&[], "macro_rules! m { ($y:expr) => { $y }; }"),
        Equal(LiteralTier),
    );
    r.pair(
        "lexical_wildcard_kept",
        (&["x"], "match x { _ => 0 }"),
        (&["x"], "match x { y => 0 }"),
        Distinct,
    );
    let nest = |depth: usize, name: &str| purrdf_testkit::text::nested("(", name, ")", depth);
    r.pair(
        "lexical_nesting_128",
        (&["left"], &nest(128, "left")),
        (&["right"], &nest(128, "right")),
        Equal(Shim),
    );
    r.single("empty_body", &[], "", None);
}

pub(crate) fn registry() -> Registry {
    let mut r = Registry {
        fixtures: Vec::new(),
        pairs: Vec::new(),
    };
    structural(&mut r);
    binders(&mut r);
    binders_more(&mut r);
    items(&mut r);
    head_blocks(&mut r);
    thresholds(&mut r);
    value_form(&mut r);
    value_form_more(&mut r);
    literal_tier(&mut r);
    literal_spellings(&mut r);
    shim_form(&mut r);
    macro_arguments(&mut r);
    numbering(&mut r);
    lexical_edges(&mut r);
    r
}

/// The table in its pinned form: one line per fixture, `id tokens body shim`,
/// with `-` for a print the fixture does not have. The bodies are printed at
/// `min_tokens` 1 so every fixture has one.
pub(crate) fn table(registry: &Registry) -> String {
    let options = CensusOptions::default()
        .min_tokens(1)
        .expect("1 is a valid min_tokens");
    let normalizer = Normalizer::new(&options);
    let mut out = String::new();
    for fixture in &registry.fixtures {
        let print = normalizer
            .print(&fixture.slice())
            .unwrap_or_else(|error| panic!("fixture {} does not normalise: {error}", fixture.id));
        let name = |hashed: &Option<raf_census::Fingerprint>| {
            hashed
                .as_ref()
                .map_or_else(|| "-".to_owned(), ToString::to_string)
        };
        writeln!(
            out,
            "{} {} {} {}",
            fixture.id,
            print.tokens,
            name(&print.body),
            name(&print.shim)
        )
        .expect("a string takes a write");
    }
    out
}
