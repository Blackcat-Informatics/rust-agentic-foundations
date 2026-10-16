---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-evidence-claims
title: "Evidence and claims"
description: "Persisted execution evidence, the claims that depend on each result, and invalidation when the evidence changes."
state: implementing
bindings:
  code:
    - raf_workplan::evidence
    - raf_workplan::exec::TrustedArgv
    - raf_workplan::Workplan::record_evidence
    - raf_workplan::Workplan::evidence_claims
    - raf_workplan::Workplan::evidence_for
    - raf_workplan::exec::TrustedWord
    - raf_workplan::exec::TrustedArgvError
  tests:
    - raf_workplan::test_crate::evidence::a_bound_test_gives_the_verdict_and_a_path_that_names_no_binding_starts_nothing
    - raf_workplan::test_crate::evidence::a_claim_on_a_value_over_its_limit_by_less_than_a_float_tells_apart_is_unsupported
    - raf_workplan::test_crate::evidence::a_record_supports_a_capability_by_its_for_list_without_a_claim
    - raf_workplan::test_crate::evidence::a_view_of_what_does_not_exist_is_an_error_and_its_neighbour_answers
    - raf_workplan::test_crate::evidence::hostile_record_text_is_read_as_text_and_reaches_no_process
    - raf_workplan::test_crate::evidence::the_views_list_the_claims_of_a_record_and_the_records_of_a_capability
    - raf_workplan::evidence::measure::tests::a_limit_is_a_name_and_a_number
    - raf_workplan::evidence::measure::tests::a_limit_never_printed_is_an_error_and_a_printed_one_is_judged
    - raf_workplan::evidence::measure::tests::a_name_printed_twice_is_an_error_and_two_names_are_not
    - raf_workplan::evidence::measure::tests::a_value_is_over_its_limit_by_its_text_and_not_by_its_nearest_float
    - raf_workplan::evidence::measure::tests::a_value_that_is_not_a_number_is_an_error_and_its_neighbour_is_not
    - raf_workplan::evidence::measure::tests::only_lines_of_a_name_an_equals_sign_and_a_number_are_values
    - raf_workplan::evidence::tests::a_capability_with_no_code_binding_or_one_that_does_not_resolve_is_refused
    - raf_workplan::evidence::tests::a_command_that_cannot_start_writes_no_record
    - raf_workplan::evidence::tests::a_new_record_never_replaces_one_and_a_replacement_keeps_nothing_of_it
    - raf_workplan::evidence::tests::a_passing_command_writes_one_record_with_its_command_values_limits_and_input_digests
    - raf_workplan::evidence::tests::a_run_killed_at_its_time_limit_records_fail_and_its_neighbour_passes
    - raf_workplan::evidence::tests::a_value_over_its_limit_by_less_than_a_float_tells_apart_records_fail
    - raf_workplan::evidence::tests::each_fault_of_the_printed_values_is_an_error_beside_a_passing_neighbour
    - raf_workplan::evidence::tests::every_refusal_of_the_request_starts_nothing_and_writes_nothing
    - raf_workplan::evidence::tests::more_values_than_a_record_keeps_is_an_error_and_the_most_it_keeps_records
    - raf_workplan::evidence::tests::output_past_the_bound_is_an_error_and_writes_nothing_and_output_at_it_records
    - raf_workplan::evidence::tests::the_argv_reaches_the_process_exactly_with_no_shell_of_the_workplan
    - raf_workplan::evidence::tests::the_command_is_asked_with_the_bounds_of_a_build_and_the_callers_locale
    - raf_workplan::evidence::tests::the_command_runs_in_the_root_with_its_input_closed_and_the_callers_environment
    - raf_workplan::evidence::tests::the_exit_status_and_the_limits_decide_the_verdict
    - raf_workplan::evidence::tests::the_recorded_line_says_why_a_verdict_is_fail
    - raf_workplan::exec::trusted::tests::a_line_with_no_command_is_refused_and_one_with_a_command_is_not
    - raf_workplan::exec::trusted::tests::a_word_that_is_not_utf8_is_refused_by_its_place
    - raf_workplan::exec::trusted::tests::the_command_is_every_word_after_the_first_separator
    - raf_workplan::exec::trusted::tests::the_text_quotes_each_word_a_shell_would_split_or_expand
    - raf_workplan::test_crate::evidence::an_edit_inside_a_bound_module_makes_its_record_stale_and_a_sibling_edit_does_not
  examples: []
depends: [cap:workplan-document-model, cap:workplan-reconcile]
---

# Evidence and claims

## Purpose

A statement such as "the parser stays under a fixed memory bound" is true only while
some execution shows it. This capability records executions as evidence and records
which current statements depend on which result, so that when a result changes, the
statements that rested on it are known and are reopened.

Open questions:

- Where evidence is stored: beside the capability, or in one directory for the
  workspace.
- How much of an execution is kept: the verdict only, or the verdict with the measured
  values.
- Whether a statement may depend on several results, and whether any one failing result
  invalidates it.

## Contract

> [!PROPOSED change:workplan-evidence-claims]
> **Evidence.** An evidence record states one execution: where its verdict came from,
> the verdict, the measured values and their limits, the capabilities it covers, and
> the digest of each input it was made against. Records are persisted as files in the
> repository, one at `work/evidence/<id>.md` for each id, and are written by
> `cargo workplan evidence record --id <id> --for cap:<slug>... [--limit name=number]...
> [--revision <commit>] [--copyright <text> --licence <id>]` with one source of the
> verdict:
>
> - `-- <program> <arg>...`: the command after `--` is run, exactly as given: with no
>   shell, standard input closed, the workspace root as its directory, the caller's
>   environment unchanged (no `--frozen` is added to a `cargo` in it), a limit of 1800
>   seconds and 16 MiB of output on each stream. Exit status 0 is `pass` and any other
>   status, a signal or a run killed at its time limit is `fail`. The measured values
>   are the lines of its standard output of the form `name=number`, where the name is
>   letters, digits, `_` and `-` from the start of the line; every other line is
>   discarded and no output is kept. A number is an optional `-`, digits and an optional
>   `.` followed by digits, with no exponent, so `1e-3` is refused and is written
>   `0.001`. A value over its limit makes the verdict `fail`; values and limits are
>   compared exactly on their decimal text, whatever their length, and never as
>   floating-point numbers. Every `name=number` line is stored as a value, up to 1024
>   values. A run killed at its time limit is recorded as `fail` with its limits as
>   given and no value, since the output of a killed process is not kept. The record
>   states the command as text for its readers and never runs it.
> - `--from-binding <path>`: the bound test or example `<path>` of a capability of
>   `--for` is run as the [completion gate](./workplan-complete-gate.md) runs it, and
>   its verdict is `pass` when it passes by the proof its kind requires. It takes no
>   limit, since it prints no value the tool reads.
>
> Nothing is written, and the command exits 2, when a value printed is not a number, a
> name is printed twice, more than 1024 values are printed, a `--limit` names a value
> a command that exited never printed (whether its status passed or failed), the output
> passes its bound, or the command cannot be started.
> Before anything runs, every capability of `--for` must exist, be loaded and bind code
> whose every `code` binding resolves, because the inputs are the digests of those
> symbols; a `--from-binding` path must be a `tests` or `examples` binding of one of
> them; and a `--revision` must be a commit id of 40 or 64 lowercase hexadecimal digits.
> The revision is stored for readers only. `by` states `human` or `agent` and no name.
> A new record never replaces a file; `--update` replaces the record of the id, and the
> replacement holds only what that run gives, limits included. Recording writes that
> one file.
>
> **Claims.** A claim is a statement in a capability document that depends on
> evidence. The document names the evidence a claim rests on, so the dependency is
> stated in the text and not inferred.
>
> **The dependency view.** The graph holds an edge from each claim to the evidence it
> rests on. `cargo workplan evidence claims <evidence-id>` answers which current
> statements depend on this result. `cargo workplan evidence for <cap:slug>` answers
> which results support a capability.
>
> **Invalidation.** When a record that passed at the merge base is replaced by one that
> fails, holds a value over its limit or the limit of the merge base, or has lost a limit
> it had there, or when the record is removed, each dependent claim receives an
> `evidence-changed` obligation in the [reconcile](./workplan-reconcile.md) list. A claim
> whose evidence does not exist, fails or holds a value over its limit is reported as
> unsupported (`claim-unsupported`), and a record one of whose input symbols has another
> digest now than the one it keeps is reported as stale (`evidence-stale`), naming the
> symbol.

## Usage

> [!PROPOSED change:workplan-evidence-claims]
> - `cargo workplan evidence record --id memory-bound --for cap:streaming-parser
>   --limit peak=64 -- cargo bench parse` runs the command and persists the evidence.
> - `cargo workplan evidence record --id parse-test --for cap:streaming-parser
>   --from-binding raf_parser::test_crate::bounds::stays_under_the_bound` takes the
>   verdict from a bound test.
> - A command whose standard output holds other `key=value` lines, such as
>   `status=ok`, fails the recording, because each such line is read as a value; keep
>   them on standard error or out of the output.
> - `cargo workplan evidence claims memory-bound` lists the statements that depend on it.
> - `cargo workplan change reconcile <slug>` includes an obligation for every claim
>   whose evidence changed.

## Examples

> [!PROPOSED change:workplan-evidence-claims]
> A claim in a capability document names its evidence, and the evidence record holds
> the result:
>
> | Part | Content |
> | --- | --- |
> | Statement | Parsing a 10 MiB input stays under 64 MiB resident. |
> | Evidence id | `memory-bound` |
> | Record | command `cargo bench parse`, verdict pass, values `peak: 41`, limits `peak: 64` |
>
> Re-recording the evidence with a printed `peak=80` records the verdict fail and makes
> the statement an obligation: the bound is still documented and no longer supported.

## Constraints

> [!PROPOSED change:workplan-evidence-claims]
> - An evidence record is a result and not a judgement. It never carries prose that
>   interprets the result.
> - A claim names its evidence explicitly. No dependency is guessed from wording.
> - Recording evidence never edits a capability document. The document changes only
>   through an authored edit that resolves the obligation.
> - Evidence is stale when the digest of one of its input symbols differs from the
>   digest the record keeps. Staleness reads no revision and asks Git nothing, so it
>   holds through squash merges and shallow clones; the revision a record states is
>   for its readers.
> - The stored command is never run, and only the command line after `--` reaches a
>   process.
> - A run lasts until the command exits and its standard output and standard error are
>   closed: a background process that keeps them open after the command exits holds
>   the run until the time limit, which records `fail`.
> - Every `name=number` line of the output is stored, not only those a limit names, up
>   to 1024 values; more is an error.

## Explanation

The documented state of a system and the supported state differ by the evidence. A
design is accepted, code exists, tests address the claims, and executions provide
evidence, and each of these becomes true at a different time. A statement about a
bound, a latency or a limit moves from planned to supported only when an execution
shows it, and moves back when the execution stops showing it.

Persisting the evidence makes the relationship inspectable. A reader asking which
current statements depend on a result gets an answer from the graph and does not have
to search the prose. A maintainer who changes a benchmark learns which statements need
review before the change merges. This is the evidence-to-claims direction of the
[two-way coverage](./workplan-two-way-coverage.md) table.

## Verification

> [!PROPOSED change:workplan-evidence-claims]
> - A fixture with one claim and one record: the dependency view lists the claim for
>   the record and the record for the capability.
> - Replacing a passing record with a failing one produces an obligation for the claim.
> - Removing a record reports the claim as unsupported.
> - A record whose bound symbol was edited is reported as stale, naming the symbol, and
>   one beside an edit of another item or a change of line endings only is not.
> - The run of the command: its argv exactly, standard input closed, bounded output, a
>   kill at the time limit (recorded as fail), the workspace root as its directory and
>   the caller's environment; each fault of the printed values is an error beside a
>   passing neighbour, and a command that cannot start writes no record.
> - A claim that names an evidence id that does not exist is an error naming the file.
