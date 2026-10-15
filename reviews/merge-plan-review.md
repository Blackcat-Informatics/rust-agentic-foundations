# Stage 3 merge-plan review: raf-workplan change lifecycle + cargo-workplan

Reviewed: `/home/paudley/Active/rust-agentic-foundations/.worktrees/8-raf-workplan-change-lifecycle-cargo.squash_notes`
(merge candidate d46f95e onto base 97795488), with `validation.md` (Stage 3 section and the
merge-time facts), `review-debt.md`, `pr-checks.txt`, `pr-comments.md`,
`reviews/completion-audit.md`, `tasks/T1..T11-review.md`, `requirements.json`, `issue.md`, the
worktree source, `Makefile`, `.github/workflows/ci.yaml`, `CHANGELOG.md`, `AGENTS.md`,
`.gitignore` and `.deficiencies`.

Tooling limit: this reviewer has file read/search tools only, with no shell. The `gh` and
`git diff` results below are the ones the coordinator recorded in `validation.md`. This review
did not re-run them.

## Recheck of the prior findings

| # | Prior finding | Status | Evidence |
| --- | --- | --- | --- |
| 1 | BLOCKING: the squash notes omit the four out-of-table Contract edits | **DISCHARGED** | Notes line 33: "Contract wording changed beyond the original edit list, for the owner to accept: the tense scan skips HTML comments, link destinations, generated blocks and the capitalised word PROPOSED and reads whole words only; rustdoc list paths read a backslash as a slash ...; 'never write files' is 'never write a file in the repository'; `change new` takes `--copyright` and `--licence` ...". This covers all four audit REQUIRED 3 items. Code spans are left out correctly, because they come from plan D6 and are not an out-of-table edit. The coordinator reports the PR body carries the same list. |
| 2 | BLOCKING: three merge preconditions not re-run | **DISCHARGED (on recorded evidence)** | `validation.md` line 155 records: headRefOid d46f95ec65b57acff3bf9ccdaea532f414b8456d with all 12 hosted jobs passing; 0 review threads and 0 reviews; `git diff --stat a72c5f5 d46f95e` touching only `crates/*/tests` paths. The headRefOid agrees with the worktree reflog head (d46f95ec65b5...). |
| 3 | ADVISORY: the fixture fix is overstated | **STILL-OPEN** (advisory) | Notes lines 41-42 still say "fixed by giving each fixture its own". `crates/raf-workplan/tests/support/project.rs:45-52,91` still shares one target directory between the two scenario fixtures. Harmless, as noted before. |
| 4 | ADVISORY: commit of the local gates | **DISCHARGED** | Notes line 37: "each `make` target was run individually on 9f1ab69 (the last commit changes test fixtures only and is covered by the hosted run)". |
| 5 | ADVISORY: cross-check scope | **DISCHARGED in validation.md; notes unchanged** | `validation.md` line 155 states that cross-check covers raf-source only and that the macOS/Windows jobs carry the new crates. Notes line 40 still reads "all 12 jobs including macOS, Windows and cross-check". That is accurate, just less precise. |
| 6 | ADVISORY: overlap with the tracked scenarios/self-gate issue | **STILL-OPEN** (advisory, tracker action) | Nothing in the inputs shows the other tracked issue was updated. |
| 7 | ADVISORY: `workplan-cli.md:102` status wording | **STILL-OPEN** (advisory, predates this branch) | Unchanged. |
| 8 | ADVISORY: second `docs_sync` behaviour change | **DISCHARGED** | Notes line 33: "a root inside a git checkout without its own `.git` entry now lists its tracked paths". |

## Review of the new text

* The notes' new owner-acceptance sentence matches the plan's Contract-edit rows (Task 5, Task 3,
  Task 6, Task 9) and the audit's REQUIRED 3. It contains no tracker number, process wording or
  attribution.
* `validation.md` line 155 says the diff touches only test paths "(plus the stage-ignored files)".
  * `.gitignore:5` ignores `/.stage/`, and the squash file list (109 files) names no `.stage` path. So
    no stage artifact is committed.
  * The parenthesis is odd wording for a tracked-tree diff, but it does not contradict the evidence.
    ADVISORY: drop it or say "no other path".

## Conclusion

Integration evidence is sufficient for the candidate:
* the base is an ancestor of the head;
* the candidate tree equals the tested tree;
* all 12 hosted jobs pass on the recorded headRefOid d46f95e;
* local gates are clean on 9f1ab69, and the delta since then is test-only.

Reusing the completion audit across 9f1ab69 and d46f95e holds, because the recorded diff touches
only `crates/*/tests`. There are no review threads or reviews.

Every issue requirement (B1-B17) maps to behaviour exercised through the binary, and none is
silently missing or moved. The owner readings are stated honestly in the notes:
* the printed PROPOSED template;
* all three capabilities delivered, with Scenarios A and B built here;
* `rustix`;
* both `docs_sync` changes;
* the two dropped claims;
* hosted cross-check by the owner's decision;
* the four extra contract edits.

`.deficiencies` holds only its notice and marker.

## Findings

1. ADVISORY (STILL-OPEN, prior 3): squash notes lines 41-42.
   REWRITE TO: "fixed by giving each `bound` and `cargo-workplan` fixture its own target directory".
2. ADVISORY (STILL-OPEN, prior 6): update the tracked scenarios/self-gate issue to say that the
   scenario part is built here and only the self-gate remains.
3. ADVISORY (STILL-OPEN, prior 7): `docs/capabilities/workplan-cli.md:102`, "planned, implementing
   with gaps, delivered", is loose for a delivered capability. It predates this branch.
4. ADVISORY (new): in `validation.md` line 155, "(plus the stage-ignored files)" is ambiguous for a
   tracked diff. Drop it.

VERDICT: PASS
