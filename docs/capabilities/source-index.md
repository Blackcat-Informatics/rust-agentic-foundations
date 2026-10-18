---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:source-index
title: "Source index"
description: "The raf-source crate: a Tree abstraction, package discovery, mod-tree following with #[path], and a symbol and use-alias index."
state: delivered
bindings:
  code:
    - raf_source::Tree
    - raf_source::DiskTree
    - raf_source::MemoryTree
    - raf_source::RelPath
    - raf_source::Workspace
    - raf_source::parse_source
    - raf_source::Resolution
    - raf_source::LoadOptions
    - raf_source::Item
    - raf_source::Target
    - raf_source::FileId
    - raf_source::Usage
    - raf_source::Budget
    - raf_source::Opened
    - raf_source::PARSE_BYTES_PER_TOKEN
    - raf_source::RECURSION_UNIT_BYTES
    - raf_source::SelfType
    - raf_source::UnresolvedImpl
    - raf_source::Workspace::resolve_in_package
    - raf_source::Workspace::unresolved_impls
    - raf_source::Workspace::exported_items
    - raf_source::Workspace::exported_items_with
    - raf_source::Exports
    - raf_source::ExportedItem
    - raf_source::Reexport
    - raf_source::ExportOptions
    - raf_source::ExportError
  tests:
    - raf_source::test_crate::relpath::a_parent_segment_above_the_root_is_refused_beside_the_one_that_stays_inside
    - raf_source::test_crate::relpath::an_absolute_input_is_refused_beside_its_relative_form
    - raf_source::test_crate::memory_tree::duplicate_keys_are_refused_literally_and_after_normalisation_beside_distinct_keys
    - raf_source::test_crate::memory_tree::keys_that_leave_the_root_are_refused_with_outside_root_beside_a_valid_key
    - raf_source::test_crate::disk_tree_containment::a_symlink_to_a_file_outside_is_refused_beside_an_in_root_symlink_that_is_accepted
    - raf_source::test_crate::disk_tree_containment::a_parent_segment_after_an_outside_directory_symlink_is_resolved_on_disk_and_refused
    - raf_source::test_crate::disk_tree_containment::a_directory_symlink_loop_ends_with_an_error_and_is_listed_but_never_entered
    - raf_source::test_crate::disk_tree_containment::a_fifo_named_as_a_file_is_refused_without_blocking_beside_a_regular_file
    - raf_source::test_crate::workspace_containment::a_symlinked_module_file_to_outside_is_refused_naming_the_module_beside_an_in_root_link
    - raf_source::test_crate::workspace_containment::a_manifest_target_path_that_leaves_the_root_is_refused_naming_the_key
    - raf_source::test_crate::workspace_containment::discovery_classifies_a_symlink_by_what_it_resolves_to
    - raf_source::test_crate::discovery_targets::each_target_kind_with_default_paths_has_its_name_kind_crate_path_and_shipping_flag
    - raf_source::test_crate::discovery_targets::two_packages_with_one_name_are_an_error_naming_both_manifests
    - raf_source::test_crate::discovery_targets::an_empty_root_list_is_a_no_roots_error_beside_dot_which_loads
    - raf_source::test_crate::discovery_targets::a_manifest_over_the_cap_is_refused_naming_it_and_one_of_exactly_the_cap_loads
    - raf_source::test_crate::discovery_targets::harness_required_features_test_and_doctest_are_read_from_each_target_table
    - raf_source::test_crate::discovery_targets::a_test_setting_of_the_wrong_type_is_a_manifest_error_naming_its_key
    - raf_source::test_crate::module_tree::a_path_attribute_in_a_mod_rs_file_is_relative_to_that_files_directory
    - raf_source::test_crate::module_tree::every_cfg_attr_path_alternative_is_followed_carrying_its_predicate
    - raf_source::test_crate::module_tree::a_module_cycle_is_an_error_naming_both_files
    - raf_source::test_crate::module_tree::a_path_that_leaves_the_tree_is_refused_naming_module_and_attribute_without_reading_it
    - raf_source::test_crate::module_tree::the_cfg_test_matrix_tags_items_and_leaves_their_neighbours_alone
    - raf_source::test_crate::module_tree::a_chain_of_files_that_each_declare_the_next_twice_is_refused_by_the_module_path_budget_opening_each_file_once_beside_a_small_diamond
    - raf_source::test_crate::module_tree::a_child_file_that_sorts_before_its_mod_rs_parent_is_placed_under_its_own_path
    - raf_source::test_crate::module_tree::an_item_is_found_only_under_its_own_module_path
    - raf_source::test_crate::symbol_index::every_kind_is_indexed_with_its_kind_line_file_and_visibility
    - raf_source::test_crate::symbol_index::cfg_twins_are_one_symbol_with_two_sorted_sites
    - raf_source::test_crate::symbol_index::test_functions_carry_the_package_target_and_crate_relative_name
    - raf_source::test_crate::symbol_index::two_loads_are_equal_and_items_and_targets_come_in_strictly_increasing_order
    - raf_source::test_crate::use_resolution::a_three_hop_chain_resolves_and_lists_each_hop
    - raf_source::test_crate::use_resolution::a_cycle_of_imports_is_cyclic_beside_a_chain_that_ends
    - raf_source::test_crate::use_resolution::missing_names_dependency_items_and_macro_generated_items_are_not_found
    - raf_source::test_crate::use_resolution::resolution_does_not_depend_on_the_order_of_the_declarations
    - raf_source::test_crate::use_resolution::a_long_chain_of_re_exports_resolves_in_either_order_without_a_cap
    - raf_source::test_crate::use_resolution::a_single_segment_import_of_a_dependency_is_an_external_alias_and_never_its_own_cycle
    - raf_source::test_crate::use_resolution::a_single_segment_import_of_a_workspace_library_binds_its_crate_root
    - raf_source::test_crate::use_resolution::a_use_path_of_edition_2015_starts_at_the_crate_root
    - raf_source::test_crate::use_resolution::a_use_head_from_a_glob_that_is_also_an_extern_prelude_crate_is_ambiguous
    - raf_source::test_crate::use_resolution::a_glob_against_a_workspace_library_is_ambiguous_in_every_declaration_order
    - raf_source::test_crate::use_resolution::a_glob_that_brings_in_an_import_of_another_crate_collides_with_the_crate
    - raf_source::test_crate::use_resolution::a_no_std_crate_does_not_name_std_without_an_extern_crate_item
    - raf_source::test_crate::use_resolution::a_glob_whose_own_path_starts_at_a_crate_is_ambiguous_with_a_crate_in_every_order
    - raf_source::test_crate::use_resolution::an_explicit_import_shadows_a_workspace_library_in_every_declaration_order
    - raf_source::test_crate::use_resolution::a_crate_name_an_explicit_import_binds_is_never_read_as_the_crate_in_every_order
    - raf_source::test_crate::use_resolution::an_import_binds_what_a_name_it_read_gains_later_in_every_order
    - raf_source::test_crate::use_resolution::a_cycle_of_explicit_imports_is_cyclic_whatever_its_globs_hold_in_every_order
    - raf_source::test_crate::use_resolution::a_cycle_of_imports_through_a_crate_name_and_a_glob_is_cyclic_in_every_order
    - raf_source::test_crate::use_resolution::an_explicit_import_shadows_the_glob_whether_or_not_the_index_sees_what_it_binds
    - raf_source::test_crate::use_resolution::an_import_that_binds_its_name_in_one_name_space_leaves_the_others_to_the_globs
    - raf_source::test_crate::use_resolution::an_import_keeps_the_globs_out_only_in_the_name_spaces_it_binds_in_every_order
    - raf_source::test_crate::use_resolution::an_import_of_a_value_named_like_a_crate_leaves_the_crate_and_the_globs_in_every_order
    - raf_source::test_crate::use_resolution::a_cycle_of_imports_with_one_glob_on_it_resolves_through_the_glob_in_every_order
    - raf_source::test_crate::use_resolution::an_import_that_bound_a_name_before_the_crates_opened_binds_the_crate_later_in_every_order
    - raf_source::test_crate::use_resolution::imports_that_wait_on_one_another_through_globs_are_decided_as_rustc_does_in_every_order
    - raf_source::test_crate::use_resolution::imports_whose_heads_are_names_their_globs_bring_in_are_decided_alike_in_every_order
    - raf_source::test_crate::use_resolution::a_name_space_binds_nothing_only_once_the_globs_that_may_bring_it_in_are_settled_in_every_order
    - raf_source::test_crate::use_resolution::the_rebuilt_globs_read_each_glob_path_in_the_settled_tables_in_every_order
    - raf_source::test_crate::use_resolution::globs_that_glob_one_another_beside_names_bound_twice_settle_in_every_order
    - raf_source::test_crate::use_resolution::a_glob_whose_path_turns_out_ambiguous_brings_in_nothing_in_every_order
    - raf_source::test_crate::use_resolution::a_glob_brings_in_what_its_target_binds_later_whatever_its_target_holds_under_its_first_segment
    - raf_source::test_crate::use_resolution::an_import_whose_glob_reads_it_starts_at_the_crate_once_no_glob_may_bring_the_name_in_every_order
    - raf_source::test_crate::use_resolution::an_item_a_glob_reaches_by_a_public_and_a_private_route_is_visible_in_every_order
    - raf_source::test_crate::use_resolution::a_cycle_through_a_glob_and_a_renamed_import_is_decided_in_each_name_space_in_every_order
    - raf_source::test_crate::use_resolution::an_import_of_its_own_module_reads_what_the_globs_bring_in_in_every_order
    - raf_source::test_crate::use_resolution::a_path_through_an_import_a_cycle_leaves_undecided_in_another_name_space_is_cyclic
    - raf_source::test_crate::use_resolution::a_cycle_whose_import_reads_its_own_name_through_another_glob_is_cyclic_in_every_order
    - raf_source::test_crate::use_resolution::a_name_space_binds_nothing_only_once_the_globs_its_globs_reach_are_settled
    - raf_source::test_crate::use_resolution::a_cycle_through_a_renamed_import_that_has_bound_nothing_is_cyclic_in_every_order
    - raf_source::test_crate::use_resolution::a_cycle_resolves_through_its_glob_when_a_place_an_import_keeps_also_waits_on_a_glob
    - raf_source::test_crate::use_resolution::an_import_through_an_import_that_keeps_its_name_keeps_its_own_from_the_globs_in_every_order
    - raf_source::test_crate::use_resolution::a_cycle_through_a_glob_the_importer_cannot_access_is_no_cycle_in_every_order
    - raf_source::test_crate::use_resolution::an_import_a_cycle_leaves_to_the_globs_binds_what_its_readers_could_access_in_every_order
    - raf_source::test_crate::use_resolution::a_name_an_import_the_reader_cannot_access_binds_is_hidden_from_it_in_every_order
    - raf_source::test_crate::use_resolution::an_import_that_reads_a_name_only_through_a_glob_it_cannot_access_finds_nothing_in_every_order
    - raf_source::test_crate::use_resolution::a_glob_the_importer_cannot_access_holds_nothing_back_in_every_order
    - raf_source::test_crate::use_resolution::cycles_through_private_globs_resolve_as_each_reader_can_access_them_in_every_order
    - raf_source::test_crate::use_resolution::a_cycle_left_to_a_glob_reads_all_that_the_glob_brings_in_once_its_round_is_decided
    - raf_source::test_crate::use_resolution::imports_that_read_a_name_through_a_glob_agree_with_the_glob_on_an_ambiguous_re_export
    - raf_source::test_crate::trait_keys::an_explicit_import_of_a_dependency_trait_shadows_a_glob_of_a_local_one
    - raf_source::test_crate::trait_keys::a_trait_through_a_glob_the_importer_cannot_access_is_keyed_at_its_definition_in_every_order
    - raf_source::test_crate::trait_keys::each_prelude_trait_loads_in_the_edition_that_has_it_and_is_refused_in_the_one_before
    - raf_source::test_crate::trait_keys::inheriting_requires_membership_by_the_members_and_exclude_lists
    - raf_source::test_crate::trait_keys::a_trait_through_a_single_segment_import_of_a_crate_is_keyed_at_the_crate
    - raf_source::test_crate::trait_keys::a_glob_through_a_re_exported_crate_is_keyed_under_the_crate
    - raf_source::test_crate::trait_keys::a_use_path_of_edition_2015_is_followed_from_the_crate_root
    - raf_source::test_crate::trait_keys::a_trait_through_an_ambiguous_use_head_keys_every_reading
    - raf_source::test_crate::trait_keys::twenty_thousand_chained_local_aliases_load_on_a_two_mib_thread
    - raf_source::test_crate::trait_keys::a_trait_found_through_twenty_thousand_chained_globs_is_keyed_on_a_two_mib_thread
    - raf_source::test_crate::trait_keys::an_import_met_again_through_a_glob_adds_nothing_and_a_pure_cycle_is_refused
    - raf_source::test_crate::trait_keys::a_trait_through_a_head_a_glob_brings_in_from_another_crate_keys_both
    - raf_source::test_crate::trait_keys::a_use_path_of_edition_2015_names_a_dependency_only_through_extern_crate
    - raf_source::test_crate::trait_keys::a_trait_name_that_two_globs_bind_keys_each_trait
    - raf_source::test_crate::trait_keys::a_trait_that_several_unknown_globs_could_give_is_refused_naming_one_glob_in_every_order
    - raf_source::test_crate::trait_keys::the_completion_audit_shape_of_a_crate_alias_and_twenty_thousand_imports_loads_on_a_two_mib_thread
    - raf_source::test_crate::target_roots::every_target_kind_resolves_at_its_crate_path_as_a_module_item_of_its_root_file
    - raf_source::test_crate::target_roots::a_first_party_module_at_a_binary_crate_path_is_ambiguous_beside_a_binary_that_is_not
    - raf_source::test_crate::prescan_budgets::delimiter_nesting_is_refused_at_one_hundred_twenty_nine_beside_one_hundred_twenty_eight
    - raf_source::test_crate::prescan_budgets::five_hundred_nested_generics_are_refused_beside_sixty_four
    - raf_source::test_crate::prescan_budgets::twenty_thousand_else_if_links_are_refused_beside_two_hundred_fifty_six
    - raf_source::test_crate::prescan_budgets::a_syntax_error_reports_its_own_line_and_column_beside_a_parsing_neighbour
    - raf_source::test_crate::prescan_budgets::the_recursion_limit_is_half_the_parse_stack_over_the_measured_cost_of_one_unit
    - raf_source::test_crate::prescan_budgets::every_operator_counts_one_for_each_of_its_characters_at_the_limit_and_one_over
    - raf_source::test_crate::prescan_budgets::every_counted_keyword_counts_one_at_the_limit_and_one_over
    - raf_source::test_crate::prescan_budgets::a_label_a_use_path_separator_and_each_delimiter_count_at_the_limit_and_one_over
    - raf_source::test_crate::prescan_budgets::a_level_returns_to_zero_where_its_constructs_end_beside_the_same_count_without_the_end
    - raf_source::test_crate::prescan_budgets::a_construct_that_continues_past_a_comma_or_a_closing_brace_keeps_its_count
    - raf_source::test_crate::prescan_budgets::a_macro_call_counts_only_its_groups_and_a_match_pattern_only_its_alternatives
    - raf_source::test_crate::prescan_budgets::chains_that_overflowed_the_parse_stack_are_refused_by_the_recursion_budget
    - raf_source::test_crate::prescan_budgets::the_eighty_kilobyte_box_pattern_is_refused_by_the_pre_scan_before_the_parser_runs
    - raf_source::test_crate::prescan_budgets::the_recursion_refusal_names_the_line_where_the_count_went_over
    - raf_source::test_crate::walk_stack::a_use_tree_at_the_recursion_limit_loads_resolves_and_drops_on_a_five_hundred_twelve_kib_thread
    - raf_source::test_crate::workspace_seeds::every_load_workspace_seed_loads_with_items_through_the_fuzz_input_code
    - raf_source::test_crate::workspace_seeds::the_self_check_the_fuzz_target_runs_first_passes_beside_an_input_that_is_not_a_tree
    - raf_source::test_crate::overflow_harness::every_committed_overflow_shape_is_refused_by_a_budget_and_none_aborts
    - raf_source::test_crate::overflow_harness::a_deliberate_overflow_aborts_and_the_runtime_names_the_parse_thread
    - raf_source::test_crate::overflow_harness::a_return_chain_that_needs_twice_the_stack_is_refused_beside_one_at_a_safe_depth
    - raf_source::test_crate::overflow_harness::every_shape_family_loads_at_its_largest_admitted_size_and_one_size_more_is_refused
    - raf_source::test_crate::overflow_harness::the_parse_thread_is_named_after_the_file_and_has_the_stated_stack
    - raf_source::test_crate::overflow_harness::the_classifier_accepts_only_a_signal_a_stack_overflow_status_or_the_runtime_message_as_an_abort
    - raf_source::test_crate::corpus_replay::every_corpus_file_is_refused_or_indexed_with_its_recorded_symbols_and_none_aborts
    - raf_source::test_crate::read_cap::disk_tree_enforces_the_cap_at_eight_mib
    - raf_source::test_crate::read_cap::the_default_cap_is_eight_mib
    - raf_source::test_crate::load_cap::a_library_root_of_exactly_the_cap_loads_and_one_byte_more_is_refused_naming_it
    - raf_source::test_crate::load_cap::a_module_file_of_exactly_the_cap_loads_and_one_byte_more_is_refused_naming_it
    - raf_source::test_crate::load_cap::a_lowered_cap_applies_to_a_child_file_and_spares_a_root_within_it
    - raf_source::test_crate::load_cap::a_cap_larger_than_the_default_admits_a_file_the_default_refuses
    - raf_source::test_crate::spawn_limit::a_parse_thread_the_system_refuses_a_stack_is_a_spawn_error_naming_the_file
    - raf_source::test_crate::source_text::crlf_line_endings_give_the_same_lines_and_spans_as_lf
    - raf_source::test_crate::source_text::a_byte_order_mark_leaves_the_lines_of_the_text_where_they_are
    - raf_source::test_crate::source_text::a_byte_order_mark_on_the_first_line_starts_the_span_of_its_item_after_the_mark
    - raf_source::test_crate::source_text::a_one_line_item_has_the_half_open_lines_n_to_n_plus_one
    - raf_source::test_crate::source_text::a_syntax_error_in_a_crlf_file_or_after_a_byte_order_mark_names_its_own_line
    - raf_source::test_crate::source_text::a_budget_refusal_in_a_crlf_file_names_the_same_line_as_in_an_lf_file
    - raf_source::test_crate::source_text::a_module_file_that_is_not_utf8_is_refused_on_disk_naming_the_file
    - raf_source::test_crate::index_disk::disk_and_memory_trees_give_equal_workspaces_and_equal_resolutions
    - raf_source::test_crate::tree_error::every_variant_names_its_path_in_display_and_in_path
    - raf_source::test_crate::fs_confinement::only_the_disk_backend_names_the_file_system_module
    - raf_source::test_crate::this_repository::this_repository_loads_from_its_root_with_the_crates_directory_as_the_root
    - raf_source::test_crate::this_repository::an_item_of_a_module_file_that_sorts_before_its_mod_rs_is_found_under_its_own_module
    - raf_source::test_crate::this_repository::a_dependency_re_exported_by_its_crate_name_is_not_found_and_not_a_cycle
    - raf_source::tree::disk::tests::linux_verifier_refuses_a_handle_link_that_leads_outside_and_accepts_one_inside
    - raf_source::tree::disk::tests::the_handle_path_verifier_refuses_a_path_outside_the_root_beside_one_inside
    - raf_source::tree::disk::tests::swap::a_directory_swapped_for_an_outside_symlink_after_the_check_is_refused
    - raf_source::tree::disk::tests::swap::a_directory_swapped_for_an_outside_symlink_before_a_listing_is_refused
    - raf_source::tree::disk::tests::a_missing_descriptor_link_is_an_io_failure_and_never_absence
    - raf_source::tree::disk::tests::swap::a_directory_swapped_after_verification_before_the_entries_are_read_is_refused
    - raf_source::tree::disk::tests::swap::a_file_whose_directory_is_moved_out_after_verification_is_refused_after_the_read
    - raf_source::tree::disk::tests::swap::an_entry_whose_directory_is_swapped_for_an_outside_symlink_is_refused
    - raf_source::tree::disk::tests::swap::an_outside_fifo_reached_by_a_swap_before_the_open_is_never_opened
    - raf_source::tree::disk::tests::strategy::every_open_that_resolves_a_name_is_a_reference_or_does_not_block
    - raf_source::tree::disk::tests::swap::a_swap_before_the_open_undone_before_the_check_is_refused
    - raf_source::tree::disk::tests::swap::a_swap_flipped_at_every_hook_point_is_refused
    - raf_source::tree::disk::tests::strategy::every_strategy_follows_an_absolute_in_root_link_spelled_through_a_linked_parent
    - raf_source::tree::disk::tests::strategy::links_into_the_outside_are_answered_alike_only_by_the_walk
    - raf_source::tree::disk::tests::strategy::a_link_that_climbs_back_into_the_root_is_read_only_by_canonicalisation
    - raf_source::tree::disk::tests::strategy::a_symlink_that_becomes_a_directory_during_a_step_is_stepped_again
    - raf_source::tree::disk::tests::strategy::a_path_three_hundred_deep_is_walked_under_a_limit_of_sixty_four_descriptors
    - raf_source::tree::disk::tests::linux_verifier_reads_the_real_descriptor_of_an_open_file
    - raf_source::tree::disk::tests::strategy::every_strategy_refuses_an_outside_symlink_and_reads_in_root_links
    - raf_source::tree::disk::tests::strategy::every_strategy_reports_entry_kinds_and_refuses_an_outside_directory
    - raf_source::tree::disk::tests::reserved_device_names_and_trailing_dots_or_spaces_are_ambiguous_on_windows_beside_plain_names
    - raf_source::test_crate::workspace_containment::a_tree_with_windows_device_names_loads_off_windows
    - raf_source::error::tests::a_spawn_error_displays_the_file_and_carries_its_cause
    - raf_source::error::tests::control_characters_in_a_path_are_escaped
    - raf_source::error::tests::every_user_derived_field_of_every_display_is_escaped
    - raf_source::test_crate::error_text::a_path_attribute_with_terminal_escapes_is_shown_escaped
    - raf_source::test_crate::error_text::a_manifest_with_control_characters_in_a_value_is_shown_escaped
    - raf_source::test_crate::error_text::a_resolved_path_or_package_with_control_characters_is_shown_escaped
    - raf_source::resolve::tests::every_user_derived_field_of_every_resolve_error_is_escaped
    - raf_source::test_crate::resolve_paths::a_path_keyword_in_any_position_is_invalid_beside_the_canonical_path
    - raf_source::test_crate::resolve_paths::a_glob_of_a_missing_local_module_loads_and_fails_each_lookup_that_reads_it
    - raf_source::test_crate::resolve_paths::a_glob_the_index_cannot_look_into_leaves_a_lookup_not_found
    - raf_source::resolve::tests::a_path_of_fifty_thousand_segments_reads_each_segment_a_bounded_number_of_times
    - raf_source::test_crate::resolve_paths::globs_of_dependencies_in_one_module_leave_a_missing_name_not_found
    - raf_source::test_crate::resolve_paths::a_cycle_of_globs_stays_cyclic_beside_globs_that_resolve
    - raf_source::test_crate::resolve_paths::a_cycle_is_reported_before_a_glob_that_names_nothing
    - raf_source::test_crate::resolve_paths::a_name_found_through_a_glob_beside_a_glob_that_names_nothing_fails
    - raf_source::test_crate::manifest_caps::exclude_at_the_cap_loads_and_one_over_is_refused
    - raf_source::test_crate::manifest_caps::members_at_the_cap_loads_and_one_over_is_refused
    - raf_source::test_crate::manifest_caps::the_duplicate_pattern_probe_is_refused_before_any_pattern_is_expanded
    - raf_source::test_crate::manifest_caps::duplicate_patterns_within_the_caps_load_with_the_tree_calls_of_one_of_each
    - raf_source::test_crate::manifest_caps::distinct_patterns_at_both_caps_load_with_no_tree_calls_for_the_excludes
    - raf_source::test_crate::manifest_caps::exclude_still_removes_literal_wildcard_and_nested_members
    - raf_source::glob::tests::a_segment_is_matched_in_at_most_pattern_times_name_steps
    - raf_source::members::tests::distinct_patterns_at_both_caps_are_tested_a_bounded_number_of_times
    - raf_source::members::tests::duplicate_patterns_are_compiled_and_tested_once
    - raf_source::test_crate::trait_keys::a_star_heavy_member_pattern_that_matches_no_directory_leaves_the_member_alone
    - raf_source::item::tests::an_item_record_is_at_most_two_hundred_fifty_six_bytes
    - raf_source::facts::tests::a_column_is_turned_into_its_byte_offset_on_short_long_and_wide_lines
    - raf_source::facts::tests::a_long_line_of_wide_characters_is_indexed_once_for_all_its_positions
    - raf_source::facts::tests::a_large_attribute_on_a_verbatim_use_group_is_read_once_for_all_its_members
    - raf_source::settle::tests::a_binding_list_stays_sorted_keeps_the_better_route_and_takes_only_its_room
    - raf_source::settle::tests::components_come_out_once_each_after_every_component_they_lead_to
    - raf_source::settle::tests::each_settle_after_the_first_is_charged_what_it_builds_again
    - raf_source::settle::tests::the_imports_left_are_decided_in_rounds_that_do_not_grow_with_them
    - raf_source::settle::tests::a_round_that_decides_nothing_more_or_a_settle_too_many_is_reported_in_every_build
    - raf_source::settle::tests::the_imports_of_a_name_are_found_among_names_that_share_its_first_eight_bytes
    - raf_source::test_crate::load_budgets::a_file_at_the_token_limit_parses_and_one_more_token_is_refused_naming_its_line
    - raf_source::test_crate::load_budgets::each_costly_shape_at_the_token_limit_loads_and_one_more_token_is_refused
    - raf_source::test_crate::load_budgets::a_doc_comment_counts_as_the_tokens_of_its_attribute_and_other_comments_count_nothing
    - raf_source::test_crate::load_budgets::each_large_fact_payload_is_charged_and_refused_at_its_line_once_over_the_fact_budget
    - raf_source::test_crate::load_budgets::the_fact_budget_counts_over_every_file_of_the_load
    - raf_source::test_crate::load_budgets::index_records_at_the_item_budget_load_and_one_byte_less_is_refused
    - raf_source::test_crate::load_budgets::a_long_module_name_is_charged_in_every_item_path_and_refused_over_the_item_budget
    - raf_source::test_crate::load_budgets::a_file_reached_by_many_module_paths_is_charged_once_per_path_and_refused_over_the_item_budget
    - raf_source::test_crate::load_budgets::the_bytes_read_at_the_total_budget_load_and_one_byte_less_is_refused_naming_the_file
    - raf_source::test_crate::load_budgets::the_default_budgets_are_the_stated_limits
    - raf_source::test_crate::load_budgets::usage_reports_what_a_load_spent
    - raf_source::test_crate::load_budgets::a_long_alias_path_in_a_file_reached_many_times_is_held_and_charged_once
    - raf_source::test_crate::load_budgets::a_long_extern_crate_name_in_a_file_reached_many_times_is_held_and_charged_once
    - raf_source::test_crate::load_budgets::a_chain_of_files_that_each_declare_the_next_twice_is_charged_what_it_keeps
    - raf_source::test_crate::load_budgets::nested_inline_modules_whose_scopes_hold_one_name_are_charged_what_they_keep
    - raf_source::test_crate::load_budgets::glob_tables_at_the_glob_budget_load_and_one_byte_less_is_refused
    - raf_source::test_crate::load_budgets::a_module_that_many_modules_glob_is_kept_once_for_all_of_them
    - raf_source::test_crate::load_budgets::a_chain_of_globs_is_charged_what_each_link_brings_in
    - raf_source::test_crate::load_budgets::modules_that_glob_one_another_are_refused_over_the_glob_budget_naming_a_glob
    - raf_source::test_crate::load_budgets::glob_tables_of_a_generated_binding_crate_are_charged_what_they_keep
    - raf_source::test_crate::load_budgets::repeated_globs_of_one_module_are_charged_once_for_what_it_holds
    - raf_source::cost::tests::a_map_of_one_entry_is_covered_by_what_a_map_costs_once
    - raf_source::cost::tests::a_list_is_charged_at_least_its_room
    - raf_source::cost::tests::a_hash_table_is_charged_at_least_its_buckets
    - raf_source::test_crate::memory_tree::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen
    - raf_source::test_crate::disk_tree_containment::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen
    - raf_source::test_crate::walk_stack::a_memory_chain_of_twenty_thousand_directories_is_refused_typed_on_a_two_mib_thread
    - raf_source::test_crate::walk_stack::a_directory_at_the_depth_limit_is_walked_and_one_below_it_is_refused
    - raf_source::test_crate::walk_stack::a_member_pattern_at_the_depth_limit_expands_and_one_deeper_is_refused_typed
    - raf_source::test_crate::walk_stack::one_hundred_twenty_eight_nested_modules_load_on_a_five_hundred_twelve_kib_thread
    - raf_source::test_crate::walk_stack::a_disk_chain_of_directories_is_refused_typed_on_a_two_mib_thread
    - raf_source::test_crate::workspace_containment::symlink_aliases_of_one_directory_parse_it_once
    - raf_source::test_crate::workspace_containment::a_file_reached_through_a_directory_symlink_is_one_file_under_the_spelling_that_sorts_first
    - raf_source::test_crate::workspace_containment::a_module_that_reaches_its_own_file_through_a_directory_symlink_is_a_cycle
    - raf_source::test_crate::module_tree::a_file_reached_under_two_spellings_is_read_once_and_keeps_the_spelling_that_sorts_first
    - raf_source::test_crate::memory_tree::the_identity_of_a_file_is_its_normalised_key_and_refuses_what_read_refuses
    - raf_source::test_crate::disk_tree_containment::the_identity_of_a_file_is_the_device_and_inode_of_its_verified_handle
    - raf_source::test_crate::symbol_index::items_on_one_long_line_of_wide_characters_get_exact_byte_offsets
    - raf_source::test_crate::symbol_index::an_inherent_impl_on_a_type_the_index_cannot_see_is_indexed_under_its_written_self_path
    - raf_source::test_crate::symbol_index::a_bitflags_style_macro_type_with_an_impl_loads_with_its_members_marked_unresolved
    - raf_source::test_crate::symbol_index::an_impl_of_a_local_trait_object_loads_with_its_members_marked_unresolved
    - raf_source::test_crate::symbol_index::a_glob_of_enum_variants_beside_an_impl_loads
    - raf_source::test_crate::symbol_index::a_glob_of_a_local_path_the_index_cannot_see_is_a_candidate_that_cannot_be_confirmed
    - raf_source::test_crate::symbol_index::a_first_party_test_crate_module_and_a_test_target_are_ambiguous_only_where_their_paths_meet
    - raf_source::test_crate::trait_keys::a_macro_call_in_a_trait_body_is_skipped
    - raf_source::test_crate::use_resolution::two_libraries_with_one_crate_name_are_ambiguous_until_the_package_is_named
    - raf_source::test_crate::use_resolution::a_crate_name_in_an_import_is_the_dependency_its_target_declares
    - raf_source::test_crate::use_resolution::a_dev_dependency_is_named_only_by_test_code_even_under_a_normal_dependencys_name
    - raf_source::test_crate::use_resolution::a_name_bound_to_several_modules_gives_a_trait_key_through_each
    - raf_source::test_crate::trait_keys::an_inherited_dependency_the_workspace_cannot_give_is_an_error_naming_the_key
    - raf_source::test_crate::discovery_targets::two_packages_whose_targets_share_crate_paths_load_and_are_told_apart_by_package
    - raf_source::test_crate::target_roots::a_lib_module_named_bin_crate_stays_in_the_library
    - raf_source::test_crate::target_roots::a_build_script_main_beside_a_lib_module_build_script_is_ambiguous
    - raf_source::test_crate::target_roots::an_explicit_path_bin_beside_a_lib_module_bin_crate_is_ambiguous_in_both_file_orders
    - raf_source::test_crate::target_roots::an_over_cap_inferred_lib_names_discovery_not_lib_path
    - raf_source::test_crate::module_tree::a_non_literal_path_attribute_is_reported_as_such_from_the_attribute
    - raf_source::test_crate::use_resolution::a_use_group_with_leading_colon_members_binds_as_the_separate_uses_do
    - raf_source::test_crate::use_resolution::a_leading_colon_member_of_a_use_group_reads_the_crate_root_in_edition_2015
    - raf_source::test_crate::use_resolution::a_crate_root_inside_a_group_under_a_path_is_a_syntax_error
    - raf_source::test_crate::symbol_index::bodiless_functions_and_types_of_attribute_macros_are_indexed_where_they_are_written
    - raf_source::test_crate::symbol_index::a_bodiless_const_or_another_verbatim_item_is_still_a_typed_error_naming_file_and_line
    - raf_source::test_crate::load_budgets::a_verbatim_use_group_and_a_bodiless_function_are_charged_as_their_plain_forms
    - raf_source::test_crate::exported_items::a_pub_function_is_exported_beside_private_and_restricted_ones_which_are_not
    - raf_source::test_crate::exported_items::a_pub_item_of_a_private_module_is_exported_only_through_a_pub_use_and_under_its_rename
    - raf_source::test_crate::exported_items::a_glob_re_export_brings_in_the_pub_items_of_its_module_only
    - raf_source::test_crate::exported_items::a_chain_of_re_exports_through_nested_modules_is_followed_and_a_restricted_hop_stops_it
    - raf_source::test_crate::exported_items::a_restricted_or_private_re_export_is_not_exported_beside_a_pub_one_that_is
    - raf_source::test_crate::exported_items::a_re_export_of_another_crate_is_a_re_export_and_not_an_item
    - raf_source::test_crate::exported_items::a_re_export_of_a_path_the_index_cannot_follow_is_listed_as_a_re_export
    - raf_source::test_crate::exported_items::enum_variants_and_struct_fields_are_not_items_of_the_surface
    - raf_source::test_crate::exported_items::the_pub_methods_of_a_pub_type_are_exported_and_those_of_a_private_type_are_not
    - raf_source::test_crate::exported_items::the_members_of_a_pub_trait_are_exported_and_those_of_a_private_trait_are_not
    - raf_source::test_crate::exported_items::doc_hidden_items_are_left_out_by_default_and_flagged_when_included
    - raf_source::test_crate::exported_items::test_code_is_not_part_of_the_surface_beside_the_shipped_items
    - raf_source::test_crate::exported_items::an_exported_macro_is_exported_and_a_local_or_generated_one_is_not
    - raf_source::test_crate::exported_items::a_binary_lists_its_own_pub_items_and_a_test_target_is_refused
    - raf_source::test_crate::exported_items::a_module_reached_by_two_pub_paths_is_listed_under_each_and_read_under_the_first
    - raf_source::test_crate::exported_items::the_surface_is_the_same_in_every_load_and_in_every_order_of_the_declarations
    - raf_source::test_crate::exported_items::an_ambiguous_a_cyclic_or_an_unresolved_public_name_fails_beside_a_neighbour_that_lists
    - raf_source::test_crate::exported_items::a_surface_over_the_item_budget_left_by_the_load_is_refused_beside_one_within_it
    - raf_source::test_crate::exported_items::twenty_thousand_chained_re_exports_are_listed_on_a_two_mib_thread
    - raf_source::test_crate::exported_items::a_chain_of_twenty_thousand_pub_globs_is_listed_on_a_two_mib_thread
    - raf_source::test_crate::exported_items::the_example_of_the_contract_lists_one_item_and_more_through_a_glob_or_a_pub_module
    - raf_source::test_crate::exported_items::a_re_export_of_an_outside_crate_that_a_pub_glob_carries_is_a_re_export_of_the_reader
    - raf_source::test_crate::exported_items::a_carried_outside_re_export_is_kept_out_by_a_restricted_glob_or_a_name_on_the_way
    - raf_source::test_crate::exported_items::a_named_re_export_of_an_enum_variant_is_a_re_export_and_a_glob_of_an_enum_adds_nothing
    - raf_source::test_crate::exported_items::a_module_of_three_thousand_pub_globs_is_listed_in_time_linear_in_its_names
    - raf_source::test_crate::exported_items::a_module_reached_by_a_hidden_and_a_shown_glob_carries_its_re_export_shown
    - raf_source::test_crate::exported_items::a_module_of_three_thousand_pub_globs_that_each_carry_a_re_export_is_listed_in_linear_time
    - raf_source::test_crate::exported_items::a_carried_re_export_is_kept_out_exactly_where_every_route_passes_a_module_that_binds_its_name
    - raf_source::test_crate::exported_items::an_item_and_a_carried_outside_re_export_of_one_name_are_both_listed_and_not_ambiguous
    - raf_source::test_crate::exported_items::readers_of_three_and_ten_thousand_carried_names_beside_sibling_shadows_list_in_bounded_time_and_bytes
    - raf_source::test_crate::exported_items::searches_for_names_that_several_binders_keep_from_a_long_route_are_refused_by_the_item_budget
    - raf_source::test_crate::exported_items::a_re_export_that_only_a_hidden_glob_carries_past_a_shadow_is_hidden_as_an_item_would_be
    - raf_source::test_crate::exported_items::a_carrier_with_pub_globs_of_its_own_is_not_its_own_binder
    - raf_source::test_crate::exported_items::of_two_carriers_of_one_name_only_the_one_with_a_free_route_is_listed_with_its_own_line
    - raf_source::test_crate::exported_items::a_binder_in_an_irreducible_cycle_that_dominates_nothing_leaves_its_name_carried
    - raf_source::test_crate::exported_items::a_hidden_glob_partway_along_a_route_hides_a_carried_re_export_no_more_than_an_item
    - raf_source::test_crate::exported_items::a_carried_re_export_is_listed_hidden_or_absent_exactly_as_an_item_of_the_same_shape
    - raf_source::test_crate::exported_items::only_a_route_that_carries_a_name_out_decides_whether_it_is_hidden_for_items_and_re_exports
    - raf_source::test_crate::exported_items::in_random_small_crates_a_carried_re_export_is_listed_hidden_or_absent_as_the_item_is
    - raf_source::test_crate::exported_items::a_re_export_two_globs_carry_is_written_as_the_first_glob_that_carries_it
    - raf_source::test_crate::this_repository::the_surface_of_each_library_and_binary_of_this_repository_lists_its_public_items
  examples:
    - raf_source::Workspace::exported_items
depends: []
---

# Source index

## Purpose

`raf-source` answers one question for the other crates: does this Rust path name a
real item, and where is it defined. The census resolves ledger homes with it and the
workplan resolves code and test bindings with it, so one walker and one index serve
both and cannot disagree.

## Contract

**Tree.** All file access goes through the `Tree` trait, which takes `RelPath` values:
validated root-relative `/`-separated UTF-8 paths. `RelPath::join` drops `.` segments
and refuses an absolute input, a backslash, a NUL, a drive-letter segment and a `..`
that climbs above the root with a typed `PathError`. A `..` below the root is kept,
because on a Unix disk it means the parent of the directory the earlier segments
really reach (Windows joins it by text; see Path containment). The trait has four
calls, each returning a `Result`: `read(path, max_bytes)` returns the text of a
file; `read_unseen(path, max_bytes, seen)` opens a file once, takes its `FileId`
from that open and reads its text from the same open unless `seen` accepts the
identity (`identity(path)` is the same call with every identity seen);
`list(path)` returns the entries of a directory sorted by name, each with a kind of
file, directory, symlink or other; and `entry(path)` returns the kind of one path or
`None` when nothing is there. Two paths have equal identities when they name one file:
`DiskTree` takes the identity from the handle it opened and verified, its device and
inode numbers on Unix and on Windows the path the system gives for the handle, below
the root, and
`MemoryTree` uses the normalised key. A device and inode pair names one file where the
file system keeps inode numbers unique and stable while the tree is read, as local
file systems do; a network or user-space mount that reuses or invents them can make
two files look like one. Because the identity and the text come from one open, no
change to the tree can pair one file's identity with another file's text.
`read_unseen` refuses what `read` refuses, and a file it does not read is not refused
for its size or encoding. The trait is object-safe and has no `Send` or `Sync` bound.
A failure is a `TreeError` that names the path: `NotFound`, `OutsideRoot`,
`NotRegular`, `SymlinkLoop`, `TooLarge`, `NotUtf8`, `NonUtf8Name`, `DuplicatePath` or
`Io`. Only `NotFound` means that nothing is there; an entry whose name is not UTF-8
fails the listing and is never dropped from it. `DiskTree::new(root)` returns a
`Result`: it canonicalises the root once, a missing root is `NotFound`, and a
symlinked root is followed to its target. `MemoryTree` holds a map of path to text,
infers directories from the keys, and exists so that fixtures build repositories
without touching the disk. It enforces `max_bytes` like the disk backend and refuses a
key such as `../a` or `/a` with `OutsideRoot`, and a key that names a path another key
already holds, whether literally or after normalisation, with `DuplicatePath`; a tree
built by the infallible conversions from such keys returns the same error from every
call. A missing `/proc` on Linux is an `Io` failure and never `NotFound`. `entry`
returns `OutsideRoot` when the directory that holds the path leads outside the root,
judged on a verified handle of that directory before and after the lookup, and
`None` when that directory is missing or is not a directory. Nothing else in the
crate opens a file.

**Path containment.** Every path the crate opens is checked against the workspace
root, so a hostile tree cannot make the index read a file outside it. This covers
files reached by `mod name;` with or without `#[path]`, explicit `path` values of
`Cargo.toml` targets, and every discovered `Cargo.toml`. A candidate leads where the
disk says: every symlink is followed and every `..` is applied against the real
directories, and where it leads must lie under the canonical root. On Linux and
Windows `DiskTree` canonicalises the resolved path and requires the canonical root as
its prefix; on other Unix systems it walks the path beneath the root, as stated
below. A candidate outside the root is an error that names the module, attribute or
manifest key, and its contents are never read; whether a rename that races the
check can make it be opened depends on the platform, as stated below. A lexical
check of `..` segments is not enough, because `link/..` leaves the root when `link`
is a directory symlink, so `DiskTree` never clamps a `..` in text. On Windows the
canonical root is a verbatim (`\\?\`) path, and the standard library joins a relative
path to it by text, removing each `..` with the segment before it, as Win32 does; the
joined path is still canonicalised and checked against the root, so containment
holds, but there `link/..` names the directory that holds `link` and not the parent
of its target. A symlink whose target is inside the root is followed, whether the
target is relative or absolute. On Linux and Windows the target is followed however
it spells the root. On other Unix systems (macOS and the BSDs) a `..` taken at the
root or outside it is `OutsideRoot`, so a target is followed only when its
resolution reaches the root by descending into it: an absolute target that
descends into the root through any spelling of it is followed,
but `../root/real.rs` from the root, `/…/root/../root/real.rs`, and an outside link
to the root whose own target climbs through `..`, are `OutsideRoot`. A tree that
holds such links loads on Linux and Windows and not on macOS or the BSDs.
`DiskTree` refuses any file that is not a regular file, such as a FIFO or a device,
and it does not descend into a directory symlink: a listing reports it with the
symlink kind, listing it is refused, and a symlink loop ends with `SymlinkLoop`. The
file type is checked on the opened handle and not by a stat before the open, and on
Unix no open follows a final symlink or blocks, so naming a FIFO cannot hang.

A read, a listing and an entry lookup each open a handle (the file to read, the
directory to list, or the directory that holds the entry), verify that handle against
the root and its type, use it, and verify it again after the use. A handle that fails
either check is refused with `OutsideRoot`, and what the use returned is discarded.
On every platform the contents of a file are read only through a handle that was
verified to lie under the root. How the handle is opened, verified and used depends
on the platform:

- On Linux the canonical path is first taken as an `O_PATH` reference, without
  following a final symlink. The kernel's own path for it (`/proc/self/fd/<fd>`)
  must lie under the canonical root and it must be a regular file or a directory as
  wanted; only then is it opened for reading, or listed, through that descriptor
  path, which names the verified inode and is never resolved by name again, and an
  entry is looked up below the same descriptor path. So an outside candidate is
  never opened for reading or listing; only an `O_PATH` reference is taken, which
  performs no I/O, device open or FIFO rendezvous. A directory swapped for a symlink
  to outside the root after the prefix check is refused, and a file or directory
  moved out of the root while it is used is refused by the check after the use.
- On other Unix systems (macOS and the BSDs) the root is opened once, and the path is
  walked one component at a time with `openat`, relative to the directory reached so
  far, never following a symlink and never blocking. The walk holds the descriptor of
  that directory only and records the device and inode of the directories it came
  through: a `..` opens `..` of the current directory and must find the recorded
  parent, and a `..` at the root leaves the root. A symlink is read with
  `readlinkat` and its target walked in its place: a relative target from the
  directory that holds the link, an absolute one from the file system root. A walk
  outside the root may only descend: it is back inside when it reaches a directory
  that is the root (same device and inode), and a `..` taken outside the root is
  `OutsideRoot`, so a link cannot leave the root and climb back into it. More than
  40 symlinks in one walk, counting the steps taken again because an entry changed
  between two calls of one step, end it with `SymlinkLoop`. No path is resolved by
  name. Outside the root only directories are opened (`O_DIRECTORY`, which opens no
  device and no FIFO), and an outside non-directory is never opened. The file is
  read, the directory listed and the entry looked up (`fstatat`, without following)
  through the walked handle. A directory renamed out of the root while the walk is
  inside it is still walked, and the check after the use refuses it: a file must
  still be the entry it was found as in its directory, and that directory, or the
  directory that was used, must reach the root through `..`, so a file or directory
  moved out of the root during the use is refused.
- On Windows the canonical path is opened as a handle, held without delete sharing (a
  directory with backup semantics), and the path the system gives for that handle
  (`GetFinalPathNameByHandleW`, normalised, with its drive letter) must lie under the
  canonical root, after the open and again after each use. A file is read through
  the handle, so its contents are those of the verified file. An outside file can be
  opened in a race and is then refused before any read. A listing and an entry
  lookup go by the handle's verified path, between the two checks, so a swap made and
  undone between them is not seen: through it a listing can return the names of an
  outside directory and an entry the kind of an outside path, never their contents.
  Windows also refuses, before canonicalising and with `OutsideRoot`, every call on a
  path with a segment that Win32 would not read as the name it spells: a reserved
  device name (`CON`, `PRN`, `AUX`, `NUL`, `COM0` to `COM9`, `LPT0` to `LPT9`, the
  superscript-digit `COM` and `LPT` names, `CONIN$` and `CONOUT$`) in any case, bare
  or with any extension; a name that ends in a dot or a space; and a name that holds
  a colon, which names an alternate data stream. `RelPath` accepts these segments on
  every platform, and elsewhere they are ordinary names.
- Elsewhere no handle can be verified, and `DiskTree::new` fails with `Io`.

Whether an outside path exists is answered per strategy. The other-Unix walk answers
`OutsideRoot` whenever a path or a link leads outside the root, whether the outside
path exists or not; the one exception is a link whose target descends from outside
into the root itself (an absolute target that spells the root through another
path, or passes an outside link to the root), which succeeds only when that outside
path exists. Linux and Windows canonicalise first, and canonicalisation follows a
link where it leads, so it tells an outside path that exists from one that does not:
a link that ends at an outside path is `OutsideRoot` when it exists and `NotFound`
(`None` for `entry`) when it does not, and a link that passes an outside directory
and climbs back into the root, such as `../outside/x/../../root/real.rs`, reads the
in-root file when the outside directory exists and is `NotFound` when it does not. A
directory swapped for such a link during a call gives the same answers. The
operating system's canonicalisation cannot be told to refuse the climb back, so
these two strategies keep these answers.

A Linux host runs the Linux strategy natively and the other-Unix and Windows
strategies through a test override, the Windows one with `/proc/self/fd/<fd>` as the
handle's path. On macOS and Windows hosts the native strategies are run only by the
CI jobs that test the crate there; for FreeBSD they are compiled and not run.

**Resource bounds.** Source is parsed in-process. Each read is capped at
`max_file_bytes`, which defaults to 8 MiB and is set by `LoadOptions::max_file_bytes`
or, for the census, by `max_file_bytes` in `[census]` (see
[census-ledger](./census-ledger.md)). A larger file is an error that names it. `syn`
recurses on more than delimiters, so before any file is parsed the crate runs a token
pre-scan, one lexer pass feeding independent counters. The lexer skips comments
(block comments nest) and every string, raw string (with any number of `#`), byte
string, C string and character literal, and tells a lifetime from a character
literal, so tokens inside them count for nothing. A file that exceeds a budget is
refused with a `LoadError` of kind `Budget`, which names the file, the budget, its
limit and the line where the count went over. A file that `syn` rejects is a `Syntax`
error with the line and column of the parser's error, read on the parse thread, and a
thread that cannot start is a `Spawn` error. The budgets are one table in the source
(`Budget::limit`), and the counters are defined as follows:

- Delimiters: the number of `(`, `[` and `{` open at once; the limit is 128, so 128
  levels parse and 129 are refused.
- Nested `<`: each delimiter level has its own count; `<` adds one unless it is part
  of `<=`, and `>` subtracts one down to zero (`->` and `=>` are single tokens and
  subtract nothing). The count of a level returns to zero at `;`, `&&` and `||`, and
  the count of the enclosing level returns to zero when a `{ ... }` block closes, so
  comparisons in separate statements do not add up. The limit is 64.
- Prefix operators: the length of a run of consecutive tokens that are `!`, `-`, `*`,
  `&` or `&&` (a `&&` is one token), with only whitespace and comments between
  them. A lifetime between two operators neither ends the run nor adds to it. `!=`,
  `-=`, `*=`, `&=` and `->` are not prefix operators. The limit is 256.
- Closure bars: a closure head is `||`, or `|` up to the next `|` at the same
  delimiter level. The count is the length of a chain of heads in which each head
  follows the previous one directly, or with `move` between them. The limit is 128.
- `else if`: within one delimiter level, the number of links, where a link is an
  `else` directly after a `}` and followed by `if`. The count returns to zero at `;`,
  at an `else` that is not followed by `if`, and at an identifier other than `else`
  directly after a `}`. A block used as the condition of a link (`else if {x} {}`)
  does not end the chain. The limit is 256.
- Recursion: one additive count of the nesting that the parser builds, and that the
  syntax tree and its drop then walk, by recursion. The counted tokens come from an
  audit of the recursive entry points of `syn` 3.0.8: in expressions the unary
  operators, the trailers (`?`, `.field`, `.await`, a call, an index), the binary
  operators, `as`, ranges, closures with their `async`, `const` and `move` heads,
  `if`, `match`, `while`, `let`, `box`, `return`, `break`, `yield`, `become`, `async`,
  `const`, `unsafe` and labelled blocks; in types references, pointers, `fn` with a
  return type, `impl` and `dyn` bounds with a return type and `for<'a>`; in patterns
  `box`, `&`, `ref`, `mut`, `@` and ranges; and `use` trees. Each delimiter level has
  its own count, and a counted token adds to the count of the level it is read in:
  an operator (`!`, `-`, `*`, `&`, `|`, `<`, `>`, `+`, `/`, `%`, `^`, `=`, `?`, `@`,
  `.`, a range, `->`, `&&`, `||` and the compound forms) adds one for each of its
  characters, except a `!` after `#`, and two for each character of `&`, `&&` and
  `<`, because a reference type and a list of generic arguments cost the parser about
  twice what the other units do; a number with a `.` adds one, because after a `.` it
  is two tuple indices; each of the keywords `as`, `async`, `become`, `box`, `break`,
  `const`, `dyn`, `fn`, `for`, `if`, `impl`, `let`, `loop`, `match`, `mod`, `move`,
  `mut`, `ref`, `return`, `static`, `try`, `unsafe`, `where`, `while` and `yield` adds
  one, and so does a label (a lifetime followed by `:`); the `(`, `[` or `{` that
  opens a group adds one to the level around it, except the `[` of an attribute and
  the delimiter of a macro call; and `::` inside a `use` declaration adds sixteen,
  because the index turns a `use` tree into its imports on the caller's thread by
  recursion, at about 1 KiB of stack for each segment in a debug build. A macro call
  (a name that is not a keyword, `!`, an optional name and a group) is kept by the
  parser as token trees, so inside its group nothing counts but the levels its own
  groups open. In the arms of a `match` (the first `{` of the `match`'s level, after a
  name, a literal, `)` or `]`, with no construct that takes a block between them), a
  `|` between the alternatives of an arm's pattern counts nothing and ends the count
  like a `,`, because the parser reads the alternatives in a loop; a guard is an
  expression and counts. A level's count returns to zero at `;` and `=>`, at a `,`
  unless a `<` of the level, a closure head (`|a, b|`) or a `where` clause before its
  item's `{` or `;` is open, and at the token after a `}` that closed a group in the
  level when that token is a literal, a lifetime, `#`, or an identifier other than
  `as`, `else` and `in`, where a new statement or item starts. The counter's value is
  the sum of the counts of every open level plus one for each open delimiter; a level
  that closes takes its count with it, and nothing else subtracts, so a `>` or a `}`
  followed by `else` adds to or keeps the count. The limit is 1927
  (`Budget::Recursion`): half of the parse stack (`PARSE_STACK_BYTES`, 64 MiB) over
  `RECURSION_UNIT_BYTES`, 17 KiB, the most that one unit takes of the parse thread's
  stack in a debug build, rounded up from 16481 bytes measured on the costliest counted
  shape (generic arguments nested in a type, `A<A<...>>`), counting the parse, the
  reading of the facts and the drop of the syntax tree. A file at the limit therefore
  takes at most half of the parse stack in a debug build, and less in a release build,
  where the costliest unit measured 4670 bytes (a `break break ...` chain). Measured in
  a debug build, a unit costs 16481 bytes for nested generic arguments, 15724 for a
  type behind references, 14808 for tuple, slice and array types, 12657 for nested
  `fn` pointer arguments, 12257 for inline modules, 10414 for nested `trait` bodies,
  10258 for closures with block bodies, 9982 for `break` chains, 8831 for `return`
  chains, 8277 for struct literals, 7358 for assignment chains, 7279 for closure
  chains, 3657 for unary operators, and at most 1118 for the trailers and binary
  operators that the parser reads in a loop and only the drop of the syntax tree
  nests. A `use` tree at the limit loads, resolves and drops on a thread of 512 KiB in
  a debug build. Over this repository and the registry scan below the highest value
  in one file was 987 (`funty`, a trait with more than 160 bounds).
- Tokens: the number of tokens of the file as the parser's lexer reads them. An
  identifier, a keyword or a literal is one, a lifetime two (the quote and the
  name), each delimiter one and each character of punctuation one (`::` and `->`
  are two). A doc comment counts as the attribute it stands for: six tokens for
  `///` and `/**`, seven for `//!` and `/*!`; other comments count for nothing. The
  count feeds no other counter. The limit is `LoadOptions::max_file_tokens`, by
  default 2000000 (`Budget::FileTokens`), and `parse_source` applies the default.
  It bounds the memory that parsing one file takes at its peak at
  `PARSE_BYTES_PER_TOKEN`, 1664 bytes, per token. The costliest shape the other
  budgets admit is one function body of 2^k + 1 empty statements, whose list of
  statements doubles its storage for the last one: it holds 1602 bytes per token at
  its peak at every k from 14 to 20, counting a grown block as held beside the
  block it replaces until that is freed, and the constant adds about 4%. The same
  measurement gives about 430 bytes per token for `a<b; ` repeated, 290 for
  statements that name a type nested 64 generic arguments deep and 230 for a run of
  attributes. The default admits the largest generated files of real crates: over
  the registry scan below the most tokens of one file was 1505567 (a time-zone
  table).

Two more budgets are counted by the module walk and not by the pre-scan. Module
nesting is bounded at 128 levels along one path, files and inline modules together.
The walk, and the index that reads its result, keep a stack of their own, so their
depth costs the caller's stack nothing: 128 nested modules load on a thread of 512 KiB
in a debug build. So does the search that keys the trait of an impl (see Symbol index):
a file of 20,000 chained aliases, and a trait reached through 20,000 chained globs,
load on a thread of 2 MiB in debug and release builds. Module walks are bounded at 262144 for one load: every arrival of
the walk at a file, over all targets, and every walk of an inline module body (the
default one and one per `cfg_attr` path alternative) is one module path, and a file
reached again is parsed once but walked again, so a chain of files that each declare
the next twice would otherwise record 2^n paths from a few kilobytes of input. The
limit is 2^18: a workspace of ten thousand files reached by five targets each records
fifty thousand, and a shared test-support diamond records one path per target and
module that reaches it, so the limit admits every realistic tree. What the walks
record is charged to the item-bytes budget below, which bounds what the limit costs: a
chain of seventeen files that each declare the next twice (2^18 - 1 walks from 1 KiB
of source) is refused by the item-bytes budget after 1.0 s at a peak of 170 MiB
resident, measured in a release build. A load that passes the limit is a `Budget`
error that names the file and the line of the declaration.

**Work and memory bounds.** Discovery and the expansion of `workspace.members`
patterns enter at most 128 directories below their root (`DirectoryDepth`), on a stack
of their own; a deeper directory is a `Budget` error that names it, which a chain of
20000 directories in memory meets on a thread of 2 MiB in a debug build, in under a
millisecond in a release build, and a chain of 2100 on disk in 22 ms with 4 MiB
resident. Four byte budgets bound one load. Each is a default that `LoadOptions` can
change:

- `max_total_bytes` (`TotalBytes`) bounds the bytes the load reads, source files
  and manifests together. Its default is min(1 GiB, max(256 MiB, 4 × the largest
  crate)) over the newest version of each of 1177 crates of a local registry, whose
  largest load read 16.9 MB (`linux-raw-sys`), so it is 256 MiB.
- `max_item_bytes` (`ItemBytes`, 512 MiB) bounds what the index and the module walk
  allocate. Every charge is an upper bound of the allocations it covers, derived from
  the layout of the containers that hold them and not measured on typical input: a
  list is charged twice its elements and its first room of four, a `BTreeMap` a fifth
  of a leaf and an eighth of an internal node for each entry and one leaf for the
  map, a hash table three buckets for each entry and eight for the table, and a
  shared string its text and two counters. An item charges its record, its places in
  the order, in its file's list and in its source group, each string it holds the
  first time the load stores it (paths, names, module paths and written self types
  are stored once and shared), and its scope entry (the name as key, a boxed entry
  and a binding in each name space) or its place among the members of its type. A
  module charges its record, its entry among the module paths, its scope table and
  the first room of its definitions; an arrival of the module walk its reach, its
  places in the orders of arrivals and its entries in the tables that mark it
  indexed; a file read its entries among the files and identities; a spelling its
  entry and text; a flattened import its entry, its places in the ordered lists of
  imports and of their keys, its strings and its flags, each binding
  it makes or replaces its scope entry and route, a glob import the set of modules its
  path reached, and each settle of the tables after
  the first what it builds again (the scope tables of the definitions and the state of
  every import), besides the bindings it makes again; each impl
  block once per module path, each trait key and each type's table of members their
  allocations; each target its entries among the crate roots and its extern prelude (its
  table, and each crate name with the libraries it stands for), each package its
  edition, and each inherent impl the index cannot see its record and the list of its
  members; and the aliased path of a type alias and the crate an `extern crate`
  item names are stored once for the file and charged once, however many module paths
  reach it. Measured with a counting allocator, no shape keeps more than it is
  charged: a chain of files that each declare the next twice keeps 1511 bytes of index
  for each arrival and is charged 3632, nested inline modules whose scopes hold one
  name keep 1443 bytes for each module and are charged 2380. An item record is at most
  256 bytes.
- `max_fact_bytes` (`FactBytes`, 256 MiB) bounds the facts of the files. Each fact
  charges its size and the room of every string and list it owns (each is trimmed
  to its length as it is built), on the parse thread, so an oversized file is
  refused at the line of the fact that passes the budget.
- `max_glob_bytes` (`GlobBytes`, 512 MiB) bounds the glob tables: what the glob imports
  of the modules bring in (see the use-alias index below). Unlike the other byte
  budgets it bounds what is held at once: the tables of one settle are released when
  they are built again, and a refusal comes at the charge that would make what is
  held pass the limit, naming the glob import it was made for. Each table charges its
  record, its class and its entries among the tables, each name it holds an entry and
  its three lists, each item in a list its room, each name's text once for the load,
  each glob's read of a table its places among the module's reads and the table's
  readers, the record of each module's visibilities, the record of which modules reach
  an explicit import through globs (an entry for each glob edge), and a change while it
  waits to be passed on its place in the list of changes and its items, all by the
  layout rules above. Measured with a counting allocator, no glob-heavy shape keeps
  more than it is charged: of four hundred modules that glob one module of a thousand names,
  of 1500 modules that glob a prelude of 150, of a chain of 4000 globs, of a chain of
  600 that each add a name, of 300 modules that glob the crate root that globs each of
  them, of a generated binding crate of 600 child modules, each globbing a parent that
  globs each child, and of a random network of 1000 modules, each keeps at most one
  byte of glob tables for each 1.5 it is charged.

A load over a budget is a `Budget` error that names the budget, the limit that
applied and the file, with the line where there is one. `Workspace::usage` reports
what a load spent against each, and for the glob tables the most they held at once.
Two figures follow, measured in a release build:

1. Retained memory. What a load keeps of its index, its source text, its facts and its
   glob tables is bounded by the item bytes (512 MiB), the source text
   (`max_total_bytes`, 256 MiB by default, which a caller may raise), the fact bytes
   (256 MiB) and the glob bytes (512 MiB): 1.5 GiB at the defaults. 300000 unit structs of their
   own names in three files load in 0.70 s at a peak of 380 MiB on the heap, and
   400000 are refused by the item-bytes budget in 1.1 s; thirty files of 4 MiB of doc
   comments made of quotes, which double when escaped, hold 253 MB of facts and load
   in 1.8 s at a peak of 382 MiB, and forty are refused by the fact-bytes budget in
   1.8 s; 31 files of 8 MiB load in 0.30 s, and 33 are refused by the total-bytes
   budget in 0.30 s. A load near all four limits at once (2000 copies of a chain of
   100 nested inline modules, 20 files of 8.3 MB of doc comments in lines of 300
   characters, 7 files of 8 MB of comments and a module of 1000 modules that each glob
   it while it globs each of them; 214 MiB read and 463, 245 and 475 MiB charged to the
   index, the facts and the glob tables) keeps 1054 MiB and peaks at 1059 MiB, in 14 to
   23 s on a shared host; without the globs the same load keeps 735 MiB and peaks at
   741 MiB, in 13 to 16 s. 1000 modules that glob the crate root, which globs each of
   them, so that every module brings in all 2000 names, load in 2.8 to 4.6 s with 436
   MiB charged to the glob tables at a peak of 360 MiB resident, and 1400 are refused
   by the glob-bytes budget in 1.9 to 3.2 s at a peak of 415 MiB; a chain of 2000
   globs that each add a name loads in 1.9 to 3.1 s with 435 MiB charged at a peak of
   354 MiB (three runs each).
2. One-file parse peak. What parsing one file holds at its peak is at most
   `max_file_tokens` × `PARSE_BYTES_PER_TOKEN` (1664 bytes): about 3.1 GiB at the
   default of 2000000 tokens. The costliest arrangement of statement lists at
   exactly 2000000 tokens loads in 0.80 s with 2623 MiB allocated at the peak, 1188
   MiB of it resident; an 8 MiB file of `a<b; ` repeated (6.7 million tokens) is
   refused by the pre-scan in 40 ms with no more allocated than its text. A caller
   on a host with less memory lowers the limit to its memory over 1664 bytes:
   `max_file_tokens = 320000` keeps the peak below 512 MiB (the costliest
   arrangement at exactly that limit loads in 0.17 s with 462 MiB allocated, 190 MiB
   resident), and refuses files above it with the same typed error.

A load holds at most figure (1) and one file's figure (2) at once: about 4.6 GiB at
the defaults, each term a limit of `LoadOptions` that a caller sets to fit its host.
The bindings that explicit imports make are charged to the index and what globs bring
in to the glob tables, so the scope tables are within figure (1): one module of 3000
functions that 3000 other modules glob, 163 KB of source and 63004 tokens, loads in
35 to 61 ms at a peak of 24 MiB resident, with 3.8 MiB charged to the glob tables, as the
importers read one table of the names. A module that globs one module of 60000 names
3000 times over, globbed by another, loads in 0.8 s with 94 MiB charged to the glob
tables and a heap peak of 100 MiB, under the 206 MiB charged in all, as gathering what a
module exports reads each table it holds once and is charged before it is allocated. While the tables settle, a load also
holds the work queue of the imports and, each time it runs empty, the graph of the
imports still undecided and the places they wait on; no budget counts them, and they
are dropped once the tables are settled. They grow with the undecided imports times
the places their paths read through globs, and the time it takes to decide them grows
faster still (the crate sets no time bound): 100 modules that glob the crate root,
which globs each of them, each with an import of a name that nothing defines, settle
in 20 to 32 s at a peak of 87 MiB resident, of which the glob tables are charged 2.5 MiB,
and 200 such modules settle in 613 s at a peak of 555 MiB resident, of which the glob
tables are charged 9.7 MB: the memory no budget counts can pass the glob budget.

Parsing then runs on a dedicated thread, named after the file, with a stack of 64
MiB (`PARSE_STACK_BYTES`), which is a stated constant and not the stack of the
caller. `parse_source` runs a visitor on the syntax tree on that thread and
returns what the visitor returns. A committed fuzz
target, `parse-source` in `fuzz/fuzz_targets/parse_source.rs`, feeds arbitrary bytes
through the pre-scan and the parse. A second, `load-workspace` in
`fuzz/fuzz_targets/load_workspace.rs`, builds a `MemoryTree` from its input, a
manifest and a few files, loads it with `Workspace::load` and resolves paths through
the index. Each has a committed corpus under `fuzz/corpus/`; that of `parse-source`,
`fuzz/corpus/parse-source/`, holds the composite and single-shape fixtures of Verification below,
files at and just over each budget, the chains that overflowed the parse stack before
the recursion budget counted them, and every input that ever aborted a run. The
recursion budget is additive over the levels, so nesting one shape inside another
adds their counts: the composite shapes of Verification and every chain of a counted
token are refused, and an admitted file takes at most half of the parse stack in a
debug build. Where a construct owns a list, a `,`, `;` or `=>` ends the count of its
level while the construct stays open; that frame is one for each delimiter level,
which the delimiter budget bounds. Shapes that try to slip past a counter (a block as
the condition of each `else if`, a lifetime between reference marks, a `,` inside
generic arguments or a closure head, an `else`, `in` or `as` after a `}`) are counted
by the definitions above and refused. The bound rests on the audit of `syn`'s
recursion and on the measurements above. A residual overflow aborts the process, so
the run fails and is never reported as a pass. A parser overflow is a robustness bug,
and an input that causes one is added to the fuzz corpus and to the budgets. `Cargo.toml` is read with the same
size cap, and the `toml` parser's own recursion limit turns deep nesting into an error
and not a crash. Every refusal the pre-scan makes is a typed error and not a panic.
The crate sets no time bound.

**Package discovery.** `Workspace::load(tree, roots)` takes explicit roots; the
crate has no default roots, each caller names its own (`"."` is the tree root), and an
empty list is a `NoRoots` error. `Workspace::load` returns the first error, in the
order discovery, targets, module walk, index; it does not collect errors.
Discovery reads each root as a directory. A directory with a `Cargo.toml` that has a
`[package]` table is a package. The walk descends, on a stack of its own and at most
128 directories below the root, through every directory that is not
a package, and through a package that also has a `[workspace]` table, so a root `.`
package with a `[workspace]` table and `crates/a` below it are both found. It does not
descend below a `[package]` manifest that lacks `[workspace]`, so a fixture package
under a tests directory is not found. It skips directories named `target` and
dot-directories. A symlink is classified by what it resolves to: a regular file inside the root
is left alone, a directory is a `DirSymlink` error, a special file such as a FIFO is
`NotRegular`, a target outside the root is `OutsideRoot`, a dangling link is
`NotFound` and a loop is `SymlinkLoop`. None of these reads more than one byte. A manifest that is a
virtual `[workspace]` contributes nothing but its children. Two packages with one name
are an error that names both manifests, and one manifest reached from two roots is one
package. Manifests are read through `Tree` under `max_file_bytes` and parsed with the
`toml` crate, so comments, single-quoted strings and inline tables are honoured; a
manifest that is not TOML, is nested deeper than the parser allows, or has a value of
the wrong type is a `Manifest` error that names the file and, when one is at fault, the
key. Packages are sorted by manifest path.

For each package the crate derives the targets, in this order: the library, the
binaries, the tests, the benches, the examples and the build script. A target has the
package name, its Cargo name, its kind, its root file, a shipping flag, a crate path
and the test settings of its table: `harness`, `required-features`, `test` and
`doctest`. A setting the table leaves out, or a target without a table, takes cargo's
default: `harness` true, no required features, `test` true for the library, the
binaries and the tests and false for benches and examples, and `doctest` true for the
library only. The build script has no table and is never tested, so its `harness`,
`test` and `doctest` are false. A setting of the wrong type is a `Manifest` error that
names its key, such as `bin.harness`. The library is `[lib]` or `src/lib.rs`, named by `[lib] name` or the package
name. Binaries are the `[[bin]]` tables, `src/main.rs`, `src/bin/*.rs` and
`src/bin/*/main.rs`; tests, benches and examples are their tables plus `tests/*.rs`,
`tests/*/main.rs` and the same under `benches/` and `examples/`. `autobins`,
`autotests`, `autobenches` and `autoexamples` set to false turn the inferred ones off,
and an explicit table is found either way. An explicit table without `path` takes the
default location for its name. The crate path is the crate name for the library
(`[lib] name`, or the package name, with `-` as `_`), `<pkg>::bin_crate::<bin>` for a
binary, `<pkg>::test_crate::<target>`, `<pkg>::bench_crate::<target>` and
`<pkg>::example_crate::<target>` for the others, and `<pkg>::build_script` for the
build script. Each crate path resolves to the target's root as a module item. Library,
binary and build-script targets are shipping, so the census sees
`build.rs`; tests, benches and examples are not. These are typed errors that name the manifest key: an explicit
target file that does not exist (checked with `entry`), a table without a `name`, two
targets with one root file, two targets of one package with one crate path,
which would merge their items, and a `path` that leaves the tree. Targets of different
packages may share a crate path, as two packages with one `[lib] name` (or names that
differ only in `-` and `_`) do: the index keys every crate by its package, and such a
path is told apart with its package (see `resolve_in_package`). A root file that
cannot be read names what led to it: the `path` key of its table when the manifest
names the file, the `[lib]` table (`lib`) when it declares the library without a path,
the `name` key of a `[[bin]]`, `[[test]]`, `[[bench]]` or `[[example]]` table without a
path, `package.build` for a named build script, and package discovery for a target
found at its default location without a table.

**Mod-tree following.** From each target root the crate follows module declarations
and parses each file with `syn`, as rustc resolves them. Without a `path` attribute,
`mod name;` is `name.rs` or `name/mod.rs` in the module directory; both existing is an
error that names both files. The module directory of a root file or a `mod.rs` file is
its own directory, and that of any other file `a.rs` is `a/` beside it. An inline
module `mod m { .. }` adds `m` to the module directory. `#[path = "p"]` on `mod name;`
names a file relative to the directory of the file that holds the declaration, or,
inside an inline module, relative to the module directory including the inline
components; on an inline module it replaces that module's directory component. A file
named by a `path` attribute is treated like a `mod.rs` for its own children. Two
`path` attributes that apply unconditionally are an error. A path in
`#[cfg_attr(predicate, path = "p")]` is another site that is followed, carrying its
predicate, and a nested `cfg_attr` is flattened with the combined predicate
(`all(outer, inner)`); the default file is followed too when it exists, and may be missing when
every alternative names a file. No `cfg` is evaluated and no item is dropped. A raw
identifier module `r#match` reads `match.rs`. A path that leaves the workspace root,
whether by `..` segments, an absolute path or a symlink, is an error that names the
module and the attribute and does not read the file. An unresolved `mod` is an error
that names the declaring file and its line. Files are told apart by their identity
in the tree (`Tree::identity`), not by their spelling. The walk detects a cycle on the
stack of files being followed, compared by identity, never with a visited set, and a
cycle is an error that names the file that would be entered twice, also when it is
reached again under another spelling (`l/x.rs` from `x.rs` where `l` links to `.`).
Module nesting along one path is bounded at 128 levels, files and inline modules
together. A file reached by several module paths, by two targets such as a shared
`tests/common/mod.rs`, or under several spellings (`src/b.rs` and `src/z/b.rs` where
`z` links to `src`), is read and parsed once, listed once under the spelling that
sorts first, and recorded under every path it was reached by; a `dir/..` pair is
collapsed only when `dir` is a real directory, so a spelling is one the disk can
follow, and one file has one spelling in the index. The walk stops at the first
error. `Workspace::targets` yields the targets, and `Workspace::files` yields each
source file once with its text and the module paths it was reached by. The facts of a
file stay inside the crate, and a caller reads them through the items of the index
(`Workspace::items_in`, `Workspace::item_at` and the other accessors). The
facts of a file are read on the parse thread and owned: items with their name line,
span, body span, attributes, visibility, documentation, test role and, for a struct, its
field shape; `mod` declarations with their raw `path` and `cfg_attr` attributes; `use`
trees; impl headers with the written self type and its leading `::`; `macro_export`;
an unnamed `const _` as an explicit kind; and the file's own inner documentation.
Documentation counts a non-blank `#[doc]` text or `#[doc = include_str!(..)]`, and
`#[doc(hidden)]` alone does not count.

**`cfg(test)` tagging.** An item has the test role `None`, `CfgTest` (guarded by a
`cfg` that implies `test`, directly or through an enclosing module) or `TestFn` (marked
`#[test]`). A predicate implies `test` by this matrix: `test` does; `all(..)` does when
any operand does; `any(..)` does when every operand does; `not(..)` and every other
predicate do not. A `cfg`, `cfg_attr`, `path` or `test` attribute of the wrong shape, a predicate
that cannot be read, and a `path` whose value is not a string literal (such as
`concat!(..)`) are a `CfgParse` error that names the file and the attribute's
line. An error in a `path` attribute of a module is reached by that attribute: its
origin names the module and the `cfg_attr` predicate that applies it, if any, and not
what reached the file. A module reached through `#[cfg(test)] mod t;`, or
through a `path` that a `test` predicate sets, is recorded with the test flag on that
way of reaching it, so every item in the file is test code when all its ways in are
tests. Test items stay in the index with that tag so that a caller decides whether they
count.

**Symbol index.** Every named item of every module tree is an `Item`: a function,
struct, enum, union, trait, type alias, constant, static, module, `macro_rules!`
macro, `extern crate` item, item of an `extern` block, method of an inherent or trait
`impl` block, default and required method of a trait, and associated constant or type.
An item has an `id`, its full `path`, its `kind`, its `namespace`, its `file`, the line
of its name, its `span` and `body` (a `SpanFact`) in lines and bytes, its `visibility`, `documented`,
`doc_hidden`, `test` role, `target` and, for an impl member, the written `self_type`
and `trait_path` and the `self_key` that says how its self type placed it (`Local`,
`Trait` or `Unresolved`). Line ranges are 1-based and half-open: the `lines` of a
`SpanFact` run from its first line to the line after its last, as in `raf_okf::Span`,
so an item on line `n` alone has the lines `n..n+1`, and its bytes run from
`start_byte` to `end_byte`, the offset just past its end. The module path in an item's `path` is the chain of `mod`
declarations that reached its file, whatever the order of the file names. The name
spaces are type, value and macro. A function, constant or static is a value; a
module, trait, enum, union, alias and `extern crate` item is a type; a unit or tuple
struct is a type and also a value, through its field shape; a `macro_rules!` macro is
a macro, and a `#[macro_export]` macro is indexed at the crate root wherever it is
written. A name can be in all three at once. `const _` defines no
name and is not indexed; it stays in the facts of its file. A macro invocation that
defines items is not expanded, so what it generates is not indexed; in a module, an
impl block, a trait and an `extern` block alike it is skipped. `documented` is
true for a non-blank `#[doc]` or `///` text and for `#[doc = include_str!(..)]`, and
`doc(hidden)` alone does not count; a module that has its own file is documented by the
file's inner `//!` text as well. The `test` role is `CfgTest` also for an item in a file
that is reached only through test code.

Two shapes of item that rustc parses but syn has no node for are read from the tokens
syn keeps, each with its own attributes, visibility and span. A function with no body,
at item level or in an impl block, is a function or method whose `body` is `None`; it
is the input of an attribute macro, such as `#[ref_cast_custom] pub fn new(s: &str) ->
&Self;`, or an item under a `cfg` that is false. A type with no definition (`#[ty] type
bool;`) is a type alias whose alias target is `None`, so no path is followed through it.
Both are indexed as written: what the macro makes of them (kept, renamed, dropped, or an
item of another kind) is not seen, as for any other item an attribute macro rewrites. The third
shape syn returns as tokens, a `use` group with a member that starts with `::`, is read
by the use-alias index below. Every other item that syn returns as tokens fails the load
with `UnsupportedItem`, naming the file and the first line of the item: a constant with
no value (`const C: u8;`, in a module or an impl block), a type with bounds and no
definition (`type T: Bound;`), a trait alias, and any other form.

Every target's crate root is also an item of the module kind, at the target's crate
path in the type name space: its file is the target's root file, its line is 1, its
span is the whole file, it is documented when the file starts with `//!` text, and it
is test code for a test or bench target. `resolve` finds it by the crate path (`demo`,
`demo::test_crate::it`, `demo::example_crate::e`, `demo::bin_crate::x`,
`demo::build_script`). The segments `test_crate`, `bench_crate`, `example_crate` and
`bin_crate` are not items: `demo::test_crate` alone names nothing unless a first-party
module has that name. A root item spans the whole file, so its source key is not
unique: it can equal the key of an item that starts at byte 0. Root items are left out
of `items_by_source`, and `items()` and `items_in(file)` include them. If a crate path
is also the path of a first-party item of another target, `resolve` returns
`Ambiguous` naming both. Every module belongs to one crate: a library module that
shares its path with another target's crate root (`pub mod bin_crate { pub mod x {..} }`
beside the binary `x`, or `pub mod build_script` beside a build script) keeps its items
in the library, and each target's items stay under its own root.

A method is recorded under the path of its impl type: an impl whose self type names,
after resolving type aliases, a struct, enum or union of the workspace records its members
under that type (`self_key` is `Local`), as inherent methods or, for a trait impl, trait
impl methods. An inherent impl whose self type names no struct, enum or union the index
can see (a type that a macro such as `bitflags!` generates, a trait object such as `dyn
LocalTrait`, a type that does not exist) does not fail the load: its members are items
marked `SelfType::Unresolved`, recorded under the self path written in the impl's
module (the module path followed by the path the self type is written with, with
`crate`, `self` and `super` taken from the module, a path that starts with `::` from
its first segment, and the first trait of a trait object; for a self type that has no path, the written self
type without spaces), so `impl Kind { fn is_a() }` in `demo::kinds` gives
`demo::kinds::Kind::is_a`. No scope or type names them. `items()` and `item_at()`
carry the marker, and `unresolved_impls()` lists each such impl once for each module
path its file is reached under (its self path, written self type, file, line, span,
target, test role and members, sorted by path, file and position), so a consumer that
needs every impl placed can refuse them. A path that names nothing else and goes through
such a self path (`demo::kinds::Kind::is_a`, `demo::kinds::Kind::other`) is
`UnresolvedSelf`; the self path itself, a type the index cannot see, is `NotFound`. An
impl of a trait on any other self
type (a path the workspace does not define, a reference, a slice, a tuple, a generic
parameter) records its members under the key of the trait followed by the method, with the
written self type and the written trait path kept on the item, and the key itself
(`TraitKey`) on the item as well (`self_key` is `Trait`). The key of a local trait is the path it is defined at.
The key of a dependency trait is its absolute path, found by following the written path
through the module's own `use` declarations and `extern crate` items, through the `use`
declarations of a local module that the path passes through (`pub mod traits { pub use
serde::Serialize; }` and a direct `serde::Serialize` are one key), and through the globs
of local modules (`use super::*` reaches what the parent imported). A `use` declaration
is followed as the Use-alias index reads it: never through its own binding, so `use
std;` with `impl std::fmt::Display for &Foo` keys `std::fmt::Display`, and `pub use
serde;` with `use crate::serde::Serialize;` in a module keys `serde::Serialize`; from
the crate root in edition 2015; and, when its first segment is ambiguous, through each
reading, so the key is `Unresolved` with a candidate for each (a local trait at its own
path). An explicit import met again while its own path is being followed adds nothing to
that way, as a glob met again does: `mod inner { use super::*; pub use serde::*; }` with
`pub use inner::Serialize;` keys `serde::Serialize`, and a trait that only a cycle of
imports names is a typed `UnresolvedTrait` error. An explicit import of the name
shadows the globs of its module here too: `use serde::Serialize;` beside `use
local::*;`, whose `local` defines a trait `Serialize`, keys `serde::Serialize`. A name read from the crate root in
edition 2015 names a crate only through an `extern crate` item there, or as the crate
the compiler puts there (`std`, or `core` under `#![no_std]`), so `use
serde::Serialize;` without `extern crate serde;` at the root keys nothing and the trait
is refused. In edition 2015 a trait path written with a leading `::` starts at the crate
root (`extern crate serde as s;` with `impl ::s::Serialize` keys `serde::Serialize`); in
later editions its first segment is a crate the impl's code can name. A trait path that
reaches local traits at several paths (two globs that bring in two traits `T`) keys
`Unresolved` with each, and a name that several `extern crate` items of a module bind
stands for each of their crates; neither takes the first. The search keeps a stack of its own, so its depth costs the
caller's stack nothing. The head of the
resulting path must be a crate the impl's code can name: the sysroot crates `std`, `core`,
`alloc`, `proc_macro` and `test`, or a crate of its target's extern prelude (see
Use-alias index): its package's library for a target other than the library and the
build script, and a crate declared in `[dependencies]` (for a library
or binary), in `[dependencies]` and `[dev-dependencies]` (for a test, bench or example
target, and for test code of a library or binary) or in `[build-dependencies]` (for the
build script), under the name it has in code (a renamed dependency, `package = ".."`, keys under the new
name, not the name of the package it renames; a dependency on a package of the
workspace, not renamed, under the crate name of that package's library; any other
under its declared name) with `-` as `_`. A path whose head is none of these is a typed
`UnresolvedTrait` error that names the file and line. `core` and `alloc` are written as
`std`, which re-exports their traits at the same paths, so the three spellings of one
trait are one key. A single-segment name that nothing in the module binds is looked up
in the prelude of the package's edition, taken from its manifest (or from
`[workspace.package]` when it inherits one). The workspace manifest is found the way
cargo finds it, by walking up from the package directory through its ancestors within
the tree: each ancestor `Cargo.toml` is read through `Tree` under `max_file_bytes` (an
ancestor without one is skipped, an unreadable or malformed one is an error), the
nearest one with a `[workspace]` table decides, as long as the package is a member of it,
as cargo requires. The members are the root package of the workspace manifest, if it has
one; the directories that match an entry of `workspace.members` and no entry of
`workspace.exclude` (an exclude entry also covers what is below it); and, transitively,
the `path` dependencies of the members, in `[dependencies]`, `[dev-dependencies]`,
`[build-dependencies]` and the `[target.*]` tables, renamed ones and `workspace = true`
ones that `[workspace.dependencies]` gives a path included, that lie inside the
workspace directory and are not excluded. A pattern may use `*`, `?` and `[...]` (with
ranges and a leading `!` or `^`) in each path segment, and is matched in time
proportional to the pattern times the name; `**`, a backslash, an unclosed bracket and an
empty segment (`a//b`, `a//`) are a typed manifest error, never ignored; one trailing `/` is
allowed (`crates/*/` is `crates/*`). `workspace.members` and `workspace.exclude` each
hold at most 4096 entries, a manifest error naming the key beyond that. Each distinct
pattern is compiled once: a duplicate is dropped, an exclude without a wildcard is a set
entry looked up once for each directory above a path, and only the excludes with a
wildcard are matched one by one, so a manifest of 20000 members and 200000 excludes
(880 KB) is refused at once. The cost that remains is the directories below a root times
the wildcard excludes: 4096 member directories against 4095 distinct wildcard excludes
load in about 50 ms, and an adversarial set of star-heavy wildcard excludes over 4096
directories takes about a second. An entry of `workspace.members` that has no wildcard names one directory, which must exist and have
a `Cargo.toml`, or it is a manifest error naming `workspace.members`, unless it is
excluded, in which case it is ignored and need not exist; a glob entry skips
what it matches that is not a directory or has no `Cargo.toml`. The member manifests are read through
`Tree` under the size cap, once for each workspace manifest, and the walk is bounded by
the module-path budget; a path dependency inside the workspace whose manifest is missing
or malformed, or whose `path` cannot be a path here (a backslash, a drive letter), is a
manifest error that names the dependency key; a path that leaves the tree or is absolute is
not a member. A `[workspace.dependencies]` entry whose `path` is not a string is a manifest
error naming it. A package that inherits
from a workspace it is not a member of is a manifest error that names `package.edition`
and the workspace manifest. Reading the workspace manifest
adds no package to what discovery found, because the roots alone decide that. An inherited edition that the
nearest workspace manifest does not set, or with no workspace manifest above the
package, is a manifest error that names `package.edition`, as it is for cargo, and a
package with no `edition` key is 2015: the traits of the first prelude, plus `FromIterator`,
`TryFrom` and `TryInto` from 2021, plus `Future` and `IntoFuture` from 2024. A glob of a
local enum (`use self::Inner::*;`) brings in its variants, in the type and the value name
spaces; a variant is neither a trait nor a module, so the glob gives a trait key nothing.
A glob of a
dependency (`use std::fmt::*;`) cannot be looked into, and neither can a glob of a path
in the crate that names nothing the index can see (a module or an enum that a macro
generates, `use self::Signal::*;`); such a glob is keyed under its path from the crate
root (`demo::signal::Signal`), and a glob does not provide a name its own path needs. A
name on the way that two globs bind to two modules gives a path through each, so the key
lists them all and is never the first of them. Likewise a trait path through a crate name
that stands for several libraries (see Use-alias index) keys `Unresolved`, with a
candidate in each library: its crate path followed by the rest of the path. If exactly one such glob in scope
could provide the name, the trait is keyed under that glob's path followed by the name, so
`use serde::*` and `use serde::ser::Serialize` can give two keys for one trait: the glob
cannot be looked into, and its path is all the key has. A glob whose first segment is
neither a local module (`crate`, `self` and `super` included) nor a crate the impl's code
can name is a typed `UnresolvedTrait`
error that names the glob's file and line when a trait key consults it; when several
such globs could give the trait, it names the first by written path, file and line,
whatever the order of the declarations. If
several could, or the name is also in the prelude, the key is `TraitKey::Unresolved`: it
holds the written path and every absolute path the trait could have, the members are
recorded under the written path, and `resolve` finds them by that text. Members of
different impls that share a key are one symbol with several sites, and where a local
trait declares the method the path resolves to the declaration. A trait's methods are recorded
under the trait.

The identity of an item is its file and the byte offset where its span starts. A file
that several module paths reach is indexed once for each path, so the same source item
can appear as several `Item` values that share a file and offset and differ in `path`
and `target`. `items` yields every item sorted by path, then name space, file and
position, strictly increasing. `items_under(prefix)` yields the item at `prefix` and the
items below it, matching whole path segments: `demo::a` selects `demo::a` and
`demo::a::b` but not `demo::ab`, and an empty prefix selects every item.
`items_by_source()` and `same_source(&item)` group the items that share
one identity, so a census can count a source item once however many paths reach it;
`item(id)` is a lookup; `items_in(file)` yields a file's items in order
of position; `item_at(file, offset)` yields the innermost items that hold the offset;
`is_shipping(&item)` is true for an item of a shipping target that is not test code;
`files()` yields each source file once, with its package (the package of the first
target in target order that reaches the file), the targets and module paths
that reach it, and whether it ships; `source(file)` is the exact text that was indexed;
and `test_fns()` returns, for each `#[test]` function (or a typed error if one's path does not start with its target's crate path), the package, the kind and Cargo
name of its target (`--lib`, `--bin <name>` or `--test <name>`) and its path inside the
crate, which is what `cargo test -p <package> --test <target> <name>` needs;
`usage()` gives what the load spent against its budgets: the bytes read, charged to
the index and to the facts, the most charged to the glob tables at once, the most
tokens of one file, the highest value of the recursion counter in one file and the
deepest directory entered. The index
has no order that depends on the tree implementation: two loads of one tree are equal,
and the same regular files on disk and in memory give equal indexes.

**Use-alias index.** Every `use` declaration contributes to a table of the names of its
module: renames, nested groups, `self` in a group, globs, a leading `::` (a crate of the
extern prelude from edition 2018 on, the crate root in edition 2015), `extern crate x as
y`, and the keywords `crate`, `self` and `super`; a `super` above the crate root is an
error that names the file and line. A leading `::` on a member of a braced group (`pub
use { ::std::fmt::Debug as D, crate::a::f };`) has no syn node; each member of such a
group, and of a group without a path nested in it, binds as the separate `use` with the
attributes and visibility of the whole declaration would, and its facts span the whole
declaration. A `::` inside a group under a path (`a::{::b}`), which rustc refuses, is a
syntax error. From edition 2018 on, a path whose first segment the
module does not bind starts at a crate of the extern prelude of the importing target. In
edition 2015 a `use` path whose first segment is not `crate`, `self` or `super` starts
at the crate root, as rustc reads it: `use inner::go;` in a module names the root's
`inner`, and a dependency is named through an `extern crate` item at the root. That
prelude holds: its package's library, for a target other than the
library and the build script; the dependencies declared for its kind (`[dependencies]`
for a library or binary, which also name `[dev-dependencies]` in test code;
`[dependencies]` and `[dev-dependencies]` for a test, bench or example;
`[build-dependencies]` for the build script), each in the `[target.*]` tables too, where
a dependency whose `path`, or whose `workspace = true` entry's `path` in the
`[workspace.dependencies]` of the package's workspace, names the directory of a package
of the workspace stands for that package's library, under the crate name of the library
or, when the entry renames its package (`package = ".."`), under the name it is declared
under; the names of `extern crate` items at the crate root (`extern crate self as name`
names the crate itself); and the sysroot crates, which are outside the index. Any other
dependency is outside the index. So `use shared::X` in a package that depends on one of
two packages whose libraries are both named `shared` reaches that one. Whether a library
is named only by test code is kept for each library a name stands for: when a library's
dependency and its dev-dependency (renamed to it) are two packages under one name, code
that is not test code names the first only, and test code names both. A name that stands
for several libraries is ambiguous whatever follows it, as rustc reports it: a path
through it, through an `extern crate` item of it or through an import of it is
`Ambiguous`, naming the crate root of each library, even where only one of them defines
the rest of the path, and in each name space the path could end in (an import of the name
alone names crates, in the type name space); a glob of it, directly or through an import
of it, brings in nothing. An import never resolves through its own binding: the
path of `use x;` does not look at the `x` it binds, and the import never waits on
itself. Where its path reads the name it binds in its own module, the explicit imports
of that name, its own among them, are not there for it: they are alternatives under
`cfg`, as rustc refuses two imports of one name, so `use shared::inner as shared;`
reads the crate `shared` and binds `shared::inner`. From edition 2018 on, a single-segment import of a crate of the extern prelude
(`use x;`, `use x::{self};`, `use x::{self as y};`, `use ::x;`) is an alias of that
crate. For a library of the workspace it binds the library's crate root item, so after
`pub use p_one;` in `p_two`, `p_two::p_one::X` resolves to `p_one::X`. For a crate outside the
index, a dependency outside the workspace or a sysroot crate, it binds nothing in the
index: a path through it, directly or through another import of it, is `NotFound`,
never `Cyclic`, and a trait key through it is the crate's absolute path. In edition 2018
and later, a first segment of a `use` path that a glob of the module brings into the
type name space and that is also a crate of the extern prelude, other than that same
crate, is ambiguous, as rustc's E0659 has it. Of the sysroot crates, the extern prelude
of a `use` path holds `core`, and `std` unless the crate root has an unconditional
`#![no_std]` (a `cfg_attr` applies it under a predicate only, so `std` stays); an
`extern crate std;` at the root adds it back. The ambiguous import binds nothing and
keeps its name indeterminate, so no glob of its module answers for it, and a path through it is
`Ambiguous`, naming what the glob brings in and the crate root of each library of the
workspace that the crate name stands for. A glob that brings in an alias of another
crate under the crate's name (`pub use std as serde;`) collides with the crate as well,
and the ambiguity names the module of that import. A glob that brings in an item of a
crate outside the workspace under that name does not: the index cannot tell the item's
name space, and only the type name space collides (`use quote::quote;` brings in a
macro). That is a known limit: a glob that brings in a type-namespace item or module of
an outside crate under the crate's name (`pub use serde::de as other;`), which rustc
refuses as E0659, is not refused, and a path through the name reads the crate. The
decision does not depend on the order of the declarations: such a path waits until the
globs of its module have brought in what they can before it starts at a crate, every
import of a module with globs is then tried again, as one that bound its name in one
name space may then bind a crate in another, and once
the tables are settled every explicit import is walked again against them; if one is
then ambiguous that was not held from the start (a glob whose own path went through an
import of a crate brought the name in after it), or a glob whose own path is then
ambiguous, the tables are settled again from the definitions with every such import
held from the start, a held glob bringing in nothing, until none is left. Each repeat
holds at least one more import, so the tables are settled at most once more than there
are imports, which is checked in release builds too (more is an `Internal` load error),
and each settle after the first is charged to `ItemBytes`
what it builds again besides the bindings it makes again, so the repeats are bounded
by the budget as well. A module's own item or explicit import of the name shadows the
crate without ambiguity, whatever the order: a path, `use` path or not, whose first
segment an explicit import of its module keeps from the globs in the type name space
(see below) never starts at a crate; it waits until the import binds the name and goes
on through what the import binds, and finds nothing there if the import never binds it
(an import of an item outside the index, or one that fails or is part of a cycle). An
import of a function under a crate's name (`use crate::vals::shared;`) leaves the type
name space, so `pub use shared::x;` beside it reads the library `shared`, and beside a
glob that brings in a module `shared` as well, through another module, is ambiguous;
and a glob that brings in only a value, or the crate
itself (`pub use serde;`), does not collide with it. A `package` value that is not a string is
a manifest error that names the dependency key, and so is a `workspace = true`
dependency when there is no workspace manifest above the package, when the package is
not a member of it, or when its `[workspace.dependencies]` has no entry of that name. A name that a
module defines or imports explicitly shadows a name that one of its globs brings in, in
each name space where it binds it, as in rustc. An explicit import keeps its name from
the globs of its module, and from the crates, in each name space until it is decided
there, and then in each name space where it binds the name, or where its path reaches an item
of a crate outside the index, a module with a glob the index cannot resolve, or nothing
that the index can settle; it leaves to the globs only a name space where it binds
nothing and its path reached places that can bind nothing there, which the index knows
only for an import that binds the name in another name space. So `pub use serde::A;` or
an import that fails, beside `pub use crate::x::*;`, leaves `A` unresolved and never
reads the glob's `A`, while `use crate::y::A;` of a struct beside a glob of a function
`A` binds the struct in the type name space and leaves the value name space to the
glob's function, for the module and for the modules that glob it. Until the import is
decided in a name space, an asker there waits, and a glob of another module does not
bring the name in there from the import's module meanwhile, as rustc hides a glob
binding of a name that a single import may still bind; a name space the import is
decided in no longer waits on what it is undecided in. What the globs of a module answer may grow as they bring in
more: a path that read the name is read again, and two items make it ambiguous, as in
rustc, so a module that globs both that module and another that brings in a function
`A` reads both functions, an ambiguity, whatever the order. A name that a module brings
in ambiguously through its globs stays `Ambiguous` for a module that globs it, also where
another glob of that module brings in one of the items: rustc then only warns
(`ambiguous_glob_reexports`) and resolves the name, and the index refuses it. That a name space binds
nothing is decided only once no glob that reads it can still bring anything in, its
own path left out as rustc leaves out the import it resolves: a glob whose path does
not resolve yet, or whose targets' globs are not settled, holds the decision back, and
a glob's own path is read without the names the glob itself may still bring in. A glob brings in only the names the
importing module can name: `pub` items, `pub(crate)`, `pub(super)` and `pub(in path)`
items when the importer is inside the restriction, and private items only when the
importer is the defining module or lies inside it. A `use` path, of an explicit import
or of a glob, is walked as rustc resolves an import: a binding that the importing module
cannot access does not count (a definition or explicit import it cannot access hides the
globs all the same, and keeps the name from them until it is decided), and a glob it
cannot access brings it nothing, so a cycle that only such a glob
closes is no cycle (`pub mod a { pub use self::b::B; pub use crate::c::*; mod b { pub fn
B() {} use super::*; } }` binds the function and leaves the type name space to `c`'s
glob). A canonical path given to `resolve` and the path of an impl's trait are not
checked for visibility. An item that the globs of a module reach by several routes, through a public
glob and a private one, is as visible from that module as its most visible route,
whichever route is kept, so a module that globs it sees the item in every order. The tables are settled to a fixed point: an import is tried again whenever a
name its path read gains a binding, so it binds what its path reaches in the settled
tables, a chain of re-exports of any length resolves, nothing caps the search, and a
cycle of globs adds nothing. What a glob brings in from its target depends on the
target and on which of the target's visibilities the importer can access, so the glob
tables keep what a module exports to globs (its bindings that shadow globs, and what its
own globs bring in where none does) once for each class of importers that can access
the same of it, and every glob of the module from an importer of that class reads that
one table: no importer holds a copy of what its globs bring in, and a module
that a thousand modules glob is held once. What a table gains is passed on as it is
gained, only to the globs that read it, only with the names that gained something, and
only to the tables of the importing module that the glob's visibility reaches and to
the imports that wait on one of those names there, so a chain of globs costs what each
link brings in, once; a glob that reads a table another glob of its module with the same
visibility reads passes on nothing more, so a module that reads one table through many
globs passes it on once. As it is passed on at once, every table is up to date when an
import reads it: after the imports left are decided (see below), the decisions take
effect in every glob table before the queue runs again, and an import of a cycle left
to a glob reads all that the glob brings in by then, as rustc does (E0659 for two
items). A name space where an import's path reached nothing,
reading there only places in modules without globs that no other import binds or, where
it found nothing, that every other import of the name was decided to bind nothing at,
is decided in the queue, as it can never bind there; so a chain of 20,000 imports that
bind nothing is decided in the queue. When nothing more resolves,
the imports left are decided. Each waits, in each name space it is undecided in, on the
places its path reads there, a name in a module in a name space; the name it binds
itself it reads as rustc does, without itself, through what the globs of its module may
bring in. A place waits on the imports undecided there that keep it from the globs and
on the places its globs may still bring the name in from. The name spaces of one import
are decided apart, as rustc holds back a glob binding only in the name space where a
single import may still bind the name. The strongly
connected components of this graph are decided in waves, each once what it waits on is
decided, and the queue runs after each wave: an import that waits on no cycle is
settled, and leaves to the globs each name space where it binds nothing and can bind
nothing; places that only a cycle of globs keeps open stop waiting. In a cycle, a name
space where an import's path reads a place that something may still bind without the
imports of the cycle is not decided, as rustc, which cannot read through a glob a name
that a single import still keeps from the globs, never determines it. A component with
two or more imports is a cycle, which rustc resolves while it ignores the import being
resolved: in a name space where exactly one place of the cycle, read by one of its
imports as the last segment of its path, is one a glob of its module would answer, no
import of the cycle waits anywhere else, and each renamed one is one rustc reads
without itself (it bound its name in another name space, and no glob of its module
brings the name in there), and leaving the place to the glob ends every loop of
imports in the cycle, that place is left to the glob for the paths that read it, and
not for the globs of other modules, and the cycle resolves through it. An import
that keeps the place then binds only what keeps every item that a path which read the
place found the glob answering there, as that path's module can access it; one that
would take such an item away, or whose path may still reach something there, is
never decided, as rustc never determines it, and a path through its name resolves
`Cyclic` (`e { pub use crate::d::T; }` beside `d { pub
use crate::e::T; pub use crate::b::*; }` gives `demo::b::T` for both, and a trait impl
through `e::T` is keyed there); a cycle where no glob answers, whose imports all bound
their names in another name space, binds nothing in this one; any other cycle, and
every import that waits on one, never settles, so the name it binds resolves `Cyclic`
and no glob or crate answers for it: `mod a { pub use crate::b::X; pub use
crate::c::*; }` beside `mod b { pub use crate::a::X; pub use crate::d::*; }` gives
`Cyclic` for `demo::a::X` and `demo::b::X`, as does a cycle with two globs of one
module or a renamed import that has bound nothing, where rustc cannot determine the
imports. Globs are not such a cycle when the path of each is held only at its
first segment, read where the path starts, which only the globs of that module could
bring in: none can resolve before another does, so each is settled bringing in
nothing, and a missing name of the module is `NotFound` (`pub use std::io::*;` beside
`pub use std::fmt::*;`, or `serde::*` beside `tokio::*`). The first segment of a glob's
path that names a crate of the extern prelude waits on no glob of its module, as for a
named import, so `std::io::*` beside a glob of a local module does not wait on it. A path through a name that an import bound in one name space while a cycle
leaves it undecided in another resolves `Cyclic` in every name space: such a cycle,
read with only what each import can access, is one that rustc does not break either, so
it refuses the crate, failing an import of the cycle (E0432), and when the cycle runs
through globs it reports an import that waits on it as one it cannot determine, whatever
it bound. A decision binds
nothing that a path read before, so the result does not depend on the order of the
declarations. Neither does the work grow with a chain or a cycle: each round decides
every import that no cycle holds, in waves that each run the queue only for what the
wave woke, so a chain of 20,000 imports that bind nothing beside globs is decided in
one round and confirmed in a second. Each round that changes something decides for good at least one
more of the at most eleven things there are to decide for each import, so the rounds
end; that is checked in release builds too, and a round that decides nothing more is an
`Internal` load error. Of two routes to one item the one that uses no
glob decides `via_glob`.

`resolve(path)` takes a canonical path such as `demo::inner::go` (a segment may be
written as a raw identifier: `r#match` and `match` are the same name) and tries the type,
value and macro name spaces in that order; `resolve_in(path, namespace)` takes one. A
path may end at a method: `demo::Thing::new`. A canonical path starts at a crate path
and names each item by its name, so a segment that is a path keyword, `crate`, `self`,
`super` or `Self`, in any position and also written as a raw identifier, makes the text
`Invalid`: `demo::crate::a::x`, `demo::self::a::x` and `demo::super::x` are `Invalid`
beside `demo::a::x`. Each segment is read once and the crate path is found by
prefixes that grow a segment at a time, so the cost of a path is linear in its length:
in a release build a path of 50,000 segments is answered in milliseconds (under 100
ms; a debug build is several times slower). The result is a `Resolution`: the
canonical `path` of the definition, the `namespace`, the `sites` (non-empty, sorted by
path, file and position), `via_glob` (true when a glob import carried the path to the
symbol), and the `chain` of locations the lookup passed through from the requested path to the
definition. The same path, name space and kind defined more than once is one symbol with
several sites, as `#[cfg(unix)] fn a()` beside `#[cfg(windows)] fn a()`. The failures are
typed: `NotFound` when the workspace defines nothing at the path (an item of a
dependency, an item that a macro generates, and a misspelling all end here),
`Ambiguous(ids)` only for a real conflict, which is two globs that bring in different
items, two imports of one name that reach different items, two canonical targets, a
path through a crate name that stands for several libraries, or a path through an import
whose first segment is ambiguous, `Cyclic` when the imports on the way depend on each
other or wait on imports that do (an import never depends on itself), `UnresolvedSelf` when the
workspace defines nothing at the path and the path goes through the self path of an
inherent impl the index cannot see, `UnresolvedGlob` when a lookup on the way reads a
glob import whose path names nothing in the workspace past its first segment, and
`Invalid` when the text is not a path. Such a glob (`use self::missing::*;`, `use
crate::a::gone::*;`, or the glob of a module that a macro generates) does not fail the
load, which brings in nothing through it, but what it would bring in is not known, so
every lookup that reads it fails, naming the path asked for, the glob's written path and
the module that declares it: a lookup of a name that the module defines or imports
explicitly in the name space looked up does not read the module's globs, and one that
reads a glob that resolves reads the globs of the modules it globs in turn, so a sibling
path that reads none of them resolves, while a name that a glob that resolves brings in
beside such a glob fails, as the broken glob could make it ambiguous. Which globs
name nothing is worked out once for the index, by the first lookup that needs it, so a
workspace without such a glob pays nothing more for a lookup. A name space whose lookup reads the glob fails
though another name space has the name (the glob could bring in a type of that name), and a path through an
import whose own path reads it fails the same way. A glob whose first segment names
nothing in the workspace is a dependency the index cannot look into, and a lookup that
reads it stays `NotFound`; so does one that reads a glob whose path names nothing only
where an import binds the name or a glob of that kind could bring it in (`use
std::io::*;` beside `use self::missing::*;` could bring in `missing`). When a path
fails for more than one reason, `Cyclic` comes first, then `Ambiguous`, then
`UnresolvedGlob`, then `NotFound`: a path that reads a glob that names nothing and a
cycle of imports is `Cyclic`. Each
message shows the path, the package and the glob's path with every control character
and bidirectional override escaped as `char::escape_debug` writes it. A symbol
is one path and kind in one target's crate: the items of two targets at one path are
two symbols and `Ambiguous`, whatever the order of their files, while the members of
trait impls on types that are not local share their trait's key across crates. The
crate path of a test, bench, example or binary target begins with the
package name and a name such as `test_crate`, so a first-party module with that name
(`mod test_crate;` in the library) and a test target share a prefix: a path that both
define (`demo::test_crate::it` when the module has `it`) is `Ambiguous`, naming the
library's item and the test target's, and a path that only one of them defines resolves
to it. A path that starts with a crate path that targets of several packages share (two
libraries named `shared`) is `Ambiguous`, naming the crate root of each, whatever
follows; `resolve_in_package(package, path)` takes the Cargo package name separately and
resolves among that package's crates only, so `resolve_in_package("p-two", "shared::X")`
names the `X` of `p-two`, and a name that is no package of the workspace is
`UnknownPackage`. A symbol that a
macro generates is not indexed, and the documentation of the capabilities that rely on
the index says so.

**Exported items.** `Workspace::exported_items(target)` lists the public surface of
a library or binary target: the items another crate can name through its crate
root. It returns `Exports`, whose `items` are `ExportedItem`s, each with the `path`
it is exported under, starting with the target's crate path (`demo::run` for `pub
use inner::go as run;`), the defining `item`, so that a re-export under another
name maps back to its definition, its `kind`, `file` and `span`, and `doc_hidden`;
and whose `reexports` are the `Reexport`s below. The surface is read from the
settled scope tables, breadth first from the crate root, through each module that a
public binding of its `mod` item reaches. A binding is public when a route of `pub`
declarations leads from it back to its definition: each `use` declaration on the
way, named, renamed or glob, is `pub` and not test code, and the definition is `pub`
(a `macro_rules!` macro is `#[macro_export]`) and not test code. Every glob that
brings the item in is searched for such a route, not only the route the tables
keep. So a `pub` item of a private module is exported only through a `pub`
re-export; a `pub(crate)`, `pub(super)`, `pub(self)`, `pub(in path)` or private item
never is; and neither is an item that a re-export which is not `pub` carries on the
way, as rustc brings a name in through a glob no more visible than the binding it
reads and refuses a named re-export of a binding less visible than itself. Test code
(an item, a module or a `use` declaration with the `cfg(test)` tag or in a file
reached only through test code, and a `#[test]` function) is never exported. A
module that several public paths reach is listed under each and read once, under
the first in breadth-first order (the shortest, then the first in the order of its
segments), a path that `#[doc(hidden)]` does not hide coming before one that it
does; a cycle of globs (`pub mod a { pub use super::*; }` beside `pub use a::*;`)
reads each module once. An item that a name binds in two name spaces, as a unit or
tuple struct, is listed once.

The members of an exported struct, enum, union or trait follow it under its
exported path (`demo::Thing::new`): every member of a trait, its methods,
associated constants and associated types, and each `pub` member of an inherent
impl of a type, wherever the impl is written, each when it is not test code. The
members of trait impls are not listed: they carry no visibility of their own, and
their interface is the trait's. Enum variants and struct fields are not items of
the index and not items of the surface, and a glob of an enum brings in nothing. A
type alias is listed alone, and the members of the type it names are listed where
that type is exported. The members of an inherent impl whose self type the index
cannot see (`unresolved_impls`) are not listed, and an item that a macro generates
is not indexed and is not listed.

A public re-export that names nothing the target defines is a `Reexport`, with the
exported `path` (`<module>::*` for a glob), the path the declaration names as
`written`, the `item` when the index sees one (an item of another crate of the
workspace, or that crate's root), and the `file` and `line` of the declaration: a
`pub use` or `pub extern crate` of another crate of the workspace or of one of its
items, of a crate outside the workspace or one of its items (`pub use
serde::Serialize;`, `pub use serde;`, `pub use std::fmt::*;`, each with no `item`),
of an enum variant (`pub use p::F::G;`, as variants are not items of the index), and
of a path the index cannot follow, such as a module that a macro generates. A `pub`
glob carries such a re-export of each module its public routes reach, through
`pub` globs that are not test code, into the module that reads it, under that
module's path (`demo::Debug` for `pub use a::*;` where a private `a` has `pub use
std::fmt::Debug;`, and `demo::*` where it has `pub use std::collections::*;`),
unless a module on every such route, the reader included, binds the name itself, in
any name space, as the index cannot tell the name space of an item it does not see;
such a re-export is hidden when only routes that leave the reader by a
`#[doc(hidden)]` glob carry it, as an item is: a hidden glob further on the way
hides nothing, and one whose shown route a module on the way shadows and whose
route by a hidden glob of the reader is free is hidden. `pub extern crate self as name;`
names the target itself and is one of its items. An `Item` carries `macro_export`,
true for a `macro_rules!` macro with `#[macro_export]`, which the listing reads.

`#[doc(hidden)]` hides a path when it is on the item, on a module of the path (on
its declaration, or as `#![doc(hidden)]` in the module's file), on the `use`
declaration that exports it (on each one, where several do), or on the type or
trait whose member it is. One rule decides which declarations export a name, for an
item and a carried re-export alike: a named `pub use` of the reader that binds it,
or a `pub` glob of the reader whose target carries the name out of the crate, that
is, binds it with a route of `pub` declarations of its own; a glob whose target
brings the name in only at `pub(crate)`, or keeps it behind a binding that is not
`pub` or an import that shadows it, exports nothing, whatever the index's tables
hold for it. So `mod c { pub struct Cell; } mod m1 { pub(crate) use crate::c::*; }
pub use m1::*; #[doc(hidden)] pub use c::*;` exports `demo::Cell` hidden, as it
does with `mod m0 { pub use crate::c::*; use std::rc::Rc as Cell; }` in place of
`m1`, and with `pub use std::cell::Cell;` in `c`, though rustdoc documents the
item of the first. `exported_items` leaves hidden items and re-exports out;
`exported_items_with(target, &options)` with `ExportOptions::include_hidden` lists
them with `doc_hidden` set, and only then reads a module under a hidden path. A
binary's surface is read the same way from its crate root: no other crate can name
it, and it is what the binary's root exposes.

Items are sorted by path, then in the order of `items()` of the defining item, and
re-exports by path, file and line, so two loads of one tree give equal surfaces,
whatever the order of the declarations. A failure is an `ExportError`:
`NotLibraryOrBinary` for a test, bench, example or build-script target; `Resolve`,
with the error that `resolve` gives for the path, when a public name is ambiguous
(two globs bring in two items of the index; an item and a carried re-export of a
crate outside the workspace under one name are both listed, as the index cannot
tell whether they collide, so `mod a { pub struct Rc; }` and `mod b { pub use
std::rc::Rc; }` globbed by the root give the item `demo::Rc` and a `Reexport`
`demo::Rc`), goes through an import that never settles (a cycle
of imports), or reads a glob whose path names nothing in the workspace, as a `pub`
glob of such a path does even when nothing reads it; `Budget`; and `CarryBudget`,
with the exported path of a carried re-export, when deciding whether the globs
carry it passes the item-bytes budget (below), its `Budget` load error naming the
file and line of the carried `use` declaration. Each binding the
listing passes is checked as `resolve` checks the last segment of a path, and every
module of a path is reached through such a binding, so a chain of re-exports is
checked once for each segment. What the listing keeps (its entries, the path of
each module it reads, the names it reads there, the globs of the modules it reads
with the views of them that hold each name, and the decision for each binding it
searched a route for) is charged to the
item-bytes budget after what the load spent, by the layout rules above, before it
is kept, so the index and one listing stay within `max_item_bytes`; a listing past
it is a `Budget` load error of `ItemBytes` that names the file and line of the item
whose entry passed it. The search for one binding's route and the items one name
binds are not charged: each is bounded by the bindings the load charged, and is
dropped before the next is made. The search keeps a stack of its own, and once it
finds a route, or finds that there is none, every binding it passed is decided for
the rest of the listing, so a chain of re-exports is followed once for all of its
names: 20,000 chained re-exports, and a chain of 20,000 `pub` globs, list on a thread
of 2 MiB. The globs of a module that bring in a name are found by the name, so a
module of 3000 `pub` globs of one name each lists in about 15 ms in a release
build. The modules a reader's routes reach are walked once, into a graph, for the
re-exports they carry. A carried name is decided first over the routes that
`#[doc(hidden)]` does not hide and, when they do not carry it, over every route,
which carries it hidden; each decision is made from the modules of that walk
that bind it, other than the carrying module: a binder with no public routes of
its own is never on the way, so with none left the name is carried; the reader,
or a binder that dominates the carrying module in the graph (is on every route
to it), keeps it out; and a single binder that does not dominate it leaves a free
route. The tree of dominators of each set of routes is made once for the reader, by
the iterative
algorithm of Cooper, Harvey and Kennedy, the first time a name needs it. Only a
name that several binders with routes of their own keep, none of them dominating,
is decided by a search from the reader past its binders, once for the name, which
keeps the answer for each module that carries it and nothing of what it walked.
Each step of the walk, of the tree and of the searches, a module or a route read,
is charged a binding to the item-bytes budget, with what they hold, so a listing
whose searches pass it is a `CarryBudget` error and never runs unbounded: a chain of
5000 globs to a module that carries 2000 names, each of which two modules beside
the chain bind, is refused in about 0.4 s in a release build, beside a chain of 50
globs to 200 such names, which lists. Whether a route avoids a set of several modules
needs that search, as no structure computed once for the reader answers it for
every set the way the tree answers it for one module. 3000 `pub` globs that each
carry a re-export of their own name, beside 3000 globs of modules that each bind
that name privately, list in about 90 ms in a release build and are charged under
16 MiB beyond the load, and 10,000 such in about 0.26 s, where walking the reach
again for each name took 12.8 s at 3000.

**API surface.** The data structs a caller reads, `Package`, `Target`, `SourceFile`,
`Reach`, `Entry` and `SpanFact`, are `#[non_exhaustive]` like `Item`, so adding a field
is not a breaking change: a pattern over one names the fields it reads and ends in
`..`, and a `Tree` backend outside the crate builds an `Entry` with `Entry::new`. The
per-file facts the index is built from are not public. `LoadOptions` is `Clone` and
not `Copy`, so adding an option that is not `Copy` is not a breaking change. `Exports`,
`ExportedItem`, `Reexport`, `ExportOptions` and `ExportError` are `#[non_exhaustive]`
as well, so a caller builds `ExportOptions` from `ExportOptions::default()`.

## Usage

Load a workspace from a real directory and ask where a path is defined. The block
needs a real tree, so it is compiled and not run:

```rust,no_run
use raf_source::{DiskTree, Workspace};

let ws = Workspace::load(&DiskTree::new(".")?, &["crates"])?;
let resolution = ws.resolve("raf_okf::Document")?;
for site in &resolution.sites {
    let item = ws.item(*site).expect("a site is an item of this workspace");
    println!("{} at {}:{}", item.path, item.file, item.name_line);
}
for target in ws.targets().filter(|t| !t.shipping) {
    println!("{} is a non-shipping target", target.name);
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

A line that starts with `# ` is hidden by rustdoc and is there so that the block
compiles.

List the public surface of a library or a binary with `ws.exported_items(id)`, for
an `id` of `ws.target_ids()`. Each `ExportedItem` of `items` maps its exported
`path` back to its definition through `ws.item(export.item)`, and each `Reexport`
of `reexports` gives what it re-exports as `written`. `exported_items_with(id,
&options)`, with `options.include_hidden` set on `ExportOptions::default()`, lists
what `#[doc(hidden)]` hides as well. The block lists the surface of each library of
a real tree, so it is compiled and not run:

```rust,no_run
use raf_source::{DiskTree, TargetKind, Workspace};

let ws = Workspace::load(&DiskTree::new(".")?, &["crates"])?;
for id in ws.target_ids() {
    let target = ws.target(id).expect("an id of this workspace");
    if target.kind != TargetKind::Lib {
        continue;
    }
    let exports = ws.exported_items(id)?;
    for export in &exports.items {
        let item = ws.item(export.item).expect("an item of this workspace");
        println!("{} is {} in {}", export.path, item.path, export.file);
    }
    for reexport in &exports.reexports {
        println!("{} re-exports {}", reexport.path, reexport.written);
    }
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Examples

With `MemoryTree` a fixture needs no disk:

```rust
use raf_source::{MemoryTree, Workspace};

let tree = MemoryTree::from([
    ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ("src/lib.rs", "mod inner;\npub use inner::go as run;\n"),
    ("src/inner.rs", "pub fn go() {}\n"),
]);
let ws = Workspace::load(&tree, &["."])?;
assert_eq!(ws.resolve("demo::run")?.path, "demo::inner::go");
# Ok::<(), Box<dyn std::error::Error>>(())
```

A `pub` item of a private module is exported only through a `pub` re-export:

```rust
use raf_source::{MemoryTree, Workspace};

let tree = MemoryTree::from([
    ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ("src/lib.rs", "mod inner;\npub use inner::go as run;\npub(crate) fn helper() {}\n"),
    ("src/inner.rs", "pub fn go() {}\npub fn unused() {}\n"),
]);
let ws = Workspace::load(&tree, &["."])?;
let lib = ws.target_ids().next().expect("the library");
let exports = ws.exported_items(lib)?;
assert_eq!(exports.items.len(), 1);
assert_eq!(exports.items[0].path, "demo::run");
assert_eq!(
    ws.item(exports.items[0].item).map(|item| item.path.as_str()),
    Some("demo::inner::go")
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

A library whose root is `mod inner; pub use inner::go as run; pub(crate) fn
helper() {}`, beside an `inner.rs` of `pub fn go() {} pub fn unused() {}`, exports
one item: `demo::run`, whose defining item is `demo::inner::go`. `helper` is
`pub(crate)`, and `unused` is a `pub` item of a private module that no re-export
reaches, so neither is listed. Adding `pub use inner::*;` exports `demo::go` and
`demo::unused` as well, and making the module `pub mod inner;` exports
`demo::inner`, `demo::inner::go` and `demo::inner::unused`.

## Constraints

- Parsing is syntactic. The crate does not expand macros, evaluate `cfg` other than
  the `test` tag, or run type inference. An item is indexed as written, so a function
  with no body or a type with no definition, which rustc accepts only as the input of
  an attribute macro or under a false `cfg`, is indexed under its written name and
  kind whatever the macro makes of it.
- Every opened path must lead inside the canonical workspace root, as the disk
  resolves it (canonicalised on Linux and Windows, walked beneath the root on other
  Unix systems), and every file read must be a regular file; the same rule bounds
  explicit `Cargo.toml` target paths.
- A file larger than `max_file_bytes` (default 8 MiB), or one that exceeds a
  budget of the pre-scan (128 delimiter levels, 64 nested `<`, 256 prefix
  operators, 128 closure bars, 256 `else if` links, 1927 units of parser recursion,
  2000000 tokens by default), or a
  module walk that passes 128 nested modules or 262144 module walks in one load, or a
  directory
  more than 128 levels below its root, or a load that passes its total, item, fact or
  glob bytes (256 MiB, 512 MiB, 256 MiB and 512 MiB by default), is refused with a
  typed error.
  Parsing runs on a dedicated 64 MiB stack, of which an admitted file takes at most
  half in a debug build. A residual overflow aborts the run and fails the gate; it is
  never a pass.
- The work queue and the graph of undecided imports that settling the scope tables
  builds are not budgeted: they are dropped once the tables are settled, and they grow
  with the undecided imports times the places their paths read through globs, while
  the time to decide them grows faster (see Work and memory bounds).
- Resolution covers items the source defines. Items from dependencies are outside the
  index and resolve to a not-found failure.
- An item that syn returns as tokens is indexed only in the three shapes the Contract
  names: a `use` group with a member that starts with `::`, a function with no body and
  a type with no definition. Any other, such as `const C: u8;` written as the input of an
  attribute macro, fails the whole load with `UnsupportedItem`, naming the file and line.
- Two visibility rules of rustc are not modelled. An explicit `pub use` of a binding
  that its module can access only through a private glob binds the name as visible as
  the import, where rustc keeps it no more visible than the binding it re-exports, so a
  module that globs it can read a name that rustc hides from it. And a name that the
  globs of a module bring in ambiguously, one of the items through a glob that a
  reading module cannot access, is the other item for that module, where rustc refuses
  the path, usually as an ambiguity (E0659) and sometimes as a name that is not there
  (E0573).
- `workspace.members` and `workspace.exclude` hold at most 4096 entries each.
- The text of an error shows input-derived text with control characters escaped by
  `char::escape_debug`, which also doubles a backslash, so it is safe to print to a
  terminal; the fields of the error keep the raw text.
- The index is built once per `Workspace::load`; callers reload to see changed files.
- Loading is serial; requiring `Tree: Sync` for a parallel load would be a breaking
  change to `Workspace::load`.
- `raf_source::syn` is re-exported so consumers use the syn version that produced the
  facts; a syn major bump is a breaking change of raf-source.
- Output order is sorted by path, so two loads of the same tree give equal results.
- The public surface is the visibility rustc gives the items of a crate, read
  syntactically: no `cfg` other than the `test` tag is evaluated, so an item under
  any other `cfg` is listed whether or not a build has it, and nothing a macro
  generates is listed. `#[doc(hidden)]` and `#[macro_export]` are read as the index
  reads them, unconditional and, for `doc`, with `hidden` alone; one that a
  `cfg_attr` applies is not read. Of the two visibility rules above that the index
  does not model, the first does not reach the surface, whose routes require every
  `use` declaration on the way to be `pub`; the second does: a name that the globs
  of a module bring in ambiguously, one of the items through a glob that a module
  which globs it cannot access, is listed under that module with the other item,
  where rustc refuses the path. A name that two globs of a listed module bring in
  as two items fails the whole listing with `Ambiguous`, also where rustc accepts
  the crate with only the `ambiguous_glob_reexports` warning (two `pub` globs, one
  of whose items is `pub(crate)`, so that another crate can name the other): the
  listing refuses the name rather than choose one, as `resolve` does. A re-export
  of an item of a crate outside the workspace that a `pub` glob carries is dropped
  where the reader or a module on the way binds its name in any name space, as the
  index cannot tell the name space of an item it does not see: beside `fn Rc() {}`,
  a carried `pub use std::rc::Rc;` is not listed, though rustc exports the type.
  A listing whose carried names need searches past several binders on long routes
  can pass the item-bytes budget, which charges their work, and is then refused
  with `CarryBudget` though rustc accepts the crate. The `doc_hidden` flag follows
  the export rule of the Contract, not rustdoc: an item that a `pub` glob brings in
  only through a `pub(crate)` glob, beside a hidden glob that exports it, is
  flagged hidden, where rustdoc documents it.
- The proofs that need a Windows host, the final path of a handle and a `DiskTree`
  refusing a reserved device name or a name ending in a dot or a space, are compiled
  on every host by `make cross-check` and run only on Windows, in the Windows CI job;
  the bound tests are those every Linux host runs: the handle-path verifier, the
  Linux handle verifier on real descriptors, the Windows-name predicate and a tree
  that holds those names loading off Windows.

## Explanation

A census that compares function bodies, and a workplan that checks that a documented
symbol exists, both fail in the same way when they trust text: a re-export makes the
public path differ from the defining path, and a module declared with `#[path]` is
invisible to a directory walk. Both tools need the compiler's view of the module
tree, and `syn` gives a close enough one without the cost of invoking the compiler.

The `Tree` trait is the seam that keeps the crate testable. Acceptance fixtures
for both consumers are repositories built in memory, and a disk-backed run uses the
same code path, so a fixture that passes says something about the real workspace.

Keeping test and example targets in the index, tagged non-shipping, lets one
consumer ignore them and another bind to them. Workplan bindings name tests and
examples, so those must resolve; the census must not count test helpers as shipped
duplicates.

## Verification

- A `MemoryTree` fixture per target kind (library, binary, test, bench, example and
  build script) with default and with explicit paths, asserting the kind, the Cargo
  name, the shipping flag, the crate path and that each target's root module is
  recorded under its crate path. A manifest whose tables set `harness`,
  `required-features`, `test` (`test = false` among them) and `doctest` away from
  cargo's defaults, beside targets of each kind without a table, which take the
  defaults; a setting of the wrong type in each kind of table is a `Manifest` error
  naming its key.
- A `compile_fail` doctest on each of `Package`, `Target`, `SourceFile`, `Reach`,
  `Entry` and `SpanFact` destructures it with every field and no `..` and must fail
  with E0638, the error of a pattern over a non-exhaustive struct. The pattern for
  `SourceFile` omits its crate-private `facts` field, so it is refused for that field
  too, and only the seal makes it E0638. A `required-features` key on the library is a
  manifest error, as cargo refuses it there.
- Discovery fixtures: a virtual manifest contributes nothing; a manifest with a
  trailing comment and single-quoted values is honoured; no roots is an error; a
  package nested below a package without `[workspace]` is not found beside a root
  package with `[workspace]` and a member below it; `target` and dot-directories are
  skipped beside a sibling; roots exclude what they do not name; the `auto*` flags and
  explicit tables; a library and a binary with one name; two packages with one name,
  two targets with one root and two targets of one package with one crate path, each
  beside a distinct-name neighbour; two packages whose libraries and binaries share
  crate paths (`x-y` and `x_y`), which load, are `Ambiguous` by `resolve` and resolve
  by `resolve_in_package`; a missing target file, a table without a name and a
  target path that leaves the tree, each naming the manifest key; a library root over
  `max_file_bytes` reached by package discovery, by the `lib` table and by `lib.path`,
  only the last naming `lib.path`; a dependency `package` that is not a string, in a
  package table and in `[workspace.dependencies]`, naming its key beside a string; a manifest of
  exactly 8 MiB loads and one byte more is refused naming the file; inline tables
  nested 1000 levels are a typed error on the caller's stack and 8 levels load.
- Mod-tree fixtures: `name.rs`, `name/mod.rs`, both present (an error naming both),
  inline and nested modules, `#[path]` in a `mod.rs` file, in a non-`mod.rs` file, inside
  inline modules in each, on an inline module in each, a `#[path]` file's children,
  `cfg_attr` alternatives with their predicates (nested ones flattened), two
  unconditional `path` attributes, raw identifiers, a cycle naming both files, a file
  reached by several paths and by two targets listed once, an unresolved `mod` naming
  the file and line, the module-depth budget at 128 and 129, a chain of files that each
  declare the next twice (2^31 paths) refused by the module-path budget at its limit
  of 262,144, with each source file opened and read once through a counting tree that
  refuses more than 200 calls, beside a small diamond; 40 nested inline modules with
  two `cfg_attr` path alternatives each, refused by the same budget with no tree call
  beyond those of a shallow load; and a first-party
  `mod test_crate;` beside a test target.
- Placement fixtures, each asserting every item path: a child file that sorts before
  the file that declares it (`a/mod.rs` with `a/b.rs`, and `a.rs` with `a/b.rs` below a
  directory module that sorts before the crate root); the children `a`, `ga` and `z` of
  a `#[path = "gen/generated.rs"]` file; and `tests/common/mod.rs` with
  `tests/common/a.rs` reached from `tests/zz.rs`. `super::super` from such a child
  reaches the crate root, and `demo::a::in_b` and `demo::a::b::in_a` are `NotFound`
  beside `demo::a::b::in_b`. In this repository `raf_okf::body::blocks::FenceBlock`
  resolves and `raf_okf::body::FenceBlock` is `NotFound`, and the `raf-workplan`
  binding fixtures reject a binding to the same misplaced shape. `raf_source::syn` and
  `raf_source::syn::File`, through `pub use syn;`, are `NotFound` beside
  `raf_source::DiskTree`, which resolves.
- Path-prefix fixtures: `items_under("demo::a")` beside the modules `demo::ab` and
  `demo::a1` returns `demo::a` and its descendants only, an empty prefix returns every
  item, and a file that the targets of two packages reach has the package of the first
  target in target order, which is the first by manifest path and not by name.
- Binding fixtures in `raf-workplan`: a `tests` binding that names an item that is
  not a function marked `#[test]` is a violation, and so is a `code` binding that
  names test code or an item of a non-shipping target. The repository's own bindings
  satisfy both rules.
- `#[path]` fixtures: a redirect inside the root resolves; `..` out of the root and
  an absolute path outside it both fail naming the module and the attribute, and a
  recording tree shows that the outside path is never read.
- `cfg(test)` fixtures: the predicate matrix, `#[cfg(test)] mod t;` tagging every item
  reached only through it, `#[test]` functions, a `test` predicate on a `path`, and
  unreadable or misshapen attributes; a `path = concat!(..)`, plain and under
  `cfg_attr`, is a `CfgParse` error that names the attribute's line and is reached by
  the `path` attribute of its module, beside a literal path that loads.
- Bound fixtures: a file over `max_file_bytes` fails with an error that names the
  file, a lowered `max_file_bytes` is honoured, and a file of 8 MiB parses at the
  default. A file at the delimiter limit (128 levels) parses and one at 129 levels
  fails. Each of these shapes just over its budget, and the shape sizes below, fails
  with a typed error that names the file, and runs in debug and in release on the
  default 2 MiB test-thread stack of the caller, so the dedicated stack and the
  pre-scan are what keep the process alive:
  - 500 nested `A<A<...>>` generics (delimiter depth 0);
  - `!` repeated 4000 times before an expression;
  - `&` repeated 20000 times before a type;
  - `||` repeated 10000 times as a closure chain;
  - an `else if` chain of 20000 links.

  A file at each budget (64 nested `<`, 256 `!`, 128 closure bars, 256 `else if`
  links) parses on the same stack.
- Token fixtures, at a limit lowered to 100000 for speed: a file of exactly the
  limit loads and one more token is refused naming its line; `a<b; ` repeated,
  statements naming a type nested 64 generic arguments deep, a run of attributes and
  the costliest shape, one body of a power of two and one more empty statements,
  each load at the limit and are refused one token over; a doc comment counts as six
  or seven tokens and other comments as none. The default limit is 2000000,
  `parse_source` refuses one token more than it before parsing, and 320000 tokens
  at 1664 bytes each stay below 512 MiB.
- Work and memory fixtures, each at the limit, where the load succeeds, and one byte
  over, where it is a `Budget` error that names the budget and the file: the bytes
  read (a manifest counts), the index records (a long module name charged in every
  item path, a file reached by a hundred module paths charged once per path, and a
  long aliased path and a long `extern crate` name in a file reached by 256 module
  paths, each charged once and not once per path, and a chain of files that each
  declare the next twice and nested inline modules each charged at least the bytes of
  index they keep),
  the facts, over several files and for each large payload: a 1 MiB doc comment, a 1
  MiB written self type and a `use` group of long names, each refused at its own line
  below its size, and the glob tables, refused at the line of a glob, while a load
  without a glob charges none. A thousand names that four hundred modules glob cost
  each importer under a kilobyte more than one importer, a chain of 2000 globs is
  charged less than 2.1 times one of 1000, 200 modules that glob the crate root, which
  globs each, are refused at 4 MiB naming a glob beside 20 that load, and a generated
  binding crate of 200 child modules is charged at least the bytes of glob tables it
  keeps. A module that globs one module of 2000 names 400 times over, globbed by
  another, is charged under a kilobyte for each more glob, in either order of
  declaration, and is refused by the glob budget one byte below what it is charged. `usage()` reports the spent bytes, the most tokens, the highest recursion
  value and the deepest directory, and an item record is at most 256 bytes.
- Unit fixtures of the structures the bounds rely on: a column of a short line, a
  long line and a long line of two-byte characters, before, on and after an index
  mark, turns into its byte offset, and a column or line past the text is refused;
  the bindings of one name stay sorted, keep the better route of a repeated item and
  take only the room the first binding needs.
- Stack fixtures, each on a small thread in debug and in release: a `MemoryTree` chain
  of 20000 directories and a `DiskTree` chain of 2100, on 2 MiB, are refused with the
  directory-depth budget naming the first directory past it, beside chains at the limit
  that load; a `workspace.members` pattern at 128 levels expands and one at 129 is
  refused; 128 nested modules, inline and in files, load on 512 KiB, and 129 are
  refused typed.
- Manifest caps and pattern cost, counted through a tree that records its calls: 4096
  `exclude` entries load with the tree calls of a load with no excludes, and 4097 are
  a `Manifest` error naming `workspace.exclude` with the tree calls of a manifest refused
  at its first read; 4096 `members` entries load with one `entry` call for each glob
  beyond the single-member load and no `list` call, and 4097 are a `Manifest` error
  naming `workspace.members`, refused with those same calls; the 880 KB manifest of
  20000 members and 200000 duplicate excludes is refused before any pattern is expanded;
  4096 duplicates of each kind load with the tree calls of one of each; 4095 distinct
  members against 4096 distinct excludes load with the tree calls of the same members
  against the one exclude that matches. Counted in the matcher, the 4095 distinct
  members and the directory they find are tested against the wildcard excludes at most
  once each, and every wildcard that has more segments than the path is skipped without
  a step, so the segment matcher runs at most 21 steps for each test; 4096 duplicate
  excludes are one wildcard tested twice. A segment of 24 `*a` pieces and a `b`, matched
  against 40 characters of `a`, takes at most the pieces plus one times the characters
  plus two steps, as do 8 and 16 such pieces. A star-heavy member pattern that matches
  no directory leaves the member alone. Literal, wildcard and nested excludes still
  remove their members.
- Error text: the `Display` of `LoadError`, `LoadErrorKind`, `Origin`, `TargetProblem`,
  `TreeError` and `PathError` escapes every control character of a path, module name, predicate,
  key or message with `char::escape_debug` (quotes stay as written), so a `#[path]` with
  terminal escape sequences, a NUL or a line break shows as text. One test builds every
  variant with such text; two load a hostile `#[path]` and a hostile `members` entry.
- Identity fixtures: `identity` gives one value for a file, a symlink to it, a hard
  link and a path through a directory symlink, and refuses what `read` refuses;
  `read_unseen` gives the identity and the text of one open, leaves a seen file
  unread and unrefused for its size, and refuses an unseen one over the cap; a file
  reached as `z/../b.rs` and as `b.rs` is read once and listed as `src/b.rs`; on disk,
  a file reached through a directory symlink is one file under the spelling that
  sorts first, a module that reaches its own file through such a link is a cycle, and
  a tree of two links back to `src` and eighteen levels of files, which spells its
  last file 2^17 ways, reads every file once and is refused by the item-bytes budget.
- Positions: twenty thousand items on one line of two-byte characters get exact byte
  offsets, and extracting the facts of that line walks at most the characters of the
  line plus one index interval for each position, so the line is indexed once however
  many items it holds.
- Composite, uncounted-shape and chain fixtures: `!` repeated 256 times before `(`
  repeated across 127 levels; `&` repeated 256 times before `(` across 127 levels, in
  a type; `&&` repeated 256 times before `(` across 127 levels; `A<` followed by 256
  `&`, repeated across 64 levels; `||` repeated 128 times before `(` across 127
  levels; 20000 repetitions each of `return `, `&mut `, `*const `, `fn() -> `, `a = `
  and `move || `; the chains `x?`, `x.a`, `x()`, `x[0]`, `+x` and `*x` repeated 130000
  times, `x as T` 78000 times, `.await` 65000 times and `.m()` 97000 times; and 20000
  repetitions each of `async || `, `async move || `, `const || `, `|| return `,
  `&raw mut *`, `break `, `yield `, `become `, `a += `, `impl Fn() -> ` as a return
  type, `extern "C" fn() -> `, `unsafe fn() -> `, `for<'a> fn() -> ` and `*mut `, and
  of `box ` in a pattern. A harness loads each as the root file of a library through
  `Workspace::load` in a child process, in debug and in release, so the parse, the
  reading of the facts and the drop all run as in any load, and asserts that the
  pre-scan refuses it with a typed error that names the
  file and the budget: the recursion budget, and the closure bar budget for the
  `move || ` chain. An abort, an exit 0 or any other end fails the fixture.
- Shape families, each loaded through `Workspace::load` in a child process in debug
  and in release: nesting that ends a list or a statement at every level (closures
  with parameters and statement bodies, tuples, arrays, struct literals, statement
  blocks, match arms, call arguments, `if let` and `else if` chains, labelled and
  `async` blocks), groups nested in a macro call, in a `macro_rules!` body and in
  attribute arguments, `match` alternatives that hold deep patterns, `where` bounds,
  `impl Trait` arguments and nested `for<'a>`, nested types (generic arguments,
  turbofish, reference and pointer chains, `fn` pointers, `dyn Fn`, tuple and array
  types), expression chains (`as`, unary operators, `?`, ranges, method calls),
  lexical edge cases (raw strings, characters beside lifetimes, nested block
  comments, raw identifiers, CRLF lines, doc comments on nested statements and long
  runs of them on nested modules, a leading `#!` line) and nested items (inline modules, functions
  in blocks, impls in functions). For each, the largest size the budgets admit loads
  with its index, and one size more is refused naming the budget that meets it
  first: the recursion budget, or the delimiter budget for the families that nest
  only through groups.
- Recursion fixtures, in process: for every counted operator, number, keyword, label,
  `use` separator and opening delimiter, a file that holds it and reaches exactly the
  limit parses and the same file with one `return` more is refused with the recursion
  budget; the limit is half of the parse stack over the measured cost of a unit. Each
  place where a level's count returns to zero (`;`, `,` between elements, `=>`, an
  identifier after a block) admits two constructs that together pass the limit, beside
  the same file with one more unit in either, which is refused; and a `,` inside
  generic arguments or a closure head, and an `else`, `in` or `as` after a `}`, keep the
  count, each at the limit and one over, and so do the commas of a `where` clause. A
  macro call counts its `!` and, inside its delimiters, only the levels its groups
  open; a `!` after a keyword is an operator. In the arms of a `match` the `|`s of a
  pattern count nothing, while a guard and a block before the arms count, each at
  the limit and one over; a macro body of 20000 counted tokens and a pattern of 20000
  alternatives parse, and a guard or a block before the arms with as many `|`s as
  the limit is refused. Chains of `for S { a } in`, of reference marks
  before generic arguments with commas, and of `a < return x > b &&` are refused, beside
  short ones that parse. Five thousand statements, elements, arms, blocks and items in
  one level parse. The refusal names the line where the count went over. An 80 KB
  `let box box ... x` is refused by the recursion budget before the parser runs, which a
  flag set by the visit closure shows, beside the same pattern at depth 100 that parses
  and sets it. A `use` path and a `use` tree of
  nested groups at the limit load, resolve and drop on a thread of 512 KiB in a debug
  build, and one separator more is refused.
- Evasion fixtures: an `if` followed by 20000 `else if {true} {}` links, and
  `type T<'a> = &'a &'a ... u8;` with 20000 reference marks, each fail with the typed
  budget error that names the file, in process and in the child-process harness; the
  same shapes at 256 links and 256 marks parse.
- Verbatim-item fixtures: `pub use { ::p_one::X as D, crate::a::f, ::p_one::inner::{self,
  go as run}, { ::std::fmt::Debug as Dbg } };` and a module's `pub use { ::p_one::X,
  super::a::g as h };` resolve each name to what the separate `use` declarations give
  (`D` to `p_one::X`, `f` to `p_two::a::f`), and `Dbg` is `NotFound` in both; in edition
  2015 a member `::inner::go` reads the crate root. The facts of a group in an inline
  module are one `use` fact per member in that module, each with the span, line, test
  role and visibility of the whole declaration. `a::{::b}` is a `Syntax` error naming the
  file and line. A bodiless function at item level, a bodiless method in an impl block
  and `type bool;` are indexed with their kind, name line, span and attributes, and no
  `body`; `const C: u8;`, a bodiless `const` in an impl block and `type T: Bound;` are
  `UnsupportedItem` naming the file and line. A group of 6000 members under a
  `cfg` predicate of 6000 names reads that attribute once, counted as one attribute read for
  the whole declaration, each member's fact carrying the test role,
  visibility, span and tree it would have alone. A group and a bodiless method are charged
  the fact bytes of the separate `use` declarations and of the method with an empty
  body, and are refused at their line once over the fact budget.
- A syntax error carries the line and column of the parser's error: a fixture with the
  error on line 3 and another on line 5 assert the exact position, beside a file that
  parses.
- The harness judges strictly: an abort, a child that panics, one given an unknown
  selector, and one that exits 3 without a result line each fail the fixture. An abort
  is recognised only as death by a signal, the Windows stack-overflow status, or the
  runtime's own stack-overflow message.
- The parse thread is named after the file it parses, so a stack-overflow message
  from the runtime names the culprit. A fixture asserts the thread name for a file,
  and a child whose visitor recurses without end on the parse thread aborts with that
  name in the runtime's message, which the judge reports as a failure. A `return`
  chain that needs twice the parse stack is refused beside one of 1000 levels, which
  parses.
- Corpus replay under `cargo test`: a test loads every file of
  `fuzz/corpus/parse-source/` as the root file of a library in a child process and asserts that each is refused
  naming its budget or exits 0 with its recorded index, and that none aborts, so a
  regression is caught without running the fuzzer.
- The fuzz target `parse-source` builds and runs for a bounded number of iterations
  over its corpus without a crash.
- The fuzz target `load-workspace` builds and runs for a bounded number of iterations
  over its corpus without a crash. It resolves every query and item path in each
  namespace and package and lists the items of each file and under each path. Before
  its first input it loads a seed with items and fails unless the index holds them,
  and the fuzz runs build without `cfg(fuzzing)`, under which `proc-macro2` drops the
  positions the index needs. Its seeds are replayed under `cargo test` through the
  same input code, and each loads with items. Each fuzz run reads inputs of up to
  400000 bytes, which every corpus file fits, with a resident-memory limit of 2048 MiB
  and a limit of 512 MiB on one allocation, so a stack overflow, a panic or an
  allocation past either limit ends the run as a finding.
- A `cfg(test)` fixture asserting the tag on an item and on a module reached only
  through it.
- Symbol fixtures: one item of every kind, asserting kind, file, line and visibility
  (`pub`, `pub(crate)` and private); the impl shapes (inherent on a local type, inherent
  through an alias, trait impls on a local type, `String`, a blanket `T`, `&Foo`, `[u8]`
  and a tuple); inherent impls the index cannot see (`Missing`, `Vec<u8>`, a generic
  `T`, `&Foo`, `[u8]`, `crate::Missing`, a `bitflags!`-style macro type with test and
  non-test impls, and `impl dyn LocalTrait`), each loading with its members marked
  `Unresolved` in `items()` and `item_at()` under the written self path, listed by
  `unresolved_impls()`, `UnresolvedSelf` through the self path and `NotFound` at it,
  beside a neighbouring impl marked `Local`, and `super` above the root in such a self
  path still an error; a macro call in a trait body skipped beside one in an impl body;
  a glob of a local enum (`use self::Inner::*;`, and through `crate`) beside a trait
  impl, which loads, beside a glob with an unknown head, which is refused; globs of a
  macro-generated enum and of an `include!`-generated module, keyed under their path in
  the crate; a module,
  a function and a macro of one name selected by name space, and unit and tuple structs
  in the value name space; `#[cfg(unix)] fn a()` beside `#[cfg(windows)] fn a()` as one
  symbol with two sorted sites; items that share a source item grouped through a shared
  `tests/common/mod.rs`; documentation (text, hidden alone, a blank `///`,
  `include_str!`, a module's inner docs); `files()`, `items_in()`, `item_at()` and
  `is_shipping()`; the `test_fns()` data for a library, a binary and a test target; a
  first-party `mod test_crate;` beside a test target, whose shared paths are ambiguous
  in either order of the files while a path only one of them defines resolves to it;
  a library module named `bin_crate`, `test_crate`, `bench_crate`, `example_crate` or
  `build_script` that keeps its items in the library beside the target that shares
  its path; a build script's `main` beside a library module `build_script` with a
  `main`, and an explicit-path binary beside a library module `bin_crate::x`, each in
  both orders of the file names, each `Ambiguous` beside a layout without the overlap.
- Alias fixtures: a rename, a nested group, `self` in a group, a glob with `via_glob`
  beside the direct path without it, a chain of three hops with its `chain`, a cycle
  that yields `Cyclic` beside a chain that ends, a cycle of explicit imports whose two
  modules glob two different `X` (`demo::a::X` and `demo::b::X` both `Cyclic` in all 24
  orders, beside the neighbour without the import of `b::X` into `a`, where both resolve
  to `demo::c::X`, and an import that waits on the cycle, `Cyclic` though its own glob
  brings in `X`, as is the name in a module that globs it), a cycle with one glob on
  it resolving through the glob in every order (two imports with a waiter and a trait
  impl keyed at the glob's trait, whose value name space is `NotFound`, and three
  imports), beside cycles with two globs of one module, with globs in two of its
  modules and of renamed imports, each `Cyclic` in all six orders, a cycle whose one glob
  answers in the value name space only, while the glob's module globs the cycle too,
  resolving there and binding nothing in the type name space, and an import that reads
  only its own name through a glob, which binds nothing there and leaves another
  module's renamed import of the same item to its glob in the other name space, as
  rustc has them, two modules that each import `shared::inner as shared` and glob each
  other, which wait on one another, `Cyclic` for both in either order, and a glob `shared::*` beside a glob that
  brings in `shared`, which brings in nothing, also once the globs are rebuilt (a path
  through it reads only the other glob's `inner`), an import whose value name space a
  glob brings in only once a glob path two modules away resolves, which binds it there
  and keeps a glob of its module out, five modules that glob one another and bind two names twice,
  which settle to one answer in all 120 orders, a cycle of two imports through a glob
  whose answer is a renamed import, which resolves through the glob in the type name space
  and binds nothing in the value name space in all 120 orders, an import of its own
  module's name that reads it through the glob of its module and binds the value only,
  leaving another module's renamed import to its glob in the type name space in all 24
  orders, an import of a function whose type name space waits on a cycle, `Cyclic` in
  every name space for it and for a module that imports from it in all 120 orders, also
  where the cycle runs through globs, beside the import resolving once the cycle is gone,
  an item that a module's globs reach by a public and a private route, visible to a
  module that globs it in all 24 orders and not through the private route alone, a
  cycle whose one glob answer is kept by an import that reads its own name back through
  another module's glob, `Cyclic` in all 720 orders, a cycle of renamed imports through
  globs that leaving the one glob answer to the glob does not end, `Cyclic` in all 120
  orders, as rustc cannot determine them, beside a cycle whose import reads its name
  back through an import of the cycle, which resolves through the glob in all 24
  orders, a cycle of two renamed imports one of which has bound nothing, `Cyclic` in
  all six orders, a cycle of three imports that a glob answers while a renamed import
  waiting on it is brought back by a glob of a module the cycle binds in, which resolves
  in all 120 orders, an import whose type name space waits on a glob whose target's
  glob waits on a cycle, which binds the cycle's item and not its own module's glob in
  16 rotations of the eight modules, an import that reads, in one name space, a name
  another import keeps there as its path reaches a glob the index cannot resolve,
  which keeps its own name from its module's glob there in all 24 orders, a cycle that
  only a glob the importing module cannot access closes, in a child module (`mod b {
  use super::*; }`) and beside it, which binds the function and leaves the type name
  space to the importer's glob in every order, beside the same with the glob public,
  `Cyclic` (rustc: E0432), and the trait form of it, keyed at `demo::c::B` in all 24
  orders and refused as `UnresolvedTrait` with the glob public, a cycle whose glob
  answer holds an item of a private glob that the other import of the cycle cannot
  access, which resolves to the item it can access in all 24 orders, an import of a
  name that another module binds with a private import beside a glob of the crate
  `other`, `NotFound` in all six orders and in 14 rotations of the program the order
  search found it in, an import that reads a name only through a private glob of the
  module it reads, `NotFound` (rustc: E0432) with an import that waits on it in all 24
  orders, an unsettled private glob that holds nothing back for a module that cannot
  access it in all 120 orders, two cycles through private globs that rustc resolves,
  each read only as its imports can access it, in all 120 orders, and a module whose two globs start at
  crates, which settle, a glob `shared::*` whose first segment a glob of its module later
  brings in, which brings in nothing in all 24 orders though its path read the library
  first, a glob whose target binds its first segment only through a cycle and a trait
  only once a chain settles, which brings the trait in, an import `use shared::inner as
  inn;` beside `use inn::*;` and a glob that may bring in `shared` until an import of a
  function leaves the type name space, which reads the library in all six orders, also
  where that import is decided only once the queue is empty, in all 24 orders,
  definitions and explicit imports
  shadowing globs (also for the modules that glob the importer), an explicit import of a
  dependency's item and one that fails each shadowing a glob of the same name, which
  are `NotFound` beside the same modules without the import, and an import of a struct
  leaving the value name space to the glob's function, also for a module that globs
  the importer, beside a private glob, which leaves `NotFound` there; the import of a
  function `m::m` beside a glob of the module `m`, which gives the module in the type
  name space and the function in the value name space, and a path through the module,
  beside the same without the import and beside an import of a dependency's item, which
  stays `NotFound`; in every order, a module that globs the importer of a struct `A`
  (beside its glob of a function `A`) and another module with a function `A`, which is
  `Ambiguous` in the value name space and the struct in the type name space, the same
  with traits and a trait impl through it keyed `Unresolved` with both traits, and an
  import of a function beside a glob of a struct read through another module; a
  function imported under the name of the library `shared`, after which `pub use
  shared::x;` reads the library, `Ambiguous` where a glob also brings in a module
  `shared` through another module; an alias whose first reading binds the glob's
  function before the crates are read and the library in the type name space after
  (`pub use other as inn;`), so `a::inn::x` is `other::x` in all six orders, and in both
  orders where the glob's function is a definition, so nothing the import read changes
  once the crates are read, two globs with
  different items that are `Ambiguous` beside two that reach one, a mutual `super::*`,
  `crate`, `self` and `super` paths, `super` above the root, `extern crate demo as d`
  and `use ::demo::..`, `extern crate self as demo` named from every module beside a
  library that cannot name itself, raw identifiers in both spellings, a chain of 5000 re-exports
  declared in either order, `demo::nope`, `pub use serde::Serialize` and a function that
  a macro generates, each `NotFound` beside a positive resolution.
- Dependency-glob fixtures: `pub use std::io::*;` beside `pub use serde::*;`, beside
  `pub use std::fmt::*;`, `serde::*` beside `tokio::*`, the first two at the crate
  root and the second in edition 2015, each leave a missing name of the module and of
  a module that globs it `NotFound` in every name space, beside the module's own item,
  which resolves through both modules; `std::io::*` beside `self::missing::*` is
  `NotFound`, and beside `crate::other::missing::*` of a local module without the name
  is `UnresolvedGlob`; a cycle of globs whose path goes through a name the other glob
  would bring in stays `Cyclic` beside two globs that glob each other and resolve.
- Order of the failures: a glob of a missing module beside a glob of a module whose
  name a cycle of imports binds is `Cyclic`, and beside two globs that bring in two
  structs `X` is `Ambiguous`; a name found through a glob that resolves beside a glob
  of a missing module, a path through such a name, `self::m::missing::*` of an empty
  local module and `crate::missing::*` at the root are each `UnresolvedGlob`.
- Canonical-path fixtures: `demo::crate::a::x`, `demo::super::x`, `demo::self::a::x`,
  `demo::a::Self`, `demo::r#crate::a::x` and a leading `crate`, `self`, `super` or
  `Self`, each `Invalid` in every name space beside `demo::a::x` and the names
  `crates`, `selfish` and `Selves`, which resolve; a missing path of 50,000 segments
  that is `NotFound` and one of 50,000 segments through a module that imports itself
  that resolves, each answered with at most four segment reads and two name lookups
  for each segment plus two, counted in the walk and stopped at that count.
- Missing-glob fixtures: a module with `pub use self::missing::*;`, a module that globs
  it, `crate::a::gone::*`, `super::a::inner::nothing::*` and an import of a name the
  missing glob would bring in load, and each lookup that reads one of those globs is
  `UnresolvedGlob` naming the glob and its module, a lookup in the value name space of a
  function there resolving while the lookup in all name spaces fails, beside the
  module's own items and a sibling, which resolve; globs of `std::io`, of `serde`, of
  `::unknown::m`, of a module below a glob of `std::collections`, of a module below
  an import of a dependency module and a module that globs those leave a missing name
  `NotFound`.
- Resolve error text: every `ResolveError` variant, with an escape sequence, a
  bidirectional override and a line break in each field it shows, displays no control
  character and shows each field escaped, and a path and a package name with an escape
  sequence and a bidirectional override, which `resolve` and `resolve_in_package`
  refuse, are shown escaped.
- Extern-alias fixtures: `pub use serde;`, `pub use std;`, `pub use serde::{self,
  Serialize};`, `pub use std::{self as s2};`, and `std` imported at the root, in a module
  and renamed, each in editions 2018, 2021 and 2024 with every path through them, and
  through a module that imports from them, `NotFound` and never `Cyclic`, beside a local
  re-export that resolves; `pub use p_one;` of a library of the workspace, plain and as
  `{self as renamed}`, reaching its items and its crate root, beside a name it does not
  define. Edition 2015 fixtures: `use inner::go;`, `use ::inner::go` and `use inner;` in
  a module resolve from the crate root, and the same file in 2018 gives `NotFound`;
  `use serde;` in a module is the root's `extern crate serde;` item, and paths through
  it are `NotFound`; a 2015 test target names its library through `extern crate` and
  not by the name alone. Ambiguity fixtures: a `use` head that a glob brings in and that
  is also a dependency is `Ambiguous` naming the glob's module, directly, through a
  second import and for `use serde;` itself, beside a module's own `serde`, which
  shadows the crate, a glob of a value, which does not collide, and the same file in
  2015, which reads from the crate root. A glob and a library of the workspace that
  share a name, by the library's name and through an `extern crate` item of it at the
  root, are `Ambiguous` with the trait keyed `Unresolved` in every order of the
  declarations, beside the same imports without the glob and with the module's own item,
  which resolve; the order fixture asserts it over all 24 orders. A glob whose own path
  goes through an import of a crate (`use shared::inner as inn; use inn::*;`) and brings
  in `other`, which is also a library of the workspace, makes `pub use other::x;`
  `Ambiguous` and the trait through `other` `Unresolved` in all 24 orders, beside the
  same imports without the glob, which read the crate. An explicit import that binds the
  name of a library of the workspace (`pub use shared::inner::other;` beside `pub use
  other::x;` and a trait through `other`) shadows the crate for the path and the trait
  key in all 24 orders, in a module without globs and in one with a glob; a name that an
  explicit import binds is never read as the crate (`pub use k::other;` where `k` globs
  an alias whose own path starts at `other`, refused by rustc) and gives one answer in
  every order, `Cyclic` for the import; an import through a name with two `cfg`
  alternatives, an alias of a local module and an alias of a module of a workspace
  library, which a module with a glob reads only once the globs have brought in what
  they can (`pub use sys::x;`), binds what the second alternative reaches in all six
  orders, with an unchanged trait key, beside `pub use sys::only_unix;`, which binds
  through the first; and two imports whose first segments are the
  names the other binds, beside a glob and the crate of one of those names (`use
  shared::inner as inn; use inn::inner as shared;`, and `pub use inn::inner::other; pub
  use other as inn;`, which rustc cannot determine), are `Cyclic` in all six orders with
  an unchanged trait key, beside the neighbour without one import, which reads the
  glob. A glob that brings in
  `pub use std as serde;` collides with the crate `serde`, beside one that brings in
  `pub use serde;` and one that brings in `pub use serde::serde;`; under `#![no_std]` a glob's module `std` is read without ambiguity,
  beside the same with no attribute, with a `cfg_attr` and with `extern crate std;`. The leading-`::` and plain-name fixtures of a
  library named from its binary and its test target are written in edition 2021, where
  rustc accepts them.
- Extern-prelude fixtures, over two packages whose libraries are both `shared`: `shared`
  and `shared::X` are `Ambiguous` naming both crate roots, `resolve_in_package` resolves
  `shared::X` in each package and gives `UnknownPackage` for a name that is no package;
  a package depending on one of them by `path` reaches it through `use shared::X`; a
  dev-dependency renamed in `[workspace.dependencies]` is named by its new name in test
  code and in a test target and not in other code; a renamed path dependency is named
  by its declared name and not by its library's, also through an `extern crate` item at
  the crate root from a nested module; a test target and a binary name their own
  package's library and the build script does not. A library whose dependency and
  renamed dev-dependency are two packages under one name names the dependency in its
  code, an `extern crate` of the name and its binary, while paths through the name from
  test code, an `extern crate` in test code and its test target, and an import of the
  name itself, are `Ambiguous` naming both crate roots, also for an item that only one
  of them defines; a trait impl through the name is keyed under the dependency's trait
  in library code and `Unresolved` with a candidate in each library in test code. A trait path, and a
  glob of an unseen path, through a name that two globs bind to two modules keys
  `Unresolved` with a candidate through each. A `workspace = true` dependency with no
  workspace above, from a workspace the package is not a member of, or without its
  entry is a manifest error naming the dependency key, beside one that loads.
- Order fixtures: an explicit import that must shadow a glob for a reader declared before
  it, and the same modules declared in every order of 1440, giving one answer for each
  path, with glob against glob staying ambiguous; a second set of 1440 orders with a
  cycle of explicit imports beside globs, an import that waits on it, a failing import
  and a chain that ends, giving `Cyclic` for the three imports of the cycle and the one
  waiting on it and one answer for each other path; glob visibility (private beside public,
  a private item into a descendant, `pub(crate)`, `pub(super)`, `pub(in path)`); the
  canonical keys of trait impls in each spelling and the error for a trait path that
  cannot be made absolute; the prelude per edition, an inherited edition, one and several
  dependency globs, a head that is no known crate, dependency tables by target kind with
  renames, and a path through a local re-export of a dependency.
- Settling fixtures: the strongly connected components of a graph come out once each,
  after every component they lead to, also for a chain of 200,000 nodes on the test
  thread's stack; a load whose tables settle twice (a glob whose own path starts at a
  crate brings in a name an import read as another crate) is charged the bindings of
  each settle and what the second builds again, and with the ambiguous import held from
  the start settles once and is charged its bindings only. The imports left are decided
  in one round for a chain of 2 and of 20,000 imports that bind nothing, which the
  queue decides, in two for the same chains beside globs, in two for an import that
  bound its name before the crates opened and binds the crate in the queue once they
  are, and in one
  round for a cycle of 2 and of 20,000 imports with an import waiting on each of them,
  beside an import held as ambiguous, which adds no round; each of those rounds but the
  last decided at least one more thing, of at most eleven for each import. A round that
  decides nothing more, and a settle past one more than there are imports, are each
  reported as an `Internal` load error naming the round or the count, in every build.
  The imports of a name are found among names that share its first eight bytes, a
  name that is a prefix of another and the same names in two modules. In debug builds
  every load
  checks, once the tables are settled, that the imports of each name, walked again
  against them, reach every item they bound and nothing the module does not bind under
  that name, and that one that did not resolve reaches nothing. A cycle of two imports
  left to a glob in the round that also decides an import further along the glob's
  chain reads both functions the chain brings in, `Ambiguous` as rustc's E0659, in 28
  orders; and the imports of a cycle left to a glob that reads a name a module
  re-exports ambiguously stay `Ambiguous` with the modules that glob it, in 36 orders.
  There rustc's own answer depends on the order of the declarations: in half of the
  orders it compiles, binding `m28::Y`, and in the other half it reports E0659. RAF's
  `Ambiguous` in every order is deliberate, as it agrees with rustc wherever rustc
  reports an error and does not depend on the order.
- Trait-key fixtures through `use` declarations: `use std;` with `impl
  std::fmt::Display for &Foo` in editions 2018, 2021 and 2024 and in a 2015 module, keyed
  `std::fmt::Display`, beside `use nope;`, which is refused; `pub use serde;` with `use
  crate::serde::Serialize;` in a module, and `pub use serde::{self, Serialize};`, keyed
  at `serde`; a glob through a re-exported crate keyed under the glob; in edition 2015,
  `use serde;` in a module through the root's `extern crate serde;`, a module's `use
  undo_log::Rollback;` keyed at the local trait, and `impl ::s::Serialize` through
  `extern crate serde as s;` keyed `serde::Serialize` beside 2018, where `::s` names no
  crate and the load is refused; a trait through an ambiguous `use` head keyed
  `Unresolved` with the local trait and the dependency's, and through a glob that brings
  in `std as serde` keyed with `serde::Serialize` and `std::Serialize`; an import met
  again through a parent glob adding nothing, so the other glob keys the trait, beside a
  pure cycle, which is refused; in edition 2015, `use serde::Serialize;` without an
  `extern crate` item at the root, or with one only in a module, refused, beside the
  same with the item, and `std` (or `core` under `#![no_std]`) named with no item, beside
  `core` in a crate with `std`, which is refused; a trait name that two globs bind keyed
  `Unresolved` with both traits; a trait that two globs of unknown crates could give
  refused naming the same glob, by written path, in all six orders; `use
  serde::Serialize;` beside a glob of a local trait `Serialize` keyed `serde::Serialize`,
  beside the same glob without the import, keyed at the local trait. On a thread of 2 MiB, in debug
  and release builds: a file of 20,000 chained local aliases (`use crate::m::T as A0;
  use A0 as A1;` and so on, no import naming itself) with `impl A19999 for X` loads keyed
  `demo::m::T`; a trait imported from `std` and reached through 20,000 chained
  globs of modules loads keyed `std::fmt::Display`; and the completion audit's own
  shape, `use std;` with `impl std::fmt::Display for &Foo` and 20,000 `use
  std::collections::HashMap as H..;` lines, loads keyed `std::fmt::Display`.
- A determinism check: two loads over the same tree give equal indexes, and `items()`
  and `targets()` come in strictly increasing order.
- A `DiskTree` run over a temporary directory gives the same index as the
  equivalent `MemoryTree` for trees of regular files. `MemoryTree` has no symlinks or
  special files, so this equivalence does not cover the containment cases, which have
  their own `DiskTree` fixtures below.
- `DiskTree` containment fixtures, each built in a temporary directory beside a
  sentinel file outside the root, asserting an error and that the sentinel is not
  read:
  - `src/inner.rs` is a symlink to a file outside the root, reached by a plain
    `mod inner;`;
  - `#[path = "vendored/../x.rs"]` where `vendored` is a symlink to a directory
    outside the root, so the `..` leaves the root only on disk;
  - a `Cargo.toml` target with `path = "../outside.rs"` and one with an absolute
    path outside the root;
  - a link to a symlink that itself points outside the root;
  - a directory-symlink loop, which ends with an error and does not recurse;
  - a FIFO named as a module file, which is refused without blocking on a read;
  - a missing root, which is `NotFound`, and a symlinked root, which is followed to
    its target and judged there;
  - under each strategy (Linux, other Unix and Windows, each run on Linux, the last
    two through a test override), a directory swapped for a symlink to outside the
    root between the containment check and the open of a read, a listing or an
    entry's directory, and a directory moved out of the root, with a symlink to
    outside put in its place, after the handle is verified and before the file is
    read, the entries are listed or the entry is looked up, each refused with
    `OutsideRoot`, leaving the outside file unread and the outside names unreturned,
    beside the same calls without a swap, which succeed;
  - under each strategy, a swap made after the containment check and undone after
    the open, before the handle is verified, for a read, a `read_unseen`, a listing
    and an entry, and a swap flipped at each of those points and again after the
    verification: each is `OutsideRoot`, and no outside text, name or kind is
    returned;
  - under the Linux and other-Unix strategies, an outside FIFO reached through such
    a swap before the open, with a writer blocked in its own open of the FIFO: the
    read is `OutsideRoot` and the writer is never released, because no reader opened
    the FIFO;
  - under each strategy, every open that resolves a name below the root, for reads,
    identities, entries and listings that reach a FIFO or lead to one, is an `O_PATH`
    reference or an `O_NONBLOCK` open. The flags are checked before the open is made,
    so a regression that drops the flag fails the check and does not wait on the FIFO;
  - under each strategy on Linux, the containment fixtures of the disk backend: an
    outside symlink and a chain to it refused beside in-root links, a `..` after an
    outside directory symlink, a FIFO and a directory named as a file, listings and
    their refusals, entry kinds and an entry below an outside directory, the identity
    of a file, its links and a path through a directory link, a symlink loop, and an
    absolute in-root link spelled through a linked parent of the root, which is
    followed, beside a `..` from it that leaves the root;
  - links into the outside, ending there or passing an outside directory and
    climbing back into the root, each to an outside path that exists and to one that
    does not: the other-Unix walk answers `OutsideRoot` to all of them, and the
    Linux and Windows strategies answer `OutsideRoot` and `None`, and the in-root
    file and `NotFound`; a `..` link that stays inside the root is followed under
    every strategy;
  - links to an in-root file whose resolution leaves the root and climbs back
    (`../root/real.rs`, an absolute `…/root/../root/real.rs`, and an outside link to
    the root whose own target climbs): Linux and Windows read the file and the
    other-Unix walk answers `OutsideRoot`, beside an absolute target through a
    linked parent of the root, which every strategy reads;
  - under the other-Unix walk, an entry that is a symlink when a step opens it and a
    directory when its status is read is stepped again, beside the same read with no
    change; and in a child process limited to 64 descriptors, a path 300 directories
    deep is read, read through a `..`, listed and looked up under every strategy;
  - the handle verifiers: the path the system gives for a handle is refused outside
    the root beside one inside it; on Linux a handle on a file outside the root is
    refused beside an in-root handle, and a missing descriptor link is `Io`; on
    Windows the final path of a handle is its canonical path and lies under the root;
  - the Windows names: the reserved device names, with extensions and in any case,
    names ending in a dot or a space and names holding a colon are recognised beside
    neighbouring plain names on every platform; on Windows a `DiskTree` refuses each
    with `OutsideRoot` beside plain names that read; elsewhere a tree with `aux/`,
    `aux/con.rs` and `nul.txt` loads and reads;
  - every target of the crate, its unit tests included, type-checks for macOS,
    FreeBSD and Windows with warnings denied (`make cross-check`), and CI runs the
    crate's tests on macOS and Windows hosts, where the other-Unix and Windows
    strategies are the native ones;
  - a symlink to a file inside the root, and a chain of such symlinks, is accepted;
  - a chain that ends outside the root, a symlinked `Cargo.toml` and a symlinked
    target file that lead outside, and every discovery classification of a symlink
    (file, directory, FIFO, outside file, outside directory, dangling, loop).
- Load-level cap and text fixtures: through `Workspace::load`, a library root and a
  module file of exactly 8 MiB (padded with comments) load and one byte more is refused
  with the cap as its limit, naming the file (the declaring file and the line of its
  `mod`, with the module file in the message); a `max_file_bytes` lowered to 1000 admits
  a child file of 1000 bytes and refuses one of 1001; a raised cap admits a file the
  default refuses. A child process under an address-space limit (`ulimit -v`) that
  cannot give the parse thread its stack ends in a `Spawn` error naming the file,
  beside the same child without a limit, which loads. A module file that is not UTF-8
  is refused naming it and the line of its `mod`, beside the same text in UTF-8, which
  loads. CRLF line endings, with and without a byte-order mark, give the lines and byte
  spans of the LF text; a byte-order mark on the first line leaves its span starting
  after the mark's bytes; a syntax error and a budget refusal name the same line in LF
  and CRLF text and after a mark. A one-line item on line `n` has the lines
  `n..n+1`, one over three lines ends before the line after its last, and a crate root
  spans every line of its file.
- Exported-item fixtures, each beside a neighbour that lists: a `pub fn` beside
  private, `pub(crate)`, `pub(super)`, `pub(self)` and `pub(in path)` items; a `pub`
  item of a private module, absent until `pub use` exports it under its name and
  under a rename, beside the module made `pub`; a `pub` glob that brings in the
  `pub` items of its module only, and a private one that brings in nothing; nested
  re-export chains through a `pub` module of a private module and through globs of
  globs, a `pub(crate)` glob or re-export on the way that keeps the name in, and an
  item that a restricted glob and a `pub` route both bring in, which is exported;
  `pub(crate)`, private and `pub(in crate)` re-exports, and a `pub` re-export of a
  `pub(crate)` item, each exporting nothing; re-exports of a dependency, of the
  standard library by a glob, of a crate of the workspace and of its glob, of a
  module a macro generates and `pub extern crate alloc;`, each a `Reexport` with its
  written path, line and, for the workspace crate, its item, beside `pub extern
  crate self as me;`, an item; enum variants and struct fields, which are not items,
  and a glob of an enum; the `pub` inherent methods and constants of a `pub` type,
  also from an impl in another module and under a renamed re-export, without its
  private, `pub(crate)`, trait impl and test methods, and nothing of a private type;
  every member of a `pub` trait and none of a private one; `#[doc(hidden)]` on an
  item, a module, a module file, a named and a glob re-export and a method, left out
  by default and listed flagged with `include_hidden`, and a module that a hidden and
  a shown re-export reach, read under the shown one; test modules, items, named and
  glob re-exports and `#[test]` functions, none listed, beside the same outside test
  code; `#[macro_export]` macros, at the root and in a module, beside a local macro,
  its `pub(crate) use` and a macro-generated function; a binary, which lists its own
  `pub` items and re-exports its library's, and a test target, refused as
  `NotLibraryOrBinary`; a module reached by two `pub` paths and a cycle of globs,
  each read once.
- A re-export of an item of another crate of the workspace that two `pub` globs
  bring in is written as the glob that comes first in the file, with its line,
  whichever of the two that is; beside it, the re-export that one glob alone
  brings in is written as that glob.
- Re-exports of a crate outside the workspace that a `pub` glob carries: from a
  private module and from a `pub` one (listed under both paths), renamed, through
  two globs, hidden by a hidden glob of the reader and not by one further on, each
  a `Reexport` of the
  reader, beside the explicit `pub use a::Debug;` that names one; none through a
  restricted or private glob, of a `pub(crate)` re-export, past a name the reader
  or a module on the way binds, or of test code. A named re-export of an enum
  variant is a `Reexport`, and a glob of the enum adds nothing. The `macro_export`
  flag of each macro item.
- A module of 3000 `pub` globs of one name each lists within 150 ms in a release
  build and a second in a debug build; 3000 that each carry a re-export of its own
  name beside 3000 that each carry `Rc` and a `Debug` the reader shadows list
  within 0.5 s in a release build and 4 s in a debug build; 3000 that each carry a
  name beside 3000 globs of modules that bind it privately list within 300 ms in a
  release build and 3 s in a debug build, charged under 32 MiB beyond the load,
  and 10,000 such list within a second in a release build.
- Carried names decided exactly: a binder that dominates the carrier, a binder
  beside a free route, a binder whose routes lead elsewhere, two binders on the two
  routes, two binders beside a third free route, two binders on one route behind a
  free module, the reader binding the name, a carrier with `pub` globs of its own,
  which is not its own binder, two carriers of one name of which only the one with
  a free route is listed, from its own line, in both orders, and a binder in an
  irreducible cycle entered at two modules, which dominates nothing, beside the
  cycle without its second entry. A chain of 5000 globs whose 2000 carried names
  two modules beside it bind is refused with `CarryBudget`, naming the re-export
  and the line of its declaration, of `ItemBytes`, beside a chain of 50 and 200
  names, which lists them.
- A re-export whose shown route a module on the way shadows and whose hidden
  route is free is hidden, as the item of the same shape is, also with two
  shadowed shown routes, beside a free shown route, which shows it; a hidden glob
  partway along the route, alone and beside a shadowed route, hides neither the
  re-export nor the item.
- A differential family of 166 small crates, over the depth of the route (0 to 2
  globs), a binder (none, on the route, off it), `#[doc(hidden)]` (none, on the
  reader's glob, on a glob partway, on the declaration, on a module partway, on
  the reader's module), a reader at the root or in a `pub` module, and an extra
  hidden route: in each, the carried `pub use std::cell::Cell;` is listed, hidden
  or absent exactly as `pub struct Cell;` in its place is.
- A seeded random differential of 450 small crates, from three fixed seeds, of one
  to six private modules of one to three globs each to one another and to the
  carrier, a quarter of them `#[doc(hidden)]` and a fifth `pub(crate)`, cycles
  allowed, binders by a private struct or a private import, hidden modules and a
  hidden carrier, and a reader of one to three globs, some hidden, at the root or
  in a `pub` module that is sometimes hidden: in each, the carried re-export and
  the item give the same paths with the same flags; the answers include absent,
  shown and hidden. The two shapes a route that does not carry the name out
  decides wrongly (a private import shadowing the glob, and a `pub(crate)` glob),
  at the root and in a `pub` module, for the item and the re-export, are listed
  hidden and left out by default.
- An item `Rc` and a carried `pub use std::rc::Rc;` under one name are both listed,
  with no ambiguity.
- A module that a hidden and a shown `pub` glob both reach carries its re-export
  shown, beside the same with the shown glob gone, where it is hidden.
- Determinism: the same surface from two calls and over four orders of the
  declarations, sorted by path.
- The library of the Examples lists `demo::run` alone, and with the glob or the
  `pub` module it lists the items the Examples name.
- Refusals: two `pub` globs that bring in two items `x` are `Ambiguous` beside two
  that bring in different names; a cycle of `pub` imports is `Cyclic` beside a chain
  that ends; a `pub` glob of `self::missing` is `UnresolvedGlob` naming the glob and
  its module beside a private one; a listing over the item-bytes budget left by the
  load is a `Budget` error of `ItemBytes` naming the file and line, beside the same
  tree with a larger budget, which lists every item.
- Stack and work: 20,000 chained `pub use` re-exports and a chain of 20,000 `pub`
  globs list on a thread of 2 MiB in debug and release builds.
- On this repository, the surface of each library and binary lists without an
  error; that of `raf-source` holds `raf_source::Workspace`, its method
  `exported_items` and `raf_source::DiskTree`, mapped to
  `raf_source::tree::disk::DiskTree`, holds no item of a private module and lists
  `raf_source::syn` as a re-export of a crate outside the workspace, and the
  `cargo-workplan` binary exports nothing.
- The `load-workspace` fuzz target lists the surface of every target of each
  workspace that loads, hidden items included.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_source::Workspace::exported_items` | doctest | [Verification](#verification) |
| `raf_source::cost::tests::a_hash_table_is_charged_at_least_its_buckets` | test | [Verification](#verification) |
| `raf_source::cost::tests::a_list_is_charged_at_least_its_room` | test | [Verification](#verification) |
| `raf_source::cost::tests::a_map_of_one_entry_is_covered_by_what_a_map_costs_once` | test | [Verification](#verification) |
| `raf_source::error::tests::a_spawn_error_displays_the_file_and_carries_its_cause` | test | [Verification](#verification) |
| `raf_source::error::tests::control_characters_in_a_path_are_escaped` | test | [Verification](#verification) |
| `raf_source::error::tests::every_user_derived_field_of_every_display_is_escaped` | test | [Verification](#verification) |
| `raf_source::facts::tests::a_column_is_turned_into_its_byte_offset_on_short_long_and_wide_lines` | test | [Verification](#verification) |
| `raf_source::facts::tests::a_large_attribute_on_a_verbatim_use_group_is_read_once_for_all_its_members` | test | [Verification](#verification) |
| `raf_source::facts::tests::a_long_line_of_wide_characters_is_indexed_once_for_all_its_positions` | test | [Verification](#verification) |
| `raf_source::glob::tests::a_segment_is_matched_in_at_most_pattern_times_name_steps` | test | [Verification](#verification) |
| `raf_source::item::tests::an_item_record_is_at_most_two_hundred_fifty_six_bytes` | test | [Verification](#verification) |
| `raf_source::members::tests::distinct_patterns_at_both_caps_are_tested_a_bounded_number_of_times` | test | [Verification](#verification) |
| `raf_source::members::tests::duplicate_patterns_are_compiled_and_tested_once` | test | [Verification](#verification) |
| `raf_source::resolve::tests::a_path_of_fifty_thousand_segments_reads_each_segment_a_bounded_number_of_times` | test | [Verification](#verification) |
| `raf_source::resolve::tests::every_user_derived_field_of_every_resolve_error_is_escaped` | test | [Verification](#verification) |
| `raf_source::settle::tests::a_binding_list_stays_sorted_keeps_the_better_route_and_takes_only_its_room` | test | [Verification](#verification) |
| `raf_source::settle::tests::a_round_that_decides_nothing_more_or_a_settle_too_many_is_reported_in_every_build` | test | [Verification](#verification) |
| `raf_source::settle::tests::components_come_out_once_each_after_every_component_they_lead_to` | test | [Verification](#verification) |
| `raf_source::settle::tests::each_settle_after_the_first_is_charged_what_it_builds_again` | test | [Verification](#verification) |
| `raf_source::settle::tests::the_imports_left_are_decided_in_rounds_that_do_not_grow_with_them` | test | [Verification](#verification) |
| `raf_source::settle::tests::the_imports_of_a_name_are_found_among_names_that_share_its_first_eight_bytes` | test | [Verification](#verification) |
| `raf_source::test_crate::corpus_replay::every_corpus_file_is_refused_or_indexed_with_its_recorded_symbols_and_none_aborts` | test | [Verification](#verification) |
| `raf_source::test_crate::discovery_targets::a_manifest_over_the_cap_is_refused_naming_it_and_one_of_exactly_the_cap_loads` | test | [Verification](#verification) |
| `raf_source::test_crate::discovery_targets::a_test_setting_of_the_wrong_type_is_a_manifest_error_naming_its_key` | test | [Verification](#verification) |
| `raf_source::test_crate::discovery_targets::an_empty_root_list_is_a_no_roots_error_beside_dot_which_loads` | test | [Verification](#verification) |
| `raf_source::test_crate::discovery_targets::each_target_kind_with_default_paths_has_its_name_kind_crate_path_and_shipping_flag` | test | [Verification](#verification) |
| `raf_source::test_crate::discovery_targets::harness_required_features_test_and_doctest_are_read_from_each_target_table` | test | [Verification](#verification) |
| `raf_source::test_crate::discovery_targets::two_packages_whose_targets_share_crate_paths_load_and_are_told_apart_by_package` | test | [Verification](#verification) |
| `raf_source::test_crate::discovery_targets::two_packages_with_one_name_are_an_error_naming_both_manifests` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_containment::a_directory_symlink_loop_ends_with_an_error_and_is_listed_but_never_entered` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_containment::a_fifo_named_as_a_file_is_refused_without_blocking_beside_a_regular_file` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_containment::a_parent_segment_after_an_outside_directory_symlink_is_resolved_on_disk_and_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_containment::a_symlink_to_a_file_outside_is_refused_beside_an_in_root_symlink_that_is_accepted` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_containment::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_containment::the_identity_of_a_file_is_the_device_and_inode_of_its_verified_handle` | test | [Verification](#verification) |
| `raf_source::test_crate::error_text::a_manifest_with_control_characters_in_a_value_is_shown_escaped` | test | [Verification](#verification) |
| `raf_source::test_crate::error_text::a_path_attribute_with_terminal_escapes_is_shown_escaped` | test | [Verification](#verification) |
| `raf_source::test_crate::error_text::a_resolved_path_or_package_with_control_characters_is_shown_escaped` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_binary_lists_its_own_pub_items_and_a_test_target_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_binder_in_an_irreducible_cycle_that_dominates_nothing_leaves_its_name_carried` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_carried_outside_re_export_is_kept_out_by_a_restricted_glob_or_a_name_on_the_way` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_carried_re_export_is_kept_out_exactly_where_every_route_passes_a_module_that_binds_its_name` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_carried_re_export_is_listed_hidden_or_absent_exactly_as_an_item_of_the_same_shape` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_carrier_with_pub_globs_of_its_own_is_not_its_own_binder` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_chain_of_re_exports_through_nested_modules_is_followed_and_a_restricted_hop_stops_it` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_chain_of_twenty_thousand_pub_globs_is_listed_on_a_two_mib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_glob_re_export_brings_in_the_pub_items_of_its_module_only` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_hidden_glob_partway_along_a_route_hides_a_carried_re_export_no_more_than_an_item` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_module_of_three_thousand_pub_globs_is_listed_in_time_linear_in_its_names` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_module_of_three_thousand_pub_globs_that_each_carry_a_re_export_is_listed_in_linear_time` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_module_reached_by_a_hidden_and_a_shown_glob_carries_its_re_export_shown` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_module_reached_by_two_pub_paths_is_listed_under_each_and_read_under_the_first` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_named_re_export_of_an_enum_variant_is_a_re_export_and_a_glob_of_an_enum_adds_nothing` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_pub_function_is_exported_beside_private_and_restricted_ones_which_are_not` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_pub_item_of_a_private_module_is_exported_only_through_a_pub_use_and_under_its_rename` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_re_export_of_a_path_the_index_cannot_follow_is_listed_as_a_re_export` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_re_export_of_an_outside_crate_that_a_pub_glob_carries_is_a_re_export_of_the_reader` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_re_export_of_another_crate_is_a_re_export_and_not_an_item` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_re_export_that_only_a_hidden_glob_carries_past_a_shadow_is_hidden_as_an_item_would_be` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_re_export_two_globs_carry_is_written_as_the_first_glob_that_carries_it` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_restricted_or_private_re_export_is_not_exported_beside_a_pub_one_that_is` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_surface_over_the_item_budget_left_by_the_load_is_refused_beside_one_within_it` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::an_ambiguous_a_cyclic_or_an_unresolved_public_name_fails_beside_a_neighbour_that_lists` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::an_exported_macro_is_exported_and_a_local_or_generated_one_is_not` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::an_item_and_a_carried_outside_re_export_of_one_name_are_both_listed_and_not_ambiguous` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::doc_hidden_items_are_left_out_by_default_and_flagged_when_included` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::enum_variants_and_struct_fields_are_not_items_of_the_surface` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::in_random_small_crates_a_carried_re_export_is_listed_hidden_or_absent_as_the_item_is` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::of_two_carriers_of_one_name_only_the_one_with_a_free_route_is_listed_with_its_own_line` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::only_a_route_that_carries_a_name_out_decides_whether_it_is_hidden_for_items_and_re_exports` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::readers_of_three_and_ten_thousand_carried_names_beside_sibling_shadows_list_in_bounded_time_and_bytes` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::searches_for_names_that_several_binders_keep_from_a_long_route_are_refused_by_the_item_budget` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::test_code_is_not_part_of_the_surface_beside_the_shipped_items` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::the_example_of_the_contract_lists_one_item_and_more_through_a_glob_or_a_pub_module` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::the_members_of_a_pub_trait_are_exported_and_those_of_a_private_trait_are_not` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::the_pub_methods_of_a_pub_type_are_exported_and_those_of_a_private_type_are_not` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::the_surface_is_the_same_in_every_load_and_in_every_order_of_the_declarations` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::twenty_thousand_chained_re_exports_are_listed_on_a_two_mib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::fs_confinement::only_the_disk_backend_names_the_file_system_module` | test | [Verification](#verification) |
| `raf_source::test_crate::index_disk::disk_and_memory_trees_give_equal_workspaces_and_equal_resolutions` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_chain_of_files_that_each_declare_the_next_twice_is_charged_what_it_keeps` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_chain_of_globs_is_charged_what_each_link_brings_in` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_doc_comment_counts_as_the_tokens_of_its_attribute_and_other_comments_count_nothing` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_file_at_the_token_limit_parses_and_one_more_token_is_refused_naming_its_line` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_file_reached_by_many_module_paths_is_charged_once_per_path_and_refused_over_the_item_budget` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_long_alias_path_in_a_file_reached_many_times_is_held_and_charged_once` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_long_extern_crate_name_in_a_file_reached_many_times_is_held_and_charged_once` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_long_module_name_is_charged_in_every_item_path_and_refused_over_the_item_budget` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_module_that_many_modules_glob_is_kept_once_for_all_of_them` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_verbatim_use_group_and_a_bodiless_function_are_charged_as_their_plain_forms` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::each_costly_shape_at_the_token_limit_loads_and_one_more_token_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::each_large_fact_payload_is_charged_and_refused_at_its_line_once_over_the_fact_budget` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::glob_tables_at_the_glob_budget_load_and_one_byte_less_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::glob_tables_of_a_generated_binding_crate_are_charged_what_they_keep` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::index_records_at_the_item_budget_load_and_one_byte_less_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::modules_that_glob_one_another_are_refused_over_the_glob_budget_naming_a_glob` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::nested_inline_modules_whose_scopes_hold_one_name_are_charged_what_they_keep` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::repeated_globs_of_one_module_are_charged_once_for_what_it_holds` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::the_bytes_read_at_the_total_budget_load_and_one_byte_less_is_refused_naming_the_file` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::the_default_budgets_are_the_stated_limits` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::the_fact_budget_counts_over_every_file_of_the_load` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::usage_reports_what_a_load_spent` | test | [Verification](#verification) |
| `raf_source::test_crate::load_cap::a_cap_larger_than_the_default_admits_a_file_the_default_refuses` | test | [Verification](#verification) |
| `raf_source::test_crate::load_cap::a_library_root_of_exactly_the_cap_loads_and_one_byte_more_is_refused_naming_it` | test | [Verification](#verification) |
| `raf_source::test_crate::load_cap::a_lowered_cap_applies_to_a_child_file_and_spares_a_root_within_it` | test | [Verification](#verification) |
| `raf_source::test_crate::load_cap::a_module_file_of_exactly_the_cap_loads_and_one_byte_more_is_refused_naming_it` | test | [Verification](#verification) |
| `raf_source::test_crate::manifest_caps::distinct_patterns_at_both_caps_load_with_no_tree_calls_for_the_excludes` | test | [Verification](#verification) |
| `raf_source::test_crate::manifest_caps::duplicate_patterns_within_the_caps_load_with_the_tree_calls_of_one_of_each` | test | [Verification](#verification) |
| `raf_source::test_crate::manifest_caps::exclude_at_the_cap_loads_and_one_over_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::manifest_caps::exclude_still_removes_literal_wildcard_and_nested_members` | test | [Verification](#verification) |
| `raf_source::test_crate::manifest_caps::members_at_the_cap_loads_and_one_over_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::manifest_caps::the_duplicate_pattern_probe_is_refused_before_any_pattern_is_expanded` | test | [Verification](#verification) |
| `raf_source::test_crate::memory_tree::duplicate_keys_are_refused_literally_and_after_normalisation_beside_distinct_keys` | test | [Verification](#verification) |
| `raf_source::test_crate::memory_tree::keys_that_leave_the_root_are_refused_with_outside_root_beside_a_valid_key` | test | [Verification](#verification) |
| `raf_source::test_crate::memory_tree::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen` | test | [Verification](#verification) |
| `raf_source::test_crate::memory_tree::the_identity_of_a_file_is_its_normalised_key_and_refuses_what_read_refuses` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::a_chain_of_files_that_each_declare_the_next_twice_is_refused_by_the_module_path_budget_opening_each_file_once_beside_a_small_diamond` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::a_child_file_that_sorts_before_its_mod_rs_parent_is_placed_under_its_own_path` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::a_file_reached_under_two_spellings_is_read_once_and_keeps_the_spelling_that_sorts_first` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::a_module_cycle_is_an_error_naming_both_files` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::a_non_literal_path_attribute_is_reported_as_such_from_the_attribute` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::a_path_attribute_in_a_mod_rs_file_is_relative_to_that_files_directory` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::a_path_that_leaves_the_tree_is_refused_naming_module_and_attribute_without_reading_it` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::an_item_is_found_only_under_its_own_module_path` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::every_cfg_attr_path_alternative_is_followed_carrying_its_predicate` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::the_cfg_test_matrix_tags_items_and_leaves_their_neighbours_alone` | test | [Verification](#verification) |
| `raf_source::test_crate::overflow_harness::a_deliberate_overflow_aborts_and_the_runtime_names_the_parse_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::overflow_harness::a_return_chain_that_needs_twice_the_stack_is_refused_beside_one_at_a_safe_depth` | test | [Verification](#verification) |
| `raf_source::test_crate::overflow_harness::every_committed_overflow_shape_is_refused_by_a_budget_and_none_aborts` | test | [Verification](#verification) |
| `raf_source::test_crate::overflow_harness::every_shape_family_loads_at_its_largest_admitted_size_and_one_size_more_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::overflow_harness::the_classifier_accepts_only_a_signal_a_stack_overflow_status_or_the_runtime_message_as_an_abort` | test | [Verification](#verification) |
| `raf_source::test_crate::overflow_harness::the_parse_thread_is_named_after_the_file_and_has_the_stated_stack` | test | [Verification](#verification) |
| `raf_source::test_crate::prescan_budgets::a_construct_that_continues_past_a_comma_or_a_closing_brace_keeps_its_count` | test | [Verification](#verification) |
| `raf_source::test_crate::prescan_budgets::a_label_a_use_path_separator_and_each_delimiter_count_at_the_limit_and_one_over` | test | [Verification](#verification) |
| `raf_source::test_crate::prescan_budgets::a_level_returns_to_zero_where_its_constructs_end_beside_the_same_count_without_the_end` | test | [Verification](#verification) |
| `raf_source::test_crate::prescan_budgets::a_macro_call_counts_only_its_groups_and_a_match_pattern_only_its_alternatives` | test | [Verification](#verification) |
| `raf_source::test_crate::prescan_budgets::a_syntax_error_reports_its_own_line_and_column_beside_a_parsing_neighbour` | test | [Verification](#verification) |
| `raf_source::test_crate::prescan_budgets::chains_that_overflowed_the_parse_stack_are_refused_by_the_recursion_budget` | test | [Verification](#verification) |
| `raf_source::test_crate::prescan_budgets::delimiter_nesting_is_refused_at_one_hundred_twenty_nine_beside_one_hundred_twenty_eight` | test | [Verification](#verification) |
| `raf_source::test_crate::prescan_budgets::every_counted_keyword_counts_one_at_the_limit_and_one_over` | test | [Verification](#verification) |
| `raf_source::test_crate::prescan_budgets::every_operator_counts_one_for_each_of_its_characters_at_the_limit_and_one_over` | test | [Verification](#verification) |
| `raf_source::test_crate::prescan_budgets::five_hundred_nested_generics_are_refused_beside_sixty_four` | test | [Verification](#verification) |
| `raf_source::test_crate::prescan_budgets::the_eighty_kilobyte_box_pattern_is_refused_by_the_pre_scan_before_the_parser_runs` | test | [Verification](#verification) |
| `raf_source::test_crate::prescan_budgets::the_recursion_limit_is_half_the_parse_stack_over_the_measured_cost_of_one_unit` | test | [Verification](#verification) |
| `raf_source::test_crate::prescan_budgets::the_recursion_refusal_names_the_line_where_the_count_went_over` | test | [Verification](#verification) |
| `raf_source::test_crate::prescan_budgets::twenty_thousand_else_if_links_are_refused_beside_two_hundred_fifty_six` | test | [Verification](#verification) |
| `raf_source::test_crate::read_cap::disk_tree_enforces_the_cap_at_eight_mib` | test | [Verification](#verification) |
| `raf_source::test_crate::read_cap::the_default_cap_is_eight_mib` | test | [Verification](#verification) |
| `raf_source::test_crate::relpath::a_parent_segment_above_the_root_is_refused_beside_the_one_that_stays_inside` | test | [Verification](#verification) |
| `raf_source::test_crate::relpath::an_absolute_input_is_refused_beside_its_relative_form` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::a_cycle_is_reported_before_a_glob_that_names_nothing` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::a_cycle_of_globs_stays_cyclic_beside_globs_that_resolve` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::a_glob_of_a_missing_local_module_loads_and_fails_each_lookup_that_reads_it` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::a_glob_the_index_cannot_look_into_leaves_a_lookup_not_found` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::a_name_found_through_a_glob_beside_a_glob_that_names_nothing_fails` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::a_path_keyword_in_any_position_is_invalid_beside_the_canonical_path` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::globs_of_dependencies_in_one_module_leave_a_missing_name_not_found` | test | [Verification](#verification) |
| `raf_source::test_crate::source_text::a_budget_refusal_in_a_crlf_file_names_the_same_line_as_in_an_lf_file` | test | [Verification](#verification) |
| `raf_source::test_crate::source_text::a_byte_order_mark_leaves_the_lines_of_the_text_where_they_are` | test | [Verification](#verification) |
| `raf_source::test_crate::source_text::a_byte_order_mark_on_the_first_line_starts_the_span_of_its_item_after_the_mark` | test | [Verification](#verification) |
| `raf_source::test_crate::source_text::a_module_file_that_is_not_utf8_is_refused_on_disk_naming_the_file` | test | [Verification](#verification) |
| `raf_source::test_crate::source_text::a_one_line_item_has_the_half_open_lines_n_to_n_plus_one` | test | [Verification](#verification) |
| `raf_source::test_crate::source_text::a_syntax_error_in_a_crlf_file_or_after_a_byte_order_mark_names_its_own_line` | test | [Verification](#verification) |
| `raf_source::test_crate::source_text::crlf_line_endings_give_the_same_lines_and_spans_as_lf` | test | [Verification](#verification) |
| `raf_source::test_crate::spawn_limit::a_parse_thread_the_system_refuses_a_stack_is_a_spawn_error_naming_the_file` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::a_bitflags_style_macro_type_with_an_impl_loads_with_its_members_marked_unresolved` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::a_bodiless_const_or_another_verbatim_item_is_still_a_typed_error_naming_file_and_line` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::a_first_party_test_crate_module_and_a_test_target_are_ambiguous_only_where_their_paths_meet` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::a_glob_of_a_local_path_the_index_cannot_see_is_a_candidate_that_cannot_be_confirmed` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::a_glob_of_enum_variants_beside_an_impl_loads` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::an_impl_of_a_local_trait_object_loads_with_its_members_marked_unresolved` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::an_inherent_impl_on_a_type_the_index_cannot_see_is_indexed_under_its_written_self_path` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::bodiless_functions_and_types_of_attribute_macros_are_indexed_where_they_are_written` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::cfg_twins_are_one_symbol_with_two_sorted_sites` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::every_kind_is_indexed_with_its_kind_line_file_and_visibility` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::items_on_one_long_line_of_wide_characters_get_exact_byte_offsets` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::test_functions_carry_the_package_target_and_crate_relative_name` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::two_loads_are_equal_and_items_and_targets_come_in_strictly_increasing_order` | test | [Verification](#verification) |
| `raf_source::test_crate::target_roots::a_build_script_main_beside_a_lib_module_build_script_is_ambiguous` | test | [Verification](#verification) |
| `raf_source::test_crate::target_roots::a_first_party_module_at_a_binary_crate_path_is_ambiguous_beside_a_binary_that_is_not` | test | [Verification](#verification) |
| `raf_source::test_crate::target_roots::a_lib_module_named_bin_crate_stays_in_the_library` | test | [Verification](#verification) |
| `raf_source::test_crate::target_roots::an_explicit_path_bin_beside_a_lib_module_bin_crate_is_ambiguous_in_both_file_orders` | test | [Verification](#verification) |
| `raf_source::test_crate::target_roots::an_over_cap_inferred_lib_names_discovery_not_lib_path` | test | [Verification](#verification) |
| `raf_source::test_crate::target_roots::every_target_kind_resolves_at_its_crate_path_as_a_module_item_of_its_root_file` | test | [Verification](#verification) |
| `raf_source::test_crate::this_repository::a_dependency_re_exported_by_its_crate_name_is_not_found_and_not_a_cycle` | test | [Verification](#verification) |
| `raf_source::test_crate::this_repository::an_item_of_a_module_file_that_sorts_before_its_mod_rs_is_found_under_its_own_module` | test | [Verification](#verification) |
| `raf_source::test_crate::this_repository::the_surface_of_each_library_and_binary_of_this_repository_lists_its_public_items` | test | [Verification](#verification) |
| `raf_source::test_crate::this_repository::this_repository_loads_from_its_root_with_the_crates_directory_as_the_root` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_glob_through_a_re_exported_crate_is_keyed_under_the_crate` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_macro_call_in_a_trait_body_is_skipped` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_star_heavy_member_pattern_that_matches_no_directory_leaves_the_member_alone` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_trait_found_through_twenty_thousand_chained_globs_is_keyed_on_a_two_mib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_trait_name_that_two_globs_bind_keys_each_trait` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_trait_that_several_unknown_globs_could_give_is_refused_naming_one_glob_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_trait_through_a_glob_the_importer_cannot_access_is_keyed_at_its_definition_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_trait_through_a_head_a_glob_brings_in_from_another_crate_keys_both` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_trait_through_a_single_segment_import_of_a_crate_is_keyed_at_the_crate` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_trait_through_an_ambiguous_use_head_keys_every_reading` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_use_path_of_edition_2015_is_followed_from_the_crate_root` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_use_path_of_edition_2015_names_a_dependency_only_through_extern_crate` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::an_explicit_import_of_a_dependency_trait_shadows_a_glob_of_a_local_one` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::an_import_met_again_through_a_glob_adds_nothing_and_a_pure_cycle_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::an_inherited_dependency_the_workspace_cannot_give_is_an_error_naming_the_key` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::each_prelude_trait_loads_in_the_edition_that_has_it_and_is_refused_in_the_one_before` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::inheriting_requires_membership_by_the_members_and_exclude_lists` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::the_completion_audit_shape_of_a_crate_alias_and_twenty_thousand_imports_loads_on_a_two_mib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::twenty_thousand_chained_local_aliases_load_on_a_two_mib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::tree_error::every_variant_names_its_path_in_display_and_in_path` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_crate_name_an_explicit_import_binds_is_never_read_as_the_crate_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_crate_name_in_an_import_is_the_dependency_its_target_declares` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_crate_root_inside_a_group_under_a_path_is_a_syntax_error` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_cycle_left_to_a_glob_reads_all_that_the_glob_brings_in_once_its_round_is_decided` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_cycle_of_explicit_imports_is_cyclic_whatever_its_globs_hold_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_cycle_of_imports_is_cyclic_beside_a_chain_that_ends` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_cycle_of_imports_through_a_crate_name_and_a_glob_is_cyclic_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_cycle_of_imports_with_one_glob_on_it_resolves_through_the_glob_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_cycle_resolves_through_its_glob_when_a_place_an_import_keeps_also_waits_on_a_glob` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_cycle_through_a_glob_and_a_renamed_import_is_decided_in_each_name_space_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_cycle_through_a_glob_the_importer_cannot_access_is_no_cycle_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_cycle_through_a_renamed_import_that_has_bound_nothing_is_cyclic_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_cycle_whose_import_reads_its_own_name_through_another_glob_is_cyclic_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_dev_dependency_is_named_only_by_test_code_even_under_a_normal_dependencys_name` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_glob_against_a_workspace_library_is_ambiguous_in_every_declaration_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_glob_brings_in_what_its_target_binds_later_whatever_its_target_holds_under_its_first_segment` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_glob_that_brings_in_an_import_of_another_crate_collides_with_the_crate` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_glob_the_importer_cannot_access_holds_nothing_back_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_glob_whose_own_path_starts_at_a_crate_is_ambiguous_with_a_crate_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_glob_whose_path_turns_out_ambiguous_brings_in_nothing_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_leading_colon_member_of_a_use_group_reads_the_crate_root_in_edition_2015` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_long_chain_of_re_exports_resolves_in_either_order_without_a_cap` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_name_an_import_the_reader_cannot_access_binds_is_hidden_from_it_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_name_bound_to_several_modules_gives_a_trait_key_through_each` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_name_space_binds_nothing_only_once_the_globs_its_globs_reach_are_settled` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_name_space_binds_nothing_only_once_the_globs_that_may_bring_it_in_are_settled_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_no_std_crate_does_not_name_std_without_an_extern_crate_item` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_path_through_an_import_a_cycle_leaves_undecided_in_another_name_space_is_cyclic` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_single_segment_import_of_a_dependency_is_an_external_alias_and_never_its_own_cycle` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_single_segment_import_of_a_workspace_library_binds_its_crate_root` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_three_hop_chain_resolves_and_lists_each_hop` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_use_group_with_leading_colon_members_binds_as_the_separate_uses_do` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_use_head_from_a_glob_that_is_also_an_extern_prelude_crate_is_ambiguous` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_use_path_of_edition_2015_starts_at_the_crate_root` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_explicit_import_shadows_a_workspace_library_in_every_declaration_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_explicit_import_shadows_the_glob_whether_or_not_the_index_sees_what_it_binds` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_a_cycle_leaves_to_the_globs_binds_what_its_readers_could_access_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_binds_what_a_name_it_read_gains_later_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_keeps_the_globs_out_only_in_the_name_spaces_it_binds_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_of_a_value_named_like_a_crate_leaves_the_crate_and_the_globs_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_of_its_own_module_reads_what_the_globs_bring_in_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_that_binds_its_name_in_one_name_space_leaves_the_others_to_the_globs` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_that_bound_a_name_before_the_crates_opened_binds_the_crate_later_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_that_reads_a_name_only_through_a_glob_it_cannot_access_finds_nothing_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_through_an_import_that_keeps_its_name_keeps_its_own_from_the_globs_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_whose_glob_reads_it_starts_at_the_crate_once_no_glob_may_bring_the_name_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_item_a_glob_reaches_by_a_public_and_a_private_route_is_visible_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::cycles_through_private_globs_resolve_as_each_reader_can_access_them_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::globs_that_glob_one_another_beside_names_bound_twice_settle_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::imports_that_read_a_name_through_a_glob_agree_with_the_glob_on_an_ambiguous_re_export` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::imports_that_wait_on_one_another_through_globs_are_decided_as_rustc_does_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::imports_whose_heads_are_names_their_globs_bring_in_are_decided_alike_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::missing_names_dependency_items_and_macro_generated_items_are_not_found` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::resolution_does_not_depend_on_the_order_of_the_declarations` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::the_rebuilt_globs_read_each_glob_path_in_the_settled_tables_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::two_libraries_with_one_crate_name_are_ambiguous_until_the_package_is_named` | test | [Verification](#verification) |
| `raf_source::test_crate::walk_stack::a_directory_at_the_depth_limit_is_walked_and_one_below_it_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::walk_stack::a_disk_chain_of_directories_is_refused_typed_on_a_two_mib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::walk_stack::a_member_pattern_at_the_depth_limit_expands_and_one_deeper_is_refused_typed` | test | [Verification](#verification) |
| `raf_source::test_crate::walk_stack::a_memory_chain_of_twenty_thousand_directories_is_refused_typed_on_a_two_mib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::walk_stack::a_use_tree_at_the_recursion_limit_loads_resolves_and_drops_on_a_five_hundred_twelve_kib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::walk_stack::one_hundred_twenty_eight_nested_modules_load_on_a_five_hundred_twelve_kib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_containment::a_file_reached_through_a_directory_symlink_is_one_file_under_the_spelling_that_sorts_first` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_containment::a_manifest_target_path_that_leaves_the_root_is_refused_naming_the_key` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_containment::a_module_that_reaches_its_own_file_through_a_directory_symlink_is_a_cycle` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_containment::a_symlinked_module_file_to_outside_is_refused_naming_the_module_beside_an_in_root_link` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_containment::a_tree_with_windows_device_names_loads_off_windows` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_containment::discovery_classifies_a_symlink_by_what_it_resolves_to` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_containment::symlink_aliases_of_one_directory_parse_it_once` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_seeds::every_load_workspace_seed_loads_with_items_through_the_fuzz_input_code` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_seeds::the_self_check_the_fuzz_target_runs_first_passes_beside_an_input_that_is_not_a_tree` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::a_missing_descriptor_link_is_an_io_failure_and_never_absence` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::linux_verifier_reads_the_real_descriptor_of_an_open_file` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::linux_verifier_refuses_a_handle_link_that_leads_outside_and_accepts_one_inside` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::reserved_device_names_and_trailing_dots_or_spaces_are_ambiguous_on_windows_beside_plain_names` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::a_link_that_climbs_back_into_the_root_is_read_only_by_canonicalisation` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::a_path_three_hundred_deep_is_walked_under_a_limit_of_sixty_four_descriptors` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::a_symlink_that_becomes_a_directory_during_a_step_is_stepped_again` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::every_open_that_resolves_a_name_is_a_reference_or_does_not_block` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::every_strategy_follows_an_absolute_in_root_link_spelled_through_a_linked_parent` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::every_strategy_refuses_an_outside_symlink_and_reads_in_root_links` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::every_strategy_reports_entry_kinds_and_refuses_an_outside_directory` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::links_into_the_outside_are_answered_alike_only_by_the_walk` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::swap::a_directory_swapped_after_verification_before_the_entries_are_read_is_refused` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::swap::a_directory_swapped_for_an_outside_symlink_after_the_check_is_refused` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::swap::a_directory_swapped_for_an_outside_symlink_before_a_listing_is_refused` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::swap::a_file_whose_directory_is_moved_out_after_verification_is_refused_after_the_read` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::swap::a_swap_before_the_open_undone_before_the_check_is_refused` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::swap::a_swap_flipped_at_every_hook_point_is_refused` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::swap::an_entry_whose_directory_is_swapped_for_an_outside_symlink_is_refused` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::swap::an_outside_fifo_reached_by_a_swap_before_the_open_is_never_opened` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::the_handle_path_verifier_refuses_a_path_outside_the_root_beside_one_inside` | test | [Verification](#verification) |
<!-- raf:generated:end -->
