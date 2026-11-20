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
    - cargo_census::run
    - cargo_census::cli
    - cargo_census::command
    - cargo_census::error
    - raf_census::Gate::validate_tree_registrations
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
    - raf_census::capnp_history
  tests:
    - cargo_census::test_crate::capnp_capture::capnp_capture_authenticates_the_exact_pinned_inputs
    - cargo_census::test_crate::capnp_capture::capnp_capture_refuses_an_open_compiler_import_set
    - cargo_census::test_crate::capnp_capture::capnp_capture_reads_literal_dependencies_missing_from_the_request
    - cargo_census::test_crate::capnp_capture::capnp_capture_refuses_import_identity_mismatch_and_root_escape
    - cargo_census::test_crate::capnp_capture::capnp_capture_refuses_oversized_inputs
    - cargo_census::test_crate::delivery::capnp_history_checks_committed_compiler_layouts
    - cargo_census::test_crate::delivery::capnp_history_compares_required_pointer_fields_in_committed_descriptors
    - raf_census::capnp_history::tests::invalid_slot_layouts_and_missing_references_refuse
    - raf_census::capnp_history::tests::required_annotation_refuses_invalid_values_and_undeclared_structs
    - raf_census::capnp_history::tests::deep_group_findings_refuse_as_a_whole_above_the_payload_limit
    - cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets
    - cargo_census::test_crate::delivery::report_exports_refuse_source_existing_files_and_git_administration
    - cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate
    - cargo_census::test_crate::delivery::module_paths_in_package_are_required_by_the_flag_beside_a_clean_run_without_it
    - cargo_census::test_crate::delivery::debt_publication_never_creates_a_declared_missing_rust_target
    - cargo_census::test_crate::delivery::embedded_debt_publication_preserves_unowned_entries_and_busy_records
    - cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record
    - raf_census::test_crate::reports::native_scalar_readers_preserve_canonical_report_refusals_in_both_formats
    - raf_census::registered::tests::native_responses_preserve_order_multiplicity_and_evaluation_errors
    - raf_census::registered::tests::protocol_version_fields_and_outcomes_are_closed_with_valid_neighbors
    - raf_census::registered::tests::membership_ordinals_and_orphans_cannot_drop_findings
    - raf_census::registered::tests::canonical_and_size_refusals_do_not_accept_partial_carriers
    - cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy
    - cargo_census::test_crate::delivery::self_test_needs_no_workspace_writes_nothing_and_prints_stable_case_lines
    - cargo_census::test_crate::delivery::self_test_ignores_the_workspace_flag_even_for_a_path_that_does_not_exist
    - cargo_census::command::tests::self_test_exits_one_with_the_failing_case_and_zero_when_every_case_holds
    - cargo_census::test_crate::delivery::registered_queries_share_cli_bytes_and_refuse_before_output
    - cargo_census::test_crate::delivery::reference_options_select_a_pinned_commit_or_a_labelled_unpinned_directory
    - cargo_census::test_crate::delivery::ledger_reference_ban_fails_check_and_refuses_unusable_inputs_without_a_partial_verdict
    - cargo_census::test_crate::delivery::registry_reference_ban_fails_check_and_every_command_reads_the_same_sealed_set
    - cargo_census::test_crate::delivery::registry_options_refuse_by_name_without_a_partial_verdict
    - cargo_census::test_crate::delivery::a_sealed_registry_selector_is_a_reference_without_a_ledger_entry
    - cargo_census::test_crate::delivery::a_git_reference_is_refused_by_the_registry_options_beside_a_clean_registry_neighbour
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

**Cap'n Proto history.** `capnp-history` compares unpacked compiler
descriptors read from explicit base and current Git commits. The caller
selects struct and enum roots by type ID. Census follows their data type
references and groups. It reports an old root or member that disappears
and changes to existing names, ordinals, types, slots, discriminants and
defaults. The layout comparison permits a defaulted field or enum variant
that preserves the old layout. Written declaration order is ignored. A field
rename can violate history policy even when Cap'n Proto allows it on
the wire. List defaults use the native representation for their element
types. Struct-valued defaults are checked as native containers and
compared by their canonical pointer bytes. The caller verifies
acceptance by application readers.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_checks_committed_compiler_layouts -->

A caller can select a compiler-declared Void field annotation for
required pointer presence. Census validates the marker in both
snapshots. It reports a newly marked ordinal in a struct reached from
the selected roots in both snapshots, including a marked append or a
field with an unchanged read default. It includes fields in
unconditional groups. An unmarked append, an unchanged or removed
marker, and a marker on a newly reached struct add no required-field
finding. The strict layout comparison also runs.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_required_pointer_fields_in_committed_descriptors -->

Capture verification authenticates both compiler captures with one
caller-selected Ed25519 key. Each signed statement binds the compiler
identity and the exact bytes of the descriptor and each requested
schema file. Census reads those bytes from the same pinned Git tree,
checks the request's compiler version, and requires each signed
source path to agree with its requested filename and file node name.
Each reported import resolves to a requested file with the reported ID.
Each literal source import also resolves to a requested file, including
imports used only in scalar defaults. A refusal produces no findings.
With capture verification omitted, the caller selects the compiled
inputs directly.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::capnp_capture_authenticates_the_exact_pinned_inputs,cargo_census::test_crate::capnp_capture::capnp_capture_refuses_an_open_compiler_import_set,cargo_census::test_crate::capnp_capture::capnp_capture_refuses_import_identity_mismatch_and_root_escape,cargo_census::test_crate::capnp_capture::capnp_capture_reads_literal_dependencies_missing_from_the_request -->

**Static contract.** `check` evaluates the native library Gate for the selected workspace. `index`, `job`, `count` and `fingerprint` derive complete native reports from that Gate. The CLI accepts no package/example executable selector and never invokes Cargo, tests, examples or custom repository binaries. Custom Rust registrations remain explicit library API operations performed by their caller; repository contents cannot instantiate them automatically.
<!-- raf:tested-by cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets -->

A repository binary supplies a Gate configuration callback to `cargo_census::run`. The callback installs repository rules and ledger extension validators after the front end validates the arguments. `job`, `count` and `fingerprint` use the configured Gate and the same index queries and renderers as the stock binary.

`baseline` in the registered binary validates the ledger's rule registrations and extensions before rendering with the default Census options. The registered checks cover rule fixtures and required rules. Job rule references resolve against built-in detectors, declarative patterns and registered rules. Source matching, reference selection and debt evaluation belong to the other Gate operations. The stock baseline requires only the workspace source.

Both carriers preserve the stock report bytes when an accepted extension is the only ledger difference. A refused registration exits before the query writes a report to stdout or a file.
<!-- raf:tested-by cargo_census::test_crate::delivery::registered_queries_share_cli_bytes_and_refuse_before_output -->

`baseline` discovers structure through Census with default options without adopting debt. `dump-ledger` reads the selected native ledger. Reports use canonical RDF with explicit format selection and bounded complete index serialization. `--out` exclusively creates a new .ttl or .nt data file after serialization; existing files, source extensions and Git administration destinations refuse. Stdout mode writes no repository data.
<!-- raf:tested-by cargo_census::test_crate::delivery::report_exports_refuse_source_existing_files_and_git_administration -->

`debt --write` uses the native debt ratchet, permitting adoption and shrink while refusing growth. Migration reads explicit input carriers and writes the converted result to stdout. Malformed inputs and unsupported declarations fail actionably.
<!-- raf:tested-by cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate,cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record -->

**Module paths in their package.** `check` and `debt --write` accept `--require-module-paths-in-package`, which selects `Gate::require_module_paths_in_package` for that run: each module `path` attribute that resolves to a file of another package, or of none, is a hard finding that `check` prints and that makes `debt --write` refuse with exit 1 and leave the debt record unchanged. Without the option no module path is judged, so one workspace is clean without it and refused with it. The option belongs to the command line alone: no ledger, report or debt record carries it, so every existing ledger, report and debt record keeps its exact bytes and the report form does not change. A persisted switch would first need the report-form rule applied, and a repository states the requirement where it runs the command. The other commands refuse the option by name.
<!-- raf:tested-by cargo_census::test_crate::delivery::module_paths_in_package_are_required_by_the_flag_beside_a_clean_run_without_it -->

**Declared reference tree.** `check`, `index`, `job`, `count`, `fingerprint` and `debt` accept one reference tree outside the lock graph. `--reference-repo LOCATION` with `--reference-commit COMMIT`, or with `--reference-pin-file FILE --reference-pin-key KEY`, compares with that repository's committed objects at the commit; the pin file is a workspace-relative TOML file and the key a dotted path to a string. `--reference-unpinned DIRECTORY` compares with a directory's current files, every index and query marks its receipts `unpinned-tree`, and every evaluating command prints one notice on stderr naming the directory while its findings and stdout stay identical to the pinned run; `debt --write` refuses it. A relative location is resolved from the main checkout of the workspace's Git repository. Any other combination of these options refuses with a message naming the accepted forms, and the commands that do not evaluate the Gate refuse every reference option rather than ignoring it. A ledger `reference` with no job homing it bans copies of that dependency: `check` exits 1 with one `REFERENCE-COPY` line per copy, `debt --write` refuses the same finding, and `index`, `job`, `count` and `fingerprint` evaluate the same inputs. A selector that matches nothing, a missing source or a malformed option refuses every one of these commands with no verdict on stdout.
<!-- raf:tested-by cargo_census::test_crate::delivery::reference_options_select_a_pinned_commit_or_a_labelled_unpinned_directory,cargo_census::test_crate::delivery::ledger_reference_ban_fails_check_and_refuses_unusable_inputs_without_a_partial_verdict,cargo_census::test_crate::delivery::registry_reference_ban_fails_check_and_every_command_reads_the_same_sealed_set,cargo_census::test_crate::delivery::registry_options_refuse_by_name_without_a_partial_verdict,cargo_census::test_crate::delivery::a_sealed_registry_selector_is_a_reference_without_a_ledger_entry -->

**Registry dependencies sealed by the lock file.** The same six commands accept the options that run the reference-copy ban against registry dependencies, with no sibling checkout and no pinned commit. `--seal-registry SELECTOR` (repeatable; an exact package name or a `prefix-*` family) seals those registry packages and makes each a reference for the ban exactly as a ledger `reference` selector is (a ledger with no `reference` entry still bans a copy of a package named by `--seal-registry`, and without the option nothing is implied), `--registry-cache DIRECTORY` names Cargo's `registry/cache` directory (default `$CARGO_HOME/registry/cache`, then `$HOME/.cargo/registry/cache`), and `--registry-features PACKAGE=FEATURE,FEATURE` states the complete feature set the workspace enables on a sealed package, once per package (`PACKAGE=` for none). Giving any of them selects the locked check of the library (`Gate::check_registry` and `Gate::write_debt_registry`, the path of the library route in [census-reference-scan](census-reference-scan.md)) for `check`, `index`, `job`, `count`, `fingerprint` and `debt`, so all six read one set of sealed sources. The command reads archive bytes from the cache and authenticates each against the checksum in `Cargo.lock`; it fetches nothing, starts no process and runs no Cargo command. It runs from a checkout of this repository or an installed `cargo-census` built from a pinned commit, the consumer needs no sibling tree, `cargo fetch --locked` must already have populated the cache, and the command performs no network access. The caller states the complete feature set of every sealed package; the loader refuses a set it cannot verify. A copied body fails `check` with exit 1 and one `REFERENCE-COPY` line, and an absent, mismatched or oversized archive, a sealed package without a declared feature set, a declaration for a package that is not sealed, an incomplete or unknown feature, a selector or declaration the lock file and manifest disagree about, and a Git reference refuse every one of these commands with exit 1, the refusal on stderr and an empty stdout. The commands that do not evaluate the Gate refuse these options too.
<!-- raf:tested-by cargo_census::test_crate::delivery::registry_reference_ban_fails_check_and_every_command_reads_the_same_sealed_set,cargo_census::test_crate::delivery::registry_options_refuse_by_name_without_a_partial_verdict,cargo_census::test_crate::delivery::a_sealed_registry_selector_is_a_reference_without_a_ledger_entry,cargo_census::test_crate::delivery::a_git_reference_is_refused_by_the_registry_options_beside_a_clean_registry_neighbour -->

**Self-test.** `self-test` and the library call `Gate::self_test` run a fixed set of seeded in-memory workspaces through the same evaluation as `check`, and report one `pass ID` or `fail ID: reason` line per case, then a `N cases, M failed` line. The command needs no workspace and `--workspace` is ignored. It reads and writes no file, creates nothing and prints no path or clock reading, so two runs of one build print identical text. The exit status is 0 only when every case holds. The cases check the evaluation engine, each built-in detector, the sanction and ratchet logic and the two grouping behaviours of the normal form, all over in-memory sources; a census that finds nothing where a case seeds a copy fails that named case instead of passing. No case reads the on-disk tree backend or compares the running build with any other. Rules registered on the Gate join the run as one `rule/ID/fixtures` case each, from their own positive and negative fixtures; required rules, extensions and reference roots do not apply.
<!-- raf:tested-by cargo_census::test_crate::delivery::self_test_needs_no_workspace_writes_nothing_and_prints_stable_case_lines,cargo_census::test_crate::delivery::self_test_ignores_the_workspace_flag_even_for_a_path_that_does_not_exist,cargo_census::command::tests::self_test_exits_one_with_the_failing_case_and_zero_when_every_case_holds,raf_census::selftest::tests::every_case_holds_on_this_build_and_two_runs_print_the_same_text -->

The stock self-test covers enum catch-all decisions through the required-rule
route, including qualified variants, imported variants, bare bindings, and
clean neighboring matches.

The cases, in the order they run:

- `normal-form/renamed-respelled-copy-groups`: two bodies of one algorithm that differ in parameter and local names, a dropped value adapter and an integer spelling form one body group.
- `normal-form/same-size-other-control-flow-separates`: two bodies of the same token count whose loop and branch differ stay ungrouped.
- `constant/respelled-forbidden-constant-is-a-copy`: a forbidden integer written in another spelling counts as a copy.
- `detector/ID/positive` and `detector/ID/negative` for each built-in detector in registry order: every fixture of its positive side is flagged and every fixture of its negative side leaves the workspace clean. The detectors are `length-prefix-frame`, `le-slice-int`, `align-up-mask`, `div-ceil-by-hand`, `xor-first-mismatch`, `hex-format-loop`, `hex-pair-radix`, `hex-digit-radix`, `hex-table`, `json-pointer-escape`, `home-literal`, `sole-name`, `vocabulary-literal`, `std-default-hasher`, `foreign-host-import`, `serde-json`, `rdf-text-writer`, `rdf-term-model`, `reifier-default-graph`, `rdf-list-walk` and `enum-catch-all`.
- `sanction/distinct-body/exact-set-holds`, `sanction/distinct-body/stale-member` and `sanction/distinct-body/stale-set`; `sanction/distinct-shim/exact-set-holds`; `sanction/distinct-multiplicity/exact-count-holds` and `sanction/distinct-multiplicity/stale-count`: an exact distinct sanction is clean, and a member that no longer resolves, a set equal to no current group or a changed source-unit count is a finding.
- `sanction/variant-detector/exact-site-holds`, `stale-path`, `stale-file` and `stale-detector`, and `sanction/variant-isomorphic/exact-site-holds` and `stale-detector`: an exact variant is classified as a variant and leaves the gate clean, and a path, file or detector that no longer matches is a finding.
- `debt/exact-record-holds`, `debt/rising-copies-refused`, `debt/falling-copies-require-lowering`, `debt/new-group-refused` and `debt/resolved-group-requires-lowering`: the ratchet accepts the exact record, refuses growth and requires a lowering write after shrinkage.
- `rule/ID/fixtures` for each rule registered on the Gate.
<!-- raf:tested-by raf_census::selftest::tests::every_case_holds_on_this_build_and_two_runs_print_the_same_text,raf_census::selftest::tests::a_detector_that_stops_firing_or_starts_over_firing_fails_its_side,raf_census::selftest::tests::a_normal_form_that_merges_or_splits_wrongly_fails_its_case,raf_census::selftest::tests::the_documented_detector_list_and_case_count_equal_the_registry -->

## Usage

Run `cargo census capnp-history --base BASE --current CURRENT` with
`--base-descriptor PATH --current-descriptor PATH` and one or more
`--type 0xID` options. Commit IDs use all 40 lowercase hexadecimal
digits. Type IDs use a `0x` prefix followed by 16 lowercase hexadecimal
digits with the high bit set. Descriptor paths are relative to the Git
repository root. Run from that root or name it with `--workspace`.
Capture the compiler request with `capnp compile -o- SCHEMA` and commit
it with the schema. Library consumers call `raf_census::capnp_history::check`.
The command exits 0 for a clean comparison or 1 with findings on stdout.
An input or resource refusal exits 1 with a diagnostic on stderr.

Add `--required-annotation 0xID` to compare required pointer presence.
Library callers supply the selected roots and optional annotation ID
through `HistoryOptions`.

To authenticate captures, supply `--capture-key HEX` and the four
repository-relative paths `--base-capture PATH`,
`--base-capture-signature PATH`, `--current-capture PATH` and
`--current-capture-signature PATH`. Use all 64 lowercase hexadecimal
digits of the Ed25519 public key. Library callers set
`HistoryOptions::capture` with the same key and both pairs of paths.

Produce the request with every schema input explicitly listed. Use
`capnp compile --no-standard-import --import-path=ROOT --src-prefix=ROOT -o-`
followed by those paths. Use the selected schema directory for both
options. Sign the exact canonical N-Triples bytes and store the raw
64-byte Ed25519 signature as a separate committed blob.

Build the statement with `raf_rdf::Builder` and render it as
`Format::NTriples`. Its class and properties use the
`https://blackcatinformatics.ca/raf/report/1#` namespace. Each subject
has one `rdf:type` and the following fields.

| Class | Required properties |
| --- | --- |
| `CapnpCaptureV1` at the graph root | `compilerVersion` uses decimal major, minor and patch joined by dots. `compilerSha256` identifies the compiler artifact. `descriptorFile` is the descriptor path from the Git root and `descriptorSha256` covers its bytes. `schemaRoot` is the path from the Git root, with an empty string for the root itself. One or more `schemaSource` IRIs select owned source rows. |
| `CapnpSchemaSource` at an owned fragment | `file` is the requested filename relative to `schemaRoot`. `sha256` covers the source bytes. |

All scalar properties are strings with exactly one value. Digests
use 64 lowercase hexadecimal digits. The Git-backed capture fixture
builds and signs this record before the command reads it.

The documented data path is illustrated by `raf_census::example_crate::registered` and `rust_agentic_foundations::example_crate::repository_census`. These examples are source references for a caller to inspect; RAF does not initiate their execution.

To host the full front end, depend on `cargo-census` and return its status from
your executable's `main`. Supply an `install_policy` function that accepts
the selected `raf_census::Gate` and returns it with the repository's rules and
validators.

```rust
fn main() -> std::process::ExitCode {
    cargo_census::run(
        std::env::args_os(),
        Some(install_policy),
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    )
}
```

The executable accepts `job ID`, `count ID`, `fingerprint SYMBOL` and
`baseline` directly, with the standard workspace and output options.

Run `cargo census visibility` for the [workspace library visibility census](./census-visibility.md).

Select a workspace with `--workspace DIRECTORY`. Use `cargo census check`, `cargo census index`, `cargo census baseline`, `cargo census job ID`, `cargo census count ID` or `cargo census fingerprint SYMBOL`. Use `--max-index-bytes` for an explicit bounded index allowance. Seal registry dependencies with `cargo census check --seal-registry acme-core --registry-features acme-core=default,std` after `cargo fetch --locked`. Require module paths to stay in their package with `cargo census check --require-module-paths-in-package`, and pass the same option to `debt --write`. Compare with a sibling repository at a reviewed commit with `cargo census check --reference-repo ../upstream --reference-commit COMMIT`, or name the commit in a pin file with `--reference-pin-file pins.toml --reference-pin-key upstream.rev`. Run `cargo census self-test` in a consumer gate before `check` to prove the installed census still detects seeded copies; it needs no workspace. A library caller uses `Gate::self_test` and reads `SelfTest::passed`, `SelfTest::cases` and its `Display` text. Help describes native migration and explicit debt options.

`cargo census embedded report|check` uses the separate source-region analysis.
`cargo census embedded debt create|shrink` explicitly publishes its independent
allowance record. See [embedded structural clones](census-embedded-clones.md)
for its syntax profile, resource-admission limits and shrink-only contract.

## Examples

A repository may contain a build script, example and test whose bodies write a marker and panic. `index` reads their source and returns the same native index as the direct library call. None of those bodies executes and no target directory is created. Exporting to src/lib.rs refuses beside successful creation of report.ttl. The stock `self-test` run in an empty directory prints `pass` lines for every case and `62 cases, 0 failed`, exits 0 and leaves the directory empty. A Gate with a registered rule whose positive fixture produces no hit reports a `fail rule/ID/fixtures` line naming the refused fixture. The registered front end includes the repository's rule cases and returns exit 1 when a case fails.

## Constraints

A capture uses UTF-8 schema source and double-quoted, unescaped import
paths. The profile refuses `embed` expressions. A relative import resolves
from its requesting file's directory. A slash-rooted import resolves from
the signed schema root. Parent segments stay within that root and must
resolve through the actual committed directory entries.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::capnp_capture_authenticates_the_exact_pinned_inputs,cargo_census::test_crate::capnp_capture::capnp_capture_refuses_import_identity_mismatch_and_root_escape,cargo_census::test_crate::capnp_capture::capnp_capture_reads_literal_dependencies_missing_from_the_request -->

A statement is limited to 128 KiB and an individual schema blob to
two MiB, subject to the pinned tree's existing read limits. The source
scan admits up to 8192 literal imports per capture.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::capnp_capture_refuses_oversized_inputs,cargo_census::test_crate::capnp_capture::capnp_capture_reads_literal_dependencies_missing_from_the_request -->

- Missing native registrations or unavailable implemented capabilities refuse rather than invoking a repository program. Imported protocol data is not an observed execution verdict. A report write failure may leave the newly created owned data file incomplete; it never truncates an existing destination.
<!-- raf:tested-by cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets -->

Debt publication requires the native Linux retained-directory capability and
filesystem support for its locking and durability operations. No lock file is
created before loading inspected source. Unsupported publication fails without
a pathname-based writer or another locking protocol.
<!-- raf:tested-by cargo_census::test_crate::delivery::debt_publication_never_creates_a_declared_missing_rust_target,cargo_census::test_crate::delivery::embedded_debt_publication_preserves_unowned_entries_and_busy_records -->

## Explanation

The native Gate supplies the policy verdict and complete configured job reports. Safe library reuse replaces the executable target transport. Reading Git-backed source uses native read-only object access without fetching or hooks.

The capture signer attests that the declared compiler and invocation
produced the descriptor from the complete input set. The compiler in the
capture recipe derives each requested filename and file node name from
the same source name. Census uses one bounded compiler request reader for
capture corroboration and layout projection. Its source scan checks
literal imports that the compiler can omit from its request, while the
request can report implicit imports such as the streaming schema.

## Verification

The Cap'n Proto fixture compares real compiler output in committed Git
objects while the worktree contains different bytes. Selected layouts
refuse a null request root, missing nodes, invalid ordinals, incompatible
list storage and overlapping active slots. Union arms may share storage.
Each descriptor is limited to two
MiB, with a traversal grant of 1,048,576 words, a nesting limit of 64
and member names of at most 256 bytes. A result exceeding two MiB of
counted finding payload refuses the entire comparison.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_checks_committed_compiler_layouts,raf_census::capnp_history::tests::invalid_slot_layouts_and_missing_references_refuse,raf_census::capnp_history::tests::deep_group_findings_refuse_as_a_whole_above_the_payload_limit -->

The required-field check refuses a missing or invalid declaration, a
duplicate application, and an application with an absent or non-Void
value. A marker must apply to an ordinary pointer slot in a declared
struct. A group, a scalar, a synthesized RPC parameter or result, and
a field activated by any enclosing union arm are invalid placements.
The marker and marked structs must have consistent declaration paths
to a file within the nesting bound.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_required_pointer_fields_in_committed_descriptors,raf_census::capnp_history::tests::required_annotation_refuses_invalid_values_and_undeclared_structs -->

Signed compiler fixtures give a layout finding across two valid
captures. They refuse a wrong key, invalid signature, stale
descriptor and stale direct or transitive source. Separate cases
refuse a missing requested row, an unused import without a target,
a mismatched file node name, a known file ID on the wrong path, and
a path above the schema root. The closed fixture includes an in-root
parent path and a slash-rooted import. A scalar-default fixture accepts
both declared files and refuses the capture with the dependency
omitted. It also exercises escaped import paths and an embed expression.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::capnp_capture_authenticates_the_exact_pinned_inputs,cargo_census::test_crate::capnp_capture::capnp_capture_refuses_an_open_compiler_import_set,cargo_census::test_crate::capnp_capture::capnp_capture_refuses_import_identity_mismatch_and_root_escape,cargo_census::test_crate::capnp_capture::capnp_capture_reads_literal_dependencies_missing_from_the_request -->

Controlled fixtures protect native CLI parity, the self-test case list, the absence of consumer execution, report destination restrictions, migration carrier bounds, monotone debt writes and the declared reference options.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_census::command::tests::self_test_exits_one_with_the_failing_case_and_zero_when_every_case_holds` | test | [Contract](#contract) |
| `cargo_census::test_crate::capnp_capture::capnp_capture_authenticates_the_exact_pinned_inputs` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::capnp_capture::capnp_capture_reads_literal_dependencies_missing_from_the_request` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::capnp_capture::capnp_capture_refuses_an_open_compiler_import_set` | test | [Contract](#contract) |
| `cargo_census::test_crate::capnp_capture::capnp_capture_refuses_import_identity_mismatch_and_root_escape` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::capnp_capture::capnp_capture_refuses_oversized_inputs` | test | [Constraints](#constraints) |
| `cargo_census::test_crate::delivery::a_git_reference_is_refused_by_the_registry_options_beside_a_clean_registry_neighbour` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::a_sealed_registry_selector_is_a_reference_without_a_ledger_entry` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_checks_committed_compiler_layouts` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_compares_required_pointer_fields_in_committed_descriptors` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::debt_publication_never_creates_a_declared_missing_rust_target` | test | [Constraints](#constraints) |
| `cargo_census::test_crate::delivery::embedded_debt_publication_preserves_unowned_entries_and_busy_records` | test | [Constraints](#constraints) |
| `cargo_census::test_crate::delivery::ledger_reference_ban_fails_check_and_refuses_unusable_inputs_without_a_partial_verdict` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy` | test | [Verification](#verification) |
| `cargo_census::test_crate::delivery::module_paths_in_package_are_required_by_the_flag_beside_a_clean_run_without_it` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::delivery::reference_options_select_a_pinned_commit_or_a_labelled_unpinned_directory` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::registered_queries_share_cli_bytes_and_refuse_before_output` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::registry_options_refuse_by_name_without_a_partial_verdict` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::registry_reference_ban_fails_check_and_every_command_reads_the_same_sealed_set` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::report_exports_refuse_source_existing_files_and_git_administration` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::self_test_ignores_the_workspace_flag_even_for_a_path_that_does_not_exist` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::self_test_needs_no_workspace_writes_nothing_and_prints_stable_case_lines` | test | [Contract](#contract) |
| `raf_census::capnp_history::tests::deep_group_findings_refuse_as_a_whole_above_the_payload_limit` | test | [Verification](#verification) |
| `raf_census::capnp_history::tests::invalid_slot_layouts_and_missing_references_refuse` | test | [Verification](#verification) |
| `raf_census::capnp_history::tests::required_annotation_refuses_invalid_values_and_undeclared_structs` | test | [Verification](#verification) |
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
