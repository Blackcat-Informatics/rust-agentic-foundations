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
    let base = descriptor::read(&neighbor, &roots, true).unwrap();
    for (last, expected) in [
        (LastField::Overlap, "overlaps active storage"),
        (LastField::Gap, "not consecutive from zero"),
        (LastField::Reference, "missing schema node"),
    ] {
        let failure = descriptor::read(&descriptor_bytes(last), &roots, true).unwrap_err();
        assert!(matches!(failure, DescriptorError::Layout(message) if message.contains(expected)));
    }
    let ordinal = deeply_grouped_descriptor("slot", 3, Some(3));
    assert!(matches!(
        descriptor::read(&ordinal, &roots, false),
        Err(DescriptorError::Layout(message)) if message.contains("ordinary group")
    ));
    let valid_list = descriptor_bytes(LastField::NumericList);
    assert!(descriptor::read(&valid_list, &roots, false).is_ok());
    let invalid_list = descriptor_bytes(LastField::PointerList);
    assert!(matches!(
        descriptor::read(&invalid_list, &roots, false),
        Err(DescriptorError::Wire(_))
    ));
    let mut message = Builder::new_default();
    let _ = message.init_root::<any_pointer::Builder<'_>>();
    let null = serialize::write_message_to_words(&message);
    assert!(matches!(
        descriptor::read(&null, &roots, false),
        Err(DescriptorError::Layout(message)) if message.contains("null root")
    ));
    let _ = message.init_root::<code_generator_request::Builder<'_>>();
    let empty = serialize::write_message_to_words(&message);
    let current = descriptor::read(&empty, &roots, false).unwrap();
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
        let base =
            descriptor::read(&deeply_grouped_descriptor("old", slots, None), &roots, true).unwrap();
        let current = descriptor::read(
            &deeply_grouped_descriptor("new", slots, None),
            &roots,
            false,
        )
        .unwrap();
        compare::changes(&base, &current)
    };
    assert_eq!(check(3).unwrap().len(), 3);
    assert!(matches!(check(384), Err(Error::FindingLimit)));
}
