// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Original declaration identity boundaries in the existing workspace fixture.

use super::*;
use crate::embedded::coordinates::Coordinates;

pub(in crate::embedded) fn without_work<T>(
    source: &str,
    operation: impl FnOnce(&Coordinates<'_, '_>) -> Result<T, Refusal>,
) {
    let required = {
        let meter = Meter::new(usize::MAX, usize::MAX);
        let _coordinates = Coordinates::new(source, None, &meter).unwrap();
        meter.usage().0
    };
    let meter = Meter::new(required, usize::MAX);
    let coordinates = Coordinates::new(source, None, &meter).unwrap();
    assert!(matches!(operation(&coordinates), Err(Refusal::Work { .. })));
}

pub(in crate::embedded) fn check() {
    pattern::fixture::check();
    ty::fixture::check();
    item_anchors();
    inner_anchors();
    for (source, expected) in [
        (
            "// prefix\n/// outer\npub(crate) unsafe extern \"C\" fn free<T>() -> T where T: Copy { #![allow(unused)] loop {} }\n",
            "/// outer\npub(crate) unsafe extern \"C\" fn free<T>() -> T where T: Copy { #![allow(unused)] loop {} }",
        ),
        ("const fn constant() {}", "const fn constant() {}"),
        ("async fn future() {}", "async fn future() {}"),
        ("extern \"C\" fn abi() {}", "extern \"C\" fn abi() {}"),
        (
            "impl T { default unsafe fn method() { #![allow(unused)] } }",
            "default unsafe fn method() { #![allow(unused)] }",
        ),
        (
            "trait T { /// outer\nunsafe fn method() { #![allow(unused)] } }",
            "/// outer\nunsafe fn method() { #![allow(unused)] }",
        ),
        ("trait T { fn declared(); }", "fn declared();"),
        (
            "// prefix\npub(crate) mod inline { #![allow(unused)] fn nested() {} }",
            "pub(crate) mod inline { #![allow(unused)] fn nested() {} }",
        ),
        (
            "// prefix\n#[path = \"data.rs\"] pub mod outline;",
            "#[path = \"data.rs\"] pub mod outline;",
        ),
    ] {
        let path = raf_source::RelPath::new("declarations.rs").unwrap();
        raf_source::parse_source(&path, source, |syntax| {
            let meter = Meter::new(usize::MAX, usize::MAX);
            let coordinates = Coordinates::new(source, None, &meter).unwrap();
            let ends = declaration(&syntax.items[0], &meter).unwrap();
            let range = coordinates.between(ends.0, ends.1).unwrap();
            let start = source.find(expected).unwrap();
            assert_eq!(range, start..start + expected.len(), "{source}");
            // The same original account admits endpoint selection before
            // any body-size-dependent operation. A refusal cannot turn into
            // synthetic call-site coordinates or a partial identity.
            assert!(matches!(
                declaration(&syntax.items[0], &Meter::new(0, usize::MAX)),
                Err(Refusal::Work { .. })
            ));
        })
        .unwrap();
    }
}

fn item_anchors() {
    // Original syntax only: these declarations need not type-check, and none
    // of their bodies, foreign declarations or macros are compiled or run.
    let declarations = [
        "const VALUE: i32 = 0;",
        "pub enum E {}",
        "extern crate core;",
        "async fn free() {}",
        "unsafe extern \"C\" { fn foreign(); }",
        "default unsafe impl T for S {}",
        "::path::m!();",
        "pub(crate) mod inline {}",
        "static mut VALUE: i32 = 0;",
        "struct S;",
        "struct Tuple<T>(T) where T: Copy;",
        "struct Named<T> where T: Copy { value: T }",
        "unsafe auto trait T {}",
        "trait Alias = Sized;",
        "type Alias = i32;",
        "union U { value: i32 }",
        "use core::mem;",
        "/// outer\nfn documented() { #![allow(unused)] }",
        "impl S { #![allow(unused)] }",
        "auto trait Automatic {}",
        "macro_rules! named { () => {} }",
    ];
    let source = format!("// prefix\n{}", declarations.join("\n"));
    let path = raf_source::RelPath::new("item-anchors.rs").unwrap();
    raf_source::parse_source(&path, &source, |syntax| {
        let meter = Meter::new(usize::MAX, usize::MAX);
        let coordinates = Coordinates::new(&source, None, &meter).unwrap();
        assert_eq!(syntax.items.len(), declarations.len());
        for (item, declaration) in syntax.items.iter().zip(declarations) {
            let first = item_start(item, &meter).unwrap().unwrap();
            assert_eq!(
                coordinates.range(first).unwrap().start,
                source.find(declaration).unwrap(),
                "{declaration}"
            );
            let (first, last) = statement::item_ends(item, &coordinates).unwrap().unwrap();
            let at = source.find(declaration).unwrap();
            assert_eq!(
                coordinates.between(first, last).unwrap(),
                at..at + declaration.len(),
                "{declaration}"
            );
            assert!(matches!(
                item_start(item, &Meter::new(0, usize::MAX)),
                Err(Refusal::Work { .. })
            ));
            without_work(&source, |context| statement::item_ends(item, context));
        }
    })
    .unwrap();
    // A parser-preserved unmodeled item must keep its existing native path;
    // direct endpoint selection must not turn it into an ordinary declaration.
    raf_source::parse_source(&path, "const trait T {}", |syntax| {
        let meter = Meter::new(usize::MAX, usize::MAX);
        assert!(matches!(&syntax.items[0], syn::Item::Verbatim(_)));
        assert!(item_start(&syntax.items[0], &meter).unwrap().is_none());
        let coordinates = Coordinates::new("const trait T {}", None, &meter).unwrap();
        let ends = statement::item_ends(&syntax.items[0], &coordinates)
            .unwrap()
            .unwrap();
        assert_eq!(coordinates.between(ends.0, ends.1).unwrap(), 0..16);
    })
    .unwrap();
}

fn inner_anchors() {
    let source = "fn owner<'a, T, const N: usize>() { #[allow(unused)] let named = S { #[allow(unused)] value: make(), 0: other(), shorthand }; let bare = make(); }";
    let path = raf_source::RelPath::new("inner-anchors.rs").unwrap();
    raf_source::parse_source(&path, source, |syntax| {
        let meter = Meter::new(usize::MAX, usize::MAX);
        let coordinates = Coordinates::new(source, None, &meter).unwrap();
        let syn::Item::Fn(function) = &syntax.items[0] else {
            panic!("fixture function")
        };
        for (parameter, expected) in function
            .sig
            .generics
            .params
            .iter()
            .zip(["'a", "T", "const N"])
        {
            let first = generic_start(parameter, &meter).unwrap();
            assert_eq!(
                coordinates.range(first).unwrap().start,
                source.find(expected).unwrap()
            );
            assert!(matches!(
                generic_start(parameter, &Meter::new(0, usize::MAX)),
                Err(Refusal::Work { .. })
            ));
        }
        assert_eq!(function.sig.generics.params.len(), 3);
        for (statement, expected) in function
            .block
            .stmts
            .iter()
            .zip(["#[allow(unused)] let", "let bare"])
        {
            let syn::Stmt::Local(local) = statement else {
                panic!("fixture local")
            };
            assert_eq!(
                coordinates
                    .range(local_start(local, &meter).unwrap())
                    .unwrap()
                    .start,
                source.find(expected).unwrap()
            );
            assert!(matches!(
                local_start(local, &Meter::new(0, usize::MAX)),
                Err(Refusal::Work { .. })
            ));
        }
        assert_eq!(function.block.stmts.len(), 2);
        let syn::Stmt::Local(local) = &function.block.stmts[0] else {
            panic!("fixture local")
        };
        let syn::Expr::Struct(value) = &*local.init.as_ref().unwrap().expr else {
            panic!("fixture struct value")
        };
        assert_eq!(value.fields.len(), 3);
        for (field, expected) in
            value
                .fields
                .iter()
                .zip(["#[allow(unused)] value", "0: other", "shorthand"])
        {
            assert_eq!(
                coordinates
                    .range(field_start(field, &meter).unwrap())
                    .unwrap()
                    .start,
                source.find(expected).unwrap()
            );
            assert!(matches!(
                field_start(field, &Meter::new(0, usize::MAX)),
                Err(Refusal::Work { .. })
            ));
        }
    })
    .unwrap();
}

fn declaration(item: &syn::Item, meter: &Meter) -> Result<Ends, Refusal> {
    match item {
        syn::Item::Fn(node) => function(node, meter),
        syn::Item::Mod(node) => module(node, meter),
        syn::Item::Impl(node) => {
            let syn::ImplItem::Fn(node) = &node.items[0] else {
                panic!("fixture impl method")
            };
            method(node, meter)
        }
        syn::Item::Trait(node) => {
            let syn::TraitItem::Fn(node) = &node.items[0] else {
                panic!("fixture trait method")
            };
            trait_function(node, meter)
        }
        _ => panic!("fixture declaration"),
    }
}
