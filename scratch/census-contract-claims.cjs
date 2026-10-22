const fs=require('fs');const p='docs/capabilities/census-normal-form.md';let s=fs.readFileSync(p,'utf8');
const a=s.indexOf('## Contract\n\n')+'## Contract\n\n'.length;const b=s.indexOf('\n## Usage',a);let body=s.slice(a,b);
const intro='The normative definition of the normal form is\n[the architecture page](../architecture/normal-form.md). This capability\nimplements it as follows.\n\n';
if(!body.startsWith(intro))throw Error('intro');body=body.slice(intro.length);
const parts=body.split(/\n(?=- )/).map(x=>x.trimEnd().replace(/^- /,'').replace(/^  /gm,''));
const t='raf_census::test_crate::',c='raf_census::';
const clauses=[
['Normalization pipeline',[t+'forms::every_registered_pair_compares_as_registered',t+'golden::every_fixture_fingerprint_is_pinned',t+'golden::the_manifest_is_pinned']],
['Value erasures',[t+'forms::the_value_form_keeps_binary_operators_and_drops_borrows',t+'forms::the_value_adapters_are_the_eleven_of_the_page',t+'forms::every_registered_pair_compares_as_registered']],
['Bound names and item names',[t+'forms::parameters_and_locals_are_numbered_together_by_first_appearance',t+'forms::self_is_not_a_parameter_and_a_field_named_like_one_is_kept',t+'forms::every_registered_pair_compares_as_registered']],
['Literal selection',[t+'forms::the_literal_tier_boundary_is_thirty_value_tokens',t+'forms::below_the_tier_the_final_sequence_keeps_literals_and_above_it_abstracts_them',t+'forms::every_threshold_counts_the_value_form']],
['Shim eligibility and grouping',[t+'forms::shim_forms_exist_for_six_to_twenty_nine_tokens',t+'forms::self_callee_query_distinguishes_generic_types_from_calls',t+'groups::a_forwarder_at_min_tokens_joins_a_shim_group_and_never_a_body_group',t+'units::shim_keys_are_absent_for_trait_impl_methods_and_self_forwarders',t+'units::self_type_arguments_keep_shim_keys_beside_generic_self_callees']],
['Minimum body size',[t+'forms::the_body_print_needs_min_tokens',t+'units::min_tokens_decides_the_body_fingerprint_at_the_default_and_at_ten',t+'groups::bodies_under_min_tokens_never_group']],
['Complete fingerprint identities',[t+'forms::the_digest_is_blake3_of_length_prefixed_tokens',t+'forms::a_fingerprint_displays_and_parses_back',t+'forms::human_abbreviations_do_not_replace_complete_identity',c+'group::tests::digest_suffixes_are_distinct_group_keys']],
['Portable token encoding',[t+'forms::the_digest_is_blake3_of_length_prefixed_tokens']],
['Explicit normal-form gate',[c+'version::tests::the_implemented_version_is_one',c+'version::tests::the_implemented_version_passes',c+'version::tests::other_versions_are_refused_naming_both_and_the_migration']],
['Load options',[c+'options::tests::defaults_are_twenty_tokens_and_eight_mebibytes',c+'options::tests::zero_min_tokens_is_refused_and_one_is_accepted',t+'units::max_file_bytes_caps_a_file_and_the_cap_itself_loads']],
['Actionable errors and panic ownership',[t+'forms::an_unlexable_slice_is_a_source_error_with_a_valid_neighbour',c+'error::tests::each_variant_renders_its_facts',c+'units::tests::collection_worker_panic_keeps_its_payload_and_tears_down_without_invalidating_caller_spans']],
['File contexts',[t+'units::file_context_names_package_path_and_shipping',t+'units::file_context_names_the_target_kinds_that_reach_the_file',t+'units::shipping_is_true_when_any_reach_ships_and_the_shipping_path_is_the_symbol']],
['Source units',[t+'units::each_kind_yields_one_unit_with_every_field',t+'units::a_module_declared_by_a_library_and_a_binary_is_one_unit',t+'units::a_support_module_shared_by_two_test_binaries_is_one_unit_that_does_not_ship',t+'units::shipping_is_true_when_any_reach_ships_and_the_shipping_path_is_the_symbol',t+'units::byte_strings_and_c_strings_are_strings']],
['Deterministic groups',[t+'groups::groups_of_one_are_dropped_and_members_follow_symbol_order',t+'groups::groups_order_by_count_then_tokens_then_fingerprint',c+'group::tests::members_order_by_symbol_then_file_then_line',c+'group::tests::digest_suffixes_are_distinct_group_keys']],
['Tables and shared constants',[t+'groups::every_form_of_a_hex_table_is_reported_in_both_cases',t+'groups::near_misses_of_a_hex_table_are_not_reported',t+'groups::constants_above_ffff_in_two_packages_are_shared',t+'groups::a_constant_in_one_package_is_not_shared',t+'groups::shared_constants_run_by_value_ascending']],
['Job classification',[t+'groups::a_job_classifies_home_entry_point_variant_and_copies',t+'groups::jobs_match_shim_and_table_fingerprints',t+'groups::a_job_that_matches_nothing_has_an_empty_list']],
['Closed canonical reports',[t+'reports::both_reports_read_back_what_they_write',t+'reports::two_cfg_alternatives_on_one_line_have_distinct_unit_nodes',t+'reports::the_index_renders_the_classified_matches_of_every_job',t+'reports::a_report_of_another_report_form_is_refused_naming_both',t+'reports::an_index_of_another_normal_form_reads_and_keeps_its_version',t+'reports::unknown_terms_are_not_a_report',t+'reports::missing_repeated_and_mistyped_properties_are_not_a_report',t+'reports::names_that_disagree_with_their_node_are_not_a_report',t+'reports::orphan_units_and_matches_without_a_named_reifier_are_not_a_report',t+'reports::an_edited_or_respaced_text_is_not_a_report',t+'reports::the_writer_refuses_a_value_outside_the_shape']],
['Version stability',[t+'golden::every_fixture_fingerprint_is_pinned',t+'golden::the_manifest_is_pinned']],
];
if(parts.length!==clauses.length)throw Error('parts '+parts.length);
const contract=parts.map((part,i)=>'### '+clauses[i][0]+'\n<!-- raf:tested-by '+clauses[i][1].join(',')+' -->\n\n'+part+'\n').join('\n');
s=s.slice(0,a)+contract+s.slice(b);
s=s.replace('## Contract\n\n','The normative definition is [the normal-form architecture](../architecture/normal-form.md).\n\n## Contract\n\n');
fs.writeFileSync(p,s);
