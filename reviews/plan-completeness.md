# Plan completeness review (second recheck)

Reviewed plan: `.stage/m2-rest-of-the-planning-essay-entity/plan.md`, Revision 3. This recheck rules on
every open finding from the Revision 2 recheck and reviews the new text only (D4, D5, D9, D14, D16, D17,
and the R2, R3, R4, R6 and R8 rows).

Inputs as before: issue.md (0 comments), issue-analysis.md (R1..R13, NEEDS-JUDGMENT 1..5),
prior-art-assessment.md, the five capability documents and records, the delivered `workplan-reconcile`,
`workplan-complete-gate`, `workplan-cli`, `workplan-document-model` and `okf-profile` documents, and
`docs/architecture/{okf-profile,normal-form}.md`.

## Requirement -> task -> check

| R-id | Requirement (source) | Task(s) | Check (Revision 3) | Status |
| --- | --- | --- | --- | --- |
| R1 | entity history, `--follow`, `-L` (body L12) | 4, 9, 10 | fixtures incl. `cap:`/`change:`/`arch:` ids, removing commit and parent, stub argv, byte identity; real use `history cap:workplan-reconcile`, `history arch:normal-form` | MAPPED |
| R2 | persisted evidence, dependency, invalidation (body L13) | 5, 9, 10 | views; over-limit replacement; removal; stale symbol; "the written record holds command, revision (when given), verdict, values, limits and input digests, and recording changes no file under `docs/`"; unknown id; misspelled marker; over-limit `pass` refused; no process; real use | MAPPED |
| R3 | assessments invalidated when inputs change (body L14) | 6, 9, 10 | each input named; `--context` "identically for human and agent"; `assessment-*` lines in `change reconcile`; condition 6; rejected never satisfies; real use | MAPPED |
| R4 | code -> docs, tests -> claims (body L15) | 2, 7, 9, 10 | public/private/re-export/descendant/test/hidden fixtures; exclusion faults; scoped `check`; whole workspace with no record; reports name raf-okf/raf-source gaps; existing fixtures green; real use `check` clean | MAPPED (A6 on a stale check term) |
| R5 | architecture pages, conditions to revisit (body L16) | 8, 10 | fixtures incl. `### Decision`; real use on both architecture pages | MAPPED |
| R6 | each `delivered` via `change complete` (AC L19) | 11 | reconcile empty, `completed: removed ...` (exit 0), `state: delivered` at HEAD, and the D16 capabilities delivered with tables regenerated and text stating the new contract | MAPPED |
| R7 | record deleted, history in Git (AC L20) | 11 | absent at HEAD; `git log --diff-filter=D -- work/changes/<slug>.md` | MAPPED |
| R8 | no deferrals; leaving scope is `moved:` (AC L21) | 11 | "every item `implemented` or `moved:change:<slug>` to an existing tracked record; none `dropped`" | MAPPED (A7 on rewording) |
| R9 | lifecycle with the tool (body L9) | 11 | reconcile then complete, commands recorded | MAPPED |
| R10, R11, R13 | one item per capability; beyond M1; premise (L9, L11) | all | per-capability tasks; premise true | MAPPED |
| R12 | depends on #9 (L24) | - | nominal, recorded | MAPPED (NEEDS-JUDGMENT 1 disposed) |

All five NEEDS-JUDGMENT items have a disposition. Structure is unchanged and sound: `## Task 1` ..
`## Task 12`, each with an Acceptance line and the shared per-task line (tests, review, commit, push,
issue update); no Task 0; Task 12 is the PR, publication and handoff.

## Prior findings

| Finding | Ruling | Evidence |
| --- | --- | --- |
| N1 delivered capabilities with no record | DISCHARGED | D16 widens the five records to list `cap:workplan-reconcile`, `cap:workplan-complete-gate`, `cap:workplan-cli` and `cap:workplan-document-model`, and gives `cap:okf-profile` and `cap:source-index` their own records. Each capability is `implementing` while a listing record is open. The R6 row checks their states, tables and text at HEAD. |
| F1 revision option and record contents | DISCHARGED | D4 adds `[--revision <commit-id>]`; D5 validates it against the commit-id pattern. The R2 row checks the fields of the written record and that nothing under `docs/` changes. |
| F3 `dropped` allowed | DISCHARGED | D14 "nothing is `dropped`"; R8 "none `dropped`". |
| A1 identical context | DISCHARGED | D17 "`assess --context` output is identical for `human` and `agent` (tested)"; the R3 row checks it. |
| A3 provenance of the scope decision | STILL-OPEN (advisory) | The owner's quote still appears only in plan.md. |
| A5 command not run by the tool | DISCHARGED | The edit list records it, and Task 12 puts D4 among the owner readings in the PR. |

## Findings

No blocking findings.

### A3 ADVISORY (carried): provenance of the coverage-scope decision

"Workplan crates only (Recommended)" is quoted only in plan.md. Name the message or comment that holds the
owner's answer, so the PR can cite it.

### A6 ADVISORY (new): the R4 row checks a digest that D9 no longer has

D9 now defines exclusions as "`{item, reason}` entries" and its findings as "an exclusion matching nothing
or with an empty reason". The R4 row still says "exclusions with empty reason, unmatched item and changed
digest fail naming the file". That check would test behaviour the plan no longer builds.

REWRITE TO (R4 row): "exclusions with an empty reason and an exclusion that matches no item each fail
naming the file".

### A7 ADVISORY (new): rewording must not hide a cut

D14 says: "A claim found untrue as written is reworded to what was built (with the reason in the PR body) or
moved". If the reworded claim is narrower than the original, the part that left must also be
`moved:change:<slug>`. REWRITE TO (D14, append): "A rewording that narrows a claim moves the part it
removes to a tracked record."

### A8 ADVISORY (new): Task 11 step (1) names only the five capabilities

D16 puts the delivered capabilities back in `implementing` while a listing record is open. Task 11's step
(1) reads "the five capabilities `implementing` with bindings". REWRITE TO: "(1) the five capabilities and
the D16 delivered capabilities `implementing`, with bindings".

VERDICT: PASS
