# Completion audit (Stage 2, independent): issue 10, HEAD 43c7e02

## Scope and method

* Reviewed: branch `paudley/10-m2-rest-of-the-planning-essay-entity` at `43c7e02` against
  `origin/main` `bca2604` (42 commits, 204 files), PR 20. Plan: `plan.md` revision 4
  (R1-R13, D1-D19). Prior audit: `reviews/completion-audit.md` (BLOCKED at `a51977f` on
  F1 module-binding history and F2 `--since` date basis). Validation: `validation.md`
  11A-11F3, "Task 12 fix F1-F2", "Task 12 stale cleanup".
* Execution surface: detached worktree `/tmp/a10` at `43c7e02` (`git worktree add`),
  `CARGO_TARGET_DIR=/tmp/a10-target`, `CARGO_BUILD_JOBS=6`; the binary was built there
  (`cargo build -p cargo-workplan --locked`). Mutating experiments ran in two independent
  clones of that copy, `/tmp/a10x` and `/tmp/a10p` (own `.git`, own `origin/HEAD`, own
  target dirs `/tmp/a10x-target`, `/tmp/a10p-target`); every scratch commit/reset stayed
  there. Nothing in the issue worktree was modified, staged, stashed or reset (only this
  report was written). No toolchain change. No live or deployment action.
* `.deficiencies` (worktree and root): notice and marker only, no entries.
* Hosted CI: `gh pr checks 20` all 12 jobs pass (test, test-macos, test-windows,
  test-release, cross-check, clippy, doc, fmt, msrv, deny, reuse, fuzz), run 38997274163,
  `headSha` 43c7e02, event `pull_request`.

## Criteria

| # | Requirement | Demonstration (real entry point) | Observation | Status |
|---|---|---|---|---|
| C1 | Each of the five capabilities is `delivered`, through `cargo-workplan change complete` | (a) `status` at HEAD. (b) `git show` of each deleting commit. (c) **Re-execution of every delivery**: on `<commit>^` in `/tmp/a10x` (`origin/HEAD` = real `origin/main` bca2604), apply only the hand edits of the commit (state line, INVENTORY test), `docs sync`, `change reconcile <slug>`, `change complete <slug>`, `git add -A`, compare `git write-tree` with the commit's tree (`/tmp/a10-scripts/repro.sh`) | (a) exit 0: delivered 12 (the 5 + 7 reopened), implementing 0, considered 0, designed 9 (census, untouched). (b) d561881, fcd69d1, 7b8ae50, 68a3342, 023df72 and 09a2c8d (entity-history again after the F1/F2 reopening) each flip `state: implementing -> delivered` and delete the record in the same commit (4 files each). (c) For all six: reconcile empty (exit 0); complete exit 0 (31-45 s) printing `completed: removed work/changes/<slug>.md`; resulting tree **identical** to the commit (e.g. d561881 `4a79e48c...` = `4a79e48c...`; 09a2c8d `44dea3a9...` = `44dea3a9...`). Commit times are consistent with validation 11F3 durations | demonstrated |
| C1' | The reopened delivered capabilities (spot check okf-profile, source-index, workplan-document-model, workplan-cli) | same re-execution with only the state line flipped (`repro2.sh`) | okf-profile ef62049 (53 s), document-model 6a132df (129 s, with the `rewrote` of its Changed-by row), workplan-cli 11ce02d (259 s, with its rewrite), source-index a8bd498 (422 s): reconcile empty, complete exit 0, trees identical to the commits (e.g. a8bd498 `66e65ae9...` = `66e65ae9...`) | demonstrated |
| C2 | The change record is deleted on completion; history in Git | `git log --diff-filter=D --stat -- work/changes/<slug>.md` for all 12; `ls work/changes`; `git diff --stat origin/main..HEAD -- work/changes/` | One deleting commit per record (entity-history twice). At HEAD `work/changes/` holds only the 9 census records, all present on `origin/main` and unchanged. `history change:workplan-entity-history` exits 2 (no loaded document; the record lives only in Git, as the Contract says: "Recovering it is a Git operation") | demonstrated |
| C3 | No deferrals: every item `implemented` or `moved:` to a tracked record | each deleted record read at `<commit>^`; diff of each of the five records `origin/main` -> pre-deletion; grep for `moved:change` | 5 M2 records: 7+2, 5, 6, 5, 5 items, all `outcome: implemented`; 7 reopened records: 6, 6, 7, 2, 7, 4, 8 items, all `implemented`. No `moved:`, no `dropped`, no new record. The item rewordings (shallow-only refusal, pickaxe last-resolved commit, adverse-only `evidence-changed`, scope-limited `check`) each trace to a plan decision (D4, D6, D9 owner decision, D11); none hides an unbuilt part (an in-limit evidence change still stales the assessment of the section holding the claim: shown below) | demonstrated for records; PR-body listing open (F-B) |
| C4 | Lifecycle run with the tool (issue context) | full scratch lifecycle in `/tmp/a10x`, all through the CLI (next section) and the six re-executions of C1 | as below | demonstrated |
| C5 | Delivered state true at HEAD: bound tests and examples | `cargo-workplan --format json verify` (no slug: every capability) in `/tmp/a10` at HEAD | exit 0, 1259 s, **1455 bindings, all passed**: 1429 tests, each with proof `1 listed, 1 passed, 0 failed, 0 ignored` (real `#[test]`s that run on Linux); 26 examples: 25 doc tests with `1 selected, 1 ok lines` (built and run, not `no_run`/`compile_fail`) and the `docs_sync` example target. Per capability (tests/examples): okf-profile 88/2, source-index 292/1, workplan-cli 197/4, complete-gate 198/2, current-rationale 16/1, document-model 250/3, entity-history 34/1, evidence-claims 32/2, generated-docs 84/3, prose-assessment 26/2, reconcile 190/4, two-way-coverage 22/1. No failing binding | demonstrated |
| C6 | Delivered state true at HEAD: no stale evidence or assessment | `assess --list`; probe: for each of the 12 a scratch `change new probe-<cap>` affecting all 7 sections, then `change reconcile` (`/tmp/a10p`, `origin/HEAD` = HEAD) | `assess --list` 54/54 `current`. Probe: 0 `evidence-stale`, 0 `assessment-stale`, 0 `claim-unsupported`, 0 `evidence-changed`, 0 `assessment-rejected`, 0 `binding-missing`, 0 `tense-mismatch` in all 12; only probe artifacts (`section-untouched`, `assessment-missing` for never-assessed sections outside every delivered record, `generated-stale` for the probe's Changed-by row) | demonstrated |
| C7 | Real use on the repository | `check`; `coverage code/tests --crate raf-workplan/cargo-workplan`; `coverage code`; `history cap:workplan-entity-history`; `history arch:normal-form`; `evidence claims gate-kills-a-test-past-its-timeout`; `evidence for cap:workplan-complete-gate` | `check` exit 0, no output. Coverage of both workplan crates exit 0, no gap; workspace `coverage code` exit 1 with 212 raf-okf/raf-source gaps (owner scope decision). `history cap:workplan-entity-history` exit 0, 11 commits; module binding `raf_workplan::history` now listed for b4ca6c8 and 6b317fc, which edited only `history.rs` / `history/render.rs` (F1 fixed). `history cap:workplan-current-rationale` lists d6f8804 with `raf_workplan::rationale`. `arch:normal-form` 3 commits; JSON `history-entry` rows. Claims: the claim at `workplan-complete-gate.md:410`; record `pass` | demonstrated |

### Full lifecycle in a scratch clone, through the CLI (`/tmp/a10x`, `origin/HEAD` = 43c7e02)

Capability `cap:workplan-two-way-coverage` (delivered; 6 of 7 sections assessed).

1. `change new audit-cov --capability cap:workplan-two-way-coverage --affects ...#usage --affects ...#explanation --copyright ... --licence CC-BY-4.0`: exit 0, `created work/changes/audit-cov.md`, prints the PROPOSED block and the state guidance (`state-delivered-proposal`, `state-delivered-open-record`).
2. `change reconcile audit-cov`: exit 1; `section-untouched` x2, `assessment-missing` (explanation). `check`: `state-delivered-open-record`.
3. Edits: `state: implementing`; a Usage sentence followed by `<!-- raf:claim evidence=audit-peak -->`; an Explanation sentence; one record item. `check` exit 1: `claim-evidence-missing` (file and line 151) and `generated-stale` (index).
4. Evidence (the requested three runs):
   * `evidence record --id audit-peak --for cap:workplan-two-way-coverage --by agent --limit peak=5 ... -- sh -c 'echo peak=3'`: exit 0, `recorded: work/evidence/audit-peak.md (pass)`; the record holds `subject: command`, the input digests of both code bindings, `verdict: pass`, `command`, `values: peak: 3`, `limits: peak: 5`.
   * same with `echo peak=9` (id `audit-peak-over`): exit 1, `(fail): over the limit: peak`, a `fail` record with `peak: 9`.
   * same with `echo peak=1e5` (id `audit-peak-exp`): exit 2, "the value `1e5` of `peak` is not a number ... nothing was recorded"; `work/evidence/` unchanged.
5. `docs sync`; `check` exit 0. Reconcile: `assessment-stale` (usage: section changed; evidence record is a new input), `assessment-missing` (explanation), `item-without-outcome`.
6. Edited the bound module `raf_workplan::coverage` (one comment in `coverage.rs`). Reconcile: **`evidence-stale`** ("`evidence:audit-peak` was made against `raf_workplan::coverage`, which has changed since"), `assessment-stale` naming the symbol, `assessment-missing`; `assess --list` shows all six two-way-coverage assessments stale with the symbol named.
7. `state: delivered`, outcome `implemented`, `docs sync`, then **`change complete audit-cov`: refused**, exit 1, 16 s, no cargo started, record kept, index and tree unchanged: `refused: 5 obligations, 4 sections without a current accepted assessment` and the five lines (see F-A on the count).
8. `evidence record --update ... -- sh -c 'echo peak=9'`: exit 1; reconcile gives `claim-unsupported` twice at line 151 (verdict fail; `peak` is 9, over its limit 5). Re-recorded `peak=3`: exit 0, `pass`.
9. `assess ...#explanation --context` and the same with `--by human`: byte-identical (1300 lines). `assess ...#usage --verdict rejected --by human`: `replaced: ... (rejected by human)`; reconcile `assessment-rejected`. Then `--verdict accepted --by agent` for all seven sections (`recorded:`/`replaced:`, exit 0). Reconcile: empty, exit 0.
10. **`change complete audit-cov`: exit 0**, 34 s (ran the 22 bound tests and the doc test), `completed: removed work/changes/audit-cov.md`; record gone; `check` exit 0; `status` delivered 12.

### Completion-gate refusals, one fault each (`/tmp/a10-scripts/cond.sh`, change listing two-way-coverage, `affects #contract`)

Every run: exit 1, record present after, tree byte-identical (status and file hashes before and after).

| Condition | Injected fault | Output headline |
|---|---|---|
| 1 | contract not edited | `refused: 1 obligation` (`section-untouched`) |
| 2 | item without outcome | `refused: 1 obligation, 1 item without outcome` |
| 3 | `> [!PROPOSED change:cond-3]` block in the contract | `refused: 2 obligations, 1 unresolved PROPOSED block, 1 finding once the record is removed` (`unresolved-proposal`, `tense-mismatch` for a delivered capability, `state-delivered-proposal` after removal: one fault necessarily reports three ways) |
| 4 | frontmatter `description` changed, no `docs sync` | `refused: 1 obligation, 1 stale generated block` (index catalogue) |
| 5 | one bound test made to fail at run time (`the_first_matching_exclusion_gives_the_reason`) | `refused: 1 bound test failing` (`0 passed, 1 failed, 0 ignored`), 28 s; a compile-breaking edit instead gives `refused: 22 bound tests failing` |
| 6 | `#explanation` in affects, edited, no assessment | `refused: 1 obligation, 1 section without a current accepted assessment` (`assessment-missing`) |

### Further probes

* `evidence-changed` (`/tmp/a10p`, merge base = HEAD): re-recording `gate-kills-a-test-past-its-timeout` with `-- sh -c 'exit 3'` gives exit 1 `fail`; reconcile of a change listing complete-gate: `claim-unsupported` (verdict fail), `evidence-changed` ("passed at the merge base and fails now"), `assessment-stale` (record changed). Removing the record: `claim-unsupported` + `evidence-changed` ("not a loaded record now"), `check` `claim-evidence-missing`.
* Coverage enforcement: an unbound `pub fn` at the `raf-workplan` crate root gives `check` exit 1 `coverage-item-unbound` and a `coverage code --crate raf-workplan` row; the same in `raf-okf` (out of scope) leaves `check` exit 0 and is reported by `coverage code --crate raf-okf`; an unbound `#[test]` in `raf-workplan/tests/coverage.rs` gives `coverage-test-unbound`. (A `pub fn` inside the bound `raf_workplan::coverage` module is covered by the module binding, as the Contract states.)
* Current rationale: removing `## Conditions to revisit` from `docs/architecture/normal-form.md` gives `check` exit 1 `decision-conditions-missing`.
* History: shallow clone (`git clone --depth 1`) `history arch:normal-form` exit 2 with the stated message; `--since 2026-13-01` exit 2 (pattern, "not passed to a process"); id `-x` exit 2; unknown/deleted `change:` id exit 2 listing the ids of its kind.
* Mutation checks (`/tmp/a10p`, reverted after): disabling the module-file history rows (`files.into_iter().take(0)`) fails 3 bound entity-history tests (`a_module_binding_lists_the_commits_of_its_files_and_not_of_a_sibling`, `a_module_binding_asks_its_span_and_then_the_log_of_each_of_its_files_in_order`, `a_module_file_that_is_not_committed_names_the_binding_and_the_file_until_it_is`); disabling condition 6 (`assessment_failures(...).take(0)`) fails `an_agent_assessment_satisfies_condition_six_and_a_rejected_one_by_either_never_does` and `each_kind_of_obligation_makes_complete_refuse_...`. The F1/F2 tests and the condition-6 test are not vacuous.

## Open findings

**F-A (required, small): the condition-6 count in the refusal headline counts obligations, not
sections.** `crates/raf-workplan/src/lifecycle/completion.rs` `summary()` counts one
`Failure::Assessment` per `assessment-*` obligation (`lifecycle/complete.rs`
`assessment_failures`), and `assessment-stale` is owed once per moved input. Demonstrated in
`/tmp/a10p`: a change affecting only `cap:workplan-two-way-coverage#usage`, with the section
and the bound `raf_workplan::coverage` both edited, prints
`refused: 2 obligations, 2 sections without a current accepted assessment` for **one**
section (JSON: two `condition: assessments` refusals for the same section). The lifecycle
run above printed "4 sections" for two. The complete-gate Contract (lines 267-277) says the
refusal counts `sections without a current accepted assessment`, and each such section is a
refusal of condition 6; no bound test covers a section with several moved inputs. The
delivered headline is false as soon as an assessment has more than one moved input, which is
the common case for module bindings.
REQUIRED: count distinct (capability, section) subjects for condition 6 (or, if the owner
prefers the per-input count, rename the count and state it in the Contract); add a fixture
with one section and two moved inputs whose headline says `1 section` beside the existing
two-section neighbour; carry it through the lifecycle on `cap:workplan-complete-gate`
(`change new`, `implementing`, reconcile, re-assess the Contract/Examples/Verification it
moves, `change complete`).

**F-B (required before merge, PR text only): the PR body does not carry the rewording lists
and has stale statements.** Plan D14/R8 ("reworded claims are listed in the PR body") and the
prior audit both require the lists in the PR body; the body only points to "the stage
validation notes (sections 11A to 11E)", which live under the ignored `.stage/` directory and
are invisible to reviewers on the forge. The body also says hosted CI "has not run on this
branch" / "none has run on this branch" (it ran and passed on 43c7e02) and that the
whole-repository `verify` was never run (this audit ran it: 1455/1455 pass).
REQUIRED: put the record-item rewordings of the 12 records (validation 11A, 11B, 11C "Record
items", with their reasons) and the delivered-sentence rewordings/removals (11A, 11B, 11C,
11D fix lists, 11E) into the PR body or a PR comment, and update the CI and verify
statements.

Advisory, not blocking:

* Validation "Task 12 stale cleanup" records a scratch commit first made with
  `-c core.hooksPath=/dev/null` and undone; it never reached a branch, but it is a hook bypass
  in a worktree of this repository and should not recur.
* The `evidence record --from-binding` record at HEAD has no `values`/`limits`; that is
  correct for a binding source.

## Scope findings and owner decisions applied

* Owner decisions quoted in the plan and applied: coverage scope "Workplan crates only
  (Recommended)" (`check` enforces `raf-workplan`, `cargo-workplan`; reports cover the
  workspace), evidence "Run the command (Recommended)", hosted CI counts for `cross-check`.
* Seven delivered capabilities were reopened and re-delivered (six planned, plus
  workplan-generated-docs), each through its own capability-named record and `change
  complete`.
* No failure shape found for the five headline capabilities: each mechanism has a real
  caller (CLI) exercised above (not DARK/TEST-ONLY), outputs are populated (not VACUOUS),
  and the tests are not weakened (mutations fail them).

## Result

C1-C7 have execution evidence. Two findings remain open: F-A is a demonstrated, small
defect in the delivered condition-6 output of `cap:workplan-complete-gate` (and so of
`cap:workplan-prose-assessment`'s gate claim), and F-B is a PR-body requirement of the plan.

VERDICT: BLOCKED

REQUIRED fixes:
1. F-A: count condition-6 refusals by section (or rename the count and state it), with a
   one-section/two-input fixture beside the two-section neighbour, carried through the
   lifecycle on `cap:workplan-complete-gate` (record, `implementing`, reconcile, re-assess,
   `change complete`); then re-run `check`, `assess --list` and the stale probe.
2. F-B: PR body (or a PR comment) lists the item and sentence rewordings with reasons, and
   the CI/verify statements are made current.
