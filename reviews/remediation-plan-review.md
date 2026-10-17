# Remediation plan review (Stage 2): recheck of Revision 2

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/10-m2-rest-of-the-planning-essay-entity/.stage/m2-rest-of-the-planning-essay-entity/remediation-plan.md`, section "Revision 2" (lines 31-43). Revision 2 says it replaces the rows and steps it names.

The first pass ended VERDICT: BLOCKED, with seven BLOCKING findings (1-7) and eight ADVISORY findings (A1-A8). Each had a REWRITE TO line. This recheck marks each one DISCHARGED or STILL-OPEN, then reviews only the text Revision 2 adds. I checked against the same evidence as the first pass:

* `.githooks/pre-commit`
* `tests/docs_contract.rs`: `INVENTORY`, `check_inventory`, and the shape pins at 1037-1056
* `crates/raf-workplan/tests/roadmap.rs`
* `docs/capabilities/workplan-document-model.md:113`, which binds the guard
* `workplan-complete-gate.md` Contract 241-418 and Examples 447
* `docs/architecture/okf-profile.md:268-272`
* `exec.rs` (`ExecError`)
* `bound/run.rs:120-134`
* `sync.rs:338`
* `lifecycle/complete.rs:397`

## Recheck of the blocking findings

| # | First-pass finding | Status | Evidence in Revision 2 |
| --- | --- | --- | --- |
| 1 | The roadmap guard was handled two contradictory ways, and the plan assumed the pre-commit hook runs it | DISCHARGED | Line 35. C1 is one reopening commit. It holds the records, the states, the `INVENTORY` states, the shape pins (implementing \|R\|, delivered 12-\|R\|), `make docs-sync`, and the guard rewording ("delivered, or implementing with its capability-named open change; no gap"). The rewording counts as part of the document-model reopening, and its Verification sentence is updated. After every commit, both test commands are run. The plan now states that the hook runs only `docs_contract` and `repo_gates`. Checked against the code, the sequence holds at every commit. R now has 10 capabilities, which leaves current-rationale and reconcile delivered. The guard's `assert_ne!(delivered group, 0)` therefore holds. `check_inventory` finds a capability-named record for each implementing capability. Under the reworded guard, completion order is free, including `change complete workplan-document-model` (condition 5 runs the guard). |
| 2 | G1 fixed the count but left the false sentence, and left the choice open | DISCHARGED | Line 36. Sections are counted as distinct (capability, section) subjects, in the text and in the JSON headline. Contract 273-277 is rewritten to "one or more obligations, one `assessment-stale` per moved input". The "or" is dropped. Affects lists #contract, #examples and #verification. The fixture and the mutant are as stated in the first pass. |
| 3 | The G2 rule contradicted the `raf:generated:` text rule and the letter-case and spacing rule | DISCHARGED | Line 37. The trigger is the first non-blank text after `<!--`, in any letter case. `raf:generated:` is exempt. Each finding kind is kept. Multi-line `raf:generated:` comments, the SPDX header and ordinary comments stay plain. The architecture page gets a new sentence. |
| 4 | The PR-body fix left out the record-item list and stale statements, and its acceptance could not fail | DISCHARGED | Line 38. Every listed item is present. The body is written after the final push. Acceptance is a `gh pr view --json body` check: no stale phrases, no `.stage`, one row per rewording, and the head SHA equal to `git rev-parse HEAD`. |
| 5 | C1 listed done items, missed undone ones, and some findings had no disposition | DISCHARGED | Line 39. C1 now has only items that are not done yet. The `--list` exit-2 sentence and the ATX-only sentence are dropped, and current-rationale is no longer reopened. The `Text file busy` lock and the id-rule item are added. Line 40 lists what is consciously accepted. Small remainder in N2. |
| 6 | Cascading stale records were handled incompletely | DISCHARGED | Line 41. Evidence is recorded first, then assessments, then a text grep over the changed behaviours, then `assess --list` and the 12-capability probe over the seven kinds. If a re-read finds a false statement, the capability is reopened. The loop repeats until clean. New records go into `INVENTORY.records` and the shape pins. Any later code change repeats step 2. Line 35 adds that each commit binds its new tests and items. |
| 7 | The final gates left out fuzz and test-release | DISCHARGED | Line 42. `make check` runs every target except cross-check (an owner decision), each named with its exit code. Fuzz and test-release (as the target) are included, plus docs-sync twice and hosted CI on the final head. |

## Recheck of the advisories

| # | First-pass advisory | Status | Evidence |
| --- | --- | --- | --- |
| A1 | G3: refuse only on an unloaded capability | DISCHARGED | Line 43: `unloaded()` is filtered to `cap:`. Unloaded change and evidence records stay at exit 0. The Contracts state the exit. |
| A2 | A3: move the escape helper out of `history` | DISCHARGED | Line 43. |
| A3 | A6: quote with `TrustedArgv::text()` | DISCHARGED | Line 43. |
| A4 | G5: build `elapsed` instead of narrowing | DISCHARGED | Line 43: `elapsed` is added to `OutputTooLarge`, and the clause "with the time it ran" holds for both kinds of kill. See N3. |
| A5 | S1: the sibling write sites | DISCHARGED | Line 43: `sync::judge` and the removal in `complete.rs` get the same check, or an explanation of why they cannot be reached, with fixtures. See N4. |
| A6 | Source-index "(test only)" was ambiguous | DISCHARGED | Lines 39 and 43: source-index is reopened, affects #verification. |
| A7 | PROPOSED blocks and hook bypass | STILL-OPEN, in part (ADVISORY) | Hook bypass is forbidden in the probe clones (line 43). Revision 2 still does not say whether the reopened text goes in `> [!PROPOSED change:<slug>]` blocks. AGENTS.md section 6 says "Proposed text is written into the capability document itself inside a `> [!PROPOSED change:<slug>]` block". See N1. |
| A8 | List each record's `affects` | DISCHARGED | Line 43 gives the list for all 10 records. See N4 and N5 for two conditional additions. |

## Findings on the new text (all ADVISORY)

**N1. The PROPOSED-block workflow is still unstated.** This is the rest of A7. The gap analysis already flagged the earlier reopening for this deviation.

REWRITE TO (add to line 43): "the reopened text is first written inside `> [!PROPOSED change:<slug>]` blocks and resolved before each `change complete`; where code and text land in one commit, that is stated in the PR body as the reason for writing the text directly."

**N2. The consciously-accepted list leaves out two gap-A5 items.** Line 40 reads "A5 maintainability items (module sizes, split JSON rendering, one inventory edit per record)". Gap A5 also names two more: "type-binding digest vs coverage asymmetry" and "no gate keeping records current". The first one is why Revision 2 adds the step-2 text grep.

REWRITE TO: "A5 maintainability items (module sizes, split JSON rendering, one inventory edit per record, the type-binding digest that does not cover method bodies, no gate that keeps records current between changes)".

**N3. G5 changes a public variant.** `ExecError` is `#[non_exhaustive]` (exec.rs:311-312), but the field set of `OutputTooLarge` is not. A new `elapsed` field breaks a downstream struct pattern written without `..`. The repository already records exhaustive-match growth in the CHANGELOG.

REWRITE TO (append to the G5 advisory): "CHANGELOG states the new `elapsed` field of `ExecError::OutputTooLarge`; the `OutputTooLarge` message gains the time, and complete-gate Contract 403-410 and SECURITY.md say 'with the time it ran' for both kills."

**N4. The affects lists assume the conditional branches will not change behaviour.** Two cases:

* workplan-generated-docs is listed as "#verification, test only". If S1's sync branch changes `sync::judge`, generated-docs states a new refusal, and its #contract and #constraints are touched.
* Line 40 prefers a G4 guard on coverage enforcement. If it is built, two-way-coverage changes behaviour beyond "#constraints".

REWRITE TO: "if S1 changes `sync::judge`, generated-docs affects also lists #contract and #constraints; if the G4 guard is built, two-way-coverage affects also lists #contract and #verification, with a fixture (an unloaded capability gives its load error and no `coverage-item-unbound` cascade) beside a clean neighbour."

**N5. Step 2 can find a false statement outside `affects`, and only "reopen" is named.** For a capability that is already reopened, the correct step is to add the section to its record's `affects` and edit it.

REWRITE TO (step 2 (iv)): "... otherwise reopen its capability, or, when it is already reopened, add the section to its record's `affects` and correct it".

**N6. The rows Revision 2 supersedes are still in the table.** Table rows G3, G5, C1, P1 and P2, Order step 1, and the header sentence on line 5 still carry the old text, for example "keep as is ... no code change" and the G5 narrowing. Revision 2 says it replaces them, but an executor reading the table first could follow the old text.

REWRITE TO: mark each superseded row "(superseded by Revision 2)", or delete it.

## Result

All seven blocking findings are discharged. A1 to A6 and A8 are discharged. A7 is open only for its PROPOSED-block half. N1 to N6 are ADVISORY: none stops a fix from closing its gap on the real entry point, and none makes the commit sequence fail.

VERDICT: PASS
