# Merge-plan review: PR 26 / issue 14 (final recheck, head a730bd2)

- **Reviewed:** `/home/paudley/Active/rust-agentic-foundations/.worktrees/14-raf-source-model-rustc-s-visibility-of.squash_notes` (final), and `validation.md` "Stage 3 head".
- **Mode:** read-only.
- **No shell, so relayed by the coordinator:**
  - the head a730bd2, with CI 12/12 on it;
  - `git diff 47da6fd..HEAD --stat` (documents, CLAUDE.md and a module comment only);
  - the five follow-up issues OPEN;
  - the PR body's correction.

## Prior findings

| Finding | Status | Evidence |
|---|---|---|
| **B1:** failing oracle runs listed as validation | **DISCHARGED** | Notes lines 39-46 give the results per family, and every figure matches `/tmp/final7-*` and `/tmp/final6-*`. |
| **A1:** "no new disagreements" residual | **DISCHARGED** | Lines 48-51: 13 and 12 new probes, all order-dependent in rustc except one glob-cycle probe with a weak baseline (audit delta B). |
| **A2:** follow-ups not named | **DISCHARGED** | Lines 74-76 name 28, 24, 27, 23 and 21, OPEN per the coordinator. |
| **A3:** "fixtures" overgeneralised | **DISCHARGED** | Lines 56-57: "each fixed, the code ones with fixtures". |
| **A4:** pointer to an ignored path | **DISCHARGED** | Lines 76-79 say it is ignored by git and archived by the merge tool. |
| **A5:** `#` template lines under message cleanup | **DISCHARGED** | ghprsq handled the same template in two earlier merges, per the coordinator. |
| **B2:** the unclassified programs called cycles | **DISCHARGED** | **Notes lines 44-46:** "(one of them in the cycle class there) ... until those shapes are modelled". **Capability line 2518:** "on the same probe", with 259 the only one named `cycle`. Checked probe by probe against `/tmp/cmp14w/all-pre-c-11044.tsv` and `/tmp/baseline-compare.txt`. |
| **B3:** commit list and file stat stale | **DISCHARGED** | Line 82: `## Commits squashed (46)`, headed by `a730bd209` and `f28f4575b`, then the 44 entries before them. The `## Files` stat is unchanged from c1e0e73: `source-index.md \| 370`, `78 files changed, 10049 insertions(+), 1012 deletions(-)`. That fits the two commits changing only lines the branch had already added (the Verification sentence and the assessment's recorded fields), so their diff against main adds no new lines. I could not re-run the stat; the coordinator states it was regenerated at a730bd2. |
| **A6:** audit verdict at a head it did not read | **DISCHARGED** | Lines 51-54: the last verdict is "at the head before the last two documentation commits", and CI is 12 of 12 on the final head. `validation.md` "Stage 3 head" names a730bd2 and puts the audit's PASS at c1e0e73. |
| **Nit:** line wrap | **DISCHARGED** | The body is rewrapped to one width. |

## New text

The only new text is the rewrap and the splice of the regenerated commit list and file stat. No new claims were introduced.

### A7 (ADVISORY): the rewrap breaks the stage path

Lines 77-78:

> `.stage/raf-source-model-rustc-s-
> visibility-of`

The wrap splits a code span at a hyphen. A plain-text reader of the log may read the path as two words.

REWRITE TO: put the whole path `.stage/raf-source-model-rustc-s-visibility-of` on one line, and break the line before it instead.

## Summary

Every finding is discharged:
- the validation figures, the residual disagreements, the follow-ups and the corrected capability text all match the logs and the audit;
- the commit list covers the 46 commits to a730bd2;
- the integration evidence (main equals the merge base, CI 12/12 on the head, an exit audit PASS with only documentation after it) is sufficient.

The one remaining item, A7, is cosmetic and does not block.

VERDICT: PASS
