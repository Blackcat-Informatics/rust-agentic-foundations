# Plan completeness review

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/9-acceptance-scenarios-a-b-workplan-self/.stage/acceptance-scenarios-a-b-workplan-self/plan.md`
Inputs: `issue.md` (0 comments), `issue-analysis.md` (R1-R19, 3 NEEDS-JUDGMENT).
Owner decision on record: "Leave the 6 work for later." The self-census half of R14 is blocked on issue 6, and the PR says "Part of #9".

## Requirement -> task -> check

| R-id | Requirement (short) | Contract row / task | Check | Status |
| --- | --- | --- | --- | --- |
| R1 | Scenario A fixture: four redesigns, amended and reversed decisions | "scenarios A and B" / Task 2 (exists) | `scenarios.rs::several_redesigns_leave_one_readable_present` (four `redesign` calls, an "Amended 2026-" record, a comment at L787 quoting the amended or reversed decisions) | MAPPED (implicit, no R-id) |
| R2-R7 | Scenario A end state: contract stated directly, no PROPOSED blocks or change files, links resolve, rationale kept, `git show` recovery | same | `assert_contract_stated_directly`, `assert_reading_path_is_clear`, `assert_links_resolve`, `assert_rationale_kept`, `assert_earlier_states_recoverable` | MAPPED (implicit) |
| R8 | Scenario B: starts `considered` with open questions, API renamed, one guarantee dropped | same | `planning_produces_the_documentation` (L1377-1389) | MAPPED (implicit) |
| R9 | `complete` refuses until Usage, Examples and bindings match and the dropped claim has an outcome | same | `refuse_until_reconciled`, `each_fault_of_the_account_...` | MAPPED (implicit) |
| R10 | It then succeeds: `delivered`, a real example, API references, limitations, traceability table | same | `assert_delivered_account` | MAPPED (implicit) |
| R11 | `tests/self_workplan.rs` runs `Gate` on this repo; wired into `make check` and the pre-commit hook | rows 1-2 / Task 1 | `cargo test --test self_workplan`, a planted fault fails, hook line `--test self_workplan`. `make test` is `cargo test --workspace` and the root package is a member, so it picks the binary up automatically | MAPPED |
| R12 | Advance states with `cargo workplan status` and `change complete` for every M1 change record | row 3 / Task 1 | asserts that no `work/changes` record names an M1 capability (only the nine census records remain on disk) | MAPPED, weak (see A2) |
| R13 | Both scenarios pass as integration tests | row 4 / Task 2 | `cargo test -p raf-workplan --test scenarios`; bound in `workplan-complete-gate` L33-36 | MAPPED |
| R14 | Self-workplan and self-census gates both pass in CI | row 5 / Task 1; census row "blocked" | CI `test` job runs `cargo test --workspace --locked` | Workplan half MAPPED; census half held under the quoted owner decision (accepted) |
| R15 | Every M1 capability `delivered`, with resolved bindings and a generated verification matrix | row 3 / Task 1 | asserts that each of the 12 holds a generated `verification-matrix` block | **UNMAPPED: the work is not planned and the plan's evidence is wrong (B1)** |
| R16 | Workplan gate turned on for this repo | row 1 / Task 1 | as R11 | MAPPED |
| R17 | M1 change records completed with the tool itself | row 3 | as R12 | MAPPED, weak (A2) |
| R18 | Depends on #6 | NEEDS-JUDGMENT 3 + census row | owner decision quoted | DISPOSED |
| R19 | Depends on #8 | NEEDS-JUDGMENT 3 | merged | DISPOSED |
| NJ1 | What "M1" means | disposition: the 12 that `roadmap.rs` pins | - | DISPOSED |
| NJ2 | Wiring vs the scenarios passing | disposition: the scenarios already pass | - | DISPOSED |
| NJ3 | #6 and #8 as hard dependencies | disposition given | - | DISPOSED |

## Findings

### B1. BLOCKING. UNMAPPED / SILENT CUT: 7 of the 12 M1 capabilities have no generated verification matrix (R15)

The plan says (Evidence found):
> "All 12 M1 capabilities `delivered` with resolved bindings and generated verification matrices"

That is false for the current tree. `<!-- raf:generated:verification-matrix -->` appears in only five capability documents: `workplan-document-model`, `workplan-reconcile`, `workplan-generated-docs`, `workplan-cli` and `workplan-complete-gate`. These seven are `state: delivered` but have no matrix block:

* `okf-profile`
* `source-index`
* `workplan-entity-history`
* `workplan-evidence-claims`
* `workplan-prose-assessment`
* `workplan-two-way-coverage`
* `workplan-current-rationale`

Task 1's own assertion ("each holds a generated `verification-matrix` block") would fail on this tree, and no task adds the blocks. As written, the plan cannot deliver R15. Its only options would be a test that fails or a test weakened to pass.

REWRITE TO (add a task before the current Task 1, then renumber):

> ## Task 1: generated verification matrices for every M1 capability
> In each of `docs/capabilities/{okf-profile,source-index,workplan-entity-history,workplan-evidence-claims,workplan-prose-assessment,workplan-two-way-coverage,workplan-current-rationale}.md`, place a `<!-- raf:generated:verification-matrix -->` / `<!-- raf:generated:end -->` pair in the Verification section. Fill the pairs by running `make docs-sync`, which is `cargo workplan docs sync` over the repo. Do not write the tables by hand. Each document's `bindings.tests`/`bindings.examples` must already resolve, as `repository_bindings_resolve_in_the_source_index` checks. Focused check: running `make docs-sync` a second time produces no diff; `cargo test --test docs_contract` passes; `grep -c 'raf:generated:verification-matrix' docs/capabilities/*.md` is nonzero for all 12 delivered capabilities. Independent task review, commit, push, and an issue comment listing the seven documents.

Also correct the Evidence line:
REWRITE TO: "All 12 M1 capabilities are `delivered` with resolved bindings. Five hold a generated verification matrix (document-model, reconcile, generated-docs, cli, complete-gate). The other seven get one in Task 1."

### B2. BLOCKING. STRUCTURE: Task 1 has no review, commit, push or issue update

> "## Task 1: tests/self_workplan.rs and its wiring ... Focused check: `cargo test --test self_workplan --test docs_contract`, and the hook run by committing."

The task has no independent task review, no push and no issue update.
REWRITE TO (append to the task): "Then an independent task review, commit (the pre-commit hook must run `--test self_workplan` on the snapshot and pass), push, and an issue comment saying the self-workplan gate is wired into the hook, `make test` and CI."

## Advisory

### A1. ADVISORY. The contract does not cite R-ids

R1-R10 are folded into one "scenarios A and B" row, and no row cites an R-id. The coverage exists: every assertion was confirmed in `crates/raf-workplan/tests/scenarios.rs`. Add an R-id column so the mapping can be checked.
REWRITE TO: prefix each contract row with its R-ids: "R1-R10, R13" for the scenarios row, "R11, R16" for the self_workplan row, "R12, R15, R17" for the M1 row, "R14 (workplan half)" for the CI row, and "R14 (census half), R18" for the blocked row.

### A2. ADVISORY. R12/R17 checks state but not the named tool

R12 names `cargo workplan status` and `change complete` as the entry points. The check, "no `work/changes` record names an M1 capability", shows the end state but not that the tool produced it.
REWRITE TO (Task 2, after the scenario evidence): "Run `cargo run --locked -q -p cargo-workplan -- --root . status` and record its output in validation.md. It must list the 12 M1 capabilities as `delivered` and show no open M1 change. Also cite the commits in which `change complete` deleted each M1 change record."

### A3. ADVISORY. Name the remaining criterion in the handoff

Task 2 should say exactly what remains open, so the "Part of #9" scope is explicit.
REWRITE TO (Task 2): "...open the PR ('Part of #9'). Post an issue comment that states which criterion remains: 'Self-census gate passes in CI' and the self-census planted-duplicate check, waiting on issue 6 per the owner decision 'Leave the 6 work for later.'"

VERDICT: NEEDS-REVISION
