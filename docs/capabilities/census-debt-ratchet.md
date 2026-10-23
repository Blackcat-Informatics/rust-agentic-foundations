---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-debt-ratchet
title: "Census debt ratchet"
description: "An exact RDF record of duplication that can only shrink."
state: delivered
bindings:
  code:
    - raf_census::report::debt
    - raf_census::Gate::write_debt
    - raf_census::gate::replace_record
    - raf_census::gate::read_debt
    - raf_census::DebtRecord
    - raf_census::DebtGroup
    - raf_census::DebtChange
    - raf_census::DebtWrite
    - raf_census::DEBT_FILE
    - raf_census::MAX_DEBT_BYTES
  tests:
    - raf_census::test_crate::debt::new_groups_supersets_and_rising_copy_counts_are_growth
    - raf_census::test_crate::debt::a_present_superset_cannot_hide_a_missing_exact_recorded_group
    - raf_census::test_crate::debt::resolved_groups_strict_subsets_and_falling_counts_require_a_lower_record
    - raf_census::test_crate::debt::a_group_of_a_different_kind_is_new_debt
    - raf_census::test_crate::debt::both_native_spellings_round_trip_and_input_order_does_not_change_bytes
    - raf_census::test_crate::debt::commas_percent_signs_and_unicode_do_not_alias_member_sets
    - raf_census::test_crate::debt::other_versions_can_be_read_but_cannot_be_compared
    - raf_census::test_crate::debt::invalid_groups_and_zero_copy_rows_are_refused
    - raf_census::test_crate::debt::recomputing_a_digest_does_not_admit_foreign_classes_or_unknown_properties
    - raf_census::test_crate::debt::counted_cfg_members_round_trip_and_lower_without_changing_symbol_identity
    - raf_census::test_crate::debt::counted_nodes_refuse_redundant_counts_and_unowned_assertions
    - raf_census::test_crate::debt::oversized_records_refuse_before_parsing_beside_a_readable_record
    - cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate
    - cargo_census::test_crate::delivery::registered_code_is_required_explicitly_without_recording_partial_debt
  examples:
    - raf_census::example_crate::debt
depends: [cap:census-normal-form, cap:census-ledger, cap:rdf-report-profile]
---

# Census debt ratchet

## Purpose

The ratchet records present duplication while refusing both new debt and stale
slack. Removing duplication must lower the record, so an old allowance cannot
hide a regression.

## Contract

### Exact counted relations
<!-- raf:tested-by raf_census::test_crate::debt::counted_cfg_members_round_trip_and_lower_without_changing_symbol_identity,raf_census::test_crate::debt::both_native_spellings_round_trip_and_input_order_does_not_change_bytes,raf_census::test_crate::debt::commas_percent_signs_and_unicode_do_not_alias_member_sets,raf_census::test_crate::debt::invalid_groups_and_zero_copy_rows_are_refused -->

The workspace-root census-debt.ttl is a canonical native RDF 1.2 DebtRecord
with normalForm and reportForm, body/shim groups and positive per-job copy counts.
Group identity is its kind and counted canonical symbols. Physical cfg
alternatives sharing a symbol retain multiplicity. Counts above one use owned
MemberMultiplicity nodes; ordinary one-count symbols keep memberSymbol.
Source lines and byte offsets are not debt identity. Equal duplication
relations coalesce, even if separate fingerprint groups project to that relation.
Both canonical RDF formats represent the same record; the stored file is Turtle.

### Growth and shrinking records
<!-- raf:tested-by raf_census::test_crate::debt::new_groups_supersets_and_rising_copy_counts_are_growth,raf_census::test_crate::debt::a_present_superset_cannot_hide_a_missing_exact_recorded_group,raf_census::test_crate::debt::resolved_groups_strict_subsets_and_falling_counts_require_a_lower_record,raf_census::test_crate::debt::a_group_of_a_different_kind_is_new_debt,cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate -->

New groups, counted supersets and rising copy counts are growth. Missing groups,
counted strict subsets and falling copy counts are shrinkage. Both fail a check.
A strict subset counts as shrinkage only when its recorded superset disappeared;
a surviving superset cannot hide a missing exact row. Growth cannot be written.
After shrinkage, debt --write or Gate::write_debt records exactly present debt.
The record never carries unused allowance.

### Adoption and version replacement
<!-- raf:tested-by raf_census::test_crate::debt::other_versions_can_be_read_but_cannot_be_compared,cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate,raf_census::example_crate::debt -->

A missing record means no debt during checking. An explicit write creates it
from present debt. A mismatched normal form refuses comparison; the explicit
write may replace that old-version record after the ledger is current. A
malformed existing record still refuses. Creation and version replacement are
visible adoption operations; deleting the record and adopting again is not a
proof that debt did not grow.

### One publication path
<!-- raf:tested-by cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate,cargo_census::test_crate::delivery::registered_code_is_required_explicitly_without_recording_partial_debt,raf_census::example_crate::debt,raf_census::test_crate::debt::oversized_records_refuse_before_parsing_beside_a_readable_record -->

Gate::write_debt and cargo census debt --write share the production evaluation
and writer. Non-debt findings prevent every write. Registered rules or nonempty
extensions require a repository's registered Gate; the built-in CLI exits 2
naming Gate::write_debt rather than skipping them. Missing executors cannot
become debt. A current-version record can only stay equal or shrink.
The writer serializes completely, checks the shared 16-MiB record cap, creates
an exclusive temporary file and replaces the destination after writing and
synchronizing it. Equal records are left unchanged.

## Usage

Run cargo census debt --write in a workspace with a native ledger and no required
repository registration. For registered rules/extensions, call Gate::write_debt
from a repository binary or xtask that installs those implementations. Check with
Gate::check or Gate::assert_clean. The executable `raf_census::example_crate::debt`
example exercises actual adoption,
growth refusal, lowering and version replacement on an owned scratch workspace.

## Examples

A recorded body group contains demo_a::len, demo_b::len and demo_c::len.
Removing demo_c leaves a strict subset. The next Gate check reports shrinkage.
Running cargo census debt --write records the two remaining members and the
next check passes. Adding a fourth member instead refuses both checking and
writing; an exact documented distinct sanction or removal resolves it.

## Constraints

- The ratchet covers duplicate groups and unenforced job-copy counts. Detector
  findings and unavailable policy execution cannot be hidden in a record.
  Readers and writers share a 16-MiB serialized cap. Unknown shapes, invalid
  counts, unowned nodes and noncanonical/digest-invalid text refuse. Persistent
  repository debt is written by a binary or xtask; fixture tests write only their
  owned scratch workspaces.
<!-- raf:tested-by cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate,cargo_census::test_crate::delivery::registered_code_is_required_explicitly_without_recording_partial_debt,raf_census::test_crate::debt::counted_nodes_refuse_redundant_counts_and_unowned_assertions,raf_census::test_crate::debt::oversized_records_refuse_before_parsing_beside_a_readable_record -->

## Explanation

The [shared debt decision](../architecture/debt-ratchet.md) requires both sides
of the ratchet. Blocking only growth leaves slack after code improves: an old
record of three copies would permit two copies to return when only one remains.
Requiring a lowering operation makes each improvement visible and keeps that
allowance equal to current code.

Counted symbol relations separate debt identity from source-line movement while
retaining distinct cfg alternatives. The exact-set check avoids treating one
remaining duplicate group as an excuse for a different vanished group. Explicit
adoption and version replacement stay reviewable because they create or replace
the tracked record. Complete serialization before publication prevents an
oversized or failed rendering from replacing a readable record. Registered code
uses the same evaluator and writer, so a command that cannot install it refuses
instead of quietly producing a smaller partial record.

## Verification

Bound cases cover both ratchet directions, kind changes, counted cfg members,
canonical formats, shape/digest refusal and version comparison. Actual command
fixtures prove adoption, growth refusal, lowering, registration refusal and
preservation of the previous file. The executable example exercises the same
real disk writer with a registered rule and version replacement.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::delivery::registered_code_is_required_explicitly_without_recording_partial_debt` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::example_crate::debt` | example target | [Contract](#contract), [Usage](#usage) |
| `raf_census::test_crate::debt::a_group_of_a_different_kind_is_new_debt` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::a_present_superset_cannot_hide_a_missing_exact_recorded_group` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::both_native_spellings_round_trip_and_input_order_does_not_change_bytes` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::commas_percent_signs_and_unicode_do_not_alias_member_sets` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::counted_cfg_members_round_trip_and_lower_without_changing_symbol_identity` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::counted_nodes_refuse_redundant_counts_and_unowned_assertions` | test | [Constraints](#constraints) |
| `raf_census::test_crate::debt::invalid_groups_and_zero_copy_rows_are_refused` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::new_groups_supersets_and_rising_copy_counts_are_growth` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::other_versions_can_be_read_but_cannot_be_compared` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::oversized_records_refuse_before_parsing_beside_a_readable_record` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::debt::recomputing_a_digest_does_not_admit_foreign_classes_or_unknown_properties` | test | [Verification](#verification) |
| `raf_census::test_crate::debt::resolved_groups_strict_subsets_and_falling_counts_require_a_lower_record` | test | [Contract](#contract) |
<!-- raf:generated:end -->
<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [census-ledger](./census-ledger.md) |
| Depends on | [census-normal-form](./census-normal-form.md) |
| Depends on | [rdf-report-profile](./rdf-report-profile.md) |
| Depended on by | [census-cli](./census-cli.md) |
| Depended on by | [census-rule-api](./census-rule-api.md) |
<!-- raf:generated:end -->
