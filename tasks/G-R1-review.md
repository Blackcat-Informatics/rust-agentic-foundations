# G-R1 review: remediation group R1

Scope: `git diff` of the 12 unstaged files (architecture/okf-profile; capabilities okf-profile, workplan-complete-gate, -document-model, -prose-assessment, -reconcile, -two-way-coverage; five work/changes/workplan-*.md), against remediation-plan R1, gaps G1, G4, G5, G6, G8, G12 and audit item A4, and the essay at origin/main (lines 78-84, 211, 236-242). Read-only; nothing edited, staged or committed.

## Evidence

| Check | Result |
| --- | --- |
| `git grep -iE 'callers can rely\|prerequisites\|substantiate' -- docs/architecture/okf-profile.md` | 3 lines (79, 82, 84) |
| `git grep -n GROUNDING.md -- docs` | 2 lines (document-model:214, two-way-coverage:94); URL byte-identical to essay :211 (commit a631f73e...) |
| Scenario B "accurate capability explanation" grep | 0 lines for the full phrase, because it wraps ("accurate capability" / "explanation"); `grep 'accurate capability'` gives 1 line. See A1 |
| `grep -n example-failing workplan-reconcile.md` | 4 lines (kinds table, Examples, Constraints, Verification) |
| PROPOSED-sections vs `affects` (fence-aware, my own script) | equal for all 5 considered records (current-rationale, entity-history, evidence-claims, prose-assessment, two-way-coverage) |
| `yq .depends workplan-prose-assessment.md` | includes `cap:workplan-evidence-claims` |
| `git grep 'raf_source::Tree' -- docs/capabilities/okf-profile.md` | empty |
| `grep 'one code path'` on the 3 files | 3 lines, identical phrase "binds at least one code path and one test" |
| `cargo test --test docs_contract --locked` | 46 passed, 0 failed (graph acyclic) |
| `make issue-refs` | rc 0, no output |
| `uvx reuse@6.2.0 lint` | compliant, 77/77 |

## Item verification

- G1: the seven definitions match essay :78-84 word for word, in current tense, in OKF order. The document-model Contract links to `#body-rules`, which exists. The Grounding URL is exact. Scenario B now lists the explanation. Delivered.
- G4: `example-failing` appears consistently in the reconcile kinds table, complete-gate condition 5, Examples, Constraints, Verification, and the document-model and two-way-coverage claims. The word "criterion" is gone from those docs. Delivered, except R-1 and R-2 below.
- G5: the depends edge is added and the graph is acyclic. The `Tree` mention is removed. The `depends` sentence is present. Delivered.
- G6: all 5 records carry `#constraints`. Delivered.
- G8: the qualification is accurate: the test checks keys, values, sections, markers and links, and bindings shape only (tests/docs_contract.rs:605-623). The empty-bindings enforcement is not yet mentioned, which is correct before R7 (see A2).
- G12: the profile table row, the complete-gate State paragraph and document-model Contract carry the same phrase. The "none inside one" sentence is fixed and not self-contradictory. Delivered.
- A4: the memory-bound sentence is replaced by text that is accurate for the shown `binding-missing` line. Delivered.

## Findings

Required:

- R-1. `work/changes/workplan-complete-gate.md` item 1 claim still reads "...a clean docs sync and passing bound tests." The designed contract now requires bound tests and examples. The claim is what the gate will check, so it contradicts condition 5. Change to "passing bound tests and examples" (a second claim, "verify runs bound tests and examples", is already right).
- R-2. `docs/capabilities/workplan-evidence-claims.md:92` (Explanation) still says "tests address the criteria". R1 removed the same sentence's "criterion" from workplan-two-way-coverage (now "tests address claims") and the plan says no stray "criterion" should remain in the workplan docs. Align it to "claims". `git grep -i criteri` otherwise hits only census docs (a different, ledger field) and AGENTS.md:182 (unrelated).

Advisory:

- A1. The Scenario B phrase "an accurate capability explanation" wraps across lines, so a literal one-line grep returns nothing. Reflow so the phrase sits on one line if the plan's Check is to be run literally.
- A2. `docs/architecture/okf-profile.md` "enforces the profile today" paragraph is silent on the empty-bindings-before-delivery rule. R7 must add it when that rule lands; do not forget it.
- A3. AGENTS.md:172 lists the completion conditions as "the bound tests pass". Not contradictory, but it omits examples; consider "bound tests and examples".
- A4. An `example-failing` obligation is both an outstanding obligation (condition 1) and a condition-5 failure, so one fault is refused twice. The complete-gate example shows one summary line and is consistent enough; a one-clause note on the overlap would remove the ambiguity.

## Verdict

VERDICT: BLOCKED
Required work: R-1 and R-2 (small text fixes). Re-run the same checks after the edit.

## Recheck

- R-1: `work/changes/workplan-complete-gate.md:16` ends "passing bound tests and examples". Closed.
- R-2: `workplan-evidence-claims.md:92` reads "tests address the claims"; `git grep -i criteri` over workplan capability docs and records is empty. Closed.
- A3: AGENTS.md:172 reads "the bound tests and examples pass". Closed.
- A4: the new paragraph after condition 5 explains that conditions 1 and 5 both report a failing example. Accurate. (Minor: the Examples refusal shows a single line rather than one per condition; not blocking.)
- A1: accepted; the phrase wraps and `grep 'accurate capability'` finds it.
- A2: carried to R7; not an R1 defect.
- Re-run: docs_contract 46 passed; `make issue-refs` rc 0; reuse lint compliant; no staged or untracked changes.

VERDICT: PASS
