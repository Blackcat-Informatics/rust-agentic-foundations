// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Coordinate admission boundaries used by the existing workspace fixture.

use super::{Coordinates, Meter, Refusal};
use proc_macro2::{LineColumn, Span};

pub(in crate::embedded) fn check() {
    opaque_ranges();
    let source = "\u{feff}#! données\r\n/// café\nfn r#type() {}\n";
    let path = raf_source::RelPath::new("coordinates.rs").unwrap();
    raf_source::parse_source(&path, source, |syntax| {
        let meter = Meter::new(usize::MAX, usize::MAX);
        let header = syntax.shebang.as_deref().unwrap();
        let coordinates = Coordinates::new(source, Some(header), &meter).unwrap();
        let characters = source.chars().count() + header.len() - header.chars().count();
        assert_eq!(coordinates.offsets.len(), characters + 1);
        assert_eq!(coordinates.lines.len(), source.lines().count() + 1);
        assert_eq!(coordinates.range(Span::call_site()).unwrap(), 0..0);
        let accent = source.find('é').unwrap();
        // A padded header column counts original bytes after the one-scalar BOM.
        let column = 1 + accent - '\u{feff}'.len_utf8();
        assert_eq!(
            coordinates.byte(LineColumn { line: 1, column }).unwrap(),
            accent
        );
        assert_eq!(
            coordinates
                .byte(LineColumn {
                    line: 1,
                    column: column + 2
                })
                .unwrap(),
            accent + 'é'.len_utf8()
        );
        for position in [
            LineColumn { line: 0, column: 0 },
            LineColumn {
                line: usize::MAX,
                column: 0,
            },
            LineColumn {
                line: 2,
                column: usize::MAX,
            },
            LineColumn {
                line: 1,
                column: column + 1,
            },
        ] {
            assert!(matches!(
                coordinates.byte(position),
                Err(Refusal::Invariant(_))
            ));
        }
        assert_eq!(
            coordinates.byte(LineColumn { line: 2, column: 4 }).unwrap(),
            source.find("café").unwrap()
        );
        assert_eq!(
            coordinates.byte(LineColumn { line: 4, column: 0 }).unwrap(),
            source.len()
        );
        assert!(matches!(
            Coordinates::new(source, Some("#! changed"), &meter),
            Err(Refusal::Invariant(_))
        ));
        allocation_neighbors(source, Some(header));
    })
    .unwrap();
    for source in ["", "\n"] {
        let meter = Meter::new(usize::MAX, usize::MAX);
        let coordinates = Coordinates::new(source, None, &meter).unwrap();
        assert_eq!(
            coordinates
                .byte(LineColumn {
                    line: source.len() + 1,
                    column: 0
                })
                .unwrap(),
            source.len()
        );
    }
}

fn opaque_ranges() {
    for (source, expected) in [
        (
            "type Alias = builtin#opaque(nested(\"café\"));",
            "builtin#opaque(nested(\"café\"))",
        ),
        (
            "fn owner() { match input { box \"café\" => (), } }",
            "box \"café\"",
        ),
    ] {
        let path = raf_source::RelPath::new("opaque-ranges.rs").unwrap();
        raf_source::parse_source(&path, source, |syntax| {
            let tokens = match &syntax.items[0] {
                syn::Item::Type(alias) => {
                    let syn::Type::Verbatim(tokens) = &*alias.ty else {
                        panic!("fixture opaque type")
                    };
                    tokens
                }
                syn::Item::Fn(function) => {
                    let syn::Stmt::Expr(syn::Expr::Match(node), _) = &function.block.stmts[0]
                    else {
                        panic!("fixture match")
                    };
                    let syn::Pat::Verbatim(tokens) = &node.arms[0].pat else {
                        panic!("fixture opaque pattern")
                    };
                    tokens
                }
                _ => panic!("fixture opaque source"),
            };
            opaque_limits(source, expected, tokens);
        })
        .unwrap();
    }
}

fn opaque_limits(source: &str, expected: &str, tokens: &proc_macro2::TokenStream) {
    let probe = Meter::new(usize::MAX, usize::MAX);
    let coordinates = Coordinates::new(source, None, &probe).unwrap();
    let initial = probe.usage().0;
    let ends = coordinates.opaque_ends(tokens).unwrap().unwrap();
    let (work, bytes) = probe.usage();
    let at = source.find(expected).unwrap();
    assert_eq!(
        coordinates.between(ends.0, ends.1).unwrap(),
        at..at + expected.len()
    );

    let exact = Meter::new(work, bytes);
    let coordinates = Coordinates::new(source, None, &exact).unwrap();
    assert!(coordinates.opaque_ends(tokens).unwrap().is_some());
    assert_eq!(exact.usage(), (work, bytes));

    let short_work = Meter::new(work - 1, bytes);
    let coordinates = Coordinates::new(source, None, &short_work).unwrap();
    assert!(matches!(
        coordinates.opaque_ends(tokens),
        Err(Refusal::Work { .. })
    ));

    let short_memory = Meter::new(usize::MAX, bytes - 1);
    let coordinates = Coordinates::new(source, None, &short_memory).unwrap();
    assert!(matches!(
        coordinates.opaque_ends(tokens),
        Err(Refusal::Memory { .. })
    ));
    assert!(
        coordinates
            .opaque_ends(&proc_macro2::TokenStream::new())
            .unwrap()
            .is_none()
    );

    // The same exact memory ceiling admits sequential copies only when every
    // iterator/payload has dropped and its original reservation is refunded.
    let reusable = Meter::new(usize::MAX, bytes);
    let coordinates = Coordinates::new(source, None, &reusable).unwrap();
    for _ in 0..2 {
        assert!(coordinates.opaque_ends(tokens).unwrap().is_some());
    }
    assert_eq!(reusable.usage(), (initial + 2 * (work - initial), bytes));
}

fn allocation_neighbors(source: &str, header: Option<&str>) {
    let probe = Meter::new(usize::MAX, usize::MAX);
    drop(Coordinates::new(source, header, &probe).unwrap());
    let (work, bytes) = probe.usage();
    let exact = Meter::new(work, bytes);
    drop(Coordinates::new(source, header, &exact).unwrap());
    assert_eq!(exact.usage(), (work, bytes));
    let short_work = Meter::new(work - 1, bytes);
    assert!(matches!(
        Coordinates::new(source, header, &short_work),
        Err(Refusal::Work { .. })
    ));
    let short_memory = Meter::new(usize::MAX, bytes - 1);
    assert!(matches!(
        Coordinates::new(source, header, &short_memory),
        Err(Refusal::Memory { .. })
    ));
    // Failed partial tables release the same original account for valid input.
    for _ in 0..2 {
        drop(Coordinates::new("", None, &short_memory).unwrap());
    }
}
