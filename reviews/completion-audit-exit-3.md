# Completion audit (Stage 2 exit, delta 3, independent): issue 10, PR 20, HEAD 62709dd

## Scope, method and limits

* Delta: `0351571..62709dd`, one commit, `test(raf-workplan): kill a process group whose leader
  exited before the kill`. It touches 8 files: `exec/system.rs` (two tests), `ci.yaml` (one
  macOS step), the complete-gate bindings and their generated rows, four complete-gate
  assessments, and the evidence record `gate-kills-a-test-past-its-timeout`.
* Previous report: `reviews/completion-audit-exit-2.md`, BLOCKED with four required fixes. Each is
  re-checked below.
* Where things ran:
  * The issue worktree, read-only: `check`, `assess --list`, `status`, `evidence for`, the
    `coverage` reports, and `cargo test` of the lib tests. `git status` was empty before and after.
  * `/tmp/x10f`: a clone of the local repository, detached at 62709dd, with `origin/main` =
    `bca2604`, which equals GitHub `main` and is the merge base. Used for the stale probe, the
    re-record and re-assess check, the mutant, and the make gates. Each step ends with
    `reset --hard` and a clean status. Nothing was committed and no hook was involved.
  * `/tmp/x10f-v`: a second clone at 62709dd, for the whole-repository `verify`.
* Files:
  * binary: `/tmp/x10f-bin/cargo-workplan`, copied from the worktree after `cargo run` had
    confirmed it was fresh;
  * scripts: `/tmp/x10f-scripts/`;
  * outputs: `/tmp/x10f-probes/` and `/tmp/x10f-mutants/`.
* I did not use `rm -rf`, modified no tracked file, bypassed no hook, and made no forge mutation
  and no CI re-run.
* Limits:
  * I have no macOS host, so the macOS half of the mutant check is inferred, not executed (see
    criterion 2).
  * The whole-repository `verify` was stopped before it finished, at the owner's request to skip
    long runs. Its result is not used. Instead, the binding counts the PR body states were
    recounted from the frontmatter.

## Criteria

| # | Criterion | Demonstration | Observation | Status |
|---|---|---|---|---|
| 1 | Stale evidence fixed; stale probe clean on all 12 capabilities | `/tmp/x10f-scripts/stale.sh` over the 12 delivered capabilities. Each gets a fresh reset to 62709dd, `change new stale-probe-<cap>` affecting all seven sections, `git add`, then `change reconcile`. Also `assess --list` and `evidence for cap:workplan-complete-gate` in the worktree, and a re-record and re-assess in `/tmp/x10f` | **Every `change new` exits 0.** evidence-claims: reconcile exit 0, no output. The other 11 exit 1 with probe artifacts only: `section-untouched` 29, `assessment-missing` 28, `generated-stale` 5. The 5 `generated-stale` lines are the `relationships` blocks of cli, complete-gate, document-model, generated-docs and reconcile. A `docs sync` with the probe record staged adds exactly `\| Changed by \| [stale-probe-...]`, so they come from the probe record. **0 lines** of `evidence-stale`, `assessment-stale`, `assessment-rejected`, `evidence-changed` or `claim-unsupported`; the `evidence-stale` line of delta 2 is gone. `assess --list`: exit 0, 56 records, all `accepted agent current`. `evidence for`: `pass`. Re-recording the evidence (`--update --from-binding raf_workplan::test_crate::bound::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran`, with the SPDX flags) exits 0 with `recorded: ... (pass)`. Re-recording the four assessments (`--verdict accepted --by agent`) gives a **0-byte diff**, so the committed records are exactly what the tool writes at HEAD | PASS |
| 2 | The N1 test: deterministic, bound in complete-gate, run by a macOS CI step; the `leader_exited = false` mutant fails a test that runs the real path | Read `system.rs` 330-386 and `ci.yaml` 146-149. Ran the two tests 100 times in sequence and 1600 times in 16 parallel streams (`repeat.sh`) on a host with load average about 40. `coverage tests` and `coverage code` for both enforced crates. CI logs of run 39041508235. Mutant `let leader_exited = false;` (`mutant.sh`). A Linux probe (`zombie_kill.py`). XNU `bsd/kern/kern_sig.c` `killpg1` | **The tests are real-path tests.** `sh -c 'exit 0'` is started with `process_group(0)` and is not reaped; after 300 ms the tests call `kill_group` and `terminate`. Both bindings are in complete-gate, the generated rows are present, `check` exits 0, and the coverage reports show no gap row. **Their verdict is deterministic.** Both branches give `Ok`: a live leader means `killpg` succeeds, and an exited leader means `EPERM` with `try_wait` = `Some`. 1700 runs, 0 failures. **CI:** the step "Test the process group kill on macOS" ran at 62709dd with "running 5 tests", both new tests ok, 5 passed (log lines 1186-1204). The Linux `test` job also ran them. **Mutant:** survives on Linux, 5 passed. That is expected: on Linux, `killpg` of a group whose only member is an unreaped zombie returns 0 (probe: state `Z`, "killpg: ok"; after the reap, `ESRCH`). **On macOS it would fail.** XNU's group filter skips `SZOMB` processes (`return !(p->p_flag & P_SYSTEM) && p->p_stat != SZOMB;`), and then `error = (karg.nfound > 0 ? 0 : (posix ? EPERM : ESRCH));`. So an exited, unreaped group gets `EPERM`, and the CI failures at 99d5487, 6efed08 and 4dcdebc show exactly that for an exited `sh`. With `leader_exited = false`, `group_signal_outcome(Err(PERM), false)` is `Err`, and both `expect`s panic. I did not execute this, because I have no macOS host | PASS (macOS mutant kill inferred; see A1) |
| 3 | PR body meets the P1 acceptance (Revision 2) | `gh pr view 20 --json body`, read at the start and at the end: identical (`cmp`), sha256 `4a82d1da...`, `updatedAt` `2026-10-17T15:04:41Z`, 206 lines. Each statement compared with `gh run list`, the CLI, the frontmatter and validation 11A-11E and C1-C3 | **Met:** no 'has not run', 'none has run', 'never run', `.stage`, `Co-Authored` or agent credit. The head SHA is `62709ddc...`, equal to `git rev-parse HEAD` and the remote. The current CI result is right: run 39041508235, 12 jobs. The whole-repository `verify` figures match the frontmatter count (1459 test bindings + 27 example bindings = 1486, the five `exec::system::tests` included). Present: the rebinds, the nominal dependency reading, the id rule, the record-item rewordings of all 12 records, the 11A-11E rows, the remediation rows (C2-B fixes 2 and 3 included), owner readings, the hook-bypass slip, the plan-review note, the consciously accepted list, and the roadmap-guard note. No stale figure: no "54"; 56 assessments, matching `assess --list`. The G5 statement is correct: `bound/run.rs` 126-128 records `elapsed` for `OutputTooLarge`, and SECURITY.md 157/234 and CHANGELOG 225-231 agree. The local-gate figures 1948 and 524 are the C3b `make test` and `make test-release` results. **False statements:** see "PR body findings" below | FAIL |
| 4 | Hosted CI 12/12 on 62709dd | `gh pr checks 20`; `gh run view 39041508235` | Exit 0, all 12 pass: clippy, cross-check, deny, doc, fmt, fuzz, msrv, reuse, test, test-macos (6m7s), test-release and test-windows. `headSha` 62709ddc005aa78e0dd542c1245713df7c2f3689, event `pull_request`, attempt 1, conclusion success. The kept racy test `output_the_tool_cannot_read_is_exit_two_and_writes_no_record` also passed on macOS (log line 1390) | PASS |

Further checks at 62709dd (all pass):

* In `/tmp/x10f`:
  * `make fmt`, `make reuse` (485/485), `make licenses` and `make issue-refs` exit 0. The last two
    run `repo_gates`, 57 passed.
  * `make docs-sync` twice: exit 0, no diff and no untracked file.
  * `cargo test --test docs_contract --test repo_gates`: 28 and 57 passed. `-p raf-workplan --test
    roadmap`: 15 passed. This replays the pre-commit gate on the tree.
* `check` exits 0 in the worktree. `status` shows delivered 12, implementing 0, designed 9.
* `.deficiencies` holds only the notice and the marker.
* The commit is authored and committed by `Patrick_Audley`, with no trailer and no agent credit.

## PR body findings (criterion 3)

1. **Line 19 misstates the CI history.** It reads: "Earlier runs on this branch were cancelled or
   failed; the one failure was `test-macos` on `4dcdebc`". `gh run list` shows 12 successes, 5
   failures and 24 cancellations. Ten green runs came before 0351571, from 43c7e02 to 6690803. The
   other failures are:
   * 8e5f568: `test` and `test-macos`, a history test;
   * 99d5487 and 6efed08: `test-macos`, the same kill race in
     `output_the_tool_cannot_read_is_exit_two_and_writes_no_record`, where both shapes say "could
     not be killed";
   * 6679168: `test-macos`, the `raf-source` `manifest_caps` 1 s bound.

   So the race appeared three times, not once. The `manifest_caps` flake is the host-speed issue
   that delta 1 (A1) wanted reviewers to know about.
2. **Line 21 is false at the head.** It reads: "`change reconcile` for each capability prints
   nothing". No change record exists for any of the 12 capabilities: `change reconcile
   workplan-cli` exits 2 with "no change record `workplan-cli`". With a probe record, 11 of the 12
   print `section-untouched`, `assessment-missing` or `generated-stale` lines (exit 1). The
   probe sentence that follows it is true.
3. **Line 197 says the local gates are green on the head.** It reads: "Final local gates and
   hosted CI cover the full-suite check ...; they are green on the head". Line 22 itself says the
   local gates ran at 4dcdebc.
4. **Line 83 says too little was removed.** It reads: "Only the five Open questions lists were
   removed outright". Validation 11C evidence-claims (5) also removed a Constraints bullet
   outright, the duplicate 1024-values bullet, and row 118 lists it as "removed". By the body's
   own definition ("the sentence is gone"), that is a removal.
5. **The rewording list misses four rewordings that validation counts.** The other rows of
   11A-11E and C1-C3 are there, itemised inside grouped cells. I counted items, not rows. The
   missing four are:
   * 11A okf-profile 2: the Validate sentence widened with the record variants (`RafDocument::Evidence|Assessment|Coverage`);
   * 11B workplan-reconcile: the two completion-gate clauses merged into "The per-kind fixture of
     the completion gate refuses each of the six trace and assessment kinds and completes once it
     is repaired ...";
   * 11B workplan-cli 5: "JSON pins of the members of each new kind" became "of each kind of the
     record commands";
   * 11C workplan-prose-assessment 3: "An assessment that moved is listed with the input" became
     "`assess --list` shows an assessment that moved with the input".

## Advisory, not blocking

* A1: the tests wait 300 ms for the shell to exit. Their verdict does not depend on this, but
  whether they exercise the `EPERM` path does. If `sh` has not exited in time on a slow runner,
  the test passes without catching the mutant. A deterministic precondition costs one call:
  `rustix::process::waitid(WaitId::Pid(pid), WaitIdOptions::EXITED | WaitIdOptions::NOWAIT)`.
  It is available on macOS with the `process` feature the crate already enables, and it waits for
  the exit without reaping. The PR body calls the tests "deterministic", which is true of the
  verdict only.
* A2: the squash-merge note of delta 1 (A3) is still absent. The repository allows merge, squash
  and rebase, and `main` uses merge commits (`bca2604`). With a merge commit, "Git keeps it"
  (line 196) holds. After a squash, the reopening records created and deleted inside the branch
  would leave no trace on `main`. State which merge method is intended, or add the note.
* A3: the whole-repository `verify` was not re-run to completion in this audit. The count of
  bindings it states matches the frontmatter, and complete-gate's 210 bindings were verified in
  delta 2 except for the two new ones. Both of those ran green in CI on Linux and macOS.

## Result

* Required fixes 1, 2 and 4 of delta 2 are done and shown with execution evidence:
  * the stale probe is clean and every record is current and byte-identical to a fresh tool run;
  * the real-path process-group tests are bound, run by a new macOS step and green there;
  * hosted CI is 12/12 on 62709dd.
* Required fix 3 (PR body) is mostly done, but the body still has four false statements and four
  missing rewordings.

VERDICT: BLOCKED

REQUIRED fix (PR body only, no code or commit needed; the head SHA and CI stay valid):

1. Line 19: replace the history sentence. Say that there were 10 earlier green runs (43c7e02 to
   6690803) and earlier failures at 8e5f568 (history test, Linux and macOS), at 99d5487 and
   6efed08 (the same macOS kill race, before the fix), and at 6679168 (the `manifest_caps` 1 s
   bound on a slow macOS runner, pre-existing on `main`).
2. Line 21: replace "`change reconcile` for each capability prints nothing" with what is true.
   There is no change record for the 12 capabilities. The probe with a throwaway record shows
   only the probe's own `section-untouched`, `assessment-missing` and `generated-stale` lines.
3. Line 197: say that the local gates ran at 4dcdebc and that hosted CI covers the head.
4. Line 83: name the removed duplicate 1024-values Constraints bullet of evidence-claims among
   the outright removals, or reword the sentence.
5. Add the four missing rewordings listed in finding 5.
