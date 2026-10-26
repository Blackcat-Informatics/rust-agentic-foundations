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
    - raf_census::migrate_legacy_with_extensions
    - raf_census::ForeignField
    - raf_census::ForeignLedgerAdapter
    - raf_census::MAX_INDEX_BYTES
    - raf_census::ReportKind
    - raf_census::ReportKind::max_bytes
    - raf_census::upgrade_report_form
    - cargo_census::bin_crate::cargo_census::main
    - cargo_census::bin_crate::cargo_census::cli
    - cargo_census::bin_crate::cargo_census::command
  tests:
    - raf_census::test_crate::reports::native_scalar_readers_preserve_canonical_report_refusals_in_both_formats
    - raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence
    - raf_census::test_crate::census_gate::extensions_require_one_real_validator_without_loosening_core_fields
    - raf_census::test_crate::census_gate::ledger_settings_keep_source_limits_and_version_refusal_ahead_of_job_interpretation
    - raf_census::test_crate::census_gate::provider_declarations_require_complete_owned_evidence_and_never_make_a_clean_gate
    - raf_census::test_crate::debt::both_native_spellings_round_trip_and_input_order_does_not_change_bytes
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
    - cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity
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

### Explicit native layout conversion
<!-- raf:tested-by raf_census::test_crate::reports::native_scalar_readers_preserve_canonical_report_refusals_in_both_formats,raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence,raf_census::test_crate::census_gate::extensions_require_one_real_validator_without_loosening_core_fields,raf_census::test_crate::debt::both_native_spellings_round_trip_and_input_order_does_not_change_bytes,cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy -->

upgrade_report_form converts exactly a canonical native form-3 Baseline, Index,
Ledger or DebtRecord to form 4. The caller supplies ReportKind, MigrationInput
and the explicit native carrier. Its per-kind input bound applies before native
parsing, and the same bound applies to output. The original Graph is edited
through the native Builder: remove the exact old header, insert the current
header and run the selected current structural validator. No policy or debt is
reconstructed from scalar rows, and no source or consumer evaluation runs.

Every normalized assertion other than reportForm survives, including quoted
matches, owned extension closures, declarations, counted multiplicities and
job copies. Canonical snapshot identity changes, so owned subject/reference
IRIs rebase; external IRIs and literal audit pins remain unchanged. Historical
normalForm and fingerprint values remain unchanged. Structural validation admits
historical ledgers without treating them as currently executable policy.
Ordinary readers remain strictly form 4. Other source versions, wrong kinds,
query roots, unknown fields, contradictions and size failures refuse without
partial output. Converted historical ledger/index carriers remain usable by
the existing fingerprint-migration operation.

### Consumer-owned foreign values
<!-- raf:tested-by raf_census::test_crate::migration_legacy::real_shaped_dialects_share_one_policy_and_one_owned_metadata_extension,cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity -->

`migrate_legacy_with_extensions` accepts borrowed `ForeignLedgerAdapter`
registrations beside the same tree, ledger and baseline. Each adapter sees the
job id and an original whole immediate job field through `ForeignField`:
the native document, exact admitting profile, occurrence index, node and path.
It declines with `None`, refuses with `Some(Err(reason))`, or claims with
`Some(Ok(predicate))`. The predicate must be an absolute consumer vocabulary IRI;
core report predicates and rdf:type cannot replace the extension's schema.
Every registered adapter is consulted for the field. Competing claims, claims
on consumed core or descriptive metadata, schema refusals and native transport
failures return an error with the original field location and cause.

The framework owns retention. It projects the original native document lazily
once under its original profile and transports every Value-subtree assertion
without a second scalar encoder. Job-owned proxy subjects preserve native
keys, kinds, values, verbatim scalar spellings, paths, ordinals and byte spans.
Native parent/document references between retained subjects relocate together;
boundary parents remain original external IRIs. Forward rdfs:member edges make
every descendant reachable within its unique owner, and prov:wasDerivedFrom
records each original subject identity. One compact document proxy per job
retains the original source IRI, digest, profile and byte length. Whole-document
authoring cover is not copied into each extension.

The adapter cannot choose which claimed descendants survive. A root leaves
unmapped accounting only after complete transport succeeds; all other roots
remain explicitly unmapped. Existing descriptive metadata keeps its declared
projection in the same extension. `migrate_legacy` uses this one implementation
with no adapters. Import preserves proof-pin literals without renewing them,
verifying their evidence or authorizing an exemption. The production Gate reads
only the resulting native ledger and its registered consumer validator.

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
baseline. Native `purrdf-toml::analyze` parses foreign syntax into its typed flat
arena; the importer tracks consumed child indices without another TOML value
tree. PurRDF's bounded JSON
reader and strict Record decoder read the baseline. Eight-byte historical and
32-byte complete digests map through their member symbols to current body/shim
fingerprints. The importer recognizes the PurRDF, katamari and decodes-shaped
metadata fields and retains them in an owned native Extension. Ordered repeated
array metadata retains order and multiplicity. Dialects are import inputs; the
Gate does not support their grammars as alternative policy formats.

The native profile admits one MiB of source, 1,048,576 nodes and statements,
32 semantic levels and 256 MiB of retained model data. Foreign containers at
depth 32 refuse while scalars at that depth remain valid. Native signed integer,
float, signed zero, NaN and temporal distinctions survive parsing. Only the exact
authored tokens inf, +inf and -inf authorize infinity; finite lexical overflow
refuses even in otherwise ignored data. Native grammar and resource errors retain
their `TomlError` cause. Decoded keys select application fields; source spelling
is not reparsed by a second grammar.

### Unmapped rows and read-only inputs
<!-- raf:tested-by raf_census::test_crate::census_gate::index_migration_retains_unmapped_rows_with_reasons_and_comments,raf_census::test_crate::migration_legacy::unknown_fields_unmapped_fingerprints_and_unprojectable_metadata_remain_visible,cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy -->

Every unresolved member, absent current fingerprint, unknown foreign field and
unprojectable value produces an UnmappedRow with location, original value and
reason. A partially resolved member set is not a complete mapping. Modern unmapped
fingerprints remain asserted. Every unmapped row is also retained as an escaped
review comment in the emitted document. Neither input nor source workspace is
modified. Output with unmapped rows requires review and canonicalization before
adoption; it is not a passing current-policy verdict.
Unclaimed nested nodes retain their own locations and original source fragments,
including empty containers, repeated array occurrences and explicit table
declarations. An implicit table's first key span is not substituted for its
descendant values. Nonfinite and temporal metadata without a declared projection
remain unmapped with their original values; import does not renew source pins.

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

For an existing canonical carrier, run cargo census migrate --report-form ledger
--input old-ledger.ttl --format turtle and capture stdout separately. Select
baseline, index or debt for those stored kinds; --format ntriples converts that
syntax without changing it. Library callers use upgrade_report_form with the
same explicit kind and carrier. Convert both historical ledger and index before
passing them to fingerprint migration; conversion retains their historical
normalForm rather than claiming current policy.

Run cargo census migrate --ledger old-ledger.ttl --index old-index.ttl to inspect
the native result on stdout. For foreign input, use cargo census migrate --legacy
--ledger foreign-ledger.toml --baseline foreign-baseline.json. Capture the output
outside both inputs and inspect unmapped review comments before adopting it.
Library callers supply a workspace-root Tree and named MigrationInput values to
`raf_census::migrate_index` or `raf_census::migrate_legacy` and receive the same
Migration structure. The latter's executable Rust documentation demonstrates
the foreign-input boundary on a MemoryTree.
Read each unmapped occurrence and its table declarations together. Array indices
distinguish repeated equal values; native spans retain the original numeric and
temporal spelling, and decoded key identities retain quoted or dotted names.

For repository-specific policy, implement `ForeignLedgerAdapter::claim` over
the borrowed native field and call `migrate_legacy_with_extensions` with the
adapter references. Return the predicate that names the retained field root;
do not construct RDF values or preprocess the source. Register a separate
`LedgerExtension` on the consumer Gate to check its native child schema and
evidence. Array members are read through rdfs:member and ordered by their native
ordinal, not by IRI or query order. The generic command has no runtime Rust
registration flag and retains the explicit unmapped result for unknown fields.
<!-- raf:tested-by raf_census::test_crate::migration_legacy::real_shaped_dialects_share_one_policy_and_one_owned_metadata_extension,cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity -->

## Examples

The layout conversion cases carry canonical Baseline, Index, Ledger and
DebtRecord data through form 3 to 4 in both syntaxes. Complete assertion equality
retains quoted matches, nested owned evidence, opaque source pins, pattern and
provider declarations, counted physical members and exact job copies. Reference
and provider execution requirements still refuse when the converted policy is
executed without their required inputs. Converted historical ledger/index inputs
drive the existing source-site fingerprint migration. Converted debt remains
unchanged against its original value; real growth refuses and shrinking remains
required by the same ratchet.
<!-- raf:tested-by raf_census::test_crate::census_gate::index_migration_splits_changed_members_and_preserves_nonhash_assertions,raf_census::test_crate::census_gate::ledger_settings_keep_source_limits_and_version_refusal_ahead_of_job_interpretation,raf_census::test_crate::census_gate::provider_declarations_require_complete_owned_evidence_and_never_make_a_clean_gate,raf_census::test_crate::debt::both_native_spellings_round_trip_and_input_order_does_not_change_bytes,cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy -->

The registered capacity-proof fixture imports two rows with ordered repeated
authority values and the unchanged source:original-proof-pin. Inverse relocation
matches every retained assertion to the supplied native projection, including
the original scalar spelling and provenance. An absent adapter leaves the rows
unmapped; competing, core-field and malformed claims refuse. The consumer reads
the native graph and accepts the valid proof. A missing field, changed file or
stale pin gives the same cause through direct Gate and registered CLI execution.
Neither path changes the source, lock, foreign inputs or baseline, and no debt
record is created. These fixture checks do not establish real consumer evidence
validity or complete real-ledger adoption.
<!-- raf:tested-by raf_census::test_crate::migration_legacy::real_shaped_dialects_share_one_policy_and_one_owned_metadata_extension,cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity -->

An older body fingerprint names two members that now have different bodies.
The native index supplies both symbols, and migration emits both new fingerprints.
If one symbol disappeared, migration retains the original assertion and reports
the missing member; the surviving member is not used to claim complete migration.
An unknown foreign metadata value remains visible with its exact original value
and a reason, including values that cannot project to the closed extension model.
The nested metadata fixture retains two equal array values at distinct locations,
an empty table, its unknown table declaration and a timestamp. A nested extension
containing an unprojectable timestamp retains all of its sibling proof values.
<!-- raf:tested-by raf_census::test_crate::migration_legacy::unknown_fields_unmapped_fingerprints_and_unprojectable_metadata_remain_visible -->

## Constraints

- Explicit layout conversion accepts only stored form 3 carriers of the named
  kind and applies that kind's byte ceiling to both input and output. Historical
  normal forms remain data; conversion does not execute policy or renew pins.
  <!-- raf:tested-by raf_census::test_crate::reports::native_scalar_readers_preserve_canonical_report_refusals_in_both_formats,raf_census::test_crate::debt::both_native_spellings_round_trip_and_input_order_does_not_change_bytes,cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy -->

An adapter claims whole immediate job fields. It cannot commandeer consumed
`ext` metadata or select only convenient descendants. Scalar source pins retain
their foreign meaning; compact document provenance is not a substitute for a
consumer's immutable proof identity. Native values and their meaningful order
and multiplicity are retained, while byte-exact TOML authoring reconstruction
is not the destination format. The same one-MiB final ledger cap applies.
<!-- raf:tested-by raf_census::test_crate::migration_legacy::real_shaped_dialects_share_one_policy_and_one_owned_metadata_extension,cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity -->

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

The foreign boundary is deliberately one-way. The supplied native TOML parser
handles old authoring syntax while native JSON Records keep baseline decoding
strict. The destination remains the same native RDF authority as new policy,
with repository metadata isolated in a validated extension. Explicit unmapped
rows preserve information the core cannot interpret, including unknown fields,
nonfinite or date-like metadata and missing summaries. A nonzero command result
keeps that review work visible. Read-only source and input handling makes it
possible to inspect migrations without changing the repositories being examined.

## Verification

The existing scalar-report, policy, debt and executable migration cases cover
all four stored carrier kinds in both syntaxes. Their native assertion comparison
preserves historical fingerprint values, extension closures, opaque source pins
and counted physical multiplicities. Current-version and wrong-kind neighbours
refuse beside successful form 3 conversion.
<!-- raf:tested-by raf_census::test_crate::reports::native_scalar_readers_preserve_canonical_report_refusals_in_both_formats,raf_census::test_crate::census_gate::extensions_require_one_real_validator_without_loosening_core_fields,raf_census::test_crate::debt::both_native_spellings_round_trip_and_input_order_does_not_change_bytes,cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy -->

The bound cases cover all three foreign dialects, modern hash splitting and
cross-key collisions, cfg sites, partial member loss, retained original values,
wrong report/version neighbours, exact foreign depth bounds and output limits.
Actual executable cases prove both modes, stdout/exit behaviour, read-only inputs
and input-size refusal without publishing a record.
The existing migration case also compares complete inverse-relocated native
facts and whole-field claim refusals. The shared registered consumer proves
valid, missing, changed and stale evidence outcomes through the real reader,
validator and CLI, including input and zero-debt preservation.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::migrate_legacy` | doctest | [Usage](#usage) |
| `raf_census::test_crate::census_gate::extensions_require_one_real_validator_without_loosening_core_fields` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::index_migration_keeps_a_new_hash_that_equals_another_old_row` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::index_migration_maps_every_cfg_shim_site_of_a_shared_symbol` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::index_migration_refuses_current_versions_wrong_reports_and_mismatched_versions` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::index_migration_refuses_oversized_split_output_beside_a_same_size_mapping` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::index_migration_retains_unmapped_rows_with_reasons_and_comments` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::index_migration_splits_changed_members_and_preserves_nonhash_assertions` | test | [Contract](#contract), [Examples](#examples) |
| `raf_census::test_crate::census_gate::ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::ledger_settings_keep_source_limits_and_version_refusal_ahead_of_job_interpretation` | test | [Examples](#examples) |
| `raf_census::test_crate::census_gate::provider_declarations_require_complete_owned_evidence_and_never_make_a_clean_gate` | test | [Examples](#examples) |
| `raf_census::test_crate::debt::both_native_spellings_round_trip_and_input_order_does_not_change_bytes` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::test_crate::migration_legacy::ambiguous_aliases_bad_layouts_and_wrong_report_inputs_refuse_without_partial_success` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::migration_legacy::both_foreign_inputs_enforce_size_and_exact_container_depth_bounds` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::migration_legacy::real_shaped_dialects_share_one_policy_and_one_owned_metadata_extension` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::test_crate::migration_legacy::unknown_fields_unmapped_fingerprints_and_unprojectable_metadata_remain_visible` | test | [Contract](#contract), [Examples](#examples) |
| `raf_census::test_crate::migration_legacy::variants_and_counted_distinct_rows_keep_their_declared_contract` | test | [Contract](#contract) |
| `raf_census::test_crate::reports::native_scalar_readers_preserve_canonical_report_refusals_in_both_formats` | test | [Contract](#contract), [Constraints](#constraints) |
<!-- raf:generated:end -->
<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [census-ledger](./census-ledger.md) |
| Depends on | [census-normal-form](./census-normal-form.md) |
| Depends on | [rdf-report-profile](./rdf-report-profile.md) |
| Depended on by | [census-cli](./census-cli.md) |
<!-- raf:generated:end -->
