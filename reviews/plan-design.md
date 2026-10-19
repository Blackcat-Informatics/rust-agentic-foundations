# Plan design review (SOLID / ETHOS)

Reviewed: `.stage/replace-wall-clock-assertions-in-tests/plan.md` and its design source
`clock-inventory.md` (issue 21).

Code read: `crates/raf-workplan/src/exec/system.rs` (whole file: `run`, `abandon`, `Capture`,
`wait`, `terminate`, `kill_group`, `group_signal_outcome`, the five group-signal tests),
`crates/raf-workplan/src/exec.rs`, `crates/raf-workplan/src/exec/fake.rs`,
`crates/raf-workplan/src/bound/run.rs:150-260` (list pass then run pass through one runner),
`crates/raf-workplan/src/bound/evidence.rs:187` (`wall` = sum of run elapsed),
`crates/raf-workplan/src/bound/tests.rs` (how `Executor` is built with a `FakeRunner`),
`crates/raf-workplan/src/evidence/tests.rs:183-201, 395-445`, `crates/raf-workplan/tests/exec.rs:1398-1710`,
`crates/raf-workplan/tests/bound.rs:290-400, 541-575`, `crates/raf-source/src/work.rs` (precedent),
`crates/raf-okf/src/profile/mod.rs:598-642`, `crates/raf-okf/src/profile/body_rules.rs:40-63`,
`crates/raf-okf/src/profile/records.rs:589-620`, `crates/raf-workplan/src/graph.rs:1340-1384`,
`crates/raf-workplan/src/rationale.rs:165-184`, `crates/raf-workplan/src/assessment.rs:656-667`,
`crates/raf-workplan/src/obligation/sources.rs:199-220`, `tests/repo_gates.rs:251-302`,
`crates/raf-workplan/Cargo.toml:22` (rustix features), workspace lints in `Cargo.toml:36-46`.

## What is sound (no finding)

* **The `Clock` seam.** `pub(super) trait Clock { elapsed; pause }` in `exec/clock.rs`, `WallClock` as
  the one production implementation, `run_with<C: Clock>` private to `exec::system` and `Runner::run`
  delegating with `WallClock::start`. Two named second implementations (Frozen, Stepped) exist in
  `cfg(test)`, so this is not speculative generality. `pub(super)` from `exec::clock` is visible to the
  sibling `exec::system`; `unreachable_pub = deny` concerns only bare `pub`; three monomorphisations
  (Wall, Frozen, Stepped), the test two under `cfg(test)`; no dead-code root. Public API and `FakeRunner`
  (`fake.rs:132` already applies `limits.timeout()` to scripted `elapsed`) unchanged. Pure `verdict` and
  `next_pause` lift exactly the two decisions in `wait` (`system.rs:233-273` and `:276`) that today can
  only be reached by starting a real process.
* **`kill -STOP $$` as the never-ending subject.** `Child::try_wait` is `waitpid(WNOHANG)` without
  `WUNTRACED`, so a stopped leader stays `None` (`system.rs:240`); the Capture threads block on pipes the
  stopped shell still holds; nothing ends it but SIGCONT or SIGKILL, and `kill_group`
  (`system.rs:304`) sends SIGKILL, which wakes and kills a stopped task. `kill` is a builtin in dash and
  bash, so test 4 with an empty `PATH` still works.
* **ShdPnd claim.** For a group-directed SIGKILL Linux adds the signal to the shared pending set and,
  being fatal, also to each thread's private set, and it stays there until the task is scheduled,
  dequeues it and exits; after that the entry is `Z` until the new parent reaps it, then gone. The
  plan's three-way acceptance (pending bit, `Z`/`X`, entry gone) covers every point of that timeline, so
  the init-reaps-first race is a pass, not a flake; the mutant (no `kill_group`) leaves the grandchild
  present in state `T` with no pending bit, which fails. Check the 0x100 bit in either `ShdPnd` or
  `SigPnd`, not `ShdPnd` alone. See ADVISORY 3 for the one gap.
* **Thread-local tallies.** The only `thread::spawn` in `raf-okf` and `raf-workplan` src is
  `Capture::start` (`system.rs:187`), which no counted path touches; `raf-source` parses on a
  `thread::scope` (`parse.rs:58`) but that runs inside `Workspace::load`, before any counted
  raf-workplan step, and on the calling thread's view once it returns. The counted steps in R1/R2 run on
  the test's thread. The raf-okf hook points are the right chokepoints: `listed` at `mod.rs:599` and
  the per-entry loop at `:611`; the `for block in &blocks[first..]` at `body_rules.rs:51`; `by_name` at
  `records.rs:592` and the per-limit loop at `:596`.
* **R4.** `FakeRunner` turns `with_elapsed(BUILD_TIMEOUT + 1 ms)` into `ExecError::Timeout` under
  `Limits::trusted()` and the neighbour at exactly `BUILD_TIMEOUT` passes only when the production limit
  is 30 min, so the "other limits" mutant is caught by the neighbour and by `Call::limits`.
* **Gate word list.** `has_word` (`repo_gates.rs:276`) is whole-word, so the `sleeping`/`Instantiate`
  clean fixtures are consistent with the implementation.

## Findings

### BLOCKING 1. R1 (and the R2 claims test) assert a shape that the stated mutants also satisfy

Plan text (inventory R1): "capability `BlocksVisited` linear in sections for fixed blocks and in blocks
for fixed sections"; "`ChangeIdsSet` linear"; "`MeasuresIndexed` linear"; with the growth idiom
"measure at 1, 2, 3 units; assert `one > 0` and `three + one <= 2*two`".

Consequence, per mutant:

* `body_rules.rs:50-55`, mutant `let first = 0` or remove `break`: section *i* then visits about the
  blocks before and inside it; with *B* blocks fixed and *S* sections varied the total is about
  `B*S/2`, linear in *S*; with *S* fixed and *B* varied it is about `B*S/2`, linear in *B*. Both
  one-dimension-at-a-time checks pass the mutant. Only scaling both together (`B = k*S`) makes it
  quadratic (`9 + 1 > 8`).
* `mod.rs:599`, mutant builds `listed` inside the `for text in texts` loop at `:611`:
  `ChangeIdsSet = affects * changes`, linear in each dimension alone; passes "linear".
* `records.rs:592`, mutant builds `by_name` inside `for limit in limits` at `:596`:
  `MeasuresIndexed = limits * values`, same shape, same miss.
* `graph.rs:1344-1357`, R2 claims mutant "rebuild `loaded`/`present`/`covered` per claim": the
  set-building count becomes `claims * records`; "counting ... the loaded/present sets built" at one
  varying dimension passes it.

So four of the plan's "fails fast on its mutant" acceptance rows are not met by the design as written.
The plan already recognises this exact shape for coverage ("pin `ExclusionTests == entries x
exclusions`"), but applies the weaker wording to R1.

REWRITE TO: for every count whose mutant creates a product, either (a) pin it exactly where the correct
value is a known function of the input (`ChangeIdsSet == changes.len()`; `MeasuresIndexed ==
values.len()`; `LoadedIndexed == evidence.len()` and `CoveredIndexed == sum of covers`;
`BlocksVisited == blocks inside the sections + the number of sections that have a block after
them`), or (b) scale both dimensions together at units 1, 2, 3 (sections *and* blocks per document;
affects *and* changes; limits *and* values; claims *and* records) and apply `three + one <= 2*two`
with a ceiling at about `3 * linear`. State per test which option it uses and name the mutant it
fails. The other R2 counts are fine as designed: `symbol_inputs` (`assessment.rs:660-665`) goes
quadratic in paths alone; `has_item` (`rationale.rs:179`) and `extent` (`:167`) go quadratic as
headings and lines grow together on the 20,000-item page; the graph prose and obligation `ScanBytes`
counts scale both factors with one size; `moves_to_unknown` pins `SlugsListed == records`.

### BLOCKING 2. R5 P1 deadlocks on the list pass and its `wall == 5 s` contradicts `Evidence::wall`

Plan text (inventory R5 P1): "run through `Executor::new(&source, &Stepped(&system_runner,
clock_factory))` using a cfg(test) `Runner` wrapper calling `run_with`"; "assert ... `wall == 5 s`".

Consequence: `Executor::test` runs two invocations through the same runner, the list pass
(`cargo test ... -- --list --exact ...`) at `run.rs:≈140-153` and then the run pass at
`run.rs:173-174`. Under the Stepped clock "on the first `pause` it blocks reading a FIFO the script
writes its pid(s) to" (inventory R3). The list pass never runs `forever`, so nobody opens the FIFO for
writing, and the first `pause` of the list pass blocks forever: the deterministic test is a hang on
correct code. Separately, `Evidence::wall` is the sum of every run's elapsed (`evidence.rs:187`), so
with any non-zero reading for the list pass `wall != 5 s`.

REWRITE TO: a cfg(test) wrapper `ClockedRunner<F: Fn(&Invocation) -> Choice>` in `exec::system`
(`pub(crate)` under `cfg(test)`) that picks per invocation: `WallClock` for an invocation whose args
contain `--list` (or any argv not matching the run pass), the FIFO-gated `Stepped` for the run pass
only. Assert the killed run's own `Run::elapsed() == 5 s` and the word-for-word reason, `Ended::TimedOut`,
and `wall >= 5 s` (as `bound/tests.rs:1484` already does). If `wall == 5 s` is wanted, the list pass
must use `Frozen(Duration::ZERO)`, which spins on `yield_now` for the length of a real `cargo test
--list`; say so if chosen.

### ADVISORY 3. The "entry gone" branch of the grandchild check is vacuous without a pre-kill capture

Plan text (R3 test 3): "`/proc/<g>/status` `ShdPnd` has bit 0x100 (SIGKILL) or state Z/X or entry
gone (compare `starttime` against pid reuse)".

Consequence: the test regains control only after `run_with` returns, i.e. after the kill, so there is
no moment at which it can read `starttime` of the live grandchild; a mis-parsed or stale pid then
passes through the "gone" branch. The one place that sees the pid before the kill is the Stepped
clock's FIFO gate.

REWRITE TO: in the gate, when the pid arrives, read `/proc/<g>/stat` and require state `T` (stopped),
`pgrp == leader pid`, and record `starttime`; after the run, pass only on pending-bit, `Z`/`X`, entry
gone, or a different `starttime` for the same pid. This also proves the subject was alive and inside the
group before the kill, which is what the mutant must change. Note: the FIFO itself needs `mkfifo`;
`rustix` is built with `features = ["process", "std"]` only (`Cargo.toml:22`), so either add `fs`
(`mknodat` with `FileType::Fifo`) or have the script create it by absolute path (`/usr/bin/mkfifo`),
since test 4 runs with an empty `PATH`. An alternative with neither: the script writes its pid to a
regular file and the gate polls that file and `/proc/<pid>/stat` for state `T` with `yield_now`; that
is an event wait, not a timer, and removes the FIFO dependency.

### ADVISORY 4. A script that dies before writing the FIFO is a hang, not a failure

Consequence: `open` of a FIFO for reading blocks until a writer appears; if `/bin/sh` exits early (a
typo in the script, a failed redirection), the test blocks until the CI `timeout-minutes` watchdog,
which the plan elsewhere treats as acceptable for the frozen-clock smoke tests. State that explicitly
for the Stepped gate too, or give the gate a leader pidfd (`rustix::process::pidfd_open`, already in the
`process` feature) and `poll` on both descriptors so an exited leader turns into a panic with the
captured stderr.

### ADVISORY 5. The look count in test 1 is stated inconsistently with the Stepped counter

Plan text: "105 looks (31 ms from 1+2+4+8+16, then 98 x 20, then 9)" while Stepped counts "looks that
advance". The schedule gives 104 pauses (5 + 98 + 1) and 105 loop iterations. Pin one of the two,
derive it from the `next_pause` pure test rather than a literal, and keep the equivalent-mutant note
(`elapsed: limits.timeout()`) as recorded.

VERDICT: NEEDS-REVISION
