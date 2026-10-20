// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The golden values of normal form 1.
//!
//! ANY CHANGE TO A PINNED VALUE IN THIS FILE REQUIRES A `normal_form` BUMP.
//! A fingerprint names what a body says under one version of the form, and a
//! ledger stores fingerprints, so a pinned value that moves without a new
//! version silently breaks every ledger. When the form changes on purpose,
//! bump `NORMAL_FORM`, run `cargo run -p raf-census --example goldens`, and
//! paste its output over `MANIFEST` and `TABLE` in the same change.
//!
//! The test fails when a registry fixture has no pinned line, when a pinned
//! line has no fixture, and when a pinned value differs from the computed one.

mod support;

use std::collections::BTreeMap;

use support::fixtures::{registry, table};

/// The manifest of normal form 1: encoding, passes, levels, thresholds, lists.
const MANIFEST: &str = "\
encoding tokens-len-u32le-utf8/blake3/first8-be-hex16\n\
passes rename_params rename_locals kind_literals erase_held\n\
levels verbatim shim literal-tier structural value\n\
literal_tier 30\n\
min_shim_tokens 6\n\
shim_max_tokens 30\n\
max_nesting 128\n\
value_adapters as_str as_ref as_slice as_mut as_deref borrow borrow_mut clone cloned copied to_owned\n\
weak_keywords gen union raw safe macro_rules\n";

/// One line per registry fixture: `id tokens body shim`.
const TABLE: &str = "\
structural_renamed_locals.a 21 body:95f29c89adf12ac3 -\n\
structural_renamed_locals.b 21 body:95f29c89adf12ac3 -\n\
structural_renamed_params.a 21 body:95f29c89adf12ac3 -\n\
structural_renamed_params.b 21 body:95f29c89adf12ac3 -\n\
structural_renamed_lifetimes.a 25 body:e405d52e0ca9edca -\n\
structural_renamed_lifetimes.b 25 body:e405d52e0ca9edca -\n\
structural_different_function.a 21 body:95f29c89adf12ac3 -\n\
structural_different_function.b 21 body:e27910f531a6495b -\n\
structural_different_method.a 12 body:797142a83adc122a -\n\
structural_different_method.b 12 body:acc1eeab251d1a15 -\n\
structural_different_field.a 10 body:181c093c13baac67 -\n\
structural_different_field.b 10 body:1faf0cd405f4f058 -\n\
structural_different_type.a 15 body:7d1a4d959447ea28 -\n\
structural_different_type.b 15 body:1c6ca5063215def7 -\n\
structural_local_type_declared.a 13 body:decd3988380fa1a5 -\n\
structural_local_type_declared.b 13 body:decd3988380fa1a5 -\n\
structural_tuple_field_kept.a 7 body:e2a9d8c5dbcad44f -\n\
structural_tuple_field_kept.b 7 body:4f345172b382003e -\n\
value_borrow.a 5 body:0f767795a70a02bb -\n\
value_borrow.b 4 body:0f767795a70a02bb -\n\
value_borrow_mut.a 6 body:0f767795a70a02bb shim:4875f30ad3a57cdd\n\
value_borrow_mut.b 4 body:0f767795a70a02bb -\n\
value_double_borrow.a 6 body:0f767795a70a02bb shim:588cf69795246421\n\
value_double_borrow.b 4 body:0f767795a70a02bb -\n\
value_deref.a 5 body:0f767795a70a02bb -\n\
value_deref.b 4 body:0f767795a70a02bb -\n\
value_ref_binding.a 20 body:c51942289b8a65e7 shim:291cf14185c65d60\n\
value_ref_binding.b 19 body:c51942289b8a65e7 shim:47be6280f16d0bb8\n\
value_ref_mut_binding.a 21 body:c51942289b8a65e7 shim:f01dfa44a5b9ec50\n\
value_ref_mut_binding.b 19 body:c51942289b8a65e7 shim:47be6280f16d0bb8\n\
value_adapter_as_str.a 8 body:0f767795a70a02bb shim:12c00273b0509843\n\
value_adapter_as_str.b 4 body:0f767795a70a02bb -\n\
value_adapter_as_ref.a 8 body:0f767795a70a02bb shim:c71589dc93efb45d\n\
value_adapter_as_ref.b 4 body:0f767795a70a02bb -\n\
value_adapter_as_slice.a 8 body:0f767795a70a02bb shim:a2e3491aabd9fe03\n\
value_adapter_as_slice.b 4 body:0f767795a70a02bb -\n\
value_adapter_as_mut.a 8 body:0f767795a70a02bb shim:1b89830c43c303ea\n\
value_adapter_as_mut.b 4 body:0f767795a70a02bb -\n\
value_adapter_as_deref.a 8 body:0f767795a70a02bb shim:8c0dd302eeae104c\n\
value_adapter_as_deref.b 4 body:0f767795a70a02bb -\n\
value_adapter_borrow.a 8 body:0f767795a70a02bb shim:4d9a8e1fe186ed30\n\
value_adapter_borrow.b 4 body:0f767795a70a02bb -\n\
value_adapter_borrow_mut.a 8 body:0f767795a70a02bb shim:708e8a338dc96dd1\n\
value_adapter_borrow_mut.b 4 body:0f767795a70a02bb -\n\
value_adapter_clone.a 8 body:0f767795a70a02bb shim:f145a3228c7c2b39\n\
value_adapter_clone.b 4 body:0f767795a70a02bb -\n\
value_adapter_cloned.a 8 body:0f767795a70a02bb shim:d5f40972ae755644\n\
value_adapter_cloned.b 4 body:0f767795a70a02bb -\n\
value_adapter_copied.a 8 body:0f767795a70a02bb shim:6512ec3acf452966\n\
value_adapter_copied.b 4 body:0f767795a70a02bb -\n\
value_adapter_to_owned.a 8 body:0f767795a70a02bb shim:43b9bc7ac6a7efa0\n\
value_adapter_to_owned.b 4 body:0f767795a70a02bb -\n\
value_adapter_with_argument_kept.a 9 body:2347a33ac3e35344 shim:2347a33ac3e35344\n\
value_adapter_with_argument_kept.b 4 body:0f767795a70a02bb -\n\
value_full_range.a 9 body:0f767795a70a02bb shim:fa623a293fce6d2a\n\
value_full_range.b 5 body:0f767795a70a02bb -\n\
value_partial_range_kept.a 10 body:94e9d03d52d1a4b5 shim:d063387bf9110515\n\
value_partial_range_kept.b 5 body:0f767795a70a02bb -\n\
value_range_to_kept.a 10 body:a6765939ad39f54a shim:93141a8e3e9bb77a\n\
value_range_to_kept.b 5 body:0f767795a70a02bb -\n\
value_let_annotation.a 11 body:b99d8c5cee486b40 -\n\
value_let_annotation.b 9 body:b99d8c5cee486b40 -\n\
value_let_annotation_generic.a 17 body:b99d8c5cee486b40 -\n\
value_let_annotation_generic.b 9 body:b99d8c5cee486b40 -\n\
value_erased_names_do_not_shift_numbers.a 22 body:fea1618f95cf041c -\n\
value_erased_names_do_not_shift_numbers.b 17 body:fea1618f95cf041c -\n\
value_type_turbofish.a 13 body:7fb66880a8b3a845 shim:e78c493e8e35ce18\n\
value_type_turbofish.b 5 body:7fb66880a8b3a845 -\n\
value_const_turbofish_kept.a 9 body:45026bbba235f1d2 shim:45026bbba235f1d2\n\
value_const_turbofish_kept.b 4 body:534ed5ba5aac4f5c -\n\
value_bool_turbofish_kept.a 9 body:87ee53773ccf49ef shim:87ee53773ccf49ef\n\
value_bool_turbofish_kept.b 9 body:020a9a61097d4835 shim:020a9a61097d4835\n\
value_index_reference.a 5 body:b195ad401e59e444 -\n\
value_index_reference.b 4 body:b195ad401e59e444 -\n\
value_binary_and_kept.a 6 body:4ff5873ae1028c47 shim:4ff5873ae1028c47\n\
value_binary_and_kept.b 5 body:ae705e38df2ce7e6 -\n\
value_binary_mul_kept.a 6 body:cf6fcb46129a87b3 shim:cf6fcb46129a87b3\n\
value_binary_mul_kept.b 5 body:ae705e38df2ce7e6 -\n\
value_logical_and_kept.a 7 body:17a8159319a7b254 shim:17a8159319a7b254\n\
value_logical_and_kept.b 5 body:ae705e38df2ce7e6 -\n\
value_compound_assign_kept.a 10 body:a972fadf6c45665f -\n\
value_compound_assign_kept.b 6 body:5e73238be60a98f7 -\n\
value_binary_after_call_kept.a 6 body:14555388857cf65f shim:14555388857cf65f\n\
value_binary_after_call_kept.b 5 body:dbd2b465e2f37a46 -\n\
tier_25.a 25 body:bae17ce423758aed -\n\
tier_25.b 25 body:291a4f75ee3b7d6e -\n\
tier_29.a 29 body:5363862fa58ae661 -\n\
tier_29.b 29 body:147e186d4c12f6b2 -\n\
tier_30.a 30 body:0fe4f26092b867f9 -\n\
tier_30.b 30 body:0fe4f26092b867f9 -\n\
tier_35.a 35 body:742087969dfcf3f6 -\n\
tier_35.b 35 body:742087969dfcf3f6 -\n\
tier_30_kinds_differ.a 30 body:0fe4f26092b867f9 -\n\
tier_30_kinds_differ.b 30 body:7ab910272c018957 -\n\
tier_30_float_is_not_int.a 30 body:a9c176eabd1d7aa9 -\n\
tier_30_float_is_not_int.b 30 body:7ab910272c018957 -\n\
tier_30_big_int_is_int.a 30 body:7ab910272c018957 -\n\
tier_30_big_int_is_int.b 30 body:7ab910272c018957 -\n\
tier_29_big_int_kept.a 29 body:2bdd17114b5c6230 -\n\
tier_29_big_int_kept.b 29 body:2f5cd06a65147fcd -\n\
tier_30_suffix_is_part_of_the_kind.a 30 body:7ab910272c018957 -\n\
tier_30_suffix_is_part_of_the_kind.b 30 body:7ab910272c018957 -\n\
tier_30_tuple_index_kept.a 30 body:719a04df3958129a -\n\
tier_30_tuple_index_kept.b 30 body:573b08b98205895a -\n\
tier_30_range_end_is_a_literal.a 30 body:1e4c474c7db0f5a5 -\n\
tier_30_range_end_is_a_literal.b 30 body:1e4c474c7db0f5a5 -\n\
literal_hex_and_decimal.a 4 body:260e2d80b4f84d52 -\n\
literal_hex_and_decimal.b 4 body:260e2d80b4f84d52 -\n\
literal_separators.a 4 body:8b9ed6eb71e56078 -\n\
literal_separators.b 4 body:8b9ed6eb71e56078 -\n\
literal_octal_binary.a 6 body:a5a021dc7409bb02 shim:a5a021dc7409bb02\n\
literal_octal_binary.b 6 body:a5a021dc7409bb02 shim:a5a021dc7409bb02\n\
literal_suffix_kept.a 4 body:d1d35c896bca55db -\n\
literal_suffix_kept.b 4 body:d1d35c896bca55db -\n\
literal_different_value.a 4 body:260e2d80b4f84d52 -\n\
literal_different_value.b 4 body:8b99bdb0caa7c4be -\n\
literal_suffix_differs.a 4 body:d1d35c896bca55db -\n\
literal_suffix_differs.b 4 body:093bbeb77c74b484 -\n\
literal_float_is_not_int.a 4 body:c8af592a4ac2eb3f -\n\
literal_float_is_not_int.b 4 body:9de2285b669c60b2 -\n\
literal_string_spelling_differs.a 4 body:31cb562ee0c5f4b3 -\n\
literal_string_spelling_differs.b 4 body:cfd65c1c5bc135c1 -\n\
shim_5 5 body:072159ca8d2de444 -\n\
shim_6 6 body:900b617d60d4ddb6 shim:900b617d60d4ddb6\n\
shim_29 29 body:0b3f0d99a9e75aa1 shim:0b3f0d99a9e75aa1\n\
shim_30 30 body:552388bd5c9348b8 -\n\
shim_renamed_params.a 6 body:0bc8930ce972b811 shim:0bc8930ce972b811\n\
shim_renamed_params.b 6 body:0bc8930ce972b811 shim:0bc8930ce972b811\n\
shim_only_own_params_renamed.a 6 body:0bc8930ce972b811 shim:2acd60af59007ce5\n\
shim_only_own_params_renamed.b 6 body:0bc8930ce972b811 shim:da2914a4c6b3f8e3\n\
shim_different_callee.a 6 body:0bc8930ce972b811 shim:0bc8930ce972b811\n\
shim_different_callee.b 6 body:7af8d8648cdb2db5 shim:7af8d8648cdb2db5\n\
shim_different_fixed_argument.a 6 body:0bc8930ce972b811 shim:0bc8930ce972b811\n\
shim_different_fixed_argument.b 6 body:69e11e4c9c80a35e shim:69e11e4c9c80a35e\n\
shim_parameter_order.a 8 body:de92a8b3421d9329 shim:de92a8b3421d9329\n\
shim_parameter_order.b 8 body:9adc3c2762f42733 shim:9adc3c2762f42733\n\
shim_let_body 13 body:b6a27333735a9fb2 -\n\
shim_two_statements 13 body:4a7648548b773432 -\n\
shim_trailing_semicolon 9 body:32a80c15e231526c shim:32a80c15e231526c\n\
shim_mentions_self 10 body:21c93a1e0ab84e9e -\n\
shim_turbofish_forwarder.a 11 body:0bc8930ce972b811 shim:0678d206215e13d2\n\
shim_turbofish_forwarder.b 11 body:0bc8930ce972b811 shim:0678d206215e13d2\n\
shim_method_turbofish 10 body:e1daab75de1838f0 shim:63453ddc13a54258\n\
shim_turbofish_type_differs.a 11 body:0bc8930ce972b811 shim:0678d206215e13d2\n\
shim_turbofish_type_differs.b 11 body:0bc8930ce972b811 shim:9ccd131097a9b704\n\
shim_turbofish_without_call 15 body:75c451cb055aa503 -\n\
shim_without_call 9 body:cd8e7181776f4160 -\n\
shim_macro_call 9 body:d6a18ab1d813e0a5 shim:d6a18ab1d813e0a5\n\
macro_arguments.a 9 body:5300562c3f556ac5 shim:f05e44a0be6bd1a3\n\
macro_arguments.b 9 body:5300562c3f556ac5 shim:a468d3a9dbc58e56\n\
macro_inline_capture.a 5 body:4add99615ee9bb66 -\n\
macro_inline_capture.b 5 body:4add99615ee9bb66 -\n\
macro_inline_capture_local.a 13 body:8b73fa3e51fc4fde -\n\
macro_inline_capture_local.b 13 body:8b73fa3e51fc4fde -\n\
macro_unknown_name_kept.a 7 body:2b20a1eef898aae0 shim:2b20a1eef898aae0\n\
macro_unknown_name_kept.b 7 body:cc7b50db97a93c13 shim:cc7b50db97a93c13\n\
macro_positional_kept.a 7 body:5f39f17eae70196b shim:5f39f17eae70196b\n\
macro_positional_kept.b 7 body:bbde246f89d6ae90 shim:bbde246f89d6ae90\n\
macro_escaped_brace_kept.a 9 body:5d0c64128f4c5f2c shim:5d0c64128f4c5f2c\n\
macro_escaped_brace_kept.b 9 body:e2b9a82f27480b9b shim:e2b9a82f27480b9b\n\
macro_unicode_escape_is_not_a_placeholder.a 5 body:149fb6d70b94c8d5 -\n\
macro_unicode_escape_is_not_a_placeholder.b 5 body:149fb6d70b94c8d5 -\n\
macro_arguments_above_tier.a 50 body:2122d4097acf996f -\n\
macro_arguments_above_tier.b 50 body:2122d4097acf996f -\n\
macro_inline_capture_above_tier.a 46 body:e09a3dbbb1020730 -\n\
macro_inline_capture_above_tier.b 46 body:e09a3dbbb1020730 -\n\
numbering_unused_trailing_parameter.a 6 body:9825213dff3a5d1a -\n\
numbering_unused_trailing_parameter.b 6 body:9825213dff3a5d1a -\n\
numbering_used_trailing_parameter.a 6 body:668143aca5669002 -\n\
numbering_used_trailing_parameter.b 6 body:9825213dff3a5d1a -\n\
numbering_placeholder_parameter.a 10 body:8c024decdf3360f1 -\n\
numbering_placeholder_parameter.b 10 body:2027574a377e6303 -\n\
numbering_placeholder_parameter_above_tier.a 51 body:efd1e2ef5564bef1 -\n\
numbering_placeholder_parameter_above_tier.b 51 body:edb8aec3a585bf74 -\n\
numbering_placeholder_versus_argument.a 10 body:8c024decdf3360f1 -\n\
numbering_placeholder_versus_argument.b 12 body:4fd853c0093a062f -\n\
lexical_gen_kept.a 9 body:0d185a5e08a04275 -\n\
lexical_gen_kept.b 9 body:b99d8c5cee486b40 -\n\
lexical_union_kept.a 9 body:21953eda92b74112 -\n\
lexical_union_kept.b 9 body:b99d8c5cee486b40 -\n\
lexical_raw_kept.a 9 body:1c8f2dac62d70929 -\n\
lexical_raw_kept.b 9 body:b99d8c5cee486b40 -\n\
lexical_safe_kept.a 9 body:4bcd02328941de90 -\n\
lexical_safe_kept.b 9 body:b99d8c5cee486b40 -\n\
lexical_macro_rules_kept.a 9 body:1ab877ecb3fa6f35 -\n\
lexical_macro_rules_kept.b 9 body:b99d8c5cee486b40 -\n\
lexical_macro_rules_definition.a 18 body:0583026ec860d231 -\n\
lexical_macro_rules_definition.b 18 body:0583026ec860d231 -\n\
lexical_wildcard_kept.a 8 body:c53227ea2077fa9c -\n\
lexical_wildcard_kept.b 8 body:b90ccc00dcd1aa81 -\n\
lexical_nesting_128.a 257 body:577e8c5e2bdea440 -\n\
lexical_nesting_128.b 257 body:577e8c5e2bdea440 -\n\
empty_body 0 - -\n";

#[test]
fn the_manifest_is_pinned() {
    assert_eq!(raf_census::manifest(), MANIFEST);
}

#[test]
fn the_pinned_version_is_the_implemented_one() {
    assert_eq!(raf_census::NORMAL_FORM, 1);
}

/// Every way `pinned` fails to be the table of `computed`: a line without a
/// pin, a pin without a line, a moved value and an id that appears twice.
fn pin_problems(computed: &str, pinned: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let mut seen = |text: &str, side: &str| {
        let mut ids = BTreeMap::new();
        for line in text.lines() {
            let (id, value) = line.split_once(' ').expect("a line has an id");
            if ids.insert(id.to_owned(), value.to_owned()).is_some() {
                problems.push(format!("{id}: {side} twice"));
            }
        }
        ids
    };
    let (computed, pinned) = (seen(computed, "computed"), seen(pinned, "pinned"));
    for (id, value) in &computed {
        match pinned.get(id) {
            None => problems.push(format!("{id}: unpinned")),
            Some(pin) if pin != value => problems.push(format!(
                "{id}: moved from `{pin}` to `{value}`; a changed pin requires a normal_form bump"
            )),
            Some(_) => {}
        }
    }
    for id in pinned.keys().filter(|id| !computed.contains_key(*id)) {
        problems.push(format!("{id}: orphan pin"));
    }
    problems
}

#[test]
fn every_fixture_fingerprint_is_pinned() {
    let computed = table(&registry());
    let problems = pin_problems(&computed, TABLE);
    assert!(
        problems.is_empty(),
        "run the goldens example to see the table: {problems:?}"
    );
}

#[test]
fn the_pin_check_names_an_unpinned_orphan_moved_and_duplicated_entry() {
    let pinned = "a 1 body:0000000000000001 -\nb 2 - -\n";
    assert!(
        pin_problems(pinned, pinned).is_empty(),
        "the neighbour passes"
    );
    let one = |computed: &str, pinned: &str| pin_problems(computed, pinned);
    assert_eq!(one("a 1 x -\nc 3 - -\n", "a 1 x -\n"), ["c: unpinned"]);
    assert_eq!(one("a 1 x -\n", "a 1 x -\nd 4 - -\n"), ["d: orphan pin"]);
    let moved = one("a 1 y -\n", "a 1 x -\n");
    assert_eq!(moved.len(), 1);
    assert!(moved[0].starts_with("a: moved") && moved[0].contains("normal_form bump"));
    assert_eq!(one("a 1 x -\n", "a 1 x -\na 1 x -\n"), ["a: pinned twice"]);
    assert_eq!(
        one("a 1 x -\na 1 x -\n", "a 1 x -\n"),
        ["a: computed twice"]
    );
}
