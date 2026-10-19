# Merge-plan review: PR 30, issue 21 (recheck after 1e5dc74)

Reviewed: `../21-replace-wall-clock-assertions-in-tests.squash_notes` (21 commits, head 1e5dc74) against the worktree files, validation.md, gap-analysis.md, pr-body.md, review-debt.md and issue.md.

Limits of this review: this session had no shell in either pass, so I did not run `git show 1e5dc74`, `gh`, `git rev-list` or a mutant clone. I read the fix in the worktree files. CI being 12 of 12 on 1e5dc74 (including test-macos) is the coordinator's statement; I could not confirm it. No mutant of the EPERM handling has been run by anyone.

## Recheck of prior findings

**B1 (BLOCKING): DISCHARGED.**
* `kill_group` (system.rs:367-369) now reads:

      let leader_ended = cfg!(target_os = "macos")
          && signalled == Err(rustix::io::Errno::PERM)
          && leader_ended(child);

* Off macOS, `leader_ended` is `false`, `leader_ended(child)` is never called, and `group_signal_outcome` sends EPERM to `Err(errno.into())`. A refused group kill on Linux is therefore reported again, as it was before 0851241.
* `terminate` is unchanged and cannot block forever. On macOS, `leader_ended` still waits only on a leader it has just sent SIGKILL to, or on one that answered ESRCH.
* The comment at system.rs:362-366, the capability text (workplan-complete-gate.md:664-670: "`EPERM` on any other system, is reported") and the squash note (lines 22-24) now say the exception is macOS-only.

What is left of the B1 rewrite does not block the merge:
* **Rewrite item 2, now ADVISORY A6:** the already-collected leader case. A leader that `wait` collected before the group signal still counts as ended on macOS, so a refused live member there is not reported. CLAUDE.md treats macOS as minimal-effort ("a platform-specific failure does not block a change"), and macOS cannot tell that case from the zombie case anyway. Consider stating this in the comment at system.rs:400-405, whose claim "the leader could be signalled, so it was already exiting" is not true in that branch.
* **Rewrite item 3, now ADVISORY A7:** the guard has no pin on Linux. The `cfg!` sits in `kill_group`, not in the pure `group_signal_outcome`, and no Linux test produces EPERM. A mutant that deletes `cfg!(target_os = "macos") &&` would pass on Linux. The behaviour itself is now correct; a pin would need the platform passed into the pure function.

**A1: STILL-OPEN (ADVISORY).** On Linux, no test reaches the EPERM branch with real processes. This is now partly covered by A7.

**A2: no action needed.** As stated above, `terminate` still cannot block forever.

**A3: DISCHARGED.** Squash note lines 16-17 name the raf-source examples carve-out.

**A4: PARTLY DISCHARGED.** The `#` comment block and the pointer to the stage directory are gone. Line 35 still says "Independent reviews of every task and a Stage 2 gap analysis found and closed the gaps above", which is process wording in a commit message. AGENTS.md section 7 governs files, so this is not a violation. REWRITE TO: delete that sentence.

New point: the note uses `## ` headers. If the merge tool commits with `--cleanup=strip`, git will drop them. Confirm that it uses whitespace cleanup or `--cleanup=verbatim`.

**A5: STILL-OPEN (ADVISORY).** The L1 content of issue 29 is unverified (no `gh`).

## New text

* **ADVISORY A8.** system.rs:403-405 wraps unevenly ("... (macOS only: `kill_group` passes `leader_ended` as\n/// false elsewhere). Any other\n/// failure ..."). rustfmt does not reflow comments. REWRITE TO: rewrap at the surrounding width.
* **ADVISORY A9.** Squash note lines 22-25 say "on macOS" twice and nest two parentheticals. REWRITE TO: "on macOS, a group kill refused with EPERM while the leader is still exiting is now completed through the leader; elsewhere EPERM is still reported as a failed kill (the tests that expose it moved under a job that runs on macOS)."
* The squash note's commit list (21) and subject are consistent with the fix commit.

## Gap-analysis items (unchanged from the first pass)

H1, M1, M2, M3, L2, L3, L4, L5 and L6 hold at HEAD. H1 holds with the macOS-only guard. L1 is unverified (A5).

## Integration

Main is unmoved (stated by the caller), and hosted CI is reported 12 of 12 on 1e5dc74. The squash note's claims match the evidence apart from the advisories above.

VERDICT: PASS
