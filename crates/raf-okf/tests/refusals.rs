// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Every refusal of the reader, each beside a neighbouring accepted case.

use std::fmt::Write as _;

use raf_okf::{
    Document, ESCAPES, ErrorKind, ItemValue, MAX_TEXT_BYTES, MAX_YAML_DEPTH, ParseError,
    ScalarStyle, Value, plain_ok,
};

/// A document whose frontmatter is `front` (which ends in a newline).
fn doc(front: &str) -> String {
    format!("---\n{front}---\n\nbody\n")
}

fn refused(text: &str) -> ParseError {
    match Document::parse(text) {
        Ok(_) => panic!("expected a refusal for {text:?}"),
        Err(error) => error,
    }
}

fn accepted(text: &str) -> Document {
    match Document::parse(text) {
        Ok(document) => document,
        Err(error) => panic!("expected {text:?} to parse, got {error}"),
    }
}

/// The scalar text of the top-level key `key`.
fn text_of(document: &Document, key: &str) -> String {
    let entry = document
        .frontmatter()
        .map()
        .get(key)
        .unwrap_or_else(|| panic!("no key {key}"));
    match entry.value() {
        Value::Scalar(scalar) => scalar.text().to_string(),
        other => panic!("{key} is not a scalar: {other:?}"),
    }
}

#[test]
fn a_document_without_a_fence_on_line_one_is_refused() {
    let error = refused("type: Architecture\n");
    assert_eq!(error.kind(), &ErrorKind::MissingFence);
    assert_eq!(
        (error.line(), error.column(), error.offset()),
        (1, Some(1), Some(0))
    );
    assert_eq!(
        refused("\n---\na: b\n---\n").kind(),
        &ErrorKind::MissingFence
    );
    assert_eq!(
        refused("---- \na: b\n---\n").kind(),
        &ErrorKind::MissingFence
    );
    accepted("---\na: b\n---\n");
}

#[test]
fn an_unclosed_fence_is_refused_at_the_opening_fence() {
    let error = refused("---\na: b\nc: d\n");
    assert_eq!(error.kind(), &ErrorKind::UnclosedFence);
    assert_eq!(error.line(), 1);
    assert_eq!(refused("---").kind(), &ErrorKind::UnclosedFence);
    assert_eq!(refused("---\n").kind(), &ErrorKind::UnclosedFence);
    // A closing line with trailing text is not a fence.
    assert_eq!(
        refused("---\na: b\n--- x\n").kind(),
        &ErrorKind::UnclosedFence
    );
    accepted("---\na: b\n---");
}

#[test]
fn a_frontmatter_that_is_not_a_block_mapping_is_refused() {
    for front in [
        "- a\n- b\n",
        "-\n",
        "[a, b]\n",
        "{a: b}\n",
        "just text\n",
        "# only a comment\n",
        "",
    ] {
        let error = refused(&doc(front));
        assert_eq!(error.kind(), &ErrorKind::NotAMapping, "{front:?}");
        assert!(error.line() >= 2, "{front:?}");
    }
    assert_eq!(refused(&doc("- a\n")).line(), 2);
    assert_eq!(refused(&doc("# c\n- a\n")).line(), 3);
    accepted(&doc("a: b\n"));
}

#[test]
fn a_flow_mapping_value_is_refused_naming_the_key() {
    let error = refused(&doc("type: Capability\nbindings: {code: []}\n"));
    assert_eq!(
        error.kind(),
        &ErrorKind::FlowMapping {
            key: "bindings".to_string()
        }
    );
    assert_eq!((error.line(), error.column()), (3, Some(11)));
    let inside_flow_list = refused(&doc("tags: [{a: b}]\n"));
    assert_eq!(
        inside_flow_list.kind(),
        &ErrorKind::FlowMapping {
            key: "tags".to_string()
        }
    );
    let as_item = refused(&doc("items:\n  - {claim: x}\n"));
    assert_eq!(
        as_item.kind(),
        &ErrorKind::FlowMapping {
            key: "items".to_string()
        }
    );
    accepted(&doc(
        "type: Capability\nbindings:\n  code: []\n  tests: []\n",
    ));
}

#[test]
fn a_duplicate_key_is_refused_at_every_level() {
    let top = refused(&doc("type: A\ntitle: x\ntype: B\n"));
    assert_eq!(
        top.kind(),
        &ErrorKind::DuplicateKey {
            key: "type".to_string()
        }
    );
    assert_eq!((top.line(), top.column()), (4, Some(1)));

    let nested = refused(&doc("bindings:\n  code: []\n  tests: []\n  code: []\n"));
    assert_eq!(
        nested.kind(),
        &ErrorKind::DuplicateKey {
            key: "code".to_string()
        }
    );
    assert_eq!((nested.line(), nested.column()), (5, Some(3)));

    let in_item = refused(&doc("items:\n  - claim: a\n    outcome: x\n    claim: b\n"));
    assert_eq!(
        in_item.kind(),
        &ErrorKind::DuplicateKey {
            key: "claim".to_string()
        }
    );
    assert_eq!(in_item.line(), 5);

    let on_item_line = refused(&doc("items:\n  - claim: a\n    claim: b\n"));
    assert_eq!(
        on_item_line.kind(),
        &ErrorKind::DuplicateKey {
            key: "claim".to_string()
        }
    );

    // The same key in different mappings, and in two items, is fine.
    accepted(&doc(
        "type: A\nbindings:\n  code: []\nitems:\n  - claim: a\n  - claim: b\ncode: x\n",
    ));
}

#[test]
fn anchors_aliases_tags_and_merge_keys_are_refused_naming_the_token() {
    let anchor = refused(&doc("a: &x value\n"));
    assert_eq!(
        anchor.kind(),
        &ErrorKind::Anchor {
            name: "x".to_string()
        }
    );
    assert_eq!((anchor.line(), anchor.column()), (2, Some(4)));
    assert_eq!(
        refused(&doc("a:\n  - &y value\n")).kind(),
        &ErrorKind::Anchor {
            name: "y".to_string()
        }
    );
    assert_eq!(
        refused(&doc("a: [&z b]\n")).kind(),
        &ErrorKind::Anchor {
            name: "z".to_string()
        }
    );
    assert_eq!(
        refused(&doc("&k key: value\n")).kind(),
        &ErrorKind::Anchor {
            name: "k".to_string()
        }
    );
    let alias = refused(&doc("a: *x\n"));
    assert_eq!(
        alias.kind(),
        &ErrorKind::Alias {
            name: "x".to_string()
        }
    );
    assert_eq!(
        refused(&doc("a: [*x]\n")).kind(),
        &ErrorKind::Alias {
            name: "x".to_string()
        }
    );
    assert_eq!(
        refused(&doc("a: !x value\n")).kind(),
        &ErrorKind::Tag {
            name: "x".to_string()
        }
    );
    assert_eq!(
        refused(&doc("a: !!str value\n")).kind(),
        &ErrorKind::Tag {
            name: "!str".to_string()
        }
    );
    let merge = refused(&doc("a: b\n<<: c\n"));
    assert_eq!(merge.kind(), &ErrorKind::MergeKey);
    assert_eq!((merge.line(), merge.column()), (3, Some(1)));
    assert_eq!(
        refused(&doc("items:\n  - <<: c\n")).kind(),
        &ErrorKind::MergeKey
    );
    // The same characters inside a value, or quoted, are plain text.
    let neighbour = accepted(&doc(
        "a: x&y\nb: \"&x\"\nc: x*y\nd: a!b\ne: v<<w\nf: '*x'\ng: \"!x\"\n",
    ));
    assert_eq!(text_of(&neighbour, "a"), "x&y");
    assert_eq!(text_of(&neighbour, "b"), "&x");
    assert_eq!(text_of(&neighbour, "g"), "!x");
}

#[test]
fn block_scalars_are_refused_naming_the_header() {
    for header in ["|", ">", "|-", ">+", "|2", ">-1"] {
        let error = refused(&doc(&format!("a: {header}\n  text\n")));
        assert_eq!(
            error.kind(),
            &ErrorKind::BlockScalar {
                header: header.to_string()
            },
            "{header}"
        );
        assert_eq!((error.line(), error.column()), (2, Some(4)));
    }
    assert_eq!(
        refused(&doc("a:\n  - |\n")).kind(),
        &ErrorKind::BlockScalar {
            header: "|".to_string()
        }
    );
    // An indicator that is not a header is an indicator character.
    assert_eq!(
        refused(&doc("a: >text\n")).kind(),
        &ErrorKind::IndicatorCharacter { indicator: '>' }
    );
    // Quoted, or in the middle of text, they are plain text.
    let neighbour = accepted(&doc("a: \"|\"\nb: x|y\nc: 'a > b'\n"));
    assert_eq!(text_of(&neighbour, "a"), "|");
    assert_eq!(text_of(&neighbour, "b"), "x|y");
}

#[test]
fn unknown_escapes_are_refused_and_every_known_escape_is_read() {
    assert_eq!(
        ESCAPES,
        [
            ('\\', '\\'),
            ('"', '"'),
            ('n', '\n'),
            ('r', '\r'),
            ('t', '\t'),
            ('0', '\0')
        ]
    );
    // Native ECHAR also decodes these, but this profile deliberately refuses
    // them at the original backslash location.
    for escape in ['b', 'f', '\''] {
        let error = refused(&doc(&format!("a: \"\\{escape}\"\n")));
        assert_eq!(error.kind(), &ErrorKind::UnknownEscape { escape });
        assert_eq!((error.line(), error.column()), (2, Some(5)));
    }
    assert_eq!(text_of(&accepted(&doc("a: \"'\"\n")), "a"), "'");
    let unknown = refused(&doc("a: \"\\q\"\n"));
    assert_eq!(unknown.kind(), &ErrorKind::UnknownEscape { escape: 'q' });
    assert_eq!((unknown.line(), unknown.column()), (2, Some(5)));
    assert_eq!(
        refused(&doc("a: \"\\x41\"\n")).kind(),
        &ErrorKind::UnknownEscape { escape: 'x' }
    );
    assert_eq!(
        refused(&doc("a: [\"ok\", \"\\u0041\"]\n")).kind(),
        &ErrorKind::UnknownEscape { escape: 'u' }
    );
    // Single-quoted text has no escapes: a backslash is itself.
    assert_eq!(text_of(&accepted(&doc("a: '\\q'\n")), "a"), "\\q");
    for (name, value) in ESCAPES {
        let text = doc(&format!("a: \"x\\{name}y\"\n"));
        assert_eq!(text_of(&accepted(&text), "a"), format!("x{value}y"));
    }
}

#[test]
fn an_unterminated_quote_and_text_after_a_quote_are_refused() {
    assert_eq!(
        refused(&doc("a: \"open\n")).kind(),
        &ErrorKind::UnterminatedQuote
    );
    assert_eq!(
        refused(&doc("a: 'open\n")).kind(),
        &ErrorKind::UnterminatedQuote
    );
    assert_eq!(
        refused(&doc("a: \"x\" y\n")).kind(),
        &ErrorKind::TextAfterQuote
    );
    assert_eq!(
        refused(&doc("a: [\"x\" y]\n")).kind(),
        &ErrorKind::TextAfterQuote
    );
    assert_eq!(text_of(&accepted(&doc("a: \"x\" # c\n")), "a"), "x");
}

#[test]
fn flow_list_shape_errors_are_refused() {
    assert_eq!(
        refused(&doc("a: [x, y\n")).kind(),
        &ErrorKind::UnterminatedFlow
    );
    assert_eq!(
        refused(&doc("a: [x, , y]\n")).kind(),
        &ErrorKind::EmptyFlowEntry
    );
    assert_eq!(
        refused(&doc("a: [x,]\n")).kind(),
        &ErrorKind::EmptyFlowEntry
    );
    assert_eq!(
        refused(&doc("a: [x] y\n")).kind(),
        &ErrorKind::TextAfterFlow
    );
    assert_eq!(refused(&doc("a: [[x]]\n")).kind(), &ErrorKind::NestedFlow);
    assert_eq!(
        refused(&doc("a:\n  - [x]\n")).kind(),
        &ErrorKind::NestedFlow
    );
    let list = accepted(&doc("a: [x, \"y z\", 'w']\nb: []\n"));
    let Value::List(items) = list.frontmatter().map().get("a").expect("a").value() else {
        panic!("a is not a list");
    };
    assert_eq!(items.items().len(), 3);
}

#[test]
fn plain_scalars_with_a_colon_space_or_an_indicator_are_refused() {
    assert_eq!(
        refused(&doc("a: b: c\n")).kind(),
        &ErrorKind::PlainColonSpace
    );
    assert_eq!(refused(&doc("a: b:\n")).kind(), &ErrorKind::PlainColonSpace);
    for indicator in ['%', '@', '`'] {
        assert_eq!(
            refused(&doc(&format!("a: {indicator}x\n"))).kind(),
            &ErrorKind::IndicatorCharacter { indicator }
        );
    }
    assert_eq!(text_of(&accepted(&doc("a: b:c\n")), "a"), "b:c");
    assert_eq!(text_of(&accepted(&doc("a: x%y\n")), "a"), "x%y");
}

#[test]
fn layout_errors_are_refused_with_their_position() {
    let spacing = refused(&doc("a:  b\n"));
    assert_eq!(spacing.kind(), &ErrorKind::Spacing);
    assert_eq!((spacing.line(), spacing.column()), (2, Some(4)));
    assert_eq!(
        refused(&doc("a: b \n")).kind(),
        &ErrorKind::TrailingWhitespace
    );
    assert_eq!(
        refused(&doc("a: b\n \n")).kind(),
        &ErrorKind::TrailingWhitespace
    );
    assert_eq!(
        refused(&doc("a:\tb\n")).kind(),
        &ErrorKind::ControlCharacter
    );
    assert_eq!(
        refused(&doc("a: b\n b: c\n")).kind(),
        &ErrorKind::Indentation
    );
    assert_eq!(
        refused(&doc("a:\n    b: c\n")).kind(),
        &ErrorKind::Indentation
    );
    assert_eq!(
        refused(&doc("a:\n  b: c\n    d: e\n")).kind(),
        &ErrorKind::Indentation
    );
    assert_eq!(refused(&doc("a:b\n")).kind(), &ErrorKind::ExpectedKey);
    assert_eq!(
        refused(&doc("a: b\nplain\n")).kind(),
        &ErrorKind::ExpectedKey
    );
    assert_eq!(
        refused(&doc("a: b\nbad key: c\n")).kind(),
        &ErrorKind::InvalidKey {
            key: "bad key".to_string()
        }
    );
    assert_eq!(
        refused(&doc("a:\n  - x\n  y: z\n")).kind(),
        &ErrorKind::ExpectedItem
    );
    assert_eq!(refused(&doc("a:\n  -\n")).kind(), &ErrorKind::EmptyItem);
    assert_eq!(
        refused(&doc("a:\n  - - x\n")).kind(),
        &ErrorKind::NestedSequence
    );
    assert_eq!(refused(&doc("a:\n  -  x\n")).kind(), &ErrorKind::Spacing);
    assert_eq!(
        refused(&doc("a:\n  - x\n  - k: v\n")).kind(),
        &ErrorKind::MixedList
    );
    accepted(&doc("a:\n  b: c\n  d: e\nf: g\n"));
}

#[test]
fn line_endings_must_agree_within_the_frontmatter() {
    let crlf = "---\r\na: b\r\n---\r\nbody\r\n";
    accepted(crlf);
    assert_eq!(
        refused("---\r\na: b\nc: d\r\n---\r\n").kind(),
        &ErrorKind::MixedLineEndings
    );
    assert_eq!(
        refused("---\na: b\r\n---\n").kind(),
        &ErrorKind::MixedLineEndings
    );
    assert_eq!(
        refused("---\r\na: b\r\n---\n").kind(),
        &ErrorKind::MixedLineEndings
    );
    // The body is free to use any line ending.
    accepted("---\na: b\n---\nbody\r\nmore\n");
}

#[test]
fn numbers_booleans_and_null_are_read_as_strings() {
    let document = accepted(&doc(
        "int: 1\nfloat: 1.5\nhex: 0x1F\nexp: 1e3\nyes: true\nnone: null\ntilde: ~\ninf: .inf\ndate: 2026-01-01\n",
    ));
    for (key, text) in [
        ("int", "1"),
        ("float", "1.5"),
        ("hex", "0x1F"),
        ("exp", "1e3"),
        ("yes", "true"),
        ("none", "null"),
        ("tilde", "~"),
        ("inf", ".inf"),
        ("date", "2026-01-01"),
    ] {
        let entry = document.frontmatter().map().get(key).expect("key");
        let scalar = entry.value().as_scalar().expect("scalar");
        assert_eq!(scalar.text(), text, "{key}");
        assert_eq!(scalar.style(), ScalarStyle::Plain, "{key}");
    }
}

#[test]
fn a_text_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused() {
    let head = "---\nk: v\n---\n";
    let at_bound = format!("{head}{}", "x".repeat(MAX_TEXT_BYTES - head.len()));
    assert_eq!(at_bound.len(), MAX_TEXT_BYTES);
    let document = accepted(&at_bound);
    assert_eq!(document.body().text().len(), MAX_TEXT_BYTES - head.len());

    let over = format!("{at_bound}x");
    let error = refused(&over);
    assert_eq!(
        error.kind(),
        &ErrorKind::TooLarge {
            len: MAX_TEXT_BYTES + 1,
            max: MAX_TEXT_BYTES
        }
    );
    assert_eq!(error.line(), 4);
    assert_eq!(error.offset(), Some(MAX_TEXT_BYTES));
    assert_eq!(error.column(), None);

    // The bound counts bytes of the whole text, wherever the extra byte lies.
    let multibyte = format!("{head}{}", "é".repeat(MAX_TEXT_BYTES));
    assert_eq!(refused(&multibyte).offset(), Some(MAX_TEXT_BYTES));
}

/// Maps nested `levels` deep: level 1 is the top-level mapping.
fn nested_maps(levels: usize) -> String {
    let mut front = String::new();
    for level in 1..levels {
        front.push_str(&"  ".repeat(level - 1));
        writeln!(front, "k{level}:").expect("writing to a String cannot fail");
    }
    front.push_str(&"  ".repeat(levels - 1));
    front.push_str("leaf: v\n");
    front
}

#[test]
fn nesting_over_thirty_two_levels_is_refused_before_descent() {
    accepted(&doc(&nested_maps(MAX_YAML_DEPTH)));
    let error = refused(&doc(&nested_maps(MAX_YAML_DEPTH + 1)));
    assert_eq!(
        error.kind(),
        &ErrorKind::DepthExceeded {
            max: MAX_YAML_DEPTH
        }
    );
    // The 33rd level is the 33rd content line, which is document line 34.
    assert_eq!((error.line(), error.column()), (34, Some(65)));

    // A flow list is one more level: 31 mappings hold one, 32 do not.
    accepted(&doc(
        &nested_maps(MAX_YAML_DEPTH - 1).replace("leaf: v", "leaf: [a]")
    ));
    let too_deep_flow = nested_maps(MAX_YAML_DEPTH).replace("leaf: v", "leaf: [a]");
    assert_eq!(
        refused(&doc(&too_deep_flow)).kind(),
        &ErrorKind::DepthExceeded {
            max: MAX_YAML_DEPTH
        }
    );
}

#[test]
fn a_hundred_thousand_levels_return_an_error_on_a_small_stack() {
    let run = |front: String| {
        std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(move || Document::parse(&doc(&front)).map(|_| ()))
            .expect("spawn")
            .join()
            .expect("the parse must not overflow the stack")
    };
    let flow = run(format!("k: {}\n", "[".repeat(100_000)));
    assert_eq!(
        flow.expect_err("deep flow").kind(),
        &ErrorKind::DepthExceeded {
            max: MAX_YAML_DEPTH
        }
    );
    let items = run(format!("k:\n  {}x\n", "- ".repeat(100_000)));
    assert_eq!(
        items.expect_err("deep items").kind(),
        &ErrorKind::NestedSequence
    );
    let spaced = run(format!("k: {}x\n", "[ ".repeat(100_000)));
    assert_eq!(
        spaced.expect_err("deep spaced flow").kind(),
        &ErrorKind::DepthExceeded {
            max: MAX_YAML_DEPTH
        }
    );
}

#[test]
fn a_billion_laughs_document_is_refused_at_the_first_anchor() {
    let text = doc("a: &a [\"lol\", \"lol\"]\nb: &b [*a, *a]\nc: &c [*b, *b]\nd: &d [*c, *c]\n");
    let error = refused(&text);
    assert_eq!(
        error.kind(),
        &ErrorKind::Anchor {
            name: "a".to_string()
        }
    );
    assert_eq!((error.line(), error.column()), (2, Some(4)));
}

#[test]
fn items_keep_their_maps_and_scalars() {
    let document = accepted(&doc(
        "items:\n  - claim: one\n    outcome: implemented\n  - claim: two\n",
    ));
    let Value::List(list) = document
        .frontmatter()
        .map()
        .get("items")
        .expect("items")
        .value()
    else {
        panic!("items is not a list");
    };
    let claims: Vec<&str> = list
        .items()
        .iter()
        .map(|item| match item.value() {
            ItemValue::Map(map) => map
                .get("claim")
                .expect("claim")
                .value()
                .as_scalar()
                .expect("scalar")
                .text(),
            ItemValue::Scalar(_) => panic!("a scalar item"),
        })
        .collect();
    assert_eq!(claims, ["one", "two"]);
}

#[test]
fn text_that_plain_ok_accepts_is_read_back_plain_in_every_position() {
    let candidates = [
        "word",
        "two words",
        "a-b_c",
        "cap:okf-profile",
        "x&y",
        "x*y",
        "a!b",
        "1.5",
        "true",
        "&anchor",
        "*alias",
        "!tag",
        "|",
        ">-",
        "| x",
        "> x",
        "%x",
        "@x",
        "`x",
        "#x",
        "a: b",
        "a:",
        "x #y",
        "- x",
        "-",
        "?",
        ": x",
        "[x]",
        "{x}",
        "a,b",
        " x",
        "x ",
        "\"x\"",
        "'x'",
        "",
        "<<",
        "a\u{1}b",
        "é",
        "-1",
        "key: value",
        "a.b",
        "path/to/x",
        "x=y",
        "~",
        "null",
        "+1",
        "a#b",
        "(x)",
    ];
    let mut accepted_count = 0;
    for text in candidates {
        if plain_ok(text) {
            accepted_count += 1;
            for source in [
                format!("k: {text}\n"),
                format!("k: [{text}]\n"),
                format!("k:\n  - {text}\n"),
            ] {
                let document = Document::parse(&doc(&source))
                    .unwrap_or_else(|error| panic!("{text:?} in {source:?}: {error}"));
                let entry = document.frontmatter().map().get("k").expect("k");
                let scalar = match entry.value() {
                    Value::Scalar(scalar) => scalar.clone(),
                    Value::List(list) => {
                        list.items()[0].value().as_scalar().expect("scalar").clone()
                    }
                    Value::Map(_) => panic!("a map"),
                };
                assert_eq!(scalar.text(), text);
                assert_eq!(scalar.style(), ScalarStyle::Plain);
            }
        }
    }
    assert!(accepted_count >= 15, "the fixtures must exercise plain_ok");
}

#[test]
fn text_that_the_reader_refuses_is_never_plain_ok() {
    for text in [
        "&x", "*x", "!x", "|", ">", "|-", ">+", "%x", "@x", "`x", "a: b", "a:", "#x", "- ", "-",
        "?", ": x", "x #y", "\"x", "'x", "[x", "{x}", "a,b", "a\tb", "",
    ] {
        assert!(!plain_ok(text), "{text:?}");
    }
}

#[test]
fn unterminated_quotes_are_refused_in_flow_list_item_and_nested_item_positions() {
    let in_flow = refused(&doc("code: [x, \"y]\n"));
    assert_eq!(in_flow.kind(), &ErrorKind::UnterminatedQuote);
    assert_eq!((in_flow.line(), in_flow.column()), (2, Some(11)));
    let as_item = refused(&doc("tests:\n  - \"cap\n"));
    assert_eq!(as_item.kind(), &ErrorKind::UnterminatedQuote);
    assert_eq!((as_item.line(), as_item.column()), (3, Some(5)));
    let nested = refused(&doc("bindings:\n  code:\n    - 'src\n"));
    assert_eq!(nested.kind(), &ErrorKind::UnterminatedQuote);
    assert_eq!((nested.line(), nested.column()), (4, Some(7)));
    // The closed neighbours parse.
    accepted(&doc(
        "code: [x, \"y\"]\ntests:\n  - \"cap\"\nbindings:\n  code:\n    - 'src'\n",
    ));
}

#[test]
fn a_colon_space_inside_a_flow_list_is_refused() {
    let error = refused(&doc("code: [x, a: b]\n"));
    assert_eq!(error.kind(), &ErrorKind::PlainColonSpace);
    assert_eq!((error.line(), error.column()), (2, Some(12)));
    let neighbour = accepted(&doc("code: [x, a:b]\n"));
    let Value::List(list) = neighbour
        .frontmatter()
        .map()
        .get("code")
        .expect("code")
        .value()
    else {
        panic!("code is a list");
    };
    assert_eq!(
        list.items()[1].value().as_scalar().expect("scalar").text(),
        "a:b"
    );
}

#[test]
fn block_indicators_inside_an_items_entry_are_refused() {
    let item = refused(&doc("items:\n  - >\n"));
    assert_eq!(
        item.kind(),
        &ErrorKind::BlockScalar {
            header: ">".to_string()
        }
    );
    assert_eq!((item.line(), item.column()), (3, Some(5)));
    let folded = refused(&doc("items:\n  - claim: x\n    outcome: > folded\n"));
    assert_eq!(
        folded.kind(),
        &ErrorKind::IndicatorCharacter { indicator: '>' }
    );
    assert_eq!((folded.line(), folded.column()), (4, Some(14)));
    let neighbour = accepted(&doc("items:\n  - claim: x\n    outcome: \"> folded\"\n"));
    assert!(neighbour.frontmatter().map().get("items").is_some());
}

#[test]
fn stray_and_over_indented_lines_after_a_flow_value_are_refused() {
    let stray = refused(&doc("bindings:\n  code: []\n- stray\n"));
    assert_eq!(stray.kind(), &ErrorKind::ExpectedKey);
    assert_eq!(stray.line(), 4);
    let deeper = refused(&doc("bindings:\n  code: []\n      extra: 1\n"));
    assert_eq!(deeper.kind(), &ErrorKind::Indentation);
    assert_eq!(deeper.line(), 4);
    let plain = refused(&doc("bindings:\n  code: []\nstray\n"));
    assert_eq!(plain.kind(), &ErrorKind::ExpectedKey);
    accepted(&doc("bindings:\n  code: []\n  tests: []\nnext: 1\n"));
}

#[test]
fn trailing_comments_are_not_part_of_the_value() {
    let document = accepted(&doc(concat!(
        "quoted: \"a # b\" # c\n",
        "single: 'It''s # x'  # c\n",
        "hash: a#b\n",
        "list: [one, \"two # 2\"] # c\n",
        "empty: # nothing\n",
        "plain: value # c\n",
        "items:\n",
        "  - claim: x # c\n",
    )));
    let map = document.frontmatter().map();
    let text = |key: &str| {
        map.get(key)
            .expect("key")
            .value()
            .as_scalar()
            .expect("scalar")
            .text()
    };
    assert_eq!(text("quoted"), "a # b");
    assert_eq!(text("single"), "It's # x");
    assert_eq!(text("hash"), "a#b");
    assert_eq!(text("empty"), "");
    assert_eq!(text("plain"), "value");
    let Value::List(list) = map.get("list").expect("list").value() else {
        panic!("list is a list");
    };
    let texts: Vec<&str> = list
        .items()
        .iter()
        .map(|item| item.value().as_scalar().expect("scalar").text())
        .collect();
    assert_eq!(texts, ["one", "two # 2"]);
    assert_eq!(
        map.get("list").expect("list").comments().trailing(),
        Some(" # c")
    );
    assert_eq!(
        map.get("single").expect("single").comments().trailing(),
        Some("  # c")
    );
    assert_eq!(
        map.get("empty").expect("empty").comments().trailing(),
        Some(" # nothing")
    );
    let Value::List(items) = map.get("items").expect("items").value() else {
        panic!("items is a list");
    };
    let ItemValue::Map(item) = items.items()[0].value() else {
        panic!("a map item");
    };
    let claim = item.get("claim").expect("claim");
    assert_eq!(claim.value().as_scalar().expect("scalar").text(), "x");
    assert_eq!(claim.comments().trailing(), Some(" # c"));
    // A comment marker with no space before it is text, and a flow list
    // with an unclosed quote hiding a marker is refused.
    assert_eq!(
        refused(&doc("list: [one, \"two # 2] # c\n")).kind(),
        &ErrorKind::UnterminatedQuote
    );
}

#[test]
fn a_leading_byte_order_mark_is_refused_at_line_one_column_one() {
    let error = refused("\u{feff}---\ntype: Architecture\n---\nbody\n");
    assert_eq!(error.kind(), &ErrorKind::MissingFence);
    assert_eq!(
        (error.line(), error.column(), error.offset()),
        (1, Some(1), Some(0))
    );
    // The same text without the mark is accepted.
    accepted("---\ntype: Architecture\n---\nbody\n");
}

#[test]
fn a_nul_in_the_body_is_kept_and_renders_back() {
    let text = "---\ntype: Architecture\n---\nbefore\0after\n\0\n";
    let document = accepted(text);
    assert_eq!(document.body().text(), "before\0after\n\0\n");
    assert_eq!(document.render(), text);
    assert_eq!(accepted(&document.render()), document);
}

#[test]
fn a_frontmatter_fence_line_with_a_trailing_space_is_not_a_fence() {
    // An opening `--- ` is no fence on line 1.
    let error = refused("--- \ntype: Architecture\n---\nbody\n");
    assert_eq!(error.kind(), &ErrorKind::MissingFence);
    assert_eq!(
        (error.line(), error.column(), error.offset()),
        (1, Some(1), Some(0))
    );
    // A closing `--- ` does not close the frontmatter, so it is never closed.
    let error = refused("---\ntype: Architecture\n--- \nbody\n");
    assert_eq!(error.kind(), &ErrorKind::UnclosedFence);
    assert_eq!(
        (error.line(), error.column(), error.offset()),
        (1, Some(1), Some(0))
    );
    // The neighbouring text with exact fences is accepted.
    accepted("---\ntype: Architecture\n---\nbody\n");
}
