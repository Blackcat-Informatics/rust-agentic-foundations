---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-complete-gate
title: "Completion gate"
description: "change complete and verify: the five completion conditions, lighter obligations for changes that leave the contract alone, deletion of the record, and the acceptance scenarios."
state: implementing
bindings:
  code:
    - raf_workplan::lifecycle::Lifecycle::complete
    - raf_workplan::lifecycle::Completion
    - raf_workplan::lifecycle::Failure
    - raf_workplan::lifecycle::Condition
    - raf_workplan::lifecycle::verify
    - raf_workplan::lifecycle::VerifyReport
    - raf_workplan::bound::Executor
    - raf_workplan::bound::Evidence
    - raf_workplan::exec::Invocation
    - raf_workplan::exec::Pattern
    - raf_workplan::exec::Programs
    - raf_workplan::exec::Limits
    - raf_workplan::exec::SystemRunner
  tests:
    - raf_workplan::test_crate::scenarios::several_redesigns_leave_one_readable_present
    - raf_workplan::test_crate::scenarios::planning_produces_the_documentation
    - raf_workplan::test_crate::scenarios::a_document_that_merely_exists_does_not_satisfy_the_scenario
    - raf_workplan::test_crate::scenarios::a_thin_document_is_not_the_account_the_scenario_builds
    - raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal
    - raf_workplan::test_crate::complete::the_cases_cover_every_kind_of_obligation_in_both_directions
    - raf_workplan::test_crate::complete::each_kind_of_obligation_makes_complete_refuse_and_leaves_the_tree_as_it_was_and_its_repair_completes
    - raf_workplan::test_crate::complete::an_item_without_outcome_and_a_move_to_nowhere_or_to_itself_refuse_condition_two
    - raf_workplan::test_crate::complete::a_proposed_block_of_the_change_refuses_condition_three_in_a_loaded_and_in_an_unloaded_document
    - raf_workplan::test_crate::complete::a_block_that_is_stale_or_missing_anywhere_refuses_condition_four
    - raf_workplan::test_crate::complete::a_failing_bound_test_refuses_condition_five_alone
    - raf_workplan::test_crate::complete::a_failing_bound_example_stands_under_conditions_one_and_five
    - raf_workplan::test_crate::complete::a_failing_doc_test_a_test_that_runs_nothing_and_an_ignored_test_refuse_beside_passing_neighbours
    - raf_workplan::test_crate::complete::an_item_without_outcome_and_a_stale_block_print_the_counts_of_the_contract
    - raf_workplan::test_crate::complete::a_capability_left_implementing_with_no_other_record_refuses_and_says_what_to_change
    - raf_workplan::test_crate::complete::a_delivered_capability_beside_its_open_record_is_what_complete_resolves
    - raf_workplan::test_crate::complete::a_delivered_capability_with_a_binding_that_does_not_resolve_refuses
    - raf_workplan::test_crate::complete::empty_bindings_satisfy_condition_five_only_while_the_capability_is_designed_or_considered
    - raf_workplan::test_crate::complete::contract_effect_none_completes_without_usage_or_example_edits_and_still_refuses_for_anything_else
    - raf_workplan::test_crate::complete::hostile_bindings_refuse_and_never_reach_cargo
    - raf_workplan::test_crate::complete::a_static_failure_spends_no_process_on_cargo_and_a_clean_change_runs_cargo_after_the_static_checks
    - raf_workplan::test_crate::complete::a_document_edited_while_the_tests_ran_refuses_and_nothing_is_written
    - raf_workplan::test_crate::complete::a_binding_added_while_the_tests_ran_is_a_change_even_when_every_condition_still_holds
    - raf_workplan::test_crate::complete::a_test_renamed_while_the_tests_ran_refuses
    - raf_workplan::test_crate::complete::an_error_after_the_rewrite_does_not_leave_a_removed_record_beside_stale_blocks
    - raf_workplan::test_crate::complete::writes::a_rewrite_that_fails_leaves_the_record_and_the_blocks_as_they_were
    - raf_workplan::test_crate::complete::prose_that_links_to_the_record_refuses_because_the_link_is_broken_once_it_is_removed
    - raf_workplan::test_crate::complete::a_broken_link_in_a_capability_the_change_does_not_list_also_blocks_completion
    - raf_workplan::test_crate::complete::an_outcome_the_profile_refuses_makes_the_record_not_load_and_is_an_error
    - raf_workplan::test_crate::bound::a_passing_test_passes_on_one_listed_entry_and_one_passed
    - raf_workplan::test_crate::bound::a_failing_test_fails_the_binding
    - raf_workplan::test_crate::bound::an_ignored_test_is_listed_and_refused
    - raf_workplan::test_crate::bound::a_test_behind_a_disabled_feature_lists_nothing_and_is_refused
    - raf_workplan::test_crate::bound::the_same_name_in_two_targets_runs_only_in_the_pinned_one
    - raf_workplan::test_crate::bound::an_integration_test_with_a_hyphen_runs_under_its_cargo_name
    - raf_workplan::test_crate::bound::a_function_that_is_not_a_test_is_refused_before_any_process
    - raf_workplan::test_crate::bound::hostile_bindings_are_unresolved_and_never_reach_cargo
    - raf_workplan::test_crate::bound::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran
    - raf_workplan::test_crate::bound::documentation_tests_follow_the_rules_in_edition_2021
    - raf_workplan::test_crate::bound::documentation_tests_follow_the_rules_in_edition_2024
    - raf_workplan::test_crate::bound::an_item_path_that_begins_with_a_hyphen_never_reaches_cargo
    - raf_workplan::test_crate::bound::an_example_that_exits_zero_passes_and_one_that_does_not_fails
    - raf_workplan::test_crate::bound::a_hyphenated_example_runs_under_its_cargo_name
    - raf_workplan::test_crate::bound::an_example_that_reads_standard_input_sees_it_closed
    - raf_workplan::test_crate::bound::an_example_that_never_ends_is_killed_and_reported_with_the_time_it_ran
    - raf_workplan::test_crate::exec::the_patterns_are_the_ones_the_documents_state
    - raf_workplan::test_crate::exec::every_pattern_refuses_hostile_values
    - raf_workplan::test_crate::exec::every_invocation_is_the_row_of_its_table
    - raf_workplan::test_crate::exec::the_limits_are_the_documented_bounds
    - raf_workplan::test_crate::exec::stubs::every_row_reaches_a_process_as_exactly_its_argv
    - raf_workplan::test_crate::exec::stubs::a_stub_named_by_cargo_runs_for_cargo_and_a_stub_on_the_path_when_it_is_unset
    - raf_workplan::test_crate::exec::stubs::standard_input_is_closed
    - raf_workplan::test_crate::exec::stubs::a_process_that_outlives_its_limit_is_killed
    - raf_workplan::test_crate::exec::stubs::output_at_the_bound_passes_and_one_byte_over_is_refused_on_each_stream
    - raf_workplan::test_crate::verify::one_passing_and_one_failing_test_print_one_line_each_and_the_run_fails
    - raf_workplan::test_crate::verify::no_slug_runs_every_capability_and_a_slug_only_the_ones_the_change_lists
    - raf_workplan::test_crate::verify::examples_and_doc_tests_are_run_and_judged_by_their_proof
    - raf_workplan::test_crate::verify::zero_tests_and_ignored_tests_fail_the_run
    - raf_workplan::test_crate::verify::hostile_bindings_are_unresolved_results_that_never_reach_cargo
    - raf_workplan::test_crate::verify::verify_writes_nothing
    - raf_workplan::test_crate::verify::verify_asks_git_nothing_and_does_not_go_through_the_version_gate
    - cargo_workplan::test_crate::complete::a_completed_change_prints_what_was_removed_and_rewritten_and_leaves_a_clean_check
    - cargo_workplan::test_crate::complete::a_failing_example_stands_under_conditions_one_and_five
    - cargo_workplan::test_crate::complete::a_failing_bound_test_is_condition_five_and_the_record_stays
    - cargo_workplan::test_crate::verify::one_passing_and_one_failing_test_print_one_line_each_and_exit_one
  examples:
    - raf_workplan::bound
    - raf_workplan::exec
depends: [cap:workplan-reconcile, cap:workplan-generated-docs]
---

# Completion gate

## Purpose

A change is complete when the current documentation says what the system now does,
and not when a status flag is set. `change complete` is the gate that checks it. It
makes documentation a condition of completing the work, in place of a final
checkbox that says "update docs".

## Contract

**Conditions.** `change complete <slug>` succeeds only when all five hold:

1. The obligation list from [reconcile](./workplan-reconcile.md) is empty, with
   zero obligations.
2. Every item in the record has an `outcome` of `implemented`, `dropped` or
   `moved:change:<slug>`, and a `moved` outcome names a change that exists.
3. No PROPOSED block for the change remains in any capability document.
4. `docs sync` would rewrite nothing, so every generated block is current.
5. The bound tests and examples pass: `cargo test` filtered to the tests in
   `bindings.tests` of each listed capability, and the targets and documentation
   tests in `bindings.examples` built and run, through the invocations below.

A failing bound example also stands as an `example-failing` obligation, so conditions
1 and 5 both report it. Condition 5 is the execution and condition 1 the record of it;
the refusal lists the fault under each.

When any condition fails, the command prints the failures, changes nothing and exits
with 1. When all hold, it deletes `work/changes/<slug>.md` and exits with 0. The
deletion is the last step and the only write. Git holds the record's history.

**Obligations by effect.** All five conditions apply to every change. The effect only
sets which obligations [reconcile](./workplan-reconcile.md) raises. A change with
`contract_effect: changes` can owe an untouched-section obligation for each section in
`affects`. A change with `contract_effect: none` records that the documented contract
is unaffected, so reconcile raises no `section-untouched` obligation for the contract,
usage and examples sections. Every other obligation kind still applies, and the
obligation list must still be empty. A small internal change does not need a new
tutorial.

**State.** Promotion to a stronger state requires reconciliation, not just a status
change. The gate refuses a listed capability whose `state` the content does not
support: a `delivered` capability needs resolved text, no PROPOSED block, and
bindings that resolve; it binds at least one code path and one test. A capability
in `implementing` may have missing bindings, and `status`, the catalogue and the
reconcile obligations report them as gaps.

**Invocation.** Bound tests and examples run only through the argv forms below. Each
process is spawned directly with no shell and with its arguments as an argv vector.
The program is the one named by `$CARGO` when that is set, and `cargo` from `PATH`
otherwise. Every call carries `--frozen`, so completion never reaches the network or
rewrites `Cargo.lock`. The placeholders are filled only from the source index, never
from raw document text:

| Binding | Argv |
| --- | --- |
| Test, list pass | `cargo test --frozen --package=<pkg> <target> -- --list --exact -- <name>` |
| Test, run pass | `cargo test --frozen --package=<pkg> <target> -- --exact -- <name>` |
| Doc test, list pass | `cargo test --frozen --package=<pkg> --doc -- --list` |
| Doc test, run pass | `cargo test --frozen --package=<pkg> --doc -- <item-path>` |
| Example target | `cargo run --frozen --package=<pkg> --example=<example>` |

`<target>` is the resolved target of the test, written `--lib`, `--bin=<bin>` or
`--test=<test>`, so that the same name in another target is never run. `<name>` is
the resolved item's path within its crate. The `--` before `<name>` is a backstop:
libtest reads a value after it as a filter even if validation regressed.

Four patterns validate the placeholders, and a value that fails its pattern is
reported as an unresolved binding and is not passed to `cargo`:

- `<name>`: `[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*`;
- `<pkg>`, `<bin>`, `<test>` and `<example>`: `[A-Za-z0-9_][A-Za-z0-9_-]*`, which
  allows the hyphens that target names carry and never begins with `-`;
- `<item-path>`: the `<name>` pattern, widened so that an impl item written by rustdoc as `Wrapper<T>::get` can be bound. Each `::`-separated segment is `[A-Za-z_][A-Za-z0-9_]*` followed by an optional generic argument list matching `<[A-Za-z0-9_,':&]*>`, with no whitespace, and the filter never begins with `-`;
- the names in the list output, which the gate parses and never places on a command line, have the form `<relative-source-path> - <item-path> (line <n>)`, where `<relative-source-path>` matches `[A-Za-z0-9_][A-Za-z0-9_./-]*`.

**Proof that the test ran (test bindings).** Exit status 0 from `cargo test` is not proof, because
libtest reports success when a filter matches nothing. A test binding counts as passed
only when the list pass shows exactly one `<name>: test` entry (an `#[ignore]`d test
is listed as a test, so the run pass must also report it ran) and the run pass reports
`1 passed; 0 failed; 0 ignored` for that target. Zero entries, more than one entry, or
`0 passed` is an unresolved-binding failure and refuses completion, so a resolved item
that is not a `#[test]`, a test behind a disabled `cfg(feature)`, and a name that
differs from the libtest name all refuse. A doc-test binding is run by listing the doc
tests of the package, selecting the entries whose item part (the text between ` - `
and ` (line `) equals the bound item path within the crate, and then running the doc
tests with the bare item path as a whitespace-free substring filter, without
`--exact`. rustdoc splits every test argument on whitespace, so a full name such as
`src/lib.rs - foo (line 4)` can never act as one exact filter. The run output is
parsed for `test <name> ... ok` lines, and every entry selected by the list pass must
appear as `... ok`, matched strictly as the line `test <name> ... ok` with nothing
between the name and `...`. A `no_run` doc test, reported as `<name> - compile ...
ok`, was built and not run, and a `compile_fail` doc test is reported as `<name> -
compile fail ... ok`. Neither satisfies built and run, so a binding that selects one
is refused with a message that tells the author to bind a runnable doc test. No
selected entry is a refusal. A selected entry that fails, is ignored or does not
appear is a refusal. Any other doc test that the substring also matches (for example
`foo_more` for the item `foo`) is ignored by the gate and neither required to pass nor
counted, but its failure still fails the `cargo test` exit status, so the binding is
refused.

**Examples.** `cargo run` for an example target runs with its standard input closed
(null) and a timeout of 300 seconds, after which the process is killed and the binding
counts as failing. The example's own exit status decides pass or fail. The run
executes the workspace's cargo configuration as `cargo test` would, so the workspace
is trusted build input (see [the security policy](../../SECURITY.md)).

**`verify [<slug>]`** runs the bound tests and the bound examples of the named change's
capabilities, or of every capability when no slug is given, and prints pass or fail for
each binding. Examples run as their target or documentation test. Evidence is not
persisted: `verify` reports the run and writes nothing. Exit code 0 means every
binding passed.

## Usage

```sh
cargo workplan change reconcile incremental-parsing
cargo workplan verify incremental-parsing
cargo workplan change complete incremental-parsing
```

`complete` is the final command. Before it, `docs preview` shows the capability as it
reads.

## Examples

A refusal, followed by success after the documentation is fixed:

```text
$ cargo workplan change complete incremental-parsing
refused: 2 obligations, 1 item without outcome, 1 stale generated block
$ cargo workplan docs sync && cargo workplan change complete incremental-parsing
completed: removed work/changes/incremental-parsing.md
```

A refusal on the fifth condition, where a bound example no longer builds:

```text
$ cargo workplan change complete incremental-parsing
refused: 1 obligation, 1 bound example failing
cap:streaming-parser  examples  example-failing  examples/chunked.rs does not build
```

## Constraints

- `complete` runs only bindings that resolve in the source index and match their
  patterns, and counts a binding as passed only by the proof its kind requires: for a
  test binding, that exactly one test ran; for a doc-test binding, that every doc test
  the list pass selected reported `... ok`; for an example binding, that the example
  exited with status 0 within the timeout. It never passes document text to `cargo` as
  a shell string or as an option.
- `complete` runs the bound tests and examples with `cargo`, so a repository whose
  tests or examples cannot compile cannot complete a change.
- A capability with empty `bindings.tests` and `bindings.examples` lists satisfies
  the fifth condition only while it is `designed` or `considered`.
- The record is deleted and not archived. Recovering it is a Git operation.
- `complete` never edits a capability document; the prose is written by a person or
  an agent before the command runs.

## Explanation

Documentation is part of completing a change because the alternative is a document
that is correct at design time and drifts from the product afterwards. A change that
names the documentation it creates or alters when it is opened, and cannot finish
until each part is current, makes the maintenance step part of the work.

Obligations scale with the change. Requiring a usage example for a refactor teaches
people to write meaningless ones; letting an internal change record that it touches no
contract keeps the gate credible where the contract does move.

Deleting the record keeps the reading path short. After completion, the capability
document is the one account; the claims, the discarded options and the intermediate
states are in Git, where an entity-aware history query can find them. The rationale
that still matters has already been written into the Explanation section.

## Verification

The gate has two flagship acceptance fixtures, built as repositories in temporary
directories.

**Scenario A: several redesigns, one readable present.** One capability passes through
four redesigns by successive changes. Decisions are amended and reversed, and
interfaces and constraints evolve. At the end the fixture asserts: the document
states the active contract directly; every link resolves; no PROPOSED block or change
file is on the reading path; the Explanation section still holds the rationale; and
every earlier state is recoverable with `git show`. The test is whether a reader needs
to reconstruct the changes to understand what applies now.

**Scenario B: planning produces the documentation.** The fixture starts with a
capability in `considered` with open questions. The implementation renames an API and
drops one guarantee. `change complete` must refuse until Usage, Examples and the
bindings match the new API and the dropped claim has the outcome `dropped`. It then
succeeds and leaves the capability `delivered` with an accurate capability
explanation, a real usage example, current API references, explicit limitations and
a test traceability table, and no proposal text presented as established
behaviour. A document that merely exists does not satisfy
the scenario.

Further checks:

- A refusal fixture per condition: a remaining obligation, an item without outcome, a
  leftover PROPOSED block, a stale generated block, a failing bound test and a bound
  example that fails to build each produce exit 1 and leave the record in place.
  The example refusal prints an `example-failing` obligation for the capability.
- A `contract_effect: none` fixture that completes without usage or example edits and
  still refuses when any other obligation remains.
- A fixture asserting the record is gone after success and the working tree has no
  other change.
- A binding fixture with a stub named by `$CARGO` (and one on `PATH` when `$CARGO` is
  unset) records each argv and asserts it equals the matching row of the invocation
  table with `--frozen`, `--package=` and the pinned target, and that no shell is
  involved.
- A hostile binding fixture: a `bindings.tests` entry `-Zunstable-options`, one
  `--include-ignored`, one `--exact`, and a path with a `;` or a space each yield an
  unresolved-binding finding, make `complete` refuse and `verify` exit 1, and are
  never passed to `cargo`.
- A zero-test fixture: a binding that resolves in the source index to a function that
  is not a `#[test]` runs zero tests, so `complete` refuses and `verify` exits 1; so
  does an `#[ignore]`d test, and a test whose name matches in two targets runs only
  in the pinned target.
- A doc-test fixture: a binding to a documented function with a passing doc test runs
  that doc test with the bare item path as the filter and passes only when its
  `test <full name> ... ok` line appears; the same fixture with a failing doc test
  makes `complete` refuse; a doc-test binding that matches no doc test refuses; a
  selected entry missing from the run output refuses; an item whose substring also
  matches a failing `foo_more` doc test refuses; an item `Wrapper<T>::get` binds and
  passes; a `no_run` doc test and a `compile_fail` doc test, each bound as a doc test,
  are refused with the message that asks for a runnable doc test; and an item path
  that begins with `-` is rejected by its pattern.
- An example fixture: a hyphenated example target runs, an example that reads stdin
  sees it closed, and an example that does not exit within the timeout fails the
  binding.
- A `verify` fixture with one passing and one failing bound test, asserting per-binding
  output and exit codes 0 and 1.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::test_crate::complete::a_completed_change_prints_what_was_removed_and_rewritten_and_leaves_a_clean_check` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::a_failing_bound_test_is_condition_five_and_the_record_stays` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::a_failing_example_stands_under_conditions_one_and_five` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::one_passing_and_one_failing_test_print_one_line_each_and_exit_one` | test | [Verification](#verification) |
| `raf_workplan::bound` | doctest | [Verification](#verification) |
| `raf_workplan::exec` | doctest | [Verification](#verification) |
| `raf_workplan::test_crate::bound::a_failing_test_fails_the_binding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::a_function_that_is_not_a_test_is_refused_before_any_process` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::a_hyphenated_example_runs_under_its_cargo_name` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::a_passing_test_passes_on_one_listed_entry_and_one_passed` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::a_test_behind_a_disabled_feature_lists_nothing_and_is_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::an_example_that_exits_zero_passes_and_one_that_does_not_fails` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::an_example_that_never_ends_is_killed_and_reported_with_the_time_it_ran` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::an_example_that_reads_standard_input_sees_it_closed` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::an_ignored_test_is_listed_and_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::an_integration_test_with_a_hyphen_runs_under_its_cargo_name` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::an_item_path_that_begins_with_a_hyphen_never_reaches_cargo` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::documentation_tests_follow_the_rules_in_edition_2021` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::documentation_tests_follow_the_rules_in_edition_2024` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::hostile_bindings_are_unresolved_and_never_reach_cargo` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bound::the_same_name_in_two_targets_runs_only_in_the_pinned_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_binding_added_while_the_tests_ran_is_a_change_even_when_every_condition_still_holds` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_block_that_is_stale_or_missing_anywhere_refuses_condition_four` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_broken_link_in_a_capability_the_change_does_not_list_also_blocks_completion` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_capability_left_implementing_with_no_other_record_refuses_and_says_what_to_change` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_delivered_capability_beside_its_open_record_is_what_complete_resolves` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_delivered_capability_with_a_binding_that_does_not_resolve_refuses` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_document_edited_while_the_tests_ran_refuses_and_nothing_is_written` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_failing_bound_example_stands_under_conditions_one_and_five` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_failing_bound_test_refuses_condition_five_alone` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_failing_doc_test_a_test_that_runs_nothing_and_an_ignored_test_refuse_beside_passing_neighbours` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_proposed_block_of_the_change_refuses_condition_three_in_a_loaded_and_in_an_unloaded_document` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_static_failure_spends_no_process_on_cargo_and_a_clean_change_runs_cargo_after_the_static_checks` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::a_test_renamed_while_the_tests_ran_refuses` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::an_error_after_the_rewrite_does_not_leave_a_removed_record_beside_stale_blocks` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::an_item_without_outcome_and_a_move_to_nowhere_or_to_itself_refuse_condition_two` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::an_item_without_outcome_and_a_stale_block_print_the_counts_of_the_contract` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::an_outcome_the_profile_refuses_makes_the_record_not_load_and_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::contract_effect_none_completes_without_usage_or_example_edits_and_still_refuses_for_anything_else` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::each_kind_of_obligation_makes_complete_refuse_and_leaves_the_tree_as_it_was_and_its_repair_completes` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::empty_bindings_satisfy_condition_five_only_while_the_capability_is_designed_or_considered` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::hostile_bindings_refuse_and_never_reach_cargo` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::prose_that_links_to_the_record_refuses_because_the_link_is_broken_once_it_is_removed` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::the_cases_cover_every_kind_of_obligation_in_both_directions` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::writes::a_rewrite_that_fails_leaves_the_record_and_the_blocks_as_they_were` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::every_invocation_is_the_row_of_its_table` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::every_pattern_refuses_hostile_values` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::stubs::a_process_that_outlives_its_limit_is_killed` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::stubs::a_stub_named_by_cargo_runs_for_cargo_and_a_stub_on_the_path_when_it_is_unset` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::stubs::every_row_reaches_a_process_as_exactly_its_argv` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::stubs::output_at_the_bound_passes_and_one_byte_over_is_refused_on_each_stream` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::stubs::standard_input_is_closed` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::the_limits_are_the_documented_bounds` | test | [Verification](#verification) |
| `raf_workplan::test_crate::exec::the_patterns_are_the_ones_the_documents_state` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::a_document_that_merely_exists_does_not_satisfy_the_scenario` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::a_thin_document_is_not_the_account_the_scenario_builds` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::planning_produces_the_documentation` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::several_redesigns_leave_one_readable_present` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::examples_and_doc_tests_are_run_and_judged_by_their_proof` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::hostile_bindings_are_unresolved_results_that_never_reach_cargo` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::no_slug_runs_every_capability_and_a_slug_only_the_ones_the_change_lists` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::one_passing_and_one_failing_test_print_one_line_each_and_the_run_fails` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::verify_asks_git_nothing_and_does_not_go_through_the_version_gate` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::verify_writes_nothing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::zero_tests_and_ignored_tests_fail_the_run` | test | [Verification](#verification) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [workplan-generated-docs](./workplan-generated-docs.md) |
| Depends on | [workplan-reconcile](./workplan-reconcile.md) |
| Depended on by | [workplan-cli](./workplan-cli.md) |
| Depended on by | [workplan-prose-assessment](./workplan-prose-assessment.md) |
| Changed by | [workplan-complete-gate](../../work/changes/workplan-complete-gate.md) |
<!-- raf:generated:end -->
