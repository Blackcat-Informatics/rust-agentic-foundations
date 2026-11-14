// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Inert source and RDF evidence fixtures. Neither imports nor reports grant execution.

use super::*;
mod fixture;
use crate::embedded::{analysis::Origin, pairing, policy, workspace};
use fixture::{
    accepts, assertion, boolean, class, count, digest, number, one, owner_key, property, put,
    reference, region_key, roundtrip, text, witness_key,
};
pub(in crate::embedded) use fixture::{graph, key};
use raf_rdf::{Builder, Graph, TermValue as Term};
use raf_source::{MemoryTree, Workspace};

const CORE: &str = "let x = input + 1; let y = x * 3; sink(y, x, input, 7);";
const WORK_BUDGET: usize = 100_000_000;
const RENAMED: &str =
    "let renamed = other + 1; let result = renamed * 3; sink(result, renamed, other, 7);";

fn load(reverse: bool) -> Workspace {
    let small = format!("fn small(input:i32) {{ {CORE} }}");
    let large = format!("fn large(other:i32) {{ prefix(); {RENAMED} tail(); }}");
    let mut files = [
        (
            "Cargo.toml",
            "[package]\nname=\"fixture\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
        ),
        (
            "src/lib.rs",
            "#![no_implicit_prelude]\nmod small; mod large;",
        ),
        (
            "src/main.rs",
            "#![no_implicit_prelude]\nmod small; mod large; fn main() {}",
        ),
        ("src/small.rs", small.as_str()),
        ("src/large.rs", large.as_str()),
    ];
    if reverse {
        files.reverse();
    }
    let tree: MemoryTree = files.into_iter().collect();
    Workspace::load(&tree, &["."]).unwrap()
}

fn analyze(workspace: &Workspace, reverse: bool) -> pairing::Matches<'_> {
    let mut analysis = workspace::collect(workspace, WORK_BUDGET, 10_000_000).unwrap();
    if reverse {
        analysis.functions.reverse();
        for function in &mut analysis.functions {
            if let Origin::Source { owners, .. } = &mut function.origin {
                owners.reverse();
            }
        }
    }
    pairing::find(analysis, policy::Limits::default()).unwrap()
}

#[test]
fn canonical_reports_keep_actual_spans_and_all_owners_under_source_and_collection_reordering() {
    let mut expected = None;
    for reverse in [false, true] {
        let workspace = load(reverse);
        let matches = analyze(&workspace, reverse);
        assert!(!matches.witnesses.is_empty());
        let graph = graph(&matches);
        let root = graph.get("").unwrap();
        assert!(
            root.values(&property("candidate")).count()
                >= root.values(&property("witness")).count()
        );
        let mut checked = false;
        for value in root.values(&property("witness")) {
            let witness = graph.get(value.iri().unwrap()).unwrap();
            let mut fragments = Vec::new();
            for endpoint in ["left", "right"] {
                let region = graph.get(reference(witness, endpoint).unwrap()).unwrap();
                let function = graph.get(reference(region, "function").unwrap()).unwrap();
                assert_eq!(function.values(&property("owner")).count(), 2);
                assert_eq!(count(function, "edition").unwrap(), 2024);
                assert!(boolean(function, "noImplicitPrelude").unwrap());
                let source = graph.get(reference(function, "source").unwrap()).unwrap();
                assert_eq!(count(source, "sourceAuthority").unwrap(), 0);
                let path = raf_source::RelPath::new(text(source, "file").unwrap()).unwrap();
                let original = workspace.file(&path).unwrap();
                assert_eq!(
                    digest(source, "sourceDigest").unwrap(),
                    purrdf_hash::hex::Lower(
                        purrdf_hash::blake3::hash(original.text.as_bytes()).as_bytes()
                    )
                    .to_string()
                );
                fragments.push(
                    &original.text
                        [count(region, "startByte").unwrap()..count(region, "endByte").unwrap()],
                );
            }
            checked |= fragments.contains(&CORE) && fragments.contains(&RENAMED);
        }
        assert!(
            checked,
            "report endpoints must resolve to the actual helper regions"
        );
        let texts = [raf_rdf::Format::Turtle, raf_rdf::Format::NTriples].map(|format| {
            let text = graph.to_text(format).unwrap();
            assert_eq!(roundtrip(&text, format).unwrap(), text);
            text
        });
        if let Some(expected) = &expected {
            assert_eq!(&texts, expected);
        } else {
            expected = Some(texts);
        }
    }
    for source in ["", "fn empty() {}"] {
        let tree = MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname=\"fixture\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
            ),
            ("src/lib.rs", source),
        ]);
        let workspace = Workspace::load(&tree, &["."]).unwrap();
        let matches = analyze(&workspace, false);
        let graph = graph(&matches);
        let root = graph.get("").unwrap();
        assert_eq!(root.values(&property("candidate")).count(), 0);
        assert_eq!(root.values(&property("witness")).count(), 0);
        assert!(accepts(&graph));
    }
    let workspace = load(false);
    fixture::exhausted_report(&analyze(&workspace, false), WORK_BUDGET);
}

fn replace(builder: &mut Builder, graph: &Graph, id: &str, field: &str, value: Term) {
    let before = one(graph.get(id).unwrap(), field).unwrap().to_owned();
    assert!(builder.remove(assertion(id, field, before)).unwrap());
    put(builder, id, field, value).unwrap();
}

#[test]
fn imported_claims_refuse_unknown_profiles_impossible_ranges_and_invalid_support_beside_valid_evidence()
 {
    let meter = Meter::new(100_000_000, 10_000_000);
    Claims::from_text(
        &" ".repeat(MAX_INPUT_BYTES + 1),
        raf_rdf::Format::Turtle,
        &meter,
        |outcome| {
            let Err(cause) = outcome else {
                panic!("oversized input must refuse")
            };
            assert!(cause.to_string().contains("input byte limit"));
        },
    );
    let workspace = load(false);
    let matches = analyze(&workspace, false);
    let graph = graph(&matches);
    let witness = reference(graph.get("").unwrap(), "witness").unwrap();
    let left = reference(graph.get(witness).unwrap(), "left").unwrap();
    let candidate = reference(graph.get(witness).unwrap(), "candidate").unwrap();
    for refusal in [
        "unknown-property",
        "profile",
        "line",
        "reversed-atoms",
        "partial-alignment",
        "seed-range",
        "candidate-only",
        "authority",
    ] {
        let mut builder = Builder::from_graph(&graph);
        match refusal {
            "unknown-property" => {
                put(&mut builder, "", "runConsumer", Term::boolean(true)).unwrap();
            }
            "profile" => replace(
                &mut builder,
                &graph,
                "",
                "matchingProfile",
                Term::simple_literal("unknown"),
            ),
            "line" => replace(&mut builder, &graph, left, "firstLine", number(0)),
            "reversed-atoms" => replace(&mut builder, &graph, left, "atomEnd", number(0)),
            "partial-alignment" => {
                let before = one(graph.get(candidate).unwrap(), "alignmentRight")
                    .unwrap()
                    .to_owned();
                assert!(
                    builder
                        .remove(assertion(candidate, "alignmentRight", before))
                        .unwrap()
                );
            }
            "seed-range" => replace(
                &mut builder,
                &graph,
                candidate,
                "leftAtom",
                number(usize::MAX),
            ),
            "candidate-only" => {
                for field in ["alignmentLeft", "alignmentRight", "alignmentLength"] {
                    let before = one(graph.get(candidate).unwrap(), field)
                        .unwrap()
                        .to_owned();
                    assert!(builder.remove(assertion(candidate, field, before)).unwrap());
                }
            }
            "authority" => {
                let function = reference(graph.get(left).unwrap(), "function").unwrap();
                let source = reference(graph.get(function).unwrap(), "source").unwrap();
                replace(&mut builder, &graph, source, "sourceAuthority", number(7));
            }
            _ => unreachable!(),
        }
        assert!(!accepts(&builder.finish().unwrap()), "{refusal}");
        assert!(accepts(&graph));
    }
}

fn rename_nodes(graph: &Graph, names: &std::collections::BTreeMap<String, String>) -> Graph {
    let mut builder = Builder::new();
    for mut triple in graph.assertions() {
        if let Some(name) = triple.subject.as_iri().and_then(|id| names.get(id)) {
            triple.subject = Term::iri(name);
        }
        if let Some(name) = triple.object.as_iri().and_then(|id| names.get(id)) {
            triple.object = Term::iri(name);
        }
        builder.insert(triple).unwrap();
    }
    builder.finish().unwrap()
}

// Consistently re-name fixture records after changing their claimed facts.
// These cases must reach semantic refusal, rather than fail on stale node IDs.
fn remint(graph: &Graph, kind: &str) -> Graph {
    let names = graph
        .nodes()
        .filter_map(|(id, node)| {
            if class(node).unwrap() != kind {
                return None;
            }
            let name = match kind {
                "Owner" => owner_key(reference(node, "function").unwrap(), node).unwrap(),
                "Region" => region_key(
                    reference(node, "function").unwrap(),
                    count(node, "startByte").unwrap(),
                    count(node, "endByte").unwrap(),
                    count(node, "atomStart").unwrap(),
                    count(node, "atomEnd").unwrap(),
                ),
                "Witness" => witness_key(
                    reference(node, "left").unwrap(),
                    reference(node, "right").unwrap(),
                    digest(node, "digest").unwrap(),
                ),
                _ => unreachable!(),
            };
            Some((id, name))
        })
        .collect();
    rename_nodes(graph, &names)
}

#[test]
fn reminted_claims_cannot_alias_native_paths_exceed_the_source_or_bypass_the_match_floor() {
    let workspace = load(false);
    let matches = analyze(&workspace, false);
    let graph = graph(&matches);
    let witness = reference(graph.get("").unwrap(), "witness").unwrap();
    let left = reference(graph.get(witness).unwrap(), "left").unwrap();
    let right = reference(graph.get(witness).unwrap(), "right").unwrap();
    let function = graph
        .get(reference(graph.get(left).unwrap(), "function").unwrap())
        .unwrap();
    let owner = function
        .values(&property("owner"))
        .next()
        .unwrap()
        .iri()
        .unwrap();
    let mut accepted = Vec::new();
    for refusal in [
        "manifest-path",
        "target-root-path",
        "owner-range",
        "short-witness",
    ] {
        let mut builder = Builder::from_graph(&graph);
        match refusal {
            "manifest-path" => replace(
                &mut builder,
                &graph,
                owner,
                "manifest",
                Term::simple_literal("./Cargo.toml"),
            ),
            "target-root-path" => replace(
                &mut builder,
                &graph,
                owner,
                "targetRoot",
                Term::simple_literal("src//lib.rs"),
            ),
            "owner-range" => replace(
                &mut builder,
                &graph,
                owner,
                "itemEndByte",
                number(usize::MAX),
            ),
            "short-witness" => {
                for endpoint in [left, right] {
                    let start = count(graph.get(endpoint).unwrap(), "atomStart").unwrap();
                    replace(&mut builder, &graph, endpoint, "atomEnd", number(start + 1));
                }
            }
            _ => unreachable!(),
        }
        let changed = builder.finish().unwrap();
        let changed = if refusal == "short-witness" {
            remint(&remint(&changed, "Region"), "Witness")
        } else {
            remint(&changed, "Owner")
        };
        if accepts(&changed) {
            accepted.push(refusal);
        }
        assert!(accepts(&graph));
    }
    assert!(
        accepted.is_empty(),
        "impossible claims were accepted: {accepted:?}"
    );
}

/// INVARIANT: the limit helper accepts a length exactly at the limit and
/// refuses one byte over it. This exercises the helper directly; the rendering
/// path is covered by the test below.
#[test]
fn the_limit_helper_refuses_one_byte_over_beside_a_length_at_the_limit() {
    let meter = Meter::new(1_000, 1_000);
    assert!(within_limit(MAX_INPUT_BYTES, MAX_INPUT_BYTES, &meter).is_ok());
    let Err(cause) = within_limit(MAX_INPUT_BYTES + 1, MAX_INPUT_BYTES, &meter) else {
        panic!("an oversized rendering must refuse")
    };
    assert!(cause.to_string().contains("output byte limit"));
}

/// INVARIANT: the rendering path itself enforces the output byte limit. The
/// same rendered report is accepted when its length equals the limit and
/// refused when the limit is one byte below its length. The production
/// constant is too large to render, so the test passes the rendered length
/// itself as the limit.
#[test]
fn rendering_a_report_refuses_one_byte_over_the_limit_beside_the_same_report_at_it() {
    let workspace = load(false);
    let matches = analyze(&workspace, false);
    let meter = matches.analysis.meter();
    Claims::produce(&matches, |outcome| {
        let claims = outcome.unwrap();
        let length =
            claims.to_text_admitted(raf_rdf::Format::Turtle, meter, |text| text.unwrap().len());
        assert!(length > 0);
        claims.to_text_within(raf_rdf::Format::Turtle, length, meter, |text| {
            assert_eq!(text.unwrap().len(), length);
        });
        claims.to_text_within(raf_rdf::Format::Turtle, length - 1, meter, |text| {
            let Err(cause) = text else {
                panic!("a rendering one byte over the limit must refuse")
            };
            assert!(cause.to_string().contains("output byte limit"), "{cause}");
        });
    });
}
