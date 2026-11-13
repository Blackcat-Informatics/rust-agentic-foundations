// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Original source is fixture data. No fixture program is compiled or run.

use super::*;
use crate::embedded::analysis::Analysis;
use crate::embedded::proof::{Region, same};
use crate::embedded::regions;

// Malformed-stream fixtures use the same inventory and production join.
fn join(
    source: &str,
    facts: &Facts<'_>,
    original: TokenStream,
    meter: &Meter,
) -> Result<(), Refusal> {
    let coordinates = Coordinates::new(source, None, meter)?;
    let original = inventory(original, meter, &coordinates)?;
    super::join(source, facts, original.as_slice(), meter).map(|_| ())
}

fn collect<'source>(analysis: Analysis<'source>, source: &'source str) -> Analysis<'source> {
    try_collect(analysis, source).unwrap()
}

fn try_collect<'source>(
    mut analysis: Analysis<'source>,
    source: &'source str,
) -> Result<Analysis<'source>, crate::embedded::collect::Failure> {
    let path = raf_source::RelPath::new("tokens.rs").unwrap();
    raf_source::parse_source_with_tokens(&path, source, |syntax, original| {
        let function = syntax
            .items
            .iter()
            .find_map(|item| match item {
                syn::Item::Fn(function) => Some(function),
                _ => None,
            })
            .unwrap();
        analysis.with_function(
            source,
            &function.sig,
            &function.block,
            (syntax, original),
            None,
            |scopes| scopes.enter_module(&syntax.items, &syntax.attrs, true),
        )
    })
    .unwrap()?;
    Ok(analysis)
}

fn span(source: &str, snippet: &str) -> Span {
    let start = source.find(snippet).unwrap();
    Span {
        start,
        end: start + snippet.len(),
    }
}

fn equal(analysis: &Analysis<'_>, left: &str, right: &str) -> bool {
    let views = analysis
        .functions()
        .iter()
        .zip([left, right])
        .map(|(function, snippet)| {
            let tokens = &function.tokens;
            let admitted = regions::admit(
                function.source,
                &function.facts,
                tokens,
                span(function.source, snippet),
                analysis.meter(),
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                function
                    .source
                    .get(admitted.bytes.start..admitted.bytes.end),
                Some(snippet)
            );
            let first_line = function.source[..admitted.bytes.start]
                .bytes()
                .filter(|b| *b == b'\n')
                .count()
                + 1;
            assert_eq!(admitted.first_line, first_line);
            assert!(admitted.last_line >= admitted.first_line);
            Region {
                bytes: admitted.bytes.start..admitted.bytes.end,
                atoms: &tokens.atoms[admitted.atoms],
                declarations: &function.facts.declarations,
            }
        })
        .collect::<Vec<_>>();
    same(&views[0], &views[1], analysis.meter()).unwrap()
}

#[test]
fn original_tokens_verify_an_embedded_renamed_region_and_reject_variable_operator_and_spacing_changes()
 {
    let left = "#![no_implicit_prelude]\n/// Helper documentation outside the body.\nfn small(input: i32) { let café = input + 1; sink(cafe\u{301}, café); }";
    let right = "#![no_implicit_prelude]\nfn large(other: i32) { let unrelated = 9; log(unrelated); let renamed = other + 1; sink(renamed, renamed); tail(); }";
    let analysis = collect(collect(Analysis::new(10_000_000, 1_000_000), left), right);
    assert!(equal(
        &analysis,
        "let café = input + 1; sink(cafe\u{301}, café);",
        "let renamed = other + 1; sink(renamed, renamed);"
    ));
    for changed in [
        "let renamed = other - 1; sink(renamed, renamed);",
        "let renamed = other + 1; sink(renamed, other);",
    ] {
        let changed_source =
            format!("#![no_implicit_prelude]\nfn changed(other: i32) {{ {changed} }}");
        let analysis = collect(
            collect(Analysis::new(10_000_000, 1_000_000), left),
            &changed_source,
        );
        assert!(!equal(
            &analysis,
            "let café = input + 1; sink(cafe\u{301}, café);",
            changed
        ));
    }
    let compact = "#![no_implicit_prelude]\nfn one(a: &i32) { let x = &&*a; }";
    let spaced = "#![no_implicit_prelude]\nfn two(b: &i32) { let y = & &*b; }";
    let analysis = collect(
        collect(Analysis::new(10_000_000, 1_000_000), compact),
        spaced,
    );
    assert!(!equal(&analysis, "let x = &&*a;", "let y = & &*b;"));

    let measured = collect(Analysis::new(10_000_000, 1_000_000), left);
    let (work, bytes) = measured.meter().usage();
    assert!(try_collect(Analysis::new(work, bytes), left).is_ok());
    assert!(matches!(
        try_collect(Analysis::new(work - 1, bytes), left),
        Err(crate::embedded::collect::Failure {
            cause: Refusal::Work { .. },
            ..
        })
    ));
    assert!(matches!(
        try_collect(Analysis::new(work, bytes - 1), left),
        Err(crate::embedded::collect::Failure {
            cause: Refusal::Memory { .. },
            ..
        })
    ));
}

#[test]
fn syntax_admission_excludes_opaque_interiors_cross_block_sequences_and_partial_utf8() {
    let source = "#![no_implicit_prelude]\nfn helper(input: i32) { let café = input; { sink(café); } let opaque = wrap!(café);\n/// Local documentation.\nfn local() {}\nsink(café); }";
    let analysis = collect(Analysis::new(10_000_000, 1_000_000), source);
    let function = &analysis.functions()[0];
    let tokens = &function.tokens;
    let admit =
        |candidate| regions::admit(source, &function.facts, tokens, candidate, analysis.meter());
    assert!(admit(span(source, "sink(café);")).unwrap().is_some());
    assert!(admit(function.facts.body).unwrap().is_none());
    assert!(admit(span(source, "fn local() {}")).unwrap().is_none());
    let after_item = source.rfind("sink(café);").unwrap();
    assert!(
        admit(Span {
            start: after_item,
            end: after_item + "sink(café);".len()
        })
        .unwrap()
        .is_some()
    );
    assert!(
        admit(span(source, "let opaque = wrap!(café);"))
            .unwrap()
            .is_none()
    );
    assert!(
        admit(span(source, "let café = input; { sink(café);"))
            .unwrap()
            .is_none()
    );
    assert!(admit(span(source, "input; { sink")).unwrap().is_none());
    let inside_utf8 = source.find('é').unwrap() + 1;
    assert!(matches!(
        admit(Span {
            start: inside_utf8,
            end: inside_utf8 + 1
        }),
        Err(Refusal::Invariant(_))
    ));

    // A lowered documentation attribute adds one native group beyond the
    // actual source delimiters, including at the admitted source-depth limit.
    let depth = raf_source::Budget::Delimiters.limit();
    let source = format!(
        "#![no_implicit_prelude]\nfn helper(input:i32) {{ {}\n/// Deep local documentation.\nconst LOCAL:i32=1; input + 1; {} }}",
        "{".repeat(depth - 1),
        "}".repeat(depth - 1)
    );
    let analysis = collect(Analysis::new(100_000_000, 10_000_000), &source);
    let function = &analysis.functions()[0];
    assert!(
        regions::admit(
            &source,
            &function.facts,
            &function.tokens,
            span(&source, "input + 1;"),
            analysis.meter()
        )
        .unwrap()
        .is_some()
    );
}

#[test]
fn control_markers_keep_their_original_anchors_beside_adjacent_closure_and_return_regions() {
    let source =
        "#![no_implicit_prelude]\nfn helper(input: i32) { let f = ||{return input;}; f() }";
    let analysis = collect(Analysis::new(10_000_000, 1_000_000), source);
    let function = &analysis.functions()[0];
    let tokens = &function.tokens;
    for snippet in [
        "||{return input;}",
        "{return input;}",
        "return input",
        "return input;",
    ] {
        let admitted = regions::admit(
            source,
            &function.facts,
            tokens,
            span(source, snippet),
            analysis.meter(),
        )
        .unwrap()
        .unwrap();
        let atoms = &tokens.atoms[admitted.atoms];
        assert_eq!(
            atoms
                .iter()
                .filter(|a| a.occurrence.is_some_and(|o| o.fixed == Some("return")))
                .count(),
            1
        );
        assert_eq!(
            atoms
                .iter()
                .filter(|a| a.occurrence.is_some_and(|o| o.fixed == Some("closure")))
                .count(),
            usize::from(snippet.starts_with("||"))
        );
    }
}

#[test]
fn missing_original_tokens_and_overlapping_or_partial_binding_spans_never_become_witnesses() {
    let source =
        "#![no_implicit_prelude]\nfn helper(input: i32) { let local = input; sink(local); }";
    let path = raf_source::RelPath::new("invalid-spans.rs").unwrap();
    raf_source::parse_source(&path, source, |syntax| {
        let syn::Item::Fn(function) = &syntax.items[0] else {
            panic!("fixture function")
        };
        let meter = Meter::new(1_000_000, 1_000_000);
        let coordinates = Coordinates::new(source, syntax.shebang.as_deref(), &meter).unwrap();
        let mut scopes = crate::embedded::scopes::Scopes::new(source, &meter, &coordinates);
        scopes
            .enter_module(&syntax.items, &syntax.attrs, true)
            .unwrap();
        let mut facts = crate::embedded::collect::function(
            source,
            &function.sig,
            &function.block,
            &mut scopes,
            &meter,
        )
        .unwrap();
        assert!(join(source, &facts, source.parse().unwrap(), &meter).is_ok());
        assert!(matches!(
            join(source, &facts, TokenStream::new(), &meter),
            Err(Refusal::Invariant(_))
        ));
        let annotation = facts.annotations[0];
        meter
            .push_retained(&mut facts.annotations, annotation)
            .unwrap();
        assert!(matches!(
            join(source, &facts, source.parse().unwrap(), &meter),
            Err(Refusal::Invariant(_))
        ));
        facts.annotations.pop();
        facts.annotations[0].span.start += 1;
        facts.annotations[0].anchor = facts.annotations[0].span;
        assert!(matches!(
            join(source, &facts, source.parse().unwrap(), &meter),
            Err(Refusal::Invariant(_))
        ));
        facts.annotations[0] = annotation;
        assert!(join(source, &facts, source.parse().unwrap(), &meter).is_ok());
    })
    .unwrap();
}
