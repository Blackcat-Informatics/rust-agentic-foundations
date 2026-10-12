# T8 review

## Scope
Five M2 capability docs (entity-history, evidence-claims, prose-assessment, two-way-coverage, current-rationale) and their five change records, read in full; checked against plan Task 8, R29, R31, okf-profile "State and body", the essay blob at 720262d, and the approved design.

## Evidence
- `cargo test --test docs_contract --locked`: 46 passed, rc 0
- R29 loop: no output, rc 0
- depends acyclic and targets exist (tsort over all 21 docs): rc 0
- `ls docs/capabilities/*.md | wc -l` = 21; `ls work/changes/*.md | wc -l` = 21
- `make issue-refs`: rc 0
- `git add -N docs work; uvx reuse@6.2.0 lint`: 77/77 compliant
- `git grep -n INITIAL_PLANNING_PLAN`: empty (grep rc 1); `test ! -e INITIAL_PLANNING_PLAN.md`: rc 0
- R31 key phrases (git grep -il in docs): Plan changes to the documented system 1, amendment 2, permanent 2, roadmap 3, requires reconciliation 2, obligation 9, contract_effect 4, current rationale 3, historical deliberation 2, Grounding 2, Scenario A 2, Scenario B 2 (all >= 1)
- Deferral/process-word grep (deferr, CHANGE-n, later, future, for now, follow-up, TODO, conversational voice) over the 10 files: no hits except "milestone 1" (advisory below)

## R31 substance (section 5, section 6, section 4 prose)
- Section 5: the current-rationale versus historical-deliberation table is reproduced in current-rationale; the shared architecture page, conditions to revisit, and the parser rationale sentence are covered. Entity history is covered through git log --follow and -L. Covered.
- Section 6: all six directions appear in the two-way-coverage table. Evidence to claims is in evidence-claims. The "planned stays planned" point, the one complete current account with gaps named, the GMEOW Grounding Contract, and documentation maintenance as identifiable work are all in two-way-coverage Explanation. Covered.
- Section 4 prose: the generated versus authored split, the precise obligation list (the two-place API difference and unsupported memory bound example), and humans and agents sharing context are in prose-assessment. Covered.
- Rule-breaking text is rewritten: no CHANGE-n, no "deferred", no conversational voice.

## State-to-body
All five are `considered` with candidate Contract, Usage, Examples, Constraints and Verification inside `[!PROPOSED change:<slug>]` blocks. Purpose and open questions are plain. Change records are valid and `affects` lists only PROPOSED sections. No deferral words. Each considered capability is tracked by its change record.

## Findings
REQUIRED: none.

ADVISORY:
1. workplan-two-way-coverage.md lines 40 and 42 say "(milestone 1)" inside the table. This is project-process wording in a contract table. Prefer "(source-index bindings check)" or similar. The test passes, so it is not a rule violation.
2. Purpose and Explanation in several considered docs use present tense about proposed behaviour (for example "This capability records executions", "the affected section receives an explicit obligation" in two-way-coverage Explanation). They read as design rationale, not as a claim of availability, so this is acceptable. "would" or "will" in the two-way-coverage Explanation sentence would remove any ambiguity.

VERDICT: PASS
