# T7 dispositions: every `#[test]` of the old `tests/docs_contract.rs`

The old file is `git show HEAD:tests/docs_contract.rs` (141 `#[test]` functions). Paths of moved tests are repository-relative; a target was checked to exist by name, and every moved test keeps its original name. A rule row names the finding kind and the named fixture in `crates/raf-workplan/tests/support/cases.rs` that produces it (run by `tests/kinds.rs` and, for the profile rules and reader errors, by `tests/rules.rs` and `tests/parse_errors.rs`). Where the old test was a matrix of positions that one fixture shows once, the note names the `raf-okf` tests that cover the other positions.

Summary: 141 tests = 18 kept + 74 moved + 49 replaced by a rule fixture. Removed model-rule tests: 123.

| # | Old test | Disposition | Note |
|---|---|---|---|
| 1 | `repository_documents_satisfy_the_profile` | kept | tests/docs_contract.rs (loads the tracked files through `violations`, which runs the model, the tracked-symlink rule, the Markdown-only rule, the frontmatter-outside rule and the inventory) |
| 2 | `repository_bindings_resolve_in_the_source_index` | kept | tests/docs_contract.rs (anti-vacuity kept: package set, raf_okf::Document / raf_source::Workspace / raf_workplan::Workplan resolve, delivered capabilities have bindings and all resolve; the binding verdict comes from the binding-* kinds) |
| 3 | `accepts_valid_fixture` | moved to `crates/raf-workplan/tests/documents.rs::accepts_valid_fixture` | The raf-workplan fixture also asserts the typed counts (3 capabilities, 1 change, 1 architecture page) and the loaded package. |
| 4 | `accepts_quoted_and_block_forms` | moved to `crates/raf-workplan/tests/documents.rs::accepts_quoted_and_block_forms` |  |
| 5 | `rejects_unparseable_frontmatter` | moved to `crates/raf-workplan/tests/documents.rs::rejects_unparseable_frontmatter` |  |
| 6 | `rejects_unknown_key` | moved to `crates/raf-workplan/tests/documents.rs::rejects_unknown_key` |  |
| 7 | `accepts_optional_tags_and_timestamp` | moved to `crates/raf-workplan/tests/documents.rs::accepts_optional_tags_and_timestamp` |  |
| 8 | `rejects_duplicate_tag` | replaced by rule `okf-duplicate-tag` (fixture `okf-duplicate-tag`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::tags_are_non_empty_unique_strings_and_a_timestamp_is_rfc_3339`. |
| 9 | `rejects_scalar_tags` | replaced by rule `okf-wrong-shape` (fixture `okf-wrong-shape`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::tags_are_non_empty_unique_strings_and_a_timestamp_is_rfc_3339`. |
| 10 | `rejects_missing_required_key` | replaced by rule `okf-missing-key` (fixture `okf-missing-key`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::every_required_key_of_every_type_is_reported_when_missing`. |
| 11 | `rejects_bad_type` | replaced by rule `okf-unknown-type` (fixture `okf-unknown-type`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::an_unknown_type_is_one_violation_at_the_type_key`. |
| 12 | `rejects_type_in_the_wrong_directory` | moved to `crates/raf-workplan/tests/documents.rs::rejects_type_in_the_wrong_directory` |  |
| 13 | `rejects_id_filename_mismatch` | moved to `crates/raf-workplan/tests/documents.rs::rejects_id_filename_mismatch` |  |
| 14 | `rejects_bad_state` | replaced by rule `okf-bad-state` (fixture `okf-bad-state`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::a_state_is_one_of_the_four`. |
| 15 | `rejects_wrong_bindings_keys` | replaced by rule `okf-bindings-keys` (fixture `okf-bindings-keys`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::bindings_have_exactly_the_keys_code_tests_and_examples_each_a_list`. |
| 16 | `rejects_missing_section` | moved to `crates/raf-workplan/tests/documents.rs::rejects_missing_section` |  |
| 17 | `rejects_sections_out_of_order` | replaced by rule `okf-section-unexpected` (fixture `okf-section-unexpected`); `okf-sections-out-of-order` (fixture `okf-sections-out-of-order`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::a_capability_holds_the_seven_sections_in_order`. |
| 18 | `rejects_empty_section` | replaced by rule `okf-section-empty` (fixture `okf-section-empty`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::a_section_is_empty_when_only_comments_and_generated_blocks_remain`. |
| 19 | `rejects_dangling_depends` | replaced by rule `depends-missing` (fixture `depends-missing`) |  |
| 20 | `rejects_depends_two_cycle` | replaced by rule `depends-cycle` (fixture `depends-cycle-two`) |  |
| 21 | `rejects_broken_relative_link` | moved to `crates/raf-workplan/tests/links.rs::rejects_broken_relative_link` |  |
| 22 | `rejects_broken_anchor` | moved to `crates/raf-workplan/tests/links.rs::rejects_broken_anchor` |  |
| 23 | `rejects_link_to_a_tracked_symlink` | moved to `crates/raf-workplan/tests/links.rs::rejects_link_to_a_tracked_symlink` |  |
| 24 | `rejects_a_tracked_symlink_that_nothing_links_to` | kept | tests/docs_contract.rs (tracked-symlink rule, now over the git-mode list) |
| 25 | `rejects_link_leaving_the_repository` | moved to `crates/raf-workplan/tests/links.rs::rejects_link_leaving_the_repository` |  |
| 26 | `anchors_follow_github_rules` | moved to `crates/raf-workplan/tests/links.rs::anchors_follow_github_rules` |  |
| 27 | `rejects_orphan_proposed_marker` | moved to `crates/raf-workplan/tests/states.rs::rejects_orphan_proposed_marker` |  |
| 28 | `rejects_malformed_proposed_marker` | replaced by rule `okf-parse-malformed-proposed` (fixture `okf-parse-malformed-proposed`) | Positions of the old matrix also covered by `crates/raf-okf/tests/markers.rs::every_misspelling_of_the_proposed_marker_is_refused_with_its_line`. |
| 29 | `rejects_considered_without_proposed_block` | moved to `crates/raf-workplan/tests/states.rs::rejects_considered_without_proposed_block` |  |
| 30 | `rejects_open_capability_without_change_record` | moved to `crates/raf-workplan/tests/states.rs::rejects_open_capability_without_change_record` |  |
| 31 | `rejects_delivered_with_proposed_block` | moved to `crates/raf-workplan/tests/states.rs::rejects_delivered_with_proposed_block` |  |
| 32 | `rejects_delivered_with_open_change_record` | moved to `crates/raf-workplan/tests/states.rs::rejects_delivered_with_open_change_record` |  |
| 33 | `rejects_affects_with_bad_section` | replaced by rule `okf-bad-affects-section` (fixture `okf-bad-affects-section`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::affects_entries_have_the_form_a_section_of_the_seven_and_a_capability_in_changes`. |
| 34 | `rejects_affects_slug_not_in_changes` | replaced by rule `okf-affects-not-in-changes` (fixture `okf-affects-not-in-changes`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::affects_entries_have_the_form_a_section_of_the_seven_and_a_capability_in_changes`. |
| 35 | `rejects_bad_contract_effect` | replaced by rule `okf-bad-contract-effect` (fixture `okf-bad-contract-effect`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::contract_effect_is_none_or_changes`. |
| 36 | `rejects_bad_outcome` | replaced by rule `okf-bad-outcome` (fixture `okf-bad-outcome`); `okf-bad-outcome` (fixture `okf-bad-outcome-deferred`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::items_need_a_claim_known_keys_and_a_known_outcome`. |
| 37 | `accepts_moved_outcome` | moved to `crates/raf-workplan/tests/references.rs::accepts_moved_outcome` | Same input shape as the old test: the outcome names a second record that exists and lists the capability. |
| 38 | `rejects_change_without_summary` | replaced by rule `okf-section-missing` (fixture `okf-section-missing`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::a_change_needs_a_non_empty_summary`. |
| 39 | `rejects_unclosed_generated_block` | replaced by rule `okf-parse-unclosed-generated` (fixture `okf-parse-unclosed-generated`) | Positions of the old matrix also covered by `crates/raf-okf/tests/markers.rs::each_generated_marker_error_names_its_line`. |
| 40 | `rejects_hand_written_capability_index` | replaced by rule `index-hand-written` (fixture `index-hand-written-after-the-catalogue`) | Positions of the old matrix also covered by `crates/raf-okf/tests/index.rs::prose_outside_the_blocks_is_refused_with_its_line`. |
| 41 | `rejects_issue_number_token` | moved to `crates/raf-workplan/tests/refs.rs::rejects_issue_number_token` |  |
| 42 | `issue_token_scan_spares_hex_colours` | moved to `crates/raf-workplan/tests/refs.rs::issue_token_scan_spares_hex_colours` |  |
| 43 | `rejects_process_wording` | moved to `crates/raf-workplan/tests/refs.rs::rejects_process_wording` |  |
| 44 | `fixture_builder_is_self_consistent` | moved to `crates/raf-workplan/tests/documents.rs::fixture_builder_is_self_consistent` |  |
| 45 | `rejects_capability_without_frontmatter` | moved to `crates/raf-workplan/tests/documents.rs::rejects_capability_without_frontmatter` |  |
| 46 | `rejects_unclosed_frontmatter` | moved to `crates/raf-workplan/tests/documents.rs::rejects_unclosed_frontmatter` |  |
| 47 | `rejects_proposed_marker_in_architecture` | moved to `crates/raf-workplan/tests/documents.rs::rejects_proposed_marker_in_architecture` |  |
| 48 | `rejects_proposed_marker_in_change` | moved to `crates/raf-workplan/tests/documents.rs::rejects_proposed_marker_in_change` |  |
| 49 | `rejects_change_naming_unknown_capability` | replaced by rule `changes-missing` (fixture `changes-missing`) |  |
| 50 | `rejects_nested_documents` | moved to `crates/raf-workplan/tests/documents.rs::rejects_nested_documents` |  |
| 51 | `rejects_acceptance_label` | moved to `crates/raf-workplan/tests/refs.rs::rejects_acceptance_label` |  |
| 52 | `rejects_resource_key` | moved to `crates/raf-workplan/tests/documents.rs::rejects_resource_key` |  |
| 53 | `inventory_accepts_a_matching_tree` | kept | tests/docs_contract.rs (INVENTORY table and its tests, now over minimal frontmatter pages and with the index row) |
| 54 | `inventory_table_has_the_documented_shape` | kept | tests/docs_contract.rs (INVENTORY table and its tests, now over minimal frontmatter pages and with the index row) |
| 55 | `inventory_rejects_a_missing_capability` | kept | tests/docs_contract.rs (INVENTORY table and its tests, now over minimal frontmatter pages and with the index row) |
| 56 | `inventory_rejects_a_missing_change_record` | kept | tests/docs_contract.rs (INVENTORY table and its tests, now over minimal frontmatter pages and with the index row) |
| 57 | `inventory_rejects_a_missing_architecture_document` | kept | tests/docs_contract.rs (INVENTORY table and its tests, now over minimal frontmatter pages and with the index row) |
| 58 | `inventory_rejects_a_state_change` | kept | tests/docs_contract.rs (INVENTORY table and its tests, now over minimal frontmatter pages and with the index row) |
| 59 | `inventory_rejects_an_id_change` | kept | tests/docs_contract.rs (INVENTORY table and its tests, now over minimal frontmatter pages and with the index row) |
| 60 | `inventory_rejects_a_document_the_table_does_not_list` | kept | tests/docs_contract.rs (INVENTORY table and its tests, now over minimal frontmatter pages and with the index row) |
| 61 | `rejects_placeholder_tokens` | moved to `crates/raf-workplan/tests/documents.rs::rejects_placeholder_tokens` |  |
| 62 | `placeholder_scan_spares_code_and_longer_words` | moved to `crates/raf-workplan/tests/documents.rs::placeholder_scan_spares_code_and_longer_words` |  |
| 63 | `rejects_sections_left_empty_by_comments` | moved to `crates/raf-workplan/tests/documents.rs::rejects_sections_left_empty_by_comments` |  |
| 64 | `comments_hide_text_but_not_the_text_beside_them` | moved to `crates/raf-workplan/tests/documents.rs::comments_hide_text_but_not_the_text_beside_them` |  |
| 65 | `rejects_unterminated_quote` | replaced by rule `okf-parse-unterminated-quote` (fixture `okf-parse-unterminated-quote`); `okf-parse-text-after-quote` (fixture `okf-parse-text-after-quote`) | Positions of the old matrix also covered by `crates/raf-okf/tests/refusals.rs::an_unterminated_quote_and_text_after_a_quote_are_refused`. |
| 66 | `rejects_unquoted_colon_space` | replaced by rule `okf-parse-plain-colon-space` (fixture `okf-parse-plain-colon-space`) | Positions of the old matrix also covered by `crates/raf-okf/tests/refusals.rs::plain_scalars_with_a_colon_space_or_an_indicator_are_refused`. |
| 67 | `rejects_leading_indicator_character` | replaced by rule `okf-parse-indicator-character` (fixture `okf-parse-indicator-character`); `okf-parse-anchor` (fixture `okf-parse-anchor`); `okf-parse-alias` (fixture `okf-parse-alias`); `okf-parse-tag` (fixture `okf-parse-tag`) | Positions of the old matrix also covered by `crates/raf-okf/tests/refusals.rs::plain_scalars_with_a_colon_space_or_an_indicator_are_refused`; `crates/raf-okf/tests/refusals.rs::anchors_aliases_tags_and_merge_keys_are_refused_naming_the_token`. |
| 68 | `rejects_block_scalar_indicator_value` | replaced by rule `okf-parse-block-scalar` (fixture `okf-parse-block-scalar`) | Positions of the old matrix also covered by `crates/raf-okf/tests/refusals.rs::block_scalars_are_refused_naming_the_header`. |
| 69 | `accepts_comments_after_scalars` | moved to `crates/raf-workplan/tests/documents.rs::accepts_comments_after_scalars` |  |
| 70 | `rejects_bad_scalars_in_every_value_position` | replaced by rule `okf-parse-unterminated-quote` (fixture `okf-parse-unterminated-quote`); `okf-parse-plain-colon-space` (fixture `okf-parse-plain-colon-space`); `okf-parse-block-scalar` (fixture `okf-parse-block-scalar`); `okf-parse-alias` (fixture `okf-parse-alias`); `okf-parse-indicator-character` (fixture `okf-parse-indicator-character`) | Positions of the old matrix also covered by `crates/raf-okf/tests/refusals.rs::unterminated_quotes_are_refused_in_flow_list_item_and_nested_item_positions`; `crates/raf-okf/tests/refusals.rs::a_colon_space_inside_a_flow_list_is_refused`; `crates/raf-okf/tests/refusals.rs::block_indicators_inside_an_items_entry_are_refused`. |
| 71 | `rejects_lowercase_proposed_marker` | replaced by rule `okf-parse-malformed-proposed` (fixture `okf-parse-malformed-proposed`) | Positions of the old matrix also covered by `crates/raf-okf/tests/markers.rs::every_misspelling_of_the_proposed_marker_is_refused_with_its_line`. |
| 72 | `rejects_proposed_marker_without_space` | replaced by rule `okf-parse-malformed-proposed` (fixture `okf-parse-malformed-proposed`) | Positions of the old matrix also covered by `crates/raf-okf/tests/markers.rs::every_misspelling_of_the_proposed_marker_is_refused_with_its_line`. |
| 73 | `rejects_proposed_marker_with_extra_spacing_or_nesting` | replaced by rule `okf-parse-malformed-proposed` (fixture `okf-parse-malformed-proposed`) | Positions of the old matrix also covered by `crates/raf-okf/tests/markers.rs::every_misspelling_of_the_proposed_marker_is_refused_with_its_line`. |
| 74 | `rejects_designed_with_proposed_block` | moved to `crates/raf-workplan/tests/states.rs::rejects_designed_with_proposed_block` |  |
| 75 | `rejects_proposed_section_missing_from_affects` | moved to `crates/raf-workplan/tests/states.rs::rejects_proposed_section_missing_from_affects` |  |
| 76 | `rejects_proposed_block_outside_a_section` | replaced by rule `okf-proposed-outside-section` (fixture `okf-proposed-outside-section`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::a_proposed_block_is_valid_only_inside_a_section_of_a_capability`. |
| 77 | `rejects_bindings_before_delivery` | replaced by rule `okf-binding-not-empty` (fixture `okf-binding-not-empty`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::bindings_by_state_designed_and_considered_bind_nothing_delivered_binds_code_and_tests`. |
| 78 | `rejects_delivered_without_code_or_test_binding` | replaced by rule `okf-binding-required` (fixture `okf-binding-required`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::bindings_by_state_designed_and_considered_bind_nothing_delivered_binds_code_and_tests`. |
| 79 | `rejects_broken_reference_definition` | moved to `crates/raf-workplan/tests/links.rs::rejects_broken_reference_definition` |  |
| 80 | `rejects_broken_href_and_src_attributes` | moved to `crates/raf-workplan/tests/links.rs::rejects_broken_href_and_src_attributes` |  |
| 81 | `rejects_broken_link_outside_the_document_directories` | moved to `crates/raf-workplan/tests/links.rs::rejects_broken_link_outside_the_document_directories` |  |
| 82 | `accepts_resolving_reference_and_attribute_links` | moved to `crates/raf-workplan/tests/links.rs::accepts_resolving_reference_and_attribute_links` |  |
| 83 | `rejects_moved_outcome_naming_a_missing_change` | replaced by rule `moved-missing` (fixture `moved-missing`) |  |
| 84 | `rejects_typed_document_outside_its_directory` | moved to `crates/raf-workplan/tests/documents.rs::rejects_typed_document_outside_its_directory` | The in-directory mismatch (`wrong-directory`) moved. A typed page outside the document directories is not loaded by the model (pinned by the moved test); the repository keeps the old refusal, widened to any frontmatter, as `docs_contract::rejects_a_typed_document_outside_the_document_directories`. |
| 85 | `rejects_non_markdown_file_in_a_document_directory` | kept | tests/docs_contract.rs (repository layout policy: the model ignores non-Markdown files, see files_that_are_not_markdown_are_not_documents) |
| 86 | `accepts_a_generated_capability_index` | moved to `crates/raf-workplan/tests/documents.rs::accepts_a_generated_capability_index` | Same input as the old test (an index with no header). The real `docs/capabilities/index.md` is also written by `make docs-sync` and judged by `generated-stale`. |
| 87 | `rejects_empty_or_non_scalar_title` | replaced by rule `okf-empty-text` (fixture `okf-empty-text`); `okf-wrong-shape` (fixture `okf-wrong-shape`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::a_title_and_a_description_are_non_empty_strings`. |
| 88 | `rejects_bad_id_prefix_or_slug` | replaced by rule `okf-invalid-id` (fixture `okf-invalid-id`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::an_id_needs_the_prefix_of_its_type_a_kebab_slug_and_the_file_name`. |
| 89 | `rejects_non_scalar_state_and_scalar_bindings` | replaced by rule `okf-wrong-shape` (fixture `okf-wrong-shape`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::a_state_is_one_of_the_four`; `crates/raf-okf/tests/profile.rs::bindings_have_exactly_the_keys_code_tests_and_examples_each_a_list`. |
| 90 | `rejects_bad_depends_shapes` | replaced by rule `okf-bad-capability-id` (fixture `okf-bad-capability-id`); `okf-wrong-shape` (fixture `okf-wrong-shape`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::depends_is_a_list_of_cap_ids_and_does_not_need_the_capability_to_exist`. |
| 91 | `rejects_self_and_three_node_dependency_cycles` | replaced by rule `depends-cycle` (fixture `depends-cycle-self`); `depends-cycle` (fixture `depends-cycle-three`) |  |
| 92 | `rejects_proposed_change_that_does_not_list_the_capability` | moved to `crates/raf-workplan/tests/states.rs::rejects_proposed_change_that_does_not_list_the_capability` |  |
| 93 | `rejects_bad_changes_list` | replaced by rule `okf-empty-changes` (fixture `okf-empty-changes`); `okf-bad-capability-id` (fixture `okf-bad-capability-id`); `okf-wrong-shape` (fixture `okf-wrong-shape`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::changes_is_a_non_empty_list_of_cap_ids`. |
| 94 | `rejects_bad_affects_shapes` | replaced by rule `okf-bad-affects` (fixture `okf-bad-affects`); `okf-wrong-shape` (fixture `okf-wrong-shape`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::affects_entries_have_the_form_a_section_of_the_seven_and_a_capability_in_changes`. |
| 95 | `rejects_non_scalar_contract_effect` | replaced by rule `okf-bad-contract-effect` (fixture `okf-bad-contract-effect`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::contract_effect_is_none_or_changes`. |
| 96 | `rejects_item_shape_errors` | replaced by rule `okf-item-unknown-key` (fixture `okf-item-unknown-key`); `okf-item-claim-empty` (fixture `okf-item-claim-empty`); `okf-parse-duplicate-key` (fixture `okf-parse-duplicate-key`); `okf-parse-mixed-list` (fixture `okf-parse-mixed-list`); `okf-parse-indentation` (fixture `okf-parse-indentation`); `okf-parse-tag` (fixture `okf-parse-tag`); `okf-parse-expected-item` (fixture `okf-parse-expected-item`); `okf-wrong-shape` (fixture `okf-wrong-shape`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::items_need_a_claim_known_keys_and_a_known_outcome`; `crates/raf-okf/tests/refusals.rs::items_keep_their_maps_and_scalars`; `crates/raf-okf/tests/refusals.rs::block_indicators_inside_an_items_entry_are_refused`; `crates/raf-okf/tests/refusals.rs::layout_errors_are_refused_with_their_position`. |
| 97 | `rejects_frontmatter_shape_errors` | replaced by rule `okf-parse-duplicate-key` (fixture `okf-parse-duplicate-key`); `okf-parse-invalid-key` (fixture `okf-parse-invalid-key`); `okf-parse-unterminated-flow` (fixture `okf-parse-unterminated-flow`); `okf-parse-empty-flow-entry` (fixture `okf-parse-empty-flow-entry`); `okf-parse-indentation` (fixture `okf-parse-indentation`); `okf-binding-not-a-list` (fixture `okf-binding-not-a-list`) | Positions of the old matrix also covered by `crates/raf-okf/tests/refusals.rs::a_duplicate_key_is_refused_at_every_level`; `crates/raf-okf/tests/refusals.rs::flow_list_shape_errors_are_refused`; `crates/raf-okf/tests/refusals.rs::layout_errors_are_refused_with_their_position`; `crates/raf-okf/tests/refusals.rs::stray_and_over_indented_lines_after_a_flow_value_are_refused`; `crates/raf-okf/tests/profile.rs::bindings_have_exactly_the_keys_code_tests_and_examples_each_a_list`. |
| 98 | `rejects_missing_type_key` | replaced by rule `okf-missing-key` (fixture `okf-missing-key`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::a_missing_type_key_is_the_only_violation`. |
| 99 | `rejects_duplicate_section` | replaced by rule `okf-section-duplicated` (fixture `okf-section-duplicated`) | Positions of the old matrix also covered by `crates/raf-okf/tests/profile.rs::a_capability_holds_the_seven_sections_in_order`. |
| 100 | `rejects_generated_marker_errors` | moved to `crates/raf-workplan/tests/plain.rs::rejects_generated_marker_errors` |  |
| 101 | `refuses_non_regular_files_instead_of_blocking` | kept | tests/docs_contract.rs (tests tests/support::read_regular) |
| 102 | `refuses_a_file_swapped_in_after_inspection` | kept | tests/docs_contract.rs (tests tests/support::check_handle) |
| 103 | `accepts_a_generated_index_with_an_spdx_header` | moved to `crates/raf-workplan/tests/documents.rs::accepts_a_generated_index_with_an_spdx_header` | Both header forms of the old test: the several-line comment and the single-line one. |
| 104 | `rejects_an_index_with_prose_outside_the_blocks` | replaced by rule `index-hand-written` (fixture `index-hand-written-after-the-catalogue`) | Positions of the old matrix also covered by `crates/raf-okf/tests/index.rs::prose_outside_the_blocks_is_refused_with_its_line`. |
| 105 | `rejects_an_index_whose_marker_is_only_in_a_code_span` | replaced by rule `index-hand-written` (fixture `index-hand-written-after-the-catalogue`); `okf-parse-index-marker` (fixture `okf-parse-index-marker`) | Positions of the old matrix also covered by `crates/raf-okf/tests/index.rs::an_index_with_no_block_or_an_unclosed_block_is_refused`. |
| 106 | `rejects_an_index_with_a_mid_line_marker` | replaced by rule `okf-parse-index-marker` (fixture `okf-parse-index-marker`); `okf-parse-index-empty` (fixture `okf-parse-index-empty`) | Positions of the old matrix also covered by `crates/raf-okf/tests/index.rs::indented_and_mid_line_markers_are_refused`; `crates/raf-okf/tests/index.rs::an_index_with_no_block_or_an_unclosed_block_is_refused`. |
| 107 | `rejects_placeholder_spellings` | moved to `crates/raf-workplan/tests/documents.rs::rejects_placeholder_spellings` |  |
| 108 | `placeholder_scan_spares_longer_words_and_dotted_prose` | moved to `crates/raf-workplan/tests/documents.rs::placeholder_scan_spares_longer_words_and_dotted_prose` |  |
| 109 | `rejects_broken_unquoted_attribute_values` | moved to `crates/raf-workplan/tests/links.rs::rejects_broken_unquoted_attribute_values` |  |
| 110 | `rejects_an_index_header_with_unknown_tags_or_in_the_wrong_place` | replaced by rule `okf-parse-index-header` (fixture `okf-parse-index-header`) | Positions of the old matrix also covered by `crates/raf-okf/tests/index.rs::a_header_with_unknown_tags_or_in_the_wrong_place_is_refused`. |
| 111 | `rejects_an_index_with_an_indented_marker` | replaced by rule `okf-parse-index-marker` (fixture `okf-parse-index-marker`) | Positions of the old matrix also covered by `crates/raf-okf/tests/index.rs::indented_and_mid_line_markers_are_refused`. |
| 112 | `reads_attributes_with_spaces_around_the_equals_sign` | moved to `crates/raf-workplan/tests/links.rs::reads_attributes_with_spaces_around_the_equals_sign` |  |
| 113 | `does_not_read_attributes_that_only_end_in_href_or_src` | moved to `crates/raf-workplan/tests/links.rs::does_not_read_attributes_that_only_end_in_href_or_src` |  |
| 114 | `rejects_broken_attributes_in_tags_split_across_lines` | moved to `crates/raf-workplan/tests/links.rs::rejects_broken_attributes_in_tags_split_across_lines` |  |
| 115 | `tag_scan_spares_prose_autolinks_and_closed_tags` | moved to `crates/raf-workplan/tests/links.rs::tag_scan_spares_prose_autolinks_and_closed_tags` |  |
| 116 | `rejects_an_index_header_tag_without_a_value` | replaced by rule `okf-parse-index-header` (fixture `okf-parse-index-header`) | Positions of the old matrix also covered by `crates/raf-okf/tests/index.rs::an_empty_header_tag_is_refused`. |
| 117 | `tag_state_does_not_survive_a_blank_line_or_a_fence` | moved to `crates/raf-workplan/tests/links.rs::tag_state_does_not_survive_a_blank_line_or_a_fence` |  |
| 118 | `reads_an_attribute_value_that_starts_on_the_next_line` | moved to `crates/raf-workplan/tests/links.rs::reads_an_attribute_value_that_starts_on_the_next_line` |  |
| 119 | `bindings_that_name_real_symbols_resolve` | moved to `crates/raf-workplan/tests/bindings.rs::bindings_that_name_real_symbols_resolve` |  |
| 120 | `a_binding_to_a_missing_path_is_reported_with_its_capability` | moved to `crates/raf-workplan/tests/bindings.rs::a_binding_to_a_missing_path_is_reported_with_its_capability` |  |
| 121 | `a_binding_to_a_macro_generated_function_is_not_found` | moved to `crates/raf-workplan/tests/bindings.rs::a_binding_to_a_macro_generated_function_is_not_found` | Original name kept; the test also asserts the macro hint. |
| 122 | `a_binding_to_an_item_under_the_module_that_declares_its_file_is_not_found` | moved to `crates/raf-workplan/tests/bindings.rs::a_binding_to_an_item_under_the_module_that_declares_its_file_is_not_found` |  |
| 123 | `every_unresolved_binding_of_every_list_is_reported` | moved to `crates/raf-workplan/tests/bindings.rs::every_unresolved_binding_of_every_list_is_reported` |  |
| 124 | `a_tests_binding_must_name_a_function_marked_test` | moved to `crates/raf-workplan/tests/bindings.rs::a_tests_binding_must_name_a_function_marked_test` |  |
| 125 | `a_code_binding_must_name_a_shipping_item` | moved to `crates/raf-workplan/tests/bindings.rs::a_code_binding_must_name_a_shipping_item` |  |
| 126 | `usage_and_examples_blocks_are_the_crate_doctests` | kept | tests/docs_contract.rs (doctest pairs) |
| 127 | `a_block_that_differs_by_one_line_is_reported` | kept | tests/docs_contract.rs (doctest pairs) |
| 128 | `doctest_comparison_fixtures` | kept | tests/docs_contract.rs (doctest pairs) |
| 129 | `a_tilde_fenced_usage_block_is_accepted_and_an_unclosed_block_is_refused` | kept | tests/docs_contract.rs (doctest pairs) |
| 130 | `rejects_a_link_split_across_lines_in_any_tracked_markdown_file` | moved to `crates/raf-workplan/tests/plain.rs::rejects_a_link_split_across_lines_in_any_tracked_markdown_file` |  |
| 131 | `rejects_a_link_split_across_lines_in_a_profile_document` | moved to `crates/raf-workplan/tests/plain.rs::rejects_a_link_split_across_lines_in_a_profile_document` |  |
| 132 | `accepts_one_line_links_and_links_in_fences_beside_the_split_rule` | moved to `crates/raf-workplan/tests/plain.rs::accepts_one_line_links_and_links_in_fences_beside_the_split_rule` |  |
| 133 | `rejects_an_empty_link_target` | moved to `crates/raf-workplan/tests/links.rs::rejects_an_empty_link_target` |  |
| 134 | `does_not_read_links_or_placeholders_inside_html_comments` | moved to `crates/raf-workplan/tests/plain.rs::does_not_read_links_or_placeholders_inside_html_comments` |  |
| 135 | `reads_placeholders_in_plain_markdown_outside_code` | moved to `crates/raf-workplan/tests/plain.rs::reads_placeholders_in_plain_markdown_outside_code` |  |
| 136 | `reports_marker_errors_of_plain_markdown_once` | moved to `crates/raf-workplan/tests/plain.rs::reports_marker_errors_of_plain_markdown_once` |  |
| 137 | `reads_a_proposed_block_in_plain_markdown_as_outside_a_capability` | moved to `crates/raf-workplan/tests/plain.rs::reads_a_proposed_block_in_plain_markdown_as_outside_a_capability` |  |
| 138 | `reads_markdown_outside_the_document_directories_with_or_without_frontmatter` | moved to `crates/raf-workplan/tests/plain.rs::reads_markdown_outside_the_document_directories_with_or_without_frontmatter` |  |
| 139 | `rejects_unclosed_frontmatter_in_plain_markdown` | moved to `crates/raf-workplan/tests/plain.rs::rejects_unclosed_frontmatter_in_plain_markdown` |  |
| 140 | `rejects_absolute_and_empty_link_targets_beside_relative_ones` | moved to `crates/raf-workplan/tests/links.rs::rejects_absolute_and_empty_link_targets_beside_relative_ones` |  |
| 141 | `rejects_a_link_whose_target_cannot_be_read_for_its_anchor` | moved to `crates/raf-workplan/tests/links.rs::rejects_a_link_whose_target_cannot_be_read_for_its_anchor` |  |

## Tests the new `tests/docs_contract.rs` adds (26 tests = 18 kept + 8 new)

`violations(&Checkout, &Workspace, &Inventory)` runs every rule family (the model through `judge`, tracked symlinks, Markdown-only document directories, frontmatter outside the document directories, the inventory). The repository test and the fixture test of each family call it, so removing a family from it fails a fixture test (checked by deleting each call in turn).

- `repository_documents_satisfy_the_document_model`: `Gate::workspace(gate_root()).assert_clean()` on the repository, including the hook snapshot (no `.git`, so git answers that it is not a work tree and every file of the snapshot stands for a tracked path).
- `the_checker_accepts_a_valid_checkout`, `the_checker_judges_the_documents_through_the_model`, `the_checker_judges_markdown_outside_the_document_directories`, `the_checker_judges_links_against_the_tracked_paths_and_symlinks`: negative and neighbouring-valid fixtures for the wiring of the thin caller.
- `rejects_a_typed_document_outside_the_document_directories`: the outside-the-directories half of the old `rejects_typed_document_outside_its_directory`, with the old scope (top level, `docs/`, `work/`) and the old strength: any parseable frontmatter on such a page is refused whatever its `type` (cases: a typed page of each type, `docs/notes.md` without `type`, `NOTES.md` with `type: Bug`, `work/notes.md` with an untyped block, `README.md` with `foo: bar`).
- `inventory_rejects_a_missing_capability_index`, `inventory_rejects_an_index_the_table_does_not_list`: the `docs/capabilities/index.md` row of `INVENTORY`, exact in both directions.

## Tests added to raf-workplan by this task (25)

- `tests/gate.rs` (13): a clean repository passes; a broken one panics listing every finding as `kind file:line message (fix)`; a missing `docs/capabilities`, a directory with no capability document, a missing directory and an unloadable source workspace are errors; a missing `work/changes` is the empty set; a real symlink on disk is `link-symlink` (tracked list and directory walk); a file the tracked list leaves out is `link-broken`; inside a git checkout the tracked paths come from the index; a work tree whose index cannot be listed is a `GateError`, not a walk; a `GIT_DIR` that names no repository is a `GateError` (run in a child process of the test binary, since setting the environment is unsafe).
- `tests/documents.rs` (11): `accepts_quoted_and_block_forms`, `accepts_optional_tags_and_timestamp`, `accepts_comments_after_scalars`, `rejects_sections_left_empty_by_comments`, `comments_hide_text_but_not_the_text_beside_them`, `rejects_placeholder_tokens`, `rejects_placeholder_spellings`, `placeholder_scan_spares_code_and_longer_words`, `placeholder_scan_spares_longer_words_and_dotted_prose`, and the two index tests `accepts_a_generated_capability_index` and `accepts_a_generated_index_with_an_spdx_header` (both header forms) under their original names.
- `tests/references.rs` (1): `accepts_moved_outcome`, with the original input shape (a second record that exists).
- `tests/bindings.rs`: `a_binding_to_a_macro_generated_function_is_not_found` takes back its original name.

## Balance

Removed model-rule tests: 123 (74 moved + 49 replaced). `crates/raf-workplan` holds 253 integration tests, 5 unit tests and 133 named fixture cases (`support/cases.rs`, run in both directions by `tests/kinds.rs`), so new + moved is at least removed.

## Deliberate differences from the old file (losses recorded, none silent)

1. The model reads Markdown outside the document directories as plain Markdown (`plain::check`) and does not validate it as a typed document. The old refusal is kept in `docs_contract` at its old strength: a page at the top level or below `docs/` or `work/`, outside the three document directories, that opens with any parseable frontmatter is a violation, whatever its `type` (an unclosed block is `plain::check`'s finding). Markdown elsewhere (issue templates, crate fixtures) may carry frontmatter.
2. Non-Markdown files in a document directory: the model ignores them (`files_that_are_not_markdown_are_not_documents`). The repository still refuses them: `rejects_non_markdown_file_in_a_document_directory` is kept.
3. `plain::check` never reports `unreadable-directory`; only `Workplan::load` does, and `docs_contract` passes the tracked list, so it has no walk.
4. `inventory_accepts_a_matching_tree` no longer also asserts that the same tree satisfies the model: the inventory fixture is minimal frontmatter pages; the model's own acceptance is `accepts_valid_fixture`.
5. The issue-reference scan of `docs_contract` is now `issue-reference` inside `check` and `plain::check`; `repo_gates` keeps the matcher fixtures over `raf_workplan::refs`.
