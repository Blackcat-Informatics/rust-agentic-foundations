# Merge-plan review: PR 22 (issue 9, workplan part)

**Reviewed:** `/home/paudley/Active/rust-agentic-foundations/.worktrees/9-acceptance-scenarios-a-b-workplan-self.squash_notes`
**Evidence read:** validation.md (Task 3, Stage 2 result, Stage 3 integration assessment), review-debt.md, gap-analysis.md, plan.md (Revision 2, local validation, dispositions), issue.md, branch-under-review.md, pr-checks.txt, pr-comments.md (plan text), git refs.
**Mode:** read-only. No tests, no timing runs.

## Focus checks

| Question | Result | Evidence |
| --- | --- | --- |
| Does the reused Stage 2 review still apply to the head? | YES | gap-analysis.md reviews head `1da058c798b28616c6dbe6d5d7d32cbf80ab2d78` on base `a9d6852`. Local branch ref, `origin/` branch ref and the squash-notes commit list all show `1da058c79` at the tip, and `origin/main` is `a9d6852df8c1...`. No commit landed after the review. |
| Is the integration evidence sufficient? | YES | main equals the merge base, so the merge candidate is the reviewed tip. `merge-tree` exits 0. pr-checks.txt shows 12/12 SUCCESS. gap-analysis.md names run 39055703234 on `1da058c`, with `repository_workplan_is_clean_and_assessed ... ok` in the `test` job log. No merge of main or rerun is needed. |
| Are the dispositions honest? | YES | review-debt.md: PR 22 has no reviews, threads or bot feedback. The only PR comments are the implementer's own plan and summaries, which matches pr-comments.md. The four Stage 2 LOW items are stage-file or cosmetic issues and are marked as not blocking merge. The notes report them as "four low notes". |
| Do the notes claim anything the evidence does not show? | NO (one advisory on wording) | Each claim checks out. Hook and workspace wiring: gap "Hook proof" and "Wiring". Seven docs gained the block, with a fixture and clean twin: gap row 7 and the `matrix` fault. Scenarios: 5 passed (gap and validation). Fault injection on a broken link, a stale assessment and a rejected verdict: gap fault table. Five milestone capabilities completed by the tool at their latest delivery: validation Task 3 and gap "Reviewer Feedback" row 2. The milestone is defined as five capabilities in plan Revision 2, line 41. The "tracked issue" for the timing tests is issue 21 (plan lines 76 and 89). |
| Does the commit close the issue, or is a requirement silently missing? | NO / NO | The notes say "Part of #9 (the issue stays open for the self-census gate)" and use no closing keyword. The one unmet issue criterion is the self-census half of "Self-workplan and self-census gates both pass in CI", plus its planted-duplicate fixture. The notes disclose it with the owner's verbatim decision. Every other issue checkbox maps to evidence: scenarios A and B (prior work, passing), `tests/self_workplan.rs` wired into `make check` and the hook, records completed with the tool, and delivered capabilities with resolved bindings and generated matrices (12/12, now enforced). |

## Findings

### ADVISORY 1: the timing sentence implies a fuller local gate than the evidence shows

Offending line in the notes:

> The `raf-okf` wall-clock probe in `make check` is not run on this machine (no timing runs); hosted CI runs it, and a tracked issue covers converting such tests.

The sentence is literally true. Read with "Validation: focused tests ...", though, it suggests that the rest of `make check` ran locally and only the `raf-okf` probe was skipped. The recorded local evidence is narrower: the focused test binaries, the scenarios and crate doc-reading tests (gap-analysis.md), and fmt and clippy (T2-review.md). No stage file records the plan's local validation list (plan.md line 76: `make fmt clippy doc deny reuse msrv fuzz cross-check licenses issue-refs`, `cargo test --workspace --locked -- --skip within_the_bound`).

The skips also go beyond the `raf-okf` probe. The gap run skipped the raf-workplan `bounded_time` tests, and the `within_the_bound` filter also matches `crates/raf-okf/tests/alloc_bound.rs` and `crates/raf-workplan/src/staged.rs`. This does not block merge, because hosted CI ran every job on the tip.

REWRITE TO: `Locally, the focused tests above, fmt and clippy ran; the full make check did not, under the no-timing rule. Hosted CI ran every job on the tip, including the wall-clock tests, and a tracked issue covers converting those tests.`

### ADVISORY 2: validation.md Task 3 remains partial (Stage 2 LOW-1)

The per-record listing still omits the later removals (`4dcdebc`, `f5a88ff`) and the five later-essay capabilities. The Stage 2 reviewer verified them independently ("each record's latest removal is a tool-shaped 'deliver ... again' commit"), so the notes' claim is supported. This affects a stage file only, and the notes need no change.

No BLOCKING findings.

VERDICT: PASS
