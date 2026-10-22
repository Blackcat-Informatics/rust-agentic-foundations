<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Scenario fixture repair: bounded metadata handoff

No product files were edited and no build or runtime test was run. Prepared
copies are under `scratch/scenario-fixture-repair`; their patch is
`tasks/queue-specialist-window-story-order.patch`. The current five-file patch
passes `git apply --check` with exit 0. It includes parser assertion extensions
and still needs the implementation owner's review, formatting and actual
scenario qualification. This is an unexecuted repair
proposal, **not** a claim that either complete scenario passes.

## Exact patch contents

- Window Contract now has distinct construction, retention and query clauses,
  each with its actual assertion owners. Usage names the actual bound rolling
  example and explains its fourth-value eviction. Constraints have actual
  capacity, zero-capacity and NaN statements with individual owners.
- A new generated integration test records NaN and a finite value, observes a
  NaN mean, evicts the NaN, then asserts the finite mean and unchanged length.
  The binding list includes this real test. No dependency/counter infrastructure
  or production window behavior is added.
- Allocation and mean-cost limitations remain visibly stated in Explanation,
  alongside their representation rationale. Account and delivered-account
  assertions still require the allocation limitation, now in its actual section;
  observable capacity/NaN constraints are additionally checked. No assertion of
  the no-allocation guarantee is manufactured.
- Each version's existing whole-text parser test now also checks a literal
  CRLF input with duplicate keys and an empty value against the exact expected
  ordered entries. The existing version 4 limit test additionally checks UTF-8
  byte count with `k=é` at four bytes and one more character crossing the limit.
  Existing ordinary-input and refusal assertions remain.
- `story::author_record` accepts an explicit optional reason per item. Its two
  scenario callers supply specific reasons for dropped line-and-column
  diagnostics, character-count limits, and window allocation guarantees.
  Unknown dropped streaming claims panic instead of acquiring generic prose.
- Scenario B's `implement` leaves the built considered/unbound state uncommitted.
  All account/refusal checks still inspect it. The existing first valid commit
  remains the later implementing capability with current bindings and settled
  ownership. `Project::commit` and its clean-gate assertion are unchanged.

## Remaining streaming document work

The patch deliberately does not attach markers to all old prose mechanically.
The implementation owner still needs the following explicit edits in
`support/streaming.rs`. Retain the existing historical wording substrings used by
`WORDING`, final direct-contract sentences, decision rationale, proposal history,
link checks and recoverable five delivered states. Add clause leads before the
existing paragraphs rather than rewriting those pinned sentences gratuitously.

Use `chunk_parser::test_crate::parser::` as the prefix of each owner below.
Markers can list two specific owners for one genuinely combined guarantee;
they must not list the whole test suite as a universal witness.

| Version | Contract clause | Actual owner suffix |
| --- | --- | --- |
| 0 | whole-text ordered entries and valid pair/empty-value shape | a_whole_text_is_parsed_in_order; a_line_that_is_not_a_pair_is_skipped |
| 0 | malformed/nonempty-key refusal by skipping | a_line_that_is_not_a_pair_is_skipped |
| 1 | chunk invariance and final unterminated line | chunks_split_anywhere_give_the_same_entries; a_last_line_without_a_line_ending_is_kept |
| 1 | whole-text convenience and ordered pairs | a_whole_text_is_parsed_in_order |
| 1 | skipped malformed/empty-key lines | a_line_that_is_not_a_pair_is_skipped |
| 2 | chunk invariance and final line | chunks_split_anywhere_give_the_same_entries; a_last_line_without_a_line_ending_is_kept |
| 2 | complete-text convenience and pair/empty-value shape | a_whole_text_is_parsed_in_order |
| 2 | blank line | a_blank_line_is_not_an_entry_and_not_an_error |
| 2 | malformed line/key and one-based error line | a_line_that_is_not_a_pair_is_an_error_that_names_its_line |
| 2 | persistent first error | the_first_error_ends_the_parse |
| 3 | chunk independence and incremental/final output | chunks_split_anywhere_give_the_same_entries; an_entry_is_returned_by_the_chunk_that_completes_its_line |
| 3 | whole-text convenience and pair/empty-value shape | a_whole_text_is_parsed_in_order |
| 3 | blank/malformed lines and persistent error | the corresponding three specific owners used for version 2 |
| 4 | chunk independence and incremental/final output | chunks_split_anywhere_give_the_same_entries; an_entry_is_returned_by_the_chunk_that_completes_its_line |
| 4 | pair/empty-value shape and complete-text input | a_whole_text_is_one_chunk |
| 4 | blank/malformed lines and persistent error | the corresponding three specific owners used for version 2 |
| 4 | byte limit and default limit | a_line_over_the_limit_is_an_error_before_its_line_ending_arrives; the_default_limit_is_sixty_five_thousand_five_hundred_thirty_six_bytes |

Separate combined blank-line/pair and malformed/persistent-error paragraphs when
adding clause leads, so their ownership remains clear. For every version the
new whole-text assertions are the concrete CRLF/duplicate/empty-value owners;
the existing final-line or incremental-output assertion owns finalization.
Use these as individually marked observable Constraints. Keep retained-buffer
representation/memory reasoning visibly in Explanation if its allocation claim
has no executable owner, preserving the existing limit and unbounded-line
rationale. Do not pretend that output equality measures allocated storage.

Usage must name an actual example binding: dump for version 0; chunks or dump
for version 1; chunks for version 2; stream for versions 3 and 4. A function name
in a code fence is not a substitute. Add concrete prose explaining what the
bound example does before the invocation. The short Examples paragraphs in
versions 0 and 3 also need real prose, not generic padding. For example:

> The executable dump example parses two literal pairs and prints each resulting
> key followed by its value on a separate output line. It demonstrates whole-text
> input without requiring the caller to manage incremental state.

This describes V0_EXAMPLES_DUMP's actual two-pair input. Do not claim it skips a
malformed example line: that target contains none. Version 3's explanation can
describe the actual three chunks and the difference between entries returned
by feed and the final unterminated entry returned by close. Check the actual
prose-word rule, excluding code/comments, for all seven sections of each version.

## Transition and refusal invariants still needing execution

`Graph::add_tested_by` validates marker binding membership independently of
delivered-only strict claim/content checks. Scenario A's build phase changes
test bindings while retaining the preceding account. An old skip-test marker
cannot silently be rebound to the new malformed-error test: they prove opposite
policies. The implementing intermediate account must explicitly stop claiming
that removed test still substantiates it, or keep that invalid state uncommitted
and inspect its actual findings until the account is settled. Preserve proposal,
history and refusal assertions; do not alter `Project::commit` or filter findings.
The fully settled/delivered five versions require their exact current owners.

Scenario B's considered build, renamed candidate bindings, partly settled prose,
THIN/empty text, failing example and dishonest guarantee must all remain genuine
negative states. The stronger policy can add real missing-clause/content/claim
findings to their exact refusal sets. Inspect those findings after the fixture
accounts are repaired; update precise expectations with actual owners rather
than weakening assertions or excluding the new policy. The unchanged final
account must still pass every original source/API/usage/matrix/history check.

No orphan ceiling, debt, broad exclusion, gate or generated-cleanliness
assertion is relaxed by this proposal. The full two-scenario runtime acceptance
and actual lifecycle execution remain required after the owner finishes and
qualifies these authored fixture changes.
