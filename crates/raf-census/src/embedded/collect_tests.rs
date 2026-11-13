// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Read Rust fixture text through Source; no fixture program is executed.

use super::*;

fn inspect<'source>(
    source: &'source str,
    check: impl FnOnce(Result<Facts<'source>, Failure>) + Send,
) {
    let path = raf_source::RelPath::new("collector.rs").unwrap();
    raf_source::parse_source(&path, source, |syntax| {
        let meter = Meter::new(1_000_000, 1_000_000);
        let coordinates = crate::embedded::coordinates::Coordinates::new(
            source,
            syntax.shebang.as_deref(),
            &meter,
        )
        .unwrap();
        let mut scopes = Scopes::new(source, &meter, &coordinates);
        scopes
            .enter_module(&syntax.items, &syntax.attrs, true)
            .unwrap();
        let (signature, body) = syntax
            .items
            .iter()
            .find_map(|item| match item {
                syn::Item::Fn(item) if item.sig.ident == "helper" => {
                    Some((&item.sig, item.block.as_ref()))
                }
                syn::Item::Impl(item) => item.items.iter().find_map(|item| match item {
                    syn::ImplItem::Fn(item) if item.sig.ident == "helper" => {
                        Some((&item.sig, &item.block))
                    }
                    _ => None,
                }),
                _ => None,
            })
            .expect("fixture helper");
        check(function(source, signature, body, &mut scopes, &meter));
    })
    .unwrap();
}

fn at<'source>(facts: &Facts<'source>, byte: usize) -> Occurrence<'source> {
    facts
        .annotations
        .iter()
        .find(|annotation| !annotation.marker && annotation.span.start == byte)
        .expect("original source occurrence")
        .occurrence
}

#[test]
fn initializers_read_the_previous_binding_and_inner_shadowing_ends_with_its_block() {
    let source = "// café shifts UTF-8 offsets.\n#![no_implicit_prelude]\n\
        fn helper(input: i32) { let x = input; { let x = x + 1; sink(x); } sink(x); }";
    inspect(source, |result| {
        let facts = result.unwrap();
        let outer_at = source.find("x = input").unwrap();
        let inner_at = source.find("x = x + 1").unwrap();
        let outer = at(&facts, outer_at).binding;
        let inner = at(&facts, inner_at).binding;
        assert_ne!(outer, inner);
        assert_eq!(at(&facts, inner_at + 4).binding, outer);
        assert_eq!(
            at(&facts, source.find("sink(x)").unwrap() + 5).binding,
            inner
        );
        assert_eq!(
            at(&facts, source.rfind("sink(x)").unwrap() + 5).binding,
            outer
        );
        assert_eq!(facts.declarations[outer].byte, outer_at);
        assert_eq!(facts.declarations[inner].byte, inner_at);
        assert!(facts.statements.iter().any(|statement| {
            source.get(statement.span.start..statement.span.end) == Some("let x = x + 1;")
        }));
        assert!(
            facts
                .statements
                .iter()
                .any(|statement| statement.block != facts.statements[0].block)
        );
    });
}

#[test]
fn let_else_divergence_precedes_activation_and_closure_parameters_do_not_replace_captures() {
    let source = "#![no_implicit_prelude]\nenum E { Some(i32), None }\n\
        fn helper(input: E) -> i32 { let x = 1; \
        let E::Some(x) = input else { return x; }; \
        let captured = x; let f = |x| x + captured; f(x) }";
    inspect(source, |result| {
        let facts = result.unwrap();
        let old = at(&facts, source.find("x = 1").unwrap()).binding;
        let new = at(&facts, source.find("Some(x)").unwrap() + 5).binding;
        assert_ne!(old, new);
        assert_eq!(
            at(&facts, source.find("return x").unwrap() + 7).binding,
            old
        );
        assert_eq!(at(&facts, source.rfind("f(x)").unwrap() + 2).binding, new);
        let parameter = at(&facts, source.find("|x|").unwrap() + 1).binding;
        assert_ne!(parameter, new);
        assert_eq!(
            at(&facts, source.find("|x| x").unwrap() + 4).binding,
            parameter
        );
        let capture = at(&facts, source.find("captured =").unwrap()).binding;
        assert_eq!(
            at(&facts, source.find("+ captured").unwrap() + 2).binding,
            capture
        );
        assert!(facts.declarations[capture].byte < source.find("|x|").unwrap());
    });
}

#[test]
fn or_patterns_share_declarations_with_their_guard_and_refuse_different_alternative_names() {
    let source = "#![no_implicit_prelude]\nfn helper(input: (i32, i32)) -> i32 { \
        match input { (x, 0) | (0, x) if x > 0 => x, _ => 0 } }";
    inspect(source, |result| {
        let facts = result.unwrap();
        let first = at(&facts, source.find("(x, 0)").unwrap() + 1).binding;
        assert_eq!(
            at(&facts, source.find("(0, x)").unwrap() + 4).binding,
            first
        );
        assert_eq!(at(&facts, source.find("if x").unwrap() + 3).binding, first);
        assert_eq!(at(&facts, source.find("=> x").unwrap() + 3).binding, first);
    });
    let different = "#![no_implicit_prelude]\nfn helper(input: (i32, i32)) -> i32 { \
        match input { (x, 0) | (0, other) => x, _ => 0 } }";
    inspect(different, |result| {
        assert!(matches!(
            result,
            Err(Failure {
                cause: Refusal::Unsupported(_),
                ..
            })
        ));
    });
}

#[test]
fn return_targets_and_shadowed_block_labels_keep_their_actual_control_destinations() {
    let source = "#![no_implicit_prelude]\nfn helper(input: i32) -> i32 { \
        let f = || { return input; }; \
        'a: loop { 'a: { break 'a; } continue 'a; } return input; }";
    inspect(source, |result| {
        let facts = result.unwrap();
        let targets = |opcode| {
            facts
                .annotations
                .iter()
                .filter(move |annotation| {
                    annotation.marker && annotation.occurrence.fixed == Some(opcode)
                })
                .map(|annotation| facts.declarations[annotation.occurrence.binding].family)
                .collect::<Vec<_>>()
        };
        assert_eq!(targets("return"), [Family::Closure, Family::Function]);
        assert_eq!(targets("break"), [Family::Label]);
        assert_eq!(targets("continue"), [Family::Loop]);
        let break_label = at(&facts, source.find("break 'a").unwrap() + 7).binding;
        let continue_label = at(&facts, source.find("continue 'a").unwrap() + 10).binding;
        assert_ne!(break_label, continue_label);
    });
}

#[test]
fn shorthand_fields_keep_fixed_names_and_unadmitted_attributes_or_boundaries_refuse() {
    let source = "#![no_implicit_prelude]\nstruct S { value: i32 } \
        fn helper(input: S) { let S { value } = input; let other = S { value }; }";
    inspect(source, |result| {
        let facts = result.unwrap();
        let declared = at(&facts, source.find("value } =").unwrap());
        let used = at(&facts, source.rfind("value }").unwrap());
        assert_eq!(declared.binding, used.binding);
        assert_eq!(declared.fixed, Some("value"));
        assert_eq!(used.fixed, Some("value"));
    });
    for source in [
        "#![no_implicit_prelude]\nfn helper(input: i32) { #[cfg(any())] { input; } }",
        "#![no_implicit_prelude]\nfn helper(input: i32) { async { input }; }",
    ] {
        inspect(source, |result| {
            assert!(matches!(
                result,
                Err(Failure {
                    cause: Refusal::Unsupported(_),
                    ..
                })
            ));
        });
    }
    inspect(
        "#![no_implicit_prelude]\nfn helper(input: i32) { #[allow(unused)] { input; } }",
        |result| {
            assert!(result.is_ok());
        },
    );
}
