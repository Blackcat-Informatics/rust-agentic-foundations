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
    - raf_workplan::root
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
    - cargo_workplan::test_crate::check::a_directory_that_is_not_a_repository_exits_two_and_says_why_on_standard_error
    - cargo_workplan::test_crate::check::a_finding_is_gone_once_it_is_fixed
    - cargo_workplan::test_crate::check::a_manifest_that_does_not_parse_exits_two
    - cargo_workplan::test_crate::check::a_root_that_does_not_exist_exits_two
    - cargo_workplan::test_crate::check::a_source_file_that_cannot_be_read_exits_two
    - cargo_workplan::test_crate::check::a_stale_generated_block_is_a_finding_and_a_sync_clears_it
    - cargo_workplan::test_crate::check::help_and_version_exit_zero_on_standard_output
    - cargo_workplan::test_crate::check::quiet_keeps_the_findings_and_the_exit_code
    - cargo_workplan::test_crate::check::quiet_on_a_clean_repository_prints_nothing_and_exits_zero
    - cargo_workplan::test_crate::complete::a_capability_left_implementing_is_refused_and_nothing_is_written
    - cargo_workplan::test_crate::complete::a_stale_generated_block_is_refused_until_the_sync_is_run
    - cargo_workplan::test_crate::complete::every_failure_is_listed_and_each_condition_is_named_in_json
    - cargo_workplan::test_crate::complete::json_for_a_completion_is_a_completed_object_and_a_rewrote_object_for_each_file
    - cargo_workplan::test_crate::complete::quiet_hides_a_completion_and_keeps_a_refusal
    - cargo_workplan::test_crate::complete::what_stops_it_looking_is_exit_two_and_the_record_stays
    - cargo_workplan::test_crate::impact::a_change_that_reaches_dependants_and_linking_sections_lists_them_sorted
    - cargo_workplan::test_crate::impact::a_record_that_does_not_load_is_exit_two_with_its_findings
    - cargo_workplan::test_crate::impact::a_slug_with_no_record_is_exit_two_and_names_the_open_changes
    - cargo_workplan::test_crate::impact::impact_finds_the_root_from_below_and_by_root
    - cargo_workplan::test_crate::impact::json_is_one_impact_object_with_its_lists
    - cargo_workplan::test_crate::impact::quiet_prints_nothing_and_the_exit_stays_zero
    - cargo_workplan::test_crate::json::a_clean_run_prints_one_empty_array
    - cargo_workplan::test_crate::json::preview_prints_a_preview_object_for_each_capability_with_the_reconciled_text
    - cargo_workplan::test_crate::json::quiet_keeps_the_findings_in_json_and_prints_an_empty_array_without_any
    - cargo_workplan::test_crate::json::status_prints_a_state_object_for_each_state_and_the_open_changes
    - cargo_workplan::test_crate::json::sync_prints_a_rewrote_object_for_each_file_and_an_empty_array_for_none
    - cargo_workplan::test_crate::json::the_json_and_the_text_forms_print_the_same_findings_in_the_same_order
    - cargo_workplan::test_crate::new::a_capability_given_twice_is_listed_once
    - cargo_workplan::test_crate::new::a_capability_that_is_missing_or_a_bad_option_is_a_usage_error_before_anything_is_read
    - cargo_workplan::test_crate::new::a_second_run_for_the_same_slug_is_exit_two_and_leaves_the_record_as_it_is
    - cargo_workplan::test_crate::new::affects_is_repeatable_and_the_record_holds_the_entries_given_in_order
    - cargo_workplan::test_crate::new::every_refusal_of_the_library_is_exit_two_with_the_tree_untouched
    - cargo_workplan::test_crate::new::json_is_one_new_change_object_with_its_fields
    - cargo_workplan::test_crate::new::new_works_from_a_subdirectory_and_by_root
    - cargo_workplan::test_crate::new::quiet_writes_the_record_and_prints_nothing
    - cargo_workplan::test_crate::new::spdx_text_that_is_not_one_line_is_exit_two_and_writes_nothing
    - cargo_workplan::test_crate::new::the_note_about_the_generated_blocks_is_there_when_the_record_makes_them_stale
    - cargo_workplan::test_crate::new::the_spdx_lines_are_the_two_comment_lines_of_the_records_of_this_repository
    - cargo_workplan::test_crate::new::the_template_left_in_a_state_that_refuses_it_is_the_finding_the_guidance_names
    - cargo_workplan::test_crate::preview::a_capability_the_change_lists_that_has_no_document_is_exit_two
    - cargo_workplan::test_crate::preview::a_change_record_that_did_not_load_is_exit_two_and_says_so
    - cargo_workplan::test_crate::preview::a_slug_that_is_not_a_slug_is_a_usage_error
    - cargo_workplan::test_crate::preview::quiet_prints_nothing_because_a_preview_is_not_a_finding
    - cargo_workplan::test_crate::preview::the_output_is_the_text_the_library_previews_under_a_header_for_each_document
    - cargo_workplan::test_crate::preview::the_preview_is_the_same_by_root_from_another_directory
    - cargo_workplan::test_crate::process::a_cargo_that_cannot_be_started_is_exit_two
    - cargo_workplan::test_crate::process::a_cargo_that_fails_is_exit_two_with_what_it_said
    - cargo_workplan::test_crate::process::a_slug_that_is_not_a_slug_is_refused_before_any_process_starts
    - cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_before_any_process_starts
    - cargo_workplan::test_crate::process::a_usage_error_of_the_change_commands_starts_no_process
    - cargo_workplan::test_crate::process::a_usage_error_starts_no_process
    - cargo_workplan::test_crate::process::output_from_cargo_that_is_not_the_metadata_is_exit_two
    - cargo_workplan::test_crate::process::root_starts_no_cargo_at_all
    - cargo_workplan::test_crate::reconcile::a_bound_example_that_fails_is_an_obligation_and_nothing_is_written_to_the_tree
    - cargo_workplan::test_crate::reconcile::everything_that_stops_it_looking_is_exit_two
    - cargo_workplan::test_crate::reconcile::json_is_one_obligation_object_for_each_obligation
    - cargo_workplan::test_crate::reconcile::quiet_keeps_the_obligations_and_hides_nothing_that_is_a_finding
    - cargo_workplan::test_crate::root::a_directory_outside_any_workspace_is_exit_two_and_names_the_command
    - cargo_workplan::test_crate::root::a_relative_root_is_taken_from_the_current_directory
    - cargo_workplan::test_crate::root::root_overrides_the_discovery_and_the_current_directory_is_not_read
    - cargo_workplan::test_crate::root::sync_from_a_subdirectory_writes_at_the_root_and_nowhere_else
    - cargo_workplan::test_crate::status::a_directory_that_is_not_a_repository_is_exit_two
    - cargo_workplan::test_crate::status::a_state_with_no_capability_is_a_row_with_a_count_of_zero
    - cargo_workplan::test_crate::status::an_implementing_capability_with_every_binding_resolved_has_no_gap_beyond_its_empty_lists
    - cargo_workplan::test_crate::status::quiet_leaves_standard_output_empty_because_the_roadmap_has_no_finding
    - cargo_workplan::test_crate::status::the_roadmap_is_the_same_by_root_from_another_directory
    - cargo_workplan::test_crate::sync::a_document_that_does_not_load_is_exit_two_and_nothing_is_written
    - cargo_workplan::test_crate::sync::a_missing_index_without_the_text_is_exit_two_naming_the_field_and_writes_nothing
    - cargo_workplan::test_crate::sync::a_refused_file_is_exit_two_and_nothing_is_written
    - cargo_workplan::test_crate::sync::a_second_sync_writes_nothing_and_prints_nothing
    - cargo_workplan::test_crate::sync::a_value_that_cannot_be_written_into_the_header_is_refused_when_the_index_is_created
    - cargo_workplan::test_crate::sync::each_field_that_is_not_given_is_named
    - cargo_workplan::test_crate::sync::quiet_hides_the_files_rewritten_and_keeps_the_exit_code
    - cargo_workplan::test_crate::sync::the_index_is_created_by_root_from_another_directory
    - cargo_workplan::test_crate::sync::the_text_is_not_read_when_the_index_exists
    - cargo_workplan::test_crate::verify::a_failing_example_and_a_binding_that_proves_nothing_are_failures
    - cargo_workplan::test_crate::verify::a_hostile_binding_is_a_failure_and_never_reaches_cargo
    - cargo_workplan::test_crate::verify::a_slug_with_no_record_and_a_document_that_does_not_load_are_exit_two
    - cargo_workplan::test_crate::verify::every_binding_passing_exits_zero
    - cargo_workplan::test_crate::verify::no_slug_runs_every_capability_and_a_slug_only_the_ones_the_change_lists
    - cargo_workplan::test_crate::verify::nothing_to_run_is_exit_zero_with_a_note_that_quiet_hides
    - cargo_workplan::test_crate::verify::quiet_prints_only_the_failing_bindings_and_keeps_the_exit
    - raf_workplan::test_crate::root::a_cargo_that_does_not_exist_is_an_error
    - raf_workplan::test_crate::root::a_directory_outside_any_workspace_is_an_error_that_says_what_cargo_said
    - raf_workplan::test_crate::root::configured_programs::discovery_does_not_run_the_programs_of_the_cargo_configuration
    - raf_workplan::test_crate::root::stubs::exactly_one_process_runs_with_the_documented_argv_directory_and_locale
    - raf_workplan::test_crate::root::stubs::hostile_output_is_an_error_and_never_a_guess
    - raf_workplan::test_crate::root::stubs::output_is_bounded_at_64_mebibytes
    - raf_workplan::test_crate::root::stubs::the_top_level_member_wins_over_a_nested_one_with_the_same_name
    - raf_workplan::test_crate::root::the_root_is_found_from_the_root_a_member_and_a_subdirectory
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_command_that_takes_a_slug_refuses_one_that_starts_with_a_dash_in_both_forms
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_must_be_a_slug
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::help_and_version_exit_zero_in_both_shapes
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_may_follow_the_command
    - cargo_workplan::bin_crate::cargo_workplan::exit::tests::a_library_error_becomes_its_display_text
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::a_finding_makes_the_exit_one_and_an_outcome_does_not
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::a_json_object_begins_with_its_kind_and_keeps_the_order_of_its_fields
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::a_record_with_no_text_has_no_line_in_text_and_an_object_in_json
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::an_empty_report_is_nothing_in_text_and_an_empty_array_in_json
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::text_with_quotes_and_control_characters_is_escaped
    - raf_workplan::root::json::tests::a_large_text_is_scanned_to_its_end
    - raf_workplan::root::json::tests::a_member_that_is_only_nested_is_not_the_member
    - raf_workplan::root::json::tests::a_member_twice_is_refused_even_when_the_values_agree
    - raf_workplan::root::json::tests::a_top_level_value_that_is_not_an_object_is_refused
    - raf_workplan::root::json::tests::a_value_that_is_not_a_string_is_refused
    - raf_workplan::root::json::tests::an_unpaired_surrogate_is_text_nobody_can_use
    - raf_workplan::root::json::tests::escapes_are_decoded
    - raf_workplan::root::json::tests::every_prefix_of_a_document_is_refused_and_no_mutation_panics
    - raf_workplan::root::json::tests::nesting_is_bounded_and_never_overflows_the_stack
    - raf_workplan::root::json::tests::numbers_of_the_grammar_pass
    - raf_workplan::root::json::tests::random_strings_of_json_punctuation_never_panic
    - raf_workplan::root::json::tests::text_that_is_not_json_is_refused
    - raf_workplan::root::json::tests::the_member_is_found_wherever_it_sits_at_the_top_level
    - raf_workplan::root::json::tests::the_output_of_cargo_metadata_gives_its_root
    - raf_workplan::root::tests::a_failing_cargo_is_an_error_with_what_it_said
    - raf_workplan::root::tests::a_root_that_is_not_an_absolute_path_is_refused
    - raf_workplan::root::tests::cargo_that_cannot_run_or_writes_too_much_is_an_error
    - raf_workplan::root::tests::output_that_does_not_give_a_root_is_an_error
    - raf_workplan::root::tests::the_root_is_the_workspace_root_member_and_the_command_is_exactly_one
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
`cargo metadata`. Cargo runs it for `cargo workplan <command>`, passing the argument
vector `cargo-workplan workplan <command>`, and it runs directly as
`cargo-workplan <command>`; both forms take the same arguments and give the same
result.

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
`workspace_root` field and bounds the output at 64 MiB. Root discovery runs before
the processes of the command itself.

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
and prints the result; it never writes a file. `docs sync` takes
`--copyright <text>` and `--licence <id>`, the two SPDX lines of a new capability
index; they are read only when `docs/capabilities/index.md` must be created, and then
a missing one is an error that names it. `change new` takes `--capability` once or
more, `--affects cap:<name>#<section>` any number of times, and `--copyright` with
`--licence`, as [reconcile](./workplan-reconcile.md) states, and prints the PROPOSED
block to place and the state each listed capability needs. `change complete` prints
`completed: removed <path>` and a `rewrote <path>` line for each file whose generated
blocks the removal rewrote, or `refused:` with the count of each kind of failure and a
line for each failure that no other line repeats.

**Exit codes.** 0 means success: no violations, an empty obligation list, a completed
change, or every binding passing. 1 means findings: violations, obligations, a refused
completion or a failing binding. 2 means the tool could not run: a usage error, an
unreadable file, a missing `git`, a slug that does not exist. A Git older than 2.30 is
exit 2 for `change reconcile` and `change complete`, a change record that does not
load (an `outcome` that is not one of the three values, for instance) is exit 2, and
`status` exits 0 whatever gaps it lists, since a gap is not a violation.

**Options.** `--root <dir>` sets the workspace root, `--format text|json` selects the
output, and `-q` suppresses everything but findings. Findings are printed to stdout in
a stable order and diagnostics to stderr. Each option is accepted before or after the
command. Under `-q` a command whose output holds no finding (`status`, `docs sync`,
`docs preview`, `change new`, `change impact`, a completion, the passing bindings of
`verify`) prints nothing on stdout and leaves out its notes on stderr, while the
obligations, a refusal and the failing bindings are findings and stay; a failure of
the tool is always written to stderr, and the exit code does not change.

**JSON.** `--format json` prints one JSON array for the run on stdout: one object for
each record the text form prints, in the same order, and `[]` when there is none.
Every object begins with `kind`, the discriminator, followed by the fields of its kind,
always present, with `null` for a field that does not apply. A finding of `check` has
the kind of its rule, from `Finding::KINDS`, and the fields `path`, `section`, `line`,
`key`, `message` and `fix`. The other kinds are `rewrote`, `state`, `open-changes`,
`preview`, `new-change`, `impact`, `obligation`, `refused`, `refusal` (one for each
failure of a refused completion, with its condition), `completed` and `binding`, and
none of them equals the kind of a rule. Their fields are documented in the crate
documentation of `cargo-workplan`. A run that could not run prints nothing on stdout,
so stdout is either one complete array or empty.

## Usage

```sh
cargo workplan check
cargo workplan status
cargo workplan change new incremental-parsing --capability cap:streaming-parser
cargo workplan docs preview incremental-parsing
cargo workplan change complete incremental-parsing
cargo-workplan --format json change reconcile incremental-parsing
```

`make docs-sync` in this repository runs `cargo-workplan docs sync` with the SPDX text
of a new index.

## Examples

```text
$ cargo workplan status
delivered     2  alpha-store, beta-index
implementing  1  streaming-parser (gap: bindings.tests is empty; changes: incremental-parsing)
designed      0
considered    0
open changes  1  incremental-parsing (cap:streaming-parser)
$ echo $?
0
```

`--format json` prints one array for the run, with one object per finding, for
scripts and agents:

```text
$ cargo workplan --format json check
[
{"kind":"link-broken","path":"docs/capabilities/alpha-store.md","section":null,"line":12,"key":null,"message":"broken link `./gone.md`","fix":"point the link at a tracked path, or remove it"}
]
```

## Constraints

- The binary holds no logic of its own. Each command calls a function in
  `raf-workplan`, so the dev-dependency `Gate` and the CLI agree.
- `check`, `status`, `docs preview`, `change impact` and `change reconcile` write
  nothing. `docs sync` writes only generated blocks. `change new` writes one file.
  `change complete` deletes one file, and before it rewrites only the generated
  blocks that the deletion stales.
- `verify` needs no Git beyond the listing of tracked paths and writes nothing in the
  repository; `status` exits 0 when it lists gaps.
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
  and the text of the blocks is present.
- An equivalence test: the violations from `check` equal those of
  `Gate::workspace(...).assert_clean()` on the same fixtures.
- Both argument shapes, `cargo workplan <command>` through a real `cargo` dispatch
  and `cargo-workplan <command>`, give the same output and exit code.
- JSON fixtures: one array per run with a `kind` first in each object, the kinds that
  are not rules never equal to a rule name, and stdout empty for a run that could not
  run.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_command_that_takes_a_slug_refuses_one_that_starts_with_a_dash_in_both_forms` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_format_that_is_not_text_or_json_is_a_usage_error` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_missing_command_an_unknown_command_and_a_missing_slug_are_usage_errors` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_must_be_a_slug` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_that_starts_with_a_dash_is_a_usage_error_however_it_is_given` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_needs_a_capability_and_both_or_neither_spdx_option` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_takes_a_slug_repeated_capabilities_and_affects_and_the_spdx_pair` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::each_command_parses_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::help_and_version_exit_zero_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_default_and_parse_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_may_follow_the_command` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_sync_text_options_parse` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::exit::tests::a_library_error_becomes_its_display_text` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::exit::tests::the_three_exits_have_the_documented_codes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::a_finding_makes_the_exit_one_and_an_outcome_does_not` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::a_json_object_begins_with_its_kind_and_keeps_the_order_of_its_fields` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::a_record_with_no_text_has_no_line_in_text_and_an_object_in_json` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::an_empty_report_is_nothing_in_text_and_an_empty_array_in_json` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::quiet_prints_only_findings_and_keeps_the_exit` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::text_with_quotes_and_control_characters_is_escaped` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_clean_repository_exits_zero_and_prints_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_directory_that_is_not_a_repository_exits_two_and_says_why_on_standard_error` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_finding_is_gone_once_it_is_fixed` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_manifest_that_does_not_parse_exits_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_repository_with_findings_exits_one_and_prints_each_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_root_that_does_not_exist_exits_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_source_file_that_cannot_be_read_exits_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_stale_generated_block_is_a_finding_and_a_sync_clears_it` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::help_and_version_exit_zero_on_standard_output` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::quiet_keeps_the_findings_and_the_exit_code` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::quiet_on_a_clean_repository_prints_nothing_and_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_exit_agrees_with_assert_clean_and_every_finding_is_in_its_message` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_findings_are_those_of_the_library_gate_in_the_same_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::usage_errors_exit_two_with_a_message_on_standard_error` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::a_capability_left_implementing_is_refused_and_nothing_is_written` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::a_stale_generated_block_is_refused_until_the_sync_is_run` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::both_shapes_complete_the_same_way` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::every_failure_is_listed_and_each_condition_is_named_in_json` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::from_new_to_a_completed_change_through_the_binary` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::json_for_a_completion_is_a_completed_object_and_a_rewrote_object_for_each_file` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::quiet_hides_a_completion_and_keeps_a_refusal` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::what_stops_it_looking_is_exit_two_and_the_record_stays` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::a_change_that_reaches_dependants_and_linking_sections_lists_them_sorted` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::a_record_that_does_not_load_is_exit_two_with_its_findings` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::a_slug_with_no_record_is_exit_two_and_names_the_open_changes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::impact_finds_the_root_from_below_and_by_root` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::impact_prints_what_the_change_reaches_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::json_is_one_impact_object_with_its_lists` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::quiet_prints_nothing_and_the_exit_stays_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::a_clean_run_prints_one_empty_array` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::a_run_that_could_not_run_prints_nothing_on_standard_output_in_json_too` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::check_prints_one_object_for_each_finding_of_the_gate_with_the_fields_of_its_location` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::every_object_is_a_single_line_inside_one_array` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::preview_prints_a_preview_object_for_each_capability_with_the_reconciled_text` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::quiet_keeps_the_findings_in_json_and_prints_an_empty_array_without_any` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::status_prints_a_state_object_for_each_state_and_the_open_changes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::sync_prints_a_rewrote_object_for_each_file_and_an_empty_array_for_none` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::the_json_and_the_text_forms_print_the_same_findings_in_the_same_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::the_kinds_that_are_not_rules_are_not_rule_names` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::a_capability_given_twice_is_listed_once` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::a_capability_that_is_missing_or_a_bad_option_is_a_usage_error_before_anything_is_read` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::a_second_run_for_the_same_slug_is_exit_two_and_leaves_the_record_as_it_is` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::affects_is_repeatable_and_the_record_holds_the_entries_given_in_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::every_refusal_of_the_library_is_exit_two_with_the_tree_untouched` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::json_is_one_new_change_object_with_its_fields` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::new_prints_the_library_text_writes_the_record_and_nothing_else_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::new_works_from_a_subdirectory_and_by_root` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::quiet_writes_the_record_and_prints_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::spdx_text_that_is_not_one_line_is_exit_two_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_note_about_the_generated_blocks_is_there_when_the_record_makes_them_stale` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_printed_template_in_a_capability_of_each_state_gives_a_clean_check_and_one_unresolved_proposal` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_spdx_lines_are_the_two_comment_lines_of_the_records_of_this_repository` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_template_left_in_a_state_that_refuses_it_is_the_finding_the_guidance_names` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::a_capability_the_change_lists_that_has_no_document_is_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::a_change_record_that_did_not_load_is_exit_two_and_says_so` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::a_slug_that_is_not_a_slug_is_a_usage_error` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::an_unknown_slug_is_exit_two_and_names_the_open_changes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::quiet_prints_nothing_because_a_preview_is_not_a_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::the_output_is_the_text_the_library_previews_under_a_header_for_each_document` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::the_preview_is_the_same_by_root_from_another_directory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::the_proposed_text_is_printed_and_the_markers_are_not` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_cargo_that_cannot_be_started_is_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_cargo_that_fails_is_exit_two_with_what_it_said` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_missing_git_is_exit_two_and_names_it` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_slug_that_is_not_a_slug_is_refused_before_any_process_starts` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_before_any_process_starts` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_by_every_command_before_any_process_starts` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_usage_error_of_the_change_commands_starts_no_process` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_usage_error_starts_no_process` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::cargo_dispatches_workplan_to_the_binary_on_path_and_passes_the_exit_code_back` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::discovery_does_not_run_the_build_rustc_program_of_the_workspace` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::discovery_runs_exactly_cargo_metadata_from_the_cargo_variable` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::discovery_runs_the_cargo_on_path_when_the_variable_is_unset` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::output_from_cargo_that_is_not_the_metadata_is_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::root_starts_no_cargo_at_all` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::a_bound_example_that_fails_is_an_obligation_and_nothing_is_written_to_the_tree` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::everything_that_stops_it_looking_is_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::json_is_one_obligation_object_for_each_obligation` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::nothing_is_written_and_no_cargo_is_started_for_a_fixture_without_bound_examples` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::quiet_keeps_the_obligations_and_hides_nothing_that_is_a_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::a_directory_outside_any_workspace_is_exit_two_and_names_the_command` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::a_relative_root_is_taken_from_the_current_directory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::every_command_finds_the_root_from_a_subdirectory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::root_overrides_the_discovery_and_the_current_directory_is_not_read` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::sync_from_a_subdirectory_writes_at_the_root_and_nowhere_else` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::the_root_is_found_from_the_root_and_from_any_directory_below_it` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::a_directory_that_is_not_a_repository_is_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::a_state_with_no_capability_is_a_row_with_a_count_of_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::an_implementing_capability_with_every_binding_resolved_has_no_gap_beyond_its_empty_lists` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::quiet_leaves_standard_output_empty_because_the_roadmap_has_no_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::the_roadmap_groups_one_capability_in_each_state_with_the_gaps_and_the_change` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::the_roadmap_is_the_same_by_root_from_another_directory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_document_that_does_not_load_is_exit_two_and_nothing_is_written` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_missing_index_with_the_text_is_created_with_that_header` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_missing_index_without_the_text_is_exit_two_naming_the_field_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_refused_file_is_exit_two_and_nothing_is_written` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_second_sync_writes_nothing_and_prints_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_value_that_cannot_be_written_into_the_header_is_refused_when_the_index_is_created` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::each_field_that_is_not_given_is_named` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::only_the_generated_blocks_are_rewritten` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::quiet_hides_the_files_rewritten_and_keeps_the_exit_code` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::the_index_is_created_by_root_from_another_directory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::the_text_is_not_read_when_the_index_exists` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::a_failing_example_and_a_binding_that_proves_nothing_are_failures` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::a_hostile_binding_is_a_failure_and_never_reaches_cargo` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::a_slug_with_no_record_and_a_document_that_does_not_load_are_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::every_binding_passing_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::json_is_one_binding_object_for_each_binding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::no_slug_runs_every_capability_and_a_slug_only_the_ones_the_change_lists` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::nothing_to_run_is_exit_zero_with_a_note_that_quiet_hides` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::quiet_prints_only_the_failing_bindings_and_keeps_the_exit` | test | [Verification](#verification) |
| `raf_workplan::Workplan::impact` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::new_change` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::preview` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::sync` | doctest | [Verification](#verification) |
| `raf_workplan::root::json::tests::a_large_text_is_scanned_to_its_end` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::a_member_that_is_only_nested_is_not_the_member` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::a_member_twice_is_refused_even_when_the_values_agree` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::a_top_level_value_that_is_not_an_object_is_refused` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::a_value_that_is_not_a_string_is_refused` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::an_unpaired_surrogate_is_text_nobody_can_use` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::escapes_are_decoded` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::every_prefix_of_a_document_is_refused_and_no_mutation_panics` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::nesting_is_bounded_and_never_overflows_the_stack` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::numbers_of_the_grammar_pass` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::random_strings_of_json_punctuation_never_panic` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::text_that_is_not_json_is_refused` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::the_member_is_found_wherever_it_sits_at_the_top_level` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::the_output_of_cargo_metadata_gives_its_root` | test | [Verification](#verification) |
| `raf_workplan::root::tests::a_failing_cargo_is_an_error_with_what_it_said` | test | [Verification](#verification) |
| `raf_workplan::root::tests::a_root_that_is_not_an_absolute_path_is_refused` | test | [Verification](#verification) |
| `raf_workplan::root::tests::cargo_that_cannot_run_or_writes_too_much_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::root::tests::output_that_does_not_give_a_root_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::root::tests::the_root_is_the_workspace_root_member_and_the_command_is_exactly_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::a_cargo_that_does_not_exist_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::a_directory_outside_any_workspace_is_an_error_that_says_what_cargo_said` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::configured_programs::discovery_does_not_run_the_programs_of_the_cargo_configuration` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::stubs::exactly_one_process_runs_with_the_documented_argv_directory_and_locale` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::stubs::hostile_output_is_an_error_and_never_a_guess` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::stubs::output_is_bounded_at_64_mebibytes` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::stubs::the_top_level_member_wins_over_a_nested_one_with_the_same_name` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::the_root_is_found_from_the_root_a_member_and_a_subdirectory` | test | [Verification](#verification) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [workplan-complete-gate](./workplan-complete-gate.md) |
| Depends on | [workplan-generated-docs](./workplan-generated-docs.md) |
| Depends on | [workplan-reconcile](./workplan-reconcile.md) |
| Changed by | [workplan-cli](../../work/changes/workplan-cli.md) |
<!-- raf:generated:end -->
