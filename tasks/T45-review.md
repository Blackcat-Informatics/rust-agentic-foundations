# T4/T5 review (commits fb79f72, 6de7de3)

VERDICT: BLOCKED

Task 4 preserves the runner's behaviour, and every mutant the brief asked for fails at once. Task 5's protocol
is right for the correct code, but it does not meet the plan's rule "no wait can hang; a hang is a failure to
fail" (Rev 1 F1/C5, Rev 2 B3). Three plausible mutants hang the test binary forever (two of them confirmed by
running them). One plan disposition (Rev 1 C4, the drain test) was not carried out. Each blocking item has a
small fix, and the fix for B3 was tried in a scratch clone.

All work was done in scratch clones (`git clone --no-hardlinks` into /tmp/t45-review-SzxqIS/{base,mut}). There
were no edits in the worktree and no `timeout` wrapper. A hung run was confirmed from `/proc` (task state and
wchan) and then SIGKILLed by pid. Two `/tmp/raf-workplan-runner-*` directories are left over from those two
killed runs.

## Gates (clone at 6de7de3)
* `cargo fmt --all --check`: exit 0.
* `cargo clippy -p raf-workplan --locked --all-targets -- -D warnings`: clean.
* `cargo test -p raf-workplan --locked --lib -- exec::`: 25 passed.
* `cargo test -p raf-workplan --locked --test exec`: 36 passed.
* `make deny`: advisories ok, bans ok, licenses ok, sources ok (exit 0).

## (a) Task 4 is behaviour-preserving: yes
I compared the old loop (`git show fb79f72^:crates/raf-workplan/src/exec/system.rs` 221-277) with the new one
(system.rs:283-334):
* The overflow check still comes first in each look, before `try_wait` and the reads. `overflow()` loads stdout
  then stderr, left to right, and reports stdout first, so when both overflow it is still stdout. The old early
  return skipped the stderr load; that difference cannot be observed.
* The order after the reads is the same: overflow, then done (status and both streams), then late.
* Late: the old `timeout.saturating_sub(elapsed).is_zero()` is the same as the new `elapsed >= timeout`. The
  pause still uses that one reading (`pause.min(timeout - elapsed)`). The schedule is unchanged: 1 ms, doubling,
  capped at 20 ms. The first look still happens before any pause.
* One extra `clock.elapsed()` now runs on the looks that end Done/Over. The value is discarded; with WallClock
  it cannot be observed.
* The `elapsed` field of Output, OutputTooLarge and Timeout is still read once, after `wait` returns and before
  `abandon`/`terminate` (system.rs:148/155/160), as `started.elapsed()` was. The clock starts at the same point,
  just before `spawn` (system.rs:124).
* `#[cfg(all(test, unix))] mod tests` became `#[cfg(test)] mod tests;`, with the unix gates moved inside the
  file.

## (b) The only clock reading is clock.rs: yes
* `grep -rnE 'Instant|SystemTime|thread::sleep|sleep\('` over the src of raf-okf, raf-workplan and
  cargo-workplan, test files excluded: only `crates/raf-workplan/src/exec/clock.rs:13,25,30,40`.
* `recv_timeout|wait_timeout|park_timeout|\.elapsed\(\)` outside tests: only clock.rs:36 and the
  `clock.elapsed()` calls in system.rs.
* One hit remains in test code: `src/evidence/tests.rs:16,404` (`Instant`). That belongs to a later task (6 or
  8) and is outside T4/T5.

## (c) Protocol soundness
For the correct code the protocol holds:
* The FIFOs are opened O_RDWR before the spawn, so neither the script's opens nor the release can block or fail
  with ENXIO.
* pidfds are taken in the first pause. Each subject is still alive and unreaped at that point: it blocks on
  `hold`, or, in Leaves, on `go`.
* After a SIGKILL the subject runs no more user code, so an absent marker is a deterministic verdict.
* `waitid(P_PIDFD, NOHANG) == ECHILD` pins the reap.
* The drop guard runs on every panic path: `clock` is dropped before `subject`, so the kill comes before the
  scratch directory is removed.
* The Stepped arithmetic is right: 31 ms over the first five pauses, then 98 x 20 ms, then 9 ms, so the limit
  passes on pause 104.

The protocol is not sound against mutants:

**B1 (blocking). `Gate::open` blocks forever when the subject ends before writing its pid.**
* Where: exec/system/tests.rs:577-584. The pid FIFO is opened O_RDWR at 473-477, so its read never returns 0.
* Confirmed with the mutant "trusted argv keeps its first word" (system.rs:106 `argv.remove(0)` removed). The
  shell gets `/bin/sh /bin/sh -c ..` and exits without writing a pid.
* Observed for `a_process_that_outlives_its_limit_is_killed`: the test thread sat in `anon_pipe_read`, the sh
  child was `Z` (defunct), and nothing changed until I killed it. All four Stepped tests, and the outer
  empty-`PATH` test through `.output()`, hang the same way. So cargo test never finishes, even though
  `a_process_inside_its_limit_passes` would fail.
* The same hang follows any early death of the subject: no `/bin/sh`, the OOM killer, a mknod or quoting
  problem the shell rejects.
* Rev 1 C5 named this exact case ("a script that dies before writing the FIFO is a hang ... so it fails
  instead"). The module doc at tests.rs:22-23 ("no step can block for ever") is false.
* Fix: make the pid read non-blocking (O_NONBLOCK on the O_RDWR descriptor). While the pid line is incomplete,
  `Stepped::pause` neither advances `now` nor opens the gate; it yields, counted against a Frozen-sized ceiling.
  A subject that exits then ends the run as Done, and the existing panic at tests.rs:392-394 ("the run ended
  before its first pause: {result:?}") reports it. A subject that never writes hits the ceiling.

**B2 (blocking). The ceilings count only pauses, so a loop that stops pausing spins forever under both clocks.**
* Where: tests.rs:250-258 and 282-296.
* Confirmed with the mutant `if pause < POLL_CEILING { clock.pause(..) }` at system.rs:331 (busy polling once
  the pause reaches its cap). The test thread was in state R with about 20 s of CPU time, and the subject was
  blocked on `hold`. Never ends.
* Production behaviour under that mutant still kills at the limit (WallClock keeps moving), so only the tests
  can catch it, and they hang instead of failing.
* Fix: count looks in `elapsed()` as well (a `Cell<usize>`, since it takes `&self`), with the same ceilings.

**B3 (blocking). The kill signal is not pinned, and a stopped survivor makes `wait_for_end` block forever.**
* Where: tests.rs:549-553 and 670-682.
* With `kill_process_group(pid, Signal::TERM)` at system.rs:360, all 16 tests pass. The default action of
  TERM, and the SIGHUP the kernel sends an orphaned group, end the shell subjects. So nothing pins the SIGKILL
  the code documents (system.rs:350).
* With `Signal::STOP`, the tests passed in one run. That depends on a race: `kill_orphaned_pgrp` sends
  SIGHUP+SIGCONT only if the member is already stopped when the leader exits. When it is not, the survivor
  stays in `T` and `wait_for_end` never returns. I reproduced this with HUP ignored in the subshell: three
  `/bin/sh` subjects in `T` with ppid 1, and the test blocked in `poll`.
* The `Fate` asserts (772-773) cannot help. They read `/proc` only after the pidfd reported the end, so
  `Running` and `KillPending` cannot occur and both asserts are always true. The SIGKILL-specific decision of
  Rev 1 "Kill detection" is gone.
* Fix, tried in the mut clone:
  * Start each script with `trap '' HUP INT TERM;`, before the fork, so the ignore is inherited. A trap inside
    the subshell is too late: the kill can land before it runs, and the TERM mutant then still passed.
  * After the release in `settle`, send `pidfd_send_signal(fd, CONT)` to each process.
* Results with the fix: clean code passed 5 of 5 runs (16 passed each). The TERM, STOP and HUP mutants each
  fail at once with `a process ran on past the kill and wrote the marker`. None hung.

**B4 (blocking, plan conformance). The drain test's deadlock detector is now the 30-minute build limit.**
* Where: tests/exec.rs:1454-1457, which now uses `Limits::trusted()` (`BUILD_TIMEOUT` = 30 min).
* Rev 1 C4: "`both_streams_are_drained_at_once` moves under the frozen clock with the pause ceiling as its
  deadlock detector; no 30 s limit remains." The inventory's earlier R3 note had kept the 30 s limit; neither
  was done.
* Consequence: a sequential-drain mutant now fails only after 30 real minutes, which is the macOS job's
  `timeout-minutes: 30`. Before, it failed after 30 s.
* Fix: add a Frozen drain test to exec/system/tests.rs, for example a `/bin/sh -c` writing 400,000 bytes to
  stderr and then to stdout, with the Frozen ceiling as the detector. Alternatively, record the deviation and
  its cost in the plan.

**Frozen ceiling (residual named by the implementer): adequate; the failure mode is a spurious failure, never a
hang.** I counted `Frozen` pauses with a Drop print added in the mut clone:
* 240 runs, 8 parallel loops on 32 CPUs: median 1,600, max 22,388.
* 576 runs, 48 concurrent test binaries (other agents' builds were also running): median 368, p99 67,896, max
  149,251. All 96 binaries passed.

The margin is at least 33x the worst count seen. Each pause is a `sched_yield`, so on a loaded host it gives
the CPU away and the count grows slowly. The weak case is a test thread on an otherwise idle CPU while the
child is slow to start (cold disk, VM steal): 5M bare yields last on the order of a second or two. A mistake
there gives the panic "the run paused N times under a clock that never moves". That is a spurious red, not a
hang. I accept it as stated; consider adding the counts to the Verification note.

## (d) Mutants (mut clone, `cargo test -p raf-workplan --locked --lib -- exec::system::tests`)
| Mutant | Result |
| --- | --- |
| `kill_group` dropped from `terminate` (`let group: io::Result<()> = Ok(())`) | 3 fail, all on `marker: true`: what_the_process_started, a_descendant_that_holds, and no_kill_program (inner and outer). 0.12 s |
| `pause.min(remaining)` -> `pause` | 4 fail: `elapsed: 2.011s` against `2s` |
| `length > bound` -> `>=` | 2 fail: the capture pure test, and `output_at_the_bound` (the 1000-byte run is refused) |
| `done = status.is_some()` | 1 fails: `a_descendant_that_holds...`, left `Ok(Output { code: Some(0), .. })` |
| own: `kill_process(pid)` (leader only) instead of `kill_process_group` | 3 fail on `marker: true` |
| own: `next_pause` returns a constant 1 ms | 5 fail: the schedule pure test, plus 4 Stepped tests on `paused 201 times, past its ceiling of 200, with the clock at 200ms` |
| own: `process_group(0)` removed | 4 fail on `the leader leads a group` |
| own: `argv.remove(0)` removed | **hangs** (B1) |
| own: no pause once at `POLL_CEILING` | **hangs** (B2) |
| own: group signal TERM / STOP | **pass** (STOP is racy and can hang) (B3) |

No subject process was left after the runs that failed normally.

## (e) Flakiness
* 30 sequential runs: 30 of 30 ok (16 passed each).
* 6 parallel loops of 10: 60 of 60 ok.
* The 96 instrumented runs under 48x load also passed.

## (f) Non-Linux
* The `cfg(all(unix, not(target_os = "linux")))` `outlive` (tests.rs:407-426) uses `while :; do :; done`
  subjects.
* `Stepped` drops its `gate`/`seen` fields there. Every item still used on macOS is used: Scratch (empty
  `PATH`), Shape, LIMIT, killed_at_the_limit, Frozen, the `&mut C` Clock impl, and the rustix waitid/Errno
  imports, which the old `cfg(all(test, unix))` module already used and macOS CI already compiled.
* Windows builds only the pure tests; their imports (`io`, `Cursor`, `Stream`, `Look`, `POLL_CEILING`) are all
  used.
* I see no dead code or unused-import risk. Not compiled here: no other target is installed.

macOS job risks (`--lib exec::system::`):
* Three of the moved tests, including `the_group_is_killed_when_no_kill_program_is_on_the_path`, now run on
  macOS for the first time. The old `tests/exec.rs` versions were never in that job.
* If a group kill fails, the busy-loop subjects burn CPU until the job ends.
* Leaves: the kill can land before the leader exits. The group always holds the live busy subshell, so the
  EPERM path is not reached.
* `exec yes` needs `/usr/bin/yes`, which macOS has.

Platform minimal effort; no blocker.

## (g) tests/exec.rs and src/exec/tests.rs
`grep -nE 'Instant|SystemTime|sleep|recv_timeout|Duration'` finds:
* :16, the import.
* :883-907, scripted FakeRunner elapsed.
* :930-931, constants.
* :952-959, Limits construction.

All are data; none reads a clock.

The deleted tests are not weakened by their replacements. Each replacement asserts exact errors (full
`ExecError` equality and the message suffix) where the old ones used `matches!` or ranges. The kill checks are
stronger: reap, marker, pgrp. Lost: the old runs went through `Runner::run`, a Programs lookup (`git
--version`) and `c_locale`; that path is still covered by the remaining stub tests. Lost, unavoidably: any pin
on `WallClock` itself. A `WallClock::elapsed` returning `ZERO` (production never kills) passes every test. The
inventory accepts this ("trusted to std"), but the Verification text does not say it (advisory A2).

src/exec/tests.rs:331-347: the stub is made executable by a `cp && chmod` child. This is sound. It is the
classic ETXTBSY race: another thread's fork inherits the writer, CLOEXEC notwithstanding, until that child
execs. This process now never holds a write descriptor on the executable. `record.txt` is written in-process
but never executed. No other lib test writes an executable (`grep 0o755|set_permissions`).

## (h) Bindings and Verification text: true (three spot checks)
* "under a clock that moves only by the pauses ... a kill at the limit gives exactly the limit": tests.rs:430-439
  plus Stepped.
* "under a clock that never moves, a kill for too much output and a run that ends each give that clock's
  reading": tests.rs:811, 911, 924, 957.
* "the process the runner started is reaped" and "never run the rest of their script": tests.rs:764-771. True
  for the SIGKILL the code sends, though the tests cannot tell it from TERM (B3).

The eight bindings were moved from `test_crate::exec::stubs::` to `exec::system::tests::`, and the further-checks
table was regenerated.

## (i) Repository rules
* No issue or pull request numbers, and no process wording, in the added lines of either commit.
* SPDX headers are on clock.rs and system/tests.rs.
* No attribution trailers.

## Advisories
* A1: the `Seen` drop (tests.rs:613) group-kills `leader.pid` even after a clean end. After a pid wraparound
  that could reach an unrelated group. Limit it to `std::thread::panicking()`.
* A2: state in the Verification text that `WallClock` itself is not under test.
