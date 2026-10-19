# Clock inventory and conversion design (crates other than raf-source)

(Read-only design pass, saved verbatim in substance. Dispositions of its open decisions are in plan.md. Legend: A = a test that asserts or bounds elapsed time; B = a test that waits for a real process or timer by design; C = test infrastructure that is a hang watchdog or a budget; D = production code where a duration is the feature (stays); E = other: a Duration as scripted data, a scratch-name nonce, a file mtime, a masked run time, a false positive.)

## Summary

* The requested grep gives 284 lines; a wider pass adds about 25. `cargo-workplan` src and `raf-okf` src read no clock. The only production clock reading is `crates/raf-workplan/src/exec/system.rs` (`Instant::now`, `thread::sleep`).
* Counters: raf-okf and raf-workplan have none. Both need a cfg(test)-only `work.rs` modelled on `crates/raf-source/src/work.rs`. Counters compile only in each crate's own unit tests, so counted tests are unit tests, as raf-source did. Copy the module (do not share by `#[path]`: that breaks `cargo package` for path dependencies).
* Runner: no injectable clock today. `FakeRunner` scripts `Output.elapsed`, already a virtual clock for code that uses a runner, but `SystemRunner` hard-codes `Instant` and `sleep`. The smallest seam is a private `Clock` trait with two methods (`elapsed`, `pause`) and a real `WallClock` in a new one-file `exec/clock.rs`, plus two pure functions (the per-look verdict and the pause schedule).
* Real processes: "the grandchild got the kill" can be checked without a clock on Linux: after the group kill returns, `/proc/<pid>/status` `ShdPnd` holds SIGKILL until the process is reaped (needs one confirming run). "The leader was killed" is checked by `waitid(WNOHANG)` returning ECHILD.
* Gate: extend `time_in_raf_source` to the three crates with a narrower word list: ban clock readings and waits (`Instant`, `SystemTime`, `UNIX_EPOCH`, `sleep`, the `*_timeout` waits, `tokio::time`); allow `Duration`, `elapsed`, `timeout` (the runner's feature, and scripted data in tests). One exempt file: `exec/clock.rs`.

## (1) Inventory

### crates/raf-okf (src has no clock)
| file:line | site | class |
|---|---|---|
| tests/validate_cost.rs:7 | `use std::time::{Duration, Instant}` | A |
| tests/validate_cost.rs:11-16 | `RELEASE_BOUND = 100 ms` | A |
| tests/validate_cost.rs:71-78 | `assert_within_bound` (skipped when `cfg!(debug_assertions)`) | A |
| tests/validate_cost.rs:80-85 | `timed`: `Instant::now()`, `elapsed()` | A |
| tests/validate_cost.rs:88-98, 102-118, 154-164 | `a_change_at_the_cap_with_maximal_affects_and_changes_validates_within_the_bound`, `a_capability_at_the_cap_with_many_sections_and_blocks_validates_within_the_bound`, `an_evidence_record_at_the_cap_with_maximal_values_and_limits_validates_within_the_bound` | A |

The regressions these guard: `check_affects` in `src/profile/mod.rs:~598` (a `HashSet` replaced a scan of `changes` for each affects entry); `visible_content` in `src/profile/body_rules.rs:~44` (`partition_point` and `break` replaced a scan of every block for each section); `src/profile/records.rs:589-596` (the `by_name` `HashMap` replaced a search of `values` for each limit). Places that refer to the release-only probe: `Makefile` (comment lines 14-17, `test-release` recipe 64-66), `.github/workflows/ci.yaml:85-86`, `AGENTS.md:159-162`, `CONTRIBUTING.md:77-78`, `docs/capabilities/okf-profile.md:448-450` and `485-486`.

### crates/raf-workplan production (D, stays)
`src/exec.rs` (Limits, timeouts BUILD 30 min, EXAMPLE and QUERY 5 min, `Output.elapsed`, `ExecError::Timeout{timeout, elapsed}` and `OutputTooLarge{elapsed}` and their Display), `src/exec/fake.rs` (applies `limits.timeout()` to the scripted elapsed: already a virtual clock), `src/exec/system.rs` (the only clock readings: `Instant::now` at 111, `elapsed` at 135/142/147, `wait` 225-276 with `thread::sleep(pause.min(remaining))`, `POLL_CEILING = 20 ms`), `src/bound/evidence.rs` (`Run.elapsed`, `Evidence.wall`), `src/bound/run.rs`, `src/obligation.rs`, `src/exec/trusted.rs`. `--since`/`--until` and `Date` in `pattern.rs`, `history.rs`, `git/log.rs`, `lifecycle/create.rs` are calendar dates, not clocks.

### crates/raf-workplan unit tests (inside src)
| file:line | site | class |
|---|---|---|
| src/graph.rs:2020-2034 | `a_section_of_many_blocks_is_read_in_linear_time`: `Instant::now`, `took < 20 s` | A |
| src/evidence/tests.rs:16, 184-199 | `Bounded` runner: a real `SystemRunner` with `timeout: Option<Duration>` | C |
| src/evidence/tests.rs:397-441 | `a_run_killed_at_its_time_limit_records_fail_and_its_neighbour_passes`: 1 s limit with `sleep 30`, `elapsed < 20 s`, 20 s watchdog | B, A, C |
| src/bound/tests.rs, src/lifecycle/tests.rs, src/git/tests.rs, src/root/tests.rs, src/exec/trusted.rs tests | scripted `Duration` in `ExecError`/`with_elapsed` given to a `FakeRunner` | E (data, deterministic) |
| src/exec/system.rs:330-390 | five group-signal tests, no clock (precedent) | — |

### crates/raf-workplan integration tests
| file:line | site | class |
|---|---|---|
| tests/exec.rs:1095-1110 | imports, `Scratch::new` nanosecond nonce via `SystemTime::now()` | E |
| tests/exec.rs:1401-1428 | `a_process_that_outlives_its_limit_is_killed`: `exec sleep 30`, 400 ms limit, `elapsed >= timeout`, `waited < 10 s` | B, A |
| tests/exec.rs:1432-1445 | `a_process_inside_its_limit_passes`: `sleep 0.1`, `elapsed >= 100 ms`, `< 20 s` | B, A |
| tests/exec.rs:1448-1492 | `alive(pid)` via /proc; `family_does_not_outlive_the_kill`: `sleep 60 &` grandchild, 3 s limit, deadline loop with `thread::sleep(20 ms)` | B, A, C |
| tests/exec.rs:1495-1533 | `what_the_process_started_does_not_outlive_the_kill`, `the_group_is_killed_when_no_kill_program_is_on_the_path` | B |
| tests/exec.rs:1537-1556 | `a_descendant_that_holds_the_pipes_after_the_exit_is_not_waited_for_past_the_limit`: `sleep 60 &`, 500 ms, `< 10 s`, sleep loop | B, A, C |
| tests/exec.rs:1574-1625 | `output_at_the_bound_passes_and_one_byte_over_is_refused_on_each_stream`: `elapsed > ZERO` | A (weak) |
| tests/exec.rs:1628-1657 | `a_process_that_never_stops_writing_is_killed_at_the_bound`: 60 s watchdog, `elapsed <= ran`, `< 10 s` | C, A |
| tests/exec.rs:1689-1708 | `both_streams_are_drained_at_once`: 30 s deadlock watchdog | C |
| tests/bound.rs:16, 25, 33, 49, 103, 171, 187-193, 799, 936 | `BUDGET = 300 s`, `Fixture.started`, `within_budget()` | C |
| tests/bound.rs:57-59 | nanosecond nonce | E |
| tests/bound.rs:313-315, 847-850 | generated `forever` fixtures with `std::thread::sleep(120 s)` strings | B (fixture) |
| tests/bound.rs:390 | `evidence.wall() > Duration::ZERO` | A (weak) |
| tests/bound.rs:542-575 | `a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran`: 5 s limit on real cargo; `waited`, `wall >= limit`, `waited >= limit && < 12 x limit` | B, A |
| tests/bound.rs:898-937 | `an_example_that_never_ends_is_killed_and_reported_with_the_time_it_ran`: 5 s limit on real cargo; `waited` | B, A |
| tests/complete.rs, tests/verify.rs (raf-workplan) | `BUDGET = 300 s`, `Fixture.started`, `within_budget` | C |
| tests/support/project.rs:18, 28, 67, 104, 243-254 | `BUDGET`, `Project.started`, `within_budget`, `elapsed()` | C |
| tests/scenarios.rs:24, 481, 799, 1441, 1525, 1591, 1764 | `project.within_budget()` | C |
| tests/coverage.rs:12, 718-729 | `a_crate_with_thousands_of_items_and_tests_is_judged_in_bounded_time`: `elapsed < 60 s` | A |
| tests/obligations.rs:1458-1516 | `moves_to_unknown_records_cost_the_same_with_few_records_as_with_many`: `many_time < few_time*5 + 1 s` | A (relative) |
| tests/obligations.rs:1535-1627 | `contract_with` returns `(result, Duration)`; `BUDGET = 30 s`; `a_very_long_line_is_one_obligation_and_is_read_in_linear_time`, `a_hundred_thousand_lines_are_a_hundred_thousand_obligations_in_order`, `text_that_is_read_as_wording_at_a_hostile_size_does_not_hang` | A |
| tests/rationale.rs:12, 553-576 | `a_hundred_thousand_headings_and_an_unterminated_fence_are_judged_in_bounded_time`: `< 60 s` | A |
| tests/assessments.rs:13, 38-40, 836-897 | `a_huge_section_and_ten_thousand_named_symbols_are_recorded_and_judged_within_bounds`: `BUDGET = 60 s` | A |
| tests/records.rs:12, 961-1051 | `RELEASE_BOUND = 5 s`, `timed_check`, `within_bound` (release only, so never in CI), `a_record_at_the_size_bound_with_the_most_measures_loads`, `a_hundred_thousand_claims_load_in_time_linear_in_their_number` | A |
| tests/root.rs:15, 36-38; tests/git.rs:14, 39-41; tests/gate.rs:17, 36, 42-44 | nanosecond nonce | E |
| tests/preview.rs:396 | `snapshot(..., SystemTime)` file mtime | E |

### crates/cargo-workplan (src has no clock)
| tests/complete.rs, tests/verify.rs | `BUDGET = 300 s`, `timed()` | C |
| tests/sync.rs:253 | `"state: sleeping"` string | E (false positive) |
| tests/evidence.rs:192-237, tests/support/mod.rs:272-280, 371-395 | masks the seconds of "in N s and was killed"; the 17 MB overflow is judged by bytes | E (stays) |

## (2) Conversions

Shared rules. Counters: add `crates/raf-okf/src/work.rs` and `crates/raf-workplan/src/work.rs`, copies of `crates/raf-source/src/work.rs` with their own `Work` enum; `add` does nothing outside cfg(test); `measure`/`measure_kind` panic at a ceiling; tallies are per thread (the counted work runs on one thread; only `exec/system.rs` spawns threads). Growth idiom (from f1603cd): measure at 1, 2, 3 units; assert `one > 0` and `three + one <= 2*two`, with a ceiling that fails fast. Counted tests are unit tests of the crate doing the work; integration tests keep their functional assertions and drop the clock. A count catches a regression that goes through the counted step, not a new uncounted loop.

### R1. raf-okf `validate_cost.rs`
Hooks (cfg(test) only): `profile/mod.rs` `check_affects`: `add(ChangeIdsSet, changes.len())` where the set is built, `add(AffectsEntries, 1)` per entry; `profile/body_rules.rs` `visible_content`: `add(BlocksVisited, 1)` first in `for block in &blocks[first..]`; `profile/records.rs:592-596`: `.inspect(|_| add(MeasuresIndexed, 1))` building `by_name`, `add(LimitLookups, 1)` per limit. New `crates/raf-okf/src/profile/cost_tests.rs`, sizes about 2,000: changes/affects `AffectsEntries == n` and `ChangeIdsSet` linear; capability `BlocksVisited` linear in sections for fixed blocks and in blocks for fixed sections; evidence `LimitLookups == n`, `MeasuresIndexed` linear. Keep `validate_cost.rs` as untimed at-the-cap checks (validate, or refused for the duplicated section), renamed without "within_the_bound". `test-release`, CI, AGENTS, CONTRIBUTING drop the probe. Mutants: build `listed` inside the affects loop; `let first = 0` or remove the `break` in `visible_content`; build `by_name` inside the limits loop.

### R2. raf-workplan algorithmic bounds
* `graph::tests::a_section_of_many_blocks_is_read_in_linear_time`: count `BlankedSteps` (each `ranges.next()` in `prose_of`, graph.rs:~1819) and `ProseChars` at 1,2,3 x 5,000 paragraphs; rename `..._is_read_in_work_linear_in_its_blocks`; mutant: rebuild the peekable merged ranges per character.
* Rationale: unit test counting `ConditionsScan` (steps of `extent`'s `find`) and `ProseLinesScanned` (the `take_while` of `has_item`) at three sizes for the 20,000-empty-conditions page and the 20,000-pair Explanation; mutants: `let first = 0` in `has_item`; `extent` finding with `<` instead of `<=`. The integration test drops the clock (`a_hundred_thousand_headings_and_an_unterminated_fence_are_judged`).
* Assessments: unit test in `assessment/tests.rs` counting `NamingLookups` in `naming()` (~line 633) through `symbol_inputs`; mutant: compute `any_strong` inside the per-path closure; integration test drops `BUDGET`.
* Obligations: extend `obligation::tests::hostile_text_is_read_in_linear_time` counting `ScanBytes` (bytes searched by `find(')')` and char steps in `word_spans`) and `PhraseTests`; mutant: `map_or(text.len(), ..)` changed to `map_or(next, ..)` so each `](` searches to the end. `moves_to_unknown_records_cost_the_same_with_few_records_as_with_many` becomes a unit test in `obligation::tests` over a Workplan on a `MemoryTree` with 10 and 2,000 records: `OutcomeChecks` equal for both (8,000), `SlugsListed` equals the records; mutant: build `known` inside the item loop in `sources.rs:~211`. In the integration tests `contract_with` drops the `Duration` and `BUDGET`.
* Records and claims: unit test in graph tests counting `ClaimChecks` (each slug of each claim, `graph.rs:~1363`) and the loaded/present sets built; mutant: rebuild `loaded`/`present`/`covered` per claim. Integration: `RELEASE_BOUND`/`within_bound` go; `a_hundred_thousand_claims_load_in_time_linear_in_their_number` becomes `..._load_with_one_finding_each`; `a_record_at_the_size_bound_with_the_most_measures_loads` keeps its name (its validation cost is raf-okf's, R1).
* Coverage: unit test in a new `coverage/tests.rs` counting `ExclusionTests` (each `matches` in `exclusion_for`, coverage.rs:~788, called from ~841) and `EntriesJudged`. The current code is entries x exclusions. Options: index the exclusions, or pin the product exactly (decision in the plan).

### R3. The runner clock seam (`exec/system.rs`)
New `exec/clock.rs`, the only file in the three crates reading a clock: `pub(super) trait Clock { fn elapsed(&self) -> Duration; fn pause(&mut self, pause: Duration); }`, `pub(super) struct WallClock(Instant)` with `start()`, `elapsed()`, `pause()` (`thread::sleep`). In `system.rs`: `impl Runner for SystemRunner { fn run(..) { self.run_with(inv, limits, WallClock::start) } }`; private `fn run_with<C: Clock>(&self, inv, limits, start: impl FnOnce() -> C)` (starts the clock just before `spawn` as today); `wait(child, out, err, timeout, clock: &mut C)`; two pure functions out of the loop: `verdict(overflow_out, overflow_err, done, elapsed, timeout) -> Look {Over(Stream), Done, Late, Again}` and `next_pause(pause) -> Duration` (1, 2, 4, 8, 16, 20, 20, ...). Public API and `FakeRunner` unchanged. The inline `mod tests` (330-390) becomes `exec/system/tests.rs`; the five existing tests keep their binding paths.
Test clocks (cfg(test), using only `Duration` and `thread::yield_now`): `Frozen(Duration)`: `elapsed()` fixed, `pause()` yields; `Stepped { now, looks, ceiling, fifo: Option<PathBuf> }`: on the first `pause` it blocks reading a FIFO the script writes its pid(s) to (a wait on an event, not a clock), then each `pause(p)` adds `p` to `now`, panicking after `ceiling` looks that advance. Scripts are `/bin/sh -c <script>` invocations of `Invocation::trusted` (no executable written, so no ETXTBSY and no serial lock); a process that never ends stops itself (`kill -STOP $$`, a shell builtin).
Tests moved to `exec::system::tests` (bindings change):
1. `a_process_that_outlives_its_limit_is_killed`: `echo $$ > fifo; kill -STOP $$`, Stepped clock, limit 2 s; asserts `Timeout { timeout: 2 s, elapsed: 2 s }` exactly, 105 looks (31 ms from 1+2+4+8+16, then 98 x 20, then 9), message "ran 2.0 s of its 2 s limit", leader reaped (`waitid(P_PID, pid, WEXITED|WNOHANG)` gives ECHILD). Mutants: remove `remaining.is_zero()`; `pause.min(remaining)` changed to `pause`; remove `abandon`/`terminate`; change `POLL_CEILING`. A drop guard SIGKILLs recorded pids so a failing mutant leaves nothing.
2. `a_process_inside_its_limit_passes`: `echo done`, `Frozen(7 ms)`, 20 s limit; asserts `Ok`, stdout, `elapsed == 7 ms`. Mutants: `elapsed: ZERO`, or taken from another reading.
3. `what_the_process_started_does_not_outlive_the_kill`: `/bin/sh -c 'echo $$ > fifo; kill -STOP $$' & wait`, Stepped gated on the FIFO (grandchild pid); asserts `Timeout`, leader reaped, and on Linux for the grandchild `/proc/<g>/status` `ShdPnd` has bit 0x100 (SIGKILL) or state Z/X or entry gone (compare `starttime` against pid reuse). Mutant: drop `kill_group` from `terminate`.
4. `the_group_is_killed_when_no_kill_program_is_on_the_path`: re-exec the lib test binary with `--exact exec::system::tests::...`, `PATH` an empty directory, a marker env var; the inner run asserts `kill` is not found, then runs test 3.
5. `a_descendant_that_holds_the_pipes_after_the_exit_is_not_waited_for_past_the_limit`: `/bin/sh -c '...STOP' & exit 0`, Stepped gated on the FIFO; asserts `Timeout` with `elapsed == limit` and SIGKILL reaching the grandchild. Mutant: return `Done` when the status is in but the streams are not.
6. `output_at_the_bound_passes_and_one_byte_over_is_refused_on_each_stream`: `Frozen(1500 ms)`; 1000 bytes pass, 1001 give `OutputTooLarge { elapsed: 1.5 s }` with "in 1.5 s and was killed".
7. `a_process_that_never_stops_writing_is_killed_at_the_bound`: `exec yes`, `Frozen(..)`: the timeout cannot fire, so reaching `OutputTooLarge` proves the bound killed it; assert `elapsed ==` the frozen reading.
New pure tests: `the_capture_stops_one_byte_past_its_bound_and_flags_the_overflow` (`Capture::start(io::repeat(b'x'), 1000)`, 1001 bytes received and `overflowed()`; a 1000-byte Cursor not flagged); a decision table for `verdict` (overflow beats done and late; done beats late; late only when not done and `elapsed >= timeout`; otherwise look again); `next_pause` schedule and ceiling. These fail every mutant that would only hang the frozen-clock smoke tests (6, 7).
Kept in `tests/exec.rs` with the real runner and no time assertions: `both_streams_are_drained_at_once` (the 30 s limit stays as a deadlock watchdog, no clock word), `the_default_bound_is_sixteen_mebibytes...`, `output_is_returned_byte_for_byte`, etc. `alive()`, `family_does_not_outlive_the_kill`, the `Instant`/`SystemTime` imports and the nonce go.

### R4. `evidence::tests::a_run_killed_at_its_time_limit_records_fail_and_its_neighbour_passes`
`FakeRunner::on(Program::Trusted, ["/bin/sh","-c","echo peak=1"], Ok(answer.with_elapsed(BUILD_TIMEOUT + 1 ms)))` with the trusted limits; assert `Fail`, `[Failed::TimedOut]`, no values, the given limit kept, the text of the record. Neighbour `with_elapsed(BUILD_TIMEOUT)` passes. The name stays; `Bounded` loses its `timeout` field. Mutants: map `Timeout` to an error instead of `fail`; drop the limit when killed; give the runner limits other than `Limits::trusted()`. The real-runner timeout is R3's job.

### R5. `tests/bound.rs` never-ending real-cargo tests
`a_test_that_never_ends_...` (542) and `an_example_that_never_ends_...` (898). The FakeRunner unit tests in `bound/tests.rs` pin the executor's mapping.
* P1 (recommended): move both into unit tests: generate the fixture crate whose `forever` test/example writes its pid to an absolute FIFO baked into the source, then `loop { std::thread::park(); }`; warm-build it (real time, asserts nothing about time); run through `Executor::new(&source, &Stepped(&system_runner, clock_factory))` using a cfg(test) `Runner` wrapper calling `run_with`; assert the reason "ran 5.0 s of its 5 s limit" word for word, `Ended::TimedOut`, `wall == 5 s`, and that the test binary's pid got SIGKILL. Mutants: the executor drops the run; the group kill misses cargo's child. Cost: the lib tests run a real cargo build.
* P2 (fallback): keep them as B (the real 5 s timer); delete `waited`, `wall >= limit`, `< 12 x limit`; replace generated `sleep` with `park`; keep the "5 s limit" reason (about 5 s real time each).
* P3 (cheapest): drop the never-ending fixture; assert `/proc/self/stat` `pgrp == ppid` (cargo's test binary is in the group cargo leads); with R3 and the fake mapping this covers the claim but loses the evidence record's end-to-end subject.
Evidence record `work/evidence/gate-kills-a-test-past-its-timeout.md` has as subject the binding `raf_workplan::test_crate::bound::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran` and digests of `raf_workplan::exec`, `SystemRunner`, others. P1 renames the subject and R3 may change digests: record it again and re-assess `work/assessments/workplan-complete-gate-contract.md`. The id stays, so `tests/docs_contract.rs:160` does not change.

### R6. Budgets (C)
Remove `BUDGET`, `started`, `within_budget` from `tests/bound.rs`, `tests/complete.rs`, `tests/verify.rs`; `tests/support/project.rs` (and its `elapsed()` and the six callers in `scenarios.rs` and its module doc); cargo-workplan `tests/complete.rs` and `tests/verify.rs` (`timed` becomes `repo.run`). Drop `tests/bound.rs:390` `wall() > ZERO` (unit tests pin `wall`; R3 test 2 pins `elapsed`). The real watchdogs are the runner's `BUILD_TIMEOUT`/`QUERY_TIMEOUT` and CI `timeout-minutes`. Optional: a counting `$CARGO` wrapper logging each call.

### R7. Nonces and mtimes (E)
`root.rs:36`, `git.rs:39`, `gate.rs:42` (and its doc at 36), `exec.rs:1108`, `bound.rs:57`: use pid plus an `AtomicUsize` with `create_dir`, retrying on `AlreadyExists` (`tests/support/disk.rs` already uses pid plus counter). `preview.rs:396`: rewrite `snapshot` as one iterative walk returning `BTreeMap<PathBuf, (Vec<u8>, impl Eq + Debug)>` so `SystemTime` is not named (or allow the type in the gate).

## (3) Bindings and Verification text that change
Tables regenerate with `make docs-sync`; each edited Verification section needs its `work/assessments/<cap>-verification.md` recorded again.
* okf-profile.md: bindings 40, 41, 120 (table 584-586): the three `validate_cost::*` tests renamed, three `raf_okf::profile::cost_tests::*` added; Verification 448-450, 485-486 ("within 100 ms in a release build") becomes the counted growth plus "at the cap validates".
* workplan-document-model.md: 306/726 `graph::tests::a_section_of_many_blocks_is_read_in_linear_time` renamed; 140/896 `records::a_hundred_thousand_claims_load_in_time_linear_in_their_number` renamed plus the new counted `graph::tests` claims test; 262/897 keeps its name; Verification 711-712.
* workplan-current-rationale.md: 16/199 renamed plus the new `rationale::tests` counted test; Verification 180.
* workplan-prose-assessment.md: 21/282 renamed plus the new `assessment::tests` counted test; Verification 243-244.
* workplan-reconcile.md: 148/675 `a_very_long_line_..._in_linear_time` renamed; 154/687 `moves_to_unknown_records_...` moves to `raf_workplan::obligation::tests::`; 217/595 `obligation::tests::hostile_text_is_read_in_linear_time` rename optional; 146 and 157 keep their names.
* workplan-two-way-coverage.md: 16/282 renamed plus the new `coverage::tests` counted test; Verification 250-251.
* workplan-complete-gate.md: these bindings move from `test_crate::exec::stubs::` to `exec::system::tests::`: 86/825, 92/830, 125/820, 128/823, 129/824, 135/835, 137/837; new pure tests (capture, verdict, pause schedule) added; 71/759 and 78/761 (bound, never-ending) change under P1; 87-91 and 140 unchanged; Verification 616-618 stays true; 635-638 ("the real runner gives the time of an output kill, more than zero and no more than the call took") becomes "the reading of its clock at the kill, pinned with a test clock"; the Contract (423-433) does not change.
* workplan-evidence-claims.md: 38/338 keep their names.

## (4) Extending the gate in `tests/repo_gates.rs`
Turn `CLOCK_FREE`/`CLOCK_WORDS` (lines 251-302) into a table of rules. raf-source: dirs `src` and `tests`, the 13 words, no exemptions. The other three crates: dirs `crates/raf-okf/{src,tests}`, `crates/raf-workplan/{src,tests,examples}`, `crates/cargo-workplan/{src,tests}`; words `Instant`, `SystemTime`, `UNIX_EPOCH`, `tokio::time`, `sleep`, `recv_timeout`, `wait_timeout`, `park_timeout`, `set_read_timeout`, `set_write_timeout`; exempt exactly `crates/raf-workplan/src/exec/clock.rs` (the gate asserts the file is tracked and names `Instant`, so the exemption cannot go stale silently). `Duration`, `std::time::Duration`, `elapsed`, `timeout`/`with_timeout`, `wall()` stay allowed (the runner's feature; scripted data in tests). The check stays per file because after R3 no production code in the three crates reads or waits on a clock except `clock.rs`; no cfg(test) parsing. Rename the prefix `time_in_raf_source_` to `time_in_crates_`, update the module doc (lines 19-21, 25); the "exempts other crates" fixture (915-926) lists `crates/raf-okf/src/lib.rs` and must change to paths outside every rule. New fixtures: a planted reading in each new directory reported with file and line; `Duration`, `elapsed` and `with_timeout` pass in the three crates while `Duration` is refused in raf-source; `clock.rs` exempt but `exec/system.rs` not; `sleeping` and `Instantiate` clean.

## (5) Commits, in order
Each leaves `make check` green and carries its own binding, Verification and assessment updates.
1. `test(raf-okf)`: work tallies, counted validation cost tests, untimed probes at the cap; `test-release`, CI, AGENTS, CONTRIBUTING drop the probe; okf-profile bindings, text, assessment.
2. `test(raf-workplan)`: work tallies, then one commit each for graph and claims (document-model), rationale, assessments and obligations (reconcile).
3. Coverage (decision in the plan).
4. `refactor(raf-workplan)`: the Clock seam, pure `verdict`/`next_pause`, `system/tests.rs` split out, no behaviour change; `cargo workplan check`; record the evidence again if stale.
5. `test(raf-workplan)`: runner kill tests with frozen and stepped clocks, `tests/exec.rs` trimmed, complete-gate bindings and text.
6. `test(raf-workplan)`: the evidence timeout through `FakeRunner`.
7. `test(raf-workplan)`: the never-ending cargo tests (P1/P2/P3), the evidence record, the contract assessment.
8. `test(raf-workplan, cargo-workplan)`: budgets removed, nonces, the preview snapshot.
9. `test`: the repository gate extended (last, once the tree is clean).

## (6) Risks and what cannot become a count
* Coverage: `exclusion_for` is entries x exclusions today; a linear count test fails on the current code. Options: (a) index the exclusions as its own perf commit; (b) pin `ExclusionTests == entries x exclusions` (linear in each dimension with the other fixed) and state the product in the Verification text.
* Uncounted loops: counts catch mutants on the counted path, not a new uncounted loop (raf-okf's original two hash-lookup regressions are of that kind). Options: (a) accept, as raf-source did; (b) put the data behind counting newtype accessors (`Listed`, `Measures`); (c) a `perf stat -e instructions:u` make target outside `cargo test` (perf counters are not reliably available on CI runners).
* Counting across crates is not possible while hooks are cfg(test)-only; R1 covers raf-okf's work.
* `/proc` `ShdPnd` relies on kernel signal semantics; confirm with one correct run and one mutant run; Linux only (macOS keeps only the error assertions; fallback: blocking `pidfd` poll, a mutant shows as a hang caught by CI `timeout-minutes`).
* Under a frozen clock, R3 tests 6 and 7 hang on a broken mutant instead of failing; the pure capture and verdict tests fail those mutants quickly.
* Equivalent mutant: `elapsed: limits.timeout()` cannot be told apart under the stepped clock because the last pause lands exactly on the limit.
* Evidence and assessments: recording again requires an agent assessment of each edited section; P1 renames the evidence subject; P1 runs a real cargo build inside lib unit tests.
* Cannot be converted to a count: that `WallClock` measures real time (trusted to std); the CI-level hang watchdog; the real 5 s timer if P2 is chosen; the end-to-end cost of cargo-workplan commands (removed; a counting `$CARGO` wrapper is the count-based option).
