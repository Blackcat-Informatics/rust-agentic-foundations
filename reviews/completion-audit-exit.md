# Completion audit (Stage 2 exit, independent): issue 10, PR 20, HEAD 4dcdebc

## Scope, method and limits

* Reviewed: branch `paudley/10-m2-rest-of-the-planning-essay-entity` at `4dcdebc` (81 commits,
  217 files, +39637/-1339 against `origin/main` `bca2604`); the remediation is `43c7e02..4dcdebc`
  (39 commits, 115 files). Plan: `plan.md` rev 4; remediation: `remediation-plan.md` rev 2;
  validation: `validation.md` "Stage 2 remediation A, B, C1, C2-A/B/C, C3a, C3b". Prior reports
  re-checked finding by finding: `gap-analysis.md`, `reviews/security.md`,
  `reviews/completion-audit-stage2.md` (all BLOCKED at 43c7e02).
* Execution surfaces (nothing in the issue worktree was modified, staged, stashed or reset; only
  this report was written there; no toolchain change; no forge mutation, no CI re-run):
  * `/tmp/x10`: `git worktree add --detach` at HEAD, `CARGO_TARGET_DIR=/tmp/x10-target`,
    `CARGO_BUILD_JOBS=6`; `cargo build -p cargo-workplan --locked` gave the binary used throughout.
    Edits there were temporary and restored (`git status` empty after each probe).
  * `/tmp/x10c`: own clone, `origin/HEAD` = real main `bca2604` (delivery re-execution, stale
    probe, per-commit hook-gate replay; `CARGO_TARGET_DIR=/tmp/x10c-target`, `/tmp/x10c-gate`).
  * `/tmp/x10d`: own clone with `origin/main` moved to HEAD, i.e. a fresh change on the delivered
    state (lifecycle, G1, G5, verify; `CARGO_TARGET_DIR=/tmp/x10d-target`).
  * Scripts outside the trees: `/tmp/x10-scripts/` (`repro.sh`, `stale.sh`, `hookgate.sh`, `g2.py`,
    `a3.py`); probes `/tmp/x10-probes/` (the security reviewer's `sec_writes.rs` and `sec_gate.rs`
    copied from `/tmp/sec10s/probes/` with only the scratch base path changed, plus
    `audit_s1_evidence.rs` and `audit_s1_assess.rs`; all moved out of the tree after the runs).
  * Clean-up: `/tmp/x10` was removed with `git worktree remove --force` and `git worktree prune`.
    The two clones `/tmp/x10c` and `/tmp/x10d` are independent clones, not worktrees, reset to HEAD
    with a clean status. They are left in place, because removing them needs a recursive delete.
* Not checked, by the caller's instruction: the PR body (F-B / P1; it is rewritten after this audit).
* `.deficiencies` (worktree and root): notice and marker only, no entries.

## Issue criteria

| # | Requirement | Demonstration (real entry point) | Observation | Status |
|---|---|---|---|---|
| 1 | Each of the five capabilities reaches `delivered` through `cargo workplan change complete` | (a) `status` at HEAD. (b) Re-execution of each final delivery commit in `/tmp/x10c`: on `<commit>^` apply only the hand edits (frontmatter state line, the `INVENTORY` diff of `tests/docs_contract.rs`), `docs sync`, `change reconcile <slug>`, `change complete <slug>`, `git add -A`, compare `git write-tree` with `<commit>^{tree}` | (a) exit 0: delivered 12 (the five plus okf-profile, source-index, cli, complete-gate, document-model, generated-docs, reconcile), implementing 0, considered 0, designed 9 (census, untouched). (b) reconcile printed nothing (exit 0), complete exit 0 printing `completed: removed work/changes/<slug>.md`, and the tree is **identical** to the commit for fd98b01 evidence-claims (73 s), 734a3aa prose-assessment (60 s), 91804c0 two-way-coverage (58 s), f8a4e7d entity-history (47 s) and 023df72 current-rationale (58 s, binary built at `023df72^`); also for 92521b0 okf-profile (94 s) and f5a88ff complete-gate (254 s, with its `rewrote` line). The five were `considered` on main | demonstrated |
| 2 | Change record deleted on completion; history in Git | `git diff --stat origin/main..HEAD -- work/changes`; `git log --diff-filter=D -- work/changes/<slug>.md`; `ls work/changes`; the lifecycle run below | The five M2 records are deleted (132 lines). Each deleting commit flips the state and deletes the record together (entity-history d561881, 09a2c8d, f8a4e7d; evidence-claims fcd69d1, fd98b01; prose-assessment 7b8ae50, 734a3aa; two-way-coverage 68a3342, 91804c0; current-rationale 023df72). Only the nine census records remain. In the lifecycle run, `complete` itself removed the record | demonstrated |
| 3 | No deferrals: any scope that leaves an item is `moved:` to a tracked change | Each deleted record read at `<commit>^`; records added on the branch listed | Every item of every deleted record is `outcome: implemented`. Final records: current-rationale 5, evidence-claims 5, prose-assessment 2, two-way-coverage 2, entity-history 1, okf-profile 2, source-index 2, generated-docs 2, document-model 3, complete-gate 3, cli 2. The earlier deliveries of the five have 0 items that are not `implemented`. No `moved:`, no `dropped`; every record the branch added is gone at HEAD. The "consciously accepted" list of remediation B (security A1, A2, A4, A5 and A7 stated in SECURITY.md and Constraints; maintainability; a completed change's history only in Git) removes nothing the issue asks for. Removed delivered sentences are judged below | demonstrated |

## Prior findings, one by one

| Finding | Demonstration | Observation | Status |
|---|---|---|---|
| Security S1 / R1 (record writes follow in-root directory links) | The security reviewer's own probes against HEAD, unchanged except the scratch base path; own probes for `work/assessments` and `work`; one mutant | Reproduction 1 (`work/coverage -> ../docs`, `record::replace`) was `Ok` with `docs/default.md` overwritten by `type: Coverage`. Now it is `Err(cannot write work/coverage: it is a symbolic link, and nothing is written through a link)` and `docs/default.md` stays `doc\n`. Reproduction 2 (`work/evidence -> ../docs/capabilities`, `Gate::workspace(..).source_roots(&["pkg"])`, `record_evidence` with `WriteMode::Replace`, real `SystemRunner`) is now `Err(cannot write work/evidence ...)`, and the capability document is unchanged. `work -> docs` for evidence (Replace and Create) is refused naming `work`, and the victim is unchanged; with `-- touch <marker>` as the command the marker never appeared, so nothing ran. `record_assessment` with `work/assessments -> ../docs/capabilities` (victim `docs/capabilities/omega-contract.md`) and with `work -> docs` is refused naming the link, the victim is unchanged, and the real-directory neighbour replaces. The other 11 write cases of `sec_writes.rs` stay refused; concurrency is unchanged (1 create wins, 16 replaces succeed). Mutant: `record::replace` judging no directory brings reproduction 1 back (`Ok`, document overwritten) | FIXED |
| G1 / F-A (condition-6 headline counts obligations) | Lifecycle in `/tmp/x10d` (below). The record affects only `cap:workplan-two-way-coverage#usage`; the section text and the body of `Workplan::coverage_report` (so also `raf_workplan::coverage`) are edited | `change complete` exits 1 in 12 s (no cargo) with **`refused: 3 obligations, 1 section without a current accepted assessment`** and three `assessment-stale` lines (section, `raf_workplan::Workplan::coverage_report`, `raf_workplan::coverage`). The JSON `refused` message is the same. The record is kept; index blobs and status are identical. Neighbour with `#contract` added: `refused: 6 obligations, 2 sections without a current accepted assessment` | FIXED |
| G2 (claim marker split over lines read as a plain comment) | `check` through the binary in `/tmp/x10`; each comment inserted after the first Contract paragraph of current-rationale (line 48), then restored | 3-line `raf:claim evidence=...`: exit 1, `line 48: malformed claim marker ... [okf-parse-malformed-claim]`. 3-line `raf:note`: exit 1, `[okf-parse-unknown-marker]`. `<!--`, a blank line, then `  RAF:Claim ...`: exit 1, malformed-claim. Neighbours exit 0: an ordinary multi-line comment, a multi-line SPDX comment, a multi-line `raf:generated:x`, and a 3-line claim inside a fence | FIXED |
| G3 (`evidence claims` answers while a capability does not load) | `state: bogus` in current-rationale, then `evidence claims` through the binary | exit 2, stdout empty, stderr `cap:workplan-current-rationale did not load, so a claim in it is unknown; fix its document first`. The same for text, JSON and an unknown id. Neighbours: the clean tree gives exit 0 with the claim at `workplan-complete-gate.md:427`; an unloaded change record (census-cli `contract_effect: bogus`) leaves exit 0 | FIXED |
| G5 (output-kill reason lost its time) | `evidence record -- sh -c 'yes'` and `'yes >&2'` in `/tmp/x10d`. A bound test of current-rationale that writes 20 MiB through `std::io::stdout()`, bound under a scratch record and run by `verify` | Trusted command: exit 2 and no record, with "`sh -c yes` wrote more than 16777216 bytes to standard output in 0.0 s and was killed; the run has no verdict; no record was written". The stderr variant is the same, quoted as `sh -c 'yes >&2'` (security A6). Bound test: `fail ... wrote more than 16777216 bytes to standard output in 0.2 s and was killed; the run has no verdict`; the other 16 bindings pass and `verify` exits 1. The timeout half is pinned by the bound fixture and by the current evidence record `gate-kills-a-test-past-its-timeout` | FIXED on Linux; on macOS see N1 |
| A3 (control and bidi characters reach the terminal) | ESC `[2J`, OSC `]0;PWNED` BEL and U+202E put into the claimed statement of complete-gate | `check` exits 0. `evidence claims gate-kills-a-test-past-its-timeout` and `assess cap:workplan-complete-gate#contract --context` exit 0 with no raw ESC, BEL or U+202E byte; the text shows as `\u{1b}[2J\u{1b}]0;PWNED\u{7} that \u{202e}runs ...`. The JSON forms carry `\u001b` and no raw ESC or BEL | FIXED |
| G4 (coverage cascade, advisory) | `state: bogus` in current-rationale, then `check` | exit 1 with `okf-bad-state` and 2 `generated-stale`, and no `coverage-item-unbound` | built |
| Carried advisories (SymbolLog wording, crate-root digest pin, `super::`, `docs_sync` lock, `binding_site` pin, `has_unloaded_documents` comment); security A1, A2, A4, A5 and A7 statements; A6 quoting | Diff read; the new tests are among the bindings `verify` ran; A6 seen live above | Done as validation B records. The SymbolLog tests now pin the start and the end of the message exactly | done |
| F-B / P1 (PR body) | not checked, by instruction | - | outside this audit |

## Further checks

* **Full lifecycle through the CLI** (`/tmp/x10d`, `origin/HEAD` = HEAD, cap:workplan-two-way-coverage):
  1. `change new audit-g1 ... --affects ...#usage --copyright ... --licence CC-BY-4.0` exits 0 and prints the PROPOSED block and the `state-delivered-*` guidance.
  2. reconcile exits 1 with `section-untouched`; `check` reports `state-delivered-open-record`.
  3. Edits: `implementing`, a Usage sentence, a comment in `coverage_report`, and one item. After `docs sync`, `check` exits 0 and reconcile gives three `assessment-stale` and an `item-without-outcome`.
  4. Outcome `implemented`, state `delivered`: **complete is refused** (G1 row).
  5. `assess ...#usage --context` is byte-identical for agent and human (1356 lines).
  6. `--verdict rejected --by human` prints `replaced: ... (rejected by human)`. complete is refused with `1 obligation, 1 section` (`assessment-rejected`).
  7. `--verdict accepted --by agent`; reconcile is empty and exits 0.
  8. **complete exits 0 in 53 s (the bound tests run)** and prints `completed: removed work/changes/audit-g1.md`. The record is gone, `check` exits 0 and `status` shows delivered 12.
* **Delivered state at HEAD** (`/tmp/x10`): `check` exits 0 with no output. `assess --list` exits 0 with 56 records, all `accepted agent current`. `coverage code` and `coverage tests` for `--crate raf-workplan` and `--crate cargo-workplan` exit 0 with no gap row.
* **`verify`** (`/tmp/x10d`, a scratch record listing okf-profile, complete-gate and entity-history): exit 0 in 362 s, 332 bindings, all `passed`:
  * okf-profile 90 tests and 2 examples, complete-gate 203 and 2, entity-history 34 and 1;
  * every test proof is `1 listed, 1 passed, 0 failed, 0 ignored`, and every example proof is `1 selected, 1 ok lines`;
  * the counts equal validation C3b.
* **Stale probe** (`/tmp/x10c`, a 7-section probe record per capability, then reconcile): okf-profile, complete-gate, entity-history and current-rationale give only probe artifacts:
  * `section-untouched`;
  * `assessment-missing` (sections never assessed);
  * one `generated-stale` (the probe's own Changed-by row).

  There are **0** `assessment-stale`, `assessment-rejected`, `evidence-stale`, `evidence-changed` and `claim-unsupported` lines.
* **Tests changed since 43c7e02** (`git diff 43c7e02..HEAD -- '*tests*'`: 22 files, +1418/-59). I read every removed line:
  * the spawn lock in `exec.rs` moved up to cover every process test (widened);
  * both SymbolLog assertions were replaced by exact start and end assertions (stronger);
  * `both` became `both_timed`, which normalises only the seconds of `in <t> s and was killed` and adds an assertion that the message holds them;
  * the `files()` helper of `evidence/tests.rs` now lists links instead of following them (a stronger snapshot);
  * the `docs_contract` counts went from 54 to 56 assessments and from 56 to 58 records.

  The one relaxation is the roadmap guard. "Each of the 12 is `delivered`" became "delivered, or implementing with its own open record, with no gap", as remediation plan rev 2 (P2) asked, with its own negative fixture. The exact state is still pinned: `INVENTORY` in `tests/docs_contract.rs` lists each capability's state and asserts `count("delivered") == 12` and `count("implementing") == 0`. Not a weakened test.
* **Delivered sentences removed since 43c7e02** (`docs`, `SECURITY.md`; 56 removed lines, all read): each is a restatement that says more:
  * the condition-6 sentence (G1), and "with the time it ran" restored (G5);
  * the marker rule (G2), and the `evidence claims` refusal (G3, in cli and evidence-claims);
  * the writer sentences (S1: document-model, generated-docs, complete-gate);
  * the SymbolLog wording ("is shorter in `HEAD`" became the precise "ends before the line the symbol starts at");
  * the roadmap rule, the crate-root digest sentence, the `super::` reading, and the SECURITY.md additions.

  Two are corrections of complete-gate Constraints text that over-claimed what pre-existing code does, not behaviour changes:
  * "a repository whose tests or examples cannot compile cannot complete a change": only bound targets are built;
  * "an empty list satisfies the fifth condition only while designed or considered": an empty `examples` list is owed only in `implementing`, as `obligation/sources.rs` does on main too.

  Both are recorded in validation C2-B (fixes 2 and 3) and belong in the PR body's rewording list. No silent narrowing found.
* **Tracker numbers and process wording:** `make issue-refs` exits 0 (57 passed).
* **Hook-bypass traces:**
  * all 81 commits were authored and committed by `Patrick_Audley`, and no message has a trailer or an agent credit;
  * validation records hooks on (`core.hooksPath=.githooks`) for every remediation part, and the only bypass ever recorded is the pre-remediation scratch slip already reported;
  * the pre-commit gate (fmt, reuse, `repo_gates` + `docs_contract`) was replayed on the tree of every commit in `43c7e02..4dcdebc` in `/tmp/x10c` (`/tmp/x10-scripts/hookgate.sh`): all 39 commits pass fmt, reuse and the two test binaries (28 and 57 passed at HEAD).
* **Hosted CI** (`gh pr checks 20`, run 39032433050, event `pull_request`, `headSha` 4dcdebc): 11 jobs pass and **test-macos fails** (finding N1).
  * The previous full run (6679168) also failed test-macos. That failure was a 1 s timing assertion in `crates/raf-source/tests/manifest_caps.rs` (it took 1.08 s), in a file this branch does not touch.
  * The last green run is at 6690803.

## Open findings

**N1 (REQUIRED): hosted CI is red at HEAD, and on macOS a kill after an output overflow can be
reported as "could not be killed, so it may still be running".**

* **Where:** run 39032433050, job test-macos, test
  `crates/cargo-workplan/tests/evidence.rs::output_the_tool_cannot_read_is_exit_two_and_writes_no_record`
  (a test this branch adds).
* **What failed:** the command was `--format=json evidence record ... -- /bin/sh -c 'head -c 17000000 /dev/zero; echo peak=1'`. The direct shape printed:

      `/bin/sh -c '...'` could not be read to its end: `/bin/sh -c '...'` wrote more than 16777216 bytes
      to standard output in <t> s and was killed; the run has no verdict; and the process could not be
      killed, so it may still be running: Operation not permitted (os error 1); no record was written

  The `cargo workplan` shape printed the plain kill message, so the assertion that both shapes agree
  (`support/mod.rs:280`) failed.
* **Likely cause** (code reading of `crates/raf-workplan/src/exec/system.rs`; the `EPERM` itself is observed):
  * `Capture::start` drops the pipe after the first byte past the bound, so the writer can die of SIGPIPE before the runner kills it.
  * `kill_group` accepts only `ESRCH`.
  * Darwin's `killpg` answers `EPERM` for a group whose members have all exited but are not yet reaped.
  * `abandon` then reports a process that "may still be running".
* **Consequences:**
  * The reason is false (the process has ended) and depends on a race.
  * It contradicts the delivered output-kill reason and the cli rule that both shapes give the same output.
  * The same race reaches bound tests in `verify` and `complete` on macOS.
  * The kill code itself is on main, but this branch adds the path and the test that expose it, and its own acceptance asks for "hosted CI green on the new head".
  * Earlier runs passed test-macos, so re-running until green would hide a real race.

REQUIRED:

1. In `kill_group` (or `terminate`), treat `EPERM` as done when the group leader has already exited (for example, `child.try_wait()` gives a status), and keep it an error otherwise. The reason after an overflow is then always the kill message.
2. Add a test that makes the writer exit before the kill, and keep the existing test, which test-macos runs.
3. Repeat step 2 of the remediation plan for the moved `raf_workplan::exec` digests: re-record `gate-kills-a-test-past-its-timeout`, re-assess the stale sections, `assess --list` all current, stale probe clean. Reopen a capability and complete it again only if its text changes.
4. Get hosted CI green on the final head, all 12 jobs, with test-macos run on that head.

Advisory, not blocking:

* A1: the 1 s wall-clock assertions of `raf-source` `manifest_caps` fail on a slow macOS runner
  (6679168). This is pre-existing on main and outside this issue. If it recurs on the final head it
  still blocks merge, so a reviewer should know it is a host-speed flake, not this change.
* A2: validation C2-B fixes 2 and 3 (complete-gate Constraints) correct text that over-claimed. List
  them with the other rewordings in the PR body.
* A3: a squash merge keeps the history of the five M2 records, which are on main. It does not keep
  the history of the reopening records created and deleted inside the branch. Say so in the PR,
  since "the history is in Git" then means the PR branch for those records.

## Reviewability of a 217-file, ~39k-line PR (the open question)

Measured size:

* production source: 73 files, +16440;
* tests: 59 files, +16326;
* docs: 15 files, +3795;
* records under `work/`: 63 files, +2448 (56 assessment records, 1 evidence record, 1 coverage record);
* fuzz and other: 7 files.

Nothing in it is unverifiable: every claim I tried could be executed. What a reviewer cannot do
from the diff alone, and needs help with:

1. **Delivery "through the tool" is a history claim, not a diff.** A reviewer sees state flips and
   deleted records; only re-execution shows that `change complete` produced them.
   * The recipe used here is `/tmp/x10-scripts/repro.sh`: on `<commit>^` apply the state line and the INVENTORY diff, run `docs sync`, `reconcile` and `complete`, then compare `git write-tree` with the commit.
   * It takes one to four minutes per capability and reproduced all seven commits tried byte for byte.
   * Putting that recipe in the PR would make the claim checkable.
2. **The 56 assessment records are agent judgements stored as digests.**
   * Their currency can be checked with the tool (`assess --list`, the stale probe). Their truth cannot.
   * No human read the sections, and at 43c7e02 two `accepted` sections were false (G1, G2).
   * A human should read at least the five Contracts and the SECURITY.md paragraphs on the trusted command and on writes.
3. **The hard code needs a map, not a line-by-line read.** The verification matrices name the test
   for each claim, and a reviewer should start there. The modules are:
   * `raf-source/src/exports.rs`: 1537 lines of re-export carry, with a super-linear cost that SECURITY.md now states;
   * `raf-workplan/src/digest.rs`: an in-crate SHA-256, cross-checked against `sha256sum` on 313 inputs by the security review;
   * `exec/trusted.rs` and `exec/system.rs`: the one new process (N1 is in this area);
   * the path resolution in `coverage.rs`.
4. The commit history is noisy: 81 commits, with two rounds of reopening and re-delivering up to ten
   capabilities. But the pre-commit gate replays green on every remediation commit, and each
   delivery commit is four files.

None of this blocks. Items 1 and 2 belong in the PR body.

## Result

* The three issue criteria are demonstrated with execution evidence.
* Every finding of the three Stage 2 reports that this audit was asked to recheck is fixed: S1, G1, G2, G3, G5 and A3, plus G4 and the carried items. Each was shown through the shipped binary or the public library path, with a passing neighbour.
* No test was weakened, no delivered sentence was silently narrowed, `make issue-refs` passes, and `.deficiencies` is empty.
* One finding is open: hosted CI is red at HEAD (test-macos). The cause is a macOS kill race whose false "may still be running" reason this branch exposes (N1).

VERDICT: BLOCKED

REQUIRED fixes:
1. N1: make the kill after an output overflow report the kill, not "could not be killed", when the
   process group has already exited (`EPERM` from `killpg` with an exited leader on macOS).
   * Add a test where the writer exits before the kill.
   * Re-record and re-assess what the moved `raf_workplan::exec` digests stale, until `assess --list` is all current and the stale probe is clean.
   * Get hosted CI green on the final head with all 12 jobs, test-macos included.
