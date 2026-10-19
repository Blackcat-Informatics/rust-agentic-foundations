# Requirements ledger: issue #21: Replace wall-clock assertions in tests with operation or allocation counts

Evidence: issue.md, requirements-candidates.md, brief.json (coverage: issue #21 + 1 comment(s) via github; link crawl: no `#N` references in the issue or its comments; forge search on ['Replace', 'wall-clock', 'assertions']: 1 other item(s); GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits; prior attempts: 0 commit(s) citing #21 (second parents walked), 0 other branch(es) named for it; PRs citing #21: 0; requirement candidates: 9 line(s); 1 comment(s) matched no rule; referenced artifacts: 6 path(s) (5 on disk), 11 identifier(s), 3 workspace package(s), 0 URL(s))

Line convention: "body L<n>" is the line within the issue body (issue.md line minus 6), matching requirements-candidates.md.

## Requirements
| R-id | Kind | Source | Quote (verbatim) | From candidate |
|------|------|--------|------------------|----------------|
| R1 | list-item | body L9 | "- `crates/raf-source/tests/exported_items.rs` (Instant and Duration bounds)" | B1 |
| R2 | list-item | body L10 | "- `crates/raf-okf/tests/validate_cost.rs` (release bound of 100 ms)" | B2 |
| R3 | list-item | body L11 | "- `crates/raf-workplan/tests/assessments.rs` (60 s budget)" | B3 |
| R4 | list-item | body L12 | "- `crates/raf-workplan/tests/rationale.rs` (60 s bound)" | B4 |
| R5 | list-item | body L13 | "- `crates/raf-workplan/src/evidence/tests.rs` (elapsed bound on a killed command)" | B5 |
| R6 | list-item | body L14 | "- the runner's own timeout and output-kill tests, where a count or a deterministic signal can replace the wait" | B6 |
| R7 | list-item | body L18 | "- `grep -rn "Instant::now" crates/*/tests crates/*/src` finds no assertion on elapsed time." | B7 |
| R8 | list-item | body L19 | "- Every converted test still fails on a mutant that removes the bound it protects." | B8 |
| R9 | list-item | body L20 | "- `make check` passes and hosted CI is green." | B9 |
| R10 | prose | body L3 | "The repository rules now forbid time-based testing: no timing runs, wall-clock benchmarks, sleep-based waits or timing assertions." | prose |
| R11 | prose | body L3 | "Measure with `perf` or by counting allocations." | prose |
| R12 | prose | body L7 | "Convert each wall-clock bound to an operation count, an allocation count or a `perf` measurement, keeping the guarantee each test protects" | prose |
| R13 | decision | comment 1 (paudley, 2026-10-18T02:52:36Z) | "Additional scope found while working on the source index: `raf-source` has its own elapsed-time tests beyond `exported_items.rs`." | prose |
| R14 | decision | comment 1 (paudley, 2026-10-18T02:52:36Z) | "Tests that assert or measure wall-clock time" | prose |
| R15 | decision | comment 1 (paudley, 2026-10-18T02:52:36Z) | "live in `resolve_paths`, `manifest_caps`, `trait_keys`, `module_tree`, `exported_items`, `prescan_budgets`, `symbol_index`, `facts` and `tree::disk`." | prose |
| R16 | decision | comment 1 (paudley, 2026-10-18T02:52:36Z) | "Each needs the same conversion to an operation or allocation count." | prose |
| R17 | decision | comment 1 (paudley, 2026-10-18T02:52:36Z) | "Until then a whole-crate `cargo test -p raf-source` runs them; the per-test skip list is long, so the conversion is the real fix." | prose |

Kinds: checkbox, modal, list-item, scope-out, dependency, question, decision, prose.

## Candidate dispositions
| Candidate | Disposition |
|-----------|-------------|
| B1 | kept → R1 |
| B2 | kept → R2 |
| B3 | kept → R3 |
| B4 | kept → R4 |
| B5 | kept → R5 |
| B6 | kept → R6 |
| B7 | kept → R7 |
| B8 | kept → R8 |
| B9 | kept → R9 |

Dropped: 0. Prose additions: R10, R11, R12 (body L3, L7). Comment 1 matched no rule; its R-rows are R13 to R17.
Not turned into rows: body L7 and L3 statements of current state ("Several tests added by the planning-essay work still assert on elapsed wall-clock time.") and the "## Acceptance" heading. These state present fact or structure, not a required behavior.

## Comment coverage
| Comment | Author | Rows / disposition |
|---------|--------|--------------------|
| 1 | paudley | R13, R14, R15, R16, R17 |

Comments: 1 of 1 covered.

## Dependencies (from brief.json relations)
- #14 related (no typed relation in brief.json; `related` list only) [CLOSED] "raf-source: model rustc's visibility of re-exported private glob bindings and of ambiguity through inaccessible globs"

## NEEDS-JUDGMENT
1. R13 vs R1: "Additional scope found while working on the source index: `raf-source` has its own elapsed-time tests beyond `exported_items.rs`." / "- `crates/raf-source/tests/exported_items.rs` (Instant and Duration bounds)": Does the issue scope cover the raf-source tests named in R14 and R15, which the body list does not name? The body names only exported_items.rs for raf-source; the comment adds the others.
2. R16 vs the invoking instruction: "Each needs the same conversion to an operation or allocation count." / the instruction's note that the raf-source part "has since been delivered by another change": Is the raf-source conversion still required under #21, or is it closed by the other change? The stage has no record of which change delivered it or which tests it covers. brief.json lists only #14 as related, and #14 is titled as a visibility issue, not a timing issue. The note is not verified against the stage.
3. R7 vs R10: "`grep -rn "Instant::now" crates/*/tests crates/*/src` finds no assertion on elapsed time." / "The repository rules now forbid time-based testing: no timing runs, wall-clock benchmarks, sleep-based waits or timing assertions.": The acceptance check matches only `Instant::now`. R10 also forbids sleep-based waits and timing assertions. Which check defines done for the non-Instant forms?
4. R6 vs R11 and R12: "where a count or a deterministic signal can replace the wait" / "Measure with `perf` or by counting allocations.": R6 allows a "deterministic signal", which R11 and R12 do not name among the methods (operation count, allocation count, `perf` measurement). Is a deterministic signal an allowed replacement?

## Self-check
Quotes checked: 17. Misses: 0. Comments covered: 1/1.
