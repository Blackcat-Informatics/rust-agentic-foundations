# Timing conversions

## Task 1: raf-okf (commit d5793a1)

Each mutant was applied in a scratch clone (`git clone --no-hardlinks` into a fresh /tmp directory), never in the worktree, with no `timeout` wrapper. Test names are in `raf_okf::profile::cost_tests`.

| Mutant | Test that fails | Observed failure |
| --- | --- | --- |
| `listed` set built inside the affects loop | `a_change_at_the_cap_judges_each_affects_entry_once_against_one_set` | `ChangeIdsSet work passed the ceiling of 27452: 54904` (second entry) |
| `let first = 0` in `visible_content` | `a_capability_at_the_cap_looks_at_each_block_once` and `the_blocks_looked_at_grow_with_sections_and_blocks_together` | `BlocksVisited work passed the ceiling of 10000: 10001`; the growth test panics at the slice in `visible_content` (a block before the section) |
| `break` removed from `visible_content` (replaced by `continue`) | `the_blocks_looked_at_grow_with_sections_and_blocks_together` | `BlocksVisited work passed the ceiling of 64000: 64001` |
| `by_name` built inside the limits loop | `an_evidence_record_at_the_cap_indexes_each_value_once_and_looks_up_each_limit_once` | `MeasuresIndexed work passed the ceiling of 44605: 44606` (second limit) |
| `listed` index deleted, `changes.iter().any(..)` per entry | `a_change_at_the_cap_judges_each_affects_entry_once_against_one_set` | `ChangeIdsSet` reads 0, expected 27452 (the quadratic scan itself runs to its end first) |
| `by_name` index deleted, `values.iter().find(..)` per limit | `an_evidence_record_at_the_cap_indexes_each_value_once_and_looks_up_each_limit_once` | `MeasuresIndexed` reads 0, expected 44605 (the quadratic search runs to its end first) |
| fence scan restarted from the page start for each line | `the_lines_tested_for_a_fence_grow_with_the_length_of_the_page`, `an_unterminated_fence_is_tested_once_for_each_line_it_swallows` | `FenceScans work passed the ceiling of 64000: 64001` and `of 5001: 5002` |

Notes. The two deleted-index mutants have no counted step of their own, so the index-building counter reads 0 and the exact count fails after the scan finishes; the scan is the mutant's own cost. The tests that parse a document at the cap outside a measure run the fence-restart mutant to its end (quadratic over 100,000 lines), so that mutant was run with the two fence tests only. A loop added elsewhere than the counted steps is not seen by a count (the limit stated in the okf-profile Verification text).

### Task 1, ticks moved to where the work is done

The `ChangeIdsSet` tick now sits on the iterator that inserts each id (`.inspect`), and a `ListedLookups` tick sits at the single lookup of `check_affects`; the cap test pins `ChangeIdsSet`, `AffectsEntries` and `ListedLookups` each to the number of entries. The `by_name` build already ticks `MeasuresIndexed` per value inserted (`.inspect`) and `LimitLookups` sits at its lookup; `visible_content` ticks per block visited.

| Mutant | Test | Observed failure |
| --- | --- | --- |
| `listed` set built inside the affects loop | `a_change_at_the_cap_judges_each_affects_entry_once_against_one_set` | `ChangeIdsSet work passed the ceiling of 27452: 27453` (fails at once) |
| set deleted, `changes.iter().any(..)` per entry (the lookup rewritten) | same | `ChangeIdsSet` reads 0, expected 27452 (the quadratic scan runs first) |

Limits of the count: a mutant that keeps the set built and ticking but never uses it is outside what the build tally sees; the lookup tick covers the path through the single lookup site. The growth test of `visible_content` under `let first = 0` alone fails on the slice-range panic in `visible_content`, not on its growth assertion; the cap test fails it on the `BlocksVisited` ceiling.

## Task 2: raf-workplan algorithmic bounds (commits 2e51549, defd0e2, 409779d, 3f0a34b)

Each mutant was applied in a scratch clone (`git clone --no-hardlinks` into a fresh /tmp directory) holding the final tree, never in the worktree, with no `timeout` wrapper, running only the named test (`cargo test -p raf-workplan --locked --lib <name>`). Tests are in `raf_workplan::{graph,rationale,assessment,obligation}::tests`.

| Mutant | Test that fails | Observed failure |
| --- | --- | --- |
| merged ranges rebuilt per character in `prose_of` | `graph::tests::a_section_of_many_blocks_is_read_in_work_linear_in_its_blocks` | `BlankedSteps work passed the ceiling of 1000000: 1000001` |
| `loaded`/`covered` sets rebuilt for each claim | `graph::tests::the_claims_of_many_capabilities_are_judged_against_sets_built_once` | `ClaimChecks work passed the ceiling of 6015: 6016` |
| `let first = 0` in `has_item` | `rationale::tests::empty_conditions_headings_are_each_judged_against_their_own_lines`, `...::the_decisions_of_an_explanation_are_each_judged_against_their_own_lines` | `ProseLinesScanned work passed the ceiling of 2010: 2011` (both) |
| `extent` finding with `<` instead of `<=` | the same two | `ProseLinesScanned ... ceiling of 2010: 2011` (page) and `ConditionsScan ... ceiling of 2010: 2011` (Explanation) |
| headings read again for each heading in `decision_page` | `rationale::tests::a_hundred_thousand_headings_are_each_read_once` | `HeadingsScanned work passed the ceiling of 200000: 200001` |
| `any_strong` computed in the per-path closure | `assessment::tests::the_bindings_are_tested_against_the_names_once_to_find_a_strong_one_and_once_to_name_them` and `...::a_long_contract_is_read_once_and_its_symbols_are_named_in_one_pass` | `NamingLookups work passed the ceiling of 4010: 4011`; the second test fails its exact count (4002000 against 4000) |
| Contract rescanned from its first line for each line | `assessment::tests::a_long_contract_is_read_once_and_its_symbols_are_named_in_one_pass` | `ContractLines work passed the ceiling of 12100: 12101` |
| `map_or(text.len(), ..)` changed to `map_or(next, ..)` | `obligation::tests::hostile_text_is_read_in_linear_time` | `ScanBytes work passed the ceiling of 480000: 519856` |
| `known` built inside the item loop | `obligation::tests::moves_to_unknown_records_cost_the_same_with_few_records_as_with_many` | `SlugsListed work passed the ceiling of 9010: 9011` |

Notes. `Workplan::load` runs the rules of current rationale, so the rationale tests measure the load; a mutant in a rule that the test measured outside the load would otherwise run unbounded (found when the first form of the headings test hung on its mutant, and corrected before the table above). The `any_strong` mutant is also caught by the pipeline test through the exact count only, since a `measure_kind` limits one kind and that test limits `ContractLines`.

## Timing conversions / Task 3: coverage

Decision: the product entries x exclusions is pinned exactly (`Work::EntriesJudged`, ticked first in `Judge::judge`; `Work::ExclusionTests`, ticked per `matches` call in `exclusion_for`). Tests are in `raf_workplan::coverage::tests`. Each mutant was applied in a scratch clone (`git clone --no-hardlinks` into a fresh /tmp directory) holding the final tree, never in the worktree, with no `timeout` wrapper, running only `cargo test -p raf-workplan --locked --lib coverage::tests::`.

| Mutant | Tests that fail | Observed failure |
| --- | --- | --- |
| `exclusion_for` called once per path of `matching` instead of once per entry | the exact product, and both growth tests | `ExclusionTests work passed the ceiling of 4920: 4921` (also 13800 and 6800 in the growth tests) |
| `exclusion_for` called a second time when the entry is not covered | the same three | the same ceilings, 4921, 13801, 6801 |
| `break` after the first match in `exclusion_for` | the same three, by the exact count | `left: 4485, right: 4920`; `12030` against `13800`; `6610` against `6800` |

## Timing conversions / Task 2 (fix)

Scratch clone in a fresh /tmp directory, one test per run, no `timeout` wrapper. Sets that the algorithms look into are now `Counted` (work.rs): every `contains` ticks a kind of work, so a lookup that bypasses the set is not counted and the exact count falls.

| Mutant | Test | Observed |
| --- | --- | --- |
| M1 `known` as a Vec scanned with `iter().any` per item | `obligation::tests::moves_to_unknown_records_...` | `SlugLookups` 8000, expected 16000 |
| M2 `partition_point` replaced by `position` (uncounted) | `rationale::tests::empty_conditions_...`, `...::the_decisions_...` | `RangeStarts` reads 0, `0 < starts` fails |
| M2b `position` keeping the counted predicate | the same two | `RangeStarts work passed the ceiling of 15010: 15011` and `16010: 16011` |
| M3 `loaded` as a Vec scanned per slug | `graph::tests::the_claims_...` | `ClaimLookups` 6000, expected 12000 |
| M3b `covered` as a Vec | the same | 8000, expected 12000 |
| ranges rebuilt per character | `graph::tests::a_section_of_many_blocks_...` | `BlankedSteps` ceiling 1000000 |
| sets rebuilt per claim | `graph::tests::the_claims_...` | `ClaimChecks` ceiling 12015 |
| `let first = 0` in `has_item` | rationale pair | `ProseLinesScanned` ceiling 15010; the Explanation test fails its exact count |
| `extent` with `<` | rationale pair | `ProseLinesScanned` ceiling 15010; `ConditionsScan` ceiling 16010 |
| headings read again per heading | `rationale::tests::a_hundred_thousand_headings_...` | `HeadingsScanned` ceiling 200000 |
| `any_strong` in the closure | `assessment::tests::the_bindings_are_tested_...`, `...::a_long_contract_...` | `NamedLookups` ceiling 12010; exact `NamingLookups` 4002000 against 4000 |
| Contract rescanned per line | `assessment::tests::a_long_contract_...` | `ContractLines` ceiling 12100 |
| paragraph re-read per line (`code_spans`) | the same | `ParagraphBytes` 4681170000, expected 780000 |
| `map_or(next, ..)` | `obligation::tests::hostile_text_is_read_in_linear_time` | `ScanBytes` ceiling 480000 |
| `known` built in the item loop | `moves_to_unknown_records_...` | `SlugsListed` ceiling 17010 |

## Timing conversions / Task 4: the runner clock seam

Refactor, no behaviour change: `exec/clock.rs` holds the only clock reading (`Clock`, `WallClock`); `SystemRunner::run` calls `run_with(.., WallClock::start)`; the loop's decisions are the pure `verdict` and `next_pause`. Tests are in `raf_workplan::exec::system::tests`. Each mutant was applied in a scratch clone (`git clone --no-hardlinks` into a fresh /tmp directory) holding the final tree, never in the worktree, with no `timeout` wrapper, running only the one named test.

| Mutant | Test that fails | Observed failure |
| --- | --- | --- |
| `verdict` with done and late swapped | `the_verdict_follows_its_precedence` | done, elapsed 100 ms: left `Late`, right `Done` |
| `elapsed >= timeout` changed to `>` | the same | not done, elapsed 100 ms: left `Again`, right `Late` |
| overflow not beating done | the same | out overflow and done: left `Done`, right `Over(Stdout)` |
| stderr checked before stdout | the same | both overflow: left `Over(Stderr)`, right `Over(Stdout)` |
| `next_pause` without the ceiling | `the_pauses_double_up_to_the_ceiling_and_stay_there` | `[1, 2, 4, 8, 16, 32, 64, 128]` ms against `[1, 2, 4, 8, 16, 20, 20, 20]` ms |
| Capture `length >= bound` | `the_capture_stops_one_byte_past_its_bound_and_flags_the_overflow` | the 1000-byte reader is flagged |
| Capture reads `bound` bytes, not `bound + 1` | the same | kept 1000, expected 1001 |

## Timing conversions / Task 5: runner kill tests under test clocks

Tests are in `raf_workplan::exec::system::tests`: `Frozen` (fixed reading, each pause counted and yielded, ceiling 5,000,000 pauses) and `Stepped` (moves by each pause, ceiling 200; the 2 s limit passes on pause 104; on Linux its first pause reads the pid FIFO and takes a pidfd of each process before the limit can pass). Subjects are `/bin/sh -c` trusted invocations that write their pids to a pid FIFO and block reading a hold FIFO (both made with `mknodat` and opened `O_RDWR` before the spawn); a leader that exits first reads a third FIFO (go), written by the gate after the pidfds are taken, so the leader is never reaped before its pidfd exists. After the run: `waitid(P_PIDFD, leader, WEXITED|WNOHANG)` must be ECHILD, the hold is released, each pidfd is polled until the process ended, then the marker must be absent and `/proc` shows each process gone, Z/X or dying of SIGKILL. Each mutant was applied to `crates/raf-workplan/src/exec/system.rs` in a scratch clone (`git clone --no-hardlinks` into a fresh /tmp directory, the working diff applied), never in the worktree, with no `timeout` wrapper, running only the named test (`cargo test -p raf-workplan --locked --lib -- --exact exec::system::tests::<name>`); every run failed at once, by an assertion or a ceiling, never by a hang, and no subject process was left (`ps` checked after each batch).

| Mutant | Test | Observed failure |
| --- | --- | --- |
| late branch removed from `verdict` | `a_process_that_outlives_its_limit_is_killed` | `the run paused 201 times, past its ceiling of 200, with the clock at 2s` |
| `pause.min(remaining)` changed to `pause` | the same | `elapsed: 2.011s` against `elapsed: 2s` |
| `abandon` removed after the wait (`Err(failure)`) | the same | `the leader was not reaped: Settled { leader: Dead('Z'), leader_reaped: false, marker: true }` |
| `terminate` returns `Ok(())` without a kill | the same | the same |
| `kill_group` dropped from `terminate` | `what_the_process_started_does_not_outlive_the_kill` | `a process ran on past the kill and wrote the marker: Settled { leader: Gone, descendant: Some(Dead('Z')), leader_reaped: true, marker: true }` |
| the same | `a_descendant_that_holds_the_pipes_after_the_exit_is_not_waited_for_past_the_limit` | the same |
| the same | `the_group_is_killed_when_no_kill_program_is_on_the_path` | the inner run fails the same way; the outer fails on its exit status |
| `elapsed: Duration::ZERO` for a finished run | `a_process_inside_its_limit_passes` | `left: 0ns, right: 7ms` |
| `elapsed: WallClock::start().elapsed()` (another reading) | the same | `left: 350ns, right: 7ms` |
| `kill_group` through an external `kill -s KILL -- -<pgid>` program | `the_group_is_killed_when_no_kill_program_is_on_the_path` | the inner run gives `Io { .. could not be killed .. No such file or directory }` instead of `Timeout` |
| the same | `what_the_process_started_does_not_outlive_the_kill` | passes, as expected: `kill` is on the `PATH` there, which is why the empty-`PATH` test exists |
| `length > bound` changed to `>=` in `Capture` | `output_at_the_bound_passes_and_one_byte_over_is_refused_on_each_stream` | the 1000-byte run is refused (`wrote more than 1000 bytes to standard output`) |
| `done` on the status alone | `a_descendant_that_holds_the_pipes_after_the_exit_is_not_waited_for_past_the_limit` | `left: Ok(Output { code: Some(0), stdout: [], .. })` against `Timeout` |
| overflow flag never read (`overflowed` returns `false`) | `a_process_that_never_stops_writing_is_killed_at_the_bound` | `killed at the bound: Output { code: None, .. }` (`yes` dies of SIGPIPE once the capture stops) |
| `elapsed: Duration::ZERO` for an output kill | `output_at_the_bound_...`, `a_process_that_never_stops_writing_...` | `elapsed: 0ns` against `1.5s` and `3s` |

Stability: `exec::` lib tests run 100 times sequentially and 600 times in six concurrent loops with no failure after two races were removed: (1) the exiting leader of the `Leaves` shape could be reaped by the runner's first look before the gate took its pidfd (`pidfd_open` ESRCH, 15 of 60 runs); the go FIFO removes it; (2) a `/proc` snapshot taken right after the kill can show a dying process in state R with no SIGKILL bit (seen once in about 400 runs), so `/proc` is read only after the pidfd reported the end and the marker carries the verdict. `exec::tests::no_shell::every_argument_is_one_element_and_no_shell_reads_it` failed once with ETXTBSY under the extra concurrent spawns (a process started by another test inherited the stub's write descriptor); the stub is now written by a child process, so this process never holds it open for writing.

Pause counts seen under `Frozen` (counts, not times): 1 to about 14,000 per run on this host, against the ceiling of 5,000,000.

## Timing conversions / Task 6: the evidence kill test under a scripted runner

`evidence::tests::a_run_killed_at_its_time_limit_records_fail_and_its_neighbour_passes` now scripts the runner: `/bin/sh -c "echo peak=1"` answers with an elapsed of `BUILD_TIMEOUT + 1 ms` (the fake applies the asked limits, so the run is a `Timeout`) and, for the neighbour, an elapsed of exactly `BUILD_TIMEOUT`. It asserts `Fail`, `[Failed::TimedOut]`, no values, the limit text `peak: 5` kept in the value and in the record text, the asked limits equal to `Limits::trusted()`, and a `Pass` with `peak=1` at the limit. No process starts, no clock is read and nothing sleeps. Each mutant was applied in a scratch clone (`git clone --no-hardlinks` into a fresh /tmp directory), running only this test (`cargo test -p raf-workplan --locked --lib -- --exact ...`).

| Mutant | Observed failure |
| --- | --- |
| `Timeout` no longer mapped to a `fail` record (an error instead) | the first record fails at once: `a killed run is recorded` |
| limit text dropped from a record with no values | `left: []` against `[Measure { name: "peak", .. }]` |
| the executor asks `Limits::trusted()` with a 48 h timeout | `left: Pass, right: Fail` |
| fake treats `elapsed == timeout` as a timeout (`>=`) | the neighbour at the limit: `left: Fail, right: Pass` |

## Timing conversions / Task 8: budgets, nonces, the preview snapshot and the root test

Removed: `BUDGET`, `within_budget`, `elapsed` and the `started` field from `crates/raf-workplan/tests/{complete,verify,scenarios}.rs` and `tests/support/project.rs`, and `BUDGET` and `timed` from `crates/cargo-workplan/tests/{complete,verify}.rs`; the module docs that spoke of a budget are reworded. A wall-clock budget is a timing assertion and it ran after the run returned, so it never caught a hang; no count replaces it (declined in the plan). `tests/bound.rs` keeps its own fixture and is converted by its own task. The scratch names of `root.rs`, `git.rs` and `gate.rs` are the process id and a counter, and a directory that already exists is skipped (`create_dir` retried on `AlreadyExists`); `exec.rs` was already free of a clock.

`preview.rs`: `snapshot` is one iterative walk returning path -> (bytes, stamp) for files and directories, the stamp being the debug text of the modification time, so no clock type is named. `age` first moves every file's stamp an hour into the past (the time is derived from the file's own stamp), so a write of the same bytes within the same file-system tick still changes the stamp. The guarantee kept: preview writes nothing, including no touch.

`tests/docs_contract.rs`: the scratch name is the process id and a counter; the FIFO test is now `refuses_non_regular_files_instead_of_reading_them` with `/dev/null` and a directory named like a document as the subjects, asserting that the refusal is the one of the inspection before `open` (`... is not a regular file`), as against the check on the opened handle (`... is not a regular file once opened`). No FIFO is opened. A loader that dropped the inspection would open a FIFO and block; for `/dev/null` and a directory, which open without blocking, the changed text of the second check is what exposes it, so the guarantee is kept without a timer and nothing is lost.

Mutants, applied in a scratch clone (`git clone --no-hardlinks` into a fresh /tmp directory, the working diff applied), running only the named test:

| Mutant | Test | Observed failure |
| --- | --- | --- |
| `Workplan::preview` sets the modification time of a document under the root to now | `preview::a_disk_fixture_is_byte_identical_afterwards` | `assertion left == right failed` at the comparison of the two snapshots |
| `Workplan::preview` rewrites a document with its own bytes | the same | the same (the aged stamp differs) |
| `read_regular` accepts a directory at both guards (`is_file() \|\| is_dir()`) | `docs_contract::refuses_non_regular_files_instead_of_reading_them` | `folder.md: Err("cannot read ...: Is a directory")`, not the typed refusal |
| `read_regular` accepts everything at both guards | the same | `null: Ok([])` |
| `read_regular` drops only its first guard (before the change: the two guards gave one text, so this passed) | the same | fails at once, with no hang: `null: Err("/dev/null is not a regular file once opened")`; clean code passes all three `refuses_` tests |

## Timing conversions / Task 5 (fix)

What changed in `exec/system/tests.rs` (and `tests/exec.rs`):
* Both test clocks count every `elapsed()` reading as well as every pause, each against the ceiling (`Count`, a `Cell`), so a runner loop that stops pausing fails instead of spinning.
* The gate of `Stepped` reads the pid FIFO without blocking (`O_NONBLOCK` on the `O_RDWR` descriptor). Until the gate opens, a pause yields and does not move the clock, counted against the frozen ceiling. A script that ends before writing its pids ends the run (Done), and `outlive` reports "the run ended before the gate opened on the processes of its script". A script that writes and then ends fails "the process ended before its limit could pass". The Leaves leader's end is also watched turn by turn (a pidfd poll with a zero wait), so the gate never blocks.
* Because the moving pauses may now start anywhere in the schedule, the limit is 2007 ms. No sum of the runner's pauses (31, 30, 28, 24, 16 or 0 ms plus 20 ms each) equals it, so the uncut-pause mutant always overshoots. The message is still "ran 2.0 s of its 2 s limit".
* Each script starts with `trap '' HUP INT TERM;` before any fork. After the hold is released, every recorded process gets SIGCONT. Only SIGKILL keeps the marker away.
* The `/proc` fate checks that ran after the end (they could not fail) are removed. The proof is the reap (`waitid` P_PIDFD ECHILD), the absent marker after release plus SIGCONT plus pidfd end, and the group of each process.
* The drop guard kills the group only on a panic (`std::thread::panicking()`); the pidfd kills stay.
* `both_streams_are_drained_at_once` moved from `tests/exec.rs` to `exec::system::tests` under `Frozen`, so no real timer is in effect. The binding moved with it.
* The Verification text says nothing tests that `WallClock` measures real time.

Mutants: a scratch clone (`git clone --no-hardlinks` into /tmp/raf-task5-fix-niCJVQ, `pwd` checked, the working diff applied, never the worktree), one test per run (`cargo test -p raf-workplan --locked --lib -- --exact exec::system::tests::<name>`), no `timeout` wrapper. No subject process was left afterwards.

| Mutant | Test | Observed |
| --- | --- | --- |
| late branch removed | outlives_its_limit | `the run made 201 pauses under a stepped clock, past the ceiling of 200` |
| `pause.min(remaining)` changed to `pause` | outlives_its_limit | `Timeout` elapsed past 2.007 s against 2.007 s |
| `abandon` removed | outlives_its_limit | `the leader was not reaped: Settled { leader_reaped: false, marker: true }` |
| `terminate` does nothing | outlives_its_limit | the same |
| `kill_group` dropped | what_the_process_started, a_descendant_that_holds, no_kill_program (inner and outer) | `a process ran on past the kill and wrote the marker` |
| `elapsed: ZERO` (finished run) | inside_its_limit | `0ns` against `7ms` |
| `elapsed: WallClock::start().elapsed()` | inside_its_limit | `120ns` against `7ms` |
| external `kill` program | no_kill_program | `Io { .. could not be killed .. }` against `Timeout` |
| the same | what_the_process_started | passes (`kill` is on `PATH`), as expected |
| `length >= bound` | output_at_the_bound | the 1000-byte run is refused |
| done on status alone | a_descendant_that_holds | `the run ended before the gate opened ...: Ok(Output { code: Some(0), .. })` |
| B1: trusted argv keeps its first word (`argv.remove(0)` made a no-op) | outlives_its_limit, a_descendant_that_holds, no_kill_program | `the run ended before the gate opened ...: Ok(Output { code: Some(126), .. })`; before the fix, a hang |
| B2: no pause once the pause reaches `POLL_CEILING` | outlives_its_limit, a_descendant_that_holds | `the run made 5000001 looks before the gate opened, past the ceiling of 5000000` (the gate opens only in a pause); before the fix, a hang |
| B3: group signal TERM | what_the_process_started, a_descendant_that_holds, no_kill_program | `a process ran on past the kill and wrote the marker`; before the fix, a pass |
| B3: group signal STOP | the same three | the same; before the fix, a pass or a hang |
| B3: group signal HUP | the same three | the same |
| B4: the two capture threads read one at a time (a shared mutex) | both_streams_are_drained_at_once | `the run made 5000001 looks under a clock that never moves, past the ceiling of 5000000`; before, under `Limits::trusted()`, it waited for the 30-minute limit |

Clean code in the clone: 5 of 5 consecutive runs of `exec::system::tests` passed (17 each). In the worktree: 6 concurrent loops of 60 runs of `exec::` passed with no failure.

The B2 and B4 mutants fail by the frozen ceiling of 5,000,000 looks; the failure is a panic at that count, never a hang.

## Timing conversions / Task 7: the never-ending cargo tests (commit 88d9e42)

Moved out of `crates/raf-workplan/tests/bound.rs` into `crates/raf-workplan/src/exec/system/tests/cargo.rs` (Linux only, `#[cfg(target_os = "linux")] mod cargo;`), same names: `exec::system::tests::cargo::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran` and `...::an_example_that_never_ends_is_killed_and_reported_with_the_time_it_ran`. They live under `exec::system::tests` so `run_with`, `Stepped`, `Subject`, `Process`, `Seen`, `Settled` and `nothing_outlived_the_kill` are reachable without a `pub(crate)` production entry.

Design:
* Per-invocation clock: `Clocked` (a test `Runner`) runs an invocation whose argv holds `--list` with `WallClock::start` and the asked limits with the timeout widened to `BUILD_TIMEOUT` (the list pass ends by itself; its real time is not under test, so a 5 s real limit would only make the outcome depend on host speed); any other invocation takes the one `Stepped` clock under the asked limits. The limits the executor asked for each invocation are recorded and asserted (both passes 5 s; the example 5 s).
* Production change (no behaviour change): `Clock::spawned(&mut self, process: u32)` with an empty default, called in `run_with` right after the spawn. Only test clocks use it. Reason: the gate must block on an event (the pid FIFO OR the end of `cargo`) and not spin: a `cargo` start takes long enough that a yield-per-pause gate could approach the 5,000,000-look ceiling on a slow host (a host-speed dependence). With the leader's pidfd taken at spawn, `Watch::turn` blocks in `poll([pid FIFO, cargo pidfd], no timeout)`; a `cargo` that ends first ends the run (fail, never a hang).
* `Stepped` takes an `Opening` (trait: `spawned`, `turn`); the Task 5 `Gate` implements it unchanged. The moving ceiling is `stepped_ceiling(limit) = 2 x (limit / POLL_CEILING)`: 200 for 2007 ms (as before), 500 for 5 s.
* Fixture: generated into the `Subject` scratch dir (`crate/`, `target/`); library function `outlive()` ignores SIGHUP/SIGINT/SIGTERM (`extern "C" signal`, `SIG_IGN`), opens the absolute pid FIFO `O_WRONLY` (the test holds it `O_RDWR`, so the open finds a reader and cannot block), writes `std::process::id()`, opens the hold FIFO `O_RDONLY` (held `O_RDWR`, cannot block), blocks in `read`, then writes the marker. `tests::forever` and `examples/forever.rs` call it; `tests::passes` is the neighbour. No sleep, no park.
* Two packages (`fx-forever-test` without an example, `fx-forever-example` with one): this host's cargo wrapper exports `CARGO_TARGET_DIR`, which overrides the crate's `build.target-dir`; one shared name made the test case's `cargo test --no-run` rewrite the uplifted `examples/forever` of the example case.
* After the run: `Subject::settle` (waitid P_PIDFD WNOHANG must be ECHILD, release hold, SIGCONT each, poll each pidfd to its end, marker absent) and `nothing_outlived_the_kill(&Seen, &Settled)` (refactored from `&Outlived`).

Assertions, test case: 2 runs; list pass `Exited(Some(0))`; run argv pinned; run `TimedOut`, `elapsed == 5 s`; reason equals "`cargo test --frozen --package=fx-forever-test --lib -- --exact -- tests::forever` ran 5.0 s of its 5 s limit and was killed; the run has no verdict"; `wall >= 5 s`; asked limits; the test binary's pid (from the pid FIFO) differs from cargo's and is in cargo's group; cargo reaped; marker absent; the neighbour `tests::passes` passes through the plain `SystemRunner`. Example case: 1 run, argv pinned, the same reason form, `elapsed == 5 s`, `wall == 5 s` (only the stepped clock's run), asked limits, reaped, marker absent. Observed: `cargo run` replaces itself with the example (the example writes the pid the runner started, so no descendant); the M2 row below confirms it.

`tests/bound.rs` (c): removed `BUDGET`, `Fixture.started`, `Fixture.package`, `within_budget` and its callers (`run`, `doc_rules`), the nanosecond nonce (now `scratch()`: pid + `AtomicUsize`, `create_dir` retried on `AlreadyExists`, crate and target inside it), `wall() > ZERO`, the module-doc budget line (replaced by a pointer to the Linux unit tests), `warm`, the `limits` parameter of `judge`, the `Duration/Instant/SystemTime/UNIX_EPOCH/Limits/Ended/Command` imports, the `forever` test of `TESTS_LIB` and `examples/forever.rs` (used only by the two removed tests). `grep -nE 'Instant|SystemTime|UNIX_EPOCH|sleep|recv_timeout|BUDGET|within_budget' crates/raf-workplan/tests/bound.rs` is empty.

Mutants: scratch clone `/tmp/raf-task7-U923M7/repo` (`git clone --no-hardlinks`, `pwd` checked, the working diff applied, never the worktree), one test per run (`cargo test -q -p raf-workplan --locked --lib -- --exact exec::system::tests::cargo::<name>`), no `timeout` wrapper. Every failing run failed at once; none hung; no fixture process left (`ps`), no scratch directory left.

| Mutant | Test (T = test case, E = example case) | Observed |
| --- | --- | --- |
| M1a executor drops the timed-out run (`Run` not pushed) | T, E | `runs.len()`: left 1 right 2 (T); left 0 right 1 (E) |
| M1b executor drops the failure of the timed-out run (`run.fail` removed) | T, E | reason `the run ended without the proof its kind requires` against the pinned `... ran 5.0 s of its 5 s limit ...` |
| M2 the group kill misses cargo's child (`kill_process(pid)` instead of `kill_process_group`) | T | `a process ran on past the kill and wrote the marker: Settled { leader_reaped: true, marker: true }` |
| the same | E | passes, as expected: `cargo run` replaced itself with the example, which is the leader that `Child::kill` reaches |
| M3 `Timeout.elapsed` from a fresh `WallClock` | T, E | `elapsed`: left 560ns / 640ns, right 5s |
| M4 Late branch removed from `verdict` | T, E | `the run made 501 pauses under a stepped clock, past the ceiling of 500` |
| M6 group signal TERM | T | `a process ran on past the kill and wrote the marker` |
| M7 group signal STOP | T | the same (SIGCONT after release resumes it) |
| M8 pause not cut to what remains | T, E | `elapsed`: left 5.011s, right 5s |
| `clock.spawned(..)` call removed | - | does not build (`spawned` is dead code under `-D warnings`) |

Stability (counts of runs, no timing): `cargo test -p raf-workplan --locked --lib exec::` 15 of 15 sequential runs passed, and 15 of 15 more after the package split. Four concurrent loops of 10 in the same worktree: 39 of 40 passed; the one failure was the example case running another process's uplifted `examples/forever` from the shared `CARGO_TARGET_DIR` that this host's cargo wrapper exports (it ran, wrote to the other process's FIFOs and ended: "the run ended before the gate opened ... pass"). Cross-process runs in one worktree share that directory; the test case is not exposed (its binary stays in `deps/` under a hash of the crate's path). A failure, never a hang. CI sets no `CARGO_TARGET_DIR`.

Evidence and assessments: `docs/capabilities/workplan-complete-gate.md` bindings moved to `exec::system::tests::cargo::*`, the Verification text gains the never-ending fixture bullet (Linux; other systems: the fake runner mapping only), the example bullet drops the timeout clause; `make docs-sync` twice (rewrote once, then nothing). `workplan evidence record --update --id gate-kills-a-test-past-its-timeout --for cap:workplan-complete-gate --by agent --copyright ... --licence CC-BY-4.0 --from-binding raf_workplan::exec::system::tests::cargo::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran` -> `recorded: ... (pass)`, exit 0; subject now the new binding, `raf_workplan::exec` digest moved. `assess --list` showed contract, examples, constraints, verification stale (exec digest; verification also edited); each context read, each accepted by agent (`replaced: ...`), exit 0; the contract assessment lists the evidence record at its new digest; SPDX headers kept. `workplan check` exit 0; `workplan evidence for cap:workplan-complete-gate` lists the record `pass`, exit 0. CI comments (macOS and Windows jobs) say the never-ending kill is tested on Linux only.

Task 7 follow-up: the `ceiling` field of `Stepped` was made Linux-only (on other Unix systems it was never read, a dead-code warning under `-D warnings` on the macOS job; no non-Linux target is installed here, so this was found by reading). The `raf_workplan::exec` digest covers the test modules, so the evidence record was recorded again (`recorded: ... (pass)`, exit 0) and the four sections assessed again, each context re-read (section texts unchanged since the first reading), exit 0; `assess --list` then shows every section current, `workplan check` exit 0, `evidence for` lists the record `pass`, exit 0.

## Remaining `elapsed` and `wall()` comparisons

Found by grepping `elapsed` and `wall` over `crates/raf-okf`, `crates/raf-workplan`, `crates/cargo-workplan` and the root `tests/`. Every value compared is scripted, counted, or a real-clock sum that cannot fail on timing; none is a bound on how long a real run took.

| Where | Comparison | Source of the value | Class |
| --- | --- | --- | --- |
| `exec/system.rs` `verdict` (270) | `elapsed >= timeout` | `Clock::elapsed()`; production `WallClock`, a stepped or frozen clock in tests | the production decision itself; tests drive it with scripted clocks |
| `exec/fake.rs` (132) | `output.elapsed > limits.timeout()` | the `elapsed` the test scripted into the `Output` | scripted |
| `exec/system/tests.rs` (138-142) | `verdict(.., elapsed, limit)` table | literals in the table | scripted |
| `exec/system/tests.rs` (892, 996, 1067), `Frozen::at` | `output.elapsed == ran` | the reading of a `Frozen` or counting test clock | scripted |
| `exec/system/tests.rs` (525) | `Timeout { elapsed: LIMIT }` | the stepped clock's value at the kill | counted (steps taken), not a reading of time |
| `exec/system/tests/cargo.rs` (454) | `killed.elapsed() == LIMIT` | the stepped clock | counted |
| `exec/system/tests/cargo.rs` (550) | `wall() == LIMIT` | the single run, under the stepped clock | counted |
| `exec/system/tests/cargo.rs` (499) | `wall() >= LIMIT` | sum of the stepped clock's run (exactly `LIMIT`) and the list pass, which runs under the real `WallClock` | real-clock sum that cannot fail on timing: the stepped part alone meets the bound and the list pass only adds a non-negative duration; it does not assert any real duration |
| `bound/tests.rs` (1484, 1524) | `evidence.wall() >= elapsed` | durations scripted into the fake runner's errors and outputs, summed | scripted |
| `bound/tests.rs` (1523) | `last.elapsed() == elapsed` | scripted | scripted |
| `evidence/tests.rs` (398, 432) | `with_elapsed(BUILD_TIMEOUT [+ 1 ms])` | scripted into the fake runner | scripted |
| `tests/exec.rs` (888, 893, 907, 827) | `with_elapsed(limit [+ 1 ms])`, `elapsed: QUERY_TIMEOUT` | scripted into the fake runner | scripted |
| `git/tests.rs`, `lifecycle/tests.rs`, `root/tests.rs`, `exec/trusted.rs` (308), `bound/tests.rs` (1469-1597) | `elapsed: Duration::from_*(..)` in an error or output | literals | scripted |
| `bound/evidence.rs` (115, 187, 245, 277) | `Run::elapsed`, the sum into `wall`, its text | copies of the executor's `elapsed` | plumbing, no comparison; real in production, shown as text |
| `bound/run.rs` (116-128) | copies `output.elapsed` / error `elapsed` into a `Run` | the runner | plumbing, no comparison |
| `tests/repo_gates.rs` (23, 278, 989-1050) | the word `elapsed` in the clock gate's tables and fixtures | text | not a time value |

No test asserts on a reading of the clock of the machine, and no test code reads one. Runs of real processes still go through the production `WallClock` as a watchdog: every test that runs a process through `SystemRunner::run` (the `tests/exec.rs` stubs, `tests/bound.rs`, the complete, verify and scenario tests, the `cargo-workplan` tests and `self_workplan`) reaches `WallClock`, which reads `Instant` and pauses with `thread::sleep` for up to 20 ms a look under 5- and 30-minute limits, and so does the list pass of `cargo.rs`. None of those readings is asserted; the one comparison that includes one, at `cargo.rs` 499, is a lower bound (`wall >= LIMIT`) that the stepped run pass alone meets, so it holds whatever the machine clock reads.
