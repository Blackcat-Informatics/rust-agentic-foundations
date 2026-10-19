# T5 fix, T6, T7, T8 review (commits fbf3d0a, d4f1c3e, e7a97d0, 88d9e42)

VERDICT: BLOCKED

One item blocks. The root FIFO test that Task 8 rewrote no longer protects its guarantee: a loader that blocks
on a FIFO is not caught. The plan accepted that loss on the premise that "detecting a block needs a timer", and
that premise is false. A clock-free fix of a few lines exists; I tried it in a scratch clone and it catches the
mutant. Everything else passes, with evidence below: the Task 5 fix, the Task 7 production hook and test runner,
the Task 6 scripted runner, the budgets and nonces, the preview snapshot, the evidence record and assessments,
the repo-wide grep, and the gates.

Method. Two scratch clones at 88d9e42 (`git clone --no-hardlinks` into /tmp/t5678-review-tICn62/{base,mut}).
`pwd` was checked before every edit, and every mutant was reverted with `git checkout -- .` in the clone. No edit
was made in the worktree apart from this file. No `timeout` wrapper and no `rm -rf` were used. Each mutant ran
only its named test. Every failing run failed at once and none hung. `ps` showed no fixture or subject process
left after any batch. No `/tmp/raf-workplan-runner-*` directory came from these runs; the five that exist are
from 16:00-16:13, left by earlier reviews. While I worked, Task 9 committed 01a15d8. That commit does not touch
`tests/docs_contract.rs` or `tests/support/mod.rs`, so the blocker stands at the new HEAD.

## Blocking

**B1. The guarantee of the root FIFO test is lost, and it could have been kept without a clock.**
* Where:
  * `tests/docs_contract.rs:1339-1343`: the doc comment says "refused by its inspection, before it is opened".
  * `tests/docs_contract.rs:1358`: the assertion accepts "not a regular file".
  * `tests/support/mod.rs:88-90` (`check_handle`) gives the same message as the first guard at
    `tests/support/mod.rs:122-124`.
* Confirmed by running the mutant that drops the first guard of `read_regular` (the `symlink_metadata` type
  check before `open`). `refuses_non_regular_files_instead_of_reading_them` **passes**: `/dev/null` and the
  directory both open without blocking, and the second guard on the handle gives the identical text.
* That mutant is exactly the regression the old FIFO test guarded: without the first guard, `open(O_RDONLY)` on
  a FIFO with no writer blocks for ever. `validation.md` (Task 8 table, last row) calls it "equivalent; the two
  guards are one contract". That is wrong. The doc comment claims something the test does not check.
* The issue requires that each test keep the guarantee it protects and fail on a mutant that removes it. This
  test does not.
* Clock-free fix, tried in the mut clone:
  * Make `check_handle`'s refusal distinguishable (for example "… is not a regular file once opened").
  * Assert that the `/dev/null` refusal is the inspection's message (`!e.contains("once opened")`).
  * Result: on clean code all 3 `refuses_` tests pass. With the first guard dropped, the test fails at once:
    `null: Err("/dev/null is not a regular file once opened")`.
  * A FIFO subject can also be added if the test opens it `O_RDWR` first: no `open` can then block, and the
    message tells the two guards apart.
  * No timer is needed and nothing can hang. The "loss stated for the PR" in validation.md then goes away.

## (1) Task 5 fix: three recorded mutants plus a flakiness run
| Mutant (system.rs) | Test | Observed |
| --- | --- | --- |
| group signal TERM (`kill_process_group(pid, Signal::TERM)`) | what_the_process_started, a_descendant_that_holds, no_kill_program, cargo test case | each `a process ran on past the kill and wrote the marker: Settled { leader_reaped: true, marker: true }`, 0.00-0.63 s |
| no pause once at the cap (`if pause < POLL_CEILING { clock.pause(..) }`) | outlives_its_limit, a_descendant_that_holds | `the run made 5000001 looks before the gate opened, past the ceiling of 5000000` (about 2 s) |
| the same | cargo test case | `the run made 501 looks under a stepped clock, past the ceiling of 500` |
| argv[0] kept (`argv.remove(0)` replaced by `argv.truncate(argv.len())`) | outlives_its_limit, a_descendant_that_holds, no_kill_program, what_the_process_started | `the run ended before the gate opened on the processes of its script: Ok(Output { code: Some(126), .. })`, 0.00 s |

`cargo test -p raf-workplan --locked --lib -- exec::system::` ran 20 times in a row: **20 of 20 passed**, 19
tests each (17 plus the 2 cargo tests).

## (2) Task 7
**The `Clock::spawned` hook does not change production behaviour.** I compared the whole production diff from
6de7de3 to 88d9e42: it is only `clock.rs` (+5, a default empty method) and `system.rs` (+1).
* The call (`system.rs:131`) comes after `command.spawn()?`. If the spawn fails, `?` returns first and the hook
  never runs.
* It comes after `process_group(0)` has taken effect, since `spawn` returns only after exec. The group is
  already the child's own, so a test clock reads the right `pgrp`.
* It comes before the pipes are taken and the captures start. With `WallClock` it is a no-op: `WallClock` does
  not override it. A test clock that is slow there can only make the child block on a full pipe, which the
  captures then drain.
* The child cannot be reaped before the hook runs, because `try_wait` first happens in `wait`. So the pidfd that
  `Watch::spawned` takes names the right process.

**Widening the list pass to 1800 s in `Clocked` is sound.**
* A real 5 s limit on a real `cargo test -- --list` would make the outcome depend on host speed, which is the
  thing this issue forbids. The 1800 s limit is a watchdog, not an assertion.
* The limits the executor asked for are still pinned through `asked` (both passes at 5 s, the example at 5 s).
* A production mutant that dropped `--list` would send the list pass to the stepped clock. The second run would
  then panic ("one run under the stepped clock").

**The two new tests, 10 runs each with exact filters:** test case 10 of 10 passed, example case 10 of 10 passed.

**Mutants:**
| Mutant | Test | Observed |
| --- | --- | --- |
| only the leader killed (`rustix::process::kill_process(pid, KILL)`) | T | `a process ran on past the kill and wrote the marker`, 1.74 s |
| the same | E | passes, as recorded (`cargo run` execs the example, which is then the leader) |
| `Timeout.elapsed` from a fresh `WallClock` | T, E | `the reading of the stepped clock at the kill`: left 370ns / 380ns, right 5s |
| own: the executor maps a timeout to `Exited(Some(0))` with a "1 passed" output and no failure (`bound/run.rs:120`) | T, E | `left: Exited(Some(0)) right: TimedOut` at once (T: "unresolved ... the result line of the run pass cannot be read"; E: "pass ...") |

**Fixture sources** (`cargo.rs:315-370`, generated `lib.rs`, `examples/forever.rs`, `Cargo.toml`): no sleep, no
park, no timer and no dependency. `outlive()` ignores signals 1, 2 and 15, opens the pid FIFO (held `O_RDWR` by
the test, so the open cannot block), writes its pid, blocks on `read` of the hold FIFO, then writes the marker.

**Evidence and assessments (in the base clone):**
* `workplan evidence for cap:workplan-complete-gate` gives `evidence:gate-kills-a-test-past-its-timeout pass ...
  (in its for list; claims at lines 436)`, exit 0.
* `workplan check` exits 0. `assess --list` shows every section `current`, exit 0.
* The record's subject is the new binding `raf_workplan::exec::system::tests::cargo::a_test_that_never_ends_...`,
  which is listed in the capability's `tests` bindings. Only the `raf_workplan::exec` digest changed.
* The claim marker (`workplan-complete-gate.md:436`, `evidence=gate-kills-a-test-past-its-timeout`) refers to the
  record by id and still sits under the paragraph about the timeout kill.

## (3) Task 6
* `git diff d4f1c3e~1 d4f1c3e --stat` shows only `src/evidence/tests.rs` and the seven
  `workplan-evidence-claims-*` assessments. Each assessment change is one digest line (487e9f… to debc1f…). Nothing
  stray was left by the two mistaken worktree edits.
* Mutant: the evidence executor asks `Limits::trusted()` with a 48 h timeout (`evidence.rs:638`). Result:
  `left: Pass, right: Fail` at `evidence/tests.rs:404`, 0.01 s.
* Clean: `evidence::tests::` 17 passed.
* Note: the neighbour "at exactly `BUILD_TIMEOUT` passes" pins the boundary of the fake (`fake.rs:132`, `>`).
  Production uses `>=` only for a run that is still going and lets Done take precedence. A scripted `Output` is a
  finished run, so the two agree, but production's boundary is pinned in `exec::system::tests`, not here.

## (4) Task 8
* **Budgets:** `grep -rnE 'BUDGET|within_budget|\btimed\('` over the three crates and the root `tests/` finds
  nothing.
* **Nonces:** no `nanos` or `now()` is left outside `clock.rs` and `repo_gates.rs`. `root.rs`, `git.rs`, `gate.rs`,
  `bound.rs` and `docs_contract.rs` use pid plus a counter, with `create_dir` retried on `AlreadyExists`.
  `root`, `git` and `gate` pass (18, 22 and 8 tests).
* **Preview:** the rewrite still proves that preview writes nothing.
  * Mutant: `Workplan::preview` sets the modification time of each previewed file under the fixture root to now.
    `a_disk_fixture_is_byte_identical_afterwards` fails at once at `preview.rs:463` (the snapshots differ). Clean,
    it passes.
  * Advisory: `age` (`preview.rs:424-430`) ages files only. Directory stamps are captured but not aged, so a
    create-then-delete inside a directory within one coarse timestamp tick could go unseen on a kernel without
    multigrain timestamps. The old snapshot did not record directories at all, so this is not a regression.
* **Root FIFO test:** see B1.

## (5) Repo-wide grep
`grep -rnE 'Instant|SystemTime|UNIX_EPOCH|thread::sleep|recv_timeout|wait_timeout|park_timeout' crates/raf-okf
crates/raf-workplan crates/cargo-workplan tests --include=*.rs` (at 88d9e42):
* `crates/raf-workplan/src/exec/clock.rs:13,30,35,45`: the one production reading and sleep (permitted).
* `tests/repo_gates.rs:263,265,267,268,269`: the gate's word table (exempt).
* `tests/repo_gates.rs:859,870,879,880,881,884,885`: fixtures of the gate (exempt).
* `tests/repo_gates.rs:907`: the clean neighbour `Instantiate` (exempt).
* Nothing else.
* The literal `grep -rn "Instant::now" crates/*/tests crates/*/src` finds only `clock.rs:35`.

## (6) cfg(test) and non-Linux builds, by reading
I found no new dead-code or unused-import risk.
* Every Linux-only item is gated with `target_os = "linux"`: `Subject`, `Gate`, `Opening`, `Seen`, `Process`,
  `Settled`, `Stepped::gated`, the `ceiling`/`gate`/`seen` fields, and `mod cargo`.
* On other Unix systems, `Stepped::new`, `moving`, `stepped_ceiling`, `Scratch`, `Shape` and `Outlived {command,
  result}` are all used, and the `spawned` override discards `process`.
* On Windows, the pure tests use `io`, `Cursor`, `Duration`, `MS`, `Stream` and `Capture`.
* Pre-existing and not changed here: `evidence/tests.rs` `Bounded` is used only by a `cfg(unix)` test, so it is
  dead on Windows, as it was before d4f1c3e. CI runs clippy with `-D warnings` on Linux only.

## (7) Text
Three Verification statements, checked against the tests:
* (a) "only SIGKILL to the group passes" for the test binary: the TERM and leader-only mutants fail the test case.
* (b) "the list pass ... under the clock of the machine and the limit of the gate": `Clocked::run`, `cargo.rs:90-93`.
* (c) "pinned word for word ... exactly 5 seconds": `killed_at_the_limit`, `cargo.rs:450-466`.

All three are true. The fbf3d0a text ("a clock of the machine that always read zero would pass every one of these
tests") is also true, since the list pass under `WallClock` does not depend on its reading.

Over all four commits, the added lines contain no issue or pull request numbers and no process wording. The one
new file (`exec/system/tests/cargo.rs`) has its SPDX header. The CI comments (macOS and Windows) are accurate.

## Advisories (not blocking)
* A1. `cargo.rs:499` `wall() >= LIMIT` compares a sum that includes a `WallClock` reading from the list pass. It
  cannot fail on timing, because the stepped 5 s alone meets the bound. The Task 9 listing of surviving
  comparisons must still classify it.
* A2. The yield-counted ceilings (`FROZEN_CEILING` = 5,000,000, under `Frozen` and before the gate opens) depend
  on host speed in principle. About 14,000 looks are seen per run, so there is roughly 350x headroom; the cargo
  gate blocks on `poll` instead.
* A3. `cargo.rs:162`: `Watch::turn` blocks in `poll(None)` on the pid FIFO and the pidfd of `cargo`. A `cargo` that
  waits on a build-directory lock held by another process waits with it; that is environmental and not a mutant.
* A4. The preview directory stamps, see (4).

## Gates (base clone at 88d9e42)
* `cargo fmt --all --check`: exit 0.
* `cargo clippy -p raf-workplan --locked --all-targets -- -D warnings`: exit 0.
* `cargo test -p raf-workplan --locked --test bound --test exec --test preview`: 17, 35 and 23 passed, exit 0.
* `make deny`: advisories ok, bans ok, licenses ok, sources ok, exit 0.
* Also run: `--test root --test git --test gate` (18, 22 and 8 passed); `--test docs_contract -- refuses_` (3
  passed); `--lib -- evidence::tests::` (17 passed).
