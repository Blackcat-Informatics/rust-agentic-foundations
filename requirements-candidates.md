# Requirement candidates — issue #21

Lines from the issue body and comments that matched a lexical rule, copied
verbatim with the rule that fired. This is a CANDIDATE list: rules both
over-match (a "should" in passing) and under-match (a requirement stated
without a keyword). The issue-analyzer keeps or drops each row, adds what
the rules missed, and accounts for every silent comment. Code fences are
skipped.

| ID | Source | Line | Rules | Text |
|---|---|---|---|---|
| B1 | body | 9 | list-under-requirement-heading | - `crates/raf-source/tests/exported_items.rs` (Instant and Duration bounds) |
| B2 | body | 10 | list-under-requirement-heading | - `crates/raf-okf/tests/validate_cost.rs` (release bound of 100 ms) |
| B3 | body | 11 | list-under-requirement-heading | - `crates/raf-workplan/tests/assessments.rs` (60 s budget) |
| B4 | body | 12 | list-under-requirement-heading | - `crates/raf-workplan/tests/rationale.rs` (60 s bound) |
| B5 | body | 13 | list-under-requirement-heading | - `crates/raf-workplan/src/evidence/tests.rs` (elapsed bound on a killed command) |
| B6 | body | 14 | list-under-requirement-heading | - the runner's own timeout and output-kill tests, where a count or a deterministic signal can replace the wait |
| B7 | body | 18 | list-under-requirement-heading | - `grep -rn "Instant::now" crates/*/tests crates/*/src` finds no assertion on elapsed time. |
| B8 | body | 19 | list-under-requirement-heading | - Every converted test still fails on a mutant that removes the bound it protects. |
| B9 | body | 20 | list-under-requirement-heading | - `make check` passes and hosted CI is green. |

## Comments that matched no rule

- comment 1 (paudley, 2026-10-18T02:52:36Z)
