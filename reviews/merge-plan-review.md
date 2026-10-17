# Merge-plan review: PR 20 (issue 10)

Reviewed:

* `validation.md` (Stage 3 integration assessment and "Stage 3 delta at the branch tip")
* `review-debt.md`
* `reviews/completion-audit-exit-3.md`
* `raw/threads.json`, `raw/pr-checks.txt`, `pr-checks.txt` and `pr-comments.md`
* `/tmp/verify-full.txt`
* the squash notes `.worktrees/10-m2-rest-of-the-planning-essay-entity.squash_notes`
* the repository rules (`CLAUDE.md` and `AGENTS.md` in the worktree)

Method: I only read files. I ran no tests, no timing runs and no git commands.

## Candidate

* Branch ref and `origin` ref: `09954770dfafbf0ae04f3b67991437ac39fcda9d`.
* main equals the merge base, so the candidate is `0995477`.
* Exit audit 3 ran at `62709dd`, one commit earlier. `0995477` replaces the 300 ms sleep with `waitid(EXITED | NOWAIT)` (`exec/system.rs:341-359`), which discharges audit advisory A1.

## Recheck of the first-pass findings

**B1: DISCHARGED.** The tip was one commit past the audit, and five records depend on the changed module.

`validation.md:2102-2104` now states the tip delta:

* the commit re-records the four complete-gate assessments and the timeout-kill evidence record;
* at the tip, `assess --list` shows 56 of 56 current;
* `evidence for cap:workplan-complete-gate` passes;
* `check` exits 0.

The squash notes (lines 23-27) say the same thing.

**B2: DISCHARGED (rebuttal accepted).** The first pass claimed that the 1486/1486 `verify` result was not shown.

`/tmp/verify-full.txt` holds:

* 1486 `pass` lines;
* no `fail` line;
* the last line `exit 0 seconds 1109`.

According to the coordinator, the file was written after the audited commit and before the tip commit. The finding was wrong: this full run was not among the inputs I had. The notes now place the run on "that same tree", one commit before the tip.

**B3: DISCHARGED.** The audit and probe results were given without their commit.

The notes now say:

* "The stale probe over all twelve capabilities was clean one commit earlier";
* `verify` ran "on that same tree";
* "The audit ran one commit before the tip";
* the tip results are named as such.

**B4: DISCHARGED.** The new no-timing rule conflicted with wall-clock tests that this branch adds.

The notes now disclose the conflict: "tests of this branch that still assert wall-clock time are tracked for conversion in issue 21 (owner decision: merge, then convert)". This moves the scope to a tracked change, which the "No deferrals" rule in `CLAUDE.md` allows, on the owner's own decision.

**A2: DISCHARGED.** The merge method was not stated.

The notes now say: "a squash keeps them only in this list". This states plainly that the reopening records survive a squash only in the message.

**A1: STILL-OPEN (advisory).** The snapshots do not tie CI to the tip.

* `raw/threads.json` still says `comments.totalCount: 2`, while `pr-comments.md` holds 4 comments.
* `raw/pr-checks.txt` is still run `38997274163`.
* `pr-checks.txt` and `validation.md` name no run id or `headSha` for the 12/12 at `0995477`.

The 12/12 on the tip rests on the caller's `gh pr checks 20` alone.

REWRITE TO (validation.md, Stage 3 delta): "Hosted CI run <id>, `headSha` `09954770d`, passes 12 of 12."

**A3: STILL-OPEN (advisory).** Notes lines 35-36 still say: "The authoritative plan is in the stage directory `.stage/m2-rest-of-the-planning-essay-entity`." That directory is local and will not exist on main, so the pointer in a permanent commit message will not resolve.

REWRITE TO: delete the sentence, or "The plan is in the description of the pull request."

## New text reviewed

* Notes lines 23-36 match `validation.md:2104` and `/tmp/verify-full.txt`. I found no new claim the evidence does not support.
* **ADVISORY (new):** `validation.md` cites the `verify` log but not its path.

  REWRITE TO (validation.md:2104): "The whole-repository `verify` log (`/tmp/verify-full.txt`: 1486 pass, 0 fail, `exit 0`) was taken on the tree before that commit."

* The Stage 3 paragraph at `validation.md:2100` still says the only change since the audit is "a test-only edit". The delta section below it corrects this: the same commit also re-recorded five records. The paragraph is not false, but it is incomplete.

None of the remaining items blocks the merge.

VERDICT: PASS
