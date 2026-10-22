---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-normal-form
title: "Census normal form, fingerprints and grouping"
description: "How the RAF census normalises function bodies, fingerprints them with BLAKE3 and groups the duplicates."
state: delivered
bindings:
  code:
    - raf_census::report::vocab
    - raf_census::REPORT_FORM
    - raf_census::ReportError
    - raf_census::Baseline
    - raf_census::Baseline::from_turtle
    - raf_census::Baseline::to_turtle
    - raf_census::BaselineGroup
    - raf_census::Census
    - raf_census::Census::baseline
    - raf_census::Census::body_groups
    - raf_census::Census::file
    - raf_census::Census::files
    - raf_census::Census::groups
    - raf_census::Census::hex_tables
    - raf_census::Census::index
    - raf_census::Census::load
    - raf_census::Census::match_jobs
    - raf_census::Census::options
    - raf_census::Census::shared_constants
    - raf_census::Census::shim_groups
    - raf_census::Census::unit
    - raf_census::Census::units
    - raf_census::Census::units_of
    - raf_census::CensusOptions
    - raf_census::CensusOptions::max_file_bytes
    - raf_census::CensusOptions::min_tokens
    - raf_census::ConstantRow
    - raf_census::DEFAULT_MAX_FILE_BYTES
    - raf_census::DEFAULT_MIN_TOKENS
    - raf_census::Divergence
    - raf_census::ENCODING
    - raf_census::Error
    - raf_census::FileCtx
    - raf_census::Fingerprint
    - raf_census::Fingerprint::abbreviated
    - raf_census::Fingerprint::body
    - raf_census::Fingerprint::kind
    - raf_census::Fingerprint::shim
    - raf_census::Form
    - raf_census::Form::is_empty
    - raf_census::Form::len
    - raf_census::Form::texts
    - raf_census::Form::tokens
    - raf_census::Group
    - raf_census::Group::member_symbols
    - raf_census::GroupKind
    - raf_census::HexTable
    - raf_census::Index
    - raf_census::Index::from_turtle
    - raf_census::Index::to_turtle
    - raf_census::IndexGroup
    - raf_census::IndexUnit
    - raf_census::Job
    - raf_census::Job::entry_points
    - raf_census::Job::forbidding
    - raf_census::Job::new
    - raf_census::Job::variants
    - raf_census::JobForbidden
    - raf_census::JobMatch
    - raf_census::JobReport
    - raf_census::JobRow
    - raf_census::LITERAL_TIER
    - raf_census::Level
    - raf_census::Level::ALL
    - raf_census::Level::id
    - raf_census::MAX_NESTING
    - raf_census::MIN_SHIM_TOKENS
    - raf_census::MatchClass
    - raf_census::MatchRow
    - raf_census::Member
    - raf_census::MemberRow
    - raf_census::NORMAL_FORM
    - raf_census::Normalizer
    - raf_census::Normalizer::explain
    - raf_census::Normalizer::new
    - raf_census::Normalizer::print
    - raf_census::Normalizer::tower
    - raf_census::PASSES
    - raf_census::ParseFingerprintError
    - raf_census::Print
    - raf_census::RuleId
    - raf_census::RuleId::as_str
    - raf_census::RuleId::new
    - raf_census::SHIM_MAX_TOKENS
    - raf_census::SharedConstant
    - raf_census::Slice
    - raf_census::Slice::new
    - raf_census::Slice::params
    - raf_census::StringKind
    - raf_census::StringLiteral
    - raf_census::TableKind
    - raf_census::TableKind::name
    - raf_census::TableRow
    - raf_census::Token
    - raf_census::TokenDiff
    - raf_census::Tower
    - raf_census::Tower::calls_through_self
    - raf_census::Tower::literal_values
    - raf_census::Tower::form
    - raf_census::Tower::new
    - raf_census::Tower::print
    - raf_census::Tower::shim_candidate
    - raf_census::Tower::value_tokens
    - raf_census::Unit
    - raf_census::UnitKind
    - raf_census::VALUE_ADAPTERS
    - raf_census::WEAK_KEYWORDS
    - raf_census::check_normal_form
    - raf_census::manifest
  tests:
    - raf_census::error::tests::each_variant_renders_its_facts
    - raf_census::error::tests::an_options_error_has_no_nested_source
    - raf_census::group::tests::digest_suffixes_are_distinct_group_keys
    - raf_census::group::tests::members_order_by_symbol_then_file_then_line
    - raf_census::normalize::literal::tests::an_integer_beyond_u128_is_still_an_int
    - raf_census::normalize::literal::tests::each_literal_kind_has_its_name
    - raf_census::normalize::literal::tests::floats_and_strings_are_not_integers
    - raf_census::normalize::literal::tests::spellings_of_one_value_respell_alike
    - raf_census::options::tests::defaults_are_twenty_tokens_and_eight_mebibytes
    - raf_census::options::tests::max_file_bytes_is_set
    - raf_census::options::tests::zero_min_tokens_is_refused_and_one_is_accepted
    - raf_census::report::mint::tests::fragments_are_ascii_and_percent_encode_reserved_bytes
    - raf_census::report::read::tests::each_token_outside_the_profile_names_its_line_and_column
    - raf_census::report::read::tests::the_graph_of_each_example_is_its_pinned_triple_set
    - raf_census::report::shape::tests::every_required_shape_property_has_a_missing_and_unknown_neighbor
    - raf_census::report::write::tests::the_digest_covers_everything_but_the_base_line
    - raf_census::report::mint::tests::fragments_are_injective_over_their_separators
    - raf_census::test_crate::forms::a_fingerprint_displays_and_parses_back
    - raf_census::test_crate::forms::a_fingerprint_refuses_every_malformed_spelling
    - raf_census::test_crate::forms::a_fixture_named_for_a_size_has_that_many_value_tokens
    - raf_census::test_crate::forms::a_shim_candidate_keeps_its_body_print_and_the_shim_print_differs_by_callee
    - raf_census::test_crate::forms::an_unlexable_slice_is_a_source_error_with_a_valid_neighbour
    - raf_census::test_crate::forms::copies_with_reordered_parameters_share_a_body_print_and_forwarders_keep_their_order
    - raf_census::test_crate::forms::every_threshold_counts_the_value_form
    - raf_census::test_crate::forms::below_the_tier_the_final_sequence_keeps_literals_and_above_it_abstracts_them
    - raf_census::test_crate::forms::every_registered_pair_compares_as_registered
    - raf_census::test_crate::forms::explain_reports_reordered_parameters_as_equal_from_the_literal_tier
    - raf_census::test_crate::forms::explain_names_the_levels_and_the_first_differing_token_with_its_line
    - raf_census::test_crate::forms::fingerprints_sort_by_kind_then_value
    - raf_census::test_crate::forms::nesting_at_the_budget_lexes_and_one_deeper_is_refused
    - raf_census::test_crate::forms::parameters_and_locals_are_numbered_together_by_first_appearance
    - raf_census::test_crate::forms::self_is_not_a_parameter_and_a_field_named_like_one_is_kept
    - raf_census::test_crate::forms::shim_forms_exist_for_six_to_twenty_nine_tokens
    - raf_census::test_crate::forms::the_body_print_needs_min_tokens
    - raf_census::test_crate::forms::human_abbreviations_do_not_replace_complete_identity
    - raf_census::test_crate::forms::fingerprint_refusals_escape_untrusted_controls_beside_valid_names
    - raf_census::test_crate::forms::self_callee_query_distinguishes_generic_types_from_calls
    - raf_census::test_crate::forms::literal_value_introspection_selects_the_production_default
    - raf_census::test_crate::forms::the_digest_is_blake3_of_length_prefixed_tokens
    - raf_census::test_crate::forms::the_literal_tier_boundary_is_thirty_value_tokens
    - raf_census::test_crate::forms::the_tower_is_monotone_over_every_registry_pair
    - raf_census::test_crate::forms::the_value_adapters_are_the_eleven_of_the_page
    - raf_census::test_crate::forms::the_value_form_keeps_binary_operators_and_drops_borrows
    - raf_census::test_crate::golden::every_fixture_fingerprint_is_pinned
    - raf_census::test_crate::golden::the_manifest_is_pinned
    - raf_census::test_crate::golden::the_pin_check_names_an_unpinned_orphan_moved_and_duplicated_entry
    - raf_census::test_crate::golden::the_pinned_version_is_the_implemented_one
    - raf_census::test_crate::groups::a_constant_in_one_package_is_not_shared
    - raf_census::test_crate::groups::a_block_after_a_constant_in_a_head_is_a_block_and_its_locals_are_renamed
    - raf_census::test_crate::groups::a_forwarder_at_min_tokens_joins_a_shim_group_and_never_a_body_group
    - raf_census::test_crate::groups::a_function_holding_a_table_is_reported_by_its_symbol
    - raf_census::test_crate::groups::a_job_classifies_home_entry_point_variant_and_copies
    - raf_census::test_crate::groups::a_job_that_matches_nothing_has_an_empty_list
    - raf_census::test_crate::groups::a_real_struct_literal_keeps_its_field_names
    - raf_census::test_crate::groups::a_shim_id_matches_the_units_that_group_under_it_and_not_the_excluded_ones
    - raf_census::test_crate::groups::bodies_that_apply_different_functions_share_no_group_but_renamed_locals_do
    - raf_census::test_crate::groups::bodies_that_read_different_lowercase_statics_share_no_group
    - raf_census::test_crate::groups::bodies_under_min_tokens_never_group
    - raf_census::test_crate::groups::constants_above_ffff_in_two_packages_are_shared
    - raf_census::test_crate::groups::copies_that_differ_by_erased_noise_group_across_the_literal_tier
    - raf_census::test_crate::groups::copies_that_reorder_their_parameters_share_a_body_group
    - raf_census::test_crate::groups::every_form_of_a_hex_table_is_reported_in_both_cases
    - raf_census::test_crate::groups::forwarders_keep_their_argument_order_in_the_shim_key
    - raf_census::test_crate::groups::groups_of_one_are_dropped_and_members_follow_symbol_order
    - raf_census::test_crate::groups::groups_order_by_count_then_tokens_then_fingerprint
    - raf_census::test_crate::groups::jobs_match_shim_and_table_fingerprints
    - raf_census::test_crate::groups::near_misses_of_a_hex_table_are_not_reported
    - raf_census::test_crate::groups::shared_constants_run_by_value_ascending
    - raf_census::test_crate::groups::shim_groups_use_the_shim_key_and_skip_trait_impls_and_self_forwarders
    - raf_census::test_crate::page::the_shape_table_is_the_reader_shape
    - raf_census::test_crate::page::the_page_names_every_pass_level_and_the_encoding
    - raf_census::test_crate::page::the_page_states_the_closed_shape_of_every_report
    - raf_census::test_crate::page::the_page_names_every_threshold_with_its_value
    - raf_census::test_crate::page::the_page_names_every_value_adapter_and_weak_keyword
    - raf_census::test_crate::page::the_page_states_that_every_threshold_counts_the_value_form
    - raf_census::test_crate::page::the_page_states_the_renaming_and_numbering_rules
    - raf_census::test_crate::reports::a_shared_constant_is_an_integer
    - raf_census::test_crate::reports::an_absolute_directory_reports_the_same_as_the_relative_spelling
    - raf_census::test_crate::reports::unknown_terms_are_not_a_report
    - raf_census::test_crate::reports::decoded_control_characters_are_escaped_in_report_diagnostics
    - raf_census::test_crate::reports::orphan_units_and_matches_without_a_named_reifier_are_not_a_report
    - raf_census::test_crate::reports::both_reports_read_back_what_they_write
    - raf_census::test_crate::reports::output_is_the_canonical_turtle
    - raf_census::test_crate::reports::the_capability_examples_hold_the_example_output
    - raf_census::test_crate::reports::the_example_output_is_the_pinned_golden
    - raf_census::test_crate::reports::the_example_output_names_the_expected_group_fingerprints
    - raf_census::test_crate::reports::the_index_renders_the_classified_matches_of_every_job
    - raf_census::test_crate::reports::two_fresh_loads_of_the_fixtures_give_identical_bytes
    - raf_census::test_crate::reports::two_fresh_loads_of_this_repository_give_identical_bytes
    - raf_census::test_crate::units::a_constant_whose_type_names_a_number_does_not_hold_it
    - raf_census::test_crate::units::a_deep_constant_type_loads_on_a_small_stack
    - raf_census::test_crate::units::a_deep_signature_loads_on_a_small_stack_and_beyond_the_budget_is_refused
    - raf_census::test_crate::units::a_module_declared_by_a_library_and_a_binary_is_one_unit
    - raf_census::test_crate::units::a_self_call_at_any_depth_removes_the_shim_key
    - raf_census::test_crate::units::a_support_module_shared_by_two_test_binaries_is_one_unit_that_does_not_ship
    - raf_census::test_crate::units::a_turbofish_call_is_a_forwarder_and_keeps_its_key_beside_a_twin
    - raf_census::test_crate::units::a_unit_is_found_by_symbol
    - raf_census::test_crate::units::a_workspace_without_roots_does_not_load
    - raf_census::test_crate::units::byte_strings_and_c_strings_are_strings
    - raf_census::test_crate::units::self_type_arguments_keep_shim_keys_beside_generic_self_callees
    - raf_census::test_crate::units::each_kind_yields_one_unit_with_every_field
    - raf_census::test_crate::units::every_spelling_of_an_integer_is_one_constant
    - raf_census::test_crate::units::file_context_names_package_path_and_shipping
    - raf_census::test_crate::units::file_context_names_the_target_kinds_that_reach_the_file
    - raf_census::test_crate::units::hex_tables_are_read_from_strings_and_arrays
    - raf_census::test_crate::units::loading_one_tree_twice_preserves_the_callers_span_storage_and_unit_results
    - raf_census::test_crate::units::caller_spans_survive_a_successful_census_load
    - raf_census::test_crate::units::caller_spans_survive_a_refused_census_load
    - raf_census::test_crate::units::loading_a_tree_with_caller_local_state_needs_no_sync_bound
    - raf_census::test_crate::units::a_held_standalone_tower_keeps_its_forms_and_print_across_census_loads
    - raf_census::units::tests::collection_worker_tears_down_on_normalization_success_and_error_beside_a_preserved_caller_span
    - raf_census::units::tests::collection_worker_panic_keeps_its_payload_and_tears_down_without_invalidating_caller_spans
    - raf_census::test_crate::units::max_file_bytes_caps_a_file_and_the_cap_itself_loads
    - raf_census::test_crate::units::min_tokens_counts_the_value_form_so_erased_noise_never_straddles_it
    - raf_census::test_crate::units::min_tokens_decides_the_body_fingerprint_at_the_default_and_at_ten
    - raf_census::test_crate::units::parameters_of_every_pattern_are_renamed
    - raf_census::test_crate::units::shim_keys_are_absent_for_trait_impl_methods_and_self_forwarders
    - raf_census::test_crate::units::shipping_is_true_when_any_reach_ships_and_the_shipping_path_is_the_symbol
    - raf_census::test_crate::units::strings_carry_their_absolute_lines
    - raf_census::test_crate::units::test_code_of_a_shipping_target_does_not_ship
    - raf_census::test_crate::units::this_repositorys_own_crates_load_into_distinct_units
    - raf_census::version::tests::other_versions_are_refused_naming_both_and_the_migration
    - raf_census::version::tests::the_implemented_version_is_one
    - raf_census::version::tests::the_implemented_version_passes
    - raf_census::test_crate::forms::the_names_of_an_attribute_are_kept_beside_a_local_of_the_same_name
    - raf_census::test_crate::groups::renamed_locals_inside_macro_arguments_land_in_one_body_group
    - raf_census::test_crate::reports::missing_repeated_and_mistyped_properties_are_not_a_report
    - raf_census::test_crate::reports::names_that_disagree_with_their_node_are_not_a_report
    - raf_census::test_crate::reports::constructs_outside_the_profile_are_not_a_report
    - raf_census::test_crate::reports::an_edited_or_respaced_text_is_not_a_report
    - raf_census::test_crate::reports::canonical_line_diagnostics_preserve_crlf_and_the_final_newline
    - raf_census::test_crate::reports::an_index_is_not_a_baseline
    - raf_census::test_crate::reports::duplicate_job_ids_are_refused
    - raf_census::test_crate::reports::a_report_of_another_report_form_is_refused_naming_both
    - raf_census::test_crate::reports::an_index_of_another_normal_form_reads_and_keeps_its_version
    - raf_census::test_crate::reports::awkward_names_round_trip
    - raf_census::test_crate::reports::two_cfg_alternatives_on_one_line_have_distinct_unit_nodes
    - raf_census::test_crate::reports::the_writer_refuses_a_value_outside_the_shape
  examples:
    - raf_census::example_crate::baseline
    - raf_census::example_crate::goldens
    - raf_census::example_crate::selection
depends: [cap:source-index]
---

# Census normal form, fingerprints and grouping

## Purpose

The census finds helpers that a workspace implements more than once. Textual
comparison misses most duplicates, because two copies of one algorithm differ in
local names, borrows and clones. The normal form rewrites a function body into a
canonical token sequence, and the fingerprint names that sequence, so equal
algorithms get equal fingerprints. This capability is the foundation every other
census capability stands on.

The input boundary is a supplied `Tree`; normalization, fingerprinting, grouping
and report conversion are deterministic functions of its admitted source and owned
values. This core opens no filesystem paths and starts no processes itself.

The census is a library: a caller loads a workspace through `raf-source`, and the
census reports which units share a fingerprint and which of them a ledger job
forbids outside its home. The fingerprint is only comparable inside one normal-form
version, so the library exposes the version and an explicit gate callers apply
before comparing fingerprints. Reading a report preserves its declared version.

## Contract

The normative definition of the normal form is
[the architecture page](../architecture/normal-form.md). This capability
implements it as follows.

- The census implements `normal_form = 1`. A body is lexed with `proc_macro2`,
  including the arguments of macro invocations, and rewritten by four passes
  (`rename_params`, `rename_locals`, `kind_literals`, `erase_held`) into its shim
  form, its structural form and its value form. Integer literals are respelled to
  decimal at lex time, so every spelling of one value is one token.
- The value form drops borrows, derefs, `ref` and `ref mut` bindings, the value adapters listed
  in `VALUE_ADAPTERS` (`as_str`, `as_ref`, `as_slice`, `as_mut`, `as_deref`,
  `borrow`, `borrow_mut`, `clone`, `cloned`, `copied`, `to_owned`), full-range
  reindexing, `let` type annotations and type-only turbofish.
- `rename_locals` renames only the names that the body or its signature binds
  (the patterns of `let`, `if let`, `while let`, `for` and `match` arms, closure
  parameters, local functions and types, labels, lifetimes, macro metavariables and
  named macro arguments) and every use of them, and numbers them together with the
  parameters by order of first appearance. Every other identifier is the name of an
  item and is kept: `map(double)` and `map(triple)` are different bodies, as are two
  reads of different statics. A name in field position is kept, `S { x }` included.
  The block after the head of an `if`, `while`, `match` or `for` is never a struct
  literal. The shim level keeps the signature order of the parameters.
- A body whose value form has fewer than 30 tokens keeps its literals verbatim (the
  literal tier). A longer body has its literals abstracted. Every threshold counts
  the tokens of the value form, delimiters included, and the count is taken before
  the literals are abstracted, so noise the value form erases never moves a body
  across a threshold.
- A body is a shim candidate when it is one expression, judged at the top level of
  the body only (no `let`, no `;` before its tail; a closure inside an argument may
  hold both), contains a call, does not mention `self` and has 6 to 29 value
  tokens; its shim form renames its own parameters in signature order.
  `MIN_SHIM_TOKENS` is 6 and `SHIM_MAX_TOKENS` is 30. A shim candidate keeps its `body` fingerprint when it
  reaches `min_tokens`, which the index lists, but it never joins a body group:
  only its `shim` fingerprint groups, and only when its `shim_key` is present. The
  `shim_key` is absent for a method of a trait implementation and for a forwarder
  that calls through `Self::`, a call at any depth whose callee path starts with
  `Self::`. An associated type inside generic arguments is not a callee; generic
  Self calls still count when their arguments include lifetimes or function types.
- A body with fewer than `min_tokens` value tokens has no body fingerprint.
  `min_tokens` defaults to 20 and `CensusOptions::min_tokens` sets it; the ledger key
  that carries it is the ledger capability's.
- A fingerprint is `<kind>:` followed by an identifier. For `body` and `shim` the
  identifier is the 64 lowercase hexadecimal digits of the complete 32-byte BLAKE3
  digest under `tokens-len-u32le-utf8/blake3/hex64`. Canonical display, parsing,
  equality, grouping and report IRIs all use the complete digest.
  `Fingerprint::abbreviated` supplies a human-only sixteen-digit prefix with an
  ellipsis; it cannot be parsed as an identity. For
  `table` and `rule` it is a fixed name. The kinds are `body`, `shim`, `table` and
  `rule`.
- BLAKE3 and hexadecimal encoding use `purrdf-hash`'s shared native primitives.
  The hash kernel has no external dependencies or C build requirement. Census
  shares the pinned native lexical package used by the document reader rather
  than loading a second registry identity. Its token encoding is
  independent of pointer width and endianness: each UTF-8 token has a little-endian
  `u32` byte length before its bytes. `cargo metadata` discovery for reference
  packages belongs to the host reference-scan layer; this core never invokes it.
- `raf_census::NORMAL_FORM` is 1. `raf_census::check_normal_form(declared)`
  accepts 1 and refuses every other version with `Error::Version`, whose message
  names the declared version, the implemented version and `cargo census migrate`.
  Reading a version out of a ledger file is the ledger's.
- `raf_census::CensusOptions` is `#[non_exhaustive]` and has `Default` and two
  setters. `min_tokens` defaults to 20, and the setter refuses 0 with
  `Error::Options`. `max_file_bytes` defaults to 8 MiB and is the per-file read cap
  of the workspace load; a file above it is a load error that names the file.
- `raf_census::Error` is the one error type: `Source` (file, line and reason of a
  slice that does not lex, text of an item that the workspace does not hold, or a
  source item of the index whose item does not resolve or that has no reach: such an
  inconsistency is an error that names the file and never drops a unit),
  `Options`, `Version`, `Report` (a typed `ReportError`), a load error of `raf-source`,
  and `CollectionSpawn` with its I/O source when the collection worker cannot start.
  A collection-worker panic resumes on the caller with the original payload after
  worker teardown. Nothing falls back to a partial result.
- `raf_census::FileCtx` describes one file of the workspace: its `package` (of
  the first target that reaches it), its `path`, `shipping`, and `target_kinds`, the
  set of raf-source `TargetKind` values (library, binary, test, bench, example,
  build script) of every target whose module tree reaches the file, test code
  included. A file of a library has `{Lib}`, a file under `tests/` has `{Test}`
  and a file under `examples/` has `{Example}`; a module that two targets declare
  has both kinds. `shipping` stays the question of whether shipped code reaches it.
- Units are collected once per source item, whatever number of targets reach it,
  so a module declared by both a library and a binary yields one unit. A unit is
  a function, a method (inherent, trait implementation or trait default), a
  constant or a static; its identity is its file, line, start byte and symbol, its symbol is
  chosen deterministically (a shipping reach over a test reach, then the smallest
  path), and it is shipping when some target that reaches it ships and the item is
  not test code in that reach. A function or method unit holds its print; a constant
  or static has no print and no fingerprint. A unit holds its integer constants by
  value, its string literals with their lines (string, byte string and C string)
  and the tables it holds.
- The census groups units by fingerprint in a `BTreeMap` from fingerprint to
  members. A group has two or more members. Groups sort by member count descending,
  then tokens descending, then fingerprint ascending, and their members by symbol,
  file, line and start byte, so the same source always produces the same output. Body and shim prints each hold a single complete fingerprint,
  without a redundant truncated identity or separate digest.
- Integer constants are compared by value in any spelling. The census recognises
  hex-digit tables (a string, a byte string or an array of 16 `char` or `u8`
  literals that is exactly the sixteen digits in one case) and reports them as
  `table:hex-lower` and `table:hex-upper`. A value above `0xffff` that units of two
  or more packages hold is a shared constant.
- A job (`raf_census::Job`) has an id, a home symbol, entry-point symbols, variant
  symbols and a forbidden set of fingerprints and integer constants.
  `Census::match_jobs` returns one report per job. A unit matches when its `body`
  fingerprint, its `shim_key` or its `table:` id is a forbidden fingerprint, or when
  one of its integer constants equals a forbidden constant by value. A match is
  `Home` when the unit's symbol is the home or an entry point, `Variant` when it is a
  listed variant, and `Copy` otherwise, and the job counts its copies. A `rule:`
  fingerprint matches no unit here.
- The baseline and index are ground RDF 1.2 graphs in one canonical Turtle 1.2
  spelling, with the closed shape in the architecture page's Reports section.
  A content-addressed BASE separates snapshot-local nodes; its BLAKE3-256 digest
  covers the document without the BASE line. This is an unkeyed integrity check,
  refusing an edit whose digest was not recomputed. Each unit's identity includes
  symbol, file, line and start byte. Job matches have named, typed RDF 1.2 reifiers
  carrying Home, Variant or Copy. Constants are decimal xsd:integer values.
  The shared purrdf-lex canonical RDF 1.2 term spelling escapes literals and IRIs.
  Readers return the declared normal form without gating it, while enforcing
  reportForm 1 and matching fingerprint namespaces. Unknown terms, missing,
  repeated and mistyped properties, dangling references, orphan units and node
  names that disagree with properties are errors. Alternate spellings and an
  incorrect BASE digest are errors. Writers validate the same closed shape.
- A normal form version is fixed by the content of its first merge to the default
  branch. After that, any change that can alter a fingerprint is a new version with
  a migration path.

## Usage

A caller can keep its own proc-macro spans and standalone normalizer forms while
loading a census. The supplied Tree may hold caller-local state; collection shares
only the already loaded workspace.

A caller loads the workspace with `raf_census::Census::load(&tree, roots,
&options)`, normalises single sources with `raf_census::Normalizer`, and asks
`Census::match_jobs` which units a set of `Job` values forbids. `Census::baseline`
and `Census::index` build the two reports. `Census::index` refuses duplicate job
ids; its jobs sort by id. `to_turtle` writes a report and `from_turtle` reads its
canonical bytes, preserving its normal-form version for the caller to judge. `check_normal_form` is the gate a ledger
reader applies to the version it finds. `cargo census` ([census-cli](./census-cli.md))
and the rule API ([census-rule-api](./census-rule-api.md)) are the designed
consumers of this library.

A fingerprint is used in two places. A ledger job lists it under
`forbidden.fingerprints` so that a copy outside the job's home fails the gate (see
[census-ledger](./census-ledger.md)). A reader compares fingerprints in a baseline
to see which functions the census considers the same.

`Normalizer::explain` compares two sources level by level and names the coarsest
level at which they still differ and the first differing token with its line, which
is how a reader learns why two bodies that look alike are reported as distinct.

`Tower::literal_values` computes retained and abstracted literal-value forms on
demand from the existing literal-tier form, with the production kind-literal and
held-value passes. The ordinary load computes only its selected form. The
`selection` example counts alternative selection policies over these forms; it
does not mint fingerprints for candidate policies or change production defaults.

## Examples

`fingerprint.to_string()` produces a complete machine identity;
`fingerprint.abbreviated()` produces human text such as `body:6bb022e241a79a28…`.
The latter is intentionally refused by the canonical parser.

These two functions are one algorithm in the value form, and the census puts them
in one `body` group:

```rust
fn total(items: &[u64]) -> u64 {
    let mut sum = 0;
    for item in items.iter() {
        sum += *item;
        if sum > 1_000_000 { sum = 1_000_000; }
    }
    sum
}

fn add_up(values: &[u64]) -> u64 {
    let mut acc: u64 = 0;
    for v in values.iter() {
        acc += *v;
        if acc > 1_000_000 { acc = 1_000_000; }
    }
    acc
}
```

The `baseline` example of the crate prints both reports of a built-in workspace
(`cargo run -p raf-census --example baseline`); given a directory it prints the
reports of that directory, with no jobs. The workspace has one body group (`alpha`
and `beta`), one shim group (`one` and `two`, forwarders to `helper`), a
hexadecimal digit table, the constant 65536 and one job `sum` whose home is
`alpha` and which forbids the body group's fingerprint and the constant 65536. The
baseline report:

```turtle
VERSION "1.2"
BASE <https://blackcatinformatics.ca/.well-known/genid/raf-census/e7bb9450ced74ef9d1a19befcf616049697ba1515baf78dbbcdd7974bca6e284>
PREFIX body: <https://blackcatinformatics.ca/raf/census/nf/1/body/>
PREFIX census: <https://blackcatinformatics.ca/raf/census#>
PREFIX shim: <https://blackcatinformatics.ca/raf/census/nf/1/shim/>
PREFIX table: <https://blackcatinformatics.ca/raf/census/table/>

<> a census:Baseline ;
    census:minTokens 20 ;
    census:normalForm 1 ;
    census:reportForm 1 ;
    census:unitCount 7 .

<#group/body:6bb022e241a79a281e80d6e233c0b8f61a027761c52bd4b0c8b232de92544609> a census:BodyGroup ;
    census:fingerprint body:6bb022e241a79a281e80d6e233c0b8f61a027761c52bd4b0c8b232de92544609 ;
    census:member <#unit/demo::alpha@src/lib.rs:1:0> ,
        <#unit/demo::beta@src/lib.rs:2:74> ;
    census:tokens 21 .

<#group/shim:203616cb54a3e53de4ca5100ce3cf2aad3b59e9421fc50c09bc93deb7c974725> a census:ShimGroup ;
    census:fingerprint shim:203616cb54a3e53de4ca5100ce3cf2aad3b59e9421fc50c09bc93deb7c974725 ;
    census:member <#unit/demo::one@src/lib.rs:4:194> ,
        <#unit/demo::two@src/lib.rs:5:237> ;
    census:tokens 6 .

<#unit/demo::alpha@src/lib.rs:1:0> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 1 ;
    census:startByte 0 ;
    census:symbol "demo::alpha" .

<#unit/demo::beta@src/lib.rs:2:74> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 2 ;
    census:startByte 74 ;
    census:symbol "demo::beta" .

<#unit/demo::digits@src/lib.rs:6:280> a census:Unit ;
    census:file "src/lib.rs" ;
    census:holds table:hex-lower ;
    census:line 6 ;
    census:startByte 280 ;
    census:symbol "demo::digits" .

<#unit/demo::one@src/lib.rs:4:194> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 4 ;
    census:startByte 194 ;
    census:symbol "demo::one" .

<#unit/demo::two@src/lib.rs:5:237> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 5 ;
    census:startByte 237 ;
    census:symbol "demo::two" .
```

The index report of the same workspace lists each unit with its `body` and `shim`
fingerprints, each group with its members, and the job with its classified matches:

```turtle
VERSION "1.2"
BASE <https://blackcatinformatics.ca/.well-known/genid/raf-census/8b9e664aaabca3c67de1cdc78b74fdc6d30ee95ec904041a3b13707d0c8a9b37>
PREFIX body: <https://blackcatinformatics.ca/raf/census/nf/1/body/>
PREFIX census: <https://blackcatinformatics.ca/raf/census#>
PREFIX shim: <https://blackcatinformatics.ca/raf/census/nf/1/shim/>

<> a census:Index ;
    census:normalForm 1 ;
    census:reportForm 1 .

<#group/body:6bb022e241a79a281e80d6e233c0b8f61a027761c52bd4b0c8b232de92544609> a census:BodyGroup ;
    census:fingerprint body:6bb022e241a79a281e80d6e233c0b8f61a027761c52bd4b0c8b232de92544609 ;
    census:member <#unit/demo::alpha@src/lib.rs:1:0> ,
        <#unit/demo::beta@src/lib.rs:2:74> ;
    census:tokens 21 .

<#group/shim:203616cb54a3e53de4ca5100ce3cf2aad3b59e9421fc50c09bc93deb7c974725> a census:ShimGroup ;
    census:fingerprint shim:203616cb54a3e53de4ca5100ce3cf2aad3b59e9421fc50c09bc93deb7c974725 ;
    census:member <#unit/demo::one@src/lib.rs:4:194> ,
        <#unit/demo::two@src/lib.rs:5:237> ;
    census:tokens 6 .

<#job/sum> a census:Job ;
    census:copies 2 ;
    census:homeSymbol "demo::alpha" ;
    census:jobId "sum" ;
    census:match <#unit/demo::BIG@src/lib.rs:7:335> ~ <#match/sum/demo::BIG@src/lib.rs:7:335> {| a census:Match ; census:matchClass census:Copy |} ,
        <#unit/demo::alpha@src/lib.rs:1:0> ~ <#match/sum/demo::alpha@src/lib.rs:1:0> {| a census:Match ; census:matchClass census:Home |} ,
        <#unit/demo::beta@src/lib.rs:2:74> ~ <#match/sum/demo::beta@src/lib.rs:2:74> {| a census:Match ; census:matchClass census:Copy |} .

<#unit/demo::BIG@src/lib.rs:7:335> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 7 ;
    census:package "demo" ;
    census:shipping true ;
    census:startByte 335 ;
    census:symbol "demo::BIG" ;
    census:tokens 0 .

<#unit/demo::alpha@src/lib.rs:1:0> a census:Unit ;
    census:body body:6bb022e241a79a281e80d6e233c0b8f61a027761c52bd4b0c8b232de92544609 ;
    census:file "src/lib.rs" ;
    census:line 1 ;
    census:package "demo" ;
    census:shipping true ;
    census:startByte 0 ;
    census:symbol "demo::alpha" ;
    census:tokens 21 .

<#unit/demo::beta@src/lib.rs:2:74> a census:Unit ;
    census:body body:6bb022e241a79a281e80d6e233c0b8f61a027761c52bd4b0c8b232de92544609 ;
    census:file "src/lib.rs" ;
    census:line 2 ;
    census:package "demo" ;
    census:shipping true ;
    census:startByte 74 ;
    census:symbol "demo::beta" ;
    census:tokens 21 .

<#unit/demo::digits@src/lib.rs:6:280> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 6 ;
    census:package "demo" ;
    census:shipping true ;
    census:startByte 280 ;
    census:symbol "demo::digits" ;
    census:tokens 1 .

<#unit/demo::helper@src/lib.rs:3:147> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 3 ;
    census:package "demo" ;
    census:shipping true ;
    census:startByte 147 ;
    census:symbol "demo::helper" ;
    census:tokens 3 .

<#unit/demo::one@src/lib.rs:4:194> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 4 ;
    census:package "demo" ;
    census:shim shim:203616cb54a3e53de4ca5100ce3cf2aad3b59e9421fc50c09bc93deb7c974725 ;
    census:shipping true ;
    census:startByte 194 ;
    census:symbol "demo::one" ;
    census:tokens 6 .

<#unit/demo::two@src/lib.rs:5:237> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 5 ;
    census:package "demo" ;
    census:shim shim:203616cb54a3e53de4ca5100ce3cf2aad3b59e9421fc50c09bc93deb7c974725 ;
    census:shipping true ;
    census:startByte 237 ;
    census:symbol "demo::two" ;
    census:tokens 6 .
```

A directory is read as a root of the tree at the current directory when it is
relative; an absolute directory is read as its parent's tree, and the file paths of
the report then begin with the directory's name, as they do for `crates`.

## Constraints

- The census reads Rust source only. Generated code and code behind macros that
  expand to items are compared as written, not as expanded.
- Macro arguments are re-lexed as token streams and compared as tokens. A file
  whose macro arguments do not lex cannot reach the census, because `raf-source`
  must parse the file; a slice that does not lex is an error that names the file and
  line.
- A doc comment inside a body is the attribute tokens `# [ doc = "text" ]`, so its
  text counts; an ordinary comment is not a token.
- A body shorter than `min_tokens` is never a `body` group, however many copies
  exist. Forwarders of 6 to 29 tokens are caught by the shim form instead.
- Body and shim fingerprints carry all 256 BLAKE3 bits. Identity assumes the
  collision resistance of BLAKE3; this is not a proof that two token sequences
  are equal. A ledger also records member symbols. Human abbreviated
  prefixes provide no collision-resistance guarantee.
- The `table` and `rule` kinds carry names, not hashes.
- Portable hashing alone does not make the complete source dependency portable.
  Census loads remain coupled to `raf-source` target support; a hash-only wasm
  build does not establish that a complete load builds or runs there. Source reads
  and containment remain the supplied `Tree`'s responsibility.
- Census unit collection uses a fresh native thread. The completed workspace is
  borrowed there, while Tree access stays on the caller's thread without a Send or Sync
  requirement. This ownership boundary is not a thread-free or complete portable-core
  guarantee. Standalone Normalizer and Tower calls still register lexer source on
  their caller thread; their returned forms contain owned text and ranges, not spans.
- A unit that several targets reach is one unit; the census never counts a shared
  module once per target.
- A shim candidate never joins a body group, so a 20-token forwarder is judged by
  its shim form alone.
- A slice that does not lex is an error that names the file and line; the census does
  not skip it. A signature or an item that does not parse never reaches the census:
  `raf-source` refuses the file when it loads.
- A constant or a static has no print and no fingerprint. Its integer constants,
  strings and tables come from its initialiser expression only, never from its type
  or attributes.
- A string literal that is not valid UTF-8 is read lossily and never names a table.
- A name that no pattern, closure, local declaration, label, lifetime, metavariable
  or named macro argument binds is an item name and is kept, so a body that uses an
  item the census cannot see bound is compared by that item's name. A name is bound
  for the whole body, not for one scope: two uses of one name are one name.
- The census parses no signature and no item. The names a function's parameters bind
  and the range of a constant's initialiser are facts of the one parse of
  `raf-source` (`Item::parameter_bindings`, `Item::initializer`), which runs on its
  own 64 MiB stack under its recursion budget; a signature that exceeds the budget
  is an `Error::Load` that names the budget, and no signature of any depth can
  overflow the stack of the caller of `Census::load`, a 2 MiB test thread included.
  What the census itself lexes is a body or an initialiser, whose nesting is
  bounded by `MAX_NESTING` (128) and by the delimiter budget of `raf-source`, which
  the file has already passed. Collection owns a fresh thread with a 2 MiB stack.
  Worker teardown releases its thread-local span table on success, collection error
  and unwinding; caller-owned spans remain valid. No global span invalidation is
  performed by Census::load.

## Explanation

The lexer owns thread-local source storage, while its input Tree may own
caller-local state. Loading that Tree before borrowing the completed workspace in
a collection worker keeps both ownership boundaries explicit. Ending the worker
releases the census's lexical storage without resetting unrelated caller spans.
Only owned text, ranges and fingerprints leave it. This native lifetime decision
does not provide a thread-free execution model.

Seven copies of one helper census had diverged across the Blackcat repositories,
each with its own normal form. A fingerprint recorded in one repository meant
nothing in another, and a change to a normaliser silently moved every ledger row.
Version 1 unifies them from the strongest parts of each: the structural form,
value form and literal tier of the largest copy, and the `shim_key` refinement that
stops trait-implementation methods and `Self::` forwarders from forming false
groups.

Versioning the form is what makes the ledger trustworthy. A fingerprint is only
comparable inside one version, so the version is written in the ledger and the
census refuses a mismatch instead of reporting spurious drift. Golden tests turn
the version-bump rule from a convention into a check: a normaliser change that
moves a fingerprint fails until the version moves with it.

The fixed selection policy is justified by [counted corpus evidence](../census-selection.md),
with machine-readable [RDF counts](../data/census-selection.ttl). A 20-token body
floor removes large small-overload groups before the marginal reduction flattens.
Literal tier 30 is the lowest sampled tier without a labelled distinct merge;
raising it loses labelled equal pairs. Shim bounds 6–29 avoid trivial result
constructors and preserve short fixed-argument forwarding. These are conservative
tradeoffs over written bodies, with no claim of measured real-source precision.
`Tower::literal_values` exposes the production candidate forms on demand; normal
loads retain the single fixed policy.

## Verification

Native hash adoption retains the complete normal-form fingerprints, human
abbreviations, report body digests and selection identities. The hand-framed
digest, immutable normal-form goldens, report fixtures and selection counts
exercise those actual callers without changing the encoding or normal-form
version.

Public loading preserves a live caller span after successful loading and a
workspace-load refusal, and held standalone forms keep their bytes and prints. A
Cell-backed Tree loads without a Sync bound. Repeated loads preserve unit results
and add no caller lexer registrations: the locked lexer's parsed-file ordinals
advance only for the caller probes. File-relative byte offsets are not a measure
of retained storage. The collection-worker adapter runs the real normalizer at its
nesting limit and refuses the next depth, with worker-local TLS destruction
observed on both paths. A panic retains its typed payload and destroys worker TLS
before it resumes on the caller. These are lifetime observations, not RSS or
allocation-byte measurements. Spawn-error formatting retains an I/O error source
and escapes controls beside a plain diagnostic neighbor.

- A count-tool test pins the labelled fixture corpus and relates its default
  selection directly to production prints. Full-suffix identity tests show two
  hashes sharing a human prefix remain distinct; parser refusals escape controls
  beside valid names.
- Golden tests pin the fingerprint of every fixture under `normal_form = 1`, beside
  a manifest of the encoding, the passes, the levels, the thresholds and the word
  lists. They fail when a fixture has no pinned value, when a pinned value has no
  fixture and when a pinned value moves. Every form has fixtures: structural,
  value, literal tier, shim and macro arguments.
- Fixture pairs differing only in bound names, parameter names, parameter order,
  borrows, clones and a `let` type annotation fingerprint equal, with the level at
  which they first become equal registered; there is one pair for each form of
  binder (`let`, a tuple or struct pattern, a closure parameter, `for`, a `match`
  arm, `if let`, `while let`, a `name @ pattern` binding, a label, a lifetime, a local function and a local
  type, a call to a bound closure, a bound name that shadows an item). Pairs
  differing in a method name, a field name, a called function, a function passed by
  name, a lowercase static or a struct field (`S { x }` and `S { y }`) fingerprint
  different. A property test checks that
  equality at a finer level implies equality at every coarser one for slices with
  the same parameter list. `explain` gives the finest level from which every coarser
  level is equal, so the same text under reordered parameters is equal from the
  literal tier and differs through the shim level.
- Pairs whose two sides straddle a threshold only through erased noise fingerprint
  equal: one across the literal tier and one across `min_tokens`, at the default and
  at 10, and the shim bounds count the value form too.
- Loaded workspaces show that bodies applying different functions, or reading
  different lowercase statics, are in no common group beside a renamed-local
  neighbour that does group, that two clamps whose block follows a constant in the
  head are in one group, that copies with reordered parameters are in one group,
  and that forwarders with a different argument order have different shim keys.
- A fixture of two 25-token bodies that differ only in a string literal
  fingerprints different, and the same pair at 35 tokens fingerprints equal.
- Shim fixtures cover the 5, 6, 29 and 30 token boundaries, a two-statement and a
  `let` body without a shim form, a trait-implementation method and a `Self::`
  forwarder each beside a neighbour that does group, a closure body with a `let`
  inside an argument that is still a candidate, and a module reached by two targets
  as one unit.
- Doc comment fixtures show two bodies that differ only in a renamed local, each
  with an inner doc comment, fingerprinting equal, and bodies whose doc comment text
  differs fingerprinting different. A test over a loaded workspace shows two
  functions that differ only in locals renamed inside `format!`, `vec!` and
  `assert_eq!` arguments in one body group, beside a neighbour that calls another
  method and is not in it.
- A fixture with a `0x`-spelled and a decimal-spelled constant yields one constant
  value.
- `check_normal_form` is tested with 0 and 2 (refused, the message naming both
  versions and `cargo census migrate`) and with 1 (accepted). `CensusOptions` is
  tested with `min_tokens` 0 (refused) and 1 (accepted), with the defaults, and with
  a file above and a file at `max_file_bytes`.
- Grouping tests show groups of one are dropped, the order of groups and members,
  the hex tables of each form and case beside the near misses that are not tables,
  and the shared constants across packages.
- A job fixture over one workspace yields a `Home` for the home, a `Home` for an
  entry point, a `Variant`, a `Copy`, and a `Copy` for a unit holding the forbidden
  constant by value, with a copy count of 2, and a job that matches nothing has no
  matches.
- The reports are tested for byte-identical output over two fresh loads, including
  this repository's own crates, and for canonical Turtle 1.2 round trips. Negative
  fixtures refuse unknown predicates/classes, missing, repeated and mistyped
  properties, inconsistent matches, node names, unsupported constructs, respacing
  and edits without a fresh digest; the canonical neighbours read. Independent
  purrdf-rdf expansions pin the graph as N-Triples, including awkward escaped names.
  The Examples above are the exact output of the `baseline` example.
- A page test reads the architecture page and fails when it does not name every
  pass, level, threshold with its value, value adapter, closed report shape and
  encoding id. Every required baseline or index shape property has a missing-property fixture and
  an unknown-property neighbour against the same production shape validator.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_census::error::tests::an_options_error_has_no_nested_source` | test | [Verification](#verification) |
| `raf_census::error::tests::each_variant_renders_its_facts` | test | [Verification](#verification) |
| `raf_census::example_crate::baseline` | example target | [Verification](#verification) |
| `raf_census::example_crate::goldens` | example target | [Verification](#verification) |
| `raf_census::example_crate::selection` | example target | [Verification](#verification) |
| `raf_census::group::tests::digest_suffixes_are_distinct_group_keys` | test | [Verification](#verification) |
| `raf_census::group::tests::members_order_by_symbol_then_file_then_line` | test | [Verification](#verification) |
| `raf_census::normalize::literal::tests::an_integer_beyond_u128_is_still_an_int` | test | [Verification](#verification) |
| `raf_census::normalize::literal::tests::each_literal_kind_has_its_name` | test | [Verification](#verification) |
| `raf_census::normalize::literal::tests::floats_and_strings_are_not_integers` | test | [Verification](#verification) |
| `raf_census::normalize::literal::tests::spellings_of_one_value_respell_alike` | test | [Verification](#verification) |
| `raf_census::options::tests::defaults_are_twenty_tokens_and_eight_mebibytes` | test | [Verification](#verification) |
| `raf_census::options::tests::max_file_bytes_is_set` | test | [Verification](#verification) |
| `raf_census::options::tests::zero_min_tokens_is_refused_and_one_is_accepted` | test | [Verification](#verification) |
| `raf_census::report::mint::tests::fragments_are_ascii_and_percent_encode_reserved_bytes` | test | [Verification](#verification) |
| `raf_census::report::mint::tests::fragments_are_injective_over_their_separators` | test | [Verification](#verification) |
| `raf_census::report::read::tests::each_token_outside_the_profile_names_its_line_and_column` | test | [Verification](#verification) |
| `raf_census::report::read::tests::the_graph_of_each_example_is_its_pinned_triple_set` | test | [Verification](#verification) |
| `raf_census::report::shape::tests::every_required_shape_property_has_a_missing_and_unknown_neighbor` | test | [Verification](#verification) |
| `raf_census::report::write::tests::the_digest_covers_everything_but_the_base_line` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::a_fingerprint_displays_and_parses_back` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::a_fingerprint_refuses_every_malformed_spelling` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::a_fixture_named_for_a_size_has_that_many_value_tokens` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::a_shim_candidate_keeps_its_body_print_and_the_shim_print_differs_by_callee` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::an_unlexable_slice_is_a_source_error_with_a_valid_neighbour` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::below_the_tier_the_final_sequence_keeps_literals_and_above_it_abstracts_them` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::copies_with_reordered_parameters_share_a_body_print_and_forwarders_keep_their_order` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::every_registered_pair_compares_as_registered` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::every_threshold_counts_the_value_form` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::explain_names_the_levels_and_the_first_differing_token_with_its_line` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::explain_reports_reordered_parameters_as_equal_from_the_literal_tier` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::fingerprint_refusals_escape_untrusted_controls_beside_valid_names` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::fingerprints_sort_by_kind_then_value` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::human_abbreviations_do_not_replace_complete_identity` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::literal_value_introspection_selects_the_production_default` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::nesting_at_the_budget_lexes_and_one_deeper_is_refused` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::parameters_and_locals_are_numbered_together_by_first_appearance` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::self_callee_query_distinguishes_generic_types_from_calls` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::self_is_not_a_parameter_and_a_field_named_like_one_is_kept` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::shim_forms_exist_for_six_to_twenty_nine_tokens` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::the_body_print_needs_min_tokens` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::the_digest_is_blake3_of_length_prefixed_tokens` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::the_literal_tier_boundary_is_thirty_value_tokens` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::the_names_of_an_attribute_are_kept_beside_a_local_of_the_same_name` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::the_tower_is_monotone_over_every_registry_pair` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::the_value_adapters_are_the_eleven_of_the_page` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::the_value_form_keeps_binary_operators_and_drops_borrows` | test | [Verification](#verification) |
| `raf_census::test_crate::golden::every_fixture_fingerprint_is_pinned` | test | [Verification](#verification) |
| `raf_census::test_crate::golden::the_manifest_is_pinned` | test | [Verification](#verification) |
| `raf_census::test_crate::golden::the_pin_check_names_an_unpinned_orphan_moved_and_duplicated_entry` | test | [Verification](#verification) |
| `raf_census::test_crate::golden::the_pinned_version_is_the_implemented_one` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::a_block_after_a_constant_in_a_head_is_a_block_and_its_locals_are_renamed` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::a_constant_in_one_package_is_not_shared` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::a_forwarder_at_min_tokens_joins_a_shim_group_and_never_a_body_group` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::a_function_holding_a_table_is_reported_by_its_symbol` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::a_job_classifies_home_entry_point_variant_and_copies` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::a_job_that_matches_nothing_has_an_empty_list` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::a_real_struct_literal_keeps_its_field_names` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::a_shim_id_matches_the_units_that_group_under_it_and_not_the_excluded_ones` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::bodies_that_apply_different_functions_share_no_group_but_renamed_locals_do` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::bodies_that_read_different_lowercase_statics_share_no_group` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::bodies_under_min_tokens_never_group` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::constants_above_ffff_in_two_packages_are_shared` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::copies_that_differ_by_erased_noise_group_across_the_literal_tier` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::copies_that_reorder_their_parameters_share_a_body_group` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::every_form_of_a_hex_table_is_reported_in_both_cases` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::forwarders_keep_their_argument_order_in_the_shim_key` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::groups_of_one_are_dropped_and_members_follow_symbol_order` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::groups_order_by_count_then_tokens_then_fingerprint` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::jobs_match_shim_and_table_fingerprints` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::near_misses_of_a_hex_table_are_not_reported` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::renamed_locals_inside_macro_arguments_land_in_one_body_group` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::shared_constants_run_by_value_ascending` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::shim_groups_use_the_shim_key_and_skip_trait_impls_and_self_forwarders` | test | [Verification](#verification) |
| `raf_census::test_crate::page::the_page_names_every_pass_level_and_the_encoding` | test | [Verification](#verification) |
| `raf_census::test_crate::page::the_page_names_every_threshold_with_its_value` | test | [Verification](#verification) |
| `raf_census::test_crate::page::the_page_names_every_value_adapter_and_weak_keyword` | test | [Verification](#verification) |
| `raf_census::test_crate::page::the_page_states_that_every_threshold_counts_the_value_form` | test | [Verification](#verification) |
| `raf_census::test_crate::page::the_page_states_the_closed_shape_of_every_report` | test | [Verification](#verification) |
| `raf_census::test_crate::page::the_page_states_the_renaming_and_numbering_rules` | test | [Verification](#verification) |
| `raf_census::test_crate::page::the_shape_table_is_the_reader_shape` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::a_report_of_another_report_form_is_refused_naming_both` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::a_shared_constant_is_an_integer` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::an_absolute_directory_reports_the_same_as_the_relative_spelling` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::an_edited_or_respaced_text_is_not_a_report` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::an_index_is_not_a_baseline` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::an_index_of_another_normal_form_reads_and_keeps_its_version` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::awkward_names_round_trip` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::both_reports_read_back_what_they_write` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::canonical_line_diagnostics_preserve_crlf_and_the_final_newline` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::constructs_outside_the_profile_are_not_a_report` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::decoded_control_characters_are_escaped_in_report_diagnostics` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::duplicate_job_ids_are_refused` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::missing_repeated_and_mistyped_properties_are_not_a_report` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::names_that_disagree_with_their_node_are_not_a_report` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::orphan_units_and_matches_without_a_named_reifier_are_not_a_report` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::output_is_the_canonical_turtle` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::the_capability_examples_hold_the_example_output` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::the_example_output_is_the_pinned_golden` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::the_example_output_names_the_expected_group_fingerprints` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::the_index_renders_the_classified_matches_of_every_job` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::the_writer_refuses_a_value_outside_the_shape` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::two_cfg_alternatives_on_one_line_have_distinct_unit_nodes` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::two_fresh_loads_of_the_fixtures_give_identical_bytes` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::two_fresh_loads_of_this_repository_give_identical_bytes` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::unknown_terms_are_not_a_report` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_constant_whose_type_names_a_number_does_not_hold_it` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_deep_constant_type_loads_on_a_small_stack` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_deep_signature_loads_on_a_small_stack_and_beyond_the_budget_is_refused` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_held_standalone_tower_keeps_its_forms_and_print_across_census_loads` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_module_declared_by_a_library_and_a_binary_is_one_unit` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_self_call_at_any_depth_removes_the_shim_key` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_support_module_shared_by_two_test_binaries_is_one_unit_that_does_not_ship` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_turbofish_call_is_a_forwarder_and_keeps_its_key_beside_a_twin` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_unit_is_found_by_symbol` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_workspace_without_roots_does_not_load` | test | [Verification](#verification) |
| `raf_census::test_crate::units::byte_strings_and_c_strings_are_strings` | test | [Verification](#verification) |
| `raf_census::test_crate::units::caller_spans_survive_a_refused_census_load` | test | [Verification](#verification) |
| `raf_census::test_crate::units::caller_spans_survive_a_successful_census_load` | test | [Verification](#verification) |
| `raf_census::test_crate::units::each_kind_yields_one_unit_with_every_field` | test | [Verification](#verification) |
| `raf_census::test_crate::units::every_spelling_of_an_integer_is_one_constant` | test | [Verification](#verification) |
| `raf_census::test_crate::units::file_context_names_package_path_and_shipping` | test | [Verification](#verification) |
| `raf_census::test_crate::units::file_context_names_the_target_kinds_that_reach_the_file` | test | [Verification](#verification) |
| `raf_census::test_crate::units::hex_tables_are_read_from_strings_and_arrays` | test | [Verification](#verification) |
| `raf_census::test_crate::units::loading_a_tree_with_caller_local_state_needs_no_sync_bound` | test | [Verification](#verification) |
| `raf_census::test_crate::units::loading_one_tree_twice_preserves_the_callers_span_storage_and_unit_results` | test | [Verification](#verification) |
| `raf_census::test_crate::units::max_file_bytes_caps_a_file_and_the_cap_itself_loads` | test | [Verification](#verification) |
| `raf_census::test_crate::units::min_tokens_counts_the_value_form_so_erased_noise_never_straddles_it` | test | [Verification](#verification) |
| `raf_census::test_crate::units::min_tokens_decides_the_body_fingerprint_at_the_default_and_at_ten` | test | [Verification](#verification) |
| `raf_census::test_crate::units::parameters_of_every_pattern_are_renamed` | test | [Verification](#verification) |
| `raf_census::test_crate::units::self_type_arguments_keep_shim_keys_beside_generic_self_callees` | test | [Verification](#verification) |
| `raf_census::test_crate::units::shim_keys_are_absent_for_trait_impl_methods_and_self_forwarders` | test | [Verification](#verification) |
| `raf_census::test_crate::units::shipping_is_true_when_any_reach_ships_and_the_shipping_path_is_the_symbol` | test | [Verification](#verification) |
| `raf_census::test_crate::units::strings_carry_their_absolute_lines` | test | [Verification](#verification) |
| `raf_census::test_crate::units::test_code_of_a_shipping_target_does_not_ship` | test | [Verification](#verification) |
| `raf_census::test_crate::units::this_repositorys_own_crates_load_into_distinct_units` | test | [Verification](#verification) |
| `raf_census::units::tests::collection_worker_panic_keeps_its_payload_and_tears_down_without_invalidating_caller_spans` | test | [Verification](#verification) |
| `raf_census::units::tests::collection_worker_tears_down_on_normalization_success_and_error_beside_a_preserved_caller_span` | test | [Verification](#verification) |
| `raf_census::version::tests::other_versions_are_refused_naming_both_and_the_migration` | test | [Verification](#verification) |
| `raf_census::version::tests::the_implemented_version_is_one` | test | [Verification](#verification) |
| `raf_census::version::tests::the_implemented_version_passes` | test | [Verification](#verification) |
<!-- raf:generated:end -->
