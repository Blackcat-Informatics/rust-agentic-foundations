# Issue #21: Replace wall-clock assertions in tests with operation or allocation counts

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github

## Body

## Problem

The repository rules now forbid time-based testing: no timing runs, wall-clock benchmarks, sleep-based waits or timing assertions. Measure with `perf` or by counting allocations. Several tests added by the planning-essay work still assert on elapsed wall-clock time.

## Scope

Convert each wall-clock bound to an operation count, an allocation count or a `perf` measurement, keeping the guarantee each test protects (bounded validation cost, bounded assessment and rationale work, bounded exported-items resolution, bounded evidence runs):

- `crates/raf-source/tests/exported_items.rs` (Instant and Duration bounds)
- `crates/raf-okf/tests/validate_cost.rs` (release bound of 100 ms)
- `crates/raf-workplan/tests/assessments.rs` (60 s budget)
- `crates/raf-workplan/tests/rationale.rs` (60 s bound)
- `crates/raf-workplan/src/evidence/tests.rs` (elapsed bound on a killed command)
- the runner's own timeout and output-kill tests, where a count or a deterministic signal can replace the wait

## Acceptance

- `grep -rn "Instant::now" crates/*/tests crates/*/src` finds no assertion on elapsed time.
- Every converted test still fails on a mutant that removes the bound it protects.
- `make check` passes and hosted CI is green.


## Comments (1)

### paudley — 2026-10-18T02:52:36Z

Additional scope found while working on the source index: `raf-source` has its own elapsed-time tests beyond `exported_items.rs`. Tests that assert or measure wall-clock time (names end in `..._in_time_linear_in_its_names`, `..._in_linear_time`, `..._in_bounded_time_and_bytes`, `..._load_at_once`, `..._is_refused_at_once`, `..._quickly_...`, `..._in_under_a_second`, `..._takes_time_in_proportion_...`, and the watchdog tests of `tree::disk::tests`) live in `resolve_paths`, `manifest_caps`, `trait_keys`, `module_tree`, `exported_items`, `prescan_budgets`, `symbol_index`, `facts` and `tree::disk`. Each needs the same conversion to an operation or allocation count. Until then a whole-crate `cargo test -p raf-source` runs them; the per-test skip list is long, so the conversion is the real fix.


