---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-cli
title: Census command line
description: Native static Census reports, ledger inspection, migration and explicit debt publication.
state: delivered
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
    - raf_census::selftest
    - raf_census::SelfTest
    - raf_census::SelfTestCase
    - raf_census::Gate::self_test
    - raf_census::MAX_INDEX_BYTES
  tests:
    - cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets
    - cargo_census::test_crate::delivery::report_exports_refuse_source_existing_files_and_git_administration
    - cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate
    - cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record
    - raf_census::test_crate::reports::native_scalar_readers_preserve_canonical_report_refusals_in_both_formats
    - raf_census::registered::tests::native_responses_preserve_order_multiplicity_and_evaluation_errors
    - raf_census::registered::tests::protocol_version_fields_and_outcomes_are_closed_with_valid_neighbors
    - raf_census::registered::tests::membership_ordinals_and_orphans_cannot_drop_findings
    - raf_census::registered::tests::canonical_and_size_refusals_do_not_accept_partial_carriers
    - cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy
    - cargo_census::test_crate::delivery::self_test_needs_no_workspace_writes_nothing_and_prints_stable_case_lines
    - cargo_census::test_crate::delivery::self_test_ignores_the_workspace_flag_even_for_a_path_that_does_not_exist
    - cargo_census::bin_crate::cargo_census::command::tests::self_test_exits_one_with_the_failing_case_and_zero_when_every_case_holds
    - cargo_census::test_crate::delivery::reference_options_select_a_pinned_commit_or_a_labelled_unpinned_directory
    - cargo_census::test_crate::delivery::ledger_reference_ban_fails_check_and_refuses_unusable_inputs_without_a_partial_verdict
    - cargo_census::test_crate::delivery::registry_reference_ban_fails_check_and_every_command_reads_the_same_sealed_set
    - cargo_census::test_crate::delivery::registry_options_refuse_by_name_without_a_partial_verdict
    - raf_census::selftest::tests::the_documented_detector_list_and_case_count_equal_the_registry
    - raf_census::selftest::tests::every_case_holds_on_this_build_and_two_runs_print_the_same_text
    - raf_census::selftest::tests::a_detector_that_stops_firing_or_starts_over_firing_fails_its_side
    - raf_census::selftest::tests::a_normal_form_that_merges_or_splits_wrongly_fails_its_case
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

**Declared reference tree.** `check`, `index`, `job`, `count`, `fingerprint` and `debt` accept one reference tree outside the lock graph. `--reference-repo LOCATION` with `--reference-commit COMMIT`, or with `--reference-pin-file FILE --reference-pin-key KEY`, compares with that repository's committed objects at the commit; the pin file is a workspace-relative TOML file and the key a dotted path to a string. `--reference-unpinned DIRECTORY` compares with a directory's current files, every index and query marks its receipts `unpinned-tree`, and every evaluating command prints one notice on stderr naming the directory while its findings and stdout stay identical to the pinned run; `debt --write` refuses it. A relative location is resolved from the main checkout of the workspace's Git repository. Any other combination of these options refuses with a message naming the accepted forms, and the commands that do not evaluate the Gate refuse every reference option rather than ignoring it. A ledger `reference` with no job homing it bans copies of that dependency: `check` exits 1 with one `REFERENCE-COPY` line per copy, `debt --write` refuses the same finding, and `index`, `job`, `count` and `fingerprint` evaluate the same inputs. A selector that matches nothing, a missing source or a malformed option refuses every one of these commands with no verdict on stdout.
<!-- raf:tested-by cargo_census::test_crate::delivery::reference_options_select_a_pinned_commit_or_a_labelled_unpinned_directory,cargo_census::test_crate::delivery::ledger_reference_ban_fails_check_and_refuses_unusable_inputs_without_a_partial_verdict,cargo_census::test_crate::delivery::registry_reference_ban_fails_check_and_every_command_reads_the_same_sealed_set,cargo_census::test_crate::delivery::registry_options_refuse_by_name_without_a_partial_verdict -->

**Registry dependencies sealed by the lock file.** The same six commands accept the options that run the reference-copy ban against registry dependencies, with no sibling checkout and no pinned commit. `--seal-registry SELECTOR` (repeatable; an exact package name or a `prefix-*` family) seals those registry packages beside the ledger's `reference` selectors, `--registry-cache DIRECTORY` names Cargo's `registry/cache` directory (default `$CARGO_HOME/registry/cache`, then `$HOME/.cargo/registry/cache`), and `--registry-features PACKAGE=FEATURE,FEATURE` states the complete feature set the workspace enables on a sealed package, once per package (`PACKAGE=` for none). Giving any of them selects the locked check of the library (`Gate::check_registry` and `Gate::write_debt_registry`, the path of the library route in [census-reference-scan](census-reference-scan.md)) for `check`, `index`, `job`, `count`, `fingerprint` and `debt`, so all six read one set of sealed sources. The command reads archive bytes from the cache and authenticates each against the checksum in `Cargo.lock`; it fetches nothing, starts no process and runs no Cargo command. It runs from a checkout of this repository or an installed `cargo-census` built from a pinned commit, the consumer needs no sibling tree, `cargo fetch --locked` must already have populated the cache, and the command performs no network access. The caller states the complete feature set of every sealed package; the loader refuses a set it cannot verify. A copied body fails `check` with exit 1 and one `REFERENCE-COPY` line, and an absent, mismatched or oversized archive, a sealed package without a declared feature set, a declaration for a package that is not sealed, an incomplete or unknown feature, a selector or declaration the lock file and manifest disagree about, and a Git reference refuse every one of these commands with exit 1, the refusal on stderr and an empty stdout. The commands that do not evaluate the Gate refuse these options too.
<!-- raf:tested-by cargo_census::test_crate::delivery::registry_reference_ban_fails_check_and_every_command_reads_the_same_sealed_set,cargo_census::test_crate::delivery::registry_options_refuse_by_name_without_a_partial_verdict -->

**Self-test.** `self-test` and the library call `Gate::self_test` run a fixed set of seeded in-memory workspaces through the same evaluation as `check`, and report one `pass ID` or `fail ID: reason` line per case, then a `N cases, M failed` line. The command needs no workspace and `--workspace` is ignored. It reads and writes no file, creates nothing and prints no path or clock reading, so two runs of one build print identical text. The exit status is 0 only when every case holds. The cases check the evaluation engine, each built-in detector, the sanction and ratchet logic and the two grouping behaviours of the normal form, all over in-memory sources; a census that finds nothing where a case seeds a copy fails that named case instead of passing. No case reads the on-disk tree backend or compares the running build with any other. Rules registered on the Gate join the run as one `rule/ID/fixtures` case each, from their own positive and negative fixtures; required rules, extensions and reference roots do not apply.
<!-- raf:tested-by cargo_census::test_crate::delivery::self_test_needs_no_workspace_writes_nothing_and_prints_stable_case_lines,cargo_census::test_crate::delivery::self_test_ignores_the_workspace_flag_even_for_a_path_that_does_not_exist,cargo_census::bin_crate::cargo_census::command::tests::self_test_exits_one_with_the_failing_case_and_zero_when_every_case_holds,raf_census::selftest::tests::every_case_holds_on_this_build_and_two_runs_print_the_same_text -->

The cases, in the order they run:

- `normal-form/renamed-respelled-copy-groups`: two bodies of one algorithm that differ in parameter and local names, a dropped value adapter and an integer spelling form one body group.
- `normal-form/same-size-other-control-flow-separates`: two bodies of the same token count whose loop and branch differ stay ungrouped.
- `constant/respelled-forbidden-constant-is-a-copy`: a forbidden integer written in another spelling counts as a copy.
- `detector/ID/positive` and `detector/ID/negative` for each built-in detector in registry order: every fixture of its positive side is flagged and every fixture of its negative side leaves the workspace clean. The detectors are `length-prefix-frame`, `le-slice-int`, `align-up-mask`, `div-ceil-by-hand`, `xor-first-mismatch`, `hex-format-loop`, `hex-pair-radix`, `hex-digit-radix`, `hex-table`, `json-pointer-escape`, `home-literal`, `sole-name`, `vocabulary-literal`, `std-default-hasher`, `serde-json`, `rdf-text-writer` and `rdf-term-model`.
- `sanction/distinct-body/exact-set-holds`, `sanction/distinct-body/stale-member` and `sanction/distinct-body/stale-set`; `sanction/distinct-shim/exact-set-holds`; `sanction/distinct-multiplicity/exact-count-holds` and `sanction/distinct-multiplicity/stale-count`: an exact distinct sanction is clean, and a member that no longer resolves, a set equal to no current group or a changed source-unit count is a finding.
- `sanction/variant-detector/exact-site-holds`, `stale-path`, `stale-file` and `stale-detector`, and `sanction/variant-isomorphic/exact-site-holds` and `stale-detector`: an exact variant is classified as a variant and leaves the gate clean, and a path, file or detector that no longer matches is a finding.
- `debt/exact-record-holds`, `debt/rising-copies-refused`, `debt/falling-copies-require-lowering`, `debt/new-group-refused` and `debt/resolved-group-requires-lowering`: the ratchet accepts the exact record, refuses growth and requires a lowering write after shrinkage.
- `rule/ID/fixtures` for each rule registered on the Gate.
<!-- raf:tested-by raf_census::selftest::tests::every_case_holds_on_this_build_and_two_runs_print_the_same_text,raf_census::selftest::tests::a_detector_that_stops_firing_or_starts_over_firing_fails_its_side,raf_census::selftest::tests::a_normal_form_that_merges_or_splits_wrongly_fails_its_case,raf_census::selftest::tests::the_documented_detector_list_and_case_count_equal_the_registry -->

## Usage

The documented data path is illustrated by `raf_census::example_crate::registered` and `rust_agentic_foundations::example_crate::repository_census`. These examples are source references for a caller to inspect; RAF does not initiate their execution.

Select a workspace with `--workspace DIRECTORY`. Use `cargo census check`, `cargo census index`, `cargo census baseline`, `cargo census job ID`, `cargo census count ID` or `cargo census fingerprint SYMBOL`. Use `--max-index-bytes` for an explicit bounded index allowance. Seal registry dependencies with `cargo census check --seal-registry acme-core --registry-features acme-core=default,std` after `cargo fetch --locked`. Compare with a sibling repository at a reviewed commit with `cargo census check --reference-repo ../upstream --reference-commit COMMIT`, or name the commit in a pin file with `--reference-pin-file pins.toml --reference-pin-key upstream.rev`. Run `cargo census self-test` in a consumer gate before `check` to prove the installed census still detects seeded copies; it needs no workspace. A library caller uses `Gate::self_test` and reads `SelfTest::passed`, `SelfTest::cases` and its `Display` text. Help describes native migration and explicit debt options.

## Examples

A repository may contain a build script, example and test whose bodies write a marker and panic. `index` reads their source and returns the same native index as the direct library call. None of those bodies executes and no target directory is created. Exporting to src/lib.rs refuses beside successful creation of report.ttl. `self-test` run in an empty directory prints `pass` lines for every case and `54 cases, 0 failed`, exits 0 and leaves the directory empty; a library report for a Gate with a registered rule whose positive fixture produces no hit fails and prints a `fail rule/ID/fixtures` line naming the refused fixture; the command itself never includes registered rules, and exits 1 when any of its own cases fail.

## Constraints

- Missing native registrations or unavailable implemented capabilities refuse rather than invoking a repository program. Imported protocol data is not an observed execution verdict. A report write failure may leave the newly created owned data file incomplete; it never truncates an existing destination.
<!-- raf:tested-by cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets -->

## Explanation

The native Gate supplies the policy verdict and complete configured job reports. Safe library reuse replaces the executable target transport. Reading Git-backed source uses native read-only object access without fetching or hooks.

## Verification

Controlled fixtures protect native CLI parity, the self-test case list, the absence of consumer execution, report destination restrictions, migration carrier bounds, monotone debt writes and the declared reference options.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_census::bin_crate::cargo_census::command::tests::self_test_exits_one_with_the_failing_case_and_zero_when_every_case_holds` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::ledger_reference_ban_fails_check_and_refuses_unusable_inputs_without_a_partial_verdict` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy` | test | [Verification](#verification) |
| `cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::delivery::reference_options_select_a_pinned_commit_or_a_labelled_unpinned_directory` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::registry_options_refuse_by_name_without_a_partial_verdict` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::registry_reference_ban_fails_check_and_every_command_reads_the_same_sealed_set` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::report_exports_refuse_source_existing_files_and_git_administration` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::self_test_ignores_the_workspace_flag_even_for_a_path_that_does_not_exist` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::self_test_needs_no_workspace_writes_nothing_and_prints_stable_case_lines` | test | [Contract](#contract) |
| `raf_census::example_crate::registered` | example target | [Usage](#usage) |
| `raf_census::registered::tests::canonical_and_size_refusals_do_not_accept_partial_carriers` | test | [Verification](#verification) |
| `raf_census::registered::tests::membership_ordinals_and_orphans_cannot_drop_findings` | test | [Verification](#verification) |
| `raf_census::registered::tests::native_responses_preserve_order_multiplicity_and_evaluation_errors` | test | [Verification](#verification) |
| `raf_census::registered::tests::protocol_version_fields_and_outcomes_are_closed_with_valid_neighbors` | test | [Verification](#verification) |
| `raf_census::selftest::tests::a_detector_that_stops_firing_or_starts_over_firing_fails_its_side` | test | [Contract](#contract) |
| `raf_census::selftest::tests::a_normal_form_that_merges_or_splits_wrongly_fails_its_case` | test | [Contract](#contract) |
| `raf_census::selftest::tests::every_case_holds_on_this_build_and_two_runs_print_the_same_text` | test | [Contract](#contract) |
| `raf_census::selftest::tests::the_documented_detector_list_and_case_count_equal_the_registry` | test | [Contract](#contract) |
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
<!-- raf:generated:end -->
