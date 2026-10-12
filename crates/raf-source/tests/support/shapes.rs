// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The source shapes that stress the parser, as text generators.
//!
//! Each generator returns a file whose only top-level item is named in the
//! shape's expected symbol list. The committed fuzz corpus holds the same
//! texts under the names below.

/// A named source text and the symbols a correct index holds for it.
pub(crate) struct Shape {
    /// The corpus file name, without the extension.
    pub(crate) name: &'static str,
    /// The source text.
    pub(crate) text: String,
    /// The top-level item names a correct index lists.
    pub(crate) symbols: &'static [&'static str],
}

/// `type T = A<A<...u8...>>;` nested `n` deep.
pub(crate) fn generics(n: usize) -> String {
    format!("type T = {}u8{};\n", "A<".repeat(n), ">".repeat(n))
}

/// A function returning `n` negations of `true`.
pub(crate) fn bangs(n: usize) -> String {
    format!("fn f() -> bool {{ {}true }}\n", "!".repeat(n))
}

/// `type T = &&...u8;` with `n` reference marks.
pub(crate) fn refs(n: usize) -> String {
    format!("type T = {}u8;\n", "&".repeat(n))
}

/// A function holding `n` empty closures in a row, each the body of the last.
pub(crate) fn bars(n: usize) -> String {
    format!("fn f() {{ let _ = {}0; }}\n", "|| ".repeat(n))
}

/// A function with an `if` followed by `n` `else if` links.
pub(crate) fn else_ifs(n: usize) -> String {
    format!(
        "fn f(a: bool) {{ if a {{}} {}}}\n",
        "else if a {} ".repeat(n)
    )
}

/// A constant wrapped in `n` parentheses.
pub(crate) fn parens(n: usize) -> String {
    format!("const X: i32 = {}0{};\n", "(".repeat(n), ")".repeat(n))
}

/// A function whose `else if` links each have a block as the condition.
pub(crate) fn block_condition_else_ifs(n: usize) -> String {
    format!(
        "fn f(a: bool) {{ if a {{}} {}}}\n",
        "else if {true} {} ".repeat(n)
    )
}

/// `type T<'a> = &'a &'a ...u8;` with `n` reference marks.
pub(crate) fn lifetime_refs(n: usize) -> String {
    format!("type T<'a> = {}u8;\n", "&'a ".repeat(n))
}

fn shape(name: &'static str, text: String, symbols: &'static [&'static str]) -> Shape {
    Shape {
        name,
        text,
        symbols,
    }
}

/// The five single shapes, each over its budget.
pub(crate) fn singles() -> Vec<Shape> {
    vec![
        shape("single_generics_500", generics(500), &["T"]),
        shape("single_bang_4000", bangs(4000), &["f"]),
        shape("single_ref_20000", refs(20000), &["T"]),
        shape("single_closure_bars_10000", bars(10000), &["f"]),
        shape("single_else_if_20000", else_ifs(20000), &["f"]),
    ]
}

/// The five composites: each part within its budget, multiplied by nesting.
pub(crate) fn composites() -> Vec<Shape> {
    let nested = |lead: &str, levels: usize| lead.repeat(levels);
    let close = |levels: usize| ")".repeat(levels);
    vec![
        shape(
            "composite_bang_paren",
            format!(
                "fn f() -> bool {{ {}true{}; }}\n",
                nested(&format!("{}(", "!".repeat(256)), 127),
                close(127)
            ),
            &["f"],
        ),
        shape(
            "composite_ref_paren",
            format!(
                "type T = {}u8{};\n",
                nested(&format!("{}(", "& ".repeat(256)), 127),
                close(127)
            ),
            &["T"],
        ),
        shape(
            "composite_double_ref_paren",
            format!(
                "type T = {}u8{};\n",
                nested(&format!("{}(", "&& ".repeat(256)), 127),
                close(127)
            ),
            &["T"],
        ),
        shape(
            "composite_generic_ref",
            format!(
                "type T = {}u8{};\n",
                nested(&format!("A<{}", "& ".repeat(256)), 64),
                ">".repeat(64)
            ),
            &["T"],
        ),
        shape(
            "composite_bars_paren",
            format!(
                "fn f() {{ let _ = {}0{}; }}\n",
                nested(&format!("{}(", "|| ".repeat(128)), 127),
                close(127)
            ),
            &["f"],
        ),
    ]
}

/// The six shapes that no budget counts, repeated 20000 times.
pub(crate) fn uncounted() -> Vec<Shape> {
    let repeat = |unit: &str| unit.repeat(20000);
    vec![
        shape(
            "uncounted_return",
            format!("fn f() {{ {}; }}\n", repeat("return ")),
            &["f"],
        ),
        shape(
            "uncounted_ref_mut",
            format!("type T = {}u8;\n", repeat("&mut ")),
            &["T"],
        ),
        shape(
            "uncounted_ptr_const",
            format!("type T = {}u8;\n", repeat("*const ")),
            &["T"],
        ),
        shape(
            "uncounted_fn_pointer",
            format!("type T = {}u8;\n", repeat("fn() -> ")),
            &["T"],
        ),
        shape(
            "uncounted_assignment",
            format!("fn f() {{ let mut a = 0; {}0; }}\n", repeat("a = ")),
            &["f"],
        ),
        shape(
            "uncounted_move_closure",
            format!("fn f() {{ let _ = {}0; }}\n", repeat("move || ")),
            &["f"],
        ),
    ]
}

/// Shapes that try to slip past a counter, and each one budget short of it.
pub(crate) fn evasions() -> Vec<Shape> {
    vec![
        shape(
            "evasion_else_if_block_condition_20000",
            block_condition_else_ifs(20000),
            &["f"],
        ),
        shape(
            "evasion_else_if_block_condition_at",
            block_condition_else_ifs(256),
            &["f"],
        ),
        shape("evasion_ref_lifetime_20000", lifetime_refs(20000), &["T"]),
        shape("evasion_ref_lifetime_at", lifetime_refs(256), &["T"]),
    ]
}

/// A file exactly at a budget and the same shape one past it.
pub(crate) fn budget_pairs() -> Vec<Shape> {
    vec![
        shape("budget_delimiters_at", parens(128), &["X"]),
        shape("budget_delimiters_over", parens(129), &["X"]),
        shape("budget_angles_at", generics(64), &["T"]),
        shape("budget_angles_over", generics(65), &["T"]),
        shape("budget_prefix_at", bangs(256), &["f"]),
        shape("budget_prefix_over", bangs(257), &["f"]),
        shape("budget_closure_bars_at", bars(128), &["f"]),
        shape("budget_closure_bars_over", bars(129), &["f"]),
        shape("budget_else_if_at", else_ifs(256), &["f"]),
        shape("budget_else_if_over", else_ifs(257), &["f"]),
    ]
}

/// Every shape that has a corpus file, in a stable order.
pub(crate) fn all() -> Vec<Shape> {
    let mut shapes = singles();
    shapes.extend(composites());
    shapes.extend(uncounted());
    shapes.extend(budget_pairs());
    shapes.extend(evasions());
    shapes
}

/// The names `all` must produce, spelled out so that a missing corpus file
/// cannot go unnoticed by renaming a shape.
pub(crate) const EXPECTED_NAMES: [&str; 30] = [
    "single_generics_500",
    "single_bang_4000",
    "single_ref_20000",
    "single_closure_bars_10000",
    "single_else_if_20000",
    "composite_bang_paren",
    "composite_ref_paren",
    "composite_double_ref_paren",
    "composite_generic_ref",
    "composite_bars_paren",
    "uncounted_return",
    "uncounted_ref_mut",
    "uncounted_ptr_const",
    "uncounted_fn_pointer",
    "uncounted_assignment",
    "uncounted_move_closure",
    "budget_delimiters_at",
    "budget_delimiters_over",
    "budget_angles_at",
    "budget_angles_over",
    "budget_prefix_at",
    "budget_prefix_over",
    "budget_closure_bars_at",
    "budget_closure_bars_over",
    "budget_else_if_at",
    "budget_else_if_over",
    "evasion_else_if_block_condition_20000",
    "evasion_else_if_block_condition_at",
    "evasion_ref_lifetime_20000",
    "evasion_ref_lifetime_at",
];
