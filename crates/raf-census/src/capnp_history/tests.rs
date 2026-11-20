// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

use std::collections::BTreeSet;

use capnp::{
    any_pointer, any_pointer_list,
    message::Builder,
    primitive_list,
    schema_capnp::{code_generator_request, field},
    serialize,
};

use super::{ChangeKind, DescriptorError, Error, TypeId, compare, descriptor};

const ROOT: TypeId = TypeId(0xa810_4019_48de_c8d3);
const MISSING: TypeId = TypeId(0xa43e_d963_88c0_fea7);

#[derive(Clone, Copy)]
enum LastField {
    Independent,
    Overlap,
    Gap,
    Reference,
    NumericList,
    PointerList,
}

fn descriptor_bytes(last: LastField) -> Vec<u8> {
    let mut message = Builder::new_default();
    let request = message.init_root::<code_generator_request::Builder<'_>>();
    let mut node = request.init_nodes(1).get(0);
    node.set_id(ROOT.0);
    let mut structure = node.init_struct();
    structure.set_data_word_count(1);
    structure.set_pointer_count(1);
    let mut fields = structure.init_fields(2);
    scalar(fields.reborrow().get(0), "first", 0, 0);
    let last_field = fields.get(1);
    match last {
        LastField::Independent => scalar(last_field, "second", 1, 1),
        LastField::Overlap => scalar(last_field, "second", 1, 0),
        LastField::Gap => scalar(last_field, "second", 2, 1),
        LastField::NumericList | LastField::PointerList => {
            list_slot(last_field, matches!(last, LastField::PointerList));
        }
        LastField::Reference => {
            let mut last_field = last_field;
            last_field.set_name("second");
            last_field.reborrow().get_ordinal().set_explicit(1);
            let mut slot = last_field.init_slot();
            slot.reborrow()
                .init_type()
                .init_struct()
                .set_type_id(MISSING.0);
            let _ = slot.init_default_value().init_struct();
        }
    }
    serialize::write_message_to_words(&message)
}

fn list_slot(mut field: field::Builder<'_>, mismatched: bool) {
    field.set_name("numbers");
    field.reborrow().get_ordinal().set_explicit(1);
    let mut slot = field.init_slot();
    slot.reborrow()
        .init_type()
        .init_list()
        .init_element_type()
        .set_uint32(());
    let pointer = slot.init_default_value().init_list();
    if mismatched {
        let _ = pointer.initn_as::<any_pointer_list::Builder<'_>>(1);
    } else {
        pointer
            .initn_as::<primitive_list::Builder<'_, u32>>(1)
            .set(0, 7);
    }
}

fn scalar(mut field: field::Builder<'_>, name: &str, ordinal: u16, offset: u32) {
    field.set_name(name);
    field.reborrow().get_ordinal().set_explicit(ordinal);
    let mut slot = field.init_slot();
    slot.set_offset(offset);
    slot.reborrow().init_type().set_uint32(());
    slot.init_default_value().set_uint32(0);
}

/// A selected type's active slots, ordinal history and dependency closure
/// must be complete before it can contribute a clean comparison.
#[test]
fn invalid_slot_layouts_and_missing_references_refuse() {
    let roots = BTreeSet::from([ROOT]);
    let neighbor = descriptor_bytes(LastField::Independent);
    let base = descriptor::read(
        &neighbor,
        &roots,
        descriptor::Annotations::default(),
        descriptor::Roots::Require(&BTreeSet::new()),
        None,
    )
    .unwrap();
    for (last, expected) in [
        (LastField::Overlap, "overlaps active storage"),
        (LastField::Gap, "not consecutive from zero"),
        (LastField::Reference, "missing schema node"),
    ] {
        let failure = descriptor::read(
            &descriptor_bytes(last),
            &roots,
            descriptor::Annotations::default(),
            descriptor::Roots::Require(&BTreeSet::new()),
            None,
        )
        .unwrap_err();
        assert!(matches!(failure, DescriptorError::Layout(message) if message.contains(expected)));
    }
    let ordinal = deeply_grouped_descriptor("slot", 3, Some(3));
    assert!(matches!(
        descriptor::read(&ordinal, &roots, descriptor::Annotations::default(), descriptor::Roots::AllowMissing(&BTreeSet::new()), None),
        Err(DescriptorError::Layout(message)) if message.contains("ordinary group")
    ));
    let valid_list = descriptor_bytes(LastField::NumericList);
    assert!(
        descriptor::read(
            &valid_list,
            &roots,
            descriptor::Annotations::default(),
            descriptor::Roots::AllowMissing(&BTreeSet::new()),
            None
        )
        .is_ok()
    );
    let invalid_list = descriptor_bytes(LastField::PointerList);
    assert!(matches!(
        descriptor::read(
            &invalid_list,
            &roots,
            descriptor::Annotations::default(),
            descriptor::Roots::AllowMissing(&BTreeSet::new()),
            None
        ),
        Err(DescriptorError::Wire(_))
    ));
    let mut message = Builder::new_default();
    let _ = message.init_root::<any_pointer::Builder<'_>>();
    let null = serialize::write_message_to_words(&message);
    assert!(matches!(
        descriptor::read(&null, &roots, descriptor::Annotations::default(), descriptor::Roots::AllowMissing(&BTreeSet::new()), None),
        Err(DescriptorError::Layout(message)) if message.contains("null root")
    ));
    let _ = message.init_root::<code_generator_request::Builder<'_>>();
    let empty = serialize::write_message_to_words(&message);
    let current = descriptor::read(
        &empty,
        &roots,
        descriptor::Annotations::default(),
        descriptor::Roots::AllowMissing(&BTreeSet::new()),
        None,
    )
    .unwrap();
    let findings = compare::changes(&base, &current).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].kind, ChangeKind::RemovedRoot);
}

fn deeply_grouped_descriptor(prefix: &str, slots: u16, outer_ordinal: Option<u16>) -> Vec<u8> {
    const DEPTH: u32 = 32;
    let mut message = Builder::new_default();
    let request = message.init_root::<code_generator_request::Builder<'_>>();
    let mut nodes = request.init_nodes(DEPTH + 1);
    let group_name = "g".repeat(256);
    for index in 0..=DEPTH {
        let mut node = nodes.reborrow().get(index);
        node.set_id(ROOT.0 + u64::from(index));
        if index != 0 {
            node.set_scope_id(ROOT.0 + u64::from(index - 1));
        }
        let mut structure = node.init_struct();
        structure.set_is_group(index != 0);
        structure.set_data_word_count(slots.div_ceil(2));
        if index == DEPTH {
            let mut fields = structure.init_fields(u32::from(slots));
            for slot in 0..slots {
                scalar(
                    fields.reborrow().get(u32::from(slot)),
                    &format!("{prefix}{slot}"),
                    slot,
                    u32::from(slot),
                );
            }
        } else {
            let mut field = structure.init_fields(1).get(0);
            field.set_name(group_name.as_str());
            if index == 0
                && let Some(ordinal) = outer_ordinal
            {
                field.reborrow().get_ordinal().set_explicit(ordinal);
            }
            field
                .init_group()
                .set_type_id(ROOT.0 + u64::from(index + 1));
        }
    }
    serialize::write_message_to_words(&message)
}

/// Reporting many fully qualified changes cannot amplify a compact nested
/// descriptor into an unbounded retained verdict.
#[test]
fn deep_group_findings_refuse_as_a_whole_above_the_payload_limit() {
    let roots = BTreeSet::from([ROOT]);
    let check = |slots| {
        let base = descriptor::read(
            &deeply_grouped_descriptor("old", slots, None),
            &roots,
            descriptor::Annotations::default(),
            descriptor::Roots::Require(&BTreeSet::new()),
            None,
        )
        .unwrap();
        let current = descriptor::read(
            &deeply_grouped_descriptor("new", slots, None),
            &roots,
            descriptor::Annotations::default(),
            descriptor::Roots::AllowMissing(&BTreeSet::new()),
            None,
        )
        .unwrap();
        compare::changes(&base, &current)
    };
    assert_eq!(check(3).unwrap().len(), 3);
    assert!(matches!(check(384), Err(Error::FindingLimit)));
}

#[derive(Clone, Copy)]
enum Enclosing {
    File,
    MissingEntry,
    DuplicateEntry,
    Orphan,
}

const REQUIRED_MARKER: TypeId = TypeId(0xe914_47cf_636b_013a);

fn required_descriptor(
    value: Option<bool>,
    enclosing: Enclosing,
    declared_marker: bool,
) -> Vec<u8> {
    const FILE: TypeId = TypeId(0xa7c2_bc02_a14a_f2d0);
    let mut message = Builder::new_default();
    let request = message.init_root::<code_generator_request::Builder<'_>>();
    let mut nodes = request.init_nodes(4);
    let mut declaration = nodes.reborrow().get(0);
    declaration.set_id(REQUIRED_MARKER.0);
    declaration.set_scope_id(FILE.0);
    let mut declaration = declaration.init_annotation();
    declaration.set_targets_field(true);
    declaration.init_type().set_void(());

    let mut file = nodes.reborrow().get(1);
    file.set_id(FILE.0);
    file.set_file(());
    let entries = match enclosing {
        Enclosing::File => 1,
        Enclosing::DuplicateEntry => 2,
        Enclosing::MissingEntry | Enclosing::Orphan => 0,
    };
    let mut nested = file.init_nested_nodes(entries + u32::from(declared_marker));
    if declared_marker {
        let mut child = nested.reborrow().get(0);
        child.set_id(REQUIRED_MARKER.0);
        child.set_name("required");
    }
    for (index, name) in (0..entries).zip(["Record", "Duplicate"]) {
        let mut child = nested.reborrow().get(index + u32::from(declared_marker));
        child.set_id(ROOT.0);
        child.set_name(name);
    }

    let mut orphan = nodes.reborrow().get(3);
    orphan.set_id(MISSING.0);
    let _ = orphan.reborrow().init_struct();
    let mut child = orphan.init_nested_nodes(1).get(0);
    child.set_id(ROOT.0);
    child.set_name("Record");

    let mut structure = nodes.get(2);
    structure.set_id(ROOT.0);
    let scope = if matches!(enclosing, Enclosing::Orphan) {
        MISSING
    } else {
        FILE
    };
    structure.set_scope_id(scope.0);
    let mut structure = structure.init_struct();
    structure.set_pointer_count(1);
    let mut field = structure.init_fields(1).get(0);
    field.set_name("label");
    field.reborrow().get_ordinal().set_explicit(0);
    let mut slot = field.reborrow().init_slot();
    slot.reborrow().init_type().set_text(());
    slot.init_default_value().set_text("");
    let mut annotation = field.init_annotations(1).get(0);
    annotation.set_id(REQUIRED_MARKER.0);
    match value {
        Some(true) => annotation.init_value().set_void(()),
        Some(false) => annotation.init_value().set_bool(true),
        None => {}
    }
    serialize::write_message_to_words(&message)
}

/// A Void marker needs a present value and an ordinary declaration that can
/// be traced through its enclosing schema nodes to a file.
#[test]
fn required_annotation_refuses_invalid_values_and_undeclared_structs() {
    let roots = BTreeSet::from([ROOT]);
    for (value, enclosing, declared_marker, diagnostic) in [
        (Some(true), Enclosing::File, true, None),
        (
            Some(false),
            Enclosing::File,
            true,
            Some("expected one Void application"),
        ),
        (
            None,
            Enclosing::File,
            true,
            Some("expected one Void application"),
        ),
        (
            Some(true),
            Enclosing::MissingEntry,
            true,
            Some("must belong to a declared struct"),
        ),
        (
            Some(true),
            Enclosing::DuplicateEntry,
            true,
            Some("must belong to a declared struct"),
        ),
        (
            Some(true),
            Enclosing::Orphan,
            true,
            Some("must belong to a declared struct"),
        ),
        (
            Some(true),
            Enclosing::File,
            false,
            Some("must belong to a declared schema scope"),
        ),
    ] {
        let bytes = required_descriptor(value, enclosing, declared_marker);
        let read = descriptor::read(
            &bytes,
            &roots,
            descriptor::Annotations {
                required: Some(REQUIRED_MARKER),
                paired_presence: None,
            },
            descriptor::Roots::Require(&BTreeSet::new()),
            None,
        );
        if let Some(diagnostic) = diagnostic {
            assert!(matches!(
                read,
                Err(DescriptorError::Layout(message)) if message.contains(diagnostic)
            ));
        } else {
            assert!(read.is_ok(), "{read:?}");
        }
    }
}
