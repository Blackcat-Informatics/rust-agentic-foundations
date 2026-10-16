---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-reconcile
title: "Change reconciliation"
description: "change new, change impact and change reconcile: opening a change, listing what it touches, and producing the documentation obligations."
state: implementing
bindings:
  code:
    - raf_workplan::Workplan::new_change
    - raf_workplan::lifecycle::NewChangeRequest
    - raf_workplan::lifecycle::NewChange
    - raf_workplan::lifecycle::NewChangeError
    - raf_workplan::Workplan::impact
    - raf_workplan::lifecycle::Impact
    - raf_workplan::Workplan::reconcile
    - raf_workplan::obligation::Reconciled
    - raf_workplan::obligation::Obligation
    - raf_workplan::obligation::ObligationKind
    - raf_workplan::obligation::Needed
    - raf_workplan::obligation::suppressed
    - raf_workplan::obligation::tense::PHRASES
    - raf_workplan::lifecycle::Lifecycle::open
    - raf_workplan::lifecycle::Lifecycle::reconcile
    - raf_workplan::lifecycle::section_digest
    - raf_workplan::git::require_version
    - raf_workplan::git::show_prefix
    - raf_workplan::git::default_merge_base
    - raf_workplan::git::blob_at
    - raf_workplan::git::MergeBase
    - raf_workplan::obligation::ClaimProblem
    - raf_workplan::obligation::OverLimit
    - raf_workplan::obligation::InputMoved
    - raf_workplan::obligation::EvidenceChange
    - raf_workplan::lifecycle
    - raf_workplan::obligation
    - raf_workplan::git
  tests:
    - raf_workplan::test_crate::lifecycle::new_writes_one_file_with_the_keys_the_id_and_a_summary_and_it_loads_clean
    - raf_workplan::test_crate::lifecycle::a_bad_slug_is_refused_before_anything_is_written_and_a_good_one_next_to_it_is_not
    - raf_workplan::test_crate::lifecycle::an_existing_file_is_refused_and_left_byte_identical
    - raf_workplan::test_crate::lifecycle::a_capability_that_does_not_exist_is_refused
    - raf_workplan::test_crate::lifecycle::a_capability_whose_document_did_not_load_is_named_and_never_skipped
    - raf_workplan::test_crate::lifecycle::an_affects_entry_must_name_a_listed_capability_and_one_of_the_seven_sections
    - raf_workplan::test_crate::lifecycle::several_capabilities_are_listed_in_the_order_given_without_repeats
    - raf_workplan::test_crate::lifecycle::the_default_affects_is_the_contract_of_each_listed_capability
    - raf_workplan::test_crate::lifecycle::explicit_affects_are_written_as_given_and_impact_reports_them_sorted
    - raf_workplan::test_crate::lifecycle::the_guidance_follows_the_state_of_each_listed_capability
    - raf_workplan::test_crate::lifecycle::the_printed_text_holds_the_template_and_the_guidance
    - raf_workplan::test_crate::lifecycle::the_template_in_a_capability_of_each_state_gives_one_block_and_a_clean_check
    - raf_workplan::test_crate::lifecycle::both_spdx_fields_open_the_record_with_the_two_comment_lines_of_the_records_of_the_repository
    - raf_workplan::test_crate::lifecycle::one_spdx_field_without_the_other_is_an_error_that_names_the_missing_one
    - raf_workplan::test_crate::lifecycle::impact_gives_the_capabilities_sections_bindings_dependants_and_linking_sections
    - raf_workplan::test_crate::lifecycle::impact_prints_each_part_in_a_fixed_order
    - raf_workplan::test_crate::obligations::each_fault_is_owed_with_its_kind_location_and_evidence_and_fixing_it_removes_the_line
    - raf_workplan::test_crate::obligations::every_static_kind_is_raised_by_a_fixture_and_the_kinds_that_need_a_process_by_none
    - raf_workplan::test_crate::obligations::the_binding_obligations_are_the_gaps_of_the_roadmap
    - raf_workplan::test_crate::obligations::contract_effect_none_keeps_every_obligation_the_documents_can_show
    - raf_workplan::test_crate::obligations::equal_inputs_print_equal_output
    - raf_workplan::test_crate::obligations::wording_that_only_looks_like_a_phrase_owes_nothing
    - raf_workplan::test_crate::obligations::text_of_a_proposed_block_is_not_read_for_wording
    - raf_workplan::test_crate::obligations::reconcile_writes_nothing
    - raf_workplan::test_crate::obligations::each_assessment_fault_is_owed_at_the_section_naming_the_input_and_a_current_acceptance_clears_it
    - raf_workplan::test_crate::obligations::contract_effect_none_keeps_the_assessments_and_another_section_is_not_judged
    - raf_workplan::test_crate::complete::each_kind_of_obligation_is_a_reconcile_line_in_a_git_repository_and_fixing_it_removes_the_line
    - raf_workplan::test_crate::evidence::an_edit_of_a_bound_symbol_makes_its_record_stale_and_nothing_else_does
    - raf_workplan::test_crate::evidence::a_stale_record_of_a_capability_the_change_does_not_list_is_not_owed
    - raf_workplan::test_crate::evidence::each_change_of_a_passing_record_since_the_merge_base_reopens_its_claim
    - raf_workplan::test_crate::evidence::a_record_that_failed_at_the_merge_base_or_was_not_there_is_compared_with_nothing
    - raf_workplan::test_crate::evidence::a_record_at_the_merge_base_that_does_not_read_is_an_error_and_not_a_pass
    - raf_workplan::test_crate::evidence::with_no_default_branch_a_claim_is_an_error_and_no_claim_asks_git_nothing
    - raf_workplan::test_crate::evidence::each_named_record_is_read_once_at_the_merge_base_with_the_blob_row
    - raf_workplan::test_crate::untouched::an_unchanged_affected_section_is_owed_with_the_line_the_contract_shows
    - raf_workplan::test_crate::untouched::a_section_edited_on_the_branch_is_not_reported_and_its_neighbour_is
    - raf_workplan::test_crate::untouched::an_edit_that_is_not_committed_counts_because_the_working_tree_is_compared
    - raf_workplan::test_crate::untouched::only_a_generated_block_changed_is_still_reported
    - raf_workplan::test_crate::untouched::a_file_absent_at_the_merge_base_counts_as_touched
    - raf_workplan::test_crate::untouched::on_the_default_branch_itself_every_affected_section_is_owed_and_it_is_not_an_error
    - raf_workplan::test_crate::untouched::a_linked_work_tree_compares_its_own_head
    - raf_workplan::test_crate::untouched::a_workspace_below_the_top_of_the_repository_is_compared_at_its_prefix
    - raf_workplan::test_crate::untouched::a_local_branch_named_origin_main_does_not_stand_in_for_the_remote_one
    - raf_workplan::test_crate::untouched::a_default_branch_named_like_an_option_is_resolved_and_never_reaches_git_as_one
    - raf_workplan::test_crate::untouched::no_default_branch_is_an_error_that_names_the_fix_and_the_fix_clears_it
    - raf_workplan::test_crate::untouched::a_file_over_one_mebibyte_at_the_merge_base_is_an_error_and_exactly_one_is_read
    - raf_workplan::test_crate::untouched::a_file_at_the_merge_base_that_does_not_parse_is_an_error_and_not_a_pass
    - raf_workplan::test_crate::untouched::contract_effect_none_leaves_out_contract_usage_and_examples_and_asks_git_for_nothing_then
    - raf_workplan::test_crate::untouched::git_2_29_stops_the_open_before_any_other_process_and_2_30_goes_on
    - raf_workplan::test_crate::untouched::the_digest_leaves_out_generated_blocks_on_both_sides_and_nothing_else
    - raf_workplan::test_crate::untouched::reconcile_writes_nothing_and_prints_the_same_list_twice
    - raf_workplan::test_crate::untouched::stubs::reconcile_asks_git_exactly_the_commands_of_the_contract_in_order
    - raf_workplan::test_crate::untouched::stubs::a_root_below_the_top_reads_each_file_at_the_prefix
    - raf_workplan::test_crate::untouched::stubs::a_prefix_the_source_path_pattern_cannot_carry_is_an_error_and_not_a_guess
    - raf_workplan::test_crate::untouched::stubs::a_stub_git_2_29_runs_one_process_and_exits_with_the_error
    - raf_workplan::test_crate::git::stubs::every_question_reaches_git_as_exactly_its_argv_in_the_work_directory
    - raf_workplan::test_crate::verify::an_example_that_does_not_build_is_a_line_and_is_gone_once_it_builds_and_runs
    - raf_workplan::test_crate::verify::a_binding_to_a_function_with_no_test_reports_zero_tests_as_failing
    - raf_workplan::test_crate::verify::test_bindings_are_not_run_by_reconcile_and_unresolved_examples_are_not_started
    - raf_workplan::test_crate::scenarios::several_redesigns_leave_one_readable_present
    - raf_workplan::test_crate::scenarios::planning_produces_the_documentation
    - cargo_workplan::test_crate::reconcile::a_list_with_obligations_is_printed_one_line_each_and_exits_one
    - cargo_workplan::test_crate::reconcile::an_empty_list_prints_nothing_and_exits_zero
    - cargo_workplan::test_crate::reconcile::fixing_an_obligation_removes_its_line_and_only_its_line
    - cargo_workplan::test_crate::new::the_printed_steps_end_in_a_clean_check_and_skipping_the_stage_step_does_not
    - cargo_workplan::test_crate::process::a_git_older_than_2_30_is_exit_two_before_any_other_process
    - raf_workplan::test_crate::git::a_blob_is_returned_byte_for_byte
    - raf_workplan::test_crate::git::a_blob_over_one_mebibyte_is_an_error_and_exactly_one_mebibyte_is_read
    - raf_workplan::test_crate::git::a_default_branch_named_like_an_option_is_a_ref_and_never_an_option
    - raf_workplan::test_crate::git::a_directory_is_an_error_and_not_an_absent_file
    - raf_workplan::test_crate::git::a_directory_that_is_no_work_tree_has_no_listing_and_a_broken_index_is_an_error
    - raf_workplan::test_crate::git::a_file_is_read_as_it_was_at_the_merge_base
    - raf_workplan::test_crate::git::a_linked_work_tree_asks_about_its_own_head_and_shares_the_remote_refs
    - raf_workplan::test_crate::git::a_linked_work_tree_is_its_own_top
    - raf_workplan::test_crate::git::a_local_branch_named_origin_main_does_not_stand_in_for_the_remote_one
    - raf_workplan::test_crate::git::a_program_that_does_not_exist_is_an_error_and_not_a_fallback
    - raf_workplan::test_crate::git::a_repository_without_origin_head_has_no_default_branch_and_the_fix_is_named
    - raf_workplan::test_crate::git::histories_with_no_common_ancestor_have_no_merge_base
    - raf_workplan::test_crate::git::stubs::a_stub_that_exits_with_a_failure_is_an_error_with_its_message
    - raf_workplan::test_crate::git::stubs::an_answer_that_is_not_a_full_reference_name_stops_the_chain_before_any_commit_is_asked_for
    - raf_workplan::test_crate::git::stubs::the_version_gate_over_a_real_process_refuses_2_29_and_accepts_2_30
    - raf_workplan::test_crate::git::the_default_branch_of_a_clone_is_the_full_remote_tracking_name
    - raf_workplan::test_crate::git::the_installed_git_is_new_enough
    - raf_workplan::test_crate::git::the_merge_base_is_where_the_branch_left_the_default_branch
    - raf_workplan::test_crate::git::the_prefix_is_where_the_directory_is_below_the_top_of_its_work_tree
    - raf_workplan::test_crate::git::the_tracked_paths_are_the_index_and_not_the_files_on_disk
    - raf_workplan::test_crate::git::the_tracked_paths_of_a_subdirectory_are_relative_to_it
    - raf_workplan::test_crate::lifecycle::a_file_or_a_link_where_work_changes_belongs_is_refused
    - raf_workplan::test_crate::lifecycle::a_link_at_the_path_is_an_existing_file_and_is_not_followed
    - raf_workplan::test_crate::lifecycle::a_slug_that_reads_as_a_tracker_reference_leaves_no_file
    - raf_workplan::test_crate::lifecycle::an_spdx_field_that_is_not_one_line_of_text_is_refused_and_nothing_is_written
    - raf_workplan::test_crate::lifecycle::impact_of_a_change_that_just_was_created_reads_the_record_from_the_disk
    - raf_workplan::test_crate::lifecycle::impact_of_a_change_that_reaches_nothing_says_none
    - raf_workplan::test_crate::lifecycle::impact_of_the_repositorys_own_change_lists_what_the_record_says
    - raf_workplan::test_crate::lifecycle::impact_tells_an_unknown_slug_from_a_record_that_did_not_load
    - raf_workplan::test_crate::lifecycle::neither_spdx_field_writes_no_comment_line
    - raf_workplan::test_crate::lifecycle::the_bad_slug_message_says_what_a_slug_is
    - raf_workplan::test_crate::lifecycle::the_guidance_for_a_delivered_capability_names_both_findings_it_avoids
    - raf_workplan::test_crate::lifecycle::the_record_beside_a_delivered_capability_is_a_finding_until_the_state_is_set
    - raf_workplan::test_crate::lifecycle::the_sync_note_is_there_when_the_target_file_was_already_stale
    - raf_workplan::test_crate::lifecycle::the_template_in_the_wrong_state_is_the_finding_the_guidance_names
    - raf_workplan::test_crate::lifecycle::work_changes_is_created_when_the_repository_has_none
    - raf_workplan::test_crate::obligations::a_block_of_another_change_in_a_capability_that_does_not_load_is_not_owed
    - raf_workplan::test_crate::obligations::a_capability_over_the_size_bound_is_not_loaded_and_refused
    - raf_workplan::test_crate::obligations::a_delivered_capability_owes_code_and_tests_but_not_examples
    - raf_workplan::test_crate::obligations::a_delivered_capability_that_breaks_another_rule_too_is_not_read_as_unbound
    - raf_workplan::test_crate::obligations::a_hundred_thousand_lines_are_a_hundred_thousand_obligations_in_order
    - raf_workplan::test_crate::obligations::a_listed_capability_that_is_not_loaded_is_an_error_and_not_a_skipped_one
    - raf_workplan::test_crate::obligations::a_very_long_line_is_one_obligation_and_is_read_in_linear_time
    - raf_workplan::test_crate::obligations::an_unknown_change_is_not_a_change_that_did_not_load
    - raf_workplan::test_crate::obligations::an_unterminated_fence_hides_the_rest_of_the_section
    - raf_workplan::test_crate::obligations::an_unterminated_fence_in_an_earlier_section_hides_the_headings_after_it
    - raf_workplan::test_crate::obligations::capabilities_that_bind_nothing_owe_no_binding
    - raf_workplan::test_crate::obligations::fixing_one_fault_removes_exactly_its_lines
    - raf_workplan::test_crate::obligations::moves_to_unknown_records_cost_the_same_with_few_records_as_with_many
    - raf_workplan::test_crate::obligations::needed_is_a_typed_value_whose_text_starts_with_the_contract_string
    - raf_workplan::test_crate::obligations::only_the_capabilities_the_change_lists_are_judged
    - raf_workplan::test_crate::obligations::text_that_is_read_as_wording_at_a_hostile_size_does_not_hang
    - raf_workplan::test_crate::obligations::the_baseline_owes_nothing_and_check_agrees
    - raf_workplan::test_crate::obligations::the_change_is_found_with_or_without_its_prefix
    - raf_workplan::test_crate::obligations::the_proposals_are_those_of_the_graph_for_this_change_only
    - raf_workplan::test_crate::obligations::two_blocks_in_one_section_print_in_document_order_and_every_line_is_in_order
    - raf_workplan::test_crate::obligations::wording_is_read_in_every_section_of_an_implementing_capability
    - raf_workplan::test_crate::untouched::a_checkout_with_converted_line_endings_is_untouched_in_every_section
    - raf_workplan::test_crate::untouched::a_default_branch_with_no_common_ancestor_is_an_error
    - raf_workplan::test_crate::untouched::a_prefix_with_a_space_is_an_error_and_a_neighbour_without_one_is_not
    - raf_workplan::test_crate::untouched::a_repository_with_no_remote_at_all_is_an_error_and_not_a_pass
    - raf_workplan::test_crate::untouched::a_root_outside_any_work_tree_is_an_error_at_the_open
    - raf_workplan::test_crate::untouched::a_section_missing_from_the_merge_base_file_counts_as_touched
    - raf_workplan::test_crate::untouched::an_affected_section_the_working_tree_file_lacks_is_an_error
    - raf_workplan::test_crate::untouched::an_unstartable_git_is_an_error_before_the_repository_is_looked_at
    - raf_workplan::test_crate::untouched::contract_effect_none_still_reports_the_other_sections
    - raf_workplan::test_crate::untouched::line_endings_do_not_touch_a_section
    - raf_workplan::test_crate::untouched::stubs::the_blob_the_stub_prints_is_what_is_compared
    - raf_workplan::test_crate::untouched::text_in_a_fence_and_in_a_proposed_block_is_authored
    - raf_workplan::test_crate::untouched::the_default_branch_moving_on_after_the_fork_does_not_touch_the_section
    - raf_workplan::git::tests::a_blob_is_read_from_the_merge_base_in_one_argument
    - raf_workplan::git::tests::a_blob_over_one_mebibyte_is_an_error_and_never_cut
    - raf_workplan::git::tests::a_directory_outside_a_work_tree_has_no_listing
    - raf_workplan::git::tests::a_failing_prefix_question_is_an_error
    - raf_workplan::git::tests::a_missing_default_branch_is_an_error_that_names_the_fix
    - raf_workplan::git::tests::a_path_missing_at_the_merge_base_is_none_and_other_failures_are_errors
    - raf_workplan::git::tests::a_prefix_the_source_path_pattern_cannot_carry_is_refused_with_its_neighbour_accepted
    - raf_workplan::git::tests::a_reference_is_resolved_to_a_commit_after_the_end_of_options
    - raf_workplan::git::tests::a_reference_named_like_an_option_is_one_argument_after_the_end_of_options
    - raf_workplan::git::tests::a_reference_that_resolves_to_nothing_or_to_text_is_an_error
    - raf_workplan::git::tests::a_text_that_is_not_a_version_is_refused
    - raf_workplan::git::tests::a_tracked_path_must_be_utf8_and_a_relative_path_of_the_tree
    - raf_workplan::git::tests::a_version_is_read_from_the_text_git_prints
    - raf_workplan::git::tests::a_version_that_cannot_be_asked_or_read_is_an_error
    - raf_workplan::git::tests::an_answer_that_is_not_a_full_reference_name_never_becomes_one
    - raf_workplan::git::tests::every_other_failure_of_the_listing_is_an_error_and_not_a_walk
    - raf_workplan::git::tests::git_2_29_is_refused_and_2_30_and_2_45_are_accepted
    - raf_workplan::git::tests::histories_without_a_common_ancestor_and_odd_answers_are_errors
    - raf_workplan::git::tests::the_default_branch_is_the_full_reference_name
    - raf_workplan::git::tests::the_merge_base_is_asked_of_head_and_the_commit
    - raf_workplan::git::tests::the_merge_base_of_the_default_branch_is_three_questions_in_order_and_stops_at_the_first_failure
    - raf_workplan::git::tests::the_prefix_is_asked_with_exactly_its_argv_in_the_c_locale
    - raf_workplan::git::tests::the_prefix_is_empty_at_the_top_and_a_path_with_a_slash_below_it
    - raf_workplan::git::tests::the_tracked_paths_are_the_nul_separated_listing_of_the_index
    - raf_workplan::git::tests::the_version_question_is_exactly_git_version_in_the_c_locale
    - raf_workplan::git::tests::versions_order_by_number_and_not_by_text
    - raf_workplan::lifecycle::changed::tests::a_value_over_a_limit_by_less_than_a_float_tells_apart_is_a_change
    - raf_workplan::lifecycle::changed::tests::an_unchanged_or_better_record_owes_nothing
    - raf_workplan::lifecycle::changed::tests::each_change_is_named
    - raf_workplan::lifecycle::tests::an_example_that_cannot_be_started_is_an_error_and_has_no_problem
    - raf_workplan::lifecycle::tests::every_way_a_run_can_fail_is_a_problem_and_a_pass_is_none
    - raf_workplan::lifecycle::tests::the_obligation_names_the_capability_the_examples_section_and_the_binding
    - raf_workplan::obligation::tests::a_failing_run_names_the_binding_and_what_happened_without_a_time_or_output
    - raf_workplan::obligation::tests::a_part_is_ordered_document_then_sections_in_document_order_then_items
    - raf_workplan::obligation::tests::a_phrase_continues_over_one_line_break_and_not_over_anything_else
    - raf_workplan::obligation::tests::all_lists_every_kind_once_in_order_with_unique_kebab_names
    - raf_workplan::obligation::tests::an_empty_list_prints_nothing
    - raf_workplan::obligation::tests::an_unterminated_fence_hides_the_rest_and_does_not_panic
    - raf_workplan::obligation::tests::contract_effect_none_suppresses_section_untouched_for_three_sections_and_nothing_else
    - raf_workplan::obligation::tests::every_phrase_has_a_sentence_that_finds_it_and_every_hit_is_a_phrase
    - raf_workplan::obligation::tests::generated_blocks_are_not_read
    - raf_workplan::obligation::tests::hostile_text_is_read_in_linear_time
    - raf_workplan::obligation::tests::near_misses_are_not_hits
    - raf_workplan::obligation::tests::only_prose_is_read_and_the_hit_names_its_section_and_line
    - raf_workplan::obligation::tests::only_the_marker_name_is_exempt_in_capitals
    - raf_workplan::obligation::tests::only_the_three_kinds_that_start_a_process_are_not_static
    - raf_workplan::obligation::tests::several_hits_of_a_phrase_on_one_line_are_one_hit_and_two_lines_are_two
    - raf_workplan::obligation::tests::text_before_the_first_section_has_no_section
    - raf_workplan::obligation::tests::the_evidence_names_the_thing_it_is_about_in_parentheses
    - raf_workplan::obligation::tests::the_evidence_strings_are_the_evidence_column_of_the_contract
    - raf_workplan::obligation::tests::the_kinds_are_the_thirteen_of_the_contract
    - raf_workplan::obligation::tests::the_list_is_sorted_by_subject_part_kind_line_then_evidence_and_is_without_repeats
    - raf_workplan::obligation::tests::the_trace_evidence_names_the_record_and_what_is_wrong_with_it
    - raf_workplan::obligation::trace::tests::a_value_over_its_limit_is_found_by_its_text_and_not_its_nearest_float
  examples:
    - raf_workplan::Workplan::new_change
    - raf_workplan::Workplan::impact
    - raf_workplan::Workplan::reconcile
    - raf_workplan::lifecycle::section_digest
depends: [cap:workplan-document-model, cap:workplan-generated-docs]
---

# Change reconciliation

## Purpose

Reconciling a change means folding it into the current account of the system. The
operation takes an open change and produces a precise list of what the documentation
still owes, each with the evidence that would settle it. A person or an agent then has
a concrete writing task rather than a vague instruction to update the docs.

## Contract

**`change new <slug> --capability cap:<name>`** creates `work/changes/<slug>.md` with
the identity `change:<slug>`, the named capability in `changes`, `contract_effect:
changes`, an empty `items` list, a title and a description for the author to replace,
and a `## Summary` stub. It fails when the slug is not kebab case, the file exists, or
the capability does not exist or its document did not load. Repeating `--capability`
lists several capabilities, and one given twice is listed once. Two further options
shape the record:

- `--affects cap:<name>#<section>`, repeatable, names a section the change affects.
  Each entry names a listed capability and one of the seven sections, or the command
  fails. Without the option the record affects `cap:<name>#contract` for each listed
  capability.
- `--copyright <text>` and `--licence <id>`, given together or not at all, open the
  frontmatter with the two SPDX comment lines, so a repository that checks the
  licence of every file accepts the record as written. A value that is empty or not
  one line of text is refused.

`change new` writes exactly that one file, created exclusively and never over an
existing file or link, and reads it back through the loader; a record that has a
finding is removed again and reported. It then prints the PROPOSED block template,
`> [!PROPOSED change:<slug>]` and one placeholder line, for the author to place in a
section named in `affects`, and the state guidance for each listed capability: a
`delivered` or `designed` capability is set to `state: implementing` in the same
commit as the block, because a PROPOSED block is a `check` finding in both states (and
beside a `delivered` capability the open record is one too); a `considered` or
`implementing` capability needs no change of state. When the new record makes a
generated block stale (the relationship tables name the open changes that list a
capability), it also prints the two steps that bring the blocks up to date:
`git add work/changes/<slug>.md`, then `docs sync`. Until the record is tracked,
`check` reports the link to it as `link-broken`.

**`change impact <slug>`** reads the record and the graph and prints, without
modifying anything: the capabilities in `changes`; the sections in `affects`; the
bindings of those capabilities; the capabilities that depend on them and the sections
that link to the affected sections, because those readers are affected too.

**`change reconcile <slug>`** prints the obligation list. An obligation is a
statement of work the documentation owes, with a kind, a location and the evidence
needed. The kinds are:

| Kind | Raised when | Evidence that resolves it |
| --- | --- | --- |
| `unresolved-proposal` | A PROPOSED block names the change | The block is replaced by plain contract text, or removed because the claim was dropped |
| `item-without-outcome` | An item has no `outcome` | An outcome of `implemented`, `dropped` or `moved:change:<slug>` |
| `binding-missing` | A capability the change lists binds nothing where it should, or a binding does not resolve | A binding that resolves through the source index |
| `section-untouched` | A section in `affects` has the same text as at the merge base | A diff of that section against the merge base, or an edit that records why none is needed |
| `example-failing` | A bound example target or documentation test of a capability the change lists fails to build or run | A passing `verify` run for the change, or a binding edit that removes the broken example |
| `generated-stale` | A generated block differs from what `docs sync` writes | A `docs sync` run |
| `tense-mismatch` | Delivered or implementing contract text uses proposal or forward-looking wording, or a delivered capability has a PROPOSED block | Text rewritten as the supported behaviour |

> [!PROPOSED change:workplan-reconcile]
> Six kinds follow `tense-mismatch` in the table, and in the order of the list:
>
> | Kind | Raised when | Evidence that resolves it |
> | --- | --- | --- |
> | `claim-unsupported` | A claim of a listed capability names an evidence record that does not load, whose verdict is `fail`, or that holds a value over its limit | A passing record within its limits for each evidence id the claim names, or the claim reworded or removed |
> | `evidence-stale` | An input of an evidence record that a listed capability's `for` list or claims reach has another digest now than the one the record keeps, or no longer resolves | The evidence recorded again with `evidence record` |
> | `evidence-changed` | An evidence record a claim of a listed capability names passed at the merge base and now fails, holds a value over a limit (its own or the one at the merge base), has lost a limit it had there, or no longer loads | The claim reviewed against the new result: the evidence passes within the limits of the merge base again, or the claim no longer names it |
> | `assessment-missing` | An authored section in `affects` has no assessment | An assessment of the section recorded with `assess` |
> | `assessment-stale` | An input of the assessment of an authored section in `affects` has another digest now than the one the record keeps, no longer resolves, is no longer an input, or is an input now and was not then | The section assessed again with `assess` |
> | `assessment-rejected` | The assessment of an authored section in `affects` is current and its verdict is `rejected` | The section revised and assessed again with `assess`, with the verdict `accepted` |
>
> `claim-unsupported` and `evidence-changed` are owed at the line of the claim
> marker, in the section that holds it, once for each record the marker names
> (and, for `evidence-changed`, once for each change). `evidence-stale` is owed
> at the record, once for each input that moved, and names the input; it compares
> the digest of the text of each input symbol now with the digest the record
> keeps, so it reads no revision and asks Git nothing. A removed record is
> both `claim-unsupported` and `evidence-changed`. The evidence names the record
> and what is wrong, for example `` (`evidence:memory-bound`: `peak` is 80, over
> its limit 64) ``.
>
> The three assessment kinds are owed in the affected section, as the
> [prose assessment](./workplan-prose-assessment.md) defines the record and its
> inputs: `assessment-missing` when no record has the id
> `assessment:<capability slug>-<section>`, `assessment-stale` once for each
> input that moved, naming it, for example `` (`assessment:streaming-parser-usage`:
> the symbol `raf_parser::Parser::push` has changed since) ``, and
> `assessment-rejected` for a current assessment with the verdict `rejected`.
> Every section of a capability that loads holds authored text, since the profile
> refuses one with nothing outside its generated blocks and comments
> (`okf-section-empty`), and a listed capability that does not load is an error.
> They compare digests the tool computes
> from the working tree, so they read no revision and ask Git nothing.

Each obligation line gives its subject (a capability, the change record or the
capability index), the part it is about (the document, a section or an item of the
record), the kind and the evidence, and for text findings `line <n>:` before the
evidence. The list is sorted by subject (the capabilities, then the change record,
then the index, each by id), part (the document, then the seven sections in the order
a capability holds them, then the items), kind (in the order of the table), line and
evidence text, so equal inputs print equal output. The exit code is 0 when the list
is empty and 1 when it is not.

**Merge base.** `section-untouched` compares each affected section in the working
tree against the version of the file at the merge base of `HEAD` and the default
branch, found by running `git`. A section is touched when its authored text differs:
its heading span, with the content of every generated block left out on both sides
and line endings read as LF, so a regenerated table is not an edit, while marker
lines, fenced code and PROPOSED blocks are authored text. A file that does not exist
at the merge base, or that has no such section there, counts as touched, and a file
there that does not parse is an error. When `HEAD` is the default branch's commit,
the merge base is `HEAD` itself and every affected section is reported.

**Invocation.** `reconcile` starts `git` directly with no shell, with arguments as an
argv vector, and takes no document text onto a command line. It first runs
`git --version` and exits 2 when Git is older than 2.30, the first release
whose `rev-parse` handles `--end-of-options`. It then asks once where the workspace
root is below the top of its work tree, and opens the repository, which lists the
tracked paths as the gate of the [document model](./workplan-document-model.md)
does. The commands are, in order:

```text
git --version
git rev-parse --show-prefix
git rev-parse --is-inside-work-tree
git ls-files -z
git symbolic-ref refs/remotes/origin/HEAD
git rev-parse --verify --end-of-options <default-ref>^{commit}
git merge-base HEAD <default-commit>
git cat-file blob <merge-base>:<prefix><path>
```

`<default-ref>` is the full ref name that `git symbolic-ref` prints, such as
`refs/remotes/origin/main`, so a local branch named `origin/main` cannot be mistaken
for it. The remote is assumed to be named `origin`. A repository where it fails has no
default branch, which is an error and not a silent pass. The ref is resolved to a
commit id by `git rev-parse --verify` and every later command receives that id, so a branch
named like an option cannot reach `git` as an option. In the last command the path
follows `<merge-base>:` inside one argument, so it cannot be read as an option.
`<path>` is a repository-relative document path from the loaded document set, and
`<prefix>` is what `git rev-parse --show-prefix` printed: empty at the top of the
work tree, or the directory of the workspace root below it, ending in `/`, so a
workspace below the top of its repository reads the file it means. A prefix that does
not match the relative-source-path pattern of the
[completion gate](./workplan-complete-gate.md) is an error, and a linked work tree is
its own top. The output of `cat-file blob` is read up to the 1 MiB document bound, and
a larger blob is an error. The standard output and the standard error of every other
`git` command are each read up to 16 MiB, and a command that writes more or runs past
300 seconds is killed and is an error.

> [!PROPOSED change:workplan-reconcile]
> `evidence-changed` reads, with the same `git cat-file blob
> <merge-base>:<prefix><path>` row and the same merge base, the record
> `work/evidence/<slug>.md` of each evidence id a claim of a listed capability
> names, once per record; `<slug>` is the id of the claim marker, which the profile
> holds to the slug pattern. The merge base is asked for once, and only when a
> section of `affects` or a claim needs it. A record that is not there at the merge
> base, or whose verdict there is `fail`, is compared with nothing; a record there
> that does not read as an evidence record is an error, and so is one over the
> 1 MiB bound.

`example-failing` builds and runs bound examples with the `--frozen` argv forms, the
patterns, the stdin, timeout and output rules, and the proof-that-it-ran rule of the
[completion gate](./workplan-complete-gate.md), for bindings that resolve in the
[source index](./source-index.md). The program is `$CARGO` when set and `cargo` from
`PATH` otherwise. The processes run the workspace's cargo and Git configuration,
which is trusted build input (see [the security policy](../../SECURITY.md)).

**`contract_effect: none`.** For a change with this effect, the obligation list is
lighter. `section-untouched` is not raised for `contract`, `usage` and `examples`,
since the change declares they are unaffected. All other kinds apply, because stale
bindings and stale generated blocks are errors regardless of the contract.

> [!PROPOSED change:workplan-reconcile]
> The assessment kinds are among the kinds that apply: a section the change lists
> in `affects` is read by the people the change reaches whether or not its
> contract moved, so its assessment is owed under either effect.

## Usage

```sh
cargo workplan change new incremental-parsing --capability cap:streaming-parser \
  --affects cap:streaming-parser#contract --affects cap:streaming-parser#usage
cargo workplan change impact incremental-parsing
cargo workplan change reconcile incremental-parsing
```

`change new` prints the PROPOSED block to place and the state each listed capability
needs. An agent receives the output of `reconcile`, writes the prose each line asks
for, runs `docs sync`, and runs `reconcile` again until the list is empty.

## Examples

Output of `reconcile` for a change that renamed an API, left the Usage section as it
was at the merge base, and still has an open proposal:

```text
cap:streaming-parser  contract  unresolved-proposal  line 31: the block is replaced by plain contract text, or removed because the claim was dropped
cap:streaming-parser  usage  section-untouched  a diff of that section against the merge base, or an edit that records why none is needed
cap:streaming-parser  verification  binding-missing  a binding that resolves through the source index (bindings.code `raf_parser::Parser::push` does not resolve: `raf_parser::Parser::push` names nothing in the workspace)
change:incremental-parsing  item 2  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
```

The `binding-missing` line shows the shape of every obligation: the capability, the
section and the evidence that settles it. A bound example that stops compiling adds a
line of kind `example-failing` for the capability in the same form.

## Constraints

- `reconcile` and `impact` never write a file in the repository. Only `change new`
  writes, and it writes one file.
- The merge base comes from `git`; a repository with no default branch reachable from
  `HEAD` makes `section-untouched` an error and not a silent pass.
- Tense findings come from a fixed list of phrases (for example `will`, `is planned`
  and `proposed`) and are reported as findings, not as proof. The list is defined in
  the tool and applies outside code fences, code spans, HTML comments, link
  destinations, PROPOSED blocks and generated blocks; a phrase matches whole words,
  and `PROPOSED` in capitals is the name of the marker and not wording.
- `example-failing` builds and runs bound examples with `cargo`, which is the one
  process `reconcile` starts besides `git`; it still writes no file in the
  repository.
- No `git` or `cargo` process receives document text except a validated path inside
  a `<commit>:<path>` argument, or a binding that resolved in the source index.
- An obligation never says "update the docs". It names the section and the evidence.

## Explanation

The failure this prevents is the reader who must apply amendments mentally. The
current text must state the whole contract that applies, so accepting a change cannot
end at appending "supersedes" to an old statement. An amendment can be an input to
reconciliation, but it must not become a standing instruction that every subsequent
reader has to remember.

A command that accepts a change therefore identifies and updates the affected current
statements. Reconciliation may revise a contract, remove an obsolete constraint,
change an example, adjust a code binding and invalidate some evidence. Those are
consequences of one change, and the tool treats them together.

Mechanical generation and explanatory writing have different jobs. The crate can find
what is wrong, and it can regenerate tables. Explanations and instructional prose need
authorship and assessment. The obligation list is the interface between the two: it is
specific enough to be a writing task, such as "the final API differs from the proposal
in these two places, so update the usage example and the lifecycle explanation".

## Verification

- A `change new` fixture asserts the file's keys, id and Summary, and the failures for
  a bad slug, an existing file and a missing capability.
- A `change new` fixture for the options: without `--affects` the record affects the
  contract of each listed capability, the entries given are written in order, and an
  entry that names an unlisted capability or an unknown section is refused; both SPDX
  options write the two comment lines, and one without the other is refused.
- The printed template placed in a capability of each of the four states, after the
  printed state guidance, gives exactly one PROPOSED block, a clean `check` and one
  `unresolved-proposal` line; and after a record that stales a generated block, the
  printed steps end in a clean `check`.
- A `change impact` fixture asserts the printed capabilities, sections, bindings and
  dependants, and that no file changed.
- One `reconcile` fixture per obligation kind in a temporary Git repository, each
  asserting the kind, location and evidence text, and that fixing it removes the line.
  The `example-failing` fixture binds an example that does not compile, asserts the
  line, and asserts that the line is gone once the example builds and runs.
- A merge-base fixture where an affected section is edited on a branch and is not
  reported, and one where it is unchanged and is reported; one where only the content
  of a generated block changed is reported too, and a workspace below the top of its
  repository is compared at its prefix.
- An invocation fixture with stub `git` and `cargo` on `PATH` records each argv and
  asserts the forms above, and a default branch named `--output=x` is resolved to a
  commit id and never reaches `git` as an option.
- A Git version fixture: a stub `git --version` reporting 2.29 makes `reconcile` exit
  2 before any other process starts, and a repository with no
  `refs/remotes/origin/HEAD` is an error.
- A default-branch fixture with a local branch literally named `origin/main` beside
  `refs/remotes/origin/main` resolves the remote-tracking commit.
- A blob over 1 MiB at the merge base is an error and not a truncated read.
- An `example-failing` fixture binding a function that is not a test shows that zero
  tests run is reported as a failing binding.
- A `contract_effect: none` fixture asserting the lighter list.
- A determinism check: two runs print identical output, and the exit code is 0 only
  for an empty list.
- Scenario A and Scenario B are described in
  [the completion gate](./workplan-complete-gate.md) and use `reconcile` at each step.

> [!PROPOSED change:workplan-reconcile]
> - One fixture per new kind: a claim on a missing, a failing and an over-limit
>   record is `claim-unsupported`, and a passing record clears it; an edit of a
>   bound symbol makes its record `evidence-stale` naming the symbol, while an edit
>   of another item and a CRLF-only change do not; and in a Git repository a
>   record replaced by a failing one, by one over its limit, by one without a
>   limit of the merge base, or removed, is `evidence-changed` at the claim, beside
>   a record that did not change and owes nothing.
> - The completion gate refuses each of the three kinds and completes once it is
>   repaired.
> - One fixture per assessment kind: an affected section with no assessment is
>   `assessment-missing`; an edit of the section, of a symbol it names, of a
>   bound symbol when it names none, and of an evidence record a claim of it
>   names each make its assessment `assessment-stale` naming that input, while a
>   regenerated block and an edit of another section do not; a current
>   `rejected` assessment is `assessment-rejected`, whether a human or an agent
>   gave it; `contract_effect: none` keeps them; and the completion gate refuses
>   each of the three and completes once the section is assessed and accepted.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::test_crate::new::the_printed_steps_end_in_a_clean_check_and_skipping_the_stage_step_does_not` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_git_older_than_2_30_is_exit_two_before_any_other_process` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::a_list_with_obligations_is_printed_one_line_each_and_exits_one` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::an_empty_list_prints_nothing_and_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::fixing_an_obligation_removes_its_line_and_only_its_line` | test | [Verification](#verification) |
| `raf_workplan::Workplan::impact` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::new_change` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::reconcile` | doctest | [Verification](#verification) |
| `raf_workplan::git::tests::a_blob_is_read_from_the_merge_base_in_one_argument` | test | [Verification](#verification) |
| `raf_workplan::git::tests::a_blob_over_one_mebibyte_is_an_error_and_never_cut` | test | [Verification](#verification) |
| `raf_workplan::git::tests::a_directory_outside_a_work_tree_has_no_listing` | test | [Verification](#verification) |
| `raf_workplan::git::tests::a_failing_prefix_question_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::git::tests::a_missing_default_branch_is_an_error_that_names_the_fix` | test | [Verification](#verification) |
| `raf_workplan::git::tests::a_path_missing_at_the_merge_base_is_none_and_other_failures_are_errors` | test | [Verification](#verification) |
| `raf_workplan::git::tests::a_prefix_the_source_path_pattern_cannot_carry_is_refused_with_its_neighbour_accepted` | test | [Verification](#verification) |
| `raf_workplan::git::tests::a_reference_is_resolved_to_a_commit_after_the_end_of_options` | test | [Verification](#verification) |
| `raf_workplan::git::tests::a_reference_named_like_an_option_is_one_argument_after_the_end_of_options` | test | [Verification](#verification) |
| `raf_workplan::git::tests::a_reference_that_resolves_to_nothing_or_to_text_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::git::tests::a_text_that_is_not_a_version_is_refused` | test | [Verification](#verification) |
| `raf_workplan::git::tests::a_tracked_path_must_be_utf8_and_a_relative_path_of_the_tree` | test | [Verification](#verification) |
| `raf_workplan::git::tests::a_version_is_read_from_the_text_git_prints` | test | [Verification](#verification) |
| `raf_workplan::git::tests::a_version_that_cannot_be_asked_or_read_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::git::tests::an_answer_that_is_not_a_full_reference_name_never_becomes_one` | test | [Verification](#verification) |
| `raf_workplan::git::tests::every_other_failure_of_the_listing_is_an_error_and_not_a_walk` | test | [Verification](#verification) |
| `raf_workplan::git::tests::git_2_29_is_refused_and_2_30_and_2_45_are_accepted` | test | [Verification](#verification) |
| `raf_workplan::git::tests::histories_without_a_common_ancestor_and_odd_answers_are_errors` | test | [Verification](#verification) |
| `raf_workplan::git::tests::the_default_branch_is_the_full_reference_name` | test | [Verification](#verification) |
| `raf_workplan::git::tests::the_merge_base_is_asked_of_head_and_the_commit` | test | [Verification](#verification) |
| `raf_workplan::git::tests::the_merge_base_of_the_default_branch_is_three_questions_in_order_and_stops_at_the_first_failure` | test | [Verification](#verification) |
| `raf_workplan::git::tests::the_prefix_is_asked_with_exactly_its_argv_in_the_c_locale` | test | [Verification](#verification) |
| `raf_workplan::git::tests::the_prefix_is_empty_at_the_top_and_a_path_with_a_slash_below_it` | test | [Verification](#verification) |
| `raf_workplan::git::tests::the_tracked_paths_are_the_nul_separated_listing_of_the_index` | test | [Verification](#verification) |
| `raf_workplan::git::tests::the_version_question_is_exactly_git_version_in_the_c_locale` | test | [Verification](#verification) |
| `raf_workplan::git::tests::versions_order_by_number_and_not_by_text` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::changed::tests::a_value_over_a_limit_by_less_than_a_float_tells_apart_is_a_change` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::changed::tests::an_unchanged_or_better_record_owes_nothing` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::changed::tests::each_change_is_named` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::section_digest` | doctest | [Verification](#verification) |
| `raf_workplan::lifecycle::tests::an_example_that_cannot_be_started_is_an_error_and_has_no_problem` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::tests::every_way_a_run_can_fail_is_a_problem_and_a_pass_is_none` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::tests::the_obligation_names_the_capability_the_examples_section_and_the_binding` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::a_failing_run_names_the_binding_and_what_happened_without_a_time_or_output` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::a_part_is_ordered_document_then_sections_in_document_order_then_items` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::a_phrase_continues_over_one_line_break_and_not_over_anything_else` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::all_lists_every_kind_once_in_order_with_unique_kebab_names` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::an_empty_list_prints_nothing` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::an_unterminated_fence_hides_the_rest_and_does_not_panic` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::contract_effect_none_suppresses_section_untouched_for_three_sections_and_nothing_else` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::every_phrase_has_a_sentence_that_finds_it_and_every_hit_is_a_phrase` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::generated_blocks_are_not_read` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::hostile_text_is_read_in_linear_time` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::near_misses_are_not_hits` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::only_prose_is_read_and_the_hit_names_its_section_and_line` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::only_the_marker_name_is_exempt_in_capitals` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::only_the_three_kinds_that_start_a_process_are_not_static` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::several_hits_of_a_phrase_on_one_line_are_one_hit_and_two_lines_are_two` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::text_before_the_first_section_has_no_section` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::the_evidence_names_the_thing_it_is_about_in_parentheses` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::the_evidence_strings_are_the_evidence_column_of_the_contract` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::the_kinds_are_the_thirteen_of_the_contract` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::the_list_is_sorted_by_subject_part_kind_line_then_evidence_and_is_without_repeats` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::the_trace_evidence_names_the_record_and_what_is_wrong_with_it` | test | [Verification](#verification) |
| `raf_workplan::obligation::trace::tests::a_value_over_its_limit_is_found_by_its_text_and_not_its_nearest_float` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::each_kind_of_obligation_is_a_reconcile_line_in_a_git_repository_and_fixing_it_removes_the_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::a_record_at_the_merge_base_that_does_not_read_is_an_error_and_not_a_pass` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::a_record_that_failed_at_the_merge_base_or_was_not_there_is_compared_with_nothing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::a_stale_record_of_a_capability_the_change_does_not_list_is_not_owed` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::an_edit_of_a_bound_symbol_makes_its_record_stale_and_nothing_else_does` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::each_change_of_a_passing_record_since_the_merge_base_reopens_its_claim` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::each_named_record_is_read_once_at_the_merge_base_with_the_blob_row` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::with_no_default_branch_a_claim_is_an_error_and_no_claim_asks_git_nothing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_blob_is_returned_byte_for_byte` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_blob_over_one_mebibyte_is_an_error_and_exactly_one_mebibyte_is_read` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_default_branch_named_like_an_option_is_a_ref_and_never_an_option` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_directory_is_an_error_and_not_an_absent_file` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_directory_that_is_no_work_tree_has_no_listing_and_a_broken_index_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_file_is_read_as_it_was_at_the_merge_base` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_linked_work_tree_asks_about_its_own_head_and_shares_the_remote_refs` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_linked_work_tree_is_its_own_top` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_local_branch_named_origin_main_does_not_stand_in_for_the_remote_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_program_that_does_not_exist_is_an_error_and_not_a_fallback` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_repository_without_origin_head_has_no_default_branch_and_the_fix_is_named` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::histories_with_no_common_ancestor_have_no_merge_base` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::stubs::a_stub_that_exits_with_a_failure_is_an_error_with_its_message` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::stubs::an_answer_that_is_not_a_full_reference_name_stops_the_chain_before_any_commit_is_asked_for` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::stubs::every_question_reaches_git_as_exactly_its_argv_in_the_work_directory` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::stubs::the_version_gate_over_a_real_process_refuses_2_29_and_accepts_2_30` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::the_default_branch_of_a_clone_is_the_full_remote_tracking_name` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::the_installed_git_is_new_enough` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::the_merge_base_is_where_the_branch_left_the_default_branch` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::the_prefix_is_where_the_directory_is_below_the_top_of_its_work_tree` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::the_tracked_paths_are_the_index_and_not_the_files_on_disk` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::the_tracked_paths_of_a_subdirectory_are_relative_to_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::a_bad_slug_is_refused_before_anything_is_written_and_a_good_one_next_to_it_is_not` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::a_capability_that_does_not_exist_is_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::a_capability_whose_document_did_not_load_is_named_and_never_skipped` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::a_file_or_a_link_where_work_changes_belongs_is_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::a_link_at_the_path_is_an_existing_file_and_is_not_followed` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::a_slug_that_reads_as_a_tracker_reference_leaves_no_file` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::an_affects_entry_must_name_a_listed_capability_and_one_of_the_seven_sections` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::an_existing_file_is_refused_and_left_byte_identical` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::an_spdx_field_that_is_not_one_line_of_text_is_refused_and_nothing_is_written` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::both_spdx_fields_open_the_record_with_the_two_comment_lines_of_the_records_of_the_repository` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::explicit_affects_are_written_as_given_and_impact_reports_them_sorted` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::impact_gives_the_capabilities_sections_bindings_dependants_and_linking_sections` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::impact_of_a_change_that_just_was_created_reads_the_record_from_the_disk` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::impact_of_a_change_that_reaches_nothing_says_none` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::impact_of_the_repositorys_own_change_lists_what_the_record_says` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::impact_prints_each_part_in_a_fixed_order` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::impact_tells_an_unknown_slug_from_a_record_that_did_not_load` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::neither_spdx_field_writes_no_comment_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::new_writes_one_file_with_the_keys_the_id_and_a_summary_and_it_loads_clean` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::one_spdx_field_without_the_other_is_an_error_that_names_the_missing_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::several_capabilities_are_listed_in_the_order_given_without_repeats` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_bad_slug_message_says_what_a_slug_is` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_default_affects_is_the_contract_of_each_listed_capability` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_guidance_follows_the_state_of_each_listed_capability` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_guidance_for_a_delivered_capability_names_both_findings_it_avoids` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_printed_text_holds_the_template_and_the_guidance` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_record_beside_a_delivered_capability_is_a_finding_until_the_state_is_set` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_sync_note_is_there_when_the_target_file_was_already_stale` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_template_in_a_capability_of_each_state_gives_one_block_and_a_clean_check` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_template_in_the_wrong_state_is_the_finding_the_guidance_names` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::work_changes_is_created_when_the_repository_has_none` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::a_block_of_another_change_in_a_capability_that_does_not_load_is_not_owed` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::a_capability_over_the_size_bound_is_not_loaded_and_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::a_delivered_capability_owes_code_and_tests_but_not_examples` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::a_delivered_capability_that_breaks_another_rule_too_is_not_read_as_unbound` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::a_hundred_thousand_lines_are_a_hundred_thousand_obligations_in_order` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::a_listed_capability_that_is_not_loaded_is_an_error_and_not_a_skipped_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::a_very_long_line_is_one_obligation_and_is_read_in_linear_time` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::an_unknown_change_is_not_a_change_that_did_not_load` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::an_unterminated_fence_hides_the_rest_of_the_section` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::an_unterminated_fence_in_an_earlier_section_hides_the_headings_after_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::capabilities_that_bind_nothing_owe_no_binding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::contract_effect_none_keeps_every_obligation_the_documents_can_show` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::contract_effect_none_keeps_the_assessments_and_another_section_is_not_judged` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::each_assessment_fault_is_owed_at_the_section_naming_the_input_and_a_current_acceptance_clears_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::each_fault_is_owed_with_its_kind_location_and_evidence_and_fixing_it_removes_the_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::equal_inputs_print_equal_output` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::every_static_kind_is_raised_by_a_fixture_and_the_kinds_that_need_a_process_by_none` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::fixing_one_fault_removes_exactly_its_lines` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::moves_to_unknown_records_cost_the_same_with_few_records_as_with_many` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::needed_is_a_typed_value_whose_text_starts_with_the_contract_string` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::only_the_capabilities_the_change_lists_are_judged` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::reconcile_writes_nothing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::text_of_a_proposed_block_is_not_read_for_wording` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::text_that_is_read_as_wording_at_a_hostile_size_does_not_hang` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::the_baseline_owes_nothing_and_check_agrees` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::the_binding_obligations_are_the_gaps_of_the_roadmap` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::the_change_is_found_with_or_without_its_prefix` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::the_proposals_are_those_of_the_graph_for_this_change_only` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::two_blocks_in_one_section_print_in_document_order_and_every_line_is_in_order` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::wording_is_read_in_every_section_of_an_implementing_capability` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::wording_that_only_looks_like_a_phrase_owes_nothing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::planning_produces_the_documentation` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::several_redesigns_leave_one_readable_present` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_checkout_with_converted_line_endings_is_untouched_in_every_section` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_default_branch_named_like_an_option_is_resolved_and_never_reaches_git_as_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_default_branch_with_no_common_ancestor_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_file_absent_at_the_merge_base_counts_as_touched` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_file_at_the_merge_base_that_does_not_parse_is_an_error_and_not_a_pass` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_file_over_one_mebibyte_at_the_merge_base_is_an_error_and_exactly_one_is_read` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_linked_work_tree_compares_its_own_head` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_local_branch_named_origin_main_does_not_stand_in_for_the_remote_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_prefix_with_a_space_is_an_error_and_a_neighbour_without_one_is_not` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_repository_with_no_remote_at_all_is_an_error_and_not_a_pass` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_root_outside_any_work_tree_is_an_error_at_the_open` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_section_edited_on_the_branch_is_not_reported_and_its_neighbour_is` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_section_missing_from_the_merge_base_file_counts_as_touched` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_workspace_below_the_top_of_the_repository_is_compared_at_its_prefix` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::an_affected_section_the_working_tree_file_lacks_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::an_edit_that_is_not_committed_counts_because_the_working_tree_is_compared` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::an_unchanged_affected_section_is_owed_with_the_line_the_contract_shows` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::an_unstartable_git_is_an_error_before_the_repository_is_looked_at` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::contract_effect_none_leaves_out_contract_usage_and_examples_and_asks_git_for_nothing_then` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::contract_effect_none_still_reports_the_other_sections` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::git_2_29_stops_the_open_before_any_other_process_and_2_30_goes_on` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::line_endings_do_not_touch_a_section` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::no_default_branch_is_an_error_that_names_the_fix_and_the_fix_clears_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::on_the_default_branch_itself_every_affected_section_is_owed_and_it_is_not_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::only_a_generated_block_changed_is_still_reported` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::reconcile_writes_nothing_and_prints_the_same_list_twice` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::stubs::a_prefix_the_source_path_pattern_cannot_carry_is_an_error_and_not_a_guess` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::stubs::a_root_below_the_top_reads_each_file_at_the_prefix` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::stubs::a_stub_git_2_29_runs_one_process_and_exits_with_the_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::stubs::reconcile_asks_git_exactly_the_commands_of_the_contract_in_order` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::stubs::the_blob_the_stub_prints_is_what_is_compared` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::text_in_a_fence_and_in_a_proposed_block_is_authored` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::the_default_branch_moving_on_after_the_fork_does_not_touch_the_section` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::the_digest_leaves_out_generated_blocks_on_both_sides_and_nothing_else` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::a_binding_to_a_function_with_no_test_reports_zero_tests_as_failing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::an_example_that_does_not_build_is_a_line_and_is_gone_once_it_builds_and_runs` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::test_bindings_are_not_run_by_reconcile_and_unresolved_examples_are_not_started` | test | [Verification](#verification) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [workplan-document-model](./workplan-document-model.md) |
| Depends on | [workplan-generated-docs](./workplan-generated-docs.md) |
| Depended on by | [workplan-cli](./workplan-cli.md) |
| Depended on by | [workplan-complete-gate](./workplan-complete-gate.md) |
| Depended on by | [workplan-evidence-claims](./workplan-evidence-claims.md) |
| Changed by | [workplan-reconcile](../../work/changes/workplan-reconcile.md) |
<!-- raf:generated:end -->
