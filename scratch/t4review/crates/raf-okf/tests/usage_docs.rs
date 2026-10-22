// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The Examples block of the capability document validates as written. That the
//! Usage and Examples `rust` blocks are the crate documentation tests is checked
//! once, by the document-contract test of the root package.

use std::path::{Path, PathBuf};

use raf_okf::{Body, DocType, Document, Location, Profile, RafDocument, Rule};

fn capability_document() -> String {
    let root = std::env::var_os("RAF_GATE_ROOT").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        PathBuf::from,
    );
    std::fs::read_to_string(root.join("docs/capabilities/okf-profile.md")).expect("the document")
}

/// The code of the first fenced block with this info string under `heading`.
fn block_under(text: &str, heading: &str, info: &str) -> String {
    let body = Body::parse(text).expect("the document parses");
    let section = body
        .sections()
        .iter()
        .find(|section| section.heading() == heading)
        .expect("the section");
    let content = section.content();
    let fenced = body
        .fenced_blocks()
        .iter()
        .find(|block| {
            block.info() == info
                && block.span().bytes.start >= content.bytes.start
                && block.span().bytes.end <= content.bytes.end
        })
        .expect("the block");
    let lines: Vec<&str> = text[fenced.span().bytes.clone()].lines().collect();
    let inner = &lines[1..lines.len() - 1];
    let mut code = inner.join("\n");
    code.push('\n');
    code
}

#[test]
fn the_examples_block_validates_between_fences_and_resource_is_rejected() {
    let text = capability_document();
    let yaml = block_under(&text, "Examples", "yaml");
    assert!(yaml.contains("bindings:\n  code: []\n  tests: []\n  examples: []\n"));
    assert!(!yaml.contains('{'), "the block form has no flow mapping");
    let body: String = [
        "Purpose",
        "Contract",
        "Usage",
        "Examples",
        "Constraints",
        "Explanation",
        "Verification",
    ]
    .iter()
    .map(|name| format!("## {name}\n\nText for {name}.\n"))
    .collect::<Vec<_>>()
    .join("\n");
    let path = "docs/capabilities/example.md";
    let wrap = |front: &str| format!("---\n{front}---\n\n{body}");

    let document = Document::parse(&wrap(&yaml)).expect("the example parses");
    let RafDocument::Capability(cap) = Profile.validate(&document, path).expect("clean") else {
        panic!("a capability");
    };
    assert_eq!(cap.header.id, "cap:example");

    let with_resource = format!("{yaml}resource: https://example.org/x\n");
    let document = Document::parse(&wrap(&with_resource)).expect("parses");
    let found = Profile
        .validate(&document, path)
        .expect_err("resource is refused");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].at(), &Location::Key("resource".to_string()));
    assert_eq!(
        found[0].rule(),
        &Rule::ResourceKey {
            doc_type: DocType::Capability
        }
    );
    let message = found[0].to_string();
    assert!(message.contains("`resource`"));
    for spec in DocType::Capability.keys() {
        assert!(message.contains(spec.key), "{message} lacks {}", spec.key);
    }
}
