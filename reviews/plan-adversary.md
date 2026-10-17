# Plan completion adversary review

* Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/9-acceptance-scenarios-a-b-workplan-self/.stage/acceptance-scenarios-a-b-workplan-self/plan.md`
* Pass: 1 of at most 3
* Inputs read: plan.md, issue.md, raw/issue.json, issue-analysis.md, prior-art-assessment.md, the tree at HEAD (a9d6852). `.baseline`, `.goals`, `CONSTITUTION.md` and ADRs do not exist in this worktree. `.deficiencies` has nothing below its marker.

## Owner scope decision on record

The plan quotes the owner: "Leave the 6 work for later." The revised deliverable is the workplan part of the issue only. The self-census gate ("Self-workplan and self-census gates both pass in CI", the census half) stays open in issue 9 and issue 6. The PR says "Part of #9" and does not close the issue. The plan's row `| self-census gate, planted duplicate fails | blocked | needs issue 6; stays open in issue 9 (owner decision above) |` matches that decision. It is **not** a pre-emptive descope, and this review does not count it against the plan.

## Findings

### F1. UNMAPPED REQUIREMENT, insufficient task (R15 "a generated verification matrix"), and a false evidence claim

Plan, "Evidence found", line 8:

> * All 12 M1 capabilities `delivered` with resolved bindings and generated verification matrices; no workplan change record remains in `work/changes` (only the nine census records).

Plan, completeness contract, line 16:

> | every M1 change record completed with the tool; every M1 capability delivered with resolved bindings and a generated verification matrix | 1 | test asserts the 12 M1 capabilities are `delivered`, each holds a generated `verification-matrix` block, and no `work/changes` record names an M1 capability |

Evidence against it: only 5 of the plan's 12 capabilities have a `<!-- raf:generated:verification-matrix -->` block:

* `docs/capabilities/workplan-cli.md:588`
* `docs/capabilities/workplan-complete-gate.md:656`
* `docs/capabilities/workplan-document-model.md:714`
* `docs/capabilities/workplan-generated-docs.md:352`
* `docs/capabilities/workplan-reconcile.md:542`

These 7 have none: `okf-profile`, `source-index`, `workplan-current-rationale`, `workplan-entity-history`, `workplan-evidence-claims`, `workplan-prose-assessment`, `workplan-two-way-coverage`. A grep for `raf:generated` in those files finds no block. For example, the Verification section of `workplan-entity-history.md` (lines 285-351) is a plain bullet list. Sync does not add the blocks on its own. `workplan-generated-docs.md:175-176` says: "A file with no generated block is not touched". `Gate::workspace` does not require the block either, which is why the repository gate passes today without them.

Task 1 ("tests/self_workplan.rs and its wiring") only writes a test, moves one test and wires it in. No task adds the 7 missing blocks. The row-3 test as planned therefore fails on 7 documents, or someone narrows it quietly while implementing. Either way R15 points to a task that cannot deliver it.

The plan's M1 disposition also contradicts the issue. The plan says "'M1' is undefined in the issue". But `raw/issue.json` puts the issue in milestone 3, titled "M1 Workplan", described as "raf-workplan + cargo-workplan milestone 1: model, check, reconcile, complete gate, scenarios A/B". The plan's 12-capability set includes the shared-crate capabilities and the M2 essay capabilities. Choosing a larger set is not a descope, but the plan then has to deliver for every member of it.

REWRITE TO (choose one; the plan must not both claim 12 and test 12 while only 5 hold a block):

* **(a) Keep the 12-capability set.** Change line 8 to: "Five of the twelve (`workplan-cli`, `workplan-complete-gate`, `workplan-document-model`, `workplan-generated-docs`, `workplan-reconcile`) hold a generated verification matrix. Seven (`okf-profile`, `source-index`, `workplan-current-rationale`, `workplan-entity-history`, `workplan-evidence-claims`, `workplan-prose-assessment`, `workplan-two-way-coverage`) do not." Add a task: "Insert an empty `<!-- raf:generated:verification-matrix -->` / `<!-- raf:generated:end -->` block into the Verification section of each of the seven, run `make docs-sync`, and commit the rewritten tables." The row-3 acceptance becomes: "`make docs-sync` on the committed tree rewrites nothing. `cargo test --test self_workplan` asserts that each of the 12 named slugs has `state: delivered`, holds exactly one `verification-matrix` block in its Verification section, and gets no `binding-*` finding from `Gate::workspace(gate_root())`. Deleting the block from any one of the 12 in a fixture copy fails that assertion and names the slug."
* **(b) Use the forge milestone.** Change the M1 disposition to cite milestone "M1 Workplan" (`raw/issue.json`, `milestone.title` and `milestone.description`). List the exact capability set that milestone's issues delivered, with the evidence for each slug, and use that set for the row-3 assertions above. The line-8 evidence claim becomes true only for that named set. Any capability dropped from the plan's current 12 must be justified from the milestone record, not from convenience.

### F2. UNFALSIFIABLE CRITERION (R12 "with `cargo workplan status` and `change complete` for every M1 change record", R17 "completed **with the tool itself**")

Plan, completeness contract, line 16 (acceptance cell):

> no `work/changes` record names an M1 capability

Plan, "Evidence found", line 8:

> no workplan change record remains in `work/changes` (only the nine census records)

The issue does not ask that the records be gone. It asks that they were completed with the tool (bold in the issue). An absence check passes the same way whether each record was closed by `cargo workplan change complete <slug>` or deleted with `git rm` and its state edited by hand. The plan never runs `cargo workplan status` or `change complete` on this repository, and never cites a commit in which the tool closed an M1 record. The tree has no such record: `work/evidence/` holds only `gate-kills-a-test-past-its-timeout.md`, and nothing under `work/` mentions `change complete`.

REWRITE TO: in Task 2, before the PR opens, add to validation.md:

1. The verbatim output of `cargo run --locked -q -p cargo-workplan -- --root . status`, run on this checkout through the shipped CLI. It must list every M1 slug (the set fixed by F1) under Delivered with no open change.
2. For each change record that ever named an M1 capability, the commit that removed it: `git log --diff-filter=D --format='%h %s' -- 'work/changes/<slug>.md'`. Show that the same commit carries the rewrite that `change complete` makes: the record removed, the capability's `state` set, and the generated blocks synced. One line per record: `<slug> <commit> completed-by-tool: yes/no`.

Any record whose removal was not a `change complete` run is not counted as done. For each one, the plan either adds a task that reopens the capability with `cargo workplan change new <slug>` and closes it with `cargo workplan change complete <slug>`, or puts it to the owner as a named scope decision. Replace the row-3 acceptance cell with these two observations, plus the F1 assertions.

### F3. UNFALSIFIABLE CRITERION, minor (R14 "Self-workplan ... gates ... pass in CI")

Plan, completeness contract, line 18:

> | self-workplan gate passes in CI | 1 | CI `test` job runs the new test binary |

Plan, Task 2, line 25:

> Record scenario evidence in validation.md (run `cargo test -p raf-workplan --test scenarios`), open the PR ("Part of #9"), post plan and confidence note.

The issue says "pass". Running is not passing, and no task observes the CI result.

REWRITE TO: acceptance: "On the PR head commit, the CI `test` job (`cargo test --workspace --locked`, `.github/workflows/ci.yaml`) concludes `success`. Its log shows `Running tests/self_workplan.rs` and every test in it `ok`." Add to Task 2: "After the PR opens, wait for the CI `test` job on the head commit and record its run ID and conclusion in validation.md. Hand off only once it has concluded `success`."

## Checked and cleared (no finding)

* **Scenarios A and B (R1-R10, R13).** `crates/raf-workplan/tests/scenarios.rs` exists and is bound in `workplan-complete-gate.md:33-36,147` and `workplan-reconcile.md:99-100`. It covers each sub-requirement:
  * four redesigns by successive changes, `Amended 2026-` records, a reversed decision: lines 8-12, 504, 734, 787
  * PROPOSED blocks and change files off the reading path: 617-629
  * links: 587
  * rationale: 648-663
  * `git show` recovery: 668-669
  * a `considered` start with open questions: 849-859, 1377-1379
  * a refusal until reconciled: 1196
  * the delivered account with example, references, limitations and traceability table: 1315-1345
  * negative controls: 1444, 1528, 1747

  "Declined: Rebuilding scenarios A and B" refuses only duplicate work. No requested work is cut.
* **R11 and R16 wiring.** Row 1 (Gate over `gate_root()` plus a fixture with a planted broken link that fails) and row 2 (the hook line plus `make test`) name real entry points. `make check` runs `make test`, which is `cargo test --workspace --locked` and includes the root package's `tests/`. `.githooks/pre-commit:38` is the line to extend.
* **The self-census row (R14, census half), R18 and R19.** Covered by the owner decision quoted above. The issue stays open and the PR does not claim the issue is closed.
* **"Resolved bindings" (R15).** `Gate::workspace` runs `Workplan::check`, which reports `binding-unresolved`, `binding-ambiguous` and `binding-wrong-role` (`crates/raf-workplan/src/finding.rs:65-68`). A clean gate therefore implies resolved bindings. F1's rewrite makes the per-slug assertion explicit.

## Required user decision

None on this pass. F1 option (b) and F2's hand-removal branch may need an owner decision, depending on what the evidence shows.

VERDICT: FINDINGS-OPEN
