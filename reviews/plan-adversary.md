# Plan completion adversary review

* Reviewed plan: `.stage/replace-wall-clock-assertions-in-tests/plan.md` (it adopts `clock-inventory.md` as its design source, so the inventory text the plan adopts is also reviewed)
* Inputs: `issue.md` (body and 1 comment), `issue-analysis.md`, `clock-inventory.md`, CLAUDE.md, and the tree at `a0bbba9` (read only, nothing run)
* Pass: 1 of 3 (no prior findings)

## Findings

### F1. UNFALSIFIABLE CRITERION: a frozen clock that cannot fail, and a stepped ceiling that skips zero pauses

Plan, contract rows:
> `| the runner's own timeout and output-kill tests deterministic | 4, 5 | ... the inventory's mutants fail; ...`
> `| every converted test still fails on a mutant | each | mutants run in a scratch clone and recorded in validation.md |`

The adopted design contradicts both rows:
> `Frozen(Duration)`: `elapsed()` fixed, `pause()` yields; `Stepped { ... }`: ... panicking after `ceiling` looks that advance.
> Under a frozen clock, R3 tests 6 and 7 hang on a broken mutant instead of failing; the pure capture and verdict tests fail those mutants quickly.

`Frozen` has no ceiling. So test 7 (`a_process_that_never_stops_writing_is_killed_at_the_bound`, `exec yes`) never finishes on the mutant that removes the bound it protects (the output kill). It hangs, and only CI `timeout-minutes` stops it. The plan answers this by pointing to other tests ("the pure ... tests fail those mutants"). That is a different test failing, not this converted test. Test 1 has the same problem. It lists "remove `remaining.is_zero()`" as a mutant that fails. But without that check the pause becomes `pause.min(ZERO)`, and a ceiling that counts only "looks that advance" never fires on zero-length pauses. So the mutant hangs too. The rows still say "each" and "the inventory's mutants fail".

REWRITE TO (adopted test-clock design and row 3): "Test clocks: `Frozen { reading, ceiling }` and `Stepped { now, ceiling, fifo }` both count every `pause` call, whether or not it advances the reading, and panic with the look count once `ceiling` is reached. Tests 6 and 7 use `Frozen` with a ceiling. Row 3 acceptance: each mutant listed for tests 1 to 7 makes that same test fail within one `cargo test` run, by an assertion or by the ceiling panic. validation.md records the failure text. A mutant that hangs, or that only CI `timeout-minutes` stops, counts as not failing."

### F2. UNFALSIFIABLE CRITERION: "each" has no per-test mutant list, and some converted tests have no mutant at all

Plan:
> `| every converted test still fails on a mutant | each | mutants run in a scratch clone and recorded in validation.md |`
> `| raf-workplan assessments, rationale, graph, obligations, records tests converted | 2, 3 | ... the inventory's mutants failing fast; ...; coverage pinned exactly |`
> `| raf-okf validate_cost.rs ... | 1 | ... the three mutants (build listed inside the affects loop; let first = 0/remove break in visible_content; build by_name inside the limits loop) each fail fast; ...`

"The inventory's mutants" names no mutant for the coverage test (inventory R2 Coverage gives none), for R3 test 4 (`the_group_is_killed_when_no_kill_program_is_on_the_path`) or for R3 test 6. So for those tests the acceptance can be met without running any mutant.

For raf-okf, the listed mutants all sit on the counted path. None is the regression the tests document. `validate_cost.rs` lines 121-123 and 146-147 say that what the evidence probe guards against is judging limits by search. The inventory says `check_affects` guards against "a scan of `changes` for each affects entry". Those are the mutants that remove the bound these tests protect, and the plan does not list them.

REWRITE TO: "`| every converted test still fails on a mutant | each | validation.md has one row per converted test: the test path, the mutant as a diff, and the assertion or panic that fails, run in a scratch clone. No row may say 'covered by another test'. The rows include:`
* `raf-okf: also (a) delete listed and test changes.iter().any(|c| c == capability) for each affects entry, and (b) delete by_name and search values.iter().find(..) for each limit. Each fails on one > 0 of ChangeIdsSet and MeasuresIndexed.`
* `coverage: a named mutant that changes the number of matches calls, for example calling exclusion_for once per path of matching. It fails ExclusionTests == entries x exclusions, pinned at entries {E, 2E, 3E} with X exclusions and at exclusions {X, 2X, 3X} with E entries, all nonzero.`
* `R3 test 4: kill_group spawns the kill program instead of calling kill_process_group.`
* `R3 test 6: Capture::start reads at most bound bytes (no saturating_add(1)), so 1001 bytes pass.`
* `the remaining mutants as named in the inventory.`

`The surviving variant, where the index is kept and a search is added beside it, is listed as the accepted limit and stated in the okf-profile Verification text. |`"

### F3. UNFALSIFIABLE CRITERION: "no assertion on elapsed time anywhere" rests on a gate that allows `elapsed`

Plan:
> `| no assertion on elapsed time anywhere | 9 | the extended gate (clock readings, sleeps, *_timeout waits banned in src and tests of the three crates, exemption only exec/clock.rs) passes on the tree and has fixtures: violation, clean neighbour, exemption |`
> `Duration`, `elapsed`, `timeout` stay allowed as the runner's feature and as scripted data

An assertion on a real run's elapsed time does not need any word the gate bans. `crates/raf-workplan/tests/exec.rs:1443-1444` today is:

`assert!(output.elapsed >= Duration::from_millis(100)); assert!(output.elapsed < Duration::from_secs(20));`

and lines 1617 and 1648 are similar. All of these pass the extended gate. The gate passing therefore cannot show the claim the row makes.

REWRITE TO: "`| no assertion on elapsed time anywhere | 9 | the extended gate passes with its fixtures, and validation.md lists every remaining elapsed, wall() and Duration comparison in the src tests and tests of raf-okf, raf-workplan and cargo-workplan, each with where its value comes from (a FakeRunner script, Frozen, Stepped, or a constant). None comes from a SystemRunner or WallClock run. |`"

### F4. UNFALSIFIABLE CRITERION: "the safe local gates" replaces the issue's `make check`

Issue acceptance: "`make check` passes and hosted CI is green."
Plan:
> `| make check passes, hosted CI green | 10 | the safe local gates plus make test and make test-release now (no timing test remains), hosted CI 12/12 |`
> `10. Handoff: safe gates + make test/make test-release (now clock-free), ...`

The plan never defines "the safe local gates". `make check` is twelve named targets (Makefile:121-133), so a subset could be reported as passing.

REWRITE TO (row 10 and task 10): "`make check` (fmt, clippy, test, test-release, doc, deny, reuse, msrv, fuzz, cross-check, licenses, issue-refs) exits 0 on the PR head commit, with its output tail in validation.md. The PR head's CI run shows all 12 jobs of `.github/workflows/ci.yaml` green: fmt, clippy, test, test-release, fuzz, cross-check, test-macos, test-windows, doc, deny, msrv, reuse."

### F5. UNMAPPED REQUIREMENT: the raf-source item and the comment have no contract row

Issue body: "`crates/raf-source/tests/exported_items.rs` (Instant and Duration bounds)". Comment: the tests in "`resolve_paths`, `manifest_caps`, `trait_keys`, `module_tree`, `exported_items`, `prescan_budgets`, `symbol_index`, `facts` and `tree::disk`. Each needs the same conversion".

Plan:
> The raf-source part of the issue (its comment) was delivered by the merged source-index change (work tallies, counting tree, clock-free tests, gate `time_in_raf_source`).

The claim matches the tree. `crates/raf-source/src` and `crates/raf-source/tests` name no clock; the only hit is `crates/raf-source/examples/registry_scan.rs`, which is outside the issue's grep. But the completeness contract has no row for it. It names no commit, gives no check that stays true once task 9 renames the gate, and gives no evidence for the issue's mutant acceptance on those tests. "Closes #21" covers this item too.

REWRITE TO (new contract row): "`| raf-source exported_items.rs and the comment's tests (resolve_paths, manifest_caps, trait_keys, module_tree, exported_items, prescan_budgets, symbol_index, facts, tree::disk) converted | none: delivered by <commits, e.g. f1603cd, 72bedc4, c8e5187, confirmed> | the renamed time_in_crates_tracked_files_name_no_clock keeps raf-source's 13-word rule with no exemption and passes; cargo test -p raf-source runs the whole crate with no skip list; validation.md cites where those commits recorded each converted test's failing mutant, or runs one mutant per listed test file |`"

### F6. PRE-EMPTIVE DESCOPE (platform, not disclosed): the never-ending cargo tests leave Windows

Plan:
> Never-ending real-cargo tests (R5): P1 (move into unit tests behind the stepped clock with a pid FIFO; ...)
> Linux-only checks (`/proc/<pid>/status` `ShdPnd`, `waitid`) assert only on Linux; other systems keep the error assertions (minimal effort).
> `| never-ending cargo tests | 7 | unit tests with stepped clock and SIGKILL check; evidence record recorded again; contract assessment re-assessed |`

The hosted Windows job runs `a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran` and `an_example_that_never_ends_...` today (`cargo test -p raf-workplan --locked --test bound`, ci.yaml:181-182). The CI comment says these runs are "part of the claim that they work on every platform". The `tests/bound.rs` module doc says "on every platform".

P1's gate (a FIFO) and its kill check (SIGKILL) are Unix only, and the existing `exec::system` test module is `cfg(all(test, unix))`. So after the move, Windows silently loses the end-to-end "a never-ending test is killed and refused" check. The Linux-only disposition covers only `ShdPnd` and `waitid`; it says nothing about Windows. "Recorded again" also names no observation that shows it was done.

REWRITE TO (line 15 and row 7): "R5 P1 on Unix (Linux also checks SIGKILL with `ShdPnd`; macOS asserts the error and the reason). On Windows: <(a) the moved tests also run there, gated by <a named non-FIFO event>, and assert `Ended::TimedOut` and 'ran 5.0 s of its 5 s limit' word for word; or (b) they are `cfg(unix)`, the owner's decision is quoted, and ci.yaml lines 140-143 and 175-178 and the `tests/bound.rs` module doc stop claiming every platform>. Row 7 acceptance adds: `cargo workplan check` reports `gate-kills-a-test-past-its-timeout` current, with the new binding as its subject, and `workplan-complete-gate-contract` current."

### F7. PRE-EMPTIVE DESCOPE: the budgets are dropped, not converted, and the reason given is wrong

Issue: "Convert each wall-clock bound to an operation count, an allocation count or a `perf` measurement, keeping the guarantee each test protects (... bounded evidence runs)".
Plan:
> Budgets (R6): dropped; the runner's own limits and CI `timeout-minutes` are the watchdogs.
> `| budgets, nonces, mtime | 8 | no BUDGET/within_budget/SystemTime in the three crates' tests |`

`within_budget()` (tests/bound.rs:187-194, tests/support/project.rs) runs after the run returns, so it never caught a hang. What it guards is the cost of a fixture that finishes ("No fixture may take longer than this, builds included"; the module doc says "Every test asserts a wall-clock budget"). The bound.rs fixtures are evidence runs of the bound executor. The replacement watchdogs bound something else: 30 min per build, and 30 min per CI job. So this guarantee goes with no replacement.

The inventory names a count-based conversion ("a counting `$CARGO` wrapper logging each call"). The plan neither adopts it nor lists it under Declined. The check in row 8 only proves the code was removed.

REWRITE TO (line 17 and row 8), one of:
* (A) "Budgets converted: a counting `$CARGO` shim logs each call. Each bound, complete, verify and scenario fixture asserts the exact number of cargo invocations it makes. A mutant that adds one cargo invocation to the executor's run fails the count."
* (B) "Budgets removed without a count by the owner's decision: <quote>. The plan states that the per-fixture cost bound is given up, and that the runner limits and CI `timeout-minutes` stop hangs only. The counting `$CARGO` shim is listed under Declined with its cost."

## Judgments on the narrowing questions

* **Declined coverage index: not a cut.** The 60 s test only bounded today's entries x exclusions work at its size. Pinning `ExclusionTests == entries x exclusions` exactly is at least as strict as that, and any extra `matches` call fails it. Indexing is a production performance enhancement that the issue does not ask for, so a tracked follow-up is acceptable. The only gap is the missing mutant (F2). No owner decision is needed.
* **Accepted uncounted-loop limit: not a cut, if F2 is applied.** The issue itself names operation counts as an acceptable conversion, and this limit comes with that method. With the documented-regression mutants added (delete the index and search instead), the counts fail on the regression these tests were written for, through `one > 0` on the index counter. Only an added search that leaves the index in place survives. Stating that in the Verification text is honest. No owner decision is needed.
* **Linux-only `ShdPnd` and `waitid` (R3): not a cut.** This follows CLAUDE.md's platform rule. The hosted macOS job does not run `tests/exec.rs` today, so moving these tests into `exec::system::tests` adds macOS CI coverage rather than removing any. The exception is the R5 tests on Windows (F6).
* **Dropped budgets: a cut** unless converted or decided by the owner (F7).
* **Widened acceptance grep: cleared.** The extended gate bans `Instant` as a whole word in every Rust file of `crates/*/src` and `crates/*/tests`, except `exec/clock.rs`, which is production code and not an assertion. So it implies the issue's grep criterion. The separate overclaim in row 9 is F3.
* **Equivalent mutant `elapsed: limits.timeout()`: cleared.** The old test (`elapsed >= timeout`) did not tell it apart either, and the plan discloses it. Test 1 pins the exact look count (105) and `elapsed == 2 s`, so the pause-schedule mutants are not equivalent.
* **Growth-idiom lower bound: cleared.** `one > 0`, together with the mutant runs required by F2, keeps the count tests from passing vacuously.

## User decisions required

These can be settled before pass 2, or the planner can choose the option that needs no decision (F6 (a), F7 (A)):
1. F6: should Windows keep an end-to-end never-ending-test check, (a) a Windows variant, or (b) a recorded platform narrowing with the CI comments and the `tests/bound.rs` doc corrected?
2. F7: should the fixture budgets be (A) converted to an exact count of cargo invocations, or (B) removed without a count, recorded as giving up the per-fixture cost bound?

VERDICT: FINDINGS-OPEN
