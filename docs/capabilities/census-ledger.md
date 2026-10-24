---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-ledger
title: "Census ledger"
description: "Closed RDF policy declarations, exact sanctions, source classification and staleness."
state: delivered
bindings:
  code:
    - raf_census::report::ledger
    - raf_census::gate::evaluate
    - raf_census::gate::owners
    - raf_census::Gate
    - raf_census::Gate::workspace
    - raf_census::Gate::root
    - raf_census::Gate::check
    - raf_census::Gate::check_tree
    - raf_census::Gate::check_locked
    - raf_census::Gate::write_debt_locked
    - raf_census::gate::Gate::evaluate_locked
    - raf_census::gate::providers
    - raf_census::Gate::assert_clean
    - raf_census::Gate::rules
    - raf_census::Gate::extensions
    - raf_census::GateReport
    - raf_census::LedgerExtension
    - raf_census::Ledger
    - raf_census::LedgerJob
    - raf_census::LedgerVariant
    - raf_census::LedgerDistinct
    - raf_census::LedgerPattern
    - raf_census::PatternMatcher
    - raf_census::PreferredProvider
    - raf_census::NamePattern
    - raf_census::MAX_LEDGER_BYTES
    - raf_census::LEDGER_FILE
    - raf_census::SYMBOL_ANCHOR_PREFIX
  tests:
    - raf_census::gate::owners::tests::equal_display_paths_keep_distinct_authorities_and_consumer_scope
    - raf_census::gate::owners::tests::every_pattern_kind_uses_the_selected_consumer_file_set
    - raf_census::test_crate::census_gate::ledger_settings_keep_source_limits_and_version_refusal_ahead_of_job_interpretation
    - raf_census::test_crate::census_gate::ledger_core_nodes_refuse_unknown_missing_and_orphan_assertions_with_valid_neighbours
    - raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence
    - raf_census::test_crate::census_gate::provider_declarations_require_complete_owned_evidence_and_never_make_a_clean_gate
    - raf_census::test_crate::census_gate::locked_gate_uses_complete_ledger_roots_and_the_same_debt_path
    - raf_census::test_crate::census_gate::configured_provider_checks_actual_returned_definitions_and_refuses_dummy_or_shadowed_calls
    - raf_census::test_crate::census_gate::configured_provider_audits_library_attributes_and_generic_computation
    - raf_census::test_crate::census_gate::configured_provider_audits_inherited_inline_and_file_module_attributes
    - raf_census::test_crate::census_gate::configured_provider_tracks_native_state_updates_and_refuses_parallel_mutation
    - raf_census::test_crate::census_gate::configured_provider_audits_selected_nominal_headers_before_method_proof
    - raf_census::test_crate::census_gate::configured_provider_audits_declared_and_actual_call_alias_headers
    - raf_census::test_crate::census_gate::configured_provider_revisits_ancestors_of_aliases_discovered_before_their_headers
    - raf_census::test_crate::census_gate::configured_provider_missing_paths_keep_the_original_typed_source_failure
    - raf_census::test_crate::census_gate::configured_provider_detects_unlisted_competitors_in_the_adapter_package
    - raf_census::test_crate::census_gate::configured_provider_homes_exempt_only_proved_consumers_and_apis
    - raf_census::test_crate::census_gate::configured_provider_checks_every_context_of_a_shared_consumer_body
    - raf_census::test_crate::census_gate::configured_provider_api_home_requires_one_identity_across_consumers
    - raf_census::test_crate::census_gate::configured_provider_targets::every_selected_target_proves_native_calls_through_its_ordinary_library
    - raf_census::test_crate::census_gate::configured_provider_targets::development_routes_prove_selected_consumers_but_never_their_ordinary_reach
    - raf_census::test_crate::census_gate::aliases_resolve_and_a_unit_selected_by_two_forbidden_fields_counts_once
    - raf_census::test_crate::census_gate::enforced_copies_and_unenforced_zero_counts_fail_with_clean_neighbours
    - raf_census::test_crate::census_gate::stale_members_fingerprints_and_variants_are_findings_beside_a_clean_policy
    - raf_census::test_crate::census_gate::a_variant_must_match_its_declared_file_and_documented_anchor
    - raf_census::test_crate::census_gate::a_home_reached_through_two_modules_uses_one_physical_owner
    - raf_census::test_crate::census_gate::reexported_module_homes_own_their_units_and_actual_home_package
    - raf_census::test_crate::census_gate::rules_and_variants_share_physical_ownership_without_widening_file_sanctions
    - raf_census::test_crate::census_gate::extensions_require_one_real_validator_without_loosening_core_fields
    - raf_census::test_crate::census_gate::distinct_sanctions_an_exact_set_and_a_new_clone_breaks_it
    - raf_census::test_crate::census_gate::cfg_alternatives_retain_debt_multiplicity_and_require_an_explicit_exact_sanction
    - raf_census::test_crate::census_gate::an_unknown_rule_is_hard_and_registered_hits_cannot_be_adopted_as_debt
    - raf_census::test_crate::census_gate::registered_rule_hits_cannot_claim_the_root_as_a_first_unit_owner
  examples:
    - raf_census::example_crate::debt
    - raf_census::example_crate::providers
depends: [cap:census-normal-form, cap:source-index, cap:rdf-report-profile]
---

# Census ledger

## Purpose

The ledger names the home of each functionality, forbidden competing code and
justified distinct implementations. Source resolution and staleness checks keep
those declarations tied to the current workspace rather than remembered counts.

## Contract

### Root-owned locked publication
<!-- raf:tested-by raf_census::example_crate::debt -->

`Gate::write_debt_locked` reads first-party source through its own DiskTree at the
Gate's discovered workspace root. The caller supplies the LockedCatalog, local
LockedManifest associations, declared target configurations and sealed
GitPackageRoots used by the complete locked evaluation. It runs the same
registered Rules, Extensions and preferred-provider proofs as `Gate::check_locked`
before debt-difference findings are added. The shared writing ratchet refuses
non-debt findings and same-form growth, preserves unchanged records and publishes
exact admissible debt. No caller-built GateReport or structural fallback can
replace that evaluation. The [debt contract](./census-debt-ratchet.md) governs
adoption, shrink, version replacement and atomic publication.
The shared configured evaluator is `raf_census::gate::Gate::evaluate_locked`.

### Grounded provider execution
<!-- raf:tested-by raf_census::gate::owners::tests::equal_display_paths_keep_distinct_authorities_and_consumer_scope,raf_census::gate::owners::tests::every_pattern_kind_uses_the_selected_consumer_file_set,raf_census::test_crate::census_gate::locked_gate_uses_complete_ledger_roots_and_the_same_debt_path,raf_census::test_crate::census_gate::configured_provider_checks_actual_returned_definitions_and_refuses_dummy_or_shadowed_calls,raf_census::test_crate::census_gate::configured_provider_tracks_native_state_updates_and_refuses_parallel_mutation,raf_census::test_crate::census_gate::configured_provider_audits_declared_and_actual_call_alias_headers,raf_census::test_crate::census_gate::configured_provider_revisits_ancestors_of_aliases_discovered_before_their_headers,raf_census::test_crate::census_gate::configured_provider_api_home_requires_one_identity_across_consumers,raf_census::test_crate::census_gate::configured_provider_homes_exempt_only_proved_consumers_and_apis,raf_census::test_crate::census_gate::configured_provider_checks_every_context_of_a_shared_consumer_body -->

`Gate::check_locked` evaluates one complete, configured locked Source workspace.
Every declared local association and sealed Git root remains authoritative.
Only first-party packages under the ledger roots supply Census units, rule
subjects and pattern occurrences; external provider source supplies lookup
evidence without becoming consumer debt. Owners retain source authority,
physical file and byte identity.
A preferred-provider selector chooses actual admitted dependency libraries.
Every declared API must resolve to one supported public function definition
in each actual consumer context. Renames and explicit library aliases retain
their original dependency eligibility and ambiguity. Configured declarations,
nominal self types, traversed aliases and module ancestors must be observed;
unmodeled rewriting attributes cannot establish executable evidence.
Consumers use a finite adapter grammar: admitted input bindings or ordinary
literals, shared references, immutable local forwarding, direct resolved native
calls and explicit native mutable-state operations. The returned value must
come from the native calls. Every required mutation event must reach that
result. Every admitted native call has a required receipt, including constructors
and reads. Its receipt must reach the returned argument/state lineage;
discarded results, unused native calls, stale snapshots, dead state and parallel local
computation refuse. Body expression attributes and opaque computation refuse.
Mutable references must be direct call arguments to admitted state bindings.
Calls with tracked mutable arguments are standalone statements; forwarding their possibly borrowed
results or mutable-reference aliases refuses instead of losing state identity.
Every native `same_source` identity of a physical consumer body receives this
proof in its actual owner context.
A home or entry point may use an exact declared public API path whose native
receipt has already been validated. All consumer contexts must agree on its
actual definition identity. Such a home needs no first-party fingerprint unit.
This does not admit arbitrary missing owners or infer authority from spelling.
Selected functionality detectors still scan all first-party consumers. A
preferred-provider row grants no package exemption, and its home and entry-point
exemptions retain only completely proved physical identities. Violations remain
non-debt failures.

### Selected target contexts
<!-- raf:tested-by raf_census::test_crate::census_gate::configured_provider_targets::every_selected_target_proves_native_calls_through_its_ordinary_library,raf_census::test_crate::census_gate::configured_provider_targets::development_routes_prove_selected_consumers_but_never_their_ordinary_reach -->

Configured binaries, integration tests, benches and examples retain their
ordinary package library, custom library names and renamed dependency routes.
A public reexport through that library must still resolve to the actual native
definition. A development dependency is available only in an eligible selected
context. A physical consumer shared with an ordinary library must also prove
that ordinary reach; the development target cannot supply the library's missing
route. Shared physical syntax must have compatible declared configurations.

### Closed native policy
<!-- raf:tested-by raf_census::test_crate::census_gate::ledger_core_nodes_refuse_unknown_missing_and_orphan_assertions_with_valid_neighbours,raf_census::test_crate::census_gate::ledger_settings_keep_source_limits_and_version_refusal_ahead_of_job_interpretation -->

The production file helpers-ledger.ttl is canonical RDF 1.2. Ledger::from_turtle
and Ledger::from_ntriples validate equivalent representations through the shared
native profile. Its Ledger root requires normalForm and reportForm; optional
minTokens and maxFileBytes default to 20 and eight MiB. Repeated root, reference,
job, distinct and pattern properties declare sets; the default root is the
workspace. An incompatible normal form refuses before job interpretation.
Unknown core properties, wrong classes, missing required values, invalid
cardinalities and unowned nodes refuse. Input is at most one MiB.

### Source classification and exact sanctions
<!-- raf:tested-by raf_census::test_crate::census_gate::aliases_resolve_and_a_unit_selected_by_two_forbidden_fields_counts_once,raf_census::test_crate::census_gate::reexported_module_homes_own_their_units_and_actual_home_package,raf_census::test_crate::census_gate::distinct_sanctions_an_exact_set_and_a_new_clone_breaks_it,raf_census::test_crate::census_gate::cfg_alternatives_retain_debt_multiplicity_and_require_an_explicit_exact_sanction,raf_census::test_crate::census_gate::a_variant_must_match_its_declared_file_and_documented_anchor -->

Each LedgerJob requires unique jobId, nonempty one-line summary, homeSymbol and
Boolean enforced. Repeated entryPoint, forbiddenName, forbiddenConstant,
forbiddenFingerprint, forbiddenPattern and variant properties express sets.
Name expressions use PurRDF's bounded ECMA-262 Unicode-mode evaluator. Integer
constants compare by value. The Source index resolves aliases; module homes
include descendant implementation units. Each physical selected item counts
once as HOME, VARIANT or COPY, even if several selectors match it.

A Variant requires symbol, file, detector, criterion, anchor and reason. It
sanctions only matching sites in its declared file. A Distinct requires
memberSymbol, kind (body or shim), criterion, anchor and reason. Its counted
member set must equal a current group exactly. Owned MemberMultiplicity nodes
with symbol and copies greater than one retain cfg alternatives; aliases cannot
increase the count. Documentation anchors are absolute urn:raf:code: IRIs
followed by a percent-encoded Source path that resolves to documented items.

### Staleness and registered code
<!-- raf:tested-by raf_census::test_crate::census_gate::stale_members_fingerprints_and_variants_are_findings_beside_a_clean_policy,raf_census::test_crate::census_gate::enforced_copies_and_unenforced_zero_counts_fail_with_clean_neighbours,raf_census::test_crate::census_gate::extensions_require_one_real_validator_without_loosening_core_fields,raf_census::test_crate::census_gate::an_unknown_rule_is_hard_and_registered_hits_cannot_be_adopted_as_debt -->

Missing homes, entries, members or variants, unmatched body/shim fingerprints,
obsolete sanctions and unenforced jobs with zero copies are findings. Enforced
copies fail the gate. A job may own one Extension. Exactly one registered
LedgerExtension must claim and validate nonempty content; zero or multiple
claims refuse. Missing custom rules require repository registration. Rule
findings cannot become recorded debt.

### Declared patterns and preferred providers
<!-- raf:tested-by raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence,raf_census::test_crate::census_gate::provider_declarations_require_complete_owned_evidence_and_never_make_a_clean_gate -->

Each Pattern has a unique RuleId-compatible patternId, one-line summary and exactly one of call,
path, macro, typePath, literalRegex or tokens. Literal expressions compile through
the native bounded matcher. Unknown references, unused declarations and
collisions with built-in ids refuse. One owned PreferredProvider per job requires
package, nonempty entryPoint APIs and nonempty consumer paths. The package is a
selector; it is not evidence of a locked identity or actual delegation.

## Usage

Supply the complete `LockedCatalog`, local `LockedManifest` associations and
sealed `GitPackageRoots` to `Gate::check_locked`. Each participating package
needs its complete declared target configuration. Declare the approved package
selector, public API paths and actual consumer paths on the job's
`PreferredProvider`; a selector such as `purrdf.*` selects the admitted project
family. Use an exact declared API path as the home when the canonical provider
implementation belongs to external source. Keep functionality detectors on the
job so unlisted competing helpers remain subject to the same policy.
Pass mutable state directly to standalone native update calls; forward only
the final nonmutating native result through immutable locals.
Keep every native call on the returned value's argument/state path; unused
constructor or observation calls do not establish returned native computation.

Call Gate::workspace(directory).rules(rules).extensions(extensions).check() for
a report, or assert_clean() from a repository test. check_tree applies the same
policy to a caller-owned workspace-root Tree. Ledger exposes validated typed
declarations and the native graph. Import foreign TOML through
[migration](./census-migration.md); the production Gate reads only native RDF.
For explicit debt publication with preferred providers, install the same rules
and extensions on the Gate and call `Gate::write_debt_locked` with the catalog,
local associations and sealed sources from a repository binary or xtask.
Its disk root supplies the actual
first-party files and destination; the supplied associations retain their typed
source authorities. The executable `raf_census::example_crate::debt` includes
ordinary and locked registered Gates on owned disk workspaces.
<!-- raf:tested-by raf_census::example_crate::debt -->

The executable `raf_census::example_crate::providers` evaluates an approved
encoder and its adapter through `check_locked`, then confirms that a competing
table in the adapter package remains a finding.

Those structural entry points refuse preferred-provider rows that need grounded
configuration. Evaluate those rows through `check_locked` with the supplied
complete source and target context, or explicitly publish their admissible debt
through `write_debt_locked`.

## Examples

A job can select package `provider`, API `provider::api::encode`, consumer
`demo::run` and that public API as its home. With the dependency renamed to
`renamed`, `pub fn run(value:u32)->u32 { renamed::api::encode(value) }` supplies
the actual admitted result. Returning `value` after discarding the native call
refuses. Passing an ordinary literal to the native API is valid input; returning
a literal after a native call still fails the returned-result proof.
For stateful hashing, declare the native `Hasher::new`, `Hasher::update` and
`Hasher::finalize` APIs. Create native state, update that exact mutable binding
through explicit associated calls, and return its native finalization result.
Updating another state or finalizing a snapshot made before the update refuses.
Use `Hasher::update(&mut state, input);` as a statement. Binding its borrowed
result or first assigning `&mut state` to a local is outside the proof grammar.
An extra unused `Hasher::new()` or finalization call also refuses because its
receipt does not reach the returned value.

A job may name demo_util::encode as its home, demo_util::public_encode as an
entry point and a body fingerprint as forbidden. Reexports resolve to physical
source owners, so those two public names cannot manufacture a second copy.
The registered-rule and extension fixtures demonstrate actual executable
registration and the same operation's refused neighbour.

The existing debt example keeps a valid configured provider consumer while a
separate unenforced constant job changes from two copies to three or one. The
locked writer adopts the initial record, refuses growth and records shrink.
An enforced registered rule and nonempty owner extension remain active: a
planted local_copy or wrong owner refuses publication, as does an unlisted
competitor selected by the provider job's detector. These are controlled disk
fixtures; their local provider source is not an external native-output witness.
<!-- raf:tested-by raf_census::example_crate::debt -->

## Constraints

The adapter proof supports explicit resolved function paths and identifier
bindings. Receiver inference, async results, arbitrary local validation or
control flow, general type substitution and macro expansion have no inferred
executable meaning. Unsupported forms refuse with the actual source location.
Native receipts follow explicit argument edges and direct mutable-state edges.
Hidden reference identities in inputs, aliases or holders are not inferred;
an update through them cannot refresh a separate result's lineage. Such a flow
refuses unless its actual native result carries every required receipt.
Item configuration comes from the supplied declared context; this operation
does not acquire an observed Cargo invocation or verify its declaration against
a compiler run. Rust type checking remains the caller's build obligation.
The same acquisition boundary applies to locked writing. Owning the DiskTree
and destination root does not authenticate the supplied catalog/configuration
against a compiler run or acquire dependency sources. Registered CLI debt
writing needs its own explicit authorization and operation agreement; the
read-only check response grants no write.
Ordinary module and item home rules remain available, but preferred-provider
exemptions require completely proved identities. A public API home is accepted
only through its already validated receipt; missing arbitrary source owners do
not become acceptable homes. Different definitions under the same public API
spelling across consumer contexts are ambiguous and refuse.
<!-- raf:tested-by raf_census::test_crate::census_gate::configured_provider_checks_actual_returned_definitions_and_refuses_dummy_or_shadowed_calls,raf_census::test_crate::census_gate::configured_provider_audits_library_attributes_and_generic_computation,raf_census::test_crate::census_gate::configured_provider_api_home_requires_one_identity_across_consumers,raf_census::test_crate::census_gate::configured_provider_missing_paths_keep_the_original_typed_source_failure -->

- Reference and preferred-provider declarations do not establish executable
  support. The Gate refuses unavailable execution paths rather than returning
  a partial clean verdict or recording debt. Declared patterns execute through
  their [syntax matcher contract](./census-patterns.md). Homes and
  sanctions require Source definitions; unexpanded macro output supplies no
  definition. TOML dialects are foreign import inputs, not production readers.
<!-- raf:tested-by raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence,raf_census::test_crate::census_gate::provider_declarations_require_complete_owned_evidence_and_never_make_a_clean_gate -->

The explicit locked writer belongs in a repository binary or xtask. The existing
debt example exercises it only on owned scratch roots and snapshots first-party
source, manifests, lock and ledger around each write. A refusal leaves prior
debt intact; a preexisting publication temporary is never claimed as owned.
<!-- raf:tested-by raf_census::example_crate::debt -->

## Explanation

One native RDF authority keeps policy parsing, storage and canonicalization in
the same shared implementation as other repository records. It avoids a second
handwritten configuration grammar and makes unknown predicates visible rather
than discarding them during deserialization. The typed view carries declared
intent without claiming that a declaration alone is executable evidence.

Physical source ownership matters because one file may be reached under several
module paths, and several cfg definitions may share one symbol. Alias spelling
cannot create extra debt members, while real alternative implementations must
retain their multiplicity. Exact sanctions bind actual members and their
documentation, so a new clone invalidates an old justification automatically.
Staleness in both directions prevents dead policy accumulating unused exceptions.
The Gate reports every selected physical item once and refuses missing executors
before a result can be mistaken for a complete evaluation.

## Verification

The executable debt example includes configured writing on owned disk roots.
Its assertions pair structural provider refusal
with locked adoption, compare exact unchanged and reduced debt, and retain
registered-rule, owner-extension and unlisted-competitor refusals. Malformed
prior records, version replacement and preexisting-temporary controls preserve
the same source and publication boundaries.
<!-- raf:tested-by raf_census::example_crate::debt -->

Configured Gate fixtures pair actual renamed native delegation with discarded,
replaced, shadowed and parallel results. Native state fixtures cover required
mutation lineage, dead state and stale snapshots. Declaration fixtures cover
active and inactive attributes on functions, aliases, nominal types and module
ancestors, including shared files observed in different module contexts.
Unlisted-function and struct fixtures verify detector coverage in the adapter
package. Home fixtures verify exact public API receipts, unrelated and module
homes, and same-identity versus distinct-root consumers. Missing native paths
retain their typed Source cause.

Actual configured Gate cases cover each selected target, ordinary own-library
reexports and development-route eligibility. Compatible shared Bench and Example
consumers refuse when their ordinary library reach has no provider route; the
same source passes with an ordinary dependency. A package with no eligible
selected target refuses during Source loading before provider execution.

The bound integration cases exercise real MemoryTree workspaces, reexports,
physical aliases, sanctions, staleness, extension ownership, closed declarations
and one-MiB boundary neighbours. Actual CLI cases exercise debt publication.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_census::example_crate::debt` | example target | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::example_crate::providers` | example target | [Usage](#usage) |
| `raf_census::gate::owners::tests::equal_display_paths_keep_distinct_authorities_and_consumer_scope` | test | [Contract](#contract) |
| `raf_census::gate::owners::tests::every_pattern_kind_uses_the_selected_consumer_file_set` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::a_home_reached_through_two_modules_uses_one_physical_owner` | test | [Verification](#verification) |
| `raf_census::test_crate::census_gate::a_variant_must_match_its_declared_file_and_documented_anchor` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::aliases_resolve_and_a_unit_selected_by_two_forbidden_fields_counts_once` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::an_unknown_rule_is_hard_and_registered_hits_cannot_be_adopted_as_debt` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::cfg_alternatives_retain_debt_multiplicity_and_require_an_explicit_exact_sanction` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::configured_provider_api_home_requires_one_identity_across_consumers` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::configured_provider_audits_declared_and_actual_call_alias_headers` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::configured_provider_audits_inherited_inline_and_file_module_attributes` | test | [Verification](#verification) |
| `raf_census::test_crate::census_gate::configured_provider_audits_library_attributes_and_generic_computation` | test | [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::configured_provider_audits_selected_nominal_headers_before_method_proof` | test | [Verification](#verification) |
| `raf_census::test_crate::census_gate::configured_provider_checks_actual_returned_definitions_and_refuses_dummy_or_shadowed_calls` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::configured_provider_checks_every_context_of_a_shared_consumer_body` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::configured_provider_detects_unlisted_competitors_in_the_adapter_package` | test | [Verification](#verification) |
| `raf_census::test_crate::census_gate::configured_provider_homes_exempt_only_proved_consumers_and_apis` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::configured_provider_missing_paths_keep_the_original_typed_source_failure` | test | [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::configured_provider_revisits_ancestors_of_aliases_discovered_before_their_headers` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::configured_provider_targets::development_routes_prove_selected_consumers_but_never_their_ordinary_reach` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::configured_provider_targets::every_selected_target_proves_native_calls_through_its_ordinary_library` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::configured_provider_tracks_native_state_updates_and_refuses_parallel_mutation` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::distinct_sanctions_an_exact_set_and_a_new_clone_breaks_it` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::enforced_copies_and_unenforced_zero_counts_fail_with_clean_neighbours` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::extensions_require_one_real_validator_without_loosening_core_fields` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::ledger_core_nodes_refuse_unknown_missing_and_orphan_assertions_with_valid_neighbours` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::ledger_settings_keep_source_limits_and_version_refusal_ahead_of_job_interpretation` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::locked_gate_uses_complete_ledger_roots_and_the_same_debt_path` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::provider_declarations_require_complete_owned_evidence_and_never_make_a_clean_gate` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::reexported_module_homes_own_their_units_and_actual_home_package` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::registered_rule_hits_cannot_claim_the_root_as_a_first_unit_owner` | test | [Verification](#verification) |
| `raf_census::test_crate::census_gate::rules_and_variants_share_physical_ownership_without_widening_file_sanctions` | test | [Verification](#verification) |
| `raf_census::test_crate::census_gate::stale_members_fingerprints_and_variants_are_findings_beside_a_clean_policy` | test | [Contract](#contract) |
<!-- raf:generated:end -->
<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [census-normal-form](./census-normal-form.md) |
| Depends on | [rdf-report-profile](./rdf-report-profile.md) |
| Depends on | [source-index](./source-index.md) |
| Depended on by | [census-cli](./census-cli.md) |
| Depended on by | [census-debt-ratchet](./census-debt-ratchet.md) |
| Depended on by | [census-migration](./census-migration.md) |
| Depended on by | [census-patterns](./census-patterns.md) |
| Depended on by | [census-reference-scan](./census-reference-scan.md) |
| Depended on by | [census-rule-api](./census-rule-api.md) |
<!-- raf:generated:end -->
