# Plan final review (Stage 1), final recheck against Revision 3

Reviewed plan: `.stage/replace-wall-clock-assertions-in-tests/plan.md`, Revisions 1-3 (lines 50-105), each superseding the earlier text where they differ; design source `clock-inventory.md`.

Reports and dispositions examined: `reviews/plan-{completeness,constraints,design,enhancement,adversary}.md` and the Revision 1, 2 and 3 dispositions of every finding (records of the earlier passes below).

Code checked across the three passes: `crates/raf-okf/src/profile/mod.rs:598-649`, `records.rs:589-626`, `body_rules.rs:41-63`; `crates/raf-source/src/work.rs`; `crates/raf-workplan/src/exec/system.rs:95-292`; `crates/raf-workplan/tests/exec.rs:1688-1709`; `tests/repo_gates.rs:251-302`; `tests/docs_contract.rs:1304-1367`; `crates/raf-source/examples/registry_scan.rs:64,556,566`; `crates/raf-workplan/Cargo.toml:14-25`; `clippy.toml`; `Cargo.toml:56`; `Makefile:12-19,58-66,121-124`; `AGENTS.md:159-162`.

## Recheck of the Revision 2 findings

| Finding | Status | Evidence |
|---|---|---|
| B3' hold release | **DISCHARGED** | line 102: the hold FIFO is opened `O_RDWR` before the spawn and kept; the release is a plain `write` of one byte; the `ENXIO` and stranded-subject reasons are stated; the pid FIFO is held the same way; the R5 P1 fixtures use the same guarantee. With the test holding a writer, the subject's `open` of the hold never blocks and the release cannot fail; the marker decision after `poll(pidfd)` is deterministic (SIGKILL is queued before `run_with` returns; a SIGKILLed task never runs user code again). Holding the pid FIFO `O_RDWR` removes `POLLHUP` on a dead writer, but the leader pidfd polled alongside it (Revision 1, "Never-ending scripts") still turns a dead script into a failure, so no hang path is reintroduced. |
| A1 test 6 mutant direction | **DISCHARGED** | line 103: `length > bound` (`system.rs:191`) mutated to `>=`, refusing a 1000-byte output. Matches the code. |
| A2 counter mutants and task | **DISCHARGED** | line 104: `ContractLines` and `HeadingsScanned` under Task 2 with named quadratic mutants; `FenceScans` under Task 1 with its mutant. |
| A3 rustix dev-dependency | **DISCHARGED** | line 105: a new `[target.'cfg(unix)'.dev-dependencies]` rustix entry with `fs`, `event`; the existing `[target.'cfg(unix)'.dependencies]` line (`Cargo.toml:21-22`, `process, std`) unchanged. Cargo unifies features per build, so the production feature set does not grow. |

## Findings

None BLOCKING. None ADVISORY beyond what Revision 3 closed.

## Open user decisions

None. The Windows narrowing (b) and the removal of the fixture budgets are taken under the standing platform rule and the timing rule and are recorded in the PR (Revision 2, line 99).

## Whole-plan judgement

Read end to end as the implementer will, with Revisions 1-3 layered over the tasks: the ten tasks are in dependency order (counters and counted tests first, the runner seam as a behaviour-preserving refactor before the tests that need it, the evidence record recorded again only against the clock-free subject, budgets and nonces once the timing tests are gone, the gate last over a clean tree, the PR as the final task); there is no Task 0; every requirement in the issue and its comment has a contract row with an acceptance an implementer can run; every converted test has a named mutant and the mutant table in validation.md is required to show the failing assertion, with hangs ruled out as a detection path by the ceilings on both test clocks and the hold-and-marker protocol; the Linux-only checks and the Windows narrowing are explicit; the declines (coverage index, shared cost crate, event-driven runner, public clock seam, counting cargo shim, perf target) each carry a technical reason and a visible cost; conventions (SPDX, assessments re-assessed per commit, no issue numbers or process wording in committed text, hooks never bypassed) are stated where the implementer needs them.

Residual limits the plan states honestly rather than hides: a brand-new uncounted loop is not caught by counts (the okf-profile Verification text records it); an added search beside a kept index survives the raf-okf mutants; the `docs_contract.rs` loader's refusal of a FIFO is now checked by type, not by non-blocking behaviour; `WallClock` measuring real time is trusted to std.

VERDICT: PASS

---

## Record of the second pass (Revision 2)

B1, B2, B4, B5, B6, B7 and advisories A1, A3, A5, A6 of the first pass DISCHARGED; B3 narrowed to B3' (the `O_NONBLOCK` write-end open of the hold FIFO: `ENXIO` on the pass path and a stranded-subject window on the mutant path); advisories: test 6 mutant direction, the new counters' mutants unnamed and mis-assigned, no `[dev-dependencies]` table yet. Verdict then: NEEDS-REVISION.

## Record of the first pass (Revision 1)

| Report | Finding | Disposition in Revision 1 | Verified |
|---|---|---|---|
| completeness | B1 raf-source contract row | table row 1 | yes |
| completeness | B2 coverage mutant | "Coverage mutants" | yes |
| completeness | B3 inputs without a bound | "Inputs without a bound" | counters named, mutants not |
| completeness | B4 root `tests/docs_contract.rs` | "Inventory additions", rows 8 and 9 | gate self-hit (B1), hang (B6) |
| constraints | C1-C7 | per-item lines | yes, except the C7 grep claim (B5) and the C5 subject change not carried into the kill check (B3) |
| design | B1, B2, A3, A4 | "Exact counts", "Per-invocation clock", "Kill detection", "Never-ending scripts" | yes |
| enhancement | 1-5 | 3, 4, 5 taken; 1, 2 declined with cost | 4 taken without its consequences (B2) |
| enhancement | 6-13 | missing | B7 |
| adversary | F1-F7 | per-item lines | F2 garbled (B4); F6/F7 left as user decisions |

First-pass findings: B1 gate flags `tests/repo_gates.rs`; B2 clippy ban breaks `make clippy` on `registry_scan.rs`; B3 kill check hangs on the mutant after the subject changed from `kill -STOP` to a FIFO read; B4 documented-regression paragraph garbled and the helper routing inapplicable to `visible_content`, R3 tests 4 and 6 without mutants; B5 literal grep "empty" impossible; B6 `docs_contract.rs` conversion relies on a hang; B7 enhancement items 6, 7, 8, 9, 11, 12 undisposed. Advisories: rustix `fs`/`event`, mutants for the new counters, commit `5aced3c` unverified, task lines unrevised, "105 looks", Task 4 filter. Verdict then: NEEDS-REVISION.
