// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Replacing spans of a body, a document and an index.

use raf_okf::{Body, Document, ErrorKind, Index, Span, SpliceError};

const TEXT: &str = "intro\n\n<!-- raf:generated:one -->\nold one\n<!-- raf:generated:end -->\n\nmiddle\n\n<!-- raf:generated:two -->\nold two\n<!-- raf:generated:end -->\n\ntail\n";

fn body() -> Body {
    Body::parse(TEXT).expect("body")
}

#[test]
fn two_generated_blocks_change_only_their_byte_ranges() {
    let mut parsed = body();
    let blocks = parsed.generated_blocks().to_vec();
    parsed
        .replace_spans(&[
            (blocks[0].content().clone(), "new one\nsecond row\n"),
            (blocks[1].content().clone(), "new two\n"),
        ])
        .expect("splice");
    assert_eq!(
        parsed.text(),
        "intro\n\n<!-- raf:generated:one -->\nnew one\nsecond row\n<!-- raf:generated:end -->\n\nmiddle\n\n<!-- raf:generated:two -->\nnew two\n<!-- raf:generated:end -->\n\ntail\n"
    );
    // The outside bytes are the original ones.
    assert!(
        parsed
            .text()
            .starts_with("intro\n\n<!-- raf:generated:one -->\n")
    );
    assert!(parsed.text().ends_with("\n\ntail\n"));
    assert_eq!(parsed.generated_blocks().len(), 2);
}

#[test]
fn edits_given_out_of_order_apply_in_one_pass() {
    let mut parsed = body();
    let blocks = parsed.generated_blocks().to_vec();
    parsed
        .replace_spans(&[
            (blocks[1].content().clone(), "B\n"),
            (blocks[0].content().clone(), "A\n"),
        ])
        .expect("splice");
    assert!(parsed.text().contains("one -->\nA\n<!--"));
    assert!(parsed.text().contains("two -->\nB\n<!--"));
}

#[test]
fn a_second_call_uses_spans_read_from_the_rebuilt_model() {
    let mut parsed = body();
    let first = parsed.generated_blocks()[0].content().clone();
    parsed
        .replace_spans(&[(first.clone(), "a much longer replacement\nover two lines\n")])
        .expect("first");
    let after = parsed.generated_blocks().to_vec();
    assert_ne!(after[1].content(), body().generated_blocks()[1].content());
    assert_eq!(
        &parsed.text()[after[1].content().bytes.clone()],
        "old two\n"
    );
    parsed
        .replace_spans(&[(after[1].content().clone(), "second\n")])
        .expect("second");
    assert!(
        parsed
            .text()
            .contains("a much longer replacement\nover two lines\n")
    );
    assert!(parsed.text().contains("two -->\nsecond\n<!--"));
    // The lines moved with the text.
    assert_eq!(parsed.generated_blocks()[1].content().lines, 11..12);
}

#[test]
fn overlapping_spans_are_refused_and_the_body_is_unchanged() {
    let mut parsed = body();
    let before = parsed.clone();
    let outer = parsed.generated_blocks()[0].outer().clone();
    let content = parsed.generated_blocks()[0].content().clone();
    let error = parsed
        .replace_spans(&[(outer.clone(), "x"), (content.clone(), "y")])
        .expect_err("overlap");
    assert_eq!(
        error,
        SpliceError::Overlap {
            offset: content.bytes.start
        }
    );
    assert_eq!(parsed.text(), before.text());
    assert_eq!(parsed.generated_blocks(), before.generated_blocks());
    // Adjacent spans, and two insertions at one point, do not overlap.
    let (a, b) = (0..0, 0..0);
    let span = |bytes| Span { lines: 1..1, bytes };
    parsed
        .replace_spans(&[(span(a), "1"), (span(b), "2")])
        .expect("insertions");
    assert!(parsed.text().starts_with("12intro\n"));
}

#[test]
fn a_malformed_result_is_refused_and_the_body_is_unchanged() {
    let mut parsed = body();
    let before = parsed.text().to_string();
    let content = parsed.generated_blocks()[0].content().clone();
    let error = parsed
        .replace_spans(&[(
            content,
            "<!-- raf:generated:end -->\n<!-- raf:generated:end -->\n",
        )])
        .expect_err("orphan end");
    match error {
        SpliceError::Reparse(inner) => assert_eq!(inner.kind(), &ErrorKind::OrphanGeneratedEnd),
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(parsed.text(), before);
    assert_eq!(parsed.generated_blocks().len(), 2);
}

#[test]
fn spans_outside_the_body_or_inside_a_character_are_refused() {
    let text = "---\ntype: Architecture\n---\n\ncaf\u{e9} text\n";
    let document = Document::parse(text).expect("document");
    let mut parsed = document.body().clone();
    let base = parsed.span().bytes.start;
    let span = |start: usize, end: usize| Span {
        lines: 1..1,
        bytes: start..end,
    };
    // The frontmatter is not part of the body.
    assert_eq!(
        parsed.replace_spans(&[(span(0, 3), "x")]),
        Err(SpliceError::OutOfBody { start: 0, end: 3 })
    );
    let end = base + parsed.text().len();
    assert_eq!(
        parsed.replace_spans(&[(span(end, end + 1), "x")]),
        Err(SpliceError::OutOfBody {
            start: end,
            end: end + 1
        })
    );
    assert_eq!(
        parsed.replace_spans(&[(span(base + 5, base + 4), "x")]),
        Err(SpliceError::OutOfBody {
            start: base + 5,
            end: base + 4
        })
    );
    // "caf" is three bytes at base+1..; the accent starts at base+4.
    let accent = base + 1 + 3;
    assert_eq!(
        parsed.replace_spans(&[(span(accent + 1, accent + 1), "x")]),
        Err(SpliceError::NotCharBoundary { offset: accent + 1 })
    );
    assert_eq!(parsed.text(), document.body().text());
    parsed
        .replace_spans(&[(span(accent, accent + 2), "e")])
        .expect("whole character");
    assert_eq!(parsed.text(), "\ncafe text\n");
}

#[test]
fn an_empty_edit_list_leaves_the_body_equal() {
    let mut parsed = body();
    let before = parsed.clone();
    parsed.replace_spans(&[]).expect("no edits");
    assert_eq!(parsed, before);
    assert_eq!(parsed.span(), before.span());
}

#[test]
fn a_document_delegates_and_renders_the_edit_with_its_frontmatter() {
    let text = "---\nid: arch:x\ntype: Architecture\n---\n\n<!-- raf:generated:list -->\nold\n<!-- raf:generated:end -->\n";
    let mut document = Document::parse(text).expect("document");
    let content = document.body().generated_blocks()[0].content().clone();
    assert_eq!(&text[content.bytes.clone()], "old\n");
    document
        .replace_spans(&[(content, "new\nrows\n")])
        .expect("splice");
    assert_eq!(
        document.render(),
        "---\ntype: Architecture\nid: arch:x\n---\n\n<!-- raf:generated:list -->\nnew\nrows\n<!-- raf:generated:end -->\n"
    );
    // After an edit the spans are in the coordinates of render().
    let rendered = document.render();
    let again = document.body().generated_blocks()[0].content().clone();
    assert_eq!(&rendered[again.bytes.clone()], "new\nrows\n");

    let before = document.clone();
    let outer = document.body().generated_blocks()[0].outer().clone();
    let content = document.body().generated_blocks()[0].content().clone();
    assert!(
        document
            .replace_spans(&[(outer, "x"), (content, "y")])
            .is_err()
    );
    assert_eq!(document, before);
    assert_eq!(document.render(), before.render());
}

#[test]
fn a_document_whose_closing_fence_has_no_line_ending_gains_one_for_a_body() {
    let mut document = Document::parse("---\ntype: Architecture\n---").expect("document");
    let at = document.body().span().bytes.clone();
    document
        .replace_spans(&[(
            Span {
                lines: 4..4,
                bytes: at,
            },
            "text\n",
        )])
        .expect("splice");
    assert_eq!(document.render(), "---\ntype: Architecture\n---\ntext\n");
    assert_eq!(
        Document::parse(&document.render()).expect("reads"),
        document
    );
}

#[test]
fn the_catalogue_of_an_index_is_replaced_through_the_index() {
    let text = "<!--\nSPDX-FileCopyrightText: 2026 Example\nSPDX-License-Identifier: CC-BY-4.0\n-->\n\n<!-- raf:generated:catalogue -->\n| old |\n<!-- raf:generated:end -->\n";
    let mut index = Index::parse(text).expect("index");
    let content = index.generated_blocks()[0].content().clone();
    assert_eq!(&text[content.bytes.clone()], "| old |\n");
    index
        .replace_spans(&[(content, "| alpha |\n| beta |\n")])
        .expect("splice");
    assert_eq!(
        index.render(),
        "<!--\nSPDX-FileCopyrightText: 2026 Example\nSPDX-License-Identifier: CC-BY-4.0\n-->\n\n<!-- raf:generated:catalogue -->\n| alpha |\n| beta |\n<!-- raf:generated:end -->\n"
    );
    assert_eq!(Index::parse(&index.render()).expect("reads"), index);

    // An edit that leaves prose outside a block is refused; the index stays.
    let before = index.clone();
    let start = index.body().span().bytes.start;
    let error = index
        .replace_spans(&[(
            Span {
                lines: 1..1,
                bytes: start..start,
            },
            "prose\n",
        )])
        .expect_err("prose");
    match error {
        SpliceError::Reparse(inner) => assert_eq!(inner.kind(), &ErrorKind::IndexText),
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(index, before);
}

#[test]
fn a_document_edit_with_a_malformed_result_is_refused_and_the_document_is_unchanged() {
    let text = "---\ntype: Architecture\n---\n\n<!-- raf:generated:list -->\nold\n<!-- raf:generated:end -->\n";
    let mut document = Document::parse(text).expect("document");
    let before = document.clone();
    let content = document.body().generated_blocks()[0].content().clone();
    let error = document
        .replace_spans(&[(
            content.clone(),
            "<!-- raf:generated:end -->\n<!-- raf:generated:end -->\n",
        )])
        .expect_err("orphan end");
    match error {
        SpliceError::Reparse(inner) => {
            assert_eq!(inner.kind(), &ErrorKind::OrphanGeneratedEnd);
            assert_eq!(inner.line(), 7);
        }
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(document, before);
    assert_eq!(document.render(), text);
    assert_eq!(document.body().span(), before.body().span());
    // The valid neighbour applies.
    document
        .replace_spans(&[(content, "new\n")])
        .expect("valid edit");
    assert!(document.render().contains("list -->\nnew\n<!--"));
}

#[test]
fn a_document_edit_over_one_mebibyte_is_refused_and_the_document_is_unchanged() {
    let text = "---\ntype: Architecture\n---\n\n<!-- raf:generated:list -->\nold\n<!-- raf:generated:end -->\n";
    let mut document = Document::parse(text).expect("document");
    let before = document.clone();
    let content = document.body().generated_blocks()[0].content().clone();
    let head = "---\ntype: Architecture\n---\n\n<!-- raf:generated:list -->\n".len();
    let tail = "<!-- raf:generated:end -->\n".len();
    let fits = raf_okf::MAX_TEXT_BYTES - head - tail;
    let over = format!("{}\n", "x".repeat(fits));
    let error = document
        .replace_spans(&[(content.clone(), over.as_str())])
        .expect_err("too large");
    match error {
        SpliceError::Reparse(inner) => assert!(matches!(inner.kind(), ErrorKind::TooLarge { .. })),
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(document, before);
    assert_eq!(document.render(), text);
    let exact = format!("{}\n", "x".repeat(fits - 1));
    document
        .replace_spans(&[(content, exact.as_str())])
        .expect("exactly at the bound");
    assert_eq!(document.render().len(), raf_okf::MAX_TEXT_BYTES);
}
