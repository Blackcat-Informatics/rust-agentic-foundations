// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Original type endpoints in the existing controlled workspace fixture.

use super::*;
use crate::embedded::coordinates::Coordinates;
use crate::embedded::native_spans::fixture::without_work;
use crate::embedded::resource::Meter;

pub(in crate::embedded) fn check() {
    for spelling in [
        "[u8; 3]",
        "[u8]",
        "(u8,)",
        "(u8)",
        "*const u8",
        "&'a mut u8",
        "for<'a> unsafe extern \"C\" fn(&'a u8) -> &'a u8",
        "fn(u8)",
        "_",
        "!",
        "<S as Trait>::Associated<T>",
        "opaque!(tokens)",
        "impl for<'a> Trait<&'a u8> + use<'a, T>",
        "dyn (Trait) + 'a",
        "Trait + Send",
        "dyn Fn(u8) -> *const u8",
        "builtin#opaque()",
    ] {
        let source = format!("// prefix\ntype Alias = {spelling};");
        let path = raf_source::RelPath::new("type-endpoints.rs").unwrap();
        raf_source::parse_source(&path, &source, |syntax| {
            let syn::Item::Type(alias) = &syntax.items[0] else {
                panic!("fixture type alias")
            };
            let meter = Meter::new(usize::MAX, usize::MAX);
            let coordinates = Coordinates::new(&source, None, &meter).unwrap();
            let (first, last) = ends(&alias.ty, &coordinates).unwrap().unwrap();
            let at = source.find(spelling).unwrap();
            assert_eq!(
                coordinates.between(first, last).unwrap(),
                at..at + spelling.len(),
                "{spelling}"
            );
            without_work(&source, |context| ends(&alias.ty, context));
        })
        .unwrap();
    }
    arguments();
    empty_components();
}

fn arguments() {
    let spelling = "Wrapper<'a, u8, 3, Assoc = u16, VALUE = {2}, Constraint: Trait>";
    let source = format!("type Alias = {spelling};");
    let path = raf_source::RelPath::new("argument-endpoints.rs").unwrap();
    raf_source::parse_source(&path, &source, |syntax| {
        let syn::Item::Type(alias) = &syntax.items[0] else {
            panic!("fixture alias")
        };
        let syn::Type::Path(path) = &*alias.ty else {
            panic!("fixture path")
        };
        let syn::PathArguments::AngleBracketed(args) =
            &path.path.segments.get(0).unwrap().arguments
        else {
            panic!("fixture arguments")
        };
        let meter = Meter::new(usize::MAX, usize::MAX);
        let coordinates = Coordinates::new(&source, None, &meter).unwrap();
        assert_eq!(args.args.len(), 6);
        for (pair, expected) in
            args.args
                .pairs()
                .zip(["'a", "u8", "3", "Assoc", "VALUE", "Constraint"])
        {
            let node = pair.into_value();
            let first = super::super::argument_start(node, &coordinates).unwrap();
            assert_eq!(
                coordinates.range(first).unwrap().start,
                source.find(expected).unwrap()
            );
            without_work(&source, |context| {
                super::super::argument_start(node, context)
            });
        }
    })
    .unwrap();
}

fn empty_components() {
    // These constructed inputs check empty-emission composition, not parser
    // acceptance of incomplete syntax. All expected positions come from data.
    let source = "type Alias = *const u8;";
    let path = raf_source::RelPath::new("empty-type-endpoints.rs").unwrap();
    raf_source::parse_source(&path, source, |syntax| {
        let syn::Item::Type(alias) = &syntax.items[0] else {
            panic!("fixture alias")
        };
        let syn::Type::Ptr(pointer) = &*alias.ty else {
            panic!("fixture pointer")
        };
        let syn::PointerMutability::Const(token) = &pointer.mutability else {
            panic!("fixture pointer qualifier")
        };
        let meter = Meter::new(usize::MAX, usize::MAX);
        let coordinates = Coordinates::new(source, None, &meter).unwrap();
        let empty = syn::Type::Verbatim(proc_macro2::TokenStream::new());
        assert!(ends(&empty, &coordinates).unwrap().is_none());
        let pointer = syn::Type::Ptr(syn::TypePtr {
            attrs: Vec::new(),
            star_token: syn::Token![*](pointer.star_token.spans[0]),
            mutability: syn::PointerMutability::Const(syn::Token![const](token.span)),
            elem: Box::new(empty),
        });
        let (first, last) = ends(&pointer, &coordinates).unwrap().unwrap();
        let at = source.find("*const").unwrap();
        assert_eq!(coordinates.between(first, last).unwrap(), at..at + 6);
    })
    .unwrap();
}
