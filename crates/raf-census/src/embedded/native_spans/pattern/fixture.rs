// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Original pattern and composite spans in controlled source data.

use super::*;
use crate::embedded::coordinates::Coordinates;
use crate::embedded::native_spans::fixture::without_work;
use crate::embedded::resource::Meter;

pub(in crate::embedded) fn check() {
    for spelling in [
        "const { 1 }",
        "bound if bound > 0",
        "ref mut binding @ Some(_)",
        "-3",
        "opaque!()",
        "| Some(x) | None",
        "(Some(x) | None)",
        "<S as Trait>::VALUE",
        "1..=3",
        "&mut value",
        "..",
        "[head, ..]",
        "S { field: value, .. }",
        "(head, tail)",
        "Some(value)",
        "_",
        "box value",
    ] {
        let source = format!("fn owner() {{ match input {{ {spelling} => (), }} }}");
        let path = raf_source::RelPath::new("pattern-endpoints.rs").unwrap();
        raf_source::parse_source(&path, &source, |syntax| {
            let function = function(syntax);
            let syn::Stmt::Expr(syn::Expr::Match(value), _) = &function.block.stmts[0] else {
                panic!("fixture match")
            };
            let arm = &value.arms[0];
            let meter = Meter::new(usize::MAX, usize::MAX);
            let coordinates = Coordinates::new(&source, None, &meter).unwrap();
            let (first, last) = ends(&arm.pat, &coordinates).unwrap().unwrap();
            let at = source.find(spelling).unwrap();
            assert_eq!(
                coordinates.between(first, last).unwrap(),
                at..at + spelling.len(),
                "{spelling}"
            );
            let first = arm_start(arm, &coordinates).unwrap().unwrap();
            assert_eq!(coordinates.range(first).unwrap().start, at);
            without_work(&source, |context| ends(&arm.pat, context));
            without_work(&source, |context| arm_start(arm, context));
        })
        .unwrap();
    }
    statements();
    attributed_arm();
}

fn function(syntax: &syn::File) -> &syn::ItemFn {
    let syn::Item::Fn(function) = &syntax.items[0] else {
        panic!("fixture function")
    };
    function
}

fn statements() {
    let statements = [
        ";",
        "#[allow(unused)] let value: Option<u8> = input;",
        "let Some(value) = input else { return; };",
        "fn nested<T>() where T: Copy {}",
        "#[allow(unused)] opaque!(tokens);",
        "opaque![tokens];",
        "opaque!{tokens}",
    ];
    let source = format!("fn owner() {{ {} }}", statements.join("\n"));
    let path = raf_source::RelPath::new("statement-endpoints.rs").unwrap();
    raf_source::parse_source(&path, &source, |syntax| {
        let function = function(syntax);
        assert_eq!(function.block.stmts.len(), statements.len());
        let meter = Meter::new(usize::MAX, usize::MAX);
        let coordinates = Coordinates::new(&source, None, &meter).unwrap();
        let mut cursor = 0;
        for (node, expected) in function.block.stmts.iter().zip(statements) {
            let (first, last) = super::super::statement::ends(node, &coordinates)
                .unwrap()
                .unwrap();
            let at = cursor + source[cursor..].find(expected).unwrap();
            cursor = at + expected.len();
            assert_eq!(
                coordinates.between(first, last).unwrap(),
                at..at + expected.len(),
                "{expected}"
            );
            without_work(&source, |context| {
                super::super::statement::ends(node, context)
            });
        }
        let syn::Stmt::Expr(empty, Some(_)) = &function.block.stmts[0] else {
            panic!("fixture lone semicolon")
        };
        let empty = expression::ends(empty, &coordinates).unwrap();
        assert_eq!(coordinates.between(empty.0, empty.1).unwrap(), 0..0);
        let syn::Stmt::Local(local) = &function.block.stmts[1] else {
            panic!("fixture local")
        };
        let (first, last) = ends(&local.pat, &coordinates).unwrap().unwrap();
        let at = source.find("value: Option<u8>").unwrap();
        assert_eq!(
            coordinates.between(first, last).unwrap(),
            at..at + "value: Option<u8>".len()
        );
    })
    .unwrap();
}

fn attributed_arm() {
    let source = "fn owner() { match input { #[allow(unused)] Some(value) => (), } }";
    let path = raf_source::RelPath::new("arm-endpoints.rs").unwrap();
    raf_source::parse_source(&path, source, |syntax| {
        let function = function(syntax);
        let syn::Stmt::Expr(syn::Expr::Match(value), _) = &function.block.stmts[0] else {
            panic!("fixture match")
        };
        let meter = Meter::new(usize::MAX, usize::MAX);
        let coordinates = Coordinates::new(source, None, &meter).unwrap();
        let first = arm_start(&value.arms[0], &coordinates).unwrap().unwrap();
        assert_eq!(
            coordinates.range(first).unwrap().start,
            source.find("#[allow").unwrap()
        );
    })
    .unwrap();
}
