---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-migration
title: "Census migration"
description: "Read-only fingerprint migration and foreign dialect import to one native RDF ledger."
state: delivered
bindings:
  code:
    - raf_census::gate::migrate
    - raf_census::Migration
    - raf_census::MigrationInput
    - raf_census::UnmappedRow
    - raf_census::migrate_index
    - raf_census::migrate_legacy
    - raf_census::MAX_INDEX_BYTES
    - cargo_census::bin_crate::cargo_census::main
    - cargo_census::bin_crate::cargo_census::cli
    - cargo_census::bin_crate::cargo_census::command
  tests:
    - raf_census::test_crate::census_gate::index_migration_splits_changed_members_and_preserves_nonhash_assertions
    - raf_census::test_crate::census_gate::index_migration_keeps_a_new_hash_that_equals_another_old_row
    - raf_census::test_crate::census_gate::index_migration_retains_unmapped_rows_with_reasons_and_comments
    - raf_census::test_crate::census_gate::index_migration_refuses_current_versions_wrong_reports_and_mismatched_versions
    - raf_census::test_crate::census_gate::index_migration_maps_every_cfg_shim_site_of_a_shared_symbol
    - raf_census::test_crate::census_gate::index_migration_refuses_oversized_split_output_beside_a_same_size_mapping
    - raf_census::test_crate::migration_legacy::real_shaped_dialects_share_one_policy_and_one_owned_metadata_extension
    - raf_census::test_crate::migration_legacy::variants_and_counted_distinct_rows_keep_their_declared_contract
    - raf_census::test_crate::migration_legacy::unknown_fields_unmapped_fingerprints_and_unprojectable_metadata_remain_visible
    - raf_census::test_crate::migration_legacy::ambiguous_aliases_bad_layouts_and_wrong_report_inputs_refuse_without_partial_success
    - raf_census::test_crate::migration_legacy::both_foreign_inputs_enforce_size_and_exact_container_depth_bounds
    - cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy
    - cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record
  examples:
    - raf_census::migrate_legacy
depends: [cap:census-normal-form, cap:census-ledger, cap:rdf-report-profile]
---

# Census migration

## Purpose

Migration carries older fingerprint assertions to the current source and native
ledger schema. It preserves the original inputs and identifies every row that
needs a human decision rather than silently dropping policy during adoption.

## Contract

### Modern native index migration
<!-- raf:tested-by raf_census::test_crate::census_gate::index_migration_splits_changed_members_and_preserves_nonhash_assertions,raf_census::test_crate::census_gate::index_migration_keeps_a_new_hash_that_equals_another_old_row,raf_census::test_crate::census_gate::index_migration_refuses_current_versions_wrong_reports_and_mismatched_versions,raf_census::test_crate::census_gate::index_migration_maps_every_cfg_shim_site_of_a_shared_symbol -->

migrate_index reads an older canonical RDF ledger and its canonical Turtle index.
Their normal forms must agree and be older than the implemented form; the index
has the supported report form. A baseline is not an index. Current fingerprints
come from actual Source sites, including singleton units and every cfg alternative,
under the ledger's roots, token floor and source-file cap. Only body/shim assertions
are remapped; nonhash declarations survive. All old assertions are removed before
the replacement union is inserted, so one new hash cannot erase another row.

### Foreign dialect import
<!-- raf:tested-by raf_census::test_crate::migration_legacy::real_shaped_dialects_share_one_policy_and_one_owned_metadata_extension,raf_census::test_crate::migration_legacy::variants_and_counted_distinct_rows_keep_their_declared_contract,raf_census::test_crate::migration_legacy::unknown_fields_unmapped_fingerprints_and_unprojectable_metadata_remain_visible,raf_census::test_crate::migration_legacy::ambiguous_aliases_bad_layouts_and_wrong_report_inputs_refuse_without_partial_success -->

migrate_legacy reads a foreign TOML ledger with an explicitly selected legacy JSON
baseline. The complete toml crate parses foreign syntax; PurRDF's bounded JSON
reader and strict Record decoder read the baseline. Eight-byte historical and
32-byte complete digests map through their member symbols to current body/shim
fingerprints. The importer recognizes the PurRDF, katamari and decodes-shaped
metadata fields and retains them in an owned native Extension. Ordered repeated
array metadata retains order and multiplicity. Dialects are import inputs; the
Gate does not support their grammars as alternative policy formats.

### Unmapped rows and read-only inputs
<!-- raf:tested-by raf_census::test_crate::census_gate::index_migration_retains_unmapped_rows_with_reasons_and_comments,raf_census::test_crate::migration_legacy::unknown_fields_unmapped_fingerprints_and_unprojectable_metadata_remain_visible,cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy -->

Every unresolved member, absent current fingerprint, unknown foreign field and
unprojectable value produces an UnmappedRow with location, original value and
reason. A partially resolved member set is not a complete mapping. Modern unmapped
fingerprints remain asserted. Every unmapped row is also retained as an escaped
review comment in the emitted document. Neither input nor source workspace is
modified. Output with unmapped rows requires review and canonicalization before
adoption; it is not a passing current-policy verdict.

### Bounded command delivery
<!-- raf:tested-by raf_census::test_crate::census_gate::index_migration_refuses_oversized_split_output_beside_a_same_size_mapping,raf_census::test_crate::migration_legacy::both_foreign_inputs_enforce_size_and_exact_container_depth_bounds,cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy,cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record -->

The actual cargo census migrate command takes --ledger and --index for modern
migration, or --legacy, --ledger and --baseline for foreign import. --workspace
selects a directory for native root discovery. Input ledger and foreign baseline
caps are one MiB; foreign containers allow at most 32 levels. The modern index
cap equals Source's default total-byte budget, MAX_INDEX_BYTES. Output, including
review comments, is at most the ledger's one-MiB cap. Inputs are read through
verified DiskTree handles. The command writes only stdout: exit 0 means all rows
mapped, exit 1 means unmapped rows or an execution/input failure, and exit 2 is
invalid usage. Input or migration failures emit no migrated document; stdout
I/O failures exit 1.

## Usage

Run cargo census migrate --ledger old-ledger.ttl --index old-index.ttl to inspect
the native result on stdout. For foreign input, use cargo census migrate --legacy
--ledger foreign-ledger.toml --baseline foreign-baseline.json. Capture the output
outside both inputs and inspect unmapped review comments before adopting it.
Library callers supply a workspace-root Tree and named MigrationInput values to
`raf_census::migrate_index` or `raf_census::migrate_legacy` and receive the same
Migration structure. The latter's executable Rust documentation demonstrates
the foreign-input boundary on a MemoryTree.

## Examples

An older body fingerprint names two members that now have different bodies.
The native index supplies both symbols, and migration emits both new fingerprints.
If one symbol disappeared, migration retains the original assertion and reports
the missing member; the surviving member is not used to claim complete migration.
An unknown foreign metadata value remains visible with its exact original value
and a reason, including values that cannot project to the closed extension model.

## Constraints

- Modern migration needs the matching older index, not a baseline or an invented
  fingerprint-to-symbol guess. Missing source or malformed input fails rather than
  producing a partial success. Unmapped output is a review document and exits 1.
  The production Gate reads only canonical native RDF; repository extension content
  still requires an actual registered validator before checking or recording debt.
<!-- raf:tested-by raf_census::test_crate::census_gate::index_migration_refuses_current_versions_wrong_reports_and_mismatched_versions,raf_census::test_crate::census_gate::index_migration_refuses_oversized_split_output_beside_a_same_size_mapping,raf_census::test_crate::migration_legacy::ambiguous_aliases_bad_layouts_and_wrong_report_inputs_refuse_without_partial_success,raf_census::test_crate::migration_legacy::both_foreign_inputs_enforce_size_and_exact_container_depth_bounds,cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy -->

## Explanation

Historical fingerprints cannot be translated by changing their prefix or width:
their meaning depends on the old normal form. Member symbols provide the bridge
to actual current source. Resolving every physical site is necessary because
several cfg alternatives can share a single displayed symbol. Retaining an old
assertion when some members disappear avoids disguising a partial match as a
complete migration.

The foreign boundary is deliberately one-way. A complete existing TOML parser
handles old authoring syntax while native JSON Records keep baseline decoding
strict. The destination remains the same native RDF authority as new policy,
with repository metadata isolated in a validated extension. Explicit unmapped
rows preserve information the core cannot interpret, including unknown fields,
nonfinite or date-like metadata and missing summaries. A nonzero command result
keeps that review work visible. Read-only source and input handling makes it
possible to inspect migrations without changing the repositories being examined.

## Verification

The bound cases cover all three foreign dialects, modern hash splitting and
cross-key collisions, cfg sites, partial member loss, retained original values,
wrong report/version neighbours, exact foreign depth bounds and output limits.
Actual executable cases prove both modes, stdout/exit behaviour, read-only inputs
and input-size refusal without publishing a record.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::migrate_legacy` | doctest | [Usage](#usage) |
| `raf_census::test_crate::census_gate::index_migration_keeps_a_new_hash_that_equals_another_old_row` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::index_migration_maps_every_cfg_shim_site_of_a_shared_symbol` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::index_migration_refuses_current_versions_wrong_reports_and_mismatched_versions` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::index_migration_refuses_oversized_split_output_beside_a_same_size_mapping` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::index_migration_retains_unmapped_rows_with_reasons_and_comments` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::index_migration_splits_changed_members_and_preserves_nonhash_assertions` | test | [Contract](#contract) |
| `raf_census::test_crate::migration_legacy::ambiguous_aliases_bad_layouts_and_wrong_report_inputs_refuse_without_partial_success` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::migration_legacy::both_foreign_inputs_enforce_size_and_exact_container_depth_bounds` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::migration_legacy::real_shaped_dialects_share_one_policy_and_one_owned_metadata_extension` | test | [Contract](#contract) |
| `raf_census::test_crate::migration_legacy::unknown_fields_unmapped_fingerprints_and_unprojectable_metadata_remain_visible` | test | [Contract](#contract) |
| `raf_census::test_crate::migration_legacy::variants_and_counted_distinct_rows_keep_their_declared_contract` | test | [Contract](#contract) |
<!-- raf:generated:end -->
<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [census-ledger](./census-ledger.md) |
| Depends on | [census-normal-form](./census-normal-form.md) |
| Depends on | [rdf-report-profile](./rdf-report-profile.md) |
| Depended on by | [census-cli](./census-cli.md) |
<!-- raf:generated:end -->
