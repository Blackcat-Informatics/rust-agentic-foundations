// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! PROPOSED blocks and generated blocks, and the errors of their markers.

use raf_okf::{Body, Document, ErrorKind, ParseError};

fn body(text: &str) -> Body {
    Body::parse(text).unwrap_or_else(|error| panic!("{error} in {text:?}"))
}

fn refused(text: &str) -> ParseError {
    match Body::parse(text) {
        Ok(_) => panic!("expected a refusal for {text:?}"),
        Err(error) => error,
    }
}

#[test]
fn a_proposed_block_reports_slug_spans_content_and_section() {
    let text = "## Purpose\n\nintro\n\n> [!PROPOSED change:add-alpha]\n> The candidate\n> contract.\n>\n> More.\n\nafter\n\n## Contract\n\n> [!PROPOSED change:beta-two]\n> Second.\n";
    let parsed = body(text);
    let blocks = parsed.proposed_blocks();
    assert_eq!(blocks.len(), 2);
    let first = &blocks[0];
    assert_eq!(first.slug(), "add-alpha");
    assert_eq!(first.section(), Some("purpose"));
    assert_eq!(first.content(), "The candidate\ncontract.\n\nMore.");
    assert_eq!(
        &text[first.span().bytes.clone()],
        "> [!PROPOSED change:add-alpha]\n> The candidate\n> contract.\n>\n> More.\n"
    );
    assert_eq!(first.span().lines, 5..10);
    assert_eq!(
        &text[first.marker().bytes.clone()],
        "> [!PROPOSED change:add-alpha]\n"
    );
    assert_eq!(first.marker().lines, 5..6);
    assert_eq!(blocks[1].slug(), "beta-two");
    assert_eq!(blocks[1].section(), Some("contract"));
    assert_eq!(blocks[1].content(), "Second.");
    // Before any second-level heading there is no section.
    assert_eq!(
        body("> [!PROPOSED change:x]\n> t\n").proposed_blocks()[0].section(),
        None
    );
}

#[test]
fn two_markers_in_a_row_make_two_blocks() {
    let parsed = body("> [!PROPOSED change:a]\n> one\n> [!PROPOSED change:b]\n> two\n");
    let slugs: Vec<&str> = parsed
        .proposed_blocks()
        .iter()
        .map(raf_okf::ProposedBlock::slug)
        .collect();
    assert_eq!(slugs, ["a", "b"]);
    assert_eq!(parsed.proposed_blocks()[0].content(), "one");
}

#[test]
fn markers_inside_a_fence_produce_no_blocks() {
    let text = "```markdown\n> [!PROPOSED change:x]\n> text\n<!-- raf:generated:list -->\n```\n";
    let parsed = body(text);
    assert!(matches!(parsed.proposed_blocks(), []));
    assert!(matches!(parsed.generated_blocks(), []));
    // A marker inside a comment-free code span line is not a marker line.
    assert!(matches!(
        body("`> [!PROPOSED change:x]`\n").proposed_blocks(),
        []
    ));
}

#[test]
fn every_misspelling_of_the_proposed_marker_is_refused_with_its_line() {
    for marker in [
        "> [!PROPOSED add-alpha]",
        "> [!proposed change:add-alpha]",
        "> [!Proposed change:add-alpha]",
        ">[!PROPOSED change:add-alpha]",
        ">  [!PROPOSED change:add-alpha]",
        "> > [!PROPOSED change:add-alpha]",
        "- > [!PROPOSED change:add-alpha]",
        "[!PROPOSED change:add-alpha]",
        "> [!PROPOSED change:Add-Alpha]",
        "> [!PROPOSED change:]",
        "> [!PROPOSED change:add-alpha] trailing",
        "> [!PROPOSED change:add-alpha",
        "* [!PROPOSED change:x]",
    ] {
        let error = refused(&format!("# t\n\ntext\n{marker}\n> body\n"));
        assert_eq!(error.kind(), &ErrorKind::MalformedProposed, "{marker}");
        assert_eq!(error.line(), 4, "{marker}");
        assert_eq!(error.column(), Some(1), "{marker}");
    }
    let indented = refused("text\n   [!PROPOSED nope]\n");
    assert_eq!((indented.line(), indented.column()), (2, Some(4)));
    // The exact marker, and words that only resemble it, are accepted.
    body("# t\n\n> [!PROPOSED change:add-alpha]\n> body\n");
    body("> [!NOTE]\n> body\n");
    body("This is not a [!PROPOSED] marker mid-line.\n");
}

#[test]
fn generated_blocks_report_kind_outer_and_content_spans() {
    let text = "intro\n\n<!-- raf:generated:catalogue -->\n| a |\n| b |\n<!-- raf:generated:end -->\n\ntail\n\n<!-- raf:generated:other-kind -->\n<!-- raf:generated:end -->\n";
    let parsed = body(text);
    let blocks = parsed.generated_blocks();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].kind(), "catalogue");
    assert_eq!(
        &text[blocks[0].outer().bytes.clone()],
        "<!-- raf:generated:catalogue -->\n| a |\n| b |\n<!-- raf:generated:end -->\n"
    );
    assert_eq!(&text[blocks[0].content().bytes.clone()], "| a |\n| b |\n");
    assert_eq!(blocks[0].outer().lines, 3..7);
    assert_eq!(blocks[0].content().lines, 4..6);
    assert_eq!(blocks[1].kind(), "other-kind");
    assert_eq!(&text[blocks[1].content().bytes.clone()], "");
    assert_eq!(blocks[1].content().lines, 11..11);
}

#[test]
fn each_generated_marker_error_names_its_line() {
    let orphan = refused("text\n\n<!-- raf:generated:end -->\n");
    assert_eq!(orphan.kind(), &ErrorKind::OrphanGeneratedEnd);
    assert_eq!(orphan.line(), 3);

    let nested = refused(
        "<!-- raf:generated:a -->\ntext\n<!-- raf:generated:b -->\n<!-- raf:generated:end -->\n",
    );
    assert_eq!(
        nested.kind(),
        &ErrorKind::NestedGenerated {
            outer: "a".to_string()
        }
    );
    assert_eq!(nested.line(), 3);

    let unclosed = refused("x\n<!-- raf:generated:matrix -->\n| a |\n");
    assert_eq!(
        unclosed.kind(),
        &ErrorKind::UnclosedGenerated {
            kind: "matrix".to_string()
        }
    );
    assert_eq!(unclosed.line(), 2);

    for (marker, line) in [
        ("<!-- raf:generated: -->", 2),
        ("<!-- raf:generated:Bad Kind -->", 2),
        ("<!-- raf:generated:matrix", 2),
        ("<!-- raf:generated:matrix --> trailing", 2),
        ("<!-- raf:generated:UPPER -->", 2),
        ("<!-- raf:generated:end", 2),
    ] {
        let error = refused(&format!("text\n{marker}\n"));
        assert_eq!(error.kind(), &ErrorKind::MalformedGenerated, "{marker}");
        assert_eq!(error.line(), line, "{marker}");
    }
    // Properly closed, and marker-like text that is not at a line start.
    body("<!-- raf:generated:a -->\nx\n<!-- raf:generated:end -->\n");
    body("mid-line <!-- raf:generated:a --> text\n");
}

#[test]
fn marker_errors_in_a_document_carry_whole_text_lines() {
    let text = "---\ntype: Capability\n---\n\n## Contract\n\n> [!proposed change:x]\n";
    let error = Document::parse(text).expect_err("malformed marker");
    assert_eq!(error.kind(), &ErrorKind::MalformedProposed);
    assert_eq!(error.line(), 7);
    assert_eq!(
        error.offset(),
        Some(text.len() - "> [!proposed change:x]\n".len())
    );
}

#[test]
fn document_parse_builds_its_body_through_the_body_model() {
    let text = "---\ntype: Capability\n---\n\n## Contract\n\n[a](b.md)\n";
    let document = Document::parse(text).expect("document");
    let direct = Body::parse(text).expect("body");
    assert_eq!(document.body(), &direct);
    assert_eq!(document.body().links(), direct.links());
    assert_eq!(document.body().sections(), direct.sections());
    assert_eq!(document.body().span(), direct.span());
}
