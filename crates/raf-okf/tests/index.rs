// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The generated capability index.

mod support;

use raf_okf::{Document, ErrorKind, Index, ParseError, is_capability_index};
use support::dataset_rows;

/// Assembled in pieces so that this file carries no SPDX tag of its own here.
const SPDX_ID: &str = concat!("SPDX-License-", "Identifier: CC-BY-4.0");
const SPDX_COPY: &str = concat!("SPDX-File", "CopyrightText: 2026 Example");

const BLOCK: &str = "<!-- raf:generated:catalogue -->\n| alpha |\n<!-- raf:generated:end -->\n";

const INDEX_IRI: &str = "https://EXAMPLE.org/索引/%7e?view=1#Navigation";

#[test]
fn navigation_dataset_preserves_generated_bytes_through_native_formats() {
    use purrdf_rdf::{
        NativeRdfFormat, ParseOptions, SerializeGraph, SerializeOptions, StatementLayer,
        parse_dataset_with, serialize_dataset_with,
    };
    let generated = include_str!("../../../docs/capabilities/index.md");
    let decorated = format!(
        "<!-- {SPDX_ID} -->\r\n\r\n{}\r\n{}",
        BLOCK.replace("alpha", "café 索引").replace('\n', "\r\n"),
        BLOCK
    );
    let options = SerializeOptions {
        selection: SerializeGraph::Dataset,
        statement_layer: StatementLayer::Emit,
        jsonld_options: None,
    };
    for text in [generated, decorated.as_str()] {
        let index = Index::parse(text).unwrap();
        let dataset = index.to_dataset(INDEX_IRI).unwrap();
        assert_eq!(Index::from_dataset(&dataset, INDEX_IRI).unwrap(), index);
        for (format, media) in [
            (NativeRdfFormat::Turtle, "text/turtle"),
            (NativeRdfFormat::NTriples, "application/n-triples"),
        ] {
            let output = serialize_dataset_with(&dataset, format, None, &options).unwrap();
            let parsed =
                parse_dataset_with(&output.bytes, media, None, &ParseOptions::default()).unwrap();
            let restored = Index::from_dataset(&parsed.dataset, INDEX_IRI).unwrap();
            assert_eq!(restored, index);
            assert_eq!(restored.render().as_bytes(), text.as_bytes());
            assert_eq!(restored.generated_blocks(), index.generated_blocks());
            assert_eq!(
                serialize_dataset_with(
                    &index.to_dataset(INDEX_IRI).unwrap(),
                    format,
                    None,
                    &options
                )
                .unwrap()
                .bytes,
                output.bytes
            );
            assert_eq!(
                serialize_dataset_with(&parsed.dataset, format, None, &options)
                    .unwrap()
                    .bytes,
                output.bytes
            );
        }
        assert!(Index::from_dataset(&dataset, &INDEX_IRI.replace("EXAMPLE", "example")).is_err());
        assert!(Document::from_dataset(&dataset, "docs/architecture/example.md").is_err());
    }
}

#[test]
fn navigation_dataset_refuses_nonclosed_shapes_and_retains_body_locations() {
    use purrdf_core::{RdfLiteral, RdfTerm};
    use raf_okf::DatasetError;
    let index = Index::parse(BLOCK).unwrap();
    let rows: Vec<_> = index.to_dataset(INDEX_IRI).unwrap().owned_quads().collect();
    let body = rows
        .iter()
        .position(|row| row.predicate.ends_with("#body"))
        .unwrap();
    let class = 1 - body;
    let mut cases = vec![vec![], vec![rows[body].clone()], vec![rows[class].clone()]];
    let mut extra = rows.clone();
    extra.push(rows[body].clone());
    extra.last_mut().unwrap().object = RdfTerm::Literal(RdfLiteral::simple("another body"));
    cases.push(extra);
    let mut duplicate = rows.clone();
    duplicate[body] = rows[class].clone();
    cases.push(duplicate);
    for (at, term) in [
        (body, RdfTerm::iri("urn:body")),
        (body, RdfTerm::blank_node("body")),
        (
            body,
            RdfTerm::Literal(RdfLiteral::typed(BLOCK, "urn:wrong-datatype")),
        ),
        (
            body,
            RdfTerm::Literal(RdfLiteral::language_tagged(BLOCK, "en")),
        ),
        (
            class,
            RdfTerm::iri("https://blackcatinformatics.ca/raf/okf/1#Document"),
        ),
    ] {
        let mut input = rows.clone();
        input[at].object = term;
        cases.push(input);
    }
    let mut input = rows.clone();
    input[body].predicate = "urn:unknown".into();
    cases.push(input);
    let mut input = rows.clone();
    input[body].subject = RdfTerm::iri("urn:other");
    cases.push(input);
    let mut input = rows.clone();
    input[body].subject = RdfTerm::blank_node("root");
    cases.push(input);
    let mut input = rows.clone();
    input[body].graph_name = Some(RdfTerm::iri("urn:graph"));
    cases.push(input);
    for input in cases {
        assert!(
            matches!(
                Index::from_dataset(&dataset_rows(&input), INDEX_IRI),
                Err(DatasetError::Shape { .. })
            ),
            "{input:?}"
        );
    }
    index_layer_refusals(&rows);
    let mut invalid = rows.clone();
    invalid[body].object = RdfTerm::Literal(RdfLiteral::simple(format!("{BLOCK}\nprose\n")));
    let DatasetError::Parse(error) =
        Index::from_dataset(&dataset_rows(&invalid), INDEX_IRI).unwrap_err()
    else {
        panic!("original parser error required")
    };
    assert_eq!(error.line(), 5);
    assert_eq!(
        Index::from_dataset(&dataset_rows(&rows), INDEX_IRI).unwrap(),
        index
    );
}

fn index_layer_refusals(rows: &[purrdf_core::RdfQuad]) {
    use raf_okf::DatasetError;
    let mut builder = purrdf_core::RdfDatasetBuilder::new();
    for row in rows {
        builder.push_owned_quad(row).unwrap();
    }
    let root = builder.intern_iri(INDEX_IRI).unwrap();
    let p = builder.intern_iri("urn:predicate").unwrap();
    let triple = builder.intern_triple(root, p, root).unwrap();
    builder.push_reifier(root, triple).unwrap();
    assert!(matches!(
        Index::from_dataset(&builder.freeze().unwrap(), INDEX_IRI),
        Err(DatasetError::Shape { .. })
    ));
    let mut builder = purrdf_core::RdfDatasetBuilder::new();
    for row in rows {
        builder.push_owned_quad(row).unwrap();
    }
    let root = builder.intern_iri(INDEX_IRI).unwrap();
    let p = builder.intern_iri("urn:annotation").unwrap();
    builder.push_annotation(root, p, root).unwrap();
    assert!(matches!(
        Index::from_dataset(&builder.freeze().unwrap(), INDEX_IRI),
        Err(DatasetError::Shape { .. })
    ));
    let mut builder = purrdf_core::RdfDatasetBuilder::new();
    for row in rows {
        builder.push_owned_quad(row).unwrap();
    }
    let graph = builder.intern_iri("urn:empty-graph").unwrap();
    builder.declare_named_graph(graph).unwrap();
    assert!(matches!(
        Index::from_dataset(&builder.freeze().unwrap(), INDEX_IRI),
        Err(DatasetError::Shape { .. })
    ));
}

#[test]
fn navigation_identity_and_aggregate_text_admission_have_exact_boundaries() {
    use raf_okf::{DatasetError, MAX_TEXT_BYTES};
    use std::error::Error;
    let index = Index::parse(BLOCK).unwrap();
    for iri in [
        "",
        "relative/path",
        "https://example.org/a b",
        "https://example.org/%xx",
    ] {
        let error = index.to_dataset(iri).unwrap_err();
        assert!(matches!(error, DatasetError::Native(_)), "{error}");
        assert!(error.source().is_some());
    }
    for iri in [
        "https://blackcatinformatics.ca/raf/okf/document/capability/cap:x",
        "https://blackcatinformatics.ca/raf/okf/node/any/0",
    ] {
        assert!(matches!(
            index.to_dataset(iri),
            Err(DatasetError::Shape { .. })
        ));
    }
    let body = format!(
        "{BLOCK}{}",
        " ".repeat(MAX_TEXT_BYTES - INDEX_IRI.len() - BLOCK.len())
    );
    let at_limit = Index::parse(&body).unwrap();
    let dataset = at_limit.to_dataset(INDEX_IRI).unwrap();
    assert_eq!(Index::from_dataset(&dataset, INDEX_IRI).unwrap(), at_limit);
    assert!(matches!(
        at_limit.to_dataset(&format!("{INDEX_IRI}x")),
        Err(DatasetError::Limit(_))
    ));
    let over = Index::parse(&format!("{body} ")).unwrap();
    assert!(matches!(
        over.to_dataset(INDEX_IRI),
        Err(DatasetError::Limit(_))
    ));
    let mut rows: Vec<_> = dataset.owned_quads().collect();
    let row = rows
        .iter_mut()
        .find(|row| row.predicate.ends_with("#body"))
        .unwrap();
    row.object = purrdf_core::RdfTerm::Literal(purrdf_core::RdfLiteral::simple(format!("{body} ")));
    assert!(matches!(
        Index::from_dataset(&dataset_rows(&rows), INDEX_IRI),
        Err(DatasetError::Limit(_))
    ));
    assert!(index.to_dataset(INDEX_IRI).is_ok());
}

fn refused(text: &str) -> ParseError {
    match Index::parse(text) {
        Ok(_) => panic!("expected a refusal for {text:?}"),
        Err(error) => error,
    }
}

#[test]
fn a_generated_only_index_is_accepted_with_either_header_form() {
    let multi = format!("<!--\n{SPDX_COPY}\n{SPDX_ID}\n-->\n\n{BLOCK}");
    let single = format!("<!-- {SPDX_ID} -->\n{BLOCK}");
    let two_single = format!("<!-- {SPDX_COPY} -->\n<!-- {SPDX_ID} -->\n\n{BLOCK}");
    for text in [
        multi,
        single,
        two_single,
        BLOCK.to_string(),
        format!("{BLOCK}\n{BLOCK}"),
    ] {
        let index = Index::parse(&text).expect(&text);
        assert_eq!(index.render(), text);
        assert!(matches!(index.generated_blocks(), [_, ..]));
    }
    let two = Index::parse(&format!("{BLOCK}\n{BLOCK}")).expect("two blocks");
    assert_eq!(two.generated_blocks().len(), 2);
    assert!(Index::parse(&BLOCK.replace('\n', "\r\n")).is_ok());
}

#[test]
fn prose_outside_the_blocks_is_refused_with_its_line() {
    for (text, line) in [
        (format!("# Index\n\n{BLOCK}"), 1),
        (format!("{BLOCK}\nHand-written note.\n"), 5),
        (
            "Generated by `<!-- raf:generated:catalogue -->` honestly.\n".to_string(),
            1,
        ),
        (format!("<!--\n{SPDX_ID}\nprose\n-->\n{BLOCK}"), 3),
    ] {
        let error = refused(&text);
        assert!(
            matches!(error.kind(), ErrorKind::IndexText | ErrorKind::IndexHeader),
            "{text:?}: {error}"
        );
        assert_eq!(error.line(), line, "{text:?}");
    }
    assert_eq!(
        refused(&format!("{BLOCK}\nnote\n")).kind(),
        &ErrorKind::IndexText
    );
}

#[test]
fn an_index_with_no_block_or_an_unclosed_block_is_refused() {
    assert_eq!(refused("\n").kind(), &ErrorKind::IndexEmpty);
    assert_eq!(refused("").kind(), &ErrorKind::IndexEmpty);
    assert_eq!(
        refused(&format!("<!-- {SPDX_ID} -->\n")).kind(),
        &ErrorKind::IndexEmpty
    );
    let unclosed = refused("<!-- raf:generated:catalogue -->\n");
    assert_eq!(
        unclosed.kind(),
        &ErrorKind::UnclosedGenerated {
            kind: "catalogue".to_string()
        }
    );
    assert_eq!(unclosed.line(), 1);
}

#[test]
fn indented_and_mid_line_markers_are_refused() {
    for indent in [" ", "    ", "\t"] {
        let open = refused(&format!(
            "{indent}<!-- raf:generated:catalogue -->\n| a |\n<!-- raf:generated:end -->\n"
        ));
        assert_eq!(open.kind(), &ErrorKind::IndexHeader, "{indent:?}");
        assert_eq!(open.line(), 1);
        let close = refused(&format!(
            "<!-- raf:generated:catalogue -->\n| a |\n{indent}<!-- raf:generated:end -->\n"
        ));
        // The indented end marker is block content, so the block never closes.
        assert_eq!(
            close.kind(),
            &ErrorKind::UnclosedGenerated {
                kind: "catalogue".to_string()
            },
            "{indent:?}"
        );
        assert_eq!(close.line(), 1);
    }
    let mid = refused("prose <!-- raf:generated:catalogue --> | a |\n<!-- raf:generated:end -->\n");
    assert_eq!(mid.kind(), &ErrorKind::IndexText);
    assert_eq!(mid.line(), 1);
    let end_mid = refused(
        "<!-- raf:generated:catalogue -->\n| a |\ntrailing <!-- raf:generated:end -->\n x\n",
    );
    assert_eq!(end_mid.line(), 1);
    assert_eq!(
        refused("<!-- raf:generated:catalogue -->\n<!-- raf:generated:catalogue -->\n<!-- raf:generated:end -->\n").kind(),
        &ErrorKind::IndexMarker
    );
    assert_eq!(
        refused("<!-- raf:generated:end -->\n").kind(),
        &ErrorKind::IndexMarker
    );
    assert_eq!(
        refused("<!-- raf:generated:Bad -->\n").kind(),
        &ErrorKind::IndexMarker
    );
}

#[test]
fn a_header_with_unknown_tags_or_in_the_wrong_place_is_refused() {
    for text in [
        format!("<!-- SPDX-Anything: x -->\n{BLOCK}"),
        format!("<!--\nSPDX-Anything: x\n-->\n{BLOCK}"),
        format!("{BLOCK}<!-- {SPDX_ID} -->\n"),
        format!("{BLOCK}\n<!--\n{SPDX_ID}\n-->\n{BLOCK}"),
        format!("<!--\n{SPDX_ID}\n"),
    ] {
        assert_eq!(refused(&text).kind(), &ErrorKind::IndexHeader, "{text:?}");
    }
}

#[test]
fn an_empty_header_tag_is_refused() {
    for header in [
        "<!-- SPDX-FileCopyrightText: -->\n".to_string(),
        "<!--\nSPDX-FileCopyrightText:   \n-->\n".to_string(),
        format!("<!--\n{SPDX_ID}\nSPDX-FileCopyrightText:\n-->\n"),
    ] {
        let error = refused(&format!("{header}{BLOCK}"));
        assert_eq!(error.kind(), &ErrorKind::IndexHeader, "{header:?}");
    }
    // A tag with a value is the neighbour that is accepted.
    assert!(Index::parse(&format!("<!--\nSPDX-FileCopyrightText: x\n-->\n{BLOCK}")).is_ok());
}

#[test]
fn document_parse_of_index_text_fails_at_line_one() {
    let text = format!("<!-- {SPDX_ID} -->\n{BLOCK}");
    assert!(Index::parse(&text).is_ok());
    let error = Document::parse(&text).expect_err("no frontmatter");
    assert_eq!(error.kind(), &ErrorKind::MissingFence);
    assert_eq!(error.line(), 1);
}

#[test]
fn only_the_capability_index_path_is_the_index() {
    assert!(is_capability_index("docs/capabilities/index.md"));
    assert!(is_capability_index("./docs/capabilities/index.md"));
    assert!(!is_capability_index("docs/index.md"));
    assert!(!is_capability_index("docs/capabilities/other.md"));
    assert!(!is_capability_index("docs/capabilities/sub/index.md"));
    assert!(!is_capability_index("x/docs/capabilities/index.md"));
    assert!(!is_capability_index(""));
}

#[test]
fn an_index_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused() {
    let open = "<!-- raf:generated:catalogue -->\n";
    let close = "<!-- raf:generated:end -->\n";
    let fill = raf_okf::MAX_TEXT_BYTES - open.len() - close.len();
    let at = format!("{open}{}{close}", "x".repeat(fill - 1) + "\n");
    assert_eq!(at.len(), raf_okf::MAX_TEXT_BYTES);
    assert!(Index::parse(&at).is_ok());
    let over = format!("{open}{}{close}", "x".repeat(fill) + "\n");
    let error = refused(&over);
    assert!(
        matches!(error.kind(), ErrorKind::TooLarge { .. }),
        "{error}"
    );
    assert_eq!(error.offset(), Some(raf_okf::MAX_TEXT_BYTES));
}
