// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Source text is controlled fixture data; no consumer program is executed.

use super::*;
use raf_source::MemoryTree;

const MANIFEST: &str = "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";

fn source<'workspace>(workspace: &'workspace Workspace, path: &str) -> &'workspace SourceFile {
    workspace.file(&RelPath::new(path).unwrap()).unwrap()
}

#[test]
fn actual_inline_outline_and_literal_path_ancestors_preserve_disabled_prelude_bindings() {
    for (root, child_path, disabled) in [
        (
            "#![no_implicit_prelude]\nmod inline { pub mod child; }",
            "src/inline/child.rs",
            true,
        ),
        (
            "mod inline { #![no_implicit_prelude]\npub mod child; }",
            "src/inline/child.rs",
            true,
        ),
        (
            "#![no_implicit_prelude]\nmod r#type { pub mod child; }",
            "src/type/child.rs",
            true,
        ),
        (
            "#[no_implicit_prelude]\n#[path = \"other.rs\"] mod child;",
            "src/other.rs",
            true,
        ),
        (
            "mod values { pub const None: i32 = 0; }\nuse values::*;\nmod child;",
            "src/child.rs",
            false,
        ),
    ] {
        let child = "fn helper(input: i32) { let None = input; sink(None); }";
        let tree = MemoryTree::from([
            ("Cargo.toml", MANIFEST),
            ("src/lib.rs", root),
            (child_path, child),
        ]);
        let workspace = Workspace::load(&tree, &["."]).unwrap();
        let file = source(&workspace, child_path);
        let (analysis, profile) =
            for_file(Analysis::new(10_000_000, 1_000_000), &workspace, file).unwrap();
        assert_eq!(profile.flags(), (disabled, false));
        assert_eq!(profile.edition(), 2024);
        if child_path == "src/other.rs" {
            let (work, bytes) = analysis.meter().usage();
            assert!(for_file(Analysis::new(work, bytes), &workspace, file).is_ok());
            assert!(matches!(
                for_file(Analysis::new(work - 1, bytes), &workspace, file),
                Err(SourceFailure::Located {
                    cause: Failure {
                        cause: Refusal::Work { .. },
                        ..
                    },
                    ..
                })
            ));
            assert!(matches!(
                for_file(Analysis::new(work, bytes - 1), &workspace, file),
                Err(SourceFailure::Located {
                    cause: Failure {
                        cause: Refusal::Memory { .. },
                        ..
                    },
                    ..
                })
            ));
        }
        let (analysis, ()) = analysis
            .inspect_source_with_tokens(file, move |analysis, syntax, original| {
                let syn::Item::Fn(function) = &syntax.items[0] else {
                    panic!("fixture function")
                };
                analysis.with_function(
                    &file.text,
                    &function.sig,
                    &function.block,
                    (syntax, original),
                    Some(profile),
                    |scopes| scopes.enter_module(&syntax.items, &syntax.attrs, false),
                )?;
                Ok(())
            })
            .unwrap();
        let function = &analysis.functions()[0];
        let local = child.find("None =").unwrap();
        assert_eq!(
            function
                .facts
                .annotations
                .iter()
                .any(|annotation| annotation.span.start == local),
            disabled
        );
    }
}

#[test]
fn shared_source_reaches_require_independent_agreement_instead_of_pooling_disabled_flags() {
    for (second_flag, admitted) in [("", false), ("#[no_implicit_prelude]", true)] {
        let root = format!(
            "#[no_implicit_prelude]\n#[path = \"shared.rs\"] mod first;\n{second_flag}\n#[path = \"shared.rs\"] mod second;"
        );
        let tree = MemoryTree::from([
            ("Cargo.toml", MANIFEST),
            ("src/lib.rs", root.as_str()),
            ("src/shared.rs", "fn helper() {}"),
        ]);
        let workspace = Workspace::load(&tree, &["."]).unwrap();
        let result = for_file(
            Analysis::new(10_000_000, 1_000_000),
            &workspace,
            source(&workspace, "src/shared.rs"),
        );
        if admitted {
            assert_eq!(result.unwrap().1.flags(), (true, false));
        } else {
            assert!(matches!(
                result,
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
    let tree = MemoryTree::from([
        ("Cargo.toml", MANIFEST),
        (
            "src/lib.rs",
            "#![no_std]\n#[path = \"shared.rs\"] mod shared;",
        ),
        (
            "src/main.rs",
            "#[path = \"shared.rs\"] mod shared; fn main() {}",
        ),
        ("src/shared.rs", "fn helper() {}"),
    ]);
    let workspace = Workspace::load(&tree, &["."]).unwrap();
    assert!(matches!(
        for_file(
            Analysis::new(10_000_000, 1_000_000),
            &workspace,
            source(&workspace, "src/shared.rs")
        ),
        Err(SourceFailure::Located {
            cause: Failure {
                cause: Refusal::Unsupported(_),
                ..
            },
            ..
        })
    ));
}

#[test]
fn ancestor_expansion_conditionals_and_custom_preludes_refuse_beside_inert_documentation() {
    for (root, admitted) in [
        ("//! Crate documentation.\nmod child;", true),
        ("#![no_std]\nmod child;", true),
        ("#![no_implicit_prelude]\nmake_items!(); mod child;", false),
        ("#[unknown] struct Host; mod child;", false),
        ("#![cfg(feature = \"active\")]\nmod child;", false),
        ("#![no_core]\nmod child;", false),
        ("#![path = \"other.rs\"]\nmod child;", false),
        ("#[prelude_import] use core::prelude::*; mod child;", false),
    ] {
        let tree = MemoryTree::from([
            ("Cargo.toml", MANIFEST),
            ("src/lib.rs", root),
            ("src/child.rs", "fn helper() {}"),
        ]);
        let workspace = Workspace::load(&tree, &["."]).unwrap();
        let file = source(&workspace, "src/child.rs");
        let result = for_file(Analysis::new(10_000_000, 1_000_000), &workspace, file);
        if admitted {
            let (_, profile) = result.unwrap();
            assert_eq!(profile.flags(), (false, root.starts_with("#![no_std]")));
        } else {
            match result {
                Err(SourceFailure::Located {
                    file,
                    cause:
                        Failure {
                            cause: Refusal::Unsupported(_),
                            ..
                        },
                }) => assert_eq!(file.as_str(), "src/lib.rs"),
                _ => panic!("expected located ancestor refusal for {root}"),
            }
        }
    }
    for (root, child_path, child) in [
        ("#[no_std] mod child;", "src/child.rs", "fn helper() {}"),
        ("mod child;", "src/child.rs", "#![no_std]\nfn helper() {}"),
        (
            "mod inline { #![no_std]\npub mod child; }",
            "src/inline/child.rs",
            "fn helper() {}",
        ),
    ] {
        let tree = MemoryTree::from([
            ("Cargo.toml", MANIFEST),
            ("src/lib.rs", root),
            (child_path, child),
        ]);
        let workspace = Workspace::load(&tree, &["."]).unwrap();
        assert!(matches!(
            for_file(
                Analysis::new(10_000_000, 1_000_000),
                &workspace,
                source(&workspace, child_path)
            ),
            Err(SourceFailure::Located {
                cause: Failure {
                    cause: Refusal::Unsupported(
                        "no_std is admitted only as an inner crate-root attribute"
                    ),
                    ..
                },
                ..
            })
        ));
    }
}
