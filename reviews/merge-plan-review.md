# Merge-plan review: PR 32 / issue 29

Reviewed: `/home/paudley/Active/rust-agentic-foundations/.worktrees/29-index-the-coverage-product-instead-of.squash_notes`
(final squash notes), against `validation.md`, `gap-analysis.md`, `review-debt.md`,
`plan.md` (Rev 2), `tasks/T1-review.md`, `pr-body.md`, `issue.md`, AGENTS.md, CLAUDE.md,
the Makefile, `crates/raf-workplan/src/coverage.rs` and `work.rs` at HEAD `9ac04cf`, and the
git refs and reflogs under `.git/`. Read-only; no shell; no timing runs.

Pass 1: BLOCKED (B1, B2). Pass 2 (recheck of the revised notes): see the end of this file.

## Coverage (pass 1)

| Question | Result |
|---|---|
| Squash-note claims backed by evidence | Mostly yes. Two exceptions: the toolchain disclosure (B1) and the reason given for the odd-colon divergence (B2) |
| Integration evidence | Enough. See below. The caller's branch point `679ca0a` is wrong; the real one is `c2f49bb` |
| Issue 29 / plan R1-R4 | All delivered, none silently missing |
| Gap-analysis HIGH (Windows target) | Disclosed honestly in `pr-body.md`. In the squash notes it is softened (B1) |
| Numbers / process wording | Only `Closes #29`. Process wording in the Machine-limit paragraph (part of B1) and possibly in the trailing sections (A2) |
| Hosted CI | 12 of 12 pass on `9ac04cf` (`review-debt.md:25-27`; `validation.md:53`). No review debt |

### Integration evidence

- `.git/refs/heads/main` = `.git/refs/remotes/origin/main` = `c2f49bb3e71c...`. This matches
  `.git/refs/ghprsq/pr/31/result`, and `FETCH_HEAD` records remote `main` at `c2f49bb`.
- The branch reflog (`.git/logs/refs/heads/paudley/29-...`, line 1) says "Created from
  c2f49bb3e71c...". After that it has only `1f7a652` and then `9ac04cf`, so the history is
  linear on top of main.
- `679ca0ae2f30...` is the old tip of main, before PR 31 merged
  (`.git/logs/refs/remotes/origin/main` lines 9-10; `ghprsq/pr/31/base`). It is not this
  branch's base.
- So "main has not moved since the branch point" is TRUE for `c2f49bb`. The squashed tree
  equals the `9ac04cf` tree that hosted CI passed on.

### Requirements (issue text plus plan R1-R4)

- R1: the index is built once (`coverage.rs:789-845`, `:1121`), with exact counts.
- R2: growth tests (b) and (c) fail against the product code (`validation.md` M7) and
  against the index built per entry (M5). Independently reproduced (`T1-review.md` section 2).
- R3: every matching exclusion is marked, not only the first (`coverage.rs:839-845`). Checked
  by mutants M1 and M6 and by the cross product.
- R4: `tests/coverage.rs` has an empty diff, and the coverage JSON is byte-identical.

The issue asks for nothing beyond this.

### Squash-note claims checked (pass 1)

| Claim | Evidence | Verdict |
|---|---|---|
| `ExclusionIndex`: two hash maps plus flags; one lookup per path and per proper ancestor | `coverage.rs:789-834` | TRUE |
| Every match marked, smallest position gives the reason; no lookups with no exclusions | `coverage.rs:821-823`, `:843`; M1, M4, M6; mutant MA | TRUE |
| Old matching functions gone from production; oracle in the test module | grep of `src/`; `tests.rs:21` | TRUE |
| Three work kinds ticked at the insert, lookup and mark sites | `coverage.rs:802`, `:826`, `:829`, `:841`; `work.rs:46-51` | TRUE |
| 240 exclusions over 120 paths; first match and full vector; rebuilt index | `gap-analysis.md:19`, `:69` | TRUE |
| Out-of-range mark is a panic | `coverage.rs:842` | TRUE |
| Local gate list exit 0 | `validation.md:51` (Handoff, head 9ac04cf) | TRUE |
| Hosted CI green including cross-check, macOS and Windows | `review-debt.md:27`; `validation.md:53` | TRUE |
| Seven named mutants each fail a named test | `validation.md:15-23` | TRUE |
| Odd-colon divergence, "which the record loader does not admit" | `T1-review.md:29-37` | PARTLY: B2 |
| Declined items | `plan.md:95-100`; `T1-review.md:103-104` | TRUE |
| Machine limit / Windows std | `validation.md:53`; `gap-analysis.md:78-105` | INCOMPLETE: B1 |

## Findings (pass 1)

- **B1 BLOCKING.** The Machine-limit paragraph described the Windows standard library as a
  standing property of the host. It did not say the addition was unauthorised or that the
  hand-run Windows check depends on it, and it used process wording ("the workflow during the
  investigation"). `pr-body.md:18` stated all of this in full.
- **B2 BLOCKING.** The note credited the record loader with excluding odd colon runs. For the
  entry-path case (`a:::b`), what rules it out is Rust path syntax (`T1-review.md:29-37`).
- **A1 ADVISORY.** Re-confirm the base with `git fetch` and `git merge-base --is-ancestor`
  before merge.
- **A2 ADVISORY.** "final head", "Commits squashed" and "Conflicts" refer to the branch
  (AGENTS.md:282-283). Drop them if they land in the commit message.
- **A3 ADVISORY.** The plan's issue-update step (`plan.md:63`) has not been done.

## Pass 2 (recheck of the revised squash notes)

Only the text changed by the fixes was reviewed.

- **B1: DISCHARGED.** `.squash_notes:36-41` now says the host "selects one dated nightly
  through its own configuration". It no longer presents the Windows library as a host
  property. It now states that the library is there "only because it was added there once",
  that adding it "is the machine owner's action and was not authorised", that "the Windows
  check of `raf-source` run by hand depends on it", that Apple and FreeBSD are covered "only by
  the hosted job", and that whether it stays "is the owner's decision". This matches
  `pr-body.md:18` on every point the gap-analysis HIGH asked for. "the workflow during the
  investigation" is gone.
- **B2: DISCHARGED.** `.squash_notes:28-29` now says "which Rust paths do not contain (entry
  paths come from the source index and exclusion paths from the record loader, which admits
  only identifier segments)". This matches both divergent cases in `T1-review.md:29-37`.
- **A1: DISCHARGED.** The coordinator reports that `git fetch` ran and
  `git merge-base --is-ancestor origin/main HEAD` returned true. The refs agree:
  `.git/refs/remotes/origin/main` and `FETCH_HEAD` both still record `c2f49bb`, the branch
  point.
- **A2: STILL-OPEN, advisory.** "final head" is removed (line 25 now reads "hosted CI green
  including cross-check, macOS and Windows"). "## Commits squashed (2)" and "## Conflicts" are
  still there. They are harmless if the squash tool strips them, and should be dropped if it
  does not.
- **A3: STILL-OPEN, advisory.** No evidence that the issue progress comment was posted. This
  does not affect the commit content.

New text, checked against the rules:

- Line 38, "while this was investigated", is mild process phrasing. It is needed to say
  honestly when the addition happened, and it names no plan, task, tracker or branch. ADVISORY
  only; no change required.
- No new issue or PR numbers: `Closes #29` is the only one. No attribution.
- Line 29 is longer than the surrounding wrap width. Cosmetic.

Hosted CI: 12 of 12 on `9ac04cf` (`review-debt.md:27`). The squashed tree is that tree.

VERDICT: PASS
