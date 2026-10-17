---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-evidence-claims
title: "Evidence and claims"
description: "Persisted execution evidence, the claims that depend on each result, and invalidation when the evidence changes."
state: delivered
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
    - raf_workplan::evidence::tests::a_directory_link_on_the_way_to_the_record_is_refused_before_the_command_runs
    - raf_workplan::evidence::tests::evidence_through_the_gate_never_replaces_a_capability_document_through_a_link
    - raf_workplan::test_crate::evidence::a_bound_test_gives_the_verdict_and_a_path_that_names_no_binding_starts_nothing
    - raf_workplan::test_crate::evidence::a_claim_on_a_value_over_its_limit_by_less_than_a_float_tells_apart_is_unsupported
    - raf_workplan::test_crate::evidence::a_record_supports_a_capability_by_its_for_list_without_a_claim
    - raf_workplan::test_crate::evidence::a_view_of_what_does_not_exist_is_an_error_and_its_neighbour_answers
    - raf_workplan::test_crate::evidence::hostile_record_text_is_read_as_text_and_reaches_no_process
    - raf_workplan::test_crate::evidence::the_views_list_the_claims_of_a_record_and_the_records_of_a_capability
    - raf_workplan::test_crate::evidence::the_claims_of_a_record_are_refused_while_a_capability_document_does_not_load
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
    - raf_workplan::exec::trusted::tests::an_error_about_the_command_quotes_its_words_as_the_record_stores_them
    - raf_workplan::test_crate::evidence::an_edit_inside_a_bound_module_makes_its_record_stale_and_a_sibling_edit_does_not
  examples:
    - raf_workplan::Workplan::record_evidence
    - raf_workplan::evidence::parse_limit
depends: [cap:workplan-document-model, cap:workplan-reconcile]
---

# Evidence and claims

## Purpose

A statement such as "the parser stays under a fixed memory bound" is true only while
some execution shows it. This capability records executions as evidence and records
which current statements depend on which result, so that when a result changes, the
statements that rested on it are known and are reopened.

## Contract

**Evidence.** An evidence record states one execution: where its verdict came from,
the verdict, the measured values and their limits, the capabilities it covers, and
the digest of each input it was made against. Records are persisted as files in the
repository, one at `work/evidence/<id>.md` for each id, and are written by
`cargo workplan evidence record --id <id> --for cap:<slug>... --by human|agent
[--limit name=number]... [--revision <commit>] [--update] [--copyright <text>
--licence <id>]` with one source of the verdict:

- `-- <program> <arg>...`: the command after `--` is run, exactly as given: with no
  shell, standard input closed, the workspace root as its directory, the caller's
  environment unchanged (no `--frozen` is added to a `cargo` in it), a limit of 1800
  seconds and 16 MiB of output on each stream. Exit status 0 is `pass` and any other
  status, a signal or a run killed at its time limit is `fail`. The measured values
  are the lines of its standard output of the form `name=number`: a line whose text
  before its first `=` is a name, letters, digits, `_` and `-` from the start of the
  line, is a value, and the rest of it must be a number; every other line is
  discarded and no output is kept. A number is an optional `-`, digits and an optional
  `.` followed by digits, with no exponent, so `1e-3` is refused and is written
  `0.001`. A value over its limit makes the verdict `fail`; values and limits are
  compared exactly on their decimal text, whatever their length, and never as
  floating-point numbers. Every `name=number` line is stored as a value, up to 1024
  values. A run killed at its time limit is recorded as `fail` with its limits as
  given and no value, since the output of a killed process is not kept. The record
  states the command as text for its readers and never runs it.
- `--from-binding <path>`: the bound test or example `<path>` of a capability of
  `--for` is run as the [completion gate](./workplan-complete-gate.md) runs it, and
  its verdict is `pass` when it passes by the proof its kind requires. It takes no
  limit, since it prints no value the tool reads.

Nothing is written, and the command exits 2, when a value printed is not a number, a
name is printed twice, more than 1024 values are printed, a `--limit` names a value
a command that exited never printed (whether its status passed or failed), the output
passes its bound, or the command cannot be started.
Before anything runs, every capability of `--for` must exist, be loaded and bind code
whose every `code` binding resolves, because the inputs are the digests of those
symbols; a `--from-binding` path must be a `tests` or `examples` binding of one of
them; and a `--revision` must be a commit id of 40 or 64 lowercase hexadecimal digits.
The revision is stored for readers only. `by` states `human` or `agent` and no name.
A new record never replaces a file; `--update` replaces the record of the id, and the
replacement holds only what that run gives, limits included. Recording writes that
one file.

**Claims.** A claim is a statement in a capability document that depends on
evidence. The document names the evidence a claim rests on, so the dependency is
stated in the text and not inferred: a claim marker,
`<!-- raf:claim evidence=<id>[,<id>...] -->`, on its own line directly after the
paragraph or list item it supports, names one or more evidence records by slug (the
syntax is stated by the [OKF profile](../architecture/okf-profile.md#claim-markers)).
A claim may rest on several records and holds only while every one of them holds. A
marker inside a PROPOSED block is a claim once the block is resolved. `check` reports
a marker that names an evidence record that does not exist
(`claim-evidence-missing`) or a record whose
`for` does not list the capability of the claim (`claim-evidence-not-for`), at the
file and line of the marker, and a record whose `for` names a capability with no
document (`evidence-for-missing`).

**The dependency view.** The graph holds an edge from each claim to the evidence it
rests on. `cargo workplan evidence claims <evidence-id>` answers which current
statements depend on this result. While a capability document exists and does not
load, the claims it holds are unknown, so `evidence claims` gives no answer and is
an error (`EvidenceLookupError::UnloadedCapabilities`, exit 2) naming each such
document, as `evidence for` is for a capability that does not load; a change
record or an evidence record that does not load holds no claim and leaves the
answer as it is. `cargo workplan evidence for <cap:slug>` answers which results
support a capability.

**Invalidation.** When a record that passed at the merge base is replaced by one that
fails, holds a value over its limit or the limit of the merge base, or has lost a limit
it had there, or when the record is removed, each dependent claim receives an
`evidence-changed` obligation in the [reconcile](./workplan-reconcile.md) list. A claim
whose evidence does not exist, fails or holds a value over its limit is reported as
unsupported (`claim-unsupported`), and a record one of whose input symbols has another
digest now than the one it keeps, or no longer resolves, is reported as stale
(`evidence-stale`), naming the symbol.

## Usage

- `cargo workplan evidence record --id memory-bound --for cap:streaming-parser
  --by agent --limit peak=64 -- cargo bench parse` runs the command and persists the
  evidence.
- `cargo workplan evidence record --id parse-test --for cap:streaming-parser
  --by human --from-binding raf_parser::test_crate::bounds::stays_under_the_bound`
  takes the verdict from a bound test.
- A command whose standard output holds other lines that start with a name and `=`,
  such as `status=ok`, has each of them read as a value: one that is not a number,
  as here, makes the recording an error that writes nothing, and a number is stored
  as one more value; keep such lines on standard error or out of the output.
- `cargo workplan evidence claims memory-bound` lists the statements that depend on it.
- `cargo workplan change reconcile <slug>` includes an obligation for every claim of
  a capability the change lists whose evidence changed.

## Examples

A claim in a capability document names its evidence with a marker after the
statement:

```markdown
Parsing a 10 MiB input stays under 64 MiB resident.
<!-- raf:claim evidence=memory-bound -->
```

and the evidence record `work/evidence/memory-bound.md` holds the result:

| Part | Content |
| --- | --- |
| Subject | `command`, with the command text `cargo bench parse` |
| Verdict | pass |
| Values | `peak: 41` |
| Limits | `peak: 64` |
| Inputs | the digest of each code binding of `cap:streaming-parser` |

Recording the evidence again with `--update` when the command prints `peak=80`
records the verdict fail, and `change reconcile` lists the statement as
`claim-unsupported`, and as `evidence-changed` when the record passed at the merge
base: the bound is documented and no longer supported.

## Constraints

- An evidence record is a result and not a judgement. It never carries prose that
  interprets the result.
- A claim names its evidence explicitly. No dependency is guessed from wording.
- Recording evidence never edits a capability document. The document changes only
  through an authored edit that resolves the obligation.
- Staleness reads no revision and asks Git nothing, so it holds through squash
  merges and shallow clones (the [decision](#decision) below gives the reasons).
- The stored command is never run, and no text of a record reaches a process: the
  command `evidence record` runs is the command line after `--`, and a
  `--from-binding` path reaches `cargo` only as the test filter of the rows the
  completion gate runs.
- A run lasts until the command exits and its standard output and standard error are
  closed: a background process that keeps them open after the command exits holds
  the run until the time limit, which records `fail`.
- Evidence is written only through real directories: before the command or the
  binding runs, `work` and `work/evidence` must each be a directory, and a symbolic
  link there, inside the root or out of it, or a file is refused with nothing run
  and nothing written, so a record never lands on a document through a link.
- The command is supervised only while `cargo-workplan` runs. On Unix it leads a
  process group of its own, so an interrupt or a hang-up from the terminal, such
  as Ctrl-C, reaches `cargo-workplan` and not the command: once `cargo-workplan`
  has died the command runs on with no time limit and no record is written. A
  descendant that leaves the group, as one that calls `setsid` or `setpgid` does,
  is not reached by the kill at the time limit and may outlive it; elsewhere than
  Unix only the process itself is killed.
- The command runs with the workspace root as its directory, so a relative
  program, such as `./tool.sh` typed in a directory below the root, resolves
  against the root and not the caller's directory, and with `.` on `PATH` a bare
  name is looked for in the root as well. The record keeps the command as it was
  typed, so name a program by an absolute path, or by a path from the root, to be
  sure of the one it names.
- An error message about the command, such as the kill for too much output or a
  program that cannot start, quotes its words as `TrustedArgv::text` writes them
  and the record stores them: a word that a shell would split or expand is in
  single quotes, so `sh -c 'yes >&2'` and `sh -c yes '>&2'` read apart.
- At most 1024 values is a bound on what `evidence record` writes. A record
  written by hand may hold more; it is bounded only by the 1 MiB bound of a
  document, and it loads and is judged as a written one is.
- The judgement of the way to a record is made by path, not through a held
  handle of each directory: a local writer that swaps a directory for a link
  between the judgement and the write is not stopped. Such a writer can already
  edit the workspace, which the trust model of the
  [security policy](../../SECURITY.md) places outside what the tool defends.

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

### Decision

A record is stale when the digest of one of its input symbols differs from the
digest it keeps. The inputs are the code bindings of the capabilities the record
covers, so a record expires when the code it was made against changes, and stays
current through a rebase, a squash merge or a shallow clone, which move commits
and leave the code as it was. The revision a record states is for its readers:
staleness reads no revision and asks Git nothing.

### Conditions to revisit

- Results come to depend on something outside the bound code of their
  capabilities, such as a dependency, the toolchain or the machine they ran on,
  often enough that records stay current while the results they hold have moved.
- Code bindings grow so wide that nearly every edit makes every record stale, so
  records are made again without anyone reading why they moved.
- Readers need to know whether a record holds for a commit other than the working
  tree, such as a release, which the digests of the working tree cannot answer.

## Verification

- A fixture with one claim and one record: the dependency view lists the claim for
  the record and the record for the capability.
- Replacing a passing record with a failing one produces an obligation for the claim.
- Removing a record reports the claim as unsupported, and as changed against the
  merge base.
- A value over its limit by less than floating-point numbers tell apart, compared
  on its decimal text, records fail, and a claim on such a value is unsupported.
- A record whose bound symbol was edited is reported as stale, naming the symbol, and
  one beside an edit of another item or a change of line endings only is not.
- The run of the command: its argv exactly, standard input closed, bounded output, a
  kill at the time limit (recorded as fail), the workspace root as its directory and
  the caller's environment; each fault of the printed values is an error beside a
  passing neighbour, and a command that cannot start writes no record.
- A claim that names an evidence id that does not exist is a finding of `check`
  naming the file and the line of the marker.
- A record that no claim names supports the capability its `for` lists, and the
  views refuse an id that is malformed, a record that does not exist and a
  capability that does not exist or does not load, while the neighbour answers.
- A record whose stored command is shell text is read as text: it reaches no
  process, and its failing verdict makes the claim on it unsupported.
- `--from-binding`: a bound test gives the verdict, and a path that is no test or
  example binding of a capability of `--for` starts nothing.
- Every refusal of the request starts nothing and writes nothing: no capability, a
  capability that does not exist, has no code binding or binds code that does not
  resolve, a limit given twice, a revision that is not a commit id, a slug that is
  not one, one SPDX line only, a replacement of a record that is not there, a limit
  with a binding source. A new record never replaces one, and a replacement keeps
  nothing of the record it replaces. More than 1024 values, and output past its
  bound, are errors that write nothing, while 1024 values and output at the bound
  record. A limit is a name and a number, and a value with an exponent is no
  number.
- The words of the command are every word after the first `--` of the command
  line; a word that is not UTF-8 is refused by its place, and the stored text
  quotes each word a shell would split or expand.
- The documentation examples of `Workplan::record_evidence` (a capability that does
  not exist is refused before anything starts or is written) and
  `evidence::parse_limit` (a limit is a name and a number; an exponent is refused)
  run.
- A directory link inside the root at `work` or at `work/evidence`, with a file of
  the record's name behind it, is refused before the command runs, for a new record
  and for a replacement, with nothing written, beside the same requests through the
  real directories; through the gate, with `work/evidence` a link to
  `docs/capabilities`, a replacement of the evidence named as a capability document
  is refused before anything runs and the document keeps its text.
- An error about the caller's command names it as its record does: the message of
  a kill for too much output and of a program that cannot start (a real run of a
  program named with a space) quote each word a shell would split or expand,
  beside a command of plain words that reads as its words.
- While a second capability document does not load, the claims of a record are
  refused naming it, for a record with claims and for an id no claim names, while
  a malformed id is refused as malformed first; beside it a change record and an
  evidence record that do not load leave the answer, as the clean repository does.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_workplan::Workplan::record_evidence` | doctest | [Verification](#verification) |
| `raf_workplan::evidence::measure::tests::a_limit_is_a_name_and_a_number` | test | [Verification](#verification) |
| `raf_workplan::evidence::measure::tests::a_limit_never_printed_is_an_error_and_a_printed_one_is_judged` | test | [Verification](#verification) |
| `raf_workplan::evidence::measure::tests::a_name_printed_twice_is_an_error_and_two_names_are_not` | test | [Verification](#verification) |
| `raf_workplan::evidence::measure::tests::a_value_is_over_its_limit_by_its_text_and_not_by_its_nearest_float` | test | [Verification](#verification) |
| `raf_workplan::evidence::measure::tests::a_value_that_is_not_a_number_is_an_error_and_its_neighbour_is_not` | test | [Verification](#verification) |
| `raf_workplan::evidence::measure::tests::only_lines_of_a_name_an_equals_sign_and_a_number_are_values` | test | [Verification](#verification) |
| `raf_workplan::evidence::parse_limit` | doctest | [Verification](#verification) |
| `raf_workplan::evidence::tests::a_capability_with_no_code_binding_or_one_that_does_not_resolve_is_refused` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::a_command_that_cannot_start_writes_no_record` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::a_directory_link_on_the_way_to_the_record_is_refused_before_the_command_runs` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::a_new_record_never_replaces_one_and_a_replacement_keeps_nothing_of_it` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::a_passing_command_writes_one_record_with_its_command_values_limits_and_input_digests` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::a_run_killed_at_its_time_limit_records_fail_and_its_neighbour_passes` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::a_value_over_its_limit_by_less_than_a_float_tells_apart_records_fail` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::each_fault_of_the_printed_values_is_an_error_beside_a_passing_neighbour` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::every_refusal_of_the_request_starts_nothing_and_writes_nothing` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::evidence_through_the_gate_never_replaces_a_capability_document_through_a_link` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::more_values_than_a_record_keeps_is_an_error_and_the_most_it_keeps_records` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::output_past_the_bound_is_an_error_and_writes_nothing_and_output_at_it_records` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::the_argv_reaches_the_process_exactly_with_no_shell_of_the_workplan` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::the_command_is_asked_with_the_bounds_of_a_build_and_the_callers_locale` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::the_command_runs_in_the_root_with_its_input_closed_and_the_callers_environment` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::the_exit_status_and_the_limits_decide_the_verdict` | test | [Verification](#verification) |
| `raf_workplan::evidence::tests::the_recorded_line_says_why_a_verdict_is_fail` | test | [Verification](#verification) |
| `raf_workplan::exec::trusted::tests::a_line_with_no_command_is_refused_and_one_with_a_command_is_not` | test | [Verification](#verification) |
| `raf_workplan::exec::trusted::tests::a_word_that_is_not_utf8_is_refused_by_its_place` | test | [Verification](#verification) |
| `raf_workplan::exec::trusted::tests::an_error_about_the_command_quotes_its_words_as_the_record_stores_them` | test | [Verification](#verification) |
| `raf_workplan::exec::trusted::tests::the_command_is_every_word_after_the_first_separator` | test | [Verification](#verification) |
| `raf_workplan::exec::trusted::tests::the_text_quotes_each_word_a_shell_would_split_or_expand` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::a_bound_test_gives_the_verdict_and_a_path_that_names_no_binding_starts_nothing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::a_claim_on_a_value_over_its_limit_by_less_than_a_float_tells_apart_is_unsupported` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::a_record_supports_a_capability_by_its_for_list_without_a_claim` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::a_view_of_what_does_not_exist_is_an_error_and_its_neighbour_answers` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::an_edit_inside_a_bound_module_makes_its_record_stale_and_a_sibling_edit_does_not` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::hostile_record_text_is_read_as_text_and_reaches_no_process` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::the_claims_of_a_record_are_refused_while_a_capability_document_does_not_load` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::the_views_list_the_claims_of_a_record_and_the_records_of_a_capability` | test | [Verification](#verification) |
<!-- raf:generated:end -->
