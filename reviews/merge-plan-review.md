# Stage 3 merge-plan review: raf-workplan model, documents, check (recheck)

- Reviewed: `/home/paudley/Active/rust-agentic-foundations/.worktrees/7-raf-workplan-model-documents-check.squash_notes` (revised) and `$STAGE/validation.md` (revised Stage 3 section, lines 15-19), together with the evidence already read in the first pass: `review-debt.md`, `reviews/completion-audit.md`, `pr-checks.txt`, `pr-comments.md`, `requirements.json`, `issue.md`, `plan.md`, AGENTS.md section 7 and CLAUDE.md.
- Focus: whether the integration evidence covers the candidate, whether reusing the completion audit holds, whether the thread dispositions are honest, whether the notes claim more than the evidence shows, and whether any requirement is missing or was moved without saying so.

## Reviewer limitation (unchanged)

This review used file-read tools only, with no shell. The diffstat and the `gh pr view` result are the coordinator's recorded output, not something I re-ran. I checked them against the facts below.
- **Refs:** the local and remote branch refs are `68fb6e9ca741...`, and `origin/main` is `762fa99f4183...`.
- **Reflog:** linear, with exactly one commit between a3d60dd and 68fb6e9.
- **End-state files:** the renamed tests at `tests/references.rs:207-244`, the binding and matrix row at `workplan-document-model.md:57` and `:439`, and the reworded sentences at `workplan-document-model.md:266-267` and `workplan-generated-docs.md:193-195`.

The recorded diffstat is consistent with that end state.
- Per-file counts: 8 + 8 + 5 = 21, which equals 11 insertions plus 10 deletions.
- `references.rs`: three renamed test functions and one reworded comment.
- `workplan-document-model.md`: one binding line, one matrix row and the reworded two-line sentence.
- `workplan-generated-docs.md`: the reworded sentence.

## Prior findings

1. **BLOCKING, overclaimed per-task reviews: DISCHARGED.** Notes lines 34-39 now read: "Tasks 1-8 each had an independent review with PASS, and a completion audit of the delivered result (Task 9 onward) ran 12 break-a-doc experiments ...; it blocked on three items, ... The audit was not re-run on the final head". This matches the stage evidence: T1-T8 review files exist, no T9 review exists, and the audit verdict is BLOCKED.
2. **BLOCKING, unrecorded fix-diff scope: DISCHARGED.** `validation.md:15-19` records `git diff --stat a3d60dd 68fb6e9`: 3 files, 11 insertions, 10 deletions, no change under `src/` or `examples/`. The experiments exercise only crate source and the `docs_sync` example, so reusing the audit's behavioural evidence holds. Hosted CI re-ran every test on the head, including the `docs_contract` Gate, which checks that bindings resolve and that blocks are in sync.
3. **ADVISORY, raf-okf additions listed incompletely: DISCHARGED.** Notes line 26 now adds "the `name` methods of both tags".
4. **ADVISORY, mixed units: DISCHARGED.** Notes line 16 reads "from 3,576 lines and 145 tests to about 1,350 lines and 26 tests". The candidate file has 1,353 lines and 26 `#[test]`s.
5. **ADVISORY, commit for the local counts unstated: DISCHARGED.** Notes line 31 reads "(local counts measured before the final doc fix)".
6. **ADVISORY, CI-to-head tie inferred: DISCHARGED.** `validation.md:18-19` records headRefOid 68fb6e9 and baseRefOid 762fa99, equal to origin/main. That makes the claim "the tested tree is the merged tree" hold for this candidate.

## Coverage of the other questions (unchanged from the first pass)

- **Threads:** both stage artifacts show 0 reviews, 0 threads and 0 comments, so nothing needs a disposition.
- **`.deficiencies`:** it holds only the notice and the marker.
- **Requirements B1-B16 (R1-R32):** all are covered. The command line moved to `cap:workplan-cli` (designed) and the obligations output to `cap:workplan-reconcile` (G16), and the notes state both moves.
- **`Closes #7`:** commit messages fall outside AGENTS.md section 7, which governs "any committed file". Precedent on main agrees.
- **Attribution:** the notes contain none.

## New findings (introduced by the revision)

### ADVISORY

7. **A missing full stop in the permanent commit message.**
   - Quoted, notes line 39: "is recorded in the validation notes There were no PR reviews, threads or".
   - REWRITE TO: "is recorded in the validation notes. There were no PR reviews, threads or".
   - Optionally, rewrap lines 16, 26 and 31, which are now longer than the 85-column wrap the rest of the message uses.

No blocking findings remain.

VERDICT: PASS
