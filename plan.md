# Plan (Rev 2): the registry scan example reads no clock; the index bounds its work by count (issue 27)

## Issue summary
`crates/raf-source/examples/registry_scan.rs` stops a crate that runs too long with `Instant`,
`Duration` and a 20 ms `thread::sleep` poll. The issue asks for "a bound on counted work (the
index already reports `Usage` and refuses at its budgets), or remove the example", and for the
gate to cover `examples/`.

## Standing goals / governing decisions
CLAUDE.md: no timing runs, wall-clock benchmarks, sleep-based waits or timing assertions (tell
every agent started: stated in every agent prompt of this change); Linux primary, other systems
minimal effort but `make cross-check` (part of `make check`) must stay green; no issue/PR numbers
or process wording in committed files; no attribution; no deferrals; no lint weakened.
Decision (Rev 1, after the five lens reviews and the user's direction to take the recommended
option): the bound is a **counted work budget** in the index, not a kernel CPU-time limit and not
removal. The kernel CPU limit was rejected because it is still a time bound that depends on the
host; removal was rejected because the example is the reproducible source of the registry
figures in the source-index capability; relying on the existing byte budgets alone was rejected
because glob settling has shapes whose cost grows faster than their bytes.

## Prior art
No prior attempt. Precedent: `Budget`/`LoadOptions`/`Usage`/`Meter` (budget.rs, options.rs,
meter.rs) for bytes and tokens; `work.rs` counts ten kinds of work at ~21 call sites, compiled
to nothing outside `cfg(test)`, and the tests assert exact counts and growth ceilings on it.
The gate (`tests/repo_gates.rs`) already holds the raf-source examples carve-out and asserts it
is needed.

## Design
* `Budget::Work` (a new `#[non_exhaustive]` variant; `BUDGETS` and `Budget::ALL` grow to 15): the glob-table work of one load, charged where the settle already charges and releases glob bytes for what it copies: one unit per read listed or gathered when a view is built (`glob_scope.rs` `charge_work` at the view gather), one unit per read a change is passed on to (`pass_on`, `arrive`), one unit per step of the supply search (`holding`'s `step`). These are the sites of the re-reading and copying that the source-index capability names as the superlinear cost of the fan-in, prelude and hub shapes; each holds `&Meter` and returns `Result<_, LoadError>`, so `Meter::work(&self, units, file, line) -> Result<(), LoadError>` (a `Cell<usize>` that is never released) refuses at the charge that passes `max_work`. The `cfg(test)` kinds (`ViewReads`, `SupplySteps`, `WalkSteps`, `Lookups`, `SegmentReads`, and the parse-thread kinds, which cannot reach a `&Meter` because it is not `Sync`) keep the test tally and are not budgeted: `Lookups` and `SegmentReads` also run after the load under `Workspace::resolve`, and `WalkSteps` belongs to `exported_items`, which runs after the load on its own meter. A load without glob imports has `work=0`. `Budget::Work.name()` is "glob table work steps".
* `LoadOptions::max_work: usize` (default: the limit of `Budget::Work`), `Usage::work: usize`.
  A load that passes it is refused with `LoadErrorKind::Budget { budget: Work, limit }`, naming
  the file, like the other budgets.
* The `work.rs` test tally and its sites are unchanged; the budget is a separate `Cell<usize>`
  on the `Meter` the load already threads. The default limit is set above the highest figure of the repository,
  the fixtures and the local registry scan (Task 4), with the figure and the shape that reached
  it documented next to the limit, as for the byte budgets.
* The example prints `work=N` in the `OK` line and in the closing largest-value block, takes
  `--max-work N` (a whole number of at least 1; default the limit of `Budget::Work`) and no timer: `wait` is a blocking `Child::wait`.
  A child that aborts or runs out of memory still ends its own line. A child whose load is
  refused by the work budget prints `ERR Budget :: <file>:<line>: exceeds the budget for glob table work steps (<limit>)` like the other budgets, and the closing block counts it under `# err Budget`. The
  outcome kind `timeout` and the flag `--timeout` are removed (stated in the module doc and the
  changelog). Remaining honest limit, written in the module doc: a child that spins in code the
  work counter does not see (parsing is bounded by the token budget) is not stopped.
* The gate: the raf-source rule gets `examples` in its directories; the carve-out arm, its
  stale-carve-out assertion and the exemption fixture are removed; clippy's allow in the example
  is removed. The signal-free child handling keeps the example compiling on every target
  (`make cross-check`).

## Completeness contract
| R | Requirement (ledger) | Task | Executable check |
|---|---|---|---|
| R1 | Replace the per-crate timeout and the sleep poll with a bound on counted work | 1, 2 | Task 1 tests (exact work counts on fixtures; a load at `max_work` accepted and one unit below refused with `Budget::Work`; a settle-heavy shape (the hub) refused at a small `max_work`: the load returns `Err` and the `work` at the refusal is `max_work` plus the last charge, below the `Usage::work` of the same shape loaded at the default); Task 2: the example run over a scratch registry prints `work=` and, with `--max-work 1000` on a crate that needs more, `<name>@<version> | ERR Budget :: ... exceeds the budget for glob table work steps (1000)` for that crate, `OK ... work=N` with N <= 1000 for the small one, and `# err Budget=1` |
| R2 | Extend the gate to `examples/` once clean | 3 | `cargo test --locked --test repo_gates` passes with the carve-out gone; scratch-clone mutant: a planted `Instant::now()` in `crates/raf-source/examples/` fails the gate with file and line; `make clippy` without the allow |
| R3 | Acceptance grep empty | 2 | `grep -rnE 'std::time\|Instant\|Duration\|sleep\|recv_timeout\|SystemTime' crates/raf-source/examples` shows hits before the change (lines 64, 72, 75, 562, 572 are the baseline) and none after (`-r` is required: without it grep on a directory prints nothing) |
| R4 | The repository gate passes with `examples/` in scope | 3 | as R2 |
| R5 | No wall-clock measurement or time-based testing (repository rule) | 1-3 | the gate and clippy ban pass over the raf-source examples; no test added reads a clock |
| R6 | The gate keeps clock spellings out of the crate's source, tests (and now examples) | 3 | `clock_rule`/`CRATE_DIRS` coverage fixtures: a planted word in each of `src`, `tests`, `examples` of raf-source is reported with file and line; `Duration` and `elapsed` (refused only by raf-source's own word list) planted in the example are reported |

## Tasks (each: implement, independent review, commit, push, issue update)
### Task 1: the work budget in the index (crates/raf-source)
Files: `budget.rs` (variant, table row, `name()`, doc), `options.rs` (`max_work`), `meter.rs` (`work` cell, `Meter::work`, `Usage::work`), `glob_scope.rs` (the work charge beside `charge_work`/`meter.globs` in the view gather, `pass_on`, `arrive` and `holding`), `error.rs`/`lib.rs` re-exports, `docs/capabilities/source-index.md` (budget table, `Usage`, Verification rows and bindings, symbol digests re-assessed, and the sentence "No budget bounds the time of these loads" rewritten to say the work budget refuses them at `max_work`), `CHANGELOG.md`. The `work.rs` tally and its 21 sites are unchanged.
Tests (counted, no clock): exact `Usage::work` on fixtures; refusal at `max_work` exactly one
below the need and acceptance at the need; the refusal arrives from a small `max_work` on the
settle shapes of the capability (private fan-in, prelude, hub) before they are settled to the
end (the metered count at the refusal equals the limit plus the last charge); the existing
growth tests still pass. The counter sees the superlinear cost: for the hub shape of `glob_scope.rs`'s module doc, doubling n multiplies `Usage::work` by more than 4 (and for the prelude and private fan-in shapes by more than 2), while for a plain chain of globs it at most doubles; counts only. Mutants (scratch clone): the charge dropped from `pass_on` alone (the hub test fails); charge dropped at a site; limit compared with
`>=`; the meter not reset between loads.
### Task 2: the example without a clock
Files: `crates/raf-source/examples/registry_scan.rs`. As in Design. Unit tests of the parts that
stay pure (the flag parser, the line builder and tally with `work=`) under `cfg(test)` with
`test = true` for the example; no `ExitStatusExt`, so Windows still builds. Checks: R3 grep; a
scratch registry of two crates (one tiny, one needing more than `--max-work 1000`) shows `OK`
with `work=` and `ERR Budget :: ... glob table work steps`; `--max-work 0` is refused with a message naming the flag and `--max-work 1` over the tiny crate (no glob imports) prints its `OK ... work=0` line (both sides of the new refusal); `make cross-check`.
### Task 3: the gate covers the examples (tests/repo_gates.rs, AGENTS.md, CONTRIBUTING.md)
As in Design and R2/R6.
### Task 4: calibration and the capability text
Files also: `budget.rs` limit and docs, `source-index.md`. Run the example over the local registry (`~/.cargo/registry/src/*`) and the repository; record
the highest `work=` and its crate; set the limit with margin and document it as the byte budgets
are; update the source-index registry paragraph (it quotes the earlier registry run) with the
new figure only if re-run; `make docs-sync` twice, assessments re-read and accepted.
### Task 5: PR and handoff
Full `make check` (including `cross-check`; hosted CI covers where the targets are absent),
PR "Closes #27", plan posted, least-confident note, the follow-up for calibration drift if any.

## NEEDS-JUDGMENT dispositions
Ledger 1/4 (counted work or remove): counted work. 2/3/5 (gate scope): R4 governs; Task 3.
Prior art: the examples directory is in this repo; no licence question; issue 21 is the
precedent for the gate and the ban.

## Declined with reasons
Kernel CPU limit (`ulimit -t`): time bound; soft/hard limit and signal subtleties; host
dependent. A shared work crate: one consumer. Removing the example: loses the registry figures.
Outcome enum in the example: the line format is the contract the tally and the doc share; a typed outcome is a refactor of a scan that is not a gate (cost: the kind table stays in three places). The other budget flags on the scan: not needed for this bound (cost: byte-budget calibration stays a manual edit). Gate coverage assertion over every CRATE_DIRS entry: declined, because R6's planted-word fixtures in each of src, tests and examples already fail if a directory is routed to no rule.
