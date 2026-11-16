# Validation for the ratcheted-rule change at 92da1f1f

## Head and base

- Merging head: 92da1f1f on paudley/209-census-debt-a-counted-per-file-ratchet.
- Base: origin/main 106e42d5, unchanged at merge preparation (0 commits behind).
- Net change: crates/raf-census (source, tests, and the ratchet test family moved to census_gate/ratchet.rs), crates/cargo-census/tests/delivery.rs, capability documents, and their assessments.

## Gates and hooks

- The pre-commit hook passed in full on each commit: 08ea247d, dae51adf and 92da1f1f. The hook's exit status was 0 in each case.
- CI run 43841522664 on 92da1f1f: 24 of 24 required checks passed. CodeRabbit reports skipped, which is not a required check.
- `git merge-tree --write-tree` of origin/main against HEAD is clean.

## Focused runs (own commands, exit status from the tool)

- census_gate: 89 tests passed, after the ratchet test family moved (log s2-gate-r5.log).
- clippy with warnings denied, raf-census tests, after the move (log s2-clippy8.log).
- Earlier runs at dae51adf: clippy with warnings denied on raf-census and cargo-census, all targets (s2-clippy7.log); census_gate 89 passed (s2-gate-r4.log).
- The review's probe fixtures reproduced both findings at 08ea247d. The probes were not rerun at dae51adf; the fix is covered by the new test rows (the two-job negative fixture and the json-pointer-escape row in the ratchet loop), which passed in the census_gate run above.

## Independent review

- One independent review at 08ea247d: BLOCKED on two required items. Both are fixed in dae51adf (review review-08ea247d.md, in this evidence directory).
- The fix delta at dae51adf was read once.
- The file-size item from the PR review was fixed in 92da1f1f. Its delta was read once, and I checked that the moved tests are unchanged apart from paths, visibility and the module declaration.

## Checks not run

- No whole-workspace gate was run; the repository's hook and CI cover the wider surface.
- No mutation testing and no timing measurement.
- The line-start performance concern from the PR review is declined (recorded on the review item); the per-hit scan is not measured here.

## Known limits

- Registered-rule and fact-detector hits are keyed at line granularity. A literal earlier on the same line is the one keyed.
- Renaming the enclosing item or the keyed literal changes the site key; this is the same key-churn limit as step 1.
