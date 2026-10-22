<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Census normal-form claim preparation

Read-only mapping of the current issue 16 checkout's Contract and Constraints.
No product files changed; no builds, suites, measurements or compiler probes ran.
The assertions below were inspected in source unless explicitly called a
candidate. This is preparation for markers, not execution evidence. No facts
from the separate resolver checkout are imported.

Use meaningful named clauses in place of the present long bullet sequence.
Keep every guarantee and limitation. A heading marker can list several exact
bound paths, for example:

```markdown
### Complete fingerprint identity
<!-- raf:tested-by raf_census::test_crate::forms::the_digest_is_blake3_of_length_prefixed_tokens,raf_census::test_crate::forms::human_abbreviations_do_not_replace_complete_identity,raf_census::group::tests::digest_suffixes_are_distinct_group_keys -->
```

In the table `T::` means the exact prefix `raf_census::test_crate::`; `C::`
means `raf_census::`. Expand them in actual markers. Paths listed here already
occur in this capability's test bindings. Quoted insertion text is more stable
than line numbers; Contract currently begins at line 291 and Constraints at 630.

## Contract splits and inspected assertion evidence

| Clause / exact insertion anchor | Proposed marker paths | Actual assertions and limits |
|---|---|---|
| Normal-form pipeline; before `The census implements normal_form = 1` | `T::forms::every_registered_pair_compares_as_registered`, `T::golden::every_fixture_fingerprint_is_pinned`, `T::golden::the_manifest_is_pinned` | `forms.rs:34` requires more than 60 registered pairs and checks exact equal-from/distinct-through levels plus first difference. `golden.rs:303,343` compares the manifest and computed fixture table with fixed pins. This is the actual finite registry, not a proof over every Rust program. |
| Value erasures; before `The value form drops borrows` | `T::forms::the_value_form_keeps_binary_operators_and_drops_borrows`, `T::forms::the_value_adapters_are_the_eleven_of_the_page`, `T::forms::every_registered_pair_compares_as_registered` | `forms.rs:216` is the specific operator/borrow test; line 238 pins the exact eleven adapter names. Registry pairs cover distinct erasures and their negative neighbors; do not use only the names-array test as proof the adapters execute. |
| Bound-name renaming; before `rename_locals renames only` | `T::forms::parameters_and_locals_are_numbered_together_by_first_appearance`, `T::forms::self_is_not_a_parameter_and_a_field_named_like_one_is_kept`, `T::forms::every_registered_pair_compares_as_registered` | `forms.rs:271` asserts exact Shim and Value token arrays showing signature order versus joint first-appearance numbering and unused parameter behavior. Line 291 checks Self and field-name preservation. The actual registry carries binder cases for let/match/if/while/for/closure/local declarations/labels/lifetimes/macros. |
| Item-name and field distinction; split before `Every other identifier` | `T::groups::bodies_that_apply_different_functions_share_no_group_but_renamed_locals_do`, `T::groups::bodies_that_read_different_lowercase_statics_share_no_group`, `T::groups::a_real_struct_literal_keeps_its_field_names`, `T::groups::a_block_after_a_constant_in_a_head_is_a_block_and_its_locals_are_renamed` | These actual integration functions are at `groups.rs:569,584,605,625`; full assertion bodies are candidates for final marker reread. They target the exact production grouping distinction. They must not be replaced by page-text keyword assertions. |
| Literal tier and threshold count; before `A body whose value form has fewer than 30` | `T::forms::the_literal_tier_boundary_is_thirty_value_tokens`, `T::forms::below_the_tier_the_final_sequence_keeps_literals_and_above_it_abstracts_them`, `T::forms::every_threshold_counts_the_value_form` | `forms.rs:199` checks 29-token literals differ, 30-token literals unify, plus 25/35 neighbors; line 258 inspects literal/abstract token content; line 628 checks every fixture's value count and concrete erased-noise neighbors for tier, minimum and shim bounds. |
| Shim eligibility; before `A body is a shim candidate` | `T::forms::shim_forms_exist_for_six_to_twenty_nine_tokens`, `T::forms::self_callee_query_distinguishes_generic_types_from_calls`, `T::groups::a_forwarder_at_min_tokens_joins_a_shim_group_and_never_a_body_group` | `forms.rs:142` checks 5/6/29/30, let/semicolon/self/no-call negatives, trailing-semicolon and macro-call positives. Line 518 distinguishes Self associated-type generic arguments from actual generic Self calls including lifetime/function-type arguments. `groups.rs:218` asserts 24-token forwarders retain both prints yet have zero body groups and exactly one shim group. |
| Shim grouping exclusions; split before `The shim_key is absent` | `T::units::shim_keys_are_absent_for_trait_impl_methods_and_self_forwarders`, `T::units::a_self_call_at_any_depth_removes_the_shim_key`, `T::units::self_type_arguments_keep_shim_keys_beside_generic_self_callees`, `T::groups::shim_groups_use_the_shim_key_and_skip_trait_impls_and_self_forwarders` | Functions verified at `units.rs:37,508,627` and `groups.rs:176`; the new low-level Self-call query at `forms.rs:518` was fully inspected. Reread these end-to-end bodies for the separate trait/default/inherent and nested-call boundaries. |
| Minimum body size; before `A body with fewer than min_tokens` | `T::forms::the_body_print_needs_min_tokens`, `T::units::min_tokens_decides_the_body_fingerprint_at_the_default_and_at_ten`, `T::groups::bodies_under_min_tokens_never_group` | `forms.rs:191` accepts exact 12 tokens and removes print at minimum 13. `groups.rs:89` checks duplicate 19-token bodies produce no groups and 21-token neighbors group. `units.rs:447` is the options-to-production caller candidate. |
| Complete fingerprint identity; before `A fingerprint is` | `T::forms::the_digest_is_blake3_of_length_prefixed_tokens`, `T::forms::a_fingerprint_displays_and_parses_back`, `T::forms::human_abbreviations_do_not_replace_complete_identity`, `C::group::tests::digest_suffixes_are_distinct_group_keys` | `forms.rs:389` hand-builds UTF-8/u32 little-endian token bytes and compares the full known BLAKE3 hex string with the production print. Line 419 round-trips complete body/shim/table/rule identities; line 451 changes only digest byte 31 and proves unequal identities but equal human prefixes, with prefix parsing refused. `group.rs:251` proves same prefix/different suffix does not group and exact-equal neighbor does. |
| Portable encoding/backend; before `BLAKE3 is requested` | `T::forms::the_digest_is_blake3_of_length_prefixed_tokens` for encoding only | `Cargo.toml:17` directly sets BLAKE3 default-features=false and pure. The inspected byte test proves the specified encoding on this target, not a full WASM build/run. No existing bound test establishes no cargo-metadata invocation across every core path; source architecture review is separate. Split these assertions rather than attach one hash test to all. |
| Implemented version and refusal; before `raf_census::NORMAL_FORM is 1` | `C::version::tests::the_implemented_version_is_one`, `C::version::tests::the_implemented_version_passes`, `C::version::tests::other_versions_are_refused_naming_both_and_the_migration` | `version.rs:33` pins 1, line 39 accepts 1, line 44 checks 0/2 refusal messages contain declared/implemented/migration instruction. Reading without gating belongs to a separate report test below. |
| Options and per-file cap; before `raf_census::CensusOptions` | `C::options::tests::defaults_are_twenty_tokens_and_eight_mebibytes`, `C::options::tests::zero_min_tokens_is_refused_and_one_is_accepted`, `C::options::tests::max_file_bytes_is_set`, `T::units::max_file_bytes_caps_a_file_and_the_cap_itself_loads` | Exact function bodies should be reread in final insertion: options tests pin defaults/setters; `units.rs:486` is the actual Census load caller's exact cap/refusal neighbor. Non_exhaustive is a source/consumer compile property, not proved by setter assertions. |
| Errors and worker boundary; before `raf_census::Error is the one error type` | `T::forms::an_unlexable_slice_is_a_source_error_with_a_valid_neighbour`, `C::error::tests::each_variant_renders_its_facts`, `C::units::tests::collection_worker_panic_keeps_its_payload_and_tears_down_without_invalidating_caller_spans` | `forms.rs:298` checks exact source path/range/reason and valid lex neighbor. `error.rs:98` verifies spawn display escaping and retained nested source, but constructs the spawn error manually. Worker unit test at `units/mod.rs:381` catches actual worker panic, downcasts original payload 37, checks TLS destructor count and preserved caller span. No forced actual spawn failure was found. |
| File context and target kinds; before `raf_census::FileCtx describes` | `T::units::file_context_names_package_path_and_shipping`, `T::units::file_context_names_the_target_kinds_that_reach_the_file`, `T::units::shipping_is_true_when_any_reach_ships_and_the_shipping_path_is_the_symbol` | `units.rs:265` checks package/path/shipping/missing lookup and exact file count; line 282 checks Lib/Test/Example sets; line 225 checks shared Lib+Test kinds and shipping symbol preference. Not every kind needs to be inferred from these three fixtures; binary/bench/build-script matrix remains a coverage candidate. |
| One unit per source identity; before `Units are collected once per source item` | `T::units::a_module_declared_by_a_library_and_a_binary_is_one_unit`, `T::units::a_support_module_shared_by_two_test_binaries_is_one_unit_that_does_not_ship`, `T::units::shipping_is_true_when_any_reach_ships_and_the_shipping_path_is_the_symbol`, `T::units::test_code_of_a_shipping_target_does_not_ship` | `units.rs:175` checks one shared unit plus main; line 197 checks one test-support unit, nonshipping, exact smallest path and shipping neighbor; line 225 proves shipping path wins despite sort order; line 254 checks library test helper/test functions do not ship. |
| Unit contents and initializer-only extraction; split before `A function or method unit holds its print` | `T::units::each_kind_yields_one_unit_with_every_field`, `T::units::a_constant_whose_type_names_a_number_does_not_hold_it`, `T::units::every_spelling_of_an_integer_is_one_constant`, `T::units::byte_strings_and_c_strings_are_strings` | `units.rs:99` is the full field-matrix candidate. Inspected line 337 checks type length 65536 excluded while actual initializer 0/4 or 2/7 retained; line 358 unifies hex/decimal/binary/octal/suffixed 255; line 672 checks exact string kinds/lines, lossy non-UTF8 value and broken-table exclusion. |
| Group identity and ordering; before `The census groups units` | `T::groups::groups_of_one_are_dropped_and_members_follow_symbol_order`, `T::groups::groups_order_by_count_then_tokens_then_fingerprint`, `C::group::tests::members_order_by_symbol_then_file_then_line`, `C::group::tests::digest_suffixes_are_distinct_group_keys` | `groups.rs:63` asserts exact member list, kind, tokens and metadata, singleton omitted; line 97 checks exact count/token sequence and fingerprint tie order. `group.rs:262` tests symbol/file/line comparator neighbors. Its current helper uses start_byte 0 throughout, so it does not directly test the final byte-offset tie. |
| Hex tables and shared constants; before `Integer constants are compared by value` | `T::groups::every_form_of_a_hex_table_is_reported_in_both_cases`, `T::groups::near_misses_of_a_hex_table_are_not_reported`, `T::groups::constants_above_ffff_in_two_packages_are_shared`, `T::groups::a_constant_in_one_package_is_not_shared`, `T::groups::shared_constants_run_by_value_ascending` | `groups.rs:266` checks all eight case/carrier combinations; line 298 checks wrong/short/mixed negatives beside valid. Lines 325/358/370 prove cross-package 65536, 65535 boundary, same-package duplicate exclusion and ascending values. |
| Job matching and classification; before `A job has an id` | `T::groups::a_job_classifies_home_entry_point_variant_and_copies`, `T::groups::jobs_match_shim_and_table_fingerprints`, `T::groups::a_job_that_matches_nothing_has_an_empty_list` | `groups.rs:403` asserts exact Home/entry/Variant/Copy members, copies 2 and match file/line. Other two bodies are candidates for the remaining shim/table/rule-matches-none boundaries. |
| Shared report profile and format equivalence; before `The baseline and index are ground RDF 1.2 graphs` | `T::reports::both_reports_read_back_what_they_write`, `T::reports::output_is_the_canonical_turtle`, `C::report::read::tests::the_graph_of_each_example_is_its_pinned_triple_set` | `reports.rs:79` round-trips both reports through both formats and checks the ground graph equality, 7 units, NF1/minimum 20. Line 106 checks VERSION 1.2/BASE/canonical ending/no CR/tab/blank nodes. `report/read.rs:23` compares three fixtures to exact native NT, sorted independent purrdf-rdf pins and reparsed NT graph. Independent parser execution itself is retained external evidence, not rerun by this test. |
| Report identity and typed jobs; split before `Each unit's identity includes` | `T::reports::two_cfg_alternatives_on_one_line_have_distinct_unit_nodes`, `T::reports::the_index_renders_the_classified_matches_of_every_job`, `T::reports::a_shared_constant_is_an_integer`, `T::reports::orphan_units_and_matches_without_a_named_reifier_are_not_a_report` | `reports.rs:372` proves same symbol/line plus different start bytes yield two distinct units and round-trip. Line 415 checks exact job classes/copies/idle report and Variant IRI; line 222 removes named reifiers and demands refusal. The decimal integer assertion body at 467 remains a candidate reread. |
| Report profile 2 and preserved normal-form version; split before `Readers return the declared normal form` | `T::reports::a_report_of_another_report_form_is_refused_naming_both`, `T::reports::an_index_of_another_normal_form_reads_and_keeps_its_version`, `T::reports::names_that_disagree_with_their_node_are_not_a_report` | `reports.rs:323` changes reportForm to 1/3, renders valid graph in both formats and demands typed Form with implemented 2. Line 344 writes normal_form 7, checks `/nf/7/`, reads exact report and separately requires version gate refusal. Line 204 changes normalForm without fingerprint namespace change and requires rejection. Thus profile version and NF version are separate contracts. |
| Closed report shape; split before `Unknown terms, missing` | `T::reports::unknown_terms_are_not_a_report`, `T::reports::missing_repeated_and_mistyped_properties_are_not_a_report`, `T::reports::names_that_disagree_with_their_node_are_not_a_report`, `T::reports::orphan_units_and_matches_without_a_named_reifier_are_not_a_report`, `C::report::shape::tests::every_required_shape_property_has_a_missing_and_unknown_neighbor` | `reports.rs:116` checks extra/foreign predicates and unknown class; line 157 missing minimum, wrong normalForm type, repeated body and bad match class/copy count; line 204 node-property mismatch; line 222 orphan/wrong fingerprint kind/no named reifier. `report/shape.rs:493` validates baseline/index positives, covers every non-debt SHAPES row with an instantiated class, rejects unknown property and every required-property deletion. |
| Canonical refusals and writer validation; split before `Alternate spellings` | `T::reports::an_edited_or_respaced_text_is_not_a_report`, `T::reports::canonical_line_diagnostics_preserve_crlf_and_the_final_newline`, `T::reports::the_writer_refuses_a_value_outside_the_shape` | `reports.rs:283` separately refuses whitespace change and edited minimum with wrong digest; line 295 checks typed CRLF/final-newline locations; line 396 truncates a body group's members and requires writer refusal. Generic syntax/refusal rules belong to the shared profile, rather than pretending this domain test covers every RDF spelling. |

## Constraints: preserve and assign specifically

1. **Source only, no macro expansion**: bind the actual normal-form registry and
   unit tests demonstrating macro arguments are tokens; retain the syntactic
   scope. Source parsing/macro item generation is the source-index capability's
   limitation, not something census tests make universally equivalent to rustc.
2. **Macro arguments and invalid slice**: use
   `T::forms::an_unlexable_slice_is_a_source_error_with_a_valid_neighbour` and
   `T::groups::renamed_locals_inside_macro_arguments_land_in_one_body_group`.
   The first was inspected; the grouping test and actual registry macro pairs
   should receive an assertion reread before marker insertion.
3. **Doc comment tokens versus ordinary comments**: the registry includes
   `doc_comment_renamed_local` and `doc_comment_text_differs`, under the inspected
   `every_registered_pair_compares_as_registered` loop. Inspect their actual
   expected pair levels before claiming the ordinary-comment neighbor too.
4. **Minimum and shims**: use the explicit body-minimum, 5/6/29/30 and
   forwarder-group markers above. Do not attach an options-value assertion alone.
5. **Complete digest/collision assumption**: use full-digest/suffix-separation and
   abbreviation-refusal tests for representation. Collision resistance is the
   cryptographic assumption explicitly stated, not a universal equality theorem
   any fixture can prove. Keep that limitation without inventing a test for it.
6. **Named table/rule fingerprints**: inspected parse/display fixtures at
   `forms.rs:419` include both kinds; malformed-name/sort tests are bound
   candidates for the exact grammar/order remainder.
7. **Portability limit**: preserve the current dependency/native-worker
   limitation; the pure BLAKE3 dependency and hand-encoded hash do not prove a
   complete core WASM build or runtime. No current complete portable acceptance
   is claimed by this map.
8. **Native collection/caller spans**: give this its own named clause. Marker
   paths should include `T::units::caller_spans_survive_a_successful_census_load`,
   `T::units::caller_spans_survive_a_refused_census_load`,
   `T::units::loading_one_tree_twice_preserves_the_callers_span_storage_and_unit_results`,
   `T::units::loading_a_tree_with_caller_local_state_needs_no_sync_bound`, and
   `T::units::a_held_standalone_tower_keeps_its_forms_and_print_across_census_loads`.
   At `units.rs:834`, caller span start/end/byte_range/source_text are captured
   before and after actual successful/refused loads. Line 936 observes source
   ordinals parsed string 1/2/3 on a fresh caller thread and compares actual units
   after repeated loads: it does not mistake file-relative byte offsets for TLS
   cleanup. Line 873 compiles and reads a Cell-backed non-Sync Tree on the caller.
   Line 911 holds an actual standalone Tower and compares forms/prints across
   successful/refused Census calls. This proves composability, not cleanup of
   the standalone Tower's caller-thread lexer registration.
9. **One unit for shared targets**: reuse the exact library/binary and two-test
   shared-file markers. A per-target assertion should never stand in for this.
10. **Shim candidate excluded from body grouping**: the inspected 24-token
    integration test explicitly checks both retained prints but no body group.
11. **Lexical failure and source parsing**: use the inspected invalid Slice and
    deep-signature source-load refusal tests; do not claim census parses a whole
    signature or item itself.
12. **Initializer-only constants/strings/tables**: the inspected constant-type
    neighbor at `units.rs:337` demonstrates numbers in types are excluded. The
    full unit kind test must also prove constants/statics have neither print nor
    fingerprint. Attribute exclusion requires an actual differing attribute
    payload neighbor if the existing fixture does not contain one.
13. **Lossy non-UTF8 strings**: `T::units::byte_strings_and_c_strings_are_strings`
    directly checks U+FFFD replacement and no table from broken bytes.
14. **Unbound names and whole-body binding**: first-appearance numbering and
    field preservation tests above establish specific boundaries. The registry's
    shadows/local/macro cases support the named-source behavior; inspect the
    whole-body shadow fixture before assigning that stronger scope statement.
15. **No signature/item reparse; depth and ownership**: split the final long
    constraint into `Source-owned signature facts`, `Bounded body lexing`,
    `Collection worker teardown` and `Caller span preservation`.
    `T::units::a_deep_signature_loads_on_a_small_stack_and_beyond_the_budget_is_refused`
    at 795 runs Census on a 2 MiB caller, accepts depth 8 × 60 and checks params,
    then refuses 24 × 60 with an actual Error::Load naming recursion.
    `T::units::a_deep_constant_type_loads_on_a_small_stack` at 826 checks actual
    initializer constant 0. `T::forms::nesting_at_the_budget_lexes_and_one_deeper_is_refused`
    at 319 accepts 128 with 257 tokens and refuses 129.
    `C::units::tests::collection_worker_tears_down_on_normalization_success_and_error_beside_a_preserved_caller_span`
    at `units/mod.rs:348` tests actual success/refused normalization on fresh
    workers, two TLS destructor executions and caller-span preservation.
    The panic counterpart at 381 checks destructor count 1 and original u32
    payload. The implementation at 138–155 borrows completed Workspace into a
    scoped worker, sets exactly 2 MiB and resumes payload; Tree is not captured.

## Shared production helpers and evidence boundaries

`Tower::literal_values` supplies both retained and abstracted value forms to the
selection example; it does not reimplement literal normalization there.
`T::forms::literal_value_introspection_selects_the_production_default` at
`forms.rs:497` checks both lengths against value_tokens and selects by the real
LITERAL_TIER for every fixture, asserting equality with production Level::Value.
`C::example_crate::selection::tests::fixture_counts_are_pinned_and_default_selection_matches_production`
at `examples/selection.rs:379` compares its policy key with actual Tower prints,
shim eligibility and forms for every fixture; it pins 261 bodies, 3900 tokens,
22 non-singleton groups and 54 pairs. `Body::new` at line 39 uses the actual
literal helper, value token count, shim form/candidate and Self-call query. Its
policy key still deliberately implements alternative floor/tier policy
selection; the test proves the chosen default agrees over the registry, not
that every hypothetical policy is a production contract. These are appropriate
markers for any claim about the experiment's shared normalization, not for a
new promise that all alternative policies are supported production APIs.

NF1 golden fixture hashes and REPORT_FORM 2 are distinct. Keep the NF1 pinned
fixture/manifest/version tests attached to normalization and digest semantics;
keep profile 1/3 refusal and NF7-preserving report read attached to reports.
Independent RDF graph pins are canonical sorted assertions, while native NT
fixtures pin actual writer order. The source-inspected test checks those bytes;
it does not rerun the independent RDF parser or inherit its acceptance merely
from the `.oracle.nt` suffix.

## Genuine missing proof or wording distinctions

- The source guarantees no whole-signature/item parse in census. Inspected
  `collect_units` consumes completed Workspace items and extracts body or
  initializer; the deep fixture is a concrete anti-regression. The absolute
  wording `no signature of any depth can overflow` also relies on raf-source's
  universal prescan/stack invariant. The two tested depths alone do not prove
  that universal quantifier. Preserve the runtime requirement and its source
  dependency; do not represent the sampled fixture as a mathematical proof.
- Actual OS refusal to create a collection worker is not exercised by the
  inspected tests. Source maps spawn_scoped errors to CollectionSpawn, and the
  error unit test verifies its chain/display using a constructed I/O error.
  Distinguish this source-proved path from an executed failure injection.
- Cell-backed Tree proves no Sync requirement; the public function signature
  and source boundary carry no Send bound either. A separate Rc-backed fixture
  would directly test !Send if required, but the existing Cell fixture should
  not be described as testing !Send by itself.
- Corrupt-source-index invariant errors (missing item id, no reach, invalid
  source span) are actual hard-error branches inspected in collection source;
  no public-fixture runtime negative was identified. They cannot normally be
  reached through a valid constructed Workspace. Do not label an unrelated
  malformed Rust file as coverage of those invariant branches.
- Full pointer-width/endian and complete WASM acceptance remain separate from
  the portable token-byte spelling and pure hash dependency. Do not weaken
  current source portability limits or claim that a hash-only target build
  proves a complete load.
- Normal-form immutability after first merge is a repository compatibility
  policy. Golden pins catch the checked fixture changes, but no test determines
  Git's first merge or proves every fingerprint-affecting future change bumps a
  version and supplies migration. Keep that as explicit compatibility policy
  (or move the policy explanation to Explanation) without a false runtime marker.
- The shared report profile owns generic RDF syntax, content addressing and
  IRI restrictions. Census domain tests and shared-library tests should be
  complementary; a census marker must name actual bound tests rather than
  silently refer to another capability's tests as though they ran here.

Unlike source-index's historical memory catalogues, this Contract has little
measurement prose to relocate. Keep its real functional guarantees in Contract.
Only compatibility rationale, cryptographic assumption explanation and the
selection experiment's observed counts belong naturally in Explanation, with
their limits preserved. Missing proof is not permission to drop a guarantee.
