// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Production source collection and matching over inert workspace fixtures.

use super::super::workspace;
use super::*;
use raf_source::{MemoryTree, Workspace};

const CORE: &str = "let x = input + 1; let y = x * 3; sink(y, x, input, 7);";
const RENAMED: &str =
    "let renamed = other + 1; let result = renamed * 3; sink(result, renamed, other, 7);";

fn load(source: &str) -> Workspace {
    Workspace::load(
        &MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
            ),
            ("src/lib.rs", source),
        ]),
        &["."],
    )
    .unwrap()
}

fn analyze(
    workspace: &Workspace,
    work: usize,
    bytes: usize,
    limits: Limits,
) -> Result<Matches<'_>, Refusal> {
    let source = workspace::collect(workspace, work, bytes).map_err(|error| match error {
        super::super::analysis::SourceFailure::Located { cause, .. } => cause.cause,
        other @ super::super::analysis::SourceFailure::Native { .. } => {
            panic!("unexpected native fixture refusal: {other}")
        }
    })?;
    find(source, limits)
}

fn text<'a>(matches: &'a Matches<'a>, site: Site) -> &'a str {
    let function = &matches.analysis.functions[site.function];
    let (file, _) = function.origin.source().unwrap();
    assert!(std::ptr::eq(file.text.as_str(), function.source));
    assert_eq!(
        site.first_line,
        function.source[..site.bytes.start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1
    );
    &function.source[site.bytes.start..site.bytes.end]
}

#[test]
fn an_embedded_helper_survives_unrelated_locals_and_a_conflicting_surrounding_binding_map() {
    let source = format!(
        "#![no_implicit_prelude]\nfn small(input: i32) {{ let conflict = input; emit(conflict, conflict); {CORE} }}\nfn large(other: i32) {{ let noise = other; let conflict = other; emit(conflict, noise); {RENAMED} tail(); }}"
    );
    let workspace = load(&source);
    let matches = analyze(&workspace, 100_000_000, 10_000_000, Limits::default()).unwrap();
    let witness = matches
        .witnesses
        .iter()
        .find(|witness| {
            text(&matches, witness.left) == CORE && text(&matches, witness.right) == RENAMED
        })
        .expect("a bad outer binding map must not poison an independently verified inner helper");
    assert!(!matches.candidates.is_empty());
    let candidate = matches.candidates[witness.candidate];
    assert!(candidate.alignment.is_some());
    for site in [witness.left, witness.right] {
        let function = &matches.analysis.functions[site.function];
        let region = Region {
            bytes: site.bytes.start..site.bytes.end,
            atoms: &function.tokens.atoms[site.atom_start..site.atom_end],
            declarations: &function.facts.declarations,
        };
        assert_eq!(
            proof::digest(&region, &matches.analysis.meter).unwrap(),
            witness.digest
        );
    }
    for changed in [
        RENAMED.replace("* 3", "- 3"),
        RENAMED.replace("result, renamed", "result, result"),
    ] {
        let source = format!(
            "#![no_implicit_prelude]\nfn small(input: i32) {{ {CORE} }}\nfn changed(other: i32) {{ prefix(); {changed} tail(); }}"
        );
        let workspace = load(&source);
        assert!(
            analyze(&workspace, 100_000_000, 10_000_000, Limits::default())
                .unwrap()
                .witnesses
                .is_empty()
        );
    }
}

#[test]
fn repetition_counts_distinct_disjoint_occurrences_while_short_input_and_boilerplate_stay_empty() {
    let block = format!("{{ {CORE} }}");
    let source =
        format!("#![no_implicit_prelude]\nfn repeated(input: i32) {{ {block} {block} {block} }}");
    let workspace = load(&source);
    let matches = analyze(&workspace, 100_000_000, 10_000_000, Limits::default()).unwrap();
    let spans = source
        .match_indices(&block)
        .map(|(start, text)| Span {
            start,
            end: start + text.len(),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        matches
            .witnesses
            .iter()
            .map(|witness| (witness.left.bytes, witness.right.bytes))
            .collect::<Vec<_>>(),
        [
            (spans[0], spans[1]),
            (spans[0], spans[2]),
            (spans[1], spans[2])
        ]
    );
    for witness in &matches.witnesses {
        assert!(!overlap(witness.left.bytes, witness.right.bytes));
        assert_eq!(text(&matches, witness.left), block);
        assert_eq!(text(&matches, witness.right), block);
    }
    // The first/last occurrence crosses other larger correspondences. Joint
    // byte containment alone must not erase that different atom alignment.
    let source = format!(
        "#![no_implicit_prelude]\nfn crossing(input:i32) {{ {block} {block} {block} {block} {block} }}"
    );
    let workspace = load(&source);
    let matches = analyze(&workspace, 100_000_000, 10_000_000, Limits::default()).unwrap();
    let spans = source
        .match_indices(&block)
        .map(|(start, text)| Span {
            start,
            end: start + text.len(),
        })
        .collect::<Vec<_>>();
    assert!(
        matches
            .witnesses
            .iter()
            .any(|witness| witness.left.bytes == spans[0] && witness.right.bytes == spans[4])
    );
    assert!(matches.witnesses.iter().any(|witness| witness.left.bytes
        == Span {
            start: spans[0].start,
            end: spans[1].end
        }
        && witness.right.bytes
            == Span {
                start: spans[3].start,
                end: spans[4].end
            }));
    assert_eq!(matches.witnesses.len(), 8);
    let boilerplate = "send(input);".repeat(16);
    let one_operator = format!("{}a", "a && b && ".repeat(4));
    let receiver_plumbing = "sink(self, input, 1);".repeat(4);
    for source in [
        String::new(),
        "fn empty() {} fn short(input:i32) { send(input); }".to_owned(),
        format!("fn one(input:i32) {{ {boilerplate} }} fn two(input:i32) {{ {boilerplate} }}"),
        format!(
            "fn one(a:bool,b:bool) {{ {one_operator}; }} fn two(a:bool,b:bool) {{ {one_operator}; }}"
        ),
        format!(
            "struct S {{}} impl S {{ fn one(&self,input:i32) {{ {receiver_plumbing} }} fn two(&self,input:i32) {{ {receiver_plumbing} }} }}"
        ),
    ] {
        let workspace = load(&format!("#![no_implicit_prelude]\n{source}"));
        let matches = analyze(&workspace, 100_000_000, 10_000_000, Limits::default()).unwrap();
        assert!(matches.witnesses.is_empty());
        assert_eq!(matches.counts.postings, 0);
    }
    for source in [
        format!(
            "fn one(a:bool,b:bool) {{ {}; }} fn two(a:bool,b:bool) {{ {}; }}",
            one_operator.replace("&& b &&", "&& b ||"),
            one_operator.replace("&& b &&", "&& b ||")
        ),
        format!(
            "struct S {{}} impl S {{ fn one(&self,input:i32) {{ {CORE} }} fn two(&self,input:i32) {{ {CORE} }} }}"
        ),
    ] {
        let workspace = load(&format!("#![no_implicit_prelude]\n{source}"));
        assert!(
            !analyze(&workspace, 100_000_000, 10_000_000, Limits::default())
                .unwrap()
                .witnesses
                .is_empty()
        );
    }
}

#[test]
fn selected_hash_collisions_remain_candidates_without_coarse_or_binding_proof() {
    for (changed, coarse) in [
        (CORE.replace("+ 1", "- 1"), false),
        (CORE.replace("y, x, input", "y, y, input"), true),
    ] {
        let workspace = load(&format!(
            "#![no_implicit_prelude]\nfn one(input:i32) {{ {CORE} }} fn two(input:i32) {{ {changed} }}"
        ));
        let analysis = workspace::collect(&workspace, 100_000_000, 10_000_000).unwrap();
        let at = analysis.functions[0]
            .tokens
            .atoms
            .iter()
            .position(|atom| atom.spelling == "+")
            .unwrap()
            - 2;
        // Only the digest is synthetic; both seed ranges and all source owners
        // are actual production records. No fixture program is executed.
        let mut postings = [
            Posting {
                digest: [0; 32],
                anchor: Anchor {
                    function: 0,
                    atom: at,
                },
            },
            Posting {
                digest: [0; 32],
                anchor: Anchor {
                    function: 1,
                    atom: at,
                },
            },
        ];
        let mut candidates = Vec::new();
        let mut witnesses = Vec::new();
        let mut counts = Counts::default();
        run(
            &analysis,
            &mut postings,
            Limits::default(),
            &mut candidates,
            &mut witnesses,
            &mut counts,
        )
        .unwrap();
        assert_eq!(counts.seed_pairs, 1);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].seed_digest, [0; 32]);
        assert_eq!(candidates[0].left, postings[0].anchor);
        assert_eq!(candidates[0].right, postings[1].anchor);
        assert_eq!(candidates[0].alignment.is_some(), coarse);
        assert!(witnesses.is_empty());
        let evidence = Matches {
            candidates,
            witnesses,
            counts,
            analysis,
        };
        let graph = super::super::report::tests::graph(&evidence);
        let root = graph.get("").unwrap();
        let vocab = super::super::report::VOCAB;
        assert_eq!(root.values(&format!("{vocab}candidate")).count(), 1);
        assert_eq!(root.values(&format!("{vocab}witness")).count(), 0);
    }
}

#[test]
fn indexing_repetition_and_failed_candidates_share_exact_original_grants_without_partial_results() {
    let workspace = load(&format!(
        "#![no_implicit_prelude]\nfn one(input:i32) {{ {CORE} }} fn two(other:i32) {{ {RENAMED} }}"
    ));
    let measured = analyze(&workspace, 100_000_000, 10_000_000, Limits::default()).unwrap();
    assert!(!measured.witnesses.is_empty());
    let counts = measured.counts;
    let exact = Limits {
        atoms: counts.atoms,
        postings: counts.postings,
        seed_pairs: counts.seed_pairs,
        candidates: measured.candidates.len(),
        regions: counts.regions,
        verified: counts.verified,
    };
    let (work, bytes) = measured.analysis.meter.usage();
    assert!(analyze(&workspace, work, bytes, exact).is_ok());
    assert!(matches!(
        analyze(&workspace, work - 1, bytes, exact),
        Err(Refusal::Work { .. })
    ));
    assert!(matches!(
        analyze(&workspace, work, bytes - 1, exact),
        Err(Refusal::Memory { .. })
    ));
    for (limits, resource) in [
        (
            Limits {
                atoms: exact.atoms - 1,
                ..exact
            },
            "indexed atoms",
        ),
        (
            Limits {
                postings: exact.postings - 1,
                ..exact
            },
            "seed postings",
        ),
        (
            Limits {
                seed_pairs: exact.seed_pairs - 1,
                ..exact
            },
            "matching-hash seed pairs",
        ),
        (
            Limits {
                candidates: exact.candidates - 1,
                ..exact
            },
            "discovery candidates",
        ),
        (
            Limits {
                regions: exact.regions - 1,
                ..exact
            },
            "syntax-region proposals",
        ),
        (
            Limits {
                verified: exact.verified - 1,
                ..exact
            },
            "verified region pairs before containment",
        ),
    ] {
        assert!(
            matches!(analyze(&workspace, 100_000_000, 10_000_000, limits), Err(Refusal::Count { resource: found, .. }) if found == resource)
        );
    }
}
