// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Lexical correspondence and refusal boundaries over classified region facts.

use super::*;

fn declaration(byte: usize) -> Declaration {
    Declaration {
        byte,
        family: Family::Value,
    }
}

fn bound(binding: usize, event: Event) -> Atom<'static> {
    Atom {
        joint: false,
        spelling: "renamable identifier",
        occurrence: Some(Occurrence {
            binding,
            event,
            fixed: None,
        }),
    }
}

fn regions<'a>(
    a: &'a [Atom<'a>],
    a_declarations: &'a [Declaration],
    b: &'a [Atom<'a>],
    b_declarations: &'a [Declaration],
) -> (Region<'a>, Region<'a>) {
    (
        Region {
            bytes: 10..100,
            atoms: a,
            declarations: a_declarations,
        },
        Region {
            bytes: 200..300,
            atoms: b,
            declarations: b_declarations,
        },
    )
}

#[test]
fn renamed_bindings_preserve_repeated_uses_and_distinct_shadowing_declarations() {
    let left = [
        bound(0, Event::Declaration),
        bound(0, Event::Use),
        bound(1, Event::Declaration),
        bound(1, Event::Use),
        bound(0, Event::Use),
    ];
    let mut right = [
        bound(2, Event::Declaration),
        bound(2, Event::Use),
        bound(3, Event::Declaration),
        bound(3, Event::Use),
        bound(2, Event::Use),
    ];
    let first = [declaration(20), declaration(40)];
    // Unrelated preceding declarations do not shift fragment correspondence.
    let second = [
        declaration(0),
        declaration(100),
        declaration(220),
        declaration(240),
    ];
    let check = |right: &[Atom<'_>]| {
        let (a, b) = regions(&left, &first, right, &second);
        same(&a, &b, &Meter::new(10_000, 10_000)).unwrap()
    };
    assert!(check(&right));
    right[4] = bound(3, Event::Use);
    assert!(!check(&right));
    right[2] = bound(2, Event::Declaration);
    assert!(!check(&right));
}

#[test]
fn captures_cannot_be_reclassified_as_internal_or_merge_distinct_variables() {
    let left = [bound(0, Event::Use), bound(0, Event::Use)];
    let same_capture = [bound(0, Event::Use), bound(0, Event::Use)];
    let distinct = [bound(0, Event::Use), bound(1, Event::Use)];
    let first = [declaration(0)];
    let captured = [declaration(150), declaration(160)];
    let internal = [declaration(220)];
    for (atoms, declarations, expected) in [
        (&same_capture[..], &captured[..], true),
        (&same_capture[..], &internal[..], false),
        (&distinct[..], &captured[..], false),
    ] {
        let (a, b) = regions(&left, &first, atoms, declarations);
        assert_eq!(same(&a, &b, &Meter::new(10_000, 10_000)).unwrap(), expected);
    }
}

#[test]
fn fixed_names_operators_shorthand_fields_and_control_opcodes_are_retained() {
    for (a, b) in [("external_a", "external_b"), ("+", "-"), ("&", "&~")] {
        let left = [Atom {
            joint: false,
            spelling: a,
            occurrence: None,
        }];
        let right = [Atom {
            joint: false,
            spelling: b,
            occurrence: None,
        }];
        let (a, b) = regions(&left, &[], &right, &[]);
        assert!(!same(&a, &b, &Meter::new(100, 100)).unwrap());
    }
    let first = [declaration(20)];
    let second = [declaration(220)];
    let mut left = bound(0, Event::Declaration);
    left.occurrence.as_mut().unwrap().fixed = Some("field_a");
    let mut right = bound(0, Event::Declaration);
    right.occurrence.as_mut().unwrap().fixed = Some("field_b");
    let check = |left: Atom<'_>,
                 right: Atom<'_>,
                 a_declarations: &[Declaration],
                 b_declarations: &[Declaration]| {
        let a_atoms = [left];
        let b_atoms = [right];
        let (a, b) = regions(&a_atoms, a_declarations, &b_atoms, b_declarations);
        same(&a, &b, &Meter::new(10_000, 10_000)).unwrap()
    };
    assert!(!check(left, right, &first, &second));
    right.occurrence.as_mut().unwrap().fixed = Some("field_a");
    assert!(check(left, right, &first, &second));
    for family in [
        Family::Receiver,
        Family::Label,
        Family::Function,
        Family::Closure,
        Family::Loop,
        Family::Try,
    ] {
        let other = [Declaration { byte: 220, family }];
        assert!(!check(left, right, &first, &other));
    }
    let targets = [Declaration {
        byte: 0,
        family: Family::Loop,
    }];
    left = bound(0, Event::Use);
    right = bound(0, Event::Use);
    left.occurrence.as_mut().unwrap().fixed = Some("break");
    right.occurrence.as_mut().unwrap().fixed = Some("continue");
    assert!(!check(left, right, &targets, &targets));
}

#[test]
fn missing_declaration_and_untyped_control_opcode_refuse_beside_valid_markers() {
    let declaration_atoms = [bound(0, Event::Declaration)];
    let outside = [declaration(0)];
    let (a, b) = regions(&declaration_atoms, &outside, &declaration_atoms, &outside);
    assert_eq!(
        same(&a, &b, &Meter::new(100, 100)),
        Err(Refusal::Invariant(
            "fragment declaration event is outside its region"
        ))
    );
    let inside_left = [declaration(20)];
    let inside_right = [declaration(220)];
    let (a, b) = regions(
        &declaration_atoms,
        &inside_left,
        &declaration_atoms,
        &inside_right,
    );
    assert!(same(&a, &b, &Meter::new(100, 100)).unwrap());
    let mut atom = bound(0, Event::Use);
    let atoms = [atom];
    let target = [Declaration {
        byte: 0,
        family: Family::Loop,
    }];
    let (a, b) = regions(&atoms, &[], &atoms, &target);
    assert_eq!(
        same(&a, &b, &Meter::new(100, 100)),
        Err(Refusal::Invariant("fragment occurrence has no declaration"))
    );
    let (a, b) = regions(&atoms, &target, &atoms, &target);
    assert_eq!(
        same(&a, &b, &Meter::new(100, 100)),
        Err(Refusal::Invariant(
            "fragment control marker has no fixed opcode"
        ))
    );
    atom.occurrence.as_mut().unwrap().fixed = Some("break");
    let marked = [atom];
    let (a, b) = regions(&marked, &target, &marked, &target);
    assert!(same(&a, &b, &Meter::new(100, 100)).unwrap());
}

#[test]
fn exact_shared_work_and_storage_grants_succeed_and_one_unit_short_refuses() {
    let atoms = [
        bound(0, Event::Use),
        bound(1, Event::Use),
        bound(0, Event::Use),
    ];
    let declarations = [declaration(0), declaration(1)];
    let (a, b) = regions(&atoms, &declarations, &atoms, &declarations);
    let measured = Meter::new(10_000, 10_000);
    assert!(same(&a, &b, &measured).unwrap());
    let (work, bytes) = measured.usage();
    assert!(same(&a, &b, &Meter::new(work, bytes)).unwrap());
    assert_eq!(
        same(&a, &b, &Meter::new(work - 1, bytes)),
        Err(Refusal::Work { limit: work - 1 })
    );
    assert_eq!(
        same(&a, &b, &Meter::new(work, bytes - 1)),
        Err(Refusal::Memory { limit: bytes - 1 })
    );
}
