// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Syntax and lexical control fixtures only. Neither fixture programs nor
//! their constants, futures, closures or macros are evaluated.

use super::analysis::{Analysis, SourceFailure};
use super::collect::{Facts, Failure};
use super::proof::{Event, Family};
use super::resource::Refusal;
use super::{pairing, policy, workspace};
use raf_source::{MemoryTree, Workspace};

fn load(source: &str, edition: u16) -> Workspace {
    let manifest =
        format!("[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"{edition}\"\n");
    Workspace::load(
        &MemoryTree::from([("Cargo.toml", manifest.as_str()), ("src/lib.rs", source)]),
        &["."],
    )
    .unwrap()
}

fn collect(workspace: &Workspace) -> Result<Analysis<'_>, SourceFailure<'_>> {
    workspace::collect(workspace, 100_000_000, 10_000_000)
}

#[test]
fn public_collection_refusals_escape_untrusted_paths_and_preserve_their_identity() {
    for name in ["ordinary.rs", "line\n\u{1b}[31m\u{202e}.rs"] {
        let module = format!("#[path = {name:?}] mod input;");
        let path = format!("src/{name}");
        let tree = MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname='fixture'\nversion='0.1.0'\nedition='2015'\n",
            ),
            ("src/lib.rs", module.as_str()),
            (path.as_str(), "async fn operation() { produce().await; }"),
        ]);
        let source = Workspace::load(&tree, &["."]).unwrap();
        let Err(error) = super::Run::analyze(&source, &super::Options::default()) else {
            panic!("the declared edition must refuse async syntax");
        };
        let super::Error::Collection { file, .. } = &error else {
            panic!("expected a located collection refusal: {error}");
        };
        assert_eq!(file.as_str(), path);
        let diagnostic = error.to_string();
        assert!(diagnostic.starts_with(&path.escape_debug().to_string()));
        assert!(!diagnostic.contains(['\n', '\u{1b}', '\u{202e}']));
    }
}

fn target(facts: &Facts<'_>, source: &str, snippet: &str, opcode: &str) -> usize {
    let token = if opcode == "try" { "?" } else { opcode };
    let at = source.find(snippet).unwrap() + snippet.find(token).unwrap();
    facts
        .annotations
        .iter()
        .find(|annotation| {
            annotation.marker
                && annotation.anchor.start == at
                && annotation.occurrence.event == Event::Use
                && annotation.occurrence.fixed == Some(opcode)
        })
        .expect("original control-use marker")
        .occurrence
        .binding
}

#[test]
fn async_function_block_and_closure_transfers_keep_their_own_future_targets() {
    let source = "#![no_implicit_prelude]\nasync fn helper(input:i32) { let outer = produce(input).await?; let future = async move { let inside = produce(input + 1).await?; return inside; }; let closure = async |x| { let y = produce(x).await?; return y + input; }; let plain = || { return input; }; return outer; }";
    let workspace = load(source, 2024);
    let analysis = collect(&workspace).unwrap();
    let facts = &analysis.functions()[0].facts;
    for (snippet, family) in [
        ("produce(input).await?", Family::AsyncFunction),
        ("produce(input + 1).await?", Family::AsyncBlock),
        ("produce(x).await?", Family::AsyncClosure),
    ] {
        let awaited = target(facts, source, snippet, "await");
        let propagated = target(facts, source, snippet, "try");
        assert_eq!(awaited, propagated);
        assert_eq!(facts.declarations[awaited].family, family);
    }
    for (snippet, family) in [
        ("return inside", Family::AsyncBlock),
        ("return y + input", Family::AsyncClosure),
        ("return input", Family::Closure),
        ("return outer", Family::AsyncFunction),
    ] {
        assert_eq!(
            facts.declarations[target(facts, source, snippet, "return")].family,
            family
        );
    }
    let workspace = load(source, 2015);
    assert!(matches!(
        collect(&workspace),
        Err(SourceFailure::Located {
            cause: Failure {
                cause: Refusal::Unsupported(_),
                ..
            },
            ..
        })
    ));

    let core = "let x = input + 1; let y = x * 3; sink(y, x, input, 7);";
    let other = core.replace("input", "other");
    let source = format!(
        "#![no_implicit_prelude]\nasync fn one(input:i32) {{ {core} return input; }} fn two(other:i32) {{ {other} return other; }}"
    );
    let workspace = load(&source, 2024);
    let matched = pairing::find(collect(&workspace).unwrap(), policy::Limits::default()).unwrap();
    assert!(!matched.witnesses.is_empty());
    for witness in &matched.witnesses {
        assert!(!source[witness.left.bytes.start..witness.left.bytes.end].contains("return"));
        assert!(!source[witness.right.bytes.start..witness.right.bytes.end].contains("return"));
    }
}

#[test]
fn try_scopes_intercept_question_marks_without_intercepting_return_or_loop_labels() {
    let source = "#![no_implicit_prelude]\n#![feature(try_blocks)]\nfn helper(input:i32) { let result = try { let x = read(input)?; let nested = try { read(x)? }; if flag(x) { return x; } x }; 'outer: loop { let result = try { break 'outer; }; } }";
    let workspace = load(source, 2024);
    let analysis = collect(&workspace).unwrap();
    let facts = &analysis.functions()[0].facts;
    let first = target(facts, source, "read(input)?", "try");
    let nested = target(facts, source, "read(x)?", "try");
    assert_ne!(first, nested);
    assert_eq!(facts.declarations[first].family, Family::Try);
    assert_eq!(facts.declarations[nested].family, Family::Try);
    assert_eq!(
        facts.declarations[target(facts, source, "return x", "return")].family,
        Family::Function
    );
    assert_eq!(
        facts.declarations[target(facts, source, "break 'outer", "break")].family,
        Family::Loop
    );
    for source in [
        "#![feature(unknown_language)]\nfn helper() {}",
        "#![feature = \"try_blocks\"]\nfn helper() {}",
        "#![feature[try_blocks]]\nfn helper() {}",
        "mod child { #![feature(try_blocks)] fn helper() {} }",
    ] {
        let workspace = load(source, 2024);
        assert!(matches!(
            collect(&workspace),
            Err(SourceFailure::Located {
                cause: Failure {
                    cause: Refusal::Unsupported(_),
                    ..
                },
                ..
            })
        ));
    }
}

#[test]
fn const_scopes_keep_generic_items_inner_bindings_and_const_function_returns() {
    let source = "#![no_implicit_prelude]\nconst fn helper<const N:usize>(input:usize) { let outer = input; let result = const { let outer = N + 1; let read_local = || outer; let noncapturing = || { return 1; }; outer }; let repeated = [input; N]; let typed: [u8; N + 1] = [0; N + 1]; call::<{ N + 2 }>(); return outer + result; }";
    let workspace = load(source, 2024);
    let analysis = collect(&workspace).unwrap();
    let facts = &analysis.functions()[0].facts;
    let original = source.find("outer = input").unwrap();
    let inner = source.find("outer = N").unwrap();
    let binding = |at| {
        facts
            .annotations
            .iter()
            .find(|annotation| !annotation.marker && annotation.span.start == at)
            .unwrap()
            .occurrence
            .binding
    };
    assert_ne!(binding(original), binding(inner));
    assert_eq!(
        binding(source.find("|| outer").unwrap() + 3),
        binding(inner)
    );
    assert_eq!(
        binding(source.find("return outer").unwrap() + 7),
        binding(original)
    );
    assert_eq!(
        facts.declarations[target(facts, source, "return outer", "return")].family,
        Family::ConstFunction
    );
    assert_eq!(
        facts.declarations[target(facts, source, "return 1", "return")].family,
        Family::Closure
    );
    assert!(
        facts
            .annotations
            .iter()
            .any(|annotation| annotation.occurrence.fixed == Some("array-length"))
    );
    assert!(
        facts
            .annotations
            .iter()
            .any(|annotation| annotation.occurrence.fixed == Some("repeat-length"))
    );
    assert!(
        facts
            .annotations
            .iter()
            .any(|annotation| annotation.occurrence.fixed == Some("const-argument"))
    );
    // Signature patterns still supply parameter declarations, but their fixed
    // const/type syntax must never create body-owned control annotations.
    for signature in [
        "fn helper<const N:usize>(S::<{ N }>(input): S<N>) { sink(input); }",
        "fn helper<const N:usize>(S::<{ N }> { value: input }: S<N>) { sink(input); }",
        "fn helper(0..=9: i32) {}",
        "fn helper(const { 1 }: i32) {}",
    ] {
        let source = format!("#![no_implicit_prelude]\n{signature}");
        let workspace = load(&source, 2024);
        let analysis = collect(&workspace).unwrap();
        let function = &analysis.functions()[0];
        assert!(function.facts.annotations.iter().all(|annotation| {
            annotation.anchor.start >= function.facts.body.start
                && annotation.anchor.end <= function.facts.body.end
        }));
        if source.contains("sink(input)") {
            let used = source.find("sink(input)").unwrap() + 5;
            let annotation = function
                .facts
                .annotations
                .iter()
                .find(|annotation| !annotation.marker && annotation.span.start == used)
                .unwrap();
            assert!(
                function.facts.declarations[annotation.occurrence.binding].byte
                    < function.facts.body.start
            );
        }
    }
}

#[test]
fn constant_capture_and_control_fences_refuse_beside_valid_lexical_neighbours() {
    for (invalid, valid) in [
        (
            "fn helper(input:i32) { const { input }; }",
            "fn helper(input:i32) { const { let input = 3; input }; }",
        ),
        (
            "fn helper(input:i32) { const { return input; }; }",
            "fn helper(input:i32) { { return input; } }",
        ),
        (
            "fn helper() { 'outer: loop { let f = async { break 'outer; }; } }",
            "fn helper() { let f = async { 'inner: loop { break 'inner; } }; }",
        ),
        (
            "async fn helper() { let f = || { produce().await; }; }",
            "async fn helper() { let f = async || { produce().await; }; }",
        ),
        (
            "fn helper(input:usize) { let a = [0; input]; }",
            "fn helper<const N:usize>() { let a = [0; N]; }",
        ),
        (
            "fn helper(input:usize) { let a: [u8; input] = []; }",
            "fn helper<const N:usize>() { let a: [u8; N] = [0; N]; }",
        ),
        (
            "fn helper(input:usize) { call::<{ input + 1 }>(); }",
            "fn helper() { call::<{ 4 + 1 }>(); }",
        ),
        (
            "fn helper(input:i32) { match input { input..=9 => 1, _ => 2 }; }",
            "fn helper(input:i32) { match input { 0..=9 => 1, _ => 2 }; }",
        ),
        (
            "fn helper(input:i32) { let f = const || input; }",
            "fn helper(input:i32) { let f = || input; }",
        ),
        (
            "fn helper(input:i32) { const { read()? }; }",
            "fn helper(input:i32) { const { let result = try { read()? }; 0 }; }",
        ),
        (
            "async fn helper() { const { produce().await }; }",
            "async fn helper() { let future = async { produce().await }; }",
        ),
    ] {
        for (source, admitted) in [(invalid, false), (valid, true)] {
            let source = format!("#![no_implicit_prelude]\n#![feature(try_blocks)]\n{source}");
            let workspace = load(&source, 2024);
            let result = collect(&workspace);
            if admitted {
                assert!(result.is_ok(), "{source}: {}", result.err().unwrap());
            } else {
                assert!(
                    matches!(
                        result,
                        Err(SourceFailure::Located {
                            cause: Failure {
                                cause: Refusal::Unsupported(_),
                                ..
                            },
                            ..
                        })
                    ),
                    "{source}"
                );
            }
        }
    }
}
