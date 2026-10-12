# Merge-plan review (Stage 3, single focused review)

Reviewed: `/home/paudley/Active/rust-agentic-foundations/.worktrees/1-scaffold-workspace-project-docs-and.squash_notes`
Head `80c14d7`, base `main` @ `720262d`, PR 11, issue 1. Reviewed on 2026-10-12.

## Coverage

Read in full: `_planning-reviews.md`; the squash notes; `validation.md` (all stages); `review-debt.md` with its
addendum; `reviews/completion-audit-stage2-exit.md` (passes 1-3); `raw/ci-waiver.md`; `raw/make-check-80c14d7.log`;
`issue.md`; `plan.md`; `remediation-plan.md`; `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `CHANGELOG.md`.
Tree checks against the "What changed" claims: `Cargo.toml`, `Makefile`, both hooks, `ci.yaml`, `dependabot.yml`,
`cliff.toml`, `.deficiencies`, `tests/support/mod.rs`, the `mod support` and `#[test]` counts in both test files,
`docs/LICENSING.zh-Hans.md` against `~/Active/purrdf/docs/LICENSING.zh-Hans.md`, the whole `cat-head-core` group in
`docs/raf-logo.svg` against `~/Active/purrdf/docs/purrdf-logo.svg`, `census-cli.md` (exit 2), the `state:` line of all
21 capability documents, the G-R8 review (pins, zizmor), and the exit-audit hook probe logs.
No commands were run; the repo and the notes file were treated as read-only.

## The six questions

| Question | Answer |
|---|---|
| Integration evidence sufficient for the actual candidate? | **Yes.** Merge base = base, 0 base-side commits, `merge-tree` rc 0 with tree `aa1b4b4` = `80c14d7^{tree}`. `make check` rc 0 ran on exactly that tree (`raw/make-check-80c14d7.log`: header `HEAD 80c14d7`, 9 step headers, `RC=0`, docs_contract 114, repo_gates 42, reuse 81/81, deny ok, msrv ok). Skipping the semantic assessor is sound: with an empty base delta there is nothing for the branch to interact with; the assessment would compare the head tree to itself. |
| Every comment or thread has an honest disposition? | **Yes, with one record gap (F5).** 0 reviews, 0 inline comments, 0 threads (`raw/s3-pr-11-*.json`). All PR and issue comments are the author's workflow posts and are listed by ID as informational. The waiver post itself, if it was published, is not in the table and no raw record of the post exists. |
| Squash notes claim only what the evidence shows? | **Mostly.** Verified: 114 + 42 tests (log and `#[test]` counts agree); every `repo_gates` rule has a `rejects_*` fixture; the whole `cat-head-core` group is line-for-line identical to purrdf's; the hooks list and what they run; the exit-2 CLI refusal; 9 + 7 designed, 5 considered; 21 change records; `.deficiencies` empty; 22 commits listed; 90 files listed. Two claims overstate (F1, F2) and three are loosely worded (F3, F4, F7). |
| Any issue requirement silently missing? | **No.** Every Scope item is in the tree. The three departures from the issue text (MSRV 1.98.1; `.goals`/`.baseline` never created; CI half of AC1) are recorded owner decisions and are stated in the notes. |
| CI waiver handled honestly? | **Yes.** The notes say "never executed", "unverified, not passed"; the waiver is quoted verbatim with its date; it is recorded in `raw/ci-waiver.md`, not in `.deficiencies`, which is correct under AGENTS.md's ledger definition (misrepresented or undone work, not an unrun external gate). Two wording points in `raw/ci-waiver.md` (F6) and one in the notes (F7). |
| Reuse of the Stage 2 audit valid? | **Yes.** Pass 3 was run on `80c14d7`, the same head; the base has not moved; no feedback, requirement, source or policy change has arrived since; the audit's only open item (AC1b) is now covered by the waiver and is still described as unverified. A fresh auditor would re-read unchanged text. |

## Findings

### F1 (REQUIRED) "differs from its purrdf source only by deletions" is not true as written

Squash notes line 32. Comparing `docs/LICENSING.zh-Hans.md` with purrdf's file: the changes are (a) `PurRDF` to `RAF`
on lines 4 and 9, (b) `®` inserted in the SPDX copyright line, and (c) deletions (the distribution-package sentence,
`、Unicode 数据`, the Unicode-3.0 sentence, the YAML/XML lines, the packaging paragraph, `、来源记录`). Every stage
record states this precisely ("after the name sed", "deletions only plus the ® on line 1", "one insertion (`®`) and 5
deletions"). The commit message is the permanent record and should match.

REWRITE TO: "The Simplified Chinese explanation is purrdf's text with the project name substituted and purrdf-only
material deleted; apart from the `®` in its SPDX line, no character was added and no new Chinese was written. Its
independent backtranslation record carries the fingerprint the gate checks."

### F2 (REQUIRED) "Declined enhancements, with reasons in `remediation-plan.md`" points to the wrong file for three of five

Squash notes lines 73-78. Where the reasons actually are:
- extra CITATION fields: `remediation-plan.md` ("historical-fit F7, second half"), correct;
- six-digit `#123456`: `remediation-plan.md` (Q-L1), correct;
- `fmt-fix` target: `gap-analysis.md` F6 ("optional polish"); the remediation plan only says "not gaps, per the
  gap-analysis adjudication table";
- future-tense prose: `reviews/quality-review.md` MEDIUM-1 ("inherently not mechanical"); `remediation-plan.md` never
  mentions it, and R7 implements only the placeholder half;
- generated-block contents before `docs sync`: `tasks/G-R7-review.md` ADV-8; not in `remediation-plan.md`.

REWRITE TO: "**Declined enhancements**, each with a recorded reason in the stage records:" and keep the five bullets.
If a file pointer is wanted, use "reasons in `remediation-plan.md`, `gap-analysis.md`, `reviews/quality-review.md`
and `tasks/G-R7-review.md`".

### F3 (advisory) "The workflow pins every action and tool"

Squash notes line 25. The three actions are SHA-pinned and the installed tools are version-pinned (cargo-deny 0.20.2,
uv 0.13.0, reuse 6.2.0), but the nightly toolchain floats by design (`rust-toolchain.toml`, AGENTS.md §2) and the
runner is `ubuntu-latest`. "Every tool" invites a reader to look for a pinned nightly.

REWRITE TO: "The workflow pins every action by commit SHA and every installed tool by version, cancels superseded
runs, and is kept current by Dependabot. The nightly toolchain floats by design."

### F4 (advisory) "every finding was fixed or adjudicated as a false positive"

Squash notes line 96. Some items were neither: V5 (double-reported frontmatter failure) was declined as cosmetic; the
CITATION fields were declined; the PGP renewal and the social-preview upload became owner actions.

REWRITE TO: "every finding was fixed, recorded as a false positive, or declined with a reason; three items became owner
actions (listed below)."

### F5 (advisory, stage records) Comment counts and the waiver post are not reconciled

`review-debt.md` counts 21 issue comments; `validation.md` Stage 3 counts 22. `validation.md` and `raw/ci-waiver.md`
say the waiver was "published to issue 1 and PR 11", but unlike every other publication (`raw/post-R*.md`,
`raw/post-stage2-final.md`) there is no raw record and no comment ID. The disposition table therefore does not cover the
one comment Stage 3 added.

REWRITE TO (review-debt addendum): add "Published as issue comment <id> and PR comment <id> on 2026-10-12; both are the
author's own posts carrying the owner's decision, informational, and raise the counts to 22 and 16." Record the IDs in
`validation.md` and drop a copy of the posted text in `raw/post-ci-waiver.md`. If the post was not made, say so and
make it before merge; the waiver text in the squash notes should not be the first public record of the decision.

### F6 (advisory) Two over-broad sentences in `raw/ci-waiver.md`

- "`actionlint` and zizmor pass on the workflow." G-R8 records zizmor v1.30.1 in offline mode, 13 suppressed.
  REWRITE TO: "`actionlint` passes; zizmor reports no findings in offline mode."
- "Every action pin was verified against upstream." G-R8 verifies the `dtolnay/rust-toolchain` pin by `ls-remote` and
  `compare`, and the setup-uv inputs; the `actions/checkout` SHA verification is not cited there.
  REWRITE TO: "The `dtolnay/rust-toolchain` pin was verified against the upstream `master` head; the other two pins carry
  their release-tag comments and were checked at Stage 1 (T3 review)." Or cite the T3 evidence for all three.

### F7 (advisory) "Its first run happens once billing is cleared" is a prediction stated as fact

Squash notes line 89. `ci.yaml` triggers only on `pull_request` and `push` to `main`; it has no `workflow_dispatch`.
Clearing billing does not re-run anything by itself; the owner must re-run the refused run or push to `main`.

REWRITE TO: "Running it, by re-running the refused run on `main` after merge or pushing a new commit, is an owner action
once billing is cleared. Anything that first execution surfaces is a new change."

## Checks that passed without comment

- The commit message carries `Closes #1` and "(issue 12)". The issue-reference gate judges tracked files, and
  `cliff.toml` consumes only the first line of each message with `#N` stripped, so neither reaches a committed file.
- No attribution text, trailer or agent identity anywhere in the notes.
- The hook refusals listed in "Validation" each have a probe log: issue token, cross-repository token (pass 2),
  altered licence, `census-cli.md` missing `Usage`, CITATION 0.2.0 bump, merge carrying a token.
- "Hooks" and "Rust gates" bullets match `.githooks/pre-commit`, `pre-merge-commit`, the Makefile and
  `tests/support/mod.rs` (one pattern set, one `git ls-files -z` list, `mod support;` in both binaries).
- Design decisions "all owner-approved": each of the four has a verbatim owner answer in `remediation-plan.md` or
  `validation.md`.

## Verdict (pass 1)

Two factual overstatements in a permanent commit message (F1, F2) need one-line edits before the squash. Everything
else is advisory. The integration evidence, the audit reuse, the semantic-assessor skip and the CI wording are sound.

VERDICT (pass 1): NEEDS-REVISION

---

## Recheck (pass 2)

Scope: only the seven changed items. Re-read the revised squash notes (lines 1-120), `validation.md:130-138`,
`review-debt.md:97-103`, `raw/post-ci-waiver.md`, and `reviews/security-review.md` E2 (lines 34-56, 260) to confirm the
pin-verification citation the correction now makes. Unchanged text was not re-audited.

| Finding | What changed | Checked against | Status |
|---|---|---|---|
| F1 | Line 32: "differs from its purrdf source only by the RAF name substitution, the `®` in its SPDX line, and deletions." | The purrdf and RAF zh-Hans files compared in pass 1: name on lines 4 and 9, `®` on line 1, the rest deletions. The sentence now names all three kinds of change. | DISCHARGED |
| F2 | Line 73 now cites, per item: CITATION fields and `#123456` in `remediation-plan.md`; `fmt-fix` in `gap-analysis.md` (F6); future-tense prose in `reviews/quality-review.md` (MEDIUM-1); generated-block contents in `tasks/G-R7-review.md` (ADV-8). | The four locations pass 1 found. Each pointer is correct. | DISCHARGED |
| F3 | Lines 25-26: "pins actions by commit SHA and installed tools by version (the nightly toolchain floats by design)". | `ci.yaml` (three SHA pins; cargo-deny 0.20.2, uv 0.13.0, reuse 6.2.0; `toolchain: nightly`). | DISCHARGED |
| F4 | Line 98: "fixed, recorded as a false positive, or declined with a recorded reason". | Covers V5 and the CITATION decline. The owner actions are listed separately at lines 110-113, so the sentence no longer has to cover them. | DISCHARGED |
| F5 | `validation.md:136` records issue comment 6121539811 and PR comment 6121540573 and names `raw/post-ci-waiver.md`; `:137` explains the 21/22 discrepancy (earlier fetch) and gives 23 after the post; `review-debt.md:103` gives the waiver post an informational disposition. `raw/post-ci-waiver.md` exists and is the waiver text. | The IDs are consistent between the two records. The disposition is honest: an author post carrying an owner decision, not feedback. | DISCHARGED |
| F6 | `validation.md:136` records that zizmor ran offline (13 suppressed) and that checkout and setup-uv were verified against their tags in `reviews/security-review.md`, dtolnay by `ls-remote` and `compare` in G-R8. | `security-review.md:38` and `:41` show `9c091bb…` = `refs/tags/v7.0.0` and `bec219d…` = `refs/tags/v10.1.0` from read-only `ls-remote`; `:260` restates it. The citation is accurate, and the correction supersedes the two over-broad sentences, which stay verbatim in `raw/ci-waiver.md` and `raw/post-ci-waiver.md` as the record of what was posted. That is the right way to correct a published text. | DISCHARGED |
| F7 | Lines 89-91: "Running it is an owner action once billing is cleared. The workflow triggers on push and pull_request and has no manual trigger, so it runs on the next push to `main` or a re-run of a refused run." | `ci.yaml:5-9` (`pull_request` on `main`, `push` on `main`, no `workflow_dispatch`). | DISCHARGED |

Residual notes, none blocking:
- `review-debt.md:97` still carries the historical "21 on the issue" figure. `validation.md:137` explains it as an
  earlier fetch, so the record is reconciled without rewriting history. No change needed.
- Line 32 of the squash notes is now a 139-character line in an otherwise hard-wrapped body. Cosmetic; wrap it if the
  notes are edited again.

No new claim was introduced by the edits, and nothing was weakened. The squash notes now state only what the tree,
`validation.md` and the reports show.

VERDICT: PASS
