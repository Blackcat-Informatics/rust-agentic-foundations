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
    - raf_source::SelfType
    - raf_source::UnresolvedImpl
    - raf_source::Workspace::resolve_in_package
    - raf_source::Workspace::unresolved_impls
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
    - raf_source::test_crate::discovery_targets::a_manifest_over_the_cap_is_refused_naming_it_and_one_of_exactly_the_cap_loads
    - raf_source::test_crate::module_tree::a_path_attribute_in_a_mod_rs_file_is_relative_to_that_files_directory
    - raf_source::test_crate::module_tree::every_cfg_attr_path_alternative_is_followed_carrying_its_predicate
    - raf_source::test_crate::module_tree::a_module_cycle_is_an_error_naming_both_files
    - raf_source::test_crate::module_tree::a_path_that_leaves_the_tree_is_refused_naming_module_and_attribute_without_reading_it
    - raf_source::test_crate::module_tree::the_cfg_test_matrix_tags_items_and_leaves_their_neighbours_alone
    - raf_source::test_crate::module_tree::a_chain_of_files_that_each_declare_the_next_twice_is_refused_quickly_beside_a_small_diamond
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
    - raf_source::test_crate::overflow_harness::composites_and_uncounted_shapes_end_in_exactly_one_allowed_outcome
    - raf_source::test_crate::overflow_harness::the_parse_thread_is_named_after_the_file_and_has_the_stated_stack
    - raf_source::test_crate::overflow_harness::the_classifier_accepts_only_a_signal_a_stack_overflow_status_or_the_runtime_message_as_an_abort
    - raf_source::test_crate::corpus_replay::every_corpus_file_ends_in_exactly_one_allowed_outcome_with_its_recorded_symbols
    - raf_source::test_crate::read_cap::disk_tree_enforces_the_cap_at_eight_mib
    - raf_source::test_crate::read_cap::the_default_cap_is_eight_mib
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
    - raf_source::tree::disk::tests::swap::a_swap_before_the_open_undone_before_the_check_is_refused
    - raf_source::tree::disk::tests::swap::a_swap_flipped_at_every_hook_point_is_refused
    - raf_source::tree::disk::tests::strategy::every_strategy_follows_an_absolute_in_root_link_spelled_through_a_linked_parent
    - raf_source::tree::disk::tests::strategy::links_into_the_outside_are_answered_alike_only_by_the_walk
    - raf_source::tree::disk::tests::strategy::a_link_that_climbs_back_into_the_root_is_read_only_by_canonicalisation
    - raf_source::tree::disk::tests::strategy::a_symlink_that_becomes_a_directory_during_a_step_is_stepped_again
    - raf_source::tree::disk::tests::strategy::a_path_three_hundred_deep_is_walked_under_a_limit_of_sixty_four_descriptors
    - raf_source::tree::disk::tests::windows::the_final_path_of_a_handle_is_its_canonical_path_and_lies_under_the_root
    - raf_source::tree::disk::tests::strategy::every_strategy_refuses_an_outside_symlink_and_reads_in_root_links
    - raf_source::tree::disk::tests::strategy::every_strategy_reports_entry_kinds_and_refuses_an_outside_directory
    - raf_source::tree::disk::tests::reserved_device_names_and_trailing_dots_or_spaces_are_ambiguous_on_windows_beside_plain_names
    - raf_source::tree::disk::tests::windows::a_disk_tree_refuses_a_reserved_device_name_or_a_trailing_dot_or_space_beside_a_plain_name
    - raf_source::test_crate::workspace_containment::a_tree_with_windows_device_names_loads_off_windows
    - raf_source::error::tests::a_spawn_error_displays_the_file_and_carries_its_cause
    - raf_source::item::tests::an_item_record_is_at_most_two_hundred_fifty_six_bytes
    - raf_source::facts::tests::a_column_is_turned_into_its_byte_offset_on_short_long_and_wide_lines
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
    - raf_source::test_crate::symbol_index::items_on_one_long_line_of_wide_characters_get_exact_byte_offsets_in_linear_time
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
  examples: []
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
resident. Three byte budgets bound one load. Each is a default that `LoadOptions` can
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
  charged: a chain of files that each declare the next twice keeps 1646 bytes of index
  for each arrival and is charged 3693, nested inline modules whose scopes hold one
  name keep 1499 bytes for each module and are charged 2420. An item record is at most
  256 bytes.
- `max_fact_bytes` (`FactBytes`, 256 MiB) bounds the facts of the files. Each fact
  charges its size and the room of every string and list it owns (each is trimmed
  to its length as it is built), on the parse thread, so an oversized file is
  refused at the line of the fact that passes the budget.

A load over a budget is a `Budget` error that names the budget, the limit that
applied and the file, with the line where there is one. `Workspace::usage` reports
what a load spent against each. Two figures follow, measured in a release build:

1. Retained memory, apart from the scope tables of imports. What a load keeps of
   its index, its source text and its facts is bounded by the item bytes (512 MiB),
   the source text (`max_total_bytes`, 256 MiB by default, which a caller may raise)
   and the fact bytes (256 MiB): 1 GiB at the defaults. 300000 unit structs of their
   own names in three files load in 0.70 s at a peak of 380 MiB on the heap, and
   400000 are refused by the item-bytes budget in 1.1 s; thirty files of 4 MiB of doc
   comments made of quotes, which double when escaped, hold 253 MB of facts and load
   in 1.8 s at a peak of 382 MiB, and forty are refused by the fact-bytes budget in
   1.8 s; 31 files of 8 MiB load in 0.30 s, and 33 are refused by the total-bytes
   budget in 0.30 s. A load near all three limits at once (2000 copies of a chain of
   100 nested inline modules, 20 files of 8.3 MB of doc comments in lines of 300
   characters and 7 files of 8 MB of comments; 214, 466 and 244 MiB charged) keeps 746
   MiB and peaks at 752 MiB, in 10 to 15 s (three runs on a shared host).
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

Apart from the scope tables of imports, a load holds at most figure (1) and one
file's figure (2) at once: about 4.1 GiB at the defaults, each term a limit of
`LoadOptions` that a caller sets to fit its host. The scope tables that `use`
declarations and globs build are bounded by no budget: their size grows with the
modules that glob a module times the names it holds, so a glob-heavy input can
exhaust a machine whatever the limits. One module of 3000 functions that 3000 other
modules glob, 166 KB of source and 66005 tokens, loads in 21 s at a peak of 7.3 GiB
on the heap and 9.2 GiB resident, while every budget stays below 14 MiB.

Parsing then runs on a dedicated thread, named after the file, with a stack of 64
MiB (`PARSE_STACK_BYTES`), which is a stated constant and not the stack of the
caller. `parse_source` runs a visitor on the syntax tree on that thread and
returns what the visitor returns. A committed fuzz
target, `parse_source` in `fuzz/fuzz_targets/parse_source.rs`, feeds arbitrary bytes
through the pre-scan and the parse. Its corpus, `fuzz/corpus/parse_source/`, is
committed and holds the composite and single-shape fixtures of Verification below,
files at and just over each budget, and every input that ever aborted a run. These
measures make a stack overflow unlikely. They do not prove its absence: the budgets
multiply across nesting (a run of prefix operators is legal inside every delimiter
level), some recursive shapes are counted by no budget, and a level of a nested
`return return ...` chain costs about 8.8 KiB of stack in a debug build and about
3.7 KiB in a release build, measured. The crate does not claim that an admitted file cannot
overflow. A residual overflow aborts the process, so the run fails and is never
reported as a pass. No budget counts the committed composite shapes or the uncounted
shapes that repeat `return `, `&mut `, `*const `, `fn() -> ` or `a = ` 20000 times, so a
file of one of them can end in that abort. Shapes that try to slip past a counter (a block as the condition of each
`else if`, a lifetime between reference marks) are counted by the definitions above
and refused. A parser overflow is a robustness bug, and an input that causes
one is added to the fuzz corpus and to the budgets. `Cargo.toml` is read with the same
size cap, and the `toml` parser's own recursion limit turns deep nesting into an error
and not a crash. Every refusal the pre-scan makes is a typed error and not a panic.
The crate sets no time bound.

**Package discovery.** `Workspace::load(tree, roots)` takes explicit roots; the
crate has no default roots, and each caller names its own (`"."` is the tree root).
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
package name, its Cargo name, its kind, its root file, a shipping flag and a crate
path. The library is `[lib]` or `src/lib.rs`, named by `[lib] name` or the package
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
source file once with its text, its facts and the module paths it was reached by. The
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
of its name, its `span` and `body` in lines and bytes, its `visibility`, `documented`,
`doc_hidden`, `test` role, `target` and, for an impl member, the written `self_type`
and `trait_path` and the `self_key` that says how its self type placed it (`Local`,
`Trait` or `Unresolved`). The module path in an item's `path` is the chain of `mod`
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
allowed (`crates/*/` is `crates/*`). An entry
of `workspace.members` that has no wildcard names one directory, which must exist and have
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
position, strictly increasing, so the items under a path prefix are a contiguous run
(`items_under`); `items_by_source()` and `same_source(&item)` group the items that share
one identity, so a census can count a source item once however many paths reach it;
`item(id)` is a lookup; `items_in(file)` yields a file's items in order
of position; `item_at(file, offset)` yields the innermost items that hold the offset;
`is_shipping(&item)` is true for an item of a shipping target that is not test code;
`files()` yields each source file once, with its package, the targets and module paths
that reach it, and whether it ships; `source(file)` is the exact text that was indexed;
and `test_fns()` returns, for each `#[test]` function (or a typed error if one's path does not start with its target's crate path), the package, the kind and Cargo
name of its target (`--lib`, `--bin <name>` or `--test <name>`) and its path inside the
crate, which is what `cargo test -p <package> --test <target> <name>` needs;
`usage()` gives what the load spent against its budgets: the bytes read, charged to
the index and to the facts, the most tokens of one file and the deepest directory
entered. The index
has no order that depends on the tree implementation: two loads of one tree are equal,
and the same regular files on disk and in memory give equal indexes.

**Use-alias index.** Every `use` declaration contributes to a table of the names of its
module: renames, nested groups, `self` in a group, globs, a leading `::` (a crate of the
extern prelude from edition 2018 on, the crate root in edition 2015), `extern crate x as
y`, and the keywords `crate`, `self` and `super`; a `super` above the crate root is an
error that names the file and line. From edition 2018 on, a path whose first segment the
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
`extern crate std;` at the root adds it back. The import binds nothing and keeps its
name indeterminate, so no glob of its module answers for it, and a path through it is
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
cycle of globs adds nothing. A name space where an import's path reached nothing,
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
imports in the cycle, that place is left to the glob for the paths that read it, though
not yet for the globs of other modules, and the cycle resolves through it. An import
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
imports. A path through a name that an import bound in one name space while a cycle
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
path may end at a method: `demo::Thing::new`. The result is a `Resolution`: the
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
inherent impl the index cannot see, and `Invalid` when the text is not a path. A symbol
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

## Constraints

- Parsing is syntactic. The crate does not expand macros, evaluate `cfg` other than
  the `test` tag, or run type inference.
- Every opened path must be, after canonicalisation, inside the canonical workspace
  root and a regular file; the same rule bounds explicit `Cargo.toml` target paths.
- A file larger than `max_file_bytes` (default 8 MiB), or one that exceeds a
  budget of the pre-scan (128 delimiter levels, 64 nested `<`, 256 prefix
  operators, 128 closure bars, 256 `else if` links, 2000000 tokens by default), or a
  module walk that passes 128 nested modules or 262144 module walks in one load, or a
  directory
  more than 128 levels below its root, or a load that passes its total, item or fact
  bytes (256 MiB, 512 MiB and 256 MiB by default), is refused with a typed error.
  Parsing runs on a dedicated 64 MiB stack. A residual overflow aborts the run and
  fails the gate; it is never a pass. No budget counts the composite shapes or most of
  the uncounted shapes of Verification, so a file of one of them can end in such an abort.
- The scope tables that `use` declarations and globs build are bounded by no
  budget, so a glob-heavy input can take more memory than the figures of the
  Contract state.
- Resolution covers items the source defines. Items from dependencies are outside the
  index and resolve to a not-found failure.
- Two visibility rules of rustc are not modelled. An explicit `pub use` of a binding
  that its module can access only through a private glob binds the name as visible as
  the import, where rustc keeps it no more visible than the binding it re-exports, so a
  module that globs it can read a name that rustc hides from it. And a name that the
  globs of a module bring in ambiguously, one of the items through a glob that a
  reading module cannot access, is the other item for that module, where rustc refuses
  the path, usually as an ambiguity (E0659) and sometimes as a name that is not there
  (E0573).
- The index is built once per `Workspace::load`; callers reload to see changed files.
- Output order is sorted by path, so two loads of the same tree give equal results.

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
  recorded under its crate path.
- Discovery fixtures: a virtual manifest contributes nothing; a manifest with a
  trailing comment and single-quoted values is honoured; no roots find nothing; a
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
  declare the next twice (2^31 paths) refused by the module-path budget, within 30 s in a
  release build, beside a small diamond, and a first-party
  `mod test_crate;` beside a test target.
- Placement fixtures, each asserting every item path: a child file that sorts before
  the file that declares it (`a/mod.rs` with `a/b.rs`, and `a.rs` with `a/b.rs` below a
  directory module that sorts before the crate root); the children `a`, `ga` and `z` of
  a `#[path = "gen/generated.rs"]` file; and `tests/common/mod.rs` with
  `tests/common/a.rs` reached from `tests/zz.rs`. `super::super` from such a child
  reaches the crate root, and `demo::a::in_b` and `demo::a::b::in_a` are `NotFound`
  beside `demo::a::b::in_b`. In this repository `raf_okf::body::blocks::FenceBlock`
  resolves and `raf_okf::body::FenceBlock` is `NotFound`, and the document-contract
  test rejects a binding to the same misplaced shape. `raf_source::syn` and
  `raf_source::syn::File`, through `pub use syn;`, are `NotFound` beside
  `raf_source::DiskTree`, which resolves.
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
  index they keep), and
  the facts, over several files and for each large payload: a 1 MiB doc comment, a 1
  MiB written self type and a `use` group of long names, each refused at its own line
  below its size. `usage()` reports the spent bytes, the most tokens and the deepest
  directory, and an item record is at most 256 bytes.
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
  offsets within the time a linear walk takes.
- Composite and uncounted-shape fixtures: `!` repeated 256 times before `(` repeated
  across 127 levels; `&` repeated 256 times before `(` across 127 levels, in a type;
  `&&` repeated 256 times before `(` across 127 levels; `A<` followed by 256 `&`,
  repeated across 64 levels; `||` repeated 128 times before `(` across 127 levels;
  and 20000 repetitions each of `return `, `&mut `, `*const `, `fn() -> `, `a = ` and
  `move || `. A harness runs each in a child process, in debug and in release, and
  asserts that it ends in exactly one of three outcomes: the pre-scan refuses it with
  a typed error that names the file and the budget; the process exits 0 with a
  correct, complete index (the symbols of the fixture, compared with the expected
  list); or the process aborts with a non-zero status. Exit 0 with a wrong or partial
  index, or with an index produced after an abort, fails the fixture.
- Evasion fixtures: an `if` followed by 20000 `else if {true} {}` links, and
  `type T<'a> = &'a &'a ... u8;` with 20000 reference marks, each fail with the typed
  budget error that names the file, in process and in the child-process harness; the
  same shapes at 256 links and 256 marks parse.
- A syntax error carries the line and column of the parser's error: a fixture with the
  error on line 3 and another on line 5 assert the exact position, beside a file that
  parses.
- The harness judges strictly: a child that panics, one given an unknown selector, and
  one that exits 3 without a result line each fail the fixture. An abort is accepted
  only as death by a signal, the Windows stack-overflow status, or the runtime's own
  stack-overflow message.
- The parse thread is named after the file it parses, so a stack-overflow message
  from the runtime names the culprit. A fixture asserts the thread name for a file
  and that the abort message of the overflow fixtures contains that name.
- Corpus replay under `cargo test`: a test parses every file of
  `fuzz/corpus/parse_source/` in a child process and asserts that each ends in one of
  the three outcomes above, so a regression is caught without running the fuzzer.
- The fuzz target `parse_source` builds and runs for a bounded number of iterations
  over its corpus without a crash. That run cuts inputs to 4096 bytes, because an
  abort would end the fuzzer; the larger corpus files are judged by the replay test
  in child processes.
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
  that name, and that one that did not resolve reaches nothing.
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
