# Enhancement audit

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/27-raf-source-the-registry-scan-example/.stage/raf-source-the-registry-scan-example/plan.md`

Standing constraints: no `.goals` file; CLAUDE.md / AGENTS.md rules apply (no timing runs or
sleep-based waits, fail hard rather than fall back, no deferrals, no issue numbers in committed
files). No item below conflicts with them; none is REJECTED.

## Items (TRANSFORMATION first)

### 1. TRANSFORMATION: make work a budgeted dimension of the load, and let the scan calibrate it

**The concept the feature is a shadow of.** The issue asks to stop a child "with a bound on
counted work". The plan reframes that as a CPU-time rlimit because "the index ... has no step
budget". That premise is incomplete: `crates/raf-source/src/work.rs` already defines
`Work { Attributes, Positions, ColumnSteps, PatternSteps, ExcludeTests, SegmentReads, Lookups,
ViewReads, WalkSteps, SupplySteps }` with `work::add(kind, amount)` called at every unit of work
in the settle, the export walk and resolution, and `measure`/`measure_kind` that panic when a
tally passes a ceiling. It is compiled to nothing outside `cfg(test)`. The index therefore has
a complete cost semantics for *space* (`Budget::{TotalBytes, ItemBytes, FactBytes, GlobBytes}`,
each charged before the allocation it bounds, with measured constants) and a test-only cost
semantics for *time*. The definitive structure is a resource-bounded computation in both
dimensions: a work (fuel) budget metered alongside the byte budgets, refused deterministically
at a limit, reported in `Usage`, and calibrated by the registry scan exactly as `tokens` and
`recursion` already are ("over the registry scan below the most tokens of one file was
1505567", `docs/capabilities/source-index.md` L665-667). The capability doc states the gap in so
many words: "No budget bounds the time of these loads" (L1700, the superlinear settle shapes).

**The enhancement.** Promote the `Work` tallies to a metered budget:

- `Budget::Work` (the enum is `#[non_exhaustive]`, so additive), with a limit in the `BUDGETS`
  table and a `LoadOptions::max_work` default;
- the counter lives on `Meter` (per load, `Cell`-based like the byte counters; the settle and
  resolution already hold a `&Meter`, see `glob_scope.rs` L986-989 where supply steps are
  already charged to `meter.items` as bytes) and is charged unconditionally in every build; the
  `cfg(test)` ceiling machinery in `work.rs` stays as the per-kind instrument;
- `Usage::work` (the struct is `#[non_exhaustive]`, additive) and the refusal
  `LoadErrorKind::Budget { budget: Budget::Work, .. }` naming the file and line of the
  declaration being settled when the limit is passed;
- the scan's `OK` line gains `work=N`, `USAGE` gains `"work"`, and the `# largest work=` line
  names the costliest crate, which is the calibration figure for the default limit.

With this, the child's refusal is machine-independent (the same crate gives the same outcome on
every host and in debug and release), the scan meets R1 as literally written, the clock leaves
the example with nothing to replace it except the kernel backstop of item 2, and the three
issues that are visibly next in this area (the tracked superlinear glob settling, which now
has a budget to refuse at and a figure to report; DoS hardening of the index as an agent-facing
reader of untrusted crates; budget calibration from the scan) all land on one structure.

**Cost:** L. Touches `budget.rs`, `meter.rs`, `options.rs`, `work.rs`, the charge sites in
`settle.rs`/`glob_scope.rs`/`resolve.rs`, `load_budgets.rs` tests, the capability doc's budget
table and the scan. The per-step cost is one saturating add on a `Cell`.

**Payoff:** decisive. Deterministic bounds in both resource dimensions; the only design that
satisfies "bound on counted work" rather than substituting a timer the kernel reads.

**Where it changes the plan:** the Decision paragraph "a counted-work bound in the index (the
index reports `Usage` and refuses at its memory/token budgets but has no step budget: settling
cost without a budget is tracked separately as superlinear glob settling)" must be rewritten: the
step counting exists; what is missing is making it budgeted and unconditional. Also NEEDS-JUDGMENT
disposition "1, 4 ... neither" and Task 1's `OK` line / `USAGE` array. If the planner declines
to fold it into this change, the no-deferrals rule requires the plan to name the tracked change
that takes it (the superlinear settling one is the natural home) and to say explicitly that the
CPU limit of Task 1 is the interim bound until that change lands, with `--cpu-limit` to be
removed or demoted to a backstop there.

### 2. ROBUSTNESS (load-bearing for Task 1's evidence): soft and hard CPU limits, and SIGKILL is not a CPU-limit signal

`ulimit -t N` in `sh` (dash, bash, busybox) sets soft and hard to the same value. Linux checks
the hard limit first: at `psecs >= hard` it sends SIGKILL and returns; SIGXCPU is sent only when
`soft <= psecs < hard`. So under the plan's wrapper a child that spins is ended by **SIGKILL
(signal 9)**, almost never by SIGXCPU, and the plan's manual check (4) ("see it end by SIGXCPU")
will observe exit 137 instead. The plan's classifier papers over this by mapping signal 9 to
`cpu-limit`, but SIGKILL is also what the OOM killer and an operator send, so a memory-starved
host would be reported as `cpu-limit`.

Set soft below hard: `ulimit -Ht $((N + 1)) && ulimit -St N`. Then SIGXCPU (24) is the one
signal that means the CPU limit, and SIGKILL means something else: report it `ERR killed`.
Change Task 1 acceptance (2) to "signal 24 gives `cpu-limit`, signal 9 gives `killed`" and the
mutant (5) accordingly; change check (4)'s expected signal.

**Cost:** S. **Payoff:** the classification is correct, and the validation evidence is true.

### 3. ROBUSTNESS: a limit the kernel refuses must fail the child, not run it unlimited

The current wrapper joins with `;` (`ulimit -v ...; exec`), so if `ulimit` fails (the inherited
hard limit is already lower, as in a container with RLIMIT_CPU set; an unsupported option) the
child runs with no limit and the scan reports `OK`. The repository's own precedent uses `&&`
(`tests/spawn_limit.rs` L56: `ulimit -v "$1" && shift && exec "$@"`), and the design rule is
fail hard. Build the wrapper as a pure function `wrapper(max_memory_kib: u64, cpu_seconds: u64)
-> String` joined with `&&`, unit-test its text (no spawn needed), and have `classify` report a
non-zero exit with no result line and an `ulimit:` line on stderr as `ERR limit :: <line>`.

**Cost:** S. **Payoff:** a silent loss of the bound becomes a reported failure; the wrapper
text, which carries the whole guarantee, gets a test.
**Where:** Task 1, `run_child` and the acceptance list.

### 4. LEVERAGE: classify into an enum, mirroring the existing child judge

`crates/raf-source/tests/support/child.rs` already has the shape the plan reinvents:
`classify(code, signaled, stdout, stderr) -> Outcome` with `Outcome::{Refused, Index, Aborted,
Failed}` and a separate renderer. The plan's `classify(...) -> String` tests string prefixes.
Return an `Outcome` enum (`Result(line) | CpuLimit | Memory | Killed | Limit | Abort { status,
last }`) with `Display`, and let `Tally::count` match on the enum rather than re-parsing
`ERR <kind>` from text. Tests then compare variants, and the `#` block cannot drift from the
line format.

**Cost:** S. **Payoff:** a typed outcome, tests that cannot pass by coincidence of text, and one
classification idiom across the crate's child runners. **Where:** Task 1, "The pure function
`classify(...) -> String`".

### 5. UTILITY: let the scan drive `LoadOptions`, so it is the calibration instrument the doc says it is

The capability doc derives defaults from this scan ("over the newest version of each of 1177
crates of a local registry ... so it is 256 MiB", L694-696), but the example calls
`Workspace::load` with defaults only. Add `--max-file-tokens`, `--max-total-bytes`,
`--max-item-bytes`, `--max-fact-bytes`, `--max-glob-bytes` (and `--max-work` with item 1),
passed through `Workspace::load_with`, so a budget candidate can be run over the registry and
its refusals counted in the `#` block. `parse_bytes` already exists for the byte flags.

**Cost:** S-M. **Payoff:** budget calibration becomes a reproducible command instead of an ad
hoc edit of the defaults; each `Budget` variant's limit gets a documented way to re-derive it.
**Where:** Task 1 flags and module doc; Task 3's check of `source-index.md`.

### 6. ROBUSTNESS: document and name what RLIMIT_CPU measures

RLIMIT_CPU sums the CPU of every thread of the child. The load runs a parse thread beside the
main thread (`parse.rs` L58-59), so `--cpu-limit 900` equals the old 15-minute wall default
only for a child that is effectively single-threaded, and a future parallel settle would halve
the effective bound. Say so in the module doc and the flag's help: "CPU seconds of the child,
all its threads together, enforced by the kernel". Also state the one visible gap the plan
already names (a child blocked without CPU is not stopped) in the doc, not only in the plan.

**Cost:** S. **Payoff:** the flag's contract is stated where its users read it.
**Where:** Task 1 "Update the module doc".

### 7. ROBUSTNESS: close the gate's coverage hole, not only the carve-out

`tests/repo_gates.rs` L649-657 asserts "no rule judges ..." only for files under `src`. Once
the examples carve-out goes, extend that assertion to every `CRATE_DIRS` entry
(`src`, `tests`, `examples`) and the root `tests/`, so a carve-out for any `tests/` or
`examples/` file cannot return without failing the gate, and drop the `dir == "examples"` arm
of `clock_rule` entirely rather than leaving a conditional. Add a fixture that a planted reading
under `crates/<other>/examples/` is also reported (today only `raf-source`'s examples were
special, but the assertion should cover all).

**Cost:** S. **Payoff:** the gate proves its own coverage instead of relying on the planner's
memory of it. **Where:** Task 2.

### 8. ROBUSTNESS: the example's unit tests and `ExitStatusExt`

`ExitStatusExt` is Unix-only; importing it unconditionally makes `cargo clippy --all-targets`
fail on Windows (minimal-effort platform, but a `#[cfg(unix)]` on the import and a
`#[cfg(target_os = "linux")]` on the test module, as `spawn_limit.rs` does, costs nothing).
In the tests, `ExitStatus::from_raw` takes a raw wait status: a signal is the low 7 bits
(24 for SIGXCPU, 9 for SIGKILL, 6 for SIGABRT), an exit code is `code << 8`; the plan's
"zero exit" and "nonzero exit" cases must be built that way. With `test = true` on the
`[[example]]`, `make test` (`cargo test --workspace`) and `make test-release`
(`cargo test -p raf-source --release`) both run them; confirm `make clippy` still compiles the
example under `-D warnings` after the clippy allow is removed.

**Cost:** S. **Payoff:** the tests encode the right statuses and the example stays buildable on
every platform the workspace compiles on. **Where:** Task 1 acceptance (2) and the `[[example]]`
table.

## Not raised

Reusing `raf-workplan`'s `SystemRunner` (pidfd and a watchdog clock) for the child: the plan
already declines it correctly (it is a clock, and it would make `raf-source` depend on
`raf-workplan`). Reading the child's `rusage` to report CPU spent per crate: declined by me, it
is a timing measurement in the sense the repository forbids and adds nothing a work budget
(item 1) does not give deterministically.

## Verdict

Items 1 and 2 are not dispositioned by the plan: item 1 because the plan's stated reason for
declining counted work ("has no step budget") overlooks `work.rs`, and item 2 because the plan's
acceptance evidence expects a signal the kernel will not send under the wrapper as written.

VERDICT: NEEDS-REVISION
