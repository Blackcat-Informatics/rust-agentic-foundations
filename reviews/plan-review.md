# Plan final review (Stage 1)

Reviewed plan: `.stage/m2-rest-of-the-planning-essay-entity/plan.md` (Revision 3 with the later edits; first pass
against the text carrying option (a) for R4; recheck 1 against the D15 rewrite and the Task 1/2/5/6/9 "Also" lines;
recheck 2 against the F1-R and N1 fixes).

Reports and dispositions examined: `reviews/plan-completeness.md` (PASS; A3, A6, A7, A8 advisory),
`reviews/plan-constraints.md` (PASS; A15-A18 advisory), `reviews/plan-design.md` (PASS; A2, A3, A7, A8, A9),
`reviews/plan-enhancement.md` (PASS; items 1-23, N1, N2), `reviews/plan-adversary.md` (BLOCKED on N2 only, with
option (a) recommended). Inputs: `issue-analysis.md`, `prior-art-assessment.md`, the five capability documents,
the five change records, `SECURITY.md` (Subprocesses), `tests/docs_contract.rs`, `crates/raf-workplan/src/rules.rs`,
`crates/raf-workplan/src/obligation/tests.rs`, `crates/raf-workplan/src/lifecycle/create.rs`,
`crates/cargo-workplan/src/cli.rs`.

Owner decisions on the record, and where the plan applies them: coverage scope "Workplan crates only
(Recommended)" (lines 28-34, D9, R4); hosted CI as `cross-check` evidence (line 35, Task 12); "Run the command
(Recommended)" (D4, D12, R2); F1 record naming, option (a) "keep the repository policy" (D15).

## 1. Fixes are real

| Finding | Disposition | Plan text | Ruling |
| --- | --- | --- | --- |
| Adversary N2 (R4 cell) | option (a) applied verbatim | R4: "a `doc(hidden)` item is absent from `coverage code`; an unbound `cfg(test)` unit test appears in `coverage tests` and is absent once a `<module>::tests::*` exclusion with a reason lists it" | DISCHARGED: matches D9 and the edit-list row word for word |
| Adversary N1 / Completeness A5 (evidence runs) | adopted | D4 quotes the owner and defines the run, verdict, values, limits, `Program::Trusted`, `--from-binding`; R2 carries the fixtures | DISCHARGED |
| Adversary F2, F4, F6, F7 | adopted | R2 record contents and no `docs/` write; D9 crate-root prefix refused; D14/R8 none `dropped`; edit list rows | DISCHARGED |
| Completeness A6, A7, A8 | adopted | R4 exclusion cell; D14 "never hides a cut"; Task 11 step (1) | DISCHARGED |
| Constraints A15, A16, A17, A18 | adopted | lines 22-23; D12 (process list, no `--frozen`, environment, four parse failures exit 2, compile-fail doctest); D5 | DISCHARGED |
| Design A2, A3, A7, A8, A9 | adopted | D17; D9 `{item, reason}`, shared helper; D1 Coverage keys | DISCHARGED |
| Enhancement 1-23, N1, N2 | adopted or declined with reason | as the enhancement recheck table records | DISCHARGED |

## 2. Declines are justified

Each Declined bullet carries a technical reason and a cost. The one declined TRANSFORMATION item (E2) has its
paragraph: binding granularity is the Contract's, conditions are human-judged, statement-level support belongs
with the strict-mode issue.

## 3. Integration and coherence

The plan reads coherently end to end: twelve tasks in dependency order, each with an Acceptance line and the
shared per-task line (tests, review, commit, push, issue update), no Task 0, Task 12 the PR. The decisions, tasks
and R-rows agree on `cfg(test)`, `doc(hidden)`, exclusions, the evidence run, the trusted process, staleness, the
inventory policy of `tests/docs_contract.rs` and the pinned reconcile table of `obligation/tests.rs`. The commit
order is now executable against the repository's own tests: each delivered capability is flipped and its
capability-named record opened in the first task that changes it (Tasks 1, 2, 5, 6, 9), the reconcile table grows
with `ObligationKind::ALL` in Tasks 5 and 6, the Inventory `records` list is edited per task, and the five
capabilities are flipped in Task 11 step (1) with `INVENTORY` in the same commit.

## Findings raised by this lens and their final status

| Finding | Status | Evidence |
| --- | --- | --- |
| F1 BLOCKING: `tests/docs_contract.rs` refuses the records and documents the plan creates | DISCHARGED | D15 lines 143-155: the inventory rule stated, six capability-named records, `records` list per task; Task 1/2/5/6/9 "Also" lines; D1 line 48 "capability-named change record `okf-profile`"; D16 line 130-131 "their own `okf-profile` and `source-index` records"; Task 1 line 225-226 "open the `okf-profile` and `workplan-document-model` records"; Task 11 line 273 "the six D16 records are completed the same way". A grep for the old names finds nothing. |
| F2 BLOCKING: delivered documents edited in the task that grows the code | DISCHARGED | D15 lines 153-155; Task 5 and 6 "Also" lines; edit-list row line 180; Task 11 step (1) no longer flips the D16 capabilities. |
| N1 BLOCKING (integration): no task flipped the five capabilities | DISCHARGED | Task 11 line 270: "(1) the five capabilities are set `implementing` (`INVENTORY` in the same commit) and their bindings, added by Tasks 7-10 per D19, are confirmed complete". |
| A1 D4 "fifth stated form" | DISCHARGED | D4 line 67. |
| A2 R2 parse-failure fixtures | STILL-OPEN (advisory) | R2 unchanged; D12 lines 116-118 and line 20 bind Task 5 to the four negative fixtures and their passing neighbours. Suggested cell, if the orchestrator wants R2 self-contained: "a `--limit` name the command never printed, a value that is not a number, a name printed twice, and output past 16 MiB on either stream each exit 2 and write no record; every limited value printed and within its limit records `pass`". |
| A3 bindings while `considered` | DISCHARGED | D19 binds in Tasks 7-10; Task 11 step (1) confirms rather than adds; `rules.rs` has no rule against bindings on a `considered` capability. |
| A4, A5, A6 | DISCHARGED | D15 line 146; "Task coverage rows" paragraph; line 166. |
| A7 revision line | ADVISORY (cosmetic) | line 3 still reads "Revision 3"; suggested "Revision 4 (Revision 3 plus the final-review edits)". |
| A8 (new, cosmetic) D1 wording | ADVISORY | D1 line 48-49: "A capability-named change record `okf-profile` lists `cap:okf-profile`, `cap:workplan-document-model`" reads as one record for two capabilities; Task 1 opens two. A capability may be listed by several records, so nothing fails; suggested: "The capability-named records `okf-profile` and `workplan-document-model` list their capabilities and are opened in Task 1". |

No BLOCKING finding remains. The advisories are optional text tidy-ups and do not hold the verdict.

## Open user decisions

None.

Caller vocabulary: PASS.

VERDICT: PASS
