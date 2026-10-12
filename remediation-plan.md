# Remediation plan — issue 1 / PR 11 (Stage 2, full path)

Reviewed head d6b6572, base origin/main @ 720262d. Stage 1 `plan.md` is unchanged.

Inputs:
- `gap-analysis.md` (G1–G16);
- `reviews/security-review.md` (S-M1, S-M2, S-L1–L5);
- `reviews/quality-review.md` (Q-H1, Q-M1–M5, Q-L1–L8);
- `historical-fit.md` (F1–F9, D1–D3; adjudicated in gap-analysis §Adjudications);
- `reviews/completion-audit-stage2.md` (open: CI; advisories A1–A6).

There is no external reviewer feedback: PR 11 has 0 reviews, 0 inline comments and 0 threads.

## User decisions (this stage, verbatim answers)

1. **Census CLI versus Gate (G3).** The question offered (1) "Gate owns it" and (2) "CLI runs repo rules". The answer was **"(1) but add an issue for (2)"**. Consequences:
   - The specs state the Gate-owns contract.
   - A new issue is filed for the CLI-runs-repo-rules mode, as a separate enhancement.
   - That issue does not cut scope here, because (1) fully resolves G3.
2. **Trust model (S-M2).** The answer was **"Workspace = build input (Recommended)"**:
   - RAF executes the workspace's cargo configuration, exactly as `cargo build` or `cargo test` would.
   - Parsing stays hardened.
   - The specs pin `--frozen`, no-shell spawning, and `--` before arguments taken from documents.
3. **Shell gates (G7).** The answer was **"Port gates to Rust (Recommended)"**.

Earlier owner decisions still bind: MSRV 1.98.1; `.goals`, `.baseline` and `.stage/` are never committed; census code may be ported from any copy without notice.

## Fix groups (one coherent commit each, pushed after review)

Each group lists its gaps, the intended fix, and the executable acceptance check. Every group also passes the pre-commit hook, `cargo test --test docs_contract --locked` and `make issue-refs` (after R6, the Rust gate), plus `uvx reuse@6.2.0 lint`.

### R1 — docs(workplan): migrate the lost essay content and tighten the workplan contracts

**Gaps:** G1, G4(a), G5 (workplan edges), G6 (records), G8 (text), G12, A4.

**Fix:**
- **G1.** Add a "Section | Holds" table to `docs/architecture/okf-profile.md` with the essay's seven definitions in current tense, and link it from `workplan-document-model.md` Contract. Add the pinned Grounding Contract URL (essay :211, commit a631f73e…) to the Explanation of `workplan-document-model.md` and `workplan-two-way-coverage.md`. Add "an accurate capability explanation" to Scenario B in `workplan-complete-gate.md`.
- **G4, option (a), which matches essay §4/§6.**
  - Add a reconcile obligation kind `example-failing`, raised when a bound example or doctest fails to build or run, with its resolution.
  - Complete-gate condition 5 becomes "bound tests and examples pass", with a refusal example.
  - Drop the word "criterion": the change record's `items` already carry the claim.
- **G5.**
  - `workplan-prose-assessment` depends on `cap:workplan-evidence-claims`.
  - Remove `raf_source::Tree` from the capability `okf-profile.md`, so `raf-okf` stays a root.
  - Add the sentence "`depends` lists the capabilities whose contract this one relies on" to `docs/architecture/okf-profile.md`.
- **G6.** Add `cap:<slug>#constraints` to `affects` in all five M2 change records. The enforcing rule lands in R7.
- **G8.** Qualify okf-profile's "enforces the profile today": binding *resolution* waits for the source index. The empty-bindings-before-delivery rule is enforced, in R7.
- **G12.** Add "delivered binds at least 1 code path and 1 test" to the delivered row of the normative profile table. Make the complete-gate wording identical, and reword the "none inside one" sentence.
- **A4.** Fix or remove the "memory bound" claim at `workplan-reconcile.md:89`.

**Check:**
- `git grep -n -iE 'callers can rely|prerequisites|substantiate' -- docs/architecture/okf-profile.md` returns at least 3 lines.
- `git grep -n GROUNDING.md -- docs` returns at least 2 lines.
- The Scenario B grep returns 1 line.
- `grep -n example-failing docs/capabilities/workplan-reconcile.md` returns at least 1 line.
- The fence-aware PROPOSED-sections-versus-affects comparison gives equal sets for all 5 records.
- `yq --front-matter=extract .depends docs/capabilities/workplan-prose-assessment.md` includes `cap:workplan-evidence-claims`.
- `git grep -n 'raf_source::Tree' -- docs/capabilities/okf-profile.md` is empty.
- `grep -n 'one code path' docs/architecture/okf-profile.md docs/capabilities/workplan-complete-gate.md docs/capabilities/workplan-document-model.md` returns 3 lines with identical wording.
- The docs_contract test passes (the graph stays acyclic).

### R2 — docs(census): Gate-owned census path and contract clarifications

**Gaps:** G3 (decision 1), G5 (census edges), G13a–g, G15a.

**Fix:**
- **G3.**
  - `Gate::write_debt(&self) -> Result<DebtWrite, Error>` writes `census-debt.json` beside the ledger. It refuses while any non-debt finding exists, the same refusal as the CLI path. It is called from the repository's own binary or xtask, never from a `#[test]`, because tests do not write tracked files. `census-debt-ratchet` Usage shows the xtask invocation. `census-cli` Contract names it as the only write path for ledgers that need registered code.
  - `cargo census check` handles ledgers that need no registered code. For a non-empty `[job.ext]` or an unregistered `rule:` id it exits 2 with a message naming the Gate path.
  - `census-cli` no longer claims to be "the same function"; it is the same function for ledgers that need no registered code, and refuses otherwise.
  - An unknown `rule:` id (not a built-in detector, a pattern or a registered rule) is a hard error in both the CLI and the Gate, listed among the ledger validation errors.
  - `census-migration` Usage sends dialect ledgers to the repository's census test.
  - Each new rule gets a Verification bullet.
  - Docs touched: census-cli, census-ledger, census-rule-api, census-debt-ratchet, census-migration.
- **G5.**
  - `census-ledger` depends on `cap:source-index`. `census-normal-form` gains the edge only if `grep -n 'raf-source\|source index' docs/capabilities/census-normal-form.md` is non-empty; the review records which.
  - `census-cli` depends on `cap:census-rule-api`.
  - `census-rule-api` depends on patterns, debt-ratchet and reference-scan.
  - The graph stays acyclic.
- **G13.**
  - a: use `--frozen` rather than `--locked`, plus a Verification bullet saying an undownloaded registry package fails with no network access.
  - b: keep `migrate --ledger <old.toml>` (fixed in Stage 1), and state that the common `--ledger` default does not apply to `migrate`.
  - c: name the shipping-only detectors in the table, with a Verification bullet.
  - d: an unreferenced pattern *fails* staleness in all three places.
  - e: "every key except `[job.ext]`".
  - f: refuse an empty positive fixture set.
  - g: state the `normal_form` bump path, and that review of `census-debt.json` creation diffs is the control against re-baselining.
- **G15a.** `census-rule-api.md:80` uses `raf-census = "0.1"`.

**Check:**
- `grep -n -E 'job\.ext' docs/capabilities/census-cli.md` and `grep -n -E 'exit(s)? 2' docs/capabilities/census-cli.md` each return at least 1 line.
- `grep -n write_debt docs/capabilities/census-rule-api.md docs/capabilities/census-debt-ratchet.md docs/capabilities/census-cli.md` returns at least 3 lines.
- `git grep -n 'same function' -- docs/capabilities/census-cli.md` gives the qualified sentence only.
- census-migration Usage names the census test or xtask for dialect ledgers.
- `grep -n -E 'unknown .*rule:|rule: .* unknown' docs/capabilities/census-ledger.md` returns at least 1 line.
- `yq --front-matter=extract .depends` includes: `cap:source-index` for census-ledger; `cap:census-rule-api` for census-cli; `cap:census-patterns`, `cap:census-debt-ratchet` and `cap:census-reference-scan` for census-rule-api.
- `git grep -n -- '--locked' docs/capabilities/census-reference-scan.md` prints no fetching claim.
- `grep -n '"1"' docs/capabilities/census-rule-api.md` is empty.
- docs_contract passes.

### R3 — docs(security): trust model, path containment and invocation rules

**Gaps:** S-M1, S-M2 (decision 2), S-L4, S-L3 (the publishable part).

**Fix:**
- **SECURITY.md.**
  - Rewrite the trust statement: the checked workspace is trusted build input. RAF runs as a dev-dependency or cargo subcommand inside it, and executes its cargo configuration exactly as `cargo build` or `cargo test` would.
  - Parsing of Rust, TOML and YAML stays hardened: a hostile file yields an error, never a panic or crash, an escape from the root, or unbounded resource use.
- **S-M1 path containment.** State the rule in `source-index.md`, `census-reference-scan.md` and `docs/architecture/okf-profile.md` (doc links):
  - canonicalise each resolved path and require it inside the canonical root;
  - refuse symlinks that resolve outside the root, and non-regular files;
  - bound explicit `Cargo.toml` target paths the same way;
  - add disk fixtures to Verification: a symlinked mod file, `dir-symlink/..`, a target path outside the root, a link to a symlink outside the root, a directory-symlink loop, and a FIFO;
  - state that `MemoryTree` equivalence does not cover these cases (`source-index.md:133-134`).
- **S-M2 invocation rules.** Every spawned `cargo` or `git` call:
  - runs with no shell;
  - takes its arguments as an argv vector, with `--` before any path or filter taken from a document;
  - uses `--frozen` for metadata;
  - runs `cargo metadata --no-deps` wherever dependency resolution is not needed, e.g. root discovery in `workplan-cli`.
  - The exact invocations are listed in each spec that spawns.
  - complete-gate runs only bindings that resolve in the source index.
  - entity-history and complete-gate each carry a Verification bullet with a `-`-prefixed path or binding.
- **S-L4.** The specs state parser resource bounds: maximum file size, maximum nesting depth (recursion refused before the stack overflows), and YAML alias expansion refused or bounded. Each bound has a Verification bullet.
- **S-L3.**
  - Publish the PGP key fingerprint in SECURITY.md.
  - Note the key's user ID versus the contact address.
  - Key renewal before 2026-12-24 is an **owner action outside the repo**, recorded in the final summary.

**Check:**
- `grep -n -i 'build input' SECURITY.md` returns at least 1 line.
- `grep -n -iE 'canonical' docs/capabilities/source-index.md docs/capabilities/census-reference-scan.md docs/architecture/okf-profile.md` returns at least 3 lines.
- `git grep -n -- '--frozen' -- docs/capabilities` returns at least 1 line, and `git grep -n -- '--no-deps' -- docs/capabilities/workplan-cli.md` returns at least 1 line.
- `grep -n -iE 'nesting|depth|maximum file size|alias' docs/capabilities/source-index.md docs/capabilities/okf-profile.md docs/capabilities/census-ledger.md` returns at least 1 line per file (F8).
- `git grep -n -e ' -- ' -- docs/capabilities/workplan-entity-history.md docs/capabilities/workplan-complete-gate.md docs/capabilities/census-reference-scan.md` returns at least 3 lines.
- `grep -n -iE 'fifo|loop' docs/capabilities/source-index.md` returns at least 2 lines.
- The fingerprint in SECURITY.md equals `gpg --show-keys` output on the embedded block, using a throwaway keyring.
- docs_contract passes.

### R4 — docs(licensing): zh-Hans and LICENSING consistency, with a fresh backtranslation

**Gaps:** G14a, b, d, e; A3; G14c.

**Fix:**
- **a.** Delete the purrdf-only "来源记录" phrase.
- **b.** Delete the distribution-package byte-identity sentence. English makes no such claim and no crate is published.
- **A3.** Restore the third-party-licence statement by deleting the purrdf-only clause "Unicode 数据和" from purrdf's original sentence (purrdf line 22).
- **Rule extension.** A3 and G14a delete text inside a sentence. This extends A6/K3 from whole-sentence deletion to token-level deletion: no new Chinese character is written, and every change is a removal.
- **d.** Fix the ambiguity count.
- **e.** `LICENSING.md` Contributions states that documentation contributions are CC-BY-4.0 and qualifies the code-licence sentence as code.
- **c.** A context-isolated agent that sees only the new Chinese file writes a fresh backtranslation. The record is updated with real timestamps, byte count and SHA-256. The backtranslation must render the shortened sentences as complete sentences; a fragment the translator reports is a defect in the deletion, fixed by a further deletion only.

**Check:**
- `git diff --no-index --word-diff=porcelain <(sed 's/PurRDF/RAF/g; s/purrdf/raf/g' ~/Active/purrdf/docs/LICENSING.zh-Hans.md) docs/LICENSING.zh-Hans.md` prints `+` lines only for the SPDX ® line and link targets; every other change is a `-` token.
- `grep -c '第三方' docs/LICENSING.zh-Hans.md` is at least 1.
- `grep -n '来源记录' docs/LICENSING.zh-Hans.md` is empty.
- The licence gate passes, comparing the new record SHA with `sha256sum`.
- `grep -n 'CC-BY' LICENSING.md` shows the Contributions sentence.

### R5 — docs: project-document consistency

**Gaps:** G15b (A1), G15c, G15d.

**Fix:**
- CHANGELOG `[Unreleased]` gains bullets for the brand, logo and README; the capability specs and architecture pages; the document-contract test and the Rust repository gates; the hardened contract rules; the CI pins; the cliff dedup; and the essay retirement.
- Split `.github/PULL_REQUEST_TEMPLATE.md`'s SPDX comment into the two-line shape, so the AGENTS.md rule ("exactly as the existing documents do") holds.
- AGENTS.md §6 names the document-contract test as today's enforcement.

**Check:**
- Each of these returns at least 1 line under `[Unreleased]`: `grep -n -iE 'brand|logo|README' CHANGELOG.md`, `grep -n -i 'capabilit' CHANGELOG.md`, `grep -n -iE 'docs_contract|document.contract' CHANGELOG.md`, `grep -n -i 'repo_gates\|repository gate' CHANGELOG.md`, `grep -n -i 'essay' CHANGELOG.md`.
- Every Markdown file's SPDX header matches the AGENTS.md rule (one awk check).
- `grep -n docs_contract AGENTS.md` returns at least 2 lines.

### R6 — test: port the licence and issue-reference gates to Rust; one pattern set

**Gaps:** G7 (decision 3), G9, Q-L1 (prose forms), G16-F7 (version agreement), Q-M5 (the hook runs the document contract).

**Fix:**
- **New file `tests/repo_gates.rs`.** It is std-only and lives in the root package. The patterns live in a shared module under `tests/` (e.g. `tests/support/patterns.rs`, included by both test files), which is the single definition site.
  1. The Mulan digests of `LICENSE-MULAN` and `LICENSES/MulanPSL-2.0.txt`, and the zh-Hans fingerprint against the backtranslation record.
  2. Issue/PR tokens and process wording over **all tracked files** (`git ls-files -z`).
     - `#[0-9]{1,5}\b` requires a prefix that is not `&`, a word character or `#`. A 3-, 4-, 6- or 8-hex-digit token is spared only when it is immediately preceded by `"`, `'` or `:`, or when it contains a letter a–f (G9).
     - Fixtures that pass: `fill="#000"`, `stroke='#abc'`, `color: #1f2`, `&#38;`, `#123456`, plus the near-miss prose `issue-reference` and `the issue`.
     - Fixtures that fail: `see #123`, `(#1234)` and `Per #12`.
     - Add the forms `issue 12`, `GH-12` and `pull request 11`.
  3. The `CITATION.cff` version equals `[workspace.package].version`.

  Every rule has a negative fixture.
- **Shared support.** `tests/support/mod.rs` holds the patterns and the file list, and both binaries use it via `mod support;`. Every item is used by both binaries, or carries a scoped `#[allow(dead_code, reason = "...")]`, so `clippy --all-targets -D warnings` stays green.
  - The file list is `git -C "$CARGO_MANIFEST_DIR" ls-files -z`, with each path read under `RAF_GATE_ROOT` (default `CARGO_MANIFEST_DIR`). The hook therefore judges the index snapshot.
  - `docs_contract.rs` uses the same root, patterns and index-derived file list. Its walker and duplicate pattern set are removed (Q-L6, second half: untracked scratch files no longer count).
- **Build location.** The test binaries are compiled from the working tree, not the index. The hook header states this; building inside the snapshot would mean a cold build per commit.
- **Makefile.** `licenses` and `issue-refs` become thin `cargo test --locked --test repo_gates <filter>` targets for the hook and for humans. The shell recipes are deleted. The header says these targets are a subset of `make test`.
- **CI (F2).** The `gates` job is deleted in the same commit: `cargo test --workspace --locked` in the `test` job runs `repo_gates`.
- **Hook.** `.githooks/pre-commit` runs `cargo test --locked --test repo_gates --test docs_contract` with `RAF_GATE_ROOT=$snapshot`, plus fmt and reuse.
- **Docs.** The AGENTS.md and CONTRIBUTING.md gate text matches the tree. AGENTS.md §6 names the capability inventory table (R7). CONTRIBUTING notes that with `core.hooksPath=.githooks`, committing on a checked-out branch runs that branch's hook.
- **Environment.** Every commit from R6 on is made with `CARGO_TARGET_DIR=/home/paudley/Active/.raf-stage2-target` exported, because `/opt` is full. Before the first R6 commit, verify that `git hook run pre-commit` passes.

**Check:**
- `cargo test --test repo_gates --locked` passes, with a non-zero count of named negative fixtures.
- `grep -c 'this\[ \]issue' Makefile tests/*.rs tests/support/*.rs` shows a single definition site.
- `make licenses issue-refs` rc 0.
- In a scratch clone, these are refused by the hook: a staged `#12` in `src/lib.rs`, an altered `LICENSE-MULAN`, a staged capability missing `## Usage`, and a CITATION version bump without a Cargo bump.
- The listed passing and failing `#` fixtures behave as stated, including the refusals of `see #123` and `(#1234)`.
- `git hook run pre-commit` on the real worktree passes, with its warm wall time recorded in `validation.md`.
- `actionlint` rc 0.
- `yq '.jobs | keys' .github/workflows/ci.yaml` no longer lists `gates`.
- `make -n check` still lists every issue-required step.

### R7 — test(docs): harden the document contract

**Gaps:** Q-H1, Q-M1–M4, Q-L2, Q-L3, Q-L5, Q-L6, Q-L7, plus the rules for G6, G8 and G12.

**Fix:**
- **Q-H1.** A single table, the capability inventory, of the 21 required capability slugs with their state family (16 designed, 5 considered), the 2 architecture ids and the 21 change records. Assert each exists on the real tree with the matching `id` and `state`. Make the `>= 1` guards unconditional. Add a negative fixture: a missing slug. The table's doc comment says: this table is the capability inventory; changing a state or adding a capability edits this table in the same commit; `raf_workplan::Gate` replaces it.
- **Q-M1.** Reject placeholder tokens (TBD, TODO, FIXME, XXX, "to be written") outside fences, and sections that are empty after stripping HTML comments.
- **Q-M2.** The frontmatter scalar parser rejects an unterminated quote, an unquoted `: ` and a leading YAML indicator character, and a scalar equal to a block indicator.
- **Q-L5.** The parser strips ` # comment` from unquoted scalars, so valid YAML is accepted.
- **Q-M3.** PROPOSED detection is case-insensitive and allows any spacing after `>`. A near-miss is reported as malformed.
- **Q-M4.** Check reference-style definitions and `href`/`src` attributes across all tracked `*.md` (README logo, `.github` templates). The profile states that links are single-line.
- **Q-L2.** `moved:change:<slug>` resolves to an existing change.
- **Q-L3.** A `designed` doc carries no PROPOSED block.
- **Q-L6.** A typed doc outside its directory is a violation.
- **Q-L7.** A negative fixture for each implemented rule that lacks one: duplicate sections, generated-marker errors, bad id prefixes, `items` shape errors and the rest named in the quality review.
- **Rules for earlier fixes:**
  - G6: every section that holds a PROPOSED block naming change X appears in X's `affects`.
  - G8: `designed` and `considered` capabilities carry empty binding lists.
  - G12: `delivered` binds at least 1 code path and at least 1 test.

**Check:**
- `cargo test --test docs_contract --locked` passes on the real tree. The count equals 46 plus the number of new fixtures, and each fixture is listed by name in `tasks/G-R7-review.md`.
- Re-run quality-review mutation probes 7, 9, 18, 19, 22, 24, 25, 26, 29, 31 (non-empty `designed` bindings), 32 and 34 (`.github` template broken link) in a scratch copy. Each now fails (rc ≠ 0) with the named rule.

### R8 — ci: pin tool versions, harden the hook path

**Gaps:** G11 / S-L1, S-L2, G16 (F4, F5, F9), S-L5.

**Fix:**
- Install `cargo install --locked cargo-deny@0.20.2`, with the version recorded beside the Makefile `deny` comment.
- Pin the `setup-uv` uv version, and set `enable-cache: false` (S-L1).
- Repin `dtolnay/rust-toolchain` to the `master` head `e2a55d2ffb04f378e9626c28d38b36d230d1e12f`. It contains `83d0610` "Safely handle action inputs", and master is never rebuilt. The comment reads `# master, 2026-10-09`. Verify: `git ls-remote https://github.com/dtolnay/rust-toolchain refs/heads/master` prints that SHA, and `gh api repos/dtolnay/rust-toolchain/compare/83d06109b7...e2a55d2ffb --jq .status` prints `ahead` or `identical`. If master has moved, pin the current head after the same compare check.
- Add a CI `concurrency` group that cancels superseded PR runs.
- Add `.githooks/pre-merge-commit`, which execs `pre-commit`.
- Add `.github/dependabot.yml` for `github-actions`.
- Quote `$(ROOT)` in Makefile recipes.

**Check:**
- `actionlint` rc 0, and `zizmor` reports no new findings.
- The `ls-remote` and `compare` outputs above are recorded in the review.
- `grep -n 'cargo-deny@0.20.2' .github/workflows/ci.yaml`.
- `grep -n concurrency .github/workflows/ci.yaml`.
- `test -x .githooks/pre-merge-commit`.
- `yq . .github/dependabot.yml` rc 0.
- `make fmt ROOT="/tmp/a b"` against a copy at a path with a space: rc 0.
- reuse lint is clean for the new files.

### R9 — build(cliff): port purrdf's per-release dedup

**Gap:** G10.

**Fix:** port purrdf's `rendered_entries`/`breaking_keys` body and the `\(\s*\)` preprocessor (purrdf 446c23318, 38b0a1da9).

**Check:** in a scratch clone, committing with the normal signing configuration and no bypass, with a two-parent merge whose branch commit repeats the squash subject, `git cliff --unreleased` prints the bullet once. It is marked BREAKING if either commit has `!`.

### R10 — publication items (no commit)

- **New issue (user decision 1).** "Run repository-registered census rules from `cargo census`". Filed with `gh issue create`, since stagectl has no issue-create wrapper (automation gap). It holds the option (2) design and links `cap:census-cli`. The body contains no attribution text.
- **G2 / AC1, the CI half.** This is an external blocker: the org billing block. Re-run after every push and whenever billing is cleared. Until all 8 jobs pass, CI stays **unverified**. Merge requires green CI or an explicit owner waiver; Stage 3 decides.
- **A6.** After merge, the owner uploads the social preview and sets the repository description (text in `docs/BRAND.md`). These are owner actions, recorded in the final summary.
- **S-L3.** The PGP key renewal before 2026-12-24 is an owner action.

## Evidenced false positives / not gaps

- **Historical-fit F6, F8 and D3.** Not gaps, per the gap-analysis adjudication table, which I accept.
- **Q-L1: 6-digit `#123456`.** This form cannot be told apart from a hex colour; `docs/*.svg` carries 6-digit colours. It stays unmatched by design, with a fixture asserting that colours pass.
- **Q-L4: bound symbols that do not exist.** Not enforceable until the source index exists, and the profile text now says so (G8). The part checkable today, empty bindings before delivery, is enforced in R7.
- **Q-L8, the MSRV question.** 1.98.1 is the owner decision ("rust 1.98.1 is fine"). It is not a gap.
- **A5.** The `.goals`/`.baseline` decision is written into the issue body itself (`issue.md` line 20). It is not a gap.

- **historical-fit F7, second half.** `CITATION.cff` lacks `date-released`, `url` and a person author. Declined while the version is unreleased: `date-released` has no true value yet, and the organisation is the author by design. The version-agreement half is fixed in R6.

## Plan review dispositions (`reviews/remediation-plan-review.md`, pass 1: NEEDS-REVISION)
- F1–F13 and F15–F18: addressed in place in R1–R10, the order, and the validation budget.
- F14: `enable-cache: false` added to R8; near-miss positive fixtures added to R6; the hooksPath note added to R6 Docs; CITATION extra fields declined (above).

## Validation budget

- The workflow's single full `make check` ran in Stage 1 on 9f93e5d.
- R6 to R8 change the Makefile, the hook, the CI workflow and the test suite, so that run no longer qualifies the final tree.
- **Conflict with the single-run budget:** one further full `make check` is required on the settled head. I will run it once, in the background, and record it as the workflow's final qualification. The Stage 1 run stays a historical snapshot.
  - It uses the same `CARGO_TARGET_DIR`.
  - It starts only after the last push, so no hook `cargo test` competes for the target lock.
  - Any fix commit after it invalidates it. That means another recorded run, not a reuse argument.
- Builds use `CARGO_TARGET_DIR=/home/paudley/Active/.raf-stage2-target`, because `/opt` is full.

## Order and publication

- R1 → R2 → R3 → R4 → R6 → R7 → R8 → R9 → R5, all sequential in one worktree. R5 runs last (F9), so the CHANGELOG describes only what is in the tree.
- Each group: a fresh sonnet implementer, then a sonnet reviewer that writes `tasks/G-RN-review.md` with `VERDICT: PASS | BLOCKED`, then commit and push.
- After each push, a gap/fix/checks/commit comment goes to issue 1 and PR 11.
- R10's issue (filed: issue 12) exists before R2 lands, so its body can be cross-checked against the Gate-owns text and the final summary can name it.
- A fresh completion auditor serves as the exit gate.
