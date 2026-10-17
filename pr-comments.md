# PR #22 — comments and review threads

3 comment(s). **Every one needs a disposition** in
`review-debt.md`: addressed (with the commit hash) or declined (with the
reason posted in reply). Bot comments count.

## 1. paudley — 

# Plan: issue 9, acceptance scenarios A and B and the workplan self-gate (workplan part)

## Issue summary
Scenarios A and B, the self-run gate on this repo, and completing M1's change records with the tool. The self-census gate depends on issue 6 (and 3, 4, 5), which are unbuilt; the owner said to leave that work for later. Owner decision (verbatim): "Leave the 6 work for later." PR therefore says "Part of #9" and issue 9 stays open for the self-census criterion. Nothing is cut: the census criterion stays tracked in 9 and 6.

## Evidence found (already built)
* R-A (scenario A) and R-B (scenario B): `crates/raf-workplan/tests/scenarios.rs`, bound in `workplan-complete-gate`, run in `verify` and CI.
* All 12 M1 capabilities `delivered` with resolved bindings and generated verification matrices; no workplan change record remains in `work/changes` (only the nine census records).
* The gate runs over this repo as `repository_documents_satisfy_the_document_model` in `tests/docs_contract.rs` (Gate::workspace), under `make test`, the pre-commit hook and CI.

## Completeness contract
| Requirement | Task | Acceptance |
| --- | --- | --- |
| `tests/self_workplan.rs` runs `raf_workplan::Gate` on this repo | 1 | `cargo test --test self_workplan` runs the Gate over `gate_root()`; a planted fault (tracked doc with a broken link in a fixture tree) fails it |
| wired into `make check` and the pre-commit hook | 1 | hook line runs `--test self_workplan`; `make test` includes it; the pre-commit snapshot run passes |
| every M1 change record completed with the tool; every M1 capability delivered with resolved bindings and a generated verification matrix | 1 | test asserts the 12 M1 capabilities are `delivered`, each holds a generated `verification-matrix` block, and no `work/changes` record names an M1 capability |
| scenarios A and B pass as integration tests | 2 | `cargo test -p raf-workplan --test scenarios` passes; the capability's Verification names them (exists) |
| self-workplan gate passes in CI | 1 | CI `test` job runs the new test binary |
| self-census gate, planted duplicate fails | blocked | needs issue 6; stays open in issue 9 (owner decision above) |

## Task 1: tests/self_workplan.rs and its wiring
Create the test file (uses `tests/support/mod.rs` `gate_root()`); move `repository_documents_satisfy_the_document_model` out of `docs_contract.rs` so one test owns it (update its module doc); add `--test self_workplan` to `.githooks/pre-commit`; adjust the Makefile and docs comments; fixture test that the gate fails on a planted fault. Focused check: `cargo test --test self_workplan --test docs_contract`, and the hook run by committing.

## Task 2: handoff
Record scenario evidence in validation.md (run `cargo test -p raf-workplan --test scenarios`), open the PR ("Part of #9"), post plan and confidence note.

## Declined
Rebuilding scenarios A and B: they exist and are bound; rebuilding would duplicate. Census gate: blocked by dependency order.

## NEEDS-JUDGMENT dispositions
* "M1" is undefined in the issue: taken as the twelve capabilities that the roadmap guard (`crates/raf-workplan/tests/roadmap.rs`) pins as delivered (`okf-profile`, `source-index`, and the ten `workplan-*` documents); the census documents belong to the census milestone. Evidence: INVENTORY in tests/docs_contract.rs (12 delivered, 9 designed).
* Wiring versus the scenarios passing: both scenarios already pass (bound in the completion gate and run in CI), so the wiring needs no wait.
* "Depends on" #6 and #8: #8 is merged; #6 (and 3, 4, 5) are unbuilt; only the self-census criterion depends on them (owner decision above).
* Prior art: no prior attempt (0 commits, branches, PRs); the nearest is `Gate::workspace` in tests/docs_contract.rs and in crates/raf-workplan/tests/scenarios.rs.

# Revision 1 (supersedes Task 1/Task 2 and the completeness table above where they differ)

Findings integrated: completeness (matrix claim wrong; task steps), constraints (timing; second state list; process wording), design (B1, A1, A2), enhancement, adversary (F1 to F3).

## Corrected evidence
The milestone label of issue 9 is "M1 Workplan" (issues 7, 8, 9). Its capabilities are `workplan-document-model`, `workplan-generated-docs`, `workplan-reconcile`, `workplan-cli` and `workplan-complete-gate`; these five hold a generated verification matrix. Seven other delivered capabilities (`okf-profile`, `source-index` and five from the later essay work) hold none. Committed files say "delivered capability" and never the milestone label.

## Task 1: the verification-matrix rule for every delivered capability
Add the marker pair to the Verification section of the seven documents lacking it, run `make docs-sync` (twice: no diff), re-assess the sections that moved (assess --list current; stale probe clean), and add the rule to the `violations` family of `tests/docs_contract.rs`, keyed off `INVENTORY`: a delivered capability holds a `verification-matrix` block (fixture: a delivered document without the block fails, with its clean twin). No new slug list.

## Task 2: tests/self_workplan.rs and its wiring
New test file (SPDX header; `mod support;` with one `#[allow(dead_code, reason = ...)]`): `Gate::workspace(gate_root()).assert_clean()` moved out of `docs_contract.rs` with its empty-load guard (`Gate::open()` loads at least the inventory's capabilities); plus every assessment of `Workplan::list_assessments` is current and accepted. No planted-fault fixture (the `gate.rs` tests of raf-workplan are the evidence and are cited). Add `--test self_workplan` to `.githooks/pre-commit`; update the stale descriptions in AGENTS.md (sections 1, 3, 6), CONTRIBUTING.md, the Makefile and pre-commit headers and the `docs_contract.rs` module doc. `make test` and CI already run it through `cargo test --workspace`. Validation commands are named in the PR; `make check` includes the `raf-okf` cost probe that asserts elapsed time, which is not run locally under the no-timing rule (tracked for conversion); hosted CI runs it.

## Task 3: tool-completed records
Record in validation.md the output of `cargo workplan status` for this repo and, for each workplan change record removed, the commit that removed it and whether `change complete` did it (the history shows 'deliver ... again' commits made with the tool for the later work; for the milestone's own records, check `git log --diff-filter=D -- work/changes`). A record not removed by the tool is re-run through `change new` and `change complete`, or goes to the owner.

## Task 4: handoff
Run `cargo test -p raf-workplan --test scenarios` and record it. Open the PR ("Part of #9": the self-census criterion stays open in issue 9 under the owner decision), wait for hosted CI and record the run result on the PR head, post the plan and the least-confident note to issue and PR. Each of Tasks 1 to 3: independent task review, commit through hooks, push, issue update.

## Declined enhancements
* A CI job running `workplan verify` over every capability: about 1100 s of runner time per run, redundant with `change complete` and `verify`, which run it per change; the whole-repository verify state is tracked by the issue on failing bindings.
* A single `GATE_TESTS` definition shared by Makefile, hook and CI: three short lines in three files; indirection costs more than it saves until the census gate adds a fourth.
* Binding the self-gate test in a capability's Verification: bindings resolve against the packages the documents describe; the root package's tests are repository policy, not a capability's behaviour.
* Passing `support::tracked_paths()` to the gate: unchanged behaviour of the moved test; the hook root has no `.git` by design.

# Revision 2 (adopts reviews/plan-review.md REWRITE TO text; supersedes the contract table above and Revision 1 where they differ)

```
## Completeness contract (Revision 1)
| R-ids | Requirement | Task | Acceptance |
| --- | --- | --- | --- |
| R15 | every delivered capability holds a generated verification matrix | 1 | `grep -l 'raf:generated:verification-matrix' docs/capabilities/*.md` lists all 12 delivered documents; a second `make docs-sync` writes nothing; `cargo test --test docs_contract` passes with the new family, and its fixture test fails a delivered page without the block and passes its clean twin; `cargo workplan assess --list` shows every assessment current |
| R11, R16 | `tests/self_workplan.rs` runs `Gate` on this repo, wired into `make check` and the pre-commit hook | 2 | `cargo test --test self_workplan` passes: `Gate::workspace(gate_root())` opens with at least one capability and `check()` is empty, and every `list_assessments()` entry is current and `Accepted`; `.githooks/pre-commit` line 38 names `--test self_workplan` and a commit through the hook passes; `docs_contract.rs` no longer holds `repository_documents_satisfy_the_document_model` |
| R12, R17 | capability states advanced with `cargo workplan status` and `change complete` | 3 | validation.md holds the verbatim `status` output (12 delivered, no open workplan change) and one line per removed workplan change record: `<slug> <commit> completed-by-tool: yes/no`; every line says yes, or the owner decision below is recorded |
| R1-R10, R13 | scenarios A and B pass as integration tests | 4 | `cargo test -p raf-workplan --test scenarios` passes; bound in `workplan-complete-gate` |
| R14 (workplan half) | self-workplan gate passes in CI | 4 | the CI `test` job on the PR head concludes success and its log shows every test of `self_workplan` ok; run id recorded in validation.md |
| R14 (census half), R18 | self-census gate, planted duplicate fails | blocked | needs issue 6; stays open in issue 9 (owner: "Leave the 6 work for later") |
| R19 | depends on #8 | done | merged |

## Local validation (replaces 'Validation commands are named in the PR')
Local validation, in this order: `make fmt clippy doc deny reuse msrv fuzz cross-check licenses issue-refs`; `cargo test --workspace --locked -- --skip within_the_bound` (the three tests of `crates/raf-okf/tests/validate_cost.rs` end in `within_the_bound`; they are the only tests not run locally, by the no-timing decision; issue 21 tracks their conversion); `cargo test -p raf-source --release --locked`. Hosted CI runs the probe in release (`ci.yaml` 85-86). The PR body states this exception beside the `make check` checklist item. Every agent started for this work is told the no-timing rule. The new test uses no `Instant`, sleeps or timeouts.

## Task 2 empty-load guard (replaces the INVENTORY reference)
"`let opened = Gate::workspace(gate_root()).open()` (panic on `Err`); `assert!(!opened.workplan().capabilities().is_empty())`; then `check()` empty (keep `assert_clean()`'s report format). The count against `INVENTORY` stays in `docs_contract.rs`, where it already is."

## Task 1 fixture
"The fixture adds one delivered capability page to `checkouts::files()` and `checkouts::INVENTORY` with bindings that resolve in `checkouts::source()`; the mutated twin removes the block; the rule's violation line names the page and `verification-matrix`."

## Added declines
* Claims-supported in the self-gate: one evidence record exists today and change complete judges claims per change; the check lands with the census self-gate, which brings the second record. Cost of declining: a claim could lose support between changes until then.
* CI verify job, shared GATE_TESTS definition: costs stated above (about 1100 s of runner time per run; three one-line edits).

## Dispositions of the final review open questions (mine, following the no-timing rule; the second is a stop-and-ask)
* Local test command: skip the three within_the_bound tests locally (the no-timing rule); hosted CI runs them. Issue 21 tracks conversion.
* Task 3: if a milestone record was not removed by change complete, I stop and ask the owner before reopening or recording as-is.

## 2. paudley — 

Least confident about: (1) the "tool completed the records" evidence for the two earliest capabilities, which were removed by hand before the binary existed and rest on the later reopening to show a tool completion; (2) the failure output of the new gate test, which was read from the code and not run on a failing case. Worth knowing: nearly all of the issue was already built by earlier work, so this PR is small by design; the self-census gate stays open here until the census crates exist, and `make check` still contains the wall-clock probe that this machine does not run (hosted CI does).

## 3. paudley — 

Stage 2 summary for this PR.

* Review: one combined review on the lightweight path (a test file, a hook line, documentation and seven generated blocks). Verdict PASS: no critical, high or medium gap; four low notes (layout of the notes file, wrapping of three edited paragraphs, failure-report layout differing from the gate's own line), none blocking.
* Executed evidence: the new gate test, the document-contract and repository-gate tests, scenarios A and B (5 tests), `workplan check` and `assess --list` pass; the gate test fails on a planted broken link, on a stale assessment and on a rejected verdict; the real pre-commit hook runs it and fails with the stale fault staged.
* Hosted CI: see the checks on this PR.
* Not built, by owner decision ("Leave the 6 work for later"): the self-census gate. The issue stays open for it.

