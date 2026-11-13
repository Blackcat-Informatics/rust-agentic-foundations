---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-cli
title: Census command line
description: Native static Census reports, ledger inspection, migration and explicit debt publication.
state: implementing
bindings:
  code:
    - cargo_census::bin_crate::cargo_census::main
    - cargo_census::bin_crate::cargo_census::cli
    - cargo_census::bin_crate::cargo_census::command
    - cargo_census::bin_crate::cargo_census::error
    - raf_census::registered
    - raf_census::RegisteredCheck
    - raf_census::RegisteredDebtWrite
    - raf_census::RegisteredOperation
    - raf_census::DEBT_WRITE_PROTOCOL_VERSION
    - raf_census::CHECK_PROTOCOL_VERSION
    - raf_census::MAX_CHECK_RESPONSE_BYTES
    - raf_census::MAX_INDEX_BYTES
  tests:
    - cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets
    - cargo_census::test_crate::delivery::report_exports_refuse_source_existing_files_and_git_administration
    - cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate
    - cargo_census::test_crate::delivery::debt_publication_never_creates_a_declared_missing_rust_target
    - cargo_census::test_crate::delivery::embedded_debt_publication_preserves_unowned_entries_and_busy_records
    - cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record
    - raf_census::test_crate::reports::native_scalar_readers_preserve_canonical_report_refusals_in_both_formats
    - raf_census::registered::tests::native_responses_preserve_order_multiplicity_and_evaluation_errors
    - raf_census::registered::tests::protocol_version_fields_and_outcomes_are_closed_with_valid_neighbors
    - raf_census::registered::tests::membership_ordinals_and_orphans_cannot_drop_findings
    - raf_census::registered::tests::canonical_and_size_refusals_do_not_accept_partial_carriers
    - cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy
  examples:
    - raf_census::example_crate::registered
    - rust_agentic_foundations::example_crate::repository_census
depends: [cap:census-ledger, cap:census-debt-ratchet, cap:census-migration, cap:census-rule-api]
---

## Purpose

Expose native duplication and ledger data without building or executing repository programs. Library Gate evaluation, native reports and explicit debt publication preserve useful structural checks while leaving consumer test and example bodies inert.

## Contract

**Static contract.** `check` evaluates the native library Gate for the selected workspace. `index`, `job`, `count` and `fingerprint` derive complete native reports from that Gate. The CLI accepts no package/example executable selector and never invokes Cargo, tests, examples or custom repository binaries. Custom Rust registrations remain explicit library API operations performed by their caller; repository contents cannot instantiate them automatically.
<!-- raf:tested-by cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets -->

`baseline` discovers structure through Census with default options without adopting debt. `dump-ledger` reads the selected native ledger. Reports use canonical RDF with explicit format selection and bounded complete index serialization. `--out` exclusively creates a new .ttl or .nt data file after serialization; existing files, source extensions and Git administration destinations refuse. Stdout mode writes no repository data.
<!-- raf:tested-by cargo_census::test_crate::delivery::report_exports_refuse_source_existing_files_and_git_administration -->

`debt --write` uses the native debt ratchet, permitting adoption and shrink while refusing growth. Migration reads explicit input carriers and writes the converted result to stdout. Malformed inputs and unsupported declarations fail actionably.
<!-- raf:tested-by cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate,cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record -->

## Usage

The documented data path is illustrated by `raf_census::example_crate::registered` and `rust_agentic_foundations::example_crate::repository_census`. These examples are source references for a caller to inspect; RAF does not initiate their execution.

Select a workspace with `--workspace DIRECTORY`. Use `cargo census check`, `cargo census index`, `cargo census baseline`, `cargo census job ID`, `cargo census count ID` or `cargo census fingerprint SYMBOL`. Use `--max-index-bytes` for an explicit bounded index allowance. Help describes native migration and explicit debt options.

`cargo census embedded report|check` uses the separate source-region analysis.
`cargo census embedded debt create|shrink` explicitly publishes its independent
allowance record. See [embedded structural clones](census-embedded-clones.md)
for its syntax profile, resource-admission limits and shrink-only contract.

## Examples

A repository may contain a build script, example and test whose bodies write a marker and panic. `index` reads their source and returns the same native index as the direct library call. None of those bodies executes and no target directory is created. Exporting to src/lib.rs refuses beside successful creation of report.ttl.

## Constraints

- Missing native registrations or unavailable implemented capabilities refuse rather than invoking a repository program. Imported protocol data is not an observed execution verdict. A report write failure may leave the newly created owned data file incomplete; it never truncates an existing destination.
<!-- raf:tested-by cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets -->

Debt publication requires the native Linux retained-directory capability and
filesystem support for its locking and durability operations. No lock file is
created before loading inspected source. Unsupported publication fails without
a pathname-based writer or another locking protocol.
<!-- raf:tested-by cargo_census::test_crate::delivery::debt_publication_never_creates_a_declared_missing_rust_target,cargo_census::test_crate::delivery::embedded_debt_publication_preserves_unowned_entries_and_busy_records -->

## Explanation

The native Gate supplies the policy verdict and complete configured job reports. Safe library reuse replaces the executable target transport. Reading Git-backed source uses native read-only object access without fetching or hooks.

## Verification

Controlled fixtures protect native CLI parity, the absence of consumer execution, report destination restrictions, migration carrier bounds and monotone debt writes.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::debt_publication_never_creates_a_declared_missing_rust_target` | test | [Constraints](#constraints) |
| `cargo_census::test_crate::delivery::embedded_debt_publication_preserves_unowned_entries_and_busy_records` | test | [Constraints](#constraints) |
| `cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy` | test | [Verification](#verification) |
| `cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::delivery::report_exports_refuse_source_existing_files_and_git_administration` | test | [Contract](#contract) |
| `raf_census::example_crate::registered` | example target | [Usage](#usage) |
| `raf_census::registered::tests::canonical_and_size_refusals_do_not_accept_partial_carriers` | test | [Verification](#verification) |
| `raf_census::registered::tests::membership_ordinals_and_orphans_cannot_drop_findings` | test | [Verification](#verification) |
| `raf_census::registered::tests::native_responses_preserve_order_multiplicity_and_evaluation_errors` | test | [Verification](#verification) |
| `raf_census::registered::tests::protocol_version_fields_and_outcomes_are_closed_with_valid_neighbors` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::native_scalar_readers_preserve_canonical_report_refusals_in_both_formats` | test | [Verification](#verification) |
| `rust_agentic_foundations::example_crate::repository_census` | example target | [Usage](#usage) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [census-debt-ratchet](./census-debt-ratchet.md) |
| Depends on | [census-ledger](./census-ledger.md) |
| Depends on | [census-migration](./census-migration.md) |
| Depends on | [census-rule-api](./census-rule-api.md) |
| Changed by | [embedded-clones](../../work/changes/embedded-clones.md) |
<!-- raf:generated:end -->
