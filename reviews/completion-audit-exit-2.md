# Completion audit (Stage 2 exit, delta 2, independent): issue 10, PR 20, HEAD 0351571

## Scope, method and limits

* Delta under review: `4dcdebc..0351571`, one commit, `fix(raf-workplan): a process group that
  answers EPERM after its leader exited is done` (6 files, +56/-7):
  * `crates/raf-workplan/src/exec/system.rs`: `kill_group` now takes `&mut Child`, calls
    `try_wait()` after `killpg`, and passes the result to a new `group_signal_outcome`. Three unit
    tests were added in `exec::system::tests`.
  * `docs/capabilities/workplan-complete-gate.md`: three bindings and three generated Verification
    rows.
  * Four complete-gate assessment records, where only the `raf_workplan::exec` digest changed
    (`78d5c919...` to `82381609...`).
* The previous report is `reviews/completion-audit-exit.md` (BLOCKED at 4dcdebc, one required fix,
  N1, in four steps). Its other findings were FIXED there, and the delta does not touch their code
  or text. They were not re-audited, except through the gates below.
* Execution surfaces:
  * The issue worktree, read-only: `check`, `assess --list`, `status`, `coverage`, `evidence for`,
    `evidence claims`, the test binaries, `make fmt` and `make reuse` (both check-only). Its
    `git status` was empty before and after.
  * `/tmp/x10e`: my own clone of the local repository, detached at 0351571, with `origin/main` =
    `bca2604`, `CARGO_TARGET_DIR=/tmp/x10e-target`. It was used for the stale probe, the re-record
    experiment, the mutants and `verify`. Every probe ends with `reset --hard` to 0351571 and a
    clean status. Nothing was committed there, and no hook was involved.
  * Binary: `/tmp/x10e-bin/cargo-workplan`, copied from the worktree's `target/debug` after
    `cargo run` had confirmed it was fresh for HEAD. Scripts are in `/tmp/x10e-scripts/`, outputs
    in `/tmp/x10e-probes/` and `/tmp/x10e-mutants/`.
* I did not use `rm -rf`. No tracked file was modified, no hook was bypassed, and I made no forge
  mutation and no CI re-run.

## Criteria

| # | Criterion | Demonstration | Observation | Status |
|---|---|---|---|---|
| 1 | N1 code: the kill after an output overflow reports the kill when the group has already exited | Read `system.rs` 280-352 (`terminate`, `kill_group`, `group_signal_outcome`). Mutant: the `EPERM if leader_exited` arm disabled (`leader_exited && false`), then `cargo test -p raf-workplan --lib exec::system::` | **The logic is correct.** `killpg` runs first, then `try_wait`, so the signal goes out before anything is reaped. `EPERM` with an exited leader is now `Ok`; `EPERM` with a live leader and every other errno still go to `abandon` ("may still be running"). `try_wait` returning `Err` makes `leader_exited` false, so it fails closed. A reaped child keeps its status in `Child`, so the later `child.kill()` and `child.wait()` in `terminate` are unchanged (MSRV 1.98.1). The mutant dies: `a_refused_signal_is_done_only_when_the_leader_has_exited` FAILED, 2 passed | FIXED (code) |
| 2 | N1 step 2: "Add a test that makes the writer exit before the kill, and keep the existing test, which test-macos runs" | Read the three new tests. Read `.github/workflows/ci.yaml` 126-151 and the test-macos job log. Mutants in `/tmp/x10e` (`/tmp/x10e-scripts/mutant.sh`): `leader_exited` replaced by `true` and by `false`, then `--lib exec::`, `--test exec --test bound` and `cargo-workplan --test evidence` | **Not met.** (a) The three tests are a truth table of `group_signal_outcome` that is given the boolean directly; no test starts a writer that exits before the kill. (b) None of the three runs on macOS, the only platform where the `EPERM` arm matters: test-macos runs `cargo test -p raf-workplan --lib bound::` (69 passed, 154 filtered out), and `exec::system::tests` appears only in the Linux `test` job. (c) Both mutants survive every test here: `true` gives 13 + 19 + 43 + 12 passed, `false` the same. The `false` mutant is the pre-fix behaviour, so on Linux nothing tells the fix from no fix. On macOS only the racy `output_the_tool_cannot_read_is_exit_two_and_writes_no_record` can catch it, and the previous report warned that a race passing once proves nothing. That test is kept and passed on test-macos at this head (log line 1369) | OPEN (required) |
| 3 | N1 step 3: re-record `gate-kills-a-test-past-its-timeout`, re-assess the stale sections, `assess --list` all current, stale probe clean | `git diff 4dcdebc..HEAD -- work/evidence`. `evidence for cap:workplan-complete-gate`. The stale probe (criterion 7). A re-record experiment in `/tmp/x10e`: `evidence record --update --id gate-kills-a-test-past-its-timeout --for cap:workplan-complete-gate --by agent --copyright ... --licence CC-BY-4.0 --from-binding raf_workplan::test_crate::bound::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran`, then a 7-section probe of complete-gate | **Not done.** The evidence record was not touched by the delta and still holds `raf_workplan::exec` digest `78d5c919...`, while the module is now `82381609...`. The four assessments were updated; the evidence was not. `evidence for` still prints `pass`, because staleness is an obligation and not a `check` finding. The experiment re-records with exit 0, `recorded: ... (pass)`, and the record diff is exactly that one digest line. After it, the probe drops `evidence-stale` but gains `assessment-stale` for `cap:workplan-complete-gate#contract` ("the evidence record `evidence:gate-kills-a-test-past-its-timeout` has changed since"), because that assessment has the evidence record as an input. So the fix is: re-record, then re-assess `#contract`, then probe again | OPEN (required) |
| 4 | Hosted CI 12/12 green on the head | `gh pr checks 20`; `gh run view 39038479309` | Exit 0. All 12 pass: clippy, cross-check, deny, doc, fmt, fuzz, msrv, reuse, test, test-macos (6m10s), test-release and test-windows. Run 39038479309, event `pull_request`, attempt 1, `headSha` 035157108ea99fc06bf4733c93ce4f423743c0b2 = `git rev-parse HEAD` | PASS at 0351571; the fixes for 2, 3 and 9 need a new commit, and CI must pass again on that head |
| 5 | `cargo run -q -p cargo-workplan -- workplan check` exits 0 | Run in the issue worktree | Exit 0, no output (12 s) | PASS |
| 6 | `assess --list`: nothing stale | Run in the issue worktree | Exit 0. 56 records, all `accepted agent current`. `assess --list` covers assessments only, so the stale evidence of criterion 3 does not show here | PASS (assessments only) |
| 7 | Stale probe clean on all 12 capabilities | `/tmp/x10e-scripts/stale.sh` over the 12 delivered capabilities in `/tmp/x10e`. Each capability gets a fresh `reset --hard` to 0351571, `change new stale-probe-<cap>` affecting all seven sections, `git add`, then `change reconcile` | Every `change new` exits 0. evidence-claims: reconcile exit 0, no output. The other 11 exit 1, with only probe artifacts plus one real line: `section-untouched` 29, `assessment-missing` 28, `generated-stale` 5 (the relationships blocks of cli, complete-gate, document-model, generated-docs and reconcile, which name the probe record). The real line is **`cap:workplan-complete-gate  document  evidence-stale  ... (evidence:gate-kills-a-test-past-its-timeout was made against raf_workplan::exec, which has changed since)`**. There are 0 `assessment-stale`, `assessment-rejected`, `evidence-changed` and `claim-unsupported` lines | FAIL |
| 8 | `cargo test -p raf-workplan --test roadmap`; `cargo test --test docs_contract --test repo_gates` | Run in the issue worktree with `--locked` | roadmap: 15 passed, exit 0. docs_contract 28 passed and repo_gates 57 passed (that is `make issue-refs`), exit 0 | PASS |
| 9 | PR body meets the P1 acceptance (remediation plan Revision 2) | `gh pr view 20 --json body`, read first and re-read at the end (`/tmp/x10e-scripts/prbody.sh`) | See "PR body" below | FAIL |

Further delta checks (all pass):

* At HEAD: `cargo test -p raf-workplan --lib exec::system::` gives 3 passed. The kept CLI test
  `output_the_tool_cannot_read_is_exit_two_and_writes_no_record` gives 1 passed.
* `coverage tests` and `coverage code` for `--crate raf-workplan` exit 0 with no gap row, so the
  three new tests are bound.
* `make fmt` and `make reuse` exit 0 (485/485 files).
* `.deficiencies` (worktree and root) holds the notice and marker only.
* The commit is authored and committed by `Patrick_Audley`, with no trailer and no agent credit.
* No delivered prose changed: only bindings and the generated table. The rule "reopen only if the
  text changes" therefore holds, and `status` shows delivered 12, implementing 0, designed 9.
* `verify` of complete-gate through a scratch record in `/tmp/x10e` (`/tmp/x10e-scripts/verify.sh`):
  * exit 0 in 420 s, with 208 bindings, all `pass`: 206 tests and 2 examples, which is the
    earlier 203 + 2 plus the three new tests;
  * each new binding is `1 listed, 1 passed, 0 failed, 0 ignored`.

## PR body

Read at the start: updated `2026-10-17T13:54:12Z`, 37 lines, sha256 `d693537f...`. The P1
acceptance (Revision 2) requires the following, and the body fails each point:

* **No 'has not run', 'none has run' or 'never run'.** All three are present:
  * line 25: "none has run on this branch";
  * line 33: "Hosted CI has not run on this branch";
  * line 17: "The stored `command` is never run".
* **Head SHA equals `git rev-parse HEAD`.** No SHA is present.
* **Current hosted CI result.** The body says the opposite of the green run.
* **One row per rewording.** There are no rows (counted in validation 11A-11E, plus the
  remediation rows). Instead the body points to "the stage validation notes" (lines 22 and 23),
  which is also a process reference in a forge text.
* **Stale or false statements:**
  * "54 assessments" and "54 of 54" (there are 56);
  * "A kill for too much output records zero time" (false since G5);
  * "The full-repository `verify` with no slug was not run as a whole".
* **Missing items:**
  * the whole-repository `verify` result;
  * the rebinds;
  * the nominal dependency reading;
  * the id-rule item;
  * the record-item rewordings of all 12 records;
  * the remediation rewordings, including C2-B fixes 2 and 3 (advisory A2 of the previous report);
  * the hook-bypass slip;
  * the consciously accepted list with reasons;
  * the roadmap-guard note;
  * the squash-merge history note (advisory A3).

The body contains no literal `.stage`, no `Co-Authored` and no agent credit.

Re-read at the end of the audit, after `verify`: the body is byte-identical (`cmp` equal, the
same sha256 `d693537f...`, `updatedAt` still `2026-10-17T13:54:12Z`). It has not been rewritten,
and every failure above stands. HEAD and the remote branch are still 0351571, and the worktree is
clean.

## Advisory, not blocking

* AD1: on macOS, a group whose leader has exited but which still holds a live member that cannot
  be signalled (for example a setuid descendant) now counts as killed. Before the fix, the same
  case was reported as "may still be running". This is the same as on Linux, where `kill(-pgid)`
  succeeds if any member was signalled. It is within the trusted-command scope that SECURITY.md
  already states (`setsid` escapes the kill). If you want to keep it, state it in the SECURITY.md
  paragraph on the kill.
* AD2: the `leader_exited = true` mutant (every `EPERM` counts as done) cannot be killed by an
  unprivileged test, because `EPERM` with a live leader needs a process of another user. That is
  acceptable. The `false` mutant (the pre-fix behaviour) can be killed, and REQUIRED 2 is about
  that one.

## Result

* The N1 code change is correct, its decision table is pinned (the `EPERM`-arm mutant dies), and
  hosted CI is 12/12 green on 0351571, test-macos included.
* Criteria 5, 6 and 8 pass.
* Three things are open:
  * N1 step 3 was not done: the cited evidence record is stale, so the stale probe is not clean.
  * N1 step 2's test does not exist: no test makes the writer exit before the kill, none of the
    new tests runs on macOS, and the pre-fix behaviour survives every test that runs here.
  * The PR body fails the P1 acceptance.

VERDICT: BLOCKED

REQUIRED fixes:

1. **Stale evidence (criteria 3 and 7).**
   * Re-record `evidence:gate-kills-a-test-past-its-timeout` with `evidence record --update
     --from-binding raf_workplan::test_crate::bound::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran
     --for cap:workplan-complete-gate --by agent`, keeping its two SPDX lines through
     `--copyright` and `--licence`. Without them the SPDX lines are dropped, as observed.
   * Then re-assess `cap:workplan-complete-gate#contract`, which goes stale because the evidence
     record is one of its inputs.
   * Then run the 12-capability probe again, until it has 0 `evidence-stale`, `assessment-stale`,
     `assessment-rejected`, `evidence-changed` and `claim-unsupported` lines, and `assess --list`
     is all current.
2. **The N1 test that was asked for (criterion 2).**
   * Add a deterministic test that starts a child in its own process group, lets it exit without
     reaping it (the writer exits before the kill), and asserts that `kill_group` or `terminate`
     returns `Ok`.
   * Make test-macos run it, for example with a test-macos step for `--lib exec::system::`, or by
     placing the test where an existing test-macos step runs it.
   * The test must fail on macOS with the `leader_exited = false` mutant.
   * Bind it in complete-gate, then repeat fix 1 for the digests it moves.
3. **PR body (criterion 9).** Rewrite it after the final push so that it meets every point of the
   P1 acceptance listed above, with the head SHA equal to the final `git rev-parse HEAD`.
4. **Hosted CI.** Get all 12 jobs green again on the final head, test-macos included.
