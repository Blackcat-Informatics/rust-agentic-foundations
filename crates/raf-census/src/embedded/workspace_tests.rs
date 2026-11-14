// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Controlled workspace source is data; none of its programs are executed.

use super::*;
use crate::embedded::collect::Span;
use crate::embedded::{proof, regions};
use raf_source::MemoryTree;

const MANIFEST: &str = "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";

fn region<'a>(function: &'a Function<'a>, snippet: &str, meter: &Meter) -> proof::Region<'a> {
    let start = function.source.find(snippet).unwrap();
    let admitted = regions::admit(
        function.source,
        &function.facts,
        &function.tokens,
        Span {
            start,
            end: start + snippet.len(),
        },
        meter,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        &function.source[admitted.bytes.start..admitted.bytes.end],
        snippet
    );
    proof::Region {
        bytes: admitted.bytes.start..admitted.bytes.end,
        atoms: &function.tokens.atoms[admitted.atoms],
        declarations: &function.facts.declarations,
    }
}

#[test]
fn actual_workspace_collection_keeps_every_reach_and_verifies_checked_embedded_regions() {
    let shared = "fn small(input: i32) { let x = input + 1; sink(x, x); }\nfn large(other: i32) { let earlier = 9; log(earlier); let renamed = other + 1; sink(renamed, renamed); tail(); }";
    let tree = MemoryTree::from([
        ("Cargo.toml", MANIFEST),
        (
            "src/lib.rs",
            "#![no_implicit_prelude]\n#[path = \"shared.rs\"] pub mod library;",
        ),
        (
            "src/main.rs",
            "#![no_implicit_prelude]\n#[path = \"shared.rs\"] mod binary; fn main() {}",
        ),
        ("src/shared.rs", shared),
    ]);
    let workspace = Workspace::load(&tree, &["."]).unwrap();
    let analysis = collect(&workspace, 100_000_000, 10_000_000).unwrap();
    assert_eq!(analysis.functions().len(), 3);
    let functions = analysis
        .functions()
        .iter()
        .filter(|function| function.origin.source().unwrap().0.path.as_str() == "src/shared.rs")
        .collect::<Vec<_>>();
    assert_eq!(functions.len(), 2);
    for function in &functions {
        let (file, owners) = function.origin.source().unwrap();
        assert!(std::ptr::eq(file, workspace.file(&file.path).unwrap()));
        assert!(std::ptr::eq(function.source, file.text.as_str()));
        assert_eq!(owners.len(), 2);
        assert_ne!(owners[0].target, owners[1].target);
        for owner in owners {
            assert!(std::ptr::eq(*owner, workspace.item(owner.id).unwrap()));
            assert_eq!(
                owner.body.as_ref().unwrap().start_byte,
                function.facts.body.start
            );
            assert_eq!(
                owner.body.as_ref().unwrap().end_byte,
                function.facts.body.end
            );
        }
    }
    let (work, bytes) = analysis.meter().usage();
    assert!(collect(&workspace, work, bytes).is_ok());
    assert!(matches!(
        collect(&workspace, work - 1, bytes),
        Err(SourceFailure::Located {
            cause: Failure {
                cause: Refusal::Work { .. },
                ..
            },
            ..
        })
    ));
    assert!(matches!(
        collect(&workspace, work, bytes - 1),
        Err(SourceFailure::Located {
            cause: Failure {
                cause: Refusal::Memory { .. },
                ..
            },
            ..
        })
    ));
    let small = region(
        functions[0],
        "let x = input + 1; sink(x, x);",
        analysis.meter(),
    );
    let large = region(
        functions[1],
        "let renamed = other + 1; sink(renamed, renamed);",
        analysis.meter(),
    );
    assert!(proof::same(&small, &large, analysis.meter()).unwrap());

    assert!(matches!(
        regions::admit(
            functions[0].source,
            &functions[0].facts,
            &functions[0].tokens,
            Span { start: 0, end: 0 },
            analysis.meter(),
        ),
        Err(Refusal::Invariant(_))
    ));
    crate::embedded::coordinates::fixture::check();
    native_spans::fixture::check();

    verify_lexical_provenance_and_original_prefix_spans();
    verify_attributed_range_extent();
}

fn verify_attributed_range_extent() {
    for (expression, semi) in [
        ("0..1", true),
        ("#[allow(unused)] 0..1", true),
        ("#[allow(unused)] ..=1", false),
        ("#[allow(unused)] 0..", false),
        ("|| { #[allow(unused)] 0..1 }", true),
        ("[0; { #[allow(unused)] 0..1 }]", true),
        ("0 as builtin#opaque()", true),
    ] {
        let ending = if semi { ";" } else { "" };
        let source = format!("#![no_implicit_prelude]\nfn helper() {{ {expression}{ending} }}");
        let tree = MemoryTree::from([("Cargo.toml", MANIFEST), ("src/lib.rs", source.as_str())]);
        let workspace = Workspace::load(&tree, &["."]).unwrap();
        let analysis = collect(&workspace, 100_000_000, 10_000_000).unwrap();
        let facts = &analysis.functions()[0].facts;
        assert!(
            facts
                .expressions
                .iter()
                .any(|span| { source.get(span.start..span.end) == Some(expression) }),
            "expression extent must cover original source: {expression}"
        );
        let statement = format!("{expression}{ending}");
        assert!(facts.statements.iter().any(|statement_span| {
            let span = statement_span.span;
            source.get(span.start..span.end) == Some(statement.as_str())
        }));
        let entry = if expression.starts_with("||") {
            Some(("closure", expression))
        } else if expression.starts_with("[0;") {
            Some(("repeat-length", "{ #[allow(unused)] 0..1 }"))
        } else {
            None
        };
        if let Some((opcode, expected)) = entry {
            assert!(facts.annotations.iter().any(|annotation| {
                annotation.marker
                    && annotation.occurrence.fixed == Some(opcode)
                    && source.get(annotation.anchor.start..annotation.anchor.end) == Some(expected)
            }));
        }
    }
}

fn verify_lexical_provenance_and_original_prefix_spans() {
    // Report provenance follows each actual lexical module, including methods
    // whose enclosing generic-value frame is above the module frame.
    let source = "#![no_std]\nfn root() {} mod inner { #![no_implicit_prelude] fn child() {} struct S; impl S { fn method(&self) {} } trait T { fn default() {} } }";
    let tree = MemoryTree::from([("Cargo.toml", MANIFEST), ("src/lib.rs", source)]);
    let workspace = Workspace::load(&tree, &["."]).unwrap();
    let analysis = collect(&workspace, 100_000_000, 10_000_000).unwrap();
    assert_eq!(analysis.functions().len(), 4);
    for function in analysis.functions() {
        let Origin::Source {
            workspace: actual,
            owners,
            profile,
            ..
        } = &function.origin
        else {
            panic!("workspace collection must retain actual source provenance")
        };
        assert!(std::ptr::eq(*actual, &raw const workspace));
        assert_eq!(profile.edition(), 2024);
        for owner in owners {
            assert_eq!(profile.flags(), (owner.name.as_str() != "root", true));
            assert!(std::ptr::eq(*owner, actual.item(owner.id).unwrap()));
            assert_eq!(actual.target_edition(owner.target), Some(profile.edition()));
        }
    }
    let mut refused = Vec::new();
    for prefix in [
        "\u{feff}",
        "#!fixture-data\n",
        "\u{feff}#! fixture données\r\n",
    ] {
        let source = format!(
            "{prefix}#![no_implicit_prelude]\nfn helper(input:i32) {{ let café=input+1; sink(café); }}"
        );
        let tree = MemoryTree::from([("Cargo.toml", MANIFEST), ("src/lib.rs", source.as_str())]);
        let workspace = Workspace::load(&tree, &["."]).unwrap();
        match collect(&workspace, 100_000_000, 10_000_000) {
            Ok(analysis) => {
                let function = &analysis.functions()[0];
                assert_eq!(
                    &source[function.facts.body.start..function.facts.body.end],
                    "{ let café=input+1; sink(café); }"
                );
                for (atom, span) in function.tokens.atoms.iter().zip(&function.tokens.spans) {
                    if !atom.spelling.is_empty() {
                        assert_eq!(source.get(span.start..span.end), Some(atom.spelling));
                    }
                }
            }
            Err(error) => refused.push(format!("{prefix:?}: {error}")),
        }
    }
    assert!(
        refused.is_empty(),
        "valid original source prefixes refused: {refused:?}"
    );
}

#[test]
fn actual_impl_and_trait_const_generics_remain_fixed_in_method_bodies() {
    let source = "#![no_implicit_prelude]\nstruct S<const N: usize> { values: [u8; N] }\nimpl<const N: usize> S<N> { fn helper(&self, input: usize) { let N = input; sink(self, N); } }\ntrait T<const M: usize> { fn default(&mut self, input: usize) { let M = input; sink(self, M); } }";
    let tree = MemoryTree::from([("Cargo.toml", MANIFEST), ("src/lib.rs", source)]);
    let workspace = Workspace::load(&tree, &["."]).unwrap();
    let analysis = collect(&workspace, 100_000_000, 10_000_000).unwrap();
    assert_eq!(analysis.functions().len(), 2);
    for (function, (name, kind)) in analysis.functions().iter().zip([
        ("N", SymbolKind::InherentMethod),
        ("M", SymbolKind::TraitDefaultMethod),
    ]) {
        let (_, owners) = function.origin.source().unwrap();
        assert!(owners.iter().all(|owner| owner.kind == kind));
        let fixed = function
            .tokens
            .atoms
            .iter()
            .filter(|atom| atom.spelling == name)
            .collect::<Vec<_>>();
        assert_eq!(fixed.len(), 2);
        assert!(fixed.iter().all(|atom| atom.occurrence.is_none()));
        let receiver = function
            .tokens
            .atoms
            .iter()
            .find(|atom| atom.spelling == "self")
            .unwrap()
            .occurrence
            .unwrap();
        let declaration = function.facts.declarations[receiver.binding];
        assert_eq!(declaration.family, proof::Family::Receiver);
        assert!(declaration.byte < function.facts.body.start);
    }
}

#[test]
fn empty_body_attributes_and_pruned_original_functions_cannot_become_accepted_owners() {
    for source in [
        "fn allowed<'a, T>() {}",
        "#[allow(unused)] fn allowed<#[allow(unused)] T>() {}",
        "async fn empty() {}",
        "const fn empty() {}",
    ] {
        let tree = MemoryTree::from([("Cargo.toml", MANIFEST), ("src/lib.rs", source)]);
        let workspace = Workspace::load(&tree, &["."]).unwrap();
        assert_eq!(
            collect(&workspace, 100_000_000, 10_000_000)
                .unwrap()
                .functions()
                .len(),
            1
        );
    }
    for source in [
        "#[unknown] fn empty() {}",
        "struct S {} impl S { #[unknown] fn empty() {} }",
        "trait T { #[unknown] fn empty() {} }",
        "fn empty<#[unknown] T>() {}",
        "fn empty<#[unknown] 'a>() {}",
    ] {
        let tree = MemoryTree::from([("Cargo.toml", MANIFEST), ("src/lib.rs", source)]);
        let workspace = Workspace::load(&tree, &["."]).unwrap();
        assert!(matches!(
            collect(&workspace, 100_000_000, 10_000_000),
            Err(SourceFailure::Located {
                cause: Failure {
                    cause: Refusal::Unsupported(_),
                    ..
                },
                ..
            })
        ));
    }
    let tree = MemoryTree::from([
        ("Cargo.toml", MANIFEST),
        (
            "src/lib.rs",
            "#[cfg(feature = \"inactive\")] fn hidden() {}\nfn visible() {}",
        ),
    ]);
    let declared = raf_source::DeclaredConfiguration::new(
        raf_source::DeclaredCfg::complete("without-inactive-feature", &[]),
        &[],
        raf_source::TargetKind::Lib,
        "fixture",
    );
    let workspace = Workspace::load_configured_target(
        &tree,
        &raf_source::RelPath::new("Cargo.toml").unwrap(),
        declared,
        &raf_source::LoadOptions::default(),
    )
    .unwrap();
    assert!(!workspace.items().any(|item| item.name == "hidden"));
    assert!(workspace.files().next().unwrap().text.contains("fn hidden"));
    assert!(matches!(
        collect(&workspace, 100_000_000, 10_000_000),
        Err(SourceFailure::Located {
            cause: Failure {
                cause: Refusal::Unsupported(_),
                ..
            },
            ..
        })
    ));
}
