Closes #21

## What changes

No test in `raf-okf`, `raf-workplan`, `cargo-workplan` or the root `tests/` asserts on a clock reading any more, and no test code reads a clock. Each former timing assertion is now a count of work done, a scripted elapsed value, or a deterministic event. Runs of real processes through `SystemRunner::run` (the `tests/exec.rs` stubs, `tests/bound.rs`, the complete/verify/scenario tests, the `cargo-workplan` tests, `self_workplan`, and the list pass of the cargo never-ending tests) still go through the production `WallClock` as a watchdog for their 5- and 30-minute limits; nothing asserts what it reads.

* **raf-okf, raf-workplan algorithms**: cfg(test)-only `work.rs` tallies (counted sets, per-lookup ticks) with exact counts or ceilings, and growth checks (`one > 0`, `three + one <= 2 * two`). Coverage pins entries x exclusions exactly.
* **Runner**: the only clock reading is `exec/clock.rs` (`Clock`, `WallClock`). `SystemRunner::run_with` takes a clock; the loop's decisions are the pure `verdict` and `next_pause`. Kill tests run under `Frozen` and `Stepped` test clocks that count every pause and look, with ceilings, and gate on FIFOs and pidfds (Linux). `Clock::spawned` is a default no-op hook called once after a successful spawn.
* **Cargo never-ending tests** moved from `tests/bound.rs` to `exec/system/tests/cargo.rs`, clock-free.
* **Evidence timeout** test uses a scripted runner elapsed (limit + 1 ms, and exactly the limit).
* **Budgets removed** from the integration tests; scratch names use process id plus a counter.
* **Gate**: `time_in_crates_*` in `tests/repo_gates.rs` refuses clock words in the `src`, `tests` and `examples` of every crate under `crates/` (derived from the path, so a new crate is judged) and in the root tests. A word followed by an `_` suffix counts (`sleep_ms`, `sleep_until`, `park_timeout_ms`, `wait_timeout_ms`, `wait_timeout_while`); `sleeping` and `Instantiate` do not. Two exempt files and the `raf-source` examples carve-out (clippy only) are asserted live. `clippy.toml` disallows `Instant`, `SystemTime`, the `std::thread` sleeps and `park_timeout` waits, the `Condvar` timeout waits and `Receiver::recv_timeout`, with two reasoned allows.
* **Process group kill (fix)**: macOS answers EPERM to a group signal that reaches no live member, and a leader that is still exiting is not yet seen by `try_wait`. After an EPERM the runner now sends SIGKILL to the leader itself and collects it; the leader counts as ended only when that signal is delivered or answers ESRCH, and any other refusal is still reported. The decision is pinned on Linux by a pure test and two real-process tests.

## Validation

`make fmt clippy test test-release doc deny reuse msrv fuzz licenses issue-refs` all exit 0 on the head. `cross-check` was not run locally (targets not installed); hosted CI covers it. Every converted test was checked against mutants in scratch clones; each fails and none hangs (the two deleted-index mutants of the cost tests fail only after their own quadratic scan has run). Hosted CI is 12 of 12 green on the head, test-macos included. The tables are in the stage validation notes.

## Disclosures

* Agents ran `rm -rf` variants despite the instruction; nothing outside scratch space was deleted.
* Two mutant runs briefly touched the worktree instead of a scratch clone; both were reverted.
* `Frozen` has a 5,000,000-look ceiling; it depends on host speed in principle (about 350x headroom observed).
* Windows/macOS: the pidfd/FIFO kill tests and the cargo never-ending tests are Linux-only. The output-limit kill tests under `exec::system::` run on macOS too; they failed there on the previous head (`could not be killed .. Operation not permitted`) because of the EPERM race above, and pass with the fix. The fix was reasoned from the failure and the kill semantics and checked on hosted macOS CI; it could not be run on a local Mac, and one green run cannot rule out the race.
* The wall-clock budgets in the integration tests are gone with no count replacing them: they ran after the run returned and never caught a hang.
* Coverage is pinned by product, not by an index: an index was declined here and is tracked in #29, which states that an index must still return every matching exclusion so that unused exclusions can be reported.
* An uncounted loop added elsewhere is not seen by a count.

## Least confident

The Linux-only pidfd/FIFO gating in the runner kill tests: it was reviewed and survived 600 concurrent runs, but it is the most intricate test code in the change. One example-case failure in 40 concurrent runs came from the host's shared `CARGO_TARGET_DIR`.
