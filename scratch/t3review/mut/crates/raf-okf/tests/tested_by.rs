// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Tested-by markers: the statements a capability maps to the tests that show
//! them, the forms the reader refuses, and the documents that may hold one.

use raf_okf::{Body, Document, ErrorKind, Location, ParseError, Profile, Rule};

fn body(text: &str) -> Body {
    Body::parse(text).unwrap_or_else(|error| panic!("{error} in {text:?}"))
}

fn refused(text: &str) -> ParseError {
    match Body::parse(text) {
        Ok(_) => panic!("expected a refusal for {text:?}"),
        Err(error) => error,
    }
}

/// The paths and the statement text of each tested-by marker of `text`.
fn markers(text: &str) -> Vec<(Vec<String>, String)> {
    body(text)
        .tested_by()
        .iter()
        .map(|marker| {
            (
                marker.paths().to_vec(),
                text[marker.statement().bytes.clone()].to_string(),
            )
        })
        .collect()
}

fn paths(names: &[&str]) -> Vec<String> {
    names.iter().map(ToString::to_string).collect()
}

const MARKER: &str = "<!-- raf:tested-by demo::test_crate::it::case -->";

#[test]
fn a_tested_by_marker_names_its_tests_and_maps_the_paragraph_above_it() {
    let text = "## Contract\n\nIntro.\n\nThe parser accepts input\nin chunks.\n<!-- raf:tested-by demo::test_crate::it::chunks,demo::run -->\n\nAfter.\n";
    let parsed = body(text);
    assert_eq!(parsed.tested_by().len(), 1);
    let marker = &parsed.tested_by()[0];
    assert_eq!(
        marker.paths(),
        ["demo::test_crate::it::chunks", "demo::run"]
    );
    assert_eq!(marker.line(), 7);
    assert_eq!(marker.marker().lines, 7..8);
    assert_eq!(
        &text[marker.marker().bytes.clone()],
        "<!-- raf:tested-by demo::test_crate::it::chunks,demo::run -->\n"
    );
    assert_eq!(marker.statement().lines, 5..7);
    assert_eq!(
        &text[marker.statement().bytes.clone()],
        "The parser accepts input\nin chunks.\n"
    );
    assert_eq!(marker.section(), Some("contract"));
    // The marker is a comment: no prose line and no claim.
    assert!(parsed.prose_lines().iter().all(|line| line.line() != 7));
    assert_eq!(parsed.claims(), []);
}

#[test]
fn a_tested_by_marker_maps_a_list_item_a_heading_or_the_statement_of_its_claim_marker() {
    assert_eq!(
        markers(&format!("- First.\n- Second item\n  goes on.\n{MARKER}\n")),
        [(
            paths(&["demo::test_crate::it::case"]),
            "- Second item\n  goes on.\n".to_string()
        )]
    );
    // The statement of a heading clause is its heading line.
    assert_eq!(
        markers(&format!(
            "## Contract\n\n### Chunked input\n{MARKER}\n\nText.\n"
        )),
        [(
            paths(&["demo::test_crate::it::case"]),
            "### Chunked input\n".to_string()
        )]
    );
    // After the claim marker of a statement, the statement is the claim's.
    let text = format!("It holds.\n<!-- raf:claim evidence=bound -->\n{MARKER}\n");
    let parsed = body(&text);
    assert_eq!(parsed.claims().len(), 1);
    assert_eq!(
        markers(&text),
        [(
            paths(&["demo::test_crate::it::case"]),
            "It holds.\n".to_string()
        )]
    );
    assert_eq!(
        parsed.tested_by()[0].statement(),
        parsed.claims()[0].statement()
    );
    // Indented under its list item, and with a generic argument list whose
    // comma does not split the entry.
    assert_eq!(
        markers("- An item.\n    <!-- raf:tested-by demo::Pair<A,B>::get,demo::run -->\n"),
        [(
            paths(&["demo::Pair<A,B>::get", "demo::run"]),
            "- An item.\n".to_string()
        )]
    );
}

#[test]
fn a_malformed_tested_by_marker_is_refused_with_its_line() {
    for marker in [
        "<!-- raf:tested-by demo::a, demo::b -->",
        "<!-- raf:tested-by demo::a,demo::a -->",
        "<!-- raf:tested-by -->",
        "<!-- raf:tested-by  -->",
        "<!-- raf:tested-by tests=demo::a -->",
        "<!-- raf:tested-by demo::a,,demo::b -->",
        "<!-- raf:tested-by demo::a, -->",
        "<!-- raf:tested-by  demo::a -->",
        "<!-- raf:tested-by demo::a  -->",
        "<!-- raf:tested-by demo::a-->",
        "<!--  raf:tested-by demo::a -->",
        "<!-- RAF:TESTED-BY demo::a -->",
        "<!-- raf:tested-by demo::a --> trailing",
        "<!-- raf:tested-by demo:a -->",
        "<!-- raf:tested-by demo::a::-->",
        "<!-- raf:tested-by 9demo::a -->",
        "<!-- raf:tested-by demo::Pair<A -->",
        "<!-- raf:tested-byx demo::a -->",
        "> <!-- raf:tested-by demo::a -->",
    ] {
        let error = refused(&format!("A statement.\n{marker}\n"));
        assert_eq!(error.kind(), &ErrorKind::MalformedTestedBy, "{marker}");
        assert_eq!(error.line(), 2, "{marker}");
    }
    // A marker in the middle of a line is not on its own line.
    let inline = refused(&format!("A statement. {MARKER}\n"));
    assert_eq!(inline.kind(), &ErrorKind::MalformedTestedBy);
    // A marker split over lines is never a plain comment.
    let split = refused("A statement.\n<!--\nraf:tested-by demo::a\n-->\n");
    assert_eq!(
        (split.kind(), split.line()),
        (&ErrorKind::MalformedTestedBy, 2)
    );
    // The neighbour: the same statement with the marker written exactly.
    assert_eq!(markers(&format!("A statement.\n{MARKER}\n")).len(), 1);
}

#[test]
fn a_tested_by_marker_with_no_statement_directly_above_is_refused() {
    let claim = "<!-- raf:claim evidence=bound -->";
    for (text, line) in [
        (format!("{MARKER}\n"), 1),
        (format!("A statement.\n\n{MARKER}\n"), 3),
        (format!("A statement.\n<!-- a note -->\n{MARKER}\n"), 3),
        (format!("A statement.\n{MARKER}\n{MARKER}\n"), 3),
        (format!("A statement.\n{MARKER}\n{claim}\n"), 2),
        (format!("```\ncode\n```\n{MARKER}\n"), 4),
        (
            format!("## Contract\n\n> [!PROPOSED change:x]\n> Proposed.\n{MARKER}\n"),
            5,
        ),
        (
            format!("<!-- raf:generated:a -->\n<!-- raf:generated:end -->\n{MARKER}\n"),
            3,
        ),
    ] {
        let error = refused(&text);
        assert_eq!(error.kind(), &ErrorKind::UnattachedTestedBy, "{text:?}");
        assert_eq!(error.line(), line, "{text:?}");
    }
    // The neighbours: directly after the statement, and after its claim
    // marker.
    assert_eq!(markers(&format!("A statement.\n{MARKER}\n")).len(), 1);
    assert_eq!(
        markers(&format!("A statement.\n{claim}\n{MARKER}\n")).len(),
        1
    );
}

#[test]
fn tested_by_markers_in_fences_code_spans_and_proposed_blocks_are_not_read() {
    let fenced =
        format!("Text.\n\n```markdown\nA claim.\n{MARKER}\n<!-- raf:tested-by a, b -->\n```\n");
    assert_eq!(body(&fenced).tested_by(), []);
    let span = format!("Write `{MARKER}` after the statement.\n");
    assert_eq!(body(&span).tested_by(), []);
    let proposed = format!(
        "## Contract\n\n> [!PROPOSED change:add-tests]\n> The bound holds.\n> {MARKER}\n> <!-- raf:tested-by a, b -->\n"
    );
    let parsed = body(&proposed);
    assert_eq!(parsed.tested_by(), []);
    assert_eq!(parsed.proposed_blocks().len(), 1);
    // Once resolved, the same text is a marker.
    let resolved = format!("## Contract\n\nThe bound holds.\n{MARKER}\n");
    assert_eq!(body(&resolved).tested_by().len(), 1);
}

/// A designed capability whose Contract holds `contract`.
fn capability(contract: &str) -> String {
    let sections: String = [
        "Purpose",
        "Contract",
        "Usage",
        "Examples",
        "Constraints",
        "Explanation",
        "Verification",
    ]
    .iter()
    .map(|name| {
        if *name == "Contract" {
            format!("## {name}\n\n{contract}\n")
        } else {
            format!("## {name}\n\nText for {name}.\n")
        }
    })
    .collect::<Vec<_>>()
    .join("\n");
    format!(
        "---\ntype: Capability\nid: cap:alpha\ntitle: Alpha\ndescription: Does alpha.\n\
         state: designed\nbindings:\n  code: []\n  tests: []\n  examples: []\ndepends: []\n---\n\n{sections}"
    )
}

#[test]
fn a_tested_by_marker_belongs_in_a_capability_document() {
    let text = format!(
        "---\ntype: Architecture\nid: arch:overview\ntitle: Overview\n\
         description: How it fits.\n---\n\n# Overview\n\nIt holds.\n{MARKER}\n"
    );
    let document = Document::parse(&text).expect("a document");
    assert_eq!(document.body().tested_by()[0].line(), 11);
    let found = Profile
        .validate(&document, "docs/architecture/overview.md")
        .expect_err("a tested-by marker outside a capability");
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].at(), &Location::Line(11));
    assert_eq!(found[0].rule(), &Rule::TestedByOutsideCapability);
    assert_eq!(
        found[0].rule().kind().name(),
        "tested-by-outside-capability"
    );
    // The neighbour: the same marker in a capability.
    let inside = capability(&format!("It holds.\n{MARKER}"));
    let document = Document::parse(&inside).expect("a document");
    Profile
        .validate(&document, "docs/capabilities/alpha.md")
        .expect("a tested-by marker belongs in a capability");
    assert_eq!(document.body().tested_by()[0].section(), Some("contract"));
}
