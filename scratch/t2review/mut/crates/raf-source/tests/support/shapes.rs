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

/// The six shapes that no run budget counts, repeated 20000 times. The
/// recursion budget refuses each of them.
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

/// Chains that the parser, the syntax tree or its drop nest one level per
/// link, each long enough to have overflowed the parse stack before the
/// recursion budget counted it, and each at most 400000 bytes.
pub(crate) fn chains() -> Vec<Shape> {
    let expression =
        |unit: &str, links: usize| format!("fn f() {{ let _ = x{}; }}\n", unit.repeat(links));
    let prefixed = |unit: &str, links: usize, end: &str| {
        format!("fn f() {{ let _ = {}{end}; }}\n", unit.repeat(links))
    };
    let statement = |unit: &str, end: &str| format!("fn f() {{ {}{end}; }}\n", unit.repeat(20000));
    let alias = |unit: &str| format!("type T = {}u8;\n", unit.repeat(20000));
    vec![
        shape("chain_try", expression("?", 130_000), &["f"]),
        shape("chain_field", expression(".a", 130_000), &["f"]),
        shape("chain_call", expression("()", 130_000), &["f"]),
        shape("chain_index", expression("[0]", 130_000), &["f"]),
        shape("chain_cast", expression(" as T", 78_000), &["f"]),
        shape("chain_add", expression("+x", 130_000), &["f"]),
        shape("chain_mul", expression("*x", 130_000), &["f"]),
        shape("chain_await", expression(".await", 65_000), &["f"]),
        shape("chain_method", expression(".m()", 97_000), &["f"]),
        shape(
            "chain_async_closure",
            prefixed("async || ", 20000, "0"),
            &["f"],
        ),
        shape(
            "chain_async_move_closure",
            prefixed("async move || ", 20000, "0"),
            &["f"],
        ),
        shape(
            "chain_const_closure",
            prefixed("const || ", 20000, "0"),
            &["f"],
        ),
        shape(
            "chain_return_closure",
            prefixed("|| return ", 20000, "0"),
            &["f"],
        ),
        shape("chain_raw_ref", prefixed("&raw mut *", 20000, "x"), &["f"]),
        shape(
            "chain_break",
            format!("fn f() {{ loop {{ {}0; }} }}\n", "break ".repeat(20000)),
            &["f"],
        ),
        shape("chain_yield", statement("yield ", "0"), &["f"]),
        shape("chain_become", statement("become ", "f()"), &["f"]),
        shape("chain_add_assign", statement("a += ", "0"), &["f"]),
        shape(
            "chain_impl_fn_return",
            format!("fn f() -> {}u8 {{ 0 }}\n", "impl Fn() -> ".repeat(20000)),
            &["f"],
        ),
        shape(
            "chain_extern_fn_pointer",
            alias("extern \"C\" fn() -> "),
            &["T"],
        ),
        shape("chain_unsafe_fn_pointer", alias("unsafe fn() -> "), &["T"]),
        shape("chain_for_fn_pointer", alias("for<'a> fn() -> "), &["T"]),
        shape("chain_ptr_mut", alias("*mut "), &["T"]),
        shape("chain_box_pattern", box_patterns(20000), &["f"]),
    ]
}

/// A `let` whose pattern is `box` repeated `n` times.
pub(crate) fn box_patterns(n: usize) -> String {
    format!("fn f() {{ let {}x = 0; }}\n", "box ".repeat(n))
}

/// A chain of `n` `return`s in an expression statement.
pub(crate) fn returns(n: usize) -> String {
    format!("fn f() {{ {}0; }}\n", "return ".repeat(n))
}

/// The recursion value of [`returns`] apart from its `return`s: `fn`, the `(`
/// of the parameter list, and the `{` of the body and its open level.
pub(crate) const RETURNS_BASE: usize = 4;

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
        shape(
            "budget_recursion_at",
            returns(raf_source::Budget::Recursion.limit() - RETURNS_BASE),
            &["f"],
        ),
        shape(
            "budget_recursion_over",
            returns(raf_source::Budget::Recursion.limit() - RETURNS_BASE + 1),
            &["f"],
        ),
    ]
}

/// Every shape that has a corpus file, in a stable order.
pub(crate) fn all() -> Vec<Shape> {
    let mut shapes = singles();
    shapes.extend(composites());
    shapes.extend(uncounted());
    shapes.extend(chains());
    shapes.extend(budget_pairs());
    shapes.extend(evasions());
    shapes
}

/// The names `all` must produce, spelled out so that a missing corpus file
/// cannot go unnoticed by renaming a shape.
pub(crate) const EXPECTED_NAMES: [&str; 56] = [
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
    "chain_try",
    "chain_field",
    "chain_call",
    "chain_index",
    "chain_cast",
    "chain_add",
    "chain_mul",
    "chain_await",
    "chain_method",
    "chain_async_closure",
    "chain_async_move_closure",
    "chain_const_closure",
    "chain_return_closure",
    "chain_raw_ref",
    "chain_break",
    "chain_yield",
    "chain_become",
    "chain_add_assign",
    "chain_impl_fn_return",
    "chain_extern_fn_pointer",
    "chain_unsafe_fn_pointer",
    "chain_for_fn_pointer",
    "chain_ptr_mut",
    "chain_box_pattern",
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
    "budget_recursion_at",
    "budget_recursion_over",
    "evasion_else_if_block_condition_20000",
    "evasion_else_if_block_condition_at",
    "evasion_ref_lifetime_20000",
    "evasion_ref_lifetime_at",
];

/// A family of shapes that grows with a size: `text(size)` is the file.
/// `at` is the largest size whose file loads; one size more is refused by
/// `over`, the budget that meets it first.
pub(crate) struct Family {
    /// The family's name, used in selectors.
    pub(crate) name: &'static str,
    /// The file of a given size.
    pub(crate) text: fn(usize) -> String,
    /// The largest size that loads.
    pub(crate) at: usize,
    /// The budget that refuses one size more.
    pub(crate) over: raf_source::Budget,
    /// The top-level item names a correct index lists.
    pub(crate) symbols: &'static [&'static str],
}

/// Counted tokens that stay open after a level's count returns to zero, so
/// that a level which ends a list at each step still nests.
const PAD: &str = "return return return return return return return return return \
                   return return return return return return return return ";

/// Reference marks that stay open in a type or a pattern.
const REFS: &str = "& & & & & & & & & & & & & & & ";

fn in_body(expression: &str) -> String {
    format!("fn f() {{ let _ = {expression}; }}\n")
}

fn nest(open: &str, inner: &str, close: &str, size: usize) -> String {
    format!("{}{inner}{}", open.repeat(size), close.repeat(size))
}

fn closures_with_parameters(size: usize) -> String {
    in_body(&nest(
        &format!("|a, b| {{ let _x = 0; {PAD}"),
        "0",
        " }",
        size,
    ))
}

fn tuples(size: usize) -> String {
    in_body(&nest(&format!("(0, {PAD}"), "0", ")", size))
}

fn arrays(size: usize) -> String {
    in_body(&nest(&format!("[0, {PAD}"), "0", "]", size))
}

fn struct_literals(size: usize) -> String {
    in_body(&nest(&format!("S {{ a: 0, b: {PAD}"), "0", " }", size))
}

fn statement_blocks(size: usize) -> String {
    in_body(&nest(&format!("{{ let _x = 0; {PAD}"), "0", " }", size))
}

fn match_arms(size: usize) -> String {
    in_body(&nest(
        &format!("match x {{ 0 => 0, _ => {PAD}"),
        "0",
        " }",
        size,
    ))
}

fn call_arguments(size: usize) -> String {
    in_body(&nest(&format!("g(0, {PAD}"), "0", ")", size))
}

fn if_let_else_if(size: usize) -> String {
    in_body(&nest(
        &format!("if let Some(_x) = y {{ 0 }} else if a {{ let _z = 0; {PAD}"),
        "0",
        " } else { 0 }",
        size,
    ))
}

fn labelled_blocks(size: usize) -> String {
    in_body(&nest(&format!("'a: {{ let _x = 0; {PAD}"), "0", " }", size))
}

fn async_blocks(size: usize) -> String {
    in_body(&nest(
        &format!("async move {{ let _x = 0; {PAD}"),
        "0",
        " }",
        size,
    ))
}

fn macro_call_groups(size: usize) -> String {
    format!(
        "fn f() {{ m!({}); }}\n",
        nest("(a, b; c => !d ", "x", ")", size)
    )
}

fn macro_rules_bodies(size: usize) -> String {
    format!(
        "macro_rules! r {{ () => {{ {} }}; }}\nfn f() {{}}\n",
        nest("{ a; b => !c ", "x", "}", size)
    )
}

fn attribute_arguments(size: usize) -> String {
    format!("#[a({})]\nfn f() {{}}\n", nest("b(c, ", "d", ")", size))
}

fn deep_pattern_alternatives(size: usize) -> String {
    let alternative = nest(
        "(box box box box box box box box box box box box ",
        "a",
        ")",
        size,
    );
    format!(
        "fn f() {{ match x {{ {} => 0, _ => 1 }}; }}\n",
        [alternative.as_str(); 8].join(" | ")
    )
}

fn where_bounds(size: usize) -> String {
    format!(
        "fn f<T, U>() where U: X, T: {} {{}}\n",
        nest(
            &format!("for<'a> Fn(&'a u8) -> {REFS}Box<dyn "),
            "X",
            ">",
            size
        )
    )
}

fn impl_trait_arguments(size: usize) -> String {
    format!(
        "fn f(x: {}) {{}}\n",
        nest("impl Fn(& & & & & & & & ", "u8", ")", size)
    )
}

fn nested_for_lifetimes(size: usize) -> String {
    format!(
        "type T = {};\n",
        nest("for<'a> fn(& & & & & & & & ", "u8", ")", size)
    )
}

fn generic_types(size: usize) -> String {
    format!(
        "type T = {};\n",
        nest(&format!("{REFS}A<"), "u8", ">", size)
    )
}

fn turbofish(size: usize) -> String {
    in_body(&format!(
        "g::<{}>()",
        nest(&format!("{REFS}A<"), "u8", ">", size)
    ))
}

fn reference_pointer_chains(size: usize) -> String {
    format!("type T<'a> = {}u8;\n", "&'a mut *const ".repeat(size))
}

fn fn_pointer_chains(size: usize) -> String {
    format!("type T = {}u8;\n", "fn(u8) -> ".repeat(size))
}

fn dyn_fn_chains(size: usize) -> String {
    format!(
        "type T = {};\n",
        nest("& & & & & & & & & & & & Box<dyn Fn() -> ", "u8", ">", size)
    )
}

fn tuple_and_array_types(size: usize) -> String {
    format!(
        "type T = {};\n",
        nest(&format!("(u8, {REFS}["), "u8", "; 1])", size)
    )
}

fn cast_chains(size: usize) -> String {
    in_body(&format!("x{}", " as u8".repeat(size)))
}

fn unary_chains(size: usize) -> String {
    in_body(&format!("{}x", "! - * & return ".repeat(size)))
}

fn try_chains(size: usize) -> String {
    in_body(&format!("x{}", "?".repeat(size)))
}

fn range_chains(size: usize) -> String {
    in_body(&format!("{}x", "..= ".repeat(size)))
}

fn method_chains(size: usize) -> String {
    in_body(&format!("x{}", ".m(0)".repeat(size)))
}

fn raw_strings(size: usize) -> String {
    in_body(&format!("{}0", "return r#\"( [ { \"# + ".repeat(size)))
}

fn chars_and_lifetimes(size: usize) -> String {
    format!(
        "fn f<'a>(_x: &'a u8) {{ let _ = {}0; }}\n",
        "return '{' as u8 + ".repeat(size)
    )
}

fn nested_block_comments(size: usize) -> String {
    in_body(&format!("{}0", "return /* ( /* { */ [ */ ".repeat(size)))
}

fn raw_identifiers(size: usize) -> String {
    in_body(&format!("{}0", "return r#match + ".repeat(size)))
}

fn crlf_lines(size: usize) -> String {
    format!("fn f() {{\r\n{}0;\r\n}}\r\n", "return\r\n".repeat(size))
}

fn inline_modules(size: usize) -> String {
    format!("{}\nfn f() {{}}\n", nest("mod a { ", "", "}", size))
}

fn fns_in_blocks(size: usize) -> String {
    format!("{}\n", nest("fn f() { let _x = 0; ", "", "}", size))
}

fn impls_in_fns(size: usize) -> String {
    format!(
        "{}\n",
        nest("fn f() { impl S { fn g() { ", "", "} } }", size)
    )
}

fn doc_comments(size: usize) -> String {
    in_body(&nest(
        &format!("{{ /// The first line.\n/** A block. */\n/// The last line.\nlet _x = 0; {PAD}"),
        "0",
        " }",
        size,
    ))
}

fn doc_comment_items(size: usize) -> String {
    let run = "/// A line of a long doc comment.\n".repeat(40);
    format!(
        "{}\nfn f() {{}}\n",
        nest(
            &format!("{run}pub mod a {{\n//! The module.\n"),
            "",
            "}\n",
            size
        )
    )
}

fn shebang(size: usize) -> String {
    format!("#!/usr/bin/env run-cargo-script\n{}", tuples(size))
}

fn family(
    name: &'static str,
    text: fn(usize) -> String,
    at: usize,
    over: raf_source::Budget,
    symbols: &'static [&'static str],
) -> Family {
    Family {
        name,
        text,
        at,
        over,
        symbols,
    }
}

/// Every family, each with its largest size that loads and the budget that
/// refuses one size more.
pub(crate) fn families() -> Vec<Family> {
    let mut all = nesting_families();
    all.extend(type_expression_and_item_families());
    all
}

/// Nesting that ends a list or a statement at every level, groups in macro
/// calls and attributes, and deep patterns in `match` alternatives.
fn nesting_families() -> Vec<Family> {
    use raf_source::Budget::{Delimiters, Recursion};
    let f: &[&str] = &["f"];
    vec![
        family(
            "closures_with_parameters",
            closures_with_parameters,
            91,
            Recursion,
            f,
        ),
        family("tuples", tuples, 101, Recursion, f),
        family("arrays", arrays, 101, Recursion, f),
        family("struct_literals", struct_literals, 101, Recursion, f),
        family("statement_blocks", statement_blocks, 101, Recursion, f),
        family("match_arms", match_arms, 96, Recursion, f),
        family("call_arguments", call_arguments, 101, Recursion, f),
        family("if_let_else_if", if_let_else_if, 76, Recursion, f),
        family("labelled_blocks", labelled_blocks, 96, Recursion, f),
        family("async_blocks", async_blocks, 91, Recursion, f),
        family("macro_call_groups", macro_call_groups, 126, Delimiters, f),
        family(
            "macro_rules_bodies",
            macro_rules_bodies,
            126,
            Delimiters,
            &["f", "r"],
        ),
        family(
            "attribute_arguments",
            attribute_arguments,
            126,
            Delimiters,
            f,
        ),
        family(
            "deep_pattern_alternatives",
            deep_pattern_alternatives,
            126,
            Delimiters,
            f,
        ),
    ]
}

/// Bounds and types, expression chains, lexical edge cases and nested items.
fn type_expression_and_item_families() -> Vec<Family> {
    use raf_source::Budget::{Delimiters, Recursion};
    let f: &[&str] = &["f"];
    let t: &[&str] = &["T"];
    vec![
        family("where_bounds", where_bounds, 46, Recursion, f),
        family(
            "impl_trait_arguments",
            impl_trait_arguments,
            101,
            Recursion,
            f,
        ),
        family(
            "nested_for_lifetimes",
            nested_for_lifetimes,
            83,
            Recursion,
            t,
        ),
        family("generic_types", generic_types, 58, Recursion, t),
        family("turbofish", turbofish, 58, Recursion, f),
        family(
            "reference_pointer_chains",
            reference_pointer_chains,
            384,
            Recursion,
            t,
        ),
        family("fn_pointer_chains", fn_pointer_chains, 481, Recursion, t),
        family("dyn_fn_chains", dyn_fn_chains, 62, Recursion, t),
        family(
            "tuple_and_array_types",
            tuple_and_array_types,
            56,
            Recursion,
            t,
        ),
        family("cast_chains", cast_chains, 1921, Recursion, f),
        family("unary_chains", unary_chains, 320, Recursion, f),
        family("try_chains", try_chains, 1921, Recursion, f),
        family("range_chains", range_chains, 640, Recursion, f),
        family("method_chains", method_chains, 960, Recursion, f),
        family("raw_strings", raw_strings, 960, Recursion, f),
        family(
            "chars_and_lifetimes",
            chars_and_lifetimes,
            639,
            Recursion,
            f,
        ),
        family(
            "nested_block_comments",
            nested_block_comments,
            1921,
            Recursion,
            f,
        ),
        family("raw_identifiers", raw_identifiers, 640, Recursion, f),
        family("doc_comments", doc_comments, 101, Recursion, f),
        family(
            "doc_comment_items",
            doc_comment_items,
            128,
            Delimiters,
            &["a", "f"],
        ),
        family("shebang", shebang, 100, Recursion, f),
        family("crlf_lines", crlf_lines, 1923, Recursion, f),
        family(
            "inline_modules",
            inline_modules,
            128,
            Delimiters,
            &["a", "f"],
        ),
        family("fns_in_blocks", fns_in_blocks, 128, Delimiters, f),
        family("impls_in_fns", impls_in_fns, 42, Delimiters, f),
    ]
}
