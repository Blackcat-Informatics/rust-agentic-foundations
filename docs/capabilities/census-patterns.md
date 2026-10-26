---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-patterns
title: "Declarative census patterns"
description: "Native RDF declarations for call, path, macro, type, string and balanced-token syntax bans."
state: delivered
bindings:
  code:
    - raf_census::gate::patterns
    - raf_census::syntax
    - raf_census::Gate::matching_limits
  tests:
    - raf_census::test_crate::census_gate::many_constant_sites_keep_indexed_matching_within_the_default_grant
    - raf_census::gate::patterns::paths::cache::tests::index_growth_retains_the_original_grant_and_old_storage_on_refusal
    - raf_census::gate::patterns::paths::cache::tests::native_storage_failures_keep_their_typed_cause
    - raf_census::test_crate::census_gate::literal_patterns_exclude_doc_comments_but_keep_written_doc_attributes
    - raf_census::gate::patterns::paths::tests::ownership_scans_spend_the_grant_once_per_byte_and_bound_retention
    - raf_census::test_crate::census_gate::transitive_block_imports_classify_their_expanded_namespace
    - raf_census::test_crate::census_gate::import_targets_keep_their_declaration_frame
    - raf_census::test_crate::census_gate::namespace_dependencies_refuse_cycles_beside_bounded_transitive_chains
    - raf_census::test_crate::census_gate::qualified_pattern_heads_use_types_beside_value_and_macro_bindings
    - raf_census::test_crate::census_gate::qualified_dependency_heads_ignore_value_imports_and_keep_type_imports
    - raf_census::test_crate::census_gate::same_head_imports_keep_distinct_type_and_value_bindings
    - raf_census::test_crate::census_gate::competing_import_namespaces_refuse_beside_distinct_namespace_imports
    - raf_census::test_crate::census_gate::default_hasher_detector_uses_visible_std_paths_and_lexical_aliases
    - raf_census::test_crate::census_gate::anonymous_qualified_receivers_refuse_beside_supported_named_receivers
    - raf_census::test_crate::census_gate::mixed_constant_binding_reaches_refuse_beside_consistent_constant_reaches
    - raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence
    - raf_census::test_crate::census_gate::lexical_patterns_match_one_token_or_group_but_never_an_unbalanced_run
    - raf_census::test_crate::census_gate::lexical_patterns_reuse_local_normalization_and_keep_parameter_context
    - raf_census::test_crate::census_gate::lexical_patterns_match_decoded_strings_in_macros_and_attributes_but_not_bytes
    - raf_census::test_crate::census_gate::lexical_patterns_do_not_mistake_a_file_root_for_its_first_written_unit
    - raf_census::test_crate::census_gate::lexical_patterns_share_an_explicit_native_budget_and_release_only_their_own_spans
    - raf_census::test_crate::census_gate::lexical_pattern_module_homes_own_source_subjects_without_sanctioning_the_file_root
    - raf_census::test_crate::census_gate::lexical_pattern_module_homes_include_macros_exported_from_their_actual_source_scope
    - raf_census::test_crate::census_gate::lexical_pattern_variants_require_the_actual_item_when_a_whole_file_span_ties_its_root
    - raf_census::test_crate::census_gate::token_patterns_include_file_level_macro_arguments_without_renaming_item_names
    - raf_census::test_crate::census_gate::token_patterns_normalize_body_local_declarations_without_renaming_module_items
    - raf_census::test_crate::census_gate::path_pattern_kinds_execute_with_positive_and_negative_syntax_neighbours
    - raf_census::test_crate::census_gate::call_patterns_follow_written_imports_and_respect_parameters_locals_and_closures
    - raf_census::test_crate::census_gate::type_patterns_follow_aliases_without_capturing_generic_or_module_shadows
    - raf_census::test_crate::census_gate::workspace_call_patterns_follow_cross_crate_reexports_through_actual_source_sites
    - raf_census::test_crate::census_gate::macro_name_patterns_follow_aliases_but_not_a_differently_named_local_macro
    - raf_census::test_crate::census_gate::block_import_patterns_use_the_import_binding_instead_of_a_shadowed_module_function
    - raf_census::test_crate::census_gate::constant_pattern_identifiers_keep_their_actual_reference_in_match_arms
    - raf_census::test_crate::census_gate::call_patterns_observe_macro_repetition_and_statement_arguments_with_clean_neighbours
    - raf_census::test_crate::census_gate::general_path_patterns_include_grouped_renamed_imports_in_their_actual_module_scope
    - raf_census::test_crate::census_gate::call_patterns_include_qualified_trait_and_inherent_paths_without_receiver_guessing
  examples:
    - raf_census::example_crate::patterns
depends: [cap:census-detectors, cap:census-ledger, cap:source-index]
---

# Declarative census patterns

## Purpose

A native ledger can ban a particular syntax shape without a repository Rule
implementation. The same Gate evaluates its matches and the job's approved homes
and variants.
<!-- raf:tested-by raf_census::test_crate::census_gate::path_pattern_kinds_execute_with_positive_and_negative_syntax_neighbours -->

## Contract

**Closed declarations.** A ledger links each Pattern through raf:pattern. A declaration has a unique
raf:patternId, one-line raf:summary and exactly one matcher:

| Matcher | Observation |
| --- | --- |
| raf:call | An explicit function-path call, including parenthesized and named qualified paths. |
| raf:path | A path occurrence in expressions, types, attributes or imports. |
| raf:macro | A written macro invocation; a single-segment name also follows written import aliases. |
| raf:typePath | A named type path, trait bound or type prefix of a qualified expression path. |
| raf:literalRegex | A decoded ordinary string literal matched by the native bounded ECMA evaluator. |
| raf:tokens | A body-normalized token sequence, where $_ consumes exactly one token or balanced group. |
<!-- raf:tested-by raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence -->

Documentation comments are prose, not written string literals; literal matching
excludes their desugared leaves. Explicit `#[doc = "..."]` attributes remain
ordinary string literals.
<!-- raf:tested-by raf_census::test_crate::census_gate::literal_patterns_exclude_doc_comments_but_keep_written_doc_attributes -->

**Validated references.** Patterns share the RuleId namespace with built-ins and registered rules. Missing,
duplicate, unknown and unreferenced declarations refuse. A referenced pattern with
no occurrence is stale for an enforced job. Jobs select ids with
raf:forbiddenPattern; a declaration alone never makes the Gate clean.
<!-- raf:tested-by raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence,raf_census::test_crate::census_gate::path_pattern_kinds_execute_with_positive_and_negative_syntax_neighbours -->

**Source and lexical context.** Workspace path and call aliases and reexports use Source's shared resolver and
actual definition sites. Source retains the owner's reading module and target;
code and import lookup preserve their different edition rules. Syntactic lexical
scopes account for parameters, generic bindings, local declarations and imports,
initializer-before-binding order, closures, conditional bindings and match arms.
Known constant patterns retain their Source identity instead of becoming locals.
Qualified path heads use the Type namespace; bare values and macros use their
own namespaces. Indexed imports retain each applicable namespace binding, so
a same-named value cannot hide a dependency's qualified path. A Type-only
inner import does not hide an outer callable value. Genuine block Type and
module shadows remain authoritative.
<!-- raf:tested-by raf_census::test_crate::census_gate::qualified_pattern_heads_use_types_beside_value_and_macro_bindings,raf_census::test_crate::census_gate::qualified_dependency_heads_ignore_value_imports_and_keep_type_imports,raf_census::test_crate::census_gate::same_head_imports_keep_distinct_type_and_value_bindings,raf_census::test_crate::census_gate::workspace_call_patterns_follow_cross_crate_reexports_through_actual_source_sites,raf_census::test_crate::census_gate::call_patterns_follow_written_imports_and_respect_parameters_locals_and_closures,raf_census::test_crate::census_gate::type_patterns_follow_aliases_without_capturing_generic_or_module_shadows,raf_census::test_crate::census_gate::block_import_patterns_use_the_import_binding_instead_of_a_shadowed_module_function,raf_census::test_crate::census_gate::constant_pattern_identifiers_keep_their_actual_reference_in_match_arms -->

Import namespaces are checked after their target's block aliases have expanded
in their declaration frames; younger scopes cannot change an imported target.
Direct occurrences still observe their own lexical shadows.
An unrelated module with the same written prefix cannot suppress a real match.
Namespace dependencies use the same bounded expansion engine without recursive
Rust calls, and cyclic dependencies refuse beside acyclic chains.
<!-- raf:tested-by raf_census::test_crate::census_gate::transitive_block_imports_classify_their_expanded_namespace,raf_census::test_crate::census_gate::namespace_dependencies_refuse_cycles_beside_bounded_transitive_chains,raf_census::test_crate::census_gate::import_targets_keep_their_declaration_frame -->

**Physical ownership.** Each physical source owner counts once. HOME covers actual module definitions,
including relocated macro exports and implementation members whose displayed
type paths belong elsewhere. VARIANT requires the actual source identity and
declared file. A crate-root span cannot sanction the first written item merely
because both start at byte zero. Unsanctioned matches report rule:<id> with the
source location, summary and required home.
<!-- raf:tested-by raf_census::test_crate::census_gate::lexical_patterns_do_not_mistake_a_file_root_for_its_first_written_unit,raf_census::test_crate::census_gate::lexical_pattern_module_homes_own_source_subjects_without_sanctioning_the_file_root,raf_census::test_crate::census_gate::lexical_pattern_module_homes_include_macros_exported_from_their_actual_source_scope,raf_census::test_crate::census_gate::lexical_pattern_variants_require_the_actual_item_when_a_whole_file_span_ties_its_root -->

**Stable token observations.** Token search includes file-level syntax and macro arguments. Existing normal-form
passes normalize each implementation body or initializer in its own parameter
context; module declaration names remain literal. A local function declaration
inside a body follows that body's local-name normalization. Wildcards never
consume an unbalanced or variable-length run. The original fingerprint path and
normal-form version are unchanged.
<!-- raf:tested-by raf_census::test_crate::census_gate::lexical_patterns_match_one_token_or_group_but_never_an_unbalanced_run,raf_census::test_crate::census_gate::lexical_patterns_reuse_local_normalization_and_keep_parameter_context,raf_census::test_crate::census_gate::token_patterns_include_file_level_macro_arguments_without_renaming_item_names,raf_census::test_crate::census_gate::token_patterns_normalize_body_local_declarations_without_renaming_module_items -->

## Usage

Run `raf_census::example_crate::patterns` for a memory-backed canonical ledger
and paired direct-call and wrapper-call checks.

Attach a pattern to the job whose home implements the approved operation, then
call Gate::workspace(directory).check() or assert_clean(). check_tree uses
the same production evaluation over a caller-owned Tree. matching_limits
sets the shared native matching step/state allowance; source loading retains
its original separate allowance.
<!-- raf:tested-by raf_census::test_crate::census_gate::path_pattern_kinds_execute_with_positive_and_negative_syntax_neighbours,raf_census::test_crate::census_gate::lexical_patterns_share_an_explicit_native_budget_and_release_only_their_own_spans -->

## Examples

The runnable `raf_census::example_crate::patterns` constructs this policy with
`raf_rdf::Builder`, serializes its canonical snapshot and checks both a direct
release copy and a clean wrapper call. This Turtle fragment shows the graph's
local names; the native writer supplies its content-addressed BASE:

~~~turtle
@prefix raf: <https://blackcatinformatics.ca/raf/report/1#> .
<> a raf:Ledger ; raf:normalForm 1 ; raf:reportForm 4 ;
   raf:job <#release> ; raf:pattern <#direct-drop> .
<#release> a raf:LedgerJob ; raf:jobId "release" ;
   raf:summary "Release values through the approved wrapper" ;
   raf:homeSymbol "demo::release" ; raf:enforced true ;
   raf:forbiddenPattern "direct-drop" .
<#direct-drop> a raf:Pattern ; raf:patternId "direct-drop" ;
   raf:summary "A direct release call outside its wrapper" ;
   raf:call "std::mem::drop" .
~~~

An equivalent tokens declaration uses raf:tokens "std :: mem :: drop ( $_ )".
A string declaration can use raf:literalRegex "^http://"; raw and escaped
ordinary strings are decoded before matching, while byte and C strings are
excluded. The production fixtures demonstrate positive and negative examples
for every matcher, renamed imports and actual cross-crate reexports.
<!-- raf:tested-by raf_census::test_crate::census_gate::path_pattern_kinds_execute_with_positive_and_negative_syntax_neighbours,raf_census::test_crate::census_gate::lexical_patterns_match_one_token_or_group_but_never_an_unbalanced_run,raf_census::test_crate::census_gate::lexical_patterns_match_decoded_strings_in_macros_and_attributes_but_not_bytes,raf_census::test_crate::census_gate::workspace_call_patterns_follow_cross_crate_reexports_through_actual_source_sites -->

## Constraints

Source ownership queries reuse the native index once per file and byte across
patterns and constant observations. Candidate scans spend the matching step
grant before lookup. The native ordered table charges logarithmic lookup and
update bounds, and exact arena growth layouts before allocation under the same
original grant. Cache associations and retained reference-vector capacity,
including spare cells, spend the state grant. The exact arena admission does
not cover Source's existing query scratch or the indexed items it lends.
Native index allocation/layout failures retain their typed cause. Generous
state ceilings do not fail merely because a derived allowance exceeds the
address space; actual unrepresentable allocation layouts still refuse.
<!-- raf:tested-by raf_census::gate::patterns::paths::tests::ownership_scans_spend_the_grant_once_per_byte_and_bound_retention,raf_census::gate::patterns::paths::cache::tests::index_growth_retains_the_original_grant_and_old_storage_on_refusal,raf_census::gate::patterns::paths::cache::tests::native_storage_failures_keep_their_typed_cause -->

Thousands of distinct constant/binding observations retain indexed matching
under the default grant, beside a different-call neighbor and a smaller-grant
refusal. Normal-form tokens and fingerprints are unchanged.
<!-- raf:tested-by raf_census::test_crate::census_gate::many_constant_sites_keep_indexed_matching_within_the_default_grant -->

- Competing written imports that do not identify one applicable namespace
  binding refuse. Provide indexed definitions or distinct aliases to make
  the import context unambiguous.
<!-- raf:tested-by raf_census::test_crate::census_gate::competing_import_namespaces_refuse_beside_distinct_namespace_imports -->

- Anonymous qualified receivers with a matching suffix and bare patterns whose
binding meaning differs across source reaches refuse beside supported named
receivers and consistently constant reaches.
<!-- raf:tested-by raf_census::test_crate::census_gate::anonymous_qualified_receivers_refuse_beside_supported_named_receivers,raf_census::test_crate::census_gate::mixed_constant_binding_reaches_refuse_beside_consistent_constant_reaches -->

- Unindexed external or standard-library paths can be compared as written syntax
after available file/block aliases. That spelling is explicitly unverified;
it authenticates no package, source identity, namespace or meaningful delegation.
Preferred-provider enforcement requires separate grounded evidence. A known
different workspace definition is never replaced by a spelling match.
<!-- raf:tested-by raf_census::test_crate::census_gate::type_patterns_follow_aliases_without_capturing_generic_or_module_shadows,raf_census::test_crate::census_gate::block_import_patterns_use_the_import_binding_instead_of_a_shadowed_module_function -->

- Matching does not expand macros or infer receiver types for method-call syntax.
Macro expression-list and statement/repetition arguments are observed through
the existing Rust parser; other macro languages remain token syntax. Named
qualified type/trait paths are supported. A relevant anonymous qualified receiver
or mixed constant/binding context refuses rather than fabricating an identity.
Structural conditions such as loop membership belong in a registered Rule.
<!-- raf:tested-by raf_census::test_crate::census_gate::call_patterns_observe_macro_repetition_and_statement_arguments_with_clean_neighbours,raf_census::test_crate::census_gate::call_patterns_include_qualified_trait_and_inherent_paths_without_receiver_guessing -->

- Matching exhaustion and allocation refusal are errors. Temporary lexical sessions
own their spans and release only their own storage on success or refusal;
caller-owned proc-macro spans remain usable. ECMA is the native dialect; no
parallel regex engine or second Rust tokenizer is used.
<!-- raf:tested-by raf_census::test_crate::census_gate::lexical_patterns_share_an_explicit_native_budget_and_release_only_their_own_spans -->

## Explanation

The closed matcher set keeps syntax policy beside the job it protects. Workspace
definition lookup stays in Source. Shared import observations serve the built-in
hasher detector and declared patterns; normal-form tokens and literal decoding
are reused. Native matching limits make unsupported work visible.
<!-- raf:tested-by raf_census::test_crate::census_gate::default_hasher_detector_uses_visible_std_paths_and_lexical_aliases,raf_census::test_crate::census_gate::path_pattern_kinds_execute_with_positive_and_negative_syntax_neighbours -->

## Verification

Production Gate fixtures cover every matcher's positive and negative case,
balanced wildcards, parameter/local normalization, decoded strings and byte/C
exclusions, real workspace reexports, lexical shadows, match constants, macro
arguments, imports and qualified calls. Ownership controls pair approved homes
and exact variants with unsanctioned neighbours. Empty declarations, stale
patterns and exhausted grants refuse beside supported cases.
<!-- raf:tested-by raf_census::test_crate::census_gate::path_pattern_kinds_execute_with_positive_and_negative_syntax_neighbours,raf_census::test_crate::census_gate::lexical_patterns_match_one_token_or_group_but_never_an_unbalanced_run,raf_census::test_crate::census_gate::lexical_patterns_share_an_explicit_native_budget_and_release_only_their_own_spans -->

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_census::example_crate::patterns` | example target | [Usage](#usage), [Examples](#examples) |
| `raf_census::gate::patterns::paths::cache::tests::index_growth_retains_the_original_grant_and_old_storage_on_refusal` | test | [Constraints](#constraints) |
| `raf_census::gate::patterns::paths::cache::tests::native_storage_failures_keep_their_typed_cause` | test | [Constraints](#constraints) |
| `raf_census::gate::patterns::paths::tests::ownership_scans_spend_the_grant_once_per_byte_and_bound_retention` | test | [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::anonymous_qualified_receivers_refuse_beside_supported_named_receivers` | test | [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::block_import_patterns_use_the_import_binding_instead_of_a_shadowed_module_function` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::call_patterns_follow_written_imports_and_respect_parameters_locals_and_closures` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::call_patterns_include_qualified_trait_and_inherent_paths_without_receiver_guessing` | test | [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::call_patterns_observe_macro_repetition_and_statement_arguments_with_clean_neighbours` | test | [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::competing_import_namespaces_refuse_beside_distinct_namespace_imports` | test | [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::constant_pattern_identifiers_keep_their_actual_reference_in_match_arms` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::default_hasher_detector_uses_visible_std_paths_and_lexical_aliases` | test | [Explanation](#explanation) |
| `raf_census::test_crate::census_gate::general_path_patterns_include_grouped_renamed_imports_in_their_actual_module_scope` | test | [Verification](#verification) |
| `raf_census::test_crate::census_gate::import_targets_keep_their_declaration_frame` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::lexical_pattern_module_homes_include_macros_exported_from_their_actual_source_scope` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::lexical_pattern_module_homes_own_source_subjects_without_sanctioning_the_file_root` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::lexical_pattern_variants_require_the_actual_item_when_a_whole_file_span_ties_its_root` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::lexical_patterns_do_not_mistake_a_file_root_for_its_first_written_unit` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::lexical_patterns_match_decoded_strings_in_macros_and_attributes_but_not_bytes` | test | [Examples](#examples) |
| `raf_census::test_crate::census_gate::lexical_patterns_match_one_token_or_group_but_never_an_unbalanced_run` | test | [Contract](#contract), [Examples](#examples) |
| `raf_census::test_crate::census_gate::lexical_patterns_reuse_local_normalization_and_keep_parameter_context` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::lexical_patterns_share_an_explicit_native_budget_and_release_only_their_own_spans` | test | [Usage](#usage), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::literal_patterns_exclude_doc_comments_but_keep_written_doc_attributes` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::macro_name_patterns_follow_aliases_but_not_a_differently_named_local_macro` | test | [Verification](#verification) |
| `raf_census::test_crate::census_gate::many_constant_sites_keep_indexed_matching_within_the_default_grant` | test | [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::mixed_constant_binding_reaches_refuse_beside_consistent_constant_reaches` | test | [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::namespace_dependencies_refuse_cycles_beside_bounded_transitive_chains` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::path_pattern_kinds_execute_with_positive_and_negative_syntax_neighbours` | test | [Purpose](#purpose), [Contract](#contract), [Usage](#usage), [Examples](#examples), [Explanation](#explanation) |
| `raf_census::test_crate::census_gate::qualified_dependency_heads_ignore_value_imports_and_keep_type_imports` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::qualified_pattern_heads_use_types_beside_value_and_macro_bindings` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::same_head_imports_keep_distinct_type_and_value_bindings` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::token_patterns_include_file_level_macro_arguments_without_renaming_item_names` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::token_patterns_normalize_body_local_declarations_without_renaming_module_items` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::transitive_block_imports_classify_their_expanded_namespace` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::type_patterns_follow_aliases_without_capturing_generic_or_module_shadows` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::workspace_call_patterns_follow_cross_crate_reexports_through_actual_source_sites` | test | [Contract](#contract), [Examples](#examples) |
<!-- raf:generated:end -->
