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
    - raf_census::test_crate::census_gate::ledger_settings_keep_source_limits_and_version_refusal_ahead_of_job_interpretation
    - raf_census::test_crate::census_gate::ledger_core_nodes_refuse_unknown_missing_and_orphan_assertions_with_valid_neighbours
    - raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence
    - raf_census::test_crate::census_gate::provider_declarations_require_complete_owned_evidence_and_never_make_a_clean_gate
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
  examples:
    - raf_census::example_crate::debt
depends: [cap:census-normal-form, cap:source-index, cap:rdf-report-profile]
---

# Census ledger

## Purpose

The ledger names the home of each functionality, forbidden competing code and
justified distinct implementations. Source resolution and staleness checks keep
those declarations tied to the current workspace rather than remembered counts.

## Contract

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

Call Gate::workspace(directory).rules(rules).extensions(extensions).check() for
a report, or assert_clean() from a repository test. check_tree applies the same
policy to a caller-owned workspace-root Tree. Ledger exposes validated typed
declarations and the native graph. Import foreign TOML through
[migration](./census-migration.md); the production Gate reads only native RDF.
The executable `raf_census::example_crate::debt` demonstrates its registered Gate
on an owned disk workspace.

## Examples

A job may name demo_util::encode as its home, demo_util::public_encode as an
entry point and a body fingerprint as forbidden. Reexports resolve to physical
source owners, so those two public names cannot manufacture a second copy.
The registered-rule and extension fixtures demonstrate actual executable
registration and the same operation's refused neighbour.

## Constraints

- Pattern, reference and preferred-provider declarations do not establish
  executable support. The Gate refuses these unavailable execution paths rather
  than returning a partial clean verdict or recording debt. Their execution
  contracts belong to the respective detector/reference capabilities. Homes and
  sanctions require Source definitions; unexpanded macro output supplies no
  definition. TOML dialects are foreign import inputs, not production readers.
<!-- raf:tested-by raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence,raf_census::test_crate::census_gate::provider_declarations_require_complete_owned_evidence_and_never_make_a_clean_gate -->

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

The bound integration cases exercise real MemoryTree workspaces, reexports,
physical aliases, sanctions, staleness, extension ownership, closed declarations
and one-MiB boundary neighbours. Actual CLI cases exercise debt publication.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_census::example_crate::debt` | example target | [Usage](#usage) |
| `raf_census::test_crate::census_gate::a_home_reached_through_two_modules_uses_one_physical_owner` | test | [Verification](#verification) |
| `raf_census::test_crate::census_gate::a_variant_must_match_its_declared_file_and_documented_anchor` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::aliases_resolve_and_a_unit_selected_by_two_forbidden_fields_counts_once` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::an_unknown_rule_is_hard_and_registered_hits_cannot_be_adopted_as_debt` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::cfg_alternatives_retain_debt_multiplicity_and_require_an_explicit_exact_sanction` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::distinct_sanctions_an_exact_set_and_a_new_clone_breaks_it` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::enforced_copies_and_unenforced_zero_counts_fail_with_clean_neighbours` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::extensions_require_one_real_validator_without_loosening_core_fields` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::ledger_core_nodes_refuse_unknown_missing_and_orphan_assertions_with_valid_neighbours` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::ledger_settings_keep_source_limits_and_version_refusal_ahead_of_job_interpretation` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::provider_declarations_require_complete_owned_evidence_and_never_make_a_clean_gate` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::reexported_module_homes_own_their_units_and_actual_home_package` | test | [Contract](#contract) |
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
