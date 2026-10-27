---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-strict
title: "Complete workplan policy"
description: "The complete default policy for examples, claims, explained public items and lines, full sections, outcome reasons and a shrinking debt record."
state: delivered
bindings:
  code:
    - raf_workplan::record
    - raf_workplan::Workplan::write_debt_in
    - raf_workplan::strict
    - raf_workplan::Workplan::claims_report
    - raf_workplan::coverage::render::claims_table
    - raf_workplan::obligation::Touched
    - raf_workplan::obligation::Touch
    - raf_workplan::coverage::lines
    - raf_workplan::Workplan::lines_report
    - raf_workplan::coverage::render::lines_table
    - raf_workplan::debt
    - raf_workplan::Gate::write_debt
    - raf_workplan::Gate::record_options
    - raf_workplan::Opened::write_debt
    - raf_workplan::work
  tests:
    - raf_workplan::strict::tests::the_default_check_judges_each_policy_unit_once_without_policy_work_during_load
    - raf_workplan::test_crate::strict::the_complete_policy_accepts_the_documented_repository
    - raf_workplan::test_crate::strict::a_capability_with_no_example_is_refused_and_one_with_one_example_is_not
    - raf_workplan::test_crate::strict::verification_without_the_matrix_is_refused_and_with_the_block_is_not
    - raf_workplan::test_crate::strict::a_section_of_nineteen_words_is_thin_and_one_of_twenty_is_not
    - raf_workplan::test_crate::strict::code_spans_fenced_code_comments_and_the_heading_are_not_prose
    - raf_workplan::test_crate::strict::usage_that_names_no_bound_example_is_refused_and_a_code_span_of_it_is_not
    - raf_workplan::test_crate::strict::constraints_without_a_list_item_are_refused_and_one_list_item_is_not
    - raf_workplan::test_crate::strict::a_binding_reached_package_is_enforced_and_an_unreached_one_is_not
    - raf_workplan::test_crate::strict::a_scoped_package_is_enforced_once_with_a_binding_present
    - raf_workplan::test_crate::strict::an_exclusion_of_the_record_takes_a_gap_of_a_bound_package_out
    - raf_workplan::test_crate::strict::a_missing_or_invalid_coverage_record_is_reported_before_package_gaps
    - raf_workplan::test_crate::strict::the_words_of_a_generated_block_are_not_prose_of_its_section
    - raf_workplan::test_crate::strict::a_capability_that_is_not_delivered_has_no_finding_of_the_content_rules
    - raf_workplan::test_crate::strict::a_code_span_inside_a_comment_names_no_example_in_usage
    - raf_workplan::test_crate::strict::a_thematic_break_and_an_empty_item_are_no_limitation
    - raf_workplan::test_crate::strict::while_a_capability_document_does_not_load_no_gap_of_a_bound_package_is_reported
    - raf_workplan::test_crate::strict::every_policy_refusal_case_is_enforced_by_the_default_check
    - raf_workplan::test_crate::strict::content_and_binding_findings_follow_load_findings
    - raf_workplan::test_crate::strict::the_removed_policy_key_is_rejected_even_for_its_former_values
    - raf_workplan::test_crate::strict::the_default_gate_judges_the_complete_policy_in_its_opened_workplan
    - raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind
    - raf_workplan::test_crate::complete::completion_after_record_removal_enforces_the_complete_policy
    - raf_workplan::test_crate::bound::a_no_run_example_meets_the_strict_example_rule_and_verify_refuses_it_beside_a_runnable_one
    - raf_okf::test_crate::records::a_coverage_record_rejects_the_removed_policy_key
    - raf_okf::test_crate::record_writer::a_coverage_record_writes_only_its_closed_policy_independent_keys
    - raf_okf::test_crate::records::a_ceiling_names_a_crate_one_whole_number_of_lines_or_a_percent_and_an_optional_class
    - raf_okf::test_crate::records::a_ceiling_of_another_shape_is_refused_with_its_rule
    - raf_okf::test_crate::records::an_exception_names_one_item_prefix_file_or_directory_and_its_reason
    - raf_okf::test_crate::records::an_exception_of_another_shape_or_without_a_reason_is_refused_with_its_rule
    - raf_okf::test_crate::records::a_valid_debt_record_validates_to_its_typed_value_and_an_empty_one_opts_out_visibly
    - raf_okf::test_crate::records::a_row_of_findings_holds_exactly_a_kind_and_a_subject_each_pair_once
    - raf_okf::test_crate::records::a_row_of_lines_holds_exactly_an_item_and_a_whole_number_of_lines_each_item_once
    - raf_okf::test_crate::records::both_lists_of_a_debt_record_are_required
    - raf_okf::test_crate::records::debt_is_one_record_at_its_one_path
    - raf_okf::test_crate::record_writer::a_debt_record_is_written_sorted_and_read_back_equal
    - raf_okf::test_crate::refusals::a_text_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_removed_policy_flag_is_rejected_before_and_after_the_command
    - cargo_workplan::test_crate::check::the_default_check_accepts_the_complete_fixture
    - cargo_workplan::test_crate::check::the_default_check_refuses_a_thin_section_without_a_policy_flag
    - cargo_workplan::test_crate::check::the_default_check_without_capability_documents_is_a_run_failure
    - cargo_workplan::test_crate::check::a_removed_policy_key_is_refused_beside_the_default_content_refusal
    - cargo_workplan::test_crate::check::policy_findings_keep_typed_locations_and_messages_in_rdf
    - cargo_workplan::test_crate::complete::the_default_completion_refuses_an_underexplained_section_and_accepts_its_neighbor
    - raf_workplan::strict::claims::tests::a_key_is_the_kebab_slug_of_its_lead_and_a_lead_is_sixty_characters
    - raf_workplan::strict::claims::tests::a_list_item_opens_at_its_marker_and_its_content_starts_after_the_space
    - raf_workplan::strict::claims::tests::a_unit_ends_on_the_line_before_the_next_unit
    - raf_okf::test_crate::tested_by::a_tested_by_marker_names_its_tests_and_maps_the_paragraph_above_it
    - raf_okf::test_crate::tested_by::a_tested_by_marker_maps_a_list_item_a_heading_or_the_statement_of_its_claim_marker
    - raf_okf::test_crate::tested_by::a_malformed_tested_by_marker_is_refused_with_its_line
    - raf_okf::test_crate::tested_by::a_tested_by_marker_with_no_statement_directly_above_is_refused
    - raf_okf::test_crate::tested_by::tested_by_markers_in_fences_code_spans_and_proposed_blocks_are_not_read
    - raf_okf::test_crate::tested_by::a_tested_by_marker_belongs_in_a_capability_document
    - raf_workplan::test_crate::claim_coverage::a_contract_is_cut_into_clauses_at_bold_leads_and_headings_and_the_text_before_is_the_opening
    - raf_workplan::test_crate::claim_coverage::a_contract_clause_covered_on_its_second_paragraph_is_covered_and_its_neighbour_is_not
    - raf_workplan::test_crate::claim_coverage::a_heading_clause_is_covered_by_a_marker_after_its_heading_and_its_text_needs_none
    - raf_workplan::test_crate::claim_coverage::constraints_are_cut_into_top_level_items_paragraphs_and_tables
    - raf_workplan::test_crate::claim_coverage::a_long_contract_with_no_clause_start_is_one_collapsed_claim_and_one_with_clauses_is_not
    - raf_workplan::test_crate::claim_coverage::an_unbound_claim_marker_is_refused_beside_covered_neighbors
    - raf_workplan::test_crate::claim_coverage::a_tested_by_marker_in_usage_names_the_bound_example_as_a_code_span_does
    - raf_workplan::test_crate::claim_coverage::fenced_generated_and_proposed_text_make_no_claim_and_evidence_does_not_cover_one
    - raf_workplan::test_crate::claim_coverage::a_bound_test_that_panics_still_covers_a_claim_without_execution
    - raf_workplan::test_crate::graph::the_graph_has_every_kind_of_edge
    - raf_workplan::test_crate::generated::a_tested_by_marker_substantiates_its_section_and_an_unmarked_binding_keeps_its_row
    - raf_workplan::test_crate::claim_coverage::the_report_lists_every_claim_of_every_delivered_capability_sorted_and_the_same_each_run
    - cargo_workplan::test_crate::claims::every_claim_names_a_test_prints_the_report_and_exits_zero
    - cargo_workplan::test_crate::claims::a_claim_that_names_no_test_is_a_finding_and_exits_one_in_text_and_rdf_alike
    - cargo_workplan::test_crate::claims::a_capability_that_is_not_loaded_exits_two_and_says_why
    - raf_workplan::test_crate::claim_coverage::a_repeated_key_takes_the_first_number_no_other_key_of_its_section_has_in_either_order
    - raf_workplan::test_crate::claim_coverage::a_thematic_break_and_an_empty_item_in_constraints_are_no_claim_and_an_item_beside_them_is_one
    - raf_workplan::test_crate::claim_coverage::a_comment_line_ends_a_constraints_paragraph_and_an_item_goes_on_only_with_its_indented_lines
    - raf_workplan::test_crate::claim_coverage::a_bold_lead_directly_after_any_heading_starts_a_contract_clause
    - cargo_workplan::test_crate::claims::the_report_sorts_by_capability_first_and_capability_prints_the_one_it_names
    - cargo_workplan::test_crate::lines::a_repository_whose_every_counted_line_is_covered_prints_the_report_and_exits_zero
    - cargo_workplan::test_crate::lines::an_orphan_line_is_a_finding_and_exits_one_in_text_and_rdf_alike_and_reverse_prints_the_same
    - cargo_workplan::test_crate::lines::a_package_that_is_not_one_exits_two_and_says_why
    - cargo_workplan::test_crate::lines::a_ceiling_row_and_its_class_decide_the_exit_of_the_default_check_on_the_binary
    - cargo_workplan::test_crate::lines::an_exception_that_matches_nothing_fails_check_under_default_and_its_matched_neighbour_passes
    - cargo_workplan::test_crate::lines::an_exception_is_the_excepted_table_and_a_lines_excepted_object_and_the_report_exits_zero
    - raf_workplan::lifecycle::touched::tests::whitespace_and_plain_comments_leave_the_stream_and_a_doc_comment_or_a_token_changes_it
    - raf_workplan::lifecycle::touched::tests::the_surface_holds_the_public_items_impl_members_by_their_header_and_trait_impl_members_of_public_types_only
    - raf_workplan::lifecycle::touched::tests::an_impl_header_is_the_tokens_before_its_brace_at_every_depth_and_a_file_that_does_not_parse_is_an_error
    - raf_workplan::lifecycle::touched::tests::the_tree_of_the_merge_base_answers_from_one_listing_and_reads_each_file_once_with_the_blob_row
    - raf_workplan::lifecycle::touched::tests::a_git_failure_while_the_merge_base_loads_is_that_error_and_a_file_that_is_not_utf8_names_the_file
    - raf_workplan::obligation::tests::the_evidence_of_the_strict_kinds_names_the_outcome_or_the_item_its_file_and_line_and_the_capability
    - raf_workplan::test_crate::obligations::a_dropped_or_moved_item_needs_a_reason_and_an_explained_outcome_is_accepted
    - raf_okf::test_crate::profile::an_item_reason_is_a_non_empty_line_beside_any_outcome_and_an_empty_one_is_refused
    - raf_workplan::test_crate::complete::a_none_change_that_edits_a_bound_signature_refuses
    - raf_workplan::test_crate::complete::an_edit_of_a_bound_body_alone_refuses
    - raf_workplan::test_crate::complete::an_edit_of_a_bound_doc_comment_refuses
    - raf_workplan::test_crate::complete::a_changed_attribute_of_a_bound_item_refuses
    - raf_workplan::test_crate::complete::a_new_bound_public_item_refuses
    - raf_workplan::test_crate::complete::an_item_renamed_below_a_bound_module_refuses
    - raf_workplan::test_crate::complete::a_reformatted_bound_body_and_a_plain_comment_complete
    - raf_workplan::test_crate::complete::a_plain_comment_outside_every_bound_item_completes
    - raf_workplan::test_crate::complete::an_item_moved_within_its_file_completes
    - raf_workplan::test_crate::complete::a_bound_item_moved_to_its_module_file_keeps_identity_but_changed_tokens_refuse
    - raf_workplan::test_crate::complete::a_none_change_repeatedly_owes_its_public_contract_obligation_until_repaired
    - raf_workplan::test_crate::complete::an_item_dropped_without_a_reason_refuses_condition_two
    - raf_workplan::test_crate::complete::an_untouched_usage_or_examples_of_a_none_change_refuses
    - raf_workplan::test_crate::complete::a_failing_bound_test_and_a_tense_mismatch_refuse_until_their_repairs_complete
    - cargo_workplan::test_crate::reconcile::an_untouched_usage_of_a_none_change_refuses_until_edited
    - cargo_workplan::test_crate::reconcile::unreadable_bound_source_at_the_merge_base_is_refused_beside_a_parsed_neighbor
    - cargo_workplan::test_crate::reconcile::an_unreadable_bound_surface_is_refused_beside_a_resolved_neighbor
    - raf_workplan::test_crate::complete::a_bound_item_made_private_or_visible_to_its_crate_alone_refuses
    - raf_workplan::test_crate::complete::a_bound_module_made_private_refuses_for_it_and_its_items
    - raf_workplan::test_crate::complete::a_public_method_of_a_bound_type_made_private_refuses
    - raf_workplan::test_crate::complete::a_changed_header_of_the_impl_block_of_a_bound_type_refuses_for_its_member
    - raf_workplan::test_crate::complete::a_method_edited_to_the_tokens_of_a_sibling_method_refuses
    - raf_workplan::test_crate::complete::a_method_removed_from_an_impl_in_another_file_or_with_its_file_refuses
    - raf_workplan::test_crate::complete::the_bound_items_of_a_file_absent_at_the_merge_base_refuse_and_a_new_file_of_private_items_completes
    - raf_workplan::test_crate::complete::a_private_helper_removed_from_a_bound_module_completes_and_a_public_item_removed_refuses
    - raf_workplan::test_crate::complete::a_trait_impl_member_of_a_bound_type_edited_or_removed_refuses_and_one_of_a_private_type_completes
    - raf_workplan::test_crate::complete::a_bound_reexport_that_names_another_item_refuses
    - raf_workplan::test_crate::complete::a_reflow_that_adds_a_trailing_comma_or_a_space_in_a_doc_comment_refuses
    - raf_workplan::test_crate::complete::an_edit_of_a_macro_that_generates_an_item_of_a_bound_module_is_not_judged
    - raf_workplan::test_crate::complete::a_bound_package_whose_surface_cannot_be_listed_is_an_error_and_without_the_second_glob_it_answers
    - raf_workplan::test_crate::complete::a_none_change_reads_each_file_of_the_bound_package_once_at_the_merge_base_and_otherwise_none
    - raf_workplan::test_crate::lines::a_crate_whose_public_surface_is_bound_and_whose_helpers_are_reached_has_no_orphan_line
    - raf_workplan::test_crate::lines::a_bound_test_reaches_its_registered_allocator_and_real_initializer_dependencies
    - raf_workplan::test_crate::lines::a_registered_allocator_is_not_a_root_of_an_unbound_target
    - raf_workplan::test_crate::lines::a_test_only_allocator_requires_a_bound_test_context_not_a_shipping_binding
    - raf_workplan::test_crate::lines::malformed_allocator_registrations_refuse_without_partial_line_results
    - raf_workplan::test_crate::lines::an_allocator_without_an_exact_indexed_static_refuses_beside_a_module_static
    - raf_workplan::test_crate::lines::every_target_is_counted_and_shared_module_units_are_unioned_once
    - raf_workplan::test_crate::lines::an_unreferenced_private_helper_is_exactly_its_span_and_the_same_helper_called_by_a_bound_function_is_not
    - raf_workplan::test_crate::lines::binding_a_type_covers_its_impl_blocks_and_their_methods_and_binding_a_method_covers_nothing_beside_it
    - raf_workplan::test_crate::lines::a_macro_invocation_is_orphan_with_its_range_and_covered_once_a_bound_function_calls_what_it_generates
    - raf_workplan::test_crate::lines::a_cfg_test_module_with_one_bound_and_one_unbound_test_reports_the_unbound_one_and_its_helper_as_test_lines
    - raf_workplan::test_crate::lines::a_package_that_no_capability_binds_is_reported_with_its_orphan_helper
    - raf_workplan::test_crate::lines::crlf_reads_as_lf_and_two_runs_give_the_same_bytes
    - raf_workplan::test_crate::lines::the_report_preserves_exact_line_counts_ranges_and_three_tables
    - raf_workplan::test_crate::lines::orphan_lines_are_enforced_without_a_policy_switch
    - raf_workplan::test_crate::lines::an_exception_by_item_prefix_file_and_directory_takes_its_lines_out_and_one_beside_it_that_matches_nothing_is_a_finding
    - raf_workplan::test_crate::lines::a_ceiling_of_lines_passes_at_its_count_and_fails_one_below_for_each_class
    - raf_workplan::test_crate::lines::a_shipping_row_alone_judges_the_shipping_lines_against_itself_and_holds_the_test_lines_to_zero
    - raf_workplan::test_crate::lines::missing_or_invalid_coverage_and_unloaded_capabilities_defer_ceiling_judgment
    - raf_workplan::test_crate::lines::a_percent_ceiling_compares_whole_numbers_at_its_boundary_and_an_empty_package_passes_it
    - raf_workplan::test_crate::lines::a_ceiling_row_of_no_package_is_scope_crate_missing_at_ceilings
    - raf_workplan::test_crate::lines::an_excepted_unit_preserves_ranges_reason_and_the_excepted_table
    - raf_workplan::test_crate::lines::a_directory_that_is_a_workspace_of_its_own_has_a_row_when_the_source_roots_reach_it_and_none_when_they_do_not
    - raf_workplan::test_crate::lines::shared_source_coverage_uses_each_packages_edition_beside_raw_names
    - raf_workplan::test_crate::lines::real_format_captures_cover_constants_while_named_bindings_and_custom_tokens_do_not
    - raf_workplan::test_crate::lines::an_impl_for_a_type_that_nothing_names_is_orphan_and_one_for_a_type_that_a_covered_unit_names_is_not
    - raf_workplan::test_crate::lines::indirect_field_dispatch_does_not_infer_a_callee_but_an_explicit_path_reaches_it
    - raf_workplan::test_crate::lines::an_excluded_item_that_nothing_covers_is_orphan_until_an_exception_names_it
    - raf_workplan::coverage::lines::tests::the_work_grows_with_the_units_and_no_faster_at_fixed_exceptions
    - raf_workplan::coverage::lines::tests::the_work_grows_with_the_exceptions_and_no_faster_at_fixed_units
    - raf_workplan::coverage::lines::tests::every_kind_of_work_is_held_to_a_ceiling_linear_in_the_units_and_the_exceptions
    - raf_workplan::coverage::lines::tests::the_default_check_sweeps_each_counted_line_unit
    - raf_workplan::coverage::lines::tests::a_percent_is_compared_as_whole_numbers_without_division_or_overflow
    - raf_workplan::coverage::lines::tests::a_path_that_may_name_what_a_macro_generates_is_a_run_and_a_method_a_field_or_a_declared_name_is_not
    - raf_workplan::coverage::lines::tests::a_walk_that_fails_is_the_one_error_of_the_report_and_one_finding_under_strict
    - raf_workplan::coverage::lines::tests::a_walk_that_fails_retains_a_single_typed_error_and_the_error_table
    - raf_workplan::test_crate::debt::without_a_record_every_strict_finding_stands_and_the_write_creates_it
    - raf_workplan::test_crate::debt::a_repository_that_passes_has_no_finding_to_hold_and_the_write_records_nothing
    - raf_workplan::test_crate::debt::a_new_strict_finding_fails_and_the_write_refuses_and_writes_nothing
    - raf_workplan::test_crate::debt::a_paid_finding_fails_until_the_write_and_then_the_record_equals_the_present_debt
    - raf_workplan::test_crate::debt::rewording_a_recorded_constraint_is_growth_beside_its_resolved_old_key
    - raf_workplan::test_crate::debt::two_writes_are_byte_identical_and_the_second_leaves_the_file_untouched
    - raf_workplan::test_crate::debt::a_finding_of_the_default_rules_stops_the_write
    - raf_workplan::test_crate::debt::the_default_gate_records_present_debt_without_a_policy_switch
    - raf_workplan::test_crate::debt::an_empty_debt_record_is_judged_without_a_policy_switch
    - raf_workplan::test_crate::debt::a_row_of_a_kind_that_is_no_debt_or_a_subject_of_another_form_is_invalid
    - raf_workplan::test_crate::debt::a_row_of_lines_that_names_no_item_or_no_line_is_invalid
    - raf_workplan::test_crate::debt::an_invalid_row_holds_nothing_so_the_write_replaces_it
    - raf_workplan::test_crate::debt::a_file_other_than_the_one_record_is_a_finding
    - raf_workplan::test_crate::debt::a_record_that_breaks_the_profile_is_the_finding_of_its_rule_and_stops_the_write
    - raf_workplan::test_crate::debt::orphan_lines_over_a_ceiling_are_held_per_item_as_a_count
    - raf_workplan::test_crate::debt::an_item_that_holds_more_lines_than_its_row_fails_naming_the_row_and_the_write_refuses
    - raf_workplan::test_crate::debt::an_orphan_item_with_no_row_is_growth_in_a_package_judged_by_its_rows
    - raf_workplan::test_crate::debt::a_package_over_a_ceiling_that_the_record_does_not_name_is_growth_not_adoption
    - raf_workplan::test_crate::debt::a_falling_count_fails_until_the_write_lowers_the_row
    - raf_workplan::test_crate::debt::an_item_that_is_gone_is_resolved_and_the_write_drops_its_row
    - raf_workplan::test_crate::debt::a_count_back_under_the_ceiling_is_resolved_and_a_package_within_it_holds_no_debt
    - raf_workplan::test_crate::debt::the_record_keeps_its_spdx_lines_and_a_new_one_takes_those_it_is_given
    - raf_workplan::test_crate::debt::every_error_of_the_write_says_what_to_do
    - raf_workplan::test_crate::debt::the_debt_kinds_are_kinds_of_strict_mode_and_the_ten_rules_that_a_row_can_hold
    - raf_workplan::test_crate::debt::the_record_is_read_back_as_the_typed_value_the_write_built
    - raf_workplan::test_crate::debt::the_default_debt_writer_judges_all_present_findings
    - raf_workplan::test_crate::debt::the_snapshot_of_every_other_file_is_unchanged_by_a_refused_write
    - raf_workplan::debt::tests::a_report_that_could_not_count_is_one_finding_no_row_holds_and_resolves_no_row
    - cargo_workplan::test_crate::debt::the_write_creates_the_record_exits_zero_and_the_repository_then_passes
    - cargo_workplan::test_crate::debt::a_second_write_is_unchanged_and_the_bytes_are_the_same
    - cargo_workplan::test_crate::debt::the_written_record_in_rdf_is_one_object_with_its_members_in_order
    - cargo_workplan::test_crate::debt::debt_that_grew_is_exit_one_with_its_findings_and_nothing_is_written
    - cargo_workplan::test_crate::debt::paid_debt_fails_the_check_with_debt_resolved_until_the_write_lowers_the_record
    - cargo_workplan::test_crate::debt::a_finding_of_the_default_rules_stops_the_write_with_exit_one
    - cargo_workplan::test_crate::debt::the_default_policy_can_record_a_current_structural_finding
    - cargo_workplan::test_crate::debt::orphan_lines_over_the_ceiling_are_recorded_per_item
    - cargo_workplan::test_crate::debt::the_spdx_lines_are_given_together_and_kept_when_the_record_is_lowered
    - cargo_workplan::test_crate::debt::a_command_misused_is_a_usage_error_before_any_work
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::debt_takes_write_and_the_spdx_pair_together_or_not_at_all
    - raf_workplan::test_crate::debt::every_kind_a_row_can_hold_is_written_with_a_subject_of_its_form_and_then_holds
    - raf_workplan::test_crate::debt::an_invalid_debt_row_is_refused_without_a_policy_selector
    - raf_workplan::test_crate::complete::the_check_once_the_record_is_removed_applies_the_debt_record_so_a_held_finding_does_not_refuse
    - raf_workplan::test_crate::complete::the_check_once_the_record_is_removed_refuses_a_debt_row_that_is_resolved
  examples:
    - raf_workplan::Workplan::check
depends: [cap:workplan-two-way-coverage, cap:workplan-complete-gate, cap:workplan-reconcile, cap:workplan-evidence-claims, cap:workplan-generated-docs, cap:source-index]
---

# Complete workplan policy

## Purpose

The complete workplan policy connects documented behavior to runnable examples,
claim-level tests, explained source items and counted lines. Every check enforces
that policy, so a delivered capability cannot omit proof or leave source debt
unreported. Completion also checks outcome reasons and token-level contract effects.

## Contract

`write_debt_in` judges the loaded `Workplan` and publishes through an injected
`RecordStore`, including a native `MemoryTree`. The disk `Gate`, `Opened` and
command-line conveniences use this same judgment and publication path. Adoption
creates the record exclusively, shrinking debt replaces it, and unchanged debt
performs no write. Growth and blocking findings refuse without changing the
store. The `record` boundary preserves paired SPDX handling and original write
causes. Reload the model from the store before judging another write.
<!-- raf:tested-by raf_workplan::test_crate::debt::two_writes_are_byte_identical_and_the_second_leaves_the_file_untouched,raf_workplan::test_crate::debt::a_paid_finding_fails_until_the_write_and_then_the_record_equals_the_present_debt,raf_workplan::test_crate::debt::a_new_strict_finding_fails_and_the_write_refuses_and_writes_nothing -->

**One policy.** Every `Workplan::check`, `Gate` caller and command judges the
complete policy. The closed coverage profile rejects `strictness`; the library
exposes no policy enum or floor, and the command parser rejects `--strict`. Loading
documents does not judge policy units, while every complete check does. The
completion check after record removal applies the same policy at both judgements.
Content and binding findings follow document-model and generated-block findings,
then the debt ratchet applies.
<!-- raf:tested-by raf_workplan::strict::tests::the_default_check_judges_each_policy_unit_once_without_policy_work_during_load,raf_workplan::test_crate::strict::every_policy_refusal_case_is_enforced_by_the_default_check,raf_workplan::test_crate::strict::content_and_binding_findings_follow_load_findings,raf_workplan::test_crate::strict::the_removed_policy_key_is_rejected_even_for_its_former_values,raf_workplan::test_crate::strict::the_default_gate_judges_the_complete_policy_in_its_opened_workplan,raf_workplan::test_crate::complete::completion_after_record_removal_enforces_the_complete_policy,cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_removed_policy_flag_is_rejected_before_and_after_the_command,raf_workplan::test_crate::debt::the_default_debt_writer_judges_all_present_findings -->

**Examples.** A `delivered` capability binds at least
one entry in `bindings.examples`; with none it is `strict-example-missing` at the key
`bindings`.
<!-- raf:tested-by raf_workplan::test_crate::strict::a_capability_with_no_example_is_refused_and_one_with_one_example_is_not -->

**Verification matrix.** A delivered capability holds a `verification-matrix`
generated block in its Verification section; without one it is
`strict-matrix-missing` at that section.
<!-- raf:tested-by raf_workplan::test_crate::strict::verification_without_the_matrix_is_refused_and_with_the_block_is_not -->

**Runnable example proof.** That the bound example runs is proven by `verify` and condition 5
of the [completion gate](./workplan-complete-gate.md), which refuse a binding whose
only documentation test is `no_run` or `compile_fail`; the complete policy adds no second
run.
<!-- raf:tested-by raf_workplan::test_crate::bound::a_no_run_example_meets_the_strict_example_rule_and_verify_refuses_it_beside_a_runnable_one -->

**Claim units.** A claim is a unit of the authored prose of the Contract and the
Constraints of a `delivered` capability, read outside fenced code, generated blocks
and PROPOSED blocks:

- in the Contract, a clause: it starts at a paragraph whose first line begins, at
  the margin, with a bold run (`**The setting.**`), or at a `###` or `####` heading
  outside fenced code, and runs to the start of the next clause or the end of the
  section. Text before the first clause start is one clause, the opening. The
  statement of a heading clause is its heading line;
- in the Constraints, each top-level list item, with the items, paragraphs and
  fences indented under it, and each paragraph or table outside a list. A thematic
  break such as `* * *` and a list item with nothing after its marker state nothing
  and are no claim.
<!-- raf:tested-by raf_workplan::test_crate::claim_coverage::fenced_generated_and_proposed_text_make_no_claim_and_evidence_does_not_cover_one,raf_workplan::test_crate::claim_coverage::a_contract_is_cut_into_clauses_at_bold_leads_and_headings_and_the_text_before_is_the_opening,raf_workplan::test_crate::claim_coverage::constraints_are_cut_into_top_level_items_paragraphs_and_tables,raf_workplan::test_crate::claim_coverage::a_thematic_break_and_an_empty_item_in_constraints_are_no_claim_and_an_item_beside_them_is_one -->

**Claim boundaries.** A blank line, a comment line, a heading, a fence and a PROPOSED or generated block
each end a paragraph, so a bold lead directly after a heading of any level starts a
clause and the paragraph after a marker line is a claim of its own; a list item goes
on after a blank or a comment line only with the lines indented to its content.
<!-- raf:tested-by raf_workplan::test_crate::claim_coverage::a_comment_line_ends_a_constraints_paragraph_and_an_item_goes_on_only_with_its_indented_lines,raf_workplan::test_crate::claim_coverage::a_bold_lead_directly_after_any_heading_starts_a_contract_clause -->

**Claim keys.** Each claim has a key, `cap:<slug>#<section>/<key>`
(`ClaimCoverage::key`): the kebab slug of the bold lead or the heading, `opening`
for the opening, and for a Constraints unit `c-` and the first 16 hexadecimal
digits of the SHA-256 digest of its text, with its comments left out and each run of
whitespace read as one space. A key that repeats in a section takes the first of
`-2`, `-3` and on that no other claim of the section has or takes, in document order,
so the keys of a section are distinct: `**Foo.**`, `**Foo.**` and `**Foo 2.**` give
`foo`, `foo-3` and `foo-2`.
<!-- raf:tested-by raf_workplan::strict::claims::tests::a_key_is_the_kebab_slug_of_its_lead_and_a_lead_is_sixty_characters,raf_workplan::test_crate::claim_coverage::constraints_are_cut_into_top_level_items_paragraphs_and_tables,raf_workplan::test_crate::claim_coverage::a_repeated_key_takes_the_first_number_no_other_key_of_its_section_has_in_either_order -->

**Test marker syntax.** A test maps to a claim through a marker on its own line directly after the statement,
or directly after the statement's claim marker:
`<!-- raf:tested-by <path>[,<path>...] -->`, with one space after the word and the
entries separated by `,` and no space. Each entry matches the `<item-path>` pattern of
the completion gate, and no entry repeats. The marker is a third family beside the
generated and the claim markers: [the OKF reader](./okf-profile.md) reads it as
`Body::tested_by`, a malformed one is `okf-parse-malformed-tested-by`, one with no
statement directly above it, or directly before a claim marker, is
`okf-parse-unattached-tested-by`, and one outside a Capability document is
`okf-tested-by-outside-capability`.
<!-- raf:tested-by raf_okf::test_crate::tested_by::a_tested_by_marker_names_its_tests_and_maps_the_paragraph_above_it,raf_okf::test_crate::tested_by::a_tested_by_marker_maps_a_list_item_a_heading_or_the_statement_of_its_claim_marker,raf_okf::test_crate::tested_by::a_malformed_tested_by_marker_is_refused_with_its_line,raf_okf::test_crate::tested_by::a_tested_by_marker_with_no_statement_directly_above_is_refused,raf_okf::test_crate::tested_by::a_tested_by_marker_belongs_in_a_capability_document -->

**Test marker ownership.** On every check, an entry that is no path
of the same capability's `bindings.tests` or `bindings.examples` is
`claim-test-unbound` at the marker's line, and each entry that is one is a
`TestedByEdge` of the graph.
<!-- raf:tested-by raf_workplan::test_crate::claim_coverage::an_unbound_claim_marker_is_refused_beside_covered_neighbors,raf_workplan::test_crate::graph::the_graph_has_every_kind_of_edge -->

**Claim coverage.** A Contract clause is covered when it holds a marker on
any of its lines; a heading clause is covered by a marker directly after its heading.
A Constraints unit is covered by a marker whose statement starts in it. An evidence
claim marker covers nothing here.
<!-- raf:tested-by raf_workplan::test_crate::claim_coverage::fenced_generated_and_proposed_text_make_no_claim_and_evidence_does_not_cover_one,raf_workplan::test_crate::claim_coverage::a_contract_clause_covered_on_its_second_paragraph_is_covered_and_its_neighbour_is_not,raf_workplan::test_crate::claim_coverage::a_heading_clause_is_covered_by_a_marker_after_its_heading_and_its_text_needs_none,raf_workplan::test_crate::claim_coverage::constraints_are_cut_into_top_level_items_paragraphs_and_tables -->

**Uncovered and collapsed claims.** Each claim that no marker covers is `strict-claim-uncovered` at the
first line of the claim, with a message that names its key and the first 60
characters of its lead or item. A Contract with no clause start is one claim; when it
holds more than `strict::MIN_SECTION_WORDS` words it is `strict-claim-unit-collapsed`
at the Contract, whether or not a marker covers it, so one marker cannot stand for a
long Contract.
<!-- raf:tested-by raf_workplan::test_crate::claim_coverage::a_contract_clause_covered_on_its_second_paragraph_is_covered_and_its_neighbour_is_not,raf_workplan::test_crate::claim_coverage::a_long_contract_with_no_clause_start_is_one_collapsed_claim_and_one_with_clauses_is_not -->

**Claim report.** `cargo workplan coverage claims [--capability cap:<slug>]` and
`Workplan::claims_report` (`strict::ClaimsReport`) list every claim of every delivered
capability with its key, line and tests, on every check, sorted by
capability, the slug of the section and line, as a table with the columns
Capability, Section, Claim, Line and Tested by (`coverage::render::claims_table`),
and in the typed Claims RDF results of the [shared report profile](./rdf-report-profile.md).
The command exits 1 when a claim of a delivered capability has no test, 0 when every
claim has one, and 2 when `--capability` names no loaded capability.
<!-- raf:tested-by raf_workplan::test_crate::claim_coverage::the_report_lists_every_claim_of_every_delivered_capability_sorted_and_the_same_each_run,cargo_workplan::test_crate::claims::every_claim_names_a_test_prints_the_report_and_exits_zero,cargo_workplan::test_crate::claims::a_claim_that_names_no_test_is_a_finding_and_exits_one_in_text_and_rdf_alike,cargo_workplan::test_crate::claims::a_capability_that_is_not_loaded_exits_two_and_says_why -->

**Usage substantiation.** A binding that
a marker names substantiates the section that holds the marker in the verification
matrix, as a code span of its path does ([generated documents](./workplan-generated-docs.md)).
<!-- raf:tested-by raf_workplan::test_crate::claim_coverage::a_tested_by_marker_in_usage_names_the_bound_example_as_a_code_span_does,raf_workplan::test_crate::generated::a_tested_by_marker_substantiates_its_section_and_an_unmarked_binding_keeps_its_row -->

**Explained items.** the code and the tests directions of
[two-way coverage](./workplan-two-way-coverage.md) are enforced for every package
that holds an item that a `code`, `tests` or `examples` binding of a loaded capability
resolves to, beside the packages of `scope`. A gap of such a package has the kind the
default rule gives it, `coverage-item-unbound`, `coverage-reexport-unbound`,
`coverage-test-unbound` or `coverage-surface-unreadable`, and its message says which
capability binds which path of the package, the reason the complete policy enforces it; a
surface that cannot be listed is reported at the key `bindings` of that capability,
and the exclusions of the coverage record apply as they do to `scope`. A repository
whose coverage record is missing or does not load has no gap of this rule, as it has
none of `scope`: the record's own finding fails the check. A
package that no binding reaches and `scope` does not name is not enforced by this
rule. The complete check combines explicit-scope enforcement with binding-reached
packages; neither reason provides a policy opt-out.
<!-- raf:tested-by raf_workplan::test_crate::strict::a_binding_reached_package_is_enforced_and_an_unreached_one_is_not,raf_workplan::test_crate::strict::a_scoped_package_is_enforced_once_with_a_binding_present,raf_workplan::test_crate::strict::an_exclusion_of_the_record_takes_a_gap_of_a_bound_package_out,raf_workplan::test_crate::strict::a_missing_or_invalid_coverage_record_is_reported_before_package_gaps,raf_workplan::test_crate::strict::while_a_capability_document_does_not_load_no_gap_of_a_bound_package_is_reported -->

**Completion.** The complete policy keeps the six conditions of the completion gate:
condition 5 runs every test and example
binding of every listed capability with the proof `verify` uses, and condition 1
refuses on each `tense-mismatch` of a listed capability. The list also holds
`contract-effect-understated`, raised by [reconcile](./workplan-reconcile.md) when the
record says `contract_effect: none` and a bound public item is touched.
<!-- raf:tested-by raf_workplan::test_crate::complete::a_failing_bound_test_and_a_tense_mismatch_refuse_until_their_repairs_complete,raf_workplan::test_crate::complete::a_none_change_repeatedly_owes_its_public_contract_obligation_until_repaired -->

**Compared public surface.** Each side is read from a source index: the working tree's, and one loaded
at the merge base from the directories of the packages that hold an item a `code`
binding resolves to. Its units are the items of the public surface of every library
and binary target of those packages (the items another crate can name, the `pub`
members of the inherent impls of an exported type and the members of an exported
trait; an item `#[doc(hidden)]` hides is not one) and the members of the trait impls
of an exported struct, enum or union. A unit is bound when a `code` binding of any
loaded capability covers it, itself or as an item below it in the sense of two-way
coverage.
<!-- raf:tested-by raf_workplan::lifecycle::touched::tests::the_surface_holds_the_public_items_impl_members_by_their_header_and_trait_impl_members_of_public_types_only -->

**Unit identity and tokens.** A unit is the same unit on both sides when its path, its kind and, for a
member of an impl block, the tokens of the impl's header (its attributes, generics,
trait, self type and where clause) are the same, and each unit is compared with that
counterpart alone, as the `proc_macro2` token stream of its whole span. A doc comment
arrives as a `#[doc]` attribute and counts; whitespace, line breaks, comments that
are not doc comments and CRLF line endings are not tokens and do not. So an edit of a
signature, a body, an attribute, a visibility or a doc comment touches a unit, and so
does a reformatting that adds or removes a token, such as the trailing comma rustfmt
adds to one parameter per line, while a reformatting that keeps the tokens and an
edit of a plain comment do not. The span of an inline module holds its items, so any
token edited inside a bound inline module touches the module too.
<!-- raf:tested-by raf_workplan::lifecycle::touched::tests::whitespace_and_plain_comments_leave_the_stream_and_a_doc_comment_or_a_token_changes_it,raf_workplan::lifecycle::touched::tests::an_impl_header_is_the_tokens_before_its_brace_at_every_depth_and_a_file_that_does_not_parse_is_an_error,raf_workplan::test_crate::complete::a_none_change_that_edits_a_bound_signature_refuses,raf_workplan::test_crate::complete::an_edit_of_a_bound_body_alone_refuses,raf_workplan::test_crate::complete::an_edit_of_a_bound_doc_comment_refuses,raf_workplan::test_crate::complete::a_changed_attribute_of_a_bound_item_refuses -->

**Touch classes.** A bound unit is
touched (`obligation::Touch`) when:

- its counterpart has another stream: `Tokens`;
- it has no counterpart: `New`, for a new item, one made public, one given a new
  logical path (moved to another module or renamed), or one under a changed impl
  header. A newly introduced public item in a new source file has no counterpart;
- it is a unit of the merge base with no counterpart: `Hidden` when the working tree
  still has an item of its path and kind off the surface (made private or
  `pub(crate)`, or behind a module made private), and `Removed` otherwise, whichever
  file or module held it, a file the change deletes included;
- it is named by a re-export path of the surface, which names another unit than at
  the merge base (`Retargeted`), is new (`New`) or is gone (`Removed`).
<!-- raf:tested-by raf_workplan::test_crate::complete::a_new_bound_public_item_refuses,raf_workplan::test_crate::complete::an_item_renamed_below_a_bound_module_refuses,raf_workplan::test_crate::complete::a_bound_item_made_private_or_visible_to_its_crate_alone_refuses,raf_workplan::test_crate::complete::a_public_method_of_a_bound_type_made_private_refuses,raf_workplan::test_crate::complete::a_changed_header_of_the_impl_block_of_a_bound_type_refuses_for_its_member,raf_workplan::test_crate::complete::a_method_removed_from_an_impl_in_another_file_or_with_its_file_refuses,raf_workplan::test_crate::complete::the_bound_items_of_a_file_absent_at_the_merge_base_refuse_and_a_new_file_of_private_items_completes,raf_workplan::test_crate::complete::a_bound_reexport_that_names_another_item_refuses -->

**Untouched units and obligations.** A unit that keeps its path, its kind, its header and its tokens is untouched wherever
it moves, so two items reordered or an item moved within its module complete. Each
touched path is one obligation on the change record, reported once, whose evidence
names the item, its file and the line of its name and the capability whose binding
covers it; the evidence needed is `contract_effect: changes`, the capability listed
and the sections it owes reconciled.
<!-- raf:tested-by raf_workplan::test_crate::complete::a_bound_item_moved_to_its_module_file_keeps_identity_but_changed_tokens_refuse,raf_workplan::test_crate::complete::an_item_moved_within_its_file_completes,raf_workplan::test_crate::complete::a_reformatted_bound_body_and_a_plain_comment_complete,raf_workplan::test_crate::complete::a_none_change_repeatedly_owes_its_public_contract_obligation_until_repaired,raf_workplan::obligation::tests::the_evidence_of_the_strict_kinds_names_the_outcome_or_the_item_its_file_and_line_and_the_capability -->

**Merge-base source loading.** The index at the merge base is loaded through a
tree of the files of the merge base: `git ls-tree -r -z --full-tree <merge-base>`
lists them once, and each file the load reads is read once with the reconcile row `git
cat-file blob <merge-base>:<prefix><path>`, within the 1 MiB bound of a blob, so the
reads are one for each manifest and source file of the packages a `code` binding
reaches. `change complete` judges twice, before and after the bound runs, and asks for
the merge base each time, but loads its index once while the merge base and the
packages are the same.
<!-- raf:tested-by raf_workplan::lifecycle::touched::tests::the_tree_of_the_merge_base_answers_from_one_listing_and_reads_each_file_once_with_the_blob_row,raf_workplan::test_crate::complete::a_none_change_reads_each_file_of_the_bound_package_once_at_the_merge_base_and_otherwise_none -->

**Unreadable compared surfaces.** A merge base whose index does not load (a file that is not
UTF-8 or does not parse, a missing module) is an error and not a pass, as is a public
surface a source index cannot list (`LifecycleError::Surface`, which names the side),
and `change reconcile` and `change complete` exit 2 for either. Condition 1 refuses on
the obligation, and the headline counts it as an obligation.
<!-- raf:tested-by raf_workplan::lifecycle::touched::tests::a_git_failure_while_the_merge_base_loads_is_that_error_and_a_file_that_is_not_utf8_names_the_file,raf_workplan::test_crate::complete::a_bound_package_whose_surface_cannot_be_listed_is_an_error_and_without_the_second_glob_it_answers,cargo_workplan::test_crate::reconcile::unreadable_bound_source_at_the_merge_base_is_refused_beside_a_parsed_neighbor,cargo_workplan::test_crate::reconcile::an_unreadable_bound_surface_is_refused_beside_a_resolved_neighbor -->

**Section prose.** Each of the seven sections of a `delivered` capability
holds at least `strict::MIN_SECTION_WORDS`, 20, words of prose: the prose lines
outside PROPOSED and generated blocks, cut into words as the tense scan cuts them,
runs of letters, digits and `_`. A shorter section is `strict-section-thin` at the
section, and its fix says that a generated block does not count as prose.
<!-- raf:tested-by raf_workplan::test_crate::strict::a_section_of_nineteen_words_is_thin_and_one_of_twenty_is_not,raf_workplan::test_crate::strict::code_spans_fenced_code_comments_and_the_heading_are_not_prose,raf_workplan::test_crate::strict::the_words_of_a_generated_block_are_not_prose_of_its_section,raf_workplan::test_crate::strict::a_capability_that_is_not_delivered_has_no_finding_of_the_content_rules -->

**Usage examples.** The Usage section names a path of `bindings.examples` in a code span or in a tested-by marker,
so the verification matrix lists Usage for an example that `verify` runs; otherwise
it is `strict-usage-unbound` at Usage. A code span inside an HTML comment names
nothing.
<!-- raf:tested-by raf_workplan::test_crate::strict::usage_that_names_no_bound_example_is_refused_and_a_code_span_of_it_is_not,raf_workplan::test_crate::strict::a_code_span_inside_a_comment_names_no_example_in_usage,raf_workplan::test_crate::claim_coverage::a_tested_by_marker_in_usage_names_the_bound_example_as_a_code_span_does -->

**Stated limitations.** The Constraints section holds at least one list item, one limitation, with
text or a code span after its marker; an empty item and a thematic break such as
`* * *` are none. Otherwise it is `strict-limitations-missing` at Constraints.
<!-- raf:tested-by raf_workplan::test_crate::strict::constraints_without_a_list_item_are_refused_and_one_list_item_is_not,raf_workplan::test_crate::strict::a_thematic_break_and_an_empty_item_are_no_limitation -->

**Reasons.** A change item takes an optional key `reason` after `outcome`, a
non-empty line of text, accepted beside any outcome or none; an empty one, or one that
is not text, is `okf-item-reason-empty`. an item whose outcome is
`dropped` or `moved:change:<slug>` and that has no `reason` is the obligation
`outcome-without-reason` on that item, whether or not a `moved` target exists, and
condition 2 of the completion gate refuses on it as it refuses on
`item-without-outcome`; the headline of the refusal counts it as an item without a
reason. Both obligations apply to every complete reconciliation and completion.
<!-- raf:tested-by raf_okf::test_crate::profile::an_item_reason_is_a_non_empty_line_beside_any_outcome_and_an_empty_one_is_refused,raf_workplan::test_crate::obligations::a_dropped_or_moved_item_needs_a_reason_and_an_explained_outcome_is_accepted,raf_workplan::test_crate::complete::an_item_dropped_without_a_reason_refuses_condition_two -->

**Untouched sections.** `contract_effect: none` leaves out
`section-untouched` for the Contract only: a Usage or an Examples section that the
record lists in `affects` and that has the text of the merge base is owed, as under
`contract_effect: changes`.
<!-- raf:tested-by raf_workplan::test_crate::complete::an_untouched_usage_or_examples_of_a_none_change_refuses,cargo_workplan::test_crate::reconcile::an_untouched_usage_of_a_none_change_refuses_until_edited -->

**Lines.** `cargo workplan coverage lines [--crate <package>]` and
`Workplan::lines_report` (`coverage::lines::LinesReport`) give the reverse coverage
of every package the loader reaches, the root package and packages that no
capability binds included, on every check. `cargo workplan coverage
--reverse [--crate <package>]` is the same command and prints the same bytes.
<!-- raf:tested-by raf_workplan::test_crate::lines::a_package_that_no_capability_binds_is_reported_with_its_orphan_helper,raf_workplan::test_crate::lines::a_directory_that_is_a_workspace_of_its_own_has_a_row_when_the_source_roots_reach_it_and_none_when_they_do_not,cargo_workplan::test_crate::lines::an_orphan_line_is_a_finding_and_exits_one_in_text_and_rdf_alike_and_reverse_prints_the_same -->

**Counted source units.** The units of a package are those of every target the [source index](./source-index.md)
lists (library, binaries, integration tests, examples, benches and build script):
every indexed item except a module and an `extern crate`, every `impl` block from its first
attribute to its closing brace, and every macro invocation and `const _` at the item
level of a module, which a parse of the file finds. The counted lines of a file are
the union of the lines of its units, and a file is counted once, for the package of
the first target that reaches it, however many targets and module paths reach it.
Other lines (blank lines, `use` and `mod` declarations, inner doc comments, the braces
of an inline module and the SPDX header) are structural: neither covered nor orphan,
and not counted.
<!-- raf:tested-by raf_workplan::test_crate::lines::every_target_is_counted_and_shared_module_units_are_unioned_once,raf_workplan::test_crate::lines::a_crate_whose_public_surface_is_bound_and_whose_helpers_are_reached_has_no_orphan_line,raf_workplan::test_crate::lines::binding_a_type_covers_its_impl_blocks_and_their_methods_and_binding_a_method_covers_nothing_beside_it,raf_workplan::test_crate::lines::an_unreferenced_private_helper_is_exactly_its_span_and_the_same_helper_called_by_a_bound_function_is_not -->

**Coverage roots and closure.** A line is covered when a covered unit holds it, and a unit is
covered when any item or impl block of its source is in the closure that
`Workspace::reachable` gives from the roots:

1. the roots are the items that a `code`, `tests` or `examples` binding of a loaded
   capability resolves to, and for an `examples` binding of the crate root of an
   example target every item and impl block of that target;
2. contains: the members of a covered type or trait, and the items and impl blocks
   of a bound module and of the modules below it; a crate root holds nothing;
3. impl-of: the impl blocks whose self type is a covered struct, enum or union, and
   the members of a covered block;
4. references: the items that a path of a covered unit's own tokens names, from the
   module of that unit. The paths are the `Ident (:: Ident)*` runs that no `.`
   precedes, and every prefix of each; a path that names nothing of the workspace
   reaches nothing, and a module a path names reaches nothing below it;
5. a macro invocation or a `const _` is covered when one of its module paths is a
   bound module or below one, or its target is an example target that an
   `examples` binding names; an invocation is also covered when a covered unit names
   a path that resolves to nothing from that unit's module and whose last segment
   is an identifier inside the invocation. The `macro_rules!` that a covered
   invocation invokes, read from its module and, for a name of one segment, from
   each module above it, is then a root, and the closure is taken again until no
   invocation is newly covered.
<!-- raf:tested-by raf_workplan::test_crate::lines::every_target_is_counted_and_shared_module_units_are_unioned_once,raf_workplan::test_crate::lines::a_crate_whose_public_surface_is_bound_and_whose_helpers_are_reached_has_no_orphan_line,raf_workplan::test_crate::lines::binding_a_type_covers_its_impl_blocks_and_their_methods_and_binding_a_method_covers_nothing_beside_it,raf_workplan::test_crate::lines::a_macro_invocation_is_orphan_with_its_range_and_covered_once_a_bound_function_calls_what_it_generates -->

**Implicit format captures.** The source index also supplies decoded implicit value and count captures from
positively identified standard `format!` calls as reference edges. Such a capture
covers the named constant's lines through the actual owning function. Explicit
named bindings cover their expressions rather than a same-named implicit value;
custom token macros add no inferred format captures. A source refusal from a
recognized malformed format call remains a counting error of this report.
<!-- raf:tested-by raf_workplan::test_crate::lines::real_format_captures_cover_constants_while_named_bindings_and_custom_tokens_do_not -->

**Compiler allocator registration.** A directly written `#[global_allocator]`
static is an additional root when its indexed target participates in the bound
roots. A test-only static requires a participating test context; it is never
reclassified as shipping code. The static's real initializer dependencies follow
the ordinary source closure. An allocator in an unbound target adds no root.
The existing bounded source parse supplies exact static spans, matched to the
index before any line is judged.
<!-- raf:tested-by raf_workplan::test_crate::lines::a_bound_test_reaches_its_registered_allocator_and_real_initializer_dependencies,raf_workplan::test_crate::lines::a_registered_allocator_is_not_a_root_of_an_unbound_target,raf_workplan::test_crate::lines::a_test_only_allocator_requires_a_bound_test_context_not_a_shipping_binding -->

**Orphan units and line classes.** Every other counted line is an orphan line, listed once, under the innermost unit
that holds it. An item a macro generates has no lines of its own: it is counted as
the span of the invocation that generates it. A line is a test line when that unit
is test code (a `#[cfg(test)]` module, a `#[test]` function) or is in a target that
does not ship (integration tests, benches, examples), and a shipping line
otherwise. `cfg` is never evaluated, so the report does not depend on the host.
<!-- raf:tested-by raf_workplan::test_crate::lines::every_target_is_counted_and_shared_module_units_are_unioned_once,raf_workplan::test_crate::lines::a_cfg_test_module_with_one_bound_and_one_unbound_test_reports_the_unbound_one_and_its_helper_as_test_lines,raf_workplan::test_crate::lines::a_macro_invocation_is_orphan_with_its_range_and_covered_once_a_bound_function_calls_what_it_generates,raf_workplan::test_crate::lines::binding_a_type_covers_its_impl_blocks_and_their_methods_and_binding_a_method_covers_nothing_beside_it -->

**Line report projection.** The report prints three tables. The crate table has the columns Crate,
Counted, Covered, Excepted, Orphan, Shipping orphan, Test
orphan, Ceiling and Debt; the module table, Crate, Module, Counted and Orphan, has a
row for each module that holds an orphan line; the item table Crate, Item, Kind, File
and Orphan lines, with merged `a-b` ranges (`coverage::render::lines_table`). When
there are any, a table of the lines each exception takes out of each unit follows,
with the exception and its reason, and a table of the reasons the lines could not be
counted. Rows are sorted by package, module path, file and first line. The typed
Lines RDF results retain package totals, ordered range rows, exception reasons
and counting errors through the [shared report profile](./rdf-report-profile.md).
Budget and storage refusals in generated-name and invoked-macro lookups are
counting errors. They stop the report before totals or orphan rows are published;
a refused macro lookup does not continue into parent scopes. The error identifies
the queried path or macro, its module and the source refusal. A genuine missing
name can still identify generated code.
<!-- raf:tested-by raf_workplan::test_crate::lines::the_report_preserves_exact_line_counts_ranges_and_three_tables,raf_workplan::test_crate::lines::an_excepted_unit_preserves_ranges_reason_and_the_excepted_table,raf_workplan::coverage::lines::tests::a_walk_that_fails_retains_a_single_typed_error_and_the_error_table,cargo_workplan::test_crate::lines::an_orphan_line_is_a_finding_and_exits_one_in_text_and_rdf_alike_and_reverse_prints_the_same,cargo_workplan::test_crate::lines::an_exception_is_the_excepted_table_and_a_lines_excepted_object_and_the_report_exits_zero -->

**Stable line reports and command outcomes.** The report holds no absolute path and no order that depends on the
host, and a file with CRLF line endings reads as the same file with LF. The command
exits 1 when it prints an orphan line or an error, 0 when it prints neither, and 2
when `--crate` names no package.
<!-- raf:tested-by raf_workplan::test_crate::lines::crlf_reads_as_lf_and_two_runs_give_the_same_bytes,cargo_workplan::test_crate::lines::a_repository_whose_every_counted_line_is_covered_prints_the_report_and_exits_zero,cargo_workplan::test_crate::lines::an_orphan_line_is_a_finding_and_exits_one_in_text_and_rdf_alike_and_reverse_prints_the_same,cargo_workplan::test_crate::lines::a_package_that_is_not_one_exits_two_and_says_why -->

**Exception targets.** The coverage record takes two more optional keys. Each
row of `exceptions` holds exactly one of `item`, an item path or a prefix
`<module>::*` below a crate root, or `file`, a repository-relative path or a
directory ending in `/`, and a non-empty `reason`. A line that no covered unit holds
is excepted when an exception matches a unit that holds it, or its file: an `item`
the unit at that path and every unit below it (the members of a type, the items of
a module, the impl blocks of a type and of a module), a prefix every unit below its
module, a `file` that file and a directory every file below it. Excepted lines are
reported with the reason of the first exception in the order of the record that
matches them, and they are not orphan lines.
<!-- raf:tested-by raf_okf::test_crate::records::an_exception_names_one_item_prefix_file_or_directory_and_its_reason,raf_workplan::test_crate::lines::an_exception_by_item_prefix_file_and_directory_takes_its_lines_out_and_one_beside_it_that_matches_nothing_is_a_finding,raf_workplan::test_crate::lines::an_excepted_unit_preserves_ranges_reason_and_the_excepted_table -->

**Exception lookup work.** The exceptions are put into an index
once and each unit looks up its own paths and the modules above them, so the work
grows with the units plus the exceptions.
<!-- raf:tested-by raf_workplan::coverage::lines::tests::the_work_grows_with_the_units_and_no_faster_at_fixed_exceptions,raf_workplan::coverage::lines::tests::the_work_grows_with_the_exceptions_and_no_faster_at_fixed_units,raf_workplan::coverage::lines::tests::every_kind_of_work_is_held_to_a_ceiling_linear_in_the_units_and_the_exceptions -->

**Exception refusals.** On every check, and without
counting a line, an exception that names nothing of the source index (an `item` no
item, a prefix no module or item at or below it, a `file` no file, a directory no
file below it) is `coverage-exception-unmatched` at `exceptions`; a row with neither
or both of `item` and `file`, a crate root, an absolute path or a `..` segment is
`okf-bad-exception`, and an empty reason is `okf-empty-reason`.
<!-- raf:tested-by raf_okf::test_crate::records::an_exception_of_another_shape_or_without_a_reason_is_refused_with_its_rule,raf_workplan::test_crate::lines::an_exception_by_item_prefix_file_and_directory_takes_its_lines_out_and_one_beside_it_that_matches_nothing_is_a_finding -->

**Ceiling rows.** Each row of
`ceilings` holds a `crate`, exactly one of `lines`, a whole number, or `percent`, a
number from 0 to 100, and an optional `of`, one of `shipping`, `test` or `all`, by
default `all`.
<!-- raf:tested-by raf_okf::test_crate::records::a_ceiling_names_a_crate_one_whole_number_of_lines_or_a_percent_and_an_optional_class,raf_okf::test_crate::records::a_ceiling_of_another_shape_is_refused_with_its_rule -->

**Ceiling classes and exact values.** Every package of the report is judged: its shipping
orphan lines against its `shipping` row, its test orphan lines against its `test`
row and their sum against its `all` row. The shipping lines and the test lines are
each held to 0 unless their own row or an `all` row bounds them, and the sum is
judged only against an `all` row, so a package with no row has no orphan line and a
package with only a `shipping` row has no test orphan line and any shipping orphan
lines up to that row. A percent ceiling compares
`orphan * 100 * scale <= percent * scale * counted` in whole numbers of any length,
`scale` being the power of ten that makes the percent whole and `counted` the counted
lines of the class, so no division rounds and a package with no counted line passes
it.
<!-- raf:tested-by raf_workplan::test_crate::lines::a_ceiling_of_lines_passes_at_its_count_and_fails_one_below_for_each_class,raf_workplan::test_crate::lines::a_shipping_row_alone_judges_the_shipping_lines_against_itself_and_holds_the_test_lines_to_zero,raf_workplan::test_crate::lines::a_percent_ceiling_compares_whole_numbers_at_its_boundary_and_an_empty_package_passes_it,raf_workplan::coverage::lines::tests::a_percent_is_compared_as_whole_numbers_without_division_or_overflow -->

**Ceiling failures.** A class over its ceiling is `coverage-lines-over-ceiling` at `ceilings`, with a
message that names the package, its counts, the class, the ceiling and its largest
orphan items; a report that could not count the lines is one such finding that says
why, and no ceiling is judged while a capability document does not load, since it
binds nothing, nor without a coverage record that loads. A row whose `crate` names
no package is `scope-crate-missing` at `ceilings`; a row of another shape, an `of` of
another value included, is `okf-bad-ceiling`, and a number that is not one is
`okf-bad-measure`. Every complete check judges the ceilings.
<!-- raf:tested-by raf_workplan::test_crate::lines::a_ceiling_of_lines_passes_at_its_count_and_fails_one_below_for_each_class,raf_workplan::test_crate::lines::a_ceiling_row_of_no_package_is_scope_crate_missing_at_ceilings,raf_workplan::coverage::lines::tests::a_walk_that_fails_is_the_one_error_of_the_report_and_one_finding_under_strict,raf_okf::test_crate::records::a_ceiling_of_another_shape_is_refused_with_its_rule,raf_workplan::test_crate::lines::missing_or_invalid_coverage_and_unloaded_capabilities_defer_ceiling_judgment -->

**Debt record.** The strict debt of a repository is the singleton record
`work/debt/default.md`, of the document type `Debt` with the id `debt:default`; any
other file in `work/debt` is `singleton-other-file`. Its keys are `type`, `id`, the
optional `tags` and `timestamp`, `findings`, rows of exactly `kind` and `subject`, and
`lines`, rows of exactly `item`, a crate-qualified item path, and `orphans`, a count;
none repeats, the writer sorts the rows, and the body is empty.
<!-- raf:tested-by raf_okf::test_crate::records::a_valid_debt_record_validates_to_its_typed_value_and_an_empty_one_opts_out_visibly,raf_okf::test_crate::records::a_row_of_findings_holds_exactly_a_kind_and_a_subject_each_pair_once,raf_okf::test_crate::records::a_row_of_lines_holds_exactly_an_item_and_a_whole_number_of_lines_each_item_once,raf_okf::test_crate::records::both_lists_of_a_debt_record_are_required,raf_okf::test_crate::records::debt_is_one_record_at_its_one_path,raf_okf::test_crate::record_writer::a_debt_record_is_written_sorted_and_read_back_equal,raf_workplan::test_crate::debt::a_file_other_than_the_one_record_is_a_finding -->

**Held findings.** Under [the debt ratchet](../architecture/debt-ratchet.md),
a strict finding whose kind and subject a `findings` row records is not reported,
  and a `findings` row that no present finding matches is `debt-resolved` at
  `findings`.
<!-- raf:tested-by raf_workplan::test_crate::debt::without_a_record_every_strict_finding_stands_and_the_write_creates_it,raf_workplan::test_crate::debt::a_paid_finding_fails_until_the_write_and_then_the_record_equals_the_present_debt -->

**Line debt.** Orphan lines are debt per item, in a package that is over a ceiling of the coverage
  record. A package over a ceiling that has a `lines` row for one of its items is
  judged by its rows, and no finding stands for the lines a row holds: an item with
  fewer present orphan lines than its row is `debt-resolved` at `lines`, an item with
  more is `coverage-lines-over-ceiling` at `lines` naming the row, and an item with no
  row is the same finding naming the missing row, which is growth. A package over a
  ceiling that has no row is judged by its ceilings, as it is without a record, and a
  row of an item that is gone, or of a package that is within its ceilings, is
  `debt-resolved` at `lines`. Line debt is a count; `coverage lines` reports the
  ranges, shows in its Debt column the fewer of the lines of each item and its row,
  and the record never holds the ranges.
<!-- raf:tested-by raf_workplan::test_crate::debt::orphan_lines_over_a_ceiling_are_held_per_item_as_a_count,raf_workplan::test_crate::debt::an_item_that_holds_more_lines_than_its_row_fails_naming_the_row_and_the_write_refuses,raf_workplan::test_crate::debt::an_orphan_item_with_no_row_is_growth_in_a_package_judged_by_its_rows,raf_workplan::test_crate::debt::a_package_over_a_ceiling_that_the_record_does_not_name_is_growth_not_adoption,raf_workplan::test_crate::debt::a_falling_count_fails_until_the_write_lowers_the_row,raf_workplan::test_crate::debt::an_item_that_is_gone_is_resolved_and_the_write_drops_its_row,raf_workplan::test_crate::debt::a_count_back_under_the_ceiling_is_resolved_and_a_package_within_it_holds_no_debt -->

**Debt subjects.** The subject of a row is the claim key for `strict-claim-uncovered`, the capability
  `cap:<slug>` for `strict-example-missing`, the section `cap:<slug>#<section>` that
  its rule judges for the other content kinds, and the reported path for a coverage
  gap.
<!-- raf:tested-by raf_workplan::test_crate::debt::every_kind_a_row_can_hold_is_written_with_a_subject_of_its_form_and_then_holds -->

**Recordable debt kinds.** What can be debt is closed by construction: only a finding that strict alone
  raises, one of the ten kinds of `debt::DEBT_KINDS`: a strict kind or a coverage kind
  of a package that strict alone enforces. A `findings` row of any other kind,
  `coverage-lines-over-ceiling` included, or whose subject does not have the form of
  its kind, and a `lines` row whose item is no item path or whose count is 0, is
  `debt-entry-invalid` at its list. A finding of the default rules, an unreadable
  surface and an obligation are never debt: an obligation belongs to an open change.
<!-- raf:tested-by raf_workplan::test_crate::debt::a_row_of_a_kind_that_is_no_debt_or_a_subject_of_another_form_is_invalid,raf_workplan::test_crate::debt::a_row_of_lines_that_names_no_item_or_no_line_is_invalid,raf_workplan::test_crate::debt::the_debt_kinds_are_kinds_of_strict_mode_and_the_ten_rules_that_a_row_can_hold -->

**Explicit debt writing.** Ordinary checks and repository test gates do not adopt
or create tracked debt. The record is written explicitly through
`cargo workplan debt --write [--copyright <text> --licence <id>]`,
`Gate::write_debt` or `Opened::write_debt`; the API does not detect whether its
caller is a test. Tests of the writer create records in their fixture repositories.
They return a `DebtWrite` (the path, the `DebtChange` `Created`, `Replaced` or
`Unchanged`, and the rows of each list) or a `DebtError`. The SPDX lines come from
`Gate::record_options` or the `RecordOptions` of `Opened::write_debt`, and a record
that exists keeps its own when none are given.
<!-- raf:tested-by raf_workplan::test_crate::debt::without_a_record_every_strict_finding_stands_and_the_write_creates_it,raf_workplan::test_crate::debt::two_writes_are_byte_identical_and_the_second_leaves_the_file_untouched,raf_workplan::test_crate::debt::the_record_keeps_its_spdx_lines_and_a_new_one_takes_those_it_is_given,cargo_workplan::test_crate::debt::the_spdx_lines_are_given_together_and_kept_when_the_record_is_lowered -->

**Debt write refusals.** The write takes the present strict debt: it refuses while a finding exists that no row can hold, a finding of the default
rules, a stale generated block or an unreadable surface among them
(`DebtError::Findings`), refuses growth against a record that exists, a finding or an
orphan item that the record does not hold or an item above its row, a package newly
over a ceiling included (`DebtError::Growth`, with the findings that grew), and
creates a missing record from the present debt, which is how a repository adopts the
documented debt.
<!-- raf:tested-by raf_workplan::test_crate::debt::a_finding_of_the_default_rules_stops_the_write,raf_workplan::test_crate::debt::a_new_strict_finding_fails_and_the_write_refuses_and_writes_nothing,raf_workplan::test_crate::debt::an_orphan_item_with_no_row_is_growth_in_a_package_judged_by_its_rows,raf_workplan::test_crate::debt::a_package_over_a_ceiling_that_the_record_does_not_name_is_growth_not_adoption,raf_workplan::test_crate::debt::a_count_back_under_the_ceiling_is_resolved_and_a_package_within_it_holds_no_debt -->

**Debt write results.** The write
creates or replaces only `work/debt/default.md`, leaves a record that already equals
the present debt untouched, and two writes of the same debt give the same bytes.
`debt --write` exits 0 when it writes, 1 when it refuses with the findings printed,
and 2 when `--write` is missing or the tool cannot run.
<!-- raf:tested-by raf_workplan::test_crate::debt::without_a_record_every_strict_finding_stands_and_the_write_creates_it,raf_workplan::test_crate::debt::two_writes_are_byte_identical_and_the_second_leaves_the_file_untouched,cargo_workplan::test_crate::debt::the_write_creates_the_record_exits_zero_and_the_repository_then_passes,cargo_workplan::test_crate::debt::debt_that_grew_is_exit_one_with_its_findings_and_nothing_is_written,cargo_workplan::test_crate::debt::a_command_misused_is_a_usage_error_before_any_work -->

**The kinds.** The closed `Finding::KINDS` registry includes example, matrix,
claim, section, usage and limitation refusals, coverage exceptions and line
ceilings, and debt resolution or malformed entries. Removed policy selectors are
ordinary unknown-key or unknown-option errors. The typed Finding RDF projection
preserves kind, exact source location, message and fix through the
[shared report profile](./rdf-report-profile.md). Outcome reasons and understated
contract effects remain obligations of the completion lifecycle.
<!-- raf:tested-by raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind,cargo_workplan::test_crate::check::policy_findings_keep_typed_locations_and_messages_in_rdf,raf_workplan::test_crate::strict::the_removed_policy_key_is_rejected_even_for_its_former_values,cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_removed_policy_flag_is_rejected_before_and_after_the_command -->

## Usage

Call `write_debt_in` with a mutable `MemoryTree` or `DiskRecordStore` and the
record options to publish the loaded model's debt. Reload from that store before
the next judgment; the method does not mutate its loaded snapshot. The ordinary
`Gate` and `Opened` disk methods and `debt --write` delegate to the same policy.

Call the runnable example on `raf_workplan::Workplan::check` to see the same
complete policy used by repository gates. Its valid record produces no finding,
while a record containing the removed policy selector is refused with its key
location. Repository commands require no policy flag:

~~~sh
cargo workplan check
cargo workplan change reconcile incremental-parsing
cargo workplan coverage claims --capability cap:streaming-parser
cargo workplan coverage lines --crate raf-okf
cargo workplan --format turtle coverage lines --crate raf-okf
~~~

## Examples

Adopting the same complete-policy findings into disk and memory stores writes
identical debt bytes. An unchanged second write leaves the store untouched.
Paying a finding replaces the record with the smaller debt, while introducing
a new finding refuses and preserves the prior record.

A delivered capability binds a runnable example and a test. Its Contract clauses
and Constraints units name those bound paths in tested-by markers. Removing the
last example is refused; restoring its binding and naming it in Usage satisfies
that requirement without changing the gate invocation.

A package with one uncalled private helper reports that helper's physical line as
orphan debt. Binding or reaching the helper covers it. A declared exception keeps
its exact target and reason visible, while an exception matching nothing is
refused. A debt record can hold present findings, but recording them is distinct
from a repository with zero uncovered items, claims and lines.

## Constraints

- The injected record path preserves the existing debt ratchet and writes only
  the debt record. It does not inject lifecycle writes or make the core host-free.
  The disk store retains the existing path-based preflight limitation.
  <!-- raf:tested-by raf_workplan::test_crate::debt::two_writes_are_byte_identical_and_the_second_leaves_the_file_untouched,raf_workplan::test_crate::debt::a_new_strict_finding_fails_and_the_write_refuses_and_writes_nothing -->

- Allocator registration roots recognize the direct bare `#[global_allocator]`
  attribute, without expanding macros or `cfg_attr`. A registration with arguments,
  repeated on a static, on a non-static item, or without an exact indexed static
  definition refuses the report with an actionable source error and no partial
  line totals. A nested static absent from the index is such a refusal.
<!-- raf:tested-by raf_workplan::test_crate::lines::a_bound_test_reaches_its_registered_allocator_and_real_initializer_dependencies,raf_workplan::test_crate::lines::malformed_allocator_registrations_refuse_without_partial_line_results,raf_workplan::test_crate::lines::an_allocator_without_an_exact_indexed_static_refuses_beside_a_module_static -->
- Reachability follows path references rather than runtime dispatch. An unnamed
  callee is not inferred from an indirect field call or from a trait-object call
  to a type that no covered unit names. The source index does not expand macros
  to discover hidden items. Matching exceptions record the uncovered lines and
  their reasons; an explicit reference can instead make the item reachable.
<!-- raf:tested-by raf_workplan::test_crate::lines::indirect_field_dispatch_does_not_infer_a_callee_but_an_explicit_path_reaches_it,raf_workplan::test_crate::lines::an_impl_for_a_type_that_nothing_names_is_orphan_and_one_for_a_type_that_a_covered_unit_names_is_not,raf_workplan::test_crate::lines::a_macro_invocation_is_orphan_with_its_range_and_covered_once_a_bound_function_calls_what_it_generates -->
- A claim's test is cited, not judged: the marker proves that a mapping exists, and
  the [prose assessment](./workplan-prose-assessment.md) of the section, which reads
  the marker with the text, is where a reader judges whether the test shows the claim.
<!-- raf:tested-by raf_workplan::test_crate::claim_coverage::a_bound_test_that_panics_still_covers_a_claim_without_execution -->
- The key of a Constraints unit is the digest of its text, so a reworded or
  re-punctuated item is a new claim, and a repeated lead key is numbered by its place,
  so a clause with the same lead inserted before it renumbers it. Against a debt
  record either is growth beside a resolved row: touching a recorded claim makes it
  owe a test.
<!-- raf:tested-by raf_workplan::test_crate::claim_coverage::constraints_are_cut_into_top_level_items_paragraphs_and_tables,raf_workplan::test_crate::claim_coverage::a_repeated_key_takes_the_first_number_no_other_key_of_its_section_has_in_either_order,raf_workplan::test_crate::debt::rewording_a_recorded_constraint_is_growth_beside_its_resolved_old_key -->
- `contract-effect-understated` compares logical paths, kinds, impl headers and
  tokens. Moving an item to another module, renaming it or retargeting its old
  re-export path is touched, as is a member whose impl header changes. A physical
  source-file move alone is not touched when those logical inputs stay identical.
<!-- raf:tested-by raf_workplan::test_crate::complete::a_bound_item_moved_to_its_module_file_keeps_identity_but_changed_tokens_refuse,raf_workplan::test_crate::complete::an_item_renamed_below_a_bound_module_refuses,raf_workplan::test_crate::complete::a_bound_reexport_that_names_another_item_refuses,raf_workplan::test_crate::complete::a_changed_header_of_the_impl_block_of_a_bound_type_refuses_for_its_member -->
- An item a macro generates is not an item of the source index, so an edit of a
  `macro_rules!` macro or of its invocation that changes a generated item is not
  judged. An item `#[doc(hidden)]` hides is not on the surface and is not judged
  either, and a member of a trait impl whose self type is not a struct, enum or union
  of the surface (an impl for a type of another crate) is not judged.
<!-- raf:tested-by raf_workplan::test_crate::complete::an_edit_of_a_macro_that_generates_an_item_of_a_bound_module_is_not_judged,raf_workplan::lifecycle::touched::tests::the_surface_holds_the_public_items_impl_members_by_their_header_and_trait_impl_members_of_public_types_only -->
- A trait impl with no member, such as `impl Eq for Frame {}`, is no unit, so adding
  or removing one is not judged.
<!-- raf:tested-by raf_workplan::lifecycle::touched::tests::the_surface_holds_the_public_items_impl_members_by_their_header_and_trait_impl_members_of_public_types_only -->
- `contract-effect-understated` loads the source index of the merge base from the
  packages that a `code` binding reaches, every target of them included: one `git
  ls-tree` and one `git cat-file` for each manifest and source file the load reads,
  only under the complete policy for a change with `contract_effect: none`.
<!-- raf:tested-by raf_workplan::test_crate::complete::a_none_change_reads_each_file_of_the_bound_package_once_at_the_merge_base_and_otherwise_none,raf_workplan::lifecycle::touched::tests::the_tree_of_the_merge_base_answers_from_one_listing_and_reads_each_file_once_with_the_blob_row -->
- `strict::MIN_SECTION_WORDS` is 20 and is not configurable. A word count is a proxy:
  it catches the one-sentence section and judges no style.
<!-- raf:tested-by raf_workplan::test_crate::strict::a_section_of_nineteen_words_is_thin_and_one_of_twenty_is_not -->
- Under the complete policy a package with no ceiling row is held to 0 orphan lines, and its
  shipping or its test lines that neither their own row nor an `all` row bounds are
  held to 0.
<!-- raf:tested-by raf_workplan::test_crate::lines::a_shipping_row_alone_judges_the_shipping_lines_against_itself_and_holds_the_test_lines_to_zero -->
- Callers cannot disable the complete policy through a flag, typed floor or coverage key.
<!-- raf:tested-by raf_workplan::test_crate::strict::every_policy_refusal_case_is_enforced_by_the_default_check,raf_workplan::test_crate::strict::the_removed_policy_key_is_rejected_even_for_its_former_values,cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_removed_policy_flag_is_rejected_before_and_after_the_command -->
- A directory that holds a cargo workspace of its own, such as a fuzz harness that
  the root workspace excludes, is a package of the source index when the source
  roots reach it, so the line report judges its lines as those of any package.
<!-- raf:tested-by raf_workplan::test_crate::lines::a_directory_that_is_a_workspace_of_its_own_has_a_row_when_the_source_roots_reach_it_and_none_when_they_do_not -->
- An item a macro generates is counted as the span of its invocation, and an
  expansion that nothing reaches is orphan with the lines of the invocation, as is the
  `macro_rules!` that generates it; a `#[cfg(test)]` module is counted as test lines,
  in its own column and under the ceiling of its class.
<!-- raf:tested-by raf_workplan::test_crate::lines::a_macro_invocation_is_orphan_with_its_range_and_covered_once_a_bound_function_calls_what_it_generates,raf_workplan::test_crate::lines::a_cfg_test_module_with_one_bound_and_one_unbound_test_reports_the_unbound_one_and_its_helper_as_test_lines -->
- A path that names what a macro generates is read from the tokens of a covered
  unit as a run of identifiers that resolves to nothing, so a local variable used
  by the name of an identifier inside an invocation covers that invocation too.
<!-- raf:tested-by raf_workplan::coverage::lines::tests::a_path_that_may_name_what_a_macro_generates_is_a_run_and_a_method_a_field_or_a_declared_name_is_not,raf_workplan::test_crate::lines::a_macro_invocation_is_orphan_with_its_range_and_covered_once_a_bound_function_calls_what_it_generates -->
- A package that no binding reaches and `scope` does not name is not enforced by the
  explained-items rule; its lines are judged by the ceilings all the same.
<!-- raf:tested-by raf_workplan::test_crate::strict::a_binding_reached_package_is_enforced_and_an_unreached_one_is_not,raf_workplan::test_crate::lines::a_package_that_no_capability_binds_is_reported_with_its_orphan_helper -->
- An exclusion of the coverage record is not an exception: an excluded item whose
  lines nothing covers is orphan until an exception names it with its reason, so a
  reason written once for the two directions is written again for the lines.
<!-- raf:tested-by raf_workplan::test_crate::lines::an_excluded_item_that_nothing_covers_is_orphan_until_an_exception_names_it -->
- The debt record holds strict findings and line counts only, and its text is held to
  the 1 MiB bound of a document.
<!-- raf:tested-by raf_workplan::test_crate::debt::a_row_of_a_kind_that_is_no_debt_or_a_subject_of_another_form_is_invalid,raf_okf::test_crate::refusals::a_text_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused -->
- A record that exists is never grown by the write: a new finding, a new orphan item
  or a package newly over a ceiling is refused until it is paid or the record is
  removed and the repository adopts anew, and the diff of that creation is the review.
  The keys of the record carry no version, so unlike the census record there is no
  second case that lets the write record growth.
<!-- raf:tested-by raf_workplan::test_crate::debt::a_new_strict_finding_fails_and_the_write_refuses_and_writes_nothing,raf_workplan::test_crate::debt::an_orphan_item_with_no_row_is_growth_in_a_package_judged_by_its_rows,raf_workplan::test_crate::debt::a_package_over_a_ceiling_that_the_record_does_not_name_is_growth_not_adoption -->

## Explanation

The complete policy makes a delivered contract independently checkable: runnable
examples show usage, bound tests substantiate individual claims, and source
coverage exposes unexplained behavior. One policy keeps direct library calls,
repository gates and command invocations consistent. The loading findings remain
separate because callers also use the document model for inspection and repair.
The debt ratchet records existing obligations without converting them into proof
of zero-debt adoption.

## Verification

The existing unchanged, shrinking and growing debt cases exercise the injected
memory path beside the disk path. They compare emitted bytes, preserve the store
on refusal or unchanged debt, and retain the existing complete-policy findings.
<!-- raf:tested-by raf_workplan::test_crate::debt::two_writes_are_byte_identical_and_the_second_leaves_the_file_untouched,raf_workplan::test_crate::debt::a_paid_finding_fails_until_the_write_and_then_the_record_equals_the_present_debt,raf_workplan::test_crate::debt::a_new_strict_finding_fails_and_the_write_refuses_and_writes_nothing -->

The bound fixtures exercise every content refusal with a neighboring accepted
document, counted work during loading and complete checks, binding-reached
packages, each claim boundary, outcome reasons and token-level contract effects.
Line fixtures cover all target kinds, overlapping physical ranges, private and
test code, exceptions, explicit ceilings, default zero ceilings and actionable
counting failures. Debt fixtures cover all ten recordable finding kinds, growth,
paid rows, invalid rows, unchanged bytes and typed refusal errors. Command
fixtures compare library results, human output and typed RDF results, including
quiet behavior and refusal exit classes. The removed selector is refused by both
the closed record profile and command parser beside valid ordinary invocations.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::debt_takes_write_and_the_spdx_pair_together_or_not_at_all` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_removed_policy_flag_is_rejected_before_and_after_the_command` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::check::a_removed_policy_key_is_refused_beside_the_default_content_refusal` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::policy_findings_keep_typed_locations_and_messages_in_rdf` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::check::the_default_check_accepts_the_complete_fixture` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_default_check_refuses_a_thin_section_without_a_policy_flag` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_default_check_without_capability_documents_is_a_run_failure` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::claims::a_capability_that_is_not_loaded_exits_two_and_says_why` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::claims::a_claim_that_names_no_test_is_a_finding_and_exits_one_in_text_and_rdf_alike` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::claims::every_claim_names_a_test_prints_the_report_and_exits_zero` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::claims::the_report_sorts_by_capability_first_and_capability_prints_the_one_it_names` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::the_default_completion_refuses_an_underexplained_section_and_accepts_its_neighbor` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::a_command_misused_is_a_usage_error_before_any_work` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::debt::a_finding_of_the_default_rules_stops_the_write_with_exit_one` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::a_second_write_is_unchanged_and_the_bytes_are_the_same` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::debt_that_grew_is_exit_one_with_its_findings_and_nothing_is_written` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::debt::orphan_lines_over_the_ceiling_are_recorded_per_item` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::paid_debt_fails_the_check_with_debt_resolved_until_the_write_lowers_the_record` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::the_default_policy_can_record_a_current_structural_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::the_spdx_lines_are_given_together_and_kept_when_the_record_is_lowered` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::debt::the_write_creates_the_record_exits_zero_and_the_repository_then_passes` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::debt::the_written_record_in_rdf_is_one_object_with_its_members_in_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::a_ceiling_row_and_its_class_decide_the_exit_of_the_default_check_on_the_binary` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::a_package_that_is_not_one_exits_two_and_says_why` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::lines::a_repository_whose_every_counted_line_is_covered_prints_the_report_and_exits_zero` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::lines::an_exception_is_the_excepted_table_and_a_lines_excepted_object_and_the_report_exits_zero` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::lines::an_exception_that_matches_nothing_fails_check_under_default_and_its_matched_neighbour_passes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::an_orphan_line_is_a_finding_and_exits_one_in_text_and_rdf_alike_and_reverse_prints_the_same` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::reconcile::an_unreadable_bound_surface_is_refused_beside_a_resolved_neighbor` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::reconcile::an_untouched_usage_of_a_none_change_refuses_until_edited` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::reconcile::unreadable_bound_source_at_the_merge_base_is_refused_beside_a_parsed_neighbor` | test | [Contract](#contract) |
| `raf_okf::test_crate::profile::an_item_reason_is_a_non_empty_line_beside_any_outcome_and_an_empty_one_is_refused` | test | [Contract](#contract) |
| `raf_okf::test_crate::record_writer::a_coverage_record_writes_only_its_closed_policy_independent_keys` | test | [Verification](#verification) |
| `raf_okf::test_crate::record_writer::a_debt_record_is_written_sorted_and_read_back_equal` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::a_ceiling_names_a_crate_one_whole_number_of_lines_or_a_percent_and_an_optional_class` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::a_ceiling_of_another_shape_is_refused_with_its_rule` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::a_coverage_record_rejects_the_removed_policy_key` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::a_row_of_findings_holds_exactly_a_kind_and_a_subject_each_pair_once` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::a_row_of_lines_holds_exactly_an_item_and_a_whole_number_of_lines_each_item_once` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::a_valid_debt_record_validates_to_its_typed_value_and_an_empty_one_opts_out_visibly` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::an_exception_names_one_item_prefix_file_or_directory_and_its_reason` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::an_exception_of_another_shape_or_without_a_reason_is_refused_with_its_rule` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::both_lists_of_a_debt_record_are_required` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::debt_is_one_record_at_its_one_path` | test | [Contract](#contract) |
| `raf_okf::test_crate::refusals::a_text_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused` | test | [Constraints](#constraints) |
| `raf_okf::test_crate::tested_by::a_malformed_tested_by_marker_is_refused_with_its_line` | test | [Contract](#contract) |
| `raf_okf::test_crate::tested_by::a_tested_by_marker_belongs_in_a_capability_document` | test | [Contract](#contract) |
| `raf_okf::test_crate::tested_by::a_tested_by_marker_maps_a_list_item_a_heading_or_the_statement_of_its_claim_marker` | test | [Contract](#contract) |
| `raf_okf::test_crate::tested_by::a_tested_by_marker_names_its_tests_and_maps_the_paragraph_above_it` | test | [Contract](#contract) |
| `raf_okf::test_crate::tested_by::a_tested_by_marker_with_no_statement_directly_above_is_refused` | test | [Contract](#contract) |
| `raf_okf::test_crate::tested_by::tested_by_markers_in_fences_code_spans_and_proposed_blocks_are_not_read` | test | [Verification](#verification) |
| `raf_workplan::Workplan::check` | doctest | [Usage](#usage) |
| `raf_workplan::coverage::lines::tests::a_path_that_may_name_what_a_macro_generates_is_a_run_and_a_method_a_field_or_a_declared_name_is_not` | test | [Constraints](#constraints) |
| `raf_workplan::coverage::lines::tests::a_percent_is_compared_as_whole_numbers_without_division_or_overflow` | test | [Contract](#contract) |
| `raf_workplan::coverage::lines::tests::a_walk_that_fails_is_the_one_error_of_the_report_and_one_finding_under_strict` | test | [Contract](#contract) |
| `raf_workplan::coverage::lines::tests::a_walk_that_fails_retains_a_single_typed_error_and_the_error_table` | test | [Contract](#contract) |
| `raf_workplan::coverage::lines::tests::every_kind_of_work_is_held_to_a_ceiling_linear_in_the_units_and_the_exceptions` | test | [Contract](#contract) |
| `raf_workplan::coverage::lines::tests::the_default_check_sweeps_each_counted_line_unit` | test | [Verification](#verification) |
| `raf_workplan::coverage::lines::tests::the_work_grows_with_the_exceptions_and_no_faster_at_fixed_units` | test | [Contract](#contract) |
| `raf_workplan::coverage::lines::tests::the_work_grows_with_the_units_and_no_faster_at_fixed_exceptions` | test | [Contract](#contract) |
| `raf_workplan::debt::tests::a_report_that_could_not_count_is_one_finding_no_row_holds_and_resolves_no_row` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::touched::tests::a_git_failure_while_the_merge_base_loads_is_that_error_and_a_file_that_is_not_utf8_names_the_file` | test | [Contract](#contract) |
| `raf_workplan::lifecycle::touched::tests::an_impl_header_is_the_tokens_before_its_brace_at_every_depth_and_a_file_that_does_not_parse_is_an_error` | test | [Contract](#contract) |
| `raf_workplan::lifecycle::touched::tests::the_surface_holds_the_public_items_impl_members_by_their_header_and_trait_impl_members_of_public_types_only` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::lifecycle::touched::tests::the_tree_of_the_merge_base_answers_from_one_listing_and_reads_each_file_once_with_the_blob_row` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::lifecycle::touched::tests::whitespace_and_plain_comments_leave_the_stream_and_a_doc_comment_or_a_token_changes_it` | test | [Contract](#contract) |
| `raf_workplan::obligation::tests::the_evidence_of_the_strict_kinds_names_the_outcome_or_the_item_its_file_and_line_and_the_capability` | test | [Contract](#contract) |
| `raf_workplan::strict::claims::tests::a_key_is_the_kebab_slug_of_its_lead_and_a_lead_is_sixty_characters` | test | [Contract](#contract) |
| `raf_workplan::strict::claims::tests::a_list_item_opens_at_its_marker_and_its_content_starts_after_the_space` | test | [Verification](#verification) |
| `raf_workplan::strict::claims::tests::a_unit_ends_on_the_line_before_the_next_unit` | test | [Verification](#verification) |
| `raf_workplan::strict::tests::the_default_check_judges_each_policy_unit_once_without_policy_work_during_load` | test | [Contract](#contract) |
| `raf_workplan::test_crate::bound::a_no_run_example_meets_the_strict_example_rule_and_verify_refuses_it_beside_a_runnable_one` | test | [Contract](#contract) |
| `raf_workplan::test_crate::claim_coverage::a_bold_lead_directly_after_any_heading_starts_a_contract_clause` | test | [Contract](#contract) |
| `raf_workplan::test_crate::claim_coverage::a_bound_test_that_panics_still_covers_a_claim_without_execution` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::claim_coverage::a_comment_line_ends_a_constraints_paragraph_and_an_item_goes_on_only_with_its_indented_lines` | test | [Contract](#contract) |
| `raf_workplan::test_crate::claim_coverage::a_contract_clause_covered_on_its_second_paragraph_is_covered_and_its_neighbour_is_not` | test | [Contract](#contract) |
| `raf_workplan::test_crate::claim_coverage::a_contract_is_cut_into_clauses_at_bold_leads_and_headings_and_the_text_before_is_the_opening` | test | [Contract](#contract) |
| `raf_workplan::test_crate::claim_coverage::a_heading_clause_is_covered_by_a_marker_after_its_heading_and_its_text_needs_none` | test | [Contract](#contract) |
| `raf_workplan::test_crate::claim_coverage::a_long_contract_with_no_clause_start_is_one_collapsed_claim_and_one_with_clauses_is_not` | test | [Contract](#contract) |
| `raf_workplan::test_crate::claim_coverage::a_repeated_key_takes_the_first_number_no_other_key_of_its_section_has_in_either_order` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::claim_coverage::a_tested_by_marker_in_usage_names_the_bound_example_as_a_code_span_does` | test | [Contract](#contract) |
| `raf_workplan::test_crate::claim_coverage::a_thematic_break_and_an_empty_item_in_constraints_are_no_claim_and_an_item_beside_them_is_one` | test | [Contract](#contract) |
| `raf_workplan::test_crate::claim_coverage::an_unbound_claim_marker_is_refused_beside_covered_neighbors` | test | [Contract](#contract) |
| `raf_workplan::test_crate::claim_coverage::constraints_are_cut_into_top_level_items_paragraphs_and_tables` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::claim_coverage::fenced_generated_and_proposed_text_make_no_claim_and_evidence_does_not_cover_one` | test | [Contract](#contract) |
| `raf_workplan::test_crate::claim_coverage::the_report_lists_every_claim_of_every_delivered_capability_sorted_and_the_same_each_run` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_bound_item_made_private_or_visible_to_its_crate_alone_refuses` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_bound_item_moved_to_its_module_file_keeps_identity_but_changed_tokens_refuse` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::a_bound_module_made_private_refuses_for_it_and_its_items` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_bound_package_whose_surface_cannot_be_listed_is_an_error_and_without_the_second_glob_it_answers` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_bound_reexport_that_names_another_item_refuses` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::a_changed_attribute_of_a_bound_item_refuses` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_changed_header_of_the_impl_block_of_a_bound_type_refuses_for_its_member` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::a_failing_bound_test_and_a_tense_mismatch_refuse_until_their_repairs_complete` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_method_edited_to_the_tokens_of_a_sibling_method_refuses` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_method_removed_from_an_impl_in_another_file_or_with_its_file_refuses` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_new_bound_public_item_refuses` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_none_change_reads_each_file_of_the_bound_package_once_at_the_merge_base_and_otherwise_none` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::a_none_change_repeatedly_owes_its_public_contract_obligation_until_repaired` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_none_change_that_edits_a_bound_signature_refuses` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_plain_comment_outside_every_bound_item_completes` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_private_helper_removed_from_a_bound_module_completes_and_a_public_item_removed_refuses` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_public_method_of_a_bound_type_made_private_refuses` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_reflow_that_adds_a_trailing_comma_or_a_space_in_a_doc_comment_refuses` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_reformatted_bound_body_and_a_plain_comment_complete` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_trait_impl_member_of_a_bound_type_edited_or_removed_refuses_and_one_of_a_private_type_completes` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::an_edit_of_a_bound_body_alone_refuses` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::an_edit_of_a_bound_doc_comment_refuses` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::an_edit_of_a_macro_that_generates_an_item_of_a_bound_module_is_not_judged` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::an_item_dropped_without_a_reason_refuses_condition_two` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::an_item_moved_within_its_file_completes` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::an_item_renamed_below_a_bound_module_refuses` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::an_untouched_usage_or_examples_of_a_none_change_refuses` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::completion_after_record_removal_enforces_the_complete_policy` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::the_bound_items_of_a_file_absent_at_the_merge_base_refuse_and_a_new_file_of_private_items_completes` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::the_check_once_the_record_is_removed_applies_the_debt_record_so_a_held_finding_does_not_refuse` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::the_check_once_the_record_is_removed_refuses_a_debt_row_that_is_resolved` | test | [Verification](#verification) |
| `raf_workplan::test_crate::debt::a_count_back_under_the_ceiling_is_resolved_and_a_package_within_it_holds_no_debt` | test | [Contract](#contract) |
| `raf_workplan::test_crate::debt::a_falling_count_fails_until_the_write_lowers_the_row` | test | [Contract](#contract) |
| `raf_workplan::test_crate::debt::a_file_other_than_the_one_record_is_a_finding` | test | [Contract](#contract) |
| `raf_workplan::test_crate::debt::a_finding_of_the_default_rules_stops_the_write` | test | [Contract](#contract) |
| `raf_workplan::test_crate::debt::a_new_strict_finding_fails_and_the_write_refuses_and_writes_nothing` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::debt::a_package_over_a_ceiling_that_the_record_does_not_name_is_growth_not_adoption` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::debt::a_paid_finding_fails_until_the_write_and_then_the_record_equals_the_present_debt` | test | [Contract](#contract) |
| `raf_workplan::test_crate::debt::a_record_that_breaks_the_profile_is_the_finding_of_its_rule_and_stops_the_write` | test | [Verification](#verification) |
| `raf_workplan::test_crate::debt::a_repository_that_passes_has_no_finding_to_hold_and_the_write_records_nothing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::debt::a_row_of_a_kind_that_is_no_debt_or_a_subject_of_another_form_is_invalid` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::debt::a_row_of_lines_that_names_no_item_or_no_line_is_invalid` | test | [Contract](#contract) |
| `raf_workplan::test_crate::debt::an_empty_debt_record_is_judged_without_a_policy_switch` | test | [Verification](#verification) |
| `raf_workplan::test_crate::debt::an_invalid_debt_row_is_refused_without_a_policy_selector` | test | [Verification](#verification) |
| `raf_workplan::test_crate::debt::an_invalid_row_holds_nothing_so_the_write_replaces_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::debt::an_item_that_holds_more_lines_than_its_row_fails_naming_the_row_and_the_write_refuses` | test | [Contract](#contract) |
| `raf_workplan::test_crate::debt::an_item_that_is_gone_is_resolved_and_the_write_drops_its_row` | test | [Contract](#contract) |
| `raf_workplan::test_crate::debt::an_orphan_item_with_no_row_is_growth_in_a_package_judged_by_its_rows` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::debt::every_error_of_the_write_says_what_to_do` | test | [Verification](#verification) |
| `raf_workplan::test_crate::debt::every_kind_a_row_can_hold_is_written_with_a_subject_of_its_form_and_then_holds` | test | [Contract](#contract) |
| `raf_workplan::test_crate::debt::orphan_lines_over_a_ceiling_are_held_per_item_as_a_count` | test | [Contract](#contract) |
| `raf_workplan::test_crate::debt::rewording_a_recorded_constraint_is_growth_beside_its_resolved_old_key` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::debt::the_debt_kinds_are_kinds_of_strict_mode_and_the_ten_rules_that_a_row_can_hold` | test | [Contract](#contract) |
| `raf_workplan::test_crate::debt::the_default_debt_writer_judges_all_present_findings` | test | [Contract](#contract) |
| `raf_workplan::test_crate::debt::the_default_gate_records_present_debt_without_a_policy_switch` | test | [Verification](#verification) |
| `raf_workplan::test_crate::debt::the_record_is_read_back_as_the_typed_value_the_write_built` | test | [Verification](#verification) |
| `raf_workplan::test_crate::debt::the_record_keeps_its_spdx_lines_and_a_new_one_takes_those_it_is_given` | test | [Contract](#contract) |
| `raf_workplan::test_crate::debt::the_snapshot_of_every_other_file_is_unchanged_by_a_refused_write` | test | [Verification](#verification) |
| `raf_workplan::test_crate::debt::two_writes_are_byte_identical_and_the_second_leaves_the_file_untouched` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::debt::without_a_record_every_strict_finding_stands_and_the_write_creates_it` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::a_tested_by_marker_substantiates_its_section_and_an_unmarked_binding_keeps_its_row` | test | [Contract](#contract) |
| `raf_workplan::test_crate::graph::the_graph_has_every_kind_of_edge` | test | [Contract](#contract) |
| `raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lines::a_bound_test_reaches_its_registered_allocator_and_real_initializer_dependencies` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::lines::a_ceiling_of_lines_passes_at_its_count_and_fails_one_below_for_each_class` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lines::a_ceiling_row_of_no_package_is_scope_crate_missing_at_ceilings` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lines::a_cfg_test_module_with_one_bound_and_one_unbound_test_reports_the_unbound_one_and_its_helper_as_test_lines` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::lines::a_crate_whose_public_surface_is_bound_and_whose_helpers_are_reached_has_no_orphan_line` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lines::a_directory_that_is_a_workspace_of_its_own_has_a_row_when_the_source_roots_reach_it_and_none_when_they_do_not` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::lines::a_macro_invocation_is_orphan_with_its_range_and_covered_once_a_bound_function_calls_what_it_generates` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::lines::a_package_that_no_capability_binds_is_reported_with_its_orphan_helper` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::lines::a_percent_ceiling_compares_whole_numbers_at_its_boundary_and_an_empty_package_passes_it` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lines::a_registered_allocator_is_not_a_root_of_an_unbound_target` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lines::a_shipping_row_alone_judges_the_shipping_lines_against_itself_and_holds_the_test_lines_to_zero` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::lines::a_test_only_allocator_requires_a_bound_test_context_not_a_shipping_binding` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lines::an_allocator_without_an_exact_indexed_static_refuses_beside_a_module_static` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::lines::an_excepted_unit_preserves_ranges_reason_and_the_excepted_table` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lines::an_exception_by_item_prefix_file_and_directory_takes_its_lines_out_and_one_beside_it_that_matches_nothing_is_a_finding` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lines::an_excluded_item_that_nothing_covers_is_orphan_until_an_exception_names_it` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::lines::an_impl_for_a_type_that_nothing_names_is_orphan_and_one_for_a_type_that_a_covered_unit_names_is_not` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::lines::an_unreferenced_private_helper_is_exactly_its_span_and_the_same_helper_called_by_a_bound_function_is_not` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lines::binding_a_type_covers_its_impl_blocks_and_their_methods_and_binding_a_method_covers_nothing_beside_it` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lines::crlf_reads_as_lf_and_two_runs_give_the_same_bytes` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lines::every_target_is_counted_and_shared_module_units_are_unioned_once` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lines::indirect_field_dispatch_does_not_infer_a_callee_but_an_explicit_path_reaches_it` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::lines::malformed_allocator_registrations_refuse_without_partial_line_results` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::lines::missing_or_invalid_coverage_and_unloaded_capabilities_defer_ceiling_judgment` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lines::orphan_lines_are_enforced_without_a_policy_switch` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lines::real_format_captures_cover_constants_while_named_bindings_and_custom_tokens_do_not` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lines::shared_source_coverage_uses_each_packages_edition_beside_raw_names` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lines::the_report_preserves_exact_line_counts_ranges_and_three_tables` | test | [Contract](#contract) |
| `raf_workplan::test_crate::obligations::a_dropped_or_moved_item_needs_a_reason_and_an_explained_outcome_is_accepted` | test | [Contract](#contract) |
| `raf_workplan::test_crate::strict::a_binding_reached_package_is_enforced_and_an_unreached_one_is_not` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::strict::a_capability_that_is_not_delivered_has_no_finding_of_the_content_rules` | test | [Contract](#contract) |
| `raf_workplan::test_crate::strict::a_capability_with_no_example_is_refused_and_one_with_one_example_is_not` | test | [Contract](#contract) |
| `raf_workplan::test_crate::strict::a_code_span_inside_a_comment_names_no_example_in_usage` | test | [Contract](#contract) |
| `raf_workplan::test_crate::strict::a_missing_or_invalid_coverage_record_is_reported_before_package_gaps` | test | [Contract](#contract) |
| `raf_workplan::test_crate::strict::a_scoped_package_is_enforced_once_with_a_binding_present` | test | [Contract](#contract) |
| `raf_workplan::test_crate::strict::a_section_of_nineteen_words_is_thin_and_one_of_twenty_is_not` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::strict::a_thematic_break_and_an_empty_item_are_no_limitation` | test | [Contract](#contract) |
| `raf_workplan::test_crate::strict::an_exclusion_of_the_record_takes_a_gap_of_a_bound_package_out` | test | [Contract](#contract) |
| `raf_workplan::test_crate::strict::code_spans_fenced_code_comments_and_the_heading_are_not_prose` | test | [Contract](#contract) |
| `raf_workplan::test_crate::strict::constraints_without_a_list_item_are_refused_and_one_list_item_is_not` | test | [Contract](#contract) |
| `raf_workplan::test_crate::strict::content_and_binding_findings_follow_load_findings` | test | [Contract](#contract) |
| `raf_workplan::test_crate::strict::every_policy_refusal_case_is_enforced_by_the_default_check` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::strict::the_complete_policy_accepts_the_documented_repository` | test | [Verification](#verification) |
| `raf_workplan::test_crate::strict::the_default_gate_judges_the_complete_policy_in_its_opened_workplan` | test | [Contract](#contract) |
| `raf_workplan::test_crate::strict::the_removed_policy_key_is_rejected_even_for_its_former_values` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::strict::the_words_of_a_generated_block_are_not_prose_of_its_section` | test | [Contract](#contract) |
| `raf_workplan::test_crate::strict::usage_that_names_no_bound_example_is_refused_and_a_code_span_of_it_is_not` | test | [Contract](#contract) |
| `raf_workplan::test_crate::strict::verification_without_the_matrix_is_refused_and_with_the_block_is_not` | test | [Contract](#contract) |
| `raf_workplan::test_crate::strict::while_a_capability_document_does_not_load_no_gap_of_a_bound_package_is_reported` | test | [Contract](#contract) |
<!-- raf:generated:end -->
