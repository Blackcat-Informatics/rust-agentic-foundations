---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-patterns
title: "Declarative census patterns"
description: "Native RDF declarations for call, method, path, macro, type, string and balanced-token syntax bans."
state: delivered
bindings:
  code:
    - raf_census::gate::patterns
    - raf_census::gate::ratchet
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
    - raf_census::test_crate::census_gate::ratchet::ratcheted_patterns_record_counted_sites_and_hold_both_directions
    - raf_census::test_crate::census_gate::ratchet::shipping_scope_hits_a_library_and_stays_silent_in_tests_test_modules_and_build_scripts
    - raf_census::test_crate::census_gate::ratchet::target_kind_scope_includes_and_excludes_kinds_beside_unscoped_hits
    - raf_census::test_crate::census_gate::ratchet::path_scope_is_silent_inside_an_excluded_prefix_and_reports_outside_it
    - raf_census::test_crate::census_gate::ratchet::contradictory_or_unknown_scope_refuses_at_load_beside_a_valid_neighbour
    - raf_census::test_crate::census_gate::ratchet::shipping_scope_follows_the_test_role_of_a_module_level_macro_invocation
    - raf_census::test_crate::census_gate::ratchet::a_shared_file_hit_is_scoped_by_the_target_whose_resolution_matched
    - raf_census::test_crate::census_gate::lexical_patterns_match_one_token_or_group_but_never_an_unbalanced_run
    - raf_census::test_crate::census_gate::lexical_patterns_reuse_local_normalization_and_keep_parameter_context
    - raf_census::test_crate::census_gate::indexed_type_patterns_follow_written_aliases_at_their_uses
    - raf_census::test_crate::census_gate::macro_type_hits_use_the_scope_of_the_reach_that_resolves_the_type
    - raf_census::test_crate::census_gate::written_type_alias_hits_share_physical_sites_and_respect_shipping_scope
    - raf_census::test_crate::census_gate::written_receiver_methods_keep_declaration_identity_and_runtime_shadowing
    - raf_census::test_crate::census_gate::written_receiver_method_scope_keeps_the_matching_reach_of_a_shared_file
    - raf_census::test_crate::census_gate::written_receiver_method_selector_refuses_non_nominal_and_malformed_paths
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
depends: [cap:census-detectors, cap:census-ledger, cap:source-index, cap:source-reference-namespace]
---

# Declarative census patterns

## Purpose

A native ledger can ban a particular syntax shape without a repository Rule
implementation. The same Gate evaluates its matches and the job's approved homes
and variants.
<!-- raf:tested-by raf_census::test_crate::census_gate::path_pattern_kinds_execute_with_positive_and_negative_syntax_neighbours -->

## Contract

Selected reference packages participate in the same structural source index
as first-party path and call occurrences. Renamed dependencies and public
reexports match actual definition identities. A selected reference target
without an indexed definition refuses rather than becoming an unverified
spelling match. Admission uses the native failed lookup's attempted targets,
including private aliases whose declared paths begin in a first-party crate.
Unrelated unindexed external spellings retain their syntax policy.
Rule subjects, consumers and debt retain their original
first-party scope.
<!-- raf:tested-by raf_census::test_crate::census_gate::workspace_call_patterns_follow_cross_crate_reexports_through_actual_source_sites -->

**Closed declarations.** A ledger links each Pattern through raf:pattern. A declaration has a unique
raf:patternId, one-line raf:summary and exactly one matcher. The numeric
inventory reserves its three guard IDs for the Ledger's numeric bounds registry.

| Matcher | Observation |
| --- | --- |
| raf:call | An explicit function-path call, including parenthesized and named qualified paths. |
| raf:path | A path occurrence in expressions, types, attributes or imports. |
| raf:macro | A written macro invocation; a single-segment name also follows written import aliases. |
| raf:typePath | A named type path, trait bound or type prefix of a qualified expression path. |
| raf:methodOn | A dot call on a local name whose written type proves the selected nominal receiver. |
| raf:literalRegex | A decoded ordinary string literal matched by the native bounded ECMA evaluator. |
| raf:tokens | A body-normalized token sequence, where $_ consumes exactly one token or balanced group. |
<!-- raf:tested-by raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence -->

**Scope.** A pattern applies to every target kind and path unless it declares a scope. The optional
properties raf:targetKind and raf:excludeTargetKind (lib, bin, test, bench, example or buildScript),
raf:includePath and raf:excludePath (workspace-relative path prefixes, matched by whole path
segment, so src/legacy covers src/legacy/mod.rs but not src/legacy.rs), and raf:shippingOnly
(stated only as true) narrow it. A hit is kept when its file lies under an include prefix (or none
is declared) and under no exclude prefix, and the target that owns the item at the hit admits the
kind: it is in targetKind (or none is declared) and not in excludeTargetKind. raf:shippingOnly
admits only non-test code of a library or binary, so a #[cfg(test)] module, an integration test, a
bench, an example and a build script are silent. A module-level macro invocation is judged by its own test role and target, so a #[cfg(test)] invocation at a crate root is silent under raf:shippingOnly. A path or call hit in a file that several targets reach is judged by the targets whose own resolution of the written path matched the declared definition, so a scope on the library does not admit a hit that only an integration test resolves. A hit outside every item is judged by its file. An
unscoped pattern reports exactly as before. Unknown kind names, a kind both included and excluded,
a scope that admits no target kind, an include prefix inside an exclude prefix, a path that is
absolute, above the root or the root itself, and raf:shippingOnly stated as false refuse at ledger
load with the property to change. A referenced pattern whose every occurrence
is out of scope is stale for an enforced job unless the pattern is ratcheted.
<!-- raf:tested-by raf_census::test_crate::census_gate::ratchet::shipping_scope_hits_a_library_and_stays_silent_in_tests_test_modules_and_build_scripts,raf_census::test_crate::census_gate::ratchet::target_kind_scope_includes_and_excludes_kinds_beside_unscoped_hits,raf_census::test_crate::census_gate::ratchet::path_scope_is_silent_inside_an_excluded_prefix_and_reports_outside_it,raf_census::test_crate::census_gate::ratchet::contradictory_or_unknown_scope_refuses_at_load_beside_a_valid_neighbour,raf_census::test_crate::census_gate::ratchet::shipping_scope_follows_the_test_role_of_a_module_level_macro_invocation,raf_census::test_crate::census_gate::ratchet::a_shared_file_hit_is_scoped_by_the_target_whose_resolution_matched -->

Documentation comments are prose, not written string literals; literal matching
excludes their desugared leaves. Explicit `#[doc = "..."]` attributes remain
ordinary string literals.
<!-- raf:tested-by raf_census::test_crate::census_gate::literal_patterns_exclude_doc_comments_but_keep_written_doc_attributes -->

**Ratcheted patterns.** A pattern may state raf:ratcheted, only as true. Its hits are then not findings: each is recorded in the debt record as a counted site keyed by file, rule and a site key (see the debt ratchet), and the generic copy finding for the matched unit is suppressed only when the ratcheted pattern alone explains the match; a unit that is also a copy through a forbidden constant, a forbidden structural fingerprint or a non-ratcheted rule still fails. A ratcheted pattern with no hit stays a recorded guard rather than going stale. Every job that references it, by a declared pattern or a forbidden rule fingerprint, must be enforced; `ratcheted` stated as false, stated twice or stated for a pattern whose job is unenforced refuses at ledger load, and a pattern that does not state it is forbidden and keeps its hard failure. A literalRegex hit is keyed on its own literal. A job may hold a built-in detector or registered rule to the same record through ratchetedRule (see the ledger and rule-API capabilities). The property selects a rule forbidden by that job.
<!-- raf:tested-by raf_census::test_crate::census_gate::ratchet::ratcheted_patterns_record_counted_sites_and_hold_both_directions,raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence -->

**Validated references.** Patterns share the RuleId namespace with built-ins and registered rules. Missing,
duplicate, unknown and unreferenced declarations refuse. A referenced pattern with
no occurrence is stale for an enforced job, unless it is ratcheted. Jobs select ids with
raf:forbiddenPattern; a declaration alone never makes the Gate clean.
<!-- raf:tested-by raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence,raf_census::test_crate::census_gate::path_pattern_kinds_execute_with_positive_and_negative_syntax_neighbours -->

**Source and lexical context.** Workspace path and call aliases and reexports use Source's shared resolver and
actual definition sites. Source retains the owner's reading module and target;
code and import lookup preserve their different edition rules. Syntactic lexical
scopes account for parameters, generic bindings, local declarations and imports,
initializer-before-binding order, closures, conditional bindings and match arms.
Nested function items can use enclosing imports visible at their declaration.
A local or generic shadow at that declaration suppresses attribution to the
import. Closures can capture enclosing bindings. Union fields are read in the
union's generic scope.
Known constant patterns retain their Source identity instead of becoming locals.
Qualified path heads use the Type namespace; bare values and macros use their
own namespaces. Indexed imports retain each applicable namespace binding, so
a same-named value cannot hide a dependency's qualified path. A Type-only
inner import does not hide an outer callable value. Genuine block Type and
module shadows remain authoritative.
<!-- raf:tested-by raf_census::test_crate::census_gate::qualified_pattern_heads_use_types_beside_value_and_macro_bindings,raf_census::test_crate::census_gate::qualified_dependency_heads_ignore_value_imports_and_keep_type_imports,raf_census::test_crate::census_gate::same_head_imports_keep_distinct_type_and_value_bindings,raf_census::test_crate::census_gate::workspace_call_patterns_follow_cross_crate_reexports_through_actual_source_sites,raf_census::test_crate::census_gate::call_patterns_follow_written_imports_and_respect_parameters_locals_and_closures,raf_census::test_crate::census_gate::type_patterns_follow_aliases_without_capturing_generic_or_module_shadows,raf_census::test_crate::census_gate::block_import_patterns_use_the_import_binding_instead_of_a_shadowed_module_function,raf_census::test_crate::census_gate::constant_pattern_identifiers_keep_their_actual_reference_in_match_arms -->

Import namespaces are checked after their target's block aliases have expanded
in their declaration frames; younger scopes cannot change an imported target.
Grouped `self` aliases supply Type bindings. Ordinary named imports retain the
namespaces their targets supply.
Direct occurrences still observe their own lexical shadows.
An unrelated module with the same written prefix cannot suppress a real match.
Namespace dependencies use the same bounded expansion engine without recursive
Rust calls, and cyclic dependencies refuse beside acyclic chains.
<!-- raf:tested-by raf_census::test_crate::census_gate::transitive_block_imports_classify_their_expanded_namespace,raf_census::test_crate::census_gate::namespace_dependencies_refuse_cycles_beside_bounded_transitive_chains,raf_census::test_crate::census_gate::import_targets_keep_their_declaration_frame -->

A type path pattern that selects an indexed first-party struct, enum or union
also matches written module-level aliases of that definition. Census resolves
the occurrence in its own lexical scope, then follows a zero-parameter named
alias chain in each declaration's context. The terminal nominal type admits
the literal const argument forms proved by Source. Ambiguous, cyclic and
conditional routes in that profile refuse at an in-scope occurrence.
Generic substitution, block-local type aliases and unindexed targets
remain outside this identity proof. A shared file contributes one physical
hit, with scope judged against the matching reaches. Within an indexed macro
invocation, the invocation's scope must admit the same target and reading
module as the owner's resolution.
<!-- raf:tested-by raf_census::test_crate::census_gate::indexed_type_patterns_follow_written_aliases_at_their_uses,raf_census::test_crate::census_gate::written_type_alias_hits_share_physical_sites_and_respect_shipping_scope,raf_census::test_crate::census_gate::macro_type_hits_use_the_scope_of_the_reach_that_resolves_the_type -->

**Written receiver types.** A selector such as `demo::Client::send` names one
physical first-party struct, enum or union and a method identifier. The selector
admits an optional leading `::` and a raw method identifier. A simple named
local or an ordinary function or closure parameter needs a bare written type
path. Census follows admitted imports, including supported grouped block
imports, reexports and unconditional nominal aliases at the binding
declaration. An opaque block wildcard on that route prevents attribution.

The receiver expression must be a plain local identifier. A nearer untyped or
unsupported binding shadows older proof. The matcher skips definitely excluded
local declarations and their initializers. A local initializer and its `else`
block are read before the local binds. A nearer block import in the Value
namespace can shadow an enclosing receiver, including a name from a wildcard.
An opaque wildcard in a nearer block stops lookup across that block. A local
binding takes precedence over a wildcard in its own block. Closures can
capture enclosing bindings.
Nested items and inline constants have independent runtime bindings.
Each source reach supplies its own proof. The method token supplies the report
location and scope, and matching reaches count once per physical occurrence.
<!-- raf:tested-by raf_census::test_crate::census_gate::written_receiver_methods_keep_declaration_identity_and_runtime_shadowing,raf_census::test_crate::census_gate::written_receiver_method_scope_keeps_the_matching_reach_of_a_shared_file,raf_census::test_crate::census_gate::written_receiver_method_selector_refuses_non_nominal_and_malformed_paths -->

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

Select indexed reference definitions for call and path policies. An alias whose
native route reaches a selected reference but has no definition refuses during
admission. Supply its missing source context or correct the symbol; changing
the alias spelling cannot convert missing identity into a syntax observation.
<!-- raf:tested-by raf_census::test_crate::census_gate::workspace_call_patterns_follow_cross_crate_reexports_through_actual_source_sites -->

Select the reference packages and supply their actual locally available roots
when an indexed matcher crosses a dependency boundary. The host gate verifies
cross-root disk paths; an opaque caller-owned tree must contain the path in
its own authority or supply an actual supported context.
<!-- raf:tested-by raf_census::test_crate::census_gate::workspace_call_patterns_follow_cross_crate_reexports_through_actual_source_sites -->

Run `raf_census::example_crate::patterns` for a memory-backed canonical ledger
and paired direct-call and wrapper-call checks.

Attach a pattern to the job whose home implements the approved operation, then
call Gate::workspace(directory).check() or assert_clean(). check_tree uses
the same production evaluation over a caller-owned Tree. matching_limits
sets the shared native matching step/state allowance; source loading retains
its original separate allowance.
<!-- raf:tested-by raf_census::test_crate::census_gate::path_pattern_kinds_execute_with_positive_and_negative_syntax_neighbours,raf_census::test_crate::census_gate::lexical_patterns_share_an_explicit_native_budget_and_release_only_their_own_spans -->

## Examples

A call through a first-party facade and a renamed reference dependency matches
the referenced function. A same-spelled local function remains a distinct
definition. Path matching also includes actual import sites; importing a
forbidden definition is itself an occurrence even if its call is shadowed.
Both private and public aliases to a missing selected reference definition
refuse beside the same aliases to an indexed definition. An unindexed standard
library call through an alias still matches its declared external spelling.
<!-- raf:tested-by raf_census::test_crate::census_gate::workspace_call_patterns_follow_cross_crate_reexports_through_actual_source_sites -->

The runnable `raf_census::example_crate::patterns` constructs this policy with
`raf_rdf::Builder`, serializes its canonical snapshot and checks both a direct
release copy and a clean wrapper call. This Turtle fragment shows the graph's
local names; the native writer supplies its content-addressed BASE:

~~~turtle
@prefix raf: <https://blackcatinformatics.ca/raf/report/1#> .
<> a raf:Ledger ; raf:normalForm 1 ; raf:reportForm 5 ;
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

Namespace lookup requires actual source contexts, not only precomputed
reference body observations. Missing source contexts or selected definitions
refuse with a remedy. Structural navigation never supplies configured provider
proof, and reference files do not become first-party rule subjects.
<!-- raf:tested-by raf_census::test_crate::census_gate::workspace_call_patterns_follow_cross_crate_reexports_through_actual_source_sites -->

Source ownership queries index the file's start-ordered spans once. Subtrees
whose spans end before the query byte can be skipped. The file index and each
examined node spend the matching step grant. The native ordered table reuses
the owners of each byte across patterns and constant observations. It charges
logarithmic lookup and update bounds and exact arena growth layouts before
allocation under the same grant. File span slots, cache associations and
retained reference-vector capacity, including spare cells, spend the state
grant. The arena admission covers the ordered table storage.
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

- Matching observes written syntax before macro expansion. Method matching
establishes the written receiver type and method name. Inferred types, automatic
adjustments and method implementation selection are outside that proof.
Macro expression-list and statement/repetition arguments are observed through
the existing Rust parser; other macro languages remain token syntax. Named
qualified type/trait paths are supported. A relevant anonymous qualified receiver
or mixed constant/binding context refuses rather than fabricating an identity.
Structural conditions such as loop membership belong in a registered Rule.
<!-- raf:tested-by raf_census::test_crate::census_gate::call_patterns_observe_macro_repetition_and_statement_arguments_with_clean_neighbours,raf_census::test_crate::census_gate::call_patterns_include_qualified_trait_and_inherent_paths_without_receiver_guessing,raf_census::test_crate::census_gate::written_receiver_methods_keep_declaration_identity_and_runtime_shadowing -->

- Matching exhaustion and allocation refusal are errors. Temporary lexical sessions
own their spans and release only their own storage on success or refusal;
caller-owned proc-macro spans remain usable. ECMA is the native dialect; no
parallel regex engine or second Rust tokenizer is used.
<!-- raf:tested-by raf_census::test_crate::census_gate::lexical_patterns_share_an_explicit_native_budget_and_release_only_their_own_spans -->

## Explanation

Fingerprint evidence can compare independent observations. Definition identity
needs one source index and the actual declaration routes between its packages.
The original consumer set remains separate from this larger lookup domain.
<!-- raf:tested-by raf_census::test_crate::census_gate::workspace_call_patterns_follow_cross_crate_reexports_through_actual_source_sites -->

The closed matcher set keeps syntax policy beside the job it protects. Workspace
definition lookup stays in Source. Shared import observations serve the built-in
hasher detector and declared patterns; normal-form tokens and literal decoding
are reused. Native matching limits make unsupported work visible.
<!-- raf:tested-by raf_census::test_crate::census_gate::default_hasher_detector_uses_visible_std_paths_and_lexical_aliases,raf_census::test_crate::census_gate::path_pattern_kinds_execute_with_positive_and_negative_syntax_neighbours -->

## Verification

The existing cross-crate pattern case includes selected reference call and
path aliases, provider reexports, local-definition neighbors and an unresolved
selected-target refusal. It also checks that reference bodies remain outside
the first-party census.
<!-- raf:tested-by raf_census::test_crate::census_gate::workspace_call_patterns_follow_cross_crate_reexports_through_actual_source_sites -->

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
| `raf_census::test_crate::census_gate::indexed_type_patterns_follow_written_aliases_at_their_uses` | test | [Contract](#contract) |
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
| `raf_census::test_crate::census_gate::macro_type_hits_use_the_scope_of_the_reach_that_resolves_the_type` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::many_constant_sites_keep_indexed_matching_within_the_default_grant` | test | [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::mixed_constant_binding_reaches_refuse_beside_consistent_constant_reaches` | test | [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::namespace_dependencies_refuse_cycles_beside_bounded_transitive_chains` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::path_pattern_kinds_execute_with_positive_and_negative_syntax_neighbours` | test | [Purpose](#purpose), [Contract](#contract), [Usage](#usage), [Examples](#examples), [Explanation](#explanation) |
| `raf_census::test_crate::census_gate::qualified_dependency_heads_ignore_value_imports_and_keep_type_imports` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::qualified_pattern_heads_use_types_beside_value_and_macro_bindings` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::ratchet::a_shared_file_hit_is_scoped_by_the_target_whose_resolution_matched` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::ratchet::contradictory_or_unknown_scope_refuses_at_load_beside_a_valid_neighbour` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::ratchet::path_scope_is_silent_inside_an_excluded_prefix_and_reports_outside_it` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::ratchet::ratcheted_patterns_record_counted_sites_and_hold_both_directions` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::ratchet::shipping_scope_follows_the_test_role_of_a_module_level_macro_invocation` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::ratchet::shipping_scope_hits_a_library_and_stays_silent_in_tests_test_modules_and_build_scripts` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::ratchet::target_kind_scope_includes_and_excludes_kinds_beside_unscoped_hits` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::same_head_imports_keep_distinct_type_and_value_bindings` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::token_patterns_include_file_level_macro_arguments_without_renaming_item_names` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::token_patterns_normalize_body_local_declarations_without_renaming_module_items` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::transitive_block_imports_classify_their_expanded_namespace` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::type_patterns_follow_aliases_without_capturing_generic_or_module_shadows` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::workspace_call_patterns_follow_cross_crate_reexports_through_actual_source_sites` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints), [Explanation](#explanation) |
| `raf_census::test_crate::census_gate::written_receiver_method_scope_keeps_the_matching_reach_of_a_shared_file` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::written_receiver_method_selector_refuses_non_nominal_and_malformed_paths` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::written_receiver_methods_keep_declaration_identity_and_runtime_shadowing` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::written_type_alias_hits_share_physical_sites_and_respect_shipping_scope` | test | [Contract](#contract) |
<!-- raf:generated:end -->
