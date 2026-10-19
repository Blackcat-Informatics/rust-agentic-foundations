# Design review: the registry scan example reads no clock

Reviewed plan: `.stage/raf-source-the-registry-scan-example/plan.md`

Code read:
- `crates/raf-source/examples/registry_scan.rs` (whole file; `run_child` 492-537, `wait` 555-574, `Tally::count` 588-621)
- `tests/repo_gates.rs` 258-390 (`ClockRule`, `clock_rule`, `clock_hits`), 619-676 (the tracked-files gate), 1202-1220 (the examples fixture)
- `crates/raf-source/tests/support/child.rs` (the precedent that classifies a child's exit: `died_by_signal` 122-134, `classify` 146-193)
- `crates/raf-workplan/src/exec/system/tests.rs` 126-152 (the other `ExitStatusExt::signal` reader)
- `crates/raf-source/Cargo.toml`, `clippy.toml`, root `Cargo.toml` 36-57 (lints), `Makefile` 45, 56-63, 102-107, 118-130
- `AGENTS.md` 151-172, `CONTRIBUTING.md` 89-100, `docs/capabilities/source-index.md` 2392

## Is `ulimit -t` the right replacement?

Yes, as a mechanism. RLIMIT_CPU bounds what the child has consumed, not what a clock in
the example has read; the kernel enforces it; it rides the seam the example already has
(`sh -c 'ulimit -v ...; exec "$0" "$@"'`, `registry_scan.rs:493-505`) so there is one
way a child is bounded and no new dependency, `unsafe` (`pre_exec`) or rustix feature. The
declared cost (a child that blocks without CPU is not stopped) is real and acceptable for a
child that only reads a local tree and parses. The plan's account of what `ulimit -t`
does is wrong in one place, though, and the design, the classification and acceptance
check (4) are all built on it. Findings 1 and 2 below correct that; finding 3 is a gate
the plan will fail as written.

## BLOCKING

### 1. `ulimit -t N` sets soft = hard, so Linux sends SIGKILL and never SIGXCPU; the plan then files every SIGKILL under `cpu-limit`

Plan: "A child killed by the limit (signal SIGXCPU, or SIGKILL at the hard limit, both set
by `ulimit -t`) is reported `ERR cpu-limit`" and "signal 24 and signal 9 give `cpu-limit`"
and acceptance (4) "`sh -c 'ulimit -t 1; exec sh -c "while :; do :; done"'` and see it end
by SIGXCPU".

`ulimit -t N` without `-S`/`-H` sets both limits to N in dash, bash and busybox. The
kernel's RLIMIT_CPU check (`check_process_timers` in `kernel/time/posix-cpu-timers.c`;
setrlimit(2) RLIMIT_CPU) tests the hard limit first and sends SIGKILL "with no further
action"; SIGXCPU is only sent while soft < hard. So:

- acceptance (4) as written observes `Killed` (signal 9, sh exit 137), not SIGXCPU; the
  evidence it records in validation.md cannot be produced by that command;
- to make the plan's run work the design makes signal 9 mean `cpu-limit`. Signal 9 is
  also what the OOM killer sends (the scan sizes `--jobs` against `MemAvailable` once,
  `registry_scan.rs:174-194`, and other processes on the host move) and what an operator's
  `kill -9` sends. Those children would be printed `ERR cpu-limit :: stopped after 900 CPU
  seconds` and counted under that kind in the `#` tally (`Tally::count`, 614-618): a wrong
  kind, silently, with a message that asserts a fact the scan does not know.

The precedent child classifier keeps "ended by a signal" as an abort and names the status
(`tests/support/child.rs:179-182`); it never guesses which signal meant what.

REWRITE TO: set the soft limit below the hard one so the first signal is unambiguous, and
classify only SIGXCPU as the limit. Wrapper (soft first, so each `setrlimit` keeps
`cur <= max`): `ulimit -v <KiB> && ulimit -S -t <n> && ulimit -H -t <n+1> && exec "$0" "$@"`.
The hard limit one second above the soft one is the backstop for a child whose SIGXCPU
disposition is ignored (SIG_IGN survives `exec`), nothing more. `classify`: `signal() ==
Some(libc::SIGXCPU)` gives `ERR cpu-limit :: stopped after <n> CPU seconds`; every other
signal, SIGKILL included, falls through to the existing `ERR abort :: <status>: <last
stderr line>`, whose `{status}` already prints `signal: 9 (SIGKILL)`. Acceptance (4)
becomes `sh -c 'ulimit -S -t 1 && ulimit -H -t 2 && exec sh -c "while :; do :; done"'; echo $?`
and records the shell's `CPU time limit exceeded` / exit 152 (128 + 24); the unit table
in (2) changes to: signal 24 gives `cpu-limit`, signal 9 and signal 6 give `abort`. The
`libc` crate is already a unix dependency of `raf-source` (`Cargo.toml:21-22`), so the
signal number need not be a literal.

### 2. `;` between the `ulimit`s and the `exec` turns a failed limit into an unbounded child, and the design has just removed the only other stop

Plan: "the child is started through `sh -c 'ulimit -v <KiB>; ulimit -t <seconds>; exec "$0" "$@"'`"
and "`wait` becomes a blocking `Child::wait`".

Today a failed `ulimit -v` (`registry_scan.rs:496`) is already swallowed by `;`, but the
poll in `wait` (555-574) still stops the child. After this plan the blocking `Child::wait`
is the only wait, so when `ulimit -t` fails (an `sh` without it, a `-S` value above an
inherited hard limit, a bad number) the child runs with no CPU bound and the scan hangs on
a spinning crate, the very case the issue's timeout existed for, and nothing is printed to
say why. That is a fallback that hides a missing capability.

REWRITE TO: `&&`-chain the wrapper as in finding 1's rewrite. A failed `ulimit` then ends
the child with the shell's message on stderr and a nonzero status, which the existing
classifier prints as `ERR abort :: exit status: 2: sh: 1: ulimit: ...` on every crate: loud
and attributable. Add this to the unit table: a nonzero exit without a result line and an
`ulimit:` stderr gives `abort` with that line (no new kind).

### 3. `ExitStatusExt::signal` in the example without a `cfg` breaks `make cross-check`, which `make check` runs

Plan: "`ERR cpu-limit ...` when the child was ended by SIGXCPU or by SIGKILL (the hard
limit; `ExitStatusExt::signal`)" and Task 3's gate list, which omits `cross-check`.

`make check` runs `cross-check` (`Makefile:128`), which is `cargo check -p raf-source
--all-targets --target x86_64-pc-windows-gnu` with `-Dwarnings` (`Makefile:45,106`);
`--all-targets` compiles the example. `std::os::unix::process::ExitStatusExt` does not
exist on that target, so the plan as written fails the repository's own gate. The
precedent already solved this once: `died_by_signal` behind `#[cfg(unix)]` with a
`#[cfg(not(unix))]` twin returning `false` (`tests/support/child.rs:122-134`); the workplan
tests keep the import inside a `#[cfg(unix)]` test (`exec/system/tests.rs:126-138`).

REWRITE TO: one small `#[cfg(unix)] fn cpu_limit_signal(status: ExitStatus) -> bool` in
the example that reads `status.signal() == Some(libc::SIGXCPU)`, with a `#[cfg(not(unix))]`
twin that returns `false` (on a non-unix host the wrapper cannot start anyway and `ERR
spawn` reports it), and `classify` takes `ExitStatus` and calls it. The `#[cfg(test)]`
module that uses `ExitStatusExt::from_raw` is itself `#[cfg(all(test, unix))]`. Add
`make cross-check` to Task 3's list (or say `make check`).

## ADVISORY

### 4. `classify` returns a `String` that `Tally::count` parses back

Plan: "`classify(status: ExitStatus, stdout: &str, stderr: &str, cpu_seconds: u64) -> String`".

This keeps the existing shape (`run_child` returns a line, `Tally::count` re-parses `ERR
<kind>` at `registry_scan.rs:614-618`), so it is not a regression and the unit tests the
plan wants can assert on the prefix. Noted only so the implementer does not widen it: the
kind table (`cpu-limit`, `memory`, `abort`, the `LoadErrorKind` names) now lives in three
places (the child's `load_one`, the parent's `classify`, the module doc). No rewrite
required for this issue.

### 5. Gate change is the right shape

Removing the `else if dir == "examples" { None }` arm (`tests/repo_gates.rs:354-355`), the
"has no tracked example" assertion (641-647), the fixture entry (1206) and the comment
(310-312) leaves one rule for all of `crates/raf-source/{src,tests,examples}` with no new
flag or carve-out. `SOURCE_CLOCK_WORDS` does not include `timeout`, so the prose
"`timeout` outcome" that the plan renames anyway, and `ulimit -t` in a doc comment, do
not trip it; `elapsed` and `Duration` leave with `wait`. No finding.

VERDICT: NEEDS-REVISION
