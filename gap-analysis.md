# Gap analysis (Stage 2, combined)

Issue: 21. PR: 30. Branch: paudley/21-replace-wall-clock-assertions-in-tests at 77f3ba8. Base: main. 95 files changed, 16 commits.
Method: read-only on the worktree. One scratch clone (/tmp/gap21-mut-ezcm0l/repo, `git clone --no-hardlinks`, pwd checked, reset after each mutant). No timing runs, no `timeout` wrapper, no rm -rf.

## Execution evidence (this review, HEAD 77f3ba8)
| Command | Result |
| --- | --- |
| `cargo check --workspace --all-targets --locked` | exit 0 |
| `cargo check --manifest-path fuzz/Cargo.toml --locked --all-targets` | exit 0 |
| `make test` | exit 0; 2075 passed, 0 failed, 2 ignored (`write_corpus` x2, by-hand generators, pre-existing); all 10 `time_in_crates_*` ran and passed |
| `make test-release` | exit 0 (raf-source only; the raf-okf release probe is gone) |
| `make clippy` | exit 0 (`disallowed-types`/`disallowed-methods` in force) |
| `make issue-refs` | exit 0 (67 passed) |
| `grep -rn "Instant::now" crates/*/tests crates/*/src` | one hit, `crates/raf-workplan/src/exec/clock.rs:42` (the reading, not an assertion) |
| `grep -rnE 'Instant\|SystemTime\|sleep\|recv_timeout' crates/raf-source/{src,tests}` | empty |
| hosted CI run 39179289785 (head 77f3ba8) | 11 of 12 pass; **test-macos FAILS** (step "Test the process group kill on macOS") |
| mutant: drop the first guard of `read_regular` (tests/support/mod.rs) | `refuses_non_regular_files_instead_of_reading_them` FAILS at once: `null: Err("/dev/null is not a regular file once opened")` (T5678 B1 fix holds) |
| mutant: plant `wait_timeout_while(..)` and `thread::park_timeout_ms(5)` in crates/raf-workplan/tests/exec.rs (staged) | `time_in_crates_tracked_files_name_no_clock` PASSES (gate gap, see M1) |

## Requirements coverage
| # | Requirement | Status | Location | Gap |
| --- | --- | --- | --- | --- |
| 1 | raf-source items (issue list + comment) | COMPLETE | earlier merged change; gate rule 1 | - |
| 2 | raf-okf `validate_cost.rs` release bound | COMPLETE | crates/raf-okf/src/{work.rs,profile/cost_tests.rs}; tests/validate_cost.rs clock-free; Makefile/CI/AGENTS/CONTRIBUTING updated | - |
| 3 | raf-workplan assessments/rationale/graph/obligations/records | COMPLETE | src/work.rs (`Counted`), unit tests in graph/rationale/assessment/obligation | inherent N2 limit (cost of one lookup) stated |
| 4 | coverage | COMPLETE | coverage.rs ticks, coverage/tests.rs exact product | index declined, issue 29 filed |
| 5 | runner timeout and output-kill tests deterministic | PARTIAL | exec/clock.rs, exec/system.rs (`run_with`, `verdict`, `next_pause`), exec/system/tests.rs | Linux green; the two Frozen output-kill tests fail on macOS CI (H1) |
| 6 | evidence elapsed bound on a killed command | COMPLETE | src/evidence/tests.rs (FakeRunner scripted elapsed) | - |
| 7 | never-ending cargo tests | COMPLETE | exec/system/tests/cargo.rs (Linux), evidence record re-recorded | - |
| 8 | budgets, nonces, mtimes, root docs_contract | COMPLETE | tests/*, tests/support/mod.rs | - |
| 9 | gate | PARTIAL | tests/repo_gates.rs `time_in_crates_*`, clippy.toml | suffixed timeout waits pass (M1); dirs hard-coded, not derived from `crates/*` (M2) |
| 10 | every converted test fails on a mutant | COMPLETE | validation.md tables; re-confirmed T5678 B1 here | some "fail fast" claims are "fail after the mutant's own quadratic run" (L2) |
| 11 | `make check` passes, hosted CI green | NOT MET | - | test-macos red (H1) |

## Acceptance criteria (issue)
| Criterion | Status | Evidence |
| --- | --- | --- |
| `grep -rn "Instant::now" crates/*/tests crates/*/src` finds no assertion on elapsed time | MET | single hit is the reading in exec/clock.rs |
| Every converted test still fails on a mutant that removes its bound | MET | validation.md (Tasks 1-8 tables); FIFO mutant re-run here |
| `make check` passes and hosted CI is green | NOT MET | local test/test-release/clippy/issue-refs exit 0; CI test-macos failed on 77f3ba8 |

## Gaps

### HIGH
**H1. Hosted CI is red: test-macos fails two moved runner tests.** Run 39179289785, step `cargo test -p raf-workplan --locked --lib exec::system::`: 15 passed, 2 failed.
* `exec::system::tests::output_at_the_bound_passes_and_one_byte_over_is_refused_on_each_stream` (tests.rs:1003) and `a_process_that_never_stops_writing_is_killed_at_the_bound` (tests.rs:1036) get `Io { .. was killed; .. the process could not be killed, so it may still be running: Operation not permitted (os error 1) }` instead of `OutputTooLarge`.
* Cause, by reading `kill_group` (system.rs:356-366): on macOS `killpg` answers EPERM for a group whose only member is exiting (the shell after `printf`, or `yes` after SIGPIPE), and `child.try_wait()` has not seen the exit yet, so `group_signal_outcome` reports the error. These tests used to live in `tests/exec.rs`, which the macOS job never ran; they now sit under `exec::system::`, which it does run, and under `Frozen` (a yield, no sleep) the runner reaches the kill right after the overflow, which hits the window. This is likely a production race on macOS that the change exposed, not one it caused.
* CLAUDE.md says a platform-specific failure does not block a change, but the issue's acceptance and the plan's contract ("hosted CI 12 of 12") are not met, and the PR body says "the kill-gate tests are Linux-only", which does not match: these output-kill tests run on macOS and fail. The fix has to be tracked somewhere: either fix the EPERM-while-exiting decision, or file a platform issue and record it in the PR. Silently narrowing the macOS CI filter would weaken a gate.

### MEDIUM
**M1. The lexical gate misses timeout waits that have a suffix.** `has_word` needs an identifier boundary, so `Condvar::wait_timeout_while`, `thread::park_timeout_ms` and `thread::sleep_ms` all pass. I confirmed this with the planted mutant above. `clippy.toml` bans only `Instant`, `SystemTime` and `std::thread::sleep`, so neither layer catches these, and neither sees a non-zero `Timespec` given to `rustix::event::poll`. AGENTS.md says the gate refuses "the `recv_timeout`, `wait_timeout`, `park_timeout` ... calls", which a reader would take to include these variants. Fix: match these as prefixes, or add `wait_timeout_while`, `wait_timeout_ms`, `park_timeout_ms` and `sleep_ms` to the words and to `disallowed-methods`, with fixtures.

**M2. The gate's directories are hard-coded.** Revision 1, enhancement 4 was marked as taken: "the lexical gate deriving its directories from `crates/*`". `CLOCK_RULES` instead lists eight fixed directories, and the fixtures only walk that table. A new crate (for example the planned `raf-census`) would get only the clippy subset, with no `*_timeout` or `UNIX_EPOCH` ban. Root `src/` and `fuzz/` are also outside the gate; both are clean today. Fix: derive the directories from `crates/*/{src,tests,examples}`, keep raf-source's stricter rule and the raf-source examples carve-out, and add a fixture that fails when a crate is not covered.

**M3. Overclaims about real clock use.** validation.md:247 says "The real clock is read in tests only where `WallClock` is the clock of the machine (the list pass of `cargo.rs`)", and the PR says "No test ... reads a clock any more". In fact every test that runs a real process through `SystemRunner::run` (tests/exec.rs stubs, tests/bound.rs, complete/verify/scenarios, the cargo-workplan tests, self_workplan) goes through `WallClock`: it reads `Instant` and runs real `thread::sleep` pauses of up to 20 ms per look, under 5- and 30-minute limits. These are watchdogs, not assertions. Removing the pause schedule was declined in the plan, so this is acceptable, but the PR and validation text should say it exactly: test code reads no clock, and the production runner under test still does.

### LOW
* **L1.** Issue 29's body proposes "an index so the work grows with entries plus exclusions". Plan Revision 2 B7(8) promised the issue would state the constraint that the Verification text states: every match is marked so that unused exclusions can be reported, so an index must still return all matches. The issue omits it. The PR says "follow-up filed" without naming #29 (a number is allowed in the PR body).
* **L2.** The PR says "each fails fast, none hangs". validation.md Task 1 records the two deleted-index mutants failing only after their own quadratic scan has finished, and the fence-restart mutant could be run only with the two fence tests. None of them hangs, but "fails fast" is too strong; say "fails, none hangs".
* **L3.** CONTRIBUTING.md says "Only `crates/raf-workplan/src/exec/clock.rs` and the gate file are exempt", but `crates/raf-source/examples/` is outside every rule (registry_scan.rs reads `Instant` and sleeps, with a clippy allow). AGENTS.md states the scope correctly; CONTRIBUTING should name the examples carve-out.
* **L4.** The allow reason in registry_scan.rs says "moving it to a scripted clock is a change of its own". It is tracked (issue 27, open) and `make issue-refs` passes, but the text reads as process wording. A plain statement would be better, for example "an example binary that polls a real child; it is not a test".
* **L5.** The repo_gates.rs module doc line 30 is far longer than the surrounding wrap (cosmetic).
* **L6.** In exec/system/tests.rs:790 a zero `Timespec` is named `now`. The poll does not block, but the name suggests a clock reading; `zero` or `no_wait` would read better.

## Previously fixed items, re-checked on HEAD
| Item | Holds | Evidence |
| --- | --- | --- |
| T1 F1 (tick where the work is done) | yes | profile/mod.rs:604 `.inspect(ChangeIdsSet)`, :644 `ListedLookups`; records.rs:595/599 |
| T2 M1-M3 (sets reachable only through `Counted`) | yes | graph.rs:1345-1367, obligation/sources.rs:202/211 |
| T45 B1 (non-blocking pid read) | yes | tests.rs:573 `OFlags::NONBLOCK` |
| T45 B2 (look ceilings) | yes | tests.rs:288-289, 351-352 (`looks` and `pauses` counts) |
| T45 B3 (only SIGKILL passes) | yes | tests.rs:618 `trap '' HUP INT TERM`, :650 SIGCONT |
| T45 B4 (drain under Frozen) | yes | tests.rs:1049; gone from tests/exec.rs |
| T5678 B1 (root FIFO guarantee) | yes | support/mod.rs:89 "once opened"; mutant re-run fails |
| `.deficiencies` | empty below marker | - |

## Shell sleeps and timed waits in fixtures
* No shell `sleep`, `read -t`, `timeout` command, `usleep`, `nanosleep`, `alarm` or timerfd appears in any fixture string or non-Rust tracked file of the four crates or the root tests.
* The `while :; do :; done` subjects (tests.rs:503-505) are for non-Linux Unix only, under the stepped clock: they spin, but no time is asserted.
* `sync.rs:253 "state: sleeping"` is a document value, not a wait.
* `preview.rs:437` moves a file's own mtime back by one hour (`set_modified`). It reads no clock.
* Fuzz corpus seeds quote documentation text ("timeout of 300 seconds"). That is data, not a test.

## Docs spot check
* okf-profile Verification matches cost_tests.rs (exact counts for `ChangeIdsSet`, `AffectsEntries` and `ListedLookups`, a ceiling at `count` for the limits, the `assert_linear` idiom, the fence counts, and the limit that uncounted loops are not seen).
* workplan-two-way-coverage Verification matches `exclusion_for`, which marks every match and keeps the first, and the four coverage tests.
* workplan-complete-gate Verification matches cargo.rs and system/tests.rs on Linux. It does not mark the Frozen output-kill statements as Linux-only, and those fail on macOS (H1).
* README needs no change. AGENTS.md is accurate. CONTRIBUTING.md has L3.

## Consumers
* `cargo check --workspace --all-targets --locked` and the fuzz workspace check pass, and so do `make clippy` and `make test`.
* `work.rs` counters compile to nothing outside `cfg(test)`. `Counted` ships as `pub(crate)`, with no public API change.
* The rustix `fs`/`event` features are dev-only.

## Priority ranking
1. [HIGH] H1: test-macos red on 77f3ba8 (EPERM while the leader exits, in `kill_group`). Fix it, or track it as a platform issue and correct the PR disclosure.
2. [MEDIUM] M1: the gate misses suffixed timeout waits (`wait_timeout_while`, `park_timeout_ms`, `sleep_ms`).
3. [MEDIUM] M2: gate directories are not derived from `crates/*`, which deviates from Revision 1.
4. [MEDIUM] M3: the "no test reads a clock" and validation.md:247 claims are inaccurate (the runner under test uses WallClock).
5. [LOW] L1-L6.

## Completion verdict
| Criterion | Verdict | Evidence |
| --- | --- | --- |
| (a) no clock word outside exec/clock.rs and tests/repo_gates.rs; other Duration-based waits; shell sleeps; Makefile/CI probes | PASS (tree) / gate has holes (M1, M2) | greps above; release probe removed from Makefile and CI; gate mutant passes |
| (b) make test / test-release run with no timing | PASS | both exit 0; no timing assertion left; WallClock remains only as the runner's watchdog (M3) |
| (c) issue 29 exists; disclosures match reality | PARTIAL | issue 29 open; body lacks the all-matches constraint (L1); PR's Linux-only claim contradicted by macOS CI (H1); "fails fast" overstated (L2) |
| (d) docs accurate; README/AGENTS/CONTRIBUTING consistent; no issue/PR numbers | PASS with LOW notes | spot checks above; `make issue-refs` exit 0; L3, L4 |
| (e) previously fixed review items hold | PASS | table above; FIFO mutant re-run fails as expected |
| (f) consumers build | PASS | workspace and fuzz `cargo check` exit 0 |
| Issue acceptance "hosted CI green" | FAIL | test-macos failed, run 39179289785 |

Overall: **NOT COMPLETE.** On Linux the work is functionally complete. The one blocker is H1, because hosted CI is not green. Under the CLAUDE.md platform rule it may be handled by a tracked macOS issue rather than fixed here, but that decision and the corrected PR disclosure are still needed. M1-M3 are small and fit in this change.
