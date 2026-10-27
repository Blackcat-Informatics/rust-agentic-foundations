---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:source-index
title: "Source index"
description: "The raf-source crate: a Tree abstraction, package discovery, mod-tree following with #[path], and a symbol and use-alias index."
state: implementing
bindings:
  code:
    - raf_source::cargo_version
    - raf_source::resolve
    - raf_source::Tree
    - raf_source::discover_root
    - raf_source::DiskTree
    - raf_source::DiskTree::read_bytes
    - raf_source::DiskTree::read_unseen_bytes
    - raf_source::OpenedBytes
    - raf_source::DiskTree::capture_file
    - raf_source::DiskTree::capture_directory
    - raf_source::DiskTree::capture_link
    - raf_source::CapturedFile
    - raf_source::CapturedDirectory
    - raf_source::CapturedLink
    - raf_source::tree::disk
    - raf_source::MemoryTree
    - raf_source::tree::memory
    - raf_source::RelPath
    - raf_source::Workspace
    - raf_source::Workspace::item_count_in
    - raf_source::Workspace::code_resolver
    - raf_source::CodeResolver
    - raf_source::CodeResolveError
    - raf_source::CodeResolver::extern_libraries_at
    - raf_source::CodeResolver::with_configured_syntax
    - raf_source::CodeResolver::require_unconditional_inherent_method
    - raf_source::ExternLibrary
    - raf_source::ExternRouteKind
    - raf_source::Workspace::load_locked_sources_under
    - raf_source::code
    - raf_source::visit_pattern_bindings
    - raf_source::CompilerPlatform
    - raf_source::CompilerPlatformError
    - raf_source::TestPlatformGuard
    - raf_source::Workspace::test_platform_guard
    - raf_source::parse_source
    - raf_source::Resolution
    - raf_source::ResolveError
    - raf_source::LoadOptions
    - raf_source::Item
    - raf_source::Target
    - raf_source::FileId
    - raf_source::tree
    - raf_source::cost::shared_text
    - raf_source::Usage
    - raf_source::Budget
    - raf_source::Opened
    - raf_source::PARSE_BYTES_PER_TOKEN
    - raf_source::RECURSION_UNIT_BYTES
    - raf_source::SelfType
    - raf_source::UnresolvedImpl
    - raf_source::Workspace::resolve_in_package
    - raf_source::Workspace::resolve_from
    - raf_source::Workspace::resolve_from_in
    - raf_source::Workspace::resolve_from_package_id
    - raf_source::Workspace::resolve_from_package_id_in
    - raf_source::Workspace::dependency_declarations
    - raf_source::DependencyDeclaration
    - raf_source::DependencyKind
    - raf_source::DependencySpec
    - raf_source::InheritedDependency
    - raf_source::LockedCatalog
    - raf_source::LockedPackage
    - raf_source::LockedPackageId
    - raf_source::LockedSource
    - raf_source::LockedSourceKind
    - raf_source::LockedGitReference
    - raf_source::PackageId
    - raf_source::SourceAuthorityId
    - raf_source::SourceTrees
    - raf_source::LockedCatalog::select_dependency
    - raf_source::RegistryIndex
    - raf_source::CommittedGitTree
    - raf_source::CommittedGitTree::open
    - raf_source::Workspace::load_locked_sources
    - raf_source::Workspace::locked_binding
    - raf_source::LockedBinding
    - raf_source::LockedManifest
    - raf_source::LockedEdge
    - raf_source::GitPackageRoots
    - raf_source::CfgAtom
    - raf_source::DeclaredCfg
    - raf_source::DeclaredCfg::complete
    - raf_source::DeclaredConfiguration
    - raf_source::ActiveConfiguration
    - raf_source::CfgSelection
    - raf_source::LockedManifest::with_configuration
    - raf_source::LockedManifest::with_configurations
    - raf_source::LockedBinding::configuration
    - raf_source::LockedBinding::configurations
    - raf_source::LockedBinding::configuration_for
    - raf_source::Workspace::resolve_active_from_package_id_in
    - raf_source::Workspace::unresolved_impls
    - raf_source::Workspace::impls
    - raf_source::Workspace::impl_block
    - raf_source::Workspace::reachable
    - raf_source::ImplBlock
    - raf_source::ImplId
    - raf_source::Node
    - raf_source::Edge
    - raf_source::Reached
    - raf_source::Reachability
    - raf_source::ReachError
    - raf_source::Workspace::exported_items
    - raf_source::Workspace::exported_items_with
    - raf_source::Exports
    - raf_source::ExportedItem
    - raf_source::Reexport
    - raf_source::ExportOptions
    - raf_source::ExportError
    - raf_source::Item::parameter_bindings
    - raf_source::Item::initializer
    - raf_source::DEFAULT_MAX_FILE_BYTES
    - raf_source::Entry
    - raf_source::EntryKind
    - raf_source::FieldShape
    - raf_source::ItemId
    - raf_source::LoadError
    - raf_source::LoadErrorKind
    - raf_source::Namespace
    - raf_source::Origin
    - raf_source::PARSE_STACK_BYTES
    - raf_source::Package
    - raf_source::PathError
    - raf_source::Reach
    - raf_source::SharedStr
    - raf_source::SourceFile
    - raf_source::SpanFact
    - raf_source::SymbolKind
    - raf_source::TargetId
    - raf_source::TargetKind
    - raf_source::TargetProblem
    - raf_source::TestFn
    - raf_source::TestRole
    - raf_source::TraitKey
    - raf_source::TreeError
    - raf_source::Vis
    - raf_source::item
    - raf_source::settle
    - raf_source::work
    - raf_source_fuzz::bin_crate::load_workspace::exercise
  tests:
    - raf_source::exports::tests::native_surface_storage_preserves_physical_failure_and_budget_ownership
    - raf_source::glob_scope::delivery::maximal_public_replacements_avoid_repeat_proofs_while_narrower_ones_are_widened
    - raf_source::glob_scope::delivery::native_delivery_growth_admits_old_and_new_spill_buffers_together
    - raf_source::glob_scope::delivery::native_delivery_spill_is_lifo_and_failed_push_releases_only_the_incoming_delta
    - raf_source::glob_scope::delivery::one_returning_carrier_retains_multiple_independently_selected_identities
    - raf_source::glob_scope::delivery::recorded_name_hash_storage_loads_at_exact_item_and_glob_grants_and_refuses_one_less
    - raf_source::glob_scope::delivery::recorded_name_rows_survive_real_reconstruction_with_new_equal_interned_names
    - raf_source::glob_scope::delivery::widest_routes_merge_firm_evidence_in_place_and_refuse_before_mutation
    - raf_source::glob_scope::equivalence::an_answer_that_differs_between_the_two_modes_is_reported_with_its_load_and_both_answers
    - raf_source::glob_scope::equivalence::the_settling_gives_the_answers_of_the_reference_mode_to_every_generated_and_corpus_program_in_three_orders
    - raf_source::glob_scope::incremental::a_covert_read_is_listed_before_its_names_arrive_so_an_item_its_view_gains_while_they_arrive_reaches_the_module
    - raf_source::glob_scope::incremental::a_covert_reader_is_woken_in_the_turn_an_active_read_of_it_would_have_so_the_imports_that_wait_on_the_name_are_queued_again_in_that_order
    - raf_source::glob_scope::incremental::a_covert_reader_takes_a_gain_in_the_turn_an_active_read_of_it_would_have_so_the_companions_of_a_name_are_kept_as_its_drops_allow
    - raf_source::glob_scope::incremental::a_module_whose_covert_read_turns_active_is_no_longer_looked_at_as_a_covert_reader_when_the_view_grows
    - raf_source::glob_scope::incremental::a_view_made_when_its_module_has_left_the_open_readers_makes_it_open_again_until_it_learns_what_it_cannot_access
    - raf_source::glob_scope::incremental::an_item_its_class_cannot_access_that_arrived_under_a_name_its_view_did_not_hold_makes_the_first_item_it_can_access_ambiguous
    - raf_source::glob_scope::incremental::an_item_that_a_view_read_through_a_glob_its_class_cannot_access_holds_makes_the_first_item_it_can_access_under_the_name_ambiguous
    - raf_source::glob_scope::lookups::a_query_after_the_load_is_charged_its_own_lookup_steps_against_the_limit_of_one_query
    - raf_source::glob_scope::lookups::every_glob_read_a_lookup_of_the_load_looks_at_is_charged_once
    - raf_source::glob_scope::lookups::every_limit_below_the_steps_of_a_query_refuses_it_and_the_refusal_comes_before_any_answer
    - raf_source::glob_scope::lookups::every_limit_below_the_work_of_the_lookups_shape_is_refused_by_the_work_budget_and_by_nothing_else
    - raf_source::glob_scope::lookups::every_query_counts_the_lookups_of_its_walks_and_of_the_views_it_looks_at_to_find_them
    - raf_source::glob_scope::lookups::every_way_of_resolving_a_path_is_a_query_with_its_own_limit
    - raf_source::glob_scope::lookups::queries_on_other_threads_each_count_from_zero
    - raf_source::glob_scope::lookups::the_lookups_of_many_named_imports_through_many_globs_are_charged_and_refused_by_a_small_limit_at_the_first_checkpoint_past_it
    - raf_source::glob_scope::lookups::the_public_surface_is_one_query_whose_lookups_and_those_of_the_paths_it_resolves_share_its_limit
    - raf_source::glob_scope::monotone::an_entry_held_as_a_companion_is_widened_as_it_is_added_again_and_made_plain_only_by_a_plain_entry
    - raf_source::glob_scope::monotone::an_entry_that_makes_a_plain_item_a_companion_again_is_refused
    - raf_source::glob_scope::monotone::an_entry_that_narrows_the_level_an_item_is_held_at_is_refused
    - raf_source::glob_scope::ranked::tests::a_complete_definition_and_relay_rank_is_already_its_full_closure
    - raf_source::glob_scope::ranked::tests::a_mixed_relay_and_a_removed_anchor_match_actual_reconstruction
    - raf_source::glob_scope::ranked::tests::absent_cells_and_ungrounded_cyclic_residents_cannot_pass_closure_or_rank
    - raf_source::glob_scope::ranked::tests::an_exact_empty_namespace_proves_bottom_without_any_transfer
    - raf_source::glob_scope::ranked::tests::an_impossible_aggregate_reservation_is_a_typed_layout_refusal_before_allocation
    - raf_source::glob_scope::ranked::tests::an_incomplete_successful_rank_still_checks_its_later_complete_input
    - raf_source::glob_scope::ranked::tests::complete_public_peer_discovery_omits_only_its_redundant_postpublication_pass
    - raf_source::glob_scope::ranked::tests::discovery_independence_scans_preserve_the_original_work_refusal
    - raf_source::glob_scope::ranked::tests::homogeneous_returning_carriers_share_an_actual_holder_retrieval
    - raf_source::glob_scope::ranked::tests::interior_levels_named_imports_and_glob_prefixes_keep_ordinary_reconstruction
    - raf_source::glob_scope::ranked::tests::named_extern_alias_and_qualified_glob_neighbors_do_not_certify_independence
    - raf_source::glob_scope::ranked::tests::proof_scratch_and_work_refuse_one_less_than_the_actual_original_grant
    - raf_source::glob_scope::ranked::tests::public_peer_rows_are_rooted_without_a_second_table_construction
    - raf_source::glob_scope::ranked::tests::roots_outside_resident_targets_still_require_the_second_discovery
    - raf_source::glob_scope::settling::a_companion_fan_in_keeps_each_operation_population_linear_without_list_copies
    - raf_source::glob_scope::settling::a_companion_hub_bounds_delivery_and_carrier_graph_growth_together
    - raf_source::glob_scope::settling::a_companion_hub_keeps_local_definitions_and_reads_without_copying_candidate_lists
    - raf_source::glob_scope::settling::a_reading_again_looks_at_the_reads_whose_view_holds_the_name_whatever_the_reads_of_the_module
    - raf_source::glob_scope::settling::named_import_lookup_routes_stay_linear_and_read_work_stays_quadratic
    - raf_source::glob_scope::settling::the_settling_of_a_chain_of_globs_counts_its_pinned_polynomials_at_n_2n_and_3n
    - raf_source::glob_scope::settling::the_settling_of_a_fan_in_of_private_globs_counts_its_pinned_polynomials_at_n_2n_and_3n
    - raf_source::glob_scope::settling::the_settling_of_a_fan_in_of_pub_globs_counts_its_pinned_polynomials_at_n_2n_and_3n
    - raf_source::glob_scope::settling::the_settling_of_a_hub_counts_its_pinned_polynomials_at_n_2n_and_3n
    - raf_source::glob_scope::settling::the_settling_of_a_module_of_many_items_globbed_from_many_modules_counts_its_pinned_polynomials_at_n_2n_and_3n
    - raf_source::glob_scope::settling::the_settling_of_a_module_whose_names_many_views_hold_counts_its_pinned_polynomials_at_n_2n_and_3n
    - raf_source::glob_scope::settling::the_settling_of_the_prelude_pattern_counts_its_pinned_polynomials_at_n_2n_and_3n
    - raf_source::glob_scope::table_bytes::each_covert_reader_open_reader_and_view_read_through_a_glob_a_class_cannot_access_is_charged_its_entry_and_the_first_room_of_its_table
    - raf_source::glob_scope::table_bytes::each_name_a_view_dropped_and_each_name_a_reading_again_set_is_charged_its_entry_and_the_first_room_of_its_record
    - raf_source::glob_scope::table_bytes::each_name_a_view_takes_is_charged_its_slot_among_the_holders_of_the_name_and_the_first_companion_its_entry
    - raf_source::glob_scope::table_bytes::each_read_of_a_view_is_charged_its_place_among_the_reads_of_its_module_of_that_view
    - raf_source::glob_scope::table_bytes::the_tables_of_the_views_that_hold_each_name_and_of_the_names_with_a_companion_are_charged_their_first_room
    - raf_source::glob_scope::tests::holder_visibility_uses_ancestry_instead_of_sibling_depth
    - raf_source::glob_scope::tests::independent_same_item_supply_preserves_the_carriers_ambiguity
    - raf_source::glob_scope::tests::named_supplier_levels_stay_with_the_declaration_that_provides_them
    - raf_source::glob_scope::tests::native_resumed_growth_admits_overlapping_buffers_before_mutation
    - raf_source::glob_scope::work_budget::a_new_view_charges_raw_reads_and_unique_source_names_before_gathering
    - raf_source::glob_scope::work_budget::actual_work_from_100_to_800_stays_within_each_shapes_doubling_bound
    - raf_source::glob_scope::work_budget::an_export_read_charges_each_visited_read_and_holder_before_returning_its_answer
    - raf_source::glob_scope::work_budget::holder_scan_sources_beyond_the_admitted_growth_sizes_are_refused
    - raf_source::impls::tests::alternate_aliases_spill_the_nominal_stack_under_its_original_grant
    - raf_source::impls::tests::missing_impl_head_admits_pending_glob_spill_and_keeps_the_missing_neighbor
    - raf_source::meter::tests::native_emission_spill_preserves_cumulative_item_admission
    - raf_source::meter::tests::native_graph_storage_preserves_overlapping_grants_and_typed_refusals
    - raf_source::meter::tests::retained_payload_moves_its_original_grant_without_readmission
    - raf_source::meter::tests::transient_reservations_refuse_atomically_and_release_on_every_exit
    - raf_source::reach::tests::code_prefix_reads_share_one_query_account_and_refuse_one_less
    - raf_source::resolve::determinacy::tests::native_query_layout_failure_is_reported_as_storage_without_allocating
    - raf_source::resolve::determinacy::tests::selected_path_proof_spills_on_real_routes_and_releases_its_scratch
    - raf_source::resolve::determinacy::tests::selected_query_and_surface_proofs_admit_exact_work_and_release_all_scratch
    - raf_source::resolve::determinacy::tests::selected_routes_admit_the_first_native_spill_exactly_and_refuse_before_mutation
    - raf_source::settle::replay::genuine_paths_are_fixed_but_glob_prefixes_and_named_shadows_require_replay
    - raf_source::settle::replay::path_visibility_and_unmatched_or_open_resident_targets_keep_replay
    - raf_source::settle::tests::a_glob_prefix_bound_in_type_still_waits_for_its_named_import
    - raf_source::settle::tests::excluding_ones_own_named_binding_reserves_the_glob_fallback_too
    - raf_source::settle::tests::native_component_grouping_refuses_one_below_its_live_storage_bound
    - raf_source::settle::tests::pending_counts_duplicate_attempts_and_refuses_before_removing_a_queued_import
    - raf_source::settle::tests::retained_import_batches_reserve_previous_results_and_new_reads_together
    - raf_source::test_crate::glob_causality::a_cycle_with_an_independent_carrier_route_keeps_its_ambiguity
    - raf_source::test_crate::glob_causality::a_final_glob_fallback_waits_for_an_outside_named_cycle
    - raf_source::test_crate::glob_causality::a_forwarded_private_import_preserves_public_peer_namespace_exports
    - raf_source::test_crate::glob_causality::a_grounded_free_namespace_releases_only_proven_empty_namespaces
    - raf_source::test_crate::glob_causality::a_private_alias_retains_a_returning_public_type
    - raf_source::test_crate::glob_causality::a_private_cut_does_not_erase_a_visible_ambiguity
    - raf_source::test_crate::glob_causality::a_private_cut_does_not_hide_a_later_visible_candidate
    - raf_source::test_crate::glob_causality::a_private_cut_does_not_waive_an_unclosed_sibling_branch
    - raf_source::test_crate::glob_causality::a_private_cut_finalizes_a_module_prefix_without_waiving_its_peer_cycle
    - raf_source::test_crate::glob_causality::a_private_cut_requires_a_closed_glob_module_prefix
    - raf_source::test_crate::glob_causality::a_private_forwarded_binding_hides_the_same_glob_item
    - raf_source::test_crate::glob_causality::a_private_glob_chain_does_not_fabricate_a_unique_item
    - raf_source::test_crate::glob_causality::a_private_glob_cut_retains_the_named_shadow_of_a_returning_namespace
    - raf_source::test_crate::glob_causality::a_private_independent_supporter_and_a_returning_cycle_keep_the_hidden_route
    - raf_source::test_crate::glob_causality::a_returning_export_ignores_only_independent_routes_below_its_visibility
    - raf_source::test_crate::glob_causality::a_settled_returning_named_peer_does_not_invent_a_namespace_export
    - raf_source::test_crate::glob_causality::a_value_import_does_not_erase_the_type_ambiguity_waiting_behind_it
    - raf_source::test_crate::glob_causality::a_visible_route_waiting_beside_a_hidden_arrival_keeps_the_ambiguity
    - raf_source::test_crate::glob_causality::an_actual_inaccessible_cut_needs_a_closed_local_candidate
    - raf_source::test_crate::glob_causality::an_ambiguous_original_import_is_refused_before_oracle_sanitization
    - raf_source::test_crate::glob_causality::an_independent_glob_does_not_shadow_an_inherited_ambiguity
    - raf_source::test_crate::glob_causality::direct_private_sources_and_missing_public_peers_do_not_export_glob_values
    - raf_source::test_crate::glob_causality::matching_private_imports_preserve_unaliased_glob_families
    - raf_source::test_crate::glob_causality::named_imports_preserve_companions_instead_of_promoting_them_to_independent_routes
    - raf_source::test_crate::glob_causality::private_forwarding_preserves_distinct_glob_ambiguity
    - raf_source::test_crate::glob_causality::rustc_refuses_the_named_import_of_the_ambiguous_type
    - raf_source::test_crate::import_determinacy::a_direct_visible_definition_makes_the_neighboring_import_determinate
    - raf_source::test_crate::import_determinacy::a_fixed_module_prefix_decides_absence_before_an_unrelated_glob_widens_a_name
    - raf_source::test_crate::import_determinacy::a_late_bound_peer_cannot_escape_through_named_and_glob_copies_or_exports
    - raf_source::test_crate::import_determinacy::a_late_namespace_refuses_the_whole_import_including_its_bound_peer
    - raf_source::test_crate::import_determinacy::a_named_import_reading_back_through_its_module_is_not_a_settled_expression
    - raf_source::test_crate::import_determinacy::a_private_type_shadow_does_not_poison_its_independent_value_glob
    - raf_source::test_crate::import_determinacy::late_visibility_refusals_and_their_neighbors_are_stable_across_declaration_orders
    - raf_source::test_crate::load_budgets::nested_definition_emission_spills_with_only_the_item_budget
    - raf_source::test_crate::load_budgets::retained_import_reads_and_blocker_cycles_obey_exact_storage_and_work_limits
    - raf_source::test_crate::load_budgets::wide_discovery_and_member_frontiers_spill_under_the_item_budget
    - raf_source::test_crate::use_resolution::a_companion_carried_through_an_independent_glob_chain_remains_ambiguous_in_every_order
    - raf_source::test_crate::use_resolution::a_crate_visible_independent_route_keeps_another_globs_ambiguity_in_every_order
    - raf_source::test_crate::use_resolution::a_hidden_companion_arriving_beside_an_independent_route_remains_ambiguous_in_every_order
    - raf_source::test_crate::use_resolution::a_private_independent_supporter_does_not_erase_companions_through_a_returning_cycle_in_every_order
    - raf_source::test_crate::use_resolution::a_returning_glob_cycle_beside_an_independent_named_route_keeps_ambiguity_in_every_order
    - raf_source::test_crate::use_resolution::an_independent_named_module_route_does_not_erase_an_ambiguous_prefix_in_every_order
    - raf_source::test_crate::use_resolution::an_independent_named_route_does_not_erase_the_companion_of_another_glob_in_every_order
    - raf_source::work::tests::the_table_holds_every_kind_once_at_the_place_of_its_counter
    - raf_source::meter::tests::native_recovery_preserves_nested_provenance_and_restores_its_own_refusal
    - raf_source::test_crate::locked_sources::explicit_first_party_roots_require_every_discovered_manifest_association
    - raf_source::test_crate::code_paths::external_library_receipts_keep_actual_names_exports_and_definition_identity
    - raf_source::test_crate::code_paths::library_receipts_preserve_effective_name_ambiguity_and_deduplicate_one_root
    - raf_source::test_crate::code_paths::external_library_query_scratch_uses_the_original_native_grants
    - raf_source::test_crate::code_paths::code_associated_calls_prefer_inherent_methods_without_erasing_symbol_inventory
    - raf_source::test_crate::code_paths::inherent_method_applicability_is_local_to_the_selected_signature
    - raf_source::test_crate::code_paths::alias_members_preserve_definition_chains_cycles_and_original_query_grants
    - raf_source::test_crate::code_paths::complete_module_paths_return_actual_type_sites_without_becoming_values
    - raf_source::test_crate::configured_targets::all_selected_target_kinds_keep_their_native_and_own_library_routes
    - raf_source::test_crate::configured_targets::missing_duplicate_and_feature_ineligible_targets_refuse_beside_complete_selection
    - raf_source::test_crate::configured_targets::shared_files_reuse_only_the_same_actual_syntax_selection
    - raf_source::test_crate::configured_targets::development_edges_require_a_real_development_or_declared_test_context
    - raf_source::test_crate::configured_targets::conditional_and_optional_edges_keep_each_targets_actual_activation
    - raf_source::test_crate::configured_targets::retained_target_lists_and_configuration_queries_spend_the_original_grants
    - raf_source::test_crate::configured_targets::configured_syntax_queries_select_the_actual_target_and_preserve_budget_causes
    - raf_source::discovery::tests::native_discovery_names_resume_retained_storage_and_refuse_the_original_grant
    - raf_source::index::tests::an_unknown_dependency_identity_is_not_an_external_or_libraryless_package
    - raf_source::test_crate::discovery_targets::independently_rooted_same_name_packages_resolve_only_by_their_selected_identity
    - raf_source::test_crate::discovery_targets::inherited_workspaces_and_renamed_path_dependencies_stay_within_each_source
    - raf_source::test_crate::locked_catalog::encoded_git_references_keep_native_decoded_identity_and_reference_kind
    - raf_source::test_crate::locked_catalog::malformed_unsupported_and_stale_evidence_has_a_valid_profile_neighbor
    - raf_source::test_crate::locked_catalog::native_catalog_storage_and_comparison_work_obey_exact_original_grants
    - raf_source::test_crate::locked_catalog::omitted_git_precision_is_not_a_wildcard_beside_the_full_revision
    - raf_source::test_crate::locked_catalog::same_name_version_sources_require_unique_complete_selection_in_either_row_order
    - raf_source::test_crate::locked_catalog::the_actual_repository_lock_resolves_all_outgoing_edges_and_native_source_precision
    - raf_source::test_crate::locked_catalog::unicode_checksum_refuses_without_panicking_beside_native_ascii_hex
    - raf_source::test_crate::symbol_index::read_only_resolution_keeps_workspace_equality_and_distinguishes_changed_definitions
    - raf_source::test_crate::trait_keys::cfg_alternatives_of_one_defining_trait_keep_one_target_identity
    - raf_source::test_crate::trait_keys::nontrait_type_hits_supply_no_trait_evidence_beside_a_real_trait_alias
    - raf_source::test_crate::use_resolution::foreign_impl_members_keep_defining_traits_distinct_under_equal_crate_paths
    - raf_source::tree::disk::tests::source_dispatch_preserves_backend_aliases_and_rejects_foreign_direct_paths
    - raf_source::tree::sources::tests::equal_relative_paths_and_backend_keys_remain_separate_sources
    - raf_source::tree::sources::tests::parent_join_and_real_directory_normalization_keep_the_selected_root
    - raf_source::tree::sources::tests::scoped_backend_ids_are_refused_before_seen_and_preserve_the_first_cause
    - raf_source::tree::sources::tests::standalone_memory_tree_refuses_another_sources_tag_before_seen_callback
    - raf_source::workspace::tests::native_package_name_index_keeps_identity_and_enforces_its_exact_grant
    - raf_source::test_crate::code_paths::code_heads_read_the_edition_2015_module_where_use_heads_read_the_crate_root
    - raf_source::test_crate::code_paths::method_and_trait_members_read_their_defining_module_instead_of_their_symbol_parent
    - raf_source::test_crate::code_paths::a_module_body_reads_itself_while_its_declaration_reads_its_parent
    - raf_source::test_crate::code_paths::code_context_tables_spend_only_the_original_load_allowance_and_preserve_usage
    - raf_source::test_crate::code_paths::exported_macros_keep_their_lexical_module_beside_their_crate_root_export
    - raf_source::test_crate::code_paths::pattern_binding_observations_exclude_names_introduced_by_guard_expressions
    - raf_source::platform::tests::parent_platform_guards_do_not_filter_the_source_inventory
    - raf_source::platform::tests::inner_crate_and_external_module_guards_are_both_observed
    - raf_source::platform::tests::nonplatform_conditions_and_cfg_attr_do_not_invent_exclusion
    - raf_source::platform::tests::compiler_platform_refuses_incomplete_or_malformed_output
    - raf_source::platform::tests::conditional_module_routes_remain_unknown
    - raf_source::platform::tests::invalid_item_is_an_actionable_error
    - raf_source::platform::tests::platform_guard_reference_storage_uses_the_remaining_item_grant
    - raf_source::platform::tests::platform_guard_work_exhaustion_is_typed_beside_a_sufficient_neighbor
    - raf_source::test_crate::rustc_oracle::diagnostic_levels_are_required_beside_a_valid_null_code
    - raf_source::test_crate::configured_sources::configured_aliases_resolve_only_active_real_definitions_and_paths
    - raf_source::test_crate::configured_sources::distinct_declared_sets_change_definitions_and_keep_their_exact_receipt
    - raf_source::test_crate::configured_sources::missing_unknown_inactive_or_incomplete_configuration_refuses_beside_valid_selection
    - raf_source::test_crate::configured_sources::target_and_host_conditions_use_the_selected_compilation_role
    - raf_source::test_crate::configured_sources::configured_syntax_and_feature_work_spend_the_original_combined_grant
    - raf_source::test_crate::configured_sources::nested_cfg_attr_boolean_guards_escaped_values_and_test_builds_select_actual_declarations
    - raf_source::test_crate::configured_sources::forwarding_default_and_inherited_feature_requirements_are_validated_at_the_actual_destination
    - raf_source::test_crate::configured_sources::cargo_feature_names_use_rust_unicode_xid_with_actual_cargo_punctuation
    - raf_source::test_crate::configured_sources::a_physical_file_cannot_silently_merge_different_active_package_contexts
    - raf_source::test_crate::configured_sources::raw_cfg_identifiers_have_one_logical_identity_and_cannot_bypass_feature_separation
    - raf_source::test_crate::configured_sources::configured_predicates_refuse_qualified_paths_beside_unqualified_names
    - raf_source::test_crate::configured_sources::empty_cfg_attr_requires_its_comma_and_boolean_literals_are_not_raw_flags
    - raf_source::test_crate::locked_sources::exact_sources_versions_and_path_edges_resolve_aliases_to_the_actual_package
    - raf_source::test_crate::locked_sources::checked_contents_do_not_let_wrong_manifest_names_or_versions_acquire_a_row
    - raf_source::test_crate::locked_sources::a_nested_same_name_and_version_cannot_replace_the_root_workspace_package
    - raf_source::test_crate::locked_sources::requested_missing_and_optional_edges_refuse_and_unrequested_edges_never_borrow_a_name
    - raf_source::test_crate::locked_sources::all_snapshots_catalog_and_walker_use_one_original_resource_grant
    - raf_source::test_crate::locked_sources::actual_version_evidence_is_required_without_changing_ordinary_missing_version_loading
    - raf_source::test_crate::locked_sources::another_checked_repository_cannot_supply_an_expected_source_by_name_or_authority
    - raf_source::test_crate::discovery_targets::relative_queries_select_the_reading_package_without_crossing_source_roots
    - raf_source::test_crate::dependency_declarations::declarations_retain_sources_flags_aliases_and_unevaluated_conditions
    - raf_source::test_crate::dependency_declarations::inheritance_preserves_both_entries_and_the_workspace_path_base
    - raf_source::test_crate::dependency_declarations::malformed_declarations_refuse_with_valid_neighbors
    - raf_source::test_crate::dependency_declarations::retained_declarations_use_the_original_item_allowance
    - raf_source::test_crate::locked_selection::rename_version_and_registry_select_only_the_owners_exact_outgoing_row
    - raf_source::test_crate::locked_selection::git_reference_kind_and_local_source_family_are_constraints_not_tree_proofs
    - raf_source::test_crate::locked_selection::workspace_inheritance_selects_its_package_and_source_without_evaluating_cfg
    - raf_source::test_crate::locked_selection::absent_mismatched_and_malformed_registry_configuration_refuses_beside_valid_indexes
    - raf_source::test_crate::locked_selection::selection_honors_work_and_retained_catalog_item_admission
    - raf_source::test_crate::locked_selection::malformed_observations_and_unknown_owners_refuse_without_an_alternate_identity
    - raf_source::test_crate::git_tree::checked_snapshot_serves_real_bytes_and_existing_loader_with_distinct_path_identity
    - raf_source::test_crate::git_tree::missing_commit_repository_and_wrong_object_kind_refuse_beside_the_exact_commit
    - raf_source::test_crate::git_tree::missing_or_substituted_blob_is_a_hard_failure_even_when_seen_would_skip_text
    - raf_source::test_crate::git_tree::malformed_tree_and_submodule_refuse_with_an_ordinary_tree_neighbor
    - raf_source::test_crate::git_tree::committed_bytes_ignore_worktree_edits_and_replacement_refs
    - raf_source::test_crate::git_tree::missing_installed_git_is_an_actionable_dependency_failure
    - raf_source::tree::git::tests::complete_directory_protocol_refuses_malformed_records_beside_exact_metadata
    - raf_source::tree::git::command::tests::protocol_size_is_checked_before_growth_and_exact_size_has_a_valid_neighbor
    - raf_source::test_crate::root::manifest_size_bound_is_enforced_beside_the_exact_limit
    - raf_source::test_crate::root::member_and_virtual_workspace_directories_resolve_without_sources
    - raf_source::test_crate::root::standalone_excluded_and_nested_packages_have_their_own_roots
    - raf_source::test_crate::root::transitive_path_dependencies_are_members
    - raf_source::test_crate::root::explicit_workspace_requires_a_safe_path_and_agreed_membership
    - raf_source::test_crate::root::malformed_manifests_and_missing_directories_fail_hard
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
    - raf_source::test_crate::discovery_targets::discovery_target_and_module_failures_precede_later_load_phases
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
    - raf_source::glob_scope::tests::the_holders_of_an_item_apart_from_each_reader_of_a_chain_of_globs_are_found_in_steps_linear_in_the_chain
    - raf_source::glob_scope::tests::the_holders_of_an_item_apart_from_readers_the_start_of_the_chain_globs_back_are_found_in_steps_linear_in_the_chain
    - raf_source::glob_scope::tests::the_holders_of_an_item_apart_from_readers_that_form_a_chain_of_their_own_are_found_in_steps_linear_in_the_chains
    - raf_source::glob_scope::tests::a_read_that_holds_no_companion_starts_no_search_for_the_holders_of_its_items
    - raf_source::glob_scope::tests::the_dominators_of_small_holder_graphs_follow_their_definition
    - raf_source::glob_scope::tests::the_dominators_of_seeded_random_graphs_follow_their_definition
    - raf_source::glob_scope::work_budget::each_shape_loads_at_its_measured_work_and_refuses_one_step_less
    - raf_source::glob_scope::work_budget::doubling_a_shape_keeps_actual_work_within_its_growth_bound
    - raf_source::glob_scope::work_budget::a_settle_heavy_shape_is_refused_at_the_charge_that_passes_a_small_limit
    - raf_source::meter::tests::work_up_to_the_limit_is_accepted_and_the_charge_that_passes_it_is_refused
    - raf_source::meter::tests::work_is_never_released_and_each_meter_counts_from_zero
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
    - raf_source::test_crate::load_budgets::glob_table_work_at_the_work_budget_loads_and_one_step_less_is_refused
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
    - raf_source::test_crate::symbol_index::parameter_bindings_are_every_bound_name_in_signature_order_beside_a_function_with_none
    - raf_source::test_crate::symbol_index::an_initializer_is_the_text_between_the_equals_and_the_semicolon_beside_a_required_constant
    - raf_source::test_crate::symbol_index::bindings_and_an_initializer_are_shared_by_every_path_that_reaches_the_file
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
    - raf_source::test_crate::target_roots::unreadable_target_roots_name_explicit_paths_or_table_names_and_build_discovery
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
    - raf_source::exports::tests::a_module_of_pub_globs_is_listed_in_view_reads_linear_in_its_names
    - raf_source::test_crate::exported_items::a_module_reached_by_a_hidden_and_a_shown_glob_carries_its_re_export_shown
    - raf_source::exports::tests::a_module_of_pub_globs_that_each_carry_a_re_export_is_listed_in_view_reads_linear_in_its_names
    - raf_source::exports::tests::a_module_of_pub_globs_that_each_carry_a_re_export_is_listed_in_walk_steps_linear_in_its_names
    - raf_source::test_crate::exported_items::a_carried_re_export_is_kept_out_exactly_where_every_route_passes_a_module_that_binds_its_name
    - raf_source::test_crate::exported_items::an_item_and_a_carried_outside_re_export_of_one_name_are_both_listed_and_not_ambiguous
    - raf_source::exports::tests::readers_of_carried_names_beside_sibling_shadows_list_in_linear_work_and_bounded_bytes
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
    - raf_source::test_crate::use_resolution::a_pub_use_of_a_name_read_through_a_private_glob_is_seen_no_wider_than_the_glob_in_every_order
    - raf_source::test_crate::use_resolution::a_chain_of_two_re_exports_through_two_private_globs_is_capped_twice_in_every_order
    - raf_source::test_crate::use_resolution::an_item_reached_by_a_capped_re_export_and_by_a_wider_glob_is_seen_as_widely_as_the_wider_in_every_order
    - raf_source::test_crate::use_resolution::a_glob_re_export_of_a_private_item_is_seen_no_wider_than_the_item_in_every_order
    - raf_source::test_crate::use_resolution::a_name_two_globs_bring_in_as_different_items_is_ambiguous_to_a_reader_of_one_glob_in_every_order
    - raf_source::test_crate::use_resolution::a_module_and_a_struct_of_one_name_through_two_globs_are_ambiguous_in_the_type_name_space_in_every_order
    - raf_source::test_crate::use_resolution::the_refusal_of_an_ambiguous_name_in_one_name_space_leaves_the_other_resolved_in_every_order
    - raf_source::test_crate::use_resolution::names_the_private_glob_does_not_make_ambiguous_still_resolve_for_a_reader_outside_in_every_order
    - raf_source::test_crate::use_resolution::a_glob_of_a_module_whose_name_is_ambiguous_carries_the_ambiguity_to_its_readers_in_every_order
    - raf_source::test_crate::use_resolution::an_import_a_reader_cannot_name_still_keeps_its_name_from_the_globs_in_every_order
    - raf_source::test_crate::use_resolution::an_import_of_a_value_a_reader_cannot_name_leaves_the_type_of_its_name_to_the_globs_in_every_order
    - raf_source::test_crate::use_resolution::a_hidden_item_does_not_make_a_name_ambiguous_where_a_glob_cycle_delivers_the_item_held_in_every_order
    - raf_source::test_crate::use_resolution::an_ambiguity_a_glob_cycle_carries_beside_a_glob_of_one_of_its_items_is_one_answer_in_every_order
    - raf_source::test_crate::use_resolution::a_hidden_module_a_glob_cycle_carries_back_beside_the_module_itself_is_no_ambiguity_in_every_order
    - raf_source::test_crate::use_resolution::an_item_a_glob_cycle_carries_back_through_the_module_itself_keeps_its_companion_in_every_order
    - raf_source::test_crate::use_resolution::a_path_through_a_module_two_globs_bring_in_as_different_modules_is_ambiguous_in_every_order
    - raf_source::test_crate::use_resolution::an_item_a_glob_cycle_carries_back_beside_a_glob_that_holds_it_from_elsewhere_drops_its_companion_in_every_order
    - raf_source::test_crate::use_resolution::a_module_a_glob_cycle_carries_back_beside_a_glob_that_holds_it_from_elsewhere_starts_a_path_in_every_order
    - raf_source::test_crate::use_resolution::an_item_a_glob_brings_in_from_its_definition_apart_from_a_cycle_drops_the_companion_the_cycle_carries_in_every_order
    - raf_source::test_crate::use_resolution::an_import_that_reads_its_name_through_the_glob_whose_companion_it_would_drop_keeps_the_ambiguity_in_every_order
    - raf_source::test_crate::exported_items::an_item_reached_only_by_a_pub_use_of_a_private_glob_binding_is_not_exported_in_every_order
    - raf_source::test_crate::exported_items::a_name_two_globs_bring_in_as_different_items_one_through_a_private_glob_is_refused_in_every_order
    - raf_source::test_crate::exported_items::a_surface_refused_for_a_module_name_a_glob_cycle_carries_lists_the_same_candidates_in_every_order
    - raf_source::test_crate::resolve_paths::a_use_path_counts_only_what_the_reading_module_can_access
    - raf_source::test_crate::resolve_paths::a_use_path_is_read_in_one_name_space_or_in_each_in_turn
    - raf_source::test_crate::resolve_paths::a_module_that_is_not_in_the_workspace_and_a_text_that_is_no_path_are_refused
    - raf_source::test_crate::resolve_paths::a_path_that_starts_with_a_name_two_globs_bring_in_as_different_modules_is_ambiguous_in_every_order
    - raf_source::test_crate::resolve_paths::a_path_that_starts_with_a_name_two_globs_bring_in_as_one_module_still_resolves_in_every_order
    - raf_source::test_crate::resolve_paths::an_import_that_starts_with_a_name_two_globs_bring_in_as_different_modules_is_refused_in_every_order
    - raf_source::test_crate::resolve_paths::a_module_of_a_crate_name_two_libraries_share_is_ambiguous_to_a_reader_as_to_a_resolve
    - raf_source::test_crate::resolve_paths::definitions_and_imports_of_one_name_under_cfg_are_alternatives_at_any_segment_of_a_path
    - raf_source::test_crate::resolve_paths::the_cfg_alternatives_of_one_module_a_glob_brings_in_are_one_candidate_in_every_order
    - raf_source::test_crate::resolve_paths::two_different_modules_of_one_name_stay_ambiguous_beside_the_cfg_alternatives_of_one
    - raf_source::test_crate::rustc_oracle::the_index_agrees_with_rustc_on_generated_programs_up_to_the_known_classes
    - raf_source::test_crate::rustc_oracle::every_corpus_program_shows_the_divergence_it_records_and_no_other
    - raf_source::test_crate::rustc_oracle::a_wrong_index_answer_is_reported_beside_the_right_one_which_is_not
    - raf_source::test_crate::rustc_oracle::a_divergence_that_matches_no_class_is_unclassified
    - raf_source::test_crate::rustc_oracle::the_minimiser_drops_what_a_divergence_does_not_need_and_keeps_its_class
    - raf_source::test_crate::rustc_oracle::an_index_answer_that_changes_with_the_order_is_reported_as_index_order
    - raf_source::test_crate::rustc_oracle::a_probe_rustc_answers_by_the_order_is_not_order_when_the_index_does_too
    - raf_source::test_crate::rustc_oracle::the_seed_base_is_read_in_hexadecimal_or_decimal_and_refuses_anything_else
    - raf_source::test_crate::target_resolution::a_helper_resolves_from_an_integration_test_root_and_from_a_module_below_it
    - raf_source::test_crate::target_resolution::a_helper_resolves_from_a_nested_module_of_a_binary
    - raf_source::test_crate::target_resolution::an_example_a_bench_and_the_build_script_are_read_from_as_well
    - raf_source::test_crate::target_resolution::a_module_path_that_no_target_holds_is_unknown_whichever_kind_of_target_it_names
    - raf_source::test_crate::target_resolution::a_support_module_that_two_test_targets_include_resolves_in_each_target_alone
    - raf_source::test_crate::target_resolution::a_module_of_the_library_at_the_crate_path_of_a_test_target_is_ambiguous_with_it
    - raf_source::resolve::tests::a_module_path_of_fifty_thousand_segments_is_read_once_to_find_its_crate
    - raf_source::test_crate::symbol_index::every_impl_block_is_listed_in_file_order_with_its_span_module_self_type_key_and_members
    - raf_source::test_crate::symbol_index::an_impl_block_of_a_file_reached_under_two_module_paths_is_listed_once_for_each
    - raf_source::test_crate::symbol_index::a_negative_impl_keeps_its_bang_in_the_trait_path_beside_a_positive_one
    - raf_source::test_crate::reachability::a_bound_type_covers_its_impl_blocks_and_their_members_and_a_bound_method_nothing_beside_it
    - raf_source::test_crate::reachability::a_bound_module_covers_its_nested_items_and_impl_blocks_and_a_crate_root_nothing
    - raf_source::test_crate::reachability::a_private_helper_called_by_a_bound_item_is_reached_and_an_unreferenced_one_is_not
    - raf_source::test_crate::reachability::a_method_call_a_field_a_lifetime_and_a_local_name_no_item_and_a_path_prefix_names_its_type
    - raf_source::test_crate::reachability::a_path_through_a_type_names_the_type_and_the_member_and_a_leading_colon_names_a_crate
    - raf_source::test_crate::reachability::paths_inside_tests_and_binaries_reach_their_own_crate_and_the_library
    - raf_source::test_crate::reachability::an_item_a_macro_generates_is_not_reached_and_a_macro_named_by_a_reached_item_is
    - raf_source::test_crate::reachability::a_cfg_test_module_is_reached_like_any_module_and_its_tests_reach_their_helpers
    - raf_source::test_crate::reachability::a_module_named_by_a_path_and_then_reached_by_contains_is_opened_then
    - raf_source::test_crate::reachability::the_answer_does_not_depend_on_the_order_of_the_roots_and_is_the_same_each_run
    - raf_source::test_crate::reachability::a_root_past_the_items_or_the_blocks_of_the_workspace_is_refused_beside_one_in_range
    - raf_source::test_crate::reachability::an_impl_block_root_reaches_its_members_and_what_its_header_names
    - raf_source::test_crate::reachability::a_file_with_a_byte_order_mark_and_crlf_lines_reaches_as_a_plain_one_does
    - raf_source::test_crate::reachability::a_member_is_read_from_the_module_of_its_block_and_a_trait_member_from_its_trait
    - raf_source::test_crate::reachability::a_path_the_resolver_refuses_reaches_nothing_beside_one_it_resolves
    - raf_source::test_crate::reachability::test_code_names_the_dev_dependencies_and_other_code_does_not
    - raf_source::test_crate::reachability::gen_is_a_name_before_edition_2024_and_a_keyword_from_it
    - raf_source::test_crate::reachability::a_shared_file_reads_keywords_in_each_reached_packages_edition
    - raf_source::test_crate::reachability::a_pointer_to_a_type_and_a_raw_borrow_of_a_static_reach_them_and_a_constant_its_type
    - raf_source::test_crate::reachability::self_paths_and_methods_called_on_self_reach_the_members_of_that_name
    - raf_source::test_crate::reachability::self_in_a_trait_and_in_its_impl_blocks_reaches_the_members_of_the_trait_and_of_the_block
    - raf_source::test_crate::reachability::a_variant_through_self_names_its_enum_as_a_path_through_the_enum_does
    - raf_source::test_crate::reachability::the_type_a_where_clause_bounds_is_reached_and_a_bound_name_is_not
    - raf_source::test_crate::reachability::a_walk_past_the_item_bytes_budget_is_refused_and_one_within_it_is_given
    - raf_source::test_crate::format_reach::implicit_captures_and_named_counts_are_value_edges_of_the_actual_function
    - raf_source::test_crate::format_reach::named_bindings_suppress_capture_and_left_hand_path_without_hiding_the_expression
    - raf_source::test_crate::format_reach::direct_import_aliases_and_block_imports_are_positive_provenance
    - raf_source::test_crate::format_reach::textual_macro_shadows_begin_after_the_declaration_and_do_not_leak_from_a_block
    - raf_source::test_crate::format_reach::unknown_named_and_glob_imports_never_become_standard_macro_certificates
    - raf_source::test_crate::format_reach::prelude_changes_and_std_head_shadows_refuse_inferred_captures
    - raf_source::test_crate::format_reach::cooked_raw_unicode_keyword_and_count_bindings_use_decoded_names
    - raf_source::test_crate::format_reach::malformed_standard_format_input_is_actionable_while_a_custom_macro_keeps_its_grammar
    - raf_source::test_crate::format_reach::conditional_or_module_prelude_changes_do_not_disable_an_unrelated_sibling
    - raf_source::test_crate::format_reach::nested_standard_expression_macros_are_transparent_but_custom_token_inputs_are_not_calls
    - raf_source::test_crate::format_reach::block_local_std_aliases_are_checked_at_the_imports_own_scope
    - raf_source::test_crate::format_reach::recursive_format_expressions_are_admitted_before_parsing_and_custom_tokens_stay_opaque
    - raf_source::test_crate::format_reach::one_physical_file_is_recognized_per_target_module_edition_and_root_order
    - raf_source::test_crate::format_reach::a_dependency_named_std_and_a_resolved_workspace_macro_are_not_the_sysroot
    - raf_source::test_crate::format_reach::every_admitted_enclosing_standard_macro_accepts_nested_format_expressions
    - raf_source::test_crate::format_reach::a_parent_files_textual_macro_also_shadows_an_imported_standard_alias
    - raf_source::reach::tests::format_candidates_charge_owned_bytes_and_return_worker_work_to_the_caller
    - raf_source::test_crate::load_budgets::an_impl_block_is_charged_its_record_and_what_it_holds
    - raf_source::reach::tests::a_run_is_an_identifier_path_that_no_dot_precedes_with_its_first_byte
    - raf_source::reach::tests::path_keywords_start_a_run_and_other_keywords_are_no_segment
    - raf_source::reach::tests::a_lifetime_a_metavariable_a_declared_name_and_a_bound_name_are_no_run
    - raf_source::reach::tests::a_byte_order_mark_and_a_shebang_are_left_out_and_offsets_are_those_of_the_text
    - raf_source::reach::tests::a_doc_comment_is_no_run_whatever_character_ends_it
    - raf_source::reach::tests::each_run_goes_to_the_innermost_unit_that_holds_its_start
    - raf_source::reach::tests::the_work_of_a_walk_is_counted_exactly
    - raf_source::reach::tests::the_work_grows_with_the_items_and_the_references_and_not_their_product
    - raf_source::reach::tests::a_path_of_fifty_thousand_segments_is_walked_once_and_not_once_for_each_prefix
    - raf_source::reach::tests::the_keywords_are_those_of_the_edition
    - raf_source::reach::tests::const_declares_nothing_so_a_pointer_and_a_raw_borrow_name_their_path
    - raf_source::reach::tests::a_where_clause_keeps_the_type_its_bound_is_put_on
    - raf_source::reach::tests::a_name_called_on_self_is_a_method_run_and_a_field_of_self_is_not
    - raf_source::reach::tests::the_work_over_types_impl_blocks_traits_and_members_grows_with_each_and_not_their_product
    - raf_source::reach::tests::what_the_walk_holds_is_charged_to_the_item_bytes_budget_on_from_the_load
    - raf_source::reach::tests::a_shared_files_edition_caches_are_each_counted_and_charged_once
    - raf_source::budget::tests::the_table_is_indexed_by_each_budgets_position
    - raf_source::cost::tests::a_shared_string_is_charged_its_padded_allocation
    - raf_source::test_crate::registry_scan::tests::child_arguments_carry_the_directory_and_the_limit
    - raf_source::test_crate::registry_scan::tests::version_components_and_prerelease_precedence_keep_their_width_and_order
    - raf_source::test_crate::registry_scan::tests::count_callers_preserve_padding_width_suffixes_and_misplaced_totals
    - raf_source::test_crate::registry_scan::tests::ok_line_prints_work_last_among_the_usage_figures
    - raf_source::test_crate::registry_scan::tests::tally_keeps_the_largest_work_and_counts_budget_refusals
    - raf_source::test_crate::registry_scan::tests::the_child_is_given_the_limit_it_was_asked_for
    - raf_source::test_crate::registry_scan::tests::the_work_limit_reaches_the_load
    - raf_source::test_crate::registry_scan::tests::work_limit_refuses_zero_and_names_the_flag
    - raf_source::facts::tests::a_verbatim_use_group_is_one_use_fact_per_member_in_its_module_with_the_whole_span
    - raf_source::facts::tests::facts_carry_names_lines_documentation_visibility_and_macro_export
    - raf_source::facts::tests::facts_record_struct_shapes_unnamed_consts_leading_colons_and_documentation_forms
    - raf_source::facts::tests::use_trees_and_impl_headers_are_recorded_as_written
    - raf_source::index::tests::a_module_file_visited_before_the_declaration_that_names_it_is_an_internal_error
    - raf_source::index::tests::a_target_whose_root_file_is_missing_is_a_typed_internal_error
    - raf_source::item::tests::shared_text_reads_and_compares_as_a_str
    - raf_source::path::display_tests::control_characters_in_a_refused_path_are_escaped
    - raf_source::test_crate::corpus_replay::a_recorded_list_that_differs_from_the_real_index_fails_the_replay
    - raf_source::test_crate::corpus_replay::child_main
    - raf_source::test_crate::corpus_replay::corpus_files_hold_exactly_the_text_of_their_shapes
    - raf_source::test_crate::corpus_replay::evasion_shapes_are_refused_naming_their_budget_beside_their_at_budget_neighbours
    - raf_source::test_crate::corpus_replay::every_named_shape_has_a_corpus_file_and_no_file_is_unrecorded
    - raf_source::test_crate::corpus_replay::files_at_a_budget_exit_zero_and_files_over_it_are_refused_naming_file_and_budget
    - raf_source::test_crate::corpus_replay::the_shape_table_matches_the_expected_names
    - raf_source::test_crate::corpus_replay::write_corpus
    - raf_source::test_crate::discovery_targets::a_first_party_module_named_like_the_test_namespace_loads_beside_a_test_target
    - raf_source::test_crate::discovery_targets::a_library_and_a_binary_with_one_name_resolve_under_their_own_crate_paths
    - raf_source::test_crate::discovery_targets::a_manifest_that_is_not_toml_or_has_neither_table_is_a_typed_error
    - raf_source::test_crate::discovery_targets::a_manifest_with_a_trailing_comment_and_single_quoted_values_is_honoured
    - raf_source::test_crate::discovery_targets::a_missing_root_is_an_error_beside_an_existing_one
    - raf_source::test_crate::discovery_targets::a_package_nested_below_a_package_without_a_workspace_is_not_discovered
    - raf_source::test_crate::discovery_targets::a_shared_test_helper_directory_is_reached_by_two_targets_and_listed_once
    - raf_source::test_crate::discovery_targets::a_target_path_that_leaves_the_tree_is_refused_and_a_path_that_stays_is_accepted
    - raf_source::test_crate::discovery_targets::a_target_table_without_a_name_is_an_error_naming_the_key
    - raf_source::test_crate::discovery_targets::a_virtual_manifest_contributes_nothing_but_its_children_are_found
    - raf_source::test_crate::discovery_targets::an_explicit_target_file_that_is_missing_is_an_error_naming_the_manifest_key
    - raf_source::test_crate::discovery_targets::autodiscovery_flags_turn_inferred_targets_off_but_explicit_tables_are_still_found
    - raf_source::test_crate::discovery_targets::deeply_nested_inline_tables_are_a_typed_error_on_the_callers_stack_and_a_shallow_one_loads
    - raf_source::test_crate::discovery_targets::each_target_kind_with_explicit_paths_resolves_beside_its_default_neighbour
    - raf_source::test_crate::discovery_targets::only_the_named_roots_are_searched
    - raf_source::test_crate::discovery_targets::required_features_on_the_library_is_a_manifest_error_beside_a_binary_that_names_them
    - raf_source::test_crate::discovery_targets::target_and_hidden_directories_are_skipped_beside_a_discovered_sibling
    - raf_source::test_crate::discovery_targets::targets_are_sorted_and_two_loads_are_equal
    - raf_source::test_crate::discovery_targets::the_root_module_of_every_target_is_recorded_under_its_crate_path
    - raf_source::test_crate::discovery_targets::two_targets_sharing_one_root_are_an_error_naming_the_second
    - raf_source::test_crate::discovery_targets::two_targets_with_one_crate_path_are_an_error_naming_both_beside_distinct_names
    - raf_source::test_crate::disk_tree_containment::a_directory_named_as_a_file_is_refused
    - raf_source::test_crate::disk_tree_containment::a_file_that_is_missing_is_not_found_and_one_that_leaves_the_root_is_outside_root
    - raf_source::test_crate::disk_tree_containment::a_file_that_is_not_utf8_is_refused_beside_one_that_is
    - raf_source::test_crate::disk_tree_containment::a_missing_root_is_an_error_beside_an_existing_one
    - raf_source::test_crate::disk_tree_containment::a_non_utf8_entry_name_fails_the_listing_beside_a_listing_without_one
    - raf_source::test_crate::disk_tree_containment::a_symlink_to_a_symlink_that_leaves_the_root_is_refused_beside_a_chain_that_stays
    - raf_source::test_crate::disk_tree_containment::a_symlinked_root_is_followed_to_its_target_and_containment_is_judged_there
    - raf_source::test_crate::disk_tree_containment::entry_kinds_cover_file_directory_symlink_other_and_absent
    - raf_source::test_crate::disk_tree_containment::listings_are_sorted_by_name
    - raf_source::test_crate::disk_tree_root::a_root_that_is_missing_is_not_found_on_every_platform
    - raf_source::test_crate::disk_tree_root::disk_and_memory_trees_agree_on_regular_files
    - raf_source::test_crate::fs_confinement::only_files_of_the_disk_backend_are_exempt
    - raf_source::test_crate::fs_confinement::the_scanner_finds_every_spelling_of_the_file_system_module
    - raf_source::test_crate::fs_confinement::the_scanner_leaves_neighbouring_paths_alone
    - raf_source::test_crate::load_cap::the_memory_tree_used_here_reads_the_padded_file_at_its_exact_size
    - raf_source::test_crate::memory_tree::a_directory_is_not_readable_as_a_file_and_a_file_is_not_listable
    - raf_source::test_crate::memory_tree::a_file_reads_back_and_a_missing_file_is_not_found
    - raf_source::test_crate::memory_tree::a_key_that_is_both_a_file_and_a_directory_is_refused
    - raf_source::test_crate::memory_tree::a_parent_segment_is_applied_to_the_trees_own_directories
    - raf_source::test_crate::memory_tree::a_tree_built_infallibly_from_a_bad_key_reports_the_error_from_every_call
    - raf_source::test_crate::memory_tree::collecting_pairs_builds_the_same_tree_as_the_array_conversion
    - raf_source::test_crate::memory_tree::directories_are_inferred_and_listings_are_sorted_with_their_kinds
    - raf_source::test_crate::memory_tree::entry_reports_file_dir_and_absence_without_an_error
    - raf_source::test_crate::module_tree::a_cfg_attr_test_path_is_tagged_test_and_nothing_is_dropped
    - raf_source::test_crate::module_tree::a_cfg_test_module_tags_every_item_reached_only_through_it
    - raf_source::test_crate::module_tree::a_file_module_declared_inside_an_inline_module_is_placed_under_the_inline_module
    - raf_source::test_crate::module_tree::a_file_named_by_a_path_attribute_is_a_mod_rs_file_for_its_children
    - raf_source::test_crate::module_tree::a_file_reached_twice_without_a_cycle_is_parsed_once_and_recorded_under_each_path
    - raf_source::test_crate::module_tree::a_flat_file_and_a_directory_module_are_both_followed_beside_each_other
    - raf_source::test_crate::module_tree::a_flat_module_file_and_its_child_below_a_parent_that_sorts_first_keep_their_own_paths
    - raf_source::test_crate::module_tree::a_malformed_cfg_path_or_test_attribute_is_an_error_naming_its_line
    - raf_source::test_crate::module_tree::a_module_with_both_name_rs_and_name_mod_rs_is_an_error_naming_both_files
    - raf_source::test_crate::module_tree::a_nested_cfg_attr_path_is_flattened_with_the_combined_predicate
    - raf_source::test_crate::module_tree::a_non_mod_file_keeps_its_children_in_a_directory_of_its_own_name
    - raf_source::test_crate::module_tree::a_path_attribute_in_a_non_mod_file_is_relative_to_the_files_directory_not_its_module_directory
    - raf_source::test_crate::module_tree::a_path_attribute_inside_an_inline_module_is_relative_to_the_module_directory
    - raf_source::test_crate::module_tree::a_path_attribute_on_an_inline_module_in_a_non_mod_file_replaces_its_directory_component
    - raf_source::test_crate::module_tree::a_path_attribute_on_an_inline_module_replaces_its_directory_component
    - raf_source::test_crate::module_tree::a_raw_identifier_module_reads_the_plain_file_name
    - raf_source::test_crate::module_tree::a_shared_test_module_reached_from_a_target_that_sorts_after_it_keeps_its_own_paths
    - raf_source::test_crate::module_tree::an_inline_module_inside_a_non_mod_file_nests_below_the_files_own_directory
    - raf_source::test_crate::module_tree::an_unparseable_cfg_is_an_error_naming_file_and_line_beside_a_parseable_one
    - raf_source::test_crate::module_tree::an_unresolved_mod_names_the_declaring_file_and_line
    - raf_source::test_crate::module_tree::inline_modules_add_directory_components_for_their_file_children
    - raf_source::test_crate::module_tree::inline_modules_with_path_alternatives_are_bounded_by_the_module_path_budget
    - raf_source::test_crate::module_tree::module_depth_of_exactly_the_budget_loads_and_one_more_is_refused
    - raf_source::test_crate::module_tree::module_nesting_beyond_the_depth_budget_is_refused_and_the_budget_itself_loads
    - raf_source::test_crate::module_tree::one_file_spelled_two_ways_is_one_file_recorded_under_both_module_paths
    - raf_source::test_crate::module_tree::super_super_from_a_child_that_sorts_before_its_parent_climbs_to_the_crate_root
    - raf_source::test_crate::module_tree::the_children_of_a_path_attribute_file_are_placed_under_its_module_whatever_their_names
    - raf_source::test_crate::module_tree::the_default_file_is_also_followed_beside_cfg_attr_alternatives_when_it_exists
    - raf_source::test_crate::module_tree::two_unconditional_path_attributes_are_an_error_beside_one
    - raf_source::test_crate::overflow_harness::a_child_that_exits_three_without_a_result_line_is_a_failure
    - raf_source::test_crate::overflow_harness::a_child_that_panics_is_a_failure_and_not_an_abort
    - raf_source::test_crate::overflow_harness::a_child_with_an_unknown_selector_is_a_failure
    - raf_source::test_crate::overflow_harness::a_panic_in_the_visitor_is_raised_again_with_its_payload
    - raf_source::test_crate::overflow_harness::a_shape_just_inside_its_budgets_exits_zero_with_the_exact_symbols
    - raf_source::test_crate::overflow_harness::child_main
    - raf_source::test_crate::overflow_harness::deep_but_admitted_source_parses_on_the_dedicated_stack_not_the_callers
    - raf_source::test_crate::overflow_harness::the_judge_rejects_a_wrong_index_a_misnamed_refusal_an_abort_and_a_stray_failure
    - raf_source::test_crate::prescan_budgets::a_block_as_the_condition_of_each_else_if_still_counts_the_chain
    - raf_source::test_crate::prescan_budgets::a_comment_or_string_that_looks_like_the_end_of_another_does_not_hide_real_tokens
    - raf_source::test_crate::prescan_budgets::a_lifetime_between_reference_marks_does_not_end_the_run
    - raf_source::test_crate::prescan_budgets::a_lifetime_is_not_the_start_of_a_character_literal
    - raf_source::test_crate::prescan_budgets::closure_heads_with_parameters_chain_like_empty_ones
    - raf_source::test_crate::prescan_budgets::four_thousand_bangs_are_refused_beside_two_hundred_fifty_six
    - raf_source::test_crate::prescan_budgets::neighbouring_valid_shapes_just_below_each_budget_parse
    - raf_source::test_crate::prescan_budgets::separate_if_chains_in_one_block_do_not_add_up_beside_the_one_long_chain
    - raf_source::test_crate::prescan_budgets::ten_thousand_closure_bars_are_refused_beside_one_hundred_twenty_eight
    - raf_source::test_crate::prescan_budgets::the_refusal_names_the_line_where_the_count_went_over
    - raf_source::test_crate::prescan_budgets::tokens_inside_strings_raw_strings_and_comments_count_for_nothing
    - raf_source::test_crate::prescan_budgets::twenty_thousand_reference_marks_are_refused
    - raf_source::test_crate::read_cap::a_zero_cap_refuses_any_content_and_accepts_an_empty_file
    - raf_source::test_crate::read_cap::memory_tree_enforces_the_cap_at_eight_mib
    - raf_source::test_crate::relpath::a_parent_segment_below_the_root_is_kept_for_the_disk_to_resolve
    - raf_source::test_crate::relpath::backslash_nul_and_drive_letter_inputs_are_refused_beside_plain_names
    - raf_source::test_crate::relpath::display_names_the_root_as_a_dot
    - raf_source::test_crate::relpath::dot_and_empty_segments_are_dropped
    - raf_source::test_crate::relpath::join_extends_a_path_and_checks_the_climb_against_the_whole
    - raf_source::test_crate::relpath::parent_and_file_name_are_lexical
    - raf_source::test_crate::rustc_oracle::write_corpus
    - raf_source::test_crate::spawn_limit::child_main
    - raf_source::test_crate::symbol_index::a_file_reached_by_targets_of_two_packages_belongs_to_the_first_target_in_target_order
    - raf_source::test_crate::symbol_index::a_method_in_another_file_than_its_type_is_recorded_under_the_type
    - raf_source::test_crate::symbol_index::a_module_a_function_and_a_macro_of_one_name_coexist_and_each_selects_by_namespace
    - raf_source::test_crate::symbol_index::a_module_in_its_own_file_is_documented_by_the_inner_docs_of_that_file
    - raf_source::test_crate::symbol_index::a_super_path_above_the_root_in_an_unresolved_self_type_is_still_an_error
    - raf_source::test_crate::symbol_index::an_item_the_index_does_not_model_is_a_typed_error_naming_file_and_line
    - raf_source::test_crate::symbol_index::an_unnamed_const_is_not_in_the_index_beside_a_named_one
    - raf_source::test_crate::symbol_index::documentation_counts_text_and_include_str_but_not_hidden_or_blank_comments
    - raf_source::test_crate::symbol_index::files_items_in_and_item_at_answer_by_position
    - raf_source::test_crate::symbol_index::impl_members_are_recorded_under_the_type_or_under_the_trait_by_the_shape_of_the_impl
    - raf_source::test_crate::symbol_index::items_that_share_a_source_item_are_grouped_by_its_identity
    - raf_source::test_crate::symbol_index::crate_roots_have_no_source_group_even_when_the_first_item_starts_at_byte_zero
    - raf_source::test_crate::symbol_index::items_under_matches_whole_path_segments
    - raf_source::test_crate::symbol_index::shipping_is_the_target_and_not_a_test_role
    - raf_source::test_crate::symbol_index::test_roles_follow_the_module_tree_and_the_item
    - raf_source::test_crate::symbol_index::unit_and_tuple_structs_are_also_values_and_named_structs_are_not
    - raf_source::test_crate::target_roots::the_namespace_segments_are_not_items_and_a_first_party_module_of_that_name_collides
    - raf_source::test_crate::target_roots::this_repository_resolves_a_test_crate_root_and_a_library_root
    - raf_source::test_crate::trait_keys::a_dependency_package_that_is_not_a_string_is_an_error_naming_the_key
    - raf_source::test_crate::trait_keys::a_glob_of_something_that_is_no_crate_or_module_is_an_error_beside_a_declared_one
    - raf_source::test_crate::trait_keys::a_head_that_is_not_a_known_crate_is_an_error_and_a_declared_dependency_loads
    - raf_source::test_crate::trait_keys::a_literal_member_without_a_manifest_is_an_error_and_a_glob_skips_such_a_directory
    - raf_source::test_crate::trait_keys::a_malformed_ancestor_manifest_is_an_error_and_a_missing_one_is_not
    - raf_source::test_crate::trait_keys::a_package_that_is_only_a_path_dependency_of_the_root_package_is_a_member
    - raf_source::test_crate::trait_keys::a_path_dependency_inside_the_workspace_with_no_manifest_is_an_error_naming_the_key
    - raf_source::test_crate::trait_keys::a_path_dependency_outside_the_workspace_directory_or_excluded_is_not_a_member
    - raf_source::test_crate::trait_keys::a_path_dependency_with_an_unusable_path_is_an_error_beside_one_that_is_outside_the_tree
    - raf_source::test_crate::trait_keys::a_path_through_a_local_re_export_of_a_dependency_has_one_key_with_the_direct_spelling
    - raf_source::test_crate::trait_keys::a_pattern_outside_the_supported_syntax_is_a_manifest_error_naming_the_list
    - raf_source::test_crate::trait_keys::a_trait_through_one_dependency_glob_is_keyed_under_that_glob
    - raf_source::test_crate::trait_keys::a_workspace_dependency_whose_path_is_not_a_string_is_an_error_naming_the_key
    - raf_source::test_crate::trait_keys::an_ancestor_manifest_over_the_size_cap_is_a_typed_error
    - raf_source::test_crate::trait_keys::an_edition_that_is_not_a_year_is_a_manifest_error_naming_the_key
    - raf_source::test_crate::trait_keys::an_empty_pattern_segment_is_a_pattern_error_beside_a_plain_pattern
    - raf_source::test_crate::trait_keys::an_excluded_literal_member_is_ignored_and_not_required_to_exist
    - raf_source::test_crate::trait_keys::an_inherited_edition_comes_from_the_workspace_and_a_missing_one_is_a_manifest_error
    - raf_source::test_crate::trait_keys::dependency_tables_are_honoured_by_target_kind_and_renames_use_the_name_in_code
    - raf_source::test_crate::trait_keys::local_traits_still_key_under_their_definition_beside_dependency_ones
    - raf_source::test_crate::trait_keys::path_dependencies_of_members_are_members_transitively_in_every_dependency_table
    - raf_source::test_crate::trait_keys::several_dependency_globs_leave_the_trait_unresolved_and_the_members_are_still_found
    - raf_source::test_crate::trait_keys::the_nearest_ancestor_with_a_workspace_table_decides_and_others_are_skipped
    - raf_source::test_crate::trait_keys::the_same_trait_through_a_glob_and_through_a_named_import_can_have_two_keys
    - raf_source::test_crate::trait_keys::the_workspace_manifest_is_found_above_the_roots_and_adds_no_package
    - raf_source::test_crate::trait_keys::the_workspace_root_package_inherits_without_being_listed
    - raf_source::test_crate::trait_keys::without_a_workspace_above_or_an_edition_in_it_the_inheritance_is_a_manifest_error
    - raf_source::test_crate::tree_error::an_unconstructible_path_converts_to_outside_root_naming_the_input
    - raf_source::test_crate::tree_error::only_io_has_a_source_and_only_not_found_is_absence
    - raf_source::test_crate::use_resolution::a_cfg_twin_module_is_one_symbol_and_its_contents_merge
    - raf_source::test_crate::use_resolution::a_glob_resolves_with_via_glob_and_the_direct_path_does_not
    - raf_source::test_crate::use_resolution::a_mutual_super_glob_resolves_a_real_item_in_both_directions
    - raf_source::test_crate::use_resolution::a_name_is_not_read_from_a_glob_while_an_explicit_import_of_it_is_unsettled
    - raf_source::test_crate::use_resolution::a_rename_a_nested_group_and_a_plain_use_each_resolve_to_the_definition
    - raf_source::test_crate::use_resolution::a_route_without_a_glob_decides_via_glob_for_an_item_reached_both_ways
    - raf_source::test_crate::use_resolution::a_trait_path_that_cannot_be_made_absolute_is_a_typed_error
    - raf_source::test_crate::use_resolution::an_invalid_path_is_an_invalid_error_beside_a_valid_one
    - raf_source::test_crate::use_resolution::cfg_attr_alternatives_index_every_file_under_the_one_module_path
    - raf_source::test_crate::use_resolution::crate_self_and_super_paths_resolve_from_the_module_they_are_written_in
    - raf_source::test_crate::use_resolution::definitions_and_explicit_imports_shadow_globs
    - raf_source::test_crate::use_resolution::explicit_imports_shadow_globs_for_the_modules_that_glob_the_importer
    - raf_source::test_crate::use_resolution::extern_crate_renames_and_leading_colon_paths_reach_the_library_from_other_targets
    - raf_source::test_crate::use_resolution::extern_crate_self_names_the_crate_itself_from_every_module
    - raf_source::test_crate::use_resolution::globs_import_only_the_names_the_importer_can_name
    - raf_source::test_crate::use_resolution::raw_identifiers_resolve_under_both_spellings
    - raf_source::test_crate::use_resolution::super_above_the_crate_root_is_an_error_naming_file_and_line
    - raf_source::test_crate::use_resolution::the_same_symbol_reached_through_a_type_alias_chain_and_a_method_path_resolves
    - raf_source::test_crate::use_resolution::trait_impls_on_foreign_types_are_keyed_by_the_canonical_trait_path_in_every_spelling
    - raf_source::test_crate::use_resolution::two_globs_that_bring_in_different_items_are_ambiguous_beside_two_that_bring_in_one
    - raf_source::test_crate::use_resolution::use_with_only_self_is_an_error_and_use_path_self_imports_the_module
    - raf_source::test_crate::workspace_containment::a_directory_symlink_loop_ends_with_an_error_and_a_plain_directory_symlink_is_refused
    - raf_source::test_crate::workspace_containment::a_fifo_named_as_a_module_file_is_refused_without_blocking
    - raf_source::test_crate::workspace_containment::a_link_to_a_symlink_that_points_outside_is_refused_naming_the_module
    - raf_source::test_crate::workspace_containment::a_missing_explicit_target_file_is_a_target_error_on_disk_too
    - raf_source::test_crate::workspace_containment::a_path_attribute_through_an_outside_directory_symlink_with_a_parent_segment_is_refused
    - raf_source::test_crate::workspace_containment::a_symlinked_manifest_to_outside_is_refused_beside_one_to_a_file_inside
    - raf_source::test_crate::workspace_containment::a_target_file_that_is_a_symlink_to_outside_is_refused_naming_the_key
    - raf_source::test_crate::workspace_containment::a_target_root_that_is_a_fifo_is_refused_without_blocking
    - raf_source::test_crate::workspace_containment::disk_and_memory_trees_give_the_same_workspace_for_regular_files
    - raf_source::tree::disk::tests::strategy::every_strategy_ends_a_symlink_loop_with_an_error_and_lists_it_without_entering
    - raf_source::tree::disk::tests::strategy::every_strategy_gives_one_identity_for_a_file_its_links_and_a_path_through_a_directory_link
    - raf_source::tree::disk::tests::strategy::every_strategy_lists_a_directory_and_refuses_a_symlink_a_file_and_an_outside_link
    - raf_source::tree::disk::tests::strategy::every_strategy_refuses_a_fifo_and_a_directory_as_a_file_without_blocking
    - raf_source::tree::disk::tests::strategy::every_strategy_refuses_a_parent_segment_after_an_outside_directory_symlink
    - raf_source::tree::disk::tests::swap::the_same_calls_with_the_hooks_and_no_swap_succeed
    - raf_source::tree::disk::tests::swap::the_same_entry_without_a_swap_is_found
    - raf_source::tree::disk::tests::swap::the_same_listing_without_a_swap_returns_the_entries
    - raf_source::tree::disk::tests::swap::captures_refuse_changed_observations_beside_stable_reads
    - raf_source::tree::disk::tests::the_native_strategy_is_the_platforms_own
    - raf_source::tree::disk::tests::windows::a_disk_tree_refuses_a_reserved_device_name_or_a_trailing_dot_or_space_beside_a_plain_name
    - raf_source::tree::disk::tests::windows::the_final_path_of_a_handle_is_its_canonical_path_and_lies_under_the_root
    - raf_source::verbatim::tests::a_bare_group_in_a_use_group_contributes_its_members
    - raf_source::verbatim::tests::a_use_group_is_not_read_in_an_impl_block
    - raf_source::verbatim::tests::a_use_group_splits_into_one_use_per_member
    - raf_source::verbatim::tests::bodiless_functions_and_types_are_read
    - raf_source::verbatim::tests::other_shapes_are_not_read
    - raf_source::work::tests::a_counter_increments_by_what_is_added
    - raf_source::work::tests::a_limited_kind_panics_at_its_ceiling
    - raf_source::work::tests::a_limited_measure_counts_the_other_kinds_without_a_ceiling
    - raf_source::work::tests::a_measure_that_panicked_leaves_neither_counts_nor_ceiling_behind
    - raf_source::work::tests::every_kind_has_its_own_counter
    - raf_source::work::tests::passing_the_ceiling_panics_at_once
    - raf_source::work::tests::reaching_the_ceiling_is_allowed
    - raf_source::work::tests::the_panic_comes_before_the_work_that_follows_it
    - raf_source::work::tests::the_result_of_the_closure_is_returned
    - raf_source::work::tests::the_tallies_are_reset_between_measures
    - raf_source::workspace::tests::a_test_function_whose_path_leaves_its_crate_path_is_a_typed_error
    - raf_source::test_crate::memory_tree::read_and_unseen_read_apply_the_exact_byte_limit_beside_a_seen_identity
    - raf_source::test_crate::memory_tree::a_workspace_loads_through_an_object_safe_tree_with_caller_local_state
    - raf_source::test_crate::index_disk::a_reload_observes_changed_source_while_the_previous_workspace_keeps_its_snapshot
    - raf_source::test_crate::root::member_and_exclude_lists_accept_4096_entries_and_refuse_one_more
    - raf_source::test_crate::disk_tree_root::root_discovery_refuses_directory_aliases_and_unsafe_parent_reductions_beside_real_starts
    - raf_source::test_crate::disk_tree_root::a_dangling_manifest_is_refused_without_falling_back_to_its_workspace_ancestor
    - raf_source::meter::tests::native_storage_capacity_failure_has_its_original_error_source
    - raf_source::meter::tests::native_storage_preserves_the_item_limit_and_cumulative_charges
    - raf_source::reach::format::identity::tests::native_propagation_spill_refuses_without_refunding_the_cumulative_grant
    - raf_source::reach::format::identity::tests::native_propagation_visits_cycles_and_duplicate_edges_once_per_active_source
    - raf_source::test_crate::format_reach::a_cfg_alternative_sibling_alias_does_not_move_the_declared_modules_prelude_attribute
    - raf_source::test_crate::reachability::an_edition_2015_body_import_is_read_from_its_module_beside_an_explicit_crate_path
    - raf_source::test_crate::reachability::impls_inside_const_and_function_blocks_are_tokens_beside_an_indexed_top_level_impl
    - raf_source::test_crate::reachability::nested_impl_self_is_read_in_the_enclosing_members_context
    - raf_source::test_crate::reachability::qualified_self_suffixes_are_not_member_edges_beside_an_unqualified_self_member
    - raf_source::settle::tests::pending_and_blocker_storage_require_the_original_grants
    - raf_source::settle::tests::repeated_glob_lookups_visit_reads_without_spending_the_settling_work_grant
    - raf_source::test_crate::this_repository::the_reexported_syn_types_are_the_types_received_by_the_parse_visitor
  examples:
    - raf_source::example_crate::code_contexts
    - raf_source::example_crate::registry_scan
    - raf_source::example_crate::corpus_generate
    - raf_source::Workspace::exported_items
    - raf_source::Workspace::impls
    - raf_source::Workspace::reachable
depends: []
---

# Source index

## Purpose

`raf-source` answers whether a Rust path names a source item and where it is
defined. Census loads the source units it compares through this index, and
Workplan resolves code and test bindings through it. Both consumers share the
module and symbol interpretation of a given workspace and target context.

Explicit locked-source loads additionally associate definitions with actual
package identities and selected committed Git bytes. A declared configuration
selects modeled declaration headers while retaining the input context needed to
distinguish that query from a structural inventory of all alternatives.

## Contract

> [!PROPOSED change:source-glob-cost]
> Incremental glob propagation preserves reference answers under every declaration order and meets the counted-work, storage and instruction-growth bounds of the glob-cost change. Native instruction counts compare identical inputs at 100, 200, 400 and 800 elements; the ambiguous hub has the same numerical contract as the other hub.

Named imports publish no namespace peer while their complete import remains
indeterminate. Private forwarding preserves hidden competitors and namespace
companions. Queries share their enclosing work account and propagate native
storage and budget failures without substituting semantic nonresolution.
<!-- raf:tested-by raf_source::test_crate::import_determinacy::a_late_bound_peer_cannot_escape_through_named_and_glob_copies_or_exports,raf_source::test_crate::import_determinacy::a_direct_visible_definition_makes_the_neighboring_import_determinate,raf_source::test_crate::glob_causality::a_value_import_does_not_erase_the_type_ambiguity_waiting_behind_it,raf_source::reach::tests::code_prefix_reads_share_one_query_account_and_refuse_one_less -->

**Checked host observations.** `DiskTree::capture_file` returns a `CapturedFile`
with the requested logical path, verified backend identity, bounded arbitrary bytes
and native permissions from one verified regular-file handle. Type and permissions
are checked again after reading. Unix device/inode identities retain hardlink
relationships. Windows uses verified path identities, so hardlink names can
differ. A copy must acquire its own backend identity. `capture_directory` returns a
`CapturedDirectory` with identity, native permissions and complete sorted entries,
including unsupported special-file entries; repeated observations must agree.
`capture_link` returns a `CapturedLink` describing the lexical link through its
verified containing-directory handle. Unix captures compare link identity, mode
and exact native target spelling before and after reading without following the
destination. Foreign authorities and outside parents refuse. Hosts without a
verified lexical-link strategy return an explicit unsupported-operation error.
<!-- raf:tested-by raf_source::test_crate::disk_tree_containment::a_file_that_is_not_utf8_is_refused_beside_one_that_is,raf_source::test_crate::disk_tree_containment::the_identity_of_a_file_is_the_device_and_inode_of_its_verified_handle,raf_source::test_crate::disk_tree_containment::listings_are_sorted_by_name,raf_source::test_crate::disk_tree_containment::a_symlink_to_a_file_outside_is_refused_beside_an_in_root_symlink_that_is_accepted,raf_source::tree::disk::tests::source_dispatch_preserves_backend_aliases_and_rejects_foreign_direct_paths -->

`DiskTree` provides inherent `read_bytes` and `read_unseen_bytes` operations
alongside the text-only `Tree` interface. Binary reads preserve arbitrary bytes
without UTF-8 validation. A read-unseen result is `OpenedBytes`: it either holds
the actual verified file identity and complete bytes from that same open, or
reports an identity accepted by the caller without reading its payload or
applying the payload ceiling. First-party authority admission, regular-file
checks, containment verification and exact byte limits are unchanged. The `disk`
implementation shares one verified bounded byte primitive with the production
text reads; those reads explicitly convert UTF-8 and still refuse invalid text.
<!-- raf:tested-by raf_source::test_crate::disk_tree_containment::a_file_that_is_not_utf8_is_refused_beside_one_that_is,raf_source::test_crate::disk_tree_containment::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen,raf_source::tree::disk::tests::source_dispatch_preserves_backend_aliases_and_rejects_foreign_direct_paths -->

`MemoryTree` supports checked `create_new` and `replace` publication as well as
reading. Creation infers missing directories, refuses an occupied file or directory
and rejects a file used as an ancestor. Replacement requires an existing regular
file. Both operations retain first-party authority admission, poisoned-constructor
refusals, normalised-key identities and every unrelated file; a refusal changes
nothing. Existing seen-read and exact byte-limit behavior is unchanged.
<!-- raf:tested-by raf_source::test_crate::memory_tree::the_identity_of_a_file_is_its_normalised_key_and_refuses_what_read_refuses -->

### Scoped source and code-query evidence
<!-- raf:tested-by raf_source::test_crate::locked_sources::explicit_first_party_roots_require_every_discovered_manifest_association,raf_source::test_crate::locked_sources::all_snapshots_catalog_and_walker_use_one_original_resource_grant,raf_source::meter::tests::native_recovery_preserves_nested_provenance_and_restores_its_own_refusal -->

`Workspace::load_locked_sources_under` selects first-party discovery roots while
retaining every discovered manifest's exact local association and every supplied
sealed external Git source. An omitted or extra local association refuses; an
explicitly narrower root admits its exact association set. Native storage keeps
one valid allocation owner for each retained buffer, spends the original load
grant and preserves the original typed refusal with its file, line and origin.

### Borrowed external library identities
<!-- raf:tested-by raf_source::test_crate::code_paths::external_library_receipts_keep_actual_names_exports_and_definition_identity,raf_source::test_crate::code_paths::library_receipts_preserve_effective_name_ambiguity_and_deduplicate_one_root,raf_source::test_crate::code_paths::complete_module_paths_return_actual_type_sites_without_becoming_values -->

`CodeResolver::extern_libraries_at` borrows actual indexed library candidates at
an owner and byte offset. `ExternLibrary` retains the actual Rust name, library
target, package and crate-root item. `ExternRouteKind` distinguishes the complete
prelude candidate group from a lexical `extern crate` declaration. Renames and
custom library names preserve actual definitions; selecting one receipt cannot
erase its original group's ambiguity. Same-package injected roots and
`extern crate self` are excluded. `ExternLibrary::resolve` checks the actual
public surface, including hidden public items and reexports, and returns their
defining sites. A private endpoint refuses, and a complete module path is a
type-namespace site rather than a value. A borrowed receipt supplies neither
durable identity nor configuration or implementation policy proof.
Candidate-root scratch is admitted through native storage and metered ordering
under the original query allowance. Its spent usage carries into alias lookup
and public-surface validation rather than receiving a fresh grant.
<!-- raf:tested-by raf_source::test_crate::code_paths::external_library_query_scratch_uses_the_original_native_grants -->

### Bounded associated-member evidence
<!-- raf:tested-by raf_source::test_crate::code_paths::code_associated_calls_prefer_inherent_methods_without_erasing_symbol_inventory,raf_source::test_crate::code_paths::alias_members_preserve_definition_chains_cycles_and_original_query_grants -->

Associated value queries follow zero-parameter plain aliases whose nominal
arguments are literal constants. The returned `Resolution::type_aliases` retains
every alias identity. Cycles, ambiguous definitions, unsupported substitution and
original admission failures refuse. Inherent precedence requires a direct nominal
declaration with matching unconstrained forwarded const arity. Specialized,
bounded, defaulted-type, type-parameter and lifetime applicability is not guessed.
`require_unconditional_inherent_method` checks that supported header subset;
configuration and unexpanded declaration attributes remain caller audits.
Applicability is local to the selected method: a generic sibling does not
disqualify an otherwise unconditional method, while its own generics or bounds
still refuse.
<!-- raf:tested-by raf_source::test_crate::code_paths::inherent_method_applicability_is_local_to_the_selected_signature -->
Canonical symbol-inventory queries preserve their original ambiguity. Alias and
public-surface work compose with the original query allowance.

### Selected target syntax inspection
<!-- raf:tested-by raf_source::test_crate::configured_targets::all_selected_target_kinds_keep_their_native_and_own_library_routes,raf_source::test_crate::configured_targets::missing_duplicate_and_feature_ineligible_targets_refuse_beside_complete_selection,raf_source::test_crate::configured_targets::shared_files_reuse_only_the_same_actual_syntax_selection,raf_source::test_crate::configured_targets::development_edges_require_a_real_development_or_declared_test_context,raf_source::test_crate::configured_targets::conditional_and_optional_edges_keep_each_targets_actual_activation,raf_source::test_crate::configured_targets::retained_target_lists_and_configuration_queries_spend_the_original_grants -->

`with_configured_syntax` inspects an actual target/file through the existing
bounded parser and declared header filter. File selection, reached-target checks,
configuration lookup and filtering spend the original query grant after its
retained reading table. Absent context, a file the target does not reach and
original parser or resource failures refuse. Physical source spans are intact;
borrowed syntax stays inside the callback.
<!-- raf:tested-by raf_source::test_crate::configured_targets::configured_syntax_queries_select_the_actual_target_and_preserve_budget_causes -->

### Source and package identity
<!-- raf:tested-by raf_source::tree::disk::tests::source_dispatch_preserves_backend_aliases_and_rejects_foreign_direct_paths,raf_source::tree::sources::tests::equal_relative_paths_and_backend_keys_remain_separate_sources,raf_source::tree::sources::tests::parent_join_and_real_directory_normalization_keep_the_selected_root,raf_source::tree::sources::tests::scoped_backend_ids_are_refused_before_seen_and_preserve_the_first_cause,raf_source::tree::sources::tests::standalone_memory_tree_refuses_another_sources_tag_before_seen_callback -->

`SourceTrees` assigns load-local `SourceAuthorityId` values to separately rooted
Trees. Equal relative paths and backend keys remain distinct across authorities;
relative normalization and aliases stay with the selected root. Direct backends
refuse another source's tagged path or handle before consulting Seen, retaining
the underlying failure. These tags distinguish source locations; they are not
proof of locked source correspondence.

`PackageId` identifies an actually discovered manifest within one load. Separately
rooted packages may share displayed names while their dependency destinations,
editions and queries stay with the selected package. Inherited workspaces and
renamed path dependencies retain the selected source root. An invalid dependency
identity is a broken invariant, distinct from a valid package without a library
or an unresolved external declaration. Name indexes and discovery scratch retain
native storage ownership and the original item/work grants.
<!-- raf:tested-by raf_source::test_crate::discovery_targets::independently_rooted_same_name_packages_resolve_only_by_their_selected_identity,raf_source::test_crate::discovery_targets::inherited_workspaces_and_renamed_path_dependencies_stay_within_each_source,raf_source::index::tests::an_unknown_dependency_identity_is_not_an_external_or_libraryless_package,raf_source::workspace::tests::native_package_name_index_keeps_identity_and_enforces_its_exact_grant,raf_source::discovery::tests::native_discovery_names_resume_retained_storage_and_refuse_the_original_grant -->

Indexed trait impl members retain a defining target only when every selected
definition is an actual Trait at one path and target. Same-target cfg alternatives
retain that identity; a struct or mixed trait/struct selection retains only its
textual key. Foreign-member fallback distinguishes different defining targets
under equal crate paths while preserving multiple implementations of one trait.
Read-only resolution leaves Workspace equality unchanged; changed definitions
remain unequal even when displayed paths agree.
<!-- raf:tested-by raf_source::test_crate::trait_keys::cfg_alternatives_of_one_defining_trait_keep_one_target_identity,raf_source::test_crate::trait_keys::nontrait_type_hits_supply_no_trait_evidence_beside_a_real_trait_alias,raf_source::test_crate::use_resolution::foreign_impl_members_keep_defining_traits_distinct_under_equal_crate_paths,raf_source::test_crate::symbol_index::read_only_resolution_keeps_workspace_equality_and_distinguishes_changed_definitions -->

### Complete locked catalog
<!-- raf:tested-by raf_source::test_crate::locked_catalog::same_name_version_sources_require_unique_complete_selection_in_either_row_order,raf_source::test_crate::locked_catalog::omitted_git_precision_is_not_a_wildcard_beside_the_full_revision,raf_source::test_crate::locked_catalog::encoded_git_references_keep_native_decoded_identity_and_reference_kind,raf_source::test_crate::locked_catalog::unicode_checksum_refuses_without_panicking_beside_native_ascii_hex,raf_source::test_crate::locked_catalog::malformed_unsupported_and_stale_evidence_has_a_valid_profile_neighbor,raf_source::test_crate::locked_catalog::native_catalog_storage_and_comparison_work_obey_exact_original_grants,raf_source::test_crate::locked_catalog::the_actual_repository_lock_resolves_all_outgoing_edges_and_native_source_precision -->

`LockedCatalog` reads complete package rows and resolves every outgoing reference
to one `LockedPackageId`. `LockedPackage` retains name, version, optional source,
checksum and outgoing rows. `LockedSource` retains source kind, repository and
precise revision or registry identity; `LockedGitReference` preserves reference
kind and native decoded identity. Ambiguous abbreviated references, omitted Git
precision, malformed checksums, unknown fields and stale edges refuse beside
valid profile neighbors. Storage and comparison work use the original supplied
allowances. A selected catalog row is not itself a source Tree admission.

The `cargo_version` adapter requires three unsigned 64-bit components without leading
zeros. Prerelease identifiers reject numeric leading zeros; build identifiers
preserve them. Requirements support exact and ordered comparisons, tilde, caret,
partial versions and wildcards, with at most 32 comma-conjoined comparators.
Only ASCII spaces are ignored around comparators. Matching ignores build metadata
and admits a prerelease only when a comparator names its same numeric triple
with a prerelease. Arbitrarily long numeric prerelease identifiers compare by
numeric value without integer conversion. Maximum components need no increment.
Parsing borrows input without allocation; native byte classes scan Cargo span
boundaries and native inline storage holds comparators. Qualifier comparison
spends the supplied work grant before matching.
<!-- raf:tested-by raf_source::test_crate::dependency_declarations::malformed_declarations_refuse_with_valid_neighbors,raf_source::test_crate::locked_catalog::malformed_unsupported_and_stale_evidence_has_a_valid_profile_neighbor,raf_source::test_crate::locked_selection::rename_version_and_registry_select_only_the_owners_exact_outgoing_row,raf_source::test_crate::locked_selection::selection_honors_work_and_retained_catalog_item_admission -->

### Conservative platform applicability
<!-- raf:tested-by raf_source::platform::tests::parent_platform_guards_do_not_filter_the_source_inventory,raf_source::platform::tests::inner_crate_and_external_module_guards_are_both_observed,raf_source::platform::tests::nonplatform_conditions_and_cfg_attr_do_not_invent_exclusion,raf_source::platform::tests::compiler_platform_refuses_incomplete_or_malformed_output,raf_source::platform::tests::conditional_module_routes_remain_unknown,raf_source::platform::tests::invalid_item_is_an_actionable_error,raf_source::platform::tests::platform_guard_reference_storage_uses_the_remaining_item_grant,raf_source::platform::tests::platform_guard_work_exhaustion_is_typed_beside_a_sufficient_neighbor -->

`Workspace::test_platform_guard` borrows retained item and enclosing-module guards
without removing definitions from the index. `CompilerPlatform::parse` reads the
actual compiler configuration supplied by its caller: OS, families and the bare
`unix` and `windows` atoms remain independent. Malformed configuration, non-string
values, non-atomic entries and missing or repeated OS facts return distinct
`CompilerPlatformError` variants; syntax failures retain their parser source.
A guard returns false only when those platform facts prove exclusion. Feature,
custom and test predicates remain
unknown; ambiguous module routes and unsupported owner ancestry also remain
unknown. Invalid item identifiers, inconsistent source facts and exhausted grants
return typed load errors. This API does not prove that a test is listed or passed.
Root-qualified predicate names refuse before rendering can erase qualification.
An empty applied cfg_attr list is a no-op with the required comma; a missing
predicate or comma is malformed. Boolean literals remain nonplatform evidence,
so an unconditional false predicate cannot manufacture a platform exemption.

### Root discovery
<!-- raf:tested-by raf_source::test_crate::root::member_and_virtual_workspace_directories_resolve_without_sources,raf_source::test_crate::root::standalone_excluded_and_nested_packages_have_their_own_roots,raf_source::test_crate::root::transitive_path_dependencies_are_members,raf_source::test_crate::root::explicit_workspace_requires_a_safe_path_and_agreed_membership,raf_source::test_crate::root::malformed_manifests_and_missing_directories_fail_hard,raf_source::test_crate::root::manifest_size_bound_is_enforced_beside_the_exact_limit,raf_source::test_crate::disk_tree_root::root_discovery_refuses_directory_aliases_and_unsafe_parent_reductions_beside_real_starts -->

`discover_root(tree, start)` discovers the nearest ancestor workspace through
`Tree`, without a process or source parsing. It validates the nearest package
against the workspace member set, including transitive path dependencies, under
the same member-walk bounds as loading. Standalone and excluded packages own
their roots; nested workspaces select the nearest root. Unlisted packages fail
with an actionable finding. `package.workspace` must safely name an existing
workspace inside the tree which includes the package. Malformed manifests,
conflicting declarations and inaccessible paths are hard errors. The starting
path must be an existing directory; discovery stops at the supplied tree boundary.
Directory symlinks are refused because their lexical ancestors can name a
different workspace; a disk caller can canonicalize the starting directory first.

### Tree interface
<!-- raf:tested-by raf_source::test_crate::memory_tree::a_file_reads_back_and_a_missing_file_is_not_found,raf_source::test_crate::memory_tree::a_directory_is_not_readable_as_a_file_and_a_file_is_not_listable,raf_source::test_crate::memory_tree::directories_are_inferred_and_listings_are_sorted_with_their_kinds,raf_source::test_crate::memory_tree::entry_reports_file_dir_and_absence_without_an_error -->

All file access goes through the `Tree` trait, which takes `RelPath` values:
validated root-relative `/`-separated UTF-8 paths.

### Relative path validation
<!-- raf:tested-by raf_source::test_crate::relpath::dot_and_empty_segments_are_dropped,raf_source::test_crate::relpath::a_parent_segment_below_the_root_is_kept_for_the_disk_to_resolve,raf_source::test_crate::relpath::a_parent_segment_above_the_root_is_refused_beside_the_one_that_stays_inside,raf_source::test_crate::relpath::an_absolute_input_is_refused_beside_its_relative_form,raf_source::test_crate::relpath::backslash_nul_and_drive_letter_inputs_are_refused_beside_plain_names,raf_source::test_crate::relpath::join_extends_a_path_and_checks_the_climb_against_the_whole -->

`RelPath::join` drops `.` segments
and refuses an absolute input, a backslash, a NUL, a drive-letter segment and a `..`
that climbs above the root with a typed `PathError`. A `..` below the root is kept,
because on a Unix disk it means the parent of the directory the earlier segments
really reach (Windows joins it by text; see Path containment).

### Tree operations
<!-- raf:tested-by raf_source::test_crate::memory_tree::a_file_reads_back_and_a_missing_file_is_not_found,raf_source::test_crate::memory_tree::a_directory_is_not_readable_as_a_file_and_a_file_is_not_listable,raf_source::test_crate::memory_tree::directories_are_inferred_and_listings_are_sorted_with_their_kinds,raf_source::test_crate::memory_tree::entry_reports_file_dir_and_absence_without_an_error,raf_source::test_crate::memory_tree::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen -->

The trait has four
calls, each returning a `Result`: `read(path, max_bytes)` returns the text of a
file; `read_unseen(path, max_bytes, seen)` opens a file once, takes its `FileId`
from that open and reads its text from the same open unless `seen` accepts the
identity (`identity(path)` is the same call with every identity seen);
`list(path)` returns the entries of a directory sorted by name, each with a kind of
file, directory, symlink or other; and `entry(path)` returns the kind of one path or
`None` when nothing is there.

### File identity and seen reads
<!-- raf:tested-by raf_source::test_crate::memory_tree::the_identity_of_a_file_is_its_normalised_key_and_refuses_what_read_refuses,raf_source::test_crate::memory_tree::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen,raf_source::test_crate::disk_tree_containment::the_identity_of_a_file_is_the_device_and_inode_of_its_verified_handle,raf_source::test_crate::disk_tree_containment::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen,raf_source::test_crate::memory_tree::a_workspace_loads_through_an_object_safe_tree_with_caller_local_state -->

Two paths have equal identities when they name one file:
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

The `tree` implementation stores backend keys as shared opaque bytes.
`FileId::from_key` accepts strings and byte-backed inputs, comparing their bytes
without a text conversion. Memory keys use the UTF-8 bytes of the normalised
path; native handle keys preserve the verified path's encoded bytes. Device
identities, sealed Git identities and source-authority tags remain distinct.
The `shared_text` allocation charge uses the retained key's byte length and its
shared header.
<!-- raf:tested-by raf_source::test_crate::memory_tree::a_workspace_loads_through_an_object_safe_tree_with_caller_local_state,raf_source::tree::sources::tests::equal_relative_paths_and_backend_keys_remain_separate_sources,raf_source::test_crate::load_budgets::index_records_at_the_item_budget_load_and_one_byte_less_is_refused -->

### Tree errors
<!-- raf:tested-by raf_source::test_crate::tree_error::every_variant_names_its_path_in_display_and_in_path,raf_source::test_crate::tree_error::only_io_has_a_source_and_only_not_found_is_absence -->

A failure is a `TreeError` that names the path: `NotFound`, `OutsideRoot`,
`NotRegular`, `SymlinkLoop`, `TooLarge`, `NotUtf8`, `NonUtf8Name`, `DuplicatePath` or
`Io`. Only `NotFound` means that nothing is there; an entry whose name is not UTF-8
fails the listing and is never dropped from it.

### Disk root construction
<!-- raf:tested-by raf_source::test_crate::disk_tree_containment::a_missing_root_is_an_error_beside_an_existing_one,raf_source::test_crate::disk_tree_containment::a_symlinked_root_is_followed_to_its_target_and_containment_is_judged_there -->

`DiskTree::new(root)` returns a
`Result`: it canonicalises the root once, a missing root is `NotFound`, and a
symlinked root is followed to its target.

### Memory map and byte admission
<!-- raf:tested-by raf_source::test_crate::memory_tree::directories_are_inferred_and_listings_are_sorted_with_their_kinds,raf_source::test_crate::memory_tree::collecting_pairs_builds_the_same_tree_as_the_array_conversion,raf_source::test_crate::memory_tree::read_and_unseen_read_apply_the_exact_byte_limit_beside_a_seen_identity -->

`MemoryTree` holds a map of path to text,
infers directories from the keys, and exists so that fixtures build repositories
without touching the disk.

### Invalid and duplicate memory keys
<!-- raf:tested-by raf_source::test_crate::memory_tree::read_and_unseen_read_apply_the_exact_byte_limit_beside_a_seen_identity,raf_source::test_crate::memory_tree::keys_that_leave_the_root_are_refused_with_outside_root_beside_a_valid_key,raf_source::test_crate::memory_tree::duplicate_keys_are_refused_literally_and_after_normalisation_beside_distinct_keys,raf_source::test_crate::memory_tree::a_tree_built_infallibly_from_a_bad_key_reports_the_error_from_every_call -->

It enforces `max_bytes` like the disk backend and refuses a
key such as `../a` or `/a` with `OutsideRoot`, and a key that names a path another key
already holds, whether literally or after normalisation, with `DuplicatePath`; a tree
built by the infallible conversions from such keys returns the same error from every
call.

### Linux descriptor failures
<!-- raf:tested-by raf_source::tree::disk::tests::a_missing_descriptor_link_is_an_io_failure_and_never_absence -->

A missing `/proc` on Linux is an `Io` failure and never `NotFound`.

### Entry lookup boundaries
<!-- raf:tested-by raf_source::tree::disk::tests::strategy::every_strategy_reports_entry_kinds_and_refuses_an_outside_directory -->

`entry`
returns `OutsideRoot` when the directory that holds the path leads outside the root,
judged on a verified handle of that directory before and after the lookup, and
`None` when that directory is missing or is not a directory.

### Disk opening ownership
<!-- raf:tested-by raf_source::test_crate::workspace_containment::a_symlinked_module_file_to_outside_is_refused_naming_the_module_beside_an_in_root_link,raf_source::test_crate::workspace_containment::a_manifest_target_path_that_leaves_the_root_is_refused_naming_the_key,raf_source::test_crate::workspace_containment::a_symlinked_manifest_to_outside_is_refused_beside_one_to_a_file_inside -->

Nothing else in the
crate opens a file.

### Path containment
<!-- raf:tested-by raf_source::test_crate::workspace_containment::a_symlinked_module_file_to_outside_is_refused_naming_the_module_beside_an_in_root_link,raf_source::test_crate::workspace_containment::a_manifest_target_path_that_leaves_the_root_is_refused_naming_the_key -->

Every path the crate opens is checked against the workspace
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
path to it by text, removing each `..` with the segment before it, as Windows does; the
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

### Regular-file boundaries
<!-- raf:tested-by raf_source::tree::disk::tests::strategy::every_strategy_refuses_a_fifo_and_a_directory_as_a_file_without_blocking,raf_source::tree::disk::tests::strategy::every_strategy_lists_a_directory_and_refuses_a_symlink_a_file_and_an_outside_link -->

`DiskTree` refuses any file that is not a regular file, such as a FIFO or a device,
and it does not descend into a directory symlink: a listing reports it with the
symlink kind, listing it is refused, and a symlink loop ends with `SymlinkLoop`. The
file type is checked on the opened handle and not by a stat before the open, and on
Unix no open follows a final symlink or blocks, so naming a FIFO cannot hang.

### Verified-handle operations
<!-- raf:tested-by raf_source::tree::disk::tests::swap::a_directory_swapped_for_an_outside_symlink_after_the_check_is_refused,raf_source::tree::disk::tests::swap::a_directory_swapped_for_an_outside_symlink_before_a_listing_is_refused,raf_source::tree::disk::tests::swap::an_entry_whose_directory_is_swapped_for_an_outside_symlink_is_refused,raf_source::tree::disk::tests::swap::captures_refuse_changed_observations_beside_stable_reads,raf_source::tree::disk::tests::swap::the_same_listing_without_a_swap_returns_the_entries,raf_source::tree::disk::tests::swap::the_same_entry_without_a_swap_is_found -->

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
  path with a segment that Windows would not read as the name it spells: a reserved
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

Standard validation runs the Linux strategy on Linux. Each backend is compiled
only for its own operating system, including in test builds; a Linux test cannot
select an alternate backend. Other platform adapters remain in source without
standard compilation or execution claims.
<!-- raf:tested-by raf_source::tree::disk::tests::the_native_strategy_is_the_platforms_own,raf_source::tree::disk::tests::source_dispatch_preserves_backend_aliases_and_rejects_foreign_direct_paths -->

### File admission and parser refusals
<!-- raf:tested-by raf_source::test_crate::read_cap::memory_tree_enforces_the_cap_at_eight_mib,raf_source::test_crate::read_cap::disk_tree_enforces_the_cap_at_eight_mib,raf_source::test_crate::read_cap::the_default_cap_is_eight_mib,raf_source::test_crate::spawn_limit::a_parse_thread_the_system_refuses_a_stack_is_a_spawn_error_naming_the_file -->

Source is parsed in-process. Each read is capped at
`max_file_bytes`, which defaults to 8 MiB and is set by `LoadOptions::max_file_bytes`
or, for the census, by `max_file_bytes` in `[census]` (see
[census-ledger](./census-ledger.md)). A larger file is an error that names it. `syn`
recurses on more than delimiters, so before any file is parsed the crate runs a token
pre-scan, one lexer pass feeding independent counters.

### Prescan lexical exclusions
<!-- raf:tested-by raf_source::test_crate::prescan_budgets::tokens_inside_strings_raw_strings_and_comments_count_for_nothing,raf_source::test_crate::prescan_budgets::a_comment_or_string_that_looks_like_the_end_of_another_does_not_hide_real_tokens,raf_source::test_crate::prescan_budgets::a_lifetime_is_not_the_start_of_a_character_literal -->

The lexer skips comments
(block comments nest) and every string, raw string (with any number of `#`), byte
string, C string and character literal, and tells a lifetime from a character
literal, so tokens inside them count for nothing.

### Typed prescan and syntax errors
<!-- raf:tested-by raf_source::test_crate::prescan_budgets::the_refusal_names_the_line_where_the_count_went_over,raf_source::test_crate::prescan_budgets::a_syntax_error_reports_its_own_line_and_column_beside_a_parsing_neighbour -->

A file that exceeds a budget is
refused with a `LoadError` of kind `Budget`, which names the file, the budget, its
limit and the line where the count went over. A file that `syn` rejects is a `Syntax`
error with the line and column of the parser's error, read on the parse thread, and a
thread that cannot start is a `Spawn` error.

### Prescan counter definitions

The budgets are one table in the source
(`Budget::limit`), and the counters are defined as follows:

- Delimiters: the number of `(`, `[` and `{` open at once; the limit is 128, so 128
  <!-- raf:tested-by raf_source::test_crate::prescan_budgets::delimiter_nesting_is_refused_at_one_hundred_twenty_nine_beside_one_hundred_twenty_eight -->
  levels parse and 129 are refused.
- Nested `<`: each delimiter level has its own count; `<` adds one unless it is part
  <!-- raf:tested-by raf_source::test_crate::prescan_budgets::five_hundred_nested_generics_are_refused_beside_sixty_four,raf_source::test_crate::prescan_budgets::neighbouring_valid_shapes_just_below_each_budget_parse -->
  of `<=`, and `>` subtracts one down to zero (`->` and `=>` are single tokens and
  subtract nothing). The count of a level returns to zero at `;`, `&&` and `||`, and
  the count of the enclosing level returns to zero when a `{ ... }` block closes, so
  comparisons in separate statements do not add up. The limit is 64.
- Prefix operators: the length of a run of consecutive tokens that are `!`, `-`, `*`,
  <!-- raf:tested-by raf_source::test_crate::prescan_budgets::four_thousand_bangs_are_refused_beside_two_hundred_fifty_six,raf_source::test_crate::prescan_budgets::twenty_thousand_reference_marks_are_refused,raf_source::test_crate::prescan_budgets::a_lifetime_between_reference_marks_does_not_end_the_run -->
  `&` or `&&` (a `&&` is one token), with only whitespace and comments between
  them. A lifetime between two operators neither ends the run nor adds to it. `!=`,
  `-=`, `*=`, `&=` and `->` are not prefix operators. The limit is 256.
- Closure bars: a closure head is `||`, or `|` up to the next `|` at the same
  <!-- raf:tested-by raf_source::test_crate::prescan_budgets::ten_thousand_closure_bars_are_refused_beside_one_hundred_twenty_eight,raf_source::test_crate::prescan_budgets::closure_heads_with_parameters_chain_like_empty_ones -->
  delimiter level. The count is the length of a chain of heads in which each head
  follows the previous one directly, or with `move` between them. The limit is 128.
- `else if`: within one delimiter level, the number of links, where a link is an
  <!-- raf:tested-by raf_source::test_crate::prescan_budgets::twenty_thousand_else_if_links_are_refused_beside_two_hundred_fifty_six,raf_source::test_crate::prescan_budgets::a_block_as_the_condition_of_each_else_if_still_counts_the_chain,raf_source::test_crate::prescan_budgets::separate_if_chains_in_one_block_do_not_add_up_beside_the_one_long_chain -->
  `else` directly after a `}` and followed by `if`. The count returns to zero at `;`,
  at an `else` that is not followed by `if`, and at an identifier other than `else`
  directly after a `}`. A block used as the condition of a link (`else if {x} {}`)
  does not end the chain. The limit is 256.
- Recursion: one additive count of the nesting that the parser builds, and that the
  <!-- raf:tested-by raf_source::test_crate::prescan_budgets::every_operator_counts_one_for_each_of_its_characters_at_the_limit_and_one_over,raf_source::test_crate::prescan_budgets::every_counted_keyword_counts_one_at_the_limit_and_one_over,raf_source::test_crate::prescan_budgets::a_label_a_use_path_separator_and_each_delimiter_count_at_the_limit_and_one_over,raf_source::test_crate::prescan_budgets::a_level_returns_to_zero_where_its_constructs_end_beside_the_same_count_without_the_end,raf_source::test_crate::prescan_budgets::a_construct_that_continues_past_a_comma_or_a_closing_brace_keeps_its_count,raf_source::test_crate::prescan_budgets::a_macro_call_counts_only_its_groups_and_a_match_pattern_only_its_alternatives,raf_source::test_crate::prescan_budgets::chains_that_overflowed_the_parse_stack_are_refused_by_the_recursion_budget,raf_source::test_crate::prescan_budgets::the_recursion_refusal_names_the_line_where_the_count_went_over -->
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
of source) is refused by the item-bytes budget at a peak of 170 MiB resident, measured in a release build. A load that passes the limit is a `Budget`
error that names the file and the line of the declaration.

### Directory and aggregate budget ownership
<!-- raf:tested-by raf_source::test_crate::walk_stack::a_memory_chain_of_twenty_thousand_directories_is_refused_typed_on_a_two_mib_thread,raf_source::test_crate::walk_stack::a_directory_at_the_depth_limit_is_walked_and_one_below_it_is_refused,raf_source::test_crate::walk_stack::a_member_pattern_at_the_depth_limit_expands_and_one_deeper_is_refused_typed,raf_source::test_crate::walk_stack::a_disk_chain_of_directories_is_refused_typed_on_a_two_mib_thread -->

Discovery and the expansion of `workspace.members`
patterns enter at most 128 directories below their root (`DirectoryDepth`), on a stack
of their own; a deeper directory is a `Budget` error that names it, which a chain of 20000 directories in memory meets on a thread of 2 MiB in a debug build and in a release build, and a chain of 2100 on disk meets with 4 MiB resident.

### Configured budget defaults
<!-- raf:tested-by raf_source::test_crate::load_budgets::the_default_budgets_are_the_stated_limits -->

Four byte budgets and a work budget bound one load. Each is a default that `LoadOptions` can
change:

### Byte-budget charge ownership
<!-- raf:tested-by raf_source::test_crate::load_budgets::the_bytes_read_at_the_total_budget_load_and_one_byte_less_is_refused_naming_the_file,raf_source::test_crate::load_budgets::index_records_at_the_item_budget_load_and_one_byte_less_is_refused,raf_source::test_crate::load_budgets::a_file_reached_by_many_module_paths_is_charged_once_per_path_and_refused_over_the_item_budget,raf_source::test_crate::load_budgets::each_large_fact_payload_is_charged_and_refused_at_its_line_once_over_the_fact_budget,raf_source::test_crate::load_budgets::the_fact_budget_counts_over_every_file_of_the_load,raf_source::test_crate::load_budgets::glob_tables_at_the_glob_budget_load_and_one_byte_less_is_refused,raf_source::test_crate::load_budgets::glob_table_work_at_the_work_budget_loads_and_one_step_less_is_refused,raf_source::test_crate::exported_items::a_surface_over_the_item_budget_left_by_the_load_is_refused_beside_one_within_it,raf_source::test_crate::reachability::a_walk_past_the_item_bytes_budget_is_refused_and_one_within_it_is_given -->

- `max_total_bytes` (`TotalBytes`) bounds the bytes the load reads, source files
  and manifests together. Its default is min(1 GiB, max(256 MiB, 4 × the largest
  crate)) over the newest version of each of 1271 crates of a local registry, whose
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
  256 bytes. The same budget, counted on from what the load holds, also bounds
  what a listing of a public surface and a walk of reachability hold (see Exported
  items and Reachability).
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
  Retained import reads, subscriptions, pending queue growth and undecided-import
  graph scratch also use this live storage grant, as does scratch for derived
  glob-fact construction and impl
  self-type and trait-path traversal. Native discovery and item-emission
  stack spills use the item-byte grant instead.
  Immutable cached glob reach and broken-import facts, their native table and
  vector allocations and shared allocation header use the original item-byte
  grant. Index clones share this immutable cache. Construction scratch and
  post-load traversal scratch retain their enclosing live grants until their
  buffers die; releasing construction scratch does not refund the cache.
- `max_work` (`Work`, 2147483648 steps, which is 2^31) bounds import settling and
  admitted path traversals during one load. Queue attempts, removals, subscription
  visits, retained reads, graph and carrier searches, candidate visits, visibility
  checks, sorting units and copied path operations count at their actual callers.
  Derived glob-fact construction, cache searches and native buffer growth also
  consume this work grant. Growth charges cover copied and initialized buffer
  words before allocation; indexed searches charge their bounded node visits.
  Impl self-type and trait-path traversal also charge their retained path segments
  and bytes to this work grant.
  Duplicate queue attempts and repeated visits count even when no new entry remains.
  Glob lookup work counts every read and holder table inspected. Those lookup charges
  accumulate until the next import, graph, settling, impl or final-glob checkpoint;
  the refusal includes the complete charge prefix through the first exceeding
  checkpoint. Other charged operations refuse before execution. All load work is
  cumulative across repeated settles and is never refunded. A refusal names the
  responsible source file and line.
- `max_query_work` (`Work`, 2147483648 steps) bounds each post-load query from zero.
  Nested resolution, alias traversal, public-surface construction and membership
  checks retain the enclosing query's work usage. An over-limit query returns a
  typed budget refusal before any semantic answer. Reachability shares one such
  account across its path and format-provenance reads.

A load over a budget is a `Budget` error that names the budget, the limit that
applied and the file, with the line where there is one. `Workspace::usage` reports
what a load spent against each, for the glob tables the most they held at once, and
for the work the steps it made.
The retained index, source text, facts and glob tables are bounded by their item, total, fact and glob-byte limits: 512, 256, 256 and 512 MiB by default, summing to 1.5 GiB. Parsing one file has the stated bound `max_file_tokens` × `PARSE_BYTES_PER_TOKEN` (1664 bytes), about 3.1 GiB at the default 2000000 tokens. Those components sum to about 4.6 GiB at the defaults; a caller sets each LoadOptions limit to fit its host. Explicit-import bindings are charged to the index; glob-provided bindings are charged to the glob tables. Gathering a module's exports reads each held table once and is charged before allocation. Native queue and graph scratch retain their original import-storage grants until their buffers die.

### Parser stack and admission
<!-- raf:tested-by raf_source::test_crate::overflow_harness::every_committed_overflow_shape_is_refused_by_a_budget_and_none_aborts,raf_source::test_crate::overflow_harness::every_shape_family_loads_at_its_largest_admitted_size_and_one_size_more_is_refused,raf_source::test_crate::overflow_harness::a_shape_just_inside_its_budgets_exits_zero_with_the_exact_symbols,raf_source::test_crate::overflow_harness::a_deliberate_overflow_aborts_and_the_runtime_names_the_parse_thread,raf_source::test_crate::overflow_harness::the_judge_rejects_a_wrong_index_a_misnamed_refusal_an_abort_and_a_stray_failure,raf_source::test_crate::overflow_harness::the_parse_thread_is_named_after_the_file_and_has_the_stated_stack,raf_source::test_crate::overflow_harness::a_panic_in_the_visitor_is_raised_again_with_its_payload,raf_source::test_crate::overflow_harness::deep_but_admitted_source_parses_on_the_dedicated_stack_not_the_callers,raf_source::test_crate::prescan_budgets::every_operator_counts_one_for_each_of_its_characters_at_the_limit_and_one_over,raf_source::test_crate::prescan_budgets::every_counted_keyword_counts_one_at_the_limit_and_one_over,raf_source::test_crate::prescan_budgets::a_level_returns_to_zero_where_its_constructs_end_beside_the_same_count_without_the_end,raf_source::test_crate::prescan_budgets::a_construct_that_continues_past_a_comma_or_a_closing_brace_keeps_its_count,raf_source::test_crate::prescan_budgets::chains_that_overflowed_the_parse_stack_are_refused_by_the_recursion_budget -->

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
size cap. The native TOML arena admits at most 128 semantic levels and 1,048,576
nodes, with a 256 MiB model allowance; parsing and destruction are iterative.
Every refusal the pre-scan makes is a typed error and not a panic.
The crate sets no time bound.

### Explicit package roots
<!-- raf:tested-by raf_source::test_crate::discovery_targets::an_empty_root_list_is_a_no_roots_error_beside_dot_which_loads,raf_source::test_crate::discovery_targets::only_the_named_roots_are_searched,raf_source::test_crate::discovery_targets::a_missing_root_is_an_error_beside_an_existing_one -->

`Workspace::load(tree, roots)` takes explicit roots; the
crate has no default roots, each caller names its own (`"."` is the tree root), and an
empty list is a `NoRoots` error.

### Load failure ordering
<!-- raf:tested-by raf_source::test_crate::discovery_targets::discovery_target_and_module_failures_precede_later_load_phases -->

`Workspace::load` returns the first error, in the
order discovery, targets, module walk, index; it does not collect errors.

### Package descent
<!-- raf:tested-by raf_source::test_crate::discovery_targets::a_package_nested_below_a_package_without_a_workspace_is_not_discovered,raf_source::test_crate::discovery_targets::a_virtual_manifest_contributes_nothing_but_its_children_are_found,raf_source::test_crate::discovery_targets::target_and_hidden_directories_are_skipped_beside_a_discovered_sibling,raf_source::test_crate::walk_stack::a_directory_at_the_depth_limit_is_walked_and_one_below_it_is_refused,raf_source::test_crate::walk_stack::a_memory_chain_of_twenty_thousand_directories_is_refused_typed_on_a_two_mib_thread -->

Discovery reads each root as a directory. A directory with a `Cargo.toml` that has a
`[package]` table is a package. The walk descends, on a stack of its own and at most
128 directories below the root, through every directory that is not
a package, and through a package that also has a `[workspace]` table, so a root `.`
package with a `[workspace]` table and `crates/a` below it are both found. It does not
descend below a `[package]` manifest that lacks `[workspace]`, so a fixture package
under a tests directory is not found. It skips directories named `target` and
dot-directories. A manifest that is a virtual `[workspace]` contributes nothing but
its children.

### Discovery symlinks
<!-- raf:tested-by raf_source::test_crate::workspace_containment::discovery_classifies_a_symlink_by_what_it_resolves_to,raf_source::test_crate::workspace_containment::a_directory_symlink_loop_ends_with_an_error_and_a_plain_directory_symlink_is_refused -->

A symlink is classified by what it resolves to: a regular file inside the root
is left alone, a directory is a `DirSymlink` error, a special file such as a FIFO is
`NotRegular`, a target outside the root is `OutsideRoot`, a dangling link is
`NotFound` and a loop is `SymlinkLoop`. None of these reads more than one byte.

### Package identity and ordering
<!-- raf:tested-by raf_source::test_crate::discovery_targets::two_packages_with_one_name_are_an_error_naming_both_manifests,raf_source::test_crate::discovery_targets::only_the_named_roots_are_searched,raf_source::test_crate::discovery_targets::targets_are_sorted_and_two_loads_are_equal -->

Two packages with one name
are an error that names both manifests, and one manifest reached from two roots is one
package. Packages are sorted by manifest path.

### Manifest syntax and admission
<!-- raf:tested-by raf_source::test_crate::discovery_targets::a_manifest_with_a_trailing_comment_and_single_quoted_values_is_honoured,raf_source::test_crate::discovery_targets::a_manifest_that_is_not_toml_or_has_neither_table_is_a_typed_error,raf_source::test_crate::discovery_targets::deeply_nested_inline_tables_are_a_typed_error_on_the_callers_stack_and_a_shallow_one_loads,raf_source::test_crate::discovery_targets::a_manifest_over_the_cap_is_refused_naming_it_and_one_of_exactly_the_cap_loads -->

Manifests are read through `Tree` under `max_file_bytes` and parsed with the
native `purrdf-toml` grammar, so comments, single-quoted strings and inline tables are honoured; a
manifest that is not TOML, is nested deeper than the parser allows, or has a value of
the wrong type is a `Manifest` error that names the file and, when one is at fault, the
key.

Manifest and lock readers consume the supplied flat TOML 1.1 arena directly,
without a generic compatibility value tree or serde dependency. The arena
retains decoded key identity, child order, source spans and typed scalar values.
Every parsed numeric value, including unused nested
metadata, retains the signed 64-bit integer and representable float domain.
Only the authored tokens inf, +inf and -inf authorize infinity; signed NaN and
signed zero remain distinct native values. Exponent overflow refuses. Lock versions
retain integer-value interpretation independently of radix or digit separators.
Manifest and Locked failures retain the original native `TomlError` as their
standard error source for grammar and parser-resource refusals; application
schema failures carry their own key or field message without a parser cause.
<!-- raf:tested-by raf_source::test_crate::discovery_targets::a_manifest_that_is_not_toml_or_has_neither_table_is_a_typed_error,raf_source::test_crate::locked_catalog::malformed_unsupported_and_stale_evidence_has_a_valid_profile_neighbor -->

### Target kinds and test settings
<!-- raf:tested-by raf_source::test_crate::discovery_targets::each_target_kind_with_default_paths_has_its_name_kind_crate_path_and_shipping_flag,raf_source::test_crate::discovery_targets::harness_required_features_test_and_doctest_are_read_from_each_target_table,raf_source::test_crate::discovery_targets::a_test_setting_of_the_wrong_type_is_a_manifest_error_naming_its_key,raf_source::test_crate::discovery_targets::required_features_on_the_library_is_a_manifest_error_beside_a_binary_that_names_them -->

For each package the crate derives the targets, in this order: the library, the
binaries, the tests, the benches, the examples and the build script. A target has the
package name, its Cargo name, its kind, its root file, a shipping flag, a crate path
and the test settings of its table: `harness`, `required-features`, `test` and
`doctest`. A setting the table leaves out, or a target without a table, takes cargo's
default: `harness` true, no required features, `test` true for the library, the
binaries and the tests and false for benches and examples, and `doctest` true for the
library only. The build script has no table and is never tested, so its `harness`,
`test` and `doctest` are false. A setting of the wrong type is a `Manifest` error that
names its key, such as `bin.harness`.

### Target paths and automatic discovery
<!-- raf:tested-by raf_source::test_crate::discovery_targets::each_target_kind_with_default_paths_has_its_name_kind_crate_path_and_shipping_flag,raf_source::test_crate::discovery_targets::each_target_kind_with_explicit_paths_resolves_beside_its_default_neighbour,raf_source::test_crate::discovery_targets::autodiscovery_flags_turn_inferred_targets_off_but_explicit_tables_are_still_found -->

The library is `[lib]` or `src/lib.rs`, named by `[lib] name` or the package
name. Binaries are the `[[bin]]` tables, `src/main.rs`, `src/bin/*.rs` and
`src/bin/*/main.rs`; tests, benches and examples are their tables plus `tests/*.rs`,
`tests/*/main.rs` and the same under `benches/` and `examples/`. `autobins`,
`autotests`, `autobenches` and `autoexamples` set to false turn the inferred ones off,
and an explicit table is found either way. An explicit table without `path` takes the
default location for its name.

### Target crate paths and shipping roles
<!-- raf:tested-by raf_source::test_crate::discovery_targets::each_target_kind_with_default_paths_has_its_name_kind_crate_path_and_shipping_flag,raf_source::test_crate::discovery_targets::each_target_kind_with_explicit_paths_resolves_beside_its_default_neighbour,raf_source::test_crate::target_roots::every_target_kind_resolves_at_its_crate_path_as_a_module_item_of_its_root_file -->

The crate path is the crate name for the library
(`[lib] name`, or the package name, with `-` as `_`), `<pkg>::bin_crate::<bin>` for a
binary, `<pkg>::test_crate::<target>`, `<pkg>::bench_crate::<target>` and
`<pkg>::example_crate::<target>` for the others, and `<pkg>::build_script` for the
build script. Each crate path resolves to the target's root as a module item. Library,
binary and build-script targets are shipping, so the census sees
`build.rs`; tests, benches and examples are not.

### Invalid target declarations
<!-- raf:tested-by raf_source::test_crate::discovery_targets::an_explicit_target_file_that_is_missing_is_an_error_naming_the_manifest_key,raf_source::test_crate::discovery_targets::a_target_table_without_a_name_is_an_error_naming_the_key,raf_source::test_crate::discovery_targets::two_targets_sharing_one_root_are_an_error_naming_the_second,raf_source::test_crate::discovery_targets::two_targets_with_one_crate_path_are_an_error_naming_both_beside_distinct_names,raf_source::test_crate::discovery_targets::a_target_path_that_leaves_the_tree_is_refused_and_a_path_that_stays_is_accepted -->

These are typed errors that name the manifest key: an explicit
target file that does not exist (checked with `entry`), a table without a `name`, two
targets with one root file, two targets of one package with one crate path,
which would merge their items, and a `path` that leaves the tree.

### Cross-package target identity
<!-- raf:tested-by raf_source::test_crate::discovery_targets::two_packages_whose_targets_share_crate_paths_load_and_are_told_apart_by_package -->

Targets of different
packages may share a crate path, as two packages with one `[lib] name` (or names that
differ only in `-` and `_`) do: the index keys every crate by its package, and such a
path is told apart with its package (see `resolve_in_package`).

### Target read origins
<!-- raf:tested-by raf_source::test_crate::target_roots::an_over_cap_inferred_lib_names_discovery_not_lib_path,raf_source::test_crate::target_roots::unreadable_target_roots_name_explicit_paths_or_table_names_and_build_discovery -->

A root file that
cannot be read names what led to it: the `path` key of its table when the manifest
names the file, the `[lib]` table (`lib`) when it declares the library without a path,
the `name` key of a `[[bin]]`, `[[test]]`, `[[bench]]` or `[[example]]` table without a
path, `package.build` for a named build script, and package discovery for a target
found at its default location without a table.

### Module directories and path attributes
<!-- raf:tested-by raf_source::test_crate::module_tree::a_flat_file_and_a_directory_module_are_both_followed_beside_each_other,raf_source::test_crate::module_tree::a_non_mod_file_keeps_its_children_in_a_directory_of_its_own_name,raf_source::test_crate::module_tree::a_module_with_both_name_rs_and_name_mod_rs_is_an_error_naming_both_files,raf_source::test_crate::module_tree::inline_modules_add_directory_components_for_their_file_children,raf_source::test_crate::module_tree::a_path_attribute_in_a_mod_rs_file_is_relative_to_that_files_directory,raf_source::test_crate::module_tree::a_path_attribute_in_a_non_mod_file_is_relative_to_the_files_directory_not_its_module_directory,raf_source::test_crate::module_tree::a_path_attribute_inside_an_inline_module_is_relative_to_the_module_directory,raf_source::test_crate::module_tree::a_path_attribute_on_an_inline_module_replaces_its_directory_component,raf_source::test_crate::module_tree::a_file_named_by_a_path_attribute_is_a_mod_rs_file_for_its_children,raf_source::test_crate::module_tree::two_unconditional_path_attributes_are_an_error_beside_one -->

From each target root the crate follows module declarations
and parses each file with `syn`, as rustc resolves them. Without a `path` attribute,
`mod name;` is `name.rs` or `name/mod.rs` in the module directory; both existing is an
error that names both files. The module directory of a root file or a `mod.rs` file is
its own directory, and that of any other file `a.rs` is `a/` beside it. An inline
module `mod m { .. }` adds `m` to the module directory. `#[path = "p"]` on `mod name;`
names a file relative to the directory of the file that holds the declaration, or,
inside an inline module, relative to the module directory including the inline
components; on an inline module it replaces that module's directory component. A file
named by a `path` attribute is treated like a `mod.rs` for its own children. Two
`path` attributes that apply unconditionally are an error.

### Conditional module paths
<!-- raf:tested-by raf_source::test_crate::module_tree::every_cfg_attr_path_alternative_is_followed_carrying_its_predicate,raf_source::test_crate::module_tree::the_default_file_is_also_followed_beside_cfg_attr_alternatives_when_it_exists,raf_source::test_crate::module_tree::a_cfg_attr_test_path_is_tagged_test_and_nothing_is_dropped -->

A path in
`#[cfg_attr(predicate, path = "p")]` is another site that is followed, carrying its
predicate, and a nested `cfg_attr` is flattened with the combined predicate
(`all(outer, inner)`); the default file is followed too when it exists, and may be missing when
every alternative names a file. In structural loads no `cfg` is evaluated and no
item is dropped; explicitly configured loads select the active headers and paths
as stated below. A raw
identifier module `r#match` reads `match.rs`. A path that leaves the workspace root,
whether by `..` segments, an absolute path or a symlink, is an error that names the
module and the attribute and does not read the file. An unresolved `mod` is an error
that names the declaring file and its line.

### Module identity and cycles
<!-- raf:tested-by raf_source::test_crate::module_tree::a_module_cycle_is_an_error_naming_both_files,raf_source::test_crate::module_tree::a_file_reached_twice_without_a_cycle_is_parsed_once_and_recorded_under_each_path -->

Files are told apart by their identity
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
source file once with its text and the module paths it was reached by.

### Owned file facts
<!-- raf:tested-by raf_source::test_crate::symbol_index::every_kind_is_indexed_with_its_kind_line_file_and_visibility,raf_source::test_crate::symbol_index::documentation_counts_text_and_include_str_but_not_hidden_or_blank_comments,raf_source::test_crate::symbol_index::parameter_bindings_are_every_bound_name_in_signature_order_beside_a_function_with_none,raf_source::test_crate::symbol_index::an_initializer_is_the_text_between_the_equals_and_the_semicolon_beside_a_required_constant,raf_source::test_crate::symbol_index::bindings_and_an_initializer_are_shared_by_every_path_that_reaches_the_file,raf_source::test_crate::module_tree::every_cfg_attr_path_alternative_is_followed_carrying_its_predicate -->

The facts of a
file stay inside the crate, and a caller reads them through the items of the index
(`Workspace::items_in`, `Workspace::item_at` and the other accessors). The
facts of a file are read on the parse thread and owned: items with their name line,
span, body span, attributes, visibility, documentation, test role, for a function the
names its parameters bind, for a constant or static the range of its value, and, for a
struct, its field shape; `mod` declarations with their raw `path` and `cfg_attr` attributes; `use`
trees; impl headers with the written self type and its leading `::`; `macro_export`;
an unnamed `const _` as an explicit kind; and the file's own inner documentation.
Documentation counts a non-blank `#[doc]` text or `#[doc = include_str!(..)]`, and
`#[doc(hidden)]` alone does not count.

### Test-role matrix
<!-- raf:tested-by raf_source::test_crate::module_tree::the_cfg_test_matrix_tags_items_and_leaves_their_neighbours_alone,raf_source::test_crate::module_tree::a_cfg_test_module_tags_every_item_reached_only_through_it -->

An item has the test role `None`, `CfgTest` (guarded by a
`cfg` that implies `test`, directly or through an enclosing module) or `TestFn` (marked
`#[test]`). A predicate implies `test` by this matrix: `test` does; `all(..)` does when
any operand does; `any(..)` does when every operand does; `not(..)` and every other
predicate do not.

### Malformed source attributes
<!-- raf:tested-by raf_source::test_crate::module_tree::a_malformed_cfg_path_or_test_attribute_is_an_error_naming_its_line,raf_source::test_crate::module_tree::an_unparseable_cfg_is_an_error_naming_file_and_line_beside_a_parseable_one,raf_source::test_crate::module_tree::a_non_literal_path_attribute_is_reported_as_such_from_the_attribute -->

A `cfg`, `cfg_attr`, `path` or `test` attribute of the wrong shape, a predicate
that cannot be read, and a `path` whose value is not a string literal (such as
`concat!(..)`) are a `CfgParse` error that names the file and the attribute's
line. An error in a `path` attribute of a module is reached by that attribute: its
origin names the module and the `cfg_attr` predicate that applies it, if any, and not
what reached the file. A module reached through `#[cfg(test)] mod t;`, or
through a `path` that a `test` predicate sets, is recorded with the test flag on that
way of reaching it, so every item in the file is test code when all its ways in are
tests. Test items stay in the index with that tag so that a caller decides whether they
count.

### Indexed item kinds
<!-- raf:tested-by raf_source::test_crate::symbol_index::every_kind_is_indexed_with_its_kind_line_file_and_visibility,raf_source::test_crate::symbol_index::impl_members_are_recorded_under_the_type_or_under_the_trait_by_the_shape_of_the_impl,raf_source::test_crate::symbol_index::test_roles_follow_the_module_tree_and_the_item -->

Every named item of every module tree is an `Item`: a function,
struct, enum, union, trait, type alias, constant, static, module, `macro_rules!`
macro, `extern crate` item, item of an `extern` block, method of an inherent or trait
`impl` block, default and required method of a trait, and associated constant or type.
An item has an `id`, its full `path`, its `kind`, its `namespace`, its `file`, the line
of its name, its `span` and `body` (a `SpanFact`) in lines and bytes, its `visibility`, `documented`,
`doc_hidden`, `test` role, `target` and, for an impl member, the written `self_type`
and `trait_path` and the `self_key` that says how its self type placed it (`Local`,
`Trait` or `Unresolved`).

### Byte and line spans
<!-- raf:tested-by raf_source::test_crate::symbol_index::items_on_one_long_line_of_wide_characters_get_exact_byte_offsets,raf_source::test_crate::source_text::a_one_line_item_has_the_half_open_lines_n_to_n_plus_one -->

Line ranges are 1-based and half-open: the `lines` of a
`SpanFact` run from its first line to the line after its last, as in `raf_okf::Span`,
so an item on line `n` alone has the lines `n..n+1`, and its bytes run from
`start_byte` to `end_byte`, the offset just past its end.

### Parameter bindings
<!-- raf:tested-by raf_source::test_crate::symbol_index::parameter_bindings_are_every_bound_name_in_signature_order_beside_a_function_with_none -->

`Item::parameter_bindings` gives the names the
parameters of a function or method bind, in signature order, as the source writes them:
`mut x`, `ref x`, the names inside tuple, struct and slice patterns, a raw identifier
with its `r#`, and `self` first for a receiver; a name that only occurs in a type is not
a binding, and the list is empty for any item that is not a function and for a function
with none.

### Initializer ranges
<!-- raf:tested-by raf_source::test_crate::symbol_index::an_initializer_is_the_text_between_the_equals_and_the_semicolon_beside_a_required_constant,raf_source::test_crate::symbol_index::bindings_and_an_initializer_are_shared_by_every_path_that_reaches_the_file -->

`Item::initializer` gives the byte range of the value of a constant,
associated constant or static, after the `=` and before the `;`, and `None` for a
required associated constant and for every other item. Both are read during the
budgeted parse, on its thread, so a caller that needs them never re-parses a signature
or an item on its own stack, and every record of a source item that several paths reach
shares them.

### Namespace and documentation metadata
<!-- raf:tested-by raf_source::test_crate::symbol_index::unit_and_tuple_structs_are_also_values_and_named_structs_are_not,raf_source::test_crate::symbol_index::a_module_a_function_and_a_macro_of_one_name_coexist_and_each_selects_by_namespace,raf_source::test_crate::symbol_index::documentation_counts_text_and_include_str_but_not_hidden_or_blank_comments,raf_source::test_crate::symbol_index::a_module_in_its_own_file_is_documented_by_the_inner_docs_of_that_file,raf_source::test_crate::symbol_index::an_unnamed_const_is_not_in_the_index_beside_a_named_one -->

The module path in an item's `path` is the chain of `mod`
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

### Written bodiless declarations
<!-- raf:tested-by raf_source::test_crate::symbol_index::bodiless_functions_and_types_of_attribute_macros_are_indexed_where_they_are_written -->

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
of `items_by_source`, `same_source(root)` returns an empty slice, and `items()` and
`items_in(file)` include them. If a crate path
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
ranges and a leading `!` or `^`) in each path segment, and a segment is matched in at most (pieces + 1) × (name length + 2) steps, where the pieces are its stars and its single characters and classes; `**`, a backslash, an unclosed bracket and an
empty segment (`a//b`, `a//`) are a typed manifest error, never ignored; one trailing `/` is
allowed (`crates/*/` is `crates/*`). `workspace.members` and `workspace.exclude` each
hold at most 4096 entries, a manifest error naming the key beyond that. Each distinct
pattern is compiled once: a duplicate is dropped, an exclude without a wildcard is a set
entry looked up once for each directory above a path, and only the excludes with a
wildcard are matched one by one, so a manifest of 20000 members and 200000 excludes
(880 KB) is refused at once. The cost that remains is the directories below a root times the wildcard excludes, counted and not timed: each member directory a pattern finds is tested once against the wildcard excludes, an exclude with more segments than the path is skipped without a step, and 4096 duplicate excludes are one wildcard tested twice. An entry of `workspace.members` that has no wildcard names one directory, which must exist and have
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

### Source identity and groups
<!-- raf:tested-by raf_source::test_crate::symbol_index::items_that_share_a_source_item_are_grouped_by_its_identity,raf_source::test_crate::symbol_index::crate_roots_have_no_source_group_even_when_the_first_item_starts_at_byte_zero -->

The identity of a written source item is its file and the byte offset where its span starts. A file
that several module paths reach is indexed once for each path, so the same source item
can appear as several `Item` values that share a file and offset and differ in `path`
and `target`.

### Stable item order and prefixes
<!-- raf:tested-by raf_source::test_crate::symbol_index::two_loads_are_equal_and_items_and_targets_come_in_strictly_increasing_order,raf_source::test_crate::symbol_index::items_under_matches_whole_path_segments -->

`items` yields every item sorted by path, then name space, file and
position, strictly increasing. `items_under(prefix)` yields the item at `prefix` and the
items below it, matching whole path segments: `demo::a` selects `demo::a` and
`demo::a::b` but not `demo::ab`, and an empty prefix selects every item.

### Source position lookups
<!-- raf:tested-by raf_source::test_crate::symbol_index::items_that_share_a_source_item_are_grouped_by_its_identity,raf_source::test_crate::symbol_index::crate_roots_have_no_source_group_even_when_the_first_item_starts_at_byte_zero,raf_source::test_crate::symbol_index::files_items_in_and_item_at_answer_by_position -->

`items_by_source()` and `same_source(&item)` group the items that share
one identity, excluding whole-file crate roots, so a census can count a source item
once however many paths reach it; `same_source(root)` is empty even when the first
written item starts at byte 0.
`item(id)` is a lookup; `items_in(file)` yields a file's items in order
of position; `item_count_in(file)` reads the native count without scanning spans,
including roots and distinct reaches, and returns zero for an unindexed file.
`item_at(file, offset)` yields the innermost items that hold the offset.

### Source-owned code reading contexts
<!-- raf:tested-by raf_source::test_crate::code_paths::code_heads_read_the_edition_2015_module_where_use_heads_read_the_crate_root,raf_source::test_crate::code_paths::method_and_trait_members_read_their_defining_module_instead_of_their_symbol_parent,raf_source::test_crate::code_paths::a_module_body_reads_itself_while_its_declaration_reads_its_parent,raf_source::test_crate::code_paths::code_context_tables_spend_only_the_original_load_allowance_and_preserve_usage,raf_source::test_crate::code_paths::exported_macros_keep_their_lexical_module_beside_their_crate_root_export,raf_source::test_crate::code_paths::pattern_binding_observations_exclude_names_introduced_by_guard_expressions -->

`Workspace::code_resolver()` prepares one reading table under the load's original
item allowance. Its usage retains the original load counters and adds the admitted
table storage. Identities remain local to that Workspace. An absent owner, a byte
outside its source span, a non-module scope and missing context are typed errors.

`CodeResolver::reading_module` uses actual source ownership rather than truncating
the displayed symbol path. Method and trait members retain their defining module;
an inline module's body reads itself while its declaration reads the parent.
Exported macros keep their lexical module beside their crate-root export name.
`is_defined_in` answers membership in an actual module or descendant, with distinct
targets kept separate. This supports module sanctions for relocated exports and
implementations written outside the module of their displayed type path.

`resolve_from_item_at` uses code-head lookup from that exact reading module;
`resolve_import_from_item_at` uses import-head lookup, including the crate-root
rule of edition 2015. Both reuse the shared resolver and preserve its errors.
Lexical parameters, locals, generics and block imports must be classified by the
caller; module lookup alone does not prove an occurrence's definition or provider
delegation. Macro-definition bodies require invocation context and refuse code
lookup; header observations retain their lexical reading module. This does not
change reachability's documented export-placement approximation.

`visit_pattern_bindings` visits syntactic binding identifiers in source order,
excluding type and expression subtrees, so match guards cannot add their own
expression-local names to the pattern's bindings. Callers resolve ambiguous
constant identifiers separately. The parameter observation used by the existing
normal form retains its original traversal policy.

### File attribution and shipping
<!-- raf:tested-by raf_source::test_crate::symbol_index::shipping_is_the_target_and_not_a_test_role,raf_source::test_crate::symbol_index::a_file_reached_by_targets_of_two_packages_belongs_to_the_first_target_in_target_order,raf_source::test_crate::symbol_index::files_items_in_and_item_at_answer_by_position -->

`is_shipping(&item)` is true for an item of a shipping target that is not test code;
`files()` yields each source file once, with its package (the package of the first
target in target order that reaches the file), the targets and module paths
that reach it, and whether it ships; `source(file)` is the exact text that was indexed.

### Test target selection
<!-- raf:tested-by raf_source::test_crate::symbol_index::test_functions_carry_the_package_target_and_crate_relative_name -->

`test_fns()` returns, for each `#[test]` function (or a typed error if one's path does not start with its target's crate path), the package, the kind and Cargo
name of its target (`--lib`, `--bin <name>` or `--test <name>`) and its path inside the
crate, which is what `cargo test -p <package> --test <target> <name>` needs.

### Reported load usage
<!-- raf:tested-by raf_source::test_crate::load_budgets::usage_reports_what_a_load_spent,raf_source::test_crate::load_budgets::glob_tables_at_the_glob_budget_load_and_one_byte_less_is_refused -->

`usage()` gives what the load spent against its budgets: the bytes read, charged to
the index and to the facts, the most charged to the glob tables at once, the most
tokens of one file, the highest value of the recursion counter in one file, the
deepest directory entered and the steps of glob table work made.

### Tree-independent index identity
<!-- raf:tested-by raf_source::test_crate::symbol_index::two_loads_are_equal_and_items_and_targets_come_in_strictly_increasing_order,raf_source::test_crate::index_disk::disk_and_memory_trees_give_equal_workspaces_and_equal_resolutions -->

The index has no order that depends on the tree implementation: two loads of one tree are equal,
and the same regular files on disk and in memory give equal indexes.

### Written import forms
<!-- raf:tested-by raf_source::test_crate::use_resolution::a_rename_a_nested_group_and_a_plain_use_each_resolve_to_the_definition,raf_source::test_crate::use_resolution::crate_self_and_super_paths_resolve_from_the_module_they_are_written_in,raf_source::test_crate::use_resolution::super_above_the_crate_root_is_an_error_naming_file_and_line,raf_source::test_crate::use_resolution::extern_crate_renames_and_leading_colon_paths_reach_the_library_from_other_targets,raf_source::test_crate::use_resolution::a_use_group_with_leading_colon_members_binds_as_the_separate_uses_do,raf_source::test_crate::use_resolution::a_leading_colon_member_of_a_use_group_reads_the_crate_root_in_edition_2015,raf_source::test_crate::use_resolution::a_crate_root_inside_a_group_under_a_path_is_a_syntax_error -->

Every `use` declaration contributes to a table of the names of its
module: renames, nested groups, `self` in a group, globs, a leading `::` (a crate of the
extern prelude from edition 2018 on, the crate root in edition 2015), `extern crate x as
y`, and the keywords `crate`, `self` and `super`; a `super` above the crate root is an
error that names the file and line. A leading `::` on a member of a braced group (`pub
use { ::std::fmt::Debug as D, crate::a::f };`) has no syn node; each member of such a
group, and of a group without a path nested in it, binds as the separate `use` with the
attributes and visibility of the whole declaration would, and its facts span the whole
declaration. A `::` inside a group under a path (`a::{::b}`), which rustc refuses, is a
syntax error.

### Edition and dependency import heads
<!-- raf:tested-by raf_source::test_crate::use_resolution::a_use_path_of_edition_2015_starts_at_the_crate_root,raf_source::test_crate::use_resolution::a_crate_name_in_an_import_is_the_dependency_its_target_declares,raf_source::test_crate::use_resolution::a_dev_dependency_is_named_only_by_test_code_even_under_a_normal_dependencys_name -->

From edition 2018 on, a path whose first segment the
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
of it, brings in nothing.

### Import self-shadow and settlement
<!-- raf:tested-by raf_source::test_crate::use_resolution::a_single_segment_import_of_a_dependency_is_an_external_alias_and_never_its_own_cycle,raf_source::test_crate::use_resolution::a_single_segment_import_of_a_workspace_library_binds_its_crate_root,raf_source::test_crate::use_resolution::a_use_path_of_edition_2015_starts_at_the_crate_root -->

An import never resolves through its own binding: the
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
never `Cyclic`, and a trait key through it is the crate's absolute path.

### Crate-head glob ambiguity
<!-- raf:tested-by raf_source::test_crate::use_resolution::a_use_head_from_a_glob_that_is_also_an_extern_prelude_crate_is_ambiguous,raf_source::test_crate::use_resolution::a_no_std_crate_does_not_name_std_without_an_extern_crate_item,raf_source::test_crate::use_resolution::a_glob_that_brings_in_an_import_of_another_crate_collides_with_the_crate,raf_source::test_crate::use_resolution::a_glob_against_a_workspace_library_is_ambiguous_in_every_declaration_order -->

In edition 2018
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
refuses as E0659, is not refused, and a path through the name reads the crate.

### Rechecking settled import heads
<!-- raf:tested-by raf_source::test_crate::use_resolution::a_crate_name_an_explicit_import_binds_is_never_read_as_the_crate_in_every_order,raf_source::test_crate::use_resolution::an_import_binds_what_a_name_it_read_gains_later_in_every_order,raf_source::test_crate::use_resolution::an_explicit_import_shadows_a_workspace_library_in_every_declaration_order,raf_source::test_crate::use_resolution::a_glob_whose_path_turns_out_ambiguous_brings_in_nothing_in_every_order -->

The
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
by the budget as well.

### Explicit name-space shadowing
<!-- raf:tested-by raf_source::test_crate::use_resolution::an_explicit_import_shadows_a_workspace_library_in_every_declaration_order,raf_source::test_crate::use_resolution::an_import_keeps_the_globs_out_only_in_the_name_spaces_it_binds_in_every_order,raf_source::test_crate::use_resolution::an_import_of_a_value_named_like_a_crate_leaves_the_crate_and_the_globs_in_every_order -->

A module's own item or explicit import of the name shadows the
crate without ambiguity, whatever the order: a path, `use` path or not, whose first
segment an explicit import of its module keeps from the globs in the type name space
(see below) never starts at a crate; it waits until the import binds the name and goes
on through what the import binds, and finds nothing there if the import never binds it
(an import of an item outside the index, or one that fails or is part of a cycle). An
import of a function under a crate's name (`use crate::vals::shared;`) leaves the type
name space, so `pub use shared::x;` beside it reads the library `shared`, and beside a
glob that brings in a module `shared` as well, through another module, is ambiguous;
and a glob that brings in only a value, or the crate
itself (`pub use serde;`), does not collide with it.

### Malformed dependency declarations
<!-- raf:tested-by raf_source::test_crate::trait_keys::a_dependency_package_that_is_not_a_string_is_an_error_naming_the_key,raf_source::test_crate::trait_keys::an_inherited_dependency_the_workspace_cannot_give_is_an_error_naming_the_key -->

A `package` value that is not a string is
a manifest error that names the dependency key, and so is a `workspace = true`
dependency when there is no workspace manifest above the package, when the package is
not a member of it, or when its `[workspace.dependencies]` has no entry of that name.

### Explicit imports and undecided name spaces
<!-- raf:tested-by raf_source::test_crate::use_resolution::definitions_and_explicit_imports_shadow_globs,raf_source::test_crate::use_resolution::explicit_imports_shadow_globs_for_the_modules_that_glob_the_importer,raf_source::test_crate::use_resolution::an_explicit_import_shadows_the_glob_whether_or_not_the_index_sees_what_it_binds,raf_source::test_crate::use_resolution::an_import_that_binds_its_name_in_one_name_space_leaves_the_others_to_the_globs,raf_source::test_crate::use_resolution::an_import_keeps_the_globs_out_only_in_the_name_spaces_it_binds_in_every_order -->

A name that a
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
decided in no longer waits on what it is undecided in.

### Growing and ambiguous glob answers
<!-- raf:tested-by raf_source::test_crate::use_resolution::an_import_binds_what_a_name_it_read_gains_later_in_every_order,raf_source::test_crate::use_resolution::an_ambiguity_a_glob_cycle_carries_beside_a_glob_of_one_of_its_items_is_one_answer_in_every_order,raf_source::test_crate::resolve_paths::a_path_that_starts_with_a_name_two_globs_bring_in_as_different_modules_is_ambiguous_in_every_order,raf_source::test_crate::resolve_paths::a_path_that_starts_with_a_name_two_globs_bring_in_as_one_module_still_resolves_in_every_order -->

What the globs of a module answer may grow as they bring in
more: a path that read the name is read again, and two items make it ambiguous, as in
rustc, so a module that globs both that module and another that brings in a function
`A` reads both functions, an ambiguity, whatever the order. A name that a module brings
in ambiguously through its globs stays `Ambiguous` for a module that globs it, also where
another glob of that module brings in one of the items: rustc then only warns
(`ambiguous_glob_reexports`) and resolves the name, and the index refuses it.

### Deferred empty-name decisions
<!-- raf:tested-by raf_source::test_crate::use_resolution::a_name_space_binds_nothing_only_once_the_globs_that_may_bring_it_in_are_settled_in_every_order,raf_source::test_crate::use_resolution::a_name_space_binds_nothing_only_once_the_globs_its_globs_reach_are_settled -->

That a name space binds
nothing is decided only once no glob that reads it can still bring anything in, its
own path left out as rustc leaves out the import it resolves: a glob whose path does
not resolve yet, or whose targets' globs are not settled, holds the decision back, and
a glob's own path is read without the names the glob itself may still bring in.

### Glob visibility and settlement
<!-- raf:tested-by raf_source::test_crate::use_resolution::globs_import_only_the_names_the_importer_can_name,raf_source::test_crate::use_resolution::a_cycle_through_a_glob_the_importer_cannot_access_is_no_cycle_in_every_order,raf_source::test_crate::use_resolution::a_name_an_import_the_reader_cannot_access_binds_is_hidden_from_it_in_every_order,raf_source::test_crate::use_resolution::an_item_a_glob_reaches_by_a_public_and_a_private_route_is_visible_in_every_order -->

A glob brings in only the names the
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
whichever route is kept, so a module that globs it sees the item in every order.

### Fixed-point import propagation
<!-- raf:tested-by raf_source::test_crate::use_resolution::a_long_chain_of_re_exports_resolves_in_either_order_without_a_cap,raf_source::test_crate::use_resolution::an_import_binds_what_a_name_it_read_gains_later_in_every_order,raf_source::test_crate::use_resolution::a_mutual_super_glob_resolves_a_real_item_in_both_directions -->

The tables are settled to a fixed point: an import is tried again whenever a
name its path read gains a binding, so it binds what its path reaches in the settled
tables, a chain of re-exports of any length resolves, nothing caps the search, and a
cycle of globs adds nothing.

### Shared glob views and incremental work
<!-- raf:tested-by raf_source::glob_scope::work_budget::each_shape_loads_at_its_measured_work_and_refuses_one_step_less,raf_source::glob_scope::tests::a_read_that_holds_no_companion_starts_no_search_for_the_holders_of_its_items -->

What a glob brings in from its target depends on the
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
bind nothing is decided in the queue.

### Undecided import components
<!-- raf:tested-by raf_source::test_crate::use_resolution::a_cycle_of_imports_is_cyclic_beside_a_chain_that_ends,raf_source::test_crate::use_resolution::imports_that_wait_on_one_another_through_globs_are_decided_as_rustc_does_in_every_order,raf_source::test_crate::use_resolution::a_cycle_of_imports_with_one_glob_on_it_resolves_through_the_glob_in_every_order,raf_source::test_crate::use_resolution::a_cycle_through_a_glob_and_a_renamed_import_is_decided_in_each_name_space_in_every_order,raf_source::test_crate::use_resolution::a_path_through_an_import_a_cycle_leaves_undecided_in_another_name_space_is_cyclic,raf_source::test_crate::use_resolution::a_cycle_through_a_renamed_import_that_has_bound_nothing_is_cyclic_in_every_order,raf_source::test_crate::use_resolution::a_cycle_left_to_a_glob_reads_all_that_the_glob_brings_in_once_its_round_is_decided -->

When nothing more resolves,
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

Companion-bearing reads use holder proofs scoped to the item, name, namespace and exact lookup place, including alias visibility. Full proof domains retain shared holder and dominator state. The unique multi-read-module domain uses a separately keyed `CutHolding` proof; other domains use the complete `Holding` proof. Both apply the same exact place and alias policy, retain unsupported or open branches, and admit their storage and work under the original grants. Membership alone does not prove independence or replacement of a returning route. Cached state retains its original owner, and transient proof storage is released after use. These operation charges establish resource boundaries, not a numerical CPU-cost guarantee.

Two more rules of rustc decide what a reader can see through globs. First, a named `pub use`, or a glob, that reads a binding through a glob its module cannot access keeps the narrower of its own visibility and the visibility of the binding it reads, measured on the ancestor chain of the importing module, so a module that globs such a re-export cannot read a name that rustc hides from it. With `mod x { pub struct A; }` and `mod m { use crate::x::*; pub mod c { pub use super::*; } }`, a crate that rustc accepts, the module `m` reads the struct `A` as `c::A` and a module outside `m` does not: `resolve_from("demo::m", "c::A")` finds `demo::x::A`, and `resolve_from("demo", "m::c::A")` is `NotFound`, where rustc refuses `use m::c::A;` in the crate root (E0603). Routes to one item combine by the widest, so an item that a capped re-export and a wider glob both bring in is as visible as the wider. Second, a name that two globs of a module bring in as different items is ambiguous in that module, and it is visible to every reader that can access at least one of the items: such a reader gets `ResolveError::Ambiguous` listing every candidate, also the item it could reach only through a glob it cannot access, where rustc refuses the path (usually as E0659, in one shape as E0573), and a reader that can access none of the items finds nothing. One item reached through both globs is one candidate, and a name that only one glob brings in is not ambiguous. The ambiguity passes to the readers of a glob of the module. An import that a reader cannot name still keeps its name from the globs, and a module's own item shadows its globs for every reader, so neither makes the name ambiguous. A path through a module that two globs bring in as different modules is ambiguous, also for a name only one of the two modules has, and so is a path whose first segment is a bare name that two globs bring in as different modules, as a later segment is: `s::A`, read in a module whose globs bring in `p::s` and `q::s`, is `Ambiguous` in that module, in its child that globs it and in a module that globs it, in the type and in the value name space, and an import that starts with that name binds nothing, as rustc refuses it (E0659). Two globs that bring in one module are one candidate, and so are the two `cfg` alternatives of one module that a glob brings in, at the first segment and at a later one, so those paths resolve.

A direct definition or named binding shadows globs in its own module and namespace. A separate glob route delivering one of an ambiguity's candidates does not erase the companion carried by another route. Removing a returning companion requires the settled carrier proof for that specific identity and route; an independently accessible item or the mere presence of a cycle supplies no such proof. An unresolved branch retains its competitors. Thus `mm`, which globs an ambiguous `m0` and a separately named route through `w`, retains both items. Likewise the returning `r`/`r::t` example retains the ambiguity between `d::X` and `p::X` for every reader. A neighbouring route with the hidden competitor removed resolves normally.

`resolve(path)` takes a canonical path such as `demo::inner::go` (a segment may be
written as a raw identifier: `r#match` and `match` are the same name) and tries the type,
value and macro name spaces in that order; `resolve_in(path, namespace)` takes one. A
path may end at a method: `demo::Thing::new`. A canonical path starts at a crate path
and names each item by its name, so a segment that is a path keyword, `crate`, `self`,
`super` or `Self`, in any position and also written as a raw identifier, makes the text
`Invalid`: `demo::crate::a::x`, `demo::self::a::x` and `demo::super::x` are `Invalid`
beside `demo::a::x`. The crate path is found by prefixes that grow a segment at a time, so the cost of a path is linear in its length: a path of 50,000 segments, found or missing, is read at most four times for each segment and looked up at most twice for each segment, counted in a debug build as in a release build. The result is a `Resolution`: the
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
`Invalid` when the text is not a path. `IndeterminateImport` names a selected
import whose complete namespace proof cannot be established, even when a peer
namespace appears bound. Native budget and storage failures retain their typed
causes and take precedence over a semantic answer. Such a glob (`use self::missing::*;`, `use
crate::a::gone::*;`, or the glob of a module that a macro generates) does not fail the
load, which brings in nothing through it, but what it would bring in is not known, so
every lookup that reads it fails, naming the path asked for, the glob's written path and
the module that declares it: a lookup of a name that the module defines or imports
explicitly in the name space looked up does not read the module's globs, and one that
reads a glob that resolves reads the globs of the modules it globs in turn, so a sibling
path that reads none of them resolves, while a name that a glob that resolves brings in
beside such a glob fails, as the broken glob could make it ambiguous. Which globs
name nothing is computed under admission while the index is built and retained
for later queries. A name space whose lookup reads the glob fails
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

### Module-relative resolution
<!-- raf:tested-by raf_source::test_crate::resolve_paths::a_use_path_counts_only_what_the_reading_module_can_access,raf_source::test_crate::resolve_paths::a_use_path_is_read_in_one_name_space_or_in_each_in_turn,raf_source::test_crate::resolve_paths::a_module_that_is_not_in_the_workspace_and_a_text_that_is_no_path_are_refused -->

`resolve_from(module, path)` resolves a path as a `use` written in the module `module` would (a canonical module path such as `demo::inner`), trying the type, value and macro name spaces in that order, and `resolve_from_in(module, path, namespace)` takes one name space. The reading module is the viewer: the path starts as code of that module starts it (`crate`, `self`, `super`, a name the module binds, or a crate of the extern prelude) and each segment counts only the bindings that module can access, as rustc settles an import, where `resolve` takes a canonical path and checks no visibility. A path that names nothing the module can access is `NotFound`, one that names several symbols is `Ambiguous`, and a module that is not in the index is `ResolveError::UnknownModule`. A reading module whose crate path is shared by libraries of several packages, two libraries with one crate name, is `Ambiguous` and lists their crate roots, as `resolve` answers for a path that starts with that crate path; a module of any other crate reads as before.

`resolve_from_package_id(package, module, path)` selects the reading module from
the exact load-local `PackageId`; `resolve_from_package_id_in(package, module,
path, namespace)` also selects one name space. Both reuse ordinary import and
visibility traversal, including aliases and reexports, while preserving each
definition's source authority. An absent package identity is
`ResolveError::UnknownPackageId`; a module absent from that package is
`ResolveError::UnknownModule`, even when another loaded package defines it.
Selection does not fall back to another package with the same displayed crate
name, and does not establish locked source correspondence or active configuration.
<!-- raf:tested-by raf_source::test_crate::discovery_targets::relative_queries_select_the_reading_package_without_crossing_source_roots -->

`Workspace::dependency_declarations(package)` returns the normal, development and
build declarations of that exact package, including unevaluated target-table
conditions. Each `DependencyDeclaration` retains its alias, kind and original
key. Its `DependencySpec` retains version, path, registry and Git selectors,
optional and default-feature flags, and requested features. An inherited entry
also retains the workspace specification and declaring manifest, preserving its
path base separately from local feature and optional settings. Malformed field
types, version requirements and conflicting source selectors are manifest
errors; retained declarations use the original item allowance. These observations
do not compute active configuration or prove a lock edge or source correspondence.
<!-- raf:tested-by raf_source::test_crate::dependency_declarations::declarations_retain_sources_flags_aliases_and_unevaluated_conditions,raf_source::test_crate::dependency_declarations::inheritance_preserves_both_entries_and_the_workspace_path_base,raf_source::test_crate::dependency_declarations::malformed_declarations_refuse_with_valid_neighbors,raf_source::test_crate::dependency_declarations::retained_declarations_use_the_original_item_allowance -->

Given an explicit owning lock row, `LockedCatalog::select_dependency` selects
exactly one of its outgoing rows using the declaration's package rename, version
requirement and source constraints, including inherited workspace constraints.
Registry declarations require a matching explicit `RegistryIndex` configuration
unless an index is written directly; registry names and default indexes are not
guessed. Registry and sparse families remain distinct, as do Git default-branch,
branch, tag and revision references. A path declaration selects a local row for
a local owner, or the owner's exact Git source for a Git workspace member.
Missing, ambiguous, malformed and incompatible evidence refuses, with
native storage admission and comparison work bounded by the supplied options.
The selected row is an observation of a locked relationship: it does not decide
optional-feature or target activation, verify source contents, or bind a supplied
source tree to a loaded `PackageId`.
<!-- raf:tested-by raf_source::test_crate::locked_selection::rename_version_and_registry_select_only_the_owners_exact_outgoing_row,raf_source::test_crate::locked_selection::git_reference_kind_and_local_source_family_are_constraints_not_tree_proofs,raf_source::test_crate::locked_selection::workspace_inheritance_selects_its_package_and_source_without_evaluating_cfg,raf_source::test_crate::locked_selection::absent_mismatched_and_malformed_registry_configuration_refuses_beside_valid_indexes,raf_source::test_crate::locked_selection::selection_honors_work_and_retained_catalog_item_admission,raf_source::test_crate::locked_selection::malformed_observations_and_unknown_owners_refuse_without_an_alternate_identity -->

`CommittedGitTree::open` admits a selected full SHA-1 Git lock revision through
trusted installed Git and its local object store. Git decodes committed objects;
RAF collects a bounded complete NUL-delimited listing and strictly checks its
fields, paths, parent directories, duplicate names and supported modes. Missing
Git, repository or objects, command failures, malformed protocol and gitlinks
refuse. Symbolic links can be listed but never traversed. Exact-size blob reads
also check the listed native SHA-1 blob identity before returning text or `Seen`.
Source bytes retain exact UTF-8 spelling and line endings; equal-content distinct
paths have distinct file identities, and inside-root parent spellings identify
the same file. Caller authority and ordinary FileId constructors cannot create
this sealed admission.
<!-- raf:tested-by raf_source::test_crate::git_tree::checked_snapshot_serves_real_bytes_and_existing_loader_with_distinct_path_identity,raf_source::test_crate::git_tree::missing_commit_repository_and_wrong_object_kind_refuse_beside_the_exact_commit,raf_source::test_crate::git_tree::missing_or_substituted_blob_is_a_hard_failure_even_when_seen_would_skip_text,raf_source::test_crate::git_tree::malformed_tree_and_submodule_refuse_with_an_ordinary_tree_neighbor,raf_source::test_crate::git_tree::missing_installed_git_is_an_actionable_dependency_failure,raf_source::tree::git::tests::complete_directory_protocol_refuses_malformed_records_beside_exact_metadata -->

The host adapter requires Git supporting disabled replacements, lazy fetching and
optional locks; unsupported commands fail. It uses committed bytes exclusively,
without checkout reads, fetching, external filters or shell execution. Worktree
edits and untracked files cannot affect its source view; this does not certify a
checkout clean or prove repository authorship. RAF's protocol buffers and
processing share the load's original grants. Git's internal CPU, allocations and
object-store access are trusted host work, outside the portable semantic meters
and DiskTree's handle-containment contract. Targets without process access cannot
construct this host Tree. It does not evaluate active configuration or preferred
provider policy.
<!-- raf:tested-by raf_source::test_crate::git_tree::committed_bytes_ignore_worktree_edits_and_replacement_refs,raf_source::test_crate::git_tree::missing_installed_git_is_an_actionable_dependency_failure,raf_source::tree::git::command::tests::protocol_size_is_checked_before_growth_and_exact_size_has_a_valid_neighbor -->

`Workspace::load_locked_sources` joins explicit local manifest locators and opaque
checked Git snapshots to actual discovered `PackageId`s. `LockedManifest` names a
candidate manifest and lock row; neither its path nor an authority label is proof.
The load checks actual names and explicit or inherited versions. Git packages
must be the unique matching package in the checked repository's root package or
root workspace membership; a nested fixture with matching labels cannot replace
that package. Other Git repository layouts are refused by this binding profile.
`locked_binding(PackageId)` exposes the computed row/name/version/source receipt.
<!-- raf:tested-by raf_source::test_crate::locked_sources::checked_contents_do_not_let_wrong_manifest_names_or_versions_acquire_a_row,raf_source::test_crate::locked_sources::a_nested_same_name_and_version_cannot_replace_the_root_workspace_package,raf_source::test_crate::locked_sources::actual_version_evidence_is_required_without_changing_ordinary_missing_version_loading,raf_source::test_crate::locked_sources::another_checked_repository_cannot_supply_an_expected_source_by_name_or_authority -->

Only explicitly requested `LockedEdge` declaration keys receive cross-package
targets. Their exact owning row selects an outgoing destination, which must have
an admitted manifest; path declarations must also name that actual destination
directory. Alias and reexport resolution then uses the existing resolver and real
destination package identity. Unrequested edges remain unresolved syntax and do
not borrow same-named packages. Requested optional or target-conditional edges
refuse without configuration admission. Catalog retention, checked snapshots,
selection and ordinary source walking share one original resource grant.
Without explicit configuration this proves committed source/package correspondence
only. It does not imply active definitions, preferred-provider policy, repository
authorship or dirty-checkout correspondence.
<!-- raf:tested-by raf_source::test_crate::locked_sources::exact_sources_versions_and_path_edges_resolve_aliases_to_the_actual_package,raf_source::test_crate::locked_sources::requested_missing_and_optional_edges_refuse_and_unrequested_edges_never_borrow_a_name,raf_source::test_crate::locked_sources::all_snapshots_catalog_and_walker_use_one_original_resource_grant -->

`LockedManifest::with_configuration` selects one exact target;
`with_configurations` selects explicit Lib, Bin, Test, Bench, Example or BuildScript
contexts, each with declared complete target cfg, separate host cfg where required
and complete already-unified enabled features. If any package is configured,
every admitted package must supply a nonempty configuration set. Each kind/name
selects exactly one actual target; duplicate contexts and unsatisfied required
features refuse. A nonlibrary, non-build-script target requires an explicitly
configured ordinary own-library companion when that package has a library, with
the same target triple and unified features. The actual manifest graph
validates enabled features, implications, optional aliases, `dep:` suppression,
forwarding, defaults and inherited feature requirements for requested edges.
Requested edges retain their exact eligible targets. Normal dependencies belong
to every non-build-script context. Dev dependencies additionally belong to declared
Lib/Bin test modes and Test/Bench/Example contexts; Build dependencies belong to
BuildScript.
An edge inactive in every selected context, an incomplete selection, unknown
features or targets and missing destination evidence refuse. A destination selects
one actual ordinary library. Build-script conditions use declared host cfg and
other dependencies use declared target cfg; Cargo's unsupported
test/feature/debug-assertion/proc-macro dependency selectors refuse.
<!-- raf:tested-by raf_source::test_crate::configured_sources::missing_unknown_inactive_or_incomplete_configuration_refuses_beside_valid_selection,raf_source::test_crate::configured_sources::forwarding_default_and_inherited_feature_requirements_are_validated_at_the_actual_destination,raf_source::test_crate::configured_sources::target_and_host_conditions_use_the_selected_compilation_role,raf_source::test_crate::configured_sources::cargo_feature_names_use_rust_unicode_xid_with_actual_cargo_punctuation,raf_source::test_crate::configured_targets::all_selected_target_kinds_keep_their_native_and_own_library_routes,raf_source::test_crate::configured_targets::missing_duplicate_and_feature_ineligible_targets_refuse_beside_complete_selection,raf_source::test_crate::configured_targets::development_edges_require_a_real_development_or_declared_test_context,raf_source::test_crate::configured_targets::conditional_and_optional_edges_keep_each_targets_actual_activation -->

Configured loads select modeled item, module, import and associated declaration
headers before fact extraction and module walking. `cfg_attr` selects applied
attributes and paths; inactive module files are not opened. Opposite cfg
alternatives consequently produce different definitions in different loads.
Raw cfg names use their Rust identifier identity, including the separate feature
key. Qualified predicate names and operators refuse. An empty `cfg_attr` applied
list requires the comma after its predicate. Boolean literals differ from raw
flags named `r#true` and `r#false`.
`LockedBinding::configuration` returns the checked context only for a singleton;
`configurations` retains the complete set and `configuration_for(kind, name)`
selects the exact target receipt. Root test identity follows declared `cfg(test)`,
without inferring it from the target kind or harness setting, and
`resolve_active_from_package_id_in` refuses structural-only packages, inactive
or missing names and sites without an admitted context. Configuration scratch
and work use the original grant. A physical file can share parsed facts only
within one package and identical selected syntax predicates/features. Different
configured package contexts or incompatible views refuse instead of reusing
cached facts. Incompatible modes of one target, host/target feature closures and
other incompatible physical views require separate Workspaces.
<!-- raf:tested-by raf_source::test_crate::configured_sources::configured_aliases_resolve_only_active_real_definitions_and_paths,raf_source::test_crate::configured_sources::distinct_declared_sets_change_definitions_and_keep_their_exact_receipt,raf_source::test_crate::configured_sources::nested_cfg_attr_boolean_guards_escaped_values_and_test_builds_select_actual_declarations,raf_source::test_crate::configured_sources::configured_syntax_and_feature_work_spend_the_original_combined_grant,raf_source::test_crate::configured_sources::a_physical_file_cannot_silently_merge_different_active_package_contexts,raf_source::test_crate::configured_sources::raw_cfg_identifiers_have_one_logical_identity_and_cannot_bypass_feature_separation,raf_source::test_crate::configured_sources::configured_predicates_refuse_qualified_paths_beside_unqualified_names,raf_source::test_crate::configured_sources::empty_cfg_attr_requires_its_comma_and_boolean_literals_are_not_raw_flags,raf_source::test_crate::configured_targets::shared_files_reuse_only_the_same_actual_syntax_selection,raf_source::test_crate::configured_targets::retained_target_lists_and_configuration_queries_spend_the_original_grants,raf_source::test_crate::configured_targets::all_selected_target_kinds_keep_their_native_and_own_library_routes -->

Configuration is caller-declared, not observed Cargo state. The caller supplies
each package's complete cfg, including package-specific or build-script custom
predicates, and its complete feature selection. Requested dependencies must
agree on the selected compilation triple, but custom predicate sets may differ.
The load checks manifest implications and requested-edge requirements; it does
not infer built-in predicates from a triple or run Cargo's feature resolver.
Filtering does not traverse function-body statements/expressions, fields or enum
variants, rewrite source bytes or produce cfg-specific Census fingerprints.
Macro expansion, type inference and preferred-provider policy are separate.
<!-- raf:tested-by raf_source::test_crate::configured_sources::target_and_host_conditions_use_the_selected_compilation_role,raf_source::test_crate::configured_sources::distinct_declared_sets_change_definitions_and_keep_their_exact_receipt -->

### Target-local resolution
<!-- raf:tested-by raf_source::test_crate::target_resolution::a_helper_resolves_from_an_integration_test_root_and_from_a_module_below_it,raf_source::test_crate::target_resolution::a_helper_resolves_from_a_nested_module_of_a_binary,raf_source::test_crate::target_resolution::an_example_a_bench_and_the_build_script_are_read_from_as_well -->

`resolve_from` and `resolve_from_in` read from a module of any target, named by its crate path: `<pkg>::test_crate::<target>`, `<pkg>::bench_crate::<target>`, `<pkg>::example_crate::<target>`, `<pkg>::bin_crate::<bin>` and `<pkg>::build_script`, and the modules below each.

### Target-origin lookup cost
<!-- raf:tested-by raf_source::resolve::tests::a_module_path_of_fifty_thousand_segments_is_read_once_to_find_its_crate -->

The crate path that starts the module path is found by prefixes that grow a segment at a time, as for `resolve`, so a module path of 50,000 segments is read once for each segment.

### Target extern preludes
<!-- raf:tested-by raf_source::test_crate::target_resolution::a_helper_resolves_from_an_integration_test_root_and_from_a_module_below_it,raf_source::test_crate::target_resolution::an_example_a_bench_and_the_build_script_are_read_from_as_well -->

A path read there starts as code of that target starts it: `crate` is the target's root, and the library of its own package and the crates of the workspace it may depend on are crates of its extern prelude, so `resolve_from("demo::test_crate::it", "demo::api")` finds the library's `api` and `crate::api` there is `NotFound`; the build script does not name the library of its package.

### Shared-file target origins
<!-- raf:tested-by raf_source::test_crate::target_resolution::a_support_module_that_two_test_targets_include_resolves_in_each_target_alone -->

A file that several targets include, such as a support module of the integration tests, is one module of each target that includes it, and a path read from one of them resolves in that target only: `support::helper` read from `demo::test_crate::a` is `demo::test_crate::a::support::helper`, one site, and the copy of target `b` is another site of the same source item, which `same_source` relates.

### Invalid and ambiguous target origins
<!-- raf:tested-by raf_source::test_crate::target_resolution::a_module_path_that_no_target_holds_is_unknown_whichever_kind_of_target_it_names,raf_source::test_crate::target_resolution::a_module_of_the_library_at_the_crate_path_of_a_test_target_is_ambiguous_with_it -->

A module path that no target holds is `ResolveError::UnknownModule`, whichever kind of target it names, and a module path that is a module of two crates, as a module `test_crate::it` of the library beside the test target `it`, is `Ambiguous` between the items that stand for the two modules.

### Impl records
<!-- raf:tested-by raf_source::test_crate::symbol_index::every_impl_block_is_listed_in_file_order_with_its_span_module_self_type_key_and_members,raf_source::test_crate::symbol_index::a_negative_impl_keeps_its_bang_in_the_trait_path_beside_a_positive_one -->

`Workspace::impls()` lists every `impl` block of every target, inherent and trait alike, once for each module path its file is reached under, in file order: sorted by file, the byte where the block starts, module path and target. Each is an `ImplBlock`: its `id`, an `ImplId` that is its place in that order and that `Workspace::impl_block(id)` takes back, its `file` and the `line` of the `impl` keyword, its `span` from its first attribute or the `impl` keyword to its closing brace, the `module` that holds it and its `target`, its `test` role, its written `self_type` and `trait_path` without spaces, its `self_items`, the key of its self type, and its `members` in source order. The key is the struct, enum or union that the self type path names from the block's module, through type aliases, as the members of the block are placed (two for the `cfg` alternatives of one type); it is empty when the self type names nothing of the workspace or a type the index cannot place: a generic parameter, a reference, a slice, a tuple, a type of a dependency or one that a macro generates, as for `impl Tr for String`, `impl<T> Tr for T`, `impl Tr for &Foo` and `impl Missing`. A negative impl keeps its `!` in `trait_path` (`!Sync`). An impl block inside a function body or any other block expression, such as `const _: () = { .. };`, is not an item of a module and is not listed, nor is one that a macro invocation would generate. Each record is charged to the item-bytes budget at the line of its `impl` keyword.

### Reachability edges
<!-- raf:tested-by raf_source::test_crate::reachability::a_private_helper_called_by_a_bound_item_is_reached_and_an_unreferenced_one_is_not,raf_source::test_crate::reachability::a_method_call_a_field_a_lifetime_and_a_local_name_no_item_and_a_path_prefix_names_its_type -->

`Workspace::reachable(roots)` takes nodes, `Node::Item` and `Node::Impl`, and gives the items and impl blocks of every target that the roots reach over three edges. Contains: a struct, enum or union to its methods and associated items, a trait to its members, and a module to its items and to the impl blocks it holds; a crate root contains nothing, and a module is opened only when it is a root or is reached by contains from an opened module, so a module that a path names reaches nothing below it, and one first named by a path and then reached by contains is opened then. Impl-of: a struct, enum or union to the impl blocks whose self type key it is, and a block to its members. References: an item or a block to each item that a path in its own tokens names. The own tokens of an item are those of its span outside the spans of the items and blocks below it, so a type, a trait and a block leave their members to themselves; a module's own tokens are its declaration before its body, and what a module body holds outside its items and blocks, its `use` declarations, a `const _` and the macro invocations at module level, is no item's; a crate root has none. A path is an `Ident (:: Ident)*` run of those tokens: an identifier that a single `.` precedes (a method call or a field), that follows a `'` (a lifetime or a label) or a `$` other than `$crate` (a metavariable), that names what `fn`, `struct`, `enum`, `union`, `trait`, `type`, `mod`, `static` or `macro_rules!` declares, or that is a run of one segment before a single `:` (a name being bound or a field, as in `helper: u8`, and so the name of a constant or of a const parameter) starts no run, except in a `where` clause, from its keyword to the `;` or the braced group that ends it, where a run before a single `:` is the type a bound is put on (`where Helper: Clone` names `Helper`). `const` declares nothing on its own, so `*const T` names `T` and `&raw const X` names `X`. `crate`, `self` and `super` start a run and `super` goes on after them, `Self` starts a run that is read as below, every other keyword is no segment and ends the run, and what follows such a word through `::` is no run (`a::self::b` and `a::Self::b` give none past `a`). An identifier is a segment only where the text spells it: outer doc comments are lexed as `#[doc = "..."]` and inner doc comments as `#![doc = "..."]`, with the `doc` identifier written nowhere in the comment, so it starts no run and no byte of the comment text is read as a segment, whatever character ends it, while a written `#[doc]` names `doc` as any attribute names its path.

### Edition-sensitive references
<!-- raf:tested-by raf_source::test_crate::reachability::gen_is_a_name_before_edition_2024_and_a_keyword_from_it,raf_source::test_crate::reachability::a_shared_file_reads_keywords_in_each_reached_packages_edition -->

The keywords are those of the edition of the package of the reached item: `gen` is a segment in editions 2015 to 2021 and a keyword from 2024, and `async`, `await`, `dyn` and `try` are segments in 2015. A raw identifier is never a keyword, and a run that `::` opens after `>` (`new` in `Vec::<u8>::new`) is none. The run is read as code of the item's module reads it, test code as test code, which also names the dev-dependencies: each proper prefix in the type name space, and the whole run in each name space. Each is walked as the resolver walks a path, as code reads it where `resolve_from_in` reads a `use` path: without the reading module's visibility, and in edition 2015 from the module and not from the crate root. It names the sites its walk reaches when they are one symbol; a prefix whose walk fails, through a name that names nothing, an import that never settles, a glob whose path names nothing or a name that names several symbols, names nothing, and every longer prefix with it. So `E::V` names the enum `E`, `S::new` names `S` and its method, and a path to a local, a generic, an item of a dependency or an item that a macro generates names nothing. The module of a method or associated item of an impl block is the block's module, that of a trait member its trait's, and a `#[macro_export]` macro is read from the crate root where it is indexed. A macro that a reached item invokes by name (`make!`) is reached, and the paths of its body are read from its module.

### Decoded format references
<!-- raf:tested-by raf_source::test_crate::format_reach::implicit_captures_and_named_counts_are_value_edges_of_the_actual_function,raf_source::test_crate::format_reach::named_bindings_suppress_capture_and_left_hand_path_without_hiding_the_expression,raf_source::test_crate::format_reach::cooked_raw_unicode_keyword_and_count_bindings_use_decoded_names -->

A standard `format!` invocation with a literal format argument contributes its
decoded implicit argument names and named width and precision counts as value
references of the item that holds the invocation. Cooked escapes, raw strings,
Unicode names and keyword names are read from the decoded string. Numeric
positions, doubled braces and ordinary string literals contribute no capture.
An explicit named argument suppresses the corresponding implicit capture and its
left-hand binding identifier is not an ordinary value path; the argument's
expression retains ordinary path references, including closure and generic commas.
Captures are resolved as value names from the item's module and test context,
with the same ambiguity and unresolved-import refusals as other references.

### Standard macro identity
<!-- raf:tested-by raf_source::test_crate::format_reach::direct_import_aliases_and_block_imports_are_positive_provenance,raf_source::test_crate::format_reach::textual_macro_shadows_begin_after_the_declaration_and_do_not_leak_from_a_block,raf_source::test_crate::format_reach::unknown_named_and_glob_imports_never_become_standard_macro_certificates,raf_source::test_crate::format_reach::prelude_changes_and_std_head_shadows_refuse_inferred_captures,raf_source::test_crate::format_reach::block_local_std_aliases_are_checked_at_the_imports_own_scope,raf_source::test_crate::format_reach::a_dependency_named_std_and_a_resolved_workspace_macro_are_not_the_sysroot -->

The extra references require positive standard-macro provenance: the enabled
standard prelude's canonical name, a direct `std::format` path in edition 2018 or
newer, or a unique direct import of that endpoint, including a rename or block
import. Textual macro shadows begin after their declaration; imports occupy their
containing scope, without leaking into an inner module. The `std` head of an import
is checked in the import's own lexical scope. Local and dependency `std` shadows,
competing named imports, uncertain external globs, `macro_use`, and attributes that
could disable the standard prelude do not certify a standard macro. A missing
ordinary lookup is never such a certificate.

### Format admission and owned context
<!-- raf:tested-by raf_source::test_crate::format_reach::recursive_format_expressions_are_admitted_before_parsing_and_custom_tokens_stay_opaque,raf_source::test_crate::format_reach::malformed_standard_format_input_is_actionable_while_a_custom_macro_keeps_its_grammar,raf_source::test_crate::format_reach::nested_standard_expression_macros_are_transparent_but_custom_token_inputs_are_not_calls,raf_source::test_crate::format_reach::one_physical_file_is_recognized_per_target_module_edition_and_root_order,raf_source::reach::tests::format_candidates_charge_owned_bytes_and_return_worker_work_to_the_caller -->

Candidate argument expressions and lexical imports pass the source pre-scan before
recursive parsing on the owned lexer worker. An identified standard call with
malformed fields or arguments, or arguments beyond the pre-scan budgets, produces
`ReachError::Source` naming the actual source and line. A custom macro's tokens
retain that macro's grammar. Nested candidates are interpreted only through
positively identified standard expression-taking macro contexts; arbitrary token
inputs, including stringification, remain opaque to capture inference. Lexical
candidates are cached per file and edition, while identity and value resolution use
the actual module, target, test flag and lexical position. Their owned scopes,
names, capture strings, refusal messages and placement rows count against the
item-bytes budget. The worker returns token and source/decode byte-work counts to
the caller; counted repeated-call fixtures grow by equal steps in both work and
retained-byte charges and refuse one byte below the admitted charge.

### Self-member references
<!-- raf:tested-by raf_source::test_crate::reachability::self_paths_and_methods_called_on_self_reach_the_members_of_that_name,raf_source::test_crate::reachability::self_in_a_trait_and_in_its_impl_blocks_reaches_the_members_of_the_trait_and_of_the_block,raf_source::test_crate::reachability::a_variant_through_self_names_its_enum_as_a_path_through_the_enum_does -->

A run that `Self` starts, and the name of a method called on `self` (`self.name(..)` or `self.name::<..>(..)`), are read in the impl block or the trait that holds the unit, not walked as paths. In a member of an impl block, or in the block's own tokens, `Self::name` and `self.name()` name the members called `name` of the struct, enum or union that the block's self type key names, in every block of it, this one among them, or of the block itself when no type keys it, and the members called `name` of the trait that the block implements, its path read from the block's module. In a trait or a member of one, they name the trait's members called `name`. Only the segment after `Self` is read, so `Self::Assoc::f` names `Assoc`. A `Self::name` that names no member, in a block whose self type key is an enum, names the enum, as `E::V` names `E`: `name` is a variant. `Self` alone, any other `Self::name` that names no member (as `Self::default()` that a derive makes), and a method called on any other value name nothing: the methods of a value's type are reached through the type.

### Reachability result and identity scope
<!-- raf:tested-by raf_source::test_crate::reachability::the_answer_does_not_depend_on_the_order_of_the_roots_and_is_the_same_each_run,raf_source::test_crate::reachability::a_root_past_the_items_or_the_blocks_of_the_workspace_is_refused_beside_one_in_range -->

The answer is a `Reachability`: `reached`, every reached node once as a `Reached` with the `edge` it was first reached by (`Root`, `Contains`, `ImplOf` or `References`) and the node that edge comes `from`, breadth first from the roots in the order of their nodes, so it does not depend on the order or the repeats of the roots, and `get(node)`, `contains(node)`, `items()` and `impls()`. A root past the items or the impl blocks of the workspace is `ReachError::UnknownItem` or `ReachError::UnknownImpl`. An `ItemId` or an `ImplId` is a place in the workspace that gave it and names no workspace, so an id of another workspace that is in range is read as the node of this workspace at that place. A file that does not lex, which a file the load parsed does not do, or a thread that cannot start to lex it, is `ReachError::Source`. What the walk holds is charged to the item-bytes budget of the load (`LoadOptions::max_item_bytes`), on from what the load holds (`Usage::item_bytes`), as it is built: the runs of each file and edition it lexed, with their segments and their places in their units, the texts it resolved, the tables of where each item is read, of the blocks of each type and module and of the crate roots, what it read of each impl block for `Self`, and its reached nodes, opened modules and queue at its longest. A walk whose charge passes the limit is `ReachError::Budget`, naming the budget, its limit and the file of what was being held. A run holds its record, the byte range of each segment and its place in its unit and no text, so on a 64-bit target each further run of a file of `a; ` runs is charged 56 bytes, less than twenty times the three bytes it takes; the tokens of a file are held only while it is lexed, as when it is parsed.

### Counted reachability work
<!-- raf:tested-by raf_source::reach::tests::the_work_of_a_walk_is_counted_exactly,raf_source::reach::tests::the_work_grows_with_the_items_and_the_references_and_not_their_product,raf_source::reach::tests::a_path_of_fifty_thousand_segments_is_walked_once_and_not_once_for_each_prefix -->

The work is counted, never timed: each file of a reached item is lexed once per package edition, on a thread of its own with the stack of the parse, whose span data is dropped with it, and its runs are given to their units in one pass; each run is resolved at most once for each module and test flag it is read with, or for each impl block or trait for a run that `Self` starts or a method called on `self`, and its prefixes in one walk of its segments, so a path of 50,000 segments reads each segment at most four times. What a run names is given to the walk when it is resolved, so a run of the same text read in the same place again gives nothing. The work of a walk grows with the items and the references of the workspace and not with their product: over a library of n functions each calling its own helper r times, the tokens read are n(9 + 3r), the steps of placing the runs nr + 4n - 3, the paths resolved n and the steps of the walk 6n + 1 + nr; over n modules below one opened module, each with a struct, an enum, a type alias, a constant, a static, a function, three impl blocks, one of them of a trait of k methods, and 2k methods that call each other through `Self::` and `self.`, every counted kind grows by equal steps in n and in k.

### Public surface visibility
<!-- raf:tested-by raf_source::test_crate::exported_items::a_pub_function_is_exported_beside_private_and_restricted_ones_which_are_not,raf_source::test_crate::exported_items::a_pub_item_of_a_private_module_is_exported_only_through_a_pub_use_and_under_its_rename,raf_source::test_crate::exported_items::a_glob_re_export_brings_in_the_pub_items_of_its_module_only -->

`Workspace::exported_items(target)` lists the public surface of
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
of 2 MiB. The globs of a module that bring in a name are found by the name, so the
reads of the glob views a listing makes grow in step with the names of a module of
`pub` globs of one name each: three sizes of crate add the same reads each time. The modules a reader's routes reach are walked once, into a graph, for the
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
the chain bind, is refused, beside a chain of 50 globs to 200 such names, which lists. Whether a route avoids a set of several modules
needs that search, as no structure computed once for the reader answers it for
every set the way the tree answers it for one module. `pub` globs that each
carry a re-export of their own name, beside globs of modules that each bind that
name privately, list in walk steps and glob view reads that grow in step with the
names, and 3000 of them are charged under 32 MiB beyond the load; walking the
reach again for each name would make the steps grow with the square of the names.

The two visibility rules of the use-alias index reach the surface as follows. A listing that meets the second rule, a name that two globs of a listed module bring in as different items, fails with `ExportError::Resolve(ResolveError::Ambiguous(_))` and lists every candidate, whatever the order of the declarations. The first rule never lists an item that is reached only through a re-export it caps: the surface requires every `use` declaration on the way to be `pub`, and a binding read through a private glob has no route of `pub` declarations, so the `pub use` of such a re-export lists the items it binds on its own and not the one behind the glob.

### Public API construction and extension guarantees
<!-- raf:tested-by raf_source::test_crate::memory_tree::entry_reports_file_dir_and_absence_without_an_error,raf_source::test_crate::symbol_index::files_items_in_and_item_at_answer_by_position,raf_source::test_crate::exported_items::doc_hidden_items_are_left_out_by_default_and_flagged_when_included,raf_source::test_crate::reachability::the_answer_does_not_depend_on_the_order_of_the_roots_and_is_the_same_each_run -->

The data structs a caller reads, `Package`, `Target`, `SourceFile`,
`Reach`, `Entry` and `SpanFact`, are `#[non_exhaustive]` like `Item`, so adding a field
is not a breaking change: a pattern over one names the fields it reads and ends in
`..`, and a `Tree` backend outside the crate builds an `Entry` with `Entry::new`. The
per-file facts the index is built from are not public. `LoadOptions` is `Clone` and
not `Copy`, so adding an option that is not `Copy` is not a breaking change. `Exports`,
`ExportedItem`, `Reexport`, `ExportOptions` and `ExportError` are `#[non_exhaustive]`
as well, so a caller builds `ExportOptions` from `ExportOptions::default()`, and so are
`ImplBlock`, `Edge`, `Reached`, `Reachability` and `ReachError`.

## Usage

Write Cargo version constraints in dependency declarations, validated by
`cargo_version`, and select an outgoing
lock row with `LockedCatalog::select_dependency`. Retain the complete package
version, including build metadata, when identifying a locked row; matching a
requirement does not replace that identity.
<!-- raf:tested-by raf_source::test_crate::locked_selection::rename_version_and_registry_select_only_the_owners_exact_outgoing_row -->

Set `LoadOptions::max_query_work` before loading to constrain each later query
independently of `max_work`. Preserve a resource error from resolution, library
membership or reachability; it does not establish that a symbol is absent.
<!-- raf:tested-by raf_source::glob_scope::lookups::every_way_of_resolving_a_path_is_a_query_with_its_own_limit,raf_source::test_crate::code_paths::external_library_query_scratch_uses_the_original_native_grants -->

Call `capture_file` with a checked `RelPath` and byte ceiling when preserving
payload and native permissions together. Use `capture_directory` to retain
membership for a later source comparison, and `capture_link` to describe a link
without reading its destination. Retain each capture's logical path and original
identity separately from any materialized copy.

Call `read_bytes` on a `DiskTree` with a checked `RelPath` and byte ceiling to
read an asset without text conversion. Call `read_unseen_bytes` with the same
arguments and an identity predicate to avoid reading an already-held payload.
Match `OpenedBytes` to distinguish the seen identity from a complete byte read.
Use ordinary `Tree` reads for source text; they use the same `disk` byte path.

Use `create_new` on a mutable `MemoryTree` for exclusive publication and
`replace` for an existing file. Both take a checked `RelPath` and owned text.
The same tree can then be read or loaded without reconstructing a reader.

Run the source checks on native Linux through `make check`. The disk fixtures use
the native Linux containment strategy; standard checks neither build another OS
target nor execute its backend on Linux.

A custom `Tree` supplies `FileId::from_key` with a string or opaque byte slice.
The `tree` implementation retains the bytes without interpreting them as a
path. Use `SourceTrees` to scope equal backend keys from independent sources;
cloning an identifier shares its key payload.
<!-- raf:tested-by raf_source::test_crate::memory_tree::a_workspace_loads_through_an_object_safe_tree_with_caller_local_state,raf_source::tree::sources::tests::equal_relative_paths_and_backend_keys_remain_separate_sources -->

Depend on raf-source and use `Workspace::load` or `LockedCatalog::load` directly;
neither path requires a parser feature selection or a serde dependency. Both
read through the supplied `Tree` and preserve their existing input bounds.

Use `load_locked_sources_under` with the desired first-party discovery roots
and their complete local associations. Attach one `DeclaredConfiguration` with
`with_configuration`, or an exact target-context slice with `with_configurations`.
Supply an ordinary own-library companion for a selected Bin/Test/Bench/Example
target when the package has a library. Obtain its receipt through the actual
target's kind/name and `LockedBinding::configuration_for`, rather than choosing
an arbitrary package configuration. At an actual owner and byte, enumerate
`extern_libraries_at` and call a borrowed receipt's `resolve` for a public API.
Use `with_configured_syntax` with the actual `TargetId` and `RelPath`. Retain
`Resolution::type_aliases` when auditing declarations behind an associated call.
<!-- raf:tested-by raf_source::test_crate::locked_sources::explicit_first_party_roots_require_every_discovered_manifest_association,raf_source::test_crate::code_paths::external_library_receipts_keep_actual_names_exports_and_definition_identity,raf_source::test_crate::code_paths::alias_members_preserve_definition_chains_cycles_and_original_query_grants,raf_source::test_crate::configured_targets::all_selected_target_kinds_keep_their_native_and_own_library_routes -->

Use `Workspace::code_resolver` for a code observation with its actual owner
`ItemId`, byte offset and namespace. `CodeResolver::reading_module` identifies
the defining module; `resolve_from_item_at` resolves the module-level name at
that observation. Lexical locals remain the caller's responsibility. The
runnable `raf_source::example_crate::code_contexts` compares this query with
import lookup in an edition 2015 child module.
<!-- raf:tested-by raf_source::test_crate::code_paths::code_heads_read_the_edition_2015_module_where_use_heads_read_the_crate_root -->

The runnable `raf_source::example_crate::registry_scan` loads a small in-memory
package, checks its module placement and prints the actual load counters when
started without arguments. Passing a registry directory runs the native child
scanner instead. The built-in package demonstrates the loader and formatter;
it does not exercise process orchestration or establish arbitrary registry
compatibility. The registry scanner retains its explicit platform and work-limit
constraints.

The runnable `raf_source::example_crate::corpus_generate` checks the actual shape
and oracle corpus writers in temporary directories without a generated-program
search. Explicit maintenance uses `shapes --write DIRECTORY` or
`oracle --write DIRECTORY`; only the latter runs the existing full oracle search.
For the committed destinations, run from the repository root:
`cargo run -p raf-source --example corpus_generate -- shapes --write fuzz/corpus/parse-source`
or `cargo run -p raf-source --example corpus_generate -- oracle --write fuzz/corpus/rustc-oracle`.

The runnable API example `raf_source::Workspace::exported_items` loads a memory-backed workspace and inspects its supported public surface.

For declared-context definition queries, attach a `DeclaredConfiguration` to
each `LockedManifest` before `Workspace::load_locked_sources`. Construct target
cfg with `DeclaredCfg::complete` and `CfgAtom` values, supply host cfg for a
build-script context, and name the exact actual target and complete features.
After loading, use `resolve_active_from_package_id_in` and follow every returned
site through its target's package ID and `LockedBinding`, including its
configuration receipt. A displayed crate name or structural provenance receipt
alone is insufficient evidence of a selected active definition.
<!-- raf:tested-by raf_source::test_crate::configured_sources::configured_aliases_resolve_only_active_real_definitions_and_paths,raf_source::test_crate::configured_sources::target_and_host_conditions_use_the_selected_compilation_role -->

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

Resolve a path as a `use` written in a module would with `ws.resolve_from(module, path)`, or in one name space with `ws.resolve_from_in(module, path, namespace)`, for a `module` such as `"demo::inner"`. The reading module is the viewer: `ws.resolve_from("demo::b", "crate::a::go")` finds `demo::a::go` when `go` is `pub` in `a`, and a private item of `a` is `ResolveError::NotFound` for `b`. A module that is not in the index is `ResolveError::UnknownModule`.

## Examples

In `cargo_version`, the requirement `>=1.2.3-alpha.2, <2` admits `1.2.3-alpha.11+001` but refuses
`1.3.0-alpha`: the latter numeric triple has no prerelease opt-in. `1.*` excludes
prereleases even beside another comparator's opt-in; `^1.*` keeps caret semantics.
`1.2.3+001` is a valid package version; `1.2.3-001` is invalid.
<!-- raf:tested-by raf_source::test_crate::locked_selection::rename_version_and_registry_select_only_the_owners_exact_outgoing_row,raf_source::test_crate::locked_catalog::malformed_unsupported_and_stale_evidence_has_a_valid_profile_neighbor -->

With `options.max_query_work = 0`, a load can succeed while a later path query
refuses with `Budget::Work`. Raising that query limit admits the same path without
raising the load-work limit. A named import whose type namespace remains
indeterminate also refuses its apparent value peer; a directly defined neighbour
still resolves. Returning private glob routes retain every distinct candidate
instead of manufacturing a unique result.
<!-- raf:tested-by raf_source::test_crate::code_paths::alias_members_preserve_definition_chains_cycles_and_original_query_grants,raf_source::test_crate::import_determinacy::a_direct_visible_definition_makes_the_neighboring_import_determinate,raf_source::test_crate::glob_causality::a_private_glob_chain_does_not_fabricate_a_unique_item -->

A binary file with Unix mode `0o751` retains both its arbitrary bytes and
executable permissions in `CapturedFile`. Unix hardlinked names retain equal original
identities. Capturing a relative or dangling link retains the exact target spelling;
capturing a link to an outside FIFO returns only the link description, while
reading that destination as a file refuses. Adding a directory entry changes its
recaptured membership without changing the directory's identity.

A file containing NUL, invalid UTF-8 and CRLF bytes reads back unchanged through
`read_bytes` at its exact byte ceiling. The text read still refuses invalid UTF-8,
and a smaller ceiling refuses before text conversion. An accepted seen identity
returns without reading that nonempty payload even when the ceiling is zero;
an empty file itself can be read with a zero-byte ceiling.

Creating `src/new/../fresh.rs` in a `MemoryTree` publishes `src/fresh.rs` under
the same normalised-key rule as its reader. Replacing its text preserves that
file's identity and the other files. Creating below an existing file or replacing
a directory refuses without changing any key or byte.

The Linux containment fixtures pair refused outside-root and FIFO reads with
valid in-root reads under the native strategy. Device-like filenames such as
`aux/con.rs` remain ordinary Linux filenames and are exercised on that basis.

A caller-owned `Tree` indexes a library and child module with binary identity
keys `[0x80]` and `[0x81]`, beside a textual-key case. The two binary keys retain
both source files even though lossy UTF-8 conversion would give them the same
spelling.
<!-- raf:tested-by raf_source::test_crate::memory_tree::a_workspace_loads_through_an_object_safe_tree_with_caller_local_state -->

The existing manifest refusal fixture pairs nested numeric overflow with valid
signed boundaries, alternate integer radices and explicit nonfinite values.
The lock fixture pairs overflowing versions with equivalent supported integer
spellings; each valid spelling selects the same lock profile.
<!-- raf:tested-by raf_source::test_crate::discovery_targets::a_manifest_that_is_not_toml_or_has_neither_table_is_a_typed_error,raf_source::test_crate::locked_catalog::malformed_unsupported_and_stale_evidence_has_a_valid_profile_neighbor -->

A renamed dependency yields its actual package, custom library target and public
defining site. A public alias chain `B = A`, `A = State<3>` resolves a supported
inherent member of nominal `State`, recording both aliases; a cycle or general
substitution refuses beside that valid neighbor. A selected integration test
reaches its package's explicitly configured ordinary library and Dev dependency,
while its production library sibling has no Dev route and the dependency library
has no unit-test-only declarations. Shared source under equal declared syntax
reuses facts; selecting incompatible test syntax refuses.
<!-- raf:tested-by raf_source::test_crate::code_paths::external_library_receipts_keep_actual_names_exports_and_definition_identity,raf_source::test_crate::code_paths::alias_members_preserve_definition_chains_cycles_and_original_query_grants,raf_source::test_crate::configured_targets::all_selected_target_kinds_keep_their_native_and_own_library_routes,raf_source::test_crate::configured_targets::shared_files_reuse_only_the_same_actual_syntax_selection -->

The executable `configured_aliases_resolve_only_active_real_definitions_and_paths`
fixture activates an optional renamed dependency through its actual feature
graph, follows an alias to the provider's selected declaration and checks the
real destination package ID. Its inactive method and conditional path neighbors
are absent. `distinct_declared_sets_change_definitions_and_keep_their_exact_receipt`
loads the same source under opposite feature selections, checks different source
sites and rejects active-definition queries against a structural-only load.
<!-- raf:tested-by raf_source::test_crate::configured_sources::configured_aliases_resolve_only_active_real_definitions_and_paths,raf_source::test_crate::configured_sources::distinct_declared_sets_change_definitions_and_keep_their_exact_receipt -->

Run `cargo run -p raf-source --example code_contexts` to load a memory-backed
edition 2015 package. Its child module's code resolves `selected` to
`demo::child::selected`, while an unqualified import head resolves to
`demo::selected`. The executable asserts both results using the same workspace.
<!-- raf:tested-by raf_source::test_crate::code_paths::code_heads_read_the_edition_2015_module_where_use_heads_read_the_crate_root -->

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

- Host captures are individual observations, not an atomic multi-file snapshot.
  A publisher must recapture and compare its complete selected source set before
  publishing. They do not materialize an execution view or transport a Runner.
  File capture preserves native permissions but does not copy ownership, ACLs or
  extended attributes. A lexical link description does not authorize following
  or materializing an outside target. Windows file and directory identities remain
  path-based, and lexical-link capture refuses. Standard validation exercises native
  Linux only.
  <!-- raf:tested-by raf_source::test_crate::disk_tree_containment::a_symlink_to_a_file_outside_is_refused_beside_an_in_root_symlink_that_is_accepted,raf_source::test_crate::disk_tree_containment::listings_are_sorted_by_name -->

- Binary reads are inherent `DiskTree` operations; `Tree` remains a UTF-8 text
  interface. They expose bytes and file identities, without executable or link
  metadata, source-tree capture, completed-root materialization or Runner
  transport. A byte read is not a multi-file snapshot guarantee.
  <!-- raf:tested-by raf_source::test_crate::disk_tree_containment::a_file_that_is_not_utf8_is_refused_beside_one_that_is,raf_source::test_crate::disk_tree_containment::a_symlink_to_a_file_outside_is_refused_beside_an_in_root_symlink_that_is_accepted -->

- Memory publication accepts first-party paths only and preserves the original
  construction fault of an invalid tree. Its directories remain inferred; it
  does not model symbolic links, permissions or filesystem durability.
  <!-- raf:tested-by raf_source::test_crate::memory_tree::the_identity_of_a_file_is_its_normalised_key_and_refuses_what_read_refuses -->

Configurations describe complete caller-declared compilation input, not an
observed Cargo invocation. Exact targets and compatible companions are explicit;
neither target kind nor physical source bytes authenticate a compilation mode.
Different packages may declare different custom predicates under equal triples.
Incompatible host/target or unit-test/dependency closures need separate loads.
Syntax inspection selects modeled declaration headers and paths; it supplies no
conditional-body transformation, macro expansion, type inference or generic alias
substitution. Associated-member checks cover only the supported plain alias and
unconditional nominal const-parameter subset. Borrowed library receipts preserve
ambiguity and public-export evidence; they do not establish that computation
delegates to that library or enforce preferred-provider policy.
<!-- raf:tested-by raf_source::test_crate::configured_sources::target_and_host_conditions_use_the_selected_compilation_role,raf_source::test_crate::configured_targets::missing_duplicate_and_feature_ineligible_targets_refuse_beside_complete_selection,raf_source::test_crate::configured_targets::shared_files_reuse_only_the_same_actual_syntax_selection,raf_source::test_crate::code_paths::code_associated_calls_prefer_inherent_methods_without_erasing_symbol_inventory,raf_source::test_crate::code_paths::alias_members_preserve_definition_chains_cycles_and_original_query_grants,raf_source::test_crate::code_paths::library_receipts_preserve_effective_name_ambiguity_and_deduplicate_one_root -->

- Backend keys must identify one file within their source authority. The `tree`
  representation shares their retained bytes, and `shared_text` charges that
  byte length and the shared header. Path containment and verified-revision
  proof belong to the selected backend.
  <!-- raf:tested-by raf_source::test_crate::memory_tree::a_workspace_loads_through_an_object_safe_tree_with_caller_local_state,raf_source::tree::sources::tests::equal_relative_paths_and_backend_keys_remain_separate_sources,raf_source::test_crate::load_budgets::index_records_at_the_item_budget_load_and_one_byte_less_is_refused -->
- Root discovery requires real directory ancestors inside its tree and refuses
  directory aliases. A disk caller can canonicalize its starting path before
  constructing the tree-relative path.
  <!-- raf:tested-by raf_source::test_crate::disk_tree_root::root_discovery_refuses_directory_aliases_and_unsafe_parent_reductions_beside_real_starts -->
- Root discovery refuses parent segments that cannot be safely reduced.
  <!-- raf:tested-by raf_source::test_crate::disk_tree_root::root_discovery_refuses_directory_aliases_and_unsafe_parent_reductions_beside_real_starts -->
- A present manifest whose target is missing is an error, rather than an
  absent manifest that permits an ancestor fallback.
  <!-- raf:tested-by raf_source::test_crate::disk_tree_root::a_dangling_manifest_is_refused_without_falling_back_to_its_workspace_ancestor -->
- Parsing is syntactic. Structural loads retain cfg alternatives and tag `test`;
  <!-- raf:tested-by raf_source::test_crate::symbol_index::bodiless_functions_and_types_of_attribute_macros_are_indexed_where_they_are_written,raf_source::test_crate::use_resolution::missing_names_dependency_items_and_macro_generated_items_are_not_found,raf_source::test_crate::configured_sources::configured_aliases_resolve_only_active_real_definitions_and_paths -->
  configured loads select the modeled declaration headers as stated above.
  Neither mode expands macros or runs type inference. In structural loads an item
  is indexed as written, so a function
  with no body or a type with no definition, which rustc accepts only as the input of
  an attribute macro or under a false `cfg`, is indexed under its written name and
  kind whatever the macro makes of it.
- For each DiskTree, every opened path must lead inside its canonical workspace root, as the disk
  <!-- raf:tested-by raf_source::test_crate::workspace_containment::a_symlinked_module_file_to_outside_is_refused_naming_the_module_beside_an_in_root_link,raf_source::test_crate::workspace_containment::a_manifest_target_path_that_leaves_the_root_is_refused_naming_the_key,raf_source::test_crate::workspace_containment::a_fifo_named_as_a_module_file_is_refused_without_blocking,raf_source::test_crate::workspace_containment::a_target_root_that_is_a_fifo_is_refused_without_blocking -->
  resolves it (canonicalised on Linux and Windows, walked beneath the root on other
  Unix systems), and every file read must be a regular file; the same rule bounds
  explicit `Cargo.toml` target paths.
- A file larger than `max_file_bytes` (default 8 MiB), or one that exceeds a
  <!-- raf:tested-by raf_source::test_crate::read_cap::memory_tree_enforces_the_cap_at_eight_mib,raf_source::test_crate::read_cap::disk_tree_enforces_the_cap_at_eight_mib,raf_source::test_crate::prescan_budgets::neighbouring_valid_shapes_just_below_each_budget_parse,raf_source::test_crate::prescan_budgets::delimiter_nesting_is_refused_at_one_hundred_twenty_nine_beside_one_hundred_twenty_eight,raf_source::test_crate::prescan_budgets::chains_that_overflowed_the_parse_stack_are_refused_by_the_recursion_budget,raf_source::test_crate::prescan_budgets::the_recursion_limit_is_half_the_parse_stack_over_the_measured_cost_of_one_unit,raf_source::test_crate::module_tree::module_nesting_beyond_the_depth_budget_is_refused_and_the_budget_itself_loads,raf_source::test_crate::module_tree::a_chain_of_files_that_each_declare_the_next_twice_is_refused_by_the_module_path_budget_opening_each_file_once_beside_a_small_diamond,raf_source::test_crate::walk_stack::a_directory_at_the_depth_limit_is_walked_and_one_below_it_is_refused,raf_source::test_crate::load_budgets::the_default_budgets_are_the_stated_limits,raf_source::test_crate::load_budgets::the_bytes_read_at_the_total_budget_load_and_one_byte_less_is_refused_naming_the_file,raf_source::test_crate::load_budgets::index_records_at_the_item_budget_load_and_one_byte_less_is_refused,raf_source::test_crate::load_budgets::the_fact_budget_counts_over_every_file_of_the_load,raf_source::test_crate::load_budgets::glob_tables_at_the_glob_budget_load_and_one_byte_less_is_refused,raf_source::test_crate::overflow_harness::every_committed_overflow_shape_is_refused_by_a_budget_and_none_aborts,raf_source::test_crate::overflow_harness::a_deliberate_overflow_aborts_and_the_runtime_names_the_parse_thread,raf_source::test_crate::overflow_harness::the_judge_rejects_a_wrong_index_a_misnamed_refusal_an_abort_and_a_stray_failure -->
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
- Queue, subscription and undecided-import graph work can grow with the places
  imports read through globs. Their storage and operations are admitted by the
  configured import-byte and load-work budgets; exceeding either refuses the load.
  <!-- raf:tested-by raf_source::settle::tests::pending_and_blocker_storage_require_the_original_grants -->
- Lookup fan-in with companions can inspect quadratically many reads. Load lookups
  are charged at their next checkpoint, and each later query owns a separate work
  limit. A typed refusal establishes a resource boundary, not an instruction-cost
  guarantee.
  <!-- raf:tested-by raf_source::settle::tests::repeated_glob_lookups_visit_reads_without_spending_the_settling_work_grant -->
- Resolution covers items the source defines. Items from dependencies are outside the
  <!-- raf:tested-by raf_source::test_crate::use_resolution::missing_names_dependency_items_and_macro_generated_items_are_not_found -->
  index and resolve to a not-found failure.
- Reachability reads paths from tokens, an approximation of use: a function stored in
  <!-- raf:tested-by raf_source::test_crate::reachability::a_method_call_a_field_a_lifetime_and_a_local_name_no_item_and_a_path_prefix_names_its_type,raf_source::test_crate::reachability::an_item_a_macro_generates_is_not_reached_and_a_macro_named_by_a_reached_item_is,raf_source::test_crate::reachability::a_module_named_by_a_path_and_then_reached_by_contains_is_opened_then,raf_source::test_crate::reachability::self_paths_and_methods_called_on_self_reach_the_members_of_that_name -->
  a field or a variable and called through it, a method called after `.` on a value
  other than `self` whose type nothing reached names, a call through a trait object to
  a type that nothing names, and an item named only inside a macro invocation at
  module level are not reached; a method called on `self` is read by its name alone,
  so it reaches every member of that name of the self type and of the trait, whichever
  the call resolves to; a local, a parameter or a field shorthand that shares its name
  with an item reaches that item; the body of a macro is read from the macro's module,
  not from where it is invoked; and a module that a path names reaches nothing below it.
- Format identity construction admits its flag and adjacency buffers and worklist spills through
  the original cumulative item-bytes account; releasing scratch does not refund
  cumulative charges, and storage failures preserve their original cause.
  <!-- raf:tested-by raf_source::meter::tests::native_storage_preserves_the_item_limit_and_cumulative_charges,raf_source::meter::tests::native_storage_capacity_failure_has_its_original_error_source,raf_source::reach::format::identity::tests::native_propagation_visits_cycles_and_duplicate_edges_once_per_active_source,raf_source::reach::format::identity::tests::native_propagation_spill_refuses_without_refunding_the_cumulative_grant -->
- Format capture inference covers literal-leading standard `format!`, not general
  <!-- raf:tested-by raf_source::test_crate::format_reach::a_cfg_alternative_sibling_alias_does_not_move_the_declared_modules_prelude_attribute -->
  macro expansion. A computed format string, an unresolved transitive re-export,
  an external glob-delivered alias, a conditional import or prelude, and a qualified
  standard path in edition 2015 lack the supported positive certificate and add no
  inferred capture. The containing contexts admitted for nested candidates are the
  standard `format`, `format_args`, `print`, `println`, `eprint`, `eprintln`, `write`,
  `writeln`, `panic`, `assert`, `assert_eq`, `assert_ne`, `debug_assert`,
  `debug_assert_eq`, `debug_assert_ne` and `vec` macros. Their own format fields,
  other than `format!`'s, are not decoded by this rule. This retains the ordinary
  module-as-code approximation for local and parameter names sharing an item name.
  <!-- raf:tested-by raf_source::test_crate::format_reach::nested_standard_expression_macros_are_transparent_but_custom_token_inputs_are_not_calls,raf_source::test_crate::format_reach::unknown_named_and_glob_imports_never_become_standard_macro_certificates,raf_source::test_crate::format_reach::one_physical_file_is_recognized_per_target_module_edition_and_root_order,raf_source::test_crate::format_reach::every_admitted_enclosing_standard_macro_accepts_nested_format_expressions -->
- `Self` names members and never the self type itself: a member that names its type
  only as `Self` does not reach the type.
  <!-- raf:tested-by raf_source::test_crate::reachability::self_paths_and_methods_called_on_self_reach_the_members_of_that_name,raf_source::test_crate::reachability::self_in_a_trait_and_in_its_impl_blocks_reaches_the_members_of_the_trait_and_of_the_block,raf_source::test_crate::reachability::a_variant_through_self_names_its_enum_as_a_path_through_the_enum_does -->
- Qualified `<Self as Tr>::name` and `<S>::name` suffixes add no direct member
  reference edge; types named inside the qualification can still reach members
  through containment.
  <!-- raf:tested-by raf_source::test_crate::reachability::qualified_self_suffixes_are_not_member_edges_beside_an_unqualified_self_member -->
- An impl block inside a function body is read as tokens of the function, its
  <!-- raf:tested-by raf_source::test_crate::reachability::nested_impl_self_is_read_in_the_enclosing_members_context -->
  `Self` as that of the block that holds the function.
- In edition 2015 a `use` declaration inside a function body is read from the module
  <!-- raf:tested-by raf_source::test_crate::reachability::an_edition_2015_body_import_is_read_from_its_module_beside_an_explicit_crate_path -->
  that holds the function, where rustc reads it from the crate root, so what it brings
  in is not reached.
- An impl block inside `const _: () = { .. };` or any other block expression is not in
  <!-- raf:tested-by raf_source::test_crate::reachability::impls_inside_const_and_function_blocks_are_tokens_beside_an_indexed_top_level_impl -->
  `impls()`: in an item it is read as tokens of the item, and a `const _` at module
  level is no item's, so a block there is never reached.
- An item that syn returns as tokens is indexed only in the three shapes the Contract
  <!-- raf:tested-by raf_source::test_crate::symbol_index::bodiless_functions_and_types_of_attribute_macros_are_indexed_where_they_are_written,raf_source::test_crate::symbol_index::a_bodiless_const_or_another_verbatim_item_is_still_a_typed_error_naming_file_and_line,raf_source::test_crate::use_resolution::a_use_group_with_leading_colon_members_binds_as_the_separate_uses_do -->
  names: a `use` group with a member that starts with `::`, a function with no body and
  a type with no definition. Any other, such as `const C: u8;` written as the input of an
  attribute macro, fails the whole load with `UnsupportedItem`, naming the file and line.
- An import whose namespace proof remains indeterminate cannot publish an apparently settled peer through a named copy, glob, exported surface or reachability edge. A cyclic or indeterminate-import disagreement with the compiler fails the differential oracle; it is not a tolerated class.
  <!-- raf:tested-by raf_source::test_crate::rustc_oracle::every_corpus_program_shows_the_divergence_it_records_and_no_other,raf_source::test_crate::rustc_oracle::the_index_agrees_with_rustc_on_generated_programs_up_to_the_known_classes -->
- For some glob ambiguities rustc's own answer depends on the order of the declarations, the first binding of the name that reaches a module deciding; the index gives one fixed answer, the same in every order, which is the ambiguity for some shapes and the item for others, and which rustc gives in some of its orders and not in the others.
  <!-- raf:tested-by raf_source::test_crate::rustc_oracle::every_corpus_program_shows_the_divergence_it_records_and_no_other,raf_source::test_crate::rustc_oracle::a_probe_rustc_answers_by_the_order_is_not_order_when_the_index_does_too,raf_source::test_crate::rustc_oracle::an_index_answer_that_changes_with_the_order_is_reported_as_index_order -->
- Companion-bearing paths can require repeated carrier searches and superlinear lookup work. Storage and counted-work limits refuse excessive resource use; they do not establish the separate instruction-cost and growth contract.
  <!-- raf:tested-by raf_source::glob_scope::work_budget::each_shape_loads_at_its_measured_work_and_refuses_one_step_less,raf_source::glob_scope::work_budget::doubling_a_shape_keeps_actual_work_within_its_growth_bound,raf_source::glob_scope::work_budget::a_settle_heavy_shape_is_refused_at_the_charge_that_passes_a_small_limit -->
- `workspace.members` and `workspace.exclude` hold at most 4096 entries each.
  <!-- raf:tested-by raf_source::test_crate::root::member_and_exclude_lists_accept_4096_entries_and_refuse_one_more -->
- The text of an error shows input-derived text with control characters escaped by
  <!-- raf:tested-by raf_source::error::tests::every_user_derived_field_of_every_display_is_escaped,raf_source::resolve::tests::every_user_derived_field_of_every_resolve_error_is_escaped,raf_source::path::display_tests::control_characters_in_a_refused_path_are_escaped -->
  `char::escape_debug`, which also doubles a backslash, so it is safe to print to a
  terminal; the fields of the error keep the raw text.
- The index is built once per `Workspace::load`; callers reload to see changed files.
  <!-- raf:tested-by raf_source::test_crate::index_disk::a_reload_observes_changed_source_while_the_previous_workspace_keeps_its_snapshot -->
- Loading is serial; requiring `Tree: Sync` for a parallel load would be a breaking
  change to `Workspace::load`.
  <!-- raf:tested-by raf_source::test_crate::memory_tree::a_workspace_loads_through_an_object_safe_tree_with_caller_local_state -->
- `raf_source::syn` is re-exported so consumers use the syn version that produced the
  <!-- raf:tested-by raf_source::test_crate::this_repository::the_reexported_syn_types_are_the_types_received_by_the_parse_visitor -->
  facts; a syn major bump is a breaking change of raf-source.
- Output order is sorted by path, so two loads of the same tree give equal results.
  <!-- raf:tested-by raf_source::test_crate::symbol_index::two_loads_are_equal_and_items_and_targets_come_in_strictly_increasing_order,raf_source::test_crate::exported_items::the_surface_is_the_same_in_every_load_and_in_every_order_of_the_declarations -->
- The public surface of a structural load is the visibility rustc gives the items of a crate, read
  <!-- raf:tested-by raf_source::test_crate::exported_items::a_pub_function_is_exported_beside_private_and_restricted_ones_which_are_not,raf_source::test_crate::exported_items::doc_hidden_items_are_left_out_by_default_and_flagged_when_included,raf_source::test_crate::exported_items::test_code_is_not_part_of_the_surface_beside_the_shipped_items,raf_source::test_crate::exported_items::an_exported_macro_is_exported_and_a_local_or_generated_one_is_not,raf_source::test_crate::exported_items::an_ambiguous_a_cyclic_or_an_unresolved_public_name_fails_beside_a_neighbour_that_lists,raf_source::test_crate::exported_items::a_carried_outside_re_export_is_kept_out_by_a_restricted_glob_or_a_name_on_the_way,raf_source::test_crate::exported_items::searches_for_names_that_several_binders_keep_from_a_long_route_are_refused_by_the_item_budget,raf_source::test_crate::exported_items::only_a_route_that_carries_a_name_out_decides_whether_it_is_hidden_for_items_and_re_exports -->
  syntactically: no `cfg` other than the `test` tag is evaluated, so an item under
  any other `cfg` is listed whether or not a build has it, and nothing a macro
  generates is listed. `#[doc(hidden)]` and `#[macro_export]` are read as the index
  reads them, unconditional and, for `doc`, with `hidden` alone; one that a
  `cfg_attr` applies is not read. Of the two visibility rules of the Contract, the first never lists an item through the re-export it caps, since the surface requires every `use` declaration on the way to be `pub`, and the second fails a listing that meets it with `Ambiguous`, as `resolve` does. A name that two globs of a listed module bring in as two items fails the whole listing with `Ambiguous`, also where rustc accepts
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
- Standard checks execute only on native Linux. The proofs that need a Windows
  host, the final path of a handle and a `DiskTree`
  <!-- raf:tested-by raf_source::tree::disk::tests::the_handle_path_verifier_refuses_a_path_outside_the_root_beside_one_inside,raf_source::tree::disk::tests::linux_verifier_reads_the_real_descriptor_of_an_open_file,raf_source::tree::disk::tests::reserved_device_names_and_trailing_dots_or_spaces_are_ambiguous_on_windows_beside_plain_names,raf_source::test_crate::workspace_containment::a_tree_with_windows_device_names_loads_off_windows -->
  refusing a reserved device name or a name ending in a dot or a space, remain
  unverified by standard checks. Linux runs the shared handle-path verifier, the
  Linux handle verifier on real descriptors and a tree holding those ordinary
  Linux names. The Windows-name predicate is tested only on Windows and is not
  executed by the standard checks.

## Explanation

The private `cargo_version` adapter supplies Cargo-specific grammar and matching
policy that the pinned native libraries do not supply. It uses native byte-class
scanning and inline storage, retains borrowed text and introduces no general-purpose version API or
separate dependency. The same implementation validates manifest requirements,
package versions and lock references, and matches outgoing lock rows.
<!-- raf:tested-by raf_source::test_crate::dependency_declarations::malformed_declarations_refuse_with_valid_neighbors,raf_source::test_crate::locked_selection::rename_version_and_registry_select_only_the_owners_exact_outgoing_row -->

Selecting fields from the native arena avoids a second owned value tree.
Whole-model scalar validation preserves numeric refusals even for fields discovery
ignores. Native integer semantics and exact authored infinity tokens differ from
XSD numeric lexical rules. Package, target, dependency and source identity remain Source
policy over the parsed values.

The locked-source path separates declaration observation, outgoing lock selection,
committed-byte admission and actual discovered package identity. This prevents
the displayed crate name or a caller-supplied authority from substituting for a
checked provider definition. Configuration is another explicit input: applying it
before fact extraction avoids publishing an unfiltered alternative as active.
The selected scope is declaration headers; raw bodies and fields remain syntax,
so this receipt does not certify an expanded or type-checked compilation.

### Memory and native buffer ownership
<!-- raf:tested-by raf_source::test_crate::load_budgets::retained_import_reads_and_blocker_cycles_obey_exact_storage_and_work_limits,raf_source::meter::tests::native_graph_storage_preserves_overlapping_grants_and_typed_refusals,raf_source::meter::tests::native_emission_spill_preserves_cumulative_item_admission -->

Source text, index records, facts and import storage have separate configured
ceilings. Import queues, subscriptions and graph scratch keep their original
native storage grants. Growing a buffer admits both its existing backing and its
replacement before mutation; destroying released scratch ends its live grant.
Item-byte charges remain cumulative for the load.

The configured retained ceilings sum to 1.5 GiB at their defaults. The separate
one-file parser ceiling is `max_file_tokens × PARSE_BYTES_PER_TOKEN`, about
3.1 GiB at the default token count. Those configured components sum to about
4.6 GiB; they are allocation-admission bounds, not measurements of resident memory
or promises that every source shape fits within them. A caller lowers each limit
to fit its host, and an over-limit load returns its original typed refusal.

### Native instruction evidence

Glob propagation shares visibility-class views and passes independent arrivals as deltas. Companion-bearing names, blocked namespaces and returning private routes retain exact reconstruction and carrier proofs. Lookup and propagation counters describe the operations actually visited; they are distinct from native CPU instruction counts. Instruction-cost claims require complete native counters over identical inputs. A successful resource refusal or counted-work test supplies no numerical CPU-cost guarantee.

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

Manifest and catalog refusal cases exercise `cargo_version` with malformed versions, overflowing
components, qualifier grammar and the comparator ceiling beside valid neighbors.
Outgoing-row cases cover all operators, partial and wildcard forms, prerelease
ordering and opt-in, build identity and maximum components. Resource cases require
a typed work refusal before a long qualifier comparison and a roomy valid neighbor.
<!-- raf:tested-by raf_source::test_crate::dependency_declarations::malformed_declarations_refuse_with_valid_neighbors,raf_source::test_crate::locked_catalog::malformed_unsupported_and_stale_evidence_has_a_valid_profile_neighbor,raf_source::test_crate::locked_selection::rename_version_and_registry_select_only_the_owners_exact_outgoing_row,raf_source::test_crate::locked_selection::selection_honors_work_and_retained_catalog_item_admission -->

### Import propagation and native storage
<!-- raf:tested-by raf_source::exports::tests::native_surface_storage_preserves_physical_failure_and_budget_ownership,raf_source::glob_scope::delivery::maximal_public_replacements_avoid_repeat_proofs_while_narrower_ones_are_widened,raf_source::glob_scope::delivery::native_delivery_growth_admits_old_and_new_spill_buffers_together,raf_source::glob_scope::delivery::native_delivery_spill_is_lifo_and_failed_push_releases_only_the_incoming_delta,raf_source::glob_scope::delivery::one_returning_carrier_retains_multiple_independently_selected_identities,raf_source::glob_scope::delivery::recorded_name_hash_storage_loads_at_exact_item_and_glob_grants_and_refuses_one_less,raf_source::glob_scope::delivery::recorded_name_rows_survive_real_reconstruction_with_new_equal_interned_names,raf_source::glob_scope::delivery::widest_routes_merge_firm_evidence_in_place_and_refuse_before_mutation,raf_source::glob_scope::equivalence::an_answer_that_differs_between_the_two_modes_is_reported_with_its_load_and_both_answers,raf_source::glob_scope::equivalence::the_settling_gives_the_answers_of_the_reference_mode_to_every_generated_and_corpus_program_in_three_orders,raf_source::glob_scope::incremental::a_covert_read_is_listed_before_its_names_arrive_so_an_item_its_view_gains_while_they_arrive_reaches_the_module,raf_source::glob_scope::incremental::a_covert_reader_is_woken_in_the_turn_an_active_read_of_it_would_have_so_the_imports_that_wait_on_the_name_are_queued_again_in_that_order,raf_source::glob_scope::incremental::a_covert_reader_takes_a_gain_in_the_turn_an_active_read_of_it_would_have_so_the_companions_of_a_name_are_kept_as_its_drops_allow,raf_source::glob_scope::incremental::a_module_whose_covert_read_turns_active_is_no_longer_looked_at_as_a_covert_reader_when_the_view_grows,raf_source::glob_scope::incremental::a_view_made_when_its_module_has_left_the_open_readers_makes_it_open_again_until_it_learns_what_it_cannot_access,raf_source::glob_scope::incremental::an_item_its_class_cannot_access_that_arrived_under_a_name_its_view_did_not_hold_makes_the_first_item_it_can_access_ambiguous,raf_source::glob_scope::incremental::an_item_that_a_view_read_through_a_glob_its_class_cannot_access_holds_makes_the_first_item_it_can_access_under_the_name_ambiguous,raf_source::glob_scope::lookups::a_query_after_the_load_is_charged_its_own_lookup_steps_against_the_limit_of_one_query,raf_source::glob_scope::lookups::every_glob_read_a_lookup_of_the_load_looks_at_is_charged_once,raf_source::glob_scope::lookups::every_limit_below_the_steps_of_a_query_refuses_it_and_the_refusal_comes_before_any_answer,raf_source::glob_scope::lookups::every_limit_below_the_work_of_the_lookups_shape_is_refused_by_the_work_budget_and_by_nothing_else,raf_source::glob_scope::lookups::every_query_counts_the_lookups_of_its_walks_and_of_the_views_it_looks_at_to_find_them,raf_source::glob_scope::lookups::every_way_of_resolving_a_path_is_a_query_with_its_own_limit,raf_source::glob_scope::lookups::queries_on_other_threads_each_count_from_zero,raf_source::glob_scope::lookups::the_lookups_of_many_named_imports_through_many_globs_are_charged_and_refused_by_a_small_limit_at_the_first_checkpoint_past_it,raf_source::glob_scope::lookups::the_public_surface_is_one_query_whose_lookups_and_those_of_the_paths_it_resolves_share_its_limit,raf_source::glob_scope::monotone::an_entry_held_as_a_companion_is_widened_as_it_is_added_again_and_made_plain_only_by_a_plain_entry,raf_source::glob_scope::monotone::an_entry_that_makes_a_plain_item_a_companion_again_is_refused,raf_source::glob_scope::monotone::an_entry_that_narrows_the_level_an_item_is_held_at_is_refused,raf_source::glob_scope::ranked::tests::a_complete_definition_and_relay_rank_is_already_its_full_closure,raf_source::glob_scope::ranked::tests::a_mixed_relay_and_a_removed_anchor_match_actual_reconstruction,raf_source::glob_scope::ranked::tests::absent_cells_and_ungrounded_cyclic_residents_cannot_pass_closure_or_rank,raf_source::glob_scope::ranked::tests::an_exact_empty_namespace_proves_bottom_without_any_transfer,raf_source::glob_scope::ranked::tests::an_impossible_aggregate_reservation_is_a_typed_layout_refusal_before_allocation,raf_source::glob_scope::ranked::tests::an_incomplete_successful_rank_still_checks_its_later_complete_input,raf_source::glob_scope::ranked::tests::complete_public_peer_discovery_omits_only_its_redundant_postpublication_pass,raf_source::glob_scope::ranked::tests::discovery_independence_scans_preserve_the_original_work_refusal,raf_source::glob_scope::ranked::tests::homogeneous_returning_carriers_share_an_actual_holder_retrieval,raf_source::glob_scope::ranked::tests::interior_levels_named_imports_and_glob_prefixes_keep_ordinary_reconstruction,raf_source::glob_scope::ranked::tests::named_extern_alias_and_qualified_glob_neighbors_do_not_certify_independence,raf_source::glob_scope::ranked::tests::proof_scratch_and_work_refuse_one_less_than_the_actual_original_grant,raf_source::glob_scope::ranked::tests::public_peer_rows_are_rooted_without_a_second_table_construction,raf_source::glob_scope::ranked::tests::roots_outside_resident_targets_still_require_the_second_discovery,raf_source::glob_scope::settling::a_companion_fan_in_keeps_each_operation_population_linear_without_list_copies,raf_source::glob_scope::settling::a_companion_hub_bounds_delivery_and_carrier_graph_growth_together,raf_source::glob_scope::settling::a_companion_hub_keeps_local_definitions_and_reads_without_copying_candidate_lists,raf_source::glob_scope::settling::a_reading_again_looks_at_the_reads_whose_view_holds_the_name_whatever_the_reads_of_the_module,raf_source::glob_scope::settling::named_import_lookup_routes_stay_linear_and_read_work_stays_quadratic,raf_source::glob_scope::settling::the_settling_of_a_chain_of_globs_counts_its_pinned_polynomials_at_n_2n_and_3n,raf_source::glob_scope::settling::the_settling_of_a_fan_in_of_private_globs_counts_its_pinned_polynomials_at_n_2n_and_3n,raf_source::glob_scope::settling::the_settling_of_a_fan_in_of_pub_globs_counts_its_pinned_polynomials_at_n_2n_and_3n,raf_source::glob_scope::settling::the_settling_of_a_hub_counts_its_pinned_polynomials_at_n_2n_and_3n,raf_source::glob_scope::settling::the_settling_of_a_module_of_many_items_globbed_from_many_modules_counts_its_pinned_polynomials_at_n_2n_and_3n,raf_source::glob_scope::settling::the_settling_of_a_module_whose_names_many_views_hold_counts_its_pinned_polynomials_at_n_2n_and_3n,raf_source::glob_scope::settling::the_settling_of_the_prelude_pattern_counts_its_pinned_polynomials_at_n_2n_and_3n,raf_source::glob_scope::table_bytes::each_covert_reader_open_reader_and_view_read_through_a_glob_a_class_cannot_access_is_charged_its_entry_and_the_first_room_of_its_table,raf_source::glob_scope::table_bytes::each_name_a_view_dropped_and_each_name_a_reading_again_set_is_charged_its_entry_and_the_first_room_of_its_record,raf_source::glob_scope::table_bytes::each_name_a_view_takes_is_charged_its_slot_among_the_holders_of_the_name_and_the_first_companion_its_entry,raf_source::glob_scope::table_bytes::each_read_of_a_view_is_charged_its_place_among_the_reads_of_its_module_of_that_view,raf_source::glob_scope::table_bytes::the_tables_of_the_views_that_hold_each_name_and_of_the_names_with_a_companion_are_charged_their_first_room,raf_source::glob_scope::tests::holder_visibility_uses_ancestry_instead_of_sibling_depth,raf_source::glob_scope::tests::independent_same_item_supply_preserves_the_carriers_ambiguity,raf_source::glob_scope::tests::named_supplier_levels_stay_with_the_declaration_that_provides_them,raf_source::glob_scope::tests::native_resumed_growth_admits_overlapping_buffers_before_mutation,raf_source::glob_scope::work_budget::a_new_view_charges_raw_reads_and_unique_source_names_before_gathering,raf_source::glob_scope::work_budget::actual_work_from_100_to_800_stays_within_each_shapes_doubling_bound,raf_source::glob_scope::work_budget::an_export_read_charges_each_visited_read_and_holder_before_returning_its_answer,raf_source::glob_scope::work_budget::holder_scan_sources_beyond_the_admitted_growth_sizes_are_refused,raf_source::impls::tests::alternate_aliases_spill_the_nominal_stack_under_its_original_grant,raf_source::impls::tests::missing_impl_head_admits_pending_glob_spill_and_keeps_the_missing_neighbor,raf_source::meter::tests::native_emission_spill_preserves_cumulative_item_admission,raf_source::meter::tests::native_graph_storage_preserves_overlapping_grants_and_typed_refusals,raf_source::meter::tests::retained_payload_moves_its_original_grant_without_readmission,raf_source::meter::tests::transient_reservations_refuse_atomically_and_release_on_every_exit,raf_source::settle::replay::genuine_paths_are_fixed_but_glob_prefixes_and_named_shadows_require_replay,raf_source::settle::replay::path_visibility_and_unmatched_or_open_resident_targets_keep_replay,raf_source::settle::tests::a_glob_prefix_bound_in_type_still_waits_for_its_named_import,raf_source::settle::tests::excluding_ones_own_named_binding_reserves_the_glob_fallback_too,raf_source::settle::tests::native_component_grouping_refuses_one_below_its_live_storage_bound,raf_source::settle::tests::pending_counts_duplicate_attempts_and_refuses_before_removing_a_queued_import,raf_source::settle::tests::retained_import_batches_reserve_previous_results_and_new_reads_together,raf_source::work::tests::the_table_holds_every_kind_once_at_the_place_of_its_counter -->

Native import fixtures cover monotone arrivals, independent returning routes, original allocation grants, queue and graph admission, exact operation populations and reference equivalence. Refusal fixtures retain the physical failure and exercise exact limits beside one-unit-short limits.

### Private routes and namespace determinacy
<!-- raf:tested-by raf_source::test_crate::glob_causality::a_cycle_with_an_independent_carrier_route_keeps_its_ambiguity,raf_source::test_crate::glob_causality::a_final_glob_fallback_waits_for_an_outside_named_cycle,raf_source::test_crate::glob_causality::a_forwarded_private_import_preserves_public_peer_namespace_exports,raf_source::test_crate::glob_causality::a_grounded_free_namespace_releases_only_proven_empty_namespaces,raf_source::test_crate::glob_causality::a_private_alias_retains_a_returning_public_type,raf_source::test_crate::glob_causality::a_private_cut_does_not_erase_a_visible_ambiguity,raf_source::test_crate::glob_causality::a_private_cut_does_not_hide_a_later_visible_candidate,raf_source::test_crate::glob_causality::a_private_cut_does_not_waive_an_unclosed_sibling_branch,raf_source::test_crate::glob_causality::a_private_cut_finalizes_a_module_prefix_without_waiving_its_peer_cycle,raf_source::test_crate::glob_causality::a_private_cut_requires_a_closed_glob_module_prefix,raf_source::test_crate::glob_causality::a_private_forwarded_binding_hides_the_same_glob_item,raf_source::test_crate::glob_causality::a_private_glob_chain_does_not_fabricate_a_unique_item,raf_source::test_crate::glob_causality::a_private_glob_cut_retains_the_named_shadow_of_a_returning_namespace,raf_source::test_crate::glob_causality::a_private_independent_supporter_and_a_returning_cycle_keep_the_hidden_route,raf_source::test_crate::glob_causality::a_returning_export_ignores_only_independent_routes_below_its_visibility,raf_source::test_crate::glob_causality::a_settled_returning_named_peer_does_not_invent_a_namespace_export,raf_source::test_crate::glob_causality::a_value_import_does_not_erase_the_type_ambiguity_waiting_behind_it,raf_source::test_crate::glob_causality::a_visible_route_waiting_beside_a_hidden_arrival_keeps_the_ambiguity,raf_source::test_crate::glob_causality::an_actual_inaccessible_cut_needs_a_closed_local_candidate,raf_source::test_crate::glob_causality::an_ambiguous_original_import_is_refused_before_oracle_sanitization,raf_source::test_crate::glob_causality::an_independent_glob_does_not_shadow_an_inherited_ambiguity,raf_source::test_crate::glob_causality::direct_private_sources_and_missing_public_peers_do_not_export_glob_values,raf_source::test_crate::glob_causality::matching_private_imports_preserve_unaliased_glob_families,raf_source::test_crate::glob_causality::named_imports_preserve_companions_instead_of_promoting_them_to_independent_routes,raf_source::test_crate::glob_causality::private_forwarding_preserves_distinct_glob_ambiguity,raf_source::test_crate::glob_causality::rustc_refuses_the_named_import_of_the_ambiguous_type,raf_source::test_crate::import_determinacy::a_direct_visible_definition_makes_the_neighboring_import_determinate,raf_source::test_crate::import_determinacy::a_fixed_module_prefix_decides_absence_before_an_unrelated_glob_widens_a_name,raf_source::test_crate::import_determinacy::a_late_bound_peer_cannot_escape_through_named_and_glob_copies_or_exports,raf_source::test_crate::import_determinacy::a_late_namespace_refuses_the_whole_import_including_its_bound_peer,raf_source::test_crate::import_determinacy::a_named_import_reading_back_through_its_module_is_not_a_settled_expression,raf_source::test_crate::import_determinacy::a_private_type_shadow_does_not_poison_its_independent_value_glob,raf_source::test_crate::import_determinacy::late_visibility_refusals_and_their_neighbors_are_stable_across_declaration_orders,raf_source::test_crate::load_budgets::nested_definition_emission_spills_with_only_the_item_budget,raf_source::test_crate::load_budgets::retained_import_reads_and_blocker_cycles_obey_exact_storage_and_work_limits,raf_source::test_crate::load_budgets::wide_discovery_and_member_frontiers_spill_under_the_item_budget,raf_source::test_crate::use_resolution::a_companion_carried_through_an_independent_glob_chain_remains_ambiguous_in_every_order,raf_source::test_crate::use_resolution::a_crate_visible_independent_route_keeps_another_globs_ambiguity_in_every_order,raf_source::test_crate::use_resolution::a_hidden_companion_arriving_beside_an_independent_route_remains_ambiguous_in_every_order,raf_source::test_crate::use_resolution::a_private_independent_supporter_does_not_erase_companions_through_a_returning_cycle_in_every_order,raf_source::test_crate::use_resolution::a_returning_glob_cycle_beside_an_independent_named_route_keeps_ambiguity_in_every_order,raf_source::test_crate::use_resolution::an_independent_named_module_route_does_not_erase_an_ambiguous_prefix_in_every_order,raf_source::test_crate::use_resolution::an_independent_named_route_does_not_erase_the_companion_of_another_glob_in_every_order -->

Private-arrival and namespace fixtures retain hidden competitors, closed-prefix proofs and distinct targets across declaration orders. Indeterminate imports refuse their apparently settled peers through copies, exported surfaces and reachability; adjacent independently defined paths remain valid.

### Query resource ownership
<!-- raf:tested-by raf_source::reach::tests::code_prefix_reads_share_one_query_account_and_refuse_one_less,raf_source::resolve::determinacy::tests::native_query_layout_failure_is_reported_as_storage_without_allocating,raf_source::resolve::determinacy::tests::selected_path_proof_spills_on_real_routes_and_releases_its_scratch,raf_source::resolve::determinacy::tests::selected_query_and_surface_proofs_admit_exact_work_and_release_all_scratch,raf_source::resolve::determinacy::tests::selected_routes_admit_the_first_native_spill_exactly_and_refuse_before_mutation -->

Selected-path and reachability queries reserve native proof scratch under their original grants, release it on every exit and report budget or physical storage failures without returning an absent edge.


A native filesystem case mutates file permissions, directory membership
and a lexical link between observations. Each refuses as a changed capture beside
stable successful captures, using one deterministic test seam without clock waits.
<!-- raf:tested-by raf_source::tree::disk::tests::swap::captures_refuse_changed_observations_beside_stable_reads -->

Native containment cases preserve executable binary captures, exact
ceilings, hardlink relationships and lexical link identities. Relative, dangling
and outside-pointing link descriptions have valid neighbours and refused outside
parent or non-link cases. Directory recapture detects added names. Authority and
directory-swap cases exercise the capture paths through the native Linux backend.
<!-- raf:tested-by raf_source::test_crate::disk_tree_containment::a_file_that_is_not_utf8_is_refused_beside_one_that_is,raf_source::test_crate::disk_tree_containment::the_identity_of_a_file_is_the_device_and_inode_of_its_verified_handle,raf_source::test_crate::disk_tree_containment::a_file_that_is_missing_is_not_found_and_one_that_leaves_the_root_is_outside_root,raf_source::test_crate::disk_tree_containment::a_symlink_to_a_file_outside_is_refused_beside_an_in_root_symlink_that_is_accepted,raf_source::test_crate::disk_tree_containment::listings_are_sorted_by_name,raf_source::tree::disk::tests::source_dispatch_preserves_backend_aliases_and_rejects_foreign_direct_paths,raf_source::tree::disk::tests::swap::a_file_whose_directory_is_moved_out_after_verification_is_refused_after_the_read,raf_source::tree::disk::tests::swap::a_directory_swapped_after_verification_before_the_entries_are_read_is_refused -->

The existing native containment cases compare exact binary payloads and verified
identities, exercise exact and insufficient byte ceilings and seen reads, retain
invalid-text refusals, and check outside-root, directory and foreign-authority
failures beside valid reads. Existing text callers execute the same byte path.
<!-- raf:tested-by raf_source::test_crate::disk_tree_containment::a_file_that_is_not_utf8_is_refused_beside_one_that_is,raf_source::test_crate::disk_tree_containment::a_directory_named_as_a_file_is_refused,raf_source::tree::disk::tests::source_dispatch_preserves_backend_aliases_and_rejects_foreign_direct_paths -->

The existing normalised-identity case exercises creation, replacement, stable
identity, unchanged neighbours, duplicate and missing destinations, directory
prefix conflicts, foreign authorities and poisoned trees without adding a
separate reader or test target.
<!-- raf:tested-by raf_source::test_crate::memory_tree::the_identity_of_a_file_is_its_normalised_key_and_refuses_what_read_refuses -->

The `tree` implementation's caller-owned fixture loads both files under textual and binary keys.
Memory identity and seen-read fixtures retain their canonical-key, refusal and
exact read-limit assertions; source dispatch keeps equal backend keys separate
under their selected authorities.
<!-- raf:tested-by raf_source::test_crate::memory_tree::a_workspace_loads_through_an_object_safe_tree_with_caller_local_state,raf_source::test_crate::memory_tree::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen,raf_source::tree::sources::tests::equal_relative_paths_and_backend_keys_remain_separate_sources -->

Library fixtures check actual Rust names, exports, hidden public definitions,
private refusal, explicit declaration roots, competing preludes and deduplication
of one root. Candidate-root scratch and its composed query use the original
native byte/work account, with sufficient default-grant success beside typed
refusal when the original Work or ItemBytes allowance is exhausted.
<!-- raf:tested-by raf_source::test_crate::code_paths::external_library_query_scratch_uses_the_original_native_grants -->
Associated-member fixtures pair supported inherent precedence and
literal-const alias chains with inventory ambiguity, cycles, unsupported
substitution and original-grant exhaustion. Scoped locked-source fixtures pair
complete local associations with omitted/extra associations and preserve native
resource causes. Configured-target fixtures check all target kinds, ordinary
own-library companions, Dev isolation, required features, target-specific
conditional/optional activation, compatible physical reuse and incompatible
refusal. Original Work and ItemBytes limits have exact-bound successes and
one-unit-short typed failures; configured host/target fixtures remain bound.
Configured-syntax queries pair actual-target inspection with absent/unreached
context and typed original-grant exhaustion controls.
<!-- raf:tested-by raf_source::test_crate::configured_targets::configured_syntax_queries_select_the_actual_target_and_preserve_budget_causes -->
Method applicability fixtures retain a nongeneric inherent method beside a
generic sibling in either order and refuse the selected method's own generic
parameters or bounds beside a same-name trait member.
<!-- raf:tested-by raf_source::test_crate::code_paths::inherent_method_applicability_is_local_to_the_selected_signature,raf_source::test_crate::code_paths::external_library_receipts_keep_actual_names_exports_and_definition_identity,raf_source::test_crate::code_paths::library_receipts_preserve_effective_name_ambiguity_and_deduplicate_one_root,raf_source::test_crate::code_paths::code_associated_calls_prefer_inherent_methods_without_erasing_symbol_inventory,raf_source::test_crate::code_paths::alias_members_preserve_definition_chains_cycles_and_original_query_grants,raf_source::test_crate::code_paths::complete_module_paths_return_actual_type_sites_without_becoming_values,raf_source::test_crate::locked_sources::explicit_first_party_roots_require_every_discovered_manifest_association,raf_source::test_crate::locked_sources::all_snapshots_catalog_and_walker_use_one_original_resource_grant,raf_source::meter::tests::native_recovery_preserves_nested_provenance_and_restores_its_own_refusal,raf_source::test_crate::configured_targets::all_selected_target_kinds_keep_their_native_and_own_library_routes,raf_source::test_crate::configured_targets::missing_duplicate_and_feature_ineligible_targets_refuse_beside_complete_selection,raf_source::test_crate::configured_targets::shared_files_reuse_only_the_same_actual_syntax_selection,raf_source::test_crate::configured_targets::development_edges_require_a_real_development_or_declared_test_context,raf_source::test_crate::configured_targets::conditional_and_optional_edges_keep_each_targets_actual_activation,raf_source::test_crate::configured_targets::retained_target_lists_and_configuration_queries_spend_the_original_grants,raf_source::test_crate::configured_sources::target_and_host_conditions_use_the_selected_compilation_role -->

Locked-source fixtures distinguish duplicate package names, versions and source
roots, check actual alias/reexport destination identities, and refuse forged
manifest labels, missing commits and substituted blobs beside valid neighbors.
Configured fixtures validate the complete feature graph, target/host roles,
inactive requested edges and definitions, raw cfg identifiers and cfg_attr grammar.
Their public loads and active queries assert actual selected sites and original
combined resource grants rather than comparing only displayed path strings.

The selectable registry-scan integration target imports the actual example's
eight assertions without duplicating their bodies. Corpus writer tests write
only owned temporary directories, compare the complete shape output and preserve
serialized oracle program and probe identities. These tests do not run the full
maintenance search or rewrite the committed corpus.
<!-- raf:tested-by raf_source::test_crate::corpus_replay::write_corpus,raf_source::test_crate::rustc_oracle::write_corpus -->

Shared physical source keeps keyword runs separately for each package edition.
Separate and combined root walks agree in both root orders and with the package
editions swapped; raw names are the valid neighbour of a reserved keyword.
Repeated reads of one file and edition neither lex nor charge it again; another
edition is counted and charged, with an exact byte-limit success beside its
one-byte-short refusal.

- Root fixtures resolve explicit sibling workspaces whose members use parent
  segments, while refusing the same package when the target workspace does not
  list it. Disk discovery also asserts that the selected root is the sibling
  workspace directory.
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
  declaration, and is refused by the glob budget one byte below what it is charged.
  The glob table work of a module globbed through a chain of two globs is ten steps:
  it loads at a limit of ten and is refused at nine, naming the line of the glob whose
  table passes it, and a load without a glob loads at a limit of none. `usage()`
  reports the spent bytes, the most tokens, the highest recursion value, the deepest
  directory and the work, which is none for a load without a glob, and an item record
  is at most 256 bytes.
- Glob table work fixtures, counted: the steps of a chain of n globs, a fan-in of n
  private globs, the prelude pattern of n children and a hub of n siblings are 2n,
  4n^2 + 6n, n(15n + 17)/2 and 2n^3 + 4n^2 + 4n at n of 8, 16 and 32. Doubling n from 32
  to 64 multiplies the count of the hub by more than four and those of the prelude and
  the fan-in by more than two, and the count of the chain grows and at most doubles. The fan-in,
  the prelude and the hub of 32, each at a limit a tenth of the way through its
  charges, are refused by the work budget naming the line of a glob, at the charge
  that passes the limit: the charges made are those of the whole load up to that one,
  and the count at the refusal is the limit and that charge, below the count of the
  whole load. A meter accepts work up to its limit and refuses the charge that passes
  it, holding the limit and that charge; it never releases work, and each meter
  counts from zero.
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
- `resolve_from` from every target (`target_resolution`): a helper resolves from an integration test root and from a module below it, the library from a test, an example and a binary and not from the build script, `crate` is the reading target's root; from a nested module of a binary in `src/bin/<name>/` and of `src/main.rs`; from an example, a bench and the build script; a support module that two test targets include resolves in each target alone, one site each, related by `same_source`; a module path that no target holds is `UnknownModule` for each kind of target beside the neighbour that one holds; a library module at the crate path of a test target is `Ambiguous` with it beside a module only the library has. A module path of 50,000 segments is read once for each segment (`resolve::tests`, counted).
- `impls()` (`symbol_index`): the impl shapes (an inherent impl, one through an alias, a trait impl on the type, on `String`, on a generic, on a reference, an impl of a type the index cannot see, a `#[cfg(test)]` block over four lines with its attribute, a block in a module naming `super::Foo`, and one inside a function body, which is not listed) in file order with their lines, spans in lines and bytes, modules, written self types and traits, self type keys and members; the id is the place and finds the block; a file reached under two module paths lists its block once for each, each keyed to its own copy of the type; each block's record is charged to the item-bytes budget (`load_budgets`); a negative impl keeps its `!` in its trait path beside a positive one.
- `reachable` (`reachability`): a bound type covers its inherent and trait impl blocks and their members and a bound method nothing beside it; a bound module covers its nested items, modules and impl blocks and a crate root nothing; a private helper called by a bound item is reached, by `References` from it, and an unreferenced one is not, through a chain of calls; a module a path names reaches nothing below it and is opened when contains reaches it after, also two modules deep below a root that opens only after the path named them; a method call, a field, a lifetime and a declared name reach nothing and `kinds::E::V` reaches `kinds` and `E` through its prefixes; a leading `::` and a path through a type from an integration test; paths inside a binary and its module and inside a test and its support module, whose copy in another target is not reached; an item a macro generates is not reached, a macro invoked by name is, with the paths of its body, a module-level invocation and a `const _` reach nothing, and a `#[macro_export]` macro is read from the crate root; a `#[cfg(test)]` module is not reached from the library, its test reaches the helper it calls through `use super::*`, and the module bound whole covers its items; an impl block root reaches its members and what its header names, and a block on a type of a dependency reaches its members through the block alone; a member of a block is read from the block's module and a trait member from its trait's; a path the resolver refuses (through a glob whose path names nothing, or a name that two globs bring in as two items) reaches nothing, beside the paths it resolves; test code, a test function, a function of a `#[cfg(test)]` module that is no test and a method of a block there, names a dev-dependency and other code of the library does not; the answer is the same for roots in any order and repeated, and over a file with a byte-order mark and CRLF lines; a root past the items or the blocks of the workspace is refused beside one in range, and an in-range id of another workspace reads as the node of this one at that place. `gen` called in a crate of edition 2015, 2018 or 2021 reaches the function, and in 2024 only `r#gen` does; a pointer to a type and a raw borrow of a static reach them, a constant its type and a const fn nothing beside it. `Self::helper`, `Self::CONST` and `self.method()` reach the members of that name in every block of their type and not the type, `Self` alone and a derive's `Self::default()` nothing, and a method called on another value is not read by its name; in a trait and in a block of it the trait's members, and in a block on a type of a dependency the block's own and the trait's; `Self::A` in a block on an enum reaches the enum and a name that is no member on a struct nothing; the type a `where` clause bounds is reached and a generic parameter is not; a walk past the item-bytes budget is `ReachError::Budget` naming the budget, its limit and the file, beside the same walk within it. The scanner and the placing of runs are unit tests (`reach::tests`): runs, path keywords and `Self`, the keywords of each edition, lifetimes, metavariables, declared and bound names, `const` that declares nothing beside the constants, const fns, const blocks and const parameters it comes with, a `where` clause and where it ends, a method called on `self` beside a field of `self` and a method of another value, a byte-order mark, a shebang and a text that does not lex, doc comments ending in a multibyte character and in a name, which start no run and reach nothing, beside a written `#[doc]`, which is a run; each run to its innermost unit.
- The work of reachability is counted (`reach::tests`), never timed: exact counts of the tokens read, the steps of placing runs, the paths resolved and the steps of the walk for three sizes, a growth by equal steps in every counted kind (those two and the segment reads and lookups of the resolver) at three sizes in items and three in references, with the paths resolved the same whatever the references; over modules of types, impl blocks, a trait and methods that call each other through `Self::` and `self.`, every node reached, a growth by equal steps in every counted kind at three sizes in types and three in members, and exact counts at four sizes; a path of 50,000 segments walked under a ceiling of four segment reads for each segment; and what the walk holds, charged exactly the size of a run's record, segment and place for each further run of a file, and refused one byte below its charge with `ReachError::Budget` naming the item-bytes budget, its limit and the file.
- `resolve_from`: a use path counts only what the reading module can access: the module itself, a child of it and a public item of a sibling resolve, and a private item of a module read from outside it and a module private to the one that declares it are `NotFound`; it is read in one name space or in each in turn; a module that is not in the workspace is `UnknownModule`, and a text that is no path is `Invalid`.
- Missing-glob fixtures: a module with `pub use self::missing::*;`, a module that globs
  it, `crate::a::gone::*`, `super::a::inner::nothing::*` and an import of a name the
  missing glob would bring in load, and each lookup that reads one of those globs is
  `UnresolvedGlob` naming the glob and its module, a lookup in the value name space of a
  function there resolving while the lookup in all name spaces fails, beside the
  module's own items and a sibling, which resolve; globs of `std::io`, of `serde`, of
  `::unknown::m`, of a module below a glob of `std::collections`, of a module below
  an import of a dependency module and a module that globs those leave a missing name
  `NotFound`. An empty glob path such as `use ::*;` follows the same unknown-source
  classification in `resolve`, without panicking; the neighboring local glob
  still resolves the module's own item.
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
- Visibility rules of rustc for globs, each in every order of the declarations (the blocks of the crate and the declarations of the module permuted together, each order giving one answer) and each beside a neighbour that resolves: a struct `A` that a module reads through a private glob and a `pub use` re-exports is not found by a reader outside that module and is found by the module, while the function `A` the module defines is found by both; the same re-export of a binding that a `pub` glob or a `pub(crate)` glob makes accessible stays as visible as the re-export; a chain of two re-exports through two private globs is capped twice; an item that a capped re-export and a wider glob both bring in is as visible as the wider; and a glob re-export of a private item is no wider than the item. A name that two globs bring in as two structs, one through a private glob, is `Ambiguous` for a reader of either glob and for a reader of a glob of that module, as rustc's E0659; a module and a struct of one name are ambiguous in the type name space and the unit struct resolves in the value name space; a refusal in one name space leaves the other resolved; and a name that the private glob does not bring in twice still resolves for a reader outside. An import a reader cannot name still keeps its name from the globs, and an import of a value that a reader cannot name leaves the type name space of its name to the globs; a path through a module that two globs bring in as different modules is `Ambiguous`, also for a name only one of them has. In the shapes where a cycle of globs carries an item or a module back to a module that already holds it, a hidden item makes no ambiguity, an ambiguity a cycle carries beside a glob of one of its items has one answer, a hidden module a cycle carries back beside the module itself makes no ambiguity, and an item that a cycle carries back through the module itself keeps its companion, so the two structs stay ambiguous in every order, as rustc has it.
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
  - under the native Linux strategy, a directory swapped for a symlink to outside the
    root between the containment check and the open of a read, a listing or an
    entry's directory, and a directory moved out of the root, with a symlink to
    outside put in its place, after the handle is verified and before the file is
    read, the entries are listed or the entry is looked up, each refused with
    `OutsideRoot`, leaving the outside file unread and the outside names unreturned,
    beside the same calls without a swap, which succeed;
  - under the native Linux strategy, a swap made after the containment check and undone after
    the open, before the handle is verified, for a read, a `read_unseen`, a listing
    and an entry, and a swap flipped at each of those points and again after the
    verification: each is `OutsideRoot`, and no outside text, name or kind is
    returned;
  - under the native Linux strategy, an outside FIFO reached through such
    a swap before the open, watched for opens through inotify: the read is
    `OutsideRoot` and the watch reports no open of the FIFO. The watch is checked
    first against an `O_PATH` reference to the FIFO, which it must not report, and
    last against a non-blocking open of the FIFO, which it must report exactly once;
  - under the native Linux strategy, every open that resolves a name below the root, for reads,
    identities, entries and listings that reach a FIFO or lead to one, is an `O_PATH`
    reference or an `O_NONBLOCK` open. The flags are checked before the open is made,
    so a regression that drops the flag fails the check and does not wait on the FIFO;
  - under the native Linux strategy, the containment fixtures of the disk backend: an
    outside symlink and a chain to it refused beside in-root links, a `..` after an
    outside directory symlink, a FIFO and a directory named as a file, listings and
    their refusals, entry kinds and an entry below an outside directory, the identity
    of a file, its links and a path through a directory link, a symlink loop, and an
    absolute in-root link spelled through a linked parent of the root, which is
    followed, beside a `..` from it that leaves the root;
  - native Linux links into the outside, ending there or passing an outside directory and
    climbing back into the root, each to an outside path that exists and to one that
    does not: the Linux strategy answers `OutsideRoot` and `None`, and the in-root
    file and `NotFound`; a `..` link that stays inside the root is followed;
  - links to an in-root file whose resolution leaves the root and climbs back
    (`../root/real.rs`, an absolute `…/root/../root/real.rs`, and an outside link to
    the root whose own target climbs): Linux reads the file, beside an absolute
    target through a linked parent of the root, which it also reads;
  - in a native Linux child process limited to 64 descriptors, a path 300
    directories deep is read, read through a `..`, listed and looked up;
  - the handle verifiers: the path the system gives for a handle is refused outside
    the root beside one inside it; on Linux a handle on a file outside the root is
    refused beside an in-root handle, and a missing descriptor link is `Io`;
  - ordinary Linux names: a tree with `aux/`, `aux/con.rs` and `nul.txt` loads and
    reads. Windows-only device-name and final-handle-path checks and other-Unix
    step-race checks are excluded from standard execution;
  - every target of the crate, its unit tests included, is checked for native
    Linux with warnings denied; standard validation builds no other OS target;
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
- The work of a listing grows in step with the names it holds, counted and not
  timed: a module of `pub` globs of one name each, at 500, 1000 and 1500 globs,
  reads the glob views a number of times whose third step adds no more than its
  second; the same holds for the view reads and the walk steps of 500, 1000 and
  1500 globs that each carry a re-export of their own name beside as many that each
  carry `Rc` and a `Debug` the reader shadows, and of as many that each carry a
  name beside as many globs of modules that bind it privately, whose listing of
  3000 is also charged under 32 MiB beyond the load. Each count is nonzero and
  stops a listing that passes 32000 of it.
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
- The two visibility rules on the surface, each in every order: a crate whose root has `pub use m::c::*;`, where `m` reads a struct `A` through a private glob and `c` re-exports it, lists the function `A` that `m` defines and not the struct, beside the same crate with a `pub` glob in `m`, which lists both; a name that two globs of `m` bring in as two structs, one through a private glob, fails the listing with `ExportError::Resolve(ResolveError::Ambiguous(_))` for `pub use m::*;` and for `pub use m::Y;`, beside a private glob that brings in another name, which lists what the public glob does, and one item reached through both globs, which lists.
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
- The differential `rustc_oracle` test compares actual rendered import aliases in each namespace through `raf_source::Workspace::resolve_from_in`. Compiler order dependence is accepted only when independently observed outcomes include the stable index answer. Resource failures, index-order instability and cyclic or indeterminate-import disagreements fail. The default draws 64 programs from each of two seed families; `RAF_ORACLE_PROGRAMS=5522` selects that many actual programs from the first family, and `RAF_ORACLE_SEED_BASE=0xc0ffee00` explicitly selects the second. The complete committed corpus is replayed with its required class. Controlled wrong answers verify the comparator, strict diagnostic parsing preserves actionable failures, and the explicit corpus writer checks deterministic output in owned temporary directories.

  | Programs | Family | Probes | Accepted | Refused | documented | order | cycle | index-order | unclassified |
  | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
  | 5,522 | `0xa11ce00` | 32,892 | 15,688 | 17,204 | 6 | 15 | 0 | 0 | 0 |
  | 5,522 | `0xc0ffee00` | 32,725 | 15,783 | 16,942 | 3 | 21 | 0 | 0 | 0 |

  Both release sweeps judge every selected program and show zero visibility-rule
  discrepancies. These finite sweeps establish the listed cases, not arbitrary
  larger generated sets. Corpus programs require `agree`, `documented` or `order`;
  no cycle discrepancy passes. Negative comparator fixtures preserve wrong-item,
  resource-refusal, index-order and independently observed compiler-order checks.
  The minimiser retains the discrepancy it reduces, and the explicit corpus
  writer verifies its deterministic native output in an owned temporary tree.

- Declaration-order fixtures retain both canonical candidates when an independent named or crate-visible route accompanies a private glob ambiguity, including module-prefix reads and returning cycles. The named-route cases cover 120 orders, returning cycles 720, hidden-arrival cases 72, and independent glob-chain cases 720. Compiler order variation is observed separately; it cannot erase the index's stable refusal. Distinct module prefixes remain ambiguous across 144 orders and 1440 import loads, while a prefix supplied as one module resolves. Target-qualified paths retain the separate identities of libraries sharing one crate name.
- Fixtures of an item or a module that a glob cycle carries back to a module beside a glob that holds it apart from the cycle, each in every order of the declarations, where rustc gives the same answer in every order. When a glob delivers the item from a module whose globs do not reach the reading module, the reading module drops the companion that the cycle carries with it: `crate::m0::A`, read from `m3::s1`, is `m1::A` in the value name space and is not found in the type name space in 288 loads, and `s0::B`, read from `m0::s0`, is `m1::s0::B` in the type name space, where the bare first segment names the module that the cycle carries back and not the companion beside it, and is not found in the value name space, in 144 loads. Where the glob that holds the item apart from the cycle reaches its definition through another glob, `crate::m0::A` read from `m3::s1` is `m1::A` in 144 loads and `crate::B` read from `m2::s1` is `m1::B` in 48. Four programs of these shapes are committed under `fuzz/corpus/rustc-oracle`, each marked `agree`.
- Returning-holder fixtures retain `Ambiguous` between `d::X` and `p::X` in `r`, its child `r::t` and an outside reader when the independent private supporter cannot prove replacement of the carried route. They cover 120 outer and 144 inner declaration orders. Removing the hidden competing route leaves all three readers resolving `d::X`.
- The search for the modules that hold an item apart from a reading module is counted by a test-only tally of its steps. Over a chain of pass-through globs of one module read by as many readers as the chain has links, over the same chain whose start globs every reader back and over a chain whose readers glob one another in turn, the steps for one, two and three units grow linearly, so a search that ran again for each reader would exceed the bound, and the glob table work of each load counts at least the steps of its search, each of which is charged to the work budget. A read that holds no companion starts no search, however large the chain. The dominators are checked against their definition on small graphs and on seeded random graphs.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_source::Workspace::exported_items` | doctest | [Usage](#usage) |
| `raf_source::Workspace::impls` | doctest | [Verification](#verification) |
| `raf_source::Workspace::reachable` | doctest | [Verification](#verification) |
| `raf_source::budget::tests::the_table_is_indexed_by_each_budgets_position` | test | [Verification](#verification) |
| `raf_source::cost::tests::a_hash_table_is_charged_at_least_its_buckets` | test | [Verification](#verification) |
| `raf_source::cost::tests::a_list_is_charged_at_least_its_room` | test | [Verification](#verification) |
| `raf_source::cost::tests::a_map_of_one_entry_is_covered_by_what_a_map_costs_once` | test | [Verification](#verification) |
| `raf_source::cost::tests::a_shared_string_is_charged_its_padded_allocation` | test | [Verification](#verification) |
| `raf_source::discovery::tests::native_discovery_names_resume_retained_storage_and_refuse_the_original_grant` | test | [Contract](#contract) |
| `raf_source::error::tests::a_spawn_error_displays_the_file_and_carries_its_cause` | test | [Verification](#verification) |
| `raf_source::error::tests::control_characters_in_a_path_are_escaped` | test | [Verification](#verification) |
| `raf_source::error::tests::every_user_derived_field_of_every_display_is_escaped` | test | [Constraints](#constraints) |
| `raf_source::example_crate::code_contexts` | example target | [Usage](#usage) |
| `raf_source::example_crate::corpus_generate` | example target | [Usage](#usage) |
| `raf_source::example_crate::registry_scan` | example target | [Usage](#usage) |
| `raf_source::exports::tests::a_module_of_pub_globs_is_listed_in_view_reads_linear_in_its_names` | test | [Verification](#verification) |
| `raf_source::exports::tests::a_module_of_pub_globs_that_each_carry_a_re_export_is_listed_in_view_reads_linear_in_its_names` | test | [Verification](#verification) |
| `raf_source::exports::tests::a_module_of_pub_globs_that_each_carry_a_re_export_is_listed_in_walk_steps_linear_in_its_names` | test | [Verification](#verification) |
| `raf_source::exports::tests::native_surface_storage_preserves_physical_failure_and_budget_ownership` | test | [Verification](#verification) |
| `raf_source::exports::tests::readers_of_carried_names_beside_sibling_shadows_list_in_linear_work_and_bounded_bytes` | test | [Verification](#verification) |
| `raf_source::facts::tests::a_column_is_turned_into_its_byte_offset_on_short_long_and_wide_lines` | test | [Verification](#verification) |
| `raf_source::facts::tests::a_large_attribute_on_a_verbatim_use_group_is_read_once_for_all_its_members` | test | [Verification](#verification) |
| `raf_source::facts::tests::a_long_line_of_wide_characters_is_indexed_once_for_all_its_positions` | test | [Verification](#verification) |
| `raf_source::facts::tests::a_verbatim_use_group_is_one_use_fact_per_member_in_its_module_with_the_whole_span` | test | [Verification](#verification) |
| `raf_source::facts::tests::facts_carry_names_lines_documentation_visibility_and_macro_export` | test | [Verification](#verification) |
| `raf_source::facts::tests::facts_record_struct_shapes_unnamed_consts_leading_colons_and_documentation_forms` | test | [Verification](#verification) |
| `raf_source::facts::tests::use_trees_and_impl_headers_are_recorded_as_written` | test | [Verification](#verification) |
| `raf_source::glob::tests::a_segment_is_matched_in_at_most_pattern_times_name_steps` | test | [Verification](#verification) |
| `raf_source::glob_scope::delivery::maximal_public_replacements_avoid_repeat_proofs_while_narrower_ones_are_widened` | test | [Verification](#verification) |
| `raf_source::glob_scope::delivery::native_delivery_growth_admits_old_and_new_spill_buffers_together` | test | [Verification](#verification) |
| `raf_source::glob_scope::delivery::native_delivery_spill_is_lifo_and_failed_push_releases_only_the_incoming_delta` | test | [Verification](#verification) |
| `raf_source::glob_scope::delivery::one_returning_carrier_retains_multiple_independently_selected_identities` | test | [Verification](#verification) |
| `raf_source::glob_scope::delivery::recorded_name_hash_storage_loads_at_exact_item_and_glob_grants_and_refuses_one_less` | test | [Verification](#verification) |
| `raf_source::glob_scope::delivery::recorded_name_rows_survive_real_reconstruction_with_new_equal_interned_names` | test | [Verification](#verification) |
| `raf_source::glob_scope::delivery::widest_routes_merge_firm_evidence_in_place_and_refuse_before_mutation` | test | [Verification](#verification) |
| `raf_source::glob_scope::equivalence::an_answer_that_differs_between_the_two_modes_is_reported_with_its_load_and_both_answers` | test | [Verification](#verification) |
| `raf_source::glob_scope::equivalence::the_settling_gives_the_answers_of_the_reference_mode_to_every_generated_and_corpus_program_in_three_orders` | test | [Verification](#verification) |
| `raf_source::glob_scope::incremental::a_covert_read_is_listed_before_its_names_arrive_so_an_item_its_view_gains_while_they_arrive_reaches_the_module` | test | [Verification](#verification) |
| `raf_source::glob_scope::incremental::a_covert_reader_is_woken_in_the_turn_an_active_read_of_it_would_have_so_the_imports_that_wait_on_the_name_are_queued_again_in_that_order` | test | [Verification](#verification) |
| `raf_source::glob_scope::incremental::a_covert_reader_takes_a_gain_in_the_turn_an_active_read_of_it_would_have_so_the_companions_of_a_name_are_kept_as_its_drops_allow` | test | [Verification](#verification) |
| `raf_source::glob_scope::incremental::a_module_whose_covert_read_turns_active_is_no_longer_looked_at_as_a_covert_reader_when_the_view_grows` | test | [Verification](#verification) |
| `raf_source::glob_scope::incremental::a_view_made_when_its_module_has_left_the_open_readers_makes_it_open_again_until_it_learns_what_it_cannot_access` | test | [Verification](#verification) |
| `raf_source::glob_scope::incremental::an_item_its_class_cannot_access_that_arrived_under_a_name_its_view_did_not_hold_makes_the_first_item_it_can_access_ambiguous` | test | [Verification](#verification) |
| `raf_source::glob_scope::incremental::an_item_that_a_view_read_through_a_glob_its_class_cannot_access_holds_makes_the_first_item_it_can_access_under_the_name_ambiguous` | test | [Verification](#verification) |
| `raf_source::glob_scope::lookups::a_query_after_the_load_is_charged_its_own_lookup_steps_against_the_limit_of_one_query` | test | [Verification](#verification) |
| `raf_source::glob_scope::lookups::every_glob_read_a_lookup_of_the_load_looks_at_is_charged_once` | test | [Verification](#verification) |
| `raf_source::glob_scope::lookups::every_limit_below_the_steps_of_a_query_refuses_it_and_the_refusal_comes_before_any_answer` | test | [Verification](#verification) |
| `raf_source::glob_scope::lookups::every_limit_below_the_work_of_the_lookups_shape_is_refused_by_the_work_budget_and_by_nothing_else` | test | [Verification](#verification) |
| `raf_source::glob_scope::lookups::every_query_counts_the_lookups_of_its_walks_and_of_the_views_it_looks_at_to_find_them` | test | [Verification](#verification) |
| `raf_source::glob_scope::lookups::every_way_of_resolving_a_path_is_a_query_with_its_own_limit` | test | [Usage](#usage) |
| `raf_source::glob_scope::lookups::queries_on_other_threads_each_count_from_zero` | test | [Verification](#verification) |
| `raf_source::glob_scope::lookups::the_lookups_of_many_named_imports_through_many_globs_are_charged_and_refused_by_a_small_limit_at_the_first_checkpoint_past_it` | test | [Verification](#verification) |
| `raf_source::glob_scope::lookups::the_public_surface_is_one_query_whose_lookups_and_those_of_the_paths_it_resolves_share_its_limit` | test | [Verification](#verification) |
| `raf_source::glob_scope::monotone::an_entry_held_as_a_companion_is_widened_as_it_is_added_again_and_made_plain_only_by_a_plain_entry` | test | [Verification](#verification) |
| `raf_source::glob_scope::monotone::an_entry_that_makes_a_plain_item_a_companion_again_is_refused` | test | [Verification](#verification) |
| `raf_source::glob_scope::monotone::an_entry_that_narrows_the_level_an_item_is_held_at_is_refused` | test | [Verification](#verification) |
| `raf_source::glob_scope::ranked::tests::a_complete_definition_and_relay_rank_is_already_its_full_closure` | test | [Verification](#verification) |
| `raf_source::glob_scope::ranked::tests::a_mixed_relay_and_a_removed_anchor_match_actual_reconstruction` | test | [Verification](#verification) |
| `raf_source::glob_scope::ranked::tests::absent_cells_and_ungrounded_cyclic_residents_cannot_pass_closure_or_rank` | test | [Verification](#verification) |
| `raf_source::glob_scope::ranked::tests::an_exact_empty_namespace_proves_bottom_without_any_transfer` | test | [Verification](#verification) |
| `raf_source::glob_scope::ranked::tests::an_impossible_aggregate_reservation_is_a_typed_layout_refusal_before_allocation` | test | [Verification](#verification) |
| `raf_source::glob_scope::ranked::tests::an_incomplete_successful_rank_still_checks_its_later_complete_input` | test | [Verification](#verification) |
| `raf_source::glob_scope::ranked::tests::complete_public_peer_discovery_omits_only_its_redundant_postpublication_pass` | test | [Verification](#verification) |
| `raf_source::glob_scope::ranked::tests::discovery_independence_scans_preserve_the_original_work_refusal` | test | [Verification](#verification) |
| `raf_source::glob_scope::ranked::tests::homogeneous_returning_carriers_share_an_actual_holder_retrieval` | test | [Verification](#verification) |
| `raf_source::glob_scope::ranked::tests::interior_levels_named_imports_and_glob_prefixes_keep_ordinary_reconstruction` | test | [Verification](#verification) |
| `raf_source::glob_scope::ranked::tests::named_extern_alias_and_qualified_glob_neighbors_do_not_certify_independence` | test | [Verification](#verification) |
| `raf_source::glob_scope::ranked::tests::proof_scratch_and_work_refuse_one_less_than_the_actual_original_grant` | test | [Verification](#verification) |
| `raf_source::glob_scope::ranked::tests::public_peer_rows_are_rooted_without_a_second_table_construction` | test | [Verification](#verification) |
| `raf_source::glob_scope::ranked::tests::roots_outside_resident_targets_still_require_the_second_discovery` | test | [Verification](#verification) |
| `raf_source::glob_scope::settling::a_companion_fan_in_keeps_each_operation_population_linear_without_list_copies` | test | [Verification](#verification) |
| `raf_source::glob_scope::settling::a_companion_hub_bounds_delivery_and_carrier_graph_growth_together` | test | [Verification](#verification) |
| `raf_source::glob_scope::settling::a_companion_hub_keeps_local_definitions_and_reads_without_copying_candidate_lists` | test | [Verification](#verification) |
| `raf_source::glob_scope::settling::a_reading_again_looks_at_the_reads_whose_view_holds_the_name_whatever_the_reads_of_the_module` | test | [Verification](#verification) |
| `raf_source::glob_scope::settling::named_import_lookup_routes_stay_linear_and_read_work_stays_quadratic` | test | [Verification](#verification) |
| `raf_source::glob_scope::settling::the_settling_of_a_chain_of_globs_counts_its_pinned_polynomials_at_n_2n_and_3n` | test | [Verification](#verification) |
| `raf_source::glob_scope::settling::the_settling_of_a_fan_in_of_private_globs_counts_its_pinned_polynomials_at_n_2n_and_3n` | test | [Verification](#verification) |
| `raf_source::glob_scope::settling::the_settling_of_a_fan_in_of_pub_globs_counts_its_pinned_polynomials_at_n_2n_and_3n` | test | [Verification](#verification) |
| `raf_source::glob_scope::settling::the_settling_of_a_hub_counts_its_pinned_polynomials_at_n_2n_and_3n` | test | [Verification](#verification) |
| `raf_source::glob_scope::settling::the_settling_of_a_module_of_many_items_globbed_from_many_modules_counts_its_pinned_polynomials_at_n_2n_and_3n` | test | [Verification](#verification) |
| `raf_source::glob_scope::settling::the_settling_of_a_module_whose_names_many_views_hold_counts_its_pinned_polynomials_at_n_2n_and_3n` | test | [Verification](#verification) |
| `raf_source::glob_scope::settling::the_settling_of_the_prelude_pattern_counts_its_pinned_polynomials_at_n_2n_and_3n` | test | [Verification](#verification) |
| `raf_source::glob_scope::table_bytes::each_covert_reader_open_reader_and_view_read_through_a_glob_a_class_cannot_access_is_charged_its_entry_and_the_first_room_of_its_table` | test | [Verification](#verification) |
| `raf_source::glob_scope::table_bytes::each_name_a_view_dropped_and_each_name_a_reading_again_set_is_charged_its_entry_and_the_first_room_of_its_record` | test | [Verification](#verification) |
| `raf_source::glob_scope::table_bytes::each_name_a_view_takes_is_charged_its_slot_among_the_holders_of_the_name_and_the_first_companion_its_entry` | test | [Verification](#verification) |
| `raf_source::glob_scope::table_bytes::each_read_of_a_view_is_charged_its_place_among_the_reads_of_its_module_of_that_view` | test | [Verification](#verification) |
| `raf_source::glob_scope::table_bytes::the_tables_of_the_views_that_hold_each_name_and_of_the_names_with_a_companion_are_charged_their_first_room` | test | [Verification](#verification) |
| `raf_source::glob_scope::tests::a_read_that_holds_no_companion_starts_no_search_for_the_holders_of_its_items` | test | [Contract](#contract) |
| `raf_source::glob_scope::tests::holder_visibility_uses_ancestry_instead_of_sibling_depth` | test | [Verification](#verification) |
| `raf_source::glob_scope::tests::independent_same_item_supply_preserves_the_carriers_ambiguity` | test | [Verification](#verification) |
| `raf_source::glob_scope::tests::named_supplier_levels_stay_with_the_declaration_that_provides_them` | test | [Verification](#verification) |
| `raf_source::glob_scope::tests::native_resumed_growth_admits_overlapping_buffers_before_mutation` | test | [Verification](#verification) |
| `raf_source::glob_scope::tests::the_dominators_of_seeded_random_graphs_follow_their_definition` | test | [Verification](#verification) |
| `raf_source::glob_scope::tests::the_dominators_of_small_holder_graphs_follow_their_definition` | test | [Verification](#verification) |
| `raf_source::glob_scope::tests::the_holders_of_an_item_apart_from_each_reader_of_a_chain_of_globs_are_found_in_steps_linear_in_the_chain` | test | [Verification](#verification) |
| `raf_source::glob_scope::tests::the_holders_of_an_item_apart_from_readers_that_form_a_chain_of_their_own_are_found_in_steps_linear_in_the_chains` | test | [Verification](#verification) |
| `raf_source::glob_scope::tests::the_holders_of_an_item_apart_from_readers_the_start_of_the_chain_globs_back_are_found_in_steps_linear_in_the_chain` | test | [Verification](#verification) |
| `raf_source::glob_scope::work_budget::a_new_view_charges_raw_reads_and_unique_source_names_before_gathering` | test | [Verification](#verification) |
| `raf_source::glob_scope::work_budget::a_settle_heavy_shape_is_refused_at_the_charge_that_passes_a_small_limit` | test | [Constraints](#constraints) |
| `raf_source::glob_scope::work_budget::actual_work_from_100_to_800_stays_within_each_shapes_doubling_bound` | test | [Verification](#verification) |
| `raf_source::glob_scope::work_budget::an_export_read_charges_each_visited_read_and_holder_before_returning_its_answer` | test | [Verification](#verification) |
| `raf_source::glob_scope::work_budget::doubling_a_shape_keeps_actual_work_within_its_growth_bound` | test | [Constraints](#constraints) |
| `raf_source::glob_scope::work_budget::each_shape_loads_at_its_measured_work_and_refuses_one_step_less` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::glob_scope::work_budget::holder_scan_sources_beyond_the_admitted_growth_sizes_are_refused` | test | [Verification](#verification) |
| `raf_source::impls::tests::alternate_aliases_spill_the_nominal_stack_under_its_original_grant` | test | [Verification](#verification) |
| `raf_source::impls::tests::missing_impl_head_admits_pending_glob_spill_and_keeps_the_missing_neighbor` | test | [Verification](#verification) |
| `raf_source::index::tests::a_module_file_visited_before_the_declaration_that_names_it_is_an_internal_error` | test | [Verification](#verification) |
| `raf_source::index::tests::a_target_whose_root_file_is_missing_is_a_typed_internal_error` | test | [Verification](#verification) |
| `raf_source::index::tests::an_unknown_dependency_identity_is_not_an_external_or_libraryless_package` | test | [Contract](#contract) |
| `raf_source::item::tests::an_item_record_is_at_most_two_hundred_fifty_six_bytes` | test | [Verification](#verification) |
| `raf_source::item::tests::shared_text_reads_and_compares_as_a_str` | test | [Verification](#verification) |
| `raf_source::members::tests::distinct_patterns_at_both_caps_are_tested_a_bounded_number_of_times` | test | [Verification](#verification) |
| `raf_source::members::tests::duplicate_patterns_are_compiled_and_tested_once` | test | [Verification](#verification) |
| `raf_source::meter::tests::native_emission_spill_preserves_cumulative_item_admission` | test | [Explanation](#explanation) |
| `raf_source::meter::tests::native_graph_storage_preserves_overlapping_grants_and_typed_refusals` | test | [Explanation](#explanation) |
| `raf_source::meter::tests::native_recovery_preserves_nested_provenance_and_restores_its_own_refusal` | test | [Contract](#contract) |
| `raf_source::meter::tests::native_storage_capacity_failure_has_its_original_error_source` | test | [Constraints](#constraints) |
| `raf_source::meter::tests::native_storage_preserves_the_item_limit_and_cumulative_charges` | test | [Constraints](#constraints) |
| `raf_source::meter::tests::retained_payload_moves_its_original_grant_without_readmission` | test | [Verification](#verification) |
| `raf_source::meter::tests::transient_reservations_refuse_atomically_and_release_on_every_exit` | test | [Verification](#verification) |
| `raf_source::meter::tests::work_is_never_released_and_each_meter_counts_from_zero` | test | [Verification](#verification) |
| `raf_source::meter::tests::work_up_to_the_limit_is_accepted_and_the_charge_that_passes_it_is_refused` | test | [Verification](#verification) |
| `raf_source::path::display_tests::control_characters_in_a_refused_path_are_escaped` | test | [Constraints](#constraints) |
| `raf_source::platform::tests::compiler_platform_refuses_incomplete_or_malformed_output` | test | [Contract](#contract) |
| `raf_source::platform::tests::conditional_module_routes_remain_unknown` | test | [Contract](#contract) |
| `raf_source::platform::tests::inner_crate_and_external_module_guards_are_both_observed` | test | [Contract](#contract) |
| `raf_source::platform::tests::invalid_item_is_an_actionable_error` | test | [Contract](#contract) |
| `raf_source::platform::tests::nonplatform_conditions_and_cfg_attr_do_not_invent_exclusion` | test | [Contract](#contract) |
| `raf_source::platform::tests::parent_platform_guards_do_not_filter_the_source_inventory` | test | [Contract](#contract) |
| `raf_source::platform::tests::platform_guard_reference_storage_uses_the_remaining_item_grant` | test | [Contract](#contract) |
| `raf_source::platform::tests::platform_guard_work_exhaustion_is_typed_beside_a_sufficient_neighbor` | test | [Contract](#contract) |
| `raf_source::reach::format::identity::tests::native_propagation_spill_refuses_without_refunding_the_cumulative_grant` | test | [Constraints](#constraints) |
| `raf_source::reach::format::identity::tests::native_propagation_visits_cycles_and_duplicate_edges_once_per_active_source` | test | [Constraints](#constraints) |
| `raf_source::reach::tests::a_byte_order_mark_and_a_shebang_are_left_out_and_offsets_are_those_of_the_text` | test | [Verification](#verification) |
| `raf_source::reach::tests::a_doc_comment_is_no_run_whatever_character_ends_it` | test | [Verification](#verification) |
| `raf_source::reach::tests::a_lifetime_a_metavariable_a_declared_name_and_a_bound_name_are_no_run` | test | [Verification](#verification) |
| `raf_source::reach::tests::a_name_called_on_self_is_a_method_run_and_a_field_of_self_is_not` | test | [Verification](#verification) |
| `raf_source::reach::tests::a_path_of_fifty_thousand_segments_is_walked_once_and_not_once_for_each_prefix` | test | [Contract](#contract) |
| `raf_source::reach::tests::a_run_is_an_identifier_path_that_no_dot_precedes_with_its_first_byte` | test | [Verification](#verification) |
| `raf_source::reach::tests::a_shared_files_edition_caches_are_each_counted_and_charged_once` | test | [Verification](#verification) |
| `raf_source::reach::tests::a_where_clause_keeps_the_type_its_bound_is_put_on` | test | [Verification](#verification) |
| `raf_source::reach::tests::code_prefix_reads_share_one_query_account_and_refuse_one_less` | test | [Contract](#contract) |
| `raf_source::reach::tests::const_declares_nothing_so_a_pointer_and_a_raw_borrow_name_their_path` | test | [Verification](#verification) |
| `raf_source::reach::tests::each_run_goes_to_the_innermost_unit_that_holds_its_start` | test | [Verification](#verification) |
| `raf_source::reach::tests::format_candidates_charge_owned_bytes_and_return_worker_work_to_the_caller` | test | [Contract](#contract) |
| `raf_source::reach::tests::path_keywords_start_a_run_and_other_keywords_are_no_segment` | test | [Verification](#verification) |
| `raf_source::reach::tests::the_keywords_are_those_of_the_edition` | test | [Verification](#verification) |
| `raf_source::reach::tests::the_work_grows_with_the_items_and_the_references_and_not_their_product` | test | [Contract](#contract) |
| `raf_source::reach::tests::the_work_of_a_walk_is_counted_exactly` | test | [Contract](#contract) |
| `raf_source::reach::tests::the_work_over_types_impl_blocks_traits_and_members_grows_with_each_and_not_their_product` | test | [Verification](#verification) |
| `raf_source::reach::tests::what_the_walk_holds_is_charged_to_the_item_bytes_budget_on_from_the_load` | test | [Verification](#verification) |
| `raf_source::resolve::determinacy::tests::native_query_layout_failure_is_reported_as_storage_without_allocating` | test | [Verification](#verification) |
| `raf_source::resolve::determinacy::tests::selected_path_proof_spills_on_real_routes_and_releases_its_scratch` | test | [Verification](#verification) |
| `raf_source::resolve::determinacy::tests::selected_query_and_surface_proofs_admit_exact_work_and_release_all_scratch` | test | [Verification](#verification) |
| `raf_source::resolve::determinacy::tests::selected_routes_admit_the_first_native_spill_exactly_and_refuse_before_mutation` | test | [Verification](#verification) |
| `raf_source::resolve::tests::a_module_path_of_fifty_thousand_segments_is_read_once_to_find_its_crate` | test | [Contract](#contract) |
| `raf_source::resolve::tests::a_path_of_fifty_thousand_segments_reads_each_segment_a_bounded_number_of_times` | test | [Verification](#verification) |
| `raf_source::resolve::tests::every_user_derived_field_of_every_resolve_error_is_escaped` | test | [Constraints](#constraints) |
| `raf_source::settle::replay::genuine_paths_are_fixed_but_glob_prefixes_and_named_shadows_require_replay` | test | [Verification](#verification) |
| `raf_source::settle::replay::path_visibility_and_unmatched_or_open_resident_targets_keep_replay` | test | [Verification](#verification) |
| `raf_source::settle::tests::a_binding_list_stays_sorted_keeps_the_better_route_and_takes_only_its_room` | test | [Verification](#verification) |
| `raf_source::settle::tests::a_glob_prefix_bound_in_type_still_waits_for_its_named_import` | test | [Verification](#verification) |
| `raf_source::settle::tests::a_round_that_decides_nothing_more_or_a_settle_too_many_is_reported_in_every_build` | test | [Verification](#verification) |
| `raf_source::settle::tests::components_come_out_once_each_after_every_component_they_lead_to` | test | [Verification](#verification) |
| `raf_source::settle::tests::each_settle_after_the_first_is_charged_what_it_builds_again` | test | [Verification](#verification) |
| `raf_source::settle::tests::excluding_ones_own_named_binding_reserves_the_glob_fallback_too` | test | [Verification](#verification) |
| `raf_source::settle::tests::native_component_grouping_refuses_one_below_its_live_storage_bound` | test | [Verification](#verification) |
| `raf_source::settle::tests::pending_and_blocker_storage_require_the_original_grants` | test | [Constraints](#constraints) |
| `raf_source::settle::tests::pending_counts_duplicate_attempts_and_refuses_before_removing_a_queued_import` | test | [Verification](#verification) |
| `raf_source::settle::tests::repeated_glob_lookups_visit_reads_without_spending_the_settling_work_grant` | test | [Constraints](#constraints) |
| `raf_source::settle::tests::retained_import_batches_reserve_previous_results_and_new_reads_together` | test | [Verification](#verification) |
| `raf_source::settle::tests::the_imports_left_are_decided_in_rounds_that_do_not_grow_with_them` | test | [Verification](#verification) |
| `raf_source::settle::tests::the_imports_of_a_name_are_found_among_names_that_share_its_first_eight_bytes` | test | [Verification](#verification) |
| `raf_source::test_crate::code_paths::a_module_body_reads_itself_while_its_declaration_reads_its_parent` | test | [Contract](#contract) |
| `raf_source::test_crate::code_paths::alias_members_preserve_definition_chains_cycles_and_original_query_grants` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_source::test_crate::code_paths::code_associated_calls_prefer_inherent_methods_without_erasing_symbol_inventory` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::code_paths::code_context_tables_spend_only_the_original_load_allowance_and_preserve_usage` | test | [Contract](#contract) |
| `raf_source::test_crate::code_paths::code_heads_read_the_edition_2015_module_where_use_heads_read_the_crate_root` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples) |
| `raf_source::test_crate::code_paths::complete_module_paths_return_actual_type_sites_without_becoming_values` | test | [Contract](#contract) |
| `raf_source::test_crate::code_paths::exported_macros_keep_their_lexical_module_beside_their_crate_root_export` | test | [Contract](#contract) |
| `raf_source::test_crate::code_paths::external_library_query_scratch_uses_the_original_native_grants` | test | [Contract](#contract), [Usage](#usage) |
| `raf_source::test_crate::code_paths::external_library_receipts_keep_actual_names_exports_and_definition_identity` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples) |
| `raf_source::test_crate::code_paths::inherent_method_applicability_is_local_to_the_selected_signature` | test | [Contract](#contract) |
| `raf_source::test_crate::code_paths::library_receipts_preserve_effective_name_ambiguity_and_deduplicate_one_root` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::code_paths::method_and_trait_members_read_their_defining_module_instead_of_their_symbol_parent` | test | [Contract](#contract) |
| `raf_source::test_crate::code_paths::pattern_binding_observations_exclude_names_introduced_by_guard_expressions` | test | [Contract](#contract) |
| `raf_source::test_crate::configured_sources::a_physical_file_cannot_silently_merge_different_active_package_contexts` | test | [Contract](#contract) |
| `raf_source::test_crate::configured_sources::cargo_feature_names_use_rust_unicode_xid_with_actual_cargo_punctuation` | test | [Contract](#contract) |
| `raf_source::test_crate::configured_sources::configured_aliases_resolve_only_active_real_definitions_and_paths` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_source::test_crate::configured_sources::configured_predicates_refuse_qualified_paths_beside_unqualified_names` | test | [Contract](#contract) |
| `raf_source::test_crate::configured_sources::configured_syntax_and_feature_work_spend_the_original_combined_grant` | test | [Contract](#contract) |
| `raf_source::test_crate::configured_sources::distinct_declared_sets_change_definitions_and_keep_their_exact_receipt` | test | [Contract](#contract), [Examples](#examples) |
| `raf_source::test_crate::configured_sources::empty_cfg_attr_requires_its_comma_and_boolean_literals_are_not_raw_flags` | test | [Contract](#contract) |
| `raf_source::test_crate::configured_sources::forwarding_default_and_inherited_feature_requirements_are_validated_at_the_actual_destination` | test | [Contract](#contract) |
| `raf_source::test_crate::configured_sources::missing_unknown_inactive_or_incomplete_configuration_refuses_beside_valid_selection` | test | [Contract](#contract) |
| `raf_source::test_crate::configured_sources::nested_cfg_attr_boolean_guards_escaped_values_and_test_builds_select_actual_declarations` | test | [Contract](#contract) |
| `raf_source::test_crate::configured_sources::raw_cfg_identifiers_have_one_logical_identity_and_cannot_bypass_feature_separation` | test | [Contract](#contract) |
| `raf_source::test_crate::configured_sources::target_and_host_conditions_use_the_selected_compilation_role` | test | [Contract](#contract), [Usage](#usage), [Constraints](#constraints) |
| `raf_source::test_crate::configured_targets::all_selected_target_kinds_keep_their_native_and_own_library_routes` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples) |
| `raf_source::test_crate::configured_targets::conditional_and_optional_edges_keep_each_targets_actual_activation` | test | [Contract](#contract) |
| `raf_source::test_crate::configured_targets::configured_syntax_queries_select_the_actual_target_and_preserve_budget_causes` | test | [Contract](#contract) |
| `raf_source::test_crate::configured_targets::development_edges_require_a_real_development_or_declared_test_context` | test | [Contract](#contract) |
| `raf_source::test_crate::configured_targets::missing_duplicate_and_feature_ineligible_targets_refuse_beside_complete_selection` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::configured_targets::retained_target_lists_and_configuration_queries_spend_the_original_grants` | test | [Contract](#contract) |
| `raf_source::test_crate::configured_targets::shared_files_reuse_only_the_same_actual_syntax_selection` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `raf_source::test_crate::corpus_replay::a_recorded_list_that_differs_from_the_real_index_fails_the_replay` | test | [Verification](#verification) |
| `raf_source::test_crate::corpus_replay::child_main` | test | [Verification](#verification) |
| `raf_source::test_crate::corpus_replay::corpus_files_hold_exactly_the_text_of_their_shapes` | test | [Verification](#verification) |
| `raf_source::test_crate::corpus_replay::evasion_shapes_are_refused_naming_their_budget_beside_their_at_budget_neighbours` | test | [Verification](#verification) |
| `raf_source::test_crate::corpus_replay::every_corpus_file_is_refused_or_indexed_with_its_recorded_symbols_and_none_aborts` | test | [Verification](#verification) |
| `raf_source::test_crate::corpus_replay::every_named_shape_has_a_corpus_file_and_no_file_is_unrecorded` | test | [Verification](#verification) |
| `raf_source::test_crate::corpus_replay::files_at_a_budget_exit_zero_and_files_over_it_are_refused_naming_file_and_budget` | test | [Verification](#verification) |
| `raf_source::test_crate::corpus_replay::the_shape_table_matches_the_expected_names` | test | [Verification](#verification) |
| `raf_source::test_crate::corpus_replay::write_corpus` | test | [Verification](#verification) |
| `raf_source::test_crate::dependency_declarations::declarations_retain_sources_flags_aliases_and_unevaluated_conditions` | test | [Contract](#contract) |
| `raf_source::test_crate::dependency_declarations::inheritance_preserves_both_entries_and_the_workspace_path_base` | test | [Contract](#contract) |
| `raf_source::test_crate::dependency_declarations::malformed_declarations_refuse_with_valid_neighbors` | test | [Contract](#contract), [Explanation](#explanation) |
| `raf_source::test_crate::dependency_declarations::retained_declarations_use_the_original_item_allowance` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::a_first_party_module_named_like_the_test_namespace_loads_beside_a_test_target` | test | [Verification](#verification) |
| `raf_source::test_crate::discovery_targets::a_library_and_a_binary_with_one_name_resolve_under_their_own_crate_paths` | test | [Verification](#verification) |
| `raf_source::test_crate::discovery_targets::a_manifest_over_the_cap_is_refused_naming_it_and_one_of_exactly_the_cap_loads` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::a_manifest_that_is_not_toml_or_has_neither_table_is_a_typed_error` | test | [Contract](#contract), [Examples](#examples) |
| `raf_source::test_crate::discovery_targets::a_manifest_with_a_trailing_comment_and_single_quoted_values_is_honoured` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::a_missing_root_is_an_error_beside_an_existing_one` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::a_package_nested_below_a_package_without_a_workspace_is_not_discovered` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::a_shared_test_helper_directory_is_reached_by_two_targets_and_listed_once` | test | [Verification](#verification) |
| `raf_source::test_crate::discovery_targets::a_target_path_that_leaves_the_tree_is_refused_and_a_path_that_stays_is_accepted` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::a_target_table_without_a_name_is_an_error_naming_the_key` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::a_test_setting_of_the_wrong_type_is_a_manifest_error_naming_its_key` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::a_virtual_manifest_contributes_nothing_but_its_children_are_found` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::an_empty_root_list_is_a_no_roots_error_beside_dot_which_loads` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::an_explicit_target_file_that_is_missing_is_an_error_naming_the_manifest_key` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::autodiscovery_flags_turn_inferred_targets_off_but_explicit_tables_are_still_found` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::deeply_nested_inline_tables_are_a_typed_error_on_the_callers_stack_and_a_shallow_one_loads` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::discovery_target_and_module_failures_precede_later_load_phases` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::each_target_kind_with_default_paths_has_its_name_kind_crate_path_and_shipping_flag` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::each_target_kind_with_explicit_paths_resolves_beside_its_default_neighbour` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::harness_required_features_test_and_doctest_are_read_from_each_target_table` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::independently_rooted_same_name_packages_resolve_only_by_their_selected_identity` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::inherited_workspaces_and_renamed_path_dependencies_stay_within_each_source` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::only_the_named_roots_are_searched` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::relative_queries_select_the_reading_package_without_crossing_source_roots` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::required_features_on_the_library_is_a_manifest_error_beside_a_binary_that_names_them` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::target_and_hidden_directories_are_skipped_beside_a_discovered_sibling` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::targets_are_sorted_and_two_loads_are_equal` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::the_root_module_of_every_target_is_recorded_under_its_crate_path` | test | [Verification](#verification) |
| `raf_source::test_crate::discovery_targets::two_packages_whose_targets_share_crate_paths_load_and_are_told_apart_by_package` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::two_packages_with_one_name_are_an_error_naming_both_manifests` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::two_targets_sharing_one_root_are_an_error_naming_the_second` | test | [Contract](#contract) |
| `raf_source::test_crate::discovery_targets::two_targets_with_one_crate_path_are_an_error_naming_both_beside_distinct_names` | test | [Contract](#contract) |
| `raf_source::test_crate::disk_tree_containment::a_directory_named_as_a_file_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_containment::a_directory_symlink_loop_ends_with_an_error_and_is_listed_but_never_entered` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_containment::a_fifo_named_as_a_file_is_refused_without_blocking_beside_a_regular_file` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_containment::a_file_that_is_missing_is_not_found_and_one_that_leaves_the_root_is_outside_root` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_containment::a_file_that_is_not_utf8_is_refused_beside_one_that_is` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::disk_tree_containment::a_missing_root_is_an_error_beside_an_existing_one` | test | [Contract](#contract) |
| `raf_source::test_crate::disk_tree_containment::a_non_utf8_entry_name_fails_the_listing_beside_a_listing_without_one` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_containment::a_parent_segment_after_an_outside_directory_symlink_is_resolved_on_disk_and_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_containment::a_symlink_to_a_file_outside_is_refused_beside_an_in_root_symlink_that_is_accepted` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::disk_tree_containment::a_symlink_to_a_symlink_that_leaves_the_root_is_refused_beside_a_chain_that_stays` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_containment::a_symlinked_root_is_followed_to_its_target_and_containment_is_judged_there` | test | [Contract](#contract) |
| `raf_source::test_crate::disk_tree_containment::entry_kinds_cover_file_directory_symlink_other_and_absent` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_containment::listings_are_sorted_by_name` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::disk_tree_containment::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen` | test | [Contract](#contract) |
| `raf_source::test_crate::disk_tree_containment::the_identity_of_a_file_is_the_device_and_inode_of_its_verified_handle` | test | [Contract](#contract) |
| `raf_source::test_crate::disk_tree_root::a_dangling_manifest_is_refused_without_falling_back_to_its_workspace_ancestor` | test | [Constraints](#constraints) |
| `raf_source::test_crate::disk_tree_root::a_root_that_is_missing_is_not_found_on_every_platform` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_root::disk_and_memory_trees_agree_on_regular_files` | test | [Verification](#verification) |
| `raf_source::test_crate::disk_tree_root::root_discovery_refuses_directory_aliases_and_unsafe_parent_reductions_beside_real_starts` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::error_text::a_manifest_with_control_characters_in_a_value_is_shown_escaped` | test | [Verification](#verification) |
| `raf_source::test_crate::error_text::a_path_attribute_with_terminal_escapes_is_shown_escaped` | test | [Verification](#verification) |
| `raf_source::test_crate::error_text::a_resolved_path_or_package_with_control_characters_is_shown_escaped` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_binary_lists_its_own_pub_items_and_a_test_target_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_binder_in_an_irreducible_cycle_that_dominates_nothing_leaves_its_name_carried` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_carried_outside_re_export_is_kept_out_by_a_restricted_glob_or_a_name_on_the_way` | test | [Constraints](#constraints) |
| `raf_source::test_crate::exported_items::a_carried_re_export_is_kept_out_exactly_where_every_route_passes_a_module_that_binds_its_name` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_carried_re_export_is_listed_hidden_or_absent_exactly_as_an_item_of_the_same_shape` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_carrier_with_pub_globs_of_its_own_is_not_its_own_binder` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_chain_of_re_exports_through_nested_modules_is_followed_and_a_restricted_hop_stops_it` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_chain_of_twenty_thousand_pub_globs_is_listed_on_a_two_mib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_glob_re_export_brings_in_the_pub_items_of_its_module_only` | test | [Contract](#contract) |
| `raf_source::test_crate::exported_items::a_hidden_glob_partway_along_a_route_hides_a_carried_re_export_no_more_than_an_item` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_module_reached_by_a_hidden_and_a_shown_glob_carries_its_re_export_shown` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_module_reached_by_two_pub_paths_is_listed_under_each_and_read_under_the_first` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_name_two_globs_bring_in_as_different_items_one_through_a_private_glob_is_refused_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_named_re_export_of_an_enum_variant_is_a_re_export_and_a_glob_of_an_enum_adds_nothing` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_pub_function_is_exported_beside_private_and_restricted_ones_which_are_not` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::exported_items::a_pub_item_of_a_private_module_is_exported_only_through_a_pub_use_and_under_its_rename` | test | [Contract](#contract) |
| `raf_source::test_crate::exported_items::a_re_export_of_a_path_the_index_cannot_follow_is_listed_as_a_re_export` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_re_export_of_an_outside_crate_that_a_pub_glob_carries_is_a_re_export_of_the_reader` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_re_export_of_another_crate_is_a_re_export_and_not_an_item` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_re_export_that_only_a_hidden_glob_carries_past_a_shadow_is_hidden_as_an_item_would_be` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_re_export_two_globs_carry_is_written_as_the_first_glob_that_carries_it` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_restricted_or_private_re_export_is_not_exported_beside_a_pub_one_that_is` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::a_surface_over_the_item_budget_left_by_the_load_is_refused_beside_one_within_it` | test | [Contract](#contract) |
| `raf_source::test_crate::exported_items::a_surface_refused_for_a_module_name_a_glob_cycle_carries_lists_the_same_candidates_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::an_ambiguous_a_cyclic_or_an_unresolved_public_name_fails_beside_a_neighbour_that_lists` | test | [Constraints](#constraints) |
| `raf_source::test_crate::exported_items::an_exported_macro_is_exported_and_a_local_or_generated_one_is_not` | test | [Constraints](#constraints) |
| `raf_source::test_crate::exported_items::an_item_and_a_carried_outside_re_export_of_one_name_are_both_listed_and_not_ambiguous` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::an_item_reached_only_by_a_pub_use_of_a_private_glob_binding_is_not_exported_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::doc_hidden_items_are_left_out_by_default_and_flagged_when_included` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::exported_items::enum_variants_and_struct_fields_are_not_items_of_the_surface` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::in_random_small_crates_a_carried_re_export_is_listed_hidden_or_absent_as_the_item_is` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::of_two_carriers_of_one_name_only_the_one_with_a_free_route_is_listed_with_its_own_line` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::only_a_route_that_carries_a_name_out_decides_whether_it_is_hidden_for_items_and_re_exports` | test | [Constraints](#constraints) |
| `raf_source::test_crate::exported_items::searches_for_names_that_several_binders_keep_from_a_long_route_are_refused_by_the_item_budget` | test | [Constraints](#constraints) |
| `raf_source::test_crate::exported_items::test_code_is_not_part_of_the_surface_beside_the_shipped_items` | test | [Constraints](#constraints) |
| `raf_source::test_crate::exported_items::the_example_of_the_contract_lists_one_item_and_more_through_a_glob_or_a_pub_module` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::the_members_of_a_pub_trait_are_exported_and_those_of_a_private_trait_are_not` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::the_pub_methods_of_a_pub_type_are_exported_and_those_of_a_private_type_are_not` | test | [Verification](#verification) |
| `raf_source::test_crate::exported_items::the_surface_is_the_same_in_every_load_and_in_every_order_of_the_declarations` | test | [Constraints](#constraints) |
| `raf_source::test_crate::exported_items::twenty_thousand_chained_re_exports_are_listed_on_a_two_mib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::format_reach::a_cfg_alternative_sibling_alias_does_not_move_the_declared_modules_prelude_attribute` | test | [Constraints](#constraints) |
| `raf_source::test_crate::format_reach::a_dependency_named_std_and_a_resolved_workspace_macro_are_not_the_sysroot` | test | [Contract](#contract) |
| `raf_source::test_crate::format_reach::a_parent_files_textual_macro_also_shadows_an_imported_standard_alias` | test | [Verification](#verification) |
| `raf_source::test_crate::format_reach::block_local_std_aliases_are_checked_at_the_imports_own_scope` | test | [Contract](#contract) |
| `raf_source::test_crate::format_reach::conditional_or_module_prelude_changes_do_not_disable_an_unrelated_sibling` | test | [Verification](#verification) |
| `raf_source::test_crate::format_reach::cooked_raw_unicode_keyword_and_count_bindings_use_decoded_names` | test | [Contract](#contract) |
| `raf_source::test_crate::format_reach::direct_import_aliases_and_block_imports_are_positive_provenance` | test | [Contract](#contract) |
| `raf_source::test_crate::format_reach::every_admitted_enclosing_standard_macro_accepts_nested_format_expressions` | test | [Constraints](#constraints) |
| `raf_source::test_crate::format_reach::implicit_captures_and_named_counts_are_value_edges_of_the_actual_function` | test | [Contract](#contract) |
| `raf_source::test_crate::format_reach::malformed_standard_format_input_is_actionable_while_a_custom_macro_keeps_its_grammar` | test | [Contract](#contract) |
| `raf_source::test_crate::format_reach::named_bindings_suppress_capture_and_left_hand_path_without_hiding_the_expression` | test | [Contract](#contract) |
| `raf_source::test_crate::format_reach::nested_standard_expression_macros_are_transparent_but_custom_token_inputs_are_not_calls` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::format_reach::one_physical_file_is_recognized_per_target_module_edition_and_root_order` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::format_reach::prelude_changes_and_std_head_shadows_refuse_inferred_captures` | test | [Contract](#contract) |
| `raf_source::test_crate::format_reach::recursive_format_expressions_are_admitted_before_parsing_and_custom_tokens_stay_opaque` | test | [Contract](#contract) |
| `raf_source::test_crate::format_reach::textual_macro_shadows_begin_after_the_declaration_and_do_not_leak_from_a_block` | test | [Contract](#contract) |
| `raf_source::test_crate::format_reach::unknown_named_and_glob_imports_never_become_standard_macro_certificates` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::fs_confinement::only_files_of_the_disk_backend_are_exempt` | test | [Verification](#verification) |
| `raf_source::test_crate::fs_confinement::only_the_disk_backend_names_the_file_system_module` | test | [Verification](#verification) |
| `raf_source::test_crate::fs_confinement::the_scanner_finds_every_spelling_of_the_file_system_module` | test | [Verification](#verification) |
| `raf_source::test_crate::fs_confinement::the_scanner_leaves_neighbouring_paths_alone` | test | [Verification](#verification) |
| `raf_source::test_crate::git_tree::checked_snapshot_serves_real_bytes_and_existing_loader_with_distinct_path_identity` | test | [Contract](#contract) |
| `raf_source::test_crate::git_tree::committed_bytes_ignore_worktree_edits_and_replacement_refs` | test | [Contract](#contract) |
| `raf_source::test_crate::git_tree::malformed_tree_and_submodule_refuse_with_an_ordinary_tree_neighbor` | test | [Contract](#contract) |
| `raf_source::test_crate::git_tree::missing_commit_repository_and_wrong_object_kind_refuse_beside_the_exact_commit` | test | [Contract](#contract) |
| `raf_source::test_crate::git_tree::missing_installed_git_is_an_actionable_dependency_failure` | test | [Contract](#contract) |
| `raf_source::test_crate::git_tree::missing_or_substituted_blob_is_a_hard_failure_even_when_seen_would_skip_text` | test | [Contract](#contract) |
| `raf_source::test_crate::glob_causality::a_cycle_with_an_independent_carrier_route_keeps_its_ambiguity` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::a_final_glob_fallback_waits_for_an_outside_named_cycle` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::a_forwarded_private_import_preserves_public_peer_namespace_exports` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::a_grounded_free_namespace_releases_only_proven_empty_namespaces` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::a_private_alias_retains_a_returning_public_type` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::a_private_cut_does_not_erase_a_visible_ambiguity` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::a_private_cut_does_not_hide_a_later_visible_candidate` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::a_private_cut_does_not_waive_an_unclosed_sibling_branch` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::a_private_cut_finalizes_a_module_prefix_without_waiving_its_peer_cycle` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::a_private_cut_requires_a_closed_glob_module_prefix` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::a_private_forwarded_binding_hides_the_same_glob_item` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::a_private_glob_chain_does_not_fabricate_a_unique_item` | test | [Examples](#examples) |
| `raf_source::test_crate::glob_causality::a_private_glob_cut_retains_the_named_shadow_of_a_returning_namespace` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::a_private_independent_supporter_and_a_returning_cycle_keep_the_hidden_route` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::a_returning_export_ignores_only_independent_routes_below_its_visibility` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::a_settled_returning_named_peer_does_not_invent_a_namespace_export` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::a_value_import_does_not_erase_the_type_ambiguity_waiting_behind_it` | test | [Contract](#contract) |
| `raf_source::test_crate::glob_causality::a_visible_route_waiting_beside_a_hidden_arrival_keeps_the_ambiguity` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::an_actual_inaccessible_cut_needs_a_closed_local_candidate` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::an_ambiguous_original_import_is_refused_before_oracle_sanitization` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::an_independent_glob_does_not_shadow_an_inherited_ambiguity` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::direct_private_sources_and_missing_public_peers_do_not_export_glob_values` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::matching_private_imports_preserve_unaliased_glob_families` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::named_imports_preserve_companions_instead_of_promoting_them_to_independent_routes` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::private_forwarding_preserves_distinct_glob_ambiguity` | test | [Verification](#verification) |
| `raf_source::test_crate::glob_causality::rustc_refuses_the_named_import_of_the_ambiguous_type` | test | [Verification](#verification) |
| `raf_source::test_crate::import_determinacy::a_direct_visible_definition_makes_the_neighboring_import_determinate` | test | [Contract](#contract), [Examples](#examples) |
| `raf_source::test_crate::import_determinacy::a_fixed_module_prefix_decides_absence_before_an_unrelated_glob_widens_a_name` | test | [Verification](#verification) |
| `raf_source::test_crate::import_determinacy::a_late_bound_peer_cannot_escape_through_named_and_glob_copies_or_exports` | test | [Contract](#contract) |
| `raf_source::test_crate::import_determinacy::a_late_namespace_refuses_the_whole_import_including_its_bound_peer` | test | [Verification](#verification) |
| `raf_source::test_crate::import_determinacy::a_named_import_reading_back_through_its_module_is_not_a_settled_expression` | test | [Verification](#verification) |
| `raf_source::test_crate::import_determinacy::a_private_type_shadow_does_not_poison_its_independent_value_glob` | test | [Verification](#verification) |
| `raf_source::test_crate::import_determinacy::late_visibility_refusals_and_their_neighbors_are_stable_across_declaration_orders` | test | [Verification](#verification) |
| `raf_source::test_crate::index_disk::a_reload_observes_changed_source_while_the_previous_workspace_keeps_its_snapshot` | test | [Constraints](#constraints) |
| `raf_source::test_crate::index_disk::disk_and_memory_trees_give_equal_workspaces_and_equal_resolutions` | test | [Contract](#contract) |
| `raf_source::test_crate::load_budgets::a_chain_of_files_that_each_declare_the_next_twice_is_charged_what_it_keeps` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_chain_of_globs_is_charged_what_each_link_brings_in` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_doc_comment_counts_as_the_tokens_of_its_attribute_and_other_comments_count_nothing` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_file_at_the_token_limit_parses_and_one_more_token_is_refused_naming_its_line` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_file_reached_by_many_module_paths_is_charged_once_per_path_and_refused_over_the_item_budget` | test | [Contract](#contract) |
| `raf_source::test_crate::load_budgets::a_long_alias_path_in_a_file_reached_many_times_is_held_and_charged_once` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_long_extern_crate_name_in_a_file_reached_many_times_is_held_and_charged_once` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_long_module_name_is_charged_in_every_item_path_and_refused_over_the_item_budget` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_module_that_many_modules_glob_is_kept_once_for_all_of_them` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::a_verbatim_use_group_and_a_bodiless_function_are_charged_as_their_plain_forms` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::an_impl_block_is_charged_its_record_and_what_it_holds` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::each_costly_shape_at_the_token_limit_loads_and_one_more_token_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::each_large_fact_payload_is_charged_and_refused_at_its_line_once_over_the_fact_budget` | test | [Contract](#contract) |
| `raf_source::test_crate::load_budgets::glob_table_work_at_the_work_budget_loads_and_one_step_less_is_refused` | test | [Contract](#contract) |
| `raf_source::test_crate::load_budgets::glob_tables_at_the_glob_budget_load_and_one_byte_less_is_refused` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::load_budgets::glob_tables_of_a_generated_binding_crate_are_charged_what_they_keep` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::index_records_at_the_item_budget_load_and_one_byte_less_is_refused` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::load_budgets::modules_that_glob_one_another_are_refused_over_the_glob_budget_naming_a_glob` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::nested_definition_emission_spills_with_only_the_item_budget` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::nested_inline_modules_whose_scopes_hold_one_name_are_charged_what_they_keep` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::repeated_globs_of_one_module_are_charged_once_for_what_it_holds` | test | [Verification](#verification) |
| `raf_source::test_crate::load_budgets::retained_import_reads_and_blocker_cycles_obey_exact_storage_and_work_limits` | test | [Explanation](#explanation) |
| `raf_source::test_crate::load_budgets::the_bytes_read_at_the_total_budget_load_and_one_byte_less_is_refused_naming_the_file` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::load_budgets::the_default_budgets_are_the_stated_limits` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::load_budgets::the_fact_budget_counts_over_every_file_of_the_load` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::load_budgets::usage_reports_what_a_load_spent` | test | [Contract](#contract) |
| `raf_source::test_crate::load_budgets::wide_discovery_and_member_frontiers_spill_under_the_item_budget` | test | [Verification](#verification) |
| `raf_source::test_crate::load_cap::a_cap_larger_than_the_default_admits_a_file_the_default_refuses` | test | [Verification](#verification) |
| `raf_source::test_crate::load_cap::a_library_root_of_exactly_the_cap_loads_and_one_byte_more_is_refused_naming_it` | test | [Verification](#verification) |
| `raf_source::test_crate::load_cap::a_lowered_cap_applies_to_a_child_file_and_spares_a_root_within_it` | test | [Verification](#verification) |
| `raf_source::test_crate::load_cap::a_module_file_of_exactly_the_cap_loads_and_one_byte_more_is_refused_naming_it` | test | [Verification](#verification) |
| `raf_source::test_crate::load_cap::the_memory_tree_used_here_reads_the_padded_file_at_its_exact_size` | test | [Verification](#verification) |
| `raf_source::test_crate::locked_catalog::encoded_git_references_keep_native_decoded_identity_and_reference_kind` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_catalog::malformed_unsupported_and_stale_evidence_has_a_valid_profile_neighbor` | test | [Contract](#contract), [Examples](#examples) |
| `raf_source::test_crate::locked_catalog::native_catalog_storage_and_comparison_work_obey_exact_original_grants` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_catalog::omitted_git_precision_is_not_a_wildcard_beside_the_full_revision` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_catalog::same_name_version_sources_require_unique_complete_selection_in_either_row_order` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_catalog::the_actual_repository_lock_resolves_all_outgoing_edges_and_native_source_precision` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_catalog::unicode_checksum_refuses_without_panicking_beside_native_ascii_hex` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_selection::absent_mismatched_and_malformed_registry_configuration_refuses_beside_valid_indexes` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_selection::git_reference_kind_and_local_source_family_are_constraints_not_tree_proofs` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_selection::malformed_observations_and_unknown_owners_refuse_without_an_alternate_identity` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_selection::rename_version_and_registry_select_only_the_owners_exact_outgoing_row` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Explanation](#explanation) |
| `raf_source::test_crate::locked_selection::selection_honors_work_and_retained_catalog_item_admission` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_selection::workspace_inheritance_selects_its_package_and_source_without_evaluating_cfg` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_sources::a_nested_same_name_and_version_cannot_replace_the_root_workspace_package` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_sources::actual_version_evidence_is_required_without_changing_ordinary_missing_version_loading` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_sources::all_snapshots_catalog_and_walker_use_one_original_resource_grant` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_sources::another_checked_repository_cannot_supply_an_expected_source_by_name_or_authority` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_sources::checked_contents_do_not_let_wrong_manifest_names_or_versions_acquire_a_row` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_sources::exact_sources_versions_and_path_edges_resolve_aliases_to_the_actual_package` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_sources::explicit_first_party_roots_require_every_discovered_manifest_association` | test | [Contract](#contract), [Usage](#usage) |
| `raf_source::test_crate::locked_sources::requested_missing_and_optional_edges_refuse_and_unrequested_edges_never_borrow_a_name` | test | [Contract](#contract) |
| `raf_source::test_crate::manifest_caps::distinct_patterns_at_both_caps_load_with_no_tree_calls_for_the_excludes` | test | [Verification](#verification) |
| `raf_source::test_crate::manifest_caps::duplicate_patterns_within_the_caps_load_with_the_tree_calls_of_one_of_each` | test | [Verification](#verification) |
| `raf_source::test_crate::manifest_caps::exclude_at_the_cap_loads_and_one_over_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::manifest_caps::exclude_still_removes_literal_wildcard_and_nested_members` | test | [Verification](#verification) |
| `raf_source::test_crate::manifest_caps::members_at_the_cap_loads_and_one_over_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::manifest_caps::the_duplicate_pattern_probe_is_refused_before_any_pattern_is_expanded` | test | [Verification](#verification) |
| `raf_source::test_crate::memory_tree::a_directory_is_not_readable_as_a_file_and_a_file_is_not_listable` | test | [Contract](#contract) |
| `raf_source::test_crate::memory_tree::a_file_reads_back_and_a_missing_file_is_not_found` | test | [Contract](#contract) |
| `raf_source::test_crate::memory_tree::a_key_that_is_both_a_file_and_a_directory_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::memory_tree::a_parent_segment_is_applied_to_the_trees_own_directories` | test | [Verification](#verification) |
| `raf_source::test_crate::memory_tree::a_tree_built_infallibly_from_a_bad_key_reports_the_error_from_every_call` | test | [Contract](#contract) |
| `raf_source::test_crate::memory_tree::a_workspace_loads_through_an_object_safe_tree_with_caller_local_state` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_source::test_crate::memory_tree::collecting_pairs_builds_the_same_tree_as_the_array_conversion` | test | [Contract](#contract) |
| `raf_source::test_crate::memory_tree::directories_are_inferred_and_listings_are_sorted_with_their_kinds` | test | [Contract](#contract) |
| `raf_source::test_crate::memory_tree::duplicate_keys_are_refused_literally_and_after_normalisation_beside_distinct_keys` | test | [Contract](#contract) |
| `raf_source::test_crate::memory_tree::entry_reports_file_dir_and_absence_without_an_error` | test | [Contract](#contract) |
| `raf_source::test_crate::memory_tree::keys_that_leave_the_root_are_refused_with_outside_root_beside_a_valid_key` | test | [Contract](#contract) |
| `raf_source::test_crate::memory_tree::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen` | test | [Contract](#contract) |
| `raf_source::test_crate::memory_tree::read_and_unseen_read_apply_the_exact_byte_limit_beside_a_seen_identity` | test | [Contract](#contract) |
| `raf_source::test_crate::memory_tree::the_identity_of_a_file_is_its_normalised_key_and_refuses_what_read_refuses` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::module_tree::a_cfg_attr_test_path_is_tagged_test_and_nothing_is_dropped` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::a_cfg_test_module_tags_every_item_reached_only_through_it` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::a_chain_of_files_that_each_declare_the_next_twice_is_refused_by_the_module_path_budget_opening_each_file_once_beside_a_small_diamond` | test | [Constraints](#constraints) |
| `raf_source::test_crate::module_tree::a_child_file_that_sorts_before_its_mod_rs_parent_is_placed_under_its_own_path` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::a_file_module_declared_inside_an_inline_module_is_placed_under_the_inline_module` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::a_file_named_by_a_path_attribute_is_a_mod_rs_file_for_its_children` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::a_file_reached_twice_without_a_cycle_is_parsed_once_and_recorded_under_each_path` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::a_file_reached_under_two_spellings_is_read_once_and_keeps_the_spelling_that_sorts_first` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::a_flat_file_and_a_directory_module_are_both_followed_beside_each_other` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::a_flat_module_file_and_its_child_below_a_parent_that_sorts_first_keep_their_own_paths` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::a_malformed_cfg_path_or_test_attribute_is_an_error_naming_its_line` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::a_module_cycle_is_an_error_naming_both_files` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::a_module_with_both_name_rs_and_name_mod_rs_is_an_error_naming_both_files` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::a_nested_cfg_attr_path_is_flattened_with_the_combined_predicate` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::a_non_literal_path_attribute_is_reported_as_such_from_the_attribute` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::a_non_mod_file_keeps_its_children_in_a_directory_of_its_own_name` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::a_path_attribute_in_a_mod_rs_file_is_relative_to_that_files_directory` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::a_path_attribute_in_a_non_mod_file_is_relative_to_the_files_directory_not_its_module_directory` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::a_path_attribute_inside_an_inline_module_is_relative_to_the_module_directory` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::a_path_attribute_on_an_inline_module_in_a_non_mod_file_replaces_its_directory_component` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::a_path_attribute_on_an_inline_module_replaces_its_directory_component` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::a_path_that_leaves_the_tree_is_refused_naming_module_and_attribute_without_reading_it` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::a_raw_identifier_module_reads_the_plain_file_name` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::a_shared_test_module_reached_from_a_target_that_sorts_after_it_keeps_its_own_paths` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::an_inline_module_inside_a_non_mod_file_nests_below_the_files_own_directory` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::an_item_is_found_only_under_its_own_module_path` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::an_unparseable_cfg_is_an_error_naming_file_and_line_beside_a_parseable_one` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::an_unresolved_mod_names_the_declaring_file_and_line` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::every_cfg_attr_path_alternative_is_followed_carrying_its_predicate` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::inline_modules_add_directory_components_for_their_file_children` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::inline_modules_with_path_alternatives_are_bounded_by_the_module_path_budget` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::module_depth_of_exactly_the_budget_loads_and_one_more_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::module_nesting_beyond_the_depth_budget_is_refused_and_the_budget_itself_loads` | test | [Constraints](#constraints) |
| `raf_source::test_crate::module_tree::one_file_spelled_two_ways_is_one_file_recorded_under_both_module_paths` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::super_super_from_a_child_that_sorts_before_its_parent_climbs_to_the_crate_root` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::the_cfg_test_matrix_tags_items_and_leaves_their_neighbours_alone` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::the_children_of_a_path_attribute_file_are_placed_under_its_module_whatever_their_names` | test | [Verification](#verification) |
| `raf_source::test_crate::module_tree::the_default_file_is_also_followed_beside_cfg_attr_alternatives_when_it_exists` | test | [Contract](#contract) |
| `raf_source::test_crate::module_tree::two_unconditional_path_attributes_are_an_error_beside_one` | test | [Contract](#contract) |
| `raf_source::test_crate::overflow_harness::a_child_that_exits_three_without_a_result_line_is_a_failure` | test | [Verification](#verification) |
| `raf_source::test_crate::overflow_harness::a_child_that_panics_is_a_failure_and_not_an_abort` | test | [Verification](#verification) |
| `raf_source::test_crate::overflow_harness::a_child_with_an_unknown_selector_is_a_failure` | test | [Verification](#verification) |
| `raf_source::test_crate::overflow_harness::a_deliberate_overflow_aborts_and_the_runtime_names_the_parse_thread` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::overflow_harness::a_panic_in_the_visitor_is_raised_again_with_its_payload` | test | [Contract](#contract) |
| `raf_source::test_crate::overflow_harness::a_return_chain_that_needs_twice_the_stack_is_refused_beside_one_at_a_safe_depth` | test | [Verification](#verification) |
| `raf_source::test_crate::overflow_harness::a_shape_just_inside_its_budgets_exits_zero_with_the_exact_symbols` | test | [Contract](#contract) |
| `raf_source::test_crate::overflow_harness::child_main` | test | [Verification](#verification) |
| `raf_source::test_crate::overflow_harness::deep_but_admitted_source_parses_on_the_dedicated_stack_not_the_callers` | test | [Contract](#contract) |
| `raf_source::test_crate::overflow_harness::every_committed_overflow_shape_is_refused_by_a_budget_and_none_aborts` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::overflow_harness::every_shape_family_loads_at_its_largest_admitted_size_and_one_size_more_is_refused` | test | [Contract](#contract) |
| `raf_source::test_crate::overflow_harness::the_classifier_accepts_only_a_signal_a_stack_overflow_status_or_the_runtime_message_as_an_abort` | test | [Verification](#verification) |
| `raf_source::test_crate::overflow_harness::the_judge_rejects_a_wrong_index_a_misnamed_refusal_an_abort_and_a_stray_failure` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::overflow_harness::the_parse_thread_is_named_after_the_file_and_has_the_stated_stack` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::a_block_as_the_condition_of_each_else_if_still_counts_the_chain` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::a_comment_or_string_that_looks_like_the_end_of_another_does_not_hide_real_tokens` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::a_construct_that_continues_past_a_comma_or_a_closing_brace_keeps_its_count` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::a_label_a_use_path_separator_and_each_delimiter_count_at_the_limit_and_one_over` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::a_level_returns_to_zero_where_its_constructs_end_beside_the_same_count_without_the_end` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::a_lifetime_between_reference_marks_does_not_end_the_run` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::a_lifetime_is_not_the_start_of_a_character_literal` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::a_macro_call_counts_only_its_groups_and_a_match_pattern_only_its_alternatives` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::a_syntax_error_reports_its_own_line_and_column_beside_a_parsing_neighbour` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::chains_that_overflowed_the_parse_stack_are_refused_by_the_recursion_budget` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::prescan_budgets::closure_heads_with_parameters_chain_like_empty_ones` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::delimiter_nesting_is_refused_at_one_hundred_twenty_nine_beside_one_hundred_twenty_eight` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::prescan_budgets::every_counted_keyword_counts_one_at_the_limit_and_one_over` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::every_operator_counts_one_for_each_of_its_characters_at_the_limit_and_one_over` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::five_hundred_nested_generics_are_refused_beside_sixty_four` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::four_thousand_bangs_are_refused_beside_two_hundred_fifty_six` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::neighbouring_valid_shapes_just_below_each_budget_parse` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::prescan_budgets::separate_if_chains_in_one_block_do_not_add_up_beside_the_one_long_chain` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::ten_thousand_closure_bars_are_refused_beside_one_hundred_twenty_eight` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::the_eighty_kilobyte_box_pattern_is_refused_by_the_pre_scan_before_the_parser_runs` | test | [Verification](#verification) |
| `raf_source::test_crate::prescan_budgets::the_recursion_limit_is_half_the_parse_stack_over_the_measured_cost_of_one_unit` | test | [Constraints](#constraints) |
| `raf_source::test_crate::prescan_budgets::the_recursion_refusal_names_the_line_where_the_count_went_over` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::the_refusal_names_the_line_where_the_count_went_over` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::tokens_inside_strings_raw_strings_and_comments_count_for_nothing` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::twenty_thousand_else_if_links_are_refused_beside_two_hundred_fifty_six` | test | [Contract](#contract) |
| `raf_source::test_crate::prescan_budgets::twenty_thousand_reference_marks_are_refused` | test | [Contract](#contract) |
| `raf_source::test_crate::reachability::a_bound_module_covers_its_nested_items_and_impl_blocks_and_a_crate_root_nothing` | test | [Verification](#verification) |
| `raf_source::test_crate::reachability::a_bound_type_covers_its_impl_blocks_and_their_members_and_a_bound_method_nothing_beside_it` | test | [Verification](#verification) |
| `raf_source::test_crate::reachability::a_cfg_test_module_is_reached_like_any_module_and_its_tests_reach_their_helpers` | test | [Verification](#verification) |
| `raf_source::test_crate::reachability::a_file_with_a_byte_order_mark_and_crlf_lines_reaches_as_a_plain_one_does` | test | [Verification](#verification) |
| `raf_source::test_crate::reachability::a_member_is_read_from_the_module_of_its_block_and_a_trait_member_from_its_trait` | test | [Verification](#verification) |
| `raf_source::test_crate::reachability::a_method_call_a_field_a_lifetime_and_a_local_name_no_item_and_a_path_prefix_names_its_type` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::reachability::a_module_named_by_a_path_and_then_reached_by_contains_is_opened_then` | test | [Constraints](#constraints) |
| `raf_source::test_crate::reachability::a_path_the_resolver_refuses_reaches_nothing_beside_one_it_resolves` | test | [Verification](#verification) |
| `raf_source::test_crate::reachability::a_path_through_a_type_names_the_type_and_the_member_and_a_leading_colon_names_a_crate` | test | [Verification](#verification) |
| `raf_source::test_crate::reachability::a_pointer_to_a_type_and_a_raw_borrow_of_a_static_reach_them_and_a_constant_its_type` | test | [Verification](#verification) |
| `raf_source::test_crate::reachability::a_private_helper_called_by_a_bound_item_is_reached_and_an_unreferenced_one_is_not` | test | [Contract](#contract) |
| `raf_source::test_crate::reachability::a_root_past_the_items_or_the_blocks_of_the_workspace_is_refused_beside_one_in_range` | test | [Contract](#contract) |
| `raf_source::test_crate::reachability::a_shared_file_reads_keywords_in_each_reached_packages_edition` | test | [Contract](#contract) |
| `raf_source::test_crate::reachability::a_variant_through_self_names_its_enum_as_a_path_through_the_enum_does` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::reachability::a_walk_past_the_item_bytes_budget_is_refused_and_one_within_it_is_given` | test | [Contract](#contract) |
| `raf_source::test_crate::reachability::an_edition_2015_body_import_is_read_from_its_module_beside_an_explicit_crate_path` | test | [Constraints](#constraints) |
| `raf_source::test_crate::reachability::an_impl_block_root_reaches_its_members_and_what_its_header_names` | test | [Verification](#verification) |
| `raf_source::test_crate::reachability::an_item_a_macro_generates_is_not_reached_and_a_macro_named_by_a_reached_item_is` | test | [Constraints](#constraints) |
| `raf_source::test_crate::reachability::gen_is_a_name_before_edition_2024_and_a_keyword_from_it` | test | [Contract](#contract) |
| `raf_source::test_crate::reachability::impls_inside_const_and_function_blocks_are_tokens_beside_an_indexed_top_level_impl` | test | [Constraints](#constraints) |
| `raf_source::test_crate::reachability::nested_impl_self_is_read_in_the_enclosing_members_context` | test | [Constraints](#constraints) |
| `raf_source::test_crate::reachability::paths_inside_tests_and_binaries_reach_their_own_crate_and_the_library` | test | [Verification](#verification) |
| `raf_source::test_crate::reachability::qualified_self_suffixes_are_not_member_edges_beside_an_unqualified_self_member` | test | [Constraints](#constraints) |
| `raf_source::test_crate::reachability::self_in_a_trait_and_in_its_impl_blocks_reaches_the_members_of_the_trait_and_of_the_block` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::reachability::self_paths_and_methods_called_on_self_reach_the_members_of_that_name` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::reachability::test_code_names_the_dev_dependencies_and_other_code_does_not` | test | [Verification](#verification) |
| `raf_source::test_crate::reachability::the_answer_does_not_depend_on_the_order_of_the_roots_and_is_the_same_each_run` | test | [Contract](#contract) |
| `raf_source::test_crate::reachability::the_type_a_where_clause_bounds_is_reached_and_a_bound_name_is_not` | test | [Verification](#verification) |
| `raf_source::test_crate::read_cap::a_zero_cap_refuses_any_content_and_accepts_an_empty_file` | test | [Verification](#verification) |
| `raf_source::test_crate::read_cap::disk_tree_enforces_the_cap_at_eight_mib` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::read_cap::memory_tree_enforces_the_cap_at_eight_mib` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::read_cap::the_default_cap_is_eight_mib` | test | [Contract](#contract) |
| `raf_source::test_crate::registry_scan::tests::child_arguments_carry_the_directory_and_the_limit` | test | [Verification](#verification) |
| `raf_source::test_crate::registry_scan::tests::count_callers_preserve_padding_width_suffixes_and_misplaced_totals` | test | [Verification](#verification) |
| `raf_source::test_crate::registry_scan::tests::ok_line_prints_work_last_among_the_usage_figures` | test | [Verification](#verification) |
| `raf_source::test_crate::registry_scan::tests::tally_keeps_the_largest_work_and_counts_budget_refusals` | test | [Verification](#verification) |
| `raf_source::test_crate::registry_scan::tests::the_child_is_given_the_limit_it_was_asked_for` | test | [Verification](#verification) |
| `raf_source::test_crate::registry_scan::tests::the_work_limit_reaches_the_load` | test | [Verification](#verification) |
| `raf_source::test_crate::registry_scan::tests::version_components_and_prerelease_precedence_keep_their_width_and_order` | test | [Verification](#verification) |
| `raf_source::test_crate::registry_scan::tests::work_limit_refuses_zero_and_names_the_flag` | test | [Verification](#verification) |
| `raf_source::test_crate::relpath::a_parent_segment_above_the_root_is_refused_beside_the_one_that_stays_inside` | test | [Contract](#contract) |
| `raf_source::test_crate::relpath::a_parent_segment_below_the_root_is_kept_for_the_disk_to_resolve` | test | [Contract](#contract) |
| `raf_source::test_crate::relpath::an_absolute_input_is_refused_beside_its_relative_form` | test | [Contract](#contract) |
| `raf_source::test_crate::relpath::backslash_nul_and_drive_letter_inputs_are_refused_beside_plain_names` | test | [Contract](#contract) |
| `raf_source::test_crate::relpath::display_names_the_root_as_a_dot` | test | [Verification](#verification) |
| `raf_source::test_crate::relpath::dot_and_empty_segments_are_dropped` | test | [Contract](#contract) |
| `raf_source::test_crate::relpath::join_extends_a_path_and_checks_the_climb_against_the_whole` | test | [Contract](#contract) |
| `raf_source::test_crate::relpath::parent_and_file_name_are_lexical` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::a_cycle_is_reported_before_a_glob_that_names_nothing` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::a_cycle_of_globs_stays_cyclic_beside_globs_that_resolve` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::a_glob_of_a_missing_local_module_loads_and_fails_each_lookup_that_reads_it` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::a_glob_the_index_cannot_look_into_leaves_a_lookup_not_found` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::a_module_of_a_crate_name_two_libraries_share_is_ambiguous_to_a_reader_as_to_a_resolve` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::a_module_that_is_not_in_the_workspace_and_a_text_that_is_no_path_are_refused` | test | [Contract](#contract) |
| `raf_source::test_crate::resolve_paths::a_name_found_through_a_glob_beside_a_glob_that_names_nothing_fails` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::a_path_keyword_in_any_position_is_invalid_beside_the_canonical_path` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::a_path_that_starts_with_a_name_two_globs_bring_in_as_different_modules_is_ambiguous_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::resolve_paths::a_path_that_starts_with_a_name_two_globs_bring_in_as_one_module_still_resolves_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::resolve_paths::a_use_path_counts_only_what_the_reading_module_can_access` | test | [Contract](#contract) |
| `raf_source::test_crate::resolve_paths::a_use_path_is_read_in_one_name_space_or_in_each_in_turn` | test | [Contract](#contract) |
| `raf_source::test_crate::resolve_paths::an_import_that_starts_with_a_name_two_globs_bring_in_as_different_modules_is_refused_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::definitions_and_imports_of_one_name_under_cfg_are_alternatives_at_any_segment_of_a_path` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::globs_of_dependencies_in_one_module_leave_a_missing_name_not_found` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::the_cfg_alternatives_of_one_module_a_glob_brings_in_are_one_candidate_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::resolve_paths::two_different_modules_of_one_name_stay_ambiguous_beside_the_cfg_alternatives_of_one` | test | [Verification](#verification) |
| `raf_source::test_crate::root::explicit_workspace_requires_a_safe_path_and_agreed_membership` | test | [Contract](#contract) |
| `raf_source::test_crate::root::malformed_manifests_and_missing_directories_fail_hard` | test | [Contract](#contract) |
| `raf_source::test_crate::root::manifest_size_bound_is_enforced_beside_the_exact_limit` | test | [Contract](#contract) |
| `raf_source::test_crate::root::member_and_exclude_lists_accept_4096_entries_and_refuse_one_more` | test | [Constraints](#constraints) |
| `raf_source::test_crate::root::member_and_virtual_workspace_directories_resolve_without_sources` | test | [Contract](#contract) |
| `raf_source::test_crate::root::standalone_excluded_and_nested_packages_have_their_own_roots` | test | [Contract](#contract) |
| `raf_source::test_crate::root::transitive_path_dependencies_are_members` | test | [Contract](#contract) |
| `raf_source::test_crate::rustc_oracle::a_divergence_that_matches_no_class_is_unclassified` | test | [Verification](#verification) |
| `raf_source::test_crate::rustc_oracle::a_probe_rustc_answers_by_the_order_is_not_order_when_the_index_does_too` | test | [Constraints](#constraints) |
| `raf_source::test_crate::rustc_oracle::a_wrong_index_answer_is_reported_beside_the_right_one_which_is_not` | test | [Verification](#verification) |
| `raf_source::test_crate::rustc_oracle::an_index_answer_that_changes_with_the_order_is_reported_as_index_order` | test | [Constraints](#constraints) |
| `raf_source::test_crate::rustc_oracle::diagnostic_levels_are_required_beside_a_valid_null_code` | test | [Verification](#verification) |
| `raf_source::test_crate::rustc_oracle::every_corpus_program_shows_the_divergence_it_records_and_no_other` | test | [Constraints](#constraints) |
| `raf_source::test_crate::rustc_oracle::the_index_agrees_with_rustc_on_generated_programs_up_to_the_known_classes` | test | [Constraints](#constraints) |
| `raf_source::test_crate::rustc_oracle::the_minimiser_drops_what_a_divergence_does_not_need_and_keeps_its_class` | test | [Verification](#verification) |
| `raf_source::test_crate::rustc_oracle::the_seed_base_is_read_in_hexadecimal_or_decimal_and_refuses_anything_else` | test | [Verification](#verification) |
| `raf_source::test_crate::rustc_oracle::write_corpus` | test | [Verification](#verification) |
| `raf_source::test_crate::source_text::a_budget_refusal_in_a_crlf_file_names_the_same_line_as_in_an_lf_file` | test | [Verification](#verification) |
| `raf_source::test_crate::source_text::a_byte_order_mark_leaves_the_lines_of_the_text_where_they_are` | test | [Verification](#verification) |
| `raf_source::test_crate::source_text::a_byte_order_mark_on_the_first_line_starts_the_span_of_its_item_after_the_mark` | test | [Verification](#verification) |
| `raf_source::test_crate::source_text::a_module_file_that_is_not_utf8_is_refused_on_disk_naming_the_file` | test | [Verification](#verification) |
| `raf_source::test_crate::source_text::a_one_line_item_has_the_half_open_lines_n_to_n_plus_one` | test | [Contract](#contract) |
| `raf_source::test_crate::source_text::a_syntax_error_in_a_crlf_file_or_after_a_byte_order_mark_names_its_own_line` | test | [Verification](#verification) |
| `raf_source::test_crate::source_text::crlf_line_endings_give_the_same_lines_and_spans_as_lf` | test | [Verification](#verification) |
| `raf_source::test_crate::spawn_limit::a_parse_thread_the_system_refuses_a_stack_is_a_spawn_error_naming_the_file` | test | [Contract](#contract) |
| `raf_source::test_crate::spawn_limit::child_main` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::a_bitflags_style_macro_type_with_an_impl_loads_with_its_members_marked_unresolved` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::a_bodiless_const_or_another_verbatim_item_is_still_a_typed_error_naming_file_and_line` | test | [Constraints](#constraints) |
| `raf_source::test_crate::symbol_index::a_file_reached_by_targets_of_two_packages_belongs_to_the_first_target_in_target_order` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::a_first_party_test_crate_module_and_a_test_target_are_ambiguous_only_where_their_paths_meet` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::a_glob_of_a_local_path_the_index_cannot_see_is_a_candidate_that_cannot_be_confirmed` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::a_glob_of_enum_variants_beside_an_impl_loads` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::a_method_in_another_file_than_its_type_is_recorded_under_the_type` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::a_module_a_function_and_a_macro_of_one_name_coexist_and_each_selects_by_namespace` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::a_module_in_its_own_file_is_documented_by_the_inner_docs_of_that_file` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::a_negative_impl_keeps_its_bang_in_the_trait_path_beside_a_positive_one` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::a_super_path_above_the_root_in_an_unresolved_self_type_is_still_an_error` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::an_impl_block_of_a_file_reached_under_two_module_paths_is_listed_once_for_each` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::an_impl_of_a_local_trait_object_loads_with_its_members_marked_unresolved` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::an_inherent_impl_on_a_type_the_index_cannot_see_is_indexed_under_its_written_self_path` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::an_initializer_is_the_text_between_the_equals_and_the_semicolon_beside_a_required_constant` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::an_item_the_index_does_not_model_is_a_typed_error_naming_file_and_line` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::an_unnamed_const_is_not_in_the_index_beside_a_named_one` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::bindings_and_an_initializer_are_shared_by_every_path_that_reaches_the_file` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::bodiless_functions_and_types_of_attribute_macros_are_indexed_where_they_are_written` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::symbol_index::cfg_twins_are_one_symbol_with_two_sorted_sites` | test | [Verification](#verification) |
| `raf_source::test_crate::symbol_index::crate_roots_have_no_source_group_even_when_the_first_item_starts_at_byte_zero` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::documentation_counts_text_and_include_str_but_not_hidden_or_blank_comments` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::every_impl_block_is_listed_in_file_order_with_its_span_module_self_type_key_and_members` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::every_kind_is_indexed_with_its_kind_line_file_and_visibility` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::files_items_in_and_item_at_answer_by_position` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::impl_members_are_recorded_under_the_type_or_under_the_trait_by_the_shape_of_the_impl` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::items_on_one_long_line_of_wide_characters_get_exact_byte_offsets` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::items_that_share_a_source_item_are_grouped_by_its_identity` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::items_under_matches_whole_path_segments` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::parameter_bindings_are_every_bound_name_in_signature_order_beside_a_function_with_none` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::read_only_resolution_keeps_workspace_equality_and_distinguishes_changed_definitions` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::shipping_is_the_target_and_not_a_test_role` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::test_functions_carry_the_package_target_and_crate_relative_name` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::test_roles_follow_the_module_tree_and_the_item` | test | [Contract](#contract) |
| `raf_source::test_crate::symbol_index::two_loads_are_equal_and_items_and_targets_come_in_strictly_increasing_order` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::symbol_index::unit_and_tuple_structs_are_also_values_and_named_structs_are_not` | test | [Contract](#contract) |
| `raf_source::test_crate::target_resolution::a_helper_resolves_from_a_nested_module_of_a_binary` | test | [Contract](#contract) |
| `raf_source::test_crate::target_resolution::a_helper_resolves_from_an_integration_test_root_and_from_a_module_below_it` | test | [Contract](#contract) |
| `raf_source::test_crate::target_resolution::a_module_of_the_library_at_the_crate_path_of_a_test_target_is_ambiguous_with_it` | test | [Contract](#contract) |
| `raf_source::test_crate::target_resolution::a_module_path_that_no_target_holds_is_unknown_whichever_kind_of_target_it_names` | test | [Contract](#contract) |
| `raf_source::test_crate::target_resolution::a_support_module_that_two_test_targets_include_resolves_in_each_target_alone` | test | [Contract](#contract) |
| `raf_source::test_crate::target_resolution::an_example_a_bench_and_the_build_script_are_read_from_as_well` | test | [Contract](#contract) |
| `raf_source::test_crate::target_roots::a_build_script_main_beside_a_lib_module_build_script_is_ambiguous` | test | [Verification](#verification) |
| `raf_source::test_crate::target_roots::a_first_party_module_at_a_binary_crate_path_is_ambiguous_beside_a_binary_that_is_not` | test | [Verification](#verification) |
| `raf_source::test_crate::target_roots::a_lib_module_named_bin_crate_stays_in_the_library` | test | [Verification](#verification) |
| `raf_source::test_crate::target_roots::an_explicit_path_bin_beside_a_lib_module_bin_crate_is_ambiguous_in_both_file_orders` | test | [Verification](#verification) |
| `raf_source::test_crate::target_roots::an_over_cap_inferred_lib_names_discovery_not_lib_path` | test | [Contract](#contract) |
| `raf_source::test_crate::target_roots::every_target_kind_resolves_at_its_crate_path_as_a_module_item_of_its_root_file` | test | [Contract](#contract) |
| `raf_source::test_crate::target_roots::the_namespace_segments_are_not_items_and_a_first_party_module_of_that_name_collides` | test | [Verification](#verification) |
| `raf_source::test_crate::target_roots::this_repository_resolves_a_test_crate_root_and_a_library_root` | test | [Verification](#verification) |
| `raf_source::test_crate::target_roots::unreadable_target_roots_name_explicit_paths_or_table_names_and_build_discovery` | test | [Contract](#contract) |
| `raf_source::test_crate::this_repository::a_dependency_re_exported_by_its_crate_name_is_not_found_and_not_a_cycle` | test | [Verification](#verification) |
| `raf_source::test_crate::this_repository::an_item_of_a_module_file_that_sorts_before_its_mod_rs_is_found_under_its_own_module` | test | [Verification](#verification) |
| `raf_source::test_crate::this_repository::the_reexported_syn_types_are_the_types_received_by_the_parse_visitor` | test | [Constraints](#constraints) |
| `raf_source::test_crate::this_repository::the_surface_of_each_library_and_binary_of_this_repository_lists_its_public_items` | test | [Verification](#verification) |
| `raf_source::test_crate::this_repository::this_repository_loads_from_its_root_with_the_crates_directory_as_the_root` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_dependency_package_that_is_not_a_string_is_an_error_naming_the_key` | test | [Contract](#contract) |
| `raf_source::test_crate::trait_keys::a_glob_of_something_that_is_no_crate_or_module_is_an_error_beside_a_declared_one` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_glob_through_a_re_exported_crate_is_keyed_under_the_crate` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_head_that_is_not_a_known_crate_is_an_error_and_a_declared_dependency_loads` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_literal_member_without_a_manifest_is_an_error_and_a_glob_skips_such_a_directory` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_macro_call_in_a_trait_body_is_skipped` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_malformed_ancestor_manifest_is_an_error_and_a_missing_one_is_not` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_package_that_is_only_a_path_dependency_of_the_root_package_is_a_member` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_path_dependency_inside_the_workspace_with_no_manifest_is_an_error_naming_the_key` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_path_dependency_outside_the_workspace_directory_or_excluded_is_not_a_member` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_path_dependency_with_an_unusable_path_is_an_error_beside_one_that_is_outside_the_tree` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_path_through_a_local_re_export_of_a_dependency_has_one_key_with_the_direct_spelling` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_pattern_outside_the_supported_syntax_is_a_manifest_error_naming_the_list` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_star_heavy_member_pattern_that_matches_no_directory_leaves_the_member_alone` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_trait_found_through_twenty_thousand_chained_globs_is_keyed_on_a_two_mib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_trait_name_that_two_globs_bind_keys_each_trait` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_trait_that_several_unknown_globs_could_give_is_refused_naming_one_glob_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_trait_through_a_glob_the_importer_cannot_access_is_keyed_at_its_definition_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_trait_through_a_head_a_glob_brings_in_from_another_crate_keys_both` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_trait_through_a_single_segment_import_of_a_crate_is_keyed_at_the_crate` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_trait_through_an_ambiguous_use_head_keys_every_reading` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_trait_through_one_dependency_glob_is_keyed_under_that_glob` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_use_path_of_edition_2015_is_followed_from_the_crate_root` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_use_path_of_edition_2015_names_a_dependency_only_through_extern_crate` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::a_workspace_dependency_whose_path_is_not_a_string_is_an_error_naming_the_key` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::an_ancestor_manifest_over_the_size_cap_is_a_typed_error` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::an_edition_that_is_not_a_year_is_a_manifest_error_naming_the_key` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::an_empty_pattern_segment_is_a_pattern_error_beside_a_plain_pattern` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::an_excluded_literal_member_is_ignored_and_not_required_to_exist` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::an_explicit_import_of_a_dependency_trait_shadows_a_glob_of_a_local_one` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::an_import_met_again_through_a_glob_adds_nothing_and_a_pure_cycle_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::an_inherited_dependency_the_workspace_cannot_give_is_an_error_naming_the_key` | test | [Contract](#contract) |
| `raf_source::test_crate::trait_keys::an_inherited_edition_comes_from_the_workspace_and_a_missing_one_is_a_manifest_error` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::cfg_alternatives_of_one_defining_trait_keep_one_target_identity` | test | [Contract](#contract) |
| `raf_source::test_crate::trait_keys::dependency_tables_are_honoured_by_target_kind_and_renames_use_the_name_in_code` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::each_prelude_trait_loads_in_the_edition_that_has_it_and_is_refused_in_the_one_before` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::inheriting_requires_membership_by_the_members_and_exclude_lists` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::local_traits_still_key_under_their_definition_beside_dependency_ones` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::nontrait_type_hits_supply_no_trait_evidence_beside_a_real_trait_alias` | test | [Contract](#contract) |
| `raf_source::test_crate::trait_keys::path_dependencies_of_members_are_members_transitively_in_every_dependency_table` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::several_dependency_globs_leave_the_trait_unresolved_and_the_members_are_still_found` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::the_completion_audit_shape_of_a_crate_alias_and_twenty_thousand_imports_loads_on_a_two_mib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::the_nearest_ancestor_with_a_workspace_table_decides_and_others_are_skipped` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::the_same_trait_through_a_glob_and_through_a_named_import_can_have_two_keys` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::the_workspace_manifest_is_found_above_the_roots_and_adds_no_package` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::the_workspace_root_package_inherits_without_being_listed` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::twenty_thousand_chained_local_aliases_load_on_a_two_mib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::trait_keys::without_a_workspace_above_or_an_edition_in_it_the_inheritance_is_a_manifest_error` | test | [Verification](#verification) |
| `raf_source::test_crate::tree_error::an_unconstructible_path_converts_to_outside_root_naming_the_input` | test | [Verification](#verification) |
| `raf_source::test_crate::tree_error::every_variant_names_its_path_in_display_and_in_path` | test | [Contract](#contract) |
| `raf_source::test_crate::tree_error::only_io_has_a_source_and_only_not_found_is_absence` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_cfg_twin_module_is_one_symbol_and_its_contents_merge` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_chain_of_two_re_exports_through_two_private_globs_is_capped_twice_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_companion_carried_through_an_independent_glob_chain_remains_ambiguous_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_crate_name_an_explicit_import_binds_is_never_read_as_the_crate_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_crate_name_in_an_import_is_the_dependency_its_target_declares` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_crate_root_inside_a_group_under_a_path_is_a_syntax_error` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_crate_visible_independent_route_keeps_another_globs_ambiguity_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_cycle_left_to_a_glob_reads_all_that_the_glob_brings_in_once_its_round_is_decided` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_cycle_of_explicit_imports_is_cyclic_whatever_its_globs_hold_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_cycle_of_imports_is_cyclic_beside_a_chain_that_ends` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_cycle_of_imports_through_a_crate_name_and_a_glob_is_cyclic_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_cycle_of_imports_with_one_glob_on_it_resolves_through_the_glob_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_cycle_resolves_through_its_glob_when_a_place_an_import_keeps_also_waits_on_a_glob` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_cycle_through_a_glob_and_a_renamed_import_is_decided_in_each_name_space_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_cycle_through_a_glob_the_importer_cannot_access_is_no_cycle_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_cycle_through_a_renamed_import_that_has_bound_nothing_is_cyclic_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_cycle_whose_import_reads_its_own_name_through_another_glob_is_cyclic_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_dev_dependency_is_named_only_by_test_code_even_under_a_normal_dependencys_name` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_glob_against_a_workspace_library_is_ambiguous_in_every_declaration_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_glob_brings_in_what_its_target_binds_later_whatever_its_target_holds_under_its_first_segment` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_glob_of_a_module_whose_name_is_ambiguous_carries_the_ambiguity_to_its_readers_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_glob_re_export_of_a_private_item_is_seen_no_wider_than_the_item_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_glob_resolves_with_via_glob_and_the_direct_path_does_not` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_glob_that_brings_in_an_import_of_another_crate_collides_with_the_crate` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_glob_the_importer_cannot_access_holds_nothing_back_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_glob_whose_own_path_starts_at_a_crate_is_ambiguous_with_a_crate_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_glob_whose_path_turns_out_ambiguous_brings_in_nothing_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_hidden_companion_arriving_beside_an_independent_route_remains_ambiguous_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_hidden_item_does_not_make_a_name_ambiguous_where_a_glob_cycle_delivers_the_item_held_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_hidden_module_a_glob_cycle_carries_back_beside_the_module_itself_is_no_ambiguity_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_leading_colon_member_of_a_use_group_reads_the_crate_root_in_edition_2015` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_long_chain_of_re_exports_resolves_in_either_order_without_a_cap` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_module_a_glob_cycle_carries_back_beside_a_glob_that_holds_it_from_elsewhere_starts_a_path_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_module_and_a_struct_of_one_name_through_two_globs_are_ambiguous_in_the_type_name_space_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_mutual_super_glob_resolves_a_real_item_in_both_directions` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_name_an_import_the_reader_cannot_access_binds_is_hidden_from_it_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_name_bound_to_several_modules_gives_a_trait_key_through_each` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_name_is_not_read_from_a_glob_while_an_explicit_import_of_it_is_unsettled` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_name_space_binds_nothing_only_once_the_globs_its_globs_reach_are_settled` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_name_space_binds_nothing_only_once_the_globs_that_may_bring_it_in_are_settled_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_name_two_globs_bring_in_as_different_items_is_ambiguous_to_a_reader_of_one_glob_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_no_std_crate_does_not_name_std_without_an_extern_crate_item` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_path_through_a_module_two_globs_bring_in_as_different_modules_is_ambiguous_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_path_through_an_import_a_cycle_leaves_undecided_in_another_name_space_is_cyclic` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_private_independent_supporter_does_not_erase_companions_through_a_returning_cycle_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_pub_use_of_a_name_read_through_a_private_glob_is_seen_no_wider_than_the_glob_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_rename_a_nested_group_and_a_plain_use_each_resolve_to_the_definition` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_returning_glob_cycle_beside_an_independent_named_route_keeps_ambiguity_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_route_without_a_glob_decides_via_glob_for_an_item_reached_both_ways` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_single_segment_import_of_a_dependency_is_an_external_alias_and_never_its_own_cycle` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_single_segment_import_of_a_workspace_library_binds_its_crate_root` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_three_hop_chain_resolves_and_lists_each_hop` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_trait_path_that_cannot_be_made_absolute_is_a_typed_error` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::a_use_group_with_leading_colon_members_binds_as_the_separate_uses_do` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::use_resolution::a_use_head_from_a_glob_that_is_also_an_extern_prelude_crate_is_ambiguous` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::a_use_path_of_edition_2015_starts_at_the_crate_root` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::an_ambiguity_a_glob_cycle_carries_beside_a_glob_of_one_of_its_items_is_one_answer_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::an_explicit_import_shadows_a_workspace_library_in_every_declaration_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::an_explicit_import_shadows_the_glob_whether_or_not_the_index_sees_what_it_binds` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::an_import_a_cycle_leaves_to_the_globs_binds_what_its_readers_could_access_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_a_reader_cannot_name_still_keeps_its_name_from_the_globs_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_binds_what_a_name_it_read_gains_later_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::an_import_keeps_the_globs_out_only_in_the_name_spaces_it_binds_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::an_import_of_a_value_a_reader_cannot_name_leaves_the_type_of_its_name_to_the_globs_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_of_a_value_named_like_a_crate_leaves_the_crate_and_the_globs_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::an_import_of_its_own_module_reads_what_the_globs_bring_in_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_that_binds_its_name_in_one_name_space_leaves_the_others_to_the_globs` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::an_import_that_bound_a_name_before_the_crates_opened_binds_the_crate_later_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_that_reads_a_name_only_through_a_glob_it_cannot_access_finds_nothing_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_that_reads_its_name_through_the_glob_whose_companion_it_would_drop_keeps_the_ambiguity_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_through_an_import_that_keeps_its_name_keeps_its_own_from_the_globs_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_import_whose_glob_reads_it_starts_at_the_crate_once_no_glob_may_bring_the_name_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_independent_named_module_route_does_not_erase_an_ambiguous_prefix_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_independent_named_route_does_not_erase_the_companion_of_another_glob_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_invalid_path_is_an_invalid_error_beside_a_valid_one` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_item_a_glob_brings_in_from_its_definition_apart_from_a_cycle_drops_the_companion_the_cycle_carries_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_item_a_glob_cycle_carries_back_beside_a_glob_that_holds_it_from_elsewhere_drops_its_companion_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_item_a_glob_cycle_carries_back_through_the_module_itself_keeps_its_companion_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::an_item_a_glob_reaches_by_a_public_and_a_private_route_is_visible_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::an_item_reached_by_a_capped_re_export_and_by_a_wider_glob_is_seen_as_widely_as_the_wider_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::cfg_attr_alternatives_index_every_file_under_the_one_module_path` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::crate_self_and_super_paths_resolve_from_the_module_they_are_written_in` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::cycles_through_private_globs_resolve_as_each_reader_can_access_them_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::definitions_and_explicit_imports_shadow_globs` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::explicit_imports_shadow_globs_for_the_modules_that_glob_the_importer` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::extern_crate_renames_and_leading_colon_paths_reach_the_library_from_other_targets` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::extern_crate_self_names_the_crate_itself_from_every_module` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::foreign_impl_members_keep_defining_traits_distinct_under_equal_crate_paths` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::globs_import_only_the_names_the_importer_can_name` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::globs_that_glob_one_another_beside_names_bound_twice_settle_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::imports_that_read_a_name_through_a_glob_agree_with_the_glob_on_an_ambiguous_re_export` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::imports_that_wait_on_one_another_through_globs_are_decided_as_rustc_does_in_every_order` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::imports_whose_heads_are_names_their_globs_bring_in_are_decided_alike_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::missing_names_dependency_items_and_macro_generated_items_are_not_found` | test | [Constraints](#constraints) |
| `raf_source::test_crate::use_resolution::names_the_private_glob_does_not_make_ambiguous_still_resolve_for_a_reader_outside_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::raw_identifiers_resolve_under_both_spellings` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::resolution_does_not_depend_on_the_order_of_the_declarations` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::super_above_the_crate_root_is_an_error_naming_file_and_line` | test | [Contract](#contract) |
| `raf_source::test_crate::use_resolution::the_rebuilt_globs_read_each_glob_path_in_the_settled_tables_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::the_refusal_of_an_ambiguous_name_in_one_name_space_leaves_the_other_resolved_in_every_order` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::the_same_symbol_reached_through_a_type_alias_chain_and_a_method_path_resolves` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::trait_impls_on_foreign_types_are_keyed_by_the_canonical_trait_path_in_every_spelling` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::two_globs_that_bring_in_different_items_are_ambiguous_beside_two_that_bring_in_one` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::two_libraries_with_one_crate_name_are_ambiguous_until_the_package_is_named` | test | [Verification](#verification) |
| `raf_source::test_crate::use_resolution::use_with_only_self_is_an_error_and_use_path_self_imports_the_module` | test | [Verification](#verification) |
| `raf_source::test_crate::walk_stack::a_directory_at_the_depth_limit_is_walked_and_one_below_it_is_refused` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::walk_stack::a_disk_chain_of_directories_is_refused_typed_on_a_two_mib_thread` | test | [Contract](#contract) |
| `raf_source::test_crate::walk_stack::a_member_pattern_at_the_depth_limit_expands_and_one_deeper_is_refused_typed` | test | [Contract](#contract) |
| `raf_source::test_crate::walk_stack::a_memory_chain_of_twenty_thousand_directories_is_refused_typed_on_a_two_mib_thread` | test | [Contract](#contract) |
| `raf_source::test_crate::walk_stack::a_use_tree_at_the_recursion_limit_loads_resolves_and_drops_on_a_five_hundred_twelve_kib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::walk_stack::one_hundred_twenty_eight_nested_modules_load_on_a_five_hundred_twelve_kib_thread` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_containment::a_directory_symlink_loop_ends_with_an_error_and_a_plain_directory_symlink_is_refused` | test | [Contract](#contract) |
| `raf_source::test_crate::workspace_containment::a_fifo_named_as_a_module_file_is_refused_without_blocking` | test | [Constraints](#constraints) |
| `raf_source::test_crate::workspace_containment::a_file_reached_through_a_directory_symlink_is_one_file_under_the_spelling_that_sorts_first` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_containment::a_link_to_a_symlink_that_points_outside_is_refused_naming_the_module` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_containment::a_manifest_target_path_that_leaves_the_root_is_refused_naming_the_key` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::workspace_containment::a_missing_explicit_target_file_is_a_target_error_on_disk_too` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_containment::a_module_that_reaches_its_own_file_through_a_directory_symlink_is_a_cycle` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_containment::a_path_attribute_through_an_outside_directory_symlink_with_a_parent_segment_is_refused` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_containment::a_symlinked_manifest_to_outside_is_refused_beside_one_to_a_file_inside` | test | [Contract](#contract) |
| `raf_source::test_crate::workspace_containment::a_symlinked_module_file_to_outside_is_refused_naming_the_module_beside_an_in_root_link` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::workspace_containment::a_target_file_that_is_a_symlink_to_outside_is_refused_naming_the_key` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_containment::a_target_root_that_is_a_fifo_is_refused_without_blocking` | test | [Constraints](#constraints) |
| `raf_source::test_crate::workspace_containment::a_tree_with_windows_device_names_loads_off_windows` | test | [Constraints](#constraints) |
| `raf_source::test_crate::workspace_containment::discovery_classifies_a_symlink_by_what_it_resolves_to` | test | [Contract](#contract) |
| `raf_source::test_crate::workspace_containment::disk_and_memory_trees_give_the_same_workspace_for_regular_files` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_containment::symlink_aliases_of_one_directory_parse_it_once` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_seeds::every_load_workspace_seed_loads_with_items_through_the_fuzz_input_code` | test | [Verification](#verification) |
| `raf_source::test_crate::workspace_seeds::the_self_check_the_fuzz_target_runs_first_passes_beside_an_input_that_is_not_a_tree` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::a_missing_descriptor_link_is_an_io_failure_and_never_absence` | test | [Contract](#contract) |
| `raf_source::tree::disk::tests::linux_verifier_reads_the_real_descriptor_of_an_open_file` | test | [Constraints](#constraints) |
| `raf_source::tree::disk::tests::linux_verifier_refuses_a_handle_link_that_leads_outside_and_accepts_one_inside` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::reserved_device_names_and_trailing_dots_or_spaces_are_ambiguous_on_windows_beside_plain_names` | test | [Constraints](#constraints) |
| `raf_source::tree::disk::tests::source_dispatch_preserves_backend_aliases_and_rejects_foreign_direct_paths` | test | [Contract](#contract) |
| `raf_source::tree::disk::tests::strategy::a_link_that_climbs_back_into_the_root_is_read_only_by_canonicalisation` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::a_path_three_hundred_deep_is_walked_under_a_limit_of_sixty_four_descriptors` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::a_symlink_that_becomes_a_directory_during_a_step_is_stepped_again` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::every_open_that_resolves_a_name_is_a_reference_or_does_not_block` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::every_strategy_ends_a_symlink_loop_with_an_error_and_lists_it_without_entering` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::every_strategy_follows_an_absolute_in_root_link_spelled_through_a_linked_parent` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::every_strategy_gives_one_identity_for_a_file_its_links_and_a_path_through_a_directory_link` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::every_strategy_lists_a_directory_and_refuses_a_symlink_a_file_and_an_outside_link` | test | [Contract](#contract) |
| `raf_source::tree::disk::tests::strategy::every_strategy_refuses_a_fifo_and_a_directory_as_a_file_without_blocking` | test | [Contract](#contract) |
| `raf_source::tree::disk::tests::strategy::every_strategy_refuses_a_parent_segment_after_an_outside_directory_symlink` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::every_strategy_refuses_an_outside_symlink_and_reads_in_root_links` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::strategy::every_strategy_reports_entry_kinds_and_refuses_an_outside_directory` | test | [Contract](#contract) |
| `raf_source::tree::disk::tests::strategy::links_into_the_outside_are_answered_alike_only_by_the_walk` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::swap::a_directory_swapped_after_verification_before_the_entries_are_read_is_refused` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::swap::a_directory_swapped_for_an_outside_symlink_after_the_check_is_refused` | test | [Contract](#contract) |
| `raf_source::tree::disk::tests::swap::a_directory_swapped_for_an_outside_symlink_before_a_listing_is_refused` | test | [Contract](#contract) |
| `raf_source::tree::disk::tests::swap::a_file_whose_directory_is_moved_out_after_verification_is_refused_after_the_read` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::swap::a_swap_before_the_open_undone_before_the_check_is_refused` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::swap::a_swap_flipped_at_every_hook_point_is_refused` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::swap::an_entry_whose_directory_is_swapped_for_an_outside_symlink_is_refused` | test | [Contract](#contract) |
| `raf_source::tree::disk::tests::swap::an_outside_fifo_reached_by_a_swap_before_the_open_is_never_opened` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::swap::captures_refuse_changed_observations_beside_stable_reads` | test | [Contract](#contract) |
| `raf_source::tree::disk::tests::swap::the_same_calls_with_the_hooks_and_no_swap_succeed` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::swap::the_same_entry_without_a_swap_is_found` | test | [Contract](#contract) |
| `raf_source::tree::disk::tests::swap::the_same_listing_without_a_swap_returns_the_entries` | test | [Contract](#contract) |
| `raf_source::tree::disk::tests::the_handle_path_verifier_refuses_a_path_outside_the_root_beside_one_inside` | test | [Constraints](#constraints) |
| `raf_source::tree::disk::tests::the_native_strategy_is_the_platforms_own` | test | [Contract](#contract) |
| `raf_source::tree::disk::tests::windows::a_disk_tree_refuses_a_reserved_device_name_or_a_trailing_dot_or_space_beside_a_plain_name` | test | [Verification](#verification) |
| `raf_source::tree::disk::tests::windows::the_final_path_of_a_handle_is_its_canonical_path_and_lies_under_the_root` | test | [Verification](#verification) |
| `raf_source::tree::git::command::tests::protocol_size_is_checked_before_growth_and_exact_size_has_a_valid_neighbor` | test | [Contract](#contract) |
| `raf_source::tree::git::tests::complete_directory_protocol_refuses_malformed_records_beside_exact_metadata` | test | [Contract](#contract) |
| `raf_source::tree::sources::tests::equal_relative_paths_and_backend_keys_remain_separate_sources` | test | [Contract](#contract), [Usage](#usage), [Constraints](#constraints) |
| `raf_source::tree::sources::tests::parent_join_and_real_directory_normalization_keep_the_selected_root` | test | [Contract](#contract) |
| `raf_source::tree::sources::tests::scoped_backend_ids_are_refused_before_seen_and_preserve_the_first_cause` | test | [Contract](#contract) |
| `raf_source::tree::sources::tests::standalone_memory_tree_refuses_another_sources_tag_before_seen_callback` | test | [Contract](#contract) |
| `raf_source::verbatim::tests::a_bare_group_in_a_use_group_contributes_its_members` | test | [Verification](#verification) |
| `raf_source::verbatim::tests::a_use_group_is_not_read_in_an_impl_block` | test | [Verification](#verification) |
| `raf_source::verbatim::tests::a_use_group_splits_into_one_use_per_member` | test | [Verification](#verification) |
| `raf_source::verbatim::tests::bodiless_functions_and_types_are_read` | test | [Verification](#verification) |
| `raf_source::verbatim::tests::other_shapes_are_not_read` | test | [Verification](#verification) |
| `raf_source::work::tests::a_counter_increments_by_what_is_added` | test | [Verification](#verification) |
| `raf_source::work::tests::a_limited_kind_panics_at_its_ceiling` | test | [Verification](#verification) |
| `raf_source::work::tests::a_limited_measure_counts_the_other_kinds_without_a_ceiling` | test | [Verification](#verification) |
| `raf_source::work::tests::a_measure_that_panicked_leaves_neither_counts_nor_ceiling_behind` | test | [Verification](#verification) |
| `raf_source::work::tests::every_kind_has_its_own_counter` | test | [Verification](#verification) |
| `raf_source::work::tests::passing_the_ceiling_panics_at_once` | test | [Verification](#verification) |
| `raf_source::work::tests::reaching_the_ceiling_is_allowed` | test | [Verification](#verification) |
| `raf_source::work::tests::the_panic_comes_before_the_work_that_follows_it` | test | [Verification](#verification) |
| `raf_source::work::tests::the_result_of_the_closure_is_returned` | test | [Verification](#verification) |
| `raf_source::work::tests::the_table_holds_every_kind_once_at_the_place_of_its_counter` | test | [Verification](#verification) |
| `raf_source::work::tests::the_tallies_are_reset_between_measures` | test | [Verification](#verification) |
| `raf_source::workspace::tests::a_test_function_whose_path_leaves_its_crate_path_is_a_typed_error` | test | [Verification](#verification) |
| `raf_source::workspace::tests::native_package_name_index_keeps_identity_and_enforces_its_exact_grant` | test | [Contract](#contract) |
<!-- raf:generated:end -->
