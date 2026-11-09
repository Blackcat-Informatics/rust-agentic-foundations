---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-cli
title: Workplan command line
description: Static repository commands and explicit owned data publication with separate documentation trust.
state: delivered
bindings:
  code:
    - cargo_workplan::bin_crate::cargo_workplan::main
    - cargo_workplan::bin_crate::cargo_workplan::cli
    - cargo_workplan::bin_crate::cargo_workplan::command
    - cargo_workplan::bin_crate::cargo_workplan::change
    - cargo_workplan::bin_crate::cargo_workplan::docs
    - cargo_workplan::bin_crate::cargo_workplan::docs::fragment
    - cargo_workplan::bin_crate::cargo_workplan::cli::FragmentArgs
    - cargo_workplan::bin_crate::cargo_workplan::render
    - cargo_workplan::bin_crate::cargo_workplan::result_graph
    - raf_workplan::root
    - cargo_workplan::bin_crate::cargo_workplan::mdbook
    - cargo_workplan::bin_crate::cargo_workplan::command::standalone
    - cargo_workplan::bin_crate::cargo_workplan::command::diagram
    - cargo_workplan::bin_crate::cargo_workplan::cli::parse
    - cargo_workplan::bin_crate::cargo_workplan::cli::Args
    - cargo_workplan::bin_crate::cargo_workplan::command::run
    - cargo_workplan::bin_crate::cargo_workplan::command::check_format
    - cargo_workplan::bin_crate::cargo_workplan::command::export
    - cargo_workplan::bin_crate::cargo_workplan::cli::Docs
    - cargo_workplan::bin_crate::cargo_workplan::cli::ExportArgs
    - cargo_workplan::bin_crate::cargo_workplan::exit::Exit
    - raf_workplan::root::discover
    - cargo_workplan::bin_crate::cargo_workplan::history::history
    - cargo_workplan::bin_crate::cargo_workplan::evidence::claims
    - cargo_workplan::bin_crate::cargo_workplan::evidence::support
    - cargo_workplan::bin_crate::cargo_workplan::assess::assess
    - cargo_workplan::bin_crate::cargo_workplan::coverage::coverage
    - cargo_workplan::bin_crate::cargo_workplan::render::Report
    - raf_workplan::escape
    - cargo_workplan::bin_crate::cargo_workplan::command::LABEL
    - cargo_workplan::bin_crate::cargo_workplan::change::complete
    - cargo_workplan::bin_crate::cargo_workplan::change::completion_records
  tests:
    - cargo_workplan::test_crate::rdf::static_fragments_preserve_aliases_and_all_bindings_without_compiler_authority
    - cargo_workplan::test_crate::complete::prospective_refusal_retains_the_record_and_every_input_byte
    - cargo_workplan::test_crate::process::a_missing_git_binary_does_not_prevent_native_reads
    - cargo_workplan::test_crate::reconcile::bound_example_bodies_are_inert_and_reconciliation_writes_nothing
    - cargo_workplan::test_crate::trusted_docs::documentation_trust_is_explicit_and_configuration_cannot_select_a_test_driver
    - cargo_workplan::bin_crate::cargo_workplan::trusted_docs::configuration::tests::compiler_flags_wrappers_and_includes_cannot_change_the_closed_doc_operation
    - cargo_workplan::test_crate::verify::verify_prints_static_results_without_git_or_consumer_execution
    - cargo_workplan::test_crate::verify::trusted_docs_is_refused_before_root_discovery_and_general_execution_is_absent
    - cargo_workplan::test_crate::complete::completion_prints_the_owned_changes_and_leaves_a_clean_static_check
    - cargo_workplan::test_crate::evidence::stored_command_text_is_never_executed_by_checks_or_dependency_views
    - cargo_workplan::test_crate::process::a_usage_error_starts_no_process
    - cargo_workplan::test_crate::history::stored_reviews_report_item_changes_after_the_reviewed_commit_is_gone
    - cargo_workplan::test_crate::rdf::standalone_publication_preserves_host_files_and_refuses_changed_owned_bytes
    - cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms
    - raf_source::test_crate::root::member_and_virtual_workspace_directories_resolve_without_sources
    - raf_source::test_crate::root::standalone_excluded_and_nested_packages_have_their_own_roots
    - raf_source::test_crate::root::transitive_path_dependencies_are_members
    - raf_source::test_crate::root::explicit_workspace_requires_a_safe_path_and_agreed_membership
    - raf_source::test_crate::root::malformed_manifests_and_missing_directories_fail_hard
    - raf_source::test_crate::root::manifest_size_bound_is_enforced_beside_the_exact_limit
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::diagnostic_write_failure_is_propagated_and_quiet_does_not_write_notes
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::quiet_filters_results_without_changing_exit_and_machine_output_stays_a_graph
    - cargo_workplan::test_crate::rdf::check_preserves_every_finding_location_in_both_equivalent_formats
    - cargo_workplan::test_crate::rdf::quiet_keeps_findings_and_an_empty_result_is_still_a_canonical_graph
    - cargo_workplan::test_crate::rdf::status_preserves_ordered_capabilities_binding_gaps_and_open_changes
    - cargo_workplan::test_crate::rdf::preview_preserves_reconciled_text_and_sync_reports_only_actual_rewrites
    - cargo_workplan::test_crate::rdf::failures_have_no_machine_output_and_json_is_rejected_before_root_loading
    - raf_workplan::test_crate::root::tree_directory_aliases_fail_instead_of_selecting_lexical_ancestors
    - raf_workplan::test_crate::root::a_dangling_manifest_is_a_finding_instead_of_an_ancestor_fallback
    - raf_workplan::test_crate::root::memory_tree_membership_discovery_needs_no_disk_adapter
    - raf_workplan::test_crate::root::a_directory_outside_any_workspace_is_an_actionable_error
    - raf_workplan::test_crate::root::a_nonexistent_directory_is_refused
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::each_command_parses_in_both_shapes
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_takes_a_slug_repeated_capabilities_and_affects_and_the_spdx_pair
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_needs_a_capability_and_both_or_neither_spdx_option
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_default_and_parse_in_both_shapes
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_sync_text_options_parse
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::formats_are_closed_and_review_accepts_json
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_missing_command_an_unknown_command_and_a_missing_slug_are_usage_errors
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_that_starts_with_a_dash_is_a_usage_error_however_it_is_given
    - cargo_workplan::bin_crate::cargo_workplan::exit::tests::the_three_exits_have_the_documented_codes
    - cargo_workplan::test_crate::check::a_clean_repository_exits_zero_and_prints_nothing
    - cargo_workplan::test_crate::check::a_repository_with_findings_exits_one_and_prints_each_finding
    - cargo_workplan::test_crate::check::the_findings_are_those_of_the_library_gate_in_the_same_order
    - cargo_workplan::test_crate::check::the_exit_agrees_with_assert_clean_and_every_finding_is_in_its_message
    - cargo_workplan::test_crate::check::usage_errors_exit_two_with_a_message_on_standard_error
    - cargo_workplan::test_crate::status::the_roadmap_groups_one_capability_in_each_state_with_the_gaps_and_the_change
    - cargo_workplan::test_crate::preview::the_proposed_text_is_printed_and_the_markers_are_not
    - cargo_workplan::test_crate::preview::an_unknown_slug_is_exit_two_and_names_the_open_changes
    - cargo_workplan::test_crate::sync::a_missing_index_with_the_text_is_created_with_that_header
    - cargo_workplan::test_crate::sync::only_the_generated_blocks_are_rewritten
    - cargo_workplan::test_crate::new::new_prints_the_library_text_writes_the_record_and_nothing_else_in_both_shapes
    - cargo_workplan::test_crate::new::the_printed_template_in_a_capability_of_each_state_gives_a_clean_check_and_one_unresolved_proposal
    - cargo_workplan::test_crate::impact::impact_prints_what_the_change_reaches_and_writes_nothing
    - cargo_workplan::test_crate::reconcile::nothing_is_written_and_no_cargo_is_started_for_a_fixture_without_bound_examples
    - cargo_workplan::test_crate::process::discovery_starts_no_cargo_from_the_cargo_variable
    - cargo_workplan::test_crate::process::discovery_starts_no_cargo_on_path_when_the_variable_is_unset
    - cargo_workplan::test_crate::process::discovery_does_not_run_the_build_rustc_program_of_the_workspace
    - cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_by_every_command_before_any_process_starts
    - cargo_workplan::test_crate::process::cargo_dispatches_workplan_to_the_binary_on_path_and_passes_the_exit_code_back
    - cargo_workplan::test_crate::root::the_root_is_found_from_the_root_and_from_any_directory_below_it
    - cargo_workplan::test_crate::root::every_command_finds_the_root_from_a_subdirectory
    - cargo_workplan::test_crate::check::a_directory_that_is_not_a_repository_exits_two_and_says_why_on_standard_error
    - cargo_workplan::test_crate::check::a_finding_is_gone_once_it_is_fixed
    - cargo_workplan::test_crate::check::a_manifest_that_does_not_parse_exits_two
    - cargo_workplan::test_crate::check::a_root_that_does_not_exist_exits_two
    - cargo_workplan::test_crate::check::a_source_file_that_cannot_be_read_exits_two
    - cargo_workplan::test_crate::check::a_stale_generated_block_is_a_finding_and_a_sync_clears_it
    - cargo_workplan::test_crate::check::help_and_version_exit_zero_on_standard_output
    - cargo_workplan::test_crate::check::quiet_keeps_the_findings_and_the_exit_code
    - cargo_workplan::test_crate::check::quiet_on_a_clean_repository_prints_nothing_and_exits_zero
    - cargo_workplan::test_crate::impact::a_change_that_reaches_dependants_and_linking_sections_lists_them_sorted
    - cargo_workplan::test_crate::impact::a_record_that_does_not_load_is_exit_two_with_its_findings
    - cargo_workplan::test_crate::impact::a_slug_with_no_record_is_exit_two_and_names_the_open_changes
    - cargo_workplan::test_crate::impact::impact_finds_the_root_from_below_and_by_root
    - cargo_workplan::test_crate::impact::rdf_preserves_impact_bindings_dependencies_links_and_list_order
    - cargo_workplan::test_crate::impact::quiet_prints_nothing_and_the_exit_stays_zero
    - cargo_workplan::test_crate::new::a_capability_given_twice_is_listed_once
    - cargo_workplan::test_crate::new::a_capability_that_is_missing_or_a_bad_option_is_a_usage_error_before_anything_is_read
    - cargo_workplan::test_crate::new::a_second_run_for_the_same_slug_is_exit_two_and_leaves_the_record_as_it_is
    - cargo_workplan::test_crate::new::affects_is_repeatable_and_the_record_holds_the_entries_given_in_order
    - cargo_workplan::test_crate::new::every_refusal_of_the_library_is_exit_two_with_the_tree_untouched
    - cargo_workplan::test_crate::new::rdf_preserves_created_change_fields_and_ordered_guidance
    - cargo_workplan::test_crate::new::new_works_from_a_subdirectory_and_by_root
    - cargo_workplan::test_crate::new::quiet_writes_the_record_and_prints_nothing
    - cargo_workplan::test_crate::new::spdx_text_that_is_not_one_line_is_exit_two_and_writes_nothing
    - cargo_workplan::test_crate::new::the_note_about_the_generated_blocks_is_there_when_the_record_makes_them_stale
    - cargo_workplan::test_crate::new::the_spdx_lines_are_the_two_comment_lines_of_the_records_of_this_repository
    - cargo_workplan::test_crate::new::the_template_left_in_a_state_that_refuses_it_is_the_finding_the_guidance_names
    - cargo_workplan::test_crate::preview::a_capability_the_change_lists_that_has_no_document_is_exit_two
    - cargo_workplan::test_crate::preview::a_change_record_that_did_not_load_is_exit_two_and_says_so
    - cargo_workplan::test_crate::preview::a_slug_that_is_not_a_slug_is_a_usage_error
    - cargo_workplan::test_crate::preview::quiet_prints_nothing_because_a_preview_is_not_a_finding
    - cargo_workplan::test_crate::preview::the_output_is_the_text_the_library_previews_under_a_header_for_each_document
    - cargo_workplan::test_crate::preview::the_preview_is_the_same_by_root_from_another_directory
    - cargo_workplan::test_crate::process::discovery_succeeds_when_cargo_cannot_be_started
    - cargo_workplan::test_crate::process::a_slug_that_is_not_a_slug_is_refused_before_any_process_starts
    - cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_before_any_process_starts
    - cargo_workplan::test_crate::process::a_usage_error_of_the_change_commands_starts_no_process
    - cargo_workplan::test_crate::process::root_starts_no_cargo_at_all
    - cargo_workplan::test_crate::reconcile::everything_that_stops_it_looking_is_exit_two
    - cargo_workplan::test_crate::reconcile::rdf_preserves_each_obligation_and_optional_location_in_library_order
    - cargo_workplan::test_crate::reconcile::quiet_keeps_the_obligations_and_hides_nothing_that_is_a_finding
    - cargo_workplan::test_crate::root::a_directory_outside_any_workspace_is_exit_two_and_names_the_command
    - cargo_workplan::test_crate::root::a_relative_root_is_taken_from_the_current_directory
    - cargo_workplan::test_crate::root::root_overrides_the_discovery_and_the_current_directory_is_not_read
    - cargo_workplan::test_crate::root::sync_from_a_subdirectory_writes_at_the_root_and_nowhere_else
    - cargo_workplan::test_crate::status::a_directory_that_is_not_a_repository_is_exit_two
    - cargo_workplan::test_crate::status::a_state_with_no_capability_is_a_row_with_a_count_of_zero
    - cargo_workplan::test_crate::status::an_implementing_capability_with_every_binding_resolved_has_no_gap_beyond_its_empty_lists
    - cargo_workplan::test_crate::status::quiet_leaves_standard_output_empty_because_the_roadmap_has_no_finding
    - cargo_workplan::test_crate::status::the_roadmap_is_the_same_by_root_from_another_directory
    - cargo_workplan::test_crate::sync::a_document_that_does_not_load_is_exit_two_and_nothing_is_written
    - cargo_workplan::test_crate::sync::a_missing_index_without_the_text_is_exit_two_naming_the_field_and_writes_nothing
    - cargo_workplan::test_crate::sync::a_refused_file_is_exit_two_and_nothing_is_written
    - cargo_workplan::test_crate::sync::a_second_sync_writes_nothing_and_prints_nothing
    - cargo_workplan::test_crate::sync::a_value_that_cannot_be_written_into_the_header_is_refused_when_the_index_is_created
    - cargo_workplan::test_crate::sync::each_field_that_is_not_given_is_named
    - cargo_workplan::test_crate::sync::quiet_hides_the_files_rewritten_and_keeps_the_exit_code
    - cargo_workplan::test_crate::sync::the_index_is_created_by_root_from_another_directory
    - cargo_workplan::test_crate::sync::the_text_is_not_read_when_the_index_exists
    - raf_workplan::test_crate::root::the_root_is_found_from_the_root_a_member_and_a_subdirectory
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_command_that_takes_a_slug_refuses_one_that_starts_with_a_dash_in_both_forms
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_must_be_a_slug
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::help_and_version_exit_zero_in_both_shapes
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_may_follow_the_command
    - cargo_workplan::bin_crate::cargo_workplan::exit::tests::a_library_error_becomes_its_display_text
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::assess_takes_a_section_and_exactly_one_mode
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_takes_a_direction_and_an_optional_package
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_claims_takes_an_optional_capability
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_lines_takes_an_optional_package_and_reverse_is_the_same_command
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::evidence_claims_and_for_take_one_id_each
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::history_refuses_an_id_or_a_day_before_anything_runs
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::history_takes_an_id_of_each_kind_and_a_day
    - cargo_workplan::test_crate::assess::a_section_that_cannot_be_assessed_is_exit_two_and_writes_nothing
    - cargo_workplan::test_crate::assess::a_verdict_writes_exactly_one_record_and_a_rejection_still_exits_zero
    - cargo_workplan::test_crate::assess::the_context_is_the_same_whoever_asks_and_writes_nothing
    - cargo_workplan::test_crate::assess::the_list_shows_each_assessment_current_until_an_input_moves
    - cargo_workplan::test_crate::coverage::an_exclusion_takes_a_gap_out_and_a_report_without_gaps_exits_zero
    - cargo_workplan::test_crate::coverage::crate_selects_a_package_by_either_name_and_refuses_one_that_is_not_there
    - cargo_workplan::test_crate::coverage::gaps_in_either_direction_are_printed_and_exit_one
    - cargo_workplan::test_crate::coverage::this_repository_has_no_code_or_test_gaps_in_its_scoped_packages
    - cargo_workplan::test_crate::claims::every_claim_names_a_test_prints_the_report_and_exits_zero
    - cargo_workplan::test_crate::claims::a_claim_that_names_no_test_is_a_finding_and_exits_one_in_text_and_rdf_alike
    - cargo_workplan::test_crate::claims::a_capability_that_is_not_loaded_exits_two_and_says_why
    - cargo_workplan::test_crate::lines::a_repository_whose_every_counted_line_is_covered_prints_the_report_and_exits_zero
    - cargo_workplan::test_crate::lines::an_orphan_line_is_a_finding_and_exits_one_in_text_and_rdf_alike_and_reverse_prints_the_same
    - cargo_workplan::test_crate::lines::a_package_that_is_not_one_exits_two_and_says_why
    - cargo_workplan::test_crate::lines::a_ceiling_row_and_its_class_decide_the_exit_of_the_default_check_on_the_binary
    - cargo_workplan::test_crate::lines::an_exception_that_matches_nothing_fails_check_under_default_and_its_matched_neighbour_passes
    - cargo_workplan::test_crate::lines::an_exception_is_the_excepted_table_and_a_lines_excepted_object_and_the_report_exits_zero
    - cargo_workplan::test_crate::assess::the_text_of_a_section_reaches_the_terminal_escaped_and_rdf_escapes_it_as_rdf
    - raf_workplan::escape::tests::each_control_and_bidirectional_formatting_character_is_escaped
    - raf_workplan::escape::tests::lines_keep_their_line_feeds_and_tabs_and_lose_every_other_control
    - cargo_workplan::test_crate::history::a_capability_lists_its_document_and_symbol_commits_newest_first_and_a_removed_binding
    - cargo_workplan::test_crate::history::a_shallow_clone_is_exit_two_and_its_full_clone_answers
    - cargo_workplan::test_crate::history::rdf_preserves_full_history_commits_sources_paths_and_removed_bindings
    - cargo_workplan::test_crate::history::quiet_prints_nothing_because_a_history_is_not_a_finding
    - cargo_workplan::test_crate::history::since_limits_the_commits_and_a_change_or_an_unknown_id_is_answered_or_refused
    - cargo_workplan::test_crate::coverage::a_surface_that_cannot_be_listed_is_a_finding_of_the_code_report_only
    - cargo_workplan::test_crate::history::a_bound_file_that_is_not_committed_is_exit_two_naming_it_until_it_is
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_removed_policy_flag_is_rejected_before_and_after_the_command
    - cargo_workplan::test_crate::check::the_default_check_accepts_the_complete_fixture
    - cargo_workplan::test_crate::check::the_default_check_refuses_a_thin_section_without_a_policy_flag
    - cargo_workplan::test_crate::check::the_default_check_without_capability_documents_is_a_run_failure
    - cargo_workplan::test_crate::check::a_removed_policy_key_is_refused_beside_the_default_content_refusal
    - cargo_workplan::test_crate::check::policy_findings_keep_typed_locations_and_messages_in_rdf
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
    - cargo_workplan::test_crate::assess::moved_inputs_preserve_raw_unresolved_reasons_only_for_unresolved_changes
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::diagnostic_flush_failure_propagates_and_quiet_never_touches_that_sink
    - cargo_workplan::test_crate::process::help_and_version_output_failures_exit_two_beside_successful_output
  examples:
    - raf_workplan::example_crate::docs_sync
    - raf_workplan::Workplan::sync
    - raf_workplan::Workplan::preview
    - raf_workplan::Workplan::new_change
    - raf_workplan::Workplan::impact
    - raf_workplan::escape::shown_lines
depends: [cap:workplan-complete-gate, cap:workplan-generated-docs, cap:workplan-evidence-claims]
---

## Purpose

Expose the library's static contract and explicit data operations through consistent human and machine reports. Command parsing and root discovery treat repository contents as data, and compilation requires its own explicit documentation authority.

## Contract

`docs fragment` returns complete raw Markdown for one imported Cargo library, an explicit public receiving module, capability and existing named diagram. Every code binding of that capability is required. The native static fragment API checks source and imported library identities and refuses private, hidden, unresolved or ambiguous references before emitting output. Diagnostics use stderr. Quiet mode and non-text formats refuse before input loading.
<!-- raf:tested-by cargo_workplan::test_crate::rdf::static_fragments_preserve_aliases_and_all_bindings_without_compiler_authority -->

**Static contract.** `cargo workplan COMMAND` and `cargo-workplan COMMAND` use the same parser. Root discovery reads manifests through the native source model. Ordinary checks, history, coverage, review and evidence queries launch no process. Verification reports declared bindings and static findings; completion checks a prospective document overlay and publishes owned documentation only.
<!-- raf:tested-by cargo_workplan::test_crate::verify::verify_prints_static_results_without_git_or_consumer_execution,cargo_workplan::test_crate::complete::completion_prints_the_owned_changes_and_leaves_a_clean_static_check -->

Only `docs build rustdoc --trust-docs` admits tool execution, with explicit package, toolchain and output selection. The flag is required on each invocation before root discovery. It accepts arbitrary build-time code effects without enabling test, example, doctest or arbitrary-command operations. Existing artifact inspection is separate and never builds or publishes.
<!-- raf:tested-by cargo_workplan::test_crate::verify::trusted_docs_is_refused_before_root_discovery_and_general_execution_is_absent -->

Exit zero denotes a successful static outcome or explicit data operation; exit one denotes findings; exit two denotes inability to answer or invalid usage. Quiet mode hides outcomes and retains findings. Text is escaped for terminal display; RDF carriers retain typed values. Saved command strings remain inert metadata.
<!-- raf:tested-by cargo_workplan::test_crate::evidence::stored_command_text_is_never_executed_by_checks_or_dependency_views,cargo_workplan::test_crate::process::a_usage_error_starts_no_process -->

## Usage

The documented data path is illustrated by `raf_workplan::example_crate::docs_sync`. These examples are source references for a caller to inspect; RAF does not initiate their execution.

The commands are `check`, `status`, `diagram`, `docs sync|preview|export|mdbook|fragment|artifacts|build`, `change new|impact|reconcile|complete`, `verify`, `history`, `evidence claims|for`, `assess`, `review`, `coverage code|tests|claims|lines` and `debt --write`. Global options select the root, output format and quiet mode. Help lists command-specific closed arguments.

Use `docs fragment --package PACKAGE --metadata META --target-directory OUTPUT --view OWNER#NAME --module MODULE --capability CAPABILITY`. The target directory must already exist and match the imported metadata. The source index uses the same bounded static default load as `diagram`; metadata features do not select effective compiler configuration. Save stdout through a caller-owned operation when an include file is wanted.

## Examples

`cargo workplan docs fragment --package demo --metadata metadata.json --target-directory artifacts --view cap:api#dependencies --module demo::nested --capability cap:api` emits a static include for a documented public module. A public alias retains its exported spelling in ordinary intra-doc references. Changing another required binding to a private or missing item refuses the complete fragment.

`cargo workplan verify shape` prints declared bindings and static findings for the repository. It never prints a consumer test execution verdict. `cargo workplan docs artifacts` reads imported metadata and messages. A missing documentation trust flag refuses even if the requested workspace is absent.

For a missing code binding, `cargo workplan --format turtle check` emits this
complete RDF 1.2 report while preserving the finding's location and repair text:

```turtle
VERSION "1.2"
BASE <https://blackcatinformatics.ca/.well-known/genid/raf-report/909ce0bb63eae85a7c56dc7483631148fac2059fcfd06d804f4a8bdf59fb4e33>
<> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://blackcatinformatics.ca/raf/report/1#Report> .
<> <https://blackcatinformatics.ca/raf/report/1#command> "check" .
<> <https://blackcatinformatics.ca/raf/report/1#reportForm> "1"^^<http://www.w3.org/2001/XMLSchema#integer> .
<> <https://blackcatinformatics.ca/raf/report/1#result> <#result/0> .
<#result/0> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://blackcatinformatics.ca/raf/report/1#Finding> .
<#result/0> <https://blackcatinformatics.ca/raf/report/1#file> "docs/capabilities/beta.md" .
<#result/0> <https://blackcatinformatics.ca/raf/report/1#fix> "write the canonical path of an item the source index defines (check the spelling, the module declarations and the target the item is in), or remove the entry; a name that a macro, derive or attribute generates is not in the index and cannot be bound" .
<#result/0> <https://blackcatinformatics.ca/raf/report/1#key> "bindings" .
<#result/0> <https://blackcatinformatics.ca/raf/report/1#kind> "binding-unresolved" .
<#result/0> <https://blackcatinformatics.ca/raf/report/1#message> "cap:beta code binding `demo::missing` does not resolve: `demo::missing` names nothing in the workspace" .
<#result/0> <https://blackcatinformatics.ca/raf/report/1#ordinal> "0"^^<http://www.w3.org/2001/XMLSchema#integer> .
```

## Constraints

Fragment generation reads data and starts no tool, build or trusted documentation operation. Its static source index and imported metadata establish neither compiler success, effective configuration, resolved Rustdoc links, package membership nor assembled-host behavior. Raw fragment Markdown is a dedicated output carrier, separate from escaped terminal reports and RDF projections.
<!-- raf:tested-by cargo_workplan::test_crate::rdf::static_fragments_preserve_aliases_and_all_bindings_without_compiler_authority -->

- RAF never edits inspected source or manifests. Document sync, change completion, authored metadata records, debt and owned documentation publication are explicit writes. The documentation trust exception permits build-time code to have arbitrary side effects; it is not a containment guarantee.
<!-- raf:tested-by cargo_workplan::test_crate::verify::trusted_docs_is_refused_before_root_discovery_and_general_execution_is_absent -->

## Explanation

The fragment command supplies the native static API with the shared named bundle, the complete capability binding selection, the exact imported library and metadata digest, and the loaded receiving module. The renderer supplies the script-free diagram and readable relationships; Rustdoc remains responsible for final intra-doc URL resolution.

CLI results own domain values so human rendering and machine projections share one static answer. No ambient pre-parser compiler dispatch or command-executing evidence recorder exists.

## Verification

The CLI fragment fixture exercises crate and nested-module output, public aliases and complete bindings, imported source mismatch, private, hidden, missing and ambiguous references, input preservation, missing compiler tools and early output-carrier refusals in both invocation forms.

CLI fixtures compare static verification with the library, exercise prospective completion and refusal, reject missing trust before discovery, and keep stored commands inert.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_command_that_takes_a_slug_refuses_one_that_starts_with_a_dash_in_both_forms` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_missing_command_an_unknown_command_and_a_missing_slug_are_usage_errors` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_must_be_a_slug` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_that_starts_with_a_dash_is_a_usage_error_however_it_is_given` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::assess_takes_a_section_and_exactly_one_mode` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_needs_a_capability_and_both_or_neither_spdx_option` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_takes_a_slug_repeated_capabilities_and_affects_and_the_spdx_pair` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_claims_takes_an_optional_capability` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_lines_takes_an_optional_package_and_reverse_is_the_same_command` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_takes_a_direction_and_an_optional_package` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::debt_takes_write_and_the_spdx_pair_together_or_not_at_all` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::each_command_parses_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::evidence_claims_and_for_take_one_id_each` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::formats_are_closed_and_review_accepts_json` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::help_and_version_exit_zero_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::history_refuses_an_id_or_a_day_before_anything_runs` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::history_takes_an_id_of_each_kind_and_a_day` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_default_and_parse_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_may_follow_the_command` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_removed_policy_flag_is_rejected_before_and_after_the_command` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_sync_text_options_parse` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::exit::tests::a_library_error_becomes_its_display_text` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::exit::tests::the_three_exits_have_the_documented_codes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::diagnostic_flush_failure_propagates_and_quiet_never_touches_that_sink` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::diagnostic_write_failure_is_propagated_and_quiet_does_not_write_notes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::quiet_filters_results_without_changing_exit_and_machine_output_stays_a_graph` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::trusted_docs::configuration::tests::compiler_flags_wrappers_and_includes_cannot_change_the_closed_doc_operation` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::assess::a_section_that_cannot_be_assessed_is_exit_two_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::assess::a_verdict_writes_exactly_one_record_and_a_rejection_still_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::assess::moved_inputs_preserve_raw_unresolved_reasons_only_for_unresolved_changes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::assess::the_context_is_the_same_whoever_asks_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::assess::the_list_shows_each_assessment_current_until_an_input_moves` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::assess::the_text_of_a_section_reaches_the_terminal_escaped_and_rdf_escapes_it_as_rdf` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_clean_repository_exits_zero_and_prints_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_directory_that_is_not_a_repository_exits_two_and_says_why_on_standard_error` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_finding_is_gone_once_it_is_fixed` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_manifest_that_does_not_parse_exits_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_removed_policy_key_is_refused_beside_the_default_content_refusal` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_repository_with_findings_exits_one_and_prints_each_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_root_that_does_not_exist_exits_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_source_file_that_cannot_be_read_exits_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_stale_generated_block_is_a_finding_and_a_sync_clears_it` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::help_and_version_exit_zero_on_standard_output` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::policy_findings_keep_typed_locations_and_messages_in_rdf` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::quiet_keeps_the_findings_and_the_exit_code` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::quiet_on_a_clean_repository_prints_nothing_and_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_default_check_accepts_the_complete_fixture` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_default_check_refuses_a_thin_section_without_a_policy_flag` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_default_check_without_capability_documents_is_a_run_failure` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_exit_agrees_with_assert_clean_and_every_finding_is_in_its_message` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_findings_are_those_of_the_library_gate_in_the_same_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::usage_errors_exit_two_with_a_message_on_standard_error` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::claims::a_capability_that_is_not_loaded_exits_two_and_says_why` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::claims::a_claim_that_names_no_test_is_a_finding_and_exits_one_in_text_and_rdf_alike` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::claims::every_claim_names_a_test_prints_the_report_and_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::completion_prints_the_owned_changes_and_leaves_a_clean_static_check` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::complete::prospective_refusal_retains_the_record_and_every_input_byte` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::coverage::a_surface_that_cannot_be_listed_is_a_finding_of_the_code_report_only` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::coverage::an_exclusion_takes_a_gap_out_and_a_report_without_gaps_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::coverage::crate_selects_a_package_by_either_name_and_refuses_one_that_is_not_there` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::coverage::gaps_in_either_direction_are_printed_and_exit_one` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::coverage::this_repository_has_no_code_or_test_gaps_in_its_scoped_packages` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::a_command_misused_is_a_usage_error_before_any_work` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::a_finding_of_the_default_rules_stops_the_write_with_exit_one` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::a_second_write_is_unchanged_and_the_bytes_are_the_same` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::debt_that_grew_is_exit_one_with_its_findings_and_nothing_is_written` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::orphan_lines_over_the_ceiling_are_recorded_per_item` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::paid_debt_fails_the_check_with_debt_resolved_until_the_write_lowers_the_record` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::the_default_policy_can_record_a_current_structural_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::the_spdx_lines_are_given_together_and_kept_when_the_record_is_lowered` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::the_write_creates_the_record_exits_zero_and_the_repository_then_passes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::the_written_record_in_rdf_is_one_object_with_its_members_in_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::stored_command_text_is_never_executed_by_checks_or_dependency_views` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::history::a_bound_file_that_is_not_committed_is_exit_two_naming_it_until_it_is` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::history::a_capability_lists_its_document_and_symbol_commits_newest_first_and_a_removed_binding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::history::a_shallow_clone_is_exit_two_and_its_full_clone_answers` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::history::quiet_prints_nothing_because_a_history_is_not_a_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::history::rdf_preserves_full_history_commits_sources_paths_and_removed_bindings` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::history::since_limits_the_commits_and_a_change_or_an_unknown_id_is_answered_or_refused` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::history::stored_reviews_report_item_changes_after_the_reviewed_commit_is_gone` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::a_change_that_reaches_dependants_and_linking_sections_lists_them_sorted` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::a_record_that_does_not_load_is_exit_two_with_its_findings` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::a_slug_with_no_record_is_exit_two_and_names_the_open_changes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::impact_finds_the_root_from_below_and_by_root` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::impact_prints_what_the_change_reaches_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::quiet_prints_nothing_and_the_exit_stays_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::rdf_preserves_impact_bindings_dependencies_links_and_list_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::a_ceiling_row_and_its_class_decide_the_exit_of_the_default_check_on_the_binary` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::a_package_that_is_not_one_exits_two_and_says_why` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::a_repository_whose_every_counted_line_is_covered_prints_the_report_and_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::an_exception_is_the_excepted_table_and_a_lines_excepted_object_and_the_report_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::an_exception_that_matches_nothing_fails_check_under_default_and_its_matched_neighbour_passes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::an_orphan_line_is_a_finding_and_exits_one_in_text_and_rdf_alike_and_reverse_prints_the_same` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::a_capability_given_twice_is_listed_once` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::a_capability_that_is_missing_or_a_bad_option_is_a_usage_error_before_anything_is_read` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::a_second_run_for_the_same_slug_is_exit_two_and_leaves_the_record_as_it_is` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::affects_is_repeatable_and_the_record_holds_the_entries_given_in_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::every_refusal_of_the_library_is_exit_two_with_the_tree_untouched` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::new_prints_the_library_text_writes_the_record_and_nothing_else_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::new_works_from_a_subdirectory_and_by_root` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::quiet_writes_the_record_and_prints_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::rdf_preserves_created_change_fields_and_ordered_guidance` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::spdx_text_that_is_not_one_line_is_exit_two_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_note_about_the_generated_blocks_is_there_when_the_record_makes_them_stale` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_printed_template_in_a_capability_of_each_state_gives_a_clean_check_and_one_unresolved_proposal` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_spdx_lines_are_the_two_comment_lines_of_the_records_of_this_repository` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_template_left_in_a_state_that_refuses_it_is_the_finding_the_guidance_names` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::a_capability_the_change_lists_that_has_no_document_is_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::a_change_record_that_did_not_load_is_exit_two_and_says_so` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::a_slug_that_is_not_a_slug_is_a_usage_error` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::an_unknown_slug_is_exit_two_and_names_the_open_changes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::quiet_prints_nothing_because_a_preview_is_not_a_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::the_output_is_the_text_the_library_previews_under_a_header_for_each_document` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::the_preview_is_the_same_by_root_from_another_directory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::the_proposed_text_is_printed_and_the_markers_are_not` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_missing_git_binary_does_not_prevent_native_reads` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_slug_that_is_not_a_slug_is_refused_before_any_process_starts` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_before_any_process_starts` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_by_every_command_before_any_process_starts` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_usage_error_of_the_change_commands_starts_no_process` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_usage_error_starts_no_process` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::process::cargo_dispatches_workplan_to_the_binary_on_path_and_passes_the_exit_code_back` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::discovery_does_not_run_the_build_rustc_program_of_the_workspace` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::discovery_starts_no_cargo_from_the_cargo_variable` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::discovery_starts_no_cargo_on_path_when_the_variable_is_unset` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::discovery_succeeds_when_cargo_cannot_be_started` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::help_and_version_output_failures_exit_two_beside_successful_output` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::root_starts_no_cargo_at_all` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::rdf::check_preserves_every_finding_location_in_both_equivalent_formats` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::rdf::failures_have_no_machine_output_and_json_is_rejected_before_root_loading` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::rdf::preview_preserves_reconciled_text_and_sync_reports_only_actual_rewrites` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::rdf::quiet_keeps_findings_and_an_empty_result_is_still_a_canonical_graph` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::rdf::standalone_publication_preserves_host_files_and_refuses_changed_owned_bytes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::rdf::static_fragments_preserve_aliases_and_all_bindings_without_compiler_authority` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::rdf::status_preserves_ordered_capabilities_binding_gaps_and_open_changes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::bound_example_bodies_are_inert_and_reconciliation_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::everything_that_stops_it_looking_is_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::nothing_is_written_and_no_cargo_is_started_for_a_fixture_without_bound_examples` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::quiet_keeps_the_obligations_and_hides_nothing_that_is_a_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::rdf_preserves_each_obligation_and_optional_location_in_library_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::a_directory_outside_any_workspace_is_exit_two_and_names_the_command` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::a_relative_root_is_taken_from_the_current_directory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::every_command_finds_the_root_from_a_subdirectory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::root_overrides_the_discovery_and_the_current_directory_is_not_read` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::sync_from_a_subdirectory_writes_at_the_root_and_nowhere_else` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::the_root_is_found_from_the_root_and_from_any_directory_below_it` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::a_directory_that_is_not_a_repository_is_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::a_state_with_no_capability_is_a_row_with_a_count_of_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::an_implementing_capability_with_every_binding_resolved_has_no_gap_beyond_its_empty_lists` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::quiet_leaves_standard_output_empty_because_the_roadmap_has_no_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::the_roadmap_groups_one_capability_in_each_state_with_the_gaps_and_the_change` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::the_roadmap_is_the_same_by_root_from_another_directory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_document_that_does_not_load_is_exit_two_and_nothing_is_written` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_missing_index_with_the_text_is_created_with_that_header` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_missing_index_without_the_text_is_exit_two_naming_the_field_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_refused_file_is_exit_two_and_nothing_is_written` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_second_sync_writes_nothing_and_prints_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_value_that_cannot_be_written_into_the_header_is_refused_when_the_index_is_created` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::each_field_that_is_not_given_is_named` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::only_the_generated_blocks_are_rewritten` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::quiet_hides_the_files_rewritten_and_keeps_the_exit_code` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::the_index_is_created_by_root_from_another_directory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::the_text_is_not_read_when_the_index_exists` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::trusted_docs::documentation_trust_is_explicit_and_configuration_cannot_select_a_test_driver` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::trusted_docs_is_refused_before_root_discovery_and_general_execution_is_absent` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::verify::verify_prints_static_results_without_git_or_consumer_execution` | test | [Contract](#contract) |
| `raf_source::test_crate::root::explicit_workspace_requires_a_safe_path_and_agreed_membership` | test | [Verification](#verification) |
| `raf_source::test_crate::root::malformed_manifests_and_missing_directories_fail_hard` | test | [Verification](#verification) |
| `raf_source::test_crate::root::manifest_size_bound_is_enforced_beside_the_exact_limit` | test | [Verification](#verification) |
| `raf_source::test_crate::root::member_and_virtual_workspace_directories_resolve_without_sources` | test | [Verification](#verification) |
| `raf_source::test_crate::root::standalone_excluded_and_nested_packages_have_their_own_roots` | test | [Verification](#verification) |
| `raf_source::test_crate::root::transitive_path_dependencies_are_members` | test | [Verification](#verification) |
| `raf_workplan::Workplan::impact` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::new_change` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::preview` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::sync` | doctest | [Verification](#verification) |
| `raf_workplan::escape::shown_lines` | doctest | [Verification](#verification) |
| `raf_workplan::escape::tests::each_control_and_bidirectional_formatting_character_is_escaped` | test | [Verification](#verification) |
| `raf_workplan::escape::tests::lines_keep_their_line_feeds_and_tabs_and_lose_every_other_control` | test | [Verification](#verification) |
| `raf_workplan::example_crate::docs_sync` | example target | [Usage](#usage) |
| `raf_workplan::test_crate::root::a_dangling_manifest_is_a_finding_instead_of_an_ancestor_fallback` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::a_directory_outside_any_workspace_is_an_actionable_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::a_nonexistent_directory_is_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::memory_tree_membership_discovery_needs_no_disk_adapter` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::the_root_is_found_from_the_root_a_member_and_a_subdirectory` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::tree_directory_aliases_fail_instead_of_selecting_lexical_ancestors` | test | [Verification](#verification) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [workplan-complete-gate](./workplan-complete-gate.md) |
| Depends on | [workplan-evidence-claims](./workplan-evidence-claims.md) |
| Depends on | [workplan-generated-docs](./workplan-generated-docs.md) |
| Depended on by | [workplan-cargo-documentation](./workplan-cargo-documentation.md) |
| Depended on by | [workplan-rustdoc](./workplan-rustdoc.md) |
<!-- raf:generated:end -->
