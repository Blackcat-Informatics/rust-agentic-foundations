# Stage 3 merge-plan review: PR 13 (issue 2, raf-okf + raf-source)

Current status: VERDICT: PASS (Recheck at the end of this file).

Reviewed: `/home/paudley/Active/rust-agentic-foundations/.worktrees/2-shared-crates-raf-okf-raf-source.squash_notes`
(the final merge commit message), candidate head `5bcd741`, base `origin/main` `2c41082`.

Evidence read: `validation.md`, `review-debt.md`, `reviews/completion-audit-stage2-exit.md`
(the initial audit and its Recheck), `issue.md`, `plan.md`, `reviews/completion-adversary.md`,
`reviews/completion-audit-stage2.md` (decision list), every `tasks/*-review.md` (verdict and recheck
headings), `tasks/X2-review.md` in full, `pr-comments.md` (R5b to R7 posts), `squash-notes-draft.md`
(the draft header), `raw/registry-scan-R4.txt`, `raw/registry-scan-R5a.txt`, `raw/R7-gates/fuzz.log`,
`raw/make-check-final.log`, `raw/X2-gates/fuzz.txt`, `deny.toml`, `fuzz/deny.toml`,
`docs/capabilities/source-index.md` and `SECURITY.md` (for the quoted figures).

Limits of this review: I had no shell in this session, so I did not re-run `git merge-tree`, `gh` or
any gate. Integration facts for the live base come from the caller's statement and from the exit
audit's Recheck. The audit recorded run 38532696298 on `5bcd741` with all 12 jobs green.

## Focus 1: integration evidence for the actual candidate

The evidence is sufficient.
- Base: `origin/main` is `2c41082`, which is the branch base, so `git merge-tree --write-tree` exits 0
  (caller). `validation.md` records the same result at 44e18b7.
- Head `5bcd741`:
  - The exit audit's Recheck ran these fresh on `5bcd741`: `cargo test --workspace --locked`
    (851 passed), `make test-release clippy doc fmt` (479 release tests), `make fuzz`, and the
    registry harness (1213 of 1233 OK).
  - Hosted run 38532696298 passed all 12 jobs, including cross-check, test-macos and test-windows.
  - `git diff bf08e45 5bcd741` touches only the X2 files. The full `make -k check` on bf08e45
    therefore still holds for everything else.

The index files are behind this evidence:
- `validation.md` still ends at "Review and commit follow" for X2. It does not mention run
  38532696298 or a merge-tree check at `5bcd741`.
- `review-debt.md` is keyed to head `bf08e45`.

Neither gap blocks the merge, because the audit Recheck holds the head evidence (see A-1).

## Focus 2: review and thread dispositions

The dispositions are honest. There are 0 forge reviews, 0 inline comments, 0 threads and 0 bot
comments, and the 33 owner workflow posts are marked informational (`review-debt.md`). Every
`tasks/*-review.md` ends in `VERDICT: PASS`, and every BLOCKED round is followed by a passing
recheck. Two counts in the notes are wrong, and some lines are duplicated (R-4, A-3).

## Findings: REQUIRED

### R-1 (BLOCKING): the provenance claim contradicts the recorded user decision

Notes line 29:
> - Port from the precedent implementations. Built: ideas and behaviour taken from the precedent sources; the AGPL copies were used for ideas only, never text. Known defects of the precedents each have a named negative fixture.

`plan.md:39-40` records the user decision differently: "Port source: the superset of every
precedent implementation, taking the best part of each, with licensing ignored (user decision)".
Section A then lists katamari and gmeow line ranges as start-from or removed sites.
`completion-audit-stage2.md:50-52` records it as decision 5, which "replaces the issue's provenance
clause". No evidence shows that no AGPL text was copied, and the decision expressly allowed it. The
notes also leave this decision out of "Decisions that differ from the issue text". A permanent
licensing claim that the record contradicts cannot go into `main`.

REWRITE TO:
> - Port from the precedent implementations. Built: by the owner's decision, the port source is "the superset of every precedent implementation, taking the best part of each, with licensing ignored" (see Decisions that differ); there is no purrdf dependency. Known defects of the precedents each have a named negative fixture.

Also add this as item 6 under "Stage 1 decisions (recorded 2026-10-12)":
> 6. "The superset of every precedent implementation, taking the best part of each, with licensing ignored." This replaces the issue's "Port from katamari ... with the `Tree` trait from purrdf" provenance clause.

### R-2 (BLOCKING): the merge-scope decision date and deadline are wrong

Notes line 80:
> - Merge scope and time cap (2026-10-14 22:25; question: "Which scope do you want for merging issue 2?"): **"Everythign - you have 6 hours - capped"**. ... with the hard deadline 04:25 on 2026-10-15.

`validation.md:239,247` gives "(2026-10-13 22:25)" and "The hard deadline is 04:25 on
2026-10-14". The exit audit's advisory A-2 already called the 2026-10-14 date impossible, and it is
still in the notes. A decision dated 2026-10-14 22:25 would also come after the 2026-10-14 01:43
decision that depends on it.

REWRITE TO:
> - Merge scope and time cap (2026-10-13 22:25; question: "Which scope do you want for merging issue 2?"): **"Everythign - you have 6 hours - capped"**. The glob budget, resolve-path validation, API sealing and the licence-pin and contract-text work all stay in this PR, with the hard deadline 04:25 on 2026-10-14.

The heading at line 72, "Owner decisions (2026-10-12, verbatim answers):", should become
"Owner decisions (verbatim answers; 2026-10-12 unless dated):".

### R-3 (BLOCKING): the head-gate figure "477/477" matches no record

Notes line 116:
> On the merge head 5bcd741, which adds only the verbatim-item fix, the per-item gates pass: raf-source 477/477 in debug and release, clippy, rustdoc, docs_contract 141, repo_gates 57, fmt.

The recorded counts on `5bcd741` are different:
- `tasks/X2-review.md:139`: "`cargo test -p raf-source` (debug) 478 passed, 0 failed".
- `completion-audit-stage2-exit.md:266-268`:
  - `cargo test --workspace --locked` 851 passed, 0 failed, 1 ignored;
  - `make test-release clippy doc fmt` EXIT 0 with 479 release tests;
  - `make fuzz` EXIT 0.

No file in the stage directory contains "477".

REWRITE TO:
> On the merge head 5bcd741, which adds only the verbatim-item fix, fresh runs pass: `cargo test --workspace --locked` 851 passed, 0 failed, 1 ignored (docs_contract 141, repo_gates 57); `make test-release clippy doc fmt` exit 0 with 479 release tests; `make fuzz` exit 0 on all three targets.

### R-4 (BLOCKING): R5b recheck count

Notes line 168:
> - R5b (import settling by rustc's rules): PASS after four rechecks; ...

`tasks/R5b-review.md` has "## Recheck", "## Recheck 2" and "## Recheck 3", and its final verdict
is under Recheck 3 (line 724). `validation.md:231` agrees: "Recheck 3, VERDICT: PASS". The
"four" was copied from `pr-comments.md:1248`.

REWRITE TO:
> - R5b (import settling by rustc's rules): PASS after three rechecks; ...

### R-5 (BLOCKING): the fuzz coverage figure is not in the evidence

Notes line 140:
> Coverage of the workspace-load target rose from 4,501 to 11,772 after fixing `--cfg fuzzing` stripping span locations.

`tasks/R7-review.md:86` records "`load-workspace` INITED cov 8507 (was 4501)", and
`raw/R7-gates/fuzz.log:1464` shows `INITED cov: 8507`. The final gate log
`raw/make-check-final.log:3376` (load-workspace run on bf08e45) shows `INITED cov: 13212`.
11,772 appears in no review and no raw log. It appears only in the orchestrator's
`pr-comments.md:1265`.

REWRITE TO:
> Coverage of the workspace-load target at corpus load rose from 4,501 to 8,507 after fixing `--cfg fuzzing` stripping span locations, and is 13,212 in the final `make -k check` on bf08e45.

### R-6 (BLOCKING): an unresolved draft note about registry counts

Notes line 134:
> - After the alias fix: 1,177 of 1,207 load (the fix record states 1,183 of 1,213); ...

This parenthetical is the draft's own reminder (`squash-notes-draft.md:9`: "The registry scan
counts differ between two stage records ... Re-confirm on the final scan"), left in the final text.
The raw scan settles it: `raw/registry-scan-R5a.txt` has 1,213 OK/ERR lines, 1,183 `| OK` lines and
the footer `# ok=1183`. The figure "1,177 of 1,207" in `validation.md:169` does not match its own
raw file.

REWRITE TO:
> - After the alias fix: 1,183 of 1,213 load (the local registry grew by six crates since the previous scan); `UnresolvedTrait` 19 to 5, ...

Correct `validation.md:169` to match (A-1).

### R-7 (BLOCKING): branch-size figure is stale and disagrees with the stat in the same message

Notes line 107:
> Branch size at drafting: 221 files, about 52.7k insertions, 1.9k deletions against `origin/main`.

The `## Files` block in the same notes says "224 files changed, 56220 insertions(+), 1890
deletions(-)". The exit audit gives 223 files, +55599/-1889 at bf08e45, which is one commit short of
the head.

REWRITE TO:
> Branch size: 224 files, 56,220 insertions, 1,890 deletions against `origin/main`.

### R-8 (BLOCKING): the headline presents a stated limit as delivered

Notes line 10:
> ... Every read stays inside the canonical root, and every load is bounded by typed budgets.

The notes' own Accepted known limits contradict this (lines 87 and 92): "The import work queue and
the graph of undecided imports ... are not budgeted", and "loading has no time bound". The Contract
says the same at `source-index.md:696-697`: 200 such modules take 613 s. Focus (4) does not allow a
limit to be presented as delivered.

REWRITE TO:
> ... Every read stays inside the canonical root, and loads are bounded by typed budgets on bytes, tokens, files, directory depth and module walks; the settle work for undecided imports and load time are not bounded (see Accepted known limits).

## Findings: ADVISORY

- **A-1. The evidence index is behind the head.** `validation.md` should gain a final section for
  `5bcd741`:
  - the X2 review PASS after recheck;
  - the audit Recheck's fresh runs;
  - hosted run 38532696298, all 12 jobs;
  - `git merge-tree --write-tree origin/main 5bcd741` exit 0;
  - the corrected R5a registry figure (R-6).

  `review-debt.md` is keyed to `bf08e45` and itself says "Re-confirm at merge time". Re-fetch the
  reviews, threads and checks for `5bcd741` immediately before the merge.
- **A-2. Some figures trace only to orchestrator posts, not to `validation.md` or a review file.**
  They are "Later full `make check` on 51d84e5 passed with 1,112 tests" (line 113), "web-sys loads
  in about 5 s at 144 MiB" (line 135), and "the earlier code aborted 39 times" with "39 shape
  families" (lines 54 and 141). Nothing contradicts them, but none is indexed. Either index them in
  `validation.md` with their source logs, or drop them.
- **A-3. The Review dispositions list repeats lines and misstates two rechecks.**
  - R11a appears twice (lines 174 and 181), and R13 twice (lines 177 and 182). Merge them into one
    line each, and give R11b a description.
  - R6 (line 169) and R7 (line 170) say "PASS". Both files have a "## Recheck" section, and R7's
    first pass is marked "Superseded by the Recheck below". Both should read "PASS after recheck",
    as line 160 implies.
- **A-4. The `raf-source` src list in "Files and purpose" (line 98) is incomplete.** It leaves out
  `cfg.rs`, `intern.rs`, `path.rs` and `verbatim.rs`, all of which the stat lists. `verbatim.rs`
  is the X2 fix.
- **A-5. Heading levels.** "### Design in brief" is followed by "### raf-okf" and "### raf-source"
  at the same level. Make the two sub-headings `####`.

## Checks that pass

- **User-decision quotes match `validation.md` verbatim:**
  - A1, A2 and G9;
  - the sign-offs;
  - "(1) - default to 2M" with its question;
  - "Add rustix + windows-sys" with its question;
  - "Everythign - you have 6 hours - capped" (typo kept);
  - "make an issue for it and move on.".

  Stage 1 decisions 1 to 5 match `plan.md`. Completion adversary F-1 to F-8 being integrated
  matches `plan.md:429`.
- **Licence pins:** `unicode-ident@=1.0.27` is pinned in `deny.toml` and `fuzz/deny.toml`, and
  `libfuzzer-sys@=0.4.13` in `fuzz/deny.toml`.
- **Contract figures match `source-index.md` and `SECURITY.md`:**
  - 1664 bytes per token, about 3.1 GiB;
  - 2623 MiB allocated and 1188 MiB resident;
  - 1054 and 1059 MiB, 1.5 GiB and 4.6 GiB;
  - a recursion limit of 1927;
  - 20 to 32 s at 87 MiB;
  - 4096 manifest entries and 262144 module walks.
- **Hosted CI runs and heads match `validation.md` and the audit:** 38280511329, 38308785911,
  38324353179, 38361953775, 38499493700, 38500276876, 38501515159 and 38532696298. The 12-job list
  for 38532696298 is complete.
- **Other counts match their records:**
  - local gates: 963 tests on 6c744af; 86 suites and 1,304 tests on bf08e45;
  - registry: baseline 1,014 and 164; R2 165 and 2,757, and 1,021; R4 1,163 of 1,207; R5c 1,204
    of 1,233; final 1,213 of 1,233;
  - race evidence: 65,536 schedules and 200k races (`tasks/R8-review.md`);
  - rustc oracle: 12 queries in 10 of 5,522 programs, and 96 in 71.
- **Commits:** 38 are listed, which agrees with the audit's 37 at bf08e45 plus `5bcd741`.
- **Issue requirements:** every scope bullet and acceptance criterion is mapped. Two deviate, and
  both are covered by recorded decisions: `resource` and `tags`/`timestamp` (decision 1), and the
  `crates/` default (decision 2). The provenance clause needs R-1. Apart from R-8, no limit is
  presented as delivered. The visibility rules are tracked in a separate issue by the owner's
  verbatim decision.
- **Hygiene:**
  - no agent attribution, co-author trailer, agent e-mail address or "generated" credit;
  - no `GhpRsq-` trailer;
  - no `PENDING:`, `DRAFT`, `TODO` or template-brace markers (the draft header was removed);
  - the only trailer is `Defect-Class: none`.

## Verdict (first pass)

Integration evidence and review dispositions are sufficient. The commit message has eight required
corrections:
- R-1 to R-7: the provenance claim, a decision date, four figures with no record behind them or
  contradicted by one, and a leftover draft note;
- R-8: one limit presented as delivered.

Each has an exact rewrite above.

First-pass verdict: NEEDS-REVISION (superseded by the Recheck below).

## Recheck

I re-read the revised squash notes in full and re-verified every changed line against the evidence.
I also grepped the notes for each phrase I had objected to: "ideas only", "four rechecks", "477",
"11,772", "221 files", "2026-10-14 22:25", "2026-10-15" and "every load is bounded". None remains.
The one hit for "477" is the diffstat line count for `scalar.rs`, which is correct.

### Prior findings

- **R-1: DISCHARGED.**
  - Line 29 now carries the REWRITE TO text: the owner's "superset … with licensing ignored"
    decision, and no purrdf dependency.
  - Stage 1 decision 6 is added at line 71. It quotes `plan.md:66-67` and names the provenance
    clause it replaces.
- **R-2: DISCHARGED.**
  - The heading at line 73 reads "Owner decisions (verbatim answers; 2026-10-12 unless dated)".
  - Line 81 gives 2026-10-13 22:25 and the deadline 04:25 on 2026-10-14.
  - Both match `validation.md:239,247`.
- **R-3: DISCHARGED.** Line 117 gives 851 passed, 0 failed, 1 ignored, then docs_contract 141,
  repo_gates 57, 479 release tests, and `make fuzz` exit 0. This matches
  `completion-audit-stage2-exit.md:266-268`.
- **R-4: DISCHARGED.** Line 169 says "three rechecks", which matches `tasks/R5b-review.md`
  (Recheck, Recheck 2 and Recheck 3).
- **R-5: DISCHARGED.** Line 141 gives 4,501 to 8,507, and 13,212 on bf08e45. These match
  `tasks/R7-review.md:86`, `raw/R7-gates/fuzz.log:1464` and `raw/make-check-final.log:3376`.
- **R-6: DISCHARGED.**
  - Line 135 gives 1,183 of 1,213 and explains the registry growth. This matches
    `raw/registry-scan-R5a.txt` (`# ok=1183`, 1,213 result lines).
  - `validation.md:169` now reads "1,163 → 1,183 (the local registry grew from 1,207 to 1,213
    crates between the scans)".
- **R-7: DISCHARGED.** Line 108 gives 224 files, 56,220 insertions and 1,890 deletions, which
  matches the `## Files` stat.
- **R-8: DISCHARGED.** Line 10 now names the budgets and states that the settle work and load time
  are not bounded. This is consistent with lines 88 and 93 and with `source-index.md:696-697`.

### Advisories

- **A-3: DISCHARGED.**
  - The duplicate R11a and R13 lines are gone.
  - R11b is described at line 180 as "sealed public types, Target test settings, half-open line
    ranges". That matches the scope of `tasks/R11b-review.md` (`#[non_exhaustive]` seals, the
    harness, test, doctest and required-features settings, and `SpanFact::lines` as a half-open
    `Range`).
  - R6 and R7 read "PASS after recheck".
- **A-5: DISCHARGED.** The crate sub-headings are now `####`.
- **A-1: STILL-OPEN (advisory).** `validation.md` still has no `5bcd741` section with run
  38532696298 and the merge-tree check. `review-debt.md` is keyed to `bf08e45`. Re-fetch the forge
  review and check state at merge time.
- **A-2: STILL-OPEN (advisory).**
  - Partly settled: the committed test pins both overflow figures. `overflow_harness.rs:118` has
    `assert_eq!(judged.len(), 35)` and `:144` has `assert_eq!(families.len(), 39)`. The "35
    committed shapes" and "39 shape families" in the notes are therefore supported by code. (The
    R6 recheck saw 36 families; the suite has since grown to 39.)
  - Still indexed only in orchestrator posts: "1,112 tests" on 51d84e5, web-sys at "5 s at
    144 MiB", and "aborted 39 times".
- **A-4: STILL-OPEN (advisory).** The `raf-source` src list at line 99 still leaves out `cfg.rs`,
  `intern.rs`, `path.rs` and `verbatim.rs`.
- **Minor (advisory).** `validation.md:169` still has the parenthetical "(1,207 crates)" after the
  file names, beside the corrected "1,207 to 1,213" text. This is internal evidence, not the
  commit message.

### New text reviewed

- **R6, line 170:** "a 39-family regression suite covers the soundness property". This matches the
  committed `families.len() == 39` assertion and the R6 recheck ("REQUIRED 1 is resolved by the
  committed … boundary test").
- **R7, line 171:** "`--no-cfg-fuzzing` fixed a target that never reached resolution". This
  matches `tasks/R7-review.md:32,51,81`.
- **R11b, line 180:** accurate, as above.
- **Hygiene, re-checked:**
  - no attribution, co-author trailer or agent e-mail address;
  - no `GhpRsq-` trailer;
  - no `PENDING:`, `DRAFT` or `TODO` markers.

No new inaccuracy was found. All eight required findings are discharged. The open advisories do
not block the merge.

VERDICT: PASS
