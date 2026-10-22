<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Evidence claim ownership map

Read-only assertion inspection of the current evidence document, evidence unit
and integration tests, measure/trusted tests, runner implementation/tests and
record publication paths. No build or test was run. Names below are inspected
owners or explicitly labelled proposed proof, not execution claims. Keep real
limitations in Constraints. Contract heading markers cover their named clause;
Constraints markers must follow the actual paragraph/list item they support.

## Exact path prefixes

- `EU`: `raf_workplan::evidence::tests::` (`src/evidence/tests.rs`).
- `EI`: `raf_workplan::test_crate::evidence::` (`tests/evidence.rs`).
- `M`: `raf_workplan::evidence::measure::tests::` (`src/evidence/measure.rs`).
- `T`: `raf_workplan::exec::trusted::tests::` (`src/exec/trusted.rs`).
- `S`: `raf_workplan::exec::system::tests::` (`src/exec/system/tests.rs`).

Expand these prefixes for actual markers. Several inspected EI/S/OKF tests
below are not currently bound by this capability; they require explicit binding
before marker use. Existing generated verification rows are not substitutes for
those bindings.

## Four Contract groups: meaningful splits

Split the current large Evidence group at the actual behavioral boundaries,
without dropping any argv, number, refusal, prerequisite or write guarantee.

| Proposed clause and insertion anchor | Inspected owners and assertions |
| --- | --- |
| Execution record fields and storage: opening Evidence paragraph, ending before command-source bullet | EU `a_passing_command_writes_one_record_with_its_command_values_limits_and_input_digests` checks exact path, typed subject/verdict/command/revision/values/limits/covers/by/input name and digest; file snapshot proves exactly one added record and every old file unchanged, noise absent. |
| Trusted command execution: command-source bullet through environment/bounds and status/signal verdict | EU `the_argv_reaches_the_process_exactly_with_no_shell_of_the_workplan` checks NUL-separated hostile argv and no shell side effect; EU `the_command_runs_in_the_root_with_its_input_closed_and_the_callers_environment` checks cwd, EOF and LC_ALL/PATH lengths; EU `the_command_is_asked_with_the_bounds_of_a_build_and_the_callers_locale` checks exact Trusted invocation, no injected Cargo flag, limits 16 MiB and BUILD_TIMEOUT and c_locale false; EU `the_exit_status_and_the_limits_decide_the_verdict` checks zero, nonzero, signal and combined failures. Environment test is representative, not enumeration of every variable; source has no env_clear or trusted env override. |
| Measurement grammar and exact limits: from measured values through decimal comparison and stored values | M `only_lines_of_a_name_an_equals_sign_and_a_number_are_values`, `a_value_that_is_not_a_number_is_an_error_and_its_neighbour_is_not`, `a_name_printed_twice_is_an_error_and_two_names_are_not`, `a_limit_never_printed_is_an_error_and_a_printed_one_is_judged`, `a_value_is_over_its_limit_by_its_text_and_not_by_its_nearest_float`, `a_limit_is_a_name_and_a_number`. Actual vectors include discarded indentation/spaces, CRLF, signed values, exponent/units/NaN refusals, case-distinct names, missing limit, equal/under/over and huge decimal neighbors. EU exact-value test additionally checks actual record verdicts. |
| Timed-out result: timeout/no-value sentences | EU `a_run_killed_at_its_time_limit_records_fail_and_its_neighbour_passes` is **FakeRunner** elapsed/timeout evidence: fail, Failed::TimedOut, no values, retained requested limits, equal-bound success. S `a_process_that_outlives_its_limit_is_killed` is actual runner kill proof using stepped clock; include both if asserting end-to-end composition. Do not call the EU test a real process kill. |
| Binding verdict source: from-binding bullet | EI `a_bound_test_gives_the_verdict_and_a_path_that_names_no_binding_starts_nothing` runs real passing/failing tests via SystemRunner, checks subject, no command, digest, by, fail and unchanged docs; bad paths use FakeRunner and assert zero calls. EU `every_refusal_of_the_request_starts_nothing_and_writes_nothing` includes limits-with-binding. The EI test does not execute all example kinds; behavior delegates to actual Executor. For each example kind claimed, inspect/bind its existing executor assertion owner before claiming that proof. |
| Run errors and no publication: paragraph beginning Nothing is written | EU `each_fault_of_the_printed_values_is_an_error_beside_a_passing_neighbour`, `more_values_than_a_record_keeps_is_an_error_and_the_most_it_keeps_records`, `output_past_the_bound_is_an_error_and_writes_nothing_and_output_at_it_records`, `a_command_that_cannot_start_writes_no_record`. These check exact error variants, no record/snapshot preservation, 1024 versus 1025, output at versus past bound and good neighbors. Exit 2 is a **CLI** property, not established by library errors: attach an inspected command caller assertion before marking that portion. |
| Request prerequisites: paragraph beginning Before anything runs, through by | EU `every_refusal_of_the_request_starts_nothing_and_writes_nothing` checks unknown/no capability, duplicate limit, bad revision/slug, half SPDX, absent replace, binding source limits/path; zero fake calls and identical snapshots. EU `a_capability_with_no_code_binding_or_one_that_does_not_resolve_is_refused` checks empty, missing and mixed bindings and sorted successful two inputs. Loaded-capability refusal and the exact 40/64 lowercase revision boundaries need precise existing assertions or new boundary neighbors; arbitrary bad `HEAD` alone does not prove both accepted lengths. Actor enum/schema owns human/agent closure, not the passing-command test alone. |
| Exclusive creation and replacement: final Evidence paragraph from A new record | EU `a_new_record_never_replaces_one_and_a_replacement_keeps_nothing_of_it` checks refusal before run, old bytes retained, replacement values and no inherited limits. Passing-command snapshot owns exactly-one-file publication, subject to explicit path-swap limitation below. |
| Claim syntax, explicit dependency and relationship checks: Claims group | EI `the_views_list_the_claims_of_a_record_and_the_records_of_a_capability` checks exact paragraph/list statement, line/section, multi-record marker, slug/prefixed equivalence, missing named evidence still a dependency, list-for edges and verdicts. Add actual `records::a_claim_on_unknown_evidence_names_the_file_and_the_line`, `a_claim_on_evidence_for_another_capability_is_a_finding`, `a_record_that_names_what_does_not_exist_is_a_finding_and_its_neighbour_is_clean` and `a_claim_inside_a_proposed_block_is_not_a_claim_until_it_is_resolved` for relationship/proposal facts. Last proves proposed exclusion; actual resolution transition is not directly exercised there. Syntax itself belongs to OKF marker parser tests, not a view test. |
| Dependency views and unloaded document refusal: Dependency view group | EI `the_views_list_the_claims_of_a_record_and_the_records_of_a_capability`, `a_record_supports_a_capability_by_its_for_list_without_a_claim`, `a_view_of_what_does_not_exist_is_an_error_and_its_neighbour_answers`, `the_claims_of_a_record_are_refused_while_a_capability_document_does_not_load`. Last checks exact UnloadedCapabilities, bad-id precedence and clean/broken-change/broken-evidence neighbors. Exit 2 still needs actual CLI assertions. |
| Merge-base invalidation: first Invalidation sentence | EI `each_change_of_a_passing_record_since_the_merge_base_reopens_its_claim` checks failed verdict, over old limit after loosening current limit, missing old limit, removal, committed changes and restored passing neighbor, plus claim removal. EI `a_record_that_failed_at_the_merge_base_or_was_not_there_is_compared_with_nothing` and `a_record_at_the_merge_base_that_does_not_read_is_an_error_and_not_a_pass` own base negatives. These are currently not bound here. |
| Unsupported/current-source evidence: remaining Invalidation sentences | EI `a_claim_on_a_value_over_its_limit_by_less_than_a_float_tells_apart_is_unsupported`, hostile failed-record test, merge-base removal test; EI `an_edit_of_a_bound_symbol_makes_its_record_stale_and_nothing_else_does` and bound module test own exact symbol/descendant edits and unchanged sibling/line-ending neighbors. Symbol edit test is inspected but currently unbound here. Missing symbol staleness needs the actual assertion; edited symbol alone does not prove unresolved input behavior. |

## Twelve Constraints, in current order

Additional actual CLI assertion inspection closes several candidate-name gaps
in the Contract table. Prefix `cargo_workplan::test_crate::evidence::`:

- `output_the_tool_cannot_read_is_exit_two_and_writes_no_record` actually checks
  exit 2, empty stdout and unchanged snapshots for exponent, duplicate value,
  missing limited value, real 16 MiB overflow and missing program, then the
  successful numeric neighbor. It does not include 1025 output values.
- `a_request_that_cannot_be_recorded_runs_nothing` checks exit 2, no command
  marker and unchanged snapshots for unknown/no-code capability, absent update,
  binding limit/path refusal, then existing-record preservation. It does not
  replace the full library prerequisite table.
- `claims_exit_two_naming_a_capability_that_does_not_load_and_answer_beside_other_records`
  checks actual exit 2 for claimed/prefixed/unclaimed ids, empty output, named
  unloaded capability and unchanged snapshot, with real clean/broken-change/
  broken-evidence command neighbors.
- `the_views_print_rdf_and_refuse_what_does_not_exist` checks actual typed RDF
  fields, unknown-id exit 2 and unchanged snapshot. These can own the specified
  CLI statuses when explicitly bound here; they are not inferred from EU/EI.

1. **Result without interpretation.** Passing-command test proves rendered result
   with no retained stdout noise, not universal semantic absence of interpretation
   in arbitrary strings. `raf_okf::test_crate::records::a_record_has_no_title_description_or_body`
   directly refuses actual prose bodies/title and accepts blank/comment neighbor
   for Evidence (also other record types). Bind that owner and state the enforceable
   no-body/closed-schema boundary. Calling a result “not a judgement” is the
   authoring meaning of those typed fields, not a prose-truth detector.
2. **Explicit claims.** EI views test plus support-without-claim test distinguish
   explicit marker dependencies from plain support. Add a neighboring identical
   ordinary paragraph that names the evidence in prose but has no marker if
   direct proof of “no dependency guessed from wording” is required. No arbitrary
   graph test should be attached as though it inspects textual inference.
3. **No capability edit.** EU passing-command snapshot checks every old file
   unchanged; EI actual binding test compares doc snapshots across pass/fail.
   These are meaningful command/binding owners for one-file recording.
4. **Staleness independent of revision/Git.** EI symbol edit and module edit tests
   run `Workplan::reconcile` on MemoryTree source, without Lifecycle/Git; exact
   edits/sibling/CRLF neighbors are real. They do not execute shallow/squash history.
   Add a request with differing arbitrary valid revisions and a runner that would
   fail on any Git call, comparing identical/stale results, if explicit no-Git
   ownership is needed. Keep record-currentness distinct from merge-base change
   invalidation, which genuinely uses Git.
5. **Stored command never executes.** EI hostile-record test loads shell text and
   calls non-executing `Workplan::reconcile`, checking unsupported fail; it has no
   process receipt or filesystem sentinel assertion. T separator/UTF8 tests prove
   provenance of actual TrustedArgv and no earlier record field is used. Actual
   binding test proves fixed filter path route. For stronger direct no-process
   proof, use actual Gate/Lifecycle caller with empty call receipts or fail-on-any
   runner, plus a hostile command sentinel, while retaining its ordinary neighbor.
6. **Leader exit plus pipe EOF.** S
   `a_descendant_that_holds_the_pipes_after_the_exit_is_not_waited_for_past_the_limit`
   actually leaves a pipe holder after leader exit, asserts Timeout and on Linux
   process cleanup via handshake/pid observations. S `a_process_inside_its_limit_passes`
   is the real completed neighbor. Combine with EU timeout-to-fail/no-values owner;
   no single test currently proves the whole evidence publication chain here.
7. **Real-directory preflight.** EU
   `a_directory_link_on_the_way_to_the_record_is_refused_before_the_command_runs`
   exercises work/work-evidence internal links in both Create/Replace, zero calls
   and unchanged snapshot, then real-directory success. Gate document-link test
   preserves capability bytes. Neither executes the binding source through the
   linked preflight; production target_fits is shared before both branches.
   A regular-file directory blocker is asserted by record-writer tests, not the
   linked-directory evidence test. Bind that exact owner or add its zero-run
   evidence counterpart; do not silently claim all three from one symlink case.
8. **Supervisor lifetime/group escape/platform limit.** Genuine proof gap: current
   S tests prove in-group descendants killed, no external kill binary needed,
   leader-exited kill handling and runner precedence. They do not kill the
   supervising cargo-workplan, exercise setsid/setpgid escape or execute non-Unix
   process-only behavior. See deterministic proof seams below. Keep this item.
9. **Root program resolution/caller directory/PATH dot.** EU cwd test proves
   process cwd, not a relative program launched from a nested caller or bare-name
   PATH lookup. `SystemRunner::program_path` and Command::current_dir back the
   intended rule; add an actual tiny root tool plus nested-caller and root-dot-PATH
   neighbors in an isolated child process (do not mutate global process cwd/PATH).
   Preserve exact stored argv comparison. Do not attach cwd-only test to all.
10. **Command quoting in diagnostics.** T
    `an_error_about_the_command_quotes_its_words_as_the_record_stores_them`
    distinguishes sh -c argument groupings, checks output-bound message prefix,
    and actually runs a missing program containing spaces with exact stored text.
    T `the_text_quotes_each_word_a_shell_would_split_or_expand` checks spaces,
    empty word, apostrophe, dollar and semicolon. These are specific owners.
11. **Writer max versus manually authored records.** EU MAX_VALUES test proves
    exactly 1024 records and 1025 refusal through FakeRunner. It does not load a
    manually authored 1025-value record. Add such a literal record with typed
    count assertion and a bounded oversized-document refusal neighbor using
    the actual public loader. Retain ordinary schema/reader limits; “bounded
    only by 1 MiB” should not imply malformed/deep YAML is accepted.
12. **Path-swap limitation.** Source inspection confirms files::target returns
    metadata, not held directory handles. A second path-based check occurs at
    final publication: swapping during Runner::run tests this recheck, not the
    narrower post-final-check/open race. No existing test executes that race.
    Keep the limitation and use the genuine production phase seam below.

## Deterministic proof designs for real gaps

**Publication seam.** `files::create_new` already has a natural phase boundary
after `target` and existing-kind refusal, before create_dir_all/OpenOptions.
Extract the unchanged remaining path-based publication into a private helper
used by production, with an explicit already-checked target/root/path/text
input. A test obtains actual `target` on a real existing directory, replaces
that directory with an internal symlink, invokes that exact helper and proves
the file lands behind the link; unchanged-directory neighbor proves normal
exclusive publication and contents. No timing, injected hook, fake failure or
new public API is needed. Do not recheck inside this helper solely for the test:
that would change the documented limitation rather than prove it. A separate
existing-file neighbor must retain exclusive creation refusal.

For `record::replace`, the analogous existing boundary is after
`files::target` confirms regular file, before canonicalize-root/join,
`sync::stage` and rename. A private production-used post-preflight replacement
helper can take the already computed RelPath and rendered text; use an
in-root alternate directory with an existing record of the same filename.
After the real preflight, swap the parent to a link, call the exact helper,
and assert the alternate record receives the replacement and ordinary old
directory contents are unchanged. Preserve temp cleanup and unchanged-real-path
neighbor. Both seams document path trust; neither is a security-hardening proposal.

**Supervisor death.** Launch actual cargo-workplan evidence recording in a
dedicated child process and a tiny command helper. FIFO/pipe receipts report
command pid/pgid and hold it before completion. Once receipts prove separate
group and live supervisor, send an ordinary OS signal to the supervisor,
collect its exit, release the helper and observe its surviving marker through
the receipt. Assert no evidence was written. Always own the helper pid in a
cleanup guard. A normal supervisor/completed command neighbor writes one record.
No elapsed-time assertion or sleep-based race is required; bounded operation
receipts and OS wait/pid handles should govern progress. Do not signal the test
runner's own group or rely on the developer terminal to generate Ctrl-C.

**Escaped descendant.** Extend the real SystemRunner's existing FIFO/stepped-clock
test apparatus with a helper that performs setsid or setpgid, acknowledges its
distinct group and blocks on an owned receipt before clock advancement. At the
runner's admitted limit assert ordinary group members are gone but escaped pid
is alive, then release/kill and collect that pid explicitly in a cleanup guard.
Keep the current ordinary same-group cleanup test as the neighboring valid
behavior. This demonstrates escape from the kill domain, not unlimited actual
wall-clock waiting. Native non-Unix behavior needs its own platform execution;
Unix source inspection/cross compilation must remain labelled as such.

All suggested tests are preparation only. No new execution, candidate count,
assessment acceptance, lifecycle completion or full repository gate is claimed.
