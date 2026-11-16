# Validation: issue 272 (failing measured record, unit without value)

VERDICT: PASS

The merging head is 12ec0258. The full independent review at 5421da6b found one missing assertion (review-5421da6b.md). Commit 12ec0258 adds it, and a bounded independent read of that diff passed (delta-12ec0258.md). Hosted CI on 12ec0258: 25 of 25 checks passed. The merge preflight reported no blockers. Coordination capped the review rounds at seven; this was the seventh, and wording or extra-test findings after it were applied without reopening review.

## Scope

Nine commits on paudley/272-workplan-a-failing-measured-record-does, 09c1ca0b through 12ec0258. The base and the branch change disjoint files, `git merge-tree` is clean, and CI ran on a merge ref containing the current main (0b8605a5).

## Evidence

- Pre-commit hook for 12ec0258 (the last commit log in the worktree run directory), read by me: repo_gates 29, docs_contract 81, self_census 4, self_workplan 1, all passed. The cargo wrapper mtime and sha256 were unchanged from start to end, and no log shows a wrapper error.
- Focused runs on 12ec0258, exit 0: raf-workplan strict 25 and kinds 6, clippy with warnings denied.
- Review runs at 5421da6b, exit 0 with the wrapper unchanged: strict and kinds, raf-okf records, clippy on both changed crates, self_workplan and docs_contract.
- PR review: the one actionable CodeRabbit finding was declined with a reply citing the issue acceptance, and the reviewer withdrew it.
- Source reading by me of the merging diff: `strict.rs` measured_evidence and Currency, `digest.rs` stale_symbol_inputs, `diagram/semantic.rs` verification_fields, `finding.rs` kinds and fix text, raf-okf `records.rs` and `violation.rs`. Not read by me: the test files and the renewed assessments, which the independent reviews covered.

## Wrapper incident

Shared cargo wrapper edits at 19:24-19:48 and a rename at 20:39 invalidated runs started before 19:48:12.84. All evidence above comes from runs after that time with the wrapper identity unchanged across each run.
