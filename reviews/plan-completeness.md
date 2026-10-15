# Plan completeness review (third recheck)

Reviewed plan: `.stage/raf-workplan-change-lifecycle-cargo/plan.md`, Revision 3.

This recheck covers the three findings the second recheck left open (N1, A7, A8). It also reviews the
text added to fix them: Task 11, Task 9, D8, the R1 row and the new rows in the Contract-edit table.

## Prior findings

| Finding | Status | Evidence |
| --- | --- | --- |
| N1 Task 11 order | DISCHARGED | Task 11 (lines 241-244) now reads "set `state: delivered` on the three capabilities (D9), then run `cargo-workplan change reconcile` on each of the three records (empty list, exit 0) and `change complete` on each (exit 0), output into `validation.md`; commit the state edits, the record deletions and the generated rewrite together". This matches D9, the R17 row and the state rules in `rules.rs:143-174`. |
| A7 Task 9 R1 wording | DISCHARGED | Task 9 (line 227) and the R1 row (line 155) now require exactly one `Graph::proposals()` block, `check` exit 0 and one `unresolved-proposal` line. Both run the check in each of the four states after the printed state guidance is followed. D8 (line 80) covers `delivered` and `designed` with "set `state: implementing`", which matches `rules.rs:138-150`. `considered` and `implementing` need no guidance, which is also correct. |
| A8 revision label | DISCHARGED | Line 3 reads "Revision 3". |

Every earlier finding is now discharged: B1-B7, N1 and A1-A8.

## New text reviewed

- **Task 11** says "write the Contract edits as PROPOSED blocks and resolve them". This puts the
  repository's own Contract edits through the lifecycle the plan builds. That is consistent with D2
  and creates no gap.
- **Task 11** says "no Contract, Constraints or Verification statement outside the Contract-edit
  table is removed or weakened; an unbuilt statement is built or put to the owner". This forbids
  silent cuts. The owner route is an explicit escalation, not a planned deferral, so it is not a
  defect.
- **Contract-edit table.** Two rows are new: the complete-gate rule "never edits prose", and the
  16 MiB per-stream capture bound. Both are listed with their reasons, so neither is a hidden cut.

## Requirement -> task -> check

| R-id | Task | Check | Status |
| --- | --- | --- | --- |
| R1 | 4, 9 | record loads clean; three failures; repeated `--capability`; template in each state gives one `proposals()` block, `check` 0 and one `unresolved-proposal` line | OK |
| R2 | 4, 8-9 | impact fixture; tree byte-identical | OK |
| R3, R19, R23 | 5, 6 | kind, location, evidence; deterministic; exit 1 iff non-empty | OK |
| R4, R5, R6, R8, R9 | 5 | fixture per kind; fixing removes the line | OK |
| R7 | 6 | real-git matrix, version, blob bounds | OK |
| R10 | 7 (with D9) | refusal per condition and per kind; state fixtures; edge cases | OK |
| R11 | 6 | `verify` matrix, exit 0/1, nothing written | OK |
| R12 | 8 | preview CLI test | OK |
| R13 | 8, 9 | parse tests, both argv shapes, exit 0/1/2 through the binary | OK |
| R14 | 5-7 | fixture per kind plus a `complete` refusal per kind | OK |
| R15 | 7 | file gone, generated rewrite, `Gate` clean after completion | OK |
| R16 | 8, 9 | per-command binary fixtures; read-only identity; `-q` | OK |
| R17 | 11 | own records: `reconcile` empty, exit 0; `complete` exit 0; output recorded | OK |
| R18 | 4 | exact `affects` value; `impact` reports it | OK |
| R20, R21 | 7 | refusal until conditions hold; fixed write order with deletion last | OK |
| R22 | 10, 11 | three capabilities `delivered` through the shipped binary; Scenario A and B built | OK |
| R24 | all | merged API | OK |

All six ledger NEEDS-JUDGMENT items and all four prior-art NEEDS-JUDGMENT items have dispositions.

Structure: Tasks 1-12, no Task 0. Each task names its tests, has an independent review, commits by
explicit path, pushes and updates the issue. Task 12 runs `make check`, checks that `.deficiencies`
is empty, creates the PR, publishes the plan and hands off. The structure passes.

## Findings

None.

VERDICT: PASS
