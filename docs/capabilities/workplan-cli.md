---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-cli
title: "cargo workplan command line"
description: "The cargo workplan subcommands, options and exit codes."
state: implementing
bindings:
  code:
    - cargo_workplan::bin_crate::cargo_workplan::main
    - cargo_workplan::bin_crate::cargo_workplan::cli::parse
    - cargo_workplan::bin_crate::cargo_workplan::cli::Args
    - cargo_workplan::bin_crate::cargo_workplan::command::run
    - cargo_workplan::bin_crate::cargo_workplan::exit::Exit
    - raf_workplan::root::discover
  tests:
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::each_command_parses_in_both_shapes
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_takes_a_slug_repeated_capabilities_and_affects_and_the_spdx_pair
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_needs_a_capability_and_both_or_neither_spdx_option
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_default_and_parse_in_both_shapes
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_sync_text_options_parse
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_format_that_is_not_text_or_json_is_a_usage_error
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_missing_command_an_unknown_command_and_a_missing_slug_are_usage_errors
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_that_starts_with_a_dash_is_a_usage_error_however_it_is_given
    - cargo_workplan::bin_crate::cargo_workplan::exit::tests::the_three_exits_have_the_documented_codes
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::quiet_prints_only_findings_and_keeps_the_exit
    - cargo_workplan::test_crate::check::a_clean_repository_exits_zero_and_prints_nothing
    - cargo_workplan::test_crate::check::a_repository_with_findings_exits_one_and_prints_each_finding
    - cargo_workplan::test_crate::check::the_findings_are_those_of_the_library_gate_in_the_same_order
    - cargo_workplan::test_crate::check::the_exit_agrees_with_assert_clean_and_every_finding_is_in_its_message
    - cargo_workplan::test_crate::check::usage_errors_exit_two_with_a_message_on_standard_error
    - cargo_workplan::test_crate::status::the_roadmap_groups_one_capability_in_each_state_with_the_gaps_and_the_change
    - cargo_workplan::test_crate::preview::the_proposed_text_is_printed_and_the_markers_are_not
    - cargo_workplan::test_crate::preview::an_unknown_slug_is_exit_two_and_names_the_open_changes
    - cargo_workplan::test_crate::sync::a_missing_index_with_the_text_is_created_with_that_header
    - cargo_workplan::test_crate::sync::only_the_generated_blocks_are_rewritten
    - cargo_workplan::test_crate::new::new_prints_the_library_text_writes_the_record_and_nothing_else_in_both_shapes
    - cargo_workplan::test_crate::new::the_printed_template_in_a_capability_of_each_state_gives_a_clean_check_and_one_unresolved_proposal
    - cargo_workplan::test_crate::impact::impact_prints_what_the_change_reaches_and_writes_nothing
    - cargo_workplan::test_crate::reconcile::nothing_is_written_and_no_cargo_is_started_for_a_fixture_without_bound_examples
    - cargo_workplan::test_crate::complete::from_new_to_a_completed_change_through_the_binary
    - cargo_workplan::test_crate::complete::both_shapes_complete_the_same_way
    - cargo_workplan::test_crate::verify::json_is_one_binding_object_for_each_binding
    - cargo_workplan::test_crate::json::check_prints_one_object_for_each_finding_of_the_gate_with_the_fields_of_its_location
    - cargo_workplan::test_crate::json::the_kinds_that_are_not_rules_are_not_rule_names
    - cargo_workplan::test_crate::json::every_object_is_a_single_line_inside_one_array
    - cargo_workplan::test_crate::json::a_run_that_could_not_run_prints_nothing_on_standard_output_in_json_too
    - cargo_workplan::test_crate::process::discovery_runs_exactly_cargo_metadata_from_the_cargo_variable
    - cargo_workplan::test_crate::process::discovery_runs_the_cargo_on_path_when_the_variable_is_unset
    - cargo_workplan::test_crate::process::discovery_does_not_run_the_build_rustc_program_of_the_workspace
    - cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_by_every_command_before_any_process_starts
    - cargo_workplan::test_crate::process::a_missing_git_is_exit_two_and_names_it
    - cargo_workplan::test_crate::process::cargo_dispatches_workplan_to_the_binary_on_path_and_passes_the_exit_code_back
    - cargo_workplan::test_crate::root::the_root_is_found_from_the_root_and_from_any_directory_below_it
    - cargo_workplan::test_crate::root::every_command_finds_the_root_from_a_subdirectory
  examples:
    - raf_workplan::Workplan::sync
    - raf_workplan::Workplan::preview
    - raf_workplan::Workplan::new_change
    - raf_workplan::Workplan::impact
depends: [cap:workplan-complete-gate, cap:workplan-reconcile, cap:workplan-generated-docs]
---

# cargo workplan command line

## Purpose

`cargo workplan` is the interactive surface of the workplan. The library checks
everything in a test; the binary is how a person or an agent opens a change, reads
what the documentation owes, and completes the work.

## Contract

The binary is `cargo-workplan`, a thin wrapper over `raf-workplan`, parsed with clap
derive. It runs from any directory inside the workspace and finds the root from
`cargo metadata`.

**Root discovery.** When `--root` is absent, the binary spawns exactly this command,
directly with no shell, as an argv vector. The program is the one named by `$CARGO`
when that is set (cargo sets it for a subcommand), and `cargo` from `PATH` otherwise:

```text
cargo metadata --format-version 1 --no-deps --frozen
```

`--no-deps` skips dependency resolution, which is not needed to find the root, and
`--frozen` forbids network access and any change to `Cargo.lock`. No value from a
document is placed on the command line. Even so, cargo reads the workspace's
`.cargo/config.toml` and the rustup proxy reads its `rust-toolchain.toml`, so the
workspace is trusted build input, as it is for `cargo build` (see
[the security policy](../../SECURITY.md)). The binary reads no JSON beyond the
`workspace_root` field and bounds the output at 64 MiB.

| Command | Effect |
| --- | --- |
| `check` | Run every rule of the [document model](./workplan-document-model.md); print each violation |
| `status` | Print the roadmap: planned, implementing with gaps, delivered |
| `docs sync` | Rewrite the generated blocks ([generated documents](./workplan-generated-docs.md)) |
| `docs preview <slug>` | Print the capabilities of the change as they read once reconciled |
| `change new <slug> --capability cap:<name>` | Create a change record |
| `change impact <slug>` | Print what the change touches |
| `change reconcile <slug>` | Print the obligation list ([reconcile](./workplan-reconcile.md)) |
| `change complete <slug>` | Run the [completion gate](./workplan-complete-gate.md) and delete the record |
| `verify [<slug>]` | Run the bound tests and examples |

`status` groups capabilities by state, lists for each `implementing` capability the
bindings that are missing or do not resolve, as gaps, and names the open changes. `docs preview` applies the
PROPOSED blocks of the named change to a copy of the text, removes the markers,
and prints the result; it never writes a file.

> [!PROPOSED change:workplan-cli]
> Cargo runs the binary for `cargo workplan <command>`, passing the argument vector
> `cargo-workplan workplan <command>`, and it runs directly as `cargo-workplan
> <command>`; both forms take the same arguments. `docs sync` takes `--copyright
> <text>` and `--licence <id>`, read only when `docs/capabilities/index.md` must be
> created. `--format json` prints one JSON array for the run, one object for each
> record with `kind` first; the kinds are documented in the crate documentation of
> `cargo-workplan`. Under `-q` a command whose output holds no finding prints nothing
> on stdout, and the exit code does not change.

**Exit codes.** 0 means success: no violations, an empty obligation list, a completed
change, or every binding passing. 1 means findings: violations, obligations, a refused
completion or a failing binding. 2 means the tool could not run: a usage error, an
unreadable file, a missing `git`, a slug that does not exist.

**Options.** `--root <dir>` sets the workspace root, `--format text|json` selects the
output, and `-q` suppresses everything but findings. Findings are printed to stdout in
a stable order and diagnostics to stderr.

## Usage

```sh
cargo workplan check
cargo workplan status
cargo workplan change new incremental-parsing --capability cap:streaming-parser
cargo workplan docs preview incremental-parsing
cargo workplan change complete incremental-parsing
```

## Examples

```text
$ cargo workplan status
delivered     2  alpha-store, beta-index
implementing  1  streaming-parser (gap: bindings.tests is empty)
designed      3  ...
$ echo $?
0
```

`--format json` prints one object per finding with `kind`, `path`, `section` and
`message`, for scripts and agents.

## Constraints

- The binary holds no logic of its own. Each command calls a function in
  `raf-workplan`, so the dev-dependency `Gate` and the CLI agree.
- `check`, `status`, `docs preview`, `change impact` and `change reconcile` write
  nothing. `docs sync` writes only generated blocks. `change new` writes one file.
  `change complete` deletes one file.

> [!PROPOSED change:workplan-cli]
> `change complete` deletes one file, and before it rewrites only the generated blocks
> that the deletion stales. `verify` needs no Git beyond the listing of tracked paths,
> and `status` exits 0 when it lists gaps.
- Every process the binary starts, directly or through `raf-workplan`, is spawned
  with no shell and an argv vector. Each spec that spawns lists its exact
  invocation, and runs `cargo` with `--frozen`. A value taken from a document
  reaches a process only after resolution and a stated pattern, and then in one of
  three forms: after `--`; inside a single `<rev>:<path>` or `-L<start>,<end>:<path>`
  argument; or as a validated test filter (see
  [the security policy](../../SECURITY.md)).
- A slug starts with a lowercase letter or digit; any other slug is a usage error.
- A command that takes a slug fails with exit 2 when `work/changes/<slug>.md` does
  not exist, except `change new`.

## Explanation

Library and binary are separate so that the check runs where it cannot be skipped, in
`cargo test`, and the lifecycle commands are available where a person is working. The
exit codes distinguish "the repository has findings" from "the tool failed", which a
script and a hook need in order to react correctly.

The command set mirrors the lifecycle. `change new` opens, `impact` and `reconcile`
read, `docs preview` shows the outcome, `verify` runs the evidence available, and
`complete` closes. `status` is the roadmap, derived from the states of the documents
rather than kept by hand.

## Verification

- An argument-parsing test per command and option, including `--capability`
  repetition and `--format`.
- A fixture repository per command asserting stdout, files written and the exit code,
  with exit 0, 1 and 2 each exercised: a clean check, a finding, and a missing slug.
- A root-discovery fixture with a stub named by `$CARGO`, and one on `PATH` with
  `$CARGO` unset, records the argv and asserts
  it equals `cargo metadata --format-version 1 --no-deps --frozen`. A second fixture
  has a `.cargo/config.toml` with `build.rustc` set to a script, and asserts that
  root discovery does not run the script.
- A slug argument that begins with `-`, such as `-rf` or `--since=x`, is a usage
  error with exit 2 before any process is spawned, and is never read as an option by
  a spawned process.
- A read-only check: the tree is byte-identical after `check`, `status`,
  `docs preview`, `change impact` and `change reconcile`.
- A `status` fixture with one capability in each state asserting the grouping and the
  gaps listed.
- A `docs preview` fixture asserting that PROPOSED markers are absent from the output
  and the proposed text is present.
- An equivalence test: the violations from `check` equal those of
  `Gate::workspace(...).assert_clean()` on the same fixtures.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_format_that_is_not_text_or_json_is_a_usage_error` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_missing_command_an_unknown_command_and_a_missing_slug_are_usage_errors` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_that_starts_with_a_dash_is_a_usage_error_however_it_is_given` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_needs_a_capability_and_both_or_neither_spdx_option` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_takes_a_slug_repeated_capabilities_and_affects_and_the_spdx_pair` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::each_command_parses_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_default_and_parse_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_sync_text_options_parse` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::exit::tests::the_three_exits_have_the_documented_codes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::quiet_prints_only_findings_and_keeps_the_exit` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_clean_repository_exits_zero_and_prints_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_repository_with_findings_exits_one_and_prints_each_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_exit_agrees_with_assert_clean_and_every_finding_is_in_its_message` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_findings_are_those_of_the_library_gate_in_the_same_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::usage_errors_exit_two_with_a_message_on_standard_error` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::both_shapes_complete_the_same_way` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::from_new_to_a_completed_change_through_the_binary` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::impact_prints_what_the_change_reaches_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::a_run_that_could_not_run_prints_nothing_on_standard_output_in_json_too` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::check_prints_one_object_for_each_finding_of_the_gate_with_the_fields_of_its_location` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::every_object_is_a_single_line_inside_one_array` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::the_kinds_that_are_not_rules_are_not_rule_names` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::new_prints_the_library_text_writes_the_record_and_nothing_else_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_printed_template_in_a_capability_of_each_state_gives_a_clean_check_and_one_unresolved_proposal` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::an_unknown_slug_is_exit_two_and_names_the_open_changes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::the_proposed_text_is_printed_and_the_markers_are_not` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_missing_git_is_exit_two_and_names_it` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_by_every_command_before_any_process_starts` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::cargo_dispatches_workplan_to_the_binary_on_path_and_passes_the_exit_code_back` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::discovery_does_not_run_the_build_rustc_program_of_the_workspace` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::discovery_runs_exactly_cargo_metadata_from_the_cargo_variable` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::discovery_runs_the_cargo_on_path_when_the_variable_is_unset` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::nothing_is_written_and_no_cargo_is_started_for_a_fixture_without_bound_examples` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::every_command_finds_the_root_from_a_subdirectory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::the_root_is_found_from_the_root_and_from_any_directory_below_it` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::the_roadmap_groups_one_capability_in_each_state_with_the_gaps_and_the_change` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_missing_index_with_the_text_is_created_with_that_header` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::only_the_generated_blocks_are_rewritten` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::json_is_one_binding_object_for_each_binding` | test | [Verification](#verification) |
| `raf_workplan::Workplan::impact` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::new_change` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::preview` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::sync` | doctest | [Verification](#verification) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [workplan-complete-gate](./workplan-complete-gate.md) |
| Depends on | [workplan-generated-docs](./workplan-generated-docs.md) |
| Depends on | [workplan-reconcile](./workplan-reconcile.md) |
| Changed by | [workplan-cli](../../work/changes/workplan-cli.md) |
<!-- raf:generated:end -->
