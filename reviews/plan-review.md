# Final plan review (Stage 1)

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/9-acceptance-scenarios-a-b-workplan-self/.stage/acceptance-scenarios-a-b-workplan-self/plan.md`. First pass judged "Revision 1" (plan lines 36-59). Recheck judged "Revision 2" (plan lines 61-90), which supersedes the original contract table and Revision 1 where they differ.

Reports examined: `reviews/plan-completeness.md` (B1, B2, A1-A3), `reviews/plan-constraints.md` (BLOCKING 1-3, ADVISORY 1-3), `reviews/plan-design.md` (B1, A1, A2), `reviews/plan-enhancement.md` (items 1-8), `reviews/plan-adversary.md` (F1-F3). Also `issue-analysis.md` (R1-R19), `prior-art-assessment.md`.

Owner decisions in force: self-census gate waits for issue 6 ("Leave the 6 work for later"); PR says "Part of #9"; issue 21 tracks the timing-test conversion; no timing runs.

Tree evidence checked (HEAD a9d6852): `docs/capabilities/*.md` (block count and `## Verification` headings), `tests/docs_contract.rs` (header, `INVENTORY` 75-98, `violations` 527-538, repository tests 552-588, `checkouts` fixtures 700-756), `tests/support/mod.rs`, `.githooks/pre-commit`, `Makefile`, `.github/workflows/ci.yaml`, `crates/raf-workplan/src/{generated.rs,gate.rs,assessment.rs}`, `crates/cargo-workplan/src/{cli.rs,command.rs}`, `crates/raf-okf/tests/validate_cost.rs`, `work/changes/`, `AGENTS.md` sections 3 and 7.

## Recheck (Revision 2)

| First-pass finding | Status | Evidence in Revision 2 |
| --- | --- | --- |
| BLOCKING 1: contract table not reissued | **DISCHARGED** | Lines 64-73: one table with R-ids, Task column 1-4/blocked/done, acceptance cells matching Revision 1 Tasks 1-4 (no planted fault, no open-record assertion, matrices and tool-completed records each have a row). |
| BLOCKING 2: local validation route unnamed | **DISCHARGED** | Line 76: the ordered command list, `--skip within_the_bound` with the three test names' suffix, `raf-source` release tests kept, CI runs the probe (`ci.yaml` 85-86), PR body states the exception, no-timing rule told to every agent, no `Instant`/sleep/timeout in the new test. |
| ADVISORY 1: empty-load guard named `INVENTORY` | **DISCHARGED** | Line 79: `open()` then `!capabilities().is_empty()` then empty `check()`; the inventory count stays in `docs_contract.rs`. |
| ADVISORY 2: Task 1 fixture lacks a delivered page | **DISCHARGED** | Line 82: one delivered capability page added to `checkouts::files()` and `checkouts::INVENTORY`, bindings resolving in `checkouts::source()`, mutated twin removes the block, violation names page and kind. |
| ADVISORY 3: claims-supported half of enhancement 1 undispositioned | **DISCHARGED** | Line 85: declined with the reason (one evidence record; `change complete` judges claims per change; lands with the census self-gate) and the cost (a claim could lose support between changes). |
| ADVISORY 4: declines of enhancement items 2 and 3 state no cost | **STILL-OPEN (advisory)** | Line 86 restates costs for the CI `verify` job and `GATE_TESTS` (items 1-dynamic and 5), which already had them. Item 2 (bind the self-gate test in a capability: the dogfood gate sits on no verification matrix; a renamed test is caught only by the hook's missing-binary cargo error) and item 3 (`tracked_paths()` to the gate: a `TMPDIR` inside another work tree makes the snapshot gate list the wrong index and fail with `link-broken` noise, a false failure not a false pass) still have reason without cost. Not blocking: both are LEVERAGE/TRANSFORMATION items whose decline reasons are technical, and neither cost is a correctness hole. |
| ADVISORY 5: name issue 21 | **DISCHARGED** | Lines 76 and 89. |

Open decisions from the first pass: both now carry a recorded disposition (lines 88-90). Decision 1 is taken as "skip the three `within_the_bound` tests locally; hosted CI runs them", which is the stricter reading of the owner's no-timing rule and consistent with issue 21. Decision 2 is "stop and ask the owner" if Task 3 finds a record not removed by `change complete`; this picks the "goes to the owner" branch of Revision 1 line 50, so the two revisions do not conflict.

New integration issue, advisory only:

### ADVISORY 6. Revision 2 opens a code fence it never closes, and its table heading says "Revision 1"

Plan line 63 is a bare ```` ``` ```` with no closing fence, so from line 64 to the end of the file every heading, list and the dispositions render as one code block; line 64 reads "## Completeness contract (Revision 1)" inside Revision 2. The plan is under `.stage/` (ignored), so nothing committed is affected, and the text is unambiguous when read raw.

REWRITE TO: delete line 63 and rename line 64 to "## Completeness contract (Revision 2)".

## Judgement of the plan as a whole (Revisions 1 and 2 read together)

* One authoritative contract (lines 64-73); every task of Revision 1 has a row with an acceptance check on a shipped entry point (`make docs-sync`, `cargo test --test docs_contract`, `cargo test --test self_workplan`, the hook line, `cargo workplan status`/`assess --list`, the scenarios binary, the CI `test` job).
* Task order holds: 1 (blocks, sync, re-assess, rule) before 2 (self-gate asserting every assessment current and accepted) before 3 (tool evidence) before 4 (handoff, PR "Part of #9", wait for CI). No Task 0. Each of Tasks 1-3 has review, commit through hooks, push and issue update (Revision 1 line 53).
* The seven documents lacking the block are the seven `delivered` documents without one in the tree; each has a `## Verification` section, so `generated.rs::misplaced` accepts the block and `render` fills it from the bindings.
* No committed file carries the milestone label, an issue number or deferral wording under either revision; the hook edit only adds a selector; no lint, feature or fingerprint change.
* Owner decision quoted verbatim; the blocked criterion is named in the contract and the handoff.

Nothing an implementer would have to guess remains beyond the two advisory items above.

## Open user decisions

None blocking. For the owner's awareness only: the coordinator has recorded the local test command as `cargo test --workspace --locked -- --skip within_the_bound` under the no-timing rule (line 89); if the owner prefers plain `make test` (the debug run measures with `Instant` and asserts nothing), say so and line 76 changes by one flag.

## First-pass findings (Revision 1), kept for the record

### BLOCKING 1 (discharged). The completeness contract was not reissued; Revision 1 contradicted the table it left standing

Lines 14 and 16 of the original table named a planted-fault fixture and a "no `work/changes` record names an M1 capability" assertion that Revision 1 dropped, and Revision 1's Tasks 1 and 3 had no rows. Replacement table supplied; adopted at lines 64-73.

### BLOCKING 2 (discharged). The local validation route was unnamed

"Validation commands are named in the PR" named none; `make test` runs `validate_cost.rs` in debug (`timed`, lines 80-85, still calls `Instant::now()`), and skipping all of `test-release` would drop the `raf-source` release tests. Command list supplied; adopted at line 76.

### ADVISORY 1-5 (see recheck table)

VERDICT: PASS
