# Plan completeness review

Reviewed plan: `.stage/replace-wall-clock-assertions-in-tests/plan.md` (with `clock-inventory.md`, which the plan adopts as its design source)
Against: `issue.md` (body plus 1 comment), `issue-analysis.md` (R1-R17, NEEDS-JUDGMENT 1-4)

## Inventory exhaustiveness (re-run against the tree)

I re-ran `Instant|SystemTime|UNIX_EPOCH|Duration|sleep|recv_timeout|elapsed|std::time|timeout` over `crates/raf-okf`, `crates/raf-workplan` and `crates/cargo-workplan` (`*.rs`). I also ran a wider pass for `park|Condvar|wait_timeout|chrono|Utc::now|deadline|budget|BUDGET|from_secs|from_millis|modified()|retry`.

* raf-okf: there are 14 hits, all in `tests/validate_cost.rs`, and all are in the inventory. `src` reads no clock.
* cargo-workplan: the hits are `tests/verify.rs` and `tests/complete.rs` (BUDGET and `Instant`) and `tests/sync.rs:253` (`"state: sleeping"`, a false positive). All are in the inventory. `src` reads no clock.
* raf-workplan src: the hits are `graph.rs:2027-2034`, `evidence/tests.rs:16,187-197,398-441`, `exec/system.rs` (111-147, 225-276) and `exec.rs`, `exec/fake.rs`, `bound/run.rs`, `bound/evidence.rs`. Scripted `Duration` data appears in `bound/tests.rs`, `root/tests.rs`, `lifecycle/tests.rs`, `git/tests.rs` and `exec/trusted.rs`. All are in the inventory.
* raf-workplan tests: the hits are `exec.rs` (1095-1110, 1401-1556, 1574-1657, 1699), `bound.rs`, `complete.rs`, `verify.rs`, `support/project.rs`, `scenarios.rs` (the `within_budget` callers, found by the wider pass), `coverage.rs:718-729`, `obligations.rs:1498-1513,1535-1627`, `rationale.rs:553-576`, `assessments.rs:40,836-897`, `records.rs:963-1051`, the nonces in `root.rs`, `git.rs` and `gate.rs`, and `preview.rs:396-403` (the mtime). All are in the inventory. `examples/docs_sync.rs` has no hit.
* No clock-reading or waiting site in the three crates is missing from the inventory.
* Outside the three crates (found by a repository-wide check, not in the inventory):
  * `tests/docs_contract.rs:1359-1361` has `recv_timeout(Duration::from_secs(10)).expect("reading a FIFO must not block")`, a timed wait that fails a test.
  * `tests/docs_contract.rs:1315-1319` has a `SystemTime` nonce.
  * `crates/raf-source/examples/registry_scan.rs:556-566` has the watchdog of a tool, not a test.
* raf-source part of the comment: `crates/raf-source/{src,tests}` hold no clock word. The gate `time_in_raf_source_tracked_files_name_no_clock` exists (`tests/repo_gates.rs:532`) and the Makefile has no per-test skip list. The plan's claim that this part was delivered holds in the tree.

## Requirement → task → check

| R-id | Requirement | Task | Check | Status |
|---|---|---|---|---|
| R1 | raf-source `exported_items.rs` | (delivered earlier) | none in the contract | UNMAPPED (B1) |
| R2 | raf-okf `validate_cost.rs` 100 ms | 1 | `cargo test -p raf-okf --lib profile::cost_tests`, three named mutants, the probe removed from Makefile, CI, AGENTS and CONTRIBUTING | OK |
| R3 | `assessments.rs` 60 s | 2 | NamingLookups counted, mutant `any_strong` per path | PARTIAL (B3: the 12,000-line section input has no count) |
| R4 | `rationale.rs` 60 s | 2 | ConditionsScan and ProseLinesScanned counted, two named mutants | PARTIAL (B3: the 100,000-headings and unterminated-fence inputs have no count) |
| R5 | `evidence/tests.rs` killed command | 6 | `FakeRunner` scripted past `BUILD_TIMEOUT`, three named mutants | OK |
| R6 | runner timeout and output-kill tests | 4, 5 | Frozen and Stepped clocks, pure `verdict`/`next_pause`, `/proc` ShdPnd, mutants named per test | OK |
| R7 | grep finds no elapsed assertion | 9 | extended gate with fixtures | OK for the crates; row overclaims "anywhere" (B4) |
| R8 | every converted test fails on a mutant | each, recorded in validation.md | mutants named in the inventory | PARTIAL (B2: no coverage mutant) |
| R9 | `make check`, hosted CI green | 10 | safe gates, `make test`, `make test-release`, CI 12/12 | OK |
| R10 | no timing runs, sleep waits or timing assertions | 2-9 | gate over the three crates and raf-source | PARTIAL (B4: `tests/docs_contract.rs`) |
| R11 | measure with perf or allocation counts | 1-3 | operation counts; perf target declined with a reason | OK |
| R12 | keep the guarantee each test protects | 1-7 | counted growth idiom | PARTIAL (B3) |
| R13-R17 | raf-source tests named in the comment | (delivered earlier) | none in the contract | UNMAPPED (B1) |
| graph, obligations, records, coverage (inventory additions) | | 2, 3 | counted tests with named mutants; coverage product pinned | coverage has no mutant (B2) |
| budgets, nonces, mtime | | 8 | no `BUDGET`, `within_budget` or `SystemTime` in the three crates' tests | OK |
| never-ending cargo tests | | 7 | P1 with stepped clock and SIGKILL; mutants named in the inventory | OK |

NEEDS-JUDGMENT dispositions:

* NJ1 and NJ2: plan line 4 says delivered (verified in the tree), but there is no contract row (B1).
* NJ3: "Acceptance grep widened" disposition. OK.
* NJ4: no explicit line. Advisory A3.

Structure: Tasks 1-10. There is no Task 0. A blanket header requires focused validation, task review, commit, push and issue update for every task. Task 10 creates the PR, publishes the plan and the confidence note, and files the follow-up. This is acceptable; see A2.

## Findings

### BLOCKING

**B1. UNMAPPED: R1 and R13-R17 have no contract row.**
The completeness contract has no row for the comment's raf-source scope (R13-R17) or for the body's `crates/raf-source/tests/exported_items.rs` (R1). Plan line 4 states only in prose that they were "delivered by the merged source-index change". Every R-id must appear in the contract with a falsifiable check. The delivery is real in the tree, so this is a contract defect, not missing work.
REWRITE TO (add as the first contract row):
`| raf-source tests of the issue body (exported_items.rs) and its comment (resolve_paths, manifest_caps, trait_keys, module_tree, exported_items, prescan_budgets, symbol_index, facts, tree::disk) clock-free (delivered by the merged source-index change) | 9 | the raf-source rule of the extended gate (renamed from time_in_raf_source_tracked_files_name_no_clock) passes on the tree with its 13 words over crates/raf-source/{src,tests}; the Makefile and CI carry no per-test --skip list for raf-source; whole-crate cargo test -p raf-source passes in make test |`

**B2. Mutant missing for the coverage conversion (R8).**
Plan, Task 3: "Coverage: pinned product test; two-way-coverage binding/text/assessment." Contract row 2 says only "coverage pinned exactly". `clock-inventory.md` R2 (Coverage) names counters (`ExclusionTests`, `EntriesJudged`) but no mutant. Every other conversion names one. Without a mutant, R8 ("Every converted test still fails on a mutant that removes the bound it protects") has no check for this test.
REWRITE TO (Task 3):
`3. Coverage: unit test in coverage/tests.rs pinning ExclusionTests == entries x exclusions and EntriesJudged == entries at (1,2,3 x entries, fixed exclusions) and (fixed entries, 1,2,3 x exclusions); mutants that must fail: (a) Judge::judge calls exclusion_for once per path of matching (for path in matching { exclusion_for(.., &[path]) }) instead of once per entry; (b) exclusion_for is called a second time in the not-covered branch; two-way-coverage binding/text/assessment.`

**B3. SILENT CUT: inputs of converted tests whose bound gets no count (R3, R4, R12).**
The issue requires "keeping the guarantee each test protects (... bounded assessment and rationale work ...)". The inventory counts only some inputs of each test. The integration tests then drop their clock, so the other inputs lose their bound without the plan saying so.
* `tests/assessments.rs:833-840`: a 12,000-line Contract is accepted and judged current. Only `NamingLookups` (symbol naming) is counted, and nothing counts the cost of digesting and judging the huge section.
* `tests/rationale.rs:546-566`: 100,000 level-3 headings under a decision, and the same page with an unterminated fence. Only "the 20,000-empty-conditions page and the 20,000-pair Explanation" are counted. The fence cost lives in raf-okf's `Body::headings`, which raf-workplan's cfg(test) counters cannot see.
The plan's "uncounted loops: accept the limit" disposition covers new loops on an otherwise counted path. It does not cover inputs that have no counted path at all.
REWRITE TO (Task 2, append):
`For each input of a converted test, the counted unit test names the counter that bounds it: assessments, the 12,000-line section counted by <named counter over the section digest/currency pass> at 1,2,3 x 4,000 lines (mutant: re-digest the section per line); rationale, the 100,000-headings page counted by ConditionsScan and HeadingsVisited in decision_page at 1,2,3 x N headings (mutant: call extent for every heading, not only conditions headings); the unterminated-fence input is raf-okf's Body parse, counted in raf-okf's cost_tests (Task 1) with a fence case (mutant: rescan for the closing fence per line). An input left uncounted is listed by name in the capability's Verification text as the accepted limit.`

**B4. Contract row 9 claims "no assertion on elapsed time anywhere" but leaves a timed wait in a test outside the gate (R7, R10).**
Plan row: `| no assertion on elapsed time anywhere | 9 | the extended gate (... banned in src and tests of the three crates ...) |`. `tests/docs_contract.rs:1359-1361` (`refuses_non_regular_files_instead_of_blocking`) fails on `recv_timeout(Duration::from_secs(10))`, a timing wait of the kind the plan removes everywhere else. `tests/docs_contract.rs:1315-1319` keeps a `SystemTime` nonce. Neither is in the inventory, and neither is covered by the gate.
REWRITE TO (Task 8, append; and row 9):
Task 8: `... and the workspace tests/docs_contract.rs: Scratch::new drops the SystemTime nanos (pid plus the existing counter, create_dir retried on AlreadyExists); refuses_non_regular_files_instead_of_blocking calls read_regular on the test thread with no channel or recv_timeout (a regression blocks and is caught by CI timeout-minutes, the watchdog disposition of the budgets).`
Row 9: `| no assertion on elapsed time in any test or library of the workspace | 8, 9 | the extended gate (rules for raf-source, the three crates and the workspace tests/ directory, exempting only crates/raf-workplan/src/exec/clock.rs and the planted fixture strings of tests/repo_gates.rs) passes on the tree, with fixtures for a violation, a clean neighbour and the exemption |`

### ADVISORY

* **A1.** Contract row 2 (`counted unit tests with growth idiom ... and the inventory's mutants failing fast`) names no test command or test names, unlike row 1. REWRITE TO: `cargo test -p raf-workplan --lib graph::tests rationale::tests assessment::tests obligation::tests coverage::tests passes with the new counted tests (names as in clock-inventory.md R2); each named mutant fails one of them`.
* **A2.** The tasks are one-line list items under a single heading, not `## Task N` sections. Each task's focused validation command lives only in the contract table. Consider giving each task its validation command inline.
* **A3.** NEEDS-JUDGMENT 4 (is a "deterministic signal" an allowed replacement) has no explicit disposition line. The summary adopts it and R6 permits it. Add: `* Deterministic signal (NJ4): allowed, as R6 states; used for the FIFO pid hand-off, SIGKILL in ShdPnd and the ECHILD reap.`
* **A4.** Obligations: name the shapes the counted `hostile_text_is_read_in_linear_time` must cover. These are `will` x 200,000, `](` x 200,000, `is\n` x 150,000, the link mix, the 180,000-word line and the 100,000 lines from `tests/obligations.rs:1552-1628`.
* **A5.** `crates/raf-source/examples/registry_scan.rs:556-566` reads a clock as a tool watchdog (class D). The renamed raf-source rule still scopes `src` and `tests` only. State this in the gate disposition so it is a decision and not an omission.
* **A6.** The inventory leaves two options open: the preview `snapshot` choice "(or allow the type in the gate)" and the "Optional: a counting `$CARGO` wrapper". Row 8 settles the preview choice implicitly (no `SystemTime`). State both choices in the plan's dispositions.

VERDICT: NEEDS-REVISION
