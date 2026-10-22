// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The deterministic writer: goldens, key order, comments, tags, constructors.

use std::path::{Path, PathBuf};
use std::process::Command;

use raf_okf::{
    ConstructError, DocType, Document, ESCAPES, Entry, Frontmatter, ItemValue, List, ListForm,
    ListItem, MAX_TEXT_BYTES, MAX_YAML_DEPTH, Map, Profile, Scalar, ScalarStyle, Value,
};

fn parse(text: &str) -> Document {
    Document::parse(text).unwrap_or_else(|error| panic!("{error} in {text:?}"))
}

fn scalar(text: &str) -> Scalar {
    Scalar::new(text).unwrap_or_else(|error| panic!("{text:?}: {error}"))
}

fn entry(key: &str, value: Value) -> Entry {
    Entry::new(key, value).unwrap_or_else(|error| panic!("{key}: {error}"))
}

fn text_entry(key: &str, text: &str) -> Entry {
    entry(key, Value::Scalar(scalar(text)))
}

/// The repository root: `RAF_GATE_ROOT` when set (the commit hook points it at
/// a checkout of the index), otherwise two levels above the crate.
fn gate_root() -> PathBuf {
    std::env::var_os("RAF_GATE_ROOT").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        PathBuf::from,
    )
}

/// The tracked Markdown documents under the document directories of every
/// type.
fn tracked_documents() -> Vec<String> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(["ls-files", "-z", "--"])
        .args(DocType::ALL.map(DocType::directory))
        .output()
        .expect("git ls-files must run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut paths: Vec<String> = output
        .stdout
        .split(|b| *b == 0)
        .filter(|name| !name.is_empty())
        .map(|name| String::from_utf8(name.to_vec()).expect("tracked paths are UTF-8"))
        .filter(|path| {
            Path::new(path)
                .extension()
                .is_some_and(|extension| extension == "md")
                && path != "docs/capabilities/index.md"
        })
        .collect();
    paths.sort();
    paths
}

/// The tracked documents of one directory, counted with a listing of that
/// directory alone: the Markdown files other than the capability index.
fn tracked_in_directory(directory: &str) -> usize {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(["ls-files", "-z", "--", directory])
        .output()
        .expect("git ls-files must run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
        .stdout
        .split(|b| *b == 0)
        .filter(|name| name.ends_with(b".md") && !name.ends_with(b"capabilities/index.md"))
        .count()
}

#[test]
fn every_tracked_document_renders_to_its_own_bytes() {
    let paths = tracked_documents();
    let root = gate_root();
    let mut round_tripped = [0_usize; DocType::ALL.len()];
    for path in &paths {
        let slot = DocType::ALL
            .iter()
            .position(|doc_type| path.starts_with(&format!("{}/", doc_type.directory())))
            .unwrap_or_else(|| panic!("{path} is in no document directory"));
        round_tripped[slot] += 1;
        let text =
            std::fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"));
        let document = Document::parse(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(
            document.render(),
            text,
            "{path} does not render to its own bytes"
        );
        let again = Document::parse(&document.render()).expect("rendered text parses");
        assert_eq!(again, document, "{path}");
    }
    // Each type is round-tripped in full: as many documents as the directory
    // of the type holds, counted on its own.
    // Change records are deleted when their change completes, and the tree
    // may hold no record of a record type, so only the capability and
    // architecture directories always hold documents.
    for (doc_type, done) in DocType::ALL.into_iter().zip(round_tripped) {
        let directory = doc_type.directory();
        let held = tracked_in_directory(directory);
        if matches!(doc_type, DocType::Capability | DocType::Architecture) {
            assert!(held >= 1, "{directory} holds no tracked document");
        }
        assert_eq!(
            done, held,
            "{directory}: {done} documents were round-tripped and {held} are tracked"
        );
    }
}

/// The committed fixture documents, one per type, written independently of the
/// repository's own documents.
const FIXTURES: [(&str, DocType, &str, &str); 7] = [
    (
        "capability.md",
        DocType::Capability,
        "docs/capabilities/ledger-sync.md",
        include_str!("fixtures/capability.md"),
    ),
    (
        "change.md",
        DocType::Change,
        "work/changes/ledger-rows.md",
        include_str!("fixtures/change.md"),
    ),
    (
        "architecture.md",
        DocType::Architecture,
        "docs/architecture/row-keys.md",
        include_str!("fixtures/architecture.md"),
    ),
    (
        "evidence.md",
        DocType::Evidence,
        "work/evidence/ledger-sync-bound.md",
        include_str!("fixtures/evidence.md"),
    ),
    (
        "assessment.md",
        DocType::Assessment,
        "work/assessments/ledger-sync-explanation.md",
        include_str!("fixtures/assessment.md"),
    ),
    (
        "coverage.md",
        DocType::Coverage,
        "work/coverage/default.md",
        include_str!("fixtures/coverage.md"),
    ),
    (
        "debt.md",
        DocType::Debt,
        "work/debt/default.md",
        include_str!("fixtures/debt.md"),
    ),
];

#[test]
fn each_fixture_document_renders_to_its_own_bytes_and_validates() {
    for (name, doc_type, path, text) in FIXTURES {
        let document = parse(text);
        assert_eq!(document.doc_type(), Some(doc_type), "{name}");
        assert_eq!(
            document.render(),
            text,
            "{name} does not render to its bytes"
        );
        assert_eq!(parse(&document.render()), document, "{name}");
        if let Err(violations) = Profile.validate(&document, path) {
            panic!("{name} does not validate: {violations:#?}");
        }
    }
    // The architecture fixture has no final newline, and keeps none.
    assert!(!FIXTURES[2].3.ends_with('\n'));
}

#[test]
fn crlf_documents_and_a_missing_final_newline_render_byte_identically() {
    for text in [
        "---\r\n# c\r\ntype: Architecture\r\nid: arch:x\r\n---\r\n\r\nbody\r\nmore\r\n",
        "---\ntype: Architecture\nid: arch:x\n---\n\nbody without final newline",
        "---\ntype: Architecture\nid: arch:x\n---\n",
        "---\ntype: Architecture\nid: arch:x\n---",
        "---\r\ntype: Architecture\r\n---",
        "---\ntype: Architecture\n---\nmixed\r\nbody\n",
    ] {
        let document = parse(text);
        assert_eq!(document.render(), text, "{text:?}");
        assert_eq!(parse(&document.render()), document);
    }
}

#[test]
fn the_body_span_locates_the_body_in_the_text() {
    let text = "---\ntype: Architecture\n# c\nid: arch:x\n---\nfirst\nsecond\n";
    let document = parse(text);
    let span = document.body().span();
    assert_eq!(&text[span.bytes.clone()], document.body().text());
    assert_eq!(document.body().text(), "first\nsecond\n");
    assert_eq!(span.lines, 6..8);
    let empty = parse("---\na: b\n---\n");
    assert_eq!(empty.body().span().lines, 4..4);
    assert_eq!(empty.body().span().bytes, 13..13);
}

const CAPABILITY_CANONICAL: &str = "---
# SPDX line
type: Capability
id: cap:x
title: \"T\"
description: \"D\"
tags: [a, b]
timestamp: 2026-01-01T00:00:00Z
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: []
extra: kept
---
body
";

#[test]
fn keys_out_of_order_render_in_table_order_and_then_stay_put() {
    let shuffled = "---
depends: []
extra: kept
bindings:
  code: []
  tests: []
  examples: []
state: designed
timestamp: 2026-01-01T00:00:00Z
tags: [a, b]
description: \"D\"
title: \"T\"
id: cap:x
# SPDX line
type: Capability
---
body
";
    let document = parse(shuffled);
    let rendered = document.render();
    // The comment attached to `type` moves with it, ahead of the key.
    assert_eq!(rendered, CAPABILITY_CANONICAL);
    let again = parse(&rendered);
    assert_eq!(again.render(), rendered);
    assert_eq!(again, document);
}

#[test]
fn record_keys_out_of_order_render_in_table_order_and_validate() {
    let shuffled = "---\nlimits:\n  peak_mib: 64\nby: agent\nfor: [cap:x]\nverdict: pass\n\
        values:\n  peak_mib: 41\ninputs:\n  - kind: symbol\n    name: x::y\n    \
        digest: 2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae\n\
        subject: binding:x::tests::case\nid: evidence:x\ntype: Evidence\n---\n";
    let document = parse(shuffled);
    let rendered = document.render();
    let keys: Vec<&str> = document
        .frontmatter()
        .map()
        .entries()
        .iter()
        .map(Entry::key)
        .collect();
    assert_eq!(keys.len(), 9);
    let order: Vec<&str> = rendered
        .lines()
        .filter(|line| !line.starts_with(' ') && line.contains(':'))
        .map(|line| line.split(':').next().unwrap_or(line))
        .collect();
    assert_eq!(
        order,
        [
            "type", "id", "subject", "inputs", "verdict", "by", "for", "values", "limits"
        ]
    );
    let again = parse(&rendered);
    assert_eq!(again.render(), rendered);
    assert_eq!(again, document);
    if let Err(violations) = Profile.validate(&again, "work/evidence/x.md") {
        panic!("{violations:#?}");
    }
}

#[test]
fn keys_outside_the_table_follow_in_input_order() {
    let text = "---
zeta: 1
type: Architecture
alpha: 2
title: \"T\"
id: arch:x
mid: 3
description: \"D\"
---
";
    let rendered = parse(text).render();
    assert_eq!(
        rendered,
        "---\ntype: Architecture\nid: arch:x\ntitle: \"T\"\ndescription: \"D\"\nzeta: 1\nalpha: 2\nmid: 3\n---\n"
    );
}

#[test]
fn a_document_of_an_unknown_type_keeps_input_order() {
    for text in [
        "---\nzeta: 1\ntype: Concept\nid: x\n---\n",
        "---\nid: x\nzeta: 1\n---\n",
        "---\ntype: capability\nid: x\ntitle: t\n---\n",
    ] {
        assert_eq!(parse(text).render(), text);
    }
    assert_eq!(parse("---\nid: x\ntype: Concept\n---\n").doc_type(), None);
    assert_eq!(
        parse("---\nid: x\ntype: Change\n---\n").doc_type(),
        Some(DocType::Change)
    );
}

#[test]
fn comments_attach_to_the_next_key_and_trailing_comments_stay_on_their_line() {
    let text = "---
# first header
# second header
type: Architecture # trailing on type

# before id, after a blank line
id: arch:x
title: \"T\"   #  spaced trailing
description: \"D\"
tags:
  # before first tag
  - b # trailing on b
  - a
# last comment

# after a blank
---
";
    let document = parse(text);
    let map = document.frontmatter().map();
    let type_entry = map.get("type").expect("type");
    assert_eq!(
        type_entry.comments().leading(),
        ["# first header", "# second header"]
    );
    assert_eq!(
        type_entry.comments().trailing(),
        Some(" # trailing on type")
    );
    let id = map.get("id").expect("id");
    assert_eq!(
        id.comments().leading(),
        ["", "# before id, after a blank line"]
    );
    assert_eq!(
        map.get("title").expect("title").comments().trailing(),
        Some("   #  spaced trailing")
    );
    let Value::List(tags) = map.get("tags").expect("tags").value() else {
        panic!("tags is a list");
    };
    assert_eq!(
        tags.items()[1].comments().leading(),
        ["  # before first tag"]
    );
    assert_eq!(
        tags.items()[1].comments().trailing(),
        Some(" # trailing on b")
    );
    assert_eq!(
        document.frontmatter().trailing_comments(),
        ["# last comment", "", "# after a blank"]
    );

    // Rendering sorts the tags, and the comments travel with their tag.
    let rendered = document.render();
    let expected = text.replace(
        "  # before first tag\n  - b # trailing on b\n  - a\n",
        "  - a\n  # before first tag\n  - b # trailing on b\n",
    );
    assert_eq!(rendered, expected);
    assert_eq!(parse(&rendered).render(), rendered);

    // Comments that follow the last key stay last after a reorder.
    let reordered = parse("---\nid: arch:x\n# on type\ntype: Architecture\n# end\n---\n").render();
    assert_eq!(
        reordered,
        "---\n# on type\ntype: Architecture\nid: arch:x\n# end\n---\n"
    );
}

#[test]
fn tags_render_sorted_and_stay_sorted() {
    let document = parse("---\ntype: Architecture\ntags: [b, a]\n---\n");
    let rendered = document.render();
    assert_eq!(rendered, "---\ntype: Architecture\ntags: [a, b]\n---\n");
    assert_eq!(parse(&rendered).render(), rendered);

    let block = parse("---\ntype: Architecture\ntags:\n  - zed\n  - \"Alpha\"\n  - mid\n---\n");
    assert_eq!(
        block.render(),
        "---\ntype: Architecture\ntags:\n  - \"Alpha\"\n  - mid\n  - zed\n---\n"
    );
    // A list under another key keeps author order.
    let other = parse("---\ntype: Architecture\nnames: [b, a]\n---\n");
    assert_eq!(
        other.render(),
        "---\ntype: Architecture\nnames: [b, a]\n---\n"
    );
}

#[test]
fn flow_lists_are_written_in_one_spacing() {
    let document = parse("---\ntype: Architecture\nnames: [ a,b ,  \"c d\" ]\n---\n");
    assert_eq!(
        document.render(),
        "---\ntype: Architecture\nnames: [a, b, \"c d\"]\n---\n"
    );
}

#[test]
fn a_parsed_document_equals_its_rendering_read_back() {
    for text in [
        CAPABILITY_CANONICAL,
        "---\n# c\ntype: Change\nid: change:x\ntitle: \"T\"\ndescription: \"D\"\nchanges: [cap:x]\naffects:\n  - cap:x#contract\ncontract_effect: changes\nitems:\n  - claim: \"One.\"\n    outcome: implemented\n  # between\n  - claim: 'Two.'\n---\n\n## Summary\n",
        "---\r\nkey: value # c\r\n---\r\ntext",
    ] {
        let document = parse(text);
        assert_eq!(parse(&document.render()), document);
        assert_eq!(document.render(), text);
    }
}

#[test]
fn equality_compares_keys_as_a_map_with_comments_and_body_bytes() {
    let base = parse("---\na: 1\nb: 2\n---\nbody\n");
    assert_eq!(base, parse("---\nb: 2\na: 1\n---\nbody\n"));
    assert_ne!(base, parse("---\na: 1\nb: 3\n---\nbody\n"));
    assert_ne!(base, parse("---\na: 1\nb: 2 # c\n---\nbody\n"));
    assert_ne!(base, parse("---\n# c\na: 1\nb: 2\n---\nbody\n"));
    assert_ne!(base, parse("---\na: 1\nb: 2\n---\nbody\n\n"));
    assert_ne!(base, parse("---\na: 1\nb: 2\n# end\n---\nbody\n"));
    assert_ne!(base, parse("---\na: 1\nb: \"2\"\n---\nbody\n"));
    // The line ending of the fences is layout and does not take part.
    assert_eq!(parse("---\na: 1\n---\nx"), parse("---\r\na: 1\r\n---\r\nx"));
}

#[test]
fn a_change_document_built_from_constructors_renders_and_reads_back_equal() {
    let claim = |text: &str, outcome: Option<&str>| {
        let mut entries = vec![text_entry("claim", text)];
        if let Some(outcome) = outcome {
            entries.push(text_entry("outcome", outcome));
        }
        ListItem::map(Map::new(entries).expect("item map"))
    };
    let flow = |texts: &[&str]| {
        Value::List(List::flow(texts.iter().map(|t| scalar(t)).collect()).expect("flow"))
    };
    let block_scalars = |texts: &[&str]| {
        let items = texts
            .iter()
            .map(|t| ListItem::scalar(scalar(t)).expect("item"))
            .collect();
        Value::List(List::block(items).expect("block"))
    };
    let items = List::block(vec![
        claim("Documents parse: with a colon.", Some("implemented")),
        claim("It's quoted\nacross lines.", None),
    ])
    .expect("items");
    let frontmatter = Frontmatter::new(
        Map::new(vec![
            entry("items", Value::List(items)),
            text_entry("type", "Change"),
            text_entry("id", "change:built"),
            text_entry("title", "A built change"),
            text_entry("description", "true"),
            entry("tags", flow(&["zed", "alpha"])),
            text_entry("timestamp", "2026-01-01T00:00:00Z"),
            entry("changes", flow(&["cap:okf-profile"])),
            entry("affects", block_scalars(&["cap:okf-profile#contract"])),
            text_entry("contract_effect", "none"),
        ])
        .expect("map"),
    );
    let document = Document::new(frontmatter, "\n## Summary\n\nBuilt.\n").expect("document");
    let rendered = document.render();
    assert_eq!(
        rendered,
        "---
type: Change
id: change:built
title: A built change
description: 'true'
tags: [alpha, zed]
timestamp: 2026-01-01T00:00:00Z
changes: [cap:okf-profile]
affects:
  - cap:okf-profile#contract
contract_effect: none
items:
  - claim: 'Documents parse: with a colon.'
    outcome: implemented
  - claim: \"It's quoted\\nacross lines.\"
---

## Summary

Built.
"
    );
    let reread = parse(&rendered);
    assert_eq!(reread.render(), rendered);
    assert_eq!(reread.doc_type(), Some(DocType::Change));
    let Value::List(items) = reread
        .frontmatter()
        .map()
        .get("items")
        .expect("items")
        .value()
    else {
        panic!("items is a list");
    };
    assert_eq!(items.form(), ListForm::Block);
    assert!(matches!(items.items()[0].value(), ItemValue::Map(_)));
}

#[test]
fn a_built_document_with_sorted_tags_equals_what_it_reads_back_as() {
    let tags = List::flow(vec![scalar("alpha"), scalar("zed")]).expect("flow");
    let map = Map::new(vec![
        text_entry("type", "Architecture"),
        entry("tags", Value::List(tags)),
    ])
    .expect("map");
    let document = Document::new(Frontmatter::new(map), "text\n").expect("document");
    assert_eq!(parse(&document.render()), document);
}

#[test]
fn constructors_choose_a_style_the_reader_reads_back() {
    for (text, style) in [
        ("word", ScalarStyle::Plain),
        ("two words", ScalarStyle::Plain),
        ("cap:okf-profile", ScalarStyle::Plain),
        ("a: b", ScalarStyle::SingleQuoted),
        ("it's # not", ScalarStyle::SingleQuoted),
        ("", ScalarStyle::SingleQuoted),
        (" lead", ScalarStyle::SingleQuoted),
        ("trail ", ScalarStyle::SingleQuoted),
        ("&x", ScalarStyle::SingleQuoted),
        ("[x]", ScalarStyle::SingleQuoted),
        ("a,b", ScalarStyle::SingleQuoted),
        ("true", ScalarStyle::SingleQuoted),
        ("1.5", ScalarStyle::SingleQuoted),
        ("+1e2", ScalarStyle::SingleQuoted),
        (".5", ScalarStyle::SingleQuoted),
        ("1.", ScalarStyle::SingleQuoted),
        ("INF", ScalarStyle::SingleQuoted),
        ("-infinity", ScalarStyle::SingleQuoted),
        ("+NaN", ScalarStyle::SingleQuoted),
        ("-.Inf", ScalarStyle::SingleQuoted),
        ("1.5x", ScalarStyle::Plain),
        ("infinite", ScalarStyle::Plain),
        ("0x1F", ScalarStyle::SingleQuoted),
        ("null", ScalarStyle::SingleQuoted),
        ("line\nbreak", ScalarStyle::DoubleQuoted),
        ("tab\there", ScalarStyle::DoubleQuoted),
        ("nul\0here", ScalarStyle::DoubleQuoted),
    ] {
        let built = scalar(text);
        assert_eq!(built.style(), style, "{text:?}");
        let map = Map::new(vec![entry("k", Value::Scalar(built))]).expect("map");
        let document = Document::new(Frontmatter::new(map), "").expect("document");
        let reread = parse(&document.render());
        assert_eq!(reread, document, "{text:?}");
        let value = reread
            .frontmatter()
            .map()
            .get("k")
            .expect("k")
            .value()
            .as_scalar()
            .expect("scalar")
            .text()
            .to_string();
        assert_eq!(value, text);
    }
}

#[test]
fn every_escape_round_trips_and_other_control_characters_are_refused() {
    for (name, value) in ESCAPES {
        let text = format!("a{value}b");
        let built = scalar(&text);
        let map = Map::new(vec![entry("k", Value::Scalar(built))]).expect("map");
        let rendered = Document::new(Frontmatter::new(map), "")
            .expect("document")
            .render();
        let reread = parse(&rendered);
        assert_eq!(
            reread
                .frontmatter()
                .map()
                .get("k")
                .expect("k")
                .value()
                .as_scalar()
                .expect("scalar")
                .text(),
            text
        );
        assert_eq!(parse(&rendered).render(), rendered, "escape {name}");
        let spelled = format!("---\nk: \"a\\{name}b\"\n---\n");
        assert_eq!(parse(&spelled).render(), spelled);
    }
    for control in [
        '\u{1}', '\u{7}', '\u{8}', '\u{b}', '\u{c}', '\u{1b}', '\u{7f}', '\u{85}',
    ] {
        assert_eq!(
            Scalar::new(format!("x{control}y")),
            Err(ConstructError::Unrepresentable { character: control })
        );
    }
    assert!(Document::parse("---\nk: \"\\q\"\n---\n").is_err());
    assert!(Document::parse("---\nk: \"\\x41\"\n---\n").is_err());
}

#[test]
fn constructors_refuse_what_the_reader_would_not_read_back() {
    assert_eq!(Map::new(vec![]).map(|_| ()), Err(ConstructError::EmptyMap));
    for key in ["", "a b", "a:b", "é", "<<"] {
        assert_eq!(
            Entry::new(key, Value::Scalar(scalar("v"))).map(|_| ()),
            Err(ConstructError::InvalidKey {
                key: key.to_string()
            })
        );
    }
    let duplicate = Map::new(vec![text_entry("k", "a"), text_entry("k", "b")]);
    assert_eq!(
        duplicate.map(|_| ()),
        Err(ConstructError::DuplicateKey {
            key: "k".to_string()
        })
    );
    assert!(Map::new(vec![text_entry("k", "a"), text_entry("j", "b")]).is_ok());

    assert_eq!(
        List::block(vec![]).map(|_| ()),
        Err(ConstructError::EmptyBlockList)
    );
    assert!(List::flow(vec![]).is_ok());
    let item = ListItem::scalar(scalar("x")).expect("item");
    let map_item = ListItem::map(Map::new(vec![text_entry("k", "v")]).expect("map"));
    assert_eq!(
        List::block(vec![item.clone(), map_item.clone()]).map(|_| ()),
        Err(ConstructError::MixedList)
    );
    assert!(List::block(vec![item.clone(), item]).is_ok());
    assert!(List::block(vec![map_item.clone(), map_item]).is_ok());

    // An unquoted scalar read from a block position cannot move into a flow
    // list when it holds a comma; a quoted one can.
    let parsed = parse("---\nk: a, b\nq: \"a, b\"\n---\n");
    let plain = parsed
        .frontmatter()
        .map()
        .get("k")
        .expect("k")
        .value()
        .as_scalar()
        .expect("scalar")
        .clone();
    let quoted = parsed
        .frontmatter()
        .map()
        .get("q")
        .expect("q")
        .value()
        .as_scalar()
        .expect("scalar")
        .clone();
    assert_eq!(
        List::flow(vec![plain.clone()]).map(|_| ()),
        Err(ConstructError::NotFlowSafe {
            text: "a, b".to_string()
        })
    );
    assert!(List::flow(vec![quoted]).is_ok());
    let empty = parse("---\nk:\n---\n");
    let empty_scalar = empty
        .frontmatter()
        .map()
        .get("k")
        .expect("k")
        .value()
        .as_scalar()
        .expect("scalar")
        .clone();
    assert_eq!(empty_scalar.text(), "");
    assert_eq!(
        List::flow(vec![empty_scalar.clone()]).map(|_| ()),
        Err(ConstructError::EmptyListItem)
    );
    assert_eq!(
        ListItem::scalar(empty_scalar).map(|_| ()),
        Err(ConstructError::EmptyListItem)
    );
    assert!(ListItem::scalar(scalar("")).is_ok());
}

#[test]
fn a_parsed_empty_value_renders_as_an_empty_value() {
    let text = "---\nk:\nj: ''\n---\n";
    let document = parse(text);
    assert_eq!(document.render(), text);
}

#[test]
fn constructed_nesting_and_size_are_bounded_like_the_reader() {
    let mut value = Value::Map(Map::new(vec![text_entry("leaf", "v")]).expect("map"));
    for level in 1..MAX_YAML_DEPTH - 1 {
        value = Value::Map(Map::new(vec![entry(&format!("k{level}"), value)]).expect("map"));
    }
    let top = Map::new(vec![entry("top", value.clone())]).expect("bound");
    let document = Document::new(Frontmatter::new(top), "").expect("document");
    assert_eq!(parse(&document.render()), document);
    assert_eq!(
        Map::new(vec![entry(
            "over",
            Value::Map(Map::new(vec![entry("top", value)]).expect("at bound"))
        )])
        .map(|_| ()),
        Err(ConstructError::DepthExceeded {
            max: MAX_YAML_DEPTH
        })
    );

    let map = Map::new(vec![text_entry("k", "v")]).expect("map");
    let big = "x".repeat(MAX_TEXT_BYTES);
    assert!(matches!(
        Document::new(Frontmatter::new(map.clone()), big),
        Err(ConstructError::TooLarge {
            max: MAX_TEXT_BYTES,
            ..
        })
    ));
    let head = "---\nk: v\n---\n";
    let fits = "x".repeat(MAX_TEXT_BYTES - head.len());
    let document = Document::new(Frontmatter::new(map), fits).expect("exactly at the bound");
    assert_eq!(parse(&document.render()), document);
}

#[test]
fn unsorted_tags_render_sorted_and_the_document_equals_its_rendering_read_back() {
    for text in [
        "---\ntype: Architecture\ntags: [b, a]\n---\nbody\n",
        "---\ntype: Capability\ntags:\n  - b # on b\n  - a\n---\nbody\n",
        "---\ntags: [z, y]\ntype: Concept\n---\n",
    ] {
        let document = parse(text);
        let rendered = document.render();
        assert_eq!(parse(&rendered), document, "{text:?}");
        assert_eq!(parse(&rendered).render(), rendered);
    }
    let document = parse("---\ntype: Architecture\ntags: [b, a]\n---\nbody\n");
    assert_eq!(
        document.render(),
        "---\ntype: Architecture\ntags: [a, b]\n---\nbody\n"
    );
    assert_eq!(
        document,
        parse("---\ntype: Architecture\ntags: [a, b]\n---\nbody\n")
    );
    // Only the tags list is ordered; other lists differ when their order differs.
    assert_ne!(
        parse("---\nnames: [b, a]\n---\n"),
        parse("---\nnames: [a, b]\n---\n")
    );
}

#[test]
fn the_body_span_uses_the_coordinates_of_the_text_the_document_came_from() {
    // Parsed: coordinates of the source text, even when render reorders keys.
    let source = "---\nid: arch:x\ntype: Architecture\n---\nbody\n";
    let parsed = parse(source);
    let span = parsed.body().span();
    assert_eq!(&source[span.bytes.clone()], "body\n");
    assert_eq!(span.lines, 5..6);

    // Built: coordinates of the rendered text.
    let map = Map::new(vec![
        text_entry("type", "Architecture"),
        text_entry("id", "arch:x"),
    ])
    .expect("map");
    let built = Document::new(Frontmatter::new(map), "first\nsecond").expect("document");
    let rendered = built.render();
    let span = built.body().span();
    assert_eq!(&rendered[span.bytes.clone()], "first\nsecond");
    assert_eq!(span.lines, 5..7);
    assert_eq!(built.body().text(), "first\nsecond");
}
