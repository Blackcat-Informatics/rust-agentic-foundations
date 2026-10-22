<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Remaining CLI claim assertion map

Read-only primary preparation against the measured strict-claims candidate and
current actual assertions. No product edits or tests run. This is an ownership
map, not a declaration that every broad paragraph is proved by one fixture.

- Record values: cli.rs tests history_takes_an_id_of_each_kind_and_a_day and
  history_refuses_an_id_or_a_day_before_anything_runs assert actual argument
  variants and bad days/ids; process.rs
  an_id_a_day_or_a_value_of_the_record_commands_is_refused_before_any_process_starts
  asserts exit 2, empty output, no recorded Cargo/Git calls, unchanged repository
  and valid neighboring process starts. Preserve both parsed-value and process
  boundary ownership. Revision/limit/package specifics need their existing parser
  fixtures as well; the process corpus alone does not show every lexical endpoint.
- Evidence argv: evidence.rs the_words_after_the_separator_are_the_command_options_included
  compares the actual command's argv file with exact separate words and the
  recorded by value. process.rs above reaches refused_by_the_reading_of_the_command
  for empty/non-UTF8 argv, special argv0 and from-binding/bare-separator conflict;
  the positive neighbor actually executes. evidence.rs
  a_replacement_holds_the_spdx_lines_of_its_own_run_only supplies replacement
  ownership. Split these independent guarantees rather than letting one argv
  marker stand for replacement metadata too.
- Generic exits: check.rs clean/findings/unreadable/usage fixtures, reconcile.rs
  empty/nonempty/blocked fixtures, complete.rs actual completion/refusal/read
  failures, verify.rs passing/failing/missing-record fixtures and status.rs gap
  fixture exercise all three classes. process.rs a_git_older_than_2_30_is_exit_two_before_any_other_process
  asserts the version boundary. These are existing real binary tests.
- Record-command exits: evidence.rs pass/fail/unreadable-output, views and malformed
  capability fixtures; assess.rs verdict/list/stale/unassessable fixtures;
  history.rs shallow/uncommitted/missing-id fixtures; coverage/claims/lines actual
  report fixtures. The existing paragraph combines many commands and should have
  meaningful per-family ownership rather than an arbitrary representative marker.
- Options: cli.rs the_options_default_and_parse_in_both_shapes,
  the_options_may_follow_the_command, the_removed_policy_flag_is_rejected_before_and_after_the_command
  and an_unknown_or_json_format_is_a_usage_error assert actual parser variants.
  Quiet guarantees belong to each command's actual quiet fixture and the shared
  typed renderer tests, with default strict refusal covered by actual check and
  completion neighbors already in the capability.
- Debt: debt.rs the_write_creates_the_record_exits_zero_and_the_repository_then_passes
  asserts exact path, stdout, only changed file and clean check; growth/default
  finding fixtures assert refusal and unchanged snapshot; misuse asserts exit 2;
  typed written-record fixture asserts path/classification/count fields.
- Read-only/change writes: impact.rs impact_prints_what_the_change_reaches_and_writes_nothing
  and reconcile.rs nothing_is_written_and_no_cargo_is_started_for_a_fixture_without_bound_examples
  compare snapshots. sync.rs only_the_generated_blocks_are_rewritten and new.rs
  new_prints_the_library_text_writes_the_record_and_nothing_else_in_both_shapes
  cover the mutating neighbors; complete.rs completed/refused fixtures own removal.
- Record/view writes: evidence.rs the_views_print_rdf_and_refuse_what_does_not_exist
  compares the final repository snapshot; pass/fail recording fixtures compare
  changed paths. assess.rs context/list fixtures compare snapshots and verdict
  fixture compares the exact single changed record. Coverage and lines fixtures
  explicitly compare snapshots. History ownership still needs its exact read-only
  assertion inspected, not inferred from its output assertions.
- Verify: verify.rs one_passing_and_one_failing_test_print_one_line_each_and_exit_one
  compares actual binding results and unchanged repository snapshot in both
  invocation forms. No inspected CLI fixture asserts the exact Git call list;
  the stronger statement that only tracked-path listing is needed still needs
  actual recorded-call proof. Existing recording Stubs forwards to real Git and
  Cargo, so it can exercise this without inventing an unrelated fake runner.
- Process argv: raf-workplan exec::tests::every_argument_is_one_element_and_no_shell_reads_it
  runs actual SystemRunner, compares NUL-separated argv, and checks that metacharacter
  text produced no extra file. Bound-plan tests compare complete frozen Cargo
  vectors for each invocation family. This supports actual process construction;
  routing every CLI caller through that implementation also needs current source
  review. No production spawn/output/status call was found in cargo-workplan/src;
  its Command references are the clap command enum, not std::process::Command.
- Slug grammar: cli.rs a_slug_must_be_a_slug plus process.rs malformed/dash
  slug fixtures assert refusal before processes with valid neighbors.
- Missing/existing change records: preview.rs an_unknown_slug_is_exit_two_and_names_the_open_changes,
  impact.rs a_slug_with_no_record_is_exit_two_and_names_the_open_changes,
  verify.rs a_slug_with_no_record_and_a_document_that_does_not_load_are_exit_two,
  complete/reconcile failure fixtures and new.rs a_second_run_for_the_same_slug_is_exit_two_and_leaves_the_record_as_it_is
  supply distinct command outcomes. Inspect each actual assertion before binding.
- Terminal text: evidence.rs a_claimed_statement_reaches_the_terminal_escaped_and_rdf_escapes_it_as_rdf,
  assess.rs the_text_of_a_section_reaches_the_terminal_escaped_and_rdf_escapes_it_as_rdf,
  history/RDF row assertions and actual escape unit tests own text and machine
  representations separately. Native bidi replacement qualification is recorded
  in queue-native-gate-example-literal-bidi.md.

Observed stale comments in touched tests still call Turtle results JSON, including
assess.rs context, impact.rs module docs and debt.rs module docs; debt's module
description also retains the removed optional-policy prerequisite. Correct those
when editing these fixtures rather than treating them as current product contracts.
