// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The pre-scan refuses a file that passes a recursion budget, in process on
//! the caller's default test-thread stack, in debug and in release.

mod support;

use raf_source::{Budget, LoadErrorKind, RelPath, parse_source};
use support::shapes::{
    bangs, bars, block_condition_else_ifs, else_ifs, generics, lifetime_refs, parens, refs,
};

fn file() -> RelPath {
    RelPath::new("src/shape.rs").expect("path")
}

/// The budget a refusal names, after checking it names the file too.
fn refusal(text: &str) -> Option<Budget> {
    match parse_source(&file(), text, |_| ()) {
        Ok(()) => None,
        Err(error) => {
            assert!(error.to_string().contains("src/shape.rs"), "{error}");
            match error.kind {
                LoadErrorKind::Budget { budget, limit } => {
                    assert_eq!(limit, budget.limit());
                    Some(budget)
                }
                other => panic!("not a budget refusal: {other:?}"),
            }
        }
    }
}

fn parses(text: &str) {
    if let Some(budget) = refusal(text) {
        panic!("refused for {budget}: {}", &text[..text.len().min(80)]);
    }
}

#[test]
fn five_hundred_nested_generics_are_refused_beside_sixty_four() {
    assert_eq!(refusal(&generics(500)), Some(Budget::Angles));
    assert_eq!(refusal(&generics(65)), Some(Budget::Angles));
    parses(&generics(64));
}

#[test]
fn four_thousand_bangs_are_refused_beside_two_hundred_fifty_six() {
    assert_eq!(refusal(&bangs(4000)), Some(Budget::PrefixOperators));
    assert_eq!(refusal(&bangs(257)), Some(Budget::PrefixOperators));
    parses(&bangs(256));
}

#[test]
fn twenty_thousand_reference_marks_are_refused() {
    assert_eq!(refusal(&refs(20000)), Some(Budget::PrefixOperators));
    // A double reference mark is one token, so 256 of them are at the budget.
    parses(&refs(512));
    assert_eq!(refusal(&refs(514)), Some(Budget::PrefixOperators));
}

#[test]
fn ten_thousand_closure_bars_are_refused_beside_one_hundred_twenty_eight() {
    assert_eq!(refusal(&bars(10000)), Some(Budget::ClosureBars));
    assert_eq!(refusal(&bars(129)), Some(Budget::ClosureBars));
    parses(&bars(128));
}

#[test]
fn closure_heads_with_parameters_chain_like_empty_ones() {
    let chain = |n: usize| format!("fn f() {{ let _ = {}0; }}\n", "|a| ".repeat(n));
    assert_eq!(refusal(&chain(129)), Some(Budget::ClosureBars));
    parses(&chain(128));
}

#[test]
fn twenty_thousand_else_if_links_are_refused_beside_two_hundred_fifty_six() {
    assert_eq!(refusal(&else_ifs(20000)), Some(Budget::ElseIf));
    assert_eq!(refusal(&else_ifs(257)), Some(Budget::ElseIf));
    parses(&else_ifs(256));
}

#[test]
fn delimiter_nesting_is_refused_at_one_hundred_twenty_nine_beside_one_hundred_twenty_eight() {
    assert_eq!(refusal(&parens(129)), Some(Budget::Delimiters));
    parses(&parens(128));
    let brackets = |n: usize| format!("const X: u8 = {}0{};\n", "[".repeat(n), "]".repeat(n));
    assert_eq!(refusal(&brackets(129)), Some(Budget::Delimiters));
}

#[test]
fn the_refusal_names_the_line_where_the_count_went_over() {
    let text = format!("fn g() {{}}\n\n{}", generics(65));
    let error = parse_source(&file(), &text, |_| ()).expect_err("refused");
    assert_eq!(error.line, Some(3));
    assert_eq!(error.file, file());
}

#[test]
fn neighbouring_valid_shapes_just_below_each_budget_parse() {
    parses(&format!(
        "fn f(x: bool) {{ {} }}",
        "let _ = !x; ".repeat(300)
    ));
    parses(&format!("fn f() {{ {} }}", "a!(); ".repeat(300)));
    parses(&format!("fn f() {{ {} }}", "let _ = |a| a; ".repeat(300)));
    parses(&format!("fn f(a: bool) {{ {} }}", "if a {} ".repeat(300)));
    parses(&format!(
        "fn f(a: bool) {{ {} }}",
        "if a {} else {} ".repeat(300)
    ));
    parses(&format!(
        "fn f(a: u8, b: u8) {{ {} }}",
        "let _ = a < b; ".repeat(300)
    ));
    parses(&format!(
        "fn f(a: u8, b: u8) -> bool {{ {}true }}",
        "a < b && ".repeat(300)
    ));
    parses(&format!(
        "fn f(a: u8, b: u8) {{ {} }}",
        "if a < b { } ".repeat(300)
    ));
    parses("fn f<'a>(x: &'a str) -> &'a str { let _c = 'a'; let _d = '\\''; x }");
    parses(
        "fn f<'a, 'b: 'a>(x: &'a u8, y: &'b u8) -> char { 'outer: loop { break 'outer; } ; 'x' }",
    );
}

#[test]
fn tokens_inside_strings_raw_strings_and_comments_count_for_nothing() {
    let lots = |unit: &str| unit.repeat(1000);
    for text in [
        format!("const S: &str = \"{}\";", lots("!")),
        format!("const S: &str = \"{}\";", lots("A<")),
        format!("const S: &str = r#\"{}\"#;", lots("(")),
        format!("const S: &str = r##\"\"# {} \"##;", lots("((")),
        format!("const S: &[u8] = b\"{}\";", lots("(")),
        format!("const S: &[u8] = br#\"{}\"#;", lots("[")),
        format!("const S: &std::ffi::CStr = c\"{}\";", lots("{{")),
        format!("const S: &std::ffi::CStr = cr#\"{}\"#;", lots("{{")),
        format!("// {}\nconst X: u8 = 0;", lots("(")),
        format!("/* {} */ const X: u8 = 0;", lots("(")),
        format!(
            "/* a /* nested {} */ still comment {} */ const X: u8 = 0;",
            lots("("),
            lots("(")
        ),
        format!("/// {}\nconst X: u8 = 0;", lots("||")),
        format!("const S: &str = \"\\\"{}\";", lots("(")),
    ] {
        parses(&text);
    }
}

#[test]
fn a_comment_or_string_that_looks_like_the_end_of_another_does_not_hide_real_tokens() {
    // The real nesting follows the literal and must still be counted.
    let after_string = format!("const S: &str = \"x\"; {}", generics(65));
    assert_eq!(refusal(&after_string), Some(Budget::Angles));
    let after_comment = format!("/* ( */ {}", generics(65));
    assert_eq!(refusal(&after_comment), Some(Budget::Angles));
    let after_raw = format!("const S: &str = r#\"\"quoted\"\"#; {}", generics(65));
    assert_eq!(refusal(&after_raw), Some(Budget::Angles));
}

#[test]
fn a_lifetime_is_not_the_start_of_a_character_literal() {
    // If `'a` were read as a character literal it would swallow what follows.
    let text = format!("fn f<'a>(x: &'a u8) {{}} {}", generics(65));
    assert_eq!(refusal(&text), Some(Budget::Angles));
    let text = format!("const C: char = 'a'; {}", generics(65));
    assert_eq!(refusal(&text), Some(Budget::Angles));
    let text = format!("const C: char = '\\u{{1F600}}'; {}", generics(65));
    assert_eq!(refusal(&text), Some(Budget::Angles));
}

#[test]
fn a_block_as_the_condition_of_each_else_if_still_counts_the_chain() {
    assert_eq!(
        refusal(&block_condition_else_ifs(20000)),
        Some(Budget::ElseIf)
    );
    assert_eq!(
        refusal(&block_condition_else_ifs(257)),
        Some(Budget::ElseIf)
    );
    parses(&block_condition_else_ifs(256));
}

#[test]
fn separate_if_chains_in_one_block_do_not_add_up_beside_the_one_long_chain() {
    // Each chain is short; an identifier after the closing brace starts a new one.
    let separate = "if a {} else if a {} ".repeat(300);
    parses(&format!("fn f(a: bool) {{ {separate} }}"));
    let with_statements = "if a {} else if a {} let _x = 0; ".repeat(300);
    parses(&format!("fn f(a: bool) {{ {with_statements} }}"));
}

#[test]
fn a_lifetime_between_reference_marks_does_not_end_the_run() {
    assert_eq!(
        refusal(&lifetime_refs(20000)),
        Some(Budget::PrefixOperators)
    );
    assert_eq!(refusal(&lifetime_refs(257)), Some(Budget::PrefixOperators));
    parses(&lifetime_refs(256));
    // Lifetimes between other tokens still end nothing that was not a run.
    parses("fn f<'a>(x: &'a u8, y: &'a u8) -> &'a u8 { x }");
}

#[test]
fn a_syntax_error_reports_its_own_line_and_column_beside_a_parsing_neighbour() {
    let at_line_three = "fn ok() {}\n\nfn 1 {}\n";
    let error = parse_source(&file(), at_line_three, |_| ()).expect_err("syntax");
    assert_eq!(error.line, Some(3));
    assert!(
        matches!(&error.kind, LoadErrorKind::Syntax { column: 4, .. }),
        "{error}"
    );
    assert!(error.to_string().contains("src/shape.rs:3"), "{error}");

    let at_line_five = "fn ok() {}\nfn also_ok() {}\n\nstruct S {\n    x: }\n";
    let error = parse_source(&file(), at_line_five, |_| ()).expect_err("syntax");
    assert_eq!(error.line, Some(5));
    assert!(
        matches!(&error.kind, LoadErrorKind::Syntax { column: 8, .. }),
        "{error}"
    );

    // A failure the lexer finds, an unbalanced brace, also names its line.
    let unbalanced = "fn ok() {}\n\n\nfn broken() {\n";
    let error = parse_source(&file(), unbalanced, |_| ()).expect_err("syntax");
    assert!(
        matches!(error.kind, LoadErrorKind::Syntax { .. }),
        "{error}"
    );
    assert_eq!(error.line, Some(4));

    parses("fn ok() {}\n\nfn also_ok() {}\n");
}

/// The recursion limit.
fn limit() -> usize {
    Budget::Recursion.limit()
}

/// The recursion value of [`wrapped`] apart from its `return`s and its
/// content: `fn`, the `(` of the parameter list, the `{` of the body and its
/// open level, `let`, `=`, and the `{` of the block that holds the content
/// and its open level.
const WRAPPED_BASE: usize = 8;

/// `content` in a block after a chain of `returns` `return`s, so its value is
/// [`WRAPPED_BASE`] + `returns` + the value of `content` on its own.
fn wrapped(returns: usize, content: &str) -> String {
    format!(
        "fn f() {{ let _ = {}{{ {content} }}; }}\n",
        "return ".repeat(returns)
    )
}

/// `content`, whose own recursion value is `value`, raised to exactly the
/// limit parses, and one `return` more is refused with the recursion budget.
fn at_and_over(content: &str, value: usize) {
    let at = limit() - WRAPPED_BASE - value;
    parses(&wrapped(at, content));
    assert_eq!(
        refusal(&wrapped(at + 1, content)),
        Some(Budget::Recursion),
        "{content}"
    );
}

#[test]
fn the_recursion_limit_is_half_the_parse_stack_over_the_measured_cost_of_one_unit() {
    assert_eq!(
        limit(),
        raf_source::PARSE_STACK_BYTES / (2 * raf_source::RECURSION_UNIT_BYTES)
    );
    // The wrapper on its own is at the limit, and one more `return` is over.
    at_and_over("", 0);
}

#[test]
fn every_operator_counts_one_for_each_of_its_characters_at_the_limit_and_one_over() {
    for (content, value) in [
        ("!x", 1),
        ("-x", 1),
        ("*x", 1),
        // Each character of `&` and `<` counts two.
        ("&x", 2),
        ("&&x", 4),
        ("a | b", 1),
        ("a || b", 2),
        ("a < b", 2),
        ("a > b", 1),
        ("a + b", 1),
        ("a / b", 1),
        ("a % b", 1),
        ("a ^ b", 1),
        ("a = b", 1),
        ("x?", 1),
        ("x.a", 1),
        ("x.await", 1),
        ("a..b", 2),
        ("a..=b", 3),
        ("a += b", 2),
        ("a -= b", 2),
        ("a *= b", 2),
        ("a /= b", 2),
        ("a %= b", 2),
        ("a ^= b", 2),
        ("a &= b", 2),
        ("a |= b", 2),
        ("a <<= b", 4),
        ("a >>= b", 3),
        ("a == b", 2),
        ("a != b", 2),
        ("a <= b", 2),
        ("a >= b", 2),
        ("a << b", 4),
        ("a >> b", 2),
        ("a && b", 4),
        // `||` 2, `->` 2, and the `{` of the body and its open level.
        ("|| -> u8 { 0 }", 6),
        // `let`, `@` and `=`.
        ("let a @ b = 0;", 3),
        // The `{` of the block and its open level, `fn`, `(` and its open
        // level, and `...`.
        ("extern \"C\" { fn g(a: u8, ...); }", 8),
        // A number with a `.` after a `.` is two tuple indices.
        ("x.0.1", 2),
    ] {
        at_and_over(content, value);
    }
}

#[test]
fn every_counted_keyword_counts_one_at_the_limit_and_one_over() {
    for (content, value) in [
        ("x as u8", 1),
        ("async {}", 3),
        ("become g()", 3),
        ("let box a = 0;", 3),
        ("break x", 1),
        ("const || 0", 3),
        ("type T = dyn A;", 2),
        ("type T = fn();", 4),
        ("for x in y {}", 3),
        ("if a {}", 3),
        ("type T = impl A;", 2),
        ("let a = 0;", 2),
        ("loop {}", 3),
        ("mod m {}", 3),
        ("match a {}", 3),
        ("move || 0", 3),
        ("let mut a = 0;", 3),
        ("let ref a = 0;", 3),
        ("return x", 1),
        ("static X: u8 = 0;", 2),
        ("try {}", 3),
        ("unsafe {}", 3),
        ("while a {}", 3),
        ("yield x", 1),
    ] {
        at_and_over(content, value);
    }
}

#[test]
fn a_label_a_use_path_separator_and_each_delimiter_count_at_the_limit_and_one_over() {
    for (content, value) in [
        // The label, and the `{` of the block and its open level.
        ("'a: {}", 3),
        // A separator of a `use` path counts sixteen.
        ("use a::b;", 16),
        ("use a::b::c;", 32),
        ("use a::{b::c, d};", 34),
        // An opening `(`, `[` or `{` counts in the level around it and opens
        // one.
        ("g(x)", 2),
        ("x[0]", 2),
        ("{}", 2),
        // An attribute's `[` and the `!` of an inner attribute count nothing.
        ("#[a] fn g() {}", 4),
        ("#![a] fn g() {}", 4),
        // `::` outside a `use` declaration counts nothing.
        ("a::b::c", 0),
    ] {
        at_and_over(content, value);
    }
}

#[test]
fn a_level_returns_to_zero_where_its_constructs_end_beside_the_same_count_without_the_end() {
    // `fn`, `(`, the body's `{` and level, `match`, and the arms' `{` and
    // level; then the pattern's `box`es, which `=>` ends, and the arm's
    // `return`s.
    let arm = |boxes: usize, returns: usize| {
        format!(
            "fn f() {{ match x {{ {}a => {}x }} }}\n",
            "box ".repeat(boxes),
            "return ".repeat(returns)
        )
    };
    let side = limit() - 7;
    parses(&arm(side, side));
    assert_eq!(refusal(&arm(side + 1, side)), Some(Budget::Recursion));
    assert_eq!(refusal(&arm(side, side + 1)), Some(Budget::Recursion));

    // `fn`, `(`, the body's `{` and level, `let`, `=`, the tuple's `(` and its
    // level; then each element, which `,` ends.
    let tuple = |left: usize, right: usize| {
        format!(
            "fn f() {{ let _ = ({}x, {}x); }}\n",
            "return ".repeat(left),
            "return ".repeat(right)
        )
    };
    let side = limit() - 8;
    parses(&tuple(side, side));
    assert_eq!(refusal(&tuple(side, side + 1)), Some(Budget::Recursion));

    // A statement that ends in a block, followed by one that starts with an
    // identifier: `fn`, `(`, the body's `{` and level, then `if`, its
    // condition and its block's `{` and level, then the next statement.
    let statements = |condition: usize, next: usize| {
        format!(
            "fn f() {{ if {}0 {{}} {}x; }}\n",
            "return ".repeat(condition),
            "return ".repeat(next)
        )
    };
    let condition = limit() - 7;
    let next = limit() - 4;
    parses(&statements(condition, next));
    assert_eq!(
        refusal(&statements(condition + 1, next)),
        Some(Budget::Recursion)
    );
    assert_eq!(
        refusal(&statements(condition, next + 1)),
        Some(Budget::Recursion)
    );

    // Many statements, elements and arms in one level each count alone.
    parses(&format!(
        "fn f(x: bool) {{ {} }}",
        "let _ = !x; ".repeat(5000)
    ));
    parses(&format!("const X: [i8; 5000] = [{}];", "-1, ".repeat(5000)));
    parses(&format!(
        "fn f(x: u8) {{ match x {{ {} _ => {{}} }} }}",
        "0 => return return 1, ".repeat(5000)
    ));
    parses(&format!(
        "fn f(a: bool) {{ {} }}",
        "if a { } else { } let _x = &&a; ".repeat(5000)
    ));
    parses(&format!(
        "mod m {{ {} }}",
        "fn g() -> u8 { 0 } ".repeat(5000)
    ));
}

#[test]
fn a_construct_that_continues_past_a_comma_or_a_closing_brace_keeps_its_count() {
    // A `,` inside an open `<`: `=`, `<`, then the `&`s after the comma, and
    // the closing `>`.
    at_and_over("type T = A<u8, & & & u8>;", 10);
    // A `,` inside a closure head: both bars and the `!`.
    at_and_over("|a, b| !x", 3);
    // A `,` inside a `where` clause: `fn`, `(`, `where`, then the body's `{`
    // and its level.
    at_and_over("fn g() where T: X, U: Y {}", 5);
    // `else`, `in` and `as` after a `}` continue the construct.
    at_and_over(
        "if a {} else if a {} else if a {} else if a {} else if a {} else {}",
        12,
    );
    at_and_over("for S { a } in return 0 {}", 5);
    at_and_over("return return S {} as u8 as u8", 5);

    // Chains whose links each close a brace or hold a comma are counted.
    let for_chain = |n: usize| {
        format!(
            "fn f() {{ {}x {} }}\n",
            "for S { a } in ".repeat(n),
            "{}".repeat(n)
        )
    };
    parses(&for_chain(100));
    assert_eq!(refusal(&for_chain(limit())), Some(Budget::Recursion));
    // Reference marks before each generic argument list, whose commas would
    // otherwise end the count of the marks before them.
    let generic_chain = |marks: usize, n: usize| {
        format!(
            "type T = {}u8{};\n",
            format!("{}A<u8, ", "& ".repeat(marks)).repeat(n),
            ">".repeat(n)
        )
    };
    parses(&generic_chain(4, 64));
    assert_eq!(refusal(&generic_chain(200, 64)), Some(Budget::Recursion));
    let compare_chain = |n: usize| {
        format!(
            "fn f() {{ let _ = {}x; }}\n",
            "a < return x > b && ".repeat(n)
        )
    };
    parses(&compare_chain(10));
    assert_eq!(refusal(&compare_chain(limit())), Some(Budget::Recursion));
}

#[test]
fn a_macro_call_counts_only_its_groups_and_a_match_pattern_only_its_alternatives() {
    // A macro call's `!` counts, and inside its delimiters only the levels
    // that its groups open.
    at_and_over("m!(!x !x !x)", 2);
    at_and_over("m![a, [b]]", 3);
    at_and_over("macro_rules! r { () => { !x } }", 3);
    // A keyword before `!` makes it an operator, not a macro call.
    at_and_over("if !(x) {}", 5);
    at_and_over("return !(x)", 4);
    // In the arms of a `match`, a `|` of a pattern counts nothing.
    at_and_over("match a { b | c | d => 0 }", 3);
    at_and_over("match f(x) { a | b => 0 }", 4);
    // A guard is an expression, and so is a block before the arms.
    at_and_over("match a { b if c | d | e => 0 }", 6);
    at_and_over("match unsafe { a } { b | c => 0 }", 6);

    let in_macro = format!("fn f() {{ m!({}); }}\n", "!x && y, ".repeat(20000));
    parses(&in_macro);
    let alternatives = format!(
        "fn f(c: char) -> bool {{ match c {{ {}'z' => true, _ => false }} }}\n",
        "'a'..='b' | ".repeat(20000)
    );
    parses(&alternatives);
    let guard = format!(
        "fn f() {{ match c {{ _ if {}a => 0 }} }}\n",
        "a | ".repeat(limit())
    );
    assert_eq!(refusal(&guard), Some(Budget::Recursion));
    let block = format!(
        "fn f() {{ match {{ {}a }} {{ _ => 0 }} }}\n",
        "a | ".repeat(limit())
    );
    assert_eq!(refusal(&block), Some(Budget::Recursion));
}

#[test]
fn chains_that_overflowed_the_parse_stack_are_refused_by_the_recursion_budget() {
    for shape in support::shapes::chains()
        .into_iter()
        .chain(support::shapes::composites())
    {
        assert_eq!(
            refusal(&shape.text),
            Some(Budget::Recursion),
            "{}",
            shape.name
        );
    }
    for shape in support::shapes::uncounted() {
        let budget = if shape.name == "uncounted_move_closure" {
            Budget::ClosureBars
        } else {
            Budget::Recursion
        };
        assert_eq!(refusal(&shape.text), Some(budget), "{}", shape.name);
    }
}

#[test]
fn the_eighty_kilobyte_box_pattern_is_refused_in_under_a_second() {
    let text = support::shapes::box_patterns(20000);
    assert!(text.len() >= 80_000);
    let started = std::time::Instant::now();
    assert_eq!(refusal(&text), Some(Budget::Recursion));
    assert!(started.elapsed() < std::time::Duration::from_secs(1));
    // The same pattern at a depth the budget admits parses.
    parses(&support::shapes::box_patterns(100));
}

#[test]
fn the_recursion_refusal_names_the_line_where_the_count_went_over() {
    let over = support::shapes::returns(limit() - support::shapes::RETURNS_BASE + 1);
    let text = format!("fn g() {{}}\n\n{over}");
    let error = parse_source(&file(), &text, |_| ()).expect_err("refused");
    assert_eq!(error.line, Some(3));
    assert!(
        matches!(error.kind, LoadErrorKind::Budget { budget: Budget::Recursion, limit: l } if l == limit()),
        "{error}"
    );
    parses(&support::shapes::returns(
        limit() - support::shapes::RETURNS_BASE,
    ));
}
