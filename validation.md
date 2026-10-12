# Validation index — issue 1

Plan: `.stage/scaffold-workspace-project-docs-and/plan.md`. Toolchain: nightly (`rust-toolchain.toml`); MSRV 1.98.1 via `rust-validate 1.98.1`.

| Criterion | Task | Verdict | Command / result | Scope |
|---|---|---|---|---|
| R1–R4, R40–R42 workspace | T1 | PASS (T1-review recheck) | tomllib asserts rc 0; `cargo metadata` 0 warnings; fmt / clippy `-D warnings` / test / doc rc 0; `cargo deny --offline check` ok; `rust-validate 1.98.1 cargo check --workspace --all-targets --locked` rc 0 | root package only; commit a4eacdc |

Full qualification (`make check`): pending (Task 9, one run).
purrdf-mention allowlist (R20): to be recorded in T5–T8.
| R10–R13, R19, R20 licensing | T2 | PASS (T2-review) | Mulan/CC-BY digests match; cmp with purrdf; zh diff is deletions only plus the ® on line 1; backtranslation SHA `01eb7226…` and 1,692 bytes match the source; `uvx reuse@6.2.0 lint` 12/12; `git grep -il purrdf` empty | commit fc6f531 |
| R5–R7, R36 gates | T3 | PASS (T3-review recheck) | `make check` ran once during T3 on the 14-file tree, rc 0. This tested the new gate itself; it is not the workflow qualification, which runs in T9 on the final tree. Missing-tool hard fails rc 2; `issue-refs` positive and negative cases; hook reads the index (/tmp clone); `shellcheck` and `actionlint` clean; pins verified | commit b7db728 |
| R8, R9, R14–R18, R20, R43 house/community docs | T4 | PASS (T4-review recheck) | PGP sha `6bb700ee…`; `yq` and cffconvert valid; `git cliff --unreleased` rc 0, with the PR-number strip tested; `make issue-refs` rc 0; reuse 27/27; purrdf and brand greps empty. Open: the docs say the document-contract test exists "today", so it must land in T6 | commit c7edb76 |
| R21–R24, R43 brand and README | T5 | PASS (T5-review) | cat-head-core sha `4ec35cf6…`; accent isolation `[]`; `xmllint` rc 0; PNG re-render `cmp` identical (local only), 1280x640; visual check done; reuse 32/32; `make issue-refs` rc 0. README capability links must exist after T6/T7 | commit 49eb42b |

R20 purrdf allowlist: `docs/BRAND.md` (family source of cat-head-core).
| R25–R27, R30, R32, R35–R37 census docs + architecture | T6 | PASS (T6-review recheck) | docs_contract: 46 tests, 45 pass; the only real-tree violations are 4 README links to T7 docs. R27 greps match; reuse 52/52; `make issue-refs` rc 0. The test is committed after T7 so every commit stays green | commit 25e1978 (docs); test pending |
| R28, R30–R32 (sections 1–4, 6, 7), R35–R37 workplan docs | T7 | PASS (T7-review recheck) | docs_contract 46/46 with 0 violations; R28 greps; essay key phrases present; reuse compliant; `make issue-refs` rc 0 | commit 9ce066f |
| Document-contract test (T6a) | T6 | PASS (T6-review recheck) | `cargo test --test docs_contract --locked` 46 passed, 0 failed | commit b499578 |
| R29, R31, R35–R37 M2 docs and essay migration | T8 | PASS (T8-review) | R29 loop no output; 21 capabilities and 21 change records; essay fully covered; INITIAL_PLANNING_PLAN gone; reuse 77/77 | commit 9f93e5d |

## Full qualification (single run for the workflow)
- Command: `make check` on commit 9f93e5d, clean tree. **rc 0.** Log: `raw/make-check-9f93e5d.log`.
- Steps: fmt ok; clippy `-D warnings` ok; `cargo test --workspace --locked`, 46 docs_contract tests passed; `cargo doc` with `-Dwarnings` ok; `cargo deny --offline check` reports advisories, bans, licenses and sources ok; `uvx reuse@6.2.0 lint` compliant; `rust-validate 1.98.1 cargo check` ok; licenses ok; issue-refs ok.
- Toolchain: nightly rustc 2026-09-13 for the gates; MSRV rustc 1.98.1 via rust-validate.
- Hosted CI: pending until the PR is opened (the workflow triggers on pull_request).
- Review files T1, T3, T4, T6, T7: each superseded initial BLOCKED line is relabelled 'Initial verdict (superseded by the Recheck below)' so stagectl reads the current recheck verdict. The history is preserved.
- Completeness review (`reviews/completeness-review.md`): one REQUIRED finding, R20. `census-reference-scan.md` used a purrdf package as its example; it now uses `acme-core`, in commit after 9f93e5d. Rerun of affected checks: docs_contract 46/46, `make issue-refs licenses` ok, reuse compliant, `git grep -il purrdf` gives only `docs/BRAND.md`. Reuse assessment: the full `make check` on 9f93e5d still covers everything else, since this change touches only one Markdown file whose checks were re-run.
- R37 hits: the only `defer|follow-up` matches are the no-deferral rule statements in AGENTS.md and CLAUDE.md, which state the rule rather than defer anything.
- **Hosted CI, PR 11 (run 38210871728, head d6b6572): NOT RUN, unverified.** All 8 jobs failed in 2–3 s with the annotation "The job was not started because recent account payments have failed or your spending limit needs to be increased." This is an organisation billing block; no step executed. Clearing it is an owner action; then re-run the workflow.
- Automation gap: `stagectl pr-create` reported "gh returned unparseable JSON" although the PR was created correctly (verified with `gh pr view 11`).

---

# Stage 2 (gap analysis and remediation)

**Review path: FULL.** Reason: 86 files (+8184/−248) spanning build/CI infrastructure (`ci.yaml` with SHA-pinned actions and supply-chain surface), a staged-index git hook, a 1,799-line std-only contract test, licensing artifacts, and 21 capability specs that set contracts for every later crate. Shared contracts and supply-chain surface rule out the lightweight path by policy.

- Reviewed head: d6b6572 (PR 11), base origin/main @ 720262d.
- Feedback surfaces (raw/pr-11*.json): 0 reviews, 0 inline comments, 0 review threads; issue/PR comments are author workflow posts only. No external reviewer feedback to adjudicate.
- `issue-analysis.md`: **reused** from Stage 1 (issue body unchanged since it was written; the only new comments are the author's own workflow posts). Not re-dispatched.
- **Automation gap:** `stagectl brief --mode review` crashes with UnicodeDecodeError when it decodes `git show HEAD:docs/social-preview.png` as UTF-8 (`stagectl/resolve.py:49` via `brief.py:67 build_review`). `branch-under-review.md` was written with direct git/gh commands instead; `gate review-brief` OK.
- Hosted CI: run 38210871728 re-requested (`gh run rerun`, rc 0) at Stage 2 start to probe whether the billing block has cleared.

## Stage 2 specialists
| Concern | Dispatched? | Reason |
|---|---|---|
| security-engineer (opus/high) | Yes | CI workflow (action pinning, token permissions, untrusted PR input), pre-commit shell hook, Makefile shell recipes, deny.toml, SECURITY.md attack surface |
| quality-engineer (sonnet/medium) | Yes | `tests/docs_contract.rs` is the only executable enforcement of the doc profile; its edge cases and negative fixtures carry the acceptance criteria |
| performance-engineer | No | No hot path, runtime code, or benchmarks; `src/lib.rs` is a 6-line doc stub |
| system-architect | No | No new code modules; capability-spec design was approved upstream in the design plan and reviewed in Stage 1, and spec content fidelity is in gap-analysis scope |
- Hosted CI re-run (run 38210871728 attempt 2, head d6b6572): **still NOT RUN.** Every job annotation reads "The job was not started because recent account payments have failed or your spending limit needs to be increased." This is an external blocker that needs an owner billing action. Also annotated: `ubuntu-latest` migrates to Ubuntu 26 from 2026-10-19.
- Environment: `/opt` is 100% full (232 MB free). The worktree `target/` symlinks to `/opt/.cargo/target/...`, and the 193 GB there belongs to other projects, not this session. Builds in Stage 2 use `CARGO_TARGET_DIR=/home/paudley/Active/.raf-stage2-target` (493 GB free). The tested inputs are unchanged; only the artifact location differs.
- Specialist reports received: `historical-fit.md` (0 CRITICAL/HIGH, 4 MEDIUM), `reviews/security-review.md` (0 CRITICAL/HIGH, 2 MEDIUM spec-contract, 5 LOW), `reviews/quality-review.md` (1 HIGH: capability set not pinned; 5 MEDIUM; 8 LOW; 42 mutation probes).
- Remediation plan: `remediation-plan.md` (R1–R10). `gate gaps` OK. Plan review is in progress (`reviews/remediation-plan-review.md`, fable/high).
- User decisions this stage (verbatim): G3 "(1) but add an issue for (2)"; S-M2 "Workspace = build input (Recommended)"; G7 "Port gates to Rust (Recommended)".
- Filed issue 12 (option 2, CLI runs repository rules), as the user asked. Body: `raw/new-issue-census-cli-repo-rules.md`. It is an enhancement on top of the fully resolved G3, not a scope cut. Note: the Stage 1 plan records an earlier "at most 10 issues" decision; this explicit request supersedes it.
- Automation gap: stagectl has no issue-create wrapper, so `gh issue create` was used directly.
- Plan review: PASS on pass 3 (`reviews/remediation-plan-review.md`). Plan published to issue 1 and PR 11.
| R1 workplan docs (G1, G4, G5w, G6, G8, G12, A4) | G-R1 | PASS (recheck) | R1 plan checks; docs_contract 46/46; issue-refs ok; reuse 77/77; hook passed | commit f711aa2 |
| R2 census docs (G3, G5c, G13a–g, G15a) | G-R2 | PASS (recheck) | R2 plan checks; docs_contract 46/46; issue-refs ok; reuse 77/77; hook passed | commit 6f8891e |
- User decision (R3, stack overflow), verbatim: "Fail-closed, honest (Recommended)". The specs state that budgets, a 64 MiB stack and a fuzz target reduce the risk of overflow; a residual overflow aborts the run and fails the gate, never passing it. This replaces the "no crash" wording of decision 2. Reason: syn recursion cannot be bounded provably in-process (G-R3 recheck RR-1).
| R3 security specs (S-M1, S-M2, S-L3, S-L4) | G-R3 | PASS (recheck 3) | R3 plan checks; docs_contract 46/46; issue-refs ok; reuse 77/77; hook passed | commit 6120029 |
| R4 licensing (G14a–e, A3) | G-R4 | PASS (recheck) | char-level difflib: only ® inserted; make licenses ok; issue-refs ok; reuse 77/77; hook passed; translator I/O in raw/backtranslation-r4/ | commit 3e20c61 |
| R6 Rust gates (G7, G9, G16-F7, Q-M5, Q-L1) | G-R6 | PASS (recheck) | repo_gates 40/40; docs_contract 46/46; clippy, fmt and msrv clean; reuse 79/79; actionlint; hook warm 0.57 s; scratch-clone refusals x4 | commit 2faa697 |
- Environment: the machine's cargo wrapper (kache) places test artifacts under `/opt/.cargo/...` even when `CARGO_TARGET_DIR` is exported. `/opt` has about 230 MB free. If a build fails with ENOSPC, that is the cause; the remedy is an owner cleanup of `/opt`, never a hook bypass.
- Update: `/opt` now has 496 GB free (73% used), freed outside this session. The disk constraint no longer applies.
| R7 document contract (Q-H1, Q-M1–M4, Q-Ls, G6/G8/G12 rules, A2, A-7) | G-R7 | PASS (recheck 4) | docs_contract 112/112; repo_gates 40/40; clippy, fmt, msrv; reuse 79/79; hook passed; 12 mutation probes fail as required | commit dba307f |
| R8 CI pins and hooks (G11, S-L1, S-L2, G16, S-L5) | G-R8 | PASS | actionlint; zizmor (offline); ls-remote and compare; scratch-clone merge refused; reuse 81/81; hook passed | commit cb90d10 |
| R9 cliff dedup (G10) | G-R9 | PASS | scratch-clone merge scenario before/after; git cliff rc 0; reuse 81/81; hook passed | commit 8ff3859 |
| R5 project docs (G15b–d, A1) | G-R5 | PASS | R5 greps; SPDX shape scan; cargo test 112+40; reuse 81/81; git cliff rc 0; hook passed | commit df65ee4 |

## Final full qualification (Stage 2, settled head)
- Command: `make check` on **df65ee4**, clean tree, with `CARGO_TARGET_DIR=/home/paudley/Active/.raf-stage2-target`. **rc 0.** Log: `raw/make-check-df65ee4.log` (start 08:45:38Z, end 08:45:41Z).
- Steps, all ok: fmt; clippy `-D warnings`; test (docs_contract 112, repo_gates 40, lib 0, doc-tests 0); doc with `-Dwarnings`; `cargo deny --offline` (advisories, bans, licenses, sources ok); reuse 81/81; msrv `rust-validate 1.98.1 cargo check --all-targets`; licenses and issue-refs (repo_gates 40/40).
- Build artifacts were reused through cargo's freshness check (the same inputs had been built by the hooks and reviews); cargo rebuilds anything whose inputs changed.
- This supersedes the Stage 1 run on 9f93e5d, which stays a historical snapshot, as the workflow's qualification. Any later fix commit invalidates it.
- Hosted CI: every run on this branch, including df65ee4's predecessors, was refused by GitHub billing ("recent account payments have failed…"). **No CI step has ever executed. CI is unverified and blocked by an external owner action.**
- Squash-notes draft: `squash-notes-draft.md`. It is a draft only; Stage 3 finalizes it.

## Stage 2 exit audit, pass 1
- `reviews/completion-audit-stage2-exit.md`: **FINDINGS-OPEN**.
- Open findings:
  - AC1b (CI never ran): external, billing.
  - F1 (MEDIUM): the `#` rule regressed on `org/repo#12`. The cause is the word-character prefix exemption, which entered through the plan-review F5 rewrite.
  - F2–F5 (LOW): text contradictions in okf-profile, census-detectors, census-migration/debt-ratchet and complete-gate.
- Remediation group **R11** was opened for F1–F5.
- The final qualification on df65ee4 is invalidated by R11. One more full `make check` will run after R11 settles; that is a recorded additional run, per the plan's validation-budget condition.
- R11 advisory V5 (the exit audit's double-reported frontmatter failure): declined. One malformed frontmatter yields two violation lines, the parse error and the missing key it causes. That is cosmetic; the gate still fails closed, and deduplicating it would need a checker refactor with no behavioural benefit.
| R11 exit-audit fixes (F1 MEDIUM, F2–F5 LOW) | G-R11 | PASS (recheck) | parity table vs the Stage 1 grep; hook refusal of org/repo#12; docs_contract 113; repo_gates 42; clippy, fmt, msrv; reuse 81/81 | commits 45bc98e, edf93d5 |
- **Recorded additional full qualification:** `make check` on **edf93d5**, clean tree, **rc 0** (log `raw/make-check-edf93d5.log`). All steps ok: docs_contract 113, repo_gates 42, deny ok, reuse compliant, msrv ok. It supersedes the df65ee4 run, which R11 invalidated. Any further commit invalidates this one.
| R11 round 2 (exit-audit N1) | G-R11 recheck 2 | PASS | clean-job walk-through; docs_contract 114; repo_gates 42; clippy, msrv; reuse | commit 80c14d7 |
- **Current full qualification:** `make check` on **80c14d7**, clean tree, **rc 0** (log `raw/make-check-80c14d7.log`): docs_contract 114, repo_gates 42, deny ok, reuse compliant, msrv ok. It supersedes the edf93d5 run; the Stage 1 run and the df65ee4/edf93d5 runs are historical snapshots.

## Stage 2 exit (final)
- **Exit gate:** `reviews/completion-audit-stage2-exit.md`, pass 3: **PASS** on **80c14d7**. Assessed changes: the full branch, origin/main..80c14d7. Pass 1 covered the full contract; passes 2 and 3 rechecked R11 and its follow-ups.
- **Criterion coverage:**
  - AC1a (`make check`): demonstrated.
  - AC1b (CI): **unverified because of an external blocker.**
  - AC2 (reuse): demonstrated.
  - AC3 (capabilities, 7 sections, resolved text): demonstrated.
  - AC4 (no issue numbers): demonstrated.
  - Every Scope checklist item: demonstrated.
  - Every Stage 2 gap and finding: fixed or recorded false positive.
- **Limits:**
  - Hosted CI has never executed; run 38229506850 on 80c14d7 was billing-refused.
  - zizmor ran offline only.
  - The social-preview PNG reproduces only with this machine's fonts.
- **Non-blocking advisories left open:**
  - The migrate usage could say the index command runs on the earlier RAF version.
  - "Names, not hashes" is loose for `tokens` patterns.
- **Stage 3 reuse:** this audit can qualify if the head stays 80c14d7 and the base delta is assessed. A new commit invalidates both the audit and the `make check` qualification.
- **Owner actions, outside the repo:**
  - Clear GitHub billing and re-run CI on the final head, or waive it explicitly.
  - Upload `docs/social-preview.png` in Settings and set the repository description (text in `docs/BRAND.md`).
  - Renew the PGP key before 2026-12-24.
- **`.deficiencies`:** notice and marker only. No user-authorized cuts were recorded.
- Stage 2 final summary posted to issue 1 and PR 11. Noted: `stagectl tasks` reports Stage 1 Task 9 as TODO because no T9-review.md exists, although its work (qualification, PR 11, handoff) was done in Stage 1.

---

# Stage 3 (finalize and merge)

## Integration assessment
- PR 11: base `main` @ 720262d, from stagectl pr-meta and a fresh `git fetch`. Head 80c14d7.
- The merge base equals the base (720262d), and there are 0 base-side commits.
- `git merge-tree --write-tree --no-messages 720262d 80c14d7`: rc 0 (clean), tree aa1b4b4. That **equals `80c14d7^{tree}`**, so the integration candidate is byte-identical to the PR head tree.
- Semantic assessment: **not dispatched**, because the base-side delta is empty. With no base change there are no shared consumers, APIs, toolchain or build settings that can interact, and the semantic comparison is degenerate.
- Evidence reuse: `make check` rc 0 on 80c14d7 (`raw/make-check-80c14d7.log`) ran on exactly the candidate tree. It qualifies the integration with no further run. No synchronization or new suite is needed.
- Branch protection on main: none (HTTP 404), so the forge has no required checks. The issue acceptance criterion "…and so does CI" still applies as a contract criterion.
- Feedback surfaces: 0 reviews, 0 inline comments, 0 threads. All 15 PR and 22 issue comments are the author's workflow posts. Raw data: `raw/s3-pr-11-*.json`, `pr-comments.md`, `pr-checks.txt`.
- Hosted CI on 80c14d7 (run 38229506850): every job refused for billing, with zero steps run. **Unverified.**
- **Owner waiver**, verbatim: "Waive CI, merge now" (2026-10-12), in answer to the CI blocker question. It is recorded and published in `raw/ci-waiver.md`. CI stays **unverified**, not passed. It is not a `.deficiencies` entry, because ledger rules exclude unrun gates and validation state.
- Review debt: `review-debt.md`, BLOCKED on CI only, then discharged by the waiver (addendum). No other feedback exists.
- **Completion audit reuse:** `reviews/completion-audit-stage2-exit.md` pass 3 PASS covers head 80c14d7, which equals the integration candidate. The base delta is empty, and no new feedback, requirement, source change or policy has arrived since. Its one open item, the CI half of AC1, is now covered by the owner waiver. It qualifies under `_validation.md`, so no fresh Stage 3 auditor is needed. This is a reuse, not a fresh audit.
- CI waiver publication: issue 1 comment 6121539811, PR 11 comment 6121540573. The posted text is in `raw/post-ci-waiver.md`, identical to `raw/ci-waiver.md`. Correction to that text: zizmor ran in **offline** mode (13 suppressed). The action pins were verified as follows: checkout and setup-uv against their tags in `reviews/security-review.md`, and dtolnay with `ls-remote` and `compare` in G-R8.
- Comment count correction: the issue had 22 comments before the waiver post and 23 after; `review-debt.md`'s figure of 21 came from an earlier fetch. Every comment is by the author.
- Merge-plan review: `reviews/merge-plan-review.md`, pass 1 NEEDS-REVISION. F1 and F2 (required) are fixed in the squash notes. F3, F4 and F7 are applied. F5 and F6 are fixed in these records.
