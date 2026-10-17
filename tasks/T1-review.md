# T1 review

Scope: uncommitted diff (7 capability docs, 7 re-assessed work/assessments files, tests/docs_contract.rs).

Evidence:
- cargo test --test docs_contract: 29 passed.
- workplan check: rc 0, no output. assess --list | grep -ci stale: 0.
- grep -L matrix marker: only index.md and the 9 census-* docs (all non-delivered/generated); all 12 delivered docs hold it.
- make docs-sync: diff hash identical before and after (no diff).
- Assessments: each of 7 changes only the digest line; SPDX header intact in all 12 files.
- Added lines contain no M1, issue-number or process wording.
- Rule non-vacuous: matrix_violations is wired into violations(), keyed off INVENTORY delivered state; the fixture test removes the block (expects exactly 1 violation naming verification-matrix), a quoted-marker case (still 1), and the clean twin (0). inventory_repo gets the block so the inventory tests stay clean.

VERDICT: PASS
