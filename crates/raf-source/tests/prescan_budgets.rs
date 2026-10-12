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
