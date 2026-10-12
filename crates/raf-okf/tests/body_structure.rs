// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Headings, anchors, sections, fences, nesting and the prose view.

use raf_okf::{Body, Document, ErrorKind, MAX_BLOCK_NESTING};

fn body(text: &str) -> Body {
    Body::parse(text).unwrap_or_else(|error| panic!("{error} in {text:?}"))
}

fn refused(text: &str) -> raf_okf::ParseError {
    match Body::parse(text) {
        Ok(_) => panic!("expected a refusal for {text:?}"),
        Err(error) => error,
    }
}

#[test]
fn sections_carry_heading_slug_and_content_up_to_the_next_second_level_heading() {
    let text = "# Title\n\nintro\n\n## Purpose\n\nwhy\n\n### Detail\n\nmore\n\n## Contract Terms\n\nwhat\n";
    let parsed = body(text);
    let sections = parsed.sections();
    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].heading(), "Purpose");
    assert_eq!(sections[0].slug(), "purpose");
    assert_eq!(sections[0].line(), 5);
    let first = sections[0].content();
    assert_eq!(
        &text[first.bytes.clone()],
        "\nwhy\n\n### Detail\n\nmore\n\n"
    );
    assert_eq!(first.lines, 6..13);
    assert_eq!(sections[1].heading(), "Contract Terms");
    assert_eq!(sections[1].slug(), "contract-terms");
    let second = sections[1].content();
    assert_eq!(&text[second.bytes.clone()], "\nwhat\n");
    assert_eq!(second.lines, 14..16);
}

#[test]
fn a_section_that_ends_the_body_without_a_final_newline_reaches_the_end() {
    let text = "## A\nline one\nline two";
    let parsed = body(text);
    let content = parsed.sections()[0].content();
    assert_eq!(&text[content.bytes.clone()], "line one\nline two");
    assert_eq!(content.lines, 2..4);
    let empty = body("## A\n## B\n");
    assert_eq!(empty.sections()[0].content().bytes, 5..5);
}

#[test]
fn spans_are_in_whole_text_coordinates_when_frontmatter_precedes_the_body() {
    let text = "---\ntype: Architecture\n---\n\n## One\n\nfirst\n\n## Two\n\nsecond\n";
    let document = Document::parse(text).expect("document");
    let sections = document.body().sections();
    assert_eq!(&text[sections[0].content().bytes.clone()], "\nfirst\n\n");
    assert_eq!(sections[0].line(), 5);
    assert_eq!(sections[0].content().lines, 6..9);
    // Body::parse over the same whole text gives the same coordinates.
    let direct = body(text);
    assert_eq!(direct.sections()[1].content(), sections[1].content());
    assert_eq!(direct.text(), document.body().text());
    assert_eq!(&text[direct.span().bytes.clone()], direct.text());
}

#[test]
fn markdown_without_frontmatter_is_the_whole_body() {
    let text = "# Readme\n\nSee [a](b.md).\n\n```text\n[no](c.md)\n```\n";
    let parsed = body(text);
    assert_eq!(parsed.text(), text);
    assert_eq!(parsed.span().bytes, 0..text.len());
    let targets: Vec<&str> = parsed.links().iter().map(raf_okf::Link::target).collect();
    assert_eq!(targets, ["b.md"]);
    // A leading rule that no later rule closes is not frontmatter.
    let rule = body("---\nnot a mapping\n\ntext [a](b.md)\n");
    assert_eq!(rule.links().len(), 1);
    assert_eq!(rule.span().bytes.start, 0);
}

#[test]
fn a_three_space_heading_is_a_section_and_a_four_space_heading_is_not() {
    assert_eq!(body("   ## Three\n").sections().len(), 1);
    assert_eq!(body("   ## Three\n").sections()[0].heading(), "Three");
    assert!(matches!(body("    ## Four\n").sections(), []));
    assert!(matches!(body("    ## Four\n").headings(), []));
    assert!(matches!(body("##No space\n").headings(), []));
    assert!(matches!(body("####### Seven\n").headings(), []));
    assert_eq!(body("###### Six\n").headings()[0].level(), 6);
}

#[test]
fn headings_of_every_level_carry_github_anchors_with_repeats_numbered() {
    let parsed = body(
        "# Why `x`: it's (fine)\n\n## Same\n\n### Same\n\n## Same ##\n\n#### Under_score & Dash-ed\n",
    );
    let anchors: Vec<&str> = parsed
        .headings()
        .iter()
        .map(raf_okf::Heading::anchor)
        .collect();
    assert_eq!(
        anchors,
        [
            "why-x-its-fine",
            "same",
            "same-1",
            "same-2",
            "under_score--dash-ed"
        ]
    );
    let levels: Vec<usize> = parsed
        .headings()
        .iter()
        .map(raf_okf::Heading::level)
        .collect();
    assert_eq!(levels, [1, 2, 3, 2, 4]);
    assert_eq!(parsed.headings()[3].text(), "Same");
    assert!(parsed.has_anchor("same-2"));
    assert!(parsed.has_anchor("why-x-its-fine"));
    assert!(!parsed.has_anchor("same-3"));
    assert!(!parsed.has_anchor("Same"));
    assert_eq!(body("# C#\n").headings()[0].text(), "C#");
}

#[test]
fn headings_inside_fences_and_comments_are_not_headings() {
    let text = "## Real\n\n```md\n## Fenced\n```\n\n<!-- ## Hidden -->\n\n<!--\n## Also hidden\n-->\n\n## After\n";
    let parsed = body(text);
    let names: Vec<&str> = parsed
        .headings()
        .iter()
        .map(raf_okf::Heading::text)
        .collect();
    assert_eq!(names, ["Real", "After"]);
}

#[test]
fn a_fence_closes_only_on_the_same_character_with_at_least_the_same_length() {
    let text = "````md\n```\n## still fenced\n```\n````\n\n## Open\n";
    let parsed = body(text);
    assert_eq!(parsed.fenced_blocks().len(), 1);
    assert_eq!(parsed.fenced_blocks()[0].info(), "md");
    assert_eq!(parsed.fenced_blocks()[0].span().lines, 1..6);
    let names: Vec<&str> = parsed
        .headings()
        .iter()
        .map(raf_okf::Heading::text)
        .collect();
    assert_eq!(names, ["Open"]);

    // A tilde fence is not closed by backticks, and the reverse.
    let mixed = "~~~\n```\n## fenced\n~~~\n## Out\n";
    assert_eq!(body(mixed).headings().len(), 1);
    // A shorter run does not close.
    let short = "`````\n````\n## fenced\n`````\n## Out\n";
    assert_eq!(body(short).headings().len(), 1);
    // A backtick fence whose info holds a backtick is not a fence.
    assert_eq!(body("``` a`b\n## Real\n").headings().len(), 1);
    // A fence that is never closed runs to the end.
    let open = body("```\n## fenced\n");
    assert!(matches!(open.headings(), []));
    assert_eq!(open.fenced_blocks()[0].span().lines, 1..3);
}

#[test]
fn fences_are_recognised_indented_up_to_three_spaces_and_inside_quotes_and_lists() {
    assert_eq!(body("   ```\n## fenced\n   ```\n").headings().len(), 0);
    assert_eq!(body("    ```\n## Real\n").headings().len(), 1);
    assert_eq!(body("> ```\n> ## fenced\n> ```\n").fenced_blocks().len(), 1);
    assert_eq!(
        body("> ```\n> ## fenced\n> ```\n## Out\n").headings().len(),
        1
    );
    let listed = body("- item\n\n  ```\n  [a](b.md)\n  ```\n\n[c](d.md)\n");
    assert_eq!(listed.fenced_blocks().len(), 1);
    let targets: Vec<&str> = listed.links().iter().map(raf_okf::Link::target).collect();
    assert_eq!(targets, ["d.md"]);
    assert_eq!(
        body("1. ```\n   ## fenced\n   ```\n").fenced_blocks().len(),
        1
    );
}

#[test]
fn a_fenced_block_exposes_its_content_and_whether_it_is_closed() {
    // Backtick and tilde fences, an info string, and a longer closing fence.
    let text = "```rust\nlet a = 1;\n\nlet b = 2;\n```\n\n~~~text\nplain\n~~~~\n";
    let parsed = body(text);
    let blocks = parsed.fenced_blocks();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].info(), "rust");
    assert_eq!(blocks[0].content(), "let a = 1;\n\nlet b = 2;\n");
    assert!(blocks[0].is_closed());
    assert_eq!(blocks[1].info(), "text");
    assert_eq!(blocks[1].content(), "plain\n");
    assert!(blocks[1].is_closed());

    // A closing fence line may carry trailing spaces; a shorter or different
    // fence does not close.
    let spaced = body("```\nx\n``` \n");
    assert!(spaced.fenced_blocks()[0].is_closed());
    assert_eq!(spaced.fenced_blocks()[0].content(), "x\n");
    let wrong = body("~~~\nx\n```\n~~\n");
    assert!(!wrong.fenced_blocks()[0].is_closed());
    assert_eq!(wrong.fenced_blocks()[0].content(), "x\n```\n~~\n");

    // Nothing between the fences, and a block that is never closed.
    let empty = body("```\n```\n");
    assert_eq!(empty.fenced_blocks()[0].content(), "");
    assert!(empty.fenced_blocks()[0].is_closed());
    let open = body("```\nlast\nline");
    assert!(!open.fenced_blocks()[0].is_closed());
    assert_eq!(open.fenced_blocks()[0].content(), "last\nline");
    let alone = body("text\n```");
    assert!(!alone.fenced_blocks()[0].is_closed());
    assert_eq!(alone.fenced_blocks()[0].content(), "");

    // Line endings are kept, and the content lies inside the span.
    let crlf = body("~~~\r\na\r\n~~~\r\n");
    assert_eq!(crlf.fenced_blocks()[0].content(), "a\r\n");
    for block in blocks {
        let inside = &text[block.span().bytes.clone()];
        assert!(inside.contains(block.content()));
    }

    // Inside a quote the prefix of each line is part of the content.
    let quoted = body("> ```\n> code\n> ```\n");
    assert_eq!(quoted.fenced_blocks()[0].content(), "> code\n");
    assert!(quoted.fenced_blocks()[0].is_closed());
}

#[test]
fn a_fenced_block_prints_and_compares_without_the_body_around_it() {
    let filler = "filler line of the body\n".repeat(2000);
    let fence = "```rust\nlet x = 1;\n```\n";
    let big = body(&format!("{fence}{filler}"));
    let many = body(&format!("{}{filler}", fence.repeat(50)));
    let printed = format!("{:?}", big.fenced_blocks()[0]);
    assert!(printed.contains("let x = 1;"), "{printed}");
    assert!(printed.len() < 300, "{} bytes: {printed}", printed.len());
    let all = format!("{:?}", many.fenced_blocks());
    assert!(all.len() < 50 * 300, "{} bytes", all.len());

    // The same block in bodies that differ elsewhere is the same block.
    let short = body(fence);
    let other = body(&format!("{fence}different text after the fences\n"));
    assert_eq!(big.fenced_blocks()[0], short.fenced_blocks()[0]);
    assert_eq!(big.fenced_blocks()[0], other.fenced_blocks()[0]);
    // A different content, info or closing state is a different block.
    for different in [
        "```rust\nlet x = 2;\n```\n",
        "```text\nlet x = 1;\n```\n",
        "```rust\nlet x = 1;\n",
    ] {
        assert_ne!(
            big.fenced_blocks()[0],
            body(different).fenced_blocks()[0],
            "{different:?}"
        );
    }
    // The same fences at another position are another block.
    let moved = body(&format!("text\n{fence}"));
    assert_ne!(big.fenced_blocks()[0], moved.fenced_blocks()[0]);
}

#[test]
fn block_nesting_of_sixty_four_levels_parses_and_sixty_five_is_refused() {
    let quotes = |n: usize| format!("{}text\n", "> ".repeat(n));
    body(&quotes(MAX_BLOCK_NESTING));
    let error = refused(&quotes(MAX_BLOCK_NESTING + 1));
    assert_eq!(error.kind(), &ErrorKind::BlockNesting { max: 64 });
    assert_eq!(error.line(), 1);
    assert_eq!(error.column(), Some(2 * 64 + 1));

    let compact = |n: usize| format!("{}text\n", ">".repeat(n));
    body(&compact(64));
    assert_eq!(
        refused(&compact(65)).kind(),
        &ErrorKind::BlockNesting { max: 64 }
    );

    let markers = |n: usize| format!("{}text\n", "- ".repeat(n));
    body(&markers(64));
    assert_eq!(refused(&format!("para\n\n{}", markers(65))).line(), 3);

    // Levels by indentation count too.
    let indented = |n: usize| {
        let mut text = String::new();
        for level in 0..n {
            text.push_str(&"  ".repeat(level));
            text.push_str("- item ");
            text.push_str(&level.to_string());
            text.push('\n');
        }
        text
    };
    body(&indented(64));
    let error = refused(&indented(65));
    assert_eq!(error.kind(), &ErrorKind::BlockNesting { max: 64 });
    assert_eq!(error.line(), 65);

    // Quotes and lists add up.
    body(&format!("{}{}x\n", "> ".repeat(32), "- ".repeat(32)));
    assert_eq!(
        refused(&format!("{}{}x\n", "> ".repeat(33), "- ".repeat(32))).line(),
        1
    );
}

#[test]
fn a_thematic_break_is_not_a_run_of_list_markers() {
    body(&format!("{}\n", "* ".repeat(100)));
    body(&format!("{}\n", "- ".repeat(100)));
    body("---\n***\n___\n");
}

#[test]
fn a_hundred_thousand_quote_markers_are_refused_on_a_small_stack() {
    let result = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| Body::parse(&format!("{}x\n", ">".repeat(100_000))).map(|_| ()))
        .expect("spawn")
        .join()
        .expect("no stack overflow");
    assert_eq!(
        result.expect_err("too deep").kind(),
        &ErrorKind::BlockNesting { max: 64 }
    );
}

#[test]
fn prose_lines_skip_fences_and_comments_and_remove_code_spans() {
    let tbd = ["T", "BD"].concat();
    let text = format!(
        "## Contract\n\nUse `{tbd}` and ``a ` {tbd} b`` here {tbd}x.\n\n```text\n{tbd}\n```\n\n<!-- {tbd} -->\nvisible <!-- {tbd} --> after\n\n<!--\n{tbd}\n-->\n\n> [!PROPOSED change:one]\n> quoted {tbd}\n\n<!-- raf:generated:list -->\n| row |\n<!-- raf:generated:end -->\n\n## Explanation\n\nplain\n"
    );
    let parsed = body(&text);
    let prose = parsed.prose_lines();
    let lines: Vec<(usize, &str)> = prose.iter().map(|p| (p.line(), p.text())).collect();
    assert_eq!(
        lines,
        [
            (1, "## Contract"),
            (3, &*format!("Use   and   here {tbd}x.")),
            (10, "visible  after"),
            (16, "> [!PROPOSED change:one]"),
            (17, &*format!("> quoted {tbd}")),
            (20, "| row |"),
            (23, "## Explanation"),
            (25, "plain"),
        ]
    );
    let flags: Vec<(Option<&str>, bool, bool)> = prose
        .iter()
        .map(|p| (p.section(), p.in_proposed(), p.in_generated()))
        .collect();
    assert_eq!(
        flags,
        [
            (Some("contract"), false, false),
            (Some("contract"), false, false),
            (Some("contract"), false, false),
            (Some("contract"), true, false),
            (Some("contract"), true, false),
            (Some("contract"), false, true),
            (Some("explanation"), false, false),
            (Some("explanation"), false, false),
        ]
    );
    // Before the first second-level heading there is no section.
    assert_eq!(body("intro\n").prose_lines()[0].section(), None);
}

#[test]
fn code_spans_of_any_backtick_length_hide_their_content() {
    let text = "a `x` b ``y ` z`` c ```w``` d `unclosed e\n";
    let prose = body(text);
    assert_eq!(prose.prose_lines()[0].text(), "a   b   c   d `unclosed e");
}

#[test]
fn only_atx_headings_are_headings_and_setext_headings_are_not() {
    assert!(matches!(body("Title\n=====\n").headings(), []));
    assert!(matches!(body("Title\n-----\n\ntext\n").sections(), []));
    assert_eq!(body("# Title\n").headings().len(), 1);
    assert_eq!(body("## Title\n").sections().len(), 1);
}

#[test]
fn indented_code_is_ordinary_text_so_links_and_markers_in_it_count() {
    let indented_link = body("    [a](b.md)\n");
    assert_eq!(indented_link.links().len(), 1);
    assert_eq!(indented_link.links()[0].target(), "b.md");
    // The same text in a fence is code.
    assert!(matches!(body("```\n    [a](b.md)\n```\n").links(), []));
    let marker = body("    > [!PROPOSED change:x]\n    > text\n");
    assert_eq!(marker.proposed_blocks().len(), 1);
    assert_eq!(marker.proposed_blocks()[0].slug(), "x");
}

#[test]
fn section_text_is_the_content_of_the_section() {
    let text = "---\ntype: Architecture\n---\n\n## One\n\nfirst\n\n## Two\n\nsecond\n";
    let document = Document::parse(text).expect("document");
    let sections = document.body().sections();
    assert_eq!(
        document.body().section_text(&sections[0]),
        Some("\nfirst\n\n")
    );
    assert_eq!(
        document.body().section_text(&sections[1]),
        Some("\nsecond\n")
    );
    // A section of another body is not found.
    let other = body("intro\n\n## A\ncontent that is long enough to lie outside\n");
    assert_eq!(document.body().section_text(&other.sections()[0]), None);
}

#[test]
fn a_body_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused() {
    let at = "x".repeat(raf_okf::MAX_TEXT_BYTES);
    assert_eq!(body(&at).text().len(), raf_okf::MAX_TEXT_BYTES);
    let over = format!("{at}x");
    let error = refused(&over);
    assert_eq!(
        error.kind(),
        &ErrorKind::TooLarge {
            len: raf_okf::MAX_TEXT_BYTES + 1,
            max: raf_okf::MAX_TEXT_BYTES
        }
    );
    assert_eq!(error.offset(), Some(raf_okf::MAX_TEXT_BYTES));
}

#[test]
fn a_document_built_with_a_malformed_body_reports_the_line_in_the_rendered_text() {
    use raf_okf::{ConstructError, Entry, Frontmatter, Map, Scalar, Value};
    let frontmatter = || {
        let value = Value::Scalar(Scalar::new("Architecture").expect("scalar"));
        let entry = Entry::new("type", value).expect("entry");
        Frontmatter::new(Map::new(vec![entry]).expect("map"))
    };
    // The head is three lines, so the body starts on line 4.
    let malformed = Document::new(frontmatter(), "intro\n> [!proposed change:x]\n");
    let Err(ConstructError::Body(error)) = malformed else {
        panic!("expected a body error, got {malformed:?}");
    };
    assert_eq!(error.kind(), &ErrorKind::MalformedProposed);
    assert_eq!(error.line(), 5);
    assert_eq!(
        error.offset(),
        Some("---\ntype: Architecture\n---\nintro\n".len())
    );

    let nested = format!("a\n\n{}x\n", "> ".repeat(65));
    let Err(ConstructError::Body(error)) = Document::new(frontmatter(), nested) else {
        panic!("expected a nesting error");
    };
    assert_eq!(error.kind(), &ErrorKind::BlockNesting { max: 64 });
    assert_eq!(error.line(), 6);

    // The neighbours that read are accepted.
    let built = Document::new(frontmatter(), "intro\n> [!PROPOSED change:x]\n").expect("valid");
    assert_eq!(built.body().proposed_blocks()[0].marker().lines, 5..6);
    Document::new(frontmatter(), format!("a\n\n{}x\n", "> ".repeat(64))).expect("64 levels");
}
