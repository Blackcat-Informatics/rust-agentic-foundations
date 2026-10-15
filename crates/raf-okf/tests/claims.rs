// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Claim markers: the statements that rest on evidence, and the refusal of
//! every `raf:` comment that is not a marker.

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

/// The evidence slugs and the statement text of each claim of `text`.
fn claims(text: &str) -> Vec<(Vec<String>, String)> {
    body(text)
        .claims()
        .iter()
        .map(|claim| {
            (
                claim.evidence().to_vec(),
                text[claim.statement().bytes.clone()].to_string(),
            )
        })
        .collect()
}

fn slugs(names: &[&str]) -> Vec<String> {
    names.iter().map(ToString::to_string).collect()
}

#[test]
fn a_claim_marker_names_its_evidence_and_supports_the_paragraph_above_it() {
    let text = "## Contract\n\nIntro.\n\nParsing a 10 MiB input\nstays under 64 MiB.\n<!-- raf:claim evidence=memory-bound -->\n\nAfter.\n";
    let parsed = body(text);
    let claim = &parsed.claims()[0];
    assert_eq!(parsed.claims().len(), 1);
    assert_eq!(claim.evidence(), ["memory-bound"]);
    assert_eq!(claim.line(), 7);
    assert_eq!(claim.marker().lines, 7..8);
    assert_eq!(
        &text[claim.marker().bytes.clone()],
        "<!-- raf:claim evidence=memory-bound -->\n"
    );
    assert_eq!(claim.statement().lines, 5..7);
    assert_eq!(
        &text[claim.statement().bytes.clone()],
        "Parsing a 10 MiB input\nstays under 64 MiB.\n"
    );
    assert_eq!(claim.section(), Some("contract"));
    // The marker is a comment, so it is no prose line.
    assert!(parsed.prose_lines().iter().all(|line| line.line() != 7));
}

#[test]
fn a_claim_may_rest_on_several_records() {
    assert_eq!(
        claims("Bounded.\n<!-- raf:claim evidence=memory-bound,time-bound,a2 -->\n"),
        [(
            slugs(&["memory-bound", "time-bound", "a2"]),
            "Bounded.\n".to_string()
        )]
    );
}

#[test]
fn a_claim_after_a_list_item_supports_that_item_only() {
    let text = "- First item.\n- Second item\n  continues here.\n  <!-- raf:claim evidence=second -->\n- Third.\n<!-- raf:claim evidence=third -->\n";
    assert_eq!(
        claims(text),
        [
            (
                slugs(&["second"]),
                "- Second item\n  continues here.\n".to_string()
            ),
            (slugs(&["third"]), "- Third.\n".to_string()),
        ]
    );
    // A numbered item, inside a quote, is an item too.
    assert_eq!(
        claims("1. One.\n2. Two.\n<!-- raf:claim evidence=two -->\n"),
        [(slugs(&["two"]), "2. Two.\n".to_string())]
    );
}

#[test]
fn markers_in_fences_code_spans_and_proposed_blocks_are_not_read() {
    let fenced = "Text.\n\n```markdown\nA claim.\n<!-- raf:claim evidence=x -->\n<!-- raf:anything -->\n```\n";
    assert_eq!(body(fenced).claims(), []);
    let span =
        "Write `<!-- raf:claim evidence=x -->` after the statement, never `<!-- raf:note -->`.\n";
    assert_eq!(body(span).claims(), []);
    // A claim inside a PROPOSED block is not a claim until the block is
    // resolved, and nothing in the block is read as a marker.
    let proposed = "## Contract\n\n> [!PROPOSED change:add-claims]\n> The bound holds.\n> <!-- raf:claim evidence=memory-bound -->\n> <!-- raf:claim evidence=Bad -->\n";
    let parsed = body(proposed);
    assert_eq!(parsed.claims(), []);
    assert_eq!(parsed.proposed_blocks().len(), 1);
    // Once resolved, the same text is a claim.
    let resolved = "## Contract\n\nThe bound holds.\n<!-- raf:claim evidence=memory-bound -->\n";
    assert_eq!(body(resolved).claims().len(), 1);
}

#[test]
fn every_other_raf_comment_is_refused_with_its_line() {
    for (text, line, column) in [
        ("Text.\n<!-- raf:note x -->\n", 2, 1),
        ("Text.\n\n  <!--raf:claims evidence=x -->\n", 3, 3),
        ("Text. <!-- RAF:evidence x -->\n", 1, 1),
        ("Text.\n<!-- raf: claim evidence=x -->\n", 2, 1),
        ("Text.\n<!-- raf:generatedfoo -->\n", 2, 1),
        (
            "Text <!-- raf:generated:a --> then <!-- raf:todo -->\n",
            1,
            1,
        ),
    ] {
        let error = refused(text);
        let wanted = if text.contains("claims evidence") {
            ErrorKind::MalformedClaim
        } else {
            ErrorKind::UnknownMarker
        };
        assert_eq!(error.kind(), &wanted, "{text:?}");
        assert_eq!(
            (error.line(), error.column()),
            (line, Some(column)),
            "{text:?}"
        );
    }
    // A plain comment, and a comment that only mentions the word, are text.
    body("Text.\n<!-- a note about raf:claim markers -->\n<!-- raf -->\n");
}

#[test]
fn a_raf_generated_comment_keeps_its_own_rules() {
    let block = "Intro.\n\n<!-- raf:generated:catalogue -->\n| a |\n<!-- raf:generated:end -->\n";
    let parsed = body(block);
    assert_eq!(parsed.generated_blocks().len(), 1);
    assert_eq!(parsed.claims(), []);
    // Mid-line, it is text, as it always was.
    body("mid-line <!-- raf:generated:a --> text\n");
    // At a line start in another spelling, it is a malformed generated marker.
    assert_eq!(
        refused("<!-- raf:generated:Bad -->\n").kind(),
        &ErrorKind::MalformedGenerated
    );
}

#[test]
fn a_malformed_claim_marker_is_refused_with_its_line() {
    for marker in [
        "<!-- raf:claim -->",
        "<!-- raf:claim evidence= -->",
        "<!-- raf:claim evidence=Upper -->",
        "<!-- raf:claim evidence=a,,b -->",
        "<!-- raf:claim evidence=a,a -->",
        "<!-- raf:claim evidence=a, b -->",
        "<!-- raf:claim evidence=a evidence=b -->",
        "<!-- raf:claim evidence=a tests=b -->",
        "<!-- raf:claim  evidence=a -->",
        "<!-- raf:claim evidence=a  -->",
        "<!-- raf:claim evidence:a -->",
        "<!-- raf:claim evidence=a-->",
        "<!--  raf:claim evidence=a -->",
        "<!-- RAF:CLAIM evidence=a -->",
        "<!-- raf:claim evidence=a --> trailing",
        "> <!-- raf:claim evidence=a -->",
    ] {
        let error = refused(&format!("A statement.\n{marker}\n"));
        assert_eq!(error.kind(), &ErrorKind::MalformedClaim, "{marker}");
        assert_eq!(error.line(), 2, "{marker}");
    }
    // A claim marker in the middle of a line is not on its own line.
    let inline = refused("A statement. <!-- raf:claim evidence=a -->\n");
    assert_eq!(inline.kind(), &ErrorKind::MalformedClaim);
    // The marker indented under a list item is well formed.
    assert_eq!(
        claims("- An item.\n    <!-- raf:claim evidence=a -->\n").len(),
        1
    );
}

#[test]
fn a_claim_marker_with_no_statement_directly_above_is_refused() {
    let marker = "<!-- raf:claim evidence=a -->";
    for (text, line) in [
        (format!("{marker}\n"), 1),
        (format!("A statement.\n\n{marker}\n"), 3),
        (format!("## Contract\n{marker}\n"), 2),
        (format!("```\ncode\n```\n{marker}\n"), 4),
        (format!("A statement.\n{marker}\n{marker}\n"), 3),
        (format!("A statement.\n<!-- a note -->\n{marker}\n"), 3),
        (
            format!("## Contract\n\n> [!PROPOSED change:x]\n> Proposed.\n{marker}\n"),
            5,
        ),
        (
            format!("<!-- raf:generated:a -->\n<!-- raf:generated:end -->\n{marker}\n"),
            3,
        ),
    ] {
        let error = refused(&text);
        assert_eq!(error.kind(), &ErrorKind::UnattachedClaim, "{text:?}");
        assert_eq!(error.line(), line, "{text:?}");
    }
}

#[test]
fn a_statement_stops_at_a_heading_a_blank_line_or_a_fence() {
    assert_eq!(
        claims("## Contract\nRight under.\n<!-- raf:claim evidence=a -->\n"),
        [(slugs(&["a"]), "Right under.\n".to_string())]
    );
    assert_eq!(
        claims("```\ncode\n```\nAfter the fence.\n<!-- raf:claim evidence=a -->\n"),
        [(slugs(&["a"]), "After the fence.\n".to_string())]
    );
}

#[test]
fn claims_in_a_document_carry_whole_text_lines() {
    let text = "---\ntype: Architecture\nid: arch:overview\ntitle: Overview\n\
                description: How it fits.\n---\n\n# Overview\n\nIt holds.\n<!-- raf:claim evidence=a -->\n";
    let document = Document::parse(text).expect("a document");
    let claim = &document.body().claims()[0];
    assert_eq!(claim.line(), 11);
    assert_eq!(&text[claim.statement().bytes.clone()], "It holds.\n");
    // Only a capability states claims.
    let found = Profile
        .validate(&document, "docs/architecture/overview.md")
        .expect_err("a claim outside a capability");
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].at(), &Location::Line(11));
    assert_eq!(found[0].rule(), &Rule::ClaimOutsideCapability);
    let without = text.replace("<!-- raf:claim evidence=a -->\n", "");
    let document = Document::parse(&without).expect("a document");
    Profile
        .validate(&document, "docs/architecture/overview.md")
        .expect("clean without the claim");
}

#[test]
fn a_capability_states_claims_in_its_sections() {
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
        format!(
            "## {name}\n\nText for {name}.\n<!-- raf:claim evidence=e-{} -->\n",
            name.to_lowercase()
        )
    })
    .collect::<Vec<_>>()
    .join("\n");
    let text = format!(
        "---\ntype: Capability\nid: cap:alpha\ntitle: Alpha\ndescription: Does alpha.\n\
         state: designed\nbindings:\n  code: []\n  tests: []\n  examples: []\ndepends: []\n---\n\n{sections}"
    );
    let document = Document::parse(&text).expect("a document");
    Profile
        .validate(&document, "docs/capabilities/alpha.md")
        .expect("claims belong in a capability");
    let sections: Vec<Option<&str>> = document
        .body()
        .claims()
        .iter()
        .map(raf_okf::Claim::section)
        .collect();
    assert_eq!(sections.len(), 7);
    assert_eq!(sections[0], Some("purpose"));
    assert_eq!(sections[6], Some("verification"));
}
