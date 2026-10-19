# Plan constraints review

Reviewed plan: `.stage/replace-wall-clock-assertions-in-tests/plan.md`, together with its design source `clock-inventory.md`, which the plan adopts with the dispositions it lists (plan.md:10).

## Governing documents read

- `CLAUDE.md` in the worktree (lines 12-32: gates, the Never list, no timing runs, platforms, no deferrals, no attribution, `.deficiencies`)
- `AGENTS.md` (sections 1, 2, 3 with the workspace lints, 4, 5, 6 and 7)
- `CONTRIBUTING.md` (design constraints, development, before you open a pull request)
- `SECURITY.md` (trust model, Subprocesses, security surface)
- `.deficiencies`: only the notice and marker. The plan adds nothing to it.
- `Cargo.toml` (`[workspace.lints]`: `unreachable_pub`, `missing_docs` and `unsafe_code` denied, edition 2024) and `crates/raf-workplan/Cargo.toml` (`rustix` 1.1.5 with the `process` and `std` features only)
- The gate design: `tests/repo_gates.rs` (module doc 4-27, `CLOCK_FREE`/`CLOCK_WORDS` 251-302, gate 531-551, fixtures 855-925), `tests/self_workplan.rs` (every assessment current and accepted, run by the pre-commit hook), `tests/docs_contract.rs:100-163` (the pinned record ids)
- Supporting evidence: `crates/raf-workplan/src/exec/system.rs`, `exec/trusted.rs` (`TrustedArgv::new` is `pub(crate)`), `exec/invocation.rs:206`, `work/evidence/gate-kills-a-test-past-its-timeout.md`, the input lists of `work/assessments/*.md`, `docs/capabilities/workplan-two-way-coverage.md:250-251`, `crates/raf-workplan/src/refs.rs` (process-wording patterns), `.gitignore:5` (`/.stage/` is ignored, so the plan's issue numbers are never committed)
- Absent, and recorded as absent: `.baseline`, `.goals`, `CONSTITUTION.md`, `PRECOMMIT_GUIDANCE.md`, ADR directories (`docs/adr/`, `docs/decisions/`), and any nested `CLAUDE.md` or `AGENTS.md` under the touched crates. The review still checks the repository rules and the user decisions.

User decisions on record (plan.md:7): "Convert timing tests first" (raf-source, already delivered) and "macOS is minimal effort". The plan is consistent with both.

## Findings

### C1. BLOCKING: task 4 schedules a timing run to record the evidence again

Governing text:
- CLAUDE.md:18-19: "**No timing runs**: time-based testing is forbidden. Write no timing runs, wall-clock benchmarks, sleep-based waits or timing assertions."

Offending plan text:
- clock-inventory.md:137: "4. `refactor(raf-workplan)`: the Clock seam, ... `cargo workplan check`; record the evidence again if stale."

The evidence record's subject is `raf_workplan::test_crate::bound::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran` (work/evidence/gate-kills-a-test-past-its-timeout.md:6). Its inputs include `raf_workplan::exec` and `raf_workplan::exec::SystemRunner` (lines 17-34). Task 4 rewrites `SystemRunner::run` and `wait`, so the record will almost certainly go stale. At task 4, though, the subject is still the real-cargo test with a 5 s real limit. The plan converts that test only in task 7 (R5 P1). Recording the evidence in task 4 therefore means running a wall-clock timing test.

Hooks do not force an early re-record. `evidence-stale` is a change obligation (crates/raf-workplan/src/obligation.rs:24) and is not part of `Workplan::check`. What the hook does enforce is the currency of `work/assessments/workplan-complete-gate-contract.md`, whose inputs include `raf_workplan::exec`, `SystemRunner` and the evidence (lines 21-60).

REWRITE TO: "4. `refactor(raf-workplan)`: the Clock seam ...; `cargo workplan check`; assess again, in this commit, every assessment the seam stales (at least `workplan-complete-gate-contract` and the other complete-gate sections whose inputs name `raf_workplan::exec` or `SystemRunner`). Do not record `gate-kills-a-test-past-its-timeout` again here, because its subject is still the real-timer test. It is recorded again only in task 7, against the clock-free subject."

Also strike R5 P2 as a fallback (clock-inventory.md:108, "keep them as B (the real 5 s timer)"). It is itself a timing run under CLAUDE.md:18-19. Commit 7's "(P1/P2/P3)" (clock-inventory.md:140) becomes "(P1)", with P3 as the only alternative that has no timer.

### C2. ADVISORY: the per-commit assessment lists miss the assessments that the hook will find stale

Governing text:
- AGENTS.md:44-45: "the self-workplan test ... requires every assessment to be current and accepted"
- AGENTS.md:141-144: the pre-commit hook runs "the `repo_gates`, `docs_contract` and `self_workplan` test binaries"
- AGENTS.md:151-152: "Never bypass a hook or a gate."
- tests/self_workplan.rs:8-10: "a section that changed after it was assessed fails here until it is assessed again"

Offending plan text:
- plan.md:34-41: each task names only the assessment of the Verification section it edits (for example "okf-profile bindings, Verification, assessment").
- clock-inventory.md:119: "each edited Verification section needs its `work/assessments/<cap>-verification.md` recorded again."

An assessment also goes stale when the digest of one of its input symbols changes. The counter hooks (`add(...)` in `check_affects`, `visible_content`, `records.rs`, `graph.rs`, `coverage.rs`, `rationale`, `obligation`, `assessment`) and the runner seam change module digests. Those modules are inputs of many sections the plan does not name. Examples:
- all six `workplan-two-way-coverage-*` sections take `raf_workplan::coverage`;
- the `workplan-complete-gate` constraints, contract and examples sections take `raf_workplan::exec`;
- the inputs of `workplan-evidence-claims-*`, `workplan-entity-history-*` and `workplan-generated-docs-*` also name the touched modules.

Without these re-assessments a commit fails its hook, and the hook cannot be bypassed.

REWRITE TO (add to the "Tasks" preamble): "In each commit, after the code edit, list the assessments that are stale or not accepted (`cargo workplan assess --list`) and assess again every one that the commit stales, not only the edited Verification sections, so `self_workplan` passes in the hook."

No new record id is created, because all the sections named exist (tests/docs_contract.rs:103-160). So no record id needs to be added to that list.

### C3. ADVISORY: the per-task validation may run timing tests that are not yet converted

Governing text:
- CLAUDE.md:18-19 (quoted in C1).

Offending plan text:
- plan.md:33: "Tasks (each: focused validation, ...)"
- clock-inventory.md:133: "Each leaves `make check` green"

Until task 8 ends, `cargo test -p raf-workplan` and `make check` still run the remaining class-A/B tests: `sleep 30`, the 5 s cargo limits and the 60 s bounds. Checking that "each leaves `make check` green" locally is therefore a timing run.

REWRITE TO: "Focused validation in tasks 2-8 runs only the tests the task converts or touches, by exact filter. No whole-crate `cargo test -p raf-workplan` or `cargo-workplan`, and no `make test`/`make check`, before task 8 is committed. The intermediate commits are judged by the hooks and by hosted CI."

Also replace "each leaves `make check` green" with "each passes its hooks".

Two related points about mutant runs:
- The pidfd fallback (clock-inventory.md:148, "a mutant shows as a hang caught by CI `timeout-minutes`") detects a mutant by a wall-clock limit. Mutant runs should name only the tests that fail fast (the pure, stepped and counted tests), and should never be wrapped in `timeout`.
- CLAUDE.md:20 requires "tell every agent you start this rule". Add to the task-review step: "every task reviewer and mutant runner is told the no-timing-runs rule."

### C4. ADVISORY: the 30 s deadlock watchdog is kept as a wall-clock bound in a test

Governing text:
- CLAUDE.md:18-19 (quoted in C1).
- The plan's own rule for class C: budgets are "dropped; the runner's own limits and CI `timeout-minutes` are the watchdogs" (plan.md:17).

Offending plan text:
- clock-inventory.md:100: "`both_streams_are_drained_at_once` (the 30 s limit stays as a deadlock watchdog, no clock word)"

This is the same class C watchdog the plan removes everywhere else. Its pass or fail depends on host speed.

REWRITE TO: "`both_streams_are_drained_at_once` moves to `exec::system::tests` under `Frozen`. A deadlock then hangs and is caught by CI `timeout-minutes`, like the other frozen-clock tests. No test-chosen wall-clock limit remains."

The alternative is to state plainly that the test keeps the runner's default limit and asserts nothing about time.

### C5. ADVISORY: `kill -STOP $$`, and the trust surface of the Clock seam

Trust surface (SECURITY.md:146-160, 232-234): the plan does not change it.
- `Clock`, `WallClock` and `run_with` are `pub(super)` or private. The public `Runner`/`SystemRunner` API and `FakeRunner` are unchanged (clock-inventory.md:89).
- `WallClock` keeps the same reading point (just before `spawn`) and the same `sleep(pause.min(remaining))`.
- The test clocks are `cfg(test)`, so no document or caller can inject a clock.
- `/bin/sh -c` scripts as `Invocation::trusted` go through the crate-private `TrustedArgv::new` (exec/trusted.rs:110). The existing evidence tests already use that pattern. It adds no shell to the product, and it does not touch the compile-fail pin that `TrustedArgv` comes only from `std::env::args_os` (SECURITY.md:150-151).
- A shell as the process under test is a test input, not a check written in shell, so AGENTS.md:207-209 ("Rust first") is not breached.

Keep it that way. Add to task 4: "no constructor of `TrustedArgv` or `Invocation::trusted` and no clock parameter becomes `pub`; SECURITY.md is unchanged because the limit's behaviour is unchanged."

`kill -STOP $$` is a deterministic signal. That fits the issue ("a count or a deterministic signal can replace the wait", issue-analysis R6) and CLAUDE.md:18-19. On Linux, the primary platform (CLAUDE.md:21), it is still unsafe as the way to park a descendant whose leader exits.

In test 5 (clock-inventory.md:96, `'...STOP' & exit 0`), the leader's exit orphans the process group. POSIX job-control semantics then send SIGHUP and SIGCONT to a stopped member. Whether the grandchild dies before the runner's kill therefore depends on whether it had stopped yet when the leader exited. As a result:
- correct code can see both streams end and return `Done`, where the test expects `Timeout`;
- the "drop `kill_group`" mutant can pass through the "entry gone" branch.

Tests 1 and 3 keep a live leader, so their group is not orphaned.

REWRITE TO (R3, clock-inventory.md:90): "a process that never ends blocks on a read of a second FIFO that the test holds open and never writes (`read _ < "$hold"`, a shell builtin). It does not stop itself, so no job-control signal is involved. After the group kill the test closes the FIFO."

Dependency and lint note on the FIFOs:
- The plan does not say how the FIFOs are made, and std has no `mkfifo`.
- Any route through `libc` or `nix` would be a new dependency, and an `unsafe` call (AGENTS.md:174-179: "`unsafe_code` [is] denied ... Library code has one `unsafe` block ... The only other one is the counting allocator").
- REWRITE TO: "The FIFOs are made by the script itself (`mkfifo`, before writing the pid) or through `rustix` added as a `[target.'cfg(unix)'.dev-dependencies]` entry with the `fs` feature. No new crate, no `unsafe`."
- Likewise, test 4 must set `PATH` only on the child `Command`. `std::env::set_var` is `unsafe` in edition 2024 (Cargo.toml:30) and would hit `unsafe_code = "deny"`.

### C6. ADVISORY: how the declines and the coverage issue read against "no deferrals"

Governing text:
- AGENTS.md:266-268: "**No deferrals.** Work is done now. Scope that leaves a change is moved to another tracked change, with its own record, before the first one completes. Do not write "later", "follow-up" or "out of scope for now" as a substitute for either."
- CLAUDE.md:27.

Coverage index (plan.md:13, 46): compliant as a decline.
- The capability promises only that "a crate with thousands of items and tests is judged in bounded time" (workplan-two-way-coverage.md:250-251).
- A pinned `entries x exclusions` product keeps that guarantee, so indexing is a new improvement, not scope leaving this change.
- Two conditions apply. The issue filed in task 10 must be described as a separate proposal, not as moved scope of this change. No committed text (Verification, test doc, commit message) may say "follow-up" or "later".
- REWRITE TO (plan.md:13): "...is declined: the capability's contract is bounded time, which the pinned product keeps; the product is stated in the Verification text; indexing is proposed separately as its own issue."

Counting newtype accessors (plan.md:14, 47): compliant only if the counted tests still catch the regression the deleted probe guarded.
- The inventory says the original raf-okf regressions are of the uncounted-loop kind (clock-inventory.md:146).
- The plan's listed mutants are counted-path variants ("build `listed` inside the affects loop", "build `by_name` inside the limits loop", clock-inventory.md:78).
- A mutant restoring the original shape keeps the set and the map built, but answers each lookup by `changes.iter().any(..)` or `values.iter().find(..)`. Under the accepted limit that mutant would pass, and the probe's guarantee (issue R12, R8) would be dropped, not kept.
- REWRITE TO: "Add the original-shape mutants (a scan of `changes` for each affects entry; a search of `values` for each limit) to task 1's mutant list. If either survives, the decline is not available: put the counted step on the lookup itself, or adopt the accessors."

The `perf stat` target (plan.md:48): compliant. Perf is one of the issue's alternatives ("an operation count, an allocation count or a `perf` measurement").

Loose items: the inventory's "Optional: a counting `$CARGO` wrapper" (clock-inventory.md:113) and "rename optional" (clock-inventory.md:124) have no disposition. Give each an explicit one (declined, or done) so nothing is left open.

### C7. ADVISORY: small convention points (all already compliant or cheap)

- **SPDX headers.** The new files (`crates/raf-okf/src/work.rs`, `crates/raf-workplan/src/work.rs`, `profile/cost_tests.rs`, `coverage/tests.rs`, `exec/clock.rs`, `exec/system/tests.rs`) need the code SPDX header (AGENTS.md:196-198). The reuse hook enforces it; add it to the task text.
- **Gate rename.** Renaming `time_in_raf_source_` to `time_in_crates_` must edit the prefix list in the module doc (tests/repo_gates.rs:25), which the plan names. It must also edit the gate's description at repo_gates.rs:19-21. AGENTS.md:46-48 lists the repository gates without the clock gate; task 9's "AGENTS/CONTRIBUTING mention" covers that.
- **Gate fixtures.** The fixtures planned for the extended gate meet AGENTS.md:215-219 ("Every rule has a negative fixture" / "A refusal is a claim too"): violation, clean neighbour, exemption, and a stale-exemption assertion. The module's "Only the standard library is used" (repo_gates.rs:26-27) must stay true.
- **Inventory table.** No capability changes state and no capability is added, so `INVENTORY` in tests/docs_contract.rs is untouched (AGENTS.md:236-238). The evidence id stays (docs_contract.rs:160).
- **Issue's acceptance grep.** Run the issue's literal grep as well as the extended gate, and show that its only hit is `exec/clock.rs` with no assertion. This avoids replacing an acceptance criterion silently (plan.md:18).
- **Issue numbers and process wording.** `Closes #21` belongs only in the PR body. `.stage/` is ignored (.gitignore:5), so `validation.md` and the plan are never committed. The planned committed names and texts contain none of the patterns of `raf_workplan::refs`.
- **Platforms.** "Linux-only checks assert only on Linux; other systems keep the error assertions" (plan.md:16) matches CLAUDE.md:21-24.
- **Test names.** The planned names follow the existing sentence style. No written rule beyond the gate prefixes in repo_gates.rs:24-25 governs them.

## Summary

There is one BLOCKING finding. The plan's commit order schedules an evidence re-record in task 4, and at that point the only possible subject is the real-timer test, which CLAUDE.md:18-19 forbids running. The fix is cheap: in task 4, assess the stale assessments again, record the evidence again only in task 7, and strike P2.

The advisories:
- C2: assessments the hook will find stale.
- C3: timing tests reached by whole-crate validation before task 8.
- C4: the kept 30 s watchdog.
- C5: the orphaned-group race of `kill -STOP $$` in test 5, and how the FIFOs are made without a new crate or `unsafe`.
- C6: an original-shape mutant that decides whether the newtype decline is allowed.
- C7: small convention points.

The Clock seam does not change the trust surface in SECURITY.md. No new dependency is required.

VERDICT: NEEDS-REVISION
