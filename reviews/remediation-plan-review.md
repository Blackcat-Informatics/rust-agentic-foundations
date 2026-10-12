# Remediation plan review (Stage 2, single focused review)

- Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/1-scaffold-workspace-project-docs-and/.stage/scaffold-workspace-project-docs-and/remediation-plan.md` (272 lines, read in full).
- Head d6b6572, base origin/main @ 720262d. Read-only: no plan or repo file edited.
- Inputs read in full: `issue.md`, `plan.md`, `gap-analysis.md`, `reviews/security-review.md`, `reviews/quality-review.md`, `historical-fit.md`, `reviews/completion-audit-stage2.md`, `validation.md`, `AGENTS.md`, `CLAUDE.md`.
- Repo files checked for the facts the plan's checks rest on: `Makefile`, `.githooks/pre-commit`, `.github/workflows/ci.yaml`, `tests/docs_contract.rs` (root resolution, pattern definitions, walker), `docs/LICENSING.zh-Hans.md`, `~/Active/purrdf/docs/LICENSING.zh-Hans.md` (the A3 sentence), `CONTRIBUTING.md` gate text.
- The three verbatim user decisions (Gate owns the census path plus a new issue; workspace = build input; port gates to Rust) are treated as binding and are honoured by R2, R3 and R6 respectively.

## Coverage

| Source | Items | Mapped in plan | Not mapped or only partly mapped |
|---|---|---|---|
| gap-analysis G1–G16 | 16 | G1 R1; G2 R10; G3 R2; G4 R1; G5 R1/R2; G6 R1+R7; G7 R6; G8 R1+R7; G9 R6; G10 R9; G11 R8; G12 R1; G13 R2; G14 R4; G15 R2/R5; G16 R6/R8 | none missing; see F6 (G3 mechanism), F13 (G12 check), F15 (unresolved "or" items) |
| security S-M1, S-M2, S-L1–L5 | 7 | M1 R3; M2 R3 (decision 2); L1 R8; L2 R8; L3 R3/R10; L4 R3; L5 R8 | M1 sub-items (symlink loop, FIFO fixtures, MemoryTree-equivalence claim) and M2(b) sub-items (`-`-prefixed fixtures, binding validation before `cargo test`) are dropped without a disposition: F7. S-L1 `enable-cache: false` dropped: F14. |
| quality Q-H1, Q-M1–M5, Q-L1–L8 | 14 | H1, M1–M4, L2, L3, L5, L6, L7 R7; M5 R6; L1 R6 + adjudication; L4, L8 adjudicated | Q-L6's untracked-file half (loader reads the working tree) is not addressed: F10. |
| historical-fit F1–F9, D1–D3 | 12 | F1 R9; F2 R6; F3 R8; F4/F5/F9 R8; F7 R6; D1/D2 R6; F6, F8, D3 adjudicated | §3 near-miss positive fixtures (`issue-reference`) not in plan: F14. F7's CITATION metadata sub-items (`date-released`, `url`) not in plan: F14. |
| completion audit A1–A6 + CI | 7 | A1 R5; A2 R2 (G15a); A3 R4; A4 R1; A5 adjudicated; A6 R10; CI R10 | none |

The adjudications in "Evidenced false positives / not gaps" (F6, F8, D3, Q-L1 6-digit, Q-L4, Q-L8, A5) are each backed by the cited evidence and I accept them.

## Findings

Blocking findings are F1–F5: each one makes a stated acceptance check fail on the correct fix, breaks a shipped gate, or opens a silent pass. F6–F18 are concrete but non-blocking.

### F1 (blocking) — R4's "deletions only" check fails on the fix it is meant to verify

- Plan lines 138–140, 146. A3 restores the third-party statement by deleting the clause "Unicode 数据和" from inside purrdf's sentence (purrdf `docs/LICENSING.zh-Hans.md:22`), and G14a deletes the phrase "来源记录" from inside RAF's line 22 (`…见[英文说明](../LICENSING.md)、来源记录和文件中的 SPDX 标识；`). Both are intra-line deletions. `diff` is line-based, so each shows as a changed line (`c`), not a deleted line (`d`), and the Stage 1 check (`| grep '^>'` prints only SPDX and link lines) prints Chinese text. The check at line 146 would therefore fail on a correct implementation and could pass only if the implementer skipped A3 and G14a.
- The Stage 1 rule A6/K3 says "keeps purrdf's sentences byte for byte apart from the name and the deletion of purrdf-only sentences". A3 as planned is a clause deletion, which that rule does not cover. The plan must say so, not claim "This keeps the deletion-only rule" (line 140).
- Rewrite line 140 to: "A3 and G14a delete text inside a sentence. This extends A6/K3 from whole-sentence deletion to token-level deletion: no new Chinese character is written, and every change is a removal."
- Rewrite line 146 to: "`git diff --no-index --word-diff=porcelain <(sed 's/PurRDF/RAF/g; s/purrdf/raf/g' ~/Active/purrdf/docs/LICENSING.zh-Hans.md) docs/LICENSING.zh-Hans.md` prints `+` lines only for the SPDX ® line and link targets; every other change is a `-` token. Additionally `grep -c '第三方' docs/LICENSING.zh-Hans.md` is at least 1 (the statement is restored)."
- Add to (c): "The backtranslation must render the two shortened sentences as complete sentences; a fragment reported by the translator is a defect in the deletion, fixed by a further deletion only."

### F2 (blocking) — R6 turns the CI `gates` job into a `cargo test` with no toolchain, and double-runs the gates in `make check`

- Plan line 178: `licenses` and `issue-refs` become `cargo test --locked --test repo_gates`. `ci.yaml:135-144` (`gates` job) runs `make licenses issue-refs` after a bare checkout with no `dtolnay/rust-toolchain` step. After R6 that job invokes cargo on a runner whose only toolchain hint is `rust-toolchain.toml` (nightly, auto-installed by rustup if present at all), i.e. an unpinned, untested path in the one job that has never executed (G2). The plan's R8 touches `ci.yaml` but does not mention the `gates` job.
- Also after R6, `make test` (`cargo test --workspace --locked`) already runs `repo_gates`, so `make check` runs the licence and issue-reference gates twice, and the CI `test` job already covers what `gates` does.
- Rewrite: add to R6 (or R8, but the commit that changes the Makefile must stay green in CI): "The CI `gates` job is deleted; `cargo test --workspace --locked` in the `test` job runs `repo_gates`. `make check` keeps `licenses` and `issue-refs` as thin filter targets for the hook and for humans, and the Makefile header says they are a subset of `make test`." Add the check: "`actionlint` rc 0 and `yq '.jobs | keys' .github/workflows/ci.yaml` no longer lists `gates`; `make -n check` still lists every issue-required step."
- If the owner prefers to keep a separate `gates` job, the rewrite is instead: "the `gates` job gains the same pinned `Install Rust` step as `test`", with the check `yq '.jobs.gates.steps[].uses' .github/workflows/ci.yaml` containing the toolchain action.

### F3 (blocking) — R6's hook design is under-specified on three points that decide whether commits keep passing

- Plan lines 177 and 179. The hook runs `cargo test --locked --test repo_gates --test docs_contract` with `RAF_GATE_ROOT=$snapshot`. That is the right shape (building inside the snapshot would be a cold build per commit), but:
  1. **Which git repository supplies the file list?** With `RAF_GATE_ROOT` pointing at a `mktemp -d` snapshot, a bare `git ls-files` run from the test's cwd works only because cargo sets cwd to the package root. The plan should pin it: "`git -C "$CARGO_MANIFEST_DIR" ls-files -z`, and each listed path is read under `RAF_GATE_ROOT`". Otherwise an implementer who runs `git -C "$RAF_GATE_ROOT" ls-files` gets "not a git repository" in the hook and a green hook in `make`, which is exactly the DARK shape the Stage 2 audit watches for.
  2. **The test binaries come from the working tree, not the index.** `cargo test` compiles `tests/` from the working tree. That is the same limit the security review noted for the Makefile today (hook and rules come from the working tree), and it should be stated in the hook header and in the plan so nobody later "fixes" it by building in the snapshot.
  3. **Build location during this stage.** The plan (line 263) builds with `CARGO_TARGET_DIR=/home/paudley/Active/.raf-stage2-target` because `/opt` is full, but the hook inherits the committing shell's environment. From the R6 commit onward every `git commit` compiles two test binaries; in a shell without that variable the hook's `cargo test` fails on "no space left on device" and the commit is refused, which pushes toward bypassing the hook (forbidden). State: "Every commit in this stage is made with `CARGO_TARGET_DIR` exported; the implementer verifies `git hook run pre-commit` passes before the first R6 commit."
- Add to the R6 check: "`git hook run pre-commit` on the real worktree passes, and its wall time on a warm target dir is recorded in `validation.md` (expected a few seconds; a cold build is a one-off)." This makes the latency cost visible rather than assumed.
- Design note on `tests/support/patterns.rs`: a module included by two test binaries emits `dead_code` for any item one binary does not use, and `clippy --all-targets -D warnings` refuses it. State the convention (`tests/support/mod.rs` with `mod support;`, and every item used by both binaries, or a scoped `#[allow(dead_code, reason = "...")]`), so the R6 commit stays green under `make clippy`.

### F4 (blocking) — R8's repin verification is wrong as written

- Plan line 220: "Repin `dtolnay/rust-toolchain` to `686976e` after verifying with `git ls-remote` that it is on upstream master". Security review E2 shows `686976e` is the head of `refs/heads/stable`, and `stable` is a branch dtolnay rebuilds on each release as master plus one "toolchain: stable" commit. It is not on master (E2: `compare/29eef336…...stable` is `diverged`; the same holds for the new stable head). The verification step would either fail or be waved through, and the new pin will be orphaned again at the next release, which is the exact L2 defect being fixed.
- Rewrite to one of:
  - "Repin to `e2a55d2ffb04f378e9626c28d38b36d230d1e12f`, the `master` head (contains `83d0610` 'Safely handle action inputs'). Verify: `git ls-remote https://github.com/dtolnay/rust-toolchain refs/heads/master` prints that SHA, and `gh api repos/dtolnay/rust-toolchain/compare/83d06109b7...e2a55d2ffb --jq .status` prints `ahead` or `identical`. The comment reads `# master, 2026-10-09`." (durable: master is never rebuilt), or
  - keep `686976e` and verify "its parent is on master and `compare/83d0610...686976e` is `ahead`", and add "this pin is re-orphaned at every upstream release; `.github/dependabot.yml` is the update path".
- Either way the check at lines 227–233 should include the `compare` output, not just `grep`.

### F5 (blocking) — R6's colour-sparing rule as worded opens a silent pass for 3- and 4-digit issue tokens

- Plan line 173: "Add `#` with a non-`&` prefix, sparing 3-, 4-, 6- and 8-digit hex colours". `#123` and `#1234` are valid 3/4-digit hex colours and also the most common shape of an issue token. A blanket "spare 3- and 4-digit hex" lets `see #123` through, which converts a fail-loud false positive (G9, LOW) into a fail-silent gap on the AC4 rule.
- Rewrite line 173 to: "`#[0-9]{1,5}\b` requires a prefix that is not `&`, a word character or `#`. A 3/4/6/8-hex-digit token is spared only when immediately preceded by `"`, `'`, `:` or `(` (attribute, CSS and function contexts), or when it contains a letter a–f. Fixtures: `fill="#000"`, `stroke='#abc'`, `color: #1f2`, `&#38;` and `#123456` pass; `see #123`, `(#1234)` and `Per #12` fail."
- Rewrite the check at lines 186–187 to list those exact lines, including the refusals of `see #123` and `(#1234)`.

### F6 — R2 leaves the Gate write path as "e.g.", which cannot land in a `designed` document

- Plan line 68: "The Gate gains the debt write path, e.g. a `Gate` method that writes `census-debt.json`, run from the repository's census test or an xtask." AC3 requires resolved text in designed docs; "e.g." and "or an xtask" leave the mechanism open and would be reproduced as hedging by the implementer.
- Rewrite: "`Gate::write_debt(&self) -> Result<DebtWrite, Error>` writes `census-debt.json` beside the ledger, refuses while any non-debt finding exists (the same refusal as the former CLI path), and is called from the repository's own binary or xtask, never from a `#[test]` (tests do not write tracked files). `census-debt-ratchet` Usage shows the xtask invocation; `census-cli` Contract names it as the only write path for ledgers that need registered code." Add the check: "`grep -n write_debt docs/capabilities/census-rule-api.md docs/capabilities/census-debt-ratchet.md docs/capabilities/census-cli.md` returns at least 3 lines."

### F7 — R3 silently drops security sub-items from M1 and M2

- M1 required fixtures: the plan lists four (line 112) but omits the symlink loop and the FIFO/non-regular file, although line 110 states the rule for non-regular files. Under "every rule has a negative fixture" the rule is unenforced without them. M1 also asked to drop the claim that `MemoryTree` equivalence covers the disk path for these cases (`source-index.md:133-134`); the plan does not touch it.
- M2(b) required: argv-vector spawning with `--` (present, line 115), plus a fixture whose slug, binding or path begins with `-` for entity-history and complete-gate, plus validation of binding names against the source index before they reach `cargo test`. The plan has no fixture and no validation sentence, and its check (lines 126–131) never greps for the `--` rule. Under decision 2 the trust model narrows, but the decision also says "the specs pin … `--` before arguments taken from documents", so the fixtures are in scope.
- Rewrite line 112: "add disk fixtures to Verification: a symlinked mod file, `dir-symlink/..`, a target path outside the root, a link to a symlink outside the root, a directory-symlink loop, and a FIFO; state that `MemoryTree` equivalence does not cover these cases." Add to S-M2: "complete-gate runs only bindings that resolve in the source index; entity-history and complete-gate each carry a Verification bullet with a `-`-prefixed path or binding." Add the check: "`git grep -n -e ' -- ' -- docs/capabilities/workplan-entity-history.md docs/capabilities/workplan-complete-gate.md docs/capabilities/census-reference-scan.md` returns at least 3 lines; `grep -n -iE 'fifo|loop' docs/capabilities/source-index.md` returns at least 2."

### F8 — R3's S-L4 check covers one of three parsers

- Plan line 129 greps only `source-index.md`, while line 119 states bounds for YAML (okf-profile) and TOML (census-ledger) too. Rewrite: "`grep -n -iE 'nesting|depth|maximum file size|alias' docs/capabilities/source-index.md docs/capabilities/okf-profile.md docs/capabilities/census-ledger.md` returns at least one line per file."

### F9 — R5 writes a CHANGELOG bullet for a test that does not exist until R6

- Plan lines 156 and 267: R5 (fifth commit) adds a bullet for "the document-contract and repository gate tests", but `tests/repo_gates.rs` lands in R6. At the R5 commit the CHANGELOG describes something not in the tree, which is the same present-tense-claim defect the T4 review blocked on. The R5 check (`grep -c '^- '` increases) is also too weak to catch a missing topic.
- Rewrite: move R5 to run after R9 (order R1 → R2 → R3 → R4 → R6 → R7 → R8 → R9 → R5), and have its CHANGELOG bullets cover the ported Rust gates, the hardened contract test, the CI pins and the cliff dedup as well. Rewrite the check to name each topic: "`grep -n -iE 'brand|logo|README' CHANGELOG.md`, `grep -n -i 'capabilit' CHANGELOG.md`, `grep -n -iE 'docs_contract|document.contract' CHANGELOG.md`, `grep -n -i 'repo_gates\|repository gate' CHANGELOG.md`, `grep -n -i 'essay' CHANGELOG.md` each return at least 1 line under `[Unreleased]`."

### F10 — R7's acceptance check is not falsifiable enough, and Q-L6 is half-closed

- Line 210: "test count above 46" is satisfied by a single added test. Rewrite: "the count equals 46 plus the number of new fixtures, each listed by name in `tasks/G-R7-review.md`; the mutation probes 7, 9, 18, 19, 22, 24, 25 (non-`.md` file in `docs/capabilities`), 26, 29, 31 (as the G8 rule: non-empty `designed` bindings), 32 and 34 (`.github/PULL_REQUEST_TEMPLATE.md` broken link) each fail with the named rule."
- Q-L6 has two halves. The plan takes "typed doc outside its directory" (line 202) and drops "the loader reads the working tree, untracked files included, so a scratch file in `docs/` can satisfy a link locally that CI will not see". R6 already introduces an index-derived file list in the shared support module; `docs_contract.rs` should use the same list. That closes Q-L6 fully and makes hook, `make test` and CI judge the same file set. Rewrite line 177's last sentence: "`docs_contract.rs` reads the same root variable, the shared patterns and the shared index-derived file list; its walker and duplicate pattern set are removed."

### F11 — R7 Q-H1 pinned table: accept the cost, but state the update rule where it bites

- Pinning 21 slugs with their state family means any state promotion or new capability is a one-line test edit. That is the right trade for AC3 (it is the only executable form of "every capability listed above exists"), but the plan should say so in the test and in `AGENTS.md` §6, or the first implementer of a census crate will read the failure as a test bug. Add to R7: "the table's doc comment says: this table is the capability inventory; changing a state or adding a capability edits this table in the same commit; `raf_workplan::Gate` replaces it." Add to R6's Docs bullet: "AGENTS.md §6 names the inventory table."

### F12 — R2's checks are not concrete enough to fail

- Lines 91–95: "grep shows a census-cli Contract sentence covering…", "`yq` depends checks for each edge", "census-ledger validation lists unknown `rule:` ids" have no command. Rewrite: "`grep -n -E 'job\.ext' docs/capabilities/census-cli.md` and `grep -n -E 'exit(s)? 2' docs/capabilities/census-cli.md` each return at least 1 line; `grep -n -E 'unknown .*rule:|rule: .* unknown' docs/capabilities/census-ledger.md` returns at least 1 line; `yq --front-matter=extract .depends` includes `cap:source-index` for census-ledger, `cap:census-rule-api` for census-cli, and `cap:census-patterns`, `cap:census-debt-ratchet`, `cap:census-reference-scan` for census-rule-api; the census-normal-form edge is added only if `grep -n 'raf-source\|source index' docs/capabilities/census-normal-form.md` is non-empty, and the review records which."

### F13 — R1's G12 check misses the third statement of the rule

- Line 59 greps two files, but G12 cites `workplan-document-model.md:103` as the first statement. Rewrite: "`grep -n 'one code path' docs/architecture/okf-profile.md docs/capabilities/workplan-complete-gate.md docs/capabilities/workplan-document-model.md` returns 3 lines with identical wording."

### F14 — Findings dropped without a disposition (each needs an "addressed" or "declined because")

- S-L1: `enable-cache: false` on `setup-uv` (security review line 206). Cheap; the sound-section reasoning ("no privileged consumer today") is a valid decline if written down.
- historical-fit §3: two near-miss positive fixtures (`issue-reference`, `the issue` without a number) so the new `issue 12` and `GH-12` forms do not refuse valid prose. Cheap and directly tied to R6's new patterns; add to R6.
- historical-fit F7 (second half): `CITATION.cff` lacks `date-released`, `url` and a person author. Acceptable to decline for an unreleased version; say so.
- Security "sound" note: CONTRIBUTING could state that `core.hooksPath=.githooks` means committing on a checked-out branch runs that branch's hook. One sentence; fits R6's Docs bullet.

### F15 — Unresolved "or" choices left to the implementer

- Line 41 (define "criterion" or drop it), line 76 ("if unit discovery uses it"), line 84 (G13b: name the input separately or state the default does not apply; Stage 1 already fixed `migrate --ledger old.toml`), line 157 (allow both SPDX shapes or split the template). Each is small, but a plan that chooses now avoids a review round. Suggested choices: drop "criterion" (the record's `items` already carry the claim); G13b: keep `--ledger` for `migrate` and state that the common default does not apply; G15c: split the template comment into two lines (one outlier changes, the rule stays "exactly as the existing documents do").

### F16 — R10 sequencing sentence is a non sequitur

- Line 270: "R10's issue is filed before R2 lands, so census-cli can describe the decision without an issue number." The doc never carries a number either way. Rewrite: "R10's issue is filed before R2 lands so its body can be cross-checked against the Gate-owns text, and so the final summary can name it."

### F17 — Validation budget: the conflict is stated correctly; two conditions are missing

- Line 262 correctly says a second full `make check` is required because R6–R8 change the gates themselves, and that the Stage 1 run becomes a historical snapshot. Add: "the run uses the same `CARGO_TARGET_DIR`; it starts only after the R9 push, so no hook `cargo test` competes for the target lock; and any fix commit after it invalidates it, which means another run (recorded as such), not a reuse argument."

### F18 — R9's check needs commits in a scratch clone

- Line 241 requires a two-parent merge in a scratch clone. The quality review could not commit in scratch copies because the machine signing hook refuses unsigned commits. State that the scratch clone commits with the normal signing config (as the completion audit's hook cases did), so the check is runnable without bypassing anything.

## The four concerns

- **Completeness.** Every gap, finding and advisory from the five input reports maps to a group or an evidenced adjudication, with the sub-item drops in F7 and F14. The three user decisions are implemented without narrowing: decision 1 is fully resolved by (1) and the new issue is additive; decision 2 narrows the trust model but keeps the hardening and adds `--no-deps` where resolution is unneeded; decision 3 is R6. No "future work" sections; R10's owner actions are external by nature and recorded as such.
- **Constraints.** No ADRs or constitution. Repo rules: AGENTS.md §5 "Rust first" and "every rule has a negative fixture" are what R6 and R7 restore. AGENTS.md §7 (no tracker numbers or process wording) holds: none of the planned doc text carries a number, and `RAF_GATE_ROOT` is a neutral name. CLAUDE.md "never bypass a hook" is the reason F3(3) is blocking. The no-deferral rule is honoured. F1 is a constraint point: A6/K3 is extended by the plan without saying so.
- **Design.** The shared pattern module, index-derived file list and runtime root variable are the right single-source design, with the three precision gaps in F3 and F10. The CI `gates` job (F2) is the one shipped entry point the plan forgot. The pinned inventory table (F11) is maintenance cost with a visible payoff. The hook now compiles Rust per commit; with a warm target dir this is seconds, and F3 asks for the number to be recorded rather than assumed.
- **Utility.** Two upgrades with clear payoff and small cost, both inside the plan's scope: (a) one index-derived file list for both tests (F10) so hook, local and CI judge identical inputs; (b) deleting the CI `gates` job rather than giving it a toolchain (F2), which removes a job that only duplicated `test`. No scope expansion is proposed.

## Verdict (pass 1)

Findings F1–F5 must be fixed in the plan before execution: F1's check fails on the correct fix, F2 and F3 break shipped gates or the commit path mid-sequence, F4 verifies the wrong property, and F5 trades a loud false positive for a silent pass. F6–F18 are concrete improvements to resolvedness and falsifiability; F7 and F14 need written dispositions.

Pass 1 verdict: NEEDS-REVISION (superseded by the Recheck below).

## Recheck (pass 2)

- Re-read the revised plan in full (302 lines, same path). Rechecked only the changed text and the eighteen prior findings, per the review protocol. Also re-verified one repo fact the F10 change depends on: `tests/docs_contract.rs` fixtures build `Repo` in memory with `repo.insert(...)` (lines 1275–1297), so removing the disk walker in favour of an index-derived file list does not break the 45 fixture tests.
- Line numbers below refer to the revised plan.

| Finding | Status | Evidence in the revised plan |
|---|---|---|
| F1 | DISCHARGED | Line 147 states the A6/K3 extension to token-level deletion. Line 153 is the word-level `git diff --no-index --word-diff=porcelain` check with `+` lines allowed only for the ® SPDX line and link targets. Line 154 adds `grep -c '第三方' … >= 1`. Line 150 adds the complete-sentence requirement on the backtranslation with the deletion-only remedy. |
| F2 | DISCHARGED | Line 193: the CI `gates` job is deleted in the R6 commit; `cargo test --workspace --locked` in `test` runs `repo_gates`. Line 192: `licenses`/`issue-refs` stay as thin filter targets, header says they are a subset of `make test`. Checks at lines 205–207: `actionlint`, `yq '.jobs \| keys'` no longer lists `gates`, `make -n check` still lists every issue-required step. |
| F3 | DISCHARGED | (1) Line 189: `git -C "$CARGO_MANIFEST_DIR" ls-files -z`, each path read under `RAF_GATE_ROOT`. (2) Line 191: binaries compiled from the working tree, stated in the hook header. (3) Line 196: `CARGO_TARGET_DIR` exported for every commit from R6 on, with a `git hook run pre-commit` pre-check. Line 204: warm wall time recorded in `validation.md`. Line 188: `tests/support/mod.rs` via `mod support;`, every item used by both binaries or a scoped `#[allow(dead_code, reason = "...")]`. |
| F4 | DISCHARGED | Line 240: pin the `master` head `e2a55d2ffb…`, comment `# master, 2026-10-09`, verified by `git ls-remote … refs/heads/master` equality and `gh api …/compare/83d06109b7...e2a55d2ffb --jq .status` printing `ahead` or `identical`; if master has moved, pin the current head after the same compare. Line 248 records both outputs in the review. |
| F5 | STILL-OPEN (one token) | Line 181 spares a 3/4/6/8-hex token "immediately preceded by `"`, `'`, `:` or `(`", while line 183 requires `(#1234)` to fail. Those two lines contradict: `(#1234)` is 4 hex digits preceded by `(`, so the rule as written spares it and the fixture as written refuses it. The `(` context came from my own pass-1 suggestion and is wrong: no colour syntax places a hex colour directly after `(`, while `(#12)` is the commonest prose shape of an issue token. Rewrite line 181 to: "…spared only when it is immediately preceded by `"`, `'` or `:`, or when it contains a letter a–f (G9)." Lines 182–183 and the check at line 203 then stand as written, including the `(#1234)` refusal. |
| F6 | DISCHARGED | Line 68 names `Gate::write_debt(&self) -> Result<DebtWrite, Error>`, its refusal, its caller (binary or xtask, never a `#[test]`), and where each doc states it. Check at line 92 greps `write_debt` in three docs. |
| F7 | DISCHARGED | Line 113 adds the directory-symlink loop and FIFO fixtures; line 114 drops the `MemoryTree` equivalence claim for these cases. Lines 121–122 add binding validation against the source index and the `-`-prefixed Verification bullets. Checks at lines 134–135 grep for ` -- ` in three specs and for `fifo\|loop` in source-index. |
| F8 | DISCHARGED | Line 133 greps all three parser specs, at least one line per file. |
| F9 | DISCHARGED | Line 297: order is R1 → R2 → R3 → R4 → R6 → R7 → R8 → R9 → R5, with R5 last so the CHANGELOG describes only what is in the tree. Line 164 lists the added topics including the Rust gates, hardened rules, CI pins and cliff dedup; line 169 names a grep per topic. |
| F10 | DISCHARGED | Line 230: count equals 46 plus the number of new fixtures, each named in `tasks/G-R7-review.md`. Line 231: probes 7, 9, 18, 19, 22, 24, 25, 26, 29, 31, 32 and 34. Line 190: `docs_contract.rs` uses the shared root, patterns and index-derived file list; walker removed (Q-L6 second half). In-memory fixtures confirmed unaffected (see above). |
| F11 | DISCHARGED | Line 214: the inventory table's doc comment states the update rule and the `raf_workplan::Gate` replacement. Line 195: AGENTS.md §6 names the inventory table. |
| F12 | DISCHARGED | Lines 91, 95, 96 give concrete greps and the enumerated `yq` edges; line 76 makes the census-normal-form edge conditional on a stated grep with the result recorded. |
| F13 | DISCHARGED | Line 59 greps all three files and requires identical wording. |
| F14 | DISCHARGED | `enable-cache: false` at line 239; near-miss positive fixtures `issue-reference` and `the issue` at line 182; hooksPath note in CONTRIBUTING at line 195; CITATION extra fields declined at line 279 with a reason (no true `date-released` yet; organisation author by design), version-agreement half fixed in R6. |
| F15 | DISCHARGED | Line 41 drops "criterion"; line 76 states the rule for the normal-form edge; line 82 keeps `migrate --ledger <old.toml>` and states the default does not apply; line 165 splits the PR template comment. |
| F16 | DISCHARGED | Line 300 gives the cross-check reason instead of the non sequitur. The issue number in this stage file is not a repo file, so the ban is not touched. |
| F17 | DISCHARGED | Lines 290–292: same `CARGO_TARGET_DIR`, starts only after the last push, any later fix commit means another recorded run. |
| F18 | DISCHARGED | Line 262: scratch-clone commits use the normal signing configuration with no bypass. |

Changed text checked for new defects: the deletion of the `gates` job (line 193) is in the same commit as the Makefile change, so CI stays coherent at every commit; the `make licenses issue-refs` check (line 201) still exercises the thin targets the hook does not use; the environment rule at line 196 covers the R6 commit itself, which is the first one gated by the new hook. No new finding.

Seventeen of eighteen findings are discharged with line-level evidence. The one open item is a single-token contradiction between a rule and its fixture (F5, line 181), with the exact rewrite given above. It is not a design question and needs no user decision, but a plan whose acceptance fixture contradicts its own rule cannot be handed to an implementer as written, so the verdict stays short of PASS until that token is removed.

Pass 2 verdict: NEEDS-REVISION (superseded by the Recheck below).

## Recheck (pass 3)

- Scope: line 181 of the revised plan only, as requested. Re-read lines 178–185.
- Line 181 now reads: "A 3-, 4-, 6- or 8-hex-digit token is spared only when it is immediately preceded by `"`, `'` or `:`, or when it contains a letter a–f (G9)." The `(` context is gone.
- Consistency with the fixtures on lines 182–183, applied by hand against the stated rule (`#[0-9]{1,5}\b` with a prefix that is not `&`, a word character or `#`):
  - `fill="#000"`: preceded by `"`, spared. Pass, as listed.
  - `stroke='#abc'`: preceded by `'` and contains letters, spared. Pass, as listed.
  - `color: #1f2`: contains `f`, spared. Pass, as listed.
  - `&#38;`: prefix `&` is excluded by the base rule. Pass, as listed.
  - `#123456`: six digits; `#[0-9]{1,5}\b` cannot match because the fifth digit is followed by a word character. Pass, as listed.
  - `see #123`: preceded by a space, all digits, no sparing context. Fail, as listed.
  - `(#1234)`: preceded by `(`, which is no longer a sparing context; all digits. Fail, as listed.
  - `Per #12`: fail, as listed.
- Rule and fixtures now agree; the check at line 203 ("the listed passing and failing `#` fixtures behave as stated, including the refusals of `see #123` and `(#1234)`") is falsifiable in both directions.

| Finding | Status | Evidence |
|---|---|---|
| F5 | DISCHARGED | Line 181 spares only `"`, `'`, `:` or a letter a–f; every fixture on lines 182–183 is consistent with it, and `(#1234)` is refused as required. |

No finding remains open. F1–F4 and F6–F18 were discharged in pass 2 and are unchanged.

VERDICT: PASS
