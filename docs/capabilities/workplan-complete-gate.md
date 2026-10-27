---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-complete-gate
title: "Completion gate"
description: "change complete and verify: the six completion conditions, lighter obligations for changes that leave the contract alone, deletion of the record, and the acceptance scenarios."
state: delivered
bindings:
  code:
    - raf_workplan::Gate::open
    - raf_workplan::Gate::open_with_runner
    - raf_workplan::Gate::tracked_list
    - raf_workplan::Gate::capture_request
    - raf_workplan::Gate::open_captured
    - raf_workplan::Opened::workplan_after
    - raf_workplan::exec::view
    - raf_workplan::lifecycle::Lifecycle::open_with
    - raf_workplan::lifecycle::Lifecycle::complete
    - raf_workplan::lifecycle::Completion
    - raf_workplan::lifecycle::Failure
    - raf_workplan::lifecycle::Condition
    - raf_workplan::lifecycle::verify
    - raf_workplan::lifecycle::VerifyReport
    - raf_workplan::bound::Executor
    - raf_workplan::bound::Evidence
    - raf_workplan::exec::Invocation
    - raf_workplan::exec::Pattern
    - raf_workplan::exec::Programs
    - raf_workplan::exec::Limits
    - raf_workplan::exec::SystemRunner
    - raf_workplan::exec
    - raf_workplan::bound
    - raf_workplan::lifecycle::Lifecycle
    - raf_workplan::staged
    - raf_workplan::git::head
  tests:
    - raf_workplan::test_crate::gate::the_programs_of_a_gate_are_started_in_its_directory
    - raf_workplan::test_crate::gate::the_tracked_paths_a_caller_gives_replace_the_listing_and_start_no_process
    - rust_agentic_foundations::test_crate::repo_gates::time_in_crates_tracked_files_name_no_clock
    - rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_a_planted_clock_reading_with_file_and_line
    - raf_workplan::bound::tests::platform_exclusion_is_distinct_from_a_pass_and_reuses_the_observation
    - raf_workplan::bound::tests::platform_unknown_or_applicable_still_requires_a_listed_test
    - raf_workplan::bound::tests::malformed_compiler_configuration_fails_without_a_host_fallback
    - raf_workplan::test_crate::bound::real_cargo_platform_exclusion_preserves_a_passing_neighbour
    - raf_workplan::bound::tests::a_platform_guarded_doc_binding_uses_rustdoc_listing_not_rustc_configuration
    - raf_workplan::exec::tests::compiler_platform_queries_pin_the_package_and_target
    - raf_workplan::test_crate::complete::completion_retains_the_justified_host_nonexecution_receipt
    - raf_workplan::test_crate::verify::a_known_opposite_host_binding_is_reported_without_a_passing_execution_proof
    - raf_workplan::test_crate::evidence::an_opposite_host_binding_cannot_publish_execution_evidence
    - cargo_workplan::test_crate::verify::host_nonapplicability_is_exit_zero_and_preserved_in_machine_output
    - raf_workplan::test_crate::complete::a_none_change_reads_each_file_of_the_bound_package_once_at_the_merge_base_and_otherwise_none
    - raf_workplan::test_crate::complete::a_none_change_that_edits_a_bound_signature_refuses
    - raf_workplan::test_crate::complete::an_edit_of_a_bound_body_alone_refuses
    - raf_workplan::test_crate::complete::an_edit_of_a_bound_doc_comment_refuses
    - raf_workplan::test_crate::complete::an_item_dropped_without_a_reason_refuses_condition_two
    - raf_workplan::test_crate::complete::an_untouched_usage_or_examples_of_a_none_change_refuses
    - raf_workplan::bound::tests::a_kill_at_either_limit_fails_the_binding_with_the_time_it_ran_and_its_limit
    - raf_workplan::test_crate::complete::a_section_with_three_moved_inputs_is_one_section_in_the_refusal_headline
    - raf_workplan::lifecycle::completion::tests::a_section_with_three_moved_inputs_is_one_section_in_the_headline
    - raf_workplan::test_crate::complete::a_linked_work_directory_is_refused_before_anything_runs_and_a_real_one_completes
    - raf_workplan::test_crate::complete::a_record_linked_while_the_tests_ran_is_refused_at_the_second_judgement
    - raf_workplan::test_crate::scenarios::several_redesigns_leave_one_readable_present
    - raf_workplan::test_crate::scenarios::planning_produces_the_documentation
    - raf_workplan::test_crate::scenarios::a_document_that_merely_exists_does_not_satisfy_the_scenario
    - raf_workplan::test_crate::scenarios::a_thin_document_is_not_the_account_the_scenario_builds
    - raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal
    - raf_workplan::test_crate::complete::the_cases_cover_every_kind_of_obligation_in_both_directions
    - raf_workplan::test_crate::complete::each_kind_of_obligation_makes_complete_refuse_and_leaves_the_tree_as_it_was_and_its_repair_completes
    - raf_workplan::test_crate::complete::an_item_without_outcome_and_a_move_to_nowhere_or_to_itself_refuse_condition_two
    - raf_workplan::test_crate::complete::a_proposed_block_of_the_change_refuses_condition_three_in_a_loaded_and_in_an_unloaded_document
    - raf_workplan::test_crate::complete::a_block_that_is_stale_or_missing_anywhere_refuses_condition_four
    - raf_workplan::test_crate::complete::a_failing_bound_test_refuses_condition_five_alone
    - raf_workplan::test_crate::complete::a_failing_bound_example_stands_under_conditions_one_and_five
    - raf_workplan::test_crate::complete::a_failing_doc_test_a_test_that_runs_nothing_and_an_ignored_test_refuse_beside_passing_neighbours
    - raf_workplan::test_crate::complete::an_item_without_outcome_and_a_stale_block_print_the_counts_of_the_contract
    - raf_workplan::test_crate::complete::a_capability_left_implementing_with_no_other_record_refuses_and_says_what_to_change
    - raf_workplan::test_crate::complete::a_delivered_capability_beside_its_open_record_is_what_complete_resolves
    - raf_workplan::test_crate::complete::a_delivered_capability_with_a_binding_that_does_not_resolve_refuses
    - raf_workplan::test_crate::complete::empty_bindings_satisfy_condition_five_only_while_the_capability_is_designed_or_considered
    - raf_workplan::test_crate::complete::contract_effect_none_leaves_only_contract_untouched_and_preserves_other_refusals
    - raf_workplan::test_crate::complete::hostile_bindings_refuse_and_never_reach_cargo
    - raf_workplan::test_crate::complete::a_static_failure_spends_no_process_on_cargo_and_a_clean_change_runs_cargo_after_the_static_checks
    - raf_workplan::test_crate::complete::a_document_edited_while_the_tests_ran_refuses_and_nothing_is_written
    - raf_workplan::test_crate::complete::a_binding_added_while_the_tests_ran_is_a_change_even_when_every_condition_still_holds
    - raf_workplan::test_crate::complete::a_test_renamed_while_the_tests_ran_refuses
    - raf_workplan::test_crate::complete::an_error_after_the_rewrite_does_not_leave_a_removed_record_beside_stale_blocks
    - raf_workplan::test_crate::complete::writes::a_rewrite_that_fails_leaves_the_record_and_the_blocks_as_they_were
    - raf_workplan::test_crate::complete::prose_that_links_to_the_record_refuses_because_the_link_is_broken_once_it_is_removed
    - raf_workplan::test_crate::complete::a_broken_link_in_a_capability_the_change_does_not_list_also_blocks_completion
    - raf_workplan::test_crate::complete::an_outcome_the_profile_refuses_makes_the_record_not_load_and_is_an_error
    - raf_workplan::test_crate::complete::an_agent_assessment_satisfies_condition_six_and_a_rejected_one_by_either_never_does
    - raf_workplan::test_crate::bound::a_passing_test_passes_on_one_listed_entry_and_one_passed
    - raf_workplan::test_crate::bound::a_failing_test_fails_the_binding
    - raf_workplan::test_crate::bound::an_ignored_test_is_listed_and_refused
    - raf_workplan::test_crate::bound::a_test_behind_a_disabled_feature_lists_nothing_and_is_refused
    - raf_workplan::test_crate::bound::the_same_name_in_two_targets_runs_only_in_the_pinned_one
    - raf_workplan::test_crate::bound::an_integration_test_with_a_hyphen_runs_under_its_cargo_name
    - raf_workplan::test_crate::bound::a_function_that_is_not_a_test_is_refused_before_any_process
    - raf_workplan::test_crate::bound::hostile_bindings_are_unresolved_and_never_reach_cargo
    - raf_workplan::exec::system::tests::cargo::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran
    - raf_workplan::test_crate::bound::documentation_tests_follow_the_rules_in_edition_2021
    - raf_workplan::test_crate::bound::documentation_tests_follow_the_rules_in_edition_2024
    - raf_workplan::test_crate::bound::an_item_path_that_begins_with_a_hyphen_never_reaches_cargo
    - raf_workplan::test_crate::bound::an_example_that_exits_zero_passes_and_one_that_does_not_fails
    - raf_workplan::test_crate::bound::a_hyphenated_example_runs_under_its_cargo_name
    - raf_workplan::test_crate::bound::an_example_that_reads_standard_input_sees_it_closed
    - raf_workplan::exec::system::tests::cargo::an_example_that_never_ends_is_killed_and_reported_with_the_time_it_ran
    - raf_workplan::test_crate::exec::the_patterns_are_the_ones_the_documents_state
    - raf_workplan::test_crate::exec::every_pattern_refuses_hostile_values
    - raf_workplan::test_crate::exec::every_invocation_is_the_row_of_its_table
    - raf_workplan::test_crate::exec::the_limits_are_the_documented_bounds
    - raf_workplan::test_crate::exec::stubs::every_row_reaches_a_process_as_exactly_its_argv
    - raf_workplan::test_crate::exec::stubs::a_stub_named_by_cargo_runs_for_cargo_and_a_stub_on_the_path_when_it_is_unset
    - raf_workplan::test_crate::exec::stubs::standard_input_is_closed
    - raf_workplan::exec::system::tests::a_process_that_outlives_its_limit_is_killed
    - raf_workplan::exec::system::tests::a_group_that_already_exited_is_done
    - raf_workplan::exec::system::tests::a_refused_signal_is_done_only_when_the_leader_has_exited
    - raf_workplan::exec::system::tests::any_other_failure_is_reported_even_after_the_leader_exited
    - raf_workplan::exec::system::tests::a_group_whose_leader_exited_before_the_kill_is_killed_without_error
    - raf_workplan::exec::system::tests::terminating_a_leader_that_exited_before_the_kill_succeeds
    - raf_workplan::exec::system::tests::a_signal_to_the_leader_ends_it_only_when_delivered_or_it_is_gone
    - raf_workplan::exec::system::tests::a_leader_that_exited_has_ended_and_is_collected
    - raf_workplan::exec::system::tests::a_leader_still_running_is_killed_and_collected_before_it_counts_as_ended
    - raf_workplan::exec::system::tests::the_verdict_follows_its_precedence
    - raf_workplan::exec::system::tests::a_zero_limit_is_passed_at_once_unless_the_run_is_done
    - raf_workplan::exec::system::tests::the_pauses_double_up_to_the_ceiling_and_stay_there
    - raf_workplan::exec::system::tests::the_capture_stops_one_byte_past_its_bound_and_flags_the_overflow
    - raf_workplan::exec::system::tests::output_at_the_bound_passes_and_one_byte_over_is_refused_on_each_stream
    - raf_workplan::test_crate::verify::one_passing_and_one_failing_test_print_one_line_each_and_the_run_fails
    - raf_workplan::test_crate::verify::no_slug_runs_every_capability_and_a_slug_only_the_ones_the_change_lists
    - raf_workplan::test_crate::verify::examples_and_doc_tests_are_run_and_judged_by_their_proof
    - raf_workplan::test_crate::verify::zero_tests_and_ignored_tests_fail_the_run
    - raf_workplan::test_crate::verify::selected_cargo_output_is_trusted_but_an_empty_list_is_refused
    - raf_workplan::bound::tests::result_counts_preserve_unsigned_spelling_and_refuse_host_overflow
    - raf_workplan::test_crate::sync::write_publishes_the_computed_snapshot_without_comparing_later_authored_edits
    - raf_workplan::test_crate::verify::hostile_bindings_are_unresolved_results_that_never_reach_cargo
    - raf_workplan::test_crate::verify::verify_writes_nothing
    - raf_workplan::test_crate::verify::verify_asks_git_nothing_and_does_not_go_through_the_version_gate
    - cargo_workplan::test_crate::complete::a_completed_change_prints_what_was_removed_and_rewritten_and_leaves_a_clean_check
    - cargo_workplan::test_crate::complete::a_failing_example_stands_under_conditions_one_and_five
    - cargo_workplan::test_crate::complete::a_failing_bound_test_is_condition_five_and_the_record_stays
    - cargo_workplan::test_crate::verify::one_passing_and_one_failing_test_print_one_line_each_and_exit_one
    - raf_workplan::test_crate::bound::a_process_that_cannot_start_is_an_error_and_not_a_verdict
    - raf_workplan::test_crate::bound::a_test_of_a_binary_runs_under_bin
    - raf_workplan::test_crate::bound::targets_cargo_cannot_select_a_test_in_are_refused_before_any_process
    - raf_workplan::test_crate::complete::a_prefixed_slug_names_the_same_change
    - raf_workplan::test_crate::complete::a_record_removed_while_the_tests_ran_is_an_error_and_nothing_is_written
    - raf_workplan::test_crate::complete::a_run_that_cannot_start_is_an_error_and_not_a_refusal
    - raf_workplan::test_crate::complete::complete_accepts_the_change_prefix_and_an_unknown_slug_or_a_record_that_did_not_load_is_an_error
    - raf_workplan::test_crate::complete::conditions_and_failures_have_stable_names_and_the_headline_is_the_first_line
    - raf_workplan::test_crate::complete::writes::a_removal_that_fails_reports_the_files_already_rewritten_and_removes_nothing_else
    - raf_workplan::test_crate::exec::a_directory_that_does_not_exist_is_a_spawn_error
    - raf_workplan::test_crate::exec::a_generic_list_holds_exactly_the_characters_of_the_pattern
    - raf_workplan::test_crate::exec::a_name_missing_from_the_search_path_is_a_spawn_error
    - raf_workplan::test_crate::exec::a_program_that_does_not_exist_is_a_spawn_error
    - raf_workplan::test_crate::exec::a_ref_cannot_be_read_as_an_option_inside_its_argument
    - raf_workplan::test_crate::exec::a_refusal_names_the_pattern_and_shows_the_value_safely
    - raf_workplan::test_crate::exec::a_runner_reference_is_a_runner
    - raf_workplan::test_crate::exec::a_test_filter_always_follows_a_separator_and_never_leads
    - raf_workplan::test_crate::exec::a_tree_path_refuses_every_hostile_value_that_is_not_printable_path_text
    - raf_workplan::test_crate::exec::each_newtype_is_built_only_by_its_pattern
    - raf_workplan::test_crate::exec::every_pattern_accepts_what_it_states
    - raf_workplan::test_crate::exec::every_pattern_refuses_what_it_does_not_state
    - raf_workplan::exec::system::tests::a_descendant_that_holds_the_pipes_after_the_exit_is_not_waited_for_past_the_limit
    - raf_workplan::test_crate::exec::stubs::a_failing_exit_code_and_a_signal_are_outputs_and_not_errors
    - raf_workplan::test_crate::exec::stubs::a_file_that_cannot_be_executed_is_a_spawn_error
    - raf_workplan::exec::system::tests::a_process_inside_its_limit_passes
    - raf_workplan::exec::system::tests::a_process_that_never_stops_writing_is_killed_at_the_bound
    - raf_workplan::exec::system::tests::both_streams_are_drained_at_once
    - raf_workplan::test_crate::exec::stubs::nothing_else_is_added_to_the_environment
    - raf_workplan::test_crate::exec::stubs::output_is_returned_byte_for_byte
    - raf_workplan::test_crate::exec::stubs::the_c_locale_is_set_when_the_output_is_parsed_and_left_alone_otherwise
    - raf_workplan::test_crate::exec::stubs::the_default_bound_is_sixteen_mebibytes_and_a_larger_one_is_a_parameter
    - raf_workplan::exec::system::tests::the_group_is_killed_when_no_kill_program_is_on_the_path
    - raf_workplan::test_crate::exec::stubs::the_process_runs_in_the_working_directory
    - raf_workplan::exec::system::tests::what_the_process_started_does_not_outlive_the_kill
    - raf_workplan::test_crate::exec::the_fake_answers_a_scripted_invocation_and_records_it
    - raf_workplan::test_crate::exec::the_fake_applies_the_output_bound_as_the_real_runner_does
    - raf_workplan::test_crate::exec::the_fake_applies_the_time_limit_as_the_real_runner_does
    - raf_workplan::test_crate::exec::the_fake_keeps_the_program_apart_from_the_arguments
    - raf_workplan::test_crate::exec::the_fake_refuses_an_invocation_it_has_no_answer_for
    - raf_workplan::test_crate::exec::the_fake_repeats_an_answer_and_a_later_one_replaces_it
    - raf_workplan::test_crate::exec::the_fake_returns_a_failing_exit_and_a_scripted_error
    - raf_workplan::test_crate::exec::the_programs_come_from_cargo_and_the_path_and_can_be_replaced
    - raf_workplan::test_crate::exec::the_real_git_starts
    - raf_workplan::test_crate::scenarios::each_fault_of_the_account_has_a_document_that_breaks_only_it
    - raf_workplan::test_crate::verify::a_capability_that_did_not_load_is_an_error_and_not_a_pass
    - raf_workplan::test_crate::verify::a_process_that_cannot_start_is_an_error_and_not_a_failing_binding
    - raf_workplan::test_crate::verify::a_run_with_nothing_bound_passes_and_prints_nothing
    - raf_workplan::test_crate::verify::an_example_that_cannot_be_started_is_an_error_and_not_a_line
    - raf_workplan::test_crate::verify::an_example_that_exits_with_a_failure_or_a_doc_test_that_fails_is_a_line
    - raf_workplan::test_crate::verify::an_unknown_slug_and_a_record_that_did_not_load_are_different_errors
    - raf_workplan::test_crate::verify::only_the_examples_of_the_capabilities_the_change_lists_are_run
    - raf_workplan::test_crate::verify::the_lines_of_a_report_are_its_text_and_each_verdict_has_a_name_and_a_reason
    - raf_workplan::bound::tests::a_benchmark_is_not_a_test_entry
    - raf_workplan::bound::tests::a_benchmark_of_the_name_is_not_a_listed_test
    - raf_workplan::bound::tests::a_binary_test_is_pinned_with_bin
    - raf_workplan::bound::tests::a_cfg_test_module_of_the_library_runs_as_lib_with_the_name_in_the_crate
    - raf_workplan::bound::tests::a_code_binding_and_an_item_that_is_no_example_are_refused
    - raf_workplan::bound::tests::a_compile_fail_doc_test_is_refused_with_a_request_for_a_runnable_one
    - raf_workplan::bound::tests::a_doc_item_path_is_refused_unless_it_matches_its_pattern
    - raf_workplan::bound::tests::a_doc_list_entry_has_the_form_of_the_pattern
    - raf_workplan::bound::tests::a_doc_list_entry_with_the_platform_separator_is_read_and_keeps_its_printed_name
    - raf_workplan::bound::tests::a_doc_status_is_never_taken_from_another_name_or_a_loose_line
    - raf_workplan::bound::tests::a_doc_status_is_read_from_the_whole_line
    - raf_workplan::bound::tests::a_doc_test_binding_lists_the_package_and_runs_the_bare_item_path
    - raf_workplan::bound::tests::a_doc_test_passes_only_on_its_own_ok_line
    - raf_workplan::bound::tests::a_failing_selected_doc_test_fails_the_binding_by_name
    - raf_workplan::bound::tests::a_failing_sibling_the_filter_also_matches_fails_the_binding
    - raf_workplan::bound::tests::a_failing_test_fails_the_binding
    - raf_workplan::bound::tests::a_function_that_is_not_a_test_is_refused_by_name
    - raf_workplan::bound::tests::a_hostile_example_name_and_package_never_reach_the_runner
    - raf_workplan::bound::tests::a_hostile_name_in_the_manifest_never_reaches_the_runner
    - raf_workplan::bound::tests::a_killed_test_is_a_failed_binding_that_reports_the_time
    - raf_workplan::bound::tests::a_list_pass_that_does_not_build_fails_the_binding
    - raf_workplan::bound::tests::a_list_with_no_entry_has_none
    - raf_workplan::bound::tests::a_list_with_one_entry_has_one_entry
    - raf_workplan::bound::tests::a_list_with_two_entries_has_two
    - raf_workplan::bound::tests::a_loose_ok_line_is_not_proof
    - raf_workplan::bound::tests::a_nested_test_of_an_integration_test_keeps_the_path_inside_the_crate
    - raf_workplan::bound::tests::a_no_run_doc_test_is_refused_with_a_request_for_a_runnable_one
    - raf_workplan::bound::tests::a_passing_result_is_read
    - raf_workplan::bound::tests::a_result_line_without_its_counts_is_an_error
    - raf_workplan::bound::tests::a_run_that_cannot_start_is_an_error_and_not_a_verdict
    - raf_workplan::bound::tests::a_run_that_passes_one_test_with_more_passed_is_refused
    - raf_workplan::bound::tests::a_selected_entry_with_no_ok_line_is_refused
    - raf_workplan::bound::tests::a_status_other_than_zero_is_never_a_pass_whatever_the_counts_say
    - raf_workplan::bound::tests::a_target_without_the_harness_or_with_required_features_is_refused
    - raf_workplan::bound::tests::a_test_in_a_benchmark_target_is_refused
    - raf_workplan::bound::tests::a_test_of_a_library_module_is_named_by_its_path_in_the_crate
    - raf_workplan::bound::tests::a_test_of_a_target_with_required_features_is_refused
    - raf_workplan::bound::tests::a_test_of_a_target_without_the_harness_is_refused
    - raf_workplan::bound::tests::a_test_with_clean_facts_is_planned_and_its_neighbours_are_refused
    - raf_workplan::bound::tests::an_example_killed_at_its_limit_fails_with_the_elapsed_time
    - raf_workplan::bound::tests::an_example_runs_under_its_cargo_name_and_the_exit_status_decides
    - raf_workplan::bound::tests::an_example_with_clean_facts_is_planned_and_hostile_ones_are_refused
    - raf_workplan::bound::tests::an_ignored_selected_doc_test_is_refused
    - raf_workplan::bound::tests::an_ignored_test_is_refused_though_the_list_shows_it
    - raf_workplan::bound::tests::an_impl_item_is_bound_under_its_self_type_as_rustdoc_writes_it
    - raf_workplan::bound::tests::an_integration_test_with_a_hyphen_is_pinned_under_its_cargo_name
    - raf_workplan::bound::tests::an_older_result_line_without_the_time_is_read
    - raf_workplan::bound::tests::another_entrys_ok_line_does_not_stand_for_the_selected_one
    - raf_workplan::bound::tests::every_pass_carries_the_limits_of_its_kind
    - raf_workplan::bound::tests::every_selected_entry_must_be_ok
    - raf_workplan::bound::tests::evidence_reads_as_a_verdict_line_and_the_processes_that_ran
    - raf_workplan::bound::tests::hostile_bindings_never_reach_the_runner
    - raf_workplan::bound::tests::no_result_line_and_two_result_lines_are_unresolved
    - raf_workplan::bound::tests::no_result_line_is_no_result
    - raf_workplan::bound::tests::no_selected_entry_is_a_refusal_and_runs_nothing
    - raf_workplan::bound::tests::noise_around_the_one_entry_does_not_matter
    - raf_workplan::bound::tests::noise_lines_are_not_entries
    - raf_workplan::bound::tests::one_entry_under_another_name_is_unresolved
    - raf_workplan::bound::tests::the_doc_list_gives_names_and_item_parts_across_blocks
    - raf_workplan::bound::tests::the_pinned_target_is_the_library_a_binary_or_an_integration_test
    - raf_workplan::bound::tests::the_root_of_the_crate_has_no_item_part
    - raf_workplan::bound::tests::too_much_output_is_a_failed_binding
    - raf_workplan::bound::tests::two_listed_entries_are_unresolved
    - raf_workplan::bound::tests::two_result_lines_are_two_results
    - raf_workplan::bound::tests::windows_line_endings_are_read
    - raf_workplan::bound::tests::zero_listed_entries_are_unresolved_and_nothing_is_run
    - raf_workplan::bound::tests::zero_passed_failed_and_ignored_results_are_read
    - raf_workplan::bound::tests::zero_passed_is_unresolved_even_with_exit_status_zero
    - raf_workplan::exec::tests::a_literal_is_the_only_free_text_and_it_is_static
    - raf_workplan::exec::tests::a_rejected_value_never_becomes_an_argument
    - raf_workplan::exec::tests::every_variant_is_walked
    - raf_workplan::exec::tests::no_shell::every_argument_is_one_element_and_no_shell_reads_it
    - raf_workplan::lifecycle::completion::tests::a_completion_prints_the_removal_and_each_rewritten_file
    - raf_workplan::lifecycle::completion::tests::a_section_without_an_accepted_assessment_counts_under_condition_six_and_prints_once
    - raf_workplan::lifecycle::completion::tests::a_stale_block_prints_a_line_unless_an_obligation_already_does
    - raf_workplan::lifecycle::completion::tests::conditions_are_numbered_as_the_contract_numbers_them
    - raf_workplan::lifecycle::completion::tests::the_counts_of_a_refusal_are_singular_or_plural_and_in_the_order_of_the_conditions
    - raf_workplan::staged::tests::a_replaced_file_reads_as_its_new_text_within_the_bound
    - raf_workplan::staged::tests::every_other_path_is_answered_as_the_inner_tree_does
    - raf_workplan::staged::tests::the_hidden_file_is_not_read_listed_or_found
    - raf_workplan::test_crate::complete::completion_after_record_removal_enforces_the_complete_policy
    - raf_workplan::test_crate::complete::the_check_once_the_record_is_removed_applies_the_debt_record_so_a_held_finding_does_not_refuse
    - raf_workplan::test_crate::complete::the_check_once_the_record_is_removed_refuses_a_debt_row_that_is_resolved
  examples:
    - raf_workplan::bound
    - raf_workplan::exec
depends: [cap:workplan-reconcile, cap:workplan-generated-docs]
---

# Completion gate

## Purpose

A change is complete when the current documentation says what the system now does,
and not when a status flag is set. `change complete` is the gate that checks it. It
makes documentation a condition of completing the work, in place of a final
checkbox that says "update docs".

## Contract

### Supplied process runner
<!-- raf:tested-by raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal,raf_workplan::test_crate::complete::a_run_that_cannot_start_is_an_error_and_not_a_refusal,raf_workplan::test_crate::gate::the_programs_of_a_gate_are_started_in_its_directory,raf_workplan::test_crate::gate::the_tracked_paths_a_caller_gives_replace_the_listing_and_start_no_process -->

The lifecycle's `open_with` sends every process through its supplied runner.
This includes the initial tracked-path queries and their repetition when
completion refreshes the repository after bound runs. Gate-configured programs
do not replace the supplied runner, whose working directory must match the gate.
The opening still asks for the Git version and prefix first. Explicit tracked
paths bypass the tracked-path queries. Without them, the existing Git query
contract determines the tracked set or recognized non-repository result.

A missing or failing supplied answer propagates as an error. There is no retry
through a system runner. A tracked-path failure during opening or refresh writes
no document, removes no record and publishes no generated block. Ordinary
Gate's ordinary `open` retains its configured system programs and runs them in the gate's
directory.

### Completion conditions
<!-- raf:tested-by raf_workplan::test_crate::complete::each_kind_of_obligation_makes_complete_refuse_and_leaves_the_tree_as_it_was_and_its_repair_completes,raf_workplan::test_crate::complete::an_item_without_outcome_and_a_move_to_nowhere_or_to_itself_refuse_condition_two,raf_workplan::test_crate::complete::a_proposed_block_of_the_change_refuses_condition_three_in_a_loaded_and_in_an_unloaded_document,raf_workplan::test_crate::complete::a_block_that_is_stale_or_missing_anywhere_refuses_condition_four,raf_workplan::test_crate::complete::a_failing_bound_test_refuses_condition_five_alone,raf_workplan::test_crate::complete::an_agent_assessment_satisfies_condition_six_and_a_rejected_one_by_either_never_does -->

`change complete <slug>` succeeds only when all six hold:

1. The obligation list from [reconcile](./workplan-reconcile.md) is empty, with
   zero obligations.
2. Every item in the record has an `outcome` of `implemented`, `dropped` or
   `moved:change:<slug>`, and a `moved` outcome names another change that exists.
3. No PROPOSED block for the change remains in any capability document.
4. `docs sync` would rewrite nothing, so every generated block is current.
5. The applicable bound tests and examples pass: `cargo test` filtered to the tests in
   `bindings.tests` of each listed capability, and the targets and documentation
   tests in `bindings.examples` built and run, through the invocations below.
   A compiler-proved platform exclusion is reported separately as not applicable.
6. Each authored section in `affects` has a current assessment with the verdict
   `accepted`, as the [prose assessment](./workplan-prose-assessment.md) records it:
   the record `work/assessments/<capability slug>-<section>.md` exists, every input it
   keeps (the section, the bound symbols, the evidence its claims name) has the digest
   it keeps, and the section has no input the record lacks.

A failing bound example also stands as an `example-failing` obligation, so conditions
1 and 5 both report it. Condition 5 is the execution and condition 1 the record of it;
the refusal lists the fault under each. An `outcome` that is not one of the three
values is refused by the profile, so the record does not load and `complete` is an
error (exit 2), not a refusal of condition 2. Condition 3 counts the blocks of every
capability document, including one that failed to load.

A section with no assessment, one whose assessment has an input that moved, and one
whose current assessment is `rejected` each refuse condition 6. An assessment by an
agent satisfies the condition as one by a human does, and a `rejected` verdict never
does. A section with nothing outside its generated blocks and comments cannot occur:
the profile refuses such a section (`okf-section-empty`), so its document does not
load and `complete` is an error before any condition is judged, and every section of
a capability that loads is authored. A section without a current accepted
assessment is one or more `assessment-missing`, `assessment-stale` or
`assessment-rejected` obligations, one `assessment-stale` for each input that moved,
so conditions 1 and 6 both report it. The refusal prints each of those lines once,
under its obligation, and its headline counts the section once, as
`sections without a current accepted assessment`, however many obligations it has:
one section with three moved inputs is `3 obligations, 1 section without a current
accepted assessment`. `complete` reads the assessment records and never writes one:
they stay as they are when the change record is removed.

### Static and execution order
<!-- raf:tested-by raf_workplan::test_crate::complete::a_static_failure_spends_no_process_on_cargo_and_a_clean_change_runs_cargo_after_the_static_checks,raf_workplan::test_crate::complete::a_document_edited_while_the_tests_ran_refuses_and_nothing_is_written,raf_workplan::test_crate::complete::a_binding_added_while_the_tests_ran_is_a_change_even_when_every_condition_still_holds,raf_workplan::test_crate::complete::a_none_change_reads_each_file_of_the_bound_package_once_at_the_merge_base_and_otherwise_none -->

The static conditions are judged first, every one of them and with no
early exit: conditions 1 to 4 (the obligation list without `example-failing`, which
needs a run), condition 6 (the currency of each assessment, from digests of the
working tree) and the check of the repository once the record is removed, described
below. Only when none fails are the bound tests and examples of condition 5 run, so a
static failure, a section without a current accepted assessment among them, starts no
`cargo` process. The repository is then opened again and the static conditions are
judged a second time, immediately before the writes, because the documents and the
source can change while `cargo` runs; a failure, or a different set of bindings to
run, refuses. The merge base is asked for again, and the files of a commit do not
change, so the source index that the complete policy loads at the merge base is loaded once
for both judgements while the merge base and the packages it is read from stay the
same.

### Repository after removal
<!-- raf:tested-by raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal,raf_workplan::test_crate::complete::a_capability_left_implementing_with_no_other_record_refuses_and_says_what_to_change,raf_workplan::test_crate::complete::a_broken_link_in_a_capability_the_change_does_not_list_also_blocks_completion,raf_workplan::test_crate::complete::a_linked_work_directory_is_refused_before_anything_runs_and_a_real_one_completes,raf_workplan::test_crate::complete::a_record_linked_while_the_tests_ran_is_refused_at_the_second_judgement,raf_workplan::test_crate::complete::an_error_after_the_rewrite_does_not_leave_a_removed_record_beside_stale_blocks -->

The state rules of the
[document model](./workplan-document-model.md) are judged on the repository as it is
once the record is removed and the generated blocks that the removal stales are
rewritten. `check` of that repository must report nothing, and every finding there
refuses completion, including one in a capability the change does not list, such as a
broken link or a link in prose to the record itself. The author, never `complete`,
sets `state: delivered` before running it: in that repository a listed capability that
is not `delivered` and that no other open record lists is a `no-open-record` finding.
A `delivered` capability beside its still-open record is exactly what `complete`
resolves, so the repository before the removal is judged by the six conditions only.

When any condition fails, the command prints the failures, changes nothing and exits
with 1. When all hold, it rewrites the generated blocks that removing the record
stales (the `Changed by` rows of the relationship tables), with the staged write of
[the generated documents](./workplan-generated-docs.md), and then, as the very last
step, deletes `work/changes/<slug>.md` and exits with 0. Those are its only writes. If
the rewrite fails the record stays, and if the deletion fails after the rewrite the
error names the files already rewritten, so a deleted record is never found beside
stale blocks. Git holds the record's history.

The record is deleted only through real directories. With each judgement of the
static conditions, the first before any `cargo` process and the second immediately
before the writes, `complete` judges the way to `work/changes/<slug>.md`: `work` and
`work/changes` must each be a directory and not a symbolic link, and the record a
regular file there. Otherwise it is an error (exit 2) that names the refused path
and the judgement that refused it, and nothing is written; refused at the first
judgement, nothing has run either, and refused at the second, the error says that
the bound tests and examples ran. A linked `work/changes` is never read as the
directory of change records, so its change is not found.

### Obligations by declared effect
<!-- raf:tested-by raf_workplan::test_crate::complete::contract_effect_none_leaves_only_contract_untouched_and_preserves_other_refusals,raf_workplan::test_crate::complete::an_untouched_usage_or_examples_of_a_none_change_refuses,raf_workplan::test_crate::complete::an_agent_assessment_satisfies_condition_six_and_a_rejected_one_by_either_never_does -->

All six conditions apply to every change. The effect only
sets which obligations [reconcile](./workplan-reconcile.md) raises. A change with
`contract_effect: changes` can owe an untouched-section obligation for each section in
`affects`. A change with `contract_effect: none` records that the documented contract
is unaffected, so reconcile raises no `section-untouched` obligation for the Contract
section alone. Affected Usage and Examples remain owed. Every other obligation kind applies, and the
obligation list must still be empty. A small internal change does not need a new
tutorial. The effect does not change condition 6 either: under `contract_effect: none`
each authored section in `affects` still needs a current accepted assessment, since
the people the change reaches read it whether or not its contract moved.

### Sole policy at both judgements
<!-- raf:tested-by raf_workplan::test_crate::complete::completion_after_record_removal_enforces_the_complete_policy,raf_workplan::test_crate::complete::a_document_edited_while_the_tests_ran_refuses_and_nothing_is_written -->

The check after record removal judges the complete
[workplan policy](./workplan-strict.md), at both the initial judgement and the
second judgement immediately before writes. A policy finding refuses completion
without writing. There is no caller-selected policy floor.

### Public token and outcome obligations
<!-- raf:tested-by raf_workplan::test_crate::complete::a_none_change_that_edits_a_bound_signature_refuses,raf_workplan::test_crate::complete::an_edit_of_a_bound_body_alone_refuses,raf_workplan::test_crate::complete::an_edit_of_a_bound_doc_comment_refuses,raf_workplan::test_crate::complete::an_item_dropped_without_a_reason_refuses_condition_two,raf_workplan::test_crate::complete::an_untouched_usage_or_examples_of_a_none_change_refuses,raf_workplan::test_crate::complete::the_check_once_the_record_is_removed_applies_the_debt_record_so_a_held_finding_does_not_refuse,raf_workplan::test_crate::complete::the_check_once_the_record_is_removed_refuses_a_debt_row_that_is_resolved -->

The six conditions retain their order. Condition 5
runs every bound test and example, and condition 1 checks supported-behavior
wording and token-level contract effects. Condition 1
refuses on `contract-effect-understated`, which reconcile raises for a
`contract_effect: none` change that alters any token of a bound public item, its doc
comments included, or whose working tree has lost one or taken it off the public
surface, and condition 2 refuses on
`outcome-without-reason`, a `dropped` or `moved` item with no `reason`, as it refuses
on `item-without-outcome`; the headline counts such an item as an item without a
reason (`refused: 1 obligation, 1 item without a reason`). Under the complete policy
`contract_effect: none` leaves out `section-untouched` for the contract only, so
condition 1 also refuses on an untouched Usage or Examples section in `affects`. A
refusal under the complete policy writes nothing, as every refusal does.

The check of the repository once the record is removed applies the strict debt
record as `check` does, so a finding the record holds does not refuse completion
and a resolved row does.

### Supported delivery state
<!-- raf:tested-by raf_workplan::test_crate::complete::a_capability_left_implementing_with_no_other_record_refuses_and_says_what_to_change,raf_workplan::test_crate::complete::a_delivered_capability_with_a_binding_that_does_not_resolve_refuses,raf_workplan::test_crate::complete::empty_bindings_satisfy_condition_five_only_while_the_capability_is_designed_or_considered -->

Promotion to a stronger state requires reconciliation, not just a status
change. The gate refuses a listed capability whose `state` the content does not
support: a `delivered` capability needs resolved text, no PROPOSED block, and
bindings that resolve; it binds at least one code path and one test. A capability
in `implementing` may have missing bindings, and `status`, the catalogue and the
reconcile obligations report them as gaps.

### Production clock admission
<!-- raf:tested-by rust_agentic_foundations::test_crate::repo_gates::time_in_crates_tracked_files_name_no_clock,rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_a_planted_clock_reading_with_file_and_line,raf_workplan::exec::system::tests::a_process_that_outlives_its_limit_is_killed,raf_workplan::exec::system::tests::the_verdict_follows_its_precedence -->

RAF's repository clock gate requires WallClock to store the standard Instant,
start it through Instant::now and return elapsed directly from that stored
value. SystemRunner::run must select that clock through the injected runner
path. The native source index identifies the actual Clock trait and its sole
production implementation; unrelated traits with the same name do not count.
Only source reached entirely through test scopes is exempt, including inherited
external modules. A condition that can hold without test remains production.

Clock stays open for the host adapter boundary. The gate admits a closed set of
source forms, refusing unsupported clock aliases, block-local implementations
and production item generation rather than guessing their meaning. The existing
scripted clocks prove limit decisions and bounded work without measuring host
time. These checks do not calibrate the host clock or strengthen process
containment or supervision after the runner exits.

### Validated execution arguments
<!-- raf:tested-by raf_workplan::test_crate::exec::every_invocation_is_the_row_of_its_table,raf_workplan::test_crate::exec::every_pattern_accepts_what_it_states,raf_workplan::test_crate::exec::every_pattern_refuses_what_it_does_not_state,raf_workplan::test_crate::exec::every_pattern_refuses_hostile_values,raf_workplan::test_crate::exec::a_test_filter_always_follows_a_separator_and_never_leads,raf_workplan::test_crate::exec::stubs::every_row_reaches_a_process_as_exactly_its_argv -->

Bound tests and examples run only through the argv forms below. Each
process is spawned directly with no shell and with its arguments as an argv vector.
The program is the one named by `$CARGO` when that is set, and `cargo` from `PATH`
otherwise. Every call carries `--frozen`, so completion never reaches the network or
rewrites `Cargo.lock`. The placeholders are filled only from the source index, never
from raw document text:

| Binding | Argv |
| --- | --- |
| Test, list pass | `cargo test --frozen --package=<pkg> <target> -- --list --exact -- <name>` |
| Test, run pass | `cargo test --frozen --package=<pkg> <target> -- --exact -- <name>` |
| Doc test, list pass | `cargo test --frozen --package=<pkg> --doc -- --list` |
| Doc test, run pass | `cargo test --frozen --package=<pkg> --doc -- <item-path>` |
| Example target | `cargo run --frozen --package=<pkg> --example=<example>` |

`<target>` is the resolved target of the test, written `--lib`, `--bin=<bin>` or
`--test=<test>`, so that the same name in another target is never run. `<name>` is
the resolved item's path within its crate. The `--` before `<name>` is a backstop:
libtest reads a value after it as a filter even if validation regressed.

Four patterns validate the values the gate handles. A placeholder value that fails
its pattern is reported as an unresolved binding and is not passed to `cargo`, and a
line of the list output whose file part fails its pattern is not read as an entry:

- `<name>`: `[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*`;
- `<pkg>`, `<bin>`, `<test>` and `<example>`: `[A-Za-z0-9_][A-Za-z0-9_-]*`, which
  allows the hyphens that target names carry and never begins with `-`;
- `<item-path>`: the `<name>` pattern, widened so that an impl item written by rustdoc as `Wrapper<T>::get` can be bound. Each `::`-separated segment is `[A-Za-z_][A-Za-z0-9_]*` followed by an optional generic argument list matching `<[A-Za-z0-9_,':&]*>`, with no whitespace, and the filter never begins with `-`;
- the names in the list output, which the gate parses and never places on a command line, have the form `<relative-source-path> - <item-path> (line <n>)`, where `<relative-source-path>` matches `[A-Za-z0-9_][A-Za-z0-9_./-]*` and has no empty, `.` or `..` segment. rustdoc prints the file part with the platform's separator, so each `\` in it is read as `/` before the pattern is applied.

Two further validators back the `git` commands of [reconcile](./workplan-reconcile.md):
a full reference name, `refs/` followed by `/`-separated segments of letters, digits
and `. _ - = + @ , #`, none empty, none beginning with `.` or ending with `.` or
`.lock`, with no `..` and at most 1024 bytes; and a commit id, 40 or 64 lowercase
hexadecimal digits. The path after `<merge-base>:` matches the relative-source-path
pattern above.

### Positive execution proof
<!-- raf:tested-by raf_workplan::bound::tests::zero_listed_entries_are_unresolved_and_nothing_is_run,raf_workplan::bound::tests::two_listed_entries_are_unresolved,raf_workplan::bound::tests::a_doc_test_passes_only_on_its_own_ok_line,raf_workplan::bound::tests::every_selected_entry_must_be_ok,raf_workplan::bound::tests::a_no_run_doc_test_is_refused_with_a_request_for_a_runnable_one,raf_workplan::bound::tests::a_compile_fail_doc_test_is_refused_with_a_request_for_a_runnable_one,raf_workplan::test_crate::complete::a_failing_doc_test_a_test_that_runs_nothing_and_an_ignored_test_refuse_beside_passing_neighbours -->

Exit status 0 from `cargo test` is not proof, because
libtest reports success when a filter matches nothing. A test binding counts as passed
only when the list pass shows exactly one `<name>: test` entry (an `#[ignore]`d test
is listed as a test, so the run pass must also report it ran) and the run pass reports
`1 passed; 0 failed; 0 ignored` for that target. Zero entries, more than one entry, or
`0 passed` is an unresolved-binding failure and refuses completion, so a resolved item
that is not a `#[test]`, a test behind a disabled `cfg(feature)`, and a name that
differs from the libtest name all refuse. A doc-test binding is run by listing the doc
tests of the package, selecting the entries whose item part (the text between ` - `
and ` (line `) equals the bound item path within the crate, and then running the doc
tests with the bare item path as a whitespace-free substring filter, without
`--exact`. rustdoc names a method by the module of its impl block, so the item path of
a method whose impl block sits in another file than its type is written under the
module of that file. rustdoc splits every test argument on whitespace, so a full name such as
`src/lib.rs - foo (line 4)` can never act as one exact filter. The run output is
parsed for `test <name> ... ok` lines, and every entry selected by the list pass must
appear as `... ok`, matched strictly as the line `test <name> ... ok` with nothing
between the name and `...`. A `no_run` doc test, reported as `<name> - compile ...
ok`, was built and not run, and a `compile_fail` doc test is reported as `<name> -
compile fail ... ok`. Neither satisfies built and run, so a binding that selects one
is refused with a message that tells the author to bind a runnable doc test. No
selected entry is a refusal. A selected entry that fails, is ignored or does not
appear is a refusal. Any other doc test that the substring also matches (for example
`foo_more` for the item `foo`) is ignored by the gate and neither required to pass nor
counted, but its failure still fails the `cargo test` exit status, so the binding is
refused.

### Executable examples
<!-- raf:tested-by raf_workplan::test_crate::bound::an_example_that_exits_zero_passes_and_one_that_does_not_fails,raf_workplan::test_crate::bound::an_example_that_reads_standard_input_sees_it_closed,raf_workplan::bound::tests::every_pass_carries_the_limits_of_its_kind,raf_workplan::test_crate::complete::a_failing_bound_example_stands_under_conditions_one_and_five -->

`cargo run` for an example target runs with its standard input closed
(null) and a timeout of 300 seconds, which includes the build that `cargo run`
performs first, after which the process is killed and the binding counts as failing.
The example's own exit status decides pass or fail. The run executes the workspace's
cargo configuration as `cargo test` would, so the workspace is trusted build input
(see [the security policy](../../SECURITY.md)).

### Execution limits and process failures
<!-- raf:tested-by raf_workplan::bound::tests::every_pass_carries_the_limits_of_its_kind,raf_workplan::bound::tests::a_run_that_cannot_start_is_an_error_and_not_a_verdict,raf_workplan::bound::tests::too_much_output_is_a_failed_binding,raf_workplan::test_crate::exec::the_limits_are_the_documented_bounds,raf_workplan::exec::system::tests::output_at_the_bound_passes_and_one_byte_over_is_refused_on_each_stream -->

Each list pass and run pass of `cargo test` may compile first and has a
timeout of 1800 seconds. The standard output and the standard error of every process
of a bound run are each read up to 16 MiB. A process that runs past its timeout or
writes more than the bound is killed, its process group with it on Unix, and the
binding counts as failing with the time it ran; a process that cannot be started is
an error and not a verdict.

An unsuccessful bound process reports its invocation and exit or signal, followed
by separately labelled stdout and stderr excerpts when those streams are nonempty.
Each excerpt trims outer whitespace, replaces invalid UTF-8, retains at most 2048
Unicode scalar values and marks truncation with an ellipsis. Internal line breaks
are retained in the execution result. Counts are parsed from the complete captured
output before presentation; diagnostic truncation does not change the verdict.
<!-- raf:tested-by raf_workplan::bound::tests::a_failing_test_fails_the_binding,raf_workplan::test_crate::bound::a_failing_test_fails_the_binding -->

A bound test that runs past its timeout is killed and fails its binding, and the
reason says that it was killed, with the time it ran and its limit.
<!-- raf:claim evidence=gate-kills-a-test-past-its-timeout -->

A bound test that writes more than the bound is killed and fails its binding in
the same way, and the reason says that it was killed, with the time it ran and the
bound.

### Verification without persistence
<!-- raf:tested-by raf_workplan::test_crate::verify::one_passing_and_one_failing_test_print_one_line_each_and_the_run_fails,raf_workplan::test_crate::verify::no_slug_runs_every_capability_and_a_slug_only_the_ones_the_change_lists,raf_workplan::test_crate::verify::verify_writes_nothing,raf_workplan::test_crate::verify::verify_asks_git_nothing_and_does_not_go_through_the_version_gate -->

`verify [<slug>]` runs the bound tests and the bound examples of the named change's
capabilities, or of every capability when no slug is given, and prints pass, fail or not-applicable for
each binding. Examples run as their target or documentation test. Evidence is not
persisted: `verify` reports the run and writes nothing. Exit code 0 means every
binding passed or has an explicit platform exclusion. `verify` asks `git` nothing beyond the listing of tracked paths, and
goes through no Git version gate.

An executed failure keeps its stored reason when rendered on one line: whitespace
is flattened without clipping away its stream excerpts. The standard process
runner's captured streams supply the bounded excerpts described above; custom
runner failure reasons are retained as supplied, without a new size guarantee.
Long unresolved or not-applicable reasons still use their existing head and tail
summary.
<!-- raf:tested-by cargo_workplan::test_crate::verify::one_passing_and_one_failing_test_print_one_line_each_and_exit_one -->

### Compiler-observed platform exclusion
<!-- raf:tested-by raf_workplan::bound::tests::platform_exclusion_is_distinct_from_a_pass_and_reuses_the_observation,raf_workplan::bound::tests::platform_unknown_or_applicable_still_requires_a_listed_test,raf_workplan::bound::tests::malformed_compiler_configuration_fails_without_a_host_fallback,raf_workplan::test_crate::bound::real_cargo_platform_exclusion_preserves_a_passing_neighbour,raf_workplan::bound::tests::a_platform_guarded_doc_binding_uses_rustdoc_listing_not_rustc_configuration,raf_workplan::exec::tests::compiler_platform_queries_pin_the_package_and_target,raf_workplan::test_crate::complete::completion_retains_the_justified_host_nonexecution_receipt,raf_workplan::test_crate::verify::a_known_opposite_host_binding_is_reported_without_a_passing_execution_proof,cargo_workplan::test_crate::verify::host_nonapplicability_is_exit_zero_and_preserved_in_machine_output -->

When a test or executable-example binding has a platform condition, the executor observes the configuration
Cargo selects for its pinned package and target with `cargo rustc --frozen
--package=<pkg> <target> -- --print=cfg`. Every resolved definition must have a
guard proved false before the result is `not-applicable`. Its receipt names the
guards, observed OS, families, bare platform atoms and query. The observation is
reused within the executor; a reused receipt does not invent another process.
No test-list or pass count is recorded for nonexecution, and `Evidence::passed`
remains false. Unknown predicates and applicable definitions still require the
ordinary proof, including refusal of zero listed tests. A failed or malformed
configuration query fails the binding; no compiler-host fallback substitutes for
the selected target.

Documentation tests retain ordinary rustdoc listing and execution: rustdoc-specific
flags can differ from rustc flags, so a rustc observation cannot authorize their
nonexecution.


Completion prepares one checked repository view through the supplied runner
before executing a bound test or example. The view contains the original
selected inputs with the change record removed and the generated replacements
applied. Static document checks and binding resolution use the captured inputs;
bound execution uses their completed transformation. Compiler configuration,
applicability observations, test listing and execution all carry that same
prepared context.

The capture includes ignored and untracked inputs, binary contents, native
permissions, directory membership and lexical links. Equal original file
identities retain their hardlink relationship, except where a generated
replacement breaks that relationship as ordinary publication does. Materialized
files have their own identities, disjoint from the original inputs. Absolute
links within the captured repository are relocated in the backing copy;
the process view preserves their original logical targets so checked native
readers retain the same containment boundary. Links
that leave the admitted repository and unsupported entries refuse preparation.
Generated additions retain the native creation permissions observed by the
publication writer; existing rewritten files retain their original permissions.

Exclusions require observed Git administration, registered sibling worktrees,
Cargo output directories or the owned execution destination. An exclusion that
contains a tracked or loaded input refuses. An external output link can identify
an independent Cargo workspace only when native workspace discovery and that
workspace's actual metadata agree on the exact output root. Its manifest, lock
and first-party configuration remain captured inputs. Other manifests are not
queried merely because they occur in the repository.
Native Git materialization creates a
legitimate detached worktree with its own index derived from the original
index. Only the selected removal and generated replacements alter its tracked
set; unrelated untracked inputs remain untracked.

Native Linux execution requires `bwrap`. Its process-local mount view exposes
checked copied inputs at their original logical paths and keeps ambient paths
above the selected Git top unchanged. Cargo discovers the same first-party
configuration once, including absolute input paths and configured executable
runners. The physical backing copy retains distinct file identities. Temporary
ancestor directories preserve observed permission modes and are read-only;
changes to writable copied inputs are detected at process boundaries. Those
process-local directory identities are distinct from original and backing-copy
identities.

The original Git observation honors the caller's effective administration,
worktree, common-directory and index selectors. Checked bounded binary capture
copies the effective index into an exclusively owned directory before Git derives
its tree; Git's cache-tree updates cannot change the caller's original index. Native preparation and bound
children explicitly select the owned worktree and its copied index. Shared Git
administration is read-only in the process view; only the exact owned
worktree administration and declared outputs are writable there. Receipts retain
the original effective index path and the owned selector paths. A missing tool,
namespace capability or invalid metadata path fails with its native cause;
there is no original-root execution fallback.

`Programs::discover_git` explicitly selects independent repository discovery for
unscoped Git queries, ignoring inherited directory, worktree, common-directory
and index selectors. The default retains those selectors. Explicit invocation
and prepared-view contexts take precedence over independent discovery, so a
bound process continues to use its admitted owned index.
Git query outputs retain the effective selector used for failure diagnostics;
scripted outputs declare that context explicitly rather than observe the host
environment. Cargo and trusted-command outputs carry no Git selector receipt.

The runner checks the completed copy against the requested transformation before
admitting it and revalidates its selected inputs around every prepared process.
It also revalidates the original selection and completion model before
publication. Changed bytes, permissions, links, membership, index or exclusion
provenance refuse completion. An execution receipt retains its prepared context,
original logical root, capture digest and tracked-index provenance. A virtual
receipt is explicitly distinguishable from a physical execution receipt.

Preparation and cleanup belong to the supplied runner. A missing preparation
implementation refuses explicitly. A context from another runner, an unknown
context and a released context cannot execute. Cleanup is explicit and fallible
on success, refusal and error paths; a cleanup failure blocks publication and
retains the preceding failure as well. Only the owned physical destination and
any empty parent created for it may be removed.
<!-- raf:tested-by raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal,raf_workplan::test_crate::complete::a_run_that_cannot_start_is_an_error_and_not_a_refusal,raf_workplan::test_crate::complete::a_document_edited_while_the_tests_ran_refuses_and_nothing_is_written -->

## Usage

Use `Lifecycle::open_with(&gate, runner)` to control every process invocation of
the lifecycle. The runner must use the gate's directory and answer the tracked
queries unless the gate has an explicit tracked set. `Lifecycle::open(programs)`
retains the ordinary system-runner path.
<!-- raf:tested-by raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal,raf_workplan::test_crate::complete::a_prefixed_slug_names_the_same_change -->

The repository's clock admission runs in the existing repository gates under
make check. Changes to clock or runner syntax must preserve the admitted standard
clock computation and production selection; unsupported forms fail with their
source location. Scripted clocks remain available only through test scopes.
<!-- raf:tested-by rust_agentic_foundations::test_crate::repo_gates::time_in_crates_tracked_files_name_no_clock -->

The runnable API examples in `raf_workplan::bound` exercise resolution of the tests and examples that the completion gate executes.

```sh
cargo workplan change reconcile incremental-parsing
cargo workplan verify incremental-parsing
cargo workplan change complete incremental-parsing
cargo workplan verify
```

`complete` is the final command. Before it, `docs preview` shows the capability as it
reads, and the author sets `state: delivered` on each listed capability and runs
`docs sync`. `verify` without a slug runs the bindings of every capability.

When a bound process fails, read the labelled stdout excerpt for captured test
assertions and the stderr excerpt for Cargo or compiler causes. A long stream's
ellipsis identifies an excerpt rather than a complete process transcript.


For native completion, configure SystemRunner for the original repository and
keep selected inputs stable until the operation finishes. Completion prepares
and releases its detached execution view; callers do not create a second
worktree or redirect bound commands to the original root.

A supplied scripted runner must explicitly support preparation. VirtualViews
provides checked capture, context admission, revalidation and release without
claiming physical execution. FakeRunner requires a view expectation and uses
on_view answers for prepared invocations. Ordinary on answers do not satisfy
prepared invocations.

## Examples

A scripted runner can open and complete a lifecycle even when the gate's configured
Git executable cannot start: the supplied answers provide the tracked paths at
opening and refresh. A missing answer or failing tracked-list response instead
returns its error and leaves every document byte unchanged.
<!-- raf:tested-by raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal,raf_workplan::test_crate::complete::a_run_that_cannot_start_is_an_error_and_not_a_refusal -->

Parsed source fixtures refuse a constant elapsed value despite an unused
Instant, a different constructor, an alternate production clock or selection,
and unsupported generation. Formatting and comments preserve admission. A
neighbor with inherited test-only clock implementations passes; a condition
that also permits a production feature does not hide an implementation.
<!-- raf:tested-by rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_a_planted_clock_reading_with_file_and_line -->

The failed-test fixture reports its literal panic cause, this test fails, from
libtest's captured stdout alongside Cargo's stderr failure. Its neighboring test
in the same target still passes. The scripted case places a result line beyond
the stdout excerpt and retains the failed count from the original bytes.
<!-- raf:tested-by raf_workplan::bound::tests::a_failing_test_fails_the_binding,raf_workplan::test_crate::bound::a_failing_test_fails_the_binding -->

A refusal, and then success once the author has recorded the outcome of the item and
synced the generated blocks:

```text
$ cargo workplan change complete incremental-parsing
refused: 2 obligations, 1 item without outcome, 1 stale generated block
change:incremental-parsing  item 2  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
docs/capabilities/index.md  document  generated-stale  line 4: a docs sync run (the `catalogue` block is stale)
$ cargo workplan docs sync && cargo workplan change complete incremental-parsing
completed: removed work/changes/incremental-parsing.md
rewrote docs/capabilities/streaming-parser.md
```

Each fault prints one line, and the headline counts it under each condition that states
it: the item without outcome is an obligation and a fault of condition 2, and the stale
catalogue an obligation and a fault of condition 4. The `rewrote` line is the
relationship table of the capability, which no longer names the removed record as a
change.

A refusal on the fifth condition, where a bound example no longer builds:

```text
$ cargo workplan change complete incremental-parsing
refused: 1 obligation, 1 bound example failing
cap:streaming-parser  examples  example-failing  a passing verify run for the change, or a binding edit that removes the broken example (`raf_parser::example_crate::chunked` fails to build or run (exit code 101))
```

A refusal on the sixth condition, where the Usage section was assessed before the
bound `push` changed:

```text
$ cargo workplan change complete incremental-parsing
refused: 1 obligation, 1 section without a current accepted assessment
cap:streaming-parser  usage  assessment-stale  the section assessed again with assess (`assessment:streaming-parser-usage`: the symbol `raf_parser::Parser::push` has changed since)
```

When the Usage section itself was also edited after it was assessed, its
assessment has two moved inputs: the refusal prints two `assessment-stale` lines,
one naming the section and one naming `raf_parser::Parser::push`, and its headline
reads `refused: 2 obligations, 1 section without a current accepted assessment`.


The native configuration case keeps a first-party rustflag present once, reads
an absolute binary input, retains the logical working directory and executes
the configured absolute Cargo runner. The source root refuses an unadmitted
write, while ordinary copied-input changes refuse before publication.

A bound self-check can require the completed repository to contain no selected
change record. It fails when run against the original repository and succeeds
in the prepared view, where the generated documents also describe the completed
state. Binary assets, executable permissions, internal lexical links and
hardlinks remain part of that view. A changed selected input or a mismatched
prepared context instead refuses completion and preserves the original record.

## Constraints

Supplying a runner controls process execution, not filesystem effects. The
lifecycle still reads and writes through its disk facilities. This interface
does not provide an in-memory write store or remove the host runner and clock
implementations from the core crate.
<!-- raf:tested-by raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal -->

- `complete` runs only bindings that resolve in the source index and match their
  patterns, and counts a binding as passed only by the proof its kind requires: for a
  test binding, that exactly one test ran; for a doc-test binding, that every doc test
  the list pass selected reported `... ok`; for an example binding, that the example
  exited with status 0 within the timeout. It never passes document text to `cargo` as
  a shell string or as an option.
  <!-- raf:tested-by raf_workplan::test_crate::complete::hostile_bindings_refuse_and_never_reach_cargo,raf_workplan::bound::tests::zero_listed_entries_are_unresolved_and_nothing_is_run,raf_workplan::bound::tests::every_selected_entry_must_be_ok,raf_workplan::test_crate::exec::every_invocation_is_the_row_of_its_table -->
- The proof guards against a filter that selects nothing, more than one test or the
  wrong one; it is not a defence against the code it runs. A bound test or example is
  workspace code and trusted build input. The caller-selected Cargo executable's
  list and result output is read as reported; valid output with status 0 does not
  attest that the executable built or ran the indexed source.
  <!-- raf:tested-by raf_workplan::test_crate::verify::selected_cargo_output_is_trusted_but_an_empty_list_is_refused,raf_workplan::bound::tests::two_listed_entries_are_unresolved,raf_workplan::bound::tests::one_entry_under_another_name_is_unresolved,raf_workplan::bound::tests::result_counts_preserve_unsigned_spelling_and_refuse_host_overflow -->
- `complete` runs the bound tests and examples with `cargo`, which builds each of
  them first, so a bound test or example that cannot compile fails its binding and
  the change that lists its capability cannot complete.
  <!-- raf:tested-by raf_workplan::bound::tests::a_list_pass_that_does_not_build_fails_the_binding,raf_workplan::test_crate::complete::each_kind_of_obligation_makes_complete_refuse_and_leaves_the_tree_as_it_was_and_its_repair_completes -->
- A capability with empty `bindings.tests` and `bindings.examples` lists gives the
  fifth condition nothing to run, which suffices only while it is `designed` or
  `considered`: in `implementing` and `delivered` an empty list is a
  `binding-missing` obligation of condition 1 (the `examples` list only in
  `implementing`).
  <!-- raf:tested-by raf_workplan::test_crate::complete::empty_bindings_satisfy_condition_five_only_while_the_capability_is_designed_or_considered -->
- The record is deleted and not archived. Recovering it is a Git operation.
  <!-- raf:tested-by raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal -->
- `complete` never edits prose. It rewrites only the generated blocks that removing
  the record stales; the prose, and the `state: delivered` that the post-removal
  check requires, are written by a person or an agent before the command runs.
  <!-- raf:tested-by raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal,raf_workplan::test_crate::complete::a_capability_left_implementing_with_no_other_record_refuses_and_says_what_to_change -->
- The second judgement re-evaluates the directory after execution and refuses a
  newly failing condition or changed binding set. A record replaced by a link is
  refused before writing. These two directory judgements provide no atomic snapshot
  across the subsequent publication; the trust limit is described below.
  <!-- raf:tested-by raf_workplan::test_crate::complete::a_document_edited_while_the_tests_ran_refuses_and_nothing_is_written,raf_workplan::test_crate::complete::a_binding_added_while_the_tests_ran_is_a_change_even_when_every_condition_still_holds,raf_workplan::test_crate::complete::a_record_linked_while_the_tests_ran_is_refused_at_the_second_judgement -->
- Publication uses the second evaluation's computed `SyncReport` through
  `Workplan::write`, which replaces the current file bytes without comparing them
  with the bytes used to compute the report. A document edited after that
  evaluation can therefore be replaced by the earlier snapshot. Record removal
  likewise uses its judged path rather than a retained file handle. The workspace
  must remain trusted during publication; completion is not an atomic snapshot.
  <!-- raf:tested-by raf_workplan::test_crate::sync::write_publishes_the_computed_snapshot_without_comparing_later_authored_edits -->


Repository capture admits at most 262144 entries, 64 MiB per regular file and
512 MiB of regular-file contents in aggregate. Missing, inaccessible,
unsupported or oversized inputs fail with an actionable cause. Capture retains
native permissions but does not transport ownership, ACLs or extended
attributes. A prepared view is a checked execution input, not a sandbox for
untrusted workspace code or a substitute for authentic compiler output.
Source parsing retains its independent load limits.
<!-- raf:tested-by raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal,raf_workplan::test_crate::complete::a_run_that_cannot_start_is_an_error_and_not_a_refusal -->

Native signed Git fixtures require Git 2.34 or newer and OpenSSH `ssh-keygen`.
Their ephemeral signing key and allowed-signers file live in the fixture's Git
administration directory. Each fixture commit is genuinely signed and verified;
the public runtime Git minimum remains 2.30.
<!-- raf:tested-by raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal -->

The native execution view requires installed Git, Cargo and `bwrap`, with host
support for its native mount namespace, and uses checked filesystem facilities. Virtual preparation retains its explicit scripted
semantics; it does not establish that Cargo ran against an in-memory tree. This
interface does not split host process and clock implementations from the core
crate. Standard validation covers native Linux only.
<!-- raf:tested-by raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal,raf_workplan::test_crate::complete::a_run_that_cannot_start_is_an_error_and_not_a_refusal -->

## Explanation

The two directory judgements do not protect the interval after the second
judgement. A document edited, or a directory on the way to the record replaced by
a link, between that judgement and the writes is not detected: the rewrite
replaces a file without comparing it with the text it was computed from, and the
removal goes by the path judged. These operations require a trusted workspace;
they are neither an attestation nor an atomic compare-and-publish operation.

Documentation is part of completing a change because the alternative is a document
that is correct at design time and drifts from the product afterwards. A change that
names the documentation it creates or alters when it is opened, and cannot finish
until each part is current, makes the maintenance step part of the work.

Obligations scale with the change. Requiring a usage example for a refactor teaches
people to write meaningless ones; letting an internal change record that it touches no
contract keeps the gate credible where the contract does move.

Deleting the record keeps the reading path short. After completion, the capability
document is the one account; the claims, the discarded options and the intermediate
states are in Git, where an entity-aware history query can find them. The rationale
that still matters has already been written into the Explanation section.


Judging documents after record removal while running their bindings against the
original repository gives the two checks different inputs. A prepared view
joins the static model, source bindings and actual execution to one explicit
transformation. Original logical paths address copied files in the native mount
view. Receipts distinguish those paths, the physical backing copy and the
process-local ancestor directories; none claims original physical identity.

## Verification

The existing completion cases exercise supplied tracked queries at initial opening
and post-run refresh, with unusable gate-configured Git. They assert invocation
ordering, repeated listing, exact successful document outcomes and unchanged
snapshots for missing and failing answers at both stages. Existing Gate cases
retain the explicit-tracked bypass, configured working directory and recognized
non-repository behavior.
<!-- raf:tested-by raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal,raf_workplan::test_crate::complete::a_run_that_cannot_start_is_an_error_and_not_a_refusal,raf_workplan::test_crate::gate::the_programs_of_a_gate_are_started_in_its_directory,raf_workplan::test_crate::gate::the_tracked_paths_a_caller_gives_replace_the_listing_and_start_no_process -->

Production clock admission combines the real repository source check with
handwritten parsed-source refusal neighbors. The existing counted scripted
runner cases establish limit decisions without reading host time.
<!-- raf:tested-by rust_agentic_foundations::test_crate::repo_gates::time_in_crates_tracked_files_name_no_clock,rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_a_planted_clock_reading_with_file_and_line -->

The process smoke test starts the real Git executable. Scripted Cargo runs
exercise the process adapter independently of workspace-root discovery, which
reads manifests without invoking Cargo.

The gate has two flagship acceptance fixtures, built as repositories in temporary
directories.

**Scenario A: several redesigns, one readable present.** One capability passes through
four redesigns by successive changes. Decisions are amended and reversed, and
interfaces and constraints evolve. At the end the fixture asserts: the document
states the active contract directly; every link resolves; no PROPOSED block or change
file is on the reading path; the Explanation section still holds the rationale; and
every earlier state is recoverable with `git show`. The test is whether a reader needs
to reconstruct the changes to understand what applies now.

**Scenario B: planning produces the documentation.** The fixture starts with a
capability in `considered` with open questions. The implementation renames an API and
drops one guarantee. `change complete` must refuse until Usage, Examples and the
bindings match the new API and the dropped claim has the outcome `dropped`. It then
succeeds and leaves the capability `delivered` with an accurate capability
explanation, a real usage example, current API references, explicit limitations and
a test traceability table, and no proposal text presented as established
behaviour. A document that merely exists does not satisfy
the scenario.

Further checks:

- A refusal fixture per condition: a remaining obligation, an item without outcome, a
  leftover PROPOSED block, a stale generated block, a failing bound test and a bound
  example that fails to build each produce exit 1 and leave the record in place.
  The example refusal prints an `example-failing` obligation for the capability.
- A refusal fixture per obligation kind, each leaving the tree byte-identical, beside
  its repair, which completes.
- State fixtures: a listed capability left `implementing` with no other open record
  refuses with `no-open-record`, a `delivered` capability beside its own open record
  completes, and one with a binding that does not resolve refuses; a broken link in a
  capability the change does not list, and a link in prose to the record, refuse.
- A `contract_effect: none` fixture that permits an untouched Contract, refuses
  untouched affected Usage and Examples, and completes after both are revised;
  another remaining obligation still refuses completion.
- A fixture asserting the record is gone after success and the working tree has no
  other change than the rewrite of the generated blocks the removal stales.
- Order fixtures: a static failure starts no `cargo` process; a document edited, a
  binding added and a test renamed while the bound tests ran each refuse with nothing
  written; a rewrite that fails leaves the record in place.
- A binding fixture runs each row of the invocation table against a stub that the
  library is given as the program `$CARGO` names; the stub records each argv, and the
  fixture asserts it equals the row, with `--frozen`, `--package=` and the pinned
  target, and with `Wrapper<T>::get` arriving as one word, so no shell is involved.
  With `$CARGO` unset the program is `cargo`, and a stub named `cargo` in the
  directories searched as `PATH` is the one that runs.
- A hostile binding fixture: a `bindings.tests` entry `-Zunstable-options`, one
  `--include-ignored`, one `--exact`, and a path with a `;` or a space each yield an
  unresolved-binding finding, make `complete` refuse and `verify` exit 1, and are
  never passed to `cargo`.
- A zero-test fixture: a binding that resolves in the source index to a function that
  is not a `#[test]` would run zero tests and is refused before any process starts, so
  `complete` refuses and `verify` exits 1; so
  does an `#[ignore]`d test, and a test whose name matches in two targets runs only
  in the pinned target.
- A doc-test fixture: a binding to a documented function with a passing doc test runs
  that doc test with the bare item path as the filter and passes only when its
  `test <full name> ... ok` line appears; the same fixture with a failing doc test
  makes `complete` refuse; a doc-test binding that matches no doc test refuses; a
  selected entry missing from the run output refuses; an item whose substring also
  matches a failing `foo_more` doc test refuses; an item `Wrapper<T>::get` binds and
  passes; a `no_run` doc test and a `compile_fail` doc test, each bound as a doc test,
  are refused with the message that asks for a runnable doc test; and an item path
  that begins with `-` never reaches `cargo`: bound, it does not resolve, and given
  to the plan, it is refused by its pattern.
- An example fixture: a hyphenated example target runs, and an example that reads
  stdin sees it closed.
- A never-ending fixture, on Linux: a bound test and a bound example that never
  end by themselves run through the executor and the real `cargo` with a limit of
  5 seconds. The list pass of the test runs under the clock of the machine and the
  limit of the gate, and the run under a clock that moves only by the pauses the
  runner asks for, so the limit passes after a bounded number of looks and nothing
  waits for it. Each fails its binding with the reason `ran 5.0 s of its 5 s
  limit`, pinned word for word, as a run that timed out after exactly 5 seconds;
  the test binary that `cargo` started is in the group `cargo` leads, ignores
  SIGHUP, SIGINT and SIGTERM, and never runs the rest of its test after the kill,
  so only SIGKILL to the group passes; and the passing test beside the one that
  never ends passes. On other systems the mapping of a kill to a failed binding is
  pinned through the fake runner only.
- A `verify` fixture with one passing and one failing bound test, asserting per-binding
  output and exit codes 0 and 1.

- A refusal fixture for each way condition 6 fails, each beside its passing
  neighbour: an affected section with no assessment, one edited after its assessment,
  and one whose current assessment is `rejected` (by a human and by an agent) each
  refuse with the failure counted under condition 6 and the tree byte-identical, and
  each completes once the section is assessed and accepted.
- Every other fixture assesses the sections its change affects with one shared
  helper, through the library, so it is judged on what it is about; a section whose
  assessment cannot be recorded gets none, so what it owes stays visible. Scenario B
  holds the step where the account is implemented and settled and its sections are
  not assessed, which refuses only for the missing assessments, each an
  `assessment-missing` obligation and a fault of condition 6, and completes once they
  are.
- The completion writes only the rewrite and the removal: the assessment records are
  left as they are.
- A kill at the timeout and a kill for too much output each fail the binding with
  a reason that gives the time it ran and the limit it passed, pinned word for
  word, and the run keeps that time. The time the real runner gives is the
  reading of the runner's clock at the kill, pinned with a test clock: under a
  clock that moves only by the pauses the runner asks for, a kill at the limit
  gives exactly the limit, and under a clock that never moves, a kill for too much
  output and a run that ends each give that clock's reading. On Linux a process
  killed at its limit, and one it started that holds the output pipes, never run
  the rest of their script after the kill, though the script ignores SIGHUP,
  SIGINT and SIGTERM and each process is sent SIGCONT afterwards, so only
  SIGKILL passes; this holds also with no `kill` program on `PATH`, and the
  process the runner started is reaped. A group signal that finds no live
  member is done, and so is one refused with `EPERM` once the leader has ended on
  macOS, which answers so while the only member is exiting: the leader is then sent
  SIGKILL itself and collected, and counts as ended only when that signal is
  delivered or finds it gone, so a leader still running is killed and reaped
  first. Any other refusal, and `EPERM` on any other system, is reported, since
  the group may still be running.
  A process that writes more than a pipe
  holds to both streams runs to its end under the clock that never moves, so
  the two streams are read at once. The repository source gate independently
  verifies the standard monotonic clock computation and its actual production
  selection, so a constant-zero production clock refuses admission. Native
  canonical trait identities and inherited test roles distinguish production
  implementations from scripted clocks. Parsed valid and invalid source data
  exercise the admission boundary; no host-clock reading or calibration is
  asserted.
- The headline counts sections for condition 6: one section whose assessment has
  three moved inputs (its text and two bound symbols) prints three
  `assessment-stale` lines under `3 obligations, 1 section without a current
  accepted assessment`, and two sections with one moved input each are two
  sections; a unit fixture pins the same counts and that the same section of
  another capability is another section.
- A directory link inside the root at `work`, made after the repository is opened,
  is refused before any `cargo` process or write, with the record behind it kept;
  a linked `work/changes` leaves the change not found, with nothing run; the same
  completion through the real directories completes. The refusal names the first
  judgement and says that nothing was written.
- A change record replaced by a symbolic link to a file in the root while the bound
  tests run still loads, and is refused by the second judgement: the error names
  the record, says that the bound tests and examples ran and that nothing was
  written, and the link and the file behind it are kept; the same completion with
  a run that changes nothing completes.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::test_crate::complete::a_completed_change_prints_what_was_removed_and_rewritten_and_leaves_a_clean_check` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::a_failing_bound_test_is_condition_five_and_the_record_stays` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::a_failing_example_stands_under_conditions_one_and_five` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::host_nonapplicability_is_exit_zero_and_preserved_in_machine_output` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::verify::one_passing_and_one_failing_test_print_one_line_each_and_exit_one` | test | [Contract](#contract) |
| `raf_workplan::bound` | doctest | [Usage](#usage) |
| `raf_workplan::bound::tests::a_benchmark_is_not_a_test_entry` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_benchmark_of_the_name_is_not_a_listed_test` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_binary_test_is_pinned_with_bin` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_cfg_test_module_of_the_library_runs_as_lib_with_the_name_in_the_crate` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_code_binding_and_an_item_that_is_no_example_are_refused` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_compile_fail_doc_test_is_refused_with_a_request_for_a_runnable_one` | test | [Contract](#contract) |
| `raf_workplan::bound::tests::a_doc_item_path_is_refused_unless_it_matches_its_pattern` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_doc_list_entry_has_the_form_of_the_pattern` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_doc_list_entry_with_the_platform_separator_is_read_and_keeps_its_printed_name` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_doc_status_is_never_taken_from_another_name_or_a_loose_line` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_doc_status_is_read_from_the_whole_line` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_doc_test_binding_lists_the_package_and_runs_the_bare_item_path` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_doc_test_passes_only_on_its_own_ok_line` | test | [Contract](#contract) |
| `raf_workplan::bound::tests::a_failing_selected_doc_test_fails_the_binding_by_name` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_failing_sibling_the_filter_also_matches_fails_the_binding` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_failing_test_fails_the_binding` | test | [Contract](#contract), [Examples](#examples) |
| `raf_workplan::bound::tests::a_function_that_is_not_a_test_is_refused_by_name` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_hostile_example_name_and_package_never_reach_the_runner` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_hostile_name_in_the_manifest_never_reaches_the_runner` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_kill_at_either_limit_fails_the_binding_with_the_time_it_ran_and_its_limit` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_killed_test_is_a_failed_binding_that_reports_the_time` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_list_pass_that_does_not_build_fails_the_binding` | test | [Constraints](#constraints) |
| `raf_workplan::bound::tests::a_list_with_no_entry_has_none` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_list_with_one_entry_has_one_entry` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_list_with_two_entries_has_two` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_loose_ok_line_is_not_proof` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_nested_test_of_an_integration_test_keeps_the_path_inside_the_crate` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_no_run_doc_test_is_refused_with_a_request_for_a_runnable_one` | test | [Contract](#contract) |
| `raf_workplan::bound::tests::a_passing_result_is_read` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_platform_guarded_doc_binding_uses_rustdoc_listing_not_rustc_configuration` | test | [Contract](#contract) |
| `raf_workplan::bound::tests::a_result_line_without_its_counts_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_run_that_cannot_start_is_an_error_and_not_a_verdict` | test | [Contract](#contract) |
| `raf_workplan::bound::tests::a_run_that_passes_one_test_with_more_passed_is_refused` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_selected_entry_with_no_ok_line_is_refused` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_status_other_than_zero_is_never_a_pass_whatever_the_counts_say` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_target_without_the_harness_or_with_required_features_is_refused` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_test_in_a_benchmark_target_is_refused` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_test_of_a_library_module_is_named_by_its_path_in_the_crate` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_test_of_a_target_with_required_features_is_refused` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_test_of_a_target_without_the_harness_is_refused` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::a_test_with_clean_facts_is_planned_and_its_neighbours_are_refused` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::an_example_killed_at_its_limit_fails_with_the_elapsed_time` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::an_example_runs_under_its_cargo_name_and_the_exit_status_decides` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::an_example_with_clean_facts_is_planned_and_hostile_ones_are_refused` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::an_ignored_selected_doc_test_is_refused` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::an_ignored_test_is_refused_though_the_list_shows_it` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::an_impl_item_is_bound_under_its_self_type_as_rustdoc_writes_it` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::an_integration_test_with_a_hyphen_is_pinned_under_its_cargo_name` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::an_older_result_line_without_the_time_is_read` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::another_entrys_ok_line_does_not_stand_for_the_selected_one` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::every_pass_carries_the_limits_of_its_kind` | test | [Contract](#contract) |
| `raf_workplan::bound::tests::every_selected_entry_must_be_ok` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::bound::tests::evidence_reads_as_a_verdict_line_and_the_processes_that_ran` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::hostile_bindings_never_reach_the_runner` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::malformed_compiler_configuration_fails_without_a_host_fallback` | test | [Contract](#contract) |
| `raf_workplan::bound::tests::no_result_line_and_two_result_lines_are_unresolved` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::no_result_line_is_no_result` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::no_selected_entry_is_a_refusal_and_runs_nothing` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::noise_around_the_one_entry_does_not_matter` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::noise_lines_are_not_entries` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::one_entry_under_another_name_is_unresolved` | test | [Constraints](#constraints) |
| `raf_workplan::bound::tests::platform_exclusion_is_distinct_from_a_pass_and_reuses_the_observation` | test | [Contract](#contract) |
| `raf_workplan::bound::tests::platform_unknown_or_applicable_still_requires_a_listed_test` | test | [Contract](#contract) |
| `raf_workplan::bound::tests::result_counts_preserve_unsigned_spelling_and_refuse_host_overflow` | test | [Constraints](#constraints) |
| `raf_workplan::bound::tests::the_doc_list_gives_names_and_item_parts_across_blocks` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::the_pinned_target_is_the_library_a_binary_or_an_integration_test` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::the_root_of_the_crate_has_no_item_part` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::too_much_output_is_a_failed_binding` | test | [Contract](#contract) |
| `raf_workplan::bound::tests::two_listed_entries_are_unresolved` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::bound::tests::two_result_lines_are_two_results` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::windows_line_endings_are_read` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::zero_listed_entries_are_unresolved_and_nothing_is_run` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::bound::tests::zero_passed_failed_and_ignored_results_are_read` | test | [Verification](#verification) |
| `raf_workplan::bound::tests::zero_passed_is_unresolved_even_with_exit_status_zero` | test | [Verification](#verification) |
| `raf_workplan::exec` | doctest | [Verification](#verification) |
| `raf_workplan::exec::system::tests::a_descendant_that_holds_the_pipes_after_the_exit_is_not_waited_for_past_the_limit` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::a_group_that_already_exited_is_done` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::a_group_whose_leader_exited_before_the_kill_is_killed_without_error` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::a_leader_still_running_is_killed_and_collected_before_it_counts_as_ended` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::a_leader_that_exited_has_ended_and_is_collected` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::a_process_inside_its_limit_passes` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::a_process_that_never_stops_writing_is_killed_at_the_bound` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::a_process_that_outlives_its_limit_is_killed` | test | [Contract](#contract) |
| `raf_workplan::exec::system::tests::a_refused_signal_is_done_only_when_the_leader_has_exited` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::a_signal_to_the_leader_ends_it_only_when_delivered_or_it_is_gone` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::a_zero_limit_is_passed_at_once_unless_the_run_is_done` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::any_other_failure_is_reported_even_after_the_leader_exited` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::both_streams_are_drained_at_once` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::cargo::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::cargo::an_example_that_never_ends_is_killed_and_reported_with_the_time_it_ran` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::output_at_the_bound_passes_and_one_byte_over_is_refused_on_each_stream` | test | [Contract](#contract) |
| `raf_workplan::exec::system::tests::terminating_a_leader_that_exited_before_the_kill_succeeds` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::the_capture_stops_one_byte_past_its_bound_and_flags_the_overflow` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::the_group_is_killed_when_no_kill_program_is_on_the_path` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::the_pauses_double_up_to_the_ceiling_and_stay_there` | test | [Verification](#verification) |
| `raf_workplan::exec::system::tests::the_verdict_follows_its_precedence` | test | [Contract](#contract) |
| `raf_workplan::exec::system::tests::what_the_process_started_does_not_outlive_the_kill` | test | [Verification](#verification) |
| `raf_workplan::exec::tests::a_literal_is_the_only_free_text_and_it_is_static` | test | [Verification](#verification) |
| `raf_workplan::exec::tests::a_rejected_value_never_becomes_an_argument` | test | [Verification](#verification) |
| `raf_workplan::exec::tests::compiler_platform_queries_pin_the_package_and_target` | test | [Contract](#contract) |
| `raf_workplan::exec::tests::every_variant_is_walked` | test | [Verification](#verification) |
| `raf_workplan::exec::tests::no_shell::every_argument_is_one_element_and_no_shell_reads_it` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::completion::tests::a_completion_prints_the_removal_and_each_rewritten_file` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::completion::tests::a_section_with_three_moved_inputs_is_one_section_in_the_headline` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::completion::tests::a_section_without_an_accepted_assessment_counts_under_condition_six_and_prints_once` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::completion::tests::a_stale_block_prints_a_line_unless_an_obligation_already_does` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::completion::tests::conditions_are_numbered_as_the_contract_numbers_them` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::completion::tests::the_counts_of_a_refusal_are_singular_or_plural_and_in_the_order_of_the_conditions` | test | [Verification](#verification) |
| `raf_workplan::staged::tests::a_replaced_file_reads_as_its_new_text_within_the_bound` | test | [Verification](#verification) |
| `raf_workplan::staged::tests::every_other_path_is_answered_as_the_inner_tree_does` | test | [Verification](#verification) |
| `raf_workplan::staged::tests::the_hidden_file_is_not_read_listed_or_found` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::a_failing_test_fails_the_binding` | test | [Contract](#contract), [Examples](#examples) |
| `raf_workplan::test_crate::bound::a_function_that_is_not_a_test_is_refused_before_any_process` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::a_hyphenated_example_runs_under_its_cargo_name` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::a_passing_test_passes_on_one_listed_entry_and_one_passed` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::a_process_that_cannot_start_is_an_error_and_not_a_verdict` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::a_test_behind_a_disabled_feature_lists_nothing_and_is_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::a_test_of_a_binary_runs_under_bin` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::an_example_that_exits_zero_passes_and_one_that_does_not_fails` | test | [Contract](#contract) |
| `raf_workplan::test_crate::bound::an_example_that_reads_standard_input_sees_it_closed` | test | [Contract](#contract) |
| `raf_workplan::test_crate::bound::an_ignored_test_is_listed_and_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::an_integration_test_with_a_hyphen_runs_under_its_cargo_name` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::an_item_path_that_begins_with_a_hyphen_never_reaches_cargo` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::documentation_tests_follow_the_rules_in_edition_2021` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::documentation_tests_follow_the_rules_in_edition_2024` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::hostile_bindings_are_unresolved_and_never_reach_cargo` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::real_cargo_platform_exclusion_preserves_a_passing_neighbour` | test | [Contract](#contract) |
| `raf_workplan::test_crate::bound::targets_cargo_cannot_select_a_test_in_are_refused_before_any_process` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::the_same_name_in_two_targets_runs_only_in_the_pinned_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_binding_added_while_the_tests_ran_is_a_change_even_when_every_condition_still_holds` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::a_block_that_is_stale_or_missing_anywhere_refuses_condition_four` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_broken_link_in_a_capability_the_change_does_not_list_also_blocks_completion` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_capability_left_implementing_with_no_other_record_refuses_and_says_what_to_change` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::a_delivered_capability_beside_its_open_record_is_what_complete_resolves` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_delivered_capability_with_a_binding_that_does_not_resolve_refuses` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::a_document_edited_while_the_tests_ran_refuses_and_nothing_is_written` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::a_failing_bound_example_stands_under_conditions_one_and_five` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_failing_bound_test_refuses_condition_five_alone` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_failing_doc_test_a_test_that_runs_nothing_and_an_ignored_test_refuse_beside_passing_neighbours` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_linked_work_directory_is_refused_before_anything_runs_and_a_real_one_completes` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_none_change_reads_each_file_of_the_bound_package_once_at_the_merge_base_and_otherwise_none` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_none_change_that_edits_a_bound_signature_refuses` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_prefixed_slug_names_the_same_change` | test | [Usage](#usage) |
| `raf_workplan::test_crate::complete::a_proposed_block_of_the_change_refuses_condition_three_in_a_loaded_and_in_an_unloaded_document` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_record_linked_while_the_tests_ran_is_refused_at_the_second_judgement` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::a_record_removed_while_the_tests_ran_is_an_error_and_nothing_is_written` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_run_that_cannot_start_is_an_error_and_not_a_refusal` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::a_section_with_three_moved_inputs_is_one_section_in_the_refusal_headline` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_static_failure_spends_no_process_on_cargo_and_a_clean_change_runs_cargo_after_the_static_checks` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::a_test_renamed_while_the_tests_ran_refuses` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::an_agent_assessment_satisfies_condition_six_and_a_rejected_one_by_either_never_does` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::an_edit_of_a_bound_body_alone_refuses` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::an_edit_of_a_bound_doc_comment_refuses` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::an_error_after_the_rewrite_does_not_leave_a_removed_record_beside_stale_blocks` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::an_item_dropped_without_a_reason_refuses_condition_two` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::an_item_without_outcome_and_a_move_to_nowhere_or_to_itself_refuse_condition_two` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::an_item_without_outcome_and_a_stale_block_print_the_counts_of_the_contract` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::an_outcome_the_profile_refuses_makes_the_record_not_load_and_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::an_untouched_usage_or_examples_of_a_none_change_refuses` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::complete_accepts_the_change_prefix_and_an_unknown_slug_or_a_record_that_did_not_load_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::completion_after_record_removal_enforces_the_complete_policy` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::completion_retains_the_justified_host_nonexecution_receipt` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::conditions_and_failures_have_stable_names_and_the_headline_is_the_first_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::contract_effect_none_leaves_only_contract_untouched_and_preserves_other_refusals` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::each_kind_of_obligation_makes_complete_refuse_and_leaves_the_tree_as_it_was_and_its_repair_completes` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::empty_bindings_satisfy_condition_five_only_while_the_capability_is_designed_or_considered` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::hostile_bindings_refuse_and_never_reach_cargo` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::prose_that_links_to_the_record_refuses_because_the_link_is_broken_once_it_is_removed` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::the_cases_cover_every_kind_of_obligation_in_both_directions` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::the_check_once_the_record_is_removed_applies_the_debt_record_so_a_held_finding_does_not_refuse` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::the_check_once_the_record_is_removed_refuses_a_debt_row_that_is_resolved` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::writes::a_removal_that_fails_reports_the_files_already_rewritten_and_removes_nothing_else` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::writes::a_rewrite_that_fails_leaves_the_record_and_the_blocks_as_they_were` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::an_opposite_host_binding_cannot_publish_execution_evidence` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::a_directory_that_does_not_exist_is_a_spawn_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::a_generic_list_holds_exactly_the_characters_of_the_pattern` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::a_name_missing_from_the_search_path_is_a_spawn_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::a_program_that_does_not_exist_is_a_spawn_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::a_ref_cannot_be_read_as_an_option_inside_its_argument` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::a_refusal_names_the_pattern_and_shows_the_value_safely` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::a_runner_reference_is_a_runner` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::a_test_filter_always_follows_a_separator_and_never_leads` | test | [Contract](#contract) |
| `raf_workplan::test_crate::exec::a_tree_path_refuses_every_hostile_value_that_is_not_printable_path_text` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::each_newtype_is_built_only_by_its_pattern` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::every_invocation_is_the_row_of_its_table` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::exec::every_pattern_accepts_what_it_states` | test | [Contract](#contract) |
| `raf_workplan::test_crate::exec::every_pattern_refuses_hostile_values` | test | [Contract](#contract) |
| `raf_workplan::test_crate::exec::every_pattern_refuses_what_it_does_not_state` | test | [Contract](#contract) |
| `raf_workplan::test_crate::exec::stubs::a_failing_exit_code_and_a_signal_are_outputs_and_not_errors` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::stubs::a_file_that_cannot_be_executed_is_a_spawn_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::stubs::a_stub_named_by_cargo_runs_for_cargo_and_a_stub_on_the_path_when_it_is_unset` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::stubs::every_row_reaches_a_process_as_exactly_its_argv` | test | [Contract](#contract) |
| `raf_workplan::test_crate::exec::stubs::nothing_else_is_added_to_the_environment` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::stubs::output_is_returned_byte_for_byte` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::stubs::standard_input_is_closed` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::stubs::the_c_locale_is_set_when_the_output_is_parsed_and_left_alone_otherwise` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::stubs::the_default_bound_is_sixteen_mebibytes_and_a_larger_one_is_a_parameter` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::stubs::the_process_runs_in_the_working_directory` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::the_fake_answers_a_scripted_invocation_and_records_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::the_fake_applies_the_output_bound_as_the_real_runner_does` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::the_fake_applies_the_time_limit_as_the_real_runner_does` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::the_fake_keeps_the_program_apart_from_the_arguments` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::the_fake_refuses_an_invocation_it_has_no_answer_for` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::the_fake_repeats_an_answer_and_a_later_one_replaces_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::the_fake_returns_a_failing_exit_and_a_scripted_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::the_limits_are_the_documented_bounds` | test | [Contract](#contract) |
| `raf_workplan::test_crate::exec::the_patterns_are_the_ones_the_documents_state` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::the_programs_come_from_cargo_and_the_path_and_can_be_replaced` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::the_real_git_starts` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::the_programs_of_a_gate_are_started_in_its_directory` | test | [Contract](#contract) |
| `raf_workplan::test_crate::gate::the_tracked_paths_a_caller_gives_replace_the_listing_and_start_no_process` | test | [Contract](#contract) |
| `raf_workplan::test_crate::scenarios::a_document_that_merely_exists_does_not_satisfy_the_scenario` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::a_thin_document_is_not_the_account_the_scenario_builds` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::each_fault_of_the_account_has_a_document_that_breaks_only_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::planning_produces_the_documentation` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::several_redesigns_leave_one_readable_present` | test | [Verification](#verification) |
| `raf_workplan::test_crate::sync::write_publishes_the_computed_snapshot_without_comparing_later_authored_edits` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::verify::a_capability_that_did_not_load_is_an_error_and_not_a_pass` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::a_known_opposite_host_binding_is_reported_without_a_passing_execution_proof` | test | [Contract](#contract) |
| `raf_workplan::test_crate::verify::a_process_that_cannot_start_is_an_error_and_not_a_failing_binding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::a_run_with_nothing_bound_passes_and_prints_nothing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::an_example_that_cannot_be_started_is_an_error_and_not_a_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::an_example_that_exits_with_a_failure_or_a_doc_test_that_fails_is_a_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::an_unknown_slug_and_a_record_that_did_not_load_are_different_errors` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::examples_and_doc_tests_are_run_and_judged_by_their_proof` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::hostile_bindings_are_unresolved_results_that_never_reach_cargo` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::no_slug_runs_every_capability_and_a_slug_only_the_ones_the_change_lists` | test | [Contract](#contract) |
| `raf_workplan::test_crate::verify::one_passing_and_one_failing_test_print_one_line_each_and_the_run_fails` | test | [Contract](#contract) |
| `raf_workplan::test_crate::verify::only_the_examples_of_the_capabilities_the_change_lists_are_run` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::selected_cargo_output_is_trusted_but_an_empty_list_is_refused` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::verify::the_lines_of_a_report_are_its_text_and_each_verdict_has_a_name_and_a_reason` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::verify_asks_git_nothing_and_does_not_go_through_the_version_gate` | test | [Contract](#contract) |
| `raf_workplan::test_crate::verify::verify_writes_nothing` | test | [Contract](#contract) |
| `raf_workplan::test_crate::verify::zero_tests_and_ignored_tests_fail_the_run` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_a_planted_clock_reading_with_file_and_line` | test | [Contract](#contract), [Examples](#examples) |
| `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_tracked_files_name_no_clock` | test | [Contract](#contract), [Usage](#usage) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [workplan-generated-docs](./workplan-generated-docs.md) |
| Depends on | [workplan-reconcile](./workplan-reconcile.md) |
| Depended on by | [workplan-cli](./workplan-cli.md) |
| Depended on by | [workplan-prose-assessment](./workplan-prose-assessment.md) |
| Depended on by | [workplan-strict](./workplan-strict.md) |
<!-- raf:generated:end -->

The completion cases pair original-root failure with completed-view success and
exercise checked binary contents, modes, links, hardlinks, exact tracked
selection and released contexts. Refusal cases cover changed inputs and invalid
prepared ownership beside passing neighbours. Scripted completion checks remain
separate evidence from actual native Git and Cargo execution.
