<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Source-index claim preparation

Read-only preparation against the issue 16 checkout. No product changes, builds,
tests or measurements were performed. This is a source-to-assertion map, not a
claim that a gate passed. Line anchors refer to the document read in this review;
the quoted insertion text is authoritative if concurrent binding additions move
the lines. No fixes or new test names from the issue 28 checkout are imported.

Use a named clause at each proposed split below, retaining all existing prose.
The heading form is `### Name`, followed by the actual `raf:tested-by` marker.
For example:

```markdown
### Root discovery
<!-- raf:tested-by raf_source::test_crate::root::member_and_virtual_workspace_directories_resolve_without_sources,raf_source::test_crate::root::standalone_excluded_and_nested_packages_have_their_own_roots -->
```

The marker belongs immediately after the new heading. Do not put one token
marker after a large old bold clause and thereby imply all its independent
measurements are proved. `claim_coverage.rs:237` explicitly tests heading markers;
its syntax accepts comma-separated actual bound paths. In the tables, `T::`
means the exact prefix `raf_source::test_crate::`; `R::` means
`raf_source::reach::tests::`. Expand these prefixes in inserted markers.

## Contract: concrete splits and verified assertions

| Proposed clause and insertion anchor | Proposed marker paths | Actual assertion evidence and limits |
|---|---|---|
| Root discovery; before `discover_root(tree, start)` | `T::root::member_and_virtual_workspace_directories_resolve_without_sources`, `T::root::standalone_excluded_and_nested_packages_have_their_own_roots` | `tests/root.rs:17` checks workspace root from four ancestor/start positions without source; line 36 checks excluded/nested/standalone roots and an unlisted-package error. |
| Root membership and explicit workspace; split before `package.workspace` | `T::root::transitive_path_dependencies_are_members`, `T::root::explicit_workspace_requires_a_safe_path_and_agreed_membership` | `root.rs:58` checks implicit path-dependency membership; line 74 checks included explicit workspace, sibling workspace, missing membership, invalid workspace shape and above-tree path. |
| Root discovery failures; before `Malformed manifests` | `T::root::malformed_manifests_and_missing_directories_fail_hard`, `T::root::manifest_size_bound_is_enforced_beside_the_exact_limit` | `root.rs:114` refuses malformed manifests, a missing directory and a file start. Line 136 accepts exact default size and refuses one extra byte as Tree TooLarge. Directory-alias and safely reduced parent handling need the separate gap below. |
| Relative path grammar; split Tree before `The trait has four calls` | `T::relpath::backslash_nul_and_drive_letter_inputs_are_refused_beside_plain_names`, `T::relpath::join_extends_a_path_and_checks_the_climb_against_the_whole`, `T::relpath::a_parent_segment_above_the_root_is_refused_beside_the_one_that_stays_inside`, `T::relpath::an_absolute_input_is_refused_beside_its_relative_form` | `relpath.rs:55` matches exact Backslash/Nul/DriveLetter variants and accepts neighboring colon names; line 73 checks dot normalization, retained parent, exact climb, one-too-far and absolute refusal. |
| File identity and seen reads; before `Two paths have equal identities` | `T::memory_tree::the_identity_of_a_file_is_its_normalised_key_and_refuses_what_read_refuses`, `T::memory_tree::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen`, `T::disk_tree_containment::the_identity_of_a_file_is_the_device_and_inode_of_its_verified_handle`, `T::disk_tree_containment::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen` | `memory_tree.rs:177` compares normalized identity, distinct file, directory/missing refusals; line 198 checks Read identity/text, Seen with zero cap, TooLarge when unseen. `disk_tree_containment.rs:320,361` checks symlink/hard-link identity, distinct inode, outside refusal, actual text/identity and seen-cap bypass. Does not prove inode stability on arbitrary mounts or every possible race. |
| Tree errors; before `A failure is a TreeError` | `T::tree_error::every_variant_names_its_path_in_display_and_in_path`, `T::tree_error::only_io_has_a_source_and_only_not_found_is_absence` | `tree_error.rs:35` checks path and display for every constructed variant; line 43 checks source presence only for Io. The latter test's name does not itself prove every absence branch: its body only checks source/display. |
| Outside path containment; retain bold Path containment opening | `T::disk_tree_containment::a_symlink_to_a_file_outside_is_refused_beside_an_in_root_symlink_that_is_accepted`, `T::disk_tree_containment::a_parent_segment_after_an_outside_directory_symlink_is_resolved_on_disk_and_refused` | `disk_tree_containment.rs:38` checks OutsideRoot plus accepted relative/absolute in-root symlinks; line 78 checks a symlink/parent escape, accepted real-directory parent and entry refusal. Platform-specific open/no-read guarantees need strategy and swap tests, not these two alone. |
| Token count and refusal; before `Tokens:` | `T::load_budgets::a_file_at_the_token_limit_parses_and_one_more_token_is_refused_naming_its_line`, `T::load_budgets::each_costly_shape_at_the_token_limit_loads_and_one_more_token_is_refused`, `T::load_budgets::a_doc_comment_counts_as_the_tokens_of_its_attribute_and_other_comments_count_nothing` | `load_budgets.rs:135` asserts exact token usage, typed FileTokens/limit/file/line and default parse_source refusal; line 175 repeats exact/one-over for four costly shapes; line 194 checks 7/6 doc-comment counts and ordinary comments ignored. These do not measure peak allocation. |
| Default byte and work budgets; before `Four byte budgets and a work budget` | `T::load_budgets::the_default_budgets_are_the_stated_limits`, `T::load_budgets::usage_reports_what_a_load_spent` | `load_budgets.rs:421` pins all byte limits, 2 million tokens, 2^31 work and 128 directory depth. Line 443 checks exact manifest/source bytes, nonzero item/fact bytes, 7 tokens, recursion 4, directory depth 2 and zero work without globs. It only proves arithmetic for the 1664 constant, not the universal memory bound. |
| Retained index budget; before `max_item_bytes` | `T::load_budgets::index_records_at_the_item_budget_load_and_one_byte_less_is_refused` | `load_budgets.rs:322` accepts measured charge exactly, refuses charge minus one with typed ItemBytes, matching limit/file/line. Source review of layouts and allocator tests remain necessary for a universal upper-bound claim. |
| Glob retained bytes; before `max_glob_bytes` | `T::load_budgets::glob_tables_at_the_glob_budget_load_and_one_byte_less_is_refused` | `load_budgets.rs:636` checks positive charge, exact success and actual definition identity, minus-one GlobBytes refusal with glob line, zero retained glob charge for no-glob neighbor. |
| Glob work refusal; before `max_work` | `T::load_budgets::glob_table_work_at_the_work_budget_loads_and_one_step_less_is_refused` | `load_budgets.rs:663` pins 10 steps for this checkout's small chain, accepts exact, refuses 9 with Work/file/line/display and allows no-glob load at zero. Do not reuse this pin after integrating the separately evolving resolver. |
| Item order and equality; before `items_under()` / final sorted-output paragraph | `T::symbol_index::two_loads_are_equal_and_items_and_targets_come_in_strictly_increasing_order` | `symbol_index.rs:804` compares two entire workspaces; asserts strictly increasing item tuple and target tuple; checks exact module prefix result. |
| Source groups; before `The identity of an item is its file` | `T::symbol_index::items_that_share_a_source_item_are_grouped_by_its_identity` | `symbol_index.rs:939` checks two target-local helper sites have the same source key and different paths, exact same_source list, ordered groups and complete non-root coverage. |
| Byte spans; split before `Line ranges are 1-based` | `T::symbol_index::items_on_one_long_line_of_wide_characters_get_exact_byte_offsets`, `T::source_text::a_one_line_item_has_the_half_open_lines_n_to_n_plus_one` | `symbol_index.rs:993` generates 20,000 wide-name functions and checks sampled exact start/body byte offsets and `1..2` lines. Source-text test supplies the direct half-open neighboring line fixture. |
| Parameter bindings; before `Item::parameter_bindings` | `T::symbol_index::parameter_bindings_are_every_bound_name_in_signature_order_beside_a_function_with_none` | `symbol_index.rs:1205` asserts exact signature order across self, tuple/ref/struct/raw/slice patterns; required/default trait members; excludes type-internal name and const/function-with-none. |
| Initializer ranges; before `Item::initializer` | `T::symbol_index::an_initializer_is_the_text_between_the_equals_and_the_semicolon_beside_a_required_constant` | `symbol_index.rs:1235` slices original text and checks array, string containing semicolon, inherent/default trait constants, required const None and function None. |
| Written bodiless declarations; before `Two shapes of item` | `T::symbol_index::bodiless_functions_and_types_of_attribute_macros_are_indexed_where_they_are_written` | `symbol_index.rs:1017` checks kind, visibility, exact line/byte ranges, absent body, documented type, test tag and nonexistent alternate path. Unsupported forms require the separate bound refusal test. |
| Resolve as a module's code; split before `resolve_from(module, path)` | `T::resolve_paths::a_use_path_counts_only_what_the_reading_module_can_access`, `T::resolve_paths::a_use_path_is_read_in_one_name_space_or_in_each_in_turn`, `T::resolve_paths::a_module_that_is_not_in_the_workspace_and_a_text_that_is_no_path_are_refused` | `resolve_paths.rs:352` checks private self/parent and public sibling acceptance, private sibling/module NotFound; line 388 distinct type/value sites; line 409 UnknownModule, invalid syntax/super plus actual successful neighbor. |
| Impl records; retain `Impl blocks` but split negative impl sentence | `T::symbol_index::every_impl_block_is_listed_in_file_order_with_its_span_module_self_type_key_and_members`, `T::symbol_index::a_negative_impl_keeps_its_bang_in_the_trait_path_beside_a_positive_one` | `symbol_index.rs:1115` asserts exact rows including aliases, trait keys, unresolved/reference/generic self types, absent function-local block, attribute-inclusive span and id lookup. Line 1167 checks `!Sync` versus `Sync`. |
| Reachable references and closed modules; split Reachability after edge definitions | `T::reachability::a_private_helper_called_by_a_bound_item_is_reached_and_an_unreferenced_one_is_not`, `T::reachability::a_method_call_a_field_a_lifetime_and_a_local_name_no_item_and_a_path_prefix_names_its_type` | `reachability.rs:133` checks exact reached set, References/from provenance and absent unused/module-beside; line 162 checks enum-prefix/type reach, local-name false positive and absent unrelated helper. |
| Reachability order, roots and identity scope; before `The answer is a Reachability` | `T::reachability::the_answer_does_not_depend_on_the_order_of_the_roots_and_is_the_same_each_run`, `T::reachability::a_root_past_the_items_or_the_blocks_of_the_workspace_is_refused_beside_one_in_range` | `reachability.rs:384` checks repeat/reverse/duplicate root equality, exact node order, Root provenance, iterators and empty roots. Line 403 checks both typed out-of-range failures, valid neighbor and explicitly proves foreign in-range ids address this workspace by position. |
| Edition-sensitive reachability; split before `The keywords are those of the edition` | `T::reachability::gen_is_a_name_before_edition_2024_and_a_keyword_from_it`, `T::reachability::a_shared_file_reads_keywords_in_each_reached_packages_edition` | `reachability.rs:584` checks normal/raw gen in 2015/2018/2021/2024. Line 607 checks one physical file shared by both edition assignments, raw-neighbor acceptance, union of separate reach sets and both root orders. |
| Self-member reachability; before `A run that Self starts` | `T::reachability::self_paths_and_methods_called_on_self_reach_the_members_of_that_name`, `T::reachability::self_in_a_trait_and_in_its_impl_blocks_reaches_the_members_of_the_trait_and_of_the_block`, `T::reachability::a_variant_through_self_names_its_enum_as_a_path_through_the_enum_does` | `reachability.rs:701` checks exact members across impls, bare Self does not reach type, derive default absent, unrelated other-value method absent; line 738 checks trait/impl/unkeyed block sets and absent lonely member; line 775 checks enum variant References edge and struct missing-member neighbor. |
| Reachability counters and cache; split long final paragraph before `The work is counted` | `R::the_work_of_a_walk_is_counted_exactly`, `R::the_work_grows_with_the_items_and_the_references_and_not_their_product`, `R::a_path_of_fifty_thousand_segments_is_walked_once_and_not_once_for_each_prefix` | `src/reach.rs:1695` pins exact four formulas; line 1718 checks finite differences in two dimensions and unchanged resolution work for repeated calls; line 1836 checks actual reached endpoint, segment lower bound, lookup ceiling and one ReachPaths. These are counted operations, no clocks. |
| Public surface visibility; retain Exported items opening, split private-module/public-use sentence | `T::exported_items::a_pub_function_is_exported_beside_private_and_restricted_ones_which_are_not`, `T::exported_items::a_pub_item_of_a_private_module_is_exported_only_through_a_pub_use_and_under_its_rename`, `T::exported_items::a_glob_re_export_brings_in_the_pub_items_of_its_module_only` | `exported_items.rs:76` exact public set and item metadata; line 103 exact rename-to-definition mapping plus public-module neighbor; line 137 exact public glob members and private-glob empty neighbor. |
| Public surface hard failures; before `The errors are` | `T::exported_items::an_ambiguous_a_cyclic_or_an_unresolved_public_name_fails_beside_a_neighbour_that_lists`, `T::exported_items::a_surface_over_the_item_budget_left_by_the_load_is_refused_beside_one_within_it` | `exported_items.rs:598` typed ambiguity, cycle and unresolved-glob refusal plus successful neighbors; line 646 checks leftover ItemBytes limit, location/display, and roomy neighbor listing exactly 200 items. |

## Remaining clauses with precise bound candidates requiring assertion reread

The following are useful insertion anchors and exact existing bound candidates.
Their function locations were verified, but their full assertions were not all
reread in this bounded review. Treat them as preparation, not signed-off coverage.

| Clause / anchor | Candidate marker paths / next inspection |
|---|---|
| Tree map/list/entry and invalid-key construction; split Tree before `DiskTree::new` and before `MemoryTree holds` | `T::memory_tree::directories_are_inferred_and_listings_are_sorted_with_their_kinds`, `T::memory_tree::entry_reports_file_dir_and_absence_without_an_error`, `T::memory_tree::duplicate_keys_are_refused_literally_and_after_normalisation_beside_distinct_keys`, `T::memory_tree::a_tree_built_infallibly_from_a_bad_key_reports_the_error_from_every_call`; `tests/memory_tree.rs:51,71,115,147`. |
| Regular files, encoding and directory names; split before `DiskTree refuses any file` | `T::disk_tree_containment::a_fifo_named_as_a_file_is_refused_without_blocking_beside_a_regular_file`, `T::disk_tree_containment::a_file_that_is_not_utf8_is_refused_beside_one_that_is`, `T::disk_tree_containment::a_non_utf8_entry_name_fails_the_listing_beside_a_listing_without_one`; `disk_tree_containment.rs:151,237,247`. |
| Platform handle strategies and race windows; split each platform bullet into its own heading | Existing bound `raf_source::tree::disk::tests::strategy::*`, `raf_source::tree::disk::tests::swap::*`, Linux verifier and Windows predicate tests are relevant. Do not use a wildcard in an actual marker. Reread the exact functions before choosing the subset; strategy overrides do not prove native Windows API execution. |
| Prescan counter rules; split before each Delimiters/Nested/Prefix/Closure/else-if/Recursion definition | Use the corresponding bound `T::prescan_budgets::*` counter-specific exact/one-over and evasion functions, not the token-limit test for unrelated counters. Keep the long syn-audit/measurement paragraph separate. |
| Caller stack and module/directory budgets; split before `Two more budgets` and `Discovery and the expansion` | `T::walk_stack::one_hundred_twenty_eight_nested_modules_load_on_a_five_hundred_twelve_kib_thread`, `T::walk_stack::a_directory_at_the_depth_limit_is_walked_and_one_below_it_is_refused`, `T::walk_stack::a_member_pattern_at_the_depth_limit_expands_and_one_deeper_is_refused_typed`, `T::trait_keys::twenty_thousand_chained_local_aliases_load_on_a_two_mib_thread`, `T::trait_keys::a_trait_found_through_twenty_thousand_chained_globs_is_keyed_on_a_two_mib_thread`; file locations `walk_stack.rs:62,100,139`, `trait_keys.rs:1191,1208`. |
| Package discovery, target defaults/settings/errors; split Package discovery at `For each package` and `These are typed errors` | Bound `T::discovery_targets::*` exact target matrix, explicit/default neighbors, wrong-type settings, duplicate targets/roots and missing-target tests. Keep each independent refusal tied to its named negative/valid neighbor. |
| Module path directories and cfg alternatives; split Mod-tree following before `A path in cfg_attr`, before `Files are told apart`, and before `The facts of a file` | Bound `T::module_tree::*` specifically tests each flat/mod.rs/inline/path directory rule, alternatives/default, identity parse-once and cycles. `T::workspace_containment::*` tests symlink aliases and same-file cycles. Do not attach only one convenient path test to all rules. |
| cfg(test) matrix and attribute refusals; retain named clause but split malformed-attribute sentence | Bound `T::module_tree::the_cfg_test_matrix_tags_items_and_leaves_their_neighbours_alone`, `T::module_tree::a_malformed_cfg_path_or_test_attribute_is_an_error_naming_its_line`, `T::module_tree::an_unparseable_cfg_is_an_error_naming_file_and_line_beside_a_parseable_one`, `T::module_tree::a_non_literal_path_attribute_is_reported_as_such_from_the_attribute`. |
| Symbol kinds, docs, namespaces and test metadata; split Symbol index before `Documentation`, `Every target`, `A method`, `The key of a dependency trait`, `The identity` | Relevant exact bound `T::symbol_index::*` kind matrix, namespaces, documentation, target/test-role, cfg twins, unresolved self, field shapes and source accessors; `T::trait_keys::*` prelude/edition/dependency/unknown-glob tests. Each is a separate guarantee family. |
| Alias settlement; split Use-alias index before edition/dependency rules, named shadowing, cycles, two visibility rules, companions and canonical path validation | Existing bound `T::use_resolution::*`, `T::resolve_paths::*`, `T::trait_keys::*` have specific identity/ambiguity/cycle/order neighbors. Preserve this checkout's existing documented divergence classes. Current `T::rustc_oracle::*` corpus tests prove captured classes, not universal rustc equivalence. |
| Target-local resolve; retain Resolution from every target heading, split shared-file and unknown-module sentences | Bound `T::target_resolution::a_helper_resolves_from_an_integration_test_root_and_from_a_module_below_it`, `T::target_resolution::a_helper_resolves_from_a_nested_module_of_a_binary`, `T::target_resolution::an_example_a_bench_and_the_build_script_are_read_from_as_well`, `T::target_resolution::a_support_module_that_two_test_targets_include_resolves_in_each_target_alone`, `T::target_resolution::a_module_path_that_no_target_holds_is_unknown_whichever_kind_of_target_it_names`. |
| Surface hidden/macros/test filtering and external carried names; split at `A public re-export`, `doc(hidden)`, `Items are sorted` and carried-name work paragraph | Bound `T::exported_items::*` exact hidden defaults/options, macro export, test filtering, external reexport versus item, carried binder/drop/hidden paths, fixed random graph relation, deterministic order and CarryBudget neighbors. These should become multiple named clauses rather than one marker covering all surface behavior. |

## Constraints: map, split and genuine proof gaps

1. **Root-start restrictions**: keep root discovery's own named constraint; map
   the inspected `root` failure tests above only to what they assert. The six
   current root tests do not directly exercise a DiskTree directory alias or
   irreducible parent-segment start. Missing-target-manifest fallback also needs
   a literal root-discovery assertion; a target-loading failure alone does not
   prove nearest-ancestor selection.
2. **Syntactic indexing**: split the bodyless declaration statement from no
   macro expansion/cfg/type inference. The inspected bodyless test proves its
   stated names/kinds. Use bound `missing_names_dependency_items_and_macro_generated_items_are_not_found`
   for actual unresolved-generated and dependency neighbors after rereading.
3. **Containment and regular files**: reuse the specific Contract containment
   and file-type markers; do not infer native Windows behavior from Linux.
4. **Typed resource refusal**: split file size, prescan, module depth/path count,
   directory depth and total/item/fact/glob bytes into individually mapped
   constraints. The inspected exact/one-below tests above prove their own budgets.
   Their charge accounting is not a universal allocator proof.
5. **Parser stack and residual abort**: the bound overflow harness has explicit
   abort classifier/child-panic/stray-failure controls. Keep measured per-unit
   stack constants separate from the no-abort corpus claim. No selected test
   proves every possible admitted syntax tree consumes at most half the stack.
6. **Unbudgeted settling queue/graph**: current implementation limitation in
   this checkout; no existing bound test establishes the peak measurements or
   general absence of an accounting path. Do not map a successful glob budget
   test to this limitation. Carry the source evidence and historical measurement
   limits explicitly while the separate resolver delivery changes the behavior.
7. **Uncounted lookup complexity**: the explicit n = 2048 / instruction number
   and `25n^2` statement are measurement claims, not proved by the small Work
   exact-limit test. Do not fabricate a marker using that test.
8. **Reachability approximation**: split unsupported indirect dispatch/macros,
   local-name false positives, macro home, and named-module-closed behavior.
   `reachability.rs:133,162,262` are concrete helper/module/local/macro neighbors.
   A universal list of every indirect form is not proved by a single negative.
9. **Self and qualified-path limits**: the inspected tests at lines 701,738,775
   prove bare Self/member/enum behavior. The `<Self as Tr>::name`, `<S>::name`
   suffix and nested block's inherited Self cases need dedicated assertion
   lookup; do not claim those from bare Self tests.
10. **Edition 2015 function-local use**: no dedicated bound negative identified
    in this review. `a_use_path_of_edition_2015_starts_at_the_crate_root` concerns
    module import resolution and is not proof of this reachability limitation.
11. **Nested impls**: the inspected impl-record fixture explicitly excludes its
    function-local block. The `const _` and other block-expression variants need
    actual assertions, not an inference from that one fixture.
12. **Unsupported verbatim forms**: use the separately bound
    `T::symbol_index::a_bodiless_const_or_another_verbatim_item_is_still_a_typed_error_naming_file_and_line`
    for refusal and the inspected bodiless-function/type test as valid neighbor.
13. **Known rustc disagreements**: preserve the current captured corpus classes;
    the bound corpus and generated oracle checks enforce their finite matrix.
    They do not prove absence of every other program disagreement.
14. **Order-sensitive rustc / deterministic index**: use captured literal order
    fixtures and the oracle instability-negative tests after assertion reread.
    Finite permutations are not all syntactically possible Rust programs.
15. **Superlinear shape measurements**: separate measured instruction ratios
    from counted polynomial rules and typed Work refusal. These require their
    own retained measurement inputs/results; do not run fresh timing or attach
    a test of a different counted shape.
16. **4096 member/exclude entries**: find exact at-limit/one-over assertions;
    none identified in the inspected `root` tests. A malformed-list test is not
    an entry-count proof.
17. **Terminal-safe error text**: exact bound candidate
    `raf_source::path::display_tests::control_characters_in_a_refused_path_are_escaped`
    proves a refused-path display fixture. Broader all-error raw-field retention
    and escaping requires equivalent assertions for other error families.
18. **Reload-only freshness**: no bound test demonstrating old workspace stays
    unchanged after tree edit plus new load observes it was identified. Sorted
    equal-load tests prove determinism, not freshness behavior.
19. **Serial/non-Send Tree contract**: `read_cap` compiles trait objects, proving
    object safety. No current bound test with an Rc/RefCell-backed non-Send,
    non-Sync Tree was identified. `walk_stack` explicitly uses `dyn Tree + Sync`
    for its thread transfer, so it cannot prove the stronger no-bound promise.
20. **syn reexport/version compatibility**: a source API or consumer compile
    assertion can establish the reexport; a future semver policy promise is not
    behavioral coverage. No appropriate existing bound test identified.
21. **Output order**: inspected full-workspace equality/strict item and target
    ordering test supplies the actual marker; surface order has its own bound
    surface determinism test.
22. **Surface limitations**: split cfg/macros, cfg_attr/doc visibility, capped
    exports, ambiguity refusal, unknown-namespace carried shadow, CarryBudget
    and rustdoc difference into individual constraints. Existing named
    exported-items tests are precise candidates; do not claim one ambiguity
    test proves all these unrelated limitations.
23. **Native platform evidence**: preserve the document's distinction between
    compiled Windows-specific paths, Linux override tests and actual hosted
    Windows execution. No local claim marker can turn a compile into that run.

Additional Contract gaps independent of the Constraints are the exact historic
registry corpus sizes/maxima, resident-memory peaks, instruction ratios, the
universal 1664-bytes-per-token parse bound, arbitrary-mount inode stability,
unsupported-host DiskTree refusal, and future non_exhaustive/semver promises.
The constant test pins a number; it does not prove the underlying universal
bound. Keep this material as explicit measurement/audit explanation with its
actual evidence, or obtain appropriate proof, rather than making zero claim
findings by attaching unrelated tests. No scope or acceptance is dropped by
this map.

## Proposed relocation versus required proof

Move the following complete historical observations to Explanation, preserving
their values, original inputs and qualification limits. The present Explanation
begins `raf-source is deliberately narrower than a compiler`; add named
subsections after that rationale. These are observations, not substitutes for
the separately stated runtime guarantees:

- **Measured parser costs**: the detailed `Measured in a debug build, a unit
  costs 16481 bytes...` catalogue, the release-build cost and registry maximum
  recursion/token observations. Preserve the actual measurement profile and
  the distinction between sampled shapes and a universal bound.
- **Registry-derived defaults**: the 1271/1251-crate sample, `linux-raw-sys`,
  `web-sys`, time-zone-table maximum and chosen default rationale. Keep the
  actual configured limits and mandatory typed refusals in Contract.
- **Measured load footprints**: both numbered `Two figures follow, measured in
  a release build` examples, the detailed MB/MiB/peak tables in prose and the
  queue/graph 87/555 MiB observations. Keep charge ownership and the existing
  unbudgeted-storage limitation visible in Contract/Constraints; moving the
  observations must not imply all live memory is bounded.
- **Measured resolver growth**: instruction ratios and original-resolver
  comparisons from the long Constraints shape paragraph, plus n = 2048's exact
  instruction observation. Keep the current asymptotic limitations and exact
  scope of Work counting in Constraints until implementation and proof change.
- **Allocation-layout rationale**: the BTree/hash/list sizing derivation may
  live in Explanation with source references. The actionable promise that a
  successful load respects the declared budgets remains in Contract and still
  requires real allocation coverage; relocation cannot excuse an undercharge.
- **Compatibility rationale**: future syn-major and field-addition semver
  policy belongs in Explanation/release policy. Keep actual object safety,
  constructors, non_exhaustive attributes and reexported API visible in Contract
  with compile evidence where those promises are required.

Potential required proof additions, distinct from prose relocation, are the
root-start directory-alias/parent restrictions, actual non-Send/non-Sync Tree
consumer compilation, member/exclude entry-count boundary, reload freshness,
qualified/nested Self and const-block reachability limitations, and unsupported
host/native-platform refusal evidence. They are gaps identified in this bounded
assertion search, not declarations that no test exists anywhere: inspect the
current bound inventory and production path before adding or assigning one.
The universal admitted-parser memory/stack and retained-allocation bounds are
substantive obligations. Historical measurements support rationale but do not
discharge them; weakening or relocating the guarantee itself would require a
real scope decision and is not proposed here.

## Additional assertion inspection: Tree backends and target discovery

Bounded preparation against current issue 16 source only. No builds/product edits
or runtime acceptance claims. `T::` retains the exact prefix defined above;
test source is `crates/raf-source/tests/<module>.rs` at the named function.
These candidate families now have assertion-body inspection as follows.
Contract heading markers may cover their clause; for any Constraints item put
its marker immediately after that item, not after a heading before it.

### Memory Tree clauses

Insertion anchor: the MemoryTree sentences currently inside `### Tree errors`.
Split map construction, ordinary operations and invalid-key refusal rather than
assigning one schema/error marker to all three.

| Proposed Contract clause | Exact inspected markers and assertions |
| --- | --- |
| Memory file reads and absence | `T::memory_tree::a_file_reads_back_and_a_missing_file_is_not_found` checks literal returned bytes, NotFound and missing-path display. `T::memory_tree::a_directory_is_not_readable_as_a_file_and_a_file_is_not_listable` checks NotRegular for directory read/file listing and NotFound for absent directory. |
| Inferred directories and ordered entries | `T::memory_tree::directories_are_inferred_and_listings_are_sorted_with_their_kinds` compares complete root/src Entry arrays, including File versus Dir and lexical name order. `T::memory_tree::entry_reports_file_dir_and_absence_without_an_error` checks file, inferred/root directory and absent None. |
| Normalized keys and construction | `T::memory_tree::a_parent_segment_is_applied_to_the_trees_own_directories` checks a normalized existing read and normalized missing neighbor. `T::memory_tree::collecting_pairs_builds_the_same_tree_as_the_array_conversion` compares the collected and array-constructed tree exactly. |
| Invalid and conflicting keys | `T::memory_tree::keys_that_leave_the_root_are_refused_with_outside_root_beside_a_valid_key` checks ../a, /a, x/../../a, OutsideRoot, raw key in display and a valid key. `T::memory_tree::a_key_that_is_both_a_file_and_a_directory_is_refused` checks NotRegular/path a and the distinct-prefix ab/b neighbor. |
| Duplicate keys and poisoned infallible construction | `T::memory_tree::duplicate_keys_are_refused_literally_and_after_normalisation_beside_distinct_keys` checks literal, src/../ and ./ aliases, DuplicatePath/raw second path, read/list/entry refusal of poisoned conversions and distinct-file success. `T::memory_tree::a_tree_built_infallibly_from_a_bad_key_reports_the_error_from_every_call` checks read/list/entry OutsideRoot and original bad path beside an otherwise valid key. |

The last test's “every call” body covers read/list/entry, not explicitly identity
or read_unseen on a poisoned tree. Do not expand that fixture's proof to those
methods by its name alone. The separate identity/read_unseen tests inspect valid
normalized keys, file versus directory/missing kinds, Seen skipping a zero cap,
and an unseen zero-cap TooLarge refusal. They support their specific identity
clause, not a universal cap boundary for every Tree operation. Map-wide
max_bytes parity remains a separate exact-boundary proof obligation.

### Disk regularity, encoding and entry clauses

Insertion anchors: regular-file sentence before platform strategies, Tree errors
non-UTF-8 names, and DiskTree root construction. These tests are `#![cfg(unix)]`;
do not present them as native Windows execution or proof of every strategy.

| Proposed Contract clause | Exact inspected markers and assertions |
| --- | --- |
| Regular-file reads | `T::disk_tree_containment::a_fifo_named_as_a_file_is_refused_without_blocking_beside_a_regular_file` uses an actual FIFO, requires read NotRegular, entry Other and regular bytes `real`. `T::disk_tree_containment::a_directory_named_as_a_file_is_refused` checks directory read and regular-file listing NotRegular. The tests assert no time; a FIFO pass is a concrete sentinel, not a universal no-open theorem. |
| UTF-8 contents and names | `T::disk_tree_containment::a_file_that_is_not_utf8_is_refused_beside_one_that_is` writes 0xFF, requires NotUtf8/path display and valid UTF-8 neighbor. `T::disk_tree_containment::a_non_utf8_entry_name_fails_the_listing_beside_a_listing_without_one` checks actual dirty-directory NonUtf8Name/path and clean listing. On macOS it instead asserts file creation refuses invalid names and the directory stays empty. Preserve that platform-specific assertion, not a skipped-listing claim. |
| Entry kinds and stable name order | `T::disk_tree_containment::entry_kinds_cover_file_directory_symlink_other_and_absent` checks regular/dir/FIFO/symlink/absent/non-directory-parent/root/sub/.. exact Option kinds. `T::disk_tree_containment::listings_are_sorted_by_name` compares sorted equality and exact B.rs,a.rs,m.rs,real.rs,z.rs sequence. |
| Root construction and canonical containment | `T::disk_tree_containment::a_missing_root_is_an_error_beside_an_existing_one` checks missing NotFound/path, existing directory success and file-root NotRegular. `T::disk_tree_containment::a_symlinked_root_is_followed_to_its_target_and_containment_is_judged_there` checks real bytes through alias root and OutsideRoot for external FIFO link. |
| Missing versus outside target | `T::disk_tree_containment::a_file_that_is_missing_is_not_found_and_one_that_leaves_the_root_is_outside_root` checks missing and dangling NotFound versus outside-link OutsideRoot. It does not test missing /proc; keep that independent host-invariant proof separate. |

The containment tests also have inspected valid/refused symlink-chain,
directory-loop, parent-after-symlink and identity/read_unseen neighbors. They
prove their concrete layouts, not race coverage or Linux handle-before-open,
other-Unix descriptor-walk, Windows normalization, unsupported-host refusal or
all directory-name encodings. Those stronger platform clauses need the actual
strategy/cross-platform tests already owned elsewhere, not these broad labels.

### Discovery and target clauses

Insertion anchor: current bold Package discovery plus the following target
paragraphs. Split the following meaningful guarantees with their own markers.

| Proposed Contract clause | Exact inspected markers and assertions |
| --- | --- |
| Explicit discovery roots | `T::discovery_targets::an_empty_root_list_is_a_no_roots_error_beside_dot_which_loads` checks typed NoRoots and one package for dot. `T::discovery_targets::only_the_named_roots_are_searched` checks one versus two packages and overlapping roots deduplicate. `T::discovery_targets::a_missing_root_is_an_error_beside_an_existing_one` checks Tree error/message and existing-root success. |
| Workspace descent and skipped directories | `T::discovery_targets::a_package_nested_below_a_package_without_a_workspace_is_not_discovered` asserts only demo, then demo/a with workspace table. `T::discovery_targets::a_virtual_manifest_contributes_nothing_but_its_children_are_found` asserts virtual zero packages/targets and one child manifest. `T::discovery_targets::target_and_hidden_directories_are_skipped_beside_a_discovered_sibling` asserts only a, excluding target/.hidden packages. |
| Manifest syntax and hard errors | `T::discovery_targets::a_manifest_with_a_trailing_comment_and_single_quoted_values_is_honoured` checks actual chosen root/name/kind/path/shipping. `T::discovery_targets::a_manifest_that_is_not_toml_or_has_neither_table_is_a_typed_error` checks Manifest/file for malformed/neither and package.name key for numeric name. `T::discovery_targets::deeply_nested_inline_tables_are_a_typed_error_on_the_callers_stack_and_a_shallow_one_loads` checks Manifest/file at 1000 and successful 8-level neighbor; no independent alternate-stack execution is implied by its name. |
| Manifest size boundary | `T::discovery_targets::a_manifest_over_the_cap_is_refused_naming_it_and_one_of_exactly_the_cap_loads` constructs exact 8 MiB and +1 bytes, checks exact success, Tree TooLarge and manifest file on overflow. This is manifest input only, not every source/total-memory budget. |
| Default target paths and shipping roles | `T::discovery_targets::each_target_kind_with_default_paths_has_its_name_kind_crate_path_and_shipping_flag` compares full ordered tuples for lib, main/bin file/bin dir, test file/test dir, bench file, example file and build.rs. It proves those actual forms; grouped bench/example defaults are not independently exercised in this body. |
| Explicit target roots and names | `T::discovery_targets::each_target_kind_with_explicit_paths_resolves_beside_its_default_neighbour` compares explicit lib/bin/test/bench/example tuples, including core-lib -> core_lib normalization. Its body constructs the explicit layout only, despite the title's “default neighbour”; the separate default test supplies that neighbor. `T::discovery_targets::a_library_and_a_binary_with_one_name_resolve_under_their_own_crate_paths` checks two target crate-path strings only. |
| Autodiscovery flags | `T::discovery_targets::autodiscovery_flags_turn_inferred_targets_off_but_explicit_tables_are_still_found` turns off all four flags, checks only explicit kept test remains, then five inferred targets with default flags. It does not independently test each explicit target kind under every false flag combination. |
| Target metadata defaults and settings | `T::discovery_targets::harness_required_features_test_and_doctest_are_read_from_each_target_table` compares exact rows for modified and unmodified lib/bin/test/bench/example/build settings, required feature order and plain-library test/doctest defaults. `T::discovery_targets::a_test_setting_of_the_wrong_type_is_a_manifest_error_naming_its_key` checks five actual wrong key types/messages. `T::discovery_targets::required_features_on_the_library_is_a_manifest_error_beside_a_binary_that_names_them` checks library Manifest/key and accepted binary feature retention. |
| Duplicate package/target identity | `T::discovery_targets::two_packages_with_one_name_are_an_error_naming_both_manifests` checks exact DuplicatePackage payload, second manifest file, first manifest display and distinct-name neighbor. `T::discovery_targets::two_targets_sharing_one_root_are_an_error_naming_the_second` checks SharedRoot root/other and bin.path. `T::discovery_targets::two_targets_with_one_crate_path_are_an_error_naming_both_beside_distinct_names` checks all four table kinds, duplicate explicit names, inferred dash/underscore collision, typed path/key and distinct-name neighbors. |
| Cross-package crate identity | `T::discovery_targets::two_packages_whose_targets_share_crate_paths_load_and_are_told_apart_by_package` checks full package/crate paths, unqualified Ambiguous two sites and package-qualified actual source file selection, with same-package collision still refused. Do not infer unrelated resolver behavior from it. |
| Missing and escaping explicit paths | `T::discovery_targets::an_explicit_target_file_that_is_missing_is_an_error_naming_the_manifest_key` checks Cargo.toml/bin.path MissingFile, present-file success and lib.path. `T::discovery_targets::a_target_table_without_a_name_is_an_error_naming_the_key` checks bin/test/bench/example NoName and exact key. `T::discovery_targets::a_target_path_that_leaves_the_tree_is_refused_and_a_path_that_stays_is_accepted` checks absolute/escaping Tree path plus bin.path, and a sibling target inside the supplied tree succeeds. |
| Target and package ordering | `T::discovery_targets::targets_are_sorted_and_two_loads_are_equal` checks a,b target order across reversed fixture insertion and repeated target/package equality. It does not compare complete Workspace equality or every target accessor. |

The named `the_root_module_of_every_target_is_recorded_under_its_crate_path`
body checks reached-file module_path/test metadata, source text and target_ids
accessors for a library and test. It does **not** call resolve, despite related
prose promising every crate path resolves to a module item. Keep that broader
promise assigned to an actual target-resolution/root-item test rather than
using this test as arbitrary proof. Similarly, the default test's summary helper
does not include Target.package; package-name propagation needs actual field
assertions if split into its own guarantee.

Still unproved by these inspected families: first error ordering across all four
load phases, discovery depth 128 and member/exclude cardinality boundaries,
symlink classification with at-most-one-byte opens, all target-kind provenance
keys for unreadable inferred roots, build=false/named build behavior, and every
table/path default combination. These are genuine proof boundaries of this
bounded search, not assertions that no current bound test exists elsewhere.
Inspect the dedicated existing families before adding tests or marking these
clauses covered. All historical/performance/host observations remain distinct
from these behavioral assertions, and no whole-contract pass is claimed.

## Concrete recovery patch and complete historical-group disposition

The actionable candidate is `tasks/source-claim-repair.patch`. It changes only
`docs/capabilities/source-index.md` when applied. Production files have not been
edited by this preparation. `git apply --check` succeeded against the current
recovery document. No builds or tests were run by this reviewer. The generator
and complete candidate remain alongside the patch for inspection; they are ignored
metadata, not product files or a second implementation.

The patch preserves existing prose, values and limitations. It replaces broad
bold leads with named clauses, inserts independent boundaries and exact owners,
and adds nine currently absent test bindings. It does not claim zero remaining
findings: splitting one compound claim can expose several meaningful uncovered
claims. The historical 28 count must not be used as the post-patch count.

Source implementation audit plus relevant actual boundary/caller fixtures are
legitimate evidence; a test need not enumerate every Rust program. The concrete
remaining weaknesses below concern specific missing ownership or unmetered storage,
not a demand for formal universal proof or an exhaustive duplicate parser test.
Existing raw error fields and Display borrowing `&self`, together with the actual
escaping owners, support raw-field retention without a new mirrored match test.

| Historical group | Concrete patch disposition and remaining action |
|---|---|
| Contract: Disk lookup failure boundary | Separate Linux descriptor failure, entry lookup and opening ownership. Exact native backend test owners cover the first two. Opening ownership remains visible for production call-site audit, rather than attaching an unrelated path test. |
| Contract: Path containment | Attach real outside-module/manifest negatives with ordinary neighbors, then split regular-file and verified-handle read/list/entry guarantees. Exact strategy/swap owners retain Linux versus native-host scope. The Windows and other-Unix strategy paragraphs remain intact. |
| Contract: Resource bounds | Split file admission, lexical exclusions, typed errors and counter definitions. Attach lexical/syntax owners and actual counter boundary/evasion owners. Keep every numeric limit, parser-stack claim and historical audit value. Parent should finish precise source/overflow evidence on the separate admission/stack clauses; do not replace the actual guarantee with a sampled-corpus-only claim. |
| Contract: Work and memory bounds | Split directory/aggregate ownership, configured defaults, byte-charge ownership and parser stack. Default-limit owner does not cover the complete layout derivation: that prose remains a separate visible claim. Original queue/graph and uncounted-lookup limitations remain unchanged. Existing actual ItemBytes/FactBytes/GlobBytes exact refusals and implementation audit are the relevant next owners for their separate charge clauses. |
| Contract: Mod-tree following | Separate ordinary directories/path attributes, cfg_attr alternatives, identity/cycles and owned facts. Exact directory/path/missing-neighbor owners and predicate/default/read-once owners are attached. Owned fact metadata remains separate so its broad list is not falsely covered by one module-path fixture. |
| Contract: cfg(test) tagging | Separate complete role matrix from malformed attributes and attach exact role/error owners. No declared-host cfg evaluation is claimed. |
| Contract: Use-alias index | Separate written forms, edition/dependency heads, import self-shadow and glob settlement. Concrete written-form and dependency/2015 owners are attached. The large self-shadow/settlement paragraphs still need smaller clauses against the existing donor use_resolution owners; no resolver-28 test names are imported and no newer resolver behavior is asserted. |
| Contract: Resolution from every target | Split actual target kinds, prefix lookup cost, target preludes, shared-file contexts and unknown/ambiguous origins. Each gets its specific target_resolution or counted prefix owner. Shared sites and same_source assertions are real, not inferred from total load equality. |
| Contract: API surface | Keep all non_exhaustive/constructor/Clone declarations and extension policy. Attach actual external consumers of Entry/source accessors/ExportOptions/Reachability; implementation declarations supply the relevant API audit. No runtime test is invented for a future policy promise. |
| Constraints: syntactic parsing | Attach actual bodyless attribute-input and missing dependency/macro-generated item owners. Source-only limitation stays explicit. |
| Constraints: containment/regular files | Attach actual module/manifest outside-root and FIFO refusal neighbors. Platform-specific race details remain in the separate Contract clauses. |
| Constraints: typed resource refusals/stack | Attach exact cap, prescan, module-depth/path-count, directory, byte-budget, overflow corpus and abort-classifier owners. Keep all original limits and half-stack statement. Use implementation audit plus these relevant controls; no formal-proof prerequisite added. |
| Constraints: unbudgeted queue/graph | Preserved unmarked. Current Pending queue and undecided-graph allocation paths have no ItemBytes/GlobBytes admission. This is a real limitation, not fixed by the new format constructor. A deterministic accounting witness can substantiate it without importing resolver-28 behavior. |
| Constraints: uncounted settling lookups | Preserved unmarked with original 4n+2, 25n² and instruction observation. A small shape can pin counted versus actual lookup operations; existing Work-budget table tests alone do not prove the missing charges. Historical instructions may move to Explanation with all values retained, while the limitation stays judged. |
| Constraints: source-defined resolution | Exact missing dependency/macro item test owns NotFound beside ordinary function/macro neighbors. This does not deny the separately documented external Reexport rows. |
| Constraints: reachability approximation | Original indirect-call/local-name/macro-origin/module-closure limitation stays visible. Split into direct-reference, same-name false-positive, macro-origin and module-root claims using existing reachability owners; do not put one marker over every independent behavior. |
| Constraints: Self/qualified/nested handling | Split bare Self, qualified suffix direct edges and nested-impl context. Existing Self tests plus the two new actual direct-edge/nested-context fixtures own these. Qualified syntax can reach a type and then its members via containment; it must not be described as making those members unreachable. |
| Constraints: edition-2015 body use | Attach new actual implicit-refusal versus explicit crate-path neighbor. Parent reports runtime pass; reviewer read actual assertions, ran no test. |
| Constraints: impls in const/block expressions | Attach new actual unnamed const/named const/function-block omission beside indexed top-level impl. Parent reports runtime pass; no assertion of general macro expansion. |
| Constraints: unsupported verbatim forms | Attach both bodyless valid forms and three UnsupportedItem/file-line refusals, plus leading-colon use-group owner. No fallback or partial load is added. |
| Constraints: known compiler disagreements | Preserve complete current limitation. Existing captured divergence/oracle corpus owners can cover the precise finite classes after checking their actual inventory; do not claim universal compiler equivalence. |
| Constraints: compiler order sensitivity | Preserve limitation. Existing declaration-order fixture families distinguish the index's deterministic answer from captured compiler answers; select exact owners after inspecting those assertions. |
| Constraints: superlinear settling | Keep all original instruction observations and current limitation. Actual glob_scope::work_budget owners prove quadratic/cubic step polynomials and first-over-limit charge/refusal. Split that operational claim from historical instruction comparisons before attaching those owners. |
| Constraints: terminal-safe errors/raw fields | Attach complete current LoadError/TreeError/Origin/TargetProblem display family, ResolveError family and refused-path family. Immutably borrowed typed fields support retention; no duplicated match-test ceremony required. |
| Constraints: syn reexport/version policy | Preserve actual reexport and breaking-major policy. A current source/API consumer plus dependency audit can substantiate ownership; no new public adapter or future semver behavior is invented. |
| Constraints: deterministic output | Attach exact sorted item/target equality and sorted surface/declaration-order owners. Current source claims only the tested deterministic representation, not native map-ID ordering. |
| Constraints: surface visibility/limitations | Preserve all current cfg/macro/hidden/visibility/namespace/CarryBudget/rustdoc limits. Split into meaningful clauses; existing exact exported_items families own each supported behavior, including actual CarryBudget and roomy neighbor. cfg_attr opacity still needs a direct assertion if not supplied by an inspected bound fixture. |
| Constraints: native Windows proof | Preserve explicit cross-compilation versus Windows runtime distinction. Relevant named native tests are real but gated; host applicability remains the separately tracked platform-verification contract. Linux overrides do not become Windows execution. |

The patch also adds the four exact native ItemAdmission/propagation owners and a
separate format-constructor storage constraint, with original cumulative charge,
scratch-release and error-cause wording. It adds the canonical-child alias neighbor
to the existing positive-format-certificate limitation. These are the parent's
actual constructor changes, not a claim that resolver settling is now metered.
All newly reported execution outcomes remain parent-owned receipts, separate from
this metadata preparation.

## Second patch against the measured seventeen-claim snapshot

`tasks/source-claim-second.patch` targets the exact source document measured by
session 2153 (`raw/recovery-source-claims-current`). Its metadata-only candidate
passed `git apply --check`; no product edit/build was performed here. It selects
100 exact existing owner paths; all were already in frontmatter. The parent must
measure the resulting claim set rather than infer a final count from the seventeen.

The patch attaches actual read-cap/spawn, directory-depth, byte-budget, overflow
corpus/admitted-family/abort-classifier, owned fact/accessor, import/visibility,
reachability, oracle and surface controls. Long import/glob statements are divided
at independent guarantees. Original release-build memory observations and original
resolver instruction observations are retained intact under named Explanation
subsections; current numeric bounds and the unmetered queue/graph/lookup limits
remain judged. Review the restated memory-envelope paragraph against the intact
historical block before applying: retained index/text/facts/glob tables sum to
1.5 GiB at defaults; the stated 1664-bytes-per-token parse bound remains; those
components sum to about 4.6 GiB. The transient unbudgeted resolver queue/graph is
explicitly outside those components, not silently represented as admitted storage.

Three deliberate missing attachments remain:

1. **Unmetered settling storage:** inspect the real private `Pending` and
   undecided wait-graph construction. A minimal deterministic unit can use the
   production wait/graph helpers on a fixture with populated waiting/watching
   state and a no-pending neighbor, checking actual rows and the original meter's
   unchanged ItemBytes/GlobBytes/Work receipt for that construction. Source audit
   must tie the helper to the actual load. Do not introduce an unrelated Vec and
   call its independent meter proof of production behavior.
2. **Uncharged glob lookups:** use the documented n-glob/n-alias shape, actual
   settled lookup and existing Work telemetry. Pin recorded table work versus
   actual glob-read traversal work and prove repeated lookups leave the meter's
   Work total unchanged. Include a definition/named-import neighbor that never
   consults globs. No clocks, instruction benchmark, new resolver or silent
   omission is needed.
3. **Public syn type seam:** existing repository indexing tests assert NotFound
   for a reexported external crate; that is not proof of the public reexport's
   actual Rust type. Add a small integration test whose parse_source visitor
   passes its argument to `fn count(file: &raf_source::syn::File) -> usize` and
   asserts one parsed function, then checks a malformed input's real Syntax error.
   This compiles the real reexport/visitor seam. Keep the documented breaking-major
   policy as policy, not an invented runtime future-version detector.

The oracle owners are actual compiler-backed corpus/generated checks and exact
instability-classifier controls. They preserve known divergences; their attachment
does not import issue 28 behavior or assert universal rustc equivalence. The Windows
clause remains explicit about native-host execution versus local verifier/predicate
coverage; its markers do not assert a foreign-host pass.
